use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::pipeline::{OutputFormat, Pipeline, ProxySettings};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub name: String,
    pub config_path: Option<String>,
    pub pipeline: Pipeline,
    pub data_source: DataSource,
    pub proxy_source: ProxySourceConfig,
    pub state: JobState,
    pub start_condition: StartCondition,
    pub hit_outputs: Vec<HitOutput>,
    pub thread_count: usize,
    pub created: DateTime<Utc>,
    pub started: Option<DateTime<Utc>>,
    pub completed: Option<DateTime<Utc>>,
    pub stats: super::RunnerStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    pub source_type: DataSourceType,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSourceType {
    File,
    Folder,
    Url,
    Inline,
    Range,
    Combinations,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySourceConfig {
    pub settings: ProxySettings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobState {
    Queued,
    Waiting,
    Running,
    Paused,
    Completed,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StartCondition {
    Immediate,
    Delayed { delay_secs: u64 },
    Scheduled { at: DateTime<Utc> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HitOutput {
    FileSystem {
        directory: String,
        format: OutputFormat,
    },
    DiscordWebhook {
        webhook_url: String,
        template: String,
    },
    TelegramBot {
        bot_token: String,
        chat_id: String,
        template: String,
    },
    CustomWebhook {
        url: String,
        method: String,
        body_template: String,
    },
}

impl Default for Job {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "New Job".into(),
            config_path: None,
            pipeline: Pipeline::default(),
            data_source: DataSource {
                source_type: DataSourceType::File,
                value: String::new(),
            },
            proxy_source: ProxySourceConfig {
                settings: ProxySettings::default(),
            },
            state: JobState::Queued,
            start_condition: StartCondition::Immediate,
            hit_outputs: vec![HitOutput::FileSystem {
                directory: "results".into(),
                format: OutputFormat::Txt,
            }],
            thread_count: 100,
            created: Utc::now(),
            started: None,
            completed: None,
            stats: super::RunnerStats {
                total: 0,
                processed: 0,
                hits: 0,
                fails: 0,
                bans: 0,
                retries: 0,
                errors: 0,
                cpm: 0.0,
                active_threads: 0,
                elapsed_secs: 0.0,
            },
        }
    }
}

