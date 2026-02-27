use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyCheckSettings {
    pub keychains: Vec<Keychain>,
    /// When true, a Fail result immediately halts the pipeline for this data entry —
    /// no subsequent Parse/function blocks are executed.  Default false for
    /// backward compatibility (existing pipelines keep running after Fail).
    #[serde(default)]
    pub stop_on_fail: bool,
}

impl Default for KeyCheckSettings {
    fn default() -> Self {
        Self {
            keychains: vec![Keychain::default()],
            stop_on_fail: false,
        }
    }
}

/// Whether all conditions in a keychain must match (AND) or any single one (OR).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum KeychainMode {
    #[default]
    And,
    Or,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keychain {
    pub result: crate::pipeline::BotStatus,
    pub conditions: Vec<KeyCondition>,
    /// How conditions are combined. Defaults to AND for backward compatibility.
    #[serde(default)]
    pub mode: KeychainMode,
}

impl Default for Keychain {
    fn default() -> Self {
        Self {
            result: crate::pipeline::BotStatus::Success,
            conditions: vec![KeyCondition::default()],
            mode: KeychainMode::And,
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
