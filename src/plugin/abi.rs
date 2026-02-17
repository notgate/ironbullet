use std::ffi::c_char;

#[repr(C)]
pub struct PluginInfo {
    pub name: *const c_char,
    pub version: *const c_char,
    pub author: *const c_char,
    pub description: *const c_char,
    pub block_count: u32,
}

#[repr(C)]
pub struct BlockInfo {
    /// "PluginName.BlockName"
    pub block_type_name: *const c_char,
    pub label: *const c_char,
    pub category: *const c_char,
    pub color: *const c_char,
    pub icon: *const c_char,
    pub settings_schema_json: *const c_char,
    pub default_settings_json: *const c_char,
}

#[repr(C)]
pub struct ExecuteResult {
    pub success: bool,
    pub updated_variables_json: *const c_char,
    pub log_message: *const c_char,
    pub error_message: *const c_char,
}

// Required DLL exports:
// plugin_info() -> *const PluginInfo
// plugin_block_info(index: u32) -> *const BlockInfo
// plugin_execute(block_index: u32, settings_json: *const c_char, variables_json: *const c_char) -> *const ExecuteResult
// plugin_free_string(ptr: *const c_char)
