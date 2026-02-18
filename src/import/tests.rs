use super::*;
use super::helpers::*;
use super::lolicode::parse_lolicode_blocks;
use super::svb::*;
use crate::pipeline::block::*;

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
