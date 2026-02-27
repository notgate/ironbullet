use uuid::Uuid;

use super::*; // includes TlsClient via block::settings_http::*

impl ExecutionContext {
    pub(super) async fn execute_http_request(
        &mut self,
        block: &Block,
        settings: &HttpRequestSettings,
        sidecar_tx: &tokio::sync::mpsc::Sender<(SidecarRequest, tokio::sync::oneshot::Sender<SidecarResponse>)>,
    ) -> crate::error::Result<()> {
        // ── Interpolate common request fields ─────────────────────────────────
        let url  = self.variables.interpolate(&settings.url);
        let body = self.variables.interpolate(&settings.body);
        let mut headers: Vec<Vec<String>> = settings.headers.iter()
            .map(|(k, v)| vec![
                self.variables.interpolate(k),
                self.variables.interpolate(v),
            ])
            .collect();

        // Inject custom cookies as a Cookie header (one per line: name=value)
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

        // ── Build the protocol-level request (shared by both backends) ────────
        let sidecar_req = SidecarRequest {
            id: Uuid::new_v4().to_string(),
            action: "request".into(),
            session: self.session_id.clone(),
            method: Some(settings.method.clone()),
            url: Some(url.clone()),
            headers: Some(headers.clone()),
            body: Some(body.clone()),
            timeout: Some(settings.timeout_ms as i64),
            proxy: self.proxy.clone(),
            // AzureTLS-specific fields — only populated when using the sidecar
            browser: None,
            ja3: self.override_ja3.clone(),
            http2fp: self.override_http2fp.clone(),
            follow_redirects: Some(settings.follow_redirects),
            max_redirects: Some(settings.max_redirects as i64),
            ssl_verify: if settings.ssl_verify { None } else { Some(false) },
            custom_ciphers: if settings.cipher_suites.is_empty() { None } else { Some(settings.cipher_suites.clone()) },
        };

        // ── Dispatch to the chosen TLS backend ────────────────────────────────
        let resp = match settings.tls_client {
            TlsClient::RustTLS => {
                // Native reqwest + rustls — no sidecar, per-request client config.
                // All standard settings apply; JA3/browser fingerprinting is N/A.
                crate::sidecar::native::execute_rustls_request(
                    &sidecar_req,
                    settings.ssl_verify,
                ).await
            }
            TlsClient::AzureTLS => {
                // Go sidecar path — azuretls with JA3/TLS fingerprinting support.
                let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
                sidecar_tx.send((sidecar_req, resp_tx)).await
                    .map_err(|_| crate::error::AppError::Sidecar(
                        "Failed to send request to sidecar".into()
                    ))?;
                resp_rx.await
                    .map_err(|_| crate::error::AppError::Sidecar(
                        "Sidecar response channel closed".into()
                    ))?
            }
        };

        if let Some(ref err) = resp.error {
            if !err.is_empty() {
                return Err(crate::error::AppError::Sidecar(err.clone()));
            }
        }

        // ── Store response into pipeline variables ────────────────────────────
        let var_prefix = if settings.response_var.is_empty() { "SOURCE" } else { &settings.response_var };

        // Body
        self.variables.set_data(var_prefix, resp.body.clone());
        // Status code
        self.variables.set_data(&format!("{}.STATUS", var_prefix), resp.status.to_string());
        // Final URL (after redirects)
        self.variables.set_data(&format!("{}.URL", var_prefix), resp.final_url.clone());
        // Response headers as JSON object
        if let Some(ref hdrs) = resp.headers {
            let hdr_str = serde_json::to_string(hdrs).unwrap_or_default();
            self.variables.set_data(&format!("{}.HEADERS", var_prefix), hdr_str);

            // Also expose individual headers as SOURCE.HEADERS.<Name>
            for (name, value) in hdrs {
                self.variables.set_data(
                    &format!("{}.HEADERS.{}", var_prefix, name.to_lowercase()),
                    value.clone(),
                );
            }
        }
        // Cookies as JSON object
        if let Some(ref cookies) = resp.cookies {
            let cookie_str = serde_json::to_string(cookies).unwrap_or_default();
            self.variables.set_data(&format!("{}.COOKIES", var_prefix), cookie_str);

            // Also expose individual cookies as SOURCE.COOKIES.<Name>
            for (name, value) in cookies {
                self.variables.set_data(
                    &format!("{}.COOKIES.{}", var_prefix, name),
                    value.clone(),
                );
            }
        }

        // Backward-compat aliases used by legacy checks
        self.variables.set_data("RESPONSECODE", resp.status.to_string());
        self.variables.set_data("ADDRESS", resp.final_url.clone());

        // ── Update block result (shown in UI block overlay) ───────────────────
        if let Some(last) = self.block_results.last_mut() {
            last.request = Some(RequestInfo {
                method: settings.method.clone(),
                url: url.clone(),
                headers: headers.iter().map(|h| (
                    h.get(0).cloned().unwrap_or_default(),
                    h.get(1).cloned().unwrap_or_default(),
                )).collect(),
                body: body.clone(),
            });
            last.response = Some(ResponseInfo {
                status_code: resp.status as u16,
                headers: resp.headers.clone().unwrap_or_default(),
                body: resp.body.clone(),
                final_url: resp.final_url.clone(),
                cookies: resp.cookies.clone().unwrap_or_default(),
                timing_ms: resp.timing_ms as u64,
            });
        }

        // ── Network log entry ─────────────────────────────────────────────────
        let cookies_sent: Vec<(String, String)> = settings.custom_cookies.lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .filter_map(|l| l.split_once('=').map(|(k, v)| (k.trim().to_string(), v.trim().to_string())))
            .collect();
        let cookies_set: Vec<(String, String)> = resp.cookies.as_ref()
            .map(|c| c.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();

        self.network_log.push(NetworkEntry {
            block_id: block.id,
            block_label: block.label.clone(),
            method: settings.method.clone(),
            url,
            status_code: resp.status as u16,
            timing_ms: resp.timing_ms as u64,
            response_size: resp.body.len(),
            cookies_set,
            cookies_sent,
        });

        Ok(())
    }
}
