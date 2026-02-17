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

fn def_width() -> f64 { 1100.0 }
fn def_height() -> f64 { 700.0 }
fn def_zoom() -> u32 { 100 }
fn def_font_size() -> u32 { 12 }
fn def_font_family() -> String { "Cascadia Code".into() }
fn def_max_threads() -> u32 { 100 }
fn def_sidecar_path() -> String { "reqflow-sidecar.exe".into() }
fn def_left_panel_width() -> u32 { 200 }
fn def_bottom_panel_height() -> u32 { 250 }
fn def_show_palette() -> bool { true }

impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            window_width: 1100.0,
            window_height: 700.0,
            window_x: None,
            window_y: None,
            zoom: 100,
            font_size: 12,
            font_family: "Cascadia Code".into(),
            last_config_path: String::new(),
            recent_configs: Vec::new(),
            default_threads: 100,
            sidecar_path: "reqflow-sidecar.exe".into(),
            left_panel_width: 200,
            bottom_panel_height: 250,
            show_block_palette: true,
            collections_path: String::new(),
            default_wordlist_path: String::new(),
            default_proxy_path: String::new(),
            plugins_path: String::new(),
        }
    }
}

pub fn config_dir() -> PathBuf {
    if let Some(appdata) = std::env::var_os("APPDATA") {
        let dir = PathBuf::from(appdata).join("reqflow");
        let _ = std::fs::create_dir_all(&dir);
        return dir;
    }
    PathBuf::from(".")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn load_config() -> GuiConfig {
    let p = config_path();
    if p.exists() {
        if let Ok(data) = std::fs::read_to_string(&p) {
            if let Ok(cfg) = serde_json::from_str::<GuiConfig>(&data) {
                return cfg;
            }
        }
    }
    GuiConfig::default()
}

pub fn save_config(cfg: &GuiConfig) {
    let p = config_path();
    if let Ok(json) = serde_json::to_string_pretty(cfg) {
        let _ = std::fs::write(&p, json);
    }
}
