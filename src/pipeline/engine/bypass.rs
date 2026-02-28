use uuid::Uuid;

use super::*;
use helpers::{generate_xacf_sensor_data, BUILTIN_USER_AGENTS};

impl ExecutionContext {
    // ── Webhook (via sidecar HTTP) ──

    pub(super) async fn execute_webhook(
        &mut self,
        settings: &WebhookSettings,
        sidecar_tx: &tokio::sync::mpsc::Sender<(SidecarRequest, tokio::sync::oneshot::Sender<SidecarResponse>)>,
    ) -> crate::error::Result<()> {
        let url = self.variables.interpolate(&settings.url);
        let body = self.variables.interpolate(&settings.body_template);
        let mut headers: Vec<Vec<String>> = settings.headers.iter()
            .map(|(k, v)| vec![self.variables.interpolate(k), self.variables.interpolate(v)])
            .collect();

        // Inject custom cookies as Cookie header
        if !settings.custom_cookies.is_empty() {
            let cookie_str = self.variables.interpolate(&settings.custom_cookies);
            let cookies: Vec<String> = cookie_str.lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .map(|l| l.to_string())
                .collect();
            if !cookies.is_empty() {
                headers.push(vec!["Cookie".into(), cookies.join("; ")]);
            }
        }

        let req = SidecarRequest {
            id: Uuid::new_v4().to_string(),
            action: "request".into(),
            session: self.session_id.clone(),
            method: Some(settings.method.clone()),
            url: Some(url),
            headers: Some(headers),
            body: Some(body),
            timeout: Some(10000),
            proxy: self.proxy.clone(),
            browser: None,
            ja3: self.override_ja3.clone(),
            http2fp: self.override_http2fp.clone(),
            follow_redirects: Some(true),
            max_redirects: Some(5),
            ssl_verify: None,
                    custom_ciphers: None,

            ..Default::default()
        };

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        sidecar_tx.send((req, resp_tx)).await
            .map_err(|_| crate::error::AppError::Sidecar("Failed to send webhook".into()))?;
        let _resp = resp_rx.await
            .map_err(|_| crate::error::AppError::Sidecar("Webhook response channel closed".into()))?;
        Ok(())
    }

    // ── Random User Agent ──

    pub(super) fn execute_random_user_agent(&mut self, settings: &RandomUserAgentSettings) -> crate::error::Result<()> {
        use rand::seq::SliceRandom;

        if settings.match_tls {
            // Use TLS profiles for matched UA + JA3 + HTTP/2 fingerprint
            let filtered: Vec<_> = TLS_PROFILES.iter()
                .filter(|p| {
                    let browser_ok = settings.browser_filter.is_empty()
                        || settings.browser_filter.iter().any(|f| f.eq_ignore_ascii_case(p.browser));
                    let platform_ok = settings.platform_filter.is_empty()
                        || settings.platform_filter.iter().any(|f| f.eq_ignore_ascii_case(p.platform));
                    browser_ok && platform_ok
                })
                .collect();
            if let Some(profile) = filtered.choose(&mut rand::thread_rng()) {
                self.variables.set_user(&settings.output_var, profile.user_agent.to_string(), settings.capture);
                self.override_ja3 = Some(profile.ja3_hash.to_string());
                self.override_http2fp = Some(profile.http2_fingerprint.to_string());
            } else if let Some(profile) = TLS_PROFILES.first() {
                self.variables.set_user(&settings.output_var, profile.user_agent.to_string(), settings.capture);
                self.override_ja3 = Some(profile.ja3_hash.to_string());
                self.override_http2fp = Some(profile.http2_fingerprint.to_string());
            }
            return Ok(());
        }

        let ua = match settings.mode {
            UserAgentMode::CustomList => {
                let list = self.variables.interpolate(&settings.custom_list);
                let items: Vec<&str> = list.lines().filter(|l| !l.trim().is_empty()).collect();
                items.choose(&mut rand::thread_rng())
                    .map(|s| s.to_string())
                    .unwrap_or_default()
            }
            UserAgentMode::Random => {
                let filtered: Vec<&(&str, &str, &str)> = BUILTIN_USER_AGENTS.iter()
                    .filter(|(_, browser, platform)| {
                        let browser_ok = settings.browser_filter.is_empty()
                            || settings.browser_filter.iter().any(|f| f.eq_ignore_ascii_case(browser));
                        let platform_ok = settings.platform_filter.is_empty()
                            || settings.platform_filter.iter().any(|f| f.eq_ignore_ascii_case(platform));
                        browser_ok && platform_ok
                    })
                    .collect();
                filtered.choose(&mut rand::thread_rng())
                    .map(|(ua, _, _)| ua.to_string())
                    .unwrap_or_else(|| BUILTIN_USER_AGENTS[0].0.to_string())
            }
        };

        self.variables.set_user(&settings.output_var, ua, settings.capture);
        Ok(())
    }

    // ── OCR Captcha ──

    pub(super) async fn execute_ocr_captcha(&mut self, settings: &OcrCaptchaSettings) -> crate::error::Result<()> {
        let input_b64 = self.variables.resolve_input(&settings.input_var);

        use base64::Engine;
        let image_bytes = base64::engine::general_purpose::STANDARD.decode(input_b64.as_bytes())
            .map_err(|e| crate::error::AppError::Pipeline(format!("OCR base64 decode error: {}", e)))?;

        // Write to temp file
        let temp_path = std::env::temp_dir().join(format!("ironbullet_ocr_{}.png", uuid::Uuid::new_v4()));
        std::fs::write(&temp_path, &image_bytes)
            .map_err(|e| crate::error::AppError::Pipeline(format!("OCR temp file write error: {}", e)))?;

        // Build tesseract args
        let mut args = rusty_tesseract::Args::default();
        args.lang = settings.language.clone();
        args.psm = Some(settings.psm as i32);
        if !settings.whitelist.is_empty() {
            args.config_variables.insert(
                "tessedit_char_whitelist".to_string(),
                settings.whitelist.clone(),
            );
        }

        let image = rusty_tesseract::Image::from_path(&temp_path)
            .map_err(|e| crate::error::AppError::Pipeline(format!("OCR image load error: {}", e)))?;

        let result = rusty_tesseract::image_to_string(&image, &args)
            .map_err(|e| crate::error::AppError::Pipeline(format!("OCR recognition error: {}", e)))?;

        // Cleanup temp file
        let _ = std::fs::remove_file(&temp_path);

        self.variables.set_user(&settings.output_var, result.trim().to_string(), settings.capture);
        Ok(())
    }

    // ── reCAPTCHA Invisible ──

    pub(super) async fn execute_recaptcha_invisible(
        &mut self,
        settings: &RecaptchaInvisibleSettings,
        sidecar_tx: &tokio::sync::mpsc::Sender<(SidecarRequest, tokio::sync::oneshot::Sender<SidecarResponse>)>,
    ) -> crate::error::Result<()> {
        let anchor_url = self.variables.interpolate(&settings.anchor_url);
        let reload_url = self.variables.interpolate(&settings.reload_url);
        let user_agent = self.variables.interpolate(&settings.user_agent);

        // Step 1: GET anchor URL to get initial recaptcha-token
        let anchor_req = SidecarRequest {
            id: uuid::Uuid::new_v4().to_string(),
            action: "request".into(),
            session: self.session_id.clone(),
            method: Some("GET".into()),
            url: Some(anchor_url.clone()),
            headers: Some(vec![
                vec!["User-Agent".into(), user_agent.clone()],
            ]),
            body: Some(String::new()),
            timeout: Some(15000),
            proxy: self.proxy.clone(),
            browser: None, ja3: self.override_ja3.clone(), http2fp: self.override_http2fp.clone(),
            follow_redirects: Some(true),
            max_redirects: Some(5),
            ssl_verify: None,
                    custom_ciphers: None,

            ..Default::default()
        };

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        sidecar_tx.send((anchor_req, resp_tx)).await
            .map_err(|_| crate::error::AppError::Sidecar("Failed to send anchor request".into()))?;
        let anchor_resp = resp_rx.await
            .map_err(|_| crate::error::AppError::Sidecar("Anchor response channel closed".into()))?;

        // Parse recaptcha-token from HTML
        let token = {
            let body = &anchor_resp.body;
            let marker = "recaptcha-token\" value=\"";
            if let Some(start) = body.find(marker) {
                let after = start + marker.len();
                if let Some(end) = body[after..].find('"') {
                    body[after..after + end].to_string()
                } else {
                    return Err(crate::error::AppError::Pipeline("Failed to parse recaptcha-token end".into()));
                }
            } else {
                return Err(crate::error::AppError::Pipeline("recaptcha-token not found in anchor response".into()));
            }
        };

        // Step 2: POST reload URL with token
        let post_body = format!(
            "v={}&reason=q&c={}&k={}&co={}&hl=en&size={}&chr=%5B89%2C64%2C27%5D&vh=13599012192&bg=!q62grYxHRvVxjUIjSFNd0mlr",
            self.variables.interpolate(&settings.v),
            token,
            self.variables.interpolate(&settings.sitekey),
            self.variables.interpolate(&settings.co),
            self.variables.interpolate(&settings.size),
        );

        let reload_req = SidecarRequest {
            id: uuid::Uuid::new_v4().to_string(),
            action: "request".into(),
            session: self.session_id.clone(),
            method: Some("POST".into()),
            url: Some(reload_url),
            headers: Some(vec![
                vec!["User-Agent".into(), user_agent],
                vec!["Content-Type".into(), "application/x-www-form-urlencoded".into()],
            ]),
            body: Some(post_body),
            timeout: Some(15000),
            proxy: self.proxy.clone(),
            browser: None, ja3: self.override_ja3.clone(), http2fp: self.override_http2fp.clone(),
            follow_redirects: Some(true),
            max_redirects: Some(5),
            ssl_verify: None,
                    custom_ciphers: None,

            ..Default::default()
        };

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        sidecar_tx.send((reload_req, resp_tx)).await
            .map_err(|_| crate::error::AppError::Sidecar("Failed to send reload request".into()))?;
        let reload_resp = resp_rx.await
            .map_err(|_| crate::error::AppError::Sidecar("Reload response channel closed".into()))?;

        // Extract rresp token
        let rresp = {
            let body = &reload_resp.body;
            let marker = "\"rresp\",\"";
            if let Some(start) = body.find(marker) {
                let after = start + marker.len();
                if let Some(end) = body[after..].find('"') {
                    body[after..after + end].to_string()
                } else {
                    return Err(crate::error::AppError::Pipeline("Failed to parse rresp end".into()));
                }
            } else {
                return Err(crate::error::AppError::Pipeline("rresp not found in reload response".into()));
            }
        };

        self.variables.set_user(&settings.output_var, rresp, settings.capture);
        Ok(())
    }

    // ── XACF Sensor ──

    pub(super) fn execute_xacf_sensor(&mut self, settings: &XacfSensorSettings) -> crate::error::Result<()> {
        let bundle_id = self.variables.interpolate(&settings.bundle_id);
        let version = self.variables.interpolate(&settings.version);
        let sensor = generate_xacf_sensor_data(&bundle_id, &version);
        self.variables.set_user(&settings.output_var, sensor, settings.capture);
        Ok(())
    }

    // ── Random Data ──

    pub(super) fn execute_random_data(&mut self, settings: &RandomDataSettings) -> crate::error::Result<()> {
        let value = match settings.data_type {
            RandomDataType::String => random_data::random_string(settings.string_length as usize, &settings.string_charset, &settings.custom_chars),
            RandomDataType::Uuid => random_data::random_uuid(),
            RandomDataType::Number => random_data::random_number(settings.number_min, settings.number_max, settings.number_decimal),
            RandomDataType::Email => random_data::random_email(),
            RandomDataType::FirstName => random_data::random_first_name(),
            RandomDataType::LastName => random_data::random_last_name(),
            RandomDataType::FullName => random_data::random_full_name(),
            RandomDataType::StreetAddress => random_data::random_street_address(),
            RandomDataType::City => random_data::random_city(),
            RandomDataType::State => random_data::random_state(),
            RandomDataType::ZipCode => random_data::random_zip(),
            RandomDataType::PhoneNumber => random_data::random_phone(),
            RandomDataType::Date => random_data::random_date(&settings.date_format, &settings.date_min, &settings.date_max),
        };
        self.variables.set_user(&settings.output_var, value, settings.capture);
        Ok(())
    }

    // ── DataDome Sensor ──

    pub(super) fn execute_datadome_sensor(&mut self, settings: &DataDomeSensorSettings) -> crate::error::Result<()> {
        let site_url = self.variables.interpolate(&settings.site_url);
        let cookie = self.variables.interpolate(&settings.cookie_datadome);
        let ua = self.variables.interpolate(&settings.user_agent);
        let custom_wasm = if settings.custom_wasm_b64.is_empty() {
            None
        } else {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD.decode(settings.custom_wasm_b64.as_bytes()).ok()
        };
        let sensor = datadome::generate_datadome_sensor(
            &site_url,
            &cookie,
            &ua,
            custom_wasm.as_deref(),
        )?;
        self.variables.set_user(&settings.output_var, sensor, settings.capture);
        Ok(())
    }

    // ── Plugin Block ──

    pub(super) fn execute_plugin_block(&mut self, settings: &PluginBlockSettings) -> crate::error::Result<()> {
        let pm = self.plugin_manager.as_ref()
            .ok_or_else(|| crate::error::AppError::Pipeline("No plugin manager available".into()))?;
        let settings_json = self.variables.interpolate(&settings.settings_json);
        let vars_snapshot = serde_json::to_string(&self.variables.snapshot()).unwrap_or_default();
        let (_, updated_vars, log) = pm.execute_block(&settings.plugin_block_type, &settings_json, &vars_snapshot)
            .map_err(|e| crate::error::AppError::Pipeline(e))?;
        for (k, v) in updated_vars {
            self.variables.set_user(&k, v, settings.capture);
        }
        // If plugin returned a log message and output_var has no value yet, store it
        if !log.is_empty() && self.variables.get(&settings.output_var).unwrap_or_default().is_empty() {
            self.variables.set_user(&settings.output_var, log, settings.capture);
        }
        Ok(())
    }

    // ── Captcha Solver (CapSolver / 2Captcha API) ──

    pub(super) async fn execute_captcha_solver(
        &mut self,
        settings: &CaptchaSolverSettings,
        sidecar_tx: &tokio::sync::mpsc::Sender<(SidecarRequest, tokio::sync::oneshot::Sender<SidecarResponse>)>,
    ) -> crate::error::Result<()> {
        let api_key = self.variables.interpolate(&settings.api_key);
        let site_key = self.variables.interpolate(&settings.site_key);
        let page_url = self.variables.interpolate(&settings.page_url);
        let timeout = std::time::Duration::from_millis(settings.timeout_ms);

        let (create_url, create_body): (String, String) = match settings.solver_service.as_str() {
            "capsolver" => {
                ("https://api.capsolver.com/createTask".into(),
                 serde_json::json!({
                     "clientKey": api_key,
                     "task": {
                         "type": match settings.captcha_type.as_str() {
                             "RecaptchaV3" => "RecaptchaV3TaskProxyless",
                             "HCaptcha" => "HCaptchaTaskProxyless",
                             _ => "RecaptchaV2TaskProxyless",
                         },
                         "websiteURL": page_url,
                         "websiteKey": site_key,
                     }
                 }).to_string())
            }
            _ => { // 2captcha
                ("https://api.2captcha.com/createTask".into(),
                 serde_json::json!({
                     "clientKey": api_key,
                     "task": {
                         "type": match settings.captcha_type.as_str() {
                             "RecaptchaV3" => "RecaptchaV3TaskProxyless",
                             "HCaptcha" => "HCaptchaTaskProxyless",
                             _ => "NoCaptchaTaskProxyless",
                         },
                         "websiteURL": page_url,
                         "websiteKey": site_key,
                     }
                 }).to_string())
            }
        };

        // Create task
        let req = SidecarRequest {
            id: Uuid::new_v4().to_string(),
            action: "request".into(),
            session: self.session_id.clone(),
            method: Some("POST".into()),
            url: Some(create_url.clone()),
            headers: Some(vec![vec!["Content-Type".into(), "application/json".into()]]),
            body: Some(create_body),
            timeout: Some(30000),
            proxy: None, browser: None, ja3: None, http2fp: None,
            follow_redirects: Some(true), max_redirects: Some(5),
            ssl_verify: None,
                    custom_ciphers: None,

            ..Default::default()
        };

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        sidecar_tx.send((req, resp_tx)).await
            .map_err(|_| crate::error::AppError::Sidecar("Failed to send captcha create".into()))?;
        let resp = resp_rx.await
            .map_err(|_| crate::error::AppError::Sidecar("Captcha create channel closed".into()))?;

        let json: serde_json::Value = serde_json::from_str(&resp.body).unwrap_or_default();
        let task_id = json["taskId"].as_str().unwrap_or("").to_string();
        if task_id.is_empty() {
            return Err(crate::error::AppError::Pipeline(format!("Captcha task creation failed: {}", resp.body)));
        }

        // Poll for result
        let poll_url = match settings.solver_service.as_str() {
            "capsolver" => "https://api.capsolver.com/getTaskResult",
            _ => "https://api.2captcha.com/getTaskResult",
        };
        let start = std::time::Instant::now();
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            let poll_body = serde_json::json!({
                "clientKey": api_key,
                "taskId": task_id,
            }).to_string();

            let req = SidecarRequest {
                id: Uuid::new_v4().to_string(),
                action: "request".into(),
                session: self.session_id.clone(),
                method: Some("POST".into()),
                url: Some(poll_url.into()),
                headers: Some(vec![vec!["Content-Type".into(), "application/json".into()]]),
                body: Some(poll_body),
                timeout: Some(15000),
                proxy: None, browser: None, ja3: None, http2fp: None,
                follow_redirects: Some(true), max_redirects: Some(5),
                ssl_verify: None,
                    custom_ciphers: None,

                ..Default::default()
            };

            let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
            sidecar_tx.send((req, resp_tx)).await.ok();
            if let Ok(resp) = resp_rx.await {
                let json: serde_json::Value = serde_json::from_str(&resp.body).unwrap_or_default();
                let status = json["status"].as_str().unwrap_or("");
                if status == "ready" {
                    let token = json["solution"]["gRecaptchaResponse"]
                        .as_str()
                        .or_else(|| json["solution"]["token"].as_str())
                        .unwrap_or("")
                        .to_string();
                    self.variables.set_user(&settings.output_var, token, settings.capture);
                    return Ok(());
                }
            }

            if start.elapsed() > timeout {
                return Err(crate::error::AppError::Pipeline("Captcha solver timeout".into()));
            }
        }
    }

    // ── Cloudflare Bypass (via FlareSolverr) ──

    pub(super) async fn execute_cloudflare_bypass(
        &mut self,
        settings: &CloudflareBypassSettings,
        sidecar_tx: &tokio::sync::mpsc::Sender<(SidecarRequest, tokio::sync::oneshot::Sender<SidecarResponse>)>,
    ) -> crate::error::Result<()> {
        let url = self.variables.interpolate(&settings.url);
        let flaresolverr = self.variables.interpolate(&settings.flaresolverr_url);

        let body = serde_json::json!({
            "cmd": "request.get",
            "url": url,
            "maxTimeout": settings.max_timeout_ms,
        }).to_string();

        let req = SidecarRequest {
            id: Uuid::new_v4().to_string(),
            action: "request".into(),
            session: self.session_id.clone(),
            method: Some("POST".into()),
            url: Some(flaresolverr),
            headers: Some(vec![vec!["Content-Type".into(), "application/json".into()]]),
            body: Some(body),
            timeout: Some(settings.max_timeout_ms as i64),
            proxy: None, browser: None, ja3: None, http2fp: None,
            follow_redirects: Some(true), max_redirects: Some(5),
            ssl_verify: None,
                    custom_ciphers: None,

            ..Default::default()
        };

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        sidecar_tx.send((req, resp_tx)).await
            .map_err(|_| crate::error::AppError::Sidecar("FlareSolverr send failed".into()))?;
        let resp = resp_rx.await
            .map_err(|_| crate::error::AppError::Sidecar("FlareSolverr channel closed".into()))?;

        let json: serde_json::Value = serde_json::from_str(&resp.body).unwrap_or_default();
        let cookies = json["solution"]["cookies"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| {
                        let name = c["name"].as_str()?;
                        let value = c["value"].as_str()?;
                        Some(format!("{}={}", name, value))
                    })
                    .collect::<Vec<_>>()
                    .join("; ")
            })
            .unwrap_or_default();

        let user_agent = json["solution"]["userAgent"].as_str().unwrap_or("").to_string();
        if !user_agent.is_empty() {
            self.variables.set_user("CF_USERAGENT", user_agent, false);
        }

        self.variables.set_user(&settings.output_var, cookies, settings.capture);
        Ok(())
    }

    // ── Akamai V3 Sensor Data ──
    // Algorithm credit: glizzykingdreko
    // https://github.com/glizzykingdreko/akamai-v3-sensor-data-helper

    pub(super) fn execute_akamai_v3_sensor(&mut self, settings: &AkamaiV3SensorSettings) -> crate::error::Result<()> {
        const ALLOWED_CHARS: &[u8] = b" !#$%&()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_`abcdefghijklmnopqrstuvwxyz{|}~";

        struct Prng {
            seed: u64,
        }

        impl Prng {
            fn new(seed: u64) -> Self {
                Self { seed }
            }

            fn next(&mut self) -> u16 {
                self.seed = (self.seed.wrapping_mul(65793) & 0xFFFFFFFF).wrapping_add(4282663);
                self.seed &= 0x7FFFFF;
                ((self.seed >> 8) & 0xFFFF) as u16
            }
        }

        match settings.mode {
            AkamaiV3Mode::Encrypt => {
                let payload = self.variables.get(&settings.payload_var).unwrap_or_default();
                let file_hash_str = self.variables.interpolate(&settings.file_hash);
                let cookie_hash_str = self.variables.interpolate(&settings.cookie_hash);
                let file_hash: u64 = file_hash_str.parse().unwrap_or(0);
                let cookie_hash: u64 = cookie_hash_str.parse().unwrap_or(8888888);

                // Step 1: Element swapping (seeded by file_hash)
                let mut elements: Vec<&str> = payload.split(':').collect();
                let n = elements.len();
                let mut prng = Prng::new(file_hash);
                let mut swap_pairs: Vec<(usize, usize)> = Vec::new();
                for _ in 0..n {
                    let a = prng.next() as usize % n;
                    let b = prng.next() as usize % n;
                    swap_pairs.push((a, b));
                }
                // Apply swaps forward
                for &(a, b) in &swap_pairs {
                    elements.swap(a, b);
                }
                let swapped = elements.join(":");

                // Step 2: Character substitution (seeded by cookie_hash)
                let mut prng2 = Prng::new(cookie_hash);
                let ac_len = ALLOWED_CHARS.len();
                let mut result = Vec::with_capacity(swapped.len());
                for &byte in swapped.as_bytes() {
                    if let Some(pos) = ALLOWED_CHARS.iter().position(|&c| c == byte) {
                        let offset = prng2.next() as usize;
                        let new_index = (pos + offset) % ac_len;
                        result.push(ALLOWED_CHARS[new_index]);
                    } else {
                        prng2.next(); // consume PRNG value even for non-allowed chars
                        result.push(byte);
                    }
                }
                let encrypted = String::from_utf8_lossy(&result).to_string();

                // Build sensor data format: 3;0;1;0;{cookie_hash};{static_hash};141659;{encrypted_data}
                let sensor_data = format!("3;0;1;0;{};{};141659;{}", cookie_hash, file_hash, encrypted);
                self.variables.set_user(&settings.output_var, sensor_data, settings.capture);
            }
            AkamaiV3Mode::Decrypt => {
                let sensor_data = self.variables.get(&settings.payload_var).unwrap_or_default();
                let file_hash_str = self.variables.interpolate(&settings.file_hash);
                let cookie_hash_str = self.variables.interpolate(&settings.cookie_hash);
                let file_hash: u64 = file_hash_str.parse().unwrap_or(0);
                let cookie_hash: u64 = cookie_hash_str.parse().unwrap_or(8888888);

                // Extract encrypted data from sensor format
                // Format: 3;0;1;0;{cookie_hash};{static_hash};141659;{encrypted_data}
                let encrypted = if let Some(idx) = sensor_data.find(";141659;") {
                    &sensor_data[idx + 8..]
                } else {
                    &sensor_data
                };

                // Step 1: Reverse character substitution (subtract offset)
                let mut prng2 = Prng::new(cookie_hash);
                let ac_len = ALLOWED_CHARS.len();
                let mut char_reversed = Vec::with_capacity(encrypted.len());
                for &byte in encrypted.as_bytes() {
                    if let Some(pos) = ALLOWED_CHARS.iter().position(|&c| c == byte) {
                        let offset = prng2.next() as usize;
                        let new_index = (pos + ac_len - (offset % ac_len)) % ac_len;
                        char_reversed.push(ALLOWED_CHARS[new_index]);
                    } else {
                        prng2.next(); // consume PRNG value
                        char_reversed.push(byte);
                    }
                }
                let char_reversed_str = String::from_utf8_lossy(&char_reversed).to_string();

                // Step 2: Reverse element swapping (apply swaps in reverse order)
                let mut elements: Vec<&str> = char_reversed_str.split(':').collect();
                let n = elements.len();
                let mut prng = Prng::new(file_hash);
                let mut swap_pairs: Vec<(usize, usize)> = Vec::new();
                for _ in 0..n {
                    let a = prng.next() as usize % n;
                    let b = prng.next() as usize % n;
                    swap_pairs.push((a, b));
                }
                // Apply swaps in reverse
                for &(a, b) in swap_pairs.iter().rev() {
                    elements.swap(a, b);
                }
                let decrypted = elements.join(":");

                self.variables.set_user(&settings.output_var, decrypted, settings.capture);
            }
            AkamaiV3Mode::ExtractCookieHash => {
                // Extract cookie_hash from bm_sz cookie value
                // decodeURIComponent(value).split('~')[2] parsed as int, default 8888888
                let bm_sz_raw = self.variables.get(&settings.payload_var).unwrap_or_default();
                let decoded = helpers::urldecoding(&bm_sz_raw);
                let cookie_hash = decoded
                    .split('~')
                    .nth(2)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(8888888);
                self.variables.set_user(&settings.output_var, cookie_hash.to_string(), settings.capture);
            }
        }
        Ok(())
    }

    // ── Laravel CSRF (HTTP GET + parse token) ──

    pub(super) async fn execute_laravel_csrf(
        &mut self,
        settings: &LaravelCsrfSettings,
        sidecar_tx: &tokio::sync::mpsc::Sender<(SidecarRequest, tokio::sync::oneshot::Sender<SidecarResponse>)>,
    ) -> crate::error::Result<()> {
        let url = self.variables.interpolate(&settings.url);
        let selector = self.variables.interpolate(&settings.csrf_selector);

        let req = SidecarRequest {
            id: Uuid::new_v4().to_string(),
            action: "request".into(),
            session: self.session_id.clone(),
            method: Some("GET".into()),
            url: Some(url),
            headers: Some(vec![]),
            body: Some(String::new()),
            timeout: Some(settings.timeout_ms as i64),
            proxy: self.proxy.clone(),
            browser: None,
            ja3: self.override_ja3.clone(),
            http2fp: self.override_http2fp.clone(),
            follow_redirects: Some(true),
            max_redirects: Some(5),
            ssl_verify: None,
                    custom_ciphers: None,

            ..Default::default()
        };

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        sidecar_tx.send((req, resp_tx)).await
            .map_err(|_| crate::error::AppError::Sidecar("CSRF request send failed".into()))?;
        let resp = resp_rx.await
            .map_err(|_| crate::error::AppError::Sidecar("CSRF response channel closed".into()))?;

        // Parse CSRF token from HTML using CSS selector
        let doc = scraper::Html::parse_document(&resp.body);
        let sel = scraper::Selector::parse(&selector)
            .map_err(|e| crate::error::AppError::Pipeline(format!("Invalid CSRF selector: {:?}", e)))?;

        let token = doc.select(&sel)
            .next()
            .and_then(|el| el.value().attr("value").or_else(|| el.value().attr("content")))
            .unwrap_or("")
            .to_string();

        // Also extract XSRF cookie if present
        if !settings.cookie_name.is_empty() {
            if let Some(ref cookies) = resp.cookies {
                if let Some(cookie_val) = cookies.get(&settings.cookie_name) {
                    self.variables.set_user(&format!("{}_COOKIE", settings.output_var), cookie_val.clone(), false);
                }
            }
        }

        self.variables.set_user(&settings.output_var, token, settings.capture);
        Ok(())
    }
}
