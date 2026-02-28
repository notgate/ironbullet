pub mod block;
pub mod variable;
pub mod engine;
pub mod codegen;
pub mod random_data;
pub mod tls_profiles;
pub mod datadome;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use block::Block;

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
pub struct Pipeline {
    #[serde(deserialize_with = "deserialize_uuid_tolerant")]
    pub id: Uuid,
    pub name: String,
    pub author: String,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub blocks: Vec<Block>,
    pub startup_blocks: Vec<Block>,
    pub data_settings: DataSettings,
    pub proxy_settings: ProxySettings,
    pub browser_settings: BrowserSettings,
    #[serde(default)]
    pub runner_settings: RunnerSettings,
    #[serde(default)]
    pub output_settings: OutputSettings,
}

impl Default for Pipeline {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "New Config".into(),
            author: String::new(),
            created: Utc::now(),
            modified: Utc::now(),
            blocks: Vec::new(),
            startup_blocks: Vec::new(),
            data_settings: DataSettings::default(),
            proxy_settings: ProxySettings::default(),
            browser_settings: BrowserSettings::default(),
            runner_settings: RunnerSettings::default(),
            output_settings: OutputSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSettings {
    pub wordlist_type: String,
    pub separator: char,
    pub slices: Vec<String>,
}

impl Default for DataSettings {
    fn default() -> Self {
        Self {
            wordlist_type: "Credentials".into(),
            separator: ':',
            slices: vec!["USER".into(), "PASS".into()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySettings {
    pub proxy_mode: ProxyMode,
    pub proxy_sources: Vec<ProxySource>,
    pub ban_duration_secs: u64,
    pub max_retries_before_ban: u32,
    /// Max checks per minute per proxy (for CpmLimited mode)
    #[serde(default)]
    pub cpm_per_proxy: u32,
    /// Named proxy groups (like OB2)
    #[serde(default)]
    pub proxy_groups: Vec<ProxyGroup>,
    /// Active proxy group name (empty = use default sources)
    #[serde(default)]
    pub active_group: String,
}

impl Default for ProxySettings {
    fn default() -> Self {
        Self {
            proxy_mode: ProxyMode::None,
            proxy_sources: Vec::new(),
            ban_duration_secs: 300,
            max_retries_before_ban: 3,
            cpm_per_proxy: 0,
            proxy_groups: Vec::new(),
            active_group: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyGroup {
    pub name: String,
    pub mode: ProxyMode,
    pub sources: Vec<ProxySource>,
    #[serde(default)]
    pub cpm_per_proxy: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxyMode {
    None,
    /// Random proxy per request
    Rotate,
    /// One proxy per account check (sticky per data line)
    Sticky,
    /// Limit checks-per-minute per proxy
    CpmLimited,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySource {
    pub source_type: ProxySourceType,
    pub value: String,
    /// Refresh interval in seconds (for URL sources)
    #[serde(default)]
    pub refresh_interval_secs: u64,
    /// If set, applies this protocol to all plain host:port lines in this source that
    /// don't already carry a protocol prefix. Accepts "Http", "Https", "Socks4", "Socks5".
    #[serde(default)]
    pub default_proxy_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxySourceType {
    File,
    Url,
    Inline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSettings {
    pub browser: String,
    pub ja3: Option<String>,
    pub http2_fingerprint: Option<String>,
    pub user_agent: Option<String>,
}

impl Default for BrowserSettings {
    fn default() -> Self {
        Self {
            browser: "chrome".into(),
            ja3: None,
            http2_fingerprint: None,
            user_agent: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerSettings {
    /// Number of threads / concurrent bots
    pub threads: u32,
    /// Skip the first N data lines
    pub skip: u32,
    /// Maximum data lines to process (0 = all)
    pub take: u32,
    /// Statuses that count as "continue" (re-queue the data line)
    pub continue_statuses: Vec<BotStatus>,
    /// Custom status label for Custom result
    pub custom_status_name: String,
    /// Maximum retries per data line before marking as error
    pub max_retries: u32,
    /// Concurrent uses per proxy (0 = unlimited)
    pub concurrent_per_proxy: u32,
    /// Start threads gradually instead of all at once
    #[serde(default)]
    pub start_threads_gradually: bool,
    /// Delay between starting each thread (ms)
    #[serde(default = "default_gradual_delay")]
    pub gradual_delay_ms: u32,
    /// Automatically adjust thread count for best CPM
    #[serde(default)]
    pub automatic_thread_count: bool,
    /// Reduce threads when retry rate is high
    #[serde(default)]
    pub lower_threads_on_retry: bool,
    /// Percentage to reduce threads by when retry rate is high (0-100)
    #[serde(default = "default_retry_reduction")]
    pub retry_thread_reduction_pct: u32,
    /// Pause execution when rate-limited
    #[serde(default)]
    pub pause_on_ratelimit: bool,
    /// Run without proxies even if proxies are configured
    #[serde(default)]
    pub only_proxyless: bool,
}

fn default_gradual_delay() -> u32 { 100 }
fn default_retry_reduction() -> u32 { 25 }

impl Default for RunnerSettings {
    fn default() -> Self {
        Self {
            threads: 100,
            skip: 0,
            take: 0,
            continue_statuses: vec![BotStatus::Retry],
            custom_status_name: "CUSTOM".into(),
            max_retries: 3,
            concurrent_per_proxy: 0,
            start_threads_gradually: true,
            gradual_delay_ms: 100,
            automatic_thread_count: false,
            lower_threads_on_retry: false,
            retry_thread_reduction_pct: 25,
            pause_on_ratelimit: false,
            only_proxyless: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSettings {
    /// Save hits to text files
    #[serde(default = "default_true")]
    pub save_to_file: bool,
    /// Save hits to database (SQLite)
    #[serde(default)]
    pub save_to_database: bool,
    /// Include full response body in output (debug mode)
    #[serde(default)]
    pub include_response: bool,
    /// Output directory for text files
    #[serde(default = "default_output_dir")]
    pub output_directory: String,
    /// Custom output format template
    #[serde(default = "default_output_format")]
    pub output_format: String,
    /// Database connection string (SQLite path)
    #[serde(default)]
    pub database_path: String,
    /// Output file format (Txt, Csv, Json)
    #[serde(default)]
    pub output_format_type: OutputFormat,
    /// Capture filters applied before writing output
    #[serde(default)]
    pub capture_filters: Vec<CaptureFilter>,
}

fn default_true() -> bool { true }
fn default_output_dir() -> String { "results".into() }
fn default_output_format() -> String { "{data} | {captures}".into() }

impl Default for OutputSettings {
    fn default() -> Self {
        Self {
            save_to_file: true,
            save_to_database: false,
            include_response: false,
            output_directory: "results".into(),
            output_format: "{data} | {captures}".into(),
            database_path: String::new(),
            output_format_type: OutputFormat::Txt,
            capture_filters: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Txt,
    Csv,
    Json,
}

impl Default for OutputFormat {
    fn default() -> Self { Self::Txt }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureFilter {
    /// Variable name to filter on ("*" for all variables)
    pub variable_name: String,
    pub filter_type: CaptureFilterType,
    pub value: String,
    pub negate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CaptureFilterType {
    Contains,
    Equals,
    StartsWith,
    EndsWith,
    MatchesRegex,
    MinLength,
    MaxLength,
    NotEmpty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BotStatus {
    None,
    Success,
    Fail,
    Ban,
    Retry,
    Error,
    Custom,
}
