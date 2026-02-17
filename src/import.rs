use std::io::Read;

use crate::pipeline::block::*;
use crate::pipeline::*;

// ────────────────────────────────────────────────────────────
// Public types
// ────────────────────────────────────────────────────────────

/// Result of importing a config, containing the parsed pipeline and any warnings
pub struct ImportResult {
    pub pipeline: Pipeline,
    pub warnings: Vec<String>,
    pub security_issues: Vec<SecurityIssue>,
}

/// A security issue found during config scanning
#[derive(Debug, Clone, serde::Serialize)]
pub struct SecurityIssue {
    pub severity: SecuritySeverity,
    pub title: String,
    pub description: String,
    pub code_snippet: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum SecuritySeverity {
    Critical,
    Warning,
}

// ────────────────────────────────────────────────────────────
// Public entry points
// ────────────────────────────────────────────────────────────

/// Import from raw file bytes — auto-detects format (ZIP for .opk, text for .loli/.json/.svb)
pub fn import_config_bytes(bytes: &[u8]) -> Result<ImportResult, String> {
    // Check for ZIP magic bytes (PK\x03\x04)
    let mut result = if bytes.len() >= 4 && bytes[0] == b'P' && bytes[1] == b'K' && bytes[2] == 3 && bytes[3] == 4 {
        import_opk(bytes)?
    } else {
        let content = std::str::from_utf8(bytes)
            .map_err(|e| format!("File is not valid UTF-8 text: {}", e))?;
        import_config(content)?
    };

    // Run security scan
    result.security_issues = scan_config_security(&result.pipeline);
    Ok(result)
}

/// Auto-detect text format and import
pub fn import_config(content: &str) -> Result<ImportResult, String> {
    // SVB format (SilverBullet) — starts with [SETTINGS]
    if content.trim_start().starts_with("[SETTINGS]") {
        return import_svb(content);
    }
    // Try JSON first (OpenBullet2 JSON format)
    if content.trim_start().starts_with('{') {
        return import_openbullet_json(content);
    }
    // Otherwise try LoliCode
    import_lolicode(content)
}

// ────────────────────────────────────────────────────────────
// OB2 .opk (ZIP archive) importer
// ────────────────────────────────────────────────────────────

/// Import an OB2 .opk file (ZIP archive containing script.loli + metadata + settings)
fn import_opk(bytes: &[u8]) -> Result<ImportResult, String> {
    let reader = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|e| format!("Failed to open .opk archive: {}", e))?;

    let mut script_loli = String::new();
    let mut metadata_json = String::new();
    let mut settings_json = String::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read archive entry: {}", e))?;
        let name = file.name().to_string();
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| format!("Failed to read {}: {}", name, e))?;

        match name.as_str() {
            "script.loli" => script_loli = content,
            "metadata.json" => metadata_json = content,
            "settings.json" => settings_json = content,
            _ => {} // readme.md, startup.loli, etc
        }
    }

    if script_loli.is_empty() {
        return Err("No script.loli found in .opk archive".into());
    }

    let mut pipeline = Pipeline::default();
    let mut warnings = Vec::new();

    // Parse metadata → name, author
    if !metadata_json.is_empty() {
        if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&metadata_json) {
            if let Some(name) = meta.get("Name").and_then(|n| n.as_str()) {
                pipeline.name = name.to_string();
            }
            if let Some(author) = meta.get("Author").and_then(|a| a.as_str()) {
                pipeline.author = author.to_string();
            }
        }
    }

    // Parse settings → runner/proxy/data settings
    if !settings_json.is_empty() {
        apply_ob2_settings(&mut pipeline, &settings_json);
    }

    // Parse the LoliCode script → blocks
    pipeline.blocks = parse_lolicode_blocks(&script_loli, &mut warnings)?;

    Ok(ImportResult { pipeline, warnings, security_issues: Vec::new() })
}

/// Map OB2 settings.json fields to Pipeline settings
fn apply_ob2_settings(pipeline: &mut Pipeline, json_str: &str) {
    let json: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return,
    };

    // GeneralSettings.SuggestedBots → threads
    if let Some(general) = json.get("GeneralSettings") {
        if let Some(bots) = general.get("SuggestedBots").and_then(|v| v.as_u64()) {
            pipeline.runner_settings.threads = bots as u32;
        }
    }

    // ProxySettings.UseProxies → proxy mode
    if let Some(proxy) = json.get("ProxySettings") {
        if proxy.get("UseProxies").and_then(|v| v.as_bool()).unwrap_or(false) {
            pipeline.proxy_settings.proxy_mode = ProxyMode::Rotate;
        }
        if let Some(ban_loop) = proxy.get("BanLoopEvasion").and_then(|v| v.as_u64()) {
            pipeline.proxy_settings.ban_duration_secs = ban_loop;
        }
    }

    // DataSettings.AllowedWordlistTypes → wordlist type + slices
    if let Some(data) = json.get("DataSettings") {
        if let Some(types) = data.get("AllowedWordlistTypes").and_then(|v| v.as_array()) {
            if let Some(first) = types.first().and_then(|v| v.as_str()) {
                pipeline.data_settings.wordlist_type = first.to_string();
                // Set default slices based on wordlist type
                match first {
                    "Credentials" => {
                        pipeline.data_settings.separator = ':';
                        pipeline.data_settings.slices = vec!["USER".into(), "PASS".into()];
                    }
                    "Emails" => {
                        pipeline.data_settings.separator = ':';
                        pipeline.data_settings.slices = vec!["EMAIL".into()];
                    }
                    _ => {}
                }
            }
        }
    }
}

// ────────────────────────────────────────────────────────────
// LoliCode BLOCK: parser (used by both .opk and plain .loli)
// ────────────────────────────────────────────────────────────

/// Parse LoliCode BLOCK:Type ... ENDBLOCK syntax
fn parse_lolicode_blocks(content: &str, warnings: &mut Vec<String>) -> Result<Vec<Block>, String> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    let mut preamble_lines = Vec::new();

    // Collect C# preamble (code before the first BLOCK:)
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with("BLOCK:") {
            break;
        }
        if !trimmed.is_empty() {
            preamble_lines.push(trimmed.to_string());
        }
        i += 1;
    }

    // If there's a meaningful C# preamble, add as a commented Script block
    if !preamble_lines.is_empty() {
        let preamble_code = preamble_lines.join("\n");
        // Only add if it contains actual logic (not just comments/whitespace)
        if preamble_code.contains("ConstantString") || preamble_code.contains("MatchRegex")
            || preamble_code.contains("data.") || preamble_code.contains("CheckCondition")
        {
            let mut block = Block::new(BlockType::Script);
            block.label = "OB2 Preamble (C#)".into();
            if let BlockSettings::Script(ref mut s) = block.settings {
                s.code = format!(
                    "// Converted from OB2 C# preamble — review and adapt for reqflow\n{}",
                    preamble_code.lines().map(|l| format!("// {}", l)).collect::<Vec<_>>().join("\n")
                );
            }
            blocks.push(block);
        }
    }

    // Parse BLOCK:Type ... ENDBLOCK segments
    while i < lines.len() {
        let trimmed = lines[i].trim();

        if trimmed.starts_with("BLOCK:") {
            let block_type_str = trimmed[6..].trim().to_string();
            let mut block_lines = Vec::new();
            let mut label = String::new();
            let mut disabled = false;
            i += 1;

            // Collect all lines until ENDBLOCK
            while i < lines.len() {
                let line = lines[i].trim();
                if line == "ENDBLOCK" {
                    i += 1;
                    break;
                }
                if line.starts_with("LABEL:") {
                    label = line[6..].trim().to_string();
                } else if line == "DISABLED" {
                    disabled = true;
                } else {
                    block_lines.push(line.to_string());
                }
                i += 1;
            }

            if let Some(mut block) = convert_ob2_block(&block_type_str, &label, &block_lines, warnings) {
                if disabled {
                    block.disabled = true;
                }
                blocks.push(block);
            }
        } else {
            i += 1;
        }
    }

    if blocks.is_empty() {
        return Err("No blocks found in LoliCode script".into());
    }

    Ok(blocks)
}

/// Convert a single OB2 block to a reqflow Block
fn convert_ob2_block(type_str: &str, label: &str, lines: &[String], warnings: &mut Vec<String>) -> Option<Block> {
    match type_str {
        "HttpRequest" => convert_http_request(label, lines),
        "Keycheck" => convert_keycheck(label, lines),
        "Parse" => convert_parse(label, lines),
        "RandomString" => convert_random_string(label, lines),
        "ConstantString" => convert_constant_string(label, lines),
        "UrlEncode" => convert_string_fn(label, lines, StringFnType::URLEncode),
        "UrlDecode" => convert_string_fn(label, lines, StringFnType::URLDecode),
        "ToUppercase" | "Uppercase" => convert_string_fn(label, lines, StringFnType::ToUpper),
        "ToLowercase" | "Lowercase" => convert_string_fn(label, lines, StringFnType::ToLower),
        "Base64Encode" => convert_string_fn(label, lines, StringFnType::Base64Encode),
        "Base64Decode" => convert_string_fn(label, lines, StringFnType::Base64Decode),
        "Replace" => convert_string_fn(label, lines, StringFnType::Replace),
        "Substring" => convert_string_fn(label, lines, StringFnType::Substring),
        "ReverseString" | "Reverse" => convert_string_fn(label, lines, StringFnType::Reverse),
        "Length" | "GetLength" => convert_string_fn(label, lines, StringFnType::Length),
        "GetRandomItem" => convert_list_fn(label, lines, ListFnType::RandomItem),
        "Md5Hash" | "MD5" | "SHA256" | "Sha256" | "SHA1" | "Sha1" | "SHA512" | "Sha512" => {
            convert_crypto_fn(label, lines, type_str)
        }
        "ClearCookies" => {
            let mut block = Block::new(BlockType::ClearCookies);
            if !label.is_empty() { block.label = label.to_string(); }
            Some(block)
        }
        "Script" | "LoliScript" => convert_script(label, lines),
        "Delay" | "Wait" => convert_delay(label, lines),
        "CountOccurrences" | "Translate" => convert_unsupported_to_script(label, type_str, lines),
        _ => {
            // Unknown block type → add as disabled Script with a note
            warnings.push(format!("Unsupported block type: {} — converted to disabled script", type_str));
            let mut block = Block::new(BlockType::Script);
            block.label = if label.is_empty() {
                format!("Unknown: {}", type_str)
            } else {
                label.to_string()
            };
            block.disabled = true;
            if let BlockSettings::Script(ref mut s) = block.settings {
                s.code = format!("// Unsupported OB2 block type: {}\n// Lines:\n{}",
                    type_str,
                    lines.iter().map(|l| format!("// {}", l)).collect::<Vec<_>>().join("\n")
                );
            }
            Some(block)
        }
    }
}

// ────────────────────────────────────────────────────────────
// Block converters
// ────────────────────────────────────────────────────────────

fn convert_http_request(label: &str, lines: &[String]) -> Option<Block> {
    let mut block = Block::new(BlockType::HttpRequest);
    if !label.is_empty() {
        block.label = label.to_string();
    }

    if let BlockSettings::HttpRequest(ref mut s) = block.settings {
        let mut raw_url = String::new();
        let mut custom_headers: Vec<(String, String)> = Vec::new();
        let mut has_custom_headers = false;
        let mut tp_url: Option<String> = None;
        let mut tp_method: Option<String> = None;
        let mut found_body = false;

        for line in lines {
            let trimmed = line.trim();

            if trimmed.starts_with("url = ") {
                // Handle url = @VarName (variable reference) → <VarName>
                let raw = trimmed[6..].trim();
                if raw.starts_with('@') {
                    raw_url = format!("<{}>", &raw[1..]);
                } else {
                    raw_url = extract_quoted_value(trimmed, "url = ");
                }
            } else if trimmed.starts_with("method = ") {
                s.method = trimmed[9..].trim().to_string();
            } else if trimmed.starts_with("customHeaders = ") {
                custom_headers = parse_custom_headers(trimmed);
                has_custom_headers = true;
            } else if trimmed.starts_with("autoRedirect = ") {
                let val = trimmed[15..].trim();
                if val.eq_ignore_ascii_case("false") {
                    s.follow_redirects = false;
                    s.auto_redirect = false;
                }
            } else if trimmed.starts_with("TYPE:") {
                let body_type = trimmed[5..].trim();
                s.body_type = match body_type {
                    "STANDARD" => BodyType::Standard,
                    "RAW" => BodyType::Raw,
                    "MULTIPART" => BodyType::Multipart,
                    _ => BodyType::Standard,
                };
            } else if trimmed.starts_with("$\"") {
                // Interpolated body: $"content" — strip $" prefix and trailing "
                found_body = true;
                let body_str = &trimmed[2..];
                s.body = if body_str.ends_with('"') {
                    body_str[..body_str.len()-1].to_string()
                } else {
                    body_str.to_string()
                };
            } else if found_body && trimmed.starts_with('"') && trimmed.ends_with('"') && !trimmed.contains(" = ") {
                // Content type line (bare quoted string after body)
                s.content_type = trimmed[1..trimmed.len()-1].to_string();
            }
            // Silently ignore: httpLibrary, securityProtocol, useCustomCipherSuites,
            // customCipherSuites, Content-Length (OB2-specific properties)
        }

        // Detect TLS proxy patterns: x-tp-* or x-url/x-proxy
        for (k, v) in &custom_headers {
            match k.as_str() {
                "x-tp-url" | "x-url" => tp_url = Some(v.clone()),
                "x-tp-method" => tp_method = Some(v.clone()),
                _ => {}
            }
        }

        if let Some(real_url) = tp_url {
            s.url = real_url;
            if let Some(real_method) = tp_method {
                s.method = real_method;
            }
            // Strip transport proxy headers, keep real headers
            s.headers = custom_headers.into_iter()
                .filter(|(k, _)| {
                    !k.starts_with("x-tp-") && k != "x-url" && k != "x-proxy"
                        && k != "x-identifier" && k != "x-session-id"
                })
                .collect();
        } else {
            s.url = raw_url;
            // Only overwrite default headers if customHeaders was present in the block
            if has_custom_headers {
                s.headers = custom_headers;
            }
        }
    }

    Some(block)
}

fn convert_keycheck(label: &str, lines: &[String]) -> Option<Block> {
    let mut block = Block::new(BlockType::KeyCheck);
    if !label.is_empty() {
        block.label = label.to_string();
    }

    if let BlockSettings::KeyCheck(ref mut s) = block.settings {
        s.keychains.clear();
        let mut current_keychain: Option<Keychain> = None;

        for line in lines {
            let trimmed = line.trim();

            if trimmed.starts_with("KEYCHAIN ") {
                // Save previous keychain
                if let Some(kc) = current_keychain.take() {
                    if !kc.conditions.is_empty() {
                        s.keychains.push(kc);
                    }
                }

                // Parse "KEYCHAIN FAIL OR" / "KEYCHAIN SUCCESS OR" / "KEYCHAIN CUSTOM OR"
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                let status = match parts.get(1).copied().unwrap_or("") {
                    "FAIL" => BotStatus::Fail,
                    "SUCCESS" => BotStatus::Success,
                    "CUSTOM" | "2FA" | "CAPTCHA" | "LOCKED" => BotStatus::Custom,
                    "BAN" => BotStatus::Ban,
                    "RETRY" => BotStatus::Retry,
                    "ERROR" => BotStatus::Error,
                    _ => BotStatus::None,
                };
                current_keychain = Some(Keychain {
                    result: status,
                    conditions: Vec::new(),
                });
            } else if trimmed.starts_with("STRINGKEY ") {
                if let Some(ref mut kc) = current_keychain {
                    if let Some(cond) = parse_stringkey(trimmed) {
                        kc.conditions.push(cond);
                    }
                }
            }
        }

        // Push last keychain
        if let Some(kc) = current_keychain {
            if !kc.conditions.is_empty() {
                s.keychains.push(kc);
            }
        }
    }

    Some(block)
}

fn convert_parse(label: &str, lines: &[String]) -> Option<Block> {
    let mut mode = String::from("LR");
    let mut input = "data.SOURCE".to_string();
    let mut j_token = String::new();
    let mut left_delim = String::new();
    let mut right_delim = String::new();
    let mut regex_pattern = String::new();
    let mut regex_output = String::new();
    let mut css_selector = String::new();
    let mut css_attribute = String::new();
    let mut output_var = "PARSED".to_string();
    let mut is_capture = false;

    for line in lines {
        let trimmed = line.trim();

        if trimmed.starts_with("input = ") {
            input = extract_value(trimmed, "input = ");
            // Redundant safety check (extract_value already strips @)
            if input.starts_with('@') {
                input = input[1..].to_string();
            }
        } else if trimmed.starts_with("jToken = ") {
            j_token = extract_quoted_value(trimmed, "jToken = ");
        } else if trimmed.starts_with("leftDelim = ") {
            left_delim = extract_quoted_value(trimmed, "leftDelim = ");
        } else if trimmed.starts_with("rightDelim = ") {
            right_delim = extract_quoted_value(trimmed, "rightDelim = ");
        } else if trimmed.starts_with("pattern = ") || trimmed.starts_with("regex = ") {
            let pfx = if trimmed.starts_with("pattern = ") { "pattern = " } else { "regex = " };
            regex_pattern = extract_quoted_value(trimmed, pfx);
        } else if trimmed.starts_with("outputFormat = ") {
            regex_output = extract_quoted_value(trimmed, "outputFormat = ");
        } else if trimmed.starts_with("cssSelector = ") || trimmed.starts_with("selector = ") {
            let pfx = if trimmed.starts_with("cssSelector = ") { "cssSelector = " } else { "selector = " };
            css_selector = extract_quoted_value(trimmed, pfx);
        } else if trimmed.starts_with("attributeName = ") || trimmed.starts_with("attribute = ") {
            let pfx = if trimmed.starts_with("attributeName = ") { "attributeName = " } else { "attribute = " };
            css_attribute = extract_quoted_value(trimmed, pfx);
        } else if trimmed.starts_with("MODE:") {
            mode = match &trimmed[5..] {
                "Json" | "json" => "Json",
                "LR" | "lr" => "LR",
                "Regex" | "regex" => "Regex",
                "CSS" | "css" => "CSS",
                _ => "LR",
            }.to_string();
        } else if trimmed.starts_with("=> VAR @") {
            output_var = trimmed[8..].trim().to_string();
            is_capture = false;
        } else if trimmed.starts_with("=> CAP @") {
            output_var = trimmed[8..].trim().to_string();
            is_capture = true;
        }
        // Silently ignore: urlEncodeOutput, recursive, useRegex, etc.
    }

    match mode.as_str() {
        "Json" => {
            let mut block = Block::new(BlockType::ParseJSON);
            if !label.is_empty() { block.label = label.to_string(); }
            if let BlockSettings::ParseJSON(ref mut s) = block.settings {
                s.input_var = input;
                s.json_path = j_token;
                s.output_var = output_var;
                s.capture = is_capture;
            }
            Some(block)
        }
        "LR" => {
            let mut block = Block::new(BlockType::ParseLR);
            if !label.is_empty() { block.label = label.to_string(); }
            if let BlockSettings::ParseLR(ref mut s) = block.settings {
                s.input_var = input;
                s.left = left_delim;
                s.right = right_delim;
                s.output_var = output_var;
                s.capture = is_capture;
            }
            Some(block)
        }
        "Regex" => {
            let mut block = Block::new(BlockType::ParseRegex);
            if !label.is_empty() { block.label = label.to_string(); }
            if let BlockSettings::ParseRegex(ref mut s) = block.settings {
                s.input_var = input;
                s.pattern = regex_pattern;
                s.output_format = if regex_output.is_empty() { "$1".into() } else { regex_output };
                s.output_var = output_var;
                s.capture = is_capture;
            }
            Some(block)
        }
        "CSS" => {
            let mut block = Block::new(BlockType::ParseCSS);
            if !label.is_empty() { block.label = label.to_string(); }
            if let BlockSettings::ParseCSS(ref mut s) = block.settings {
                s.input_var = input;
                s.selector = css_selector;
                s.attribute = if css_attribute.is_empty() { "innerText".into() } else { css_attribute };
                s.output_var = output_var;
                s.capture = is_capture;
            }
            Some(block)
        }
        _ => None,
    }
}

fn convert_random_string(label: &str, lines: &[String]) -> Option<Block> {
    let mut pattern = String::new();
    let mut output_var = "RANDOM".to_string();
    let mut is_capture = false;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("input = ") {
            pattern = extract_quoted_value(trimmed, "input = ");
        } else if trimmed.starts_with("=> VAR @") {
            output_var = trimmed[8..].trim().to_string();
            is_capture = false;
        } else if trimmed.starts_with("=> CAP @") {
            output_var = trimmed[8..].trim().to_string();
            is_capture = true;
        }
    }

    // Map to RandomData block — OB2 ?m = hex char, pattern with dashes → UUID-like
    let mut block = Block::new(BlockType::RandomData);
    block.label = if label.is_empty() { "Random String".into() } else { label.to_string() };

    if let BlockSettings::RandomData(ref mut s) = block.settings {
        // Detect pattern type: if all ?m and dashes → hex string / UUID-like
        let stripped = pattern.replace('-', "").replace("?m", "");
        if stripped.is_empty() && pattern.contains("?m") {
            // Pure hex pattern with optional dashes
            let hex_len = pattern.matches("?m").count();
            s.data_type = RandomDataType::String;
            s.string_length = hex_len as u32;
            s.string_charset = "custom".into();
            s.custom_chars = "0123456789abcdef".into();
        } else {
            // Generic pattern — store as custom chars with length
            s.data_type = RandomDataType::String;
            s.string_length = pattern.len() as u32 / 2; // each ?x = 1 char
            s.string_charset = "alphanumeric".into();
        }
        s.output_var = output_var;
        s.capture = is_capture;
    }

    Some(block)
}

fn convert_constant_string(label: &str, lines: &[String]) -> Option<Block> {
    let mut value = String::new();
    let mut output_var = "CONST".to_string();
    let mut is_capture = false;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("value = ") {
            value = extract_quoted_value(trimmed, "value = ");
        } else if trimmed.starts_with("=> VAR @") {
            output_var = trimmed[8..].trim().to_string();
            is_capture = false;
        } else if trimmed.starts_with("=> CAP @") {
            output_var = trimmed[8..].trim().to_string();
            is_capture = true;
        }
    }

    let mut block = Block::new(BlockType::SetVariable);
    block.label = if label.is_empty() { "Constant".into() } else { label.to_string() };

    if let BlockSettings::SetVariable(ref mut s) = block.settings {
        s.name = output_var;
        s.value = value;
        s.capture = is_capture;
    }

    Some(block)
}

fn convert_string_fn(label: &str, lines: &[String], fn_type: StringFnType) -> Option<Block> {
    let mut input_var = String::new();
    let mut output_var = "RESULT".to_string();
    let mut is_capture = false;
    let mut replace_what = String::new();
    let mut replace_with = String::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("input = ") {
            input_var = extract_value(trimmed, "input = ");
            if input_var.starts_with('@') { input_var = input_var[1..].to_string(); }
            // If input is a simple <varName> interpolation, extract just the var name
            if input_var.starts_with('<') && input_var.ends_with('>')
                && !input_var[1..input_var.len()-1].contains('<')
            {
                input_var = input_var[1..input_var.len()-1].to_string();
            }
        } else if trimmed.starts_with("replaceWhat = ") {
            replace_what = extract_quoted_value(trimmed, "replaceWhat = ");
        } else if trimmed.starts_with("replaceWith = ") {
            replace_with = extract_quoted_value(trimmed, "replaceWith = ");
        } else if trimmed.starts_with("=> VAR @") {
            output_var = trimmed[8..].trim().to_string();
            is_capture = false;
        } else if trimmed.starts_with("=> CAP @") {
            output_var = trimmed[8..].trim().to_string();
            is_capture = true;
        }
    }

    let mut block = Block::new(BlockType::StringFunction);
    block.label = if label.is_empty() { format!("{:?}", fn_type) } else { label.to_string() };

    if let BlockSettings::StringFunction(ref mut s) = block.settings {
        s.function_type = fn_type;
        s.input_var = input_var;
        s.output_var = output_var;
        s.capture = is_capture;
        if !replace_what.is_empty() {
            s.param1 = replace_what;
        }
        if !replace_with.is_empty() {
            s.param2 = replace_with;
        }
    }

    Some(block)
}

fn convert_list_fn(label: &str, lines: &[String], fn_type: ListFnType) -> Option<Block> {
    let mut input_var = String::new();
    let mut output_var = "RESULT".to_string();
    let mut is_capture = false;
    let mut inline_list = String::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("input = ") || trimmed.starts_with("list = ") {
            let prefix = if trimmed.starts_with("input = ") { "input = " } else { "list = " };
            let raw = trimmed[prefix.len()..].trim();
            if raw.starts_with('[') {
                // Inline JSON array: list = ["item1", "item2", ...]
                inline_list = raw.to_string();
            } else {
                input_var = extract_value(trimmed, prefix);
                if input_var.starts_with('@') { input_var = input_var[1..].to_string(); }
            }
        } else if trimmed.starts_with("=> VAR @") {
            output_var = trimmed[8..].trim().to_string();
            is_capture = false;
        } else if trimmed.starts_with("=> CAP @") {
            output_var = trimmed[8..].trim().to_string();
            is_capture = true;
        }
    }

    let mut block = Block::new(BlockType::ListFunction);
    block.label = if label.is_empty() { format!("{:?}", fn_type) } else { label.to_string() };

    if let BlockSettings::ListFunction(ref mut s) = block.settings {
        s.function_type = fn_type;
        s.input_var = input_var;
        s.output_var = output_var;
        s.capture = is_capture;
        if !inline_list.is_empty() {
            s.param1 = inline_list;
        }
    }

    Some(block)
}

fn convert_script(label: &str, lines: &[String]) -> Option<Block> {
    let mut block = Block::new(BlockType::Script);
    block.label = if label.is_empty() { "Script".into() } else { label.to_string() };

    if let BlockSettings::Script(ref mut s) = block.settings {
        s.code = lines.join("\n");
    }

    Some(block)
}

fn convert_delay(label: &str, lines: &[String]) -> Option<Block> {
    let mut block = Block::new(BlockType::Delay);
    block.label = if label.is_empty() { "Delay".into() } else { label.to_string() };

    for line in lines {
        let trimmed = line.trim();
        if let Ok(ms) = trimmed.parse::<u64>() {
            if let BlockSettings::Delay(ref mut s) = block.settings {
                s.min_ms = ms;
                s.max_ms = ms;
            }
        }
    }

    Some(block)
}

/// Convert an unsupported OB2 block type to a Script block preserving all data
fn convert_unsupported_to_script(label: &str, block_type: &str, lines: &[String]) -> Option<Block> {
    let mut block = Block::new(BlockType::Script);
    block.label = if label.is_empty() {
        format!("OB2: {}", block_type)
    } else {
        label.to_string()
    };
    // Parse output variable if present
    let mut output_info = String::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("=> VAR @") || trimmed.starts_with("=> CAP @") {
            output_info = format!("\n// Output: {}", trimmed);
        }
    }
    if let BlockSettings::Script(ref mut s) = block.settings {
        s.code = format!(
            "// OB2 {} block — review and implement manually{}\n{}",
            block_type,
            output_info,
            lines.iter().map(|l| format!("// {}", l)).collect::<Vec<_>>().join("\n")
        );
    }
    Some(block)
}

/// Convert crypto function blocks (Md5Hash, SHA256, etc.)
fn convert_crypto_fn(label: &str, lines: &[String], type_str: &str) -> Option<Block> {
    let mut block = Block::new(BlockType::CryptoFunction);
    if !label.is_empty() { block.label = label.to_string(); }

    let mut input_var = String::new();
    let mut output_var = "HASH".to_string();
    let mut is_capture = false;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("input = ") {
            input_var = extract_value(trimmed, "input = ");
            if input_var.starts_with('@') { input_var = input_var[1..].to_string(); }
        } else if trimmed.starts_with("=> VAR @") {
            output_var = trimmed[8..].trim().to_string();
            is_capture = false;
        } else if trimmed.starts_with("=> CAP @") {
            output_var = trimmed[8..].trim().to_string();
            is_capture = true;
        }
    }

    if let BlockSettings::CryptoFunction(ref mut s) = block.settings {
        s.function_type = match type_str {
            "Md5Hash" | "MD5" => CryptoFnType::MD5,
            "SHA1" | "Sha1" => CryptoFnType::SHA1,
            "SHA256" | "Sha256" => CryptoFnType::SHA256,
            "SHA512" | "Sha512" => CryptoFnType::SHA512,
            _ => CryptoFnType::MD5,
        };
        s.input_var = input_var;
        s.output_var = output_var;
        s.capture = is_capture;
    }

    Some(block)
}

// ────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────

/// Extract a quoted value after a prefix: `key = "value"` → `value`
/// Also handles `$"value"` (C# interpolated strings)
fn extract_quoted_value(line: &str, prefix: &str) -> String {
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
fn extract_value(line: &str, prefix: &str) -> String {
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
fn parse_custom_headers(line: &str) -> Vec<(String, String)> {
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
fn parse_stringkey(line: &str) -> Option<KeyCondition> {
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

// ────────────────────────────────────────────────────────────
// Legacy JSON-based OB2 importer (for non-LoliCode .json configs)
// ────────────────────────────────────────────────────────────

fn import_openbullet_json(content: &str) -> Result<ImportResult, String> {
    let json: serde_json::Value = serde_json::from_str(content)
        .map_err(|e| format!("Failed to parse config JSON: {}", e))?;

    let mut pipeline = Pipeline::default();
    pipeline.name = json.get("Settings")
        .and_then(|s| s.get("Name"))
        .and_then(|n| n.as_str())
        .unwrap_or("Imported OB2 Config")
        .to_string();

    let mut blocks = Vec::new();

    if let Some(ob_blocks) = json.get("Settings")
        .and_then(|s| s.get("Blocks"))
        .and_then(|b| b.as_array())
    {
        for ob_block in ob_blocks {
            let block_type = ob_block.get("Type").and_then(|t| t.as_str()).unwrap_or("");
            match block_type {
                "HttpRequestBlock" | "Request" => {
                    let mut block = Block::new(BlockType::HttpRequest);
                    if let BlockSettings::HttpRequest(ref mut s) = block.settings {
                        s.method = ob_block.get("Method")
                            .and_then(|m| m.as_str())
                            .unwrap_or("GET")
                            .to_string();
                        s.url = ob_block.get("Url")
                            .and_then(|u| u.as_str())
                            .unwrap_or("")
                            .to_string();
                    }
                    blocks.push(block);
                }
                "KeycheckBlock" | "Keycheck" => {
                    blocks.push(Block::new(BlockType::KeyCheck));
                }
                "ParseBlock" => {
                    let mode = ob_block.get("Mode").and_then(|m| m.as_str()).unwrap_or("");
                    match mode {
                        "LR" => blocks.push(Block::new(BlockType::ParseLR)),
                        "JSON" => blocks.push(Block::new(BlockType::ParseJSON)),
                        "Regex" => blocks.push(Block::new(BlockType::ParseRegex)),
                        "CSS" => blocks.push(Block::new(BlockType::ParseCSS)),
                        _ => blocks.push(Block::new(BlockType::ParseLR)),
                    }
                }
                _ => {}
            }
        }
    }

    if blocks.is_empty() {
        return Err("No recognizable blocks found in OpenBullet config".into());
    }

    pipeline.blocks = blocks;
    Ok(ImportResult { pipeline, warnings: Vec::new(), security_issues: Vec::new() })
}

/// Attempt to import a LoliCode (.loli) text config into a Pipeline
pub fn import_lolicode(content: &str) -> Result<ImportResult, String> {
    let mut pipeline = Pipeline::default();
    let mut warnings = Vec::new();
    pipeline.name = "Imported LoliCode".into();

    // Use the full BLOCK: parser if the content has BLOCK: directives
    if content.contains("BLOCK:") {
        pipeline.blocks = parse_lolicode_blocks(content, &mut warnings)?;
        return Ok(ImportResult { pipeline, warnings, security_issues: Vec::new() });
    }

    // Fallback: legacy line-by-line scanning for older LoliCode format
    let mut blocks = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        if trimmed.starts_with("REQUEST ") || trimmed.starts_with("request ") {
            let parts: Vec<&str> = trimmed.splitn(3, ' ').collect();
            if parts.len() >= 3 {
                let method = parts[1].to_uppercase();
                let url = parts[2].trim_matches('"').to_string();
                let mut block = Block::new(BlockType::HttpRequest);
                if let BlockSettings::HttpRequest(ref mut s) = block.settings {
                    s.method = method;
                    s.url = url;
                }
                blocks.push(block);
            }
        } else if trimmed.starts_with("KEYCHECK") || trimmed.starts_with("keycheck") {
            blocks.push(Block::new(BlockType::KeyCheck));
        } else if trimmed.starts_with("PARSE ") || trimmed.starts_with("parse ") {
            if trimmed.contains("LR") || trimmed.contains("lr") {
                blocks.push(Block::new(BlockType::ParseLR));
            } else if trimmed.contains("JSON") || trimmed.contains("json") {
                blocks.push(Block::new(BlockType::ParseJSON));
            } else if trimmed.contains("REGEX") || trimmed.contains("regex") {
                blocks.push(Block::new(BlockType::ParseRegex));
            } else if trimmed.contains("CSS") || trimmed.contains("css") {
                blocks.push(Block::new(BlockType::ParseCSS));
            }
        }
    }

    if blocks.is_empty() {
        return Err("No recognizable blocks found in LoliCode config".into());
    }

    pipeline.blocks = blocks;
    Ok(ImportResult { pipeline, warnings, security_issues: Vec::new() })
}

// ────────────────────────────────────────────────────────────
// SVB (SilverBullet / OpenBullet 1) importer
// ────────────────────────────────────────────────────────────

/// Import a SilverBullet .svb config (text file with [SETTINGS] + [SCRIPT] sections)
fn import_svb(content: &str) -> Result<ImportResult, String> {
    let mut pipeline = Pipeline::default();
    let mut warnings = Vec::new();

    // Split into [SETTINGS] and [SCRIPT] sections
    let script_start = content.find("[SCRIPT]");
    let settings_str = if let Some(script_pos) = script_start {
        let settings_block = &content[..script_pos];
        // Strip [SETTINGS] header
        settings_block.trim_start()
            .strip_prefix("[SETTINGS]")
            .unwrap_or(settings_block)
            .trim()
    } else {
        ""
    };
    let script_str = if let Some(script_pos) = script_start {
        &content[script_pos + 8..] // skip "[SCRIPT]"
    } else {
        return Err("No [SCRIPT] section found in .svb file".into());
    };

    // Parse settings JSON
    if !settings_str.is_empty() {
        apply_svb_settings(&mut pipeline, settings_str);
    }

    // Parse script
    pipeline.blocks = parse_svb_script(script_str, &mut warnings)?;

    if pipeline.blocks.is_empty() {
        return Err("No blocks found in SVB script".into());
    }

    Ok(ImportResult { pipeline, warnings, security_issues: Vec::new() })
}

/// Map SVB settings JSON fields to Pipeline settings
fn apply_svb_settings(pipeline: &mut Pipeline, json_str: &str) {
    let json: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return,
    };

    if let Some(name) = json.get("Name").and_then(|v| v.as_str()) {
        pipeline.name = name.trim().to_string();
    }
    if let Some(author) = json.get("Author").and_then(|v| v.as_str()) {
        if !author.is_empty() {
            pipeline.author = author.to_string();
        }
    }
    if let Some(bots) = json.get("SuggestedBots").and_then(|v| v.as_u64()) {
        if bots > 0 {
            pipeline.runner_settings.threads = bots as u32;
        }
    }
    if json.get("NeedsProxies").and_then(|v| v.as_bool()).unwrap_or(false) {
        pipeline.proxy_settings.proxy_mode = ProxyMode::Rotate;
    }

    // Wordlist type → data slices
    let wl = json.get("AllowedWordlist1").and_then(|v| v.as_str()).unwrap_or("");
    match wl {
        "MailPass" | "Credentials" | "" => {
            pipeline.data_settings.separator = ':';
            pipeline.data_settings.slices = vec!["USER".into(), "PASS".into()];
        }
        "Emails" => {
            pipeline.data_settings.separator = ':';
            pipeline.data_settings.slices = vec!["EMAIL".into()];
        }
        _ => {
            pipeline.data_settings.wordlist_type = wl.to_string();
        }
    }
}

/// Parse the [SCRIPT] section of a SVB file into blocks
fn parse_svb_script(content: &str, warnings: &mut Vec<String>) -> Result<Vec<Block>, String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut blocks = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let raw = lines[i];
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        // Skip indented lines that belong to a previous block (shouldn't happen at top level)
        if raw.starts_with(' ') || raw.starts_with('\t') {
            i += 1;
            continue;
        }

        // Parse optional #label or !#label prefix
        let (label, disabled, cmd) = parse_svb_prefix(trimmed);

        if cmd.starts_with("REQUEST ") {
            let (block, next_i) = parse_svb_request(&label, disabled, cmd, &lines, i + 1);
            blocks.push(block);
            i = next_i;
        } else if cmd == "KEYCHECK" || cmd.starts_with("KEYCHECK ") {
            let (block, next_i) = parse_svb_keycheck(&label, disabled, &lines, i + 1);
            blocks.push(block);
            i = next_i;
        } else if cmd.starts_with("PARSE ") {
            if let Some(block) = parse_svb_parse_line(&label, disabled, cmd) {
                blocks.push(block);
            }
            i += 1;
        } else if cmd.starts_with("FUNCTION ") {
            let (block_opt, next_i) = parse_svb_function(&label, disabled, cmd, &lines, i + 1, warnings);
            if let Some(block) = block_opt {
                blocks.push(block);
            }
            i = next_i;
        } else if cmd.starts_with("IF ") {
            let (block, next_i) = parse_svb_if(cmd, &lines, i + 1);
            blocks.push(block);
            i = next_i;
        } else if cmd.starts_with("SET ") {
            if let Some(block) = parse_svb_set(&label, disabled, cmd) {
                blocks.push(block);
            }
            i += 1;
        } else if cmd.starts_with("UTILITY ") {
            blocks.push(parse_svb_utility(&label, disabled, cmd));
            i += 1;
        } else {
            // Unknown top-level command
            let cmd_word = cmd.split_whitespace().next().unwrap_or(cmd);
            if cmd_word != "ELSE" && cmd_word != "ENDIF" {
                warnings.push(format!("Unknown SVB command: {}", cmd_word));
            }
            i += 1;
        }
    }

    Ok(blocks)
}

// ────────────────────────────────────────────────────────────
// SVB line prefix parser
// ────────────────────────────────────────────────────────────

/// Parse SVB line prefix: `#LABEL cmd...` or `!#LABEL cmd...` or plain `cmd...`
fn parse_svb_prefix(line: &str) -> (String, bool, &str) {
    if line.starts_with("!#") {
        let rest = &line[2..];
        if let Some(space_pos) = rest.find(' ') {
            let label = rest[..space_pos].to_string();
            let cmd = rest[space_pos + 1..].trim();
            (label, true, cmd)
        } else {
            (rest.to_string(), true, "")
        }
    } else if line.starts_with('#') {
        let rest = &line[1..];
        if let Some(space_pos) = rest.find(' ') {
            let label = rest[..space_pos].to_string();
            let cmd = rest[space_pos + 1..].trim();
            (label, false, cmd)
        } else {
            (rest.to_string(), false, "")
        }
    } else {
        (String::new(), false, line)
    }
}

// ────────────────────────────────────────────────────────────
// SVB REQUEST parser
// ────────────────────────────────────────────────────────────

fn parse_svb_request(label: &str, disabled: bool, cmd: &str, lines: &[&str], start: usize) -> (Block, usize) {
    let mut block = Block::new(BlockType::HttpRequest);
    if !label.is_empty() { block.label = label.to_string(); }
    block.disabled = disabled;

    // Parse "REQUEST METHOD "url" [AutoRedirect=FALSE]"
    let after_request = &cmd[8..]; // skip "REQUEST "
    let (method, rest) = after_request.split_once(' ').unwrap_or((after_request, ""));
    let (url_raw, _opts) = svb_extract_quoted(rest.trim());
    let auto_redirect = !cmd.contains("AutoRedirect=FALSE");

    if let BlockSettings::HttpRequest(ref mut s) = block.settings {
        s.method = method.to_string();
        s.url = convert_svb_var_refs(&url_raw);
        if !auto_redirect {
            s.follow_redirects = false;
            s.auto_redirect = false;
        }
    }

    // Collect following indented lines: CONTENT, CONTENTTYPE, HEADER
    let mut i = start;
    let mut headers: Vec<(String, String)> = Vec::new();
    let mut body = String::new();
    let mut content_type = String::new();

    while i < lines.len() {
        let raw = lines[i];
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            // Check if next non-empty line is indented (still part of this block)
            let mut j = i + 1;
            while j < lines.len() && lines[j].trim().is_empty() { j += 1; }
            if j < lines.len() && (lines[j].starts_with(' ') || lines[j].starts_with('\t')) {
                i += 1;
                continue;
            }
            // Blank line followed by non-indented = end of block
            i += 1;
            break;
        }

        // Non-indented line = new command
        if !raw.starts_with(' ') && !raw.starts_with('\t') {
            break;
        }

        if trimmed.starts_with("CONTENT ") {
            let (val, _) = svb_extract_quoted(&trimmed[8..]);
            body = convert_svb_var_refs(&val);
        } else if trimmed.starts_with("CONTENTTYPE ") {
            let (val, _) = svb_extract_quoted(&trimmed[12..]);
            content_type = val;
        } else if trimmed.starts_with("HEADER ") {
            let (val, _) = svb_extract_quoted(&trimmed[7..]);
            // Header format: "Key: Value" (split on first ": ")
            if let Some((k, v)) = val.split_once(": ") {
                headers.push((k.to_string(), convert_svb_var_refs(v)));
            } else if let Some((k, v)) = val.split_once(':') {
                headers.push((k.to_string(), convert_svb_var_refs(v.trim())));
            }
        }
        // Ignore unknown indented lines (Content-Length, etc.)

        i += 1;
    }

    if let BlockSettings::HttpRequest(ref mut s) = block.settings {
        if !body.is_empty() {
            s.body = body;
            s.body_type = BodyType::Standard;
        }
        if !content_type.is_empty() {
            s.content_type = content_type;
        }
        if !headers.is_empty() {
            s.headers = headers;
        }
    }

    (block, i)
}

// ────────────────────────────────────────────────────────────
// SVB KEYCHECK parser
// ────────────────────────────────────────────────────────────

fn parse_svb_keycheck(label: &str, disabled: bool, lines: &[&str], start: usize) -> (Block, usize) {
    let mut block = Block::new(BlockType::KeyCheck);
    if !label.is_empty() { block.label = label.to_string(); }
    block.disabled = disabled;

    if let BlockSettings::KeyCheck(ref mut s) = block.settings {
        s.keychains.clear();
        let mut current_keychain: Option<Keychain> = None;

        let mut i = start;
        while i < lines.len() {
            let raw = lines[i];
            let trimmed = raw.trim();

            if trimmed.is_empty() {
                // Check if next non-empty line is indented
                let mut j = i + 1;
                while j < lines.len() && lines[j].trim().is_empty() { j += 1; }
                if j < lines.len() && (lines[j].starts_with(' ') || lines[j].starts_with('\t')) {
                    i += 1;
                    continue;
                }
                i += 1;
                break;
            }

            if !raw.starts_with(' ') && !raw.starts_with('\t') {
                break;
            }

            if trimmed.starts_with("KEYCHAIN ") {
                // Save previous keychain
                if let Some(kc) = current_keychain.take() {
                    if !kc.conditions.is_empty() {
                        s.keychains.push(kc);
                    }
                }
                // Parse: "KEYCHAIN Status [Label] OR"
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                let status = match parts.get(1).copied().unwrap_or("") {
                    "Success" => BotStatus::Success,
                    "Failure" => BotStatus::Fail,
                    "Ban" => BotStatus::Ban,
                    "Retry" => BotStatus::Retry,
                    "Custom" => BotStatus::Custom,
                    _ => BotStatus::None,
                };
                current_keychain = Some(Keychain {
                    result: status,
                    conditions: Vec::new(),
                });
            } else if trimmed.starts_with("KEY ") {
                if let Some(ref mut kc) = current_keychain {
                    if let Some(cond) = parse_svb_key(&trimmed[4..]) {
                        kc.conditions.push(cond);
                    }
                }
            }

            i += 1;
        }

        if let Some(kc) = current_keychain {
            if !kc.conditions.is_empty() {
                s.keychains.push(kc);
            }
        }

        return (block, i);
    }

    (block, start)
}

/// Parse SVB KEY condition: `"value"` or `"<VAR>" Comparison "value"`
fn parse_svb_key(after_key: &str) -> Option<KeyCondition> {
    let after = after_key.trim();

    // Extract first quoted string
    let (first_val, rest) = svb_extract_quoted(after);
    if first_val.is_empty() && !after.starts_with('"') {
        return None;
    }
    let rest = rest.trim();

    // If rest is empty, it's a simple KEY "value" → Contains against SOURCE
    if rest.is_empty() {
        return Some(KeyCondition {
            source: "data.SOURCE".to_string(),
            comparison: Comparison::Contains,
            value: first_val,
        });
    }

    // Check if first_val is a <VAR> reference
    if first_val.starts_with('<') && first_val.ends_with('>') {
        let var_name = &first_val[1..first_val.len() - 1];
        let source = convert_svb_source_name(var_name);

        // Parse comparison and value
        let (comparison_str, value_part) = rest.split_once(' ').unwrap_or((rest, ""));
        let comparison = match comparison_str {
            "Contains" => Comparison::Contains,
            "DoesNotContain" => Comparison::NotContains,
            "EqualTo" | "Is" => Comparison::EqualTo,
            "NotEqualTo" | "IsNot" => Comparison::NotEqualTo,
            "MatchesRegex" => Comparison::MatchesRegex,
            "GreaterThan" => Comparison::GreaterThan,
            "LessThan" | "LessThanOrEqual" => Comparison::LessThan,
            "Exists" => Comparison::Exists,
            "DoesNotExist" => Comparison::NotExists,
            _ => Comparison::Contains,
        };
        let (value, _) = svb_extract_quoted(value_part.trim());
        return Some(KeyCondition { source, comparison, value });
    }

    // Not a var ref — simple KEY "value" with trailing junk
    Some(KeyCondition {
        source: "data.SOURCE".to_string(),
        comparison: Comparison::Contains,
        value: first_val,
    })
}

// ────────────────────────────────────────────────────────────
// SVB PARSE parser (single-line)
// ────────────────────────────────────────────────────────────

fn parse_svb_parse_line(label: &str, disabled: bool, cmd: &str) -> Option<Block> {
    let after = cmd.strip_prefix("PARSE ")?.trim();

    // Extract source (first quoted string)
    let (source_raw, rest) = svb_extract_quoted(after);
    let source = convert_svb_source_ref(&source_raw);
    let rest = rest.trim();

    if rest.starts_with("LR ") {
        let rest = &rest[3..];
        let (left, rest) = svb_extract_quoted(rest.trim());
        let (right, rest) = svb_extract_quoted(rest.trim());
        let (is_capture, output_var) = parse_svb_output(rest);

        let mut block = Block::new(BlockType::ParseLR);
        if !label.is_empty() { block.label = label.to_string(); }
        block.disabled = disabled;
        if let BlockSettings::ParseLR(ref mut s) = block.settings {
            s.input_var = source;
            s.left = left;
            s.right = right;
            s.output_var = output_var;
            s.capture = is_capture;
        }
        Some(block)
    } else if rest.starts_with("JSON ") {
        let rest = &rest[5..];
        let (json_path, rest) = svb_extract_quoted(rest.trim());
        let (is_capture, output_var) = parse_svb_output(rest);

        let mut block = Block::new(BlockType::ParseJSON);
        if !label.is_empty() { block.label = label.to_string(); }
        block.disabled = disabled;
        if let BlockSettings::ParseJSON(ref mut s) = block.settings {
            s.input_var = source;
            s.json_path = json_path;
            s.output_var = output_var;
            s.capture = is_capture;
        }
        Some(block)
    } else {
        // Unknown parse mode → Script
        let mut block = Block::new(BlockType::Script);
        block.label = if label.is_empty() { "Parse".into() } else { label.to_string() };
        block.disabled = disabled;
        if let BlockSettings::Script(ref mut s) = block.settings {
            s.code = format!("// SVB PARSE with unknown mode\n// {}", cmd);
        }
        Some(block)
    }
}

// ────────────────────────────────────────────────────────────
// SVB FUNCTION parser
// ────────────────────────────────────────────────────────────

fn parse_svb_function(
    label: &str, disabled: bool, cmd: &str, lines: &[&str], start: usize,
    warnings: &mut Vec<String>,
) -> (Option<Block>, usize) {
    let after = cmd.strip_prefix("FUNCTION ").unwrap_or("").trim();

    // GetRandomUA [BROWSER IOS] -> VAR "name"
    if after.starts_with("GetRandomUA") {
        let (_, output_var) = parse_svb_output(after);
        let mut block = Block::new(BlockType::RandomUserAgent);
        block.label = if label.is_empty() { "Random UA".into() } else { label.to_string() };
        block.disabled = disabled;
        if let BlockSettings::RandomUserAgent(ref mut s) = block.settings {
            s.output_var = output_var;
        }
        return (Some(block), start);
    }

    // RandomString "pattern" -> VAR "name"
    if after.starts_with("RandomString ") {
        let rest = &after[13..];
        let (pattern, rest) = svb_extract_quoted(rest.trim());
        let (_, output_var) = parse_svb_output(rest);

        let mut block = Block::new(BlockType::RandomData);
        block.label = if label.is_empty() { "Random String".into() } else { label.to_string() };
        block.disabled = disabled;
        if let BlockSettings::RandomData(ref mut s) = block.settings {
            s.data_type = RandomDataType::String;
            s.output_var = output_var;
            // Count pattern chars: ?u = upper, ?l = lower, ?d = digit, ?h = hex
            let char_count = pattern.matches('?').count();
            s.string_length = if char_count > 0 { char_count as u32 } else { pattern.len() as u32 };
            if pattern.contains("?h") {
                s.string_charset = "custom".into();
                s.custom_chars = "0123456789abcdef".into();
            } else {
                s.string_charset = "alphanumeric".into();
            }
        }
        return (Some(block), start);
    }

    // Constant "value" -> VAR/CAP "name"
    if after.starts_with("Constant ") {
        let rest = &after[9..];
        let (value, rest) = svb_extract_quoted(rest.trim());
        let (is_capture, output_var) = parse_svb_output(rest);

        let mut block = Block::new(BlockType::SetVariable);
        block.label = if label.is_empty() { "Constant".into() } else { label.to_string() };
        block.disabled = disabled;
        if let BlockSettings::SetVariable(ref mut s) = block.settings {
            s.name = output_var;
            s.value = convert_svb_var_refs(&value);
            s.capture = is_capture;
        }
        return (Some(block), start);
    }

    // ToUppercase/ToLowercase "input" -> VAR "name"
    if after.starts_with("ToUppercase ") || after.starts_with("ToLowercase ") {
        let is_upper = after.starts_with("ToUppercase");
        let rest = if is_upper { &after[12..] } else { &after[12..] };
        let (input_raw, rest) = svb_extract_quoted(rest.trim());
        let (is_capture, output_var) = parse_svb_output(rest);
        let input_var = extract_svb_simple_var(&input_raw);

        let fn_type = if is_upper { StringFnType::ToUpper } else { StringFnType::ToLower };
        let mut block = Block::new(BlockType::StringFunction);
        block.label = if label.is_empty() { format!("{:?}", fn_type) } else { label.to_string() };
        block.disabled = disabled;
        if let BlockSettings::StringFunction(ref mut s) = block.settings {
            s.function_type = fn_type;
            s.input_var = input_var;
            s.output_var = output_var;
            s.capture = is_capture;
        }
        return (Some(block), start);
    }

    // Replace "what" "with" "input" -> VAR/CAP "name"
    if after.starts_with("Replace ") {
        let rest = &after[8..];
        let (what, rest) = svb_extract_quoted(rest.trim());
        let (with, rest) = svb_extract_quoted(rest.trim());
        let (input_raw, rest) = svb_extract_quoted(rest.trim());
        let (is_capture, output_var) = parse_svb_output(rest);
        let input_var = extract_svb_simple_var(&input_raw);

        let mut block = Block::new(BlockType::StringFunction);
        block.label = if label.is_empty() { "Replace".into() } else { label.to_string() };
        block.disabled = disabled;
        if let BlockSettings::StringFunction(ref mut s) = block.settings {
            s.function_type = StringFnType::Replace;
            s.input_var = input_var;
            s.output_var = output_var;
            s.capture = is_capture;
            s.param1 = what;
            s.param2 = with;
        }
        return (Some(block), start);
    }

    // Split "separator" index "input" -> VAR "name"
    if after.starts_with("Split ") {
        let rest = &after[6..];
        let (sep, rest) = svb_extract_quoted(rest.trim());
        let rest = rest.trim();
        // Index is an unquoted number
        let (index_str, rest) = rest.split_once(' ').unwrap_or((rest, ""));
        let (input_raw, rest) = svb_extract_quoted(rest.trim());
        let (_, output_var) = parse_svb_output(rest);
        let input_var = extract_svb_simple_var(&input_raw);

        let mut block = Block::new(BlockType::Script);
        block.label = if label.is_empty() { "Split".into() } else { label.to_string() };
        block.disabled = disabled;
        if let BlockSettings::Script(ref mut s) = block.settings {
            s.code = format!(
                "// SVB Split: split <{}> by \"{}\" take index {}, store in {}\n// Implement in pipeline logic",
                input_var, sep, index_str, output_var
            );
        }
        return (Some(block), start);
    }

    // Unescape "input" -> VAR/CAP "name"
    if after.starts_with("Unescape ") {
        let rest = &after[9..];
        let (input_raw, rest) = svb_extract_quoted(rest.trim());
        let (is_capture, output_var) = parse_svb_output(rest);
        let input_var = extract_svb_simple_var(&input_raw);

        let mut block = Block::new(BlockType::Script);
        block.label = if label.is_empty() { "Unescape".into() } else { label.to_string() };
        block.disabled = disabled;
        if let BlockSettings::Script(ref mut s) = block.settings {
            s.code = format!(
                "// SVB Unescape: unescape <{}>, store as {} (capture={})\n// Implement HTML/Unicode unescape",
                input_var, output_var, is_capture
            );
        }
        return (Some(block), start);
    }

    // UnixTimeToDate "format" "input" -> VAR/CAP "name"
    if after.starts_with("UnixTimeToDate ") {
        let rest = &after[15..];
        let (format_str, rest) = svb_extract_quoted(rest.trim());
        let (input_raw, rest) = svb_extract_quoted(rest.trim());
        let (is_capture, output_var) = parse_svb_output(rest);
        let input_var = extract_svb_simple_var(&input_raw);

        let mut block = Block::new(BlockType::Script);
        block.label = if label.is_empty() { "Unix Time to Date".into() } else { label.to_string() };
        block.disabled = disabled;
        if let BlockSettings::Script(ref mut s) = block.settings {
            s.code = format!(
                "// SVB UnixTimeToDate: convert <{}> with format \"{}\" → {} (capture={})",
                input_var, format_str, output_var, is_capture
            );
        }
        return (Some(block), start);
    }

    // Translate (multi-line with KEY/VALUE pairs)
    if after.starts_with("Translate") {
        return parse_svb_translate(label, disabled, lines, start, warnings);
    }

    // Unknown function type
    let fn_name = after.split_whitespace().next().unwrap_or("unknown");
    warnings.push(format!("Unknown SVB function: {}", fn_name));
    let mut block = Block::new(BlockType::Script);
    block.label = if label.is_empty() { format!("SVB: {}", fn_name) } else { label.to_string() };
    block.disabled = disabled;
    if let BlockSettings::Script(ref mut s) = block.settings {
        s.code = format!("// Unknown SVB FUNCTION: {}", after);
    }
    (Some(block), start)
}

/// Parse SVB FUNCTION Translate (multi-line KEY/VALUE lookup table)
fn parse_svb_translate(
    label: &str, disabled: bool, lines: &[&str], start: usize,
    _warnings: &mut Vec<String>,
) -> (Option<Block>, usize) {
    let mut i = start;
    let mut entries = Vec::new();
    let mut input_var = String::new();
    let mut output_var = "RESULT".to_string();

    // Collect KEY "k" VALUE "v" lines and the final "input" -> VAR "name" line
    while i < lines.len() {
        let raw = lines[i];
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            let mut j = i + 1;
            while j < lines.len() && lines[j].trim().is_empty() { j += 1; }
            if j < lines.len() && (lines[j].starts_with(' ') || lines[j].starts_with('\t')) {
                i += 1;
                continue;
            }
            i += 1;
            break;
        }
        if !raw.starts_with(' ') && !raw.starts_with('\t') {
            break;
        }

        if trimmed.starts_with("KEY ") {
            // KEY "k" VALUE "v"
            let rest = &trimmed[4..];
            let (key, rest) = svb_extract_quoted(rest.trim());
            let rest = rest.trim();
            if rest.starts_with("VALUE ") {
                let (val, _) = svb_extract_quoted(&rest[6..]);
                entries.push((key, val));
            }
        } else if trimmed.contains("-> VAR ") || trimmed.contains("-> CAP ") {
            // Final line: "input" -> VAR "name"
            let (input_raw, _) = svb_extract_quoted(trimmed);
            input_var = extract_svb_simple_var(&input_raw);
            let (_, out) = parse_svb_output(trimmed);
            output_var = out;
            i += 1;
            break;
        }

        i += 1;
    }

    let mut block = Block::new(BlockType::Script);
    block.label = if label.is_empty() { "Translate".into() } else { label.to_string() };
    block.disabled = disabled;
    if let BlockSettings::Script(ref mut s) = block.settings {
        let table = entries.iter()
            .map(|(k, v)| format!("//   \"{}\" => \"{}\"", k, v))
            .collect::<Vec<_>>()
            .join("\n");
        s.code = format!(
            "// SVB Translate: lookup table on <{}> → {}\n// Entries ({}):\n{}",
            input_var, output_var, entries.len(), table
        );
    }

    (Some(block), i)
}

// ────────────────────────────────────────────────────────────
// SVB IF/ELSE/ENDIF parser
// ────────────────────────────────────────────────────────────

fn parse_svb_if(cmd: &str, lines: &[&str], start: usize) -> (Block, usize) {
    // Parse: IF "<VAR>" CONTAINS "value"
    let mut block = Block::new(BlockType::IfElse);
    block.label = "If / Else".to_string();

    // Extract condition from the IF line
    let after_if = cmd.strip_prefix("IF ").unwrap_or("").trim();
    let (source_raw, rest) = svb_extract_quoted(after_if);
    let source_var = extract_svb_simple_var(&source_raw);
    let rest = rest.trim();
    let (comparison_str, value_part) = rest.split_once(' ').unwrap_or((rest, ""));
    let comparison = match comparison_str {
        "CONTAINS" | "Contains" => Comparison::Contains,
        "DOESNOTCONTAIN" | "DoesNotContain" => Comparison::NotContains,
        "EQUALTO" | "EqualTo" | "IS" => Comparison::EqualTo,
        "NOTEQUALTO" | "NotEqualTo" => Comparison::NotEqualTo,
        _ => Comparison::Contains,
    };
    let (value, _) = svb_extract_quoted(value_part.trim());

    if let BlockSettings::IfElse(ref mut s) = block.settings {
        s.condition = KeyCondition {
            source: source_var,
            comparison,
            value,
        };
    }

    // Collect true/false branch lines (as inner blocks)
    let mut true_blocks = Vec::new();
    let mut false_blocks = Vec::new();
    let mut in_else = false;
    let mut i = start;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        // Parse prefix for inner lines too
        let (inner_label, inner_disabled, inner_cmd) = parse_svb_prefix(trimmed);

        if inner_cmd == "ENDIF" {
            i += 1;
            break;
        }
        if inner_cmd == "ELSE" {
            in_else = true;
            i += 1;
            continue;
        }
        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        // Parse inner command as a block
        if inner_cmd.starts_with("SET ") {
            if let Some(inner_block) = parse_svb_set(&inner_label, inner_disabled, inner_cmd) {
                if in_else { false_blocks.push(inner_block); }
                else { true_blocks.push(inner_block); }
            }
        } else if inner_cmd.starts_with("FUNCTION ") {
            // Simple inline function (no multi-line in IF blocks)
            let mut dummy_warnings = Vec::new();
            let (block_opt, _) = parse_svb_function(&inner_label, inner_disabled, inner_cmd, lines, i + 1, &mut dummy_warnings);
            if let Some(inner_block) = block_opt {
                if in_else { false_blocks.push(inner_block); }
                else { true_blocks.push(inner_block); }
            }
        } else {
            // Generic inner content → Script
            let mut inner_block = Block::new(BlockType::Script);
            inner_block.label = if inner_label.is_empty() { "Script".into() } else { inner_label };
            inner_block.disabled = inner_disabled;
            if let BlockSettings::Script(ref mut s) = inner_block.settings {
                s.code = format!("// {}", inner_cmd);
            }
            if in_else { false_blocks.push(inner_block); }
            else { true_blocks.push(inner_block); }
        }

        i += 1;
    }

    if let BlockSettings::IfElse(ref mut s) = block.settings {
        s.true_blocks = true_blocks;
        s.false_blocks = false_blocks;
    }

    (block, i)
}

// ────────────────────────────────────────────────────────────
// SVB SET parser
// ────────────────────────────────────────────────────────────

/// Parse: SET CAP "name" VAR "value" or SET VAR "name" VAR "value"
fn parse_svb_set(label: &str, disabled: bool, cmd: &str) -> Option<Block> {
    let after = cmd.strip_prefix("SET ")?.trim();

    let is_capture = after.starts_with("CAP ");
    let rest = if is_capture {
        &after[4..]
    } else if after.starts_with("VAR ") {
        &after[4..]
    } else {
        after
    };

    let (name, rest) = svb_extract_quoted(rest.trim());
    let rest = rest.trim();
    // Skip "VAR" keyword if present
    let rest = rest.strip_prefix("VAR ").unwrap_or(rest).trim();
    let (value, _) = svb_extract_quoted(rest);

    let mut block = Block::new(BlockType::SetVariable);
    block.label = if label.is_empty() { "Set Variable".into() } else { label.to_string() };
    block.disabled = disabled;
    if let BlockSettings::SetVariable(ref mut s) = block.settings {
        s.name = name;
        s.value = convert_svb_var_refs(&value);
        s.capture = is_capture;
    }
    Some(block)
}

// ────────────────────────────────────────────────────────────
// SVB UTILITY parser
// ────────────────────────────────────────────────────────────

fn parse_svb_utility(label: &str, disabled: bool, cmd: &str) -> Block {
    let mut block = Block::new(BlockType::Script);
    block.label = if label.is_empty() { "Utility".into() } else { label.to_string() };
    block.disabled = disabled;
    if let BlockSettings::Script(ref mut s) = block.settings {
        s.code = format!("// SVB UTILITY — review and implement manually\n// {}", cmd);
    }
    block
}

// ────────────────────────────────────────────────────────────
// SVB helpers
// ────────────────────────────────────────────────────────────

/// Extract next quoted string from SVB text. Returns (extracted_string, rest_of_input)
fn svb_extract_quoted(s: &str) -> (String, &str) {
    let s = s.trim();
    if !s.starts_with('"') {
        return (String::new(), s);
    }
    let bytes = s.as_bytes();
    let mut i = 1;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            i += 2;
            continue;
        }
        if bytes[i] == b'"' {
            let inner = &s[1..i];
            let rest = &s[i + 1..];
            return (inner.replace("\\\"", "\""), rest);
        }
        i += 1;
    }
    // No closing quote
    (s[1..].to_string(), "")
}

/// Convert SVB source name to reqflow variable name
/// SOURCE → data.SOURCE, RESPONSECODE → data.RESPONSECODE, other → as-is
fn convert_svb_source_name(name: &str) -> String {
    match name {
        "SOURCE" => "data.SOURCE".to_string(),
        "RESPONSECODE" => "data.RESPONSECODE".to_string(),
        "ADDRESS" => "data.ADDRESS".to_string(),
        _ => name.to_string(),
    }
}

/// Convert SVB source reference (from PARSE) to reqflow variable name
/// `<SOURCE>` → data.SOURCE, `<COOKIES(name)>` → data.COOKIES["name"], `<varName>` → varName
fn convert_svb_source_ref(source: &str) -> String {
    if source == "<SOURCE>" { return "data.SOURCE".to_string(); }
    if source == "<RESPONSECODE>" { return "data.RESPONSECODE".to_string(); }

    // <COOKIES(name)> → data.COOKIES["name"]
    if source.starts_with("<COOKIES(") && source.ends_with(")>") {
        let name = &source[9..source.len() - 2];
        return format!("data.COOKIES[\"{}\"]", name);
    }

    // <HEADERS(name)> → data.HEADERS["name"]
    if source.starts_with("<HEADERS(") && source.ends_with(")>") {
        let name = &source[9..source.len() - 2];
        return format!("data.HEADERS[\"{}\"]", name);
    }

    // <varName> → varName
    if source.starts_with('<') && source.ends_with('>') {
        return source[1..source.len() - 1].to_string();
    }

    source.to_string()
}

/// Convert SVB data variable references: <USER> → <input.USER>, <PASS> → <input.PASS>
fn convert_svb_var_refs(text: &str) -> String {
    text.replace("<USER>", "<input.USER>")
        .replace("<PASS>", "<input.PASS>")
        .replace("<EMAIL>", "<input.EMAIL>")
}

/// Extract simple variable name from SVB input string
/// `<varName>` → varName, `<input.USER>` → input.USER, other → as-is
fn extract_svb_simple_var(input: &str) -> String {
    let input = convert_svb_var_refs(input);
    if input.starts_with('<') && input.ends_with('>')
        && !input[1..input.len()-1].contains('<')
    {
        input[1..input.len()-1].to_string()
    } else {
        input
    }
}

/// Parse SVB output: `-> VAR "name"` or `-> CAP "name"` from a string
/// Returns (is_capture, variable_name)
fn parse_svb_output(s: &str) -> (bool, String) {
    if let Some(pos) = s.find("-> VAR ") {
        let rest = &s[pos + 7..];
        let (name, _) = svb_extract_quoted(rest.trim());
        return (false, if name.is_empty() { "RESULT".into() } else { name });
    }
    if let Some(pos) = s.find("-> CAP ") {
        let rest = &s[pos + 7..];
        let (name, _) = svb_extract_quoted(rest.trim());
        return (true, if name.is_empty() { "RESULT".into() } else { name });
    }
    (false, "RESULT".to_string())
}

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

/// Extract the domain from a URL
fn extract_domain(url: &str) -> Option<String> {
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
fn extract_host(url: &str) -> Option<String> {
    extract_domain(url)
}

/// Truncate code to show context around a matched pattern
fn truncate_code(code: &str, pattern: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lolicode_blocks_http_and_keycheck() {
        let script = r#"
BLOCK:HttpRequest
LABEL:Login
  url = "https://example.com/login"
  method = POST
  TYPE:STANDARD
  $"username=<input.USER>&password=<input.PASS>"
  "application/x-www-form-urlencoded"
ENDBLOCK

BLOCK:Keycheck
  KEYCHAIN SUCCESS OR
    STRINGKEY @data.SOURCE Contains "Welcome"
  KEYCHAIN FAIL OR
    STRINGKEY @data.SOURCE Contains "Invalid"
ENDBLOCK

BLOCK:Parse
LABEL:Get Token
  input = @data.SOURCE
  jToken = "token"
  MODE:Json
  => VAR @authToken
ENDBLOCK
"#;
        let mut warnings = Vec::new();
        let blocks = parse_lolicode_blocks(script, &mut warnings).unwrap();
        assert_eq!(blocks.len(), 3);

        // HTTP Request
        assert_eq!(blocks[0].label, "Login");
        if let BlockSettings::HttpRequest(ref s) = blocks[0].settings {
            assert_eq!(s.url, "https://example.com/login");
            assert_eq!(s.method, "POST");
            assert!(s.body.contains("username=<input.USER>"));
            assert_eq!(s.content_type, "application/x-www-form-urlencoded");
        } else {
            panic!("Expected HttpRequest");
        }

        // Keycheck
        if let BlockSettings::KeyCheck(ref s) = blocks[1].settings {
            assert_eq!(s.keychains.len(), 2);
            assert_eq!(s.keychains[0].result, BotStatus::Success);
            assert_eq!(s.keychains[0].conditions[0].value, "Welcome");
            assert_eq!(s.keychains[1].result, BotStatus::Fail);
        } else {
            panic!("Expected KeyCheck");
        }

        // Parse JSON
        assert_eq!(blocks[2].label, "Get Token");
        if let BlockSettings::ParseJSON(ref s) = blocks[2].settings {
            assert_eq!(s.input_var, "data.SOURCE");
            assert_eq!(s.json_path, "token");
            assert_eq!(s.output_var, "authToken");
            assert!(!s.capture);
        } else {
            panic!("Expected ParseJSON");
        }
    }

    #[test]
    fn test_parse_lr_with_capture() {
        let script = r#"
BLOCK:Parse
LABEL:Get Email
  input = @data.SOURCE
  leftDelim = "email\":\""
  rightDelim = "\""
  MODE:LR
  => CAP @email
ENDBLOCK
"#;
        let mut warnings = Vec::new();
        let blocks = parse_lolicode_blocks(script, &mut warnings).unwrap();
        assert_eq!(blocks.len(), 1);

        if let BlockSettings::ParseLR(ref s) = blocks[0].settings {
            assert_eq!(s.left, "email\":\"");
            assert_eq!(s.right, "\"");
            assert_eq!(s.output_var, "email");
            assert!(s.capture);
        } else {
            panic!("Expected ParseLR");
        }
    }

    #[test]
    fn test_constant_string_and_random() {
        let script = r#"
BLOCK:ConstantString
LABEL:Author
  value = "TestAuthor"
  => CAP @author
ENDBLOCK

BLOCK:RandomString
  input = "?m?m?m?m-?m?m?m?m"
  => VAR @deviceId
ENDBLOCK
"#;
        let mut warnings = Vec::new();
        let blocks = parse_lolicode_blocks(script, &mut warnings).unwrap();
        assert_eq!(blocks.len(), 2);

        // ConstantString → SetVariable
        if let BlockSettings::SetVariable(ref s) = blocks[0].settings {
            assert_eq!(s.name, "author");
            assert_eq!(s.value, "TestAuthor");
            assert!(s.capture);
        } else {
            panic!("Expected SetVariable");
        }

        // RandomString → RandomData
        if let BlockSettings::RandomData(ref s) = blocks[1].settings {
            assert_eq!(s.output_var, "deviceId");
            assert_eq!(s.string_length, 8); // 8 hex chars
            assert_eq!(s.custom_chars, "0123456789abcdef");
        } else {
            panic!("Expected RandomData");
        }
    }

    #[test]
    fn test_xtp_proxy_headers_extracted() {
        let script = r#"
BLOCK:HttpRequest
LABEL:Auth
  url = "http://localhost:9000"
  method = POST
  customHeaders = ${("x-tp-url", "https://api.example.com/token"), ("x-tp-method", "POST"), ("x-tp-chid", "Chrome_123"), ("Content-Type", "application/json"), ("User-Agent", "MyApp/1.0")}
  TYPE:STANDARD
  $"grant_type=password"
  "application/json"
ENDBLOCK
"#;
        let mut warnings = Vec::new();
        let blocks = parse_lolicode_blocks(script, &mut warnings).unwrap();
        assert_eq!(blocks.len(), 1);

        if let BlockSettings::HttpRequest(ref s) = blocks[0].settings {
            // x-tp-url should be the actual URL
            assert_eq!(s.url, "https://api.example.com/token");
            assert_eq!(s.method, "POST");
            // x-tp-* headers should be stripped, only real headers remain
            assert_eq!(s.headers.len(), 2);
            assert_eq!(s.headers[0].0, "Content-Type");
            assert_eq!(s.headers[1].0, "User-Agent");
            assert_eq!(s.body, "grant_type=password");
        } else {
            panic!("Expected HttpRequest");
        }
    }

    #[test]
    fn test_preamble_becomes_script_block() {
        let script = r#"
string RealProxy = ConstantString(data, "");
data.UseProxy = false;

BLOCK:HttpRequest
  url = "https://example.com"
ENDBLOCK
"#;
        let mut warnings = Vec::new();
        let blocks = parse_lolicode_blocks(script, &mut warnings).unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].label, "OB2 Preamble (C#)");

        if let BlockSettings::Script(ref s) = blocks[0].settings {
            assert!(s.code.contains("ConstantString"));
        } else {
            panic!("Expected Script for preamble");
        }
    }

    #[test]
    fn test_opk_psn_full_import() {
        let path = "data/OB2/psn.opk";
        if let Ok(bytes) = std::fs::read(path) {
            let pipeline = import_config_bytes(&bytes).unwrap().pipeline;
            assert!(!pipeline.name.is_empty());
            assert!(pipeline.blocks.len() >= 10, "PSN should have 10+ blocks, got {}", pipeline.blocks.len());

            // First HTTP block: should have autoRedirect = False
            let http0 = pipeline.blocks.iter().find(|b| matches!(b.settings, BlockSettings::HttpRequest(_))).unwrap();
            if let BlockSettings::HttpRequest(ref s) = http0.settings {
                assert!(!s.url.is_empty(), "PSN first HTTP URL should be populated");
                assert!(!s.follow_redirects, "PSN first HTTP should have follow_redirects=false");
            }

            // Should have Parse blocks reading cookies
            let cookie_parse = pipeline.blocks.iter().find(|b| {
                if let BlockSettings::ParseLR(ref s) = b.settings {
                    s.input_var.contains("COOKIES")
                } else { false }
            });
            assert!(cookie_parse.is_some(), "PSN should have a Parse block reading cookies");

            // Second HTTP block should have url = @Linked1 → <Linked1>
            let http_blocks: Vec<_> = pipeline.blocks.iter().filter(|b| matches!(b.settings, BlockSettings::HttpRequest(_))).collect();
            if http_blocks.len() >= 2 {
                if let BlockSettings::HttpRequest(ref s) = http_blocks[1].settings {
                    assert!(s.url.contains('<'), "PSN second HTTP URL should be a variable ref <Linked1>, got: {}", s.url);
                }
            }

            // HTTP blocks with {(...)} headers should have them parsed
            // (block 2 with @Linked1 has sony.com headers, block 3 ssocookie has headers)
            let http_with_many_headers = pipeline.blocks.iter().find(|b| {
                if let BlockSettings::HttpRequest(ref s) = b.settings {
                    s.headers.len() > 5
                } else { false }
            });
            assert!(http_with_many_headers.is_some(), "PSN should have at least one HTTP block with parsed brace-style headers");
        }
    }

    #[test]
    fn test_opk_hotmail_full_import() {
        let path = "data/OB2/HOTMAIL X PAYPAL.opk";
        if let Ok(bytes) = std::fs::read(path) {
            let pipeline = import_config_bytes(&bytes).unwrap().pipeline;
            assert!(pipeline.blocks.len() >= 20, "HOTMAIL should have 20+ blocks, got {}", pipeline.blocks.len());

            // Should have UrlEncode blocks with input from $"<input.USER>"
            let url_encode = pipeline.blocks.iter().find(|b| {
                if let BlockSettings::StringFunction(ref s) = b.settings {
                    matches!(s.function_type, StringFnType::URLEncode) && !s.input_var.is_empty()
                } else { false }
            });
            assert!(url_encode.is_some(), "HOTMAIL should have UrlEncode with populated input");
            if let Some(block) = url_encode {
                if let BlockSettings::StringFunction(ref s) = block.settings {
                    assert_eq!(s.input_var, "input.USER", "Should extract variable from $\"<input.USER>\"");
                }
            }

            // HTTP blocks should have body populated
            let http_with_body = pipeline.blocks.iter().find(|b| {
                if let BlockSettings::HttpRequest(ref s) = b.settings {
                    !s.body.is_empty() && s.body.contains("login")
                } else { false }
            });
            assert!(http_with_body.is_some(), "HOTMAIL should have HTTP block with body containing login data");

            // HTTP blocks should have headers from {(...)} format
            let http_with_headers = pipeline.blocks.iter().find(|b| {
                if let BlockSettings::HttpRequest(ref s) = b.settings {
                    s.headers.len() > 5
                } else { false }
            });
            assert!(http_with_headers.is_some(), "HOTMAIL should have HTTP block with many headers");

            // Should have disabled blocks
            let disabled_count = pipeline.blocks.iter().filter(|b| b.disabled).count();
            assert!(disabled_count >= 1, "HOTMAIL should have at least 1 disabled block");
        }
    }

    #[test]
    fn test_opk_paramount_full_import() {
        let path = "data/OB2/PARAMOUNT+ TLS.opk";
        if let Ok(bytes) = std::fs::read(path) {
            let pipeline = import_config_bytes(&bytes).unwrap().pipeline;
            assert!(pipeline.blocks.len() >= 10, "PARAMOUNT should have 10+ blocks, got {}", pipeline.blocks.len());

            // Should have x-url proxy pattern extracted
            let http_blocks: Vec<_> = pipeline.blocks.iter().filter(|b| matches!(b.settings, BlockSettings::HttpRequest(_))).collect();
            assert!(!http_blocks.is_empty(), "PARAMOUNT should have HTTP blocks");

            for block in &http_blocks {
                if let BlockSettings::HttpRequest(ref s) = block.settings {
                    // x-url blocks should have the real URL, not localhost
                    assert!(!s.url.contains("localhost"), "PARAMOUNT HTTP URL should not be localhost after x-url extraction, got: {}", s.url);
                    // x-url, x-proxy headers should be stripped
                    assert!(!s.headers.iter().any(|(k, _)| k == "x-url"), "x-url header should be stripped");
                    assert!(!s.headers.iter().any(|(k, _)| k == "x-proxy"), "x-proxy header should be stripped");
                    // Body should be populated for POST requests
                    if s.method == "POST" {
                        assert!(!matches!(s.body_type, BodyType::None) || !s.body.is_empty(), "POST block should have body");
                    }
                }
            }
        }
    }

    #[test]
    fn test_opk_payback_full_import() {
        let path = "data/OB2/PAYBACK.DE LEAK.opk";
        if let Ok(bytes) = std::fs::read(path) {
            let pipeline = import_config_bytes(&bytes).unwrap().pipeline;
            assert!(pipeline.blocks.len() >= 15, "PAYBACK should have 15+ blocks, got {}", pipeline.blocks.len());

            // Should have GetRandomItem blocks with inline lists
            let list_blocks: Vec<_> = pipeline.blocks.iter().filter(|b| {
                if let BlockSettings::ListFunction(ref s) = b.settings {
                    matches!(s.function_type, ListFnType::RandomItem) && !s.param1.is_empty()
                } else { false }
            }).collect();
            assert!(!list_blocks.is_empty(), "PAYBACK should have GetRandomItem with inline list data");

            // Should have HTTP blocks with body and headers
            let http_with_body = pipeline.blocks.iter().find(|b| {
                if let BlockSettings::HttpRequest(ref s) = b.settings {
                    !s.body.is_empty() && !s.headers.is_empty()
                } else { false }
            });
            assert!(http_with_body.is_some(), "PAYBACK should have HTTP blocks with both body and headers");

            // Should have autoRedirect = False on at least one HTTP block
            let no_redirect = pipeline.blocks.iter().find(|b| {
                if let BlockSettings::HttpRequest(ref s) = b.settings {
                    !s.follow_redirects
                } else { false }
            });
            assert!(no_redirect.is_some(), "PAYBACK should have at least one HTTP block with follow_redirects=false");

            // Should have Parse JSON blocks with populated jToken paths
            let json_parse = pipeline.blocks.iter().find(|b| {
                if let BlockSettings::ParseJSON(ref s) = b.settings {
                    !s.json_path.is_empty()
                } else { false }
            });
            assert!(json_parse.is_some(), "PAYBACK should have ParseJSON blocks with populated json_path");

            // Should have ConstantString with $"..." interpolated values
            let interp_const = pipeline.blocks.iter().find(|b| {
                if let BlockSettings::SetVariable(ref s) = b.settings {
                    s.value.contains('<') && s.value.contains('>')
                } else { false }
            });
            assert!(interp_const.is_some(), "PAYBACK should have SetVariable with interpolated values");
        }
    }

    #[test]
    fn test_string_fn_blocks() {
        let script = r#"
BLOCK:UrlEncode
  input = @myVar
  => VAR @encoded
ENDBLOCK

BLOCK:ClearCookies
ENDBLOCK
"#;
        let mut warnings = Vec::new();
        let blocks = parse_lolicode_blocks(script, &mut warnings).unwrap();
        assert_eq!(blocks.len(), 2);

        if let BlockSettings::StringFunction(ref s) = blocks[0].settings {
            assert!(matches!(s.function_type, StringFnType::URLEncode));
            assert_eq!(s.input_var, "myVar");
            assert_eq!(s.output_var, "encoded");
        } else {
            panic!("Expected StringFunction");
        }

        assert!(matches!(blocks[1].settings, BlockSettings::ClearCookies));
    }

    #[test]
    fn test_custom_headers_parsing() {
        // ${...} format
        let line = r#"customHeaders = ${("Host", "example.com"), ("X-Custom", "value with spaces")}"#;
        let headers = parse_custom_headers(line);
        assert_eq!(headers.len(), 2);
        assert_eq!(headers[0], ("Host".to_string(), "example.com".to_string()));
        assert_eq!(headers[1], ("X-Custom".to_string(), "value with spaces".to_string()));

        // {...} format (without $ prefix — PSN/HOTMAIL style)
        let line2 = r#"customHeaders = {("Accept", "text/html"), ("Host", "login.live.com")}"#;
        let headers2 = parse_custom_headers(line2);
        assert_eq!(headers2.len(), 2);
        assert_eq!(headers2[0], ("Accept".to_string(), "text/html".to_string()));
        assert_eq!(headers2[1], ("Host".to_string(), "login.live.com".to_string()));
    }

    #[test]
    fn test_stringkey_parsing() {
        let cond = parse_stringkey(r#"STRINGKEY @data.SOURCE Contains "access_token""#).unwrap();
        assert_eq!(cond.source, "data.SOURCE");
        assert!(matches!(cond.comparison, Comparison::Contains));
        assert_eq!(cond.value, "access_token");

        let cond2 = parse_stringkey(r#"STRINGKEY @PLAN DoesNotContain "free""#).unwrap();
        assert_eq!(cond2.source, "PLAN");
        assert!(matches!(cond2.comparison, Comparison::NotContains));
        assert_eq!(cond2.value, "free");
    }

    #[test]
    fn test_auto_redirect_false() {
        let script = r#"
BLOCK:HttpRequest
LABEL:No Redirect
  url = "https://example.com/redirect"
  autoRedirect = False
  TYPE:STANDARD
  $""
  "application/x-www-form-urlencoded"
ENDBLOCK
"#;
        let mut warnings = Vec::new();
        let blocks = parse_lolicode_blocks(script, &mut warnings).unwrap();
        assert_eq!(blocks.len(), 1);
        if let BlockSettings::HttpRequest(ref s) = blocks[0].settings {
            assert_eq!(s.url, "https://example.com/redirect");
            assert!(!s.follow_redirects);
            assert!(!s.auto_redirect);
            assert!(s.body.is_empty()); // $"" = empty body
        } else {
            panic!("Expected HttpRequest");
        }
    }

    #[test]
    fn test_url_variable_reference() {
        let script = r#"
BLOCK:HttpRequest
  url = @myRedirectUrl
  customHeaders = {("Host", "example.com"), ("Accept", "*/*")}
  TYPE:STANDARD
  $""
  "text/html"
ENDBLOCK
"#;
        let mut warnings = Vec::new();
        let blocks = parse_lolicode_blocks(script, &mut warnings).unwrap();
        assert_eq!(blocks.len(), 1);
        if let BlockSettings::HttpRequest(ref s) = blocks[0].settings {
            assert_eq!(s.url, "<myRedirectUrl>");
            assert_eq!(s.headers.len(), 2);
            assert_eq!(s.headers[0].0, "Host");
        } else {
            panic!("Expected HttpRequest");
        }
    }

    #[test]
    fn test_interpolated_input_string_fn() {
        let script = r#"
BLOCK:UrlEncode
  input = $"<input.USER>"
  => VAR @encoded
ENDBLOCK
"#;
        let mut warnings = Vec::new();
        let blocks = parse_lolicode_blocks(script, &mut warnings).unwrap();
        assert_eq!(blocks.len(), 1);
        if let BlockSettings::StringFunction(ref s) = blocks[0].settings {
            assert_eq!(s.input_var, "input.USER"); // $"<input.USER>" → input.USER
            assert_eq!(s.output_var, "encoded");
        } else {
            panic!("Expected StringFunction");
        }
    }

    #[test]
    fn test_cookie_header_indexed_input() {
        let script = r#"
BLOCK:Parse
LABEL:Get Cookie
  input = @data.COOKIES["session_id"]
  MODE:LR
  => VAR @sessionCookie
ENDBLOCK

BLOCK:Parse
LABEL:Get Header
  input = @data.HEADERS["Location"]
  MODE:LR
  => VAR @redirectUrl
ENDBLOCK

BLOCK:Parse
LABEL:Get Address
  input = @data.ADDRESS
  leftDelim = "code="
  rightDelim = "&"
  MODE:LR
  => VAR @authCode
ENDBLOCK
"#;
        let mut warnings = Vec::new();
        let blocks = parse_lolicode_blocks(script, &mut warnings).unwrap();
        assert_eq!(blocks.len(), 3);

        if let BlockSettings::ParseLR(ref s) = blocks[0].settings {
            assert_eq!(s.input_var, "data.COOKIES[\"session_id\"]");
            assert_eq!(s.output_var, "sessionCookie");
        } else {
            panic!("Expected ParseLR for cookie");
        }

        if let BlockSettings::ParseLR(ref s) = blocks[1].settings {
            assert_eq!(s.input_var, "data.HEADERS[\"Location\"]");
            assert_eq!(s.output_var, "redirectUrl");
        } else {
            panic!("Expected ParseLR for header");
        }

        if let BlockSettings::ParseLR(ref s) = blocks[2].settings {
            assert_eq!(s.input_var, "data.ADDRESS");
            assert_eq!(s.left, "code=");
            assert_eq!(s.right, "&");
            assert_eq!(s.output_var, "authCode");
        } else {
            panic!("Expected ParseLR for address");
        }
    }

    #[test]
    fn test_inline_list_get_random_item() {
        let script = r#"
BLOCK:GetRandomItem
LABEL:Pick Device
  list = ["iPhone", "Android", "iPad"]
  => VAR @device
ENDBLOCK
"#;
        let mut warnings = Vec::new();
        let blocks = parse_lolicode_blocks(script, &mut warnings).unwrap();
        assert_eq!(blocks.len(), 1);
        if let BlockSettings::ListFunction(ref s) = blocks[0].settings {
            assert!(matches!(s.function_type, ListFnType::RandomItem));
            assert_eq!(s.output_var, "device");
            assert!(s.param1.contains("iPhone"));
            assert!(s.param1.contains("Android"));
        } else {
            panic!("Expected ListFunction");
        }
    }

    #[test]
    fn test_disabled_block() {
        let script = r#"
BLOCK:Parse
DISABLED
LABEL:Skipped
  input = @data.SOURCE
  leftDelim = "test"
  rightDelim = "end"
  MODE:LR
  => VAR @result
ENDBLOCK
"#;
        let mut warnings = Vec::new();
        let blocks = parse_lolicode_blocks(script, &mut warnings).unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].disabled);
        assert_eq!(blocks[0].label, "Skipped");
    }

    #[test]
    fn test_interpolated_constant_string() {
        let script = r#"
BLOCK:ConstantString
  value = $"<firstName> <lastName>"
  => CAP @fullName
ENDBLOCK
"#;
        let mut warnings = Vec::new();
        let blocks = parse_lolicode_blocks(script, &mut warnings).unwrap();
        assert_eq!(blocks.len(), 1);
        if let BlockSettings::SetVariable(ref s) = blocks[0].settings {
            assert_eq!(s.value, "<firstName> <lastName>");
            assert_eq!(s.name, "fullName");
            assert!(s.capture);
        } else {
            panic!("Expected SetVariable");
        }
    }

    #[test]
    fn test_x_url_proxy_pattern() {
        let script = r#"
BLOCK:HttpRequest
  url = "http://localhost:2024"
  method = POST
  customHeaders = ${("host", "www.example.com"), ("user-agent", "Mozilla/5.0"), ("x-proxy", "<proxy>"), ("x-url", "https://www.example.com/login"), ("x-identifier", "chrome"), ("x-session-id", "<guid>")}
  TYPE:STANDARD
  $"user=test&pass=test"
  "application/x-www-form-urlencoded"
ENDBLOCK
"#;
        let mut warnings = Vec::new();
        let blocks = parse_lolicode_blocks(script, &mut warnings).unwrap();
        assert_eq!(blocks.len(), 1);
        if let BlockSettings::HttpRequest(ref s) = blocks[0].settings {
            // x-url should become the real URL
            assert_eq!(s.url, "https://www.example.com/login");
            // x-url, x-proxy, x-identifier, x-session-id should be stripped
            assert!(!s.headers.iter().any(|(k, _)| k == "x-url"));
            assert!(!s.headers.iter().any(|(k, _)| k == "x-proxy"));
            assert!(!s.headers.iter().any(|(k, _)| k == "x-identifier"));
            assert!(!s.headers.iter().any(|(k, _)| k == "x-session-id"));
            // Real headers should remain
            assert!(s.headers.iter().any(|(k, _)| k == "host"));
            assert!(s.headers.iter().any(|(k, _)| k == "user-agent"));
            assert_eq!(s.body, "user=test&pass=test");
        } else {
            panic!("Expected HttpRequest");
        }
    }

    #[test]
    fn test_keycheck_custom_statuses() {
        let script = r#"
BLOCK:Keycheck
  KEYCHAIN 2FA OR
    STRINGKEY @data.SOURCE Contains "two_factor"
  KEYCHAIN CAPTCHA OR
    STRINGKEY @data.SOURCE Contains "captcha_required"
  KEYCHAIN LOCKED OR
    STRINGKEY @data.SOURCE Contains "account_locked"
ENDBLOCK
"#;
        let mut warnings = Vec::new();
        let blocks = parse_lolicode_blocks(script, &mut warnings).unwrap();
        assert_eq!(blocks.len(), 1);
        if let BlockSettings::KeyCheck(ref s) = blocks[0].settings {
            assert_eq!(s.keychains.len(), 3);
            assert!(matches!(s.keychains[0].result, BotStatus::Custom));
            assert!(matches!(s.keychains[1].result, BotStatus::Custom));
            assert!(matches!(s.keychains[2].result, BotStatus::Custom));
        } else {
            panic!("Expected KeyCheck");
        }
    }

    // ────────────────────────────────────────────────────────────
    // SVB tests
    // ────────────────────────────────────────────────────────────

    #[test]
    fn test_svb_prefix_parsing() {
        let (label, disabled, cmd) = parse_svb_prefix("#LOGIN REQUEST POST \"url\"");
        assert_eq!(label, "LOGIN");
        assert!(!disabled);
        assert_eq!(cmd, "REQUEST POST \"url\"");

        let (label, disabled, cmd) = parse_svb_prefix("!#ADS FUNCTION Constant \"test\"");
        assert_eq!(label, "ADS");
        assert!(disabled);
        assert_eq!(cmd, "FUNCTION Constant \"test\"");

        let (label, disabled, cmd) = parse_svb_prefix("KEYCHECK");
        assert!(label.is_empty());
        assert!(!disabled);
        assert_eq!(cmd, "KEYCHECK");
    }

    #[test]
    fn test_svb_extract_quoted() {
        let (val, rest) = svb_extract_quoted("\"hello world\" extra");
        assert_eq!(val, "hello world");
        assert_eq!(rest, " extra");

        let (val, _) = svb_extract_quoted("\"escaped \\\"quote\\\"\"");
        assert_eq!(val, "escaped \"quote\"");

        let (val, _) = svb_extract_quoted("not quoted");
        assert!(val.is_empty());
    }

    #[test]
    fn test_svb_source_conversion() {
        assert_eq!(convert_svb_source_ref("<SOURCE>"), "data.SOURCE");
        assert_eq!(convert_svb_source_ref("<COOKIES(flwssn)>"), "data.COOKIES[\"flwssn\"]");
        assert_eq!(convert_svb_source_ref("<myVar>"), "myVar");
    }

    #[test]
    fn test_svb_var_refs_conversion() {
        assert_eq!(convert_svb_var_refs("email=<USER>&pass=<PASS>"), "email=<input.USER>&pass=<input.PASS>");
        assert_eq!(convert_svb_var_refs("<ua>"), "<ua>"); // non-data vars unchanged
    }

    #[test]
    fn test_svb_deexoptions_import() {
        let path = "data/OB2/deexoptions.com.svb";
        if let Ok(bytes) = std::fs::read(path) {
            let result = import_config_bytes(&bytes).unwrap();
            let pipeline = result.pipeline;

            assert_eq!(pipeline.name, "deexoptions.com");
            assert!(pipeline.blocks.len() >= 12, "deexoptions should have 12+ blocks, got {}", pipeline.blocks.len());

            // Should have a RandomUserAgent block
            let rua = pipeline.blocks.iter().find(|b| matches!(b.block_type, BlockType::RandomUserAgent));
            assert!(rua.is_some(), "deexoptions should have a RandomUserAgent block");

            // Should have HTTP POST blocks with body and headers
            let http_with_body = pipeline.blocks.iter().find(|b| {
                if let BlockSettings::HttpRequest(ref s) = b.settings {
                    !s.body.is_empty() && s.method == "POST"
                } else { false }
            });
            assert!(http_with_body.is_some(), "deexoptions should have POST blocks with body");

            // HTTP blocks should not follow redirects (AutoRedirect=FALSE)
            let no_redirect = pipeline.blocks.iter().find(|b| {
                if let BlockSettings::HttpRequest(ref s) = b.settings {
                    !s.follow_redirects
                } else { false }
            });
            assert!(no_redirect.is_some(), "deexoptions should have follow_redirects=false");

            // Should have keychains
            let kc = pipeline.blocks.iter().find(|b| matches!(b.block_type, BlockType::KeyCheck));
            assert!(kc.is_some(), "deexoptions should have KeyCheck blocks");
            if let BlockSettings::KeyCheck(ref s) = kc.unwrap().settings {
                assert!(!s.keychains.is_empty(), "KeyCheck should have keychains");
            }

            // Should have ParseJSON blocks
            let json_parse = pipeline.blocks.iter().find(|b| {
                if let BlockSettings::ParseJSON(ref s) = b.settings {
                    !s.json_path.is_empty()
                } else { false }
            });
            assert!(json_parse.is_some(), "deexoptions should have ParseJSON blocks");

            // HTTP headers should contain User-Agent with variable ref
            if let Some(block) = http_with_body {
                if let BlockSettings::HttpRequest(ref s) = block.settings {
                    let has_ua = s.headers.iter().any(|(k, _)| k == "User-Agent");
                    assert!(has_ua, "HTTP block should have User-Agent header");
                }
            }
        }
    }

    #[test]
    fn test_svb_cyberghost_import() {
        let path = "data/OB2/CYBERGHOST.svb";
        if let Ok(bytes) = std::fs::read(path) {
            let result = import_config_bytes(&bytes).unwrap();
            let pipeline = result.pipeline;

            assert_eq!(pipeline.name, "[CYBERGHOST]");
            assert_eq!(pipeline.author, "@Firexkeyboard");
            assert!(pipeline.blocks.len() >= 15, "CYBERGHOST should have 15+ blocks, got {}", pipeline.blocks.len());

            // Should have RandomData blocks (from FUNCTION RandomString)
            let random = pipeline.blocks.iter().filter(|b| matches!(b.block_type, BlockType::RandomData)).count();
            assert!(random >= 2, "CYBERGHOST should have 2+ RandomData blocks, got {}", random);

            // Should have HTTP POST and GET blocks
            let http_post = pipeline.blocks.iter().find(|b| {
                if let BlockSettings::HttpRequest(ref s) = b.settings {
                    s.method == "POST" && !s.body.is_empty()
                } else { false }
            });
            assert!(http_post.is_some(), "CYBERGHOST should have POST block with body");

            let http_get = pipeline.blocks.iter().find(|b| {
                if let BlockSettings::HttpRequest(ref s) = b.settings {
                    s.method == "GET"
                } else { false }
            });
            assert!(http_get.is_some(), "CYBERGHOST should have GET block");

            // HTTP blocks should have multiple headers
            let http_many_headers = pipeline.blocks.iter().find(|b| {
                if let BlockSettings::HttpRequest(ref s) = b.settings {
                    s.headers.len() > 5
                } else { false }
            });
            assert!(http_many_headers.is_some(), "CYBERGHOST should have HTTP block with 5+ headers");

            // Should have ParseLR blocks
            let parse_lr = pipeline.blocks.iter().find(|b| {
                if let BlockSettings::ParseLR(ref s) = b.settings {
                    !s.left.is_empty()
                } else { false }
            });
            assert!(parse_lr.is_some(), "CYBERGHOST should have ParseLR blocks");

            // Should have IfElse block
            let if_else = pipeline.blocks.iter().find(|b| matches!(b.block_type, BlockType::IfElse));
            assert!(if_else.is_some(), "CYBERGHOST should have IfElse block");

            // KeyCheck with Custom status
            let custom_kc = pipeline.blocks.iter().find(|b| {
                if let BlockSettings::KeyCheck(ref s) = b.settings {
                    s.keychains.iter().any(|kc| matches!(kc.result, BotStatus::Custom))
                } else { false }
            });
            assert!(custom_kc.is_some(), "CYBERGHOST should have KeyCheck with Custom status");

            // Should have SetVariable with capture
            let set_cap = pipeline.blocks.iter().find(|b| {
                if let BlockSettings::SetVariable(ref s) = b.settings {
                    s.capture
                } else { false }
            });
            assert!(set_cap.is_some(), "CYBERGHOST should have SetVariable with capture");
        }
    }

    #[test]
    fn test_svb_nflix3_import() {
        let path = "data/OB2/NFLIX3.svb";
        if let Ok(bytes) = std::fs::read(path) {
            let result = import_config_bytes(&bytes).unwrap();
            let pipeline = result.pipeline;

            assert_eq!(pipeline.name, "NFLIX3");
            assert_eq!(pipeline.runner_settings.threads, 50);
            assert!(pipeline.blocks.len() >= 25, "NFLIX3 should have 25+ blocks, got {}", pipeline.blocks.len());

            // Should have disabled blocks (the !#ADS ones)
            let disabled_count = pipeline.blocks.iter().filter(|b| b.disabled).count();
            assert!(disabled_count >= 3, "NFLIX3 should have 3+ disabled blocks, got {}", disabled_count);

            // Should have RandomUserAgent block
            let rua = pipeline.blocks.iter().find(|b| matches!(b.block_type, BlockType::RandomUserAgent));
            assert!(rua.is_some(), "NFLIX3 should have RandomUserAgent block");

            // Should have cookie PARSE blocks
            let cookie_parse = pipeline.blocks.iter().find(|b| {
                if let BlockSettings::ParseLR(ref s) = b.settings {
                    s.input_var.contains("COOKIES")
                } else { false }
            });
            assert!(cookie_parse.is_some(), "NFLIX3 should have PARSE block reading cookies");

            // Should have a Translate block (as Script)
            let translate = pipeline.blocks.iter().find(|b| {
                if let BlockSettings::Script(ref s) = b.settings {
                    s.code.contains("Translate")
                } else { false }
            });
            assert!(translate.is_some(), "NFLIX3 should have Translate script block");

            // HTTP POST with body and many headers
            let http_post = pipeline.blocks.iter().find(|b| {
                if let BlockSettings::HttpRequest(ref s) = b.settings {
                    s.method == "POST" && !s.body.is_empty() && s.headers.len() > 10
                } else { false }
            });
            assert!(http_post.is_some(), "NFLIX3 should have POST block with body and 10+ headers");

            // Should have Replace blocks
            let replace = pipeline.blocks.iter().find(|b| {
                if let BlockSettings::StringFunction(ref s) = b.settings {
                    matches!(s.function_type, StringFnType::Replace)
                } else { false }
            });
            assert!(replace.is_some(), "NFLIX3 should have Replace string function");

            // Body should have <input.USER> and <input.PASS> (converted from <USER>/<PASS>)
            if let Some(block) = http_post {
                if let BlockSettings::HttpRequest(ref s) = block.settings {
                    assert!(s.body.contains("<input.PASS>"), "HTTP body should have <input.PASS>, got body: {}...", &s.body[..100.min(s.body.len())]);
                    assert!(s.body.contains("<input.USER>"), "HTTP body should have <input.USER>");
                }
            }
        }
    }

    #[test]
    fn test_svb_key_parsing() {
        // Simple KEY "value" → Contains against SOURCE
        let cond = parse_svb_key("\"oauth_token\"").unwrap();
        assert_eq!(cond.source, "data.SOURCE");
        assert!(matches!(cond.comparison, Comparison::Contains));
        assert_eq!(cond.value, "oauth_token");

        // KEY "<RESPONSECODE>" Contains "429"
        let cond = parse_svb_key("\"<RESPONSECODE>\" Contains \"429\"").unwrap();
        assert_eq!(cond.source, "data.RESPONSECODE");
        assert!(matches!(cond.comparison, Comparison::Contains));
        assert_eq!(cond.value, "429");

        // KEY "<DAYS LEFT>" GreaterThan "0"
        let cond = parse_svb_key("\"<DAYS LEFT>\" GreaterThan \"0\"").unwrap();
        assert_eq!(cond.source, "DAYS LEFT");
        assert!(matches!(cond.comparison, Comparison::GreaterThan));
        assert_eq!(cond.value, "0");
    }

    #[test]
    fn test_svb_parse_output() {
        let (cap, name) = parse_svb_output("Recursive=TRUE CreateEmpty=FALSE -> VAR \"country1\"");
        assert!(!cap);
        assert_eq!(name, "country1");

        let (cap, name) = parse_svb_output("CreateEmpty=FALSE -> CAP \"PlanName\"");
        assert!(cap);
        assert_eq!(name, "PlanName");
    }
}
