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

        _ => {
            let resp = IpcResponse::err(&cmd_name, format!("Unknown command: {}", cmd_name));
            Some(serde_json::to_string(&resp).unwrap_or_default())
        }
    }
}
