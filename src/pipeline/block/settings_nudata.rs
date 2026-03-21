use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuDataSensorSettings {
    #[serde(default)]
    pub site: String,
    #[serde(default)]
    pub init_url: String,
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default = "default_sdk_version")]
    pub sdk_version: String,
    #[serde(default)]
    pub href: String,
    #[serde(default)]
    pub proxy: String,
    #[serde(default = "default_output_var")]
    pub output_var: String,
    #[serde(default)]
    pub sid_var: String,
    #[serde(default = "default_true")]
    pub capture: bool,
    #[serde(default = "default_solver_url")]
    pub solver_url: String,
}

fn default_mode() -> String { "mobile".to_string() }
fn default_sdk_version() -> String { "2.7.5".to_string() }
fn default_output_var() -> String { "NDS_PMD".to_string() }
fn default_true() -> bool { true }
fn default_solver_url() -> String { "http://127.0.0.1:7450".to_string() }

impl Default for NuDataSensorSettings {
    fn default() -> Self {
        Self {
            site: String::new(),
            init_url: String::new(),
            mode: default_mode(),
            sdk_version: default_sdk_version(),
            href: String::new(),
            proxy: String::new(),
            output_var: default_output_var(),
            sid_var: String::new(),
            capture: true,
            solver_url: default_solver_url(),
        }
    }
}
