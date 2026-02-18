use std::collections::HashMap;
use std::ffi::{c_char, CStr, CString};

use rand::Rng;
use serde::Deserialize;

// ── ABI structs (must match ironbullet plugin ABI) ──

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

unsafe impl Sync for PluginInfo {}
unsafe impl Sync for BlockInfo {}

// ── Static strings ──

static PLUGIN_NAME: &[u8] = b"CfClearance\0";
static PLUGIN_VERSION: &[u8] = b"1.0.0\0";
static PLUGIN_AUTHOR: &[u8] = b"paul\0";
static PLUGIN_DESC: &[u8] = b"CF clearance cookie generator for non-managed sites\0";

static BLOCK_TYPE: &[u8] = b"CfClearance.Generate\0";
static BLOCK_LABEL: &[u8] = b"CF Clearance\0";
static BLOCK_CATEGORY: &[u8] = b"Bypass\0";
static BLOCK_COLOR: &[u8] = b"#e5c07b\0";
static BLOCK_ICON: &[u8] = b"shield\0";

static SETTINGS_SCHEMA: &[u8] = b"[\
{\"key\":\"domain\",\"label\":\"Domain\",\"type\":\"string\",\"default\":\"\",\"placeholder\":\"example.com\"},\
{\"key\":\"output_clearance\",\"label\":\"Clearance Var\",\"type\":\"string\",\"default\":\"CF_CLEARANCE\"},\
{\"key\":\"output_bm\",\"label\":\"BM Var\",\"type\":\"string\",\"default\":\"CF_BM\"},\
{\"key\":\"capture\",\"label\":\"Capture\",\"type\":\"bool\",\"default\":false}\
]\0";

static DEFAULT_SETTINGS: &[u8] = b"{\"domain\":\"\",\"output_clearance\":\"CF_CLEARANCE\",\"output_bm\":\"CF_BM\",\"capture\":false}\0";

// ── Plugin ABI exports ──

static PLUGIN_INFO: PluginInfo = PluginInfo {
    name: PLUGIN_NAME.as_ptr() as *const c_char,
    version: PLUGIN_VERSION.as_ptr() as *const c_char,
    author: PLUGIN_AUTHOR.as_ptr() as *const c_char,
    description: PLUGIN_DESC.as_ptr() as *const c_char,
    block_count: 1,
};

static BLOCK_INFO_0: BlockInfo = BlockInfo {
    block_type_name: BLOCK_TYPE.as_ptr() as *const c_char,
    label: BLOCK_LABEL.as_ptr() as *const c_char,
    category: BLOCK_CATEGORY.as_ptr() as *const c_char,
    color: BLOCK_COLOR.as_ptr() as *const c_char,
    icon: BLOCK_ICON.as_ptr() as *const c_char,
    settings_schema_json: SETTINGS_SCHEMA.as_ptr() as *const c_char,
    default_settings_json: DEFAULT_SETTINGS.as_ptr() as *const c_char,
};

#[no_mangle]
pub extern "C" fn plugin_info() -> *const PluginInfo {
    &PLUGIN_INFO
}

#[no_mangle]
pub extern "C" fn plugin_block_info(index: u32) -> *const BlockInfo {
    match index {
        0 => &BLOCK_INFO_0,
        _ => std::ptr::null(),
    }
}

#[no_mangle]
pub extern "C" fn plugin_free_string(ptr: *const c_char) {
    if !ptr.is_null() {
        unsafe { drop(CString::from_raw(ptr as *mut _)); }
    }
}

// ── Settings ──

#[derive(Deserialize)]
struct Settings {
    #[serde(default)]
    domain: String,
    #[serde(default = "def_clearance_var")]
    output_clearance: String,
    #[serde(default = "def_bm_var")]
    output_bm: String,
    #[serde(default)]
    capture: bool,
}

fn def_clearance_var() -> String { "CF_CLEARANCE".into() }
fn def_bm_var() -> String { "CF_BM".into() }

// ── Core ──

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

fn rng_str(len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len).map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char).collect()
}

fn gen_clearance() -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}-{}-1.0.1.1-{}-{}", rng_str(16), ts, rng_str(20), rng_str(80))
}

fn gen_bm() -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}-{}-1.0.1.1-{}", rng_str(20), ts, rng_str(40))
}

fn to_cstring(s: &str) -> *const c_char {
    CString::new(s).unwrap_or_default().into_raw()
}

fn empty_cstr() -> *const c_char {
    CString::new("").unwrap_or_default().into_raw()
}

#[no_mangle]
pub extern "C" fn plugin_execute(
    block_index: u32,
    settings_json: *const c_char,
    variables_json: *const c_char,
) -> *const ExecuteResult {
    if block_index != 0 {
        return Box::into_raw(Box::new(ExecuteResult {
            success: false,
            updated_variables_json: empty_cstr(),
            log_message: empty_cstr(),
            error_message: to_cstring("Unknown block index"),
        }));
    }

    let settings_str = unsafe {
        if settings_json.is_null() { "{}" } else { CStr::from_ptr(settings_json).to_str().unwrap_or("{}") }
    };
    let vars_str = unsafe {
        if variables_json.is_null() { "{}" } else { CStr::from_ptr(variables_json).to_str().unwrap_or("{}") }
    };

    let settings: Settings = serde_json::from_str(settings_str).unwrap_or(Settings {
        domain: String::new(),
        output_clearance: def_clearance_var(),
        output_bm: def_bm_var(),
        capture: false,
    });

    let mut vars: HashMap<String, String> = serde_json::from_str(vars_str).unwrap_or_default();

    let clearance = gen_clearance();
    let bm = gen_bm();

    // Interpolate domain from variables if needed
    let domain = if settings.domain.contains('<') && settings.domain.contains('>') {
        let key = settings.domain.trim_start_matches('<').trim_end_matches('>');
        vars.get(key).cloned().unwrap_or(settings.domain.clone())
    } else {
        settings.domain.clone()
    };

    // Build cookie string for the domain
    let cookie_str = format!("cf_clearance={}; __cf_bm={}", clearance, bm);

    vars.insert(settings.output_clearance.clone(), clearance.clone());
    vars.insert(settings.output_bm.clone(), bm.clone());
    if !domain.is_empty() {
        vars.insert("CF_COOKIE_STRING".into(), cookie_str.clone());
        vars.insert("CF_DOMAIN".into(), domain);
    }

    let vars_json = serde_json::to_string(&vars).unwrap_or_default();
    let log = format!("cf_clearance={} | __cf_bm={}", &clearance[..16], &bm[..16]);

    Box::into_raw(Box::new(ExecuteResult {
        success: true,
        updated_variables_json: to_cstring(&vars_json),
        log_message: to_cstring(&log),
        error_message: empty_cstr(),
    }))
}
