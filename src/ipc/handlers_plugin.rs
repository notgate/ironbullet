use std::sync::Arc;
use tokio::sync::Mutex;

use ironbullet::plugin::manager::PluginManager;

use super::{AppState, IpcResponse};

pub(super) fn get_plugin_blocks(
    state: Arc<Mutex<AppState>>,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let s = state.lock().await;
            let blocks = s.plugin_manager.all_block_infos();
            let plugins = s.plugin_manager.all_plugin_metas();
            let resp = IpcResponse::ok("plugin_blocks_loaded", serde_json::json!({
                "blocks": serde_json::to_value(&blocks).unwrap_or_default(),
                "plugins": serde_json::to_value(&plugins).unwrap_or_default(),
            }));
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
        });
    }
}

pub(super) fn import_plugin(
    state: Arc<Mutex<AppState>>,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let pick_path = rfd::FileDialog::new()
                .set_title("Import Plugin")
                .add_filter("Plugin DLL", &["dll"])
                .add_filter("All files", &["*"])
                .pick_file();
            if let Some(src_path) = pick_path {
                let mut s = state.lock().await;
                // Resolve plugins directory
                let plugins_dir = if s.config.plugins_path.is_empty() {
                    let dir = std::env::current_exe()
                        .ok()
                        .and_then(|e| e.parent().map(|p| p.join("plugins")))
                        .unwrap_or_else(|| std::path::PathBuf::from("plugins"));
                    s.config.plugins_path = dir.display().to_string();
                    dir
                } else {
                    std::path::PathBuf::from(&s.config.plugins_path)
                };
                // Create plugins dir if needed
                let _ = std::fs::create_dir_all(&plugins_dir);
                // Copy DLL
                let file_name = src_path.file_name().unwrap_or_default();
                let dest = plugins_dir.join(file_name);
                match std::fs::copy(&src_path, &dest) {
                    Ok(_) => {
                        // Rescan plugins
                        let path = s.config.plugins_path.clone();
                        let mut pm = PluginManager::new();
                        pm.scan_directory(&path);
                        s.plugin_manager = Arc::new(pm);
                        let blocks = s.plugin_manager.all_block_infos();
                        let plugins = s.plugin_manager.all_plugin_metas();
                        let resp = IpcResponse::ok("plugin_blocks_loaded", serde_json::json!({
                            "blocks": serde_json::to_value(&blocks).unwrap_or_default(),
                            "plugins": serde_json::to_value(&plugins).unwrap_or_default(),
                        }));
                        eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                    }
                    Err(e) => {
                        let resp = IpcResponse::err("import_error", format!("Failed to copy plugin: {}", e));
                        eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                    }
                }
            }
        });
    }
}

pub(super) fn reload_plugins(
    state: Arc<Mutex<AppState>>,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let mut s = state.lock().await;
            let path = s.config.plugins_path.clone();
            let mut pm = PluginManager::new();
            pm.scan_directory(&path);
            s.plugin_manager = Arc::new(pm);
            let blocks = s.plugin_manager.all_block_infos();
            let plugins = s.plugin_manager.all_plugin_metas();
            let resp = IpcResponse::ok("plugin_blocks_loaded", serde_json::json!({
                "blocks": serde_json::to_value(&blocks).unwrap_or_default(),
                "plugins": serde_json::to_value(&plugins).unwrap_or_default(),
            }));
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
        });
    }
}

pub(super) fn compile_plugin(
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let project_dir = data.get("project_dir").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let lib_rs = data.get("lib_rs").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let cargo_toml = data.get("cargo_toml").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let release = data.get("release").and_then(|v| v.as_bool()).unwrap_or(true);

            // Determine project path (use temp dir if no project_dir specified)
            let dir = if project_dir.is_empty() {
                let tmp = std::env::temp_dir().join("ironbullet-plugin-build");
                let _ = std::fs::create_dir_all(&tmp);
                tmp
            } else {
                std::path::PathBuf::from(&project_dir)
            };

            // Write files
            let src_dir = dir.join("src");
            let _ = std::fs::create_dir_all(&src_dir);
            if let Err(e) = std::fs::write(dir.join("Cargo.toml"), &cargo_toml) {
                let resp = IpcResponse::ok("compile_output", serde_json::json!({
                    "line": format!("error: Failed to write Cargo.toml: {}", e),
                    "done": true, "success": false,
                }));
                eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                return;
            }
            if let Err(e) = std::fs::write(src_dir.join("lib.rs"), &lib_rs) {
                let resp = IpcResponse::ok("compile_output", serde_json::json!({
                    "line": format!("error: Failed to write lib.rs: {}", e),
                    "done": true, "success": false,
                }));
                eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                return;
            }

            // Send start message
            let start_resp = IpcResponse::ok("compile_output", serde_json::json!({
                "line": format!("$ cargo build {} --manifest-path {}", if release { "--release" } else { "" }, dir.join("Cargo.toml").display()),
                "done": false, "success": false,
            }));
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&start_resp).unwrap_or_default()));

            // Run cargo build
            let mut cmd = std::process::Command::new("cargo");
            cmd.arg("build");
            if release { cmd.arg("--release"); }
            cmd.arg("--manifest-path").arg(dir.join("Cargo.toml"));
            cmd.stdout(std::process::Stdio::piped());
            cmd.stderr(std::process::Stdio::piped());

            match cmd.output() {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    // Send stdout lines
                    for line in stdout.lines() {
                        let resp = IpcResponse::ok("compile_output", serde_json::json!({
                            "line": line, "done": false, "success": false,
                        }));
                        eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                    }
                    // Send stderr lines
                    for line in stderr.lines() {
                        let resp = IpcResponse::ok("compile_output", serde_json::json!({
                            "line": line, "done": false, "success": false,
                        }));
                        eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                    }
                    let success = output.status.success();
                    let profile = if release { "release" } else { "debug" };
                    let dll_path = if success {
                        let target_dir = dir.join("target").join(profile);
                        // Find .dll file
                        std::fs::read_dir(&target_dir)
                            .ok()
                            .and_then(|entries| {
                                entries.filter_map(|e| e.ok())
                                    .find(|e| e.path().extension().map(|x| x == "dll").unwrap_or(false))
                                    .map(|e| e.path().display().to_string())
                            })
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };
                    let done_resp = IpcResponse::ok("compile_output", serde_json::json!({
                        "line": if success { format!("Build succeeded! DLL: {}", dll_path) } else { "Build failed.".to_string() },
                        "done": true, "success": success, "dll_path": dll_path,
                    }));
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&done_resp).unwrap_or_default()));
                }
                Err(e) => {
                    let resp = IpcResponse::ok("compile_output", serde_json::json!({
                        "line": format!("error: Failed to run cargo: {} -- is Rust installed?", e),
                        "done": true, "success": false,
                    }));
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                }
            }
        });
    }
}

pub(super) fn save_plugin_files(
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let lib_rs = data.get("lib_rs").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let cargo_toml = data.get("cargo_toml").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let dir_str = data.get("dir").and_then(|v| v.as_str()).unwrap_or("").to_string();

            let dir = if dir_str.is_empty() {
                // Use file picker
                let pick = rfd::FileDialog::new()
                    .set_title("Choose plugin project directory")
                    .pick_folder();
                match pick {
                    Some(p) => p,
                    None => {
                        let resp = IpcResponse::err("save_plugin_result", "No directory selected".into());
                        eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                        return;
                    }
                }
            } else {
                std::path::PathBuf::from(&dir_str)
            };

            let src_dir = dir.join("src");
            let _ = std::fs::create_dir_all(&src_dir);
            let _ = std::fs::write(dir.join("Cargo.toml"), &cargo_toml);
            let _ = std::fs::write(src_dir.join("lib.rs"), &lib_rs);

            let resp = IpcResponse::ok("save_plugin_result", serde_json::json!({
                "dir": dir.display().to_string(),
            }));
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
        });
    }
}
