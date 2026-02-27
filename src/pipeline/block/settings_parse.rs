use serde::{Deserialize, Serialize};

fn default_parse_input() -> String { "data.SOURCE".to_string() }
fn default_parse_cookie_input() -> String { "data.COOKIES".to_string() }

// ── Parse LR ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseLRSettings {
    #[serde(default = "default_parse_input")]
    pub input_var: String,
    pub left: String,
    pub right: String,
    pub output_var: String,
    pub capture: bool,
    pub recursive: bool,
    pub case_insensitive: bool,
}

impl Default for ParseLRSettings {
    fn default() -> Self {
        Self {
            input_var: default_parse_input(),
            left: String::new(),
            right: String::new(),
            output_var: "PARSED".into(),
            capture: false,
            recursive: false,
            case_insensitive: false,
        }
    }
}

// ── Parse Regex ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseRegexSettings {
    #[serde(default = "default_parse_input")]
    pub input_var: String,
    pub pattern: String,
    pub output_format: String,
    pub output_var: String,
    pub capture: bool,
    pub multi_line: bool,
}

impl Default for ParseRegexSettings {
    fn default() -> Self {
        Self {
            input_var: default_parse_input(),
            pattern: String::new(),
            output_format: "$1".into(),
            output_var: "PARSED".into(),
            capture: false,
            multi_line: false,
        }
    }
}

// ── Parse JSON ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseJSONSettings {
    #[serde(default = "default_parse_input")]
    pub input_var: String,
    pub json_path: String,
    pub output_var: String,
    pub capture: bool,
}

impl Default for ParseJSONSettings {
    fn default() -> Self {
        Self {
            input_var: default_parse_input(),
            json_path: String::new(),
            output_var: "PARSED".into(),
            capture: false,
        }
    }
}

// ── Parse CSS ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseCSSSettings {
    #[serde(default = "default_parse_input")]
    pub input_var: String,
    pub selector: String,
    pub attribute: String,
    pub output_var: String,
    pub capture: bool,
    pub index: i32,
}

impl Default for ParseCSSSettings {
    fn default() -> Self {
        Self {
            input_var: default_parse_input(),
            selector: String::new(),
            attribute: "innerText".into(),
            output_var: "PARSED".into(),
            capture: false,
            index: 0,
        }
    }
}

// ── Parse XPath ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseXPathSettings {
    #[serde(default = "default_parse_input")]
    pub input_var: String,
    pub xpath: String,
    pub output_var: String,
    pub capture: bool,
}

impl Default for ParseXPathSettings {
    fn default() -> Self {
        Self {
            input_var: default_parse_input(),
            xpath: String::new(),
            output_var: "PARSED".into(),
            capture: false,
        }
    }
}

// ── Parse Cookie ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseCookieSettings {
    #[serde(default = "default_parse_cookie_input")]
    pub input_var: String,
    pub cookie_name: String,
    pub output_var: String,
    pub capture: bool,
}

impl Default for ParseCookieSettings {
    fn default() -> Self {
        Self {
            input_var: default_parse_cookie_input(),
            cookie_name: String::new(),
            output_var: "PARSED".into(),
            capture: false,
        }
    }
}

// ── Unified Parse Block ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParseMode {
    // Explicit variant names must match frontend option values exactly.
    // Legacy snake_case aliases are included for backward compatibility.
    #[serde(alias = "l_r")]
    LR,
    #[serde(alias = "regex")]
    Regex,
    #[serde(alias = "json")]
    Json,
    #[serde(alias = "css")]
    Css,
    #[serde(alias = "x_path")]
    XPath,
    #[serde(alias = "cookie")]
    Cookie,
    #[serde(alias = "lambda")]
    Lambda,
}

impl Default for ParseMode {
    fn default() -> Self { ParseMode::LR }
}

fn default_parse_mode() -> ParseMode { ParseMode::LR }
fn default_output_format() -> String { "$1".into() }
fn default_attribute() -> String { "innerText".into() }
fn default_lambda() -> String { "x => x.split(',')[0]".into() }
fn default_parse_output_var() -> String { "PARSED".into() }

/// Unified parse block that replaces ParseLR / ParseRegex / ParseJSON /
/// ParseCSS / ParseXPath / ParseCookie / LambdaParser.
/// Old block types remain functional for backward compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseSettings {
    #[serde(default = "default_parse_mode")]
    pub parse_mode: ParseMode,

    // ── Common ──
    #[serde(default = "default_parse_input")]
    pub input_var: String,
    #[serde(default = "default_parse_output_var")]
    pub output_var: String,
    #[serde(default)]
    pub capture: bool,

    // ── LR ──
    #[serde(default)]
    pub left: String,
    #[serde(default)]
    pub right: String,
    #[serde(default)]
    pub recursive: bool,
    #[serde(default)]
    pub case_insensitive: bool,

    // ── Regex ──
    #[serde(default)]
    pub pattern: String,
    #[serde(default = "default_output_format")]
    pub output_format: String,
    #[serde(default)]
    pub multi_line: bool,

    // ── JSON ──
    #[serde(default)]
    pub json_path: String,

    // ── CSS ──
    #[serde(default)]
    pub selector: String,
    #[serde(default = "default_attribute")]
    pub attribute: String,
    #[serde(default)]
    pub index: i32,

    // ── XPath ──
    #[serde(default)]
    pub xpath: String,

    // ── Cookie ──
    #[serde(default)]
    pub cookie_name: String,

    // ── Lambda ──
    #[serde(default = "default_lambda")]
    pub lambda_expression: String,
}

impl Default for ParseSettings {
    fn default() -> Self {
        Self {
            parse_mode: ParseMode::LR,
            input_var: default_parse_input(),
            output_var: "PARSED".into(),
            capture: false,
            left: String::new(), right: String::new(),
            recursive: false, case_insensitive: false,
            pattern: String::new(), output_format: "$1".into(), multi_line: false,
            json_path: String::new(),
            selector: String::new(), attribute: "innerText".into(), index: 0,
            xpath: String::new(),
            cookie_name: String::new(),
            lambda_expression: "x => x.split(',')[0]".into(),
        }
    }
}
