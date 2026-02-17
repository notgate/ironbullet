use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Sidecar error: {0}")]
    Sidecar(String),

    #[error("Pipeline error: {0}")]
    Pipeline(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Variable not found: {0}")]
    VariableNotFound(String),

    #[error("Block execution failed: {0}")]
    BlockExecution(String),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
