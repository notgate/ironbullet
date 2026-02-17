use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Deserialize a UUID that tolerates empty strings (maps to nil UUID)
fn deserialize_uuid_tolerant<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(Uuid::nil())
    } else {
        Uuid::parse_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    #[serde(deserialize_with = "deserialize_uuid_tolerant")]
    pub id: Uuid,
    pub block_type: BlockType,
    pub label: String,
    pub disabled: bool,
    pub safe_mode: bool,
    pub settings: BlockSettings,
}

impl Block {
    pub fn new(block_type: BlockType) -> Self {
        let label = block_type.default_label().to_string();
        let settings = block_type.default_settings();
        Self {
            id: Uuid::new_v4(),
            block_type,
            label,
            disabled: false,
            safe_mode: false,
            settings,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockType {
    HttpRequest,
    ParseLR,
    ParseRegex,
    ParseJSON,
    ParseCSS,
    ParseXPath,
    ParseCookie,
    KeyCheck,
    StringFunction,
    ListFunction,
    CryptoFunction,
    ConversionFunction,
    IfElse,
    Loop,
    Delay,
    Script,
    Log,
    SetVariable,
    ClearCookies,
    // Networking
    Webhook,
    WebSocket,
    // Protocol requests
    TcpRequest,
    UdpRequest,
    FtpRequest,
    SshRequest,
    ImapRequest,
    SmtpRequest,
    PopRequest,
    // Bypass / Anti-bot
    CaptchaSolver,
    CloudflareBypass,
    LaravelCsrf,
    // Data management
    DateFunction,
    CaseSwitch,
    CookieContainer,
    // Browser automation
    BrowserOpen,
    NavigateTo,
    ClickElement,
    TypeText,
    WaitForElement,
    GetElementText,
    Screenshot,
    ExecuteJs,
    // New blocks
    RandomUserAgent,
    OcrCaptcha,
    RecaptchaInvisible,
    XacfSensor,
    // Phase 1-4 blocks
    RandomData,
    DataDomeSensor,
    Plugin,
    // Organization
    Group,
}

impl BlockType {
    pub fn default_label(&self) -> &'static str {
        match self {
            Self::HttpRequest => "HTTP Request",
            Self::ParseLR => "Parse LR",
            Self::ParseRegex => "Parse Regex",
            Self::ParseJSON => "Parse JSON",
            Self::ParseCSS => "Parse CSS",
            Self::ParseXPath => "Parse XPath",
            Self::ParseCookie => "Parse Cookie",
            Self::KeyCheck => "Key Check",
            Self::StringFunction => "String Function",
            Self::ListFunction => "List Function",
            Self::CryptoFunction => "Crypto Function",
            Self::ConversionFunction => "Conversion",
            Self::IfElse => "If / Else",
            Self::Loop => "Loop",
            Self::Delay => "Delay",
            Self::Script => "Script",
            Self::Log => "Log",
            Self::SetVariable => "Set Variable",
            Self::ClearCookies => "Clear Cookies",
            Self::Webhook => "Webhook",
            Self::WebSocket => "WebSocket",
            Self::TcpRequest => "TCP Request",
            Self::UdpRequest => "UDP Request",
            Self::FtpRequest => "FTP Request",
            Self::SshRequest => "SSH Request",
            Self::ImapRequest => "IMAP Request",
            Self::SmtpRequest => "SMTP Request",
            Self::PopRequest => "POP Request",
            Self::CaptchaSolver => "Captcha Solver",
            Self::CloudflareBypass => "Cloudflare Bypass",
            Self::LaravelCsrf => "Laravel CSRF",
            Self::DateFunction => "Date Function",
            Self::CaseSwitch => "Case / Switch",
            Self::CookieContainer => "Cookie Container",
            Self::BrowserOpen => "Browser Open",
            Self::NavigateTo => "Navigate To",
            Self::ClickElement => "Click Element",
            Self::TypeText => "Type Text",
            Self::WaitForElement => "Wait For Element",
            Self::GetElementText => "Get Element Text",
            Self::Screenshot => "Screenshot",
            Self::ExecuteJs => "Execute JS",
            Self::RandomUserAgent => "Random User Agent",
            Self::OcrCaptcha => "OCR Captcha",
            Self::RecaptchaInvisible => "reCAPTCHA Invisible",
            Self::XacfSensor => "XACF Sensor",
            Self::RandomData => "Random Data",
            Self::DataDomeSensor => "DataDome Sensor",
            Self::Plugin => "Plugin Block",
            Self::Group => "Group",
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            Self::HttpRequest | Self::TcpRequest | Self::UdpRequest | Self::FtpRequest | Self::SshRequest | Self::ImapRequest | Self::SmtpRequest | Self::PopRequest => "Requests",
            Self::CaptchaSolver | Self::CloudflareBypass | Self::LaravelCsrf | Self::OcrCaptcha | Self::RecaptchaInvisible => "Bypass",
            Self::XacfSensor | Self::DataDomeSensor => "Sensors",
            Self::RandomUserAgent | Self::RandomData | Self::Plugin => "Utilities",
            Self::ParseLR | Self::ParseRegex | Self::ParseJSON | Self::ParseCSS | Self::ParseXPath | Self::ParseCookie => "Parsing",
            Self::KeyCheck => "Checks",
            Self::StringFunction | Self::ListFunction | Self::CryptoFunction | Self::ConversionFunction | Self::DateFunction | Self::CookieContainer => "Functions",
            Self::IfElse | Self::Loop | Self::Delay | Self::Script | Self::CaseSwitch | Self::Group => "Control",
            Self::Log | Self::SetVariable | Self::ClearCookies | Self::Webhook | Self::WebSocket => "Utilities",
            Self::BrowserOpen | Self::NavigateTo | Self::ClickElement | Self::TypeText | Self::WaitForElement | Self::GetElementText | Self::Screenshot | Self::ExecuteJs => "Browser",
        }
    }

    pub fn color(&self) -> &'static str {
        match self.category() {
            "Requests" => "#0078d4",
            "Parsing" => "#4ec9b0",
            "Checks" => "#d7ba7d",
            "Functions" => "#c586c0",
            "Control" => "#dcdcaa",
            "Utilities" => "#858585",
            "Bypass" => "#e5c07b",
            "Browser" => "#e06c75",
            "Sensors" => "#2dd4bf",
            _ => "#858585",
        }
    }

    pub fn default_settings(&self) -> BlockSettings {
        match self {
            Self::HttpRequest => BlockSettings::HttpRequest(HttpRequestSettings::default()),
            Self::ParseLR => BlockSettings::ParseLR(ParseLRSettings::default()),
            Self::ParseRegex => BlockSettings::ParseRegex(ParseRegexSettings::default()),
            Self::ParseJSON => BlockSettings::ParseJSON(ParseJSONSettings::default()),
            Self::ParseCSS => BlockSettings::ParseCSS(ParseCSSSettings::default()),
            Self::ParseXPath => BlockSettings::ParseXPath(ParseXPathSettings::default()),
            Self::ParseCookie => BlockSettings::ParseCookie(ParseCookieSettings::default()),
            Self::KeyCheck => BlockSettings::KeyCheck(KeyCheckSettings::default()),
            Self::StringFunction => BlockSettings::StringFunction(StringFunctionSettings::default()),
            Self::ListFunction => BlockSettings::ListFunction(ListFunctionSettings::default()),
            Self::CryptoFunction => BlockSettings::CryptoFunction(CryptoFunctionSettings::default()),
            Self::ConversionFunction => BlockSettings::ConversionFunction(ConversionFunctionSettings::default()),
            Self::IfElse => BlockSettings::IfElse(IfElseSettings::default()),
            Self::Loop => BlockSettings::Loop(LoopSettings::default()),
            Self::Delay => BlockSettings::Delay(DelaySettings::default()),
            Self::Script => BlockSettings::Script(ScriptSettings::default()),
            Self::Log => BlockSettings::Log(LogSettings::default()),
            Self::SetVariable => BlockSettings::SetVariable(SetVariableSettings::default()),
            Self::ClearCookies => BlockSettings::ClearCookies,
            Self::Webhook => BlockSettings::Webhook(WebhookSettings::default()),
            Self::WebSocket => BlockSettings::WebSocket(WebSocketSettings::default()),
            Self::TcpRequest => BlockSettings::TcpRequest(TcpRequestSettings::default()),
            Self::UdpRequest => BlockSettings::UdpRequest(UdpRequestSettings::default()),
            Self::FtpRequest => BlockSettings::FtpRequest(FtpRequestSettings::default()),
            Self::SshRequest => BlockSettings::SshRequest(SshRequestSettings::default()),
            Self::ImapRequest => BlockSettings::ImapRequest(ImapRequestSettings::default()),
            Self::SmtpRequest => BlockSettings::SmtpRequest(SmtpRequestSettings::default()),
            Self::PopRequest => BlockSettings::PopRequest(PopRequestSettings::default()),
            Self::CaptchaSolver => BlockSettings::CaptchaSolver(CaptchaSolverSettings::default()),
            Self::CloudflareBypass => BlockSettings::CloudflareBypass(CloudflareBypassSettings::default()),
            Self::LaravelCsrf => BlockSettings::LaravelCsrf(LaravelCsrfSettings::default()),
            Self::DateFunction => BlockSettings::DateFunction(DateFunctionSettings::default()),
            Self::CaseSwitch => BlockSettings::CaseSwitch(CaseSwitchSettings::default()),
            Self::CookieContainer => BlockSettings::CookieContainer(CookieContainerSettings::default()),
            Self::BrowserOpen => BlockSettings::BrowserOpen(BrowserOpenSettings::default()),
            Self::NavigateTo => BlockSettings::NavigateTo(NavigateToSettings::default()),
            Self::ClickElement => BlockSettings::ClickElement(ClickElementSettings::default()),
            Self::TypeText => BlockSettings::TypeText(TypeTextSettings::default()),
            Self::WaitForElement => BlockSettings::WaitForElement(WaitForElementSettings::default()),
            Self::GetElementText => BlockSettings::GetElementText(GetElementTextSettings::default()),
            Self::Screenshot => BlockSettings::Screenshot(ScreenshotSettings::default()),
            Self::ExecuteJs => BlockSettings::ExecuteJs(ExecuteJsSettings::default()),
            Self::RandomUserAgent => BlockSettings::RandomUserAgent(RandomUserAgentSettings::default()),
            Self::OcrCaptcha => BlockSettings::OcrCaptcha(OcrCaptchaSettings::default()),
            Self::RecaptchaInvisible => BlockSettings::RecaptchaInvisible(RecaptchaInvisibleSettings::default()),
            Self::XacfSensor => BlockSettings::XacfSensor(XacfSensorSettings::default()),
            Self::RandomData => BlockSettings::RandomData(RandomDataSettings::default()),
            Self::DataDomeSensor => BlockSettings::DataDomeSensor(DataDomeSensorSettings::default()),
            Self::Plugin => BlockSettings::Plugin(PluginBlockSettings::default()),
            Self::Group => BlockSettings::Group(GroupSettings::default()),
        }
    }
}

// ── Block Settings (type-specific) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BlockSettings {
    HttpRequest(HttpRequestSettings),
    ParseLR(ParseLRSettings),
    ParseRegex(ParseRegexSettings),
    ParseJSON(ParseJSONSettings),
    ParseCSS(ParseCSSSettings),
    ParseXPath(ParseXPathSettings),
    ParseCookie(ParseCookieSettings),
    KeyCheck(KeyCheckSettings),
    StringFunction(StringFunctionSettings),
    ListFunction(ListFunctionSettings),
    CryptoFunction(CryptoFunctionSettings),
    ConversionFunction(ConversionFunctionSettings),
    IfElse(IfElseSettings),
    Loop(LoopSettings),
    Delay(DelaySettings),
    Script(ScriptSettings),
    Log(LogSettings),
    SetVariable(SetVariableSettings),
    ClearCookies,
    // Networking
    Webhook(WebhookSettings),
    WebSocket(WebSocketSettings),
    // Protocol requests
    TcpRequest(TcpRequestSettings),
    UdpRequest(UdpRequestSettings),
    FtpRequest(FtpRequestSettings),
    SshRequest(SshRequestSettings),
    ImapRequest(ImapRequestSettings),
    SmtpRequest(SmtpRequestSettings),
    PopRequest(PopRequestSettings),
    // Bypass / Anti-bot
    CaptchaSolver(CaptchaSolverSettings),
    CloudflareBypass(CloudflareBypassSettings),
    LaravelCsrf(LaravelCsrfSettings),
    // Data management
    DateFunction(DateFunctionSettings),
    CaseSwitch(CaseSwitchSettings),
    CookieContainer(CookieContainerSettings),
    // Browser automation
    BrowserOpen(BrowserOpenSettings),
    NavigateTo(NavigateToSettings),
    ClickElement(ClickElementSettings),
    TypeText(TypeTextSettings),
    WaitForElement(WaitForElementSettings),
    GetElementText(GetElementTextSettings),
    Screenshot(ScreenshotSettings),
    ExecuteJs(ExecuteJsSettings),
    // New blocks
    RandomUserAgent(RandomUserAgentSettings),
    OcrCaptcha(OcrCaptchaSettings),
    RecaptchaInvisible(RecaptchaInvisibleSettings),
    XacfSensor(XacfSensorSettings),
    // Phase 1-4 blocks
    RandomData(RandomDataSettings),
    DataDomeSensor(DataDomeSensorSettings),
    Plugin(PluginBlockSettings),
    // Organization
    Group(GroupSettings),
}

// ── HTTP Request ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequestSettings {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub body_type: BodyType,
    pub content_type: String,
    pub follow_redirects: bool,
    pub max_redirects: u32,
    pub timeout_ms: u64,
    pub auto_redirect: bool,
    pub basic_auth: Option<(String, String)>,
    #[serde(default = "default_http_version")]
    pub http_version: String,
    /// Variable name prefix for response storage.
    /// Body → {response_var}, Headers → {response_var}.HEADERS, Cookies → {response_var}.COOKIES,
    /// Status → {response_var}.STATUS, URL → {response_var}.URL
    #[serde(default = "default_response_var")]
    pub response_var: String,
    /// Custom cookies to send with the request (one per line: name=value)
    #[serde(default)]
    pub custom_cookies: String,
}

fn default_response_var() -> String {
    "SOURCE".to_string()
}

fn default_http_version() -> String {
    "HTTP/1.1".to_string()
}

impl Default for HttpRequestSettings {
    fn default() -> Self {
        Self {
            method: "GET".into(),
            url: String::new(),
            headers: vec![
                ("User-Agent".into(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".into()),
                ("Accept".into(), "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".into()),
                ("Accept-Language".into(), "en-US,en;q=0.5".into()),
            ],
            body: String::new(),
            body_type: BodyType::None,
            content_type: "application/x-www-form-urlencoded".into(),
            follow_redirects: true,
            max_redirects: 8,
            timeout_ms: 10000,
            auto_redirect: true,
            basic_auth: None,
            http_version: default_http_version(),
            response_var: default_response_var(),
            custom_cookies: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BodyType {
    None,
    Standard,
    Raw,
    Multipart,
    BasicAuth,
}

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

fn default_parse_input() -> String { "data.SOURCE".to_string() }
fn default_parse_cookie_input() -> String { "data.COOKIES".to_string() }

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

// ── Key Check ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyCheckSettings {
    pub keychains: Vec<Keychain>,
}

impl Default for KeyCheckSettings {
    fn default() -> Self {
        Self {
            keychains: vec![Keychain::default()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keychain {
    pub result: super::BotStatus,
    pub conditions: Vec<KeyCondition>,
}

impl Default for Keychain {
    fn default() -> Self {
        Self {
            result: super::BotStatus::Success,
            conditions: vec![KeyCondition::default()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyCondition {
    pub source: String,
    pub comparison: Comparison,
    pub value: String,
}

impl Default for KeyCondition {
    fn default() -> Self {
        Self {
            source: "data.RESPONSECODE".into(),
            comparison: Comparison::EqualTo,
            value: "200".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Comparison {
    Contains,
    NotContains,
    EqualTo,
    NotEqualTo,
    MatchesRegex,
    GreaterThan,
    LessThan,
    Exists,
    NotExists,
}

// ── String Function ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringFunctionSettings {
    pub function_type: StringFnType,
    pub input_var: String,
    pub output_var: String,
    pub capture: bool,
    pub param1: String,
    pub param2: String,
}

impl Default for StringFunctionSettings {
    fn default() -> Self {
        Self {
            function_type: StringFnType::Replace,
            input_var: String::new(),
            output_var: "RESULT".into(),
            capture: false,
            param1: String::new(),
            param2: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StringFnType {
    Replace,
    Substring,
    Trim,
    ToUpper,
    ToLower,
    URLEncode,
    URLDecode,
    Base64Encode,
    Base64Decode,
    HTMLEntityEncode,
    HTMLEntityDecode,
    Split,
    RandomString,
    Reverse,
    Length,
}

// ── List Function ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFunctionSettings {
    pub function_type: ListFnType,
    pub input_var: String,
    pub output_var: String,
    pub capture: bool,
    pub param1: String,
}

impl Default for ListFunctionSettings {
    fn default() -> Self {
        Self {
            function_type: ListFnType::Join,
            input_var: String::new(),
            output_var: "RESULT".into(),
            capture: false,
            param1: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ListFnType {
    Join,
    Sort,
    Shuffle,
    Add,
    Remove,
    Deduplicate,
    RandomItem,
    Length,
}

// ── Crypto Function ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoFunctionSettings {
    pub function_type: CryptoFnType,
    pub input_var: String,
    pub output_var: String,
    pub capture: bool,
    pub key: String,
}

impl Default for CryptoFunctionSettings {
    fn default() -> Self {
        Self {
            function_type: CryptoFnType::MD5,
            input_var: String::new(),
            output_var: "HASH".into(),
            capture: false,
            key: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CryptoFnType {
    MD5,
    SHA1,
    SHA256,
    SHA512,
    SHA384,
    CRC32,
    HMACSHA256,
    HMACSHA512,
    HMACMD5,
    BCryptHash,
    BCryptVerify,
    Base64Encode,
    Base64Decode,
    AESEncrypt,
    AESDecrypt,
}

// ── Conversion Function ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionFunctionSettings {
    pub input_var: String,
    pub output_var: String,
    pub capture: bool,
    pub from_type: String,
    pub to_type: String,
}

impl Default for ConversionFunctionSettings {
    fn default() -> Self {
        Self {
            input_var: String::new(),
            output_var: "CONVERTED".into(),
            capture: false,
            from_type: "string".into(),
            to_type: "int".into(),
        }
    }
}

// ── If/Else ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfElseSettings {
    pub condition: KeyCondition,
    pub true_blocks: Vec<Block>,
    pub false_blocks: Vec<Block>,
}

impl Default for IfElseSettings {
    fn default() -> Self {
        Self {
            condition: KeyCondition::default(),
            true_blocks: Vec::new(),
            false_blocks: Vec::new(),
        }
    }
}

// ── Loop ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopSettings {
    pub loop_type: LoopType,
    pub list_var: String,
    pub item_var: String,
    pub count: u32,
    pub blocks: Vec<Block>,
}

impl Default for LoopSettings {
    fn default() -> Self {
        Self {
            loop_type: LoopType::ForEach,
            list_var: String::new(),
            item_var: "ITEM".into(),
            count: 1,
            blocks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoopType {
    ForEach,
    Repeat,
}

// ── Delay ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelaySettings {
    pub min_ms: u64,
    pub max_ms: u64,
}

impl Default for DelaySettings {
    fn default() -> Self {
        Self {
            min_ms: 1000,
            max_ms: 1000,
        }
    }
}

// ── Script ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptSettings {
    pub code: String,
    pub output_var: String,
    pub capture: bool,
}

impl Default for ScriptSettings {
    fn default() -> Self {
        Self {
            code: String::new(),
            output_var: "RESULT".into(),
            capture: false,
        }
    }
}

// ── Log ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSettings {
    pub message: String,
}

impl Default for LogSettings {
    fn default() -> Self {
        Self {
            message: String::new(),
        }
    }
}

// ── Set Variable ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetVariableSettings {
    pub name: String,
    pub value: String,
    pub capture: bool,
}

impl Default for SetVariableSettings {
    fn default() -> Self {
        Self {
            name: String::new(),
            value: String::new(),
            capture: false,
        }
    }
}

// ── Webhook ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookSettings {
    pub url: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub body_template: String,
    pub content_type: String,
    #[serde(default)]
    pub custom_cookies: String,
}

impl Default for WebhookSettings {
    fn default() -> Self {
        Self {
            url: String::new(),
            method: "POST".into(),
            headers: Vec::new(),
            body_template: String::new(),
            content_type: "application/json".into(),
            custom_cookies: String::new(),
        }
    }
}

// ── WebSocket ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketSettings {
    pub url: String,
    pub action: String, // "connect", "send", "receive", "close"
    pub message: String,
    pub output_var: String,
    pub timeout_ms: u64,
}

impl Default for WebSocketSettings {
    fn default() -> Self {
        Self {
            url: String::new(),
            action: "connect".into(),
            message: String::new(),
            output_var: "WS_RESPONSE".into(),
            timeout_ms: 10000,
        }
    }
}

// ── Browser Automation ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserOpenSettings {
    pub headless: bool,
    pub browser_type: String,
    pub proxy: String,
    pub extra_args: String,
}

impl Default for BrowserOpenSettings {
    fn default() -> Self {
        Self {
            headless: true,
            browser_type: "chromium".into(),
            proxy: String::new(),
            extra_args: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigateToSettings {
    pub url: String,
    pub wait_until: String,
    pub timeout_ms: u64,
    #[serde(default)]
    pub custom_cookies: String,
}

impl Default for NavigateToSettings {
    fn default() -> Self {
        Self {
            url: String::new(),
            wait_until: "load".into(),
            timeout_ms: 30000,
            custom_cookies: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickElementSettings {
    pub selector: String,
    pub wait_for_navigation: bool,
    pub timeout_ms: u64,
}

impl Default for ClickElementSettings {
    fn default() -> Self {
        Self {
            selector: String::new(),
            wait_for_navigation: false,
            timeout_ms: 5000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeTextSettings {
    pub selector: String,
    pub text: String,
    pub clear_first: bool,
    pub delay_ms: u64,
}

impl Default for TypeTextSettings {
    fn default() -> Self {
        Self {
            selector: String::new(),
            text: String::new(),
            clear_first: true,
            delay_ms: 50,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitForElementSettings {
    pub selector: String,
    pub state: String,
    pub timeout_ms: u64,
}

impl Default for WaitForElementSettings {
    fn default() -> Self {
        Self {
            selector: String::new(),
            state: "visible".into(),
            timeout_ms: 10000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetElementTextSettings {
    pub selector: String,
    pub attribute: String,
    pub output_var: String,
    pub capture: bool,
}

impl Default for GetElementTextSettings {
    fn default() -> Self {
        Self {
            selector: String::new(),
            attribute: String::new(),
            output_var: "ELEMENT_TEXT".into(),
            capture: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotSettings {
    pub full_page: bool,
    pub selector: String,
    pub output_var: String,
}

impl Default for ScreenshotSettings {
    fn default() -> Self {
        Self {
            full_page: false,
            selector: String::new(),
            output_var: "SCREENSHOT_B64".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteJsSettings {
    pub code: String,
    pub output_var: String,
    pub capture: bool,
}

impl Default for ExecuteJsSettings {
    fn default() -> Self {
        Self {
            code: String::new(),
            output_var: "JS_RESULT".into(),
            capture: false,
        }
    }
}

// ── Protocol Requests ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpRequestSettings {
    pub host: String,
    pub port: u16,
    pub data: String,
    pub output_var: String,
    pub timeout_ms: u64,
    pub use_tls: bool,
    pub capture: bool,
}

impl Default for TcpRequestSettings {
    fn default() -> Self {
        Self { host: String::new(), port: 80, data: String::new(), output_var: "TCP_RESPONSE".into(), timeout_ms: 5000, use_tls: false, capture: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdpRequestSettings {
    pub host: String,
    pub port: u16,
    pub data: String,
    pub output_var: String,
    pub timeout_ms: u64,
    pub capture: bool,
}

impl Default for UdpRequestSettings {
    fn default() -> Self {
        Self { host: String::new(), port: 53, data: String::new(), output_var: "UDP_RESPONSE".into(), timeout_ms: 5000, capture: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtpRequestSettings {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub command: String,
    pub output_var: String,
    pub timeout_ms: u64,
    pub capture: bool,
}

impl Default for FtpRequestSettings {
    fn default() -> Self {
        Self { host: String::new(), port: 21, username: String::new(), password: String::new(), command: "LIST".into(), output_var: "FTP_RESPONSE".into(), timeout_ms: 10000, capture: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshRequestSettings {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub command: String,
    pub output_var: String,
    pub timeout_ms: u64,
    pub capture: bool,
}

impl Default for SshRequestSettings {
    fn default() -> Self {
        Self { host: String::new(), port: 22, username: String::new(), password: String::new(), command: String::new(), output_var: "SSH_RESPONSE".into(), timeout_ms: 10000, capture: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImapRequestSettings {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
    pub command: String,
    pub output_var: String,
    pub timeout_ms: u64,
    pub capture: bool,
}

impl Default for ImapRequestSettings {
    fn default() -> Self {
        Self { host: String::new(), port: 993, username: String::new(), password: String::new(), use_tls: true, command: "LOGIN".into(), output_var: "IMAP_RESPONSE".into(), timeout_ms: 10000, capture: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpRequestSettings {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
    pub command: String,
    pub output_var: String,
    pub timeout_ms: u64,
    pub capture: bool,
}

impl Default for SmtpRequestSettings {
    fn default() -> Self {
        Self { host: String::new(), port: 587, username: String::new(), password: String::new(), use_tls: true, command: "EHLO".into(), output_var: "SMTP_RESPONSE".into(), timeout_ms: 10000, capture: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopRequestSettings {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
    pub command: String,
    pub output_var: String,
    pub timeout_ms: u64,
    pub capture: bool,
}

impl Default for PopRequestSettings {
    fn default() -> Self {
        Self { host: String::new(), port: 995, username: String::new(), password: String::new(), use_tls: true, command: "STAT".into(), output_var: "POP_RESPONSE".into(), timeout_ms: 10000, capture: false }
    }
}

// ── Date Function ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateFunctionSettings {
    pub function_type: DateFnType,
    pub input_var: String,
    pub output_var: String,
    pub format: String,
    pub amount: i64,
    pub unit: String,
    pub capture: bool,
}

impl Default for DateFunctionSettings {
    fn default() -> Self {
        Self {
            function_type: DateFnType::Now,
            input_var: String::new(),
            output_var: "DATE".into(),
            format: "%Y-%m-%d %H:%M:%S".into(),
            amount: 0,
            unit: "seconds".into(),
            capture: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DateFnType {
    Now,
    FormatDate,
    ParseDate,
    AddTime,
    SubtractTime,
    UnixTimestamp,
    UnixToDate,
}

// ── Case / Switch ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseSwitchSettings {
    pub input_var: String,
    pub cases: Vec<CaseBranch>,
    pub default_value: String,
    pub output_var: String,
    pub capture: bool,
}

impl Default for CaseSwitchSettings {
    fn default() -> Self {
        Self {
            input_var: "data.RESPONSECODE".into(),
            cases: vec![
                CaseBranch { match_value: "200".into(), result_value: "SUCCESS".into() },
                CaseBranch { match_value: "403".into(), result_value: "BAN".into() },
            ],
            default_value: "FAIL".into(),
            output_var: "RESULT".into(),
            capture: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseBranch {
    pub match_value: String,
    pub result_value: String,
}

// ── Cookie Container (OpenBullet-style) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieContainerSettings {
    /// Path to cookie file (Netscape format) or raw cookie text
    pub source: String,
    /// "file" or "text" — read from file path or use source as raw text
    #[serde(default = "default_cookie_source_type")]
    pub source_type: String,
    /// Domain to filter cookies by (empty = all)
    pub domain: String,
    /// Variable name to store extracted cookies
    pub output_var: String,
    /// Whether to capture as user variable
    pub capture: bool,
    /// Also store in Netscape format variable
    #[serde(default)]
    pub save_netscape: bool,
}

fn default_cookie_source_type() -> String { "text".to_string() }

impl Default for CookieContainerSettings {
    fn default() -> Self {
        Self {
            source: String::new(),
            source_type: "text".into(),
            domain: String::new(),
            output_var: "COOKIES".into(),
            capture: false,
            save_netscape: false,
        }
    }
}

// ── Bypass / Anti-bot ──

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

// ── Group (organizational container) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSettings {
    #[serde(default)]
    pub blocks: Vec<Block>,
    #[serde(default = "default_true")]
    pub collapsed: bool,
}

fn default_true() -> bool { true }

impl Default for GroupSettings {
    fn default() -> Self {
        Self {
            blocks: Vec::new(),
            collapsed: false,
        }
    }
}
