use crate::pipeline::block::*;

// ────────────────────────────────────────────────────────────
// Shared helpers used by multiple import submodules
// ────────────────────────────────────────────────────────────

/// Extract a quoted value after a prefix: `key = "value"` → `value`
/// Also handles `$"value"` (C# interpolated strings)
pub(super) fn extract_quoted_value(line: &str, prefix: &str) -> String {
    let after = line[prefix.len()..].trim();
    // Handle $"..." interpolated string — strip $ prefix, treat as regular quoted
    let after = if after.starts_with("$\"") {
        &after[1..]
    } else {
        after
    };
    if after.starts_with('"') && after.len() >= 2 {
        // Find the matching closing quote (handle escaped quotes)
        let chars: Vec<char> = after.chars().collect();
        let mut end = 1;
        while end < chars.len() {
            if chars[end] == '"' && (end == 0 || chars[end - 1] != '\\') {
                break;
            }
            end += 1;
        }
        let inner: String = chars[1..end].iter().collect();
        inner.replace("\\\"", "\"")
    } else {
        after.to_string()
    }
}

/// Extract value (may or may not be quoted, may be $"interpolated" or @variable)
pub(super) fn extract_value(line: &str, prefix: &str) -> String {
    let after = line[prefix.len()..].trim();

    // Handle $"..." interpolated string
    if after.starts_with("$\"") {
        let inner = &after[2..];
        if let Some(end) = inner.rfind('"') {
            return inner[..end].to_string();
        }
        return inner.to_string();
    }

    // Handle @variable reference (strip @ prefix)
    if after.starts_with('@') {
        return after[1..].to_string();
    }

    // Handle regular quoted string
    if after.starts_with('"') && after.ends_with('"') && after.len() >= 2 {
        after[1..after.len() - 1].to_string()
    } else {
        after.to_string()
    }
}

/// Parse OB2 custom headers: `customHeaders = ${("key", "value"), ...}` or `{("key", "value"), ...}`
pub(super) fn parse_custom_headers(line: &str) -> Vec<(String, String)> {
    let mut headers = Vec::new();

    // Find start of header block — either ${(...)} or {(...)}
    let start;
    if let Some(s) = line.find("${") {
        start = s + 2; // skip ${
    } else if let Some(s) = line.find('{') {
        start = s + 1; // skip {
    } else {
        return headers;
    };
    let content = &line[start..];
    let end = match content.rfind('}') {
        Some(e) => e,
        None => return headers,
    };
    let inner = &content[..end];

    // Parse each ("key", "value") tuple — quote-aware parenthesis tracking
    let chars: Vec<char> = inner.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '(' {
            let mut depth = 1;
            let paren_start = i + 1;
            i += 1;
            let mut in_quote = false;
            let mut escape = false;
            while i < chars.len() && depth > 0 {
                if escape {
                    escape = false;
                    i += 1;
                    continue;
                }
                if chars[i] == '\\' {
                    escape = true;
                    i += 1;
                    continue;
                }
                if chars[i] == '"' {
                    in_quote = !in_quote;
                }
                if !in_quote {
                    if chars[i] == '(' { depth += 1; }
                    if chars[i] == ')' { depth -= 1; }
                }
                i += 1;
            }
            let paren_content: String = chars[paren_start..i - 1].iter().collect();
            if let Some((k, v)) = parse_header_pair(&paren_content) {
                headers.push((k, v));
            }
        } else {
            i += 1;
        }
    }

    headers
}

/// Parse a single header pair: `"key", "value"` → (key, value)
fn parse_header_pair(content: &str) -> Option<(String, String)> {
    let mut parts = Vec::new();
    let mut in_quote = false;
    let mut current = String::new();
    let mut escape = false;

    for ch in content.chars() {
        if escape {
            current.push(ch);
            escape = false;
            continue;
        }
        if ch == '\\' {
            escape = true;
            continue;
        }
        if ch == '"' {
            in_quote = !in_quote;
            continue;
        }
        if ch == ',' && !in_quote {
            parts.push(std::mem::take(&mut current));
            continue;
        }
        if in_quote {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }

    if parts.len() == 2 {
        Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
    } else {
        None
    }
}

/// Parse a STRINGKEY condition: `STRINGKEY @data.SOURCE Contains "value"`
pub(super) fn parse_stringkey(line: &str) -> Option<KeyCondition> {
    let after = line.strip_prefix("STRINGKEY ")?.trim();

    // Find source (first token, may start with @)
    let (source_raw, rest) = after.split_once(' ')?;
    let source = source_raw.strip_prefix('@').unwrap_or(source_raw).to_string();

    // Find comparison operator and value
    let (comparison_str, value_raw) = rest.trim().split_once(' ')?;

    let comparison = match comparison_str {
        "Contains" => Comparison::Contains,
        "DoesNotContain" => Comparison::NotContains,
        "EqualTo" | "Is" => Comparison::EqualTo,
        "NotEqualTo" | "IsNot" => Comparison::NotEqualTo,
        "MatchesRegex" => Comparison::MatchesRegex,
        "GreaterThan" => Comparison::GreaterThan,
        "LessThan" => Comparison::LessThan,
        "Exists" => Comparison::Exists,
        "DoesNotExist" => Comparison::NotExists,
        _ => Comparison::Contains,
    };

    let value = value_raw.trim().trim_matches('"').to_string();

    Some(KeyCondition { source, comparison, value })
}

/// Extract the domain from a URL
pub(super) fn extract_domain(url: &str) -> Option<String> {
    let url = url.trim();
    // Handle variable URLs like <varName>
    if url.starts_with('<') { return None; }
    let after_scheme = if let Some(pos) = url.find("://") {
        &url[pos + 3..]
    } else {
        url
    };
    let host_port = after_scheme.split('/').next()?;
    let host = host_port.split(':').next()?;
    if host.is_empty() || host.contains('<') { return None; }
    Some(host.to_lowercase())
}

/// Extract the host portion from a URL
pub(super) fn extract_host(url: &str) -> Option<String> {
    extract_domain(url)
}

/// Truncate code to show context around a matched pattern
pub(super) fn truncate_code(code: &str, pattern: &str) -> String {
    let code_lower = code.to_lowercase();
    let pattern_lower = pattern.to_lowercase();
    if let Some(pos) = code_lower.find(&pattern_lower) {
        let start = pos.saturating_sub(80);
        let end = (pos + pattern.len() + 80).min(code.len());
        let snippet = &code[start..end];
        if start > 0 || end < code.len() {
            format!("...{}...", snippet.trim())
        } else {
            snippet.trim().to_string()
        }
    } else {
        if code.len() > 300 {
            format!("{}...", &code[..300])
        } else {
            code.to_string()
        }
    }
}
