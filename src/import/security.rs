use crate::pipeline::block::*;
use crate::pipeline::*;

use super::helpers::*;
use super::SecurityIssue;
use super::SecuritySeverity;

// ────────────────────────────────────────────────────────────
// Config security scanner
// ────────────────────────────────────────────────────────────

/// Known data exfiltration domains and URL patterns
const EXFIL_PATTERNS: &[&str] = &[
    "discord.com/api/webhooks",
    "discordapp.com/api/webhooks",
    "api.telegram.org/bot",
    "hooks.slack.com/services",
    "pastebin.com/api",
    "hastebin.com",
    "transfer.sh",
    "file.io",
    "0x0.st",
    "webhook.site",
    "requestbin.com",
    "pipedream.net",
    "hookbin.com",
    "beeceptor.com",
    "requestcatcher.com",
    "ngrok.io",
    "ngrok-free.app",
    "serveo.net",
    "localhost.run",
    "loca.lt",
];

/// Suspicious script code patterns
const SCRIPT_PATTERNS: &[(&str, &str)] = &[
    ("powershell", "PowerShell command execution"),
    ("cmd.exe", "Windows command prompt execution"),
    ("cmd /c", "Windows command execution"),
    ("/bin/sh", "Shell command execution"),
    ("/bin/bash", "Bash command execution"),
    ("Process.Start", "Process execution (.NET)"),
    ("Runtime.exec", "Process execution (Java)"),
    ("os.system(", "System command execution (Python)"),
    ("subprocess", "Subprocess execution (Python)"),
    ("eval(", "Dynamic code evaluation"),
    ("exec(", "Dynamic code execution"),
    ("System.Diagnostics", ".NET process diagnostics"),
    ("WScript.Shell", "Windows scripting host"),
    ("Shell32", "Windows shell API"),
    ("reg add", "Windows registry modification"),
    ("reg delete", "Windows registry deletion"),
    ("net user", "Windows user account manipulation"),
    ("netsh", "Windows network configuration"),
    ("schtasks", "Windows scheduled task manipulation"),
    ("certutil -decode", "Certificate utility for file decoding"),
    ("bitsadmin", "Background file transfer"),
    ("mshta", "Microsoft HTML Application execution"),
    ("rundll32", "DLL execution via rundll32"),
];

/// Scan a pipeline for security issues
pub fn scan_config_security(pipeline: &Pipeline) -> Vec<SecurityIssue> {
    let mut issues = Vec::new();

    // Collect all target domains from HTTP requests (first 2 unique domains are "expected")
    let mut target_domains: Vec<String> = Vec::new();
    for block in &pipeline.blocks {
        if let BlockSettings::HttpRequest(ref s) = block.settings {
            if let Some(domain) = extract_domain(&s.url) {
                if !target_domains.contains(&domain) {
                    target_domains.push(domain);
                }
            }
        }
    }
    // Consider the first 3 unique domains as "legitimate targets"
    let legit_domains: Vec<String> = target_domains.iter().take(3).cloned().collect();

    for block in &pipeline.blocks {
        scan_block(block, &legit_domains, &mut issues);
    }

    issues
}

fn scan_block(block: &Block, legit_domains: &[String], issues: &mut Vec<SecurityIssue>) {
    match &block.settings {
        BlockSettings::HttpRequest(s) => {
            scan_http_request(block, s, legit_domains, issues);
        }
        BlockSettings::Script(s) => {
            scan_script(block, &s.code, issues);
        }
        BlockSettings::IfElse(s) => {
            for b in &s.true_blocks { scan_block(b, legit_domains, issues); }
            for b in &s.false_blocks { scan_block(b, legit_domains, issues); }
        }
        _ => {}
    }
}

fn scan_http_request(
    block: &Block, s: &HttpRequestSettings,
    legit_domains: &[String], issues: &mut Vec<SecurityIssue>,
) {
    let url_lower = s.url.to_lowercase();

    // Check for data exfiltration URLs
    for pattern in EXFIL_PATTERNS {
        if url_lower.contains(pattern) {
            issues.push(SecurityIssue {
                severity: SecuritySeverity::Critical,
                title: "Data exfiltration endpoint detected".into(),
                description: format!(
                    "Block \"{}\" sends data to a known exfiltration service. \
                    Credentials or captured data may be sent to an attacker-controlled endpoint.",
                    if block.label.is_empty() { "HTTP Request" } else { &block.label }
                ),
                code_snippet: format!("URL: {}\nMethod: {}\nBody: {}", s.url, s.method,
                    if s.body.len() > 200 { format!("{}...", &s.body[..200]) } else { s.body.clone() }
                ),
            });
            break;
        }
    }

    // Check if request sends credentials to a non-target domain
    if let Some(domain) = extract_domain(&s.url) {
        let is_legit = legit_domains.iter().any(|d| domain == *d || domain.ends_with(&format!(".{}", d)));
        if !is_legit {
            let body_has_creds = s.body.contains("<input.USER>") || s.body.contains("<input.PASS>")
                || s.body.contains("<USER>") || s.body.contains("<PASS>");
            let url_has_creds = s.url.contains("<input.USER>") || s.url.contains("<input.PASS>")
                || s.url.contains("<USER>") || s.url.contains("<PASS>");
            if body_has_creds || url_has_creds {
                issues.push(SecurityIssue {
                    severity: SecuritySeverity::Critical,
                    title: "Credentials sent to third-party domain".into(),
                    description: format!(
                        "Block \"{}\" sends username/password data to \"{}\" which is not \
                        the primary target domain. This may be credential harvesting.",
                        if block.label.is_empty() { "HTTP Request" } else { &block.label },
                        domain
                    ),
                    code_snippet: format!("URL: {}\nBody: {}", s.url,
                        if s.body.len() > 200 { format!("{}...", &s.body[..200]) } else { s.body.clone() }
                    ),
                });
            }
        }
    }

    // Check for raw IP addresses in URLs (potential C2)
    if let Some(host) = extract_host(&s.url) {
        let looks_like_ip = host.chars().all(|c| c.is_ascii_digit() || c == '.');
        if looks_like_ip && host.contains('.') && host != "127.0.0.1" {
            issues.push(SecurityIssue {
                severity: SecuritySeverity::Warning,
                title: "Request to raw IP address".into(),
                description: format!(
                    "Block \"{}\" targets a raw IP address ({}) instead of a domain name. \
                    This is unusual and could indicate a command-and-control server.",
                    if block.label.is_empty() { "HTTP Request" } else { &block.label },
                    host
                ),
                code_snippet: format!("URL: {}", s.url),
            });
        }
    }
}

fn scan_script(block: &Block, code: &str, issues: &mut Vec<SecurityIssue>) {
    let code_lower = code.to_lowercase();

    for (pattern, description) in SCRIPT_PATTERNS {
        if code_lower.contains(&pattern.to_lowercase()) {
            issues.push(SecurityIssue {
                severity: SecuritySeverity::Critical,
                title: format!("Suspicious script: {}", description),
                description: format!(
                    "Block \"{}\" contains code that may execute system commands or \
                    perform dangerous operations.",
                    if block.label.is_empty() { "Script" } else { &block.label }
                ),
                code_snippet: truncate_code(code, *pattern),
            });
        }
    }

    // Check for base64-encoded URLs in scripts
    if code.contains("base64") || code.contains("Base64") || code.contains("atob(") {
        issues.push(SecurityIssue {
            severity: SecuritySeverity::Warning,
            title: "Base64 encoding detected in script".into(),
            description: format!(
                "Block \"{}\" uses Base64 encoding which may be hiding malicious URLs or payloads.",
                if block.label.is_empty() { "Script" } else { &block.label }
            ),
            code_snippet: truncate_code(code, "base64"),
        });
    }

    // Check for exfiltration URLs in scripts
    let code_lower_ref = &code_lower;
    for pattern in EXFIL_PATTERNS {
        if code_lower_ref.contains(pattern) {
            issues.push(SecurityIssue {
                severity: SecuritySeverity::Critical,
                title: "Exfiltration URL in script".into(),
                description: format!(
                    "Block \"{}\" contains a URL to a known data exfiltration service.",
                    if block.label.is_empty() { "Script" } else { &block.label }
                ),
                code_snippet: truncate_code(code, pattern),
            });
            break;
        }
    }
}
