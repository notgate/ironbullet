use serde::{Deserialize, Serialize};

// ── ByteArray Function ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByteArraySettings {
    pub operation: ByteArrayOp,
    pub input_var: String,
    pub output_var: String,
    pub encoding: String,
    pub capture: bool,
}

impl Default for ByteArraySettings {
    fn default() -> Self {
        Self {
            operation: ByteArrayOp::ToHex,
            input_var: String::new(),
            output_var: "RESULT".into(),
            encoding: "hex".into(),
            capture: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ByteArrayOp {
    ToHex,
    FromHex,
    ToBase64,
    FromBase64,
    ToUtf8,
    FromUtf8,
}

// ── Constants Block ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantsSettings {
    pub constants: Vec<ConstantEntry>,
}

impl Default for ConstantsSettings {
    fn default() -> Self {
        Self {
            constants: vec![
                ConstantEntry { name: "API_KEY".into(), value: "your-key-here".into() },
                ConstantEntry { name: "BASE_URL".into(), value: "https://api.example.com".into() },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantEntry {
    pub name: String,
    pub value: String,
}

// ── Dictionary Function ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionarySettings {
    pub operation: DictOp,
    pub dict_var: String,
    pub key: String,
    pub value: String,
    pub output_var: String,
    pub capture: bool,
}

impl Default for DictionarySettings {
    fn default() -> Self {
        Self {
            operation: DictOp::Get,
            dict_var: "DICT".into(),
            key: String::new(),
            value: String::new(),
            output_var: "RESULT".into(),
            capture: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DictOp {
    Get,
    Set,
    Remove,
    Exists,
    Keys,
    Values,
}

// ── Float Function ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloatFunctionSettings {
    pub function_type: FloatFnType,
    pub input_var: String,
    pub param1: String,
    pub param2: String,
    pub output_var: String,
    pub capture: bool,
}

impl Default for FloatFunctionSettings {
    fn default() -> Self {
        Self {
            function_type: FloatFnType::Round,
            input_var: String::new(),
            param1: String::new(),
            param2: String::new(),
            output_var: "RESULT".into(),
            capture: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FloatFnType {
    Round,
    Ceil,
    Floor,
    Abs,
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Sqrt,
    Min,
    Max,
}

// ── Integer Function ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegerFunctionSettings {
    pub function_type: IntegerFnType,
    pub input_var: String,
    pub param1: String,
    pub param2: String,
    pub output_var: String,
    pub capture: bool,
}

impl Default for IntegerFunctionSettings {
    fn default() -> Self {
        Self {
            function_type: IntegerFnType::Add,
            input_var: String::new(),
            param1: String::new(),
            param2: String::new(),
            output_var: "RESULT".into(),
            capture: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegerFnType {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    Abs,
    Min,
    Max,
}

// ── Time Function ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeFunctionSettings {
    pub function_type: TimeFnType,
    pub input_var: String,
    pub timezone: String,
    pub target_timezone: String,
    pub format: String,
    pub output_var: String,
    pub capture: bool,
}

impl Default for TimeFunctionSettings {
    fn default() -> Self {
        Self {
            function_type: TimeFnType::ConvertTimezone,
            input_var: String::new(),
            timezone: "UTC".into(),
            target_timezone: "America/New_York".into(),
            format: "%Y-%m-%d %H:%M:%S".into(),
            output_var: "RESULT".into(),
            capture: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeFnType {
    ConvertTimezone,
    GetTimezone,
    IsDST,
    DurationBetween,
    AddDuration,
    SubtractDuration,
}

// ── Generate GUID ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateGUIDSettings {
    pub guid_version: GUIDVersion,
    pub namespace: String,
    pub name: String,
    pub output_var: String,
    pub capture: bool,
}

impl Default for GenerateGUIDSettings {
    fn default() -> Self {
        Self {
            guid_version: GUIDVersion::V4,
            namespace: String::new(),
            name: String::new(),
            output_var: "GUID".into(),
            capture: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GUIDVersion {
    V1,
    V4,
    V5,
}

// ── Phone Country ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneCountrySettings {
    pub input_var: String,
    pub output_var: String,
    pub output_format: PhoneOutputFormat,
    pub capture: bool,
}

impl Default for PhoneCountrySettings {
    fn default() -> Self {
        Self {
            input_var: String::new(),
            output_var: "COUNTRY".into(),
            output_format: PhoneOutputFormat::CountryCode,
            capture: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhoneOutputFormat {
    CountryCode,
    CountryName,
    ISO2,
    ISO3,
}

// ── Lambda Parser ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LambdaParserSettings {
    pub input_var: String,
    pub lambda_expression: String,
    pub output_var: String,
    pub capture: bool,
}

impl Default for LambdaParserSettings {
    fn default() -> Self {
        Self {
            input_var: "data.SOURCE".into(),
            lambda_expression: "x => x.split(',')[0]".into(),
            output_var: "RESULT".into(),
            capture: false,
        }
    }
}
