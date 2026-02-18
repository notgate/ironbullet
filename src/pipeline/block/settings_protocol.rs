use serde::{Deserialize, Serialize};

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
