use serde::{Deserialize, Serialize};

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
    pub result: crate::pipeline::BotStatus,
    pub conditions: Vec<KeyCondition>,
}

impl Default for Keychain {
    fn default() -> Self {
        Self {
            result: crate::pipeline::BotStatus::Success,
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
