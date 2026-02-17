use serde::{Deserialize, Serialize};
use crate::pipeline::Pipeline;

/// .rfx config file format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfxConfig {
    pub version: u32,
    pub metadata: RfxMetadata,
    pub pipeline: Pipeline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfxMetadata {
    pub name: String,
    pub author: String,
    pub created: String,
    pub modified: String,
}

impl RfxConfig {
    pub fn from_pipeline(pipeline: &Pipeline) -> Self {
        Self {
            version: 1,
            metadata: RfxMetadata {
                name: pipeline.name.clone(),
                author: pipeline.author.clone(),
                created: pipeline.created.to_rfc3339(),
                modified: pipeline.modified.to_rfc3339(),
            },
            pipeline: pipeline.clone(),
        }
    }

    pub fn save_to_file(&self, path: &str) -> crate::error::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> crate::error::Result<Self> {
        let data = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&data)?;
        Ok(config)
    }
}
