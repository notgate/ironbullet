use std::sync::Arc;
use tokio::sync::Mutex;

use ironbullet::config::{self, RecentConfigEntry};
use ironbullet::export::format::RfxConfig;
use ironbullet::pipeline::Pipeline;

use super::{AppState, IpcResponse};

pub(super) fn get_config(
    state: Arc<Mutex<AppState>>,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let s = state.lock().await;
            let resp = IpcResponse::ok("config_loaded", serde_json::to_value(&s.config).unwrap_or_default());
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
        });
    }
}

pub(super) fn save_config(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let mut s = state.lock().await;
            use ironbullet::plugin::manager::PluginManager;
            // Merge incoming fields into existing config
            if let Some(v) = data.get("zoom").and_then(|v| v.as_u64()) { s.config.zoom = v as u32; }
            if let Some(v) = data.get("font_size").and_then(|v| v.as_u64()) { s.config.font_size = v as u32; }
            if let Some(v) = data.get("font_family").and_then(|v| v.as_str()) { s.config.font_family = v.to_string(); }
            if let Some(v) = data.get("default_threads").and_then(|v| v.as_u64()) { s.config.default_threads = v as u32; }
            if let Some(v) = data.get("left_panel_width").and_then(|v| v.as_u64()) { s.config.left_panel_width = v as u32; }
            if let Some(v) = data.get("bottom_panel_height").and_then(|v| v.as_u64()) { s.config.bottom_panel_height = v as u32; }
            if let Some(v) = data.get("show_block_palette").and_then(|v| v.as_bool()) { s.config.show_block_palette = v; }
            if let Some(v) = data.get("collections_path").and_then(|v| v.as_str()) { s.config.collections_path = v.to_string(); }
            if let Some(v) = data.get("default_wordlist_path").and_then(|v| v.as_str()) { s.config.default_wordlist_path = v.to_string(); }
            if let Some(v) = data.get("default_proxy_path").and_then(|v| v.as_str()) { s.config.default_proxy_path = v.to_string(); }
            if let Some(v) = data.get("plugins_path").and_then(|v| v.as_str()) {
                s.config.plugins_path = v.to_string();
                let mut pm = PluginManager::new();
                pm.scan_directory(v);
                s.plugin_manager = Arc::new(pm);
            }
            config::save_config(&s.config);
            let resp = IpcResponse::ok("config_saved", serde_json::json!({}));
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
        });
    }
}

pub(super) fn get_pipeline(
    state: Arc<Mutex<AppState>>,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let s = state.lock().await;
            let resp = IpcResponse::ok("pipeline_loaded", serde_json::to_value(&s.pipeline).unwrap_or_default());
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
        });
    }
}

pub(super) fn update_pipeline(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let mut s = state.lock().await;
            if let Ok(pipeline) = serde_json::from_value::<Pipeline>(data) {
                s.pipeline = pipeline;
            }
            let resp = IpcResponse::ok("pipeline_updated", serde_json::json!({}));
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
        });
    }
}

pub(super) fn save_pipeline(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let path = data.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let force_dialog = data.get("force_dialog").and_then(|v| v.as_bool()).unwrap_or(false);
            let save_path = if path.is_empty() || force_dialog {
                rfd::FileDialog::new()
                    .set_title("Save Config")
                    .add_filter("ironbullet config", &["rfx"])
                    .save_file()
                    .map(|p| p.display().to_string())
            } else {
                Some(path)
            };
            if let Some(save_path) = save_path {
                let mut s = state.lock().await;
                // Track current save path for auto-save
                s.pipeline_path = Some(save_path.clone());
                let config = RfxConfig::from_pipeline(&s.pipeline);
                match config.save_to_file(&save_path) {
                    Ok(()) => {
                        // Track in recent configs
                        let pipeline_name = s.pipeline.name.clone();
                        s.config.recent_configs.retain(|r| r.path != save_path);
                        s.config.recent_configs.insert(0, RecentConfigEntry {
                            path: save_path.clone(),
                            name: pipeline_name,
                            description: String::new(),
                            last_opened: chrono::Utc::now().to_rfc3339(),
                        });
                        if s.config.recent_configs.len() > 10 {
                            s.config.recent_configs.truncate(10);
                        }
                        s.config.last_config_path = save_path.clone();
                        config::save_config(&s.config);
                        let resp = IpcResponse::ok("pipeline_saved", serde_json::json!({ "path": save_path }));
                        eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                    }
                    Err(e) => {
                        let resp = IpcResponse::err("pipeline_saved", e.to_string());
                        eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                    }
                }
            }
        });
    }
}

pub(super) fn load_pipeline(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let load_path = data.get("path").and_then(|v| v.as_str()).map(|s| s.to_string());
            let pick_path = if load_path.is_some() {
                load_path.map(|p| std::path::PathBuf::from(p))
            } else {
                rfd::FileDialog::new()
                    .set_title("Open Config")
                    .add_filter("ironbullet config", &["rfx"])
                    .pick_file()
            };
            if let Some(path) = pick_path {
                let path_str = path.display().to_string();
                match RfxConfig::load_from_file(&path_str) {
                    Ok(config) => {
                        let mut s = state.lock().await;
                        s.pipeline = config.pipeline;
                        s.pipeline_path = Some(path_str.clone());
                        // Track in recent configs
                        let pipeline_name = s.pipeline.name.clone();
                        s.config.recent_configs.retain(|r| r.path != path_str);
                        s.config.recent_configs.insert(0, RecentConfigEntry {
                            path: path_str.clone(),
                            name: pipeline_name,
                            description: String::new(),
                            last_opened: chrono::Utc::now().to_rfc3339(),
                        });
                        if s.config.recent_configs.len() > 10 {
                            s.config.recent_configs.truncate(10);
                        }
                        s.config.last_config_path = path_str.clone();
                        config::save_config(&s.config);
                        let mut pipeline_val = serde_json::to_value(&s.pipeline).unwrap_or_default();
                        if let Some(obj) = pipeline_val.as_object_mut() {
                            obj.insert("_file_path".to_string(), serde_json::Value::String(path_str));
                        }
                        let resp = IpcResponse::ok("pipeline_loaded", pipeline_val);
                        eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                    }
                    Err(e) => {
                        let resp = IpcResponse::err("pipeline_loaded", e.to_string());
                        eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                    }
                }
            }
        });
    }
}

pub(super) fn get_recent_configs(
    state: Arc<Mutex<AppState>>,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let s = state.lock().await;
            let resp = IpcResponse::ok("recent_configs", serde_json::to_value(&s.config.recent_configs).unwrap_or_default());
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
        });
    }
}

pub(super) fn setup_default_dirs(
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            // Create default dirs next to the executable
            let base = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.to_path_buf()))
                .unwrap_or_else(|| std::path::PathBuf::from("."));

            let dirs = ["wordlists", "proxies", "configs", "results"];
            let mut created: Vec<String> = Vec::new();
            let mut paths: std::collections::HashMap<String, String> = std::collections::HashMap::new();

            for name in &dirs {
                let dir = base.join(name);
                if !dir.exists() {
                    if let Ok(()) = std::fs::create_dir_all(&dir) {
                        created.push(name.to_string());
                    }
                }
                paths.insert(name.to_string(), dir.display().to_string());
            }

            let resp = IpcResponse::ok("dirs_created", serde_json::json!({
                "created": created,
                "paths": paths,
            }));
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
        });
    }
}
