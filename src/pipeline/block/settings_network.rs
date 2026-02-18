use serde::{Deserialize, Serialize};

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
