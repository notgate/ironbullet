use serde::{Deserialize, Serialize};

// ── Akamai V3 Sensor ──
// Algorithm credit: glizzykingdreko
// https://github.com/glizzykingdreko/akamai-v3-sensor-data-helper

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AkamaiV3SensorSettings {
    pub mode: AkamaiV3Mode,
    pub payload_var: String,
    pub file_hash: String,
    pub cookie_hash: String,
    pub output_var: String,
    pub capture: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AkamaiV3Mode {
    Encrypt,
    Decrypt,
    ExtractCookieHash,
}

impl Default for AkamaiV3SensorSettings {
    fn default() -> Self {
        Self {
            mode: AkamaiV3Mode::Encrypt,
            payload_var: "SENSOR_PAYLOAD".into(),
            file_hash: String::new(),
            cookie_hash: "8888888".into(),
            output_var: "AKAMAI_SENSOR".into(),
            capture: false,
        }
    }
}
