use serde::{Deserialize, Serialize};

/// Preset IP pool strategy for X-Forwarded-For / X-Real-IP injection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IpSpoofStrategy {
    /// Pick a random public IPv4 on every request
    RandomPublic,
    /// Rotate through a fixed list (one per line)
    FixedList,
    /// Use the pipeline's proxy IP (forward the proxy's IP as if it's the client)
    FromProxy,
    /// Manual — user provides the exact value (supports variable interpolation)
    Manual,
}

impl Default for IpSpoofStrategy {
    fn default() -> Self { Self::RandomPublic }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderSpoofSettings {
    /// Inject X-Forwarded-For
    #[serde(default = "default_true")]
    pub inject_xff: bool,
    /// Inject X-Real-IP
    #[serde(default)]
    pub inject_x_real_ip: bool,
    /// Inject CF-Connecting-IP (Cloudflare origin header)
    #[serde(default)]
    pub inject_cf_connecting_ip: bool,
    /// Inject True-Client-IP (Akamai / Cloudflare enterprise)
    #[serde(default)]
    pub inject_true_client_ip: bool,
    /// IP strategy for all injected headers
    #[serde(default)]
    pub strategy: IpSpoofStrategy,
    /// Fixed IP list (newline-separated) — used by FixedList strategy
    #[serde(default)]
    pub fixed_ips: String,
    /// Manual value — used by Manual strategy (supports variable interpolation)
    #[serde(default)]
    pub manual_value: String,
    /// Store the chosen IP into this variable so subsequent blocks can reuse it.
    /// Empty = don't store.
    #[serde(default)]
    pub output_var: String,
    /// Also inject X-Forwarded-Proto: https
    #[serde(default = "default_true")]
    pub inject_proto: bool,
    /// Also inject X-Forwarded-Host matching the request's Host header
    #[serde(default)]
    pub inject_host: bool,
}

fn default_true() -> bool { true }

impl Default for HeaderSpoofSettings {
    fn default() -> Self {
        Self {
            inject_xff: true,
            inject_x_real_ip: false,
            inject_cf_connecting_ip: false,
            inject_true_client_ip: false,
            strategy: IpSpoofStrategy::RandomPublic,
            fixed_ips: String::new(),
            manual_value: String::new(),
            output_var: String::new(),
            inject_proto: true,
            inject_host: false,
        }
    }
}
