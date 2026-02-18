use std::sync::Arc;
use tokio::sync::Mutex;

use ironbullet::export::rust_codegen;
use ironbullet::pipeline::Pipeline;

use super::{AppState, IpcResponse};

pub(super) fn generate_code(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let pipeline = if let Some(p) = data.get("pipeline") {
                serde_json::from_value::<Pipeline>(p.clone()).ok()
            } else {
                None
            };
            let pipeline = match &pipeline {
                Some(p) => p,
                None => {
                    let s = state.lock().await;
                    let code = rust_codegen::generate_rust_code(&s.pipeline);
                    let resp = IpcResponse::ok("code_generated", serde_json::json!({ "code": code }));
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                    return;
                }
            };
            let code = rust_codegen::generate_rust_code(pipeline);
            let resp = IpcResponse::ok("code_generated", serde_json::json!({ "code": code }));
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
        });
    }
}

pub(super) fn save_code(
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let code = data.get("code").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let save_path = rfd::FileDialog::new()
                .set_title("Save Generated Code")
                .add_filter("Rust source", &["rs"])
                .add_filter("All files", &["*"])
                .save_file();
            if let Some(path) = save_path {
                match std::fs::write(&path, &code) {
                    Ok(()) => {
                        let resp = IpcResponse::ok("code_saved", serde_json::json!({ "path": path.display().to_string() }));
                        eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                    }
                    Err(e) => {
                        let resp = IpcResponse::err("code_saved", e.to_string());
                        eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                    }
                }
            }
        });
    }
}

pub(super) fn import_config(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let path = data.get("path").and_then(|v| v.as_str()).map(|s| s.to_string());
            let pick_path = if let Some(p) = path {
                Some(std::path::PathBuf::from(p))
            } else {
                rfd::FileDialog::new()
                    .set_title("Import Config")
                    .add_filter("Config files (*.svb, *.opk, *.loli)", &["svb", "opk", "loli", "json"])
                    .add_filter("All files", &["*"])
                    .pick_file()
            };
            if let Some(path) = pick_path {
                match std::fs::read(&path) {
                    Ok(bytes) => {
                        match ironbullet::import::import_config_bytes(&bytes) {
                            Ok(result) => {
                                let mut s = state.lock().await;
                                s.pipeline = result.pipeline;
                                let mut data = serde_json::to_value(&s.pipeline).unwrap_or_default();
                                if !result.warnings.is_empty() {
                                    data["_import_warnings"] = serde_json::to_value(&result.warnings).unwrap_or_default();
                                }
                                if !result.security_issues.is_empty() {
                                    data["_security_issues"] = serde_json::to_value(&result.security_issues).unwrap_or_default();
                                }
                                let resp = IpcResponse::ok("pipeline_loaded", data);
                                eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                            }
                            Err(e) => {
                                let resp = IpcResponse::err("pipeline_loaded", format!("Import failed: {}", e));
                                eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                            }
                        }
                    }
                    Err(e) => {
                        let resp = IpcResponse::err("pipeline_loaded", format!("Failed to read file: {}", e));
                        eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                    }
                }
            }
        });
    }
}

pub(super) fn list_collections(
    state: Arc<Mutex<AppState>>,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let s = state.lock().await;
            let path = s.config.collections_path.clone();
            drop(s);

            let mut configs: Vec<serde_json::Value> = Vec::new();
            if !path.is_empty() {
                if let Ok(entries) = std::fs::read_dir(&path) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.extension().map(|e| e == "rfx").unwrap_or(false) {
                            let name = p.file_stem().unwrap_or_default().to_string_lossy().to_string();
                            configs.push(serde_json::json!({
                                "path": p.display().to_string(),
                                "name": name,
                            }));
                        }
                    }
                }
            }
            configs.sort_by(|a, b| {
                let na = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let nb = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
                na.to_lowercase().cmp(&nb.to_lowercase())
            });
            let resp = IpcResponse::ok("collections_list", serde_json::json!(configs));
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
        });
    }
}

pub(super) fn browse_folder(
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let field = data.get("field").and_then(|v| v.as_str()).unwrap_or("").to_string();
            if let Some(path) = rfd::FileDialog::new().set_title("Select Folder").pick_folder() {
                let resp = IpcResponse::ok("folder_selected", serde_json::json!({
                    "field": field,
                    "path": path.display().to_string(),
                }));
                eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
            }
        });
    }
}

pub(super) fn browse_file(
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let field = data.get("field").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let mut dialog = rfd::FileDialog::new();
            dialog = match field.as_str() {
                "wordlist" => dialog.set_title("Select Wordlist")
                    .add_filter("Text files", &["txt", "csv", "lst"])
                    .add_filter("All files", &["*"]),
                "proxies" => dialog.set_title("Select Proxy File")
                    .add_filter("Text files", &["txt", "csv", "lst"])
                    .add_filter("All files", &["*"]),
                _ => dialog.set_title("Select File")
                    .add_filter("All files", &["*"]),
            };

            if let Some(path) = dialog.pick_file() {
                let resp = IpcResponse::ok("file_selected", serde_json::json!({
                    "field": field,
                    "path": path.display().to_string(),
                }));
                eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
            }
        });
    }
}
