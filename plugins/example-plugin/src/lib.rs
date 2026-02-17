use std::collections::HashMap;
use std::ffi::{c_char, CStr, CString};
use std::sync::OnceLock;

// ── ABI structs (must match reqflow's plugin::abi) ──

#[repr(C)]
pub struct PluginInfo {
    pub name: *const c_char,
    pub version: *const c_char,
    pub author: *const c_char,
    pub description: *const c_char,
    pub block_count: u32,
}

unsafe impl Send for PluginInfo {}
unsafe impl Sync for PluginInfo {}

#[repr(C)]
pub struct BlockInfo {
    pub block_type_name: *const c_char,
    pub label: *const c_char,
    pub category: *const c_char,
    pub color: *const c_char,
    pub icon: *const c_char,
    pub settings_schema_json: *const c_char,
    pub default_settings_json: *const c_char,
}

unsafe impl Send for BlockInfo {}
unsafe impl Sync for BlockInfo {}

#[repr(C)]
pub struct ExecuteResult {
    pub success: bool,
    pub updated_variables_json: *const c_char,
    pub log_message: *const c_char,
    pub error_message: *const c_char,
}

// ── Static strings (leaked CStrings for stable pointers) ──

fn leak_cstring(s: &str) -> *const c_char {
    CString::new(s).unwrap().into_raw() as *const c_char
}

// ── Plugin info ──

static PLUGIN_INFO: OnceLock<PluginInfo> = OnceLock::new();
static BLOCK_INFO: OnceLock<BlockInfo> = OnceLock::new();

fn get_plugin_info() -> &'static PluginInfo {
    PLUGIN_INFO.get_or_init(|| PluginInfo {
        name: leak_cstring("ExamplePlugin"),
        version: leak_cstring("0.1.0"),
        author: leak_cstring("reqflow"),
        description: leak_cstring("Example plugin that reverses strings"),
        block_count: 1,
    })
}

fn get_block_info() -> &'static BlockInfo {
    BLOCK_INFO.get_or_init(|| BlockInfo {
        block_type_name: leak_cstring("ExamplePlugin.ReverseString"),
        label: leak_cstring("Reverse String"),
        category: leak_cstring("Utilities"),
        color: leak_cstring("#9b59b6"),
        icon: leak_cstring("repeat"),
        settings_schema_json: leak_cstring(r#"{"type":"object","properties":{"input_var":{"type":"string","title":"Input Variable","default":"data.SOURCE"}}}"#),
        default_settings_json: leak_cstring(r#"{"input_var":"data.SOURCE"}"#),
    })
}

// ── Exports ──

#[no_mangle]
pub extern "C" fn plugin_info() -> *const PluginInfo {
    get_plugin_info() as *const PluginInfo
}

#[no_mangle]
pub extern "C" fn plugin_block_info(index: u32) -> *const BlockInfo {
    if index == 0 {
        get_block_info() as *const BlockInfo
    } else {
        std::ptr::null()
    }
}

#[no_mangle]
pub extern "C" fn plugin_execute(
    _block_index: u32,
    settings_json: *const c_char,
    variables_json: *const c_char,
) -> *const ExecuteResult {
    let settings_str = if settings_json.is_null() {
        "{}".to_string()
    } else {
        unsafe { CStr::from_ptr(settings_json).to_string_lossy().to_string() }
    };

    let vars_str = if variables_json.is_null() {
        "{}".to_string()
    } else {
        unsafe { CStr::from_ptr(variables_json).to_string_lossy().to_string() }
    };

    // Parse settings
    let settings: HashMap<String, String> =
        serde_json::from_str(&settings_str).unwrap_or_default();
    let input_var = settings
        .get("input_var")
        .cloned()
        .unwrap_or_else(|| "data.SOURCE".to_string());

    // Parse variables
    let vars: HashMap<String, String> = serde_json::from_str(&vars_str).unwrap_or_default();
    let input_value = vars.get(&input_var).cloned().unwrap_or_default();

    // Reverse the string
    let reversed: String = input_value.chars().rev().collect();

    // Build updated variables
    let mut updated = vars.clone();
    updated.insert("PLUGIN_RESULT".to_string(), reversed.clone());

    let updated_json = serde_json::to_string(&updated).unwrap_or_else(|_| "{}".to_string());
    let log_msg = format!("Reversed '{}' → '{}'", input_value, reversed);

    let result = Box::new(ExecuteResult {
        success: true,
        updated_variables_json: CString::new(updated_json).unwrap().into_raw(),
        log_message: CString::new(log_msg).unwrap().into_raw(),
        error_message: std::ptr::null(),
    });

    Box::into_raw(result) as *const ExecuteResult
}

#[no_mangle]
pub extern "C" fn plugin_free_string(ptr: *const c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(CString::from_raw(ptr as *mut c_char));
        }
    }
}
