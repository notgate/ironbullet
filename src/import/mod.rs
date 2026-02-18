mod helpers;
mod lolicode;
mod ob2_json;
mod opk;
mod security;
mod svb;

#[cfg(test)]
mod tests;

use crate::pipeline::*;

use self::ob2_json::import_openbullet_json;
use self::opk::import_opk;
use self::svb::import_svb;

// ────────────────────────────────────────────────────────────
// Public types
// ────────────────────────────────────────────────────────────

/// Result of importing a config, containing the parsed pipeline and any warnings
pub struct ImportResult {
    pub pipeline: Pipeline,
    pub warnings: Vec<String>,
    pub security_issues: Vec<SecurityIssue>,
}

/// A security issue found during config scanning
#[derive(Debug, Clone, serde::Serialize)]
pub struct SecurityIssue {
    pub severity: SecuritySeverity,
    pub title: String,
    pub description: String,
    pub code_snippet: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum SecuritySeverity {
    Critical,
    Warning,
}

// ────────────────────────────────────────────────────────────
// Public entry points
// ────────────────────────────────────────────────────────────

/// Import from raw file bytes — auto-detects format (ZIP for .opk, text for .loli/.json/.svb)
pub fn import_config_bytes(bytes: &[u8]) -> Result<ImportResult, String> {
    // Check for ZIP magic bytes (PK\x03\x04)
    let mut result = if bytes.len() >= 4 && bytes[0] == b'P' && bytes[1] == b'K' && bytes[2] == 3 && bytes[3] == 4 {
        import_opk(bytes)?
    } else {
        let content = std::str::from_utf8(bytes)
            .map_err(|e| format!("File is not valid UTF-8 text: {}", e))?;
        import_config(content)?
    };

    // Run security scan
    result.security_issues = scan_config_security(&result.pipeline);
    Ok(result)
}

/// Auto-detect text format and import
pub fn import_config(content: &str) -> Result<ImportResult, String> {
    // SVB format (SilverBullet) — starts with [SETTINGS]
    if content.trim_start().starts_with("[SETTINGS]") {
        return import_svb(content);
    }
    // Try JSON first (OpenBullet2 JSON format)
    if content.trim_start().starts_with('{') {
        return import_openbullet_json(content);
    }
    // Otherwise try LoliCode
    lolicode::import_lolicode(content)
}

// Re-export publicly accessible items (they were `pub` in the original monolithic file)
pub use self::lolicode::import_lolicode;
pub use self::security::scan_config_security;
