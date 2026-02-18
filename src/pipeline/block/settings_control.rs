use serde::{Deserialize, Serialize};

use super::Block;
use super::settings_check::KeyCondition;

// ── If/Else ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfElseSettings {
    pub condition: KeyCondition,
    pub true_blocks: Vec<Block>,
    pub false_blocks: Vec<Block>,
}

impl Default for IfElseSettings {
    fn default() -> Self {
        Self {
            condition: KeyCondition::default(),
            true_blocks: Vec::new(),
            false_blocks: Vec::new(),
        }
    }
}

// ── Loop ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopSettings {
    pub loop_type: LoopType,
    pub list_var: String,
    pub item_var: String,
    pub count: u32,
    pub blocks: Vec<Block>,
}

impl Default for LoopSettings {
    fn default() -> Self {
        Self {
            loop_type: LoopType::ForEach,
            list_var: String::new(),
            item_var: "ITEM".into(),
            count: 1,
            blocks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoopType {
    ForEach,
    Repeat,
}

// ── Delay ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelaySettings {
    pub min_ms: u64,
    pub max_ms: u64,
}

impl Default for DelaySettings {
    fn default() -> Self {
        Self {
            min_ms: 1000,
            max_ms: 1000,
        }
    }
}

// ── Script ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptSettings {
    pub code: String,
    pub output_var: String,
    pub capture: bool,
}

impl Default for ScriptSettings {
    fn default() -> Self {
        Self {
            code: String::new(),
            output_var: "RESULT".into(),
            capture: false,
        }
    }
}

// ── Log ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSettings {
    pub message: String,
}

impl Default for LogSettings {
    fn default() -> Self {
        Self {
            message: String::new(),
        }
    }
}

// ── Set Variable ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetVariableSettings {
    pub name: String,
    pub value: String,
    pub capture: bool,
}

impl Default for SetVariableSettings {
    fn default() -> Self {
        Self {
            name: String::new(),
            value: String::new(),
            capture: false,
        }
    }
}
