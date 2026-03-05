mod block_tree;
mod handlers_block;
mod handlers_config;
mod handlers_file;
mod handlers_job;
mod handlers_plugin;
mod handlers_runner;
mod handlers_update;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use ironbullet::config::{self, GuiConfig};
use ironbullet::pipeline::Pipeline;
use ironbullet::runner::{RunnerOrchestrator, HitResult};
use ironbullet::runner::job_manager::JobManager;
use ironbullet::sidecar::SidecarManager;
use ironbullet::plugin::manager::PluginManager;

/// IPC command from frontend
#[derive(Deserialize)]
pub struct IpcCmd {
    pub cmd: String,
    #[serde(default)]
    pub data: serde_json::Value,
}

/// IPC response to frontend
#[derive(Serialize)]
pub struct IpcResponse {
    pub cmd: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl IpcResponse {
    pub fn ok(cmd: &str, data: serde_json::Value) -> Self {
        Self {
            cmd: cmd.to_string(),
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(cmd: &str, error: String) -> Self {
        Self {
            cmd: cmd.to_string(),
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

/// Shared application state accessible from IPC handlers
pub struct AppState {
    pub config: GuiConfig,
    pub pipeline: Pipeline,
    /// Last file path the pipeline was saved to or loaded from.
    /// When set, block mutations will auto-save to this path.
    pub pipeline_path: Option<String>,
    pub sidecar: SidecarManager,
    pub runner: Option<Arc<RunnerOrchestrator>>,
    pub hits: Vec<HitResult>,
    pub job_manager: JobManager,
    pub plugin_manager: Arc<PluginManager>,
    /// Abort handle for the active browser-capture task (Inspector panel).
    pub browser_capture_abort: Option<tokio::task::AbortHandle>,
    /// Temp user-data-dir used by the current Chrome capture session.
    /// Cleaned up when the session ends or a new one starts, preventing
    /// Chrome from finding a locked profile from a zombie prior instance.
    pub browser_capture_profile: Option<std::path::PathBuf>,
    /// Abort handle for the local HTTP proxy capture server (Inspector panel).
    pub inspect_proxy_abort: Option<tokio::task::AbortHandle>,
    /// Port the proxy capture server is listening on.
    pub inspect_proxy_port: Option<u16>,
}

impl AppState {
    pub fn new() -> Self {
        let cfg = config::load_config();
        let mut pm = PluginManager::new();
        if !cfg.plugins_path.is_empty() {
            pm.scan_directory(&cfg.plugins_path);
        }
        let pipeline_path = if !cfg.last_config_path.is_empty() {
            Some(cfg.last_config_path.clone())
        } else {
            None
        };
        Self {
            config: cfg,
            pipeline: Pipeline::default(),
            pipeline_path,
            sidecar: SidecarManager::new(),
            runner: None,
            hits: Vec::new(),
            job_manager: JobManager::new(),
            plugin_manager: Arc::new(pm),
            browser_capture_abort: None,
            browser_capture_profile: None,
            inspect_proxy_abort: None,
            inspect_proxy_port: None,
        }
    }
}

/// Resolve a sidecar path: always prefer exe-relative over the config value.
///
/// The stored config path may be stale (from a previous install location).
/// We always check next to the current executable first so the app works
/// even if the user reinstalls to a different folder.
fn resolve_sidecar_path(configured: &str) -> String {
    // Canonical sidecar filename for the current OS (ignore what config says
    // the name is — it's always "reqflow-sidecar[.exe]").
    let sidecar_name = if cfg!(target_os = "windows") {
        "reqflow-sidecar.exe"
    } else {
        "reqflow-sidecar"
    };

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            // 1. Next to the exe — ALWAYS preferred (deployed layout)
            let next_to_exe = dir.join(sidecar_name);
            if next_to_exe.exists() {
                return next_to_exe.display().to_string();
            }

            // 2. exe_dir/../../sidecar/ — dev layout (target/release/ → project/sidecar/)
            let dev = dir.join("../../sidecar").join(sidecar_name);
            if dev.exists() {
                return dev.canonicalize().unwrap_or(dev).display().to_string();
            }
            // 3. exe_dir/../sidecar/ — alt dev layout
            let alt = dir.join("../sidecar").join(sidecar_name);
            if alt.exists() {
                return alt.canonicalize().unwrap_or(alt).display().to_string();
            }
        }
    }

    // 4. Absolute config path (user may have configured a custom path)
    let p = std::path::Path::new(configured);
    if p.is_absolute() && p.exists() {
        return configured.to_string();
    }
    // 5. CWD-relative fallbacks
    if let Ok(cwd) = std::env::current_dir() {
        let cwd_sidecar = cwd.join("sidecar").join(sidecar_name);
        if cwd_sidecar.exists() {
            return cwd_sidecar.display().to_string();
        }
        let cwd_direct = cwd.join(sidecar_name);
        if cwd_direct.exists() {
            return cwd_direct.display().to_string();
        }
    }
    configured.to_string()
}

/// Handle an IPC command and return a JSON response string
pub fn handle_ipc_cmd(
    cmd: &IpcCmd,
    state: &Arc<Mutex<AppState>>,
    eval_js: impl Fn(String) + Send + 'static,
) -> Option<String> {
    let state = state.clone();
    let cmd_name = cmd.cmd.clone();
    let data = cmd.data.clone();

    match cmd_name.as_str() {
        "get_config" => {
            handlers_config::get_config(state, eval_js);
            None
        }
        "save_config" => {
            handlers_config::save_config(state, data, eval_js);
            None
        }
        "get_pipeline" => {
            handlers_config::get_pipeline(state, eval_js);
            None
        }
        "update_pipeline" => {
            handlers_config::update_pipeline(state, data, eval_js);
            None
        }
        "save_pipeline" => {
            handlers_config::save_pipeline(state, data, eval_js);
            None
        }
        "load_pipeline" => {
            handlers_config::load_pipeline(state, data, eval_js);
            None
        }
        "get_recent_configs" => {
            handlers_config::get_recent_configs(state, eval_js);
            None
        }
        "setup_default_dirs" => {
            handlers_config::setup_default_dirs(eval_js);
            None
        }

        "add_block" => {
            handlers_block::add_block(state, data, eval_js);
            None
        }
        "remove_block" => {
            handlers_block::remove_block(state, data, eval_js);
            None
        }
        "move_block" => {
            handlers_block::move_block(state, data, eval_js);
            None
        }
        "add_block_nested" => {
            handlers_block::add_block_nested(state, data, eval_js);
            None
        }
        "move_block_to_nested" => {
            handlers_block::move_block_to_nested(state, data, eval_js);
            None
        }
        "update_block" => {
            handlers_block::update_block(state, data, eval_js);
            None
        }
        "remove_blocks" => {
            handlers_block::remove_blocks(state, data, eval_js);
            None
        }
        "paste_blocks" => {
            handlers_block::paste_blocks(state, data, eval_js);
            None
        }
        "toggle_blocks" => {
            handlers_block::toggle_blocks(state, data, eval_js);
            None
        }
        "move_blocks_to" => {
            handlers_block::move_blocks_to(state, data, eval_js);
            None
        }
        "group_blocks" => {
            handlers_block::group_blocks(state, data, eval_js);
            None
        }

        "debug_pipeline" => {
            handlers_runner::debug_pipeline(state, data, eval_js);
            None
        }
        "start_runner" => {
            handlers_runner::start_runner(state, data, eval_js);
            None
        }
        "pause_runner" => {
            handlers_runner::pause_runner(state);
            None
        }
        "resume_runner" => {
            handlers_runner::resume_runner(state);
            None
        }
        "stop_runner" => {
            handlers_runner::stop_runner(state);
            None
        }
        "get_runner_stats" => {
            handlers_runner::get_runner_stats(state, eval_js);
            None
        }
        "check_proxies" => {
            handlers_runner::check_proxies(state, eval_js);
            None
        }
        "probe_url" => {
            handlers_runner::probe_url(data, eval_js);
            None
        }
        "site_inspect" => {
            handlers_runner::site_inspect(state, data, eval_js);
            None
        }
        "inspect_browser_open" => {
            handlers_runner::inspect_browser_open(state, data, eval_js);
            None
        }
        "inspect_browser_close" => {
            handlers_runner::inspect_browser_close(state, eval_js);
            None
        }
        "inspect_proxy_start" => {
            handlers_runner::inspect_proxy_start(state, data, eval_js);
            None
        }
        "inspect_proxy_stop" => {
            handlers_runner::inspect_proxy_stop(state, eval_js);
            None
        }

        "create_job" => {
            handlers_job::create_job(state, data, eval_js);
            None
        }
        "remove_job" => {
            handlers_job::remove_job(state, data, eval_js);
            None
        }
        "list_jobs" => {
            handlers_job::list_jobs(state, eval_js);
            None
        }
        "start_job" => {
            handlers_job::start_job(state, data, eval_js);
            None
        }
        "pause_job" => {
            handlers_job::pause_job(state, data, eval_js);
            None
        }
        "resume_job" => {
            handlers_job::resume_job(state, data, eval_js);
            None
        }
        "stop_job" => {
            handlers_job::stop_job(state, data, eval_js);
            None
        }
        "get_job_stats" => {
            handlers_job::get_job_stats(state, data, eval_js);
            None
        }
        "update_job" => {
            handlers_job::update_job(state, data, eval_js);
            None
        }
        "get_job_hits" => {
            handlers_job::get_job_hits(state, data, eval_js);
            None
        }

        "generate_code" => {
            handlers_file::generate_code(state, data, eval_js);
            None
        }
        "save_code" => {
            handlers_file::save_code(data, eval_js);
            None
        }
        "import_config" => {
            handlers_file::import_config(state, data, eval_js);
            None
        }
        "list_collections" => {
            handlers_file::list_collections(state, eval_js);
            None
        }
        "list_configs" => {
            handlers_file::list_configs(data, eval_js);
            None
        }
        "browse_folder" => {
            handlers_file::browse_folder(data, eval_js);
            None
        }
        "browse_file" => {
            handlers_file::browse_file(data, eval_js);
            None
        }

        "get_plugin_blocks" => {
            handlers_plugin::get_plugin_blocks(state, eval_js);
            None
        }
        "import_plugin" => {
            handlers_plugin::import_plugin(state, eval_js);
            None
        }
        "reload_plugins" => {
            handlers_plugin::reload_plugins(state, eval_js);
            None
        }
        "compile_plugin" => {
            handlers_plugin::compile_plugin(data, eval_js);
            None
        }
        "save_plugin_files" => {
            handlers_plugin::save_plugin_files(data, eval_js);
            None
        }

        "check_for_updates" => {
            handlers_update::check_for_updates(state, eval_js);
            None
        }
        "download_update" => {
            handlers_update::download_update(state, data, eval_js);
            None
        }

        "open_url" => {
            let url = data.get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if !url.is_empty() {
                #[cfg(target_os = "windows")]
                let _ = std::process::Command::new("cmd")
                    .args(["/c", "start", "", &url])
                    .spawn();
                #[cfg(target_os = "macos")]
                let _ = std::process::Command::new("open").arg(&url).spawn();
                #[cfg(target_os = "linux")]
                let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
            }
            None
        }

        "check_chrome" => {
            let chrome = find_chrome_executable();
            let found = chrome.is_some();
            let path = chrome
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            let resp = IpcResponse::ok("check_chrome", serde_json::json!({ "found": found, "path": path }));
            Some(serde_json::to_string(&resp).unwrap_or_default())
        }

        _ => {
            let resp = IpcResponse::err(&cmd_name, format!("Unknown command: {}", cmd_name));
            Some(serde_json::to_string(&resp).unwrap_or_default())
        }
    }
}

/// Locate a usable Chrome / Chromium executable.
///
/// Checks fixed known install paths first (handles cases where the binary
/// is not on PATH — e.g. Windows user-level installs, Linux snap/flatpak),
/// then falls back to PATH-based search via `which`/`where`.
fn find_chrome_executable() -> Option<std::path::PathBuf> {
    // ── Windows: admin-level and user-level install paths ────────────────
    #[cfg(target_os = "windows")]
    {
        // Build the user-level LocalAppData path dynamically since it varies per user.
        let mut user_paths: Vec<String> = Vec::new();
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            user_paths.push(format!(r"{}\Google\Chrome\Application\chrome.exe", local));
            user_paths.push(format!(r"{}\Chromium\Application\chrome.exe", local));
        }
        let fixed: Vec<&str> = [
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files\Chromium\Application\chrome.exe",
            r"C:\Program Files (x86)\Chromium\Application\chrome.exe",
        ].iter().copied()
            .chain(user_paths.iter().map(|s| s.as_str()))
            .collect();
        for path in &fixed {
            let p = std::path::Path::new(path);
            if p.exists() { return Some(p.to_path_buf()); }
        }
        // PATH fallback on Windows
        for name in &["chrome.exe", "chromium.exe"] {
            if let Ok(out) = std::process::Command::new("where").arg(name).output() {
                if out.status.success() {
                    let s = String::from_utf8_lossy(&out.stdout);
                    if let Some(first) = s.lines().next() {
                        let p = std::path::PathBuf::from(first.trim());
                        if p.exists() { return Some(p); }
                    }
                }
            }
        }
    }

    // ── macOS: standard application bundle paths ──────────────────────────
    #[cfg(target_os = "macos")]
    {
        let fixed: &[&str] = &[
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
            "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
        ];
        for path in fixed {
            let p = std::path::Path::new(path);
            if p.exists() { return Some(p.to_path_buf()); }
        }
    }

    // ── Linux: fixed paths for snap, flatpak, and common package managers ──
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let fixed: &[&str] = &[
            // Standard package manager installs
            "/usr/bin/google-chrome",
            "/usr/bin/google-chrome-stable",
            "/usr/bin/chromium-browser",
            "/usr/bin/chromium",
            // Snap installs
            "/snap/bin/chromium",
            "/snap/bin/google-chrome",
            // Flatpak wrapper scripts (vary by distro — check common locations)
            "/var/lib/flatpak/exports/bin/com.google.Chrome",
            "/usr/local/bin/google-chrome",
            "/opt/google/chrome/chrome",
        ];
        for path in fixed {
            let p = std::path::Path::new(path);
            if p.exists() { return Some(p.to_path_buf()); }
        }
    }

    // ── All platforms: PATH fallback via which (Linux/macOS) ─────────────
    #[cfg(not(target_os = "windows"))]
    {
        let names: &[&str] = &[
            "google-chrome", "google-chrome-stable", "chromium-browser", "chromium", "chrome",
        ];
        for name in names {
            if let Ok(out) = std::process::Command::new("which").arg(name).output() {
                if out.status.success() {
                    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if !s.is_empty() { return Some(std::path::PathBuf::from(s)); }
                }
            }
        }
    }

    None
}
