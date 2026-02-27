use serde::{Deserialize, Serialize};

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
    /// When false, skip TLS certificate verification (for debugging / self-signed certs)
    #[serde(default = "default_ssl_verify")]
    pub ssl_verify: bool,
    /// Optional dash-separated IANA cipher suite IDs to override browser defaults.
    /// e.g. "4865-4866-4867-49195-49199-49196-49200-52393-52392"
    /// Leave empty to use the browser profile's built-in cipher list.
    #[serde(default)]
    pub cipher_suites: String,
}

fn default_ssl_verify() -> bool { true }

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
            ssl_verify: true,
            cipher_suites: String::new(),
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
