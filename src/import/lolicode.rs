use crate::pipeline::block::*;
use crate::pipeline::*;

use super::helpers::*;
use super::ImportResult;

// ────────────────────────────────────────────────────────────
// LoliCode BLOCK: parser (used by both .opk and plain .loli)
// ────────────────────────────────────────────────────────────

/// Parse LoliCode BLOCK:Type ... ENDBLOCK syntax
pub(super) fn parse_lolicode_blocks(content: &str, warnings: &mut Vec<String>) -> Result<Vec<Block>, String> {
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
                    "// Converted from OB2 C# preamble — review and adapt for ironbullet\n{}",
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

/// Convert a single OB2 block to a ironbullet Block
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
                    mode: crate::pipeline::block::settings_check::KeychainMode::And,
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
    // Parse output variable and capture flag if present
    let mut output_info = String::new();
    let mut parsed_output_var = String::new();
    let mut parsed_capture = false;
    for line in lines {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("=> CAP @") {
            parsed_output_var = rest.trim().to_string();
            parsed_capture = true;
            output_info = format!("\n// Output: {}", trimmed);
        } else if let Some(rest) = trimmed.strip_prefix("=> VAR @") {
            parsed_output_var = rest.trim().to_string();
            parsed_capture = false;
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
        if !parsed_output_var.is_empty() {
            s.output_var = parsed_output_var;
            s.capture = parsed_capture;
        }
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
