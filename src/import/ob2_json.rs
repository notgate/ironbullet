use crate::pipeline::block::*;
use crate::pipeline::*;

use super::ImportResult;

// ────────────────────────────────────────────────────────────
// Legacy JSON-based OB2 importer (for non-LoliCode .json configs)
// ────────────────────────────────────────────────────────────

pub(super) fn import_openbullet_json(content: &str) -> Result<ImportResult, String> {
    let json: serde_json::Value = serde_json::from_str(content)
        .map_err(|e| format!("Failed to parse config JSON: {}", e))?;

    let mut pipeline = Pipeline::default();
    pipeline.name = json.get("Settings")
        .and_then(|s| s.get("Name"))
        .and_then(|n| n.as_str())
        .unwrap_or("Imported OB2 Config")
        .to_string();

    let mut blocks = Vec::new();

    if let Some(ob_blocks) = json.get("Settings")
        .and_then(|s| s.get("Blocks"))
        .and_then(|b| b.as_array())
    {
        for ob_block in ob_blocks {
            let block_type = ob_block.get("Type").and_then(|t| t.as_str()).unwrap_or("");
            match block_type {
                "HttpRequestBlock" | "Request" => {
                    let mut block = Block::new(BlockType::HttpRequest);
                    if let BlockSettings::HttpRequest(ref mut s) = block.settings {
                        s.method = ob_block.get("Method")
                            .and_then(|m| m.as_str())
                            .unwrap_or("GET")
                            .to_string();
                        s.url = ob_block.get("Url")
                            .and_then(|u| u.as_str())
                            .unwrap_or("")
                            .to_string();
                    }
                    blocks.push(block);
                }
                "KeycheckBlock" | "Keycheck" => {
                    blocks.push(Block::new(BlockType::KeyCheck));
                }
                "ParseBlock" => {
                    let mode = ob_block.get("Mode").and_then(|m| m.as_str()).unwrap_or("");
                    match mode {
                        "LR" => blocks.push(Block::new(BlockType::ParseLR)),
                        "JSON" => blocks.push(Block::new(BlockType::ParseJSON)),
                        "Regex" => blocks.push(Block::new(BlockType::ParseRegex)),
                        "CSS" => blocks.push(Block::new(BlockType::ParseCSS)),
                        _ => blocks.push(Block::new(BlockType::ParseLR)),
                    }
                }
                _ => {}
            }
        }
    }

    if blocks.is_empty() {
        return Err("No recognizable blocks found in OpenBullet config".into());
    }

    pipeline.blocks = blocks;
    Ok(ImportResult { pipeline, warnings: Vec::new(), security_issues: Vec::new() })
}
