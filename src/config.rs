use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct GuiConfig {
    #[serde(default = "def_width")]
    pub window_width: f64,
    #[serde(default = "def_height")]
    pub window_height: f64,
    #[serde(default)]
    pub window_x: Option<i32>,
    #[serde(default)]
    pub window_y: Option<i32>,
    #[serde(default = "def_zoom")]
    pub zoom: u32,
    #[serde(default = "def_font_size")]
    pub font_size: u32,
    #[serde(default = "def_font_family")]
    pub font_family: String,
    #[serde(default = "def_font_weight")]
    pub font_weight: String,
    #[serde(default)]
    pub last_config_path: String,
    #[serde(default)]
    pub recent_configs: Vec<RecentConfigEntry>,
    #[serde(default = "def_max_threads")]
    pub default_threads: u32,
    #[serde(default = "def_sidecar_path")]
    pub sidecar_path: String,
    #[serde(default = "def_left_panel_width")]
    pub left_panel_width: u32,
    #[serde(default = "def_bottom_panel_height")]
    pub bottom_panel_height: u32,
    #[serde(default = "def_show_palette")]
    pub show_block_palette: bool,
    #[serde(default)]
    pub collections_path: String,
    #[serde(default)]
    pub default_wordlist_path: String,
    #[serde(default)]
    pub default_proxy_path: String,
    #[serde(default)]
    pub plugins_path: String,
    /// Global proxy groups (persist across config switches)
    #[serde(default)]
    pub proxy_groups: Vec<crate::pipeline::ProxyGroup>,
    /// User-specified Chrome/Chromium executable path for browser blocks.
    /// When empty, IronBullet uses chromiumoxide's built-in auto-discovery.
    #[serde(default)]
    pub chrome_executable_path: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RecentConfigEntry {
    pub path: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub last_opened: String,
}

fn def_width() -> f64 { 1318.0 }
fn def_height() -> f64 { 946.0 }
fn def_zoom() -> u32 { 100 }
fn def_font_size() -> u32 { 12 }
fn def_font_family() -> String { "Cascadia Code".into() }
fn def_font_weight() -> String { "400".into() }
fn def_max_threads() -> u32 { 100 }
fn def_sidecar_path() -> String {
    if cfg!(target_os = "windows") { "reqflow-sidecar.exe".into() } else { "reqflow-sidecar".into() }
}
fn def_left_panel_width() -> u32 { 200 }
fn def_bottom_panel_height() -> u32 { 250 }
fn def_show_palette() -> bool { true }

impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            window_width: 1318.0,
            window_height: 946.0,
            window_x: None,
            window_y: None,
            zoom: 100,
            font_size: 12,
            font_family: "Cascadia Code".into(),
            font_weight: "400".into(),
            last_config_path: String::new(),
            recent_configs: Vec::new(),
            default_threads: 100,
            sidecar_path: def_sidecar_path(),
            left_panel_width: 200,
            bottom_panel_height: 250,
            show_block_palette: true,
            collections_path: String::new(),
            default_wordlist_path: String::new(),
            default_proxy_path: String::new(),
            plugins_path: String::new(),
            proxy_groups: Vec::new(),
            chrome_executable_path: String::new(),
        }
    }
}

pub fn config_dir() -> PathBuf {
    // Windows: %APPDATA%\ironbullet
    // Linux:   ~/.config/ironbullet
    // macOS:   ~/Library/Application Support/ironbullet
    let dir = if let Some(d) = dirs::config_dir() {
        d.join("ironbullet")
    } else if let Some(appdata) = std::env::var_os("APPDATA") {
        PathBuf::from(appdata).join("ironbullet")
    } else {
        // Last resort: next to the exe
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("config")))
            .unwrap_or_else(|| PathBuf::from("config"))
    };
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Default data directory for wordlists, proxies, etc.
/// Placed alongside config dir so it survives exe replacement.
pub fn data_dir() -> PathBuf {
    let dir = config_dir().join("data");
    let _ = std::fs::create_dir_all(dir.join("wordlists"));
    let _ = std::fs::create_dir_all(dir.join("proxies"));
    dir
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn autosave_path() -> PathBuf {
    config_dir().join("autosave.rfx")
}

pub fn load_config() -> GuiConfig {
    let p = config_path();

    // Auto-backup before loading — protects against mid-write corruption
    if p.exists() {
        let bak = config_dir().join("config.json.bak");
        let _ = std::fs::copy(&p, &bak);
    }

    if p.exists() {
        if let Ok(data) = std::fs::read_to_string(&p) {
            if let Ok(cfg) = serde_json::from_str::<GuiConfig>(&data) {
                return cfg;
            }
        }
    }

    // First launch — set default data paths and persist them
    let mut cfg = GuiConfig::default();
    let dd = data_dir();
    cfg.default_wordlist_path = dd.join("wordlists").to_string_lossy().to_string();
    cfg.default_proxy_path    = dd.join("proxies").to_string_lossy().to_string();
    save_config(&cfg);
    cfg
}

pub fn save_config(cfg: &GuiConfig) {
    let p = config_path();
    if let Ok(json) = serde_json::to_string_pretty(cfg) {
        let _ = std::fs::write(&p, json);
    }
}
