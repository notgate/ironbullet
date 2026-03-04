use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JwtAlgorithm {
    HS256,
    HS384,
    HS512,
}

impl Default for JwtAlgorithm {
    fn default() -> Self { Self::HS256 }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JwtAction {
    /// Sign a set of claims → produce a JWT token string
    Sign,
    /// Decode and verify a JWT token → extract claims into variables
    Decode,
}

impl Default for JwtAction {
    fn default() -> Self { Self::Sign }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtSettings {
    /// Sign or Decode
    #[serde(default)]
    pub action: JwtAction,
    /// HMAC secret (for HS256/384/512). Supports variable interpolation.
    #[serde(default)]
    pub secret: String,
    /// Algorithm
    #[serde(default)]
    pub algorithm: JwtAlgorithm,
    /// JSON claims payload for Sign (supports variable interpolation in values).
    /// e.g. `{"sub": "{USER}", "iat": "{TIMESTAMP}", "exp": "{EXP}"}`
    #[serde(default)]
    pub claims: String,
    /// Token input for Decode — the raw JWT string, usually a variable like `{TOKEN}`
    #[serde(default)]
    pub token_input: String,
    /// Variable name to store the resulting token (Sign) or decoded claims JSON (Decode)
    #[serde(default = "default_output_var")]
    pub output_var: String,
    /// For Sign: expiry in seconds from now added as `exp` claim.
    /// 0 = no exp claim added.
    #[serde(default)]
    pub expires_in_secs: u64,
    /// When decoding: fail the block if signature is invalid or token is expired
    #[serde(default = "default_true")]
    pub verify_on_decode: bool,
}

fn default_output_var() -> String { "JWT".to_string() }
fn default_true() -> bool { true }

impl Default for JwtSettings {
    fn default() -> Self {
        Self {
            action: JwtAction::Sign,
            secret: String::new(),
            algorithm: JwtAlgorithm::HS256,
            claims: r#"{"sub": "{USER}", "iat": 0}"#.to_string(),
            token_input: String::new(),
            output_var: default_output_var(),
            expires_in_secs: 0,
            verify_on_decode: true,
        }
    }
}
