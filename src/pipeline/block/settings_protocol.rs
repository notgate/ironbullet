use serde::{Deserialize, Serialize};

fn default_smtp_action() -> String { "VERIFY".into() }
fn default_ftp_command() -> String { "LIST".into() }
fn default_imap_command() -> String { "LOGIN".into() }
fn default_pop_command() -> String { "STAT".into() }
fn default_imap_mailbox() -> String { "INBOX".into() }
fn default_message_num() -> u32 { 1 }

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
    #[serde(default = "default_ftp_command")]
    pub command: String,
    /// Remote path used for RETR, STOR, DELE, MKD, RMD, CWD
    #[serde(default)]
    pub remote_path: String,
    pub output_var: String,
    pub timeout_ms: u64,
    pub capture: bool,
}

impl Default for FtpRequestSettings {
    fn default() -> Self {
        Self {
            host: String::new(), port: 21,
            username: String::new(), password: String::new(),
            command: "LIST".into(), remote_path: String::new(),
            output_var: "FTP_RESPONSE".into(), timeout_ms: 10000, capture: false,
        }
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
        Self { host: String::new(), port: 22, username: String::new(), password: String::new(), command: "banner".into(), output_var: "SSH_RESPONSE".into(), timeout_ms: 10000, capture: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImapRequestSettings {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
    #[serde(default = "default_imap_command")]
    pub command: String,
    /// Mailbox name for SELECT / FETCH / SEARCH actions
    #[serde(default = "default_imap_mailbox")]
    pub mailbox: String,
    /// Message number for FETCH / DELE actions
    #[serde(default = "default_message_num")]
    pub message_num: u32,
    pub output_var: String,
    pub timeout_ms: u64,
    pub capture: bool,
}

impl Default for ImapRequestSettings {
    fn default() -> Self {
        Self {
            host: String::new(), port: 993,
            username: String::new(), password: String::new(),
            use_tls: true, command: "LOGIN".into(),
            mailbox: "INBOX".into(), message_num: 1,
            output_var: "IMAP_RESPONSE".into(), timeout_ms: 10000, capture: false,
        }
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
    /// "VERIFY" (just login check) or "SEND_EMAIL" (full email delivery)
    #[serde(default = "default_smtp_action")]
    pub action: String,
    /// Sender address for SEND_EMAIL (defaults to username if empty)
    #[serde(default)]
    pub from: String,
    /// Comma-separated recipients for SEND_EMAIL
    #[serde(default)]
    pub to: String,
    /// Subject line for SEND_EMAIL
    #[serde(default)]
    pub subject: String,
    /// Plain-text body for SEND_EMAIL
    #[serde(default)]
    pub body_template: String,
    pub output_var: String,
    pub timeout_ms: u64,
    pub capture: bool,
}

impl Default for SmtpRequestSettings {
    fn default() -> Self {
        Self {
            host: String::new(), port: 587,
            username: String::new(), password: String::new(),
            use_tls: true, command: "EHLO".into(),
            action: "VERIFY".into(),
            from: String::new(), to: String::new(),
            subject: String::new(), body_template: String::new(),
            output_var: "SMTP_RESPONSE".into(), timeout_ms: 10000, capture: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopRequestSettings {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
    #[serde(default = "default_pop_command")]
    pub command: String,
    /// Message number for RETR / DELE
    #[serde(default = "default_message_num")]
    pub message_num: u32,
    pub output_var: String,
    pub timeout_ms: u64,
    pub capture: bool,
}

impl Default for PopRequestSettings {
    fn default() -> Self {
        Self {
            host: String::new(), port: 995,
            username: String::new(), password: String::new(),
            use_tls: true, command: "STAT".into(),
            message_num: 1,
            output_var: "POP_RESPONSE".into(), timeout_ms: 10000, capture: false,
        }
    }
}
