use std::sync::Arc;
use tokio::sync::Mutex;

use ironbullet::pipeline::block::{Block, BlockSettings, BlockType, GroupSettings};

use super::block_tree::{
    add_block_to_nested, extract_block_recursive, find_block_mut,
    remove_block_recursive, set_block_disabled_recursive,
};
use super::{AppState, IpcResponse};

pub(super) fn add_block(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
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
}

pub(super) fn remove_block(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
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
}

pub(super) fn move_block(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
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
}

pub(super) fn add_block_nested(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
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
}

pub(super) fn move_block_to_nested(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
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
}

pub(super) fn update_block(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
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
}

pub(super) fn remove_blocks(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
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
}

pub(super) fn paste_blocks(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
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
}

pub(super) fn toggle_blocks(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
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
}

pub(super) fn move_blocks_to(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
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
            let to = data.get("to").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            if let Some(ids) = data.get("ids").and_then(|v| v.as_array()) {
                let id_set: std::collections::HashSet<String> = ids.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                let mut selected: Vec<Block> = Vec::new();
                let mut remaining: Vec<Block> = Vec::new();
                for block in s.pipeline.blocks.drain(..) {
                    if id_set.contains(&block.id.to_string()) {
                        selected.push(block);
                    } else {
                        remaining.push(block);
                    }
                }
                let insert_at = to.min(remaining.len());
                for (i, block) in selected.into_iter().enumerate() {
                    remaining.insert(insert_at + i, block);
                }
                s.pipeline.blocks = remaining;
            }
            let mut resp_data = serde_json::to_value(&s.pipeline).unwrap_or_default();
            if let (Some(tid), Some(obj)) = (data.get("_tab_id").cloned(), resp_data.as_object_mut()) {
                obj.insert("_tab_id".to_string(), tid);
            }
            let resp = IpcResponse::ok("pipeline_loaded", resp_data);
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
        });
    }
}

pub(super) fn group_blocks(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
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
                let id_set: std::collections::HashSet<String> = ids.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                // Find the position of the first selected block (insertion point)
                let insert_pos = s.pipeline.blocks.iter()
                    .position(|b| id_set.contains(&b.id.to_string()))
                    .unwrap_or(0);
                // Extract selected blocks in order, keep remaining
                let mut selected: Vec<Block> = Vec::new();
                let mut remaining: Vec<Block> = Vec::new();
                for block in s.pipeline.blocks.drain(..) {
                    if id_set.contains(&block.id.to_string()) {
                        selected.push(block);
                    } else {
                        remaining.push(block);
                    }
                }
                // Create a Group block containing the selected blocks
                let mut group = Block::new(BlockType::Group);
                group.label = format!("Group ({} blocks)", selected.len());
                group.settings = BlockSettings::Group(GroupSettings {
                    blocks: selected,
                    collapsed: false,
                });
                let adj_pos = insert_pos.min(remaining.len());
                remaining.insert(adj_pos, group);
                s.pipeline.blocks = remaining;
            }
            let mut resp_data = serde_json::to_value(&s.pipeline).unwrap_or_default();
            if let (Some(tid), Some(obj)) = (data.get("_tab_id").cloned(), resp_data.as_object_mut()) {
                obj.insert("_tab_id".to_string(), tid);
            }
            let resp = IpcResponse::ok("pipeline_loaded", resp_data);
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
        });
    }
}
