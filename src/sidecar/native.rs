//! Native HTTP backend using reqwest+rustls — no Go sidecar needed.
//!
//! Two entrypoints:
//!  - `create_native_backend()`: persistent channel-based backend for debug runs
//!    (builds one client shared across requests, fast for simple testing).
//!  - `execute_rustls_request()`: per-request function used by the full runner
//!    when a block's tls_client is set to RustTLS. Builds a fresh client each
//!    call so that per-request settings (proxy, ssl_verify, redirects) take effect.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};

use super::protocol::{SidecarRequest, SidecarResponse};

// ── Persistent backend (debug runner) ──────────────────────────────────────

/// Create an in-process HTTP backend that speaks the sidecar protocol.
/// Spawns a background task that processes requests using reqwest+rustls.
/// Used by debug_pipeline; shares a single client instance.
pub fn create_native_backend() -> mpsc::Sender<(SidecarRequest, oneshot::Sender<SidecarResponse>)> {
    let (tx, mut rx) = mpsc::channel::<(SidecarRequest, oneshot::Sender<SidecarResponse>)>(64);

    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .danger_accept_invalid_certs(true)  // debug-friendly
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::limited(10))
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap_or_default();

    tokio::spawn(async move {
        while let Some((req, resp_tx)) = rx.recv().await {
            let client = client.clone();
            tokio::spawn(async move {
                let response = execute_with_client(&client, &req).await;
                let _ = resp_tx.send(response);
            });
        }
    });

    tx
}

// ── Per-request RustTLS backend (full runner) ───────────────────────────────

/// Execute a single HTTP request using reqwest+rustls.
///
/// Accepts an optional pre-built client for cookie persistence across multiple HTTP
/// blocks in the same pipeline execution. Returns the client so the caller can reuse
/// it for subsequent requests (cookie jar survives between blocks, matching how
/// azuretls sessions work). A None `existing_client` builds a fresh one.
pub async fn execute_rustls_request(
    req: &SidecarRequest,
    ssl_verify: bool,
    proxy_insecure: bool,
    existing_client: Option<reqwest::Client>,
) -> (SidecarResponse, reqwest::Client) {
    let id = req.id.clone();

    let client = if let Some(c) = existing_client {
        // Reuse existing client (shared cookie jar, same TLS session pool).
        // Note: ssl_verify and proxy changes between blocks take effect on
        // newly established connections; existing keep-alive connections are
        // unaffected. For per-block proxy changes use AzureTLS instead.
        c
    } else {
        // Build a new session-scoped client with all per-block settings.
        let mut builder = reqwest::Client::builder()
            .use_rustls_tls()
            .cookie_store(true)
            // Accept invalid certs when: ssl_verify=false (target TLS disabled) OR
            // proxy_insecure=true (HTTPS proxy with self-signed cert).
            // Note: reqwest/rustls applies this at the client level — it affects
            // both the proxy CONNECT handshake and the target TLS handshake.
            // To skip only the proxy TLS, use AzureTLS instead.
            .danger_accept_invalid_certs(!ssl_verify || proxy_insecure)
            // Connect timeout separate from total request timeout —
            // prevents dead hosts from hanging for OS TCP timeout (~2 min)
            .connect_timeout(Duration::from_secs(10))
            // Cap idle connections per host to avoid fd exhaustion at high thread counts.
            // reqwest default is unbounded; at 200+ workers this can leak descriptors.
            .pool_max_idle_per_host(2)
            // Aggressively evict idle connections — on Windows, connections sitting in
            // the pool still hold their local port. Short TTL means ports are released
            // quickly, reducing TIME_WAIT accumulation at high thread counts.
            .pool_idle_timeout(Duration::from_secs(4))
            // TCP keepalive probes to detect silently dead connections behind proxies.
            .tcp_keepalive(Duration::from_secs(30))
            // Disable Nagle — send data immediately without waiting to coalesce packets.
            // Reduces first-byte latency on POST requests.
            .tcp_nodelay(true);

        if let Some(ref proxy_str) = req.proxy {
            if !proxy_str.is_empty() {
                match reqwest::Proxy::all(proxy_str) {
                    Ok(proxy) => { builder = builder.proxy(proxy); }
                    Err(e) => {
                        let err = error_response(id, 0, String::new(), format!("Invalid proxy '{}': {}", proxy_str, e));
                        let fallback = reqwest::Client::builder().use_rustls_tls().build().unwrap_or_default();
                        return (err, fallback);
                    }
                }
            }
        }

        let follow = req.follow_redirects.unwrap_or(true);
        let max_redirects = req.max_redirects.unwrap_or(8) as usize;
        builder = if follow && max_redirects > 0 {
            builder.redirect(reqwest::redirect::Policy::limited(max_redirects))
        } else {
            builder.redirect(reqwest::redirect::Policy::none())
        };

        match builder.build() {
            Ok(c) => c,
            Err(e) => {
                let err = error_response(id, 0, String::new(), format!("Client build error: {}", e));
                let fallback = reqwest::Client::builder().use_rustls_tls().build().unwrap_or_default();
                return (err, fallback);
            }
        }
    };

    let resp = execute_with_client(&client, req).await;
    (resp, client)
}

// ── Core request execution ──────────────────────────────────────────────────

async fn execute_with_client(client: &reqwest::Client, req: &SidecarRequest) -> SidecarResponse {
    let id = req.id.clone();

    if req.action != "request" {
        // No-op for session management actions (those are AzureTLS sidecar concepts)
        return ok_response(id, 0, String::new(), String::new(), None, None, 0);
    }

    let url = match &req.url {
        Some(u) if !u.is_empty() => u.clone(),
        _ => return error_response(id, 0, String::new(), "No URL provided".into()),
    };

    // Method
    let method = match req.method.as_deref().unwrap_or("GET").to_uppercase().as_str() {
        "GET"     => reqwest::Method::GET,
        "POST"    => reqwest::Method::POST,
        "PUT"     => reqwest::Method::PUT,
        "DELETE"  => reqwest::Method::DELETE,
        "PATCH"   => reqwest::Method::PATCH,
        "HEAD"    => reqwest::Method::HEAD,
        "OPTIONS" => reqwest::Method::OPTIONS,
        "TRACE"   => reqwest::Method::TRACE,
        "CONNECT" => reqwest::Method::CONNECT,
        other     => {
            return error_response(id, 0, url, format!("Unknown HTTP method: {}", other));
        }
    };

    let mut rb = client.request(method, &url);

    // Timeout
    if let Some(ms) = req.timeout {
        if ms > 0 {
            rb = rb.timeout(Duration::from_millis(ms as u64));
        }
    }

    // Headers — apply in order, last value wins for duplicates
    if let Some(ref headers) = req.headers {
        for pair in headers {
            if pair.len() >= 2 {
                match (
                    reqwest::header::HeaderName::from_bytes(pair[0].trim().as_bytes()),
                    reqwest::header::HeaderValue::from_str(pair[1].trim()),
                ) {
                    (Ok(name), Ok(value)) => { rb = rb.header(name, value); }
                    (Err(e), _) => tracing_warn(&format!("Bad header name '{}': {}", pair[0], e)),
                    (_, Err(e)) => tracing_warn(&format!("Bad header value for '{}': {}", pair[0], e)),
                }
            }
        }
    }

    // Body
    if let Some(ref body) = req.body {
        if !body.is_empty() {
            rb = rb.body(body.clone());
        }
    }

    // ── Execute ──
    let start = Instant::now();
    match rb.send().await {
        Ok(resp) => {
            let timing_ms = start.elapsed().as_millis() as i64;
            let status     = resp.status().as_u16() as i32;
            let final_url  = resp.url().to_string();

            // Collect response headers (multi-value: last wins for duplicates)
            let mut hdrs: HashMap<String, String> = HashMap::new();
            for (name, value) in resp.headers() {
                if let Ok(v) = value.to_str() {
                    hdrs.insert(name.to_string(), v.to_string());
                }
            }

            // Extract cookies from the response cookie jar.
            // NOTE: when cookie_store(true) is enabled, reqwest moves Set-Cookie headers
            // into the internal jar and strips them from resp.headers() — so reading
            // resp.headers().get_all(SET_COOKIE) always returns empty.
            // resp.cookies() iterates the cookies set by THIS response (from the jar),
            // which is the correct source.
            let mut cookies: HashMap<String, String> = HashMap::new();
            for cookie in resp.cookies() {
                cookies.insert(cookie.name().to_string(), cookie.value().to_string());
            }

            let body = resp.text().await.unwrap_or_default();

            ok_response(
                id, status, final_url, body,
                Some(hdrs),
                if cookies.is_empty() { None } else { Some(cookies) },
                timing_ms,
            )
        }
        Err(e) => {
            let timing_ms = start.elapsed().as_millis() as i64;
            let detail = format_reqwest_error(&e, &url);
            SidecarResponse {
                id, status: 0, body: String::new(), final_url: url,
                headers: None, cookies: None, error: Some(detail), timing_ms,
                ..Default::default()
            }
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn ok_response(
    id: String, status: i32, final_url: String, body: String,
    headers: Option<HashMap<String, String>>,
    cookies: Option<HashMap<String, String>>,
    timing_ms: i64,
) -> SidecarResponse {
    SidecarResponse { id, status, final_url, body, headers, cookies, error: None, timing_ms, ..Default::default() }
}

fn error_response(id: String, status: i32, final_url: String, error: String) -> SidecarResponse {
    SidecarResponse {
        id, status, final_url, body: String::new(),
        headers: None, cookies: None, error: Some(error), timing_ms: 0,
        ..Default::default()
    }
}

/// Enrich reqwest errors with actionable context hints.
fn format_reqwest_error(e: &reqwest::Error, url: &str) -> String {
    if e.is_timeout() {
        return format!("Request timed out: {}", url);
    }
    if e.is_connect() {
        return format!("Connection refused or host unreachable: {}", url);
    }
    if e.is_request() {
        return format!("Invalid request (bad URL or headers?): {}", e);
    }
    // TLS errors
    let s = e.to_string();
    if s.contains("certificate") || s.contains("tls") || s.contains("ssl") || s.contains("TLS") {
        return format!(
            "TLS error — try enabling ssl_verify=false if using self-signed certs: {}",
            s
        );
    }
    format!("{}", e)
}

fn tracing_warn(msg: &str) {
    eprintln!("[rustls-backend] warn: {}", msg);
}
