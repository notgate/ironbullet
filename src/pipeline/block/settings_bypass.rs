use serde::{Deserialize, Serialize};

// ── Captcha Solver ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaSolverSettings {
    pub solver_service: String,
    pub api_key: String,
    pub site_key: String,
    pub page_url: String,
    pub captcha_type: String,
    pub output_var: String,
    pub timeout_ms: u64,
    pub capture: bool,
}

impl Default for CaptchaSolverSettings {
    fn default() -> Self {
        Self { solver_service: "capsolver".into(), api_key: String::new(), site_key: String::new(), page_url: String::new(), captcha_type: "RecaptchaV2".into(), output_var: "CAPTCHA_TOKEN".into(), timeout_ms: 120000, capture: false }
    }
}

// ── Cloudflare Bypass ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareBypassSettings {
    pub url: String,
    pub flaresolverr_url: String,
    pub max_timeout_ms: u64,
    pub output_var: String,
    pub capture: bool,
}

impl Default for CloudflareBypassSettings {
    fn default() -> Self {
        Self { url: String::new(), flaresolverr_url: "http://localhost:8191/v1".into(), max_timeout_ms: 60000, output_var: "CF_COOKIES".into(), capture: false }
    }
}

// ── Laravel CSRF ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaravelCsrfSettings {
    pub url: String,
    pub csrf_selector: String,
    pub cookie_name: String,
    pub output_var: String,
    pub timeout_ms: u64,
    pub capture: bool,
}

impl Default for LaravelCsrfSettings {
    fn default() -> Self {
        Self { url: String::new(), csrf_selector: "input[name=\"_token\"]".into(), cookie_name: "XSRF-TOKEN".into(), output_var: "CSRF_TOKEN".into(), timeout_ms: 10000, capture: false }
    }
}

// ── Random User Agent ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomUserAgentSettings {
    pub mode: UserAgentMode,
    pub browser_filter: Vec<String>,
    pub platform_filter: Vec<String>,
    pub custom_list: String,
    pub output_var: String,
    pub capture: bool,
    #[serde(default)]
    pub match_tls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserAgentMode {
    Random,
    CustomList,
}

impl Default for RandomUserAgentSettings {
    fn default() -> Self {
        Self {
            mode: UserAgentMode::Random,
            browser_filter: vec!["Chrome".into(), "Firefox".into(), "Safari".into(), "Edge".into()],
            platform_filter: vec!["Desktop".into(), "Mobile".into()],
            custom_list: String::new(),
            output_var: "USER_AGENT".into(),
            capture: false,
            match_tls: false,
        }
    }
}

// ── OCR Captcha ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrCaptchaSettings {
    pub input_var: String,
    pub language: String,
    pub psm: u32,
    pub whitelist: String,
    pub output_var: String,
    pub capture: bool,
}

impl Default for OcrCaptchaSettings {
    fn default() -> Self {
        Self {
            input_var: "SCREENSHOT_B64".into(),
            language: "eng".into(),
            psm: 7,
            whitelist: String::new(),
            output_var: "OCR_TEXT".into(),
            capture: false,
        }
    }
}

// ── reCAPTCHA Invisible ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecaptchaInvisibleSettings {
    pub ar: String,
    pub sitekey: String,
    pub co: String,
    pub hi: String,
    pub v: String,
    pub size: String,
    pub action: String,
    pub cb: String,
    pub anchor_url: String,
    pub reload_url: String,
    pub user_agent: String,
    pub output_var: String,
    pub capture: bool,
}

impl Default for RecaptchaInvisibleSettings {
    fn default() -> Self {
        Self {
            ar: String::new(),
            sitekey: String::new(),
            co: String::new(),
            hi: String::new(),
            v: String::new(),
            size: "invisible".into(),
            action: String::new(),
            cb: String::new(),
            anchor_url: String::new(),
            reload_url: String::new(),
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".into(),
            output_var: "RECAPTCHA_TOKEN".into(),
            capture: false,
        }
    }
}

// ── XACF Sensor ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XacfSensorSettings {
    pub bundle_id: String,
    pub version: String,
    pub output_var: String,
    pub capture: bool,
}

impl Default for XacfSensorSettings {
    fn default() -> Self {
        Self {
            bundle_id: String::new(),
            version: "2.1.2".into(),
            output_var: "XACF_SENSOR".into(),
            capture: false,
        }
    }
}

// ── Random Data ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RandomDataType {
    String,
    Uuid,
    Number,
    Email,
    FirstName,
    LastName,
    FullName,
    StreetAddress,
    City,
    State,
    ZipCode,
    PhoneNumber,
    Date,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomDataSettings {
    pub data_type: RandomDataType,
    pub output_var: String,
    pub capture: bool,
    #[serde(default = "default_string_length")]
    pub string_length: u32,
    #[serde(default = "default_string_charset")]
    pub string_charset: String,
    #[serde(default)]
    pub custom_chars: String,
    #[serde(default)]
    pub number_min: i64,
    #[serde(default = "default_number_max")]
    pub number_max: i64,
    #[serde(default)]
    pub number_decimal: bool,
    #[serde(default = "default_date_format")]
    pub date_format: String,
    #[serde(default)]
    pub date_min: String,
    #[serde(default)]
    pub date_max: String,
}

fn default_string_length() -> u32 { 16 }
fn default_string_charset() -> String { "alphanumeric".into() }
fn default_number_max() -> i64 { 100 }
fn default_date_format() -> String { "%Y-%m-%d".into() }

impl Default for RandomDataSettings {
    fn default() -> Self {
        Self {
            data_type: RandomDataType::String,
            output_var: "RANDOM".into(),
            capture: false,
            string_length: 16,
            string_charset: "alphanumeric".into(),
            custom_chars: String::new(),
            number_min: 0,
            number_max: 100,
            number_decimal: false,
            date_format: "%Y-%m-%d".into(),
            date_min: String::new(),
            date_max: String::new(),
        }
    }
}

// ── DataDome Sensor ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDomeSensorSettings {
    pub site_url: String,
    pub cookie_datadome: String,
    pub user_agent: String,
    #[serde(default)]
    pub custom_wasm_b64: String,
    #[serde(default = "default_dd_output")]
    pub output_var: String,
    pub capture: bool,
}

fn default_dd_output() -> String { "DD_SENSOR".into() }

impl Default for DataDomeSensorSettings {
    fn default() -> Self {
        Self {
            site_url: String::new(),
            cookie_datadome: String::new(),
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".into(),
            custom_wasm_b64: String::new(),
            output_var: "DD_SENSOR".into(),
            capture: false,
        }
    }
}

// ── Plugin Block ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginBlockSettings {
    pub plugin_block_type: String,
    #[serde(default)]
    pub settings_json: String,
    #[serde(default = "default_plugin_output")]
    pub output_var: String,
    pub capture: bool,
}

fn default_plugin_output() -> String { "PLUGIN_RESULT".into() }

impl Default for PluginBlockSettings {
    fn default() -> Self {
        Self {
            plugin_block_type: String::new(),
            settings_json: "{}".into(),
            output_var: "PLUGIN_RESULT".into(),
            capture: false,
        }
    }
}
