use std::io::Read;

use crate::pipeline::*;

use super::lolicode::parse_lolicode_blocks;
use super::ImportResult;

// ────────────────────────────────────────────────────────────
// OB2 .opk (ZIP archive) importer
// ────────────────────────────────────────────────────────────

/// Import an OB2 .opk file (ZIP archive containing script.loli + metadata + settings)
pub(super) fn import_opk(bytes: &[u8]) -> Result<ImportResult, String> {
    let reader = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|e| format!("Failed to open .opk archive: {}", e))?;

    let mut script_loli = String::new();
    let mut metadata_json = String::new();
    let mut settings_json = String::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read archive entry: {}", e))?;
        let name = file.name().to_string();
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| format!("Failed to read {}: {}", name, e))?;

        match name.as_str() {
            "script.loli" => script_loli = content,
            "metadata.json" => metadata_json = content,
            "settings.json" => settings_json = content,
            _ => {} // readme.md, startup.loli, etc
        }
    }

    if script_loli.is_empty() {
        return Err("No script.loli found in .opk archive".into());
    }

    let mut pipeline = Pipeline::default();
    let mut warnings = Vec::new();

    // Parse metadata → name, author
    if !metadata_json.is_empty() {
        if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&metadata_json) {
            if let Some(name) = meta.get("Name").and_then(|n| n.as_str()) {
                pipeline.name = name.to_string();
            }
            if let Some(author) = meta.get("Author").and_then(|a| a.as_str()) {
                pipeline.author = author.to_string();
            }
        }
    }

    // Parse settings → runner/proxy/data settings
    if !settings_json.is_empty() {
        apply_ob2_settings(&mut pipeline, &settings_json);
    }

    // Parse the LoliCode script → blocks
    pipeline.blocks = parse_lolicode_blocks(&script_loli, &mut warnings)?;

    Ok(ImportResult { pipeline, warnings, security_issues: Vec::new() })
}

/// Map OB2 settings.json fields to Pipeline settings
pub(super) fn apply_ob2_settings(pipeline: &mut Pipeline, json_str: &str) {
    let json: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return,
    };

    // GeneralSettings.SuggestedBots → threads
    if let Some(general) = json.get("GeneralSettings") {
        if let Some(bots) = general.get("SuggestedBots").and_then(|v| v.as_u64()) {
            pipeline.runner_settings.threads = bots as u32;
        }
    }

    // ProxySettings.UseProxies → proxy mode
    if let Some(proxy) = json.get("ProxySettings") {
        if proxy.get("UseProxies").and_then(|v| v.as_bool()).unwrap_or(false) {
            pipeline.proxy_settings.proxy_mode = ProxyMode::Rotate;
        }
        if let Some(ban_loop) = proxy.get("BanLoopEvasion").and_then(|v| v.as_u64()) {
            pipeline.proxy_settings.ban_duration_secs = ban_loop;
        }
    }

    // DataSettings.AllowedWordlistTypes → wordlist type + slices
    if let Some(data) = json.get("DataSettings") {
        if let Some(types) = data.get("AllowedWordlistTypes").and_then(|v| v.as_array()) {
            if let Some(first) = types.first().and_then(|v| v.as_str()) {
                pipeline.data_settings.wordlist_type = first.to_string();
                // Set default slices based on wordlist type
                match first {
                    "Credentials" => {
                        pipeline.data_settings.separator = ':';
                        pipeline.data_settings.slices = vec!["USER".into(), "PASS".into()];
                    }
                    "Emails" => {
                        pipeline.data_settings.separator = ':';
                        pipeline.data_settings.slices = vec!["EMAIL".into()];
                    }
                    _ => {}
                }
            }
        }
    }
}
