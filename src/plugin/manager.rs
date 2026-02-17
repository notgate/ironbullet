use std::collections::HashMap;
use std::ffi::{c_char, CStr, CString};
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::abi::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMeta {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub dll_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginBlockMeta {
    pub block_type_name: String,
    pub label: String,
    pub category: String,
    pub color: String,
    pub icon: String,
    pub settings_schema_json: String,
    pub default_settings_json: String,
    pub plugin_name: String,
    pub block_index: u32,
}

struct LoadedPlugin {
    meta: PluginMeta,
    blocks: Vec<PluginBlockMeta>,
    #[allow(dead_code)]
    lib: libloading::Library,
}

pub struct PluginManager {
    plugins: Vec<LoadedPlugin>,
}

impl std::fmt::Debug for PluginManager {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PluginManager({} plugins)", self.plugins.len())
    }
}

unsafe impl Send for PluginManager {}
unsafe impl Sync for PluginManager {}

impl PluginManager {
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    pub fn scan_directory(&mut self, path: &str) {
        self.plugins.clear();
        let dir = Path::new(path);
        if !dir.is_dir() {
            return;
        }
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().map(|e| e == "dll").unwrap_or(false) {
                    if let Err(e) = self.load_plugin(&p) {
                        eprintln!("Failed to load plugin {:?}: {}", p, e);
                    }
                }
            }
        }
    }

    fn load_plugin(&mut self, path: &Path) -> Result<(), String> {
        unsafe {
            let lib = libloading::Library::new(path)
                .map_err(|e| format!("Failed to load DLL: {}", e))?;

            // Load plugin_info
            let plugin_info_fn: libloading::Symbol<unsafe extern "C" fn() -> *const PluginInfo> =
                lib.get(b"plugin_info")
                    .map_err(|e| format!("Missing plugin_info export: {}", e))?;

            let info_ptr = plugin_info_fn();
            if info_ptr.is_null() {
                return Err("plugin_info returned null".into());
            }
            let info = &*info_ptr;
            let name = cstr_to_string(info.name);
            let version = cstr_to_string(info.version);
            let author = cstr_to_string(info.author);
            let description = cstr_to_string(info.description);
            let block_count = info.block_count;

            // Load block info
            let block_info_fn: libloading::Symbol<unsafe extern "C" fn(u32) -> *const BlockInfo> =
                lib.get(b"plugin_block_info")
                    .map_err(|e| format!("Missing plugin_block_info export: {}", e))?;

            let mut blocks = Vec::new();
            for i in 0..block_count {
                let bi_ptr = block_info_fn(i);
                if bi_ptr.is_null() {
                    continue;
                }
                let bi = &*bi_ptr;
                blocks.push(PluginBlockMeta {
                    block_type_name: cstr_to_string(bi.block_type_name),
                    label: cstr_to_string(bi.label),
                    category: cstr_to_string(bi.category),
                    color: cstr_to_string(bi.color),
                    icon: cstr_to_string(bi.icon),
                    settings_schema_json: cstr_to_string(bi.settings_schema_json),
                    default_settings_json: cstr_to_string(bi.default_settings_json),
                    plugin_name: name.clone(),
                    block_index: i,
                });
            }

            let meta = PluginMeta {
                name,
                version,
                author,
                description,
                dll_path: path.display().to_string(),
            };

            self.plugins.push(LoadedPlugin { meta, blocks, lib });
            Ok(())
        }
    }

    pub fn all_plugin_metas(&self) -> Vec<PluginMeta> {
        self.plugins.iter().map(|p| p.meta.clone()).collect()
    }

    pub fn all_block_infos(&self) -> Vec<PluginBlockMeta> {
        self.plugins.iter().flat_map(|p| p.blocks.clone()).collect()
    }

    pub fn execute_block(
        &self,
        type_name: &str,
        settings_json: &str,
        variables_json: &str,
    ) -> Result<(bool, HashMap<String, String>, String), String> {
        for plugin in &self.plugins {
            for block_meta in &plugin.blocks {
                if block_meta.block_type_name == type_name {
                    return self.call_execute(&plugin.lib, block_meta.block_index, settings_json, variables_json);
                }
            }
        }
        Err(format!("Plugin block '{}' not found", type_name))
    }

    fn call_execute(
        &self,
        lib: &libloading::Library,
        block_index: u32,
        settings_json: &str,
        variables_json: &str,
    ) -> Result<(bool, HashMap<String, String>, String), String> {
        unsafe {
            let execute_fn: libloading::Symbol<
                unsafe extern "C" fn(u32, *const c_char, *const c_char) -> *const ExecuteResult,
            > = lib
                .get(b"plugin_execute")
                .map_err(|e| format!("Missing plugin_execute export: {}", e))?;

            let free_fn: libloading::Symbol<unsafe extern "C" fn(*const c_char)> = lib
                .get(b"plugin_free_string")
                .map_err(|e| format!("Missing plugin_free_string export: {}", e))?;

            let settings_cstr = CString::new(settings_json).map_err(|e| e.to_string())?;
            let vars_cstr = CString::new(variables_json).map_err(|e| e.to_string())?;

            let result_ptr = execute_fn(block_index, settings_cstr.as_ptr(), vars_cstr.as_ptr());
            if result_ptr.is_null() {
                return Err("plugin_execute returned null".into());
            }
            let result = &*result_ptr;

            if !result.success {
                let err = cstr_to_string(result.error_message);
                // Free allocated strings
                free_fn(result.updated_variables_json);
                free_fn(result.log_message);
                free_fn(result.error_message);
                return Err(err);
            }

            let vars_json_str = cstr_to_string(result.updated_variables_json);
            let log_msg = cstr_to_string(result.log_message);

            // Free allocated strings
            free_fn(result.updated_variables_json);
            free_fn(result.log_message);
            free_fn(result.error_message);

            let updated_vars: HashMap<String, String> =
                serde_json::from_str(&vars_json_str).unwrap_or_default();

            Ok((true, updated_vars, log_msg))
        }
    }
}

unsafe fn cstr_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        CStr::from_ptr(ptr).to_string_lossy().to_string()
    }
}
