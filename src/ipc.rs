use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use reqflow::config::{self, GuiConfig};
use reqflow::export::format::RfxConfig;
use reqflow::export::rust_codegen;
use reqflow::pipeline::block::{Block, BlockSettings, BlockType};
use reqflow::pipeline::engine::ExecutionContext;
use reqflow::pipeline::Pipeline;
use reqflow::runner::{RunnerOrchestrator, HitResult};
use reqflow::runner::data_pool::DataPool;
use reqflow::runner::job::Job;
use reqflow::runner::job_manager::JobManager;
use reqflow::runner::proxy_pool::ProxyPool;
use reqflow::sidecar::SidecarManager;
use reqflow::plugin::manager::PluginManager;

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
        Self {
            config: cfg,
            pipeline: Pipeline::default(),
            sidecar: SidecarManager::new(),
            runner: None,
            hits: Vec::new(),
            job_manager: JobManager::new(),
            plugin_manager: Arc::new(pm),
        }
    }
}

/// Resolve a sidecar path: if relative, search multiple locations.
fn resolve_sidecar_path(configured: &str) -> String {
    let p = std::path::Path::new(configured);
    if p.is_absolute() && p.exists() {
        return configured.to_string();
    }
    let filename = std::path::Path::new(configured)
        .file_name()
        .unwrap_or(std::ffi::OsStr::new(configured));
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            // 1. Next to the exe (deployed layout)
            let full = dir.join(configured);
            if full.exists() {
                return full.display().to_string();
            }
            // 2. exe_dir/../../sidecar/<filename> (dev: target/release/ -> project root sidecar/)
            let dev = dir.join("../../sidecar").join(filename);
            if dev.exists() {
                return dev.canonicalize().unwrap_or(dev).display().to_string();
            }
            // 3. exe_dir/../sidecar/<filename> (alt dev layout)
            let alt = dir.join("../sidecar").join(filename);
            if alt.exists() {
                return alt.canonicalize().unwrap_or(alt).display().to_string();
            }
        }
    }
    // 4. CWD-relative fallbacks
    if let Ok(cwd) = std::env::current_dir() {
        let cwd_sidecar = cwd.join("sidecar").join(filename);
        if cwd_sidecar.exists() {
            return cwd_sidecar.display().to_string();
        }
        let cwd_direct = cwd.join(configured);
        if cwd_direct.exists() {
            return cwd_direct.display().to_string();
        }
    }
    configured.to_string()
}

/// Recursively find a mutable block by ID in a block tree (searches inside IfElse/Loop)
fn find_block_mut(blocks: &mut Vec<Block>, id: uuid::Uuid) -> Option<&mut Block> {
    for block in blocks.iter_mut() {
        if block.id == id {
            return Some(block);
        }
        match &mut block.settings {
            BlockSettings::IfElse(s) => {
                if let Some(b) = find_block_mut(&mut s.true_blocks, id) { return Some(b); }
                if let Some(b) = find_block_mut(&mut s.false_blocks, id) { return Some(b); }
            }
            BlockSettings::Loop(s) => {
                if let Some(b) = find_block_mut(&mut s.blocks, id) { return Some(b); }
            }
            BlockSettings::Group(s) => {
                if let Some(b) = find_block_mut(&mut s.blocks, id) { return Some(b); }
            }
            _ => {}
        }
    }
    None
}

/// Recursively remove a block by ID from a block tree
fn remove_block_recursive(blocks: &mut Vec<Block>, id: uuid::Uuid) -> bool {
    let len = blocks.len();
    blocks.retain(|b| b.id != id);
    if blocks.len() < len {
        return true;
    }
    for block in blocks.iter_mut() {
        match &mut block.settings {
            BlockSettings::IfElse(s) => {
                if remove_block_recursive(&mut s.true_blocks, id) { return true; }
                if remove_block_recursive(&mut s.false_blocks, id) { return true; }
            }
            BlockSettings::Loop(s) => {
                if remove_block_recursive(&mut s.blocks, id) { return true; }
            }
            BlockSettings::Group(s) => {
                if remove_block_recursive(&mut s.blocks, id) { return true; }
            }
            _ => {}
        }
    }
    false
}

/// Recursively set disabled on a block by ID
fn set_block_disabled_recursive(blocks: &mut Vec<Block>, id: uuid::Uuid, disabled: bool) -> bool {
    if let Some(block) = find_block_mut(blocks, id) {
        block.disabled = disabled;
        return true;
    }
    false
}

/// Extract a block by ID from anywhere in the tree, returning (extracted_block, success)
fn extract_block_recursive(blocks: &mut Vec<Block>, id: uuid::Uuid) -> Option<Block> {
    // Check top-level
    if let Some(pos) = blocks.iter().position(|b| b.id == id) {
        return Some(blocks.remove(pos));
    }
    // Check nested
    for block in blocks.iter_mut() {
        match &mut block.settings {
            BlockSettings::IfElse(s) => {
                if let Some(b) = extract_block_recursive(&mut s.true_blocks, id) { return Some(b); }
                if let Some(b) = extract_block_recursive(&mut s.false_blocks, id) { return Some(b); }
            }
            BlockSettings::Loop(s) => {
                if let Some(b) = extract_block_recursive(&mut s.blocks, id) { return Some(b); }
            }
            BlockSettings::Group(s) => {
                if let Some(b) = extract_block_recursive(&mut s.blocks, id) { return Some(b); }
            }
            _ => {}
        }
    }
    None
}

/// Add a block into a nested container (IfElse branch or Loop body)
fn add_block_to_nested(blocks: &mut Vec<Block>, parent_id: uuid::Uuid, branch: &str, new_block: Block, index: Option<usize>) -> bool {
    for block in blocks.iter_mut() {
        if block.id == parent_id {
            let target = match (&mut block.settings, branch) {
                (BlockSettings::IfElse(s), "true") => Some(&mut s.true_blocks),
                (BlockSettings::IfElse(s), "false") => Some(&mut s.false_blocks),
                (BlockSettings::Loop(s), "body") => Some(&mut s.blocks),
                (BlockSettings::Group(s), "body") => Some(&mut s.blocks),
                _ => None,
            };
            if let Some(target) = target {
                if let Some(idx) = index {
                    if idx <= target.len() {
                        target.insert(idx, new_block);
                    } else {
                        target.push(new_block);
                    }
                } else {
                    target.push(new_block);
                }
                return true;
            }
        }
        // Recurse into nested containers
        match &mut block.settings {
            BlockSettings::IfElse(s) => {
                if add_block_to_nested(&mut s.true_blocks, parent_id, branch, new_block.clone(), index) { return true; }
                if add_block_to_nested(&mut s.false_blocks, parent_id, branch, new_block.clone(), index) { return true; }
            }
            BlockSettings::Loop(s) => {
                if add_block_to_nested(&mut s.blocks, parent_id, branch, new_block.clone(), index) { return true; }
            }
            BlockSettings::Group(s) => {
                if add_block_to_nested(&mut s.blocks, parent_id, branch, new_block.clone(), index) { return true; }
            }
            _ => {}
        }
    }
    false
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
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                let state = state.clone();
                handle.spawn(async move {
                    let s = state.lock().await;
                    let resp = IpcResponse::ok("config_loaded", serde_json::to_value(&s.config).unwrap_or_default());
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                });
            }
            None
        }

        "save_config" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let mut s = state.lock().await;
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
            None
        }

        "get_pipeline" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let s = state.lock().await;
                    let resp = IpcResponse::ok("pipeline_loaded", serde_json::to_value(&s.pipeline).unwrap_or_default());
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                });
            }
            None
        }

        "update_pipeline" => {
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
            None
        }

        "add_block" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let mut s = state.lock().await;
                    if let Some(blk_val) = data.get("_blocks") {
                        if let Ok(blks) = serde_json::from_value::<Vec<Block>>(blk_val.clone()) { s.pipeline.blocks = blks; }
                    }
                    if let Some(sblk_val) = data.get("_startup_blocks") {
                        if let Ok(sblks) = serde_json::from_value::<Vec<Block>>(sblk_val.clone()) { s.pipeline.startup_blocks = sblks; }
                    }
                    if let Some(bt) = data.get("block_type").and_then(|v| serde_json::from_value::<BlockType>(v.clone()).ok()) {
                        let mut block = Block::new(bt);
                        if bt == BlockType::Plugin {
                            if let BlockSettings::Plugin(ref mut ps) = block.settings {
                                if let Some(pbt) = data.get("plugin_block_type").and_then(|v| v.as_str()) {
                                    ps.plugin_block_type = pbt.to_string();
                                }
                                if let Some(sj) = data.get("settings_json").and_then(|v| v.as_str()) {
                                    ps.settings_json = sj.to_string();
                                }
                            }
                            if let Some(lbl) = data.get("label").and_then(|v| v.as_str()) {
                                block.label = lbl.to_string();
                            }
                        }
                        let index = data.get("index").and_then(|v| v.as_u64()).map(|v| v as usize);
                        if let Some(idx) = index {
                            if idx <= s.pipeline.blocks.len() {
                                s.pipeline.blocks.insert(idx, block);
                            } else {
                                s.pipeline.blocks.push(block);
                            }
                        } else {
                            s.pipeline.blocks.push(block);
                        }
                    }
                    let mut resp_data = serde_json::to_value(&s.pipeline).unwrap_or_default();
                    if let (Some(tid), Some(obj)) = (data.get("_tab_id").cloned(), resp_data.as_object_mut()) {
                        obj.insert("_tab_id".to_string(), tid);
                    }
                    let resp = IpcResponse::ok("pipeline_loaded", resp_data);
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                });
            }
            None
        }

        "remove_block" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let mut s = state.lock().await;
                    if let Some(blk_val) = data.get("_blocks") {
                        if let Ok(blks) = serde_json::from_value::<Vec<Block>>(blk_val.clone()) { s.pipeline.blocks = blks; }
                    }
                    if let Some(sblk_val) = data.get("_startup_blocks") {
                        if let Ok(sblks) = serde_json::from_value::<Vec<Block>>(sblk_val.clone()) { s.pipeline.startup_blocks = sblks; }
                    }
                    if let Some(id) = data.get("block_id").and_then(|v| v.as_str()) {
                        if let Ok(uuid) = uuid::Uuid::parse_str(id) {
                            remove_block_recursive(&mut s.pipeline.blocks, uuid);
                        }
                    }
                    let mut resp_data = serde_json::to_value(&s.pipeline).unwrap_or_default();
                    if let (Some(tid), Some(obj)) = (data.get("_tab_id").cloned(), resp_data.as_object_mut()) {
                        obj.insert("_tab_id".to_string(), tid);
                    }
                    let resp = IpcResponse::ok("pipeline_loaded", resp_data);
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                });
            }
            None
        }

        "move_block" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let mut s = state.lock().await;
                    if let Some(blk_val) = data.get("_blocks") {
                        if let Ok(blks) = serde_json::from_value::<Vec<Block>>(blk_val.clone()) { s.pipeline.blocks = blks; }
                    }
                    if let Some(sblk_val) = data.get("_startup_blocks") {
                        if let Ok(sblks) = serde_json::from_value::<Vec<Block>>(sblk_val.clone()) { s.pipeline.startup_blocks = sblks; }
                    }
                    let from = data.get("from").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let to = data.get("to").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let len = s.pipeline.blocks.len();
                    if from < len && to <= len && from != to {
                        let block = s.pipeline.blocks.remove(from);
                        let to = to.min(s.pipeline.blocks.len());
                        s.pipeline.blocks.insert(to, block);
                    }
                    let mut resp_data = serde_json::to_value(&s.pipeline).unwrap_or_default();
                    if let (Some(tid), Some(obj)) = (data.get("_tab_id").cloned(), resp_data.as_object_mut()) {
                        obj.insert("_tab_id".to_string(), tid);
                    }
                    let resp = IpcResponse::ok("pipeline_loaded", resp_data);
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                });
            }
            None
        }

        "add_block_nested" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let mut s = state.lock().await;
                    if let Some(blk_val) = data.get("_blocks") {
                        if let Ok(blks) = serde_json::from_value::<Vec<Block>>(blk_val.clone()) { s.pipeline.blocks = blks; }
                    }
                    if let Some(sblk_val) = data.get("_startup_blocks") {
                        if let Ok(sblks) = serde_json::from_value::<Vec<Block>>(sblk_val.clone()) { s.pipeline.startup_blocks = sblks; }
                    }
                    let parent_id = data.get("parent_id").and_then(|v| v.as_str()).unwrap_or("");
                    let branch = data.get("branch").and_then(|v| v.as_str()).unwrap_or("true").to_string();
                    let index = data.get("index").and_then(|v| v.as_u64()).map(|v| v as usize);
                    if let Ok(parent_uuid) = uuid::Uuid::parse_str(parent_id) {
                        if let Some(bt) = data.get("block_type").and_then(|v| serde_json::from_value::<BlockType>(v.clone()).ok()) {
                            let block = Block::new(bt);
                            add_block_to_nested(&mut s.pipeline.blocks, parent_uuid, &branch, block, index);
                        }
                    }
                    let mut resp_data = serde_json::to_value(&s.pipeline).unwrap_or_default();
                    if let (Some(tid), Some(obj)) = (data.get("_tab_id").cloned(), resp_data.as_object_mut()) {
                        obj.insert("_tab_id".to_string(), tid);
                    }
                    let resp = IpcResponse::ok("pipeline_loaded", resp_data);
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                });
            }
            None
        }

        "move_block_to_nested" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let mut s = state.lock().await;
                    if let Some(blk_val) = data.get("_blocks") {
                        if let Ok(blks) = serde_json::from_value::<Vec<Block>>(blk_val.clone()) { s.pipeline.blocks = blks; }
                    }
                    if let Some(sblk_val) = data.get("_startup_blocks") {
                        if let Ok(sblks) = serde_json::from_value::<Vec<Block>>(sblk_val.clone()) { s.pipeline.startup_blocks = sblks; }
                    }
                    let block_id = data.get("block_id").and_then(|v| v.as_str()).unwrap_or("");
                    let parent_id = data.get("parent_id").and_then(|v| v.as_str()).unwrap_or("");
                    let branch = data.get("branch").and_then(|v| v.as_str()).unwrap_or("true").to_string();
                    let index = data.get("index").and_then(|v| v.as_u64()).map(|v| v as usize);
                    if let (Ok(block_uuid), Ok(parent_uuid)) = (uuid::Uuid::parse_str(block_id), uuid::Uuid::parse_str(parent_id)) {
                        // Extract the block from its current location
                        if let Some(block) = extract_block_recursive(&mut s.pipeline.blocks, block_uuid) {
                            // Insert into new location
                            if !add_block_to_nested(&mut s.pipeline.blocks, parent_uuid, &branch, block.clone(), index) {
                                // If parent not found, put back at top level
                                s.pipeline.blocks.push(block);
                            }
                        }
                    }
                    let mut resp_data = serde_json::to_value(&s.pipeline).unwrap_or_default();
                    if let (Some(tid), Some(obj)) = (data.get("_tab_id").cloned(), resp_data.as_object_mut()) {
                        obj.insert("_tab_id".to_string(), tid);
                    }
                    let resp = IpcResponse::ok("pipeline_loaded", resp_data);
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                });
            }
            None
        }

        "update_block" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                let tab_id = data.get("_tab_id").cloned();
                let blocks_sync = data.get("_blocks").cloned();
                let startup_blocks_sync = data.get("_startup_blocks").cloned();
                handle.spawn(async move {
                    let mut s = state.lock().await;
                    if let Some(blk_val) = blocks_sync {
                        if let Ok(blks) = serde_json::from_value::<Vec<Block>>(blk_val) { s.pipeline.blocks = blks; }
                    }
                    if let Some(sblk_val) = startup_blocks_sync {
                        if let Ok(sblks) = serde_json::from_value::<Vec<Block>>(sblk_val) { s.pipeline.startup_blocks = sblks; }
                    }
                    if let Ok(updated_block) = serde_json::from_value::<Block>(data) {
                        let id = updated_block.id;
                        if let Some(block) = find_block_mut(&mut s.pipeline.blocks, id) {
                            *block = updated_block;
                        }
                    }
                    let mut resp_data = serde_json::to_value(&s.pipeline).unwrap_or_default();
                    if let (Some(tid), Some(obj)) = (tab_id, resp_data.as_object_mut()) {
                        obj.insert("_tab_id".to_string(), tid);
                    }
                    let resp = IpcResponse::ok("pipeline_loaded", resp_data);
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                });
            }
            None
        }

        "remove_blocks" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let mut s = state.lock().await;
                    if let Some(blk_val) = data.get("_blocks") {
                        if let Ok(blks) = serde_json::from_value::<Vec<Block>>(blk_val.clone()) { s.pipeline.blocks = blks; }
                    }
                    if let Some(sblk_val) = data.get("_startup_blocks") {
                        if let Ok(sblks) = serde_json::from_value::<Vec<Block>>(sblk_val.clone()) { s.pipeline.startup_blocks = sblks; }
                    }
                    if let Some(ids) = data.get("ids").and_then(|v| v.as_array()) {
                        for id_val in ids {
                            if let Some(id) = id_val.as_str() {
                                if let Ok(uuid) = uuid::Uuid::parse_str(id) {
                                    remove_block_recursive(&mut s.pipeline.blocks, uuid);
                                }
                            }
                        }
                    }
                    let mut resp_data = serde_json::to_value(&s.pipeline).unwrap_or_default();
                    if let (Some(tid), Some(obj)) = (data.get("_tab_id").cloned(), resp_data.as_object_mut()) {
                        obj.insert("_tab_id".to_string(), tid);
                    }
                    let resp = IpcResponse::ok("pipeline_loaded", resp_data);
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                });
            }
            None
        }

        "paste_blocks" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let mut s = state.lock().await;
                    if let Some(blk_val) = data.get("_blocks") {
                        if let Ok(blks) = serde_json::from_value::<Vec<Block>>(blk_val.clone()) { s.pipeline.blocks = blks; }
                    }
                    if let Some(sblk_val) = data.get("_startup_blocks") {
                        if let Ok(sblks) = serde_json::from_value::<Vec<Block>>(sblk_val.clone()) { s.pipeline.startup_blocks = sblks; }
                    }
                    // Deserialize blocks to paste and assign new UUIDs
                    if let Some(blocks_val) = data.get("blocks") {
                        if let Ok(mut blocks) = serde_json::from_value::<Vec<Block>>(blocks_val.clone()) {
                            fn reassign_ids(blocks: &mut Vec<Block>) {
                                for b in blocks.iter_mut() {
                                    b.id = uuid::Uuid::new_v4();
                                    match &mut b.settings {
                                        BlockSettings::IfElse(s) => { reassign_ids(&mut s.true_blocks); reassign_ids(&mut s.false_blocks); }
                                        BlockSettings::Loop(s) => { reassign_ids(&mut s.blocks); }
                                        BlockSettings::Group(s) => { reassign_ids(&mut s.blocks); }
                                        _ => {}
                                    }
                                }
                            }
                            reassign_ids(&mut blocks);
                            let index = data.get("index").and_then(|v| v.as_u64()).map(|v| v as usize);
                            if let Some(idx) = index {
                                let pos = idx.min(s.pipeline.blocks.len());
                                for (i, block) in blocks.into_iter().enumerate() {
                                    s.pipeline.blocks.insert(pos + i, block);
                                }
                            } else {
                                s.pipeline.blocks.extend(blocks);
                            }
                        }
                    }
                    let mut resp_data = serde_json::to_value(&s.pipeline).unwrap_or_default();
                    if let (Some(tid), Some(obj)) = (data.get("_tab_id").cloned(), resp_data.as_object_mut()) {
                        obj.insert("_tab_id".to_string(), tid);
                    }
                    let resp = IpcResponse::ok("pipeline_loaded", resp_data);
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                });
            }
            None
        }

        "toggle_blocks" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let mut s = state.lock().await;
                    if let Some(blk_val) = data.get("_blocks") {
                        if let Ok(blks) = serde_json::from_value::<Vec<Block>>(blk_val.clone()) { s.pipeline.blocks = blks; }
                    }
                    if let Some(sblk_val) = data.get("_startup_blocks") {
                        if let Ok(sblks) = serde_json::from_value::<Vec<Block>>(sblk_val.clone()) { s.pipeline.startup_blocks = sblks; }
                    }
                    let disabled = data.get("disabled").and_then(|v| v.as_bool()).unwrap_or(false);
                    if let Some(ids) = data.get("ids").and_then(|v| v.as_array()) {
                        for id_val in ids {
                            if let Some(id) = id_val.as_str() {
                                if let Ok(uuid) = uuid::Uuid::parse_str(id) {
                                    set_block_disabled_recursive(&mut s.pipeline.blocks, uuid, disabled);
                                }
                            }
                        }
                    }
                    let mut resp_data = serde_json::to_value(&s.pipeline).unwrap_or_default();
                    if let (Some(tid), Some(obj)) = (data.get("_tab_id").cloned(), resp_data.as_object_mut()) {
                        obj.insert("_tab_id".to_string(), tid);
                    }
                    let resp = IpcResponse::ok("pipeline_loaded", resp_data);
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                });
            }
            None
        }

        "generate_code" => {
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
            None
        }

        "save_code" => {
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
            None
        }

        "get_recent_configs" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let s = state.lock().await;
                    let resp = IpcResponse::ok("recent_configs", serde_json::to_value(&s.config.recent_configs).unwrap_or_default());
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                });
            }
            None
        }

        "save_pipeline" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let path = data.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let force_dialog = data.get("force_dialog").and_then(|v| v.as_bool()).unwrap_or(false);
                    let save_path = if path.is_empty() || force_dialog {
                        rfd::FileDialog::new()
                            .set_title("Save Config")
                            .add_filter("reqflow config", &["rfx"])
                            .save_file()
                            .map(|p| p.display().to_string())
                    } else {
                        Some(path)
                    };
                    if let Some(save_path) = save_path {
                        let mut s = state.lock().await;
                        let config = RfxConfig::from_pipeline(&s.pipeline);
                        match config.save_to_file(&save_path) {
                            Ok(()) => {
                                // Track in recent configs
                                let pipeline_name = s.pipeline.name.clone();
                                s.config.recent_configs.retain(|r| r.path != save_path);
                                s.config.recent_configs.insert(0, config::RecentConfigEntry {
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
            None
        }

        "load_pipeline" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let load_path = data.get("path").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let pick_path = if load_path.is_some() {
                        load_path.map(|p| std::path::PathBuf::from(p))
                    } else {
                        rfd::FileDialog::new()
                            .set_title("Open Config")
                            .add_filter("reqflow config", &["rfx"])
                            .pick_file()
                    };
                    if let Some(path) = pick_path {
                        let path_str = path.display().to_string();
                        match RfxConfig::load_from_file(&path_str) {
                            Ok(config) => {
                                let mut s = state.lock().await;
                                s.pipeline = config.pipeline;
                                // Track in recent configs
                                let pipeline_name = s.pipeline.name.clone();
                                s.config.recent_configs.retain(|r| r.path != path_str);
                                s.config.recent_configs.insert(0, config::RecentConfigEntry {
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
            None
        }

        "get_runner_stats" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let s = state.lock().await;
                    let stats = if let Some(ref runner) = s.runner {
                        Some(runner.get_stats())
                    } else {
                        None
                    };
                    let resp = IpcResponse::ok("runner_stats", serde_json::to_value(&stats).unwrap_or_default());
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                });
            }
            None
        }

        "debug_pipeline" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let s = state.lock().await;
                    let blocks = s.pipeline.blocks.clone();
                    let data_settings = s.pipeline.data_settings.clone();
                    let pm = s.plugin_manager.clone();
                    drop(s); // Release lock before async execution

                    // Use native reqwest backend — no sidecar needed for debug
                    let native_tx = reqflow::sidecar::native::create_native_backend();

                    let mut ctx = ExecutionContext::new(uuid::Uuid::new_v4().to_string());
                    ctx.plugin_manager = Some(pm);

                    // Populate test data variables from frontend
                    if let Some(test_line) = data.get("test_data_line").and_then(|v| v.as_str()) {
                        if !test_line.is_empty() {
                            let parts: Vec<&str> = test_line.split(data_settings.separator).collect();
                            for (i, slice_name) in data_settings.slices.iter().enumerate() {
                                if let Some(part) = parts.get(i) {
                                    ctx.variables.set_input(slice_name, part.to_string());
                                }
                            }
                        }
                    }
                    if let Some(test_proxy) = data.get("test_proxy").and_then(|v| v.as_str()) {
                        if !test_proxy.is_empty() {
                            ctx.proxy = Some(test_proxy.to_string());
                        }
                    }

                    let exec_result = ctx.execute_blocks(&blocks, &native_tx).await;

                    // Send last result as debug_step (backward compat)
                    let result = ctx.block_results.last().cloned();
                    let resp = IpcResponse::ok("debug_step", serde_json::to_value(&result).unwrap_or_default());
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));

                    // Always send all block results for the response viewer
                    let resp = IpcResponse::ok("debug_results", serde_json::to_value(&ctx.block_results).unwrap_or_default());
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));

                    // Send network log for network viewer
                    let resp = IpcResponse::ok("network_log", serde_json::to_value(&ctx.network_log).unwrap_or_default());
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));

                    // If there was an error, also send it as a log message
                    if let Err(e) = exec_result {
                        let resp = IpcResponse::err("debug_step", format!("Pipeline error: {}", e));
                        eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                    }
                });
            }
            None
        }

        "start_runner" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                let inner_handle = handle.clone();
                handle.spawn(async move {
                    let threads = data.get("threads").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
                    let mut s = state.lock().await;

                    // Start sidecar if needed
                    let sidecar_path = resolve_sidecar_path(&s.config.sidecar_path);
                    if !s.sidecar.is_running() {
                        match s.sidecar.start(&sidecar_path).await {
                            Ok(_) => {}
                            Err(e) => {
                                let resp = IpcResponse::err("runner_error", format!("Failed to start sidecar: {}", e));
                                eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                                return;
                            }
                        }
                    }

                    // Get a fresh sidecar tx by restarting
                    s.sidecar.stop().await;
                    let sidecar_tx = match s.sidecar.start(&sidecar_path).await {
                        Ok(tx) => tx,
                        Err(e) => {
                            let resp = IpcResponse::err("runner_error", format!("Failed to start sidecar: {}", e));
                            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                            return;
                        }
                    };

                    // Load data from wordlist file
                    let wordlist_path = data.get("wordlist_path").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let data_pool = if !wordlist_path.is_empty() {
                        match DataPool::from_file(&wordlist_path, true) {
                            Ok(dp) => dp,
                            Err(e) => {
                                let resp = IpcResponse::err("runner_error", format!("Failed to load wordlist '{}': {}", wordlist_path, e));
                                eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                                return;
                            }
                        }
                    } else {
                        // Try inline data from the data field
                        let lines: Vec<String> = data.get("data_lines")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                            .unwrap_or_default();
                        DataPool::new(lines)
                    };

                    // Load proxies from file path if provided
                    let proxy_path = data.get("proxy_path").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let proxy_pool = if !proxy_path.is_empty() {
                        match ProxyPool::from_file(&proxy_path, s.pipeline.proxy_settings.ban_duration_secs as u64) {
                            Ok(pp) => pp,
                            Err(e) => {
                                let resp = IpcResponse::err("runner_error", format!("Failed to load proxies '{}': {}", proxy_path, e));
                                eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                                return;
                            }
                        }
                    } else {
                        ProxyPool::empty()
                    };

                    let (hits_tx, mut hits_rx) = tokio::sync::mpsc::channel::<HitResult>(1024);

                    let pipeline = s.pipeline.clone();
                    let pm = s.plugin_manager.clone();
                    let runner = Arc::new(RunnerOrchestrator::new(
                        pipeline,
                        data_pool,
                        proxy_pool,
                        sidecar_tx,
                        threads,
                        hits_tx,
                        Some(pm),
                    ));

                    s.runner = Some(runner.clone());
                    s.hits.clear();

                    // Drop the lock before spawning long-running tasks
                    drop(s);

                    // Wrap eval_js in Arc<Mutex> so both spawned tasks can use it
                    let eval_js = Arc::new(tokio::sync::Mutex::new(eval_js));

                    // Notify frontend that runner has started
                    {
                        let ejs = eval_js.lock().await;
                        let resp = IpcResponse::ok("runner_started", serde_json::json!(null));
                        ejs(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                    }

                    // Spawn hit collector — streams each hit to frontend
                    let state2 = state.clone();
                    let eval_js2 = eval_js.clone();
                    inner_handle.spawn(async move {
                        while let Some(hit) = hits_rx.recv().await {
                            let hit_data = serde_json::json!({
                                "data_line": hit.data_line,
                                "captures": hit.captures,
                                "proxy": hit.proxy,
                            });
                            let resp = IpcResponse::ok("runner_hit", hit_data);
                            let ejs = eval_js2.lock().await;
                            ejs(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                            drop(ejs);
                            let mut s = state2.lock().await;
                            s.hits.push(hit);
                        }
                    });

                    // Spawn runner
                    let eval_js3 = eval_js.clone();
                    inner_handle.spawn(async move {
                        runner.start().await;
                        // Runner finished — notify frontend
                        let ejs = eval_js3.lock().await;
                        let resp = IpcResponse::ok("runner_stopped", serde_json::json!(null));
                        ejs(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                    });
                });
            }
            None
        }

        "pause_runner" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let s = state.lock().await;
                    if let Some(ref runner) = s.runner {
                        runner.pause();
                    }
                });
            }
            None
        }

        "resume_runner" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let s = state.lock().await;
                    if let Some(ref runner) = s.runner {
                        runner.resume();
                    }
                });
            }
            None
        }

        "stop_runner" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let s = state.lock().await;
                    if let Some(ref runner) = s.runner {
                        runner.stop();
                    }
                });
            }
            None
        }

        "import_config" => {
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
                                match reqflow::import::import_config_bytes(&bytes) {
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
            None
        }

        "list_collections" => {
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
            None
        }

        "browse_folder" => {
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
            None
        }

        "browse_file" => {
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
            None
        }

        "check_proxies" => {
            let state = state.clone();
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let s = state.lock().await;
                    let mut all_sources = s.pipeline.proxy_settings.proxy_sources.clone();
                    // Also gather sources from active proxy group
                    if !s.pipeline.proxy_settings.active_group.is_empty() {
                        if let Some(group) = s.pipeline.proxy_settings.proxy_groups.iter()
                            .find(|g| g.name == s.pipeline.proxy_settings.active_group) {
                            all_sources = group.sources.clone();
                        }
                    }
                    drop(s);

                    // Load proxies from sources
                    let mut proxies: Vec<String> = Vec::new();
                    for src in &all_sources {
                        match src.source_type {
                            reqflow::pipeline::ProxySourceType::File => {
                                if let Ok(content) = std::fs::read_to_string(&src.value) {
                                    proxies.extend(content.lines().filter(|l| !l.trim().is_empty()).map(|l| l.trim().to_string()));
                                }
                            }
                            reqflow::pipeline::ProxySourceType::Inline => {
                                proxies.extend(src.value.lines().filter(|l| !l.trim().is_empty()).map(|l| l.trim().to_string()));
                            }
                            reqflow::pipeline::ProxySourceType::Url => {
                                if let Ok(resp) = reqwest::get(&src.value).await {
                                    if let Ok(text) = resp.text().await {
                                        proxies.extend(text.lines().filter(|l| !l.trim().is_empty()).map(|l| l.trim().to_string()));
                                    }
                                }
                            }
                        }
                    }

                    let total = proxies.len();
                    let mut alive = 0u32;
                    let mut dead = 0u32;

                    // Simple connectivity check: try to connect through each proxy
                    for proxy_str in &proxies {
                        let check_result = async {
                            let proxy = reqwest::Proxy::all(proxy_str).map_err(|e| e.to_string())?;
                            let client = reqwest::Client::builder()
                                .proxy(proxy)
                                .timeout(std::time::Duration::from_secs(8))
                                .build()
                                .map_err(|e| e.to_string())?;
                            client.get("https://httpbin.org/ip")
                                .send()
                                .await
                                .map_err(|e| e.to_string())?;
                            Ok::<_, String>(())
                        }.await;

                        match check_result {
                            Ok(_) => alive += 1,
                            Err(_) => dead += 1,
                        }
                    }

                    let resp = IpcResponse::ok("proxy_check_result", serde_json::json!({
                        "alive": alive,
                        "dead": dead,
                        "total": total,
                    }));
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                });
            }
            None
        }

        "create_job" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let mut s = state.lock().await;
                    let mut job = if let Ok(j) = serde_json::from_value::<Job>(data.clone()) {
                        j
                    } else {
                        Job::default()
                    };
                    // Override name if provided
                    if let Some(name) = data.get("name").and_then(|v| v.as_str()) {
                        job.name = name.to_string();
                    }
                    // Override config path if provided
                    if let Some(path) = data.get("config_path").and_then(|v| v.as_str()) {
                        job.config_path = Some(path.to_string());
                        if let Ok(rfx) = RfxConfig::load_from_file(path) {
                            job.pipeline = rfx.pipeline;
                        }
                    }
                    let id = s.job_manager.add_job(job);
                    let resp = IpcResponse::ok("job_created", serde_json::json!({ "id": id.to_string() }));
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                });
            }
            None
        }

        "remove_job" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let mut s = state.lock().await;
                    if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                        if let Ok(uuid) = uuid::Uuid::parse_str(id) {
                            s.job_manager.remove_job(uuid);
                        }
                    }
                    let jobs = s.job_manager.list_jobs();
                    let resp = IpcResponse::ok("jobs_list", serde_json::to_value(jobs).unwrap_or_default());
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                });
            }
            None
        }

        "list_jobs" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let s = state.lock().await;
                    let jobs = s.job_manager.list_jobs();
                    let resp = IpcResponse::ok("jobs_list", serde_json::to_value(jobs).unwrap_or_default());
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                });
            }
            None
        }

        "start_job" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                let inner_handle = handle.clone();
                handle.spawn(async move {
                    let mut s = state.lock().await;
                    let id_str = data.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    let uuid = match uuid::Uuid::parse_str(id_str) {
                        Ok(u) => u,
                        Err(_) => {
                            let resp = IpcResponse::err("job_stats_update", "Invalid job ID".into());
                            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                            return;
                        }
                    };

                    // Start sidecar if needed
                    let sidecar_path = resolve_sidecar_path(&s.config.sidecar_path);
                    if !s.sidecar.is_running() {
                        s.sidecar.stop().await;
                        match s.sidecar.start(&sidecar_path).await {
                            Ok(_) => {}
                            Err(e) => {
                                let resp = IpcResponse::err("job_stats_update", format!("Failed to start sidecar: {}", e));
                                eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                                return;
                            }
                        }
                    }
                    let sidecar_tx = match s.sidecar.start(&sidecar_path).await {
                        Ok(tx) => tx,
                        Err(e) => {
                            let resp = IpcResponse::err("job_stats_update", format!("Failed to start sidecar: {}", e));
                            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                            return;
                        }
                    };

                    let pm = s.plugin_manager.clone();
                    let result = s.job_manager.start_job(uuid, sidecar_tx, Some(pm));
                    drop(s);

                    if let Some((runner, mut hits_rx)) = result {
                        let job_id = uuid;
                        let state2 = state.clone();

                        // Spawn hit collector for this job
                        inner_handle.spawn(async move {
                            while let Some(hit) = hits_rx.recv().await {
                                let mut s = state2.lock().await;
                                s.job_manager.add_hit(job_id, hit);
                            }
                        });

                        // Use a channel to relay eval_js calls from multiple tasks
                        let (js_tx, mut js_rx) = tokio::sync::mpsc::channel::<String>(64);

                        // eval_js consumer — owns eval_js, calls it for each message
                        inner_handle.spawn(async move {
                            while let Some(js) = js_rx.recv().await {
                                eval_js(js);
                            }
                        });

                        // Spawn periodic stats push
                        let state3 = state.clone();
                        let js_tx2 = js_tx.clone();
                        let runner2 = runner.clone();
                        inner_handle.spawn(async move {
                            loop {
                                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                                let mut s = state3.lock().await;
                                s.job_manager.update_job_stats(job_id);
                                let jobs = s.job_manager.list_jobs();
                                let resp = IpcResponse::ok("jobs_list", serde_json::to_value(jobs).unwrap_or_default());
                                let _ = js_tx2.send(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default())).await;
                                if !runner2.is_running() { break; }
                            }
                        });

                        // Spawn runner
                        let state4 = state.clone();
                        inner_handle.spawn(async move {
                            runner.start().await;
                            // Mark completed
                            let mut s = state4.lock().await;
                            if let Some(job) = s.job_manager.get_job_mut(job_id) {
                                job.state = reqflow::runner::job::JobState::Completed;
                                job.completed = Some(chrono::Utc::now());
                            }
                            s.job_manager.update_job_stats(job_id);
                            let jobs = s.job_manager.list_jobs();
                            let resp = IpcResponse::ok("jobs_list", serde_json::to_value(jobs).unwrap_or_default());
                            let _ = js_tx.send(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default())).await;
                        });
                    }
                });
            }
            None
        }

        "pause_job" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let mut s = state.lock().await;
                    if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                        if let Ok(uuid) = uuid::Uuid::parse_str(id) {
                            s.job_manager.pause_job(uuid);
                        }
                    }
                    let jobs = s.job_manager.list_jobs();
                    let resp = IpcResponse::ok("jobs_list", serde_json::to_value(jobs).unwrap_or_default());
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                });
            }
            None
        }

        "resume_job" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let mut s = state.lock().await;
                    if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                        if let Ok(uuid) = uuid::Uuid::parse_str(id) {
                            s.job_manager.resume_job(uuid);
                        }
                    }
                    let jobs = s.job_manager.list_jobs();
                    let resp = IpcResponse::ok("jobs_list", serde_json::to_value(jobs).unwrap_or_default());
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                });
            }
            None
        }

        "stop_job" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let mut s = state.lock().await;
                    if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                        if let Ok(uuid) = uuid::Uuid::parse_str(id) {
                            s.job_manager.stop_job(uuid);
                        }
                    }
                    let jobs = s.job_manager.list_jobs();
                    let resp = IpcResponse::ok("jobs_list", serde_json::to_value(jobs).unwrap_or_default());
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                });
            }
            None
        }

        "get_job_stats" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let mut s = state.lock().await;
                    if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                        if let Ok(uuid) = uuid::Uuid::parse_str(id) {
                            s.job_manager.update_job_stats(uuid);
                            let stats = s.job_manager.get_job_stats(uuid);
                            let resp = IpcResponse::ok("job_stats_update", serde_json::json!({
                                "id": id,
                                "stats": serde_json::to_value(&stats).unwrap_or_default(),
                            }));
                            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                        }
                    }
                });
            }
            None
        }

        "get_job_hits" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let s = state.lock().await;
                    if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                        if let Ok(uuid) = uuid::Uuid::parse_str(id) {
                            let hits = s.job_manager.get_job_hits(uuid);
                            let resp = IpcResponse::ok("job_hits", serde_json::json!({
                                "id": id,
                                "hits": serde_json::to_value(&hits).unwrap_or_default(),
                            }));
                            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                        }
                    }
                });
            }
            None
        }

        "get_plugin_blocks" => {
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
            None
        }

        "import_plugin" => {
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
            None
        }

        "reload_plugins" => {
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
            None
        }

        "compile_plugin" => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                handle.spawn(async move {
                    let project_dir = data.get("project_dir").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let lib_rs = data.get("lib_rs").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let cargo_toml = data.get("cargo_toml").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let release = data.get("release").and_then(|v| v.as_bool()).unwrap_or(true);

                    // Determine project path (use temp dir if no project_dir specified)
                    let dir = if project_dir.is_empty() {
                        let tmp = std::env::temp_dir().join("reqflow-plugin-build");
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
                                "line": format!("error: Failed to run cargo: {} — is Rust installed?", e),
                                "done": true, "success": false,
                            }));
                            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                        }
                    }
                });
            }
            None
        }

        "save_plugin_files" => {
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
            None
        }

        _ => {
            let resp = IpcResponse::err(&cmd_name, format!("Unknown command: {}", cmd_name));
            Some(serde_json::to_string(&resp).unwrap_or_default())
        }
    }
}
