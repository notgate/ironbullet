use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request sent from Rust to Go sidecar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarRequest {
    pub id: String,
    pub action: String,
    pub session: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ja3: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http2fp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_redirects: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_redirects: Option<i64>,
    /// Skip TLS certificate verification when false (default = verify = true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssl_verify: Option<bool>,
    /// Dash-separated IANA cipher suite IDs overriding the browser profile defaults.
    /// e.g. "4865-4866-4867-49195-49199-49196-49200"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_ciphers: Option<String>,
}

/// Response from Go sidecar back to Rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarResponse {
    pub id: String,
    #[serde(default)]
    pub status: i32,
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub cookies: Option<HashMap<String, String>>,
    #[serde(default)]
    pub final_url: String,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub timing_ms: i64,
}
