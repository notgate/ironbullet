use uuid::Uuid;

use super::*;

impl ExecutionContext {
    pub(super) async fn execute_http_request(
        &mut self,
        block: &Block,
        settings: &HttpRequestSettings,
        sidecar_tx: &tokio::sync::mpsc::Sender<(SidecarRequest, tokio::sync::oneshot::Sender<SidecarResponse>)>,
    ) -> crate::error::Result<()> {
        let url = self.variables.interpolate(&settings.url);
        let body = self.variables.interpolate(&settings.body);
        let mut headers: Vec<Vec<String>> = settings.headers.iter()
            .map(|(k, v)| vec![
                self.variables.interpolate(k),
                self.variables.interpolate(v),
            ])
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
            url: Some(url.clone()),
            headers: Some(headers.clone()),
            body: Some(body.clone()),
            timeout: Some(settings.timeout_ms as i64),
            proxy: self.proxy.clone(),
            browser: None,
            ja3: self.override_ja3.clone(),
            http2fp: self.override_http2fp.clone(),
            follow_redirects: Some(settings.follow_redirects),
            max_redirects: Some(settings.max_redirects as i64),
            ssl_verify: if settings.ssl_verify { None } else { Some(false) },
            custom_ciphers: if settings.cipher_suites.is_empty() { None } else { Some(settings.cipher_suites.clone()) },
        };

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        sidecar_tx.send((req, resp_tx)).await
            .map_err(|_| crate::error::AppError::Sidecar("Failed to send request to sidecar".into()))?;

        let resp = resp_rx.await
            .map_err(|_| crate::error::AppError::Sidecar("Sidecar response channel closed".into()))?;

        if let Some(ref err) = resp.error {
            if !err.is_empty() {
                return Err(crate::error::AppError::Sidecar(err.clone()));
            }
        }

        // Store response data in named variables
        let var_prefix = if settings.response_var.is_empty() { "SOURCE" } else { &settings.response_var };
        self.variables.set_data(var_prefix, resp.body.clone());
        self.variables.set_data(&format!("{}.STATUS", var_prefix), resp.status.to_string());
        self.variables.set_data(&format!("{}.URL", var_prefix), resp.final_url.clone());
        if let Some(ref hdrs) = resp.headers {
            let hdr_str = serde_json::to_string(hdrs).unwrap_or_default();
            self.variables.set_data(&format!("{}.HEADERS", var_prefix), hdr_str);
        }
        if let Some(ref cookies) = resp.cookies {
            let cookie_str = serde_json::to_string(cookies).unwrap_or_default();
            self.variables.set_data(&format!("{}.COOKIES", var_prefix), cookie_str);
        }
        // Backward compat: always set RESPONSECODE and ADDRESS
        self.variables.set_data("RESPONSECODE", resp.status.to_string());
        self.variables.set_data("ADDRESS", resp.final_url.clone());

        // Update block result with request/response info
        if let Some(last) = self.block_results.last_mut() {
            last.request = Some(RequestInfo {
                method: settings.method.clone(),
                url: url.clone(),
                headers: headers.iter().map(|h| {
                    (h.get(0).cloned().unwrap_or_default(), h.get(1).cloned().unwrap_or_default())
                }).collect(),
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

        // Add to network log
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
