//! Native HTTP backend using reqwest — replaces Go sidecar for debug execution.
//!
//! Returns a channel sender with the same type signature as SidecarManager::start(),
//! so ExecutionContext needs zero changes.

use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::{mpsc, oneshot};

use super::protocol::{SidecarRequest, SidecarResponse};

/// Create an in-process HTTP backend that speaks the sidecar protocol.
/// Spawns a background task that processes requests using reqwest.
pub fn create_native_backend() -> mpsc::Sender<(SidecarRequest, oneshot::Sender<SidecarResponse>)> {
    let (tx, mut rx) = mpsc::channel::<(SidecarRequest, oneshot::Sender<SidecarResponse>)>(64);

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::limited(10))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default();

    tokio::spawn(async move {
        while let Some((req, resp_tx)) = rx.recv().await {
            let client = client.clone();
            tokio::spawn(async move {
                let response = execute_request(&client, &req).await;
                let _ = resp_tx.send(response);
            });
        }
    });

    tx
}

async fn execute_request(client: &reqwest::Client, req: &SidecarRequest) -> SidecarResponse {
    let id = req.id.clone();

    // Only handle "request" action; other actions are no-ops
    if req.action != "request" {
        return SidecarResponse {
            id,
            status: 0,
            headers: None,
            body: String::new(),
            cookies: None,
            final_url: String::new(),
            error: None,
            timing_ms: 0,
        };
    }

    let url = match &req.url {
        Some(u) => u.clone(),
        None => {
            return SidecarResponse {
                id,
                status: 0,
                headers: None,
                body: String::new(),
                cookies: None,
                final_url: String::new(),
                error: Some("No URL provided".into()),
                timing_ms: 0,
            };
        }
    };

    let method = req.method.as_deref().unwrap_or("GET");
    let reqwest_method = match method.to_uppercase().as_str() {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        "DELETE" => reqwest::Method::DELETE,
        "PATCH" => reqwest::Method::PATCH,
        "HEAD" => reqwest::Method::HEAD,
        "OPTIONS" => reqwest::Method::OPTIONS,
        _ => reqwest::Method::GET,
    };

    // Build the request
    let mut builder = client.request(reqwest_method, &url);

    // Set timeout if specified
    if let Some(timeout_ms) = req.timeout {
        if timeout_ms > 0 {
            builder = builder.timeout(std::time::Duration::from_millis(timeout_ms as u64));
        }
    }

    // Set headers
    if let Some(ref headers) = req.headers {
        for pair in headers {
            if pair.len() >= 2 {
                if let Ok(name) = reqwest::header::HeaderName::from_bytes(pair[0].as_bytes()) {
                    if let Ok(value) = reqwest::header::HeaderValue::from_str(&pair[1]) {
                        builder = builder.header(name, value);
                    }
                }
            }
        }
    }

    // Set body
    if let Some(ref body) = req.body {
        if !body.is_empty() {
            builder = builder.body(body.clone());
        }
    }

    // Execute
    let start = Instant::now();
    match builder.send().await {
        Ok(resp) => {
            let timing_ms = start.elapsed().as_millis() as i64;
            let status = resp.status().as_u16() as i32;
            let final_url = resp.url().to_string();

            // Collect headers
            let mut headers = HashMap::new();
            for (name, value) in resp.headers() {
                if let Ok(v) = value.to_str() {
                    headers.insert(name.to_string(), v.to_string());
                }
            }

            // Extract cookies from set-cookie headers
            let mut cookies = HashMap::new();
            for value in resp.headers().get_all(reqwest::header::SET_COOKIE) {
                if let Ok(v) = value.to_str() {
                    // Parse "name=value; ..." format
                    if let Some(eq_pos) = v.find('=') {
                        let name = &v[..eq_pos];
                        let rest = &v[eq_pos + 1..];
                        let value = rest.split(';').next().unwrap_or(rest);
                        cookies.insert(name.to_string(), value.to_string());
                    }
                }
            }

            // Read body
            let body = resp.text().await.unwrap_or_default();

            SidecarResponse {
                id,
                status,
                headers: Some(headers),
                body,
                cookies: if cookies.is_empty() { None } else { Some(cookies) },
                final_url,
                error: None,
                timing_ms,
            }
        }
        Err(e) => {
            let timing_ms = start.elapsed().as_millis() as i64;
            SidecarResponse {
                id,
                status: 0,
                headers: None,
                body: String::new(),
                cookies: None,
                final_url: url,
                error: Some(e.to_string()),
                timing_ms,
            }
        }
    }
}
