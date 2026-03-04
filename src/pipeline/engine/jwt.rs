use super::*;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use hmac::{Hmac, Mac};
use sha2::{Sha256, Sha384, Sha512};

type HmacSha256 = Hmac<Sha256>;
type HmacSha384 = Hmac<Sha384>;
type HmacSha512 = Hmac<Sha512>;

impl ExecutionContext {
    pub(super) async fn execute_jwt_token(&mut self, _block: &Block, settings: &JwtSettings) -> crate::error::Result<()> {
        match settings.action {
            JwtAction::Sign => self.jwt_sign(settings),
            JwtAction::Decode => self.jwt_decode(settings),
        }
    }

    fn jwt_sign(&mut self, settings: &JwtSettings) -> crate::error::Result<()> {
        let secret = self.variables.interpolate(&settings.secret);
        if secret.is_empty() {
            self.log.push(LogEntry {
                timestamp_ms: elapsed_ms(),
                block_id: Uuid::nil(),
                block_label: "JWT".into(),
                message: "JWT Sign failed: secret is empty".into(),
            });
            return Ok(());
        }

        // Parse claims JSON with variable interpolation on each value
        let claims_raw = self.variables.interpolate(&settings.claims);
        let mut claims: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(&claims_raw)
                .unwrap_or_else(|_| serde_json::Map::new());

        // Add standard time claims
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Only set iat if not already present or set to 0
        if !claims.contains_key("iat") || claims.get("iat").and_then(|v| v.as_u64()) == Some(0) {
            claims.insert("iat".to_string(), serde_json::Value::Number(now.into()));
        }
        if settings.expires_in_secs > 0 && !claims.contains_key("exp") {
            claims.insert("exp".to_string(), serde_json::Value::Number((now + settings.expires_in_secs).into()));
        }

        let alg_str = match settings.algorithm {
            JwtAlgorithm::HS256 => "HS256",
            JwtAlgorithm::HS384 => "HS384",
            JwtAlgorithm::HS512 => "HS512",
        };

        // Build header
        let header = serde_json::json!({"alg": alg_str, "typ": "JWT"});
        let header_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header).unwrap_or_default());
        let payload_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&claims).unwrap_or_default());

        let signing_input = format!("{}.{}", header_b64, payload_b64);

        let signature = match settings.algorithm {
            JwtAlgorithm::HS256 => {
                let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
                    .map_err(|e| crate::error::AppError::Pipeline(format!("JWT HMAC key error: {}", e)))?;
                mac.update(signing_input.as_bytes());
                URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
            }
            JwtAlgorithm::HS384 => {
                let mut mac = HmacSha384::new_from_slice(secret.as_bytes())
                    .map_err(|e| crate::error::AppError::Pipeline(format!("JWT HMAC key error: {}", e)))?;
                mac.update(signing_input.as_bytes());
                URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
            }
            JwtAlgorithm::HS512 => {
                let mut mac = HmacSha512::new_from_slice(secret.as_bytes())
                    .map_err(|e| crate::error::AppError::Pipeline(format!("JWT HMAC key error: {}", e)))?;
                mac.update(signing_input.as_bytes());
                URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
            }
        };

        let token = format!("{}.{}", signing_input, signature);
        let out_var = if settings.output_var.is_empty() { "JWT" } else { &settings.output_var };
        self.variables.set_user(out_var, token.clone(), true);

        self.log.push(LogEntry {
            timestamp_ms: elapsed_ms(),
            block_id: Uuid::nil(),
            block_label: "JWT".into(),
            message: format!("JWT signed ({alg_str}) → {out_var}={}", &token[..token.len().min(32)]),
        });
        Ok(())
    }

    fn jwt_decode(&mut self, settings: &JwtSettings) -> crate::error::Result<()> {
        let token = self.variables.interpolate(&settings.token_input);
        let parts: Vec<&str> = token.splitn(3, '.').collect();
        if parts.len() != 3 {
            self.log.push(LogEntry {
                timestamp_ms: elapsed_ms(),
                block_id: Uuid::nil(),
                block_label: "JWT".into(),
                message: "JWT Decode failed: not a valid JWT (expected 3 parts)".into(),
            });
            return Ok(());
        }

        let payload_bytes = URL_SAFE_NO_PAD.decode(parts[1])
            .or_else(|_| base64::engine::general_purpose::STANDARD.decode(parts[1]))
            .unwrap_or_default();

        if settings.verify_on_decode && !settings.secret.is_empty() {
            let secret = self.variables.interpolate(&settings.secret);
            let signing_input = format!("{}.{}", parts[0], parts[1]);
            let sig_bytes = URL_SAFE_NO_PAD.decode(parts[2])
                .or_else(|_| base64::engine::general_purpose::STANDARD.decode(parts[2]))
                .unwrap_or_default();

            let valid = match settings.algorithm {
                JwtAlgorithm::HS256 => {
                    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).ok();
                    mac.as_mut().map(|m| {
                        m.update(signing_input.as_bytes());
                        m.clone().verify_slice(&sig_bytes).is_ok()
                    }).unwrap_or(false)
                }
                JwtAlgorithm::HS384 => {
                    let mut mac = HmacSha384::new_from_slice(secret.as_bytes()).ok();
                    mac.as_mut().map(|m| {
                        m.update(signing_input.as_bytes());
                        m.clone().verify_slice(&sig_bytes).is_ok()
                    }).unwrap_or(false)
                }
                JwtAlgorithm::HS512 => {
                    let mut mac = HmacSha512::new_from_slice(secret.as_bytes()).ok();
                    mac.as_mut().map(|m| {
                        m.update(signing_input.as_bytes());
                        m.clone().verify_slice(&sig_bytes).is_ok()
                    }).unwrap_or(false)
                }
            };

            if !valid {
                self.log.push(LogEntry {
                    timestamp_ms: elapsed_ms(),
                    block_id: Uuid::nil(),
                    block_label: "JWT".into(),
                    message: "JWT Decode: signature invalid".into(),
                });
                self.status = BotStatus::Fail;
                return Ok(());
            }

            // Check expiry
            if let Ok(claims) = serde_json::from_slice::<serde_json::Value>(&payload_bytes) {
                if let Some(exp) = claims.get("exp").and_then(|v| v.as_u64()) {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    if now > exp {
                        self.log.push(LogEntry {
                            timestamp_ms: elapsed_ms(),
                            block_id: Uuid::nil(),
                            block_label: "JWT".into(),
                            message: format!("JWT Decode: token expired (exp={})", exp),
                        });
                        self.status = BotStatus::Fail;
                        return Ok(());
                    }
                }
            }
        }

        let claims_str = String::from_utf8_lossy(&payload_bytes).to_string();
        let out_var = if settings.output_var.is_empty() { "JWT_CLAIMS" } else { &settings.output_var };
        self.variables.set_user(out_var, claims_str.clone(), true);

        // Also set individual claim variables as CLAIM_<key>
        if let Ok(claims_map) = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&claims_str) {
            for (k, v) in &claims_map {
                let val_str = match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                self.variables.set_user(&format!("CLAIM_{}", k.to_uppercase()), val_str, true);
            }
        }

        self.log.push(LogEntry {
            timestamp_ms: elapsed_ms(),
            block_id: Uuid::nil(),
            block_label: "JWT".into(),
            message: format!("JWT decoded → {out_var}"),
        });
        Ok(())
    }
}
