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
