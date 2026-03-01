use std::sync::Arc;
use tokio::sync::Mutex;

use ironbullet::pipeline::engine::ExecutionContext;
use ironbullet::runner::{RunnerOrchestrator, HitResult};
use ironbullet::runner::data_pool::DataPool;
use ironbullet::runner::proxy_pool::ProxyPool;

use super::{resolve_sidecar_path, AppState, IpcResponse};

pub(super) fn debug_pipeline(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let s = state.lock().await;
            // If the frontend sends a full pipeline snapshot, use it (keeps settings in sync)
            let frontend_pipeline = data.get("pipeline")
                .and_then(|v| serde_json::from_value::<ironbullet::pipeline::Pipeline>(v.clone()).ok());
            let mut blocks = frontend_pipeline.as_ref().map(|p| p.blocks.clone()).unwrap_or_else(|| s.pipeline.blocks.clone());
            // If block_ids is provided, restrict execution to only those blocks (Debug Block feature)
            if let Some(ids_val) = data.get("block_ids") {
                if let Some(ids) = ids_val.as_array() {
                    let ids: std::collections::HashSet<String> = ids.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    if !ids.is_empty() {
                        blocks.retain(|b| ids.contains(&b.id.to_string()));
                    }
                }
            }
            let data_settings = frontend_pipeline.as_ref().map(|p| p.data_settings.clone()).unwrap_or_else(|| s.pipeline.data_settings.clone());
            let pm = s.plugin_manager.clone();
            drop(s); // Release lock before async execution

            let native_tx = ironbullet::sidecar::native::create_native_backend();

            let mut ctx = ExecutionContext::new(uuid::Uuid::new_v4().to_string());
            ctx.plugin_manager = Some(pm);

            // Populate test data variables from frontend
            if let Some(test_line) = data.get("test_data_line").and_then(|v| v.as_str()) {
                if !test_line.is_empty() {
                    let parts: Vec<&str> = test_line.split(data_settings.separator).collect();
                    for (i, slice_name) in data_settings.slices.iter().enumerate() {
                        if let Some(part) = parts.get(i) {
                            ctx.variables.set_input(slice_name, part.to_string());
                        }
                    }
                }
            }
            if let Some(test_proxy) = data.get("test_proxy").and_then(|v| v.as_str()) {
                if !test_proxy.is_empty() {
                    ctx.proxy = Some(test_proxy.to_string());
                }
            }

            let exec_result = ctx.execute_blocks(&blocks, &native_tx).await;

            // Send last result as debug_step (backward compat)
            let result = ctx.block_results.last().cloned();
            let resp = IpcResponse::ok("debug_step", serde_json::to_value(&result).unwrap_or_default());
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));

            // Always send all block results for the response viewer
            let resp = IpcResponse::ok("debug_results", serde_json::to_value(&ctx.block_results).unwrap_or_default());
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));

            // Send network log for network viewer
            let resp = IpcResponse::ok("network_log", serde_json::to_value(&ctx.network_log).unwrap_or_default());
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));

            // If there was an error, also send it as a log message
            if let Err(e) = exec_result {
                let resp = IpcResponse::err("debug_step", format!("Pipeline error: {}", e));
                eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
            }
        });
    }
}

pub(super) fn start_runner(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        let inner_handle = handle.clone();
        handle.spawn(async move {
            let threads = data.get("threads").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
            let mut s = state.lock().await;

            // Start sidecar if needed
            let sidecar_path = resolve_sidecar_path(&s.config.sidecar_path);
            if !s.sidecar.is_running() {
                match s.sidecar.start(&sidecar_path).await {
                    Ok(_) => {}
                    Err(e) => {
                        let resp = IpcResponse::err("runner_error", format!("Failed to start sidecar: {}", e));
                        eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                        return;
                    }
                }
            }

            // Get a fresh sidecar tx by restarting
            s.sidecar.stop().await;
            let sidecar_tx = match s.sidecar.start(&sidecar_path).await {
                Ok(tx) => tx,
                Err(e) => {
                    let resp = IpcResponse::err("runner_error", format!("Failed to start sidecar: {}", e));
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                    return;
                }
            };

            // Load data from wordlist file
            let wordlist_path = data.get("wordlist_path").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let data_pool = if !wordlist_path.is_empty() {
                match DataPool::from_file(&wordlist_path, true) {
                    Ok(dp) => dp,
                    Err(e) => {
                        let resp = IpcResponse::err("runner_error", format!("Failed to load wordlist '{}': {}", wordlist_path, e));
                        eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                        return;
                    }
                }
            } else {
                // Try inline data from the data field
                let lines: Vec<String> = data.get("data_lines")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();
                DataPool::new(lines)
            };

            // Load proxies from file path if provided
            let proxy_path = data.get("proxy_path").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let proxy_pool = if !proxy_path.is_empty() {
                match ProxyPool::from_file(&proxy_path, s.pipeline.proxy_settings.ban_duration_secs as u64) {
                    Ok(pp) => pp,
                    Err(e) => {
                        let resp = IpcResponse::err("runner_error", format!("Failed to load proxies '{}': {}", proxy_path, e));
                        eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                        return;
                    }
                }
            } else {
                ProxyPool::empty()
            };

            let (hits_tx, mut hits_rx) = tokio::sync::mpsc::channel::<HitResult>(1024);

            // Use frontend pipeline snapshot if provided (keeps runner_settings, proxy_settings, etc. in sync)
            let pipeline = data.get("pipeline")
                .and_then(|v| serde_json::from_value::<ironbullet::pipeline::Pipeline>(v.clone()).ok())
                .unwrap_or_else(|| s.pipeline.clone());
            let proxy_mode = pipeline.proxy_settings.proxy_mode.clone();
            let pm = s.plugin_manager.clone();
            let runner = Arc::new(RunnerOrchestrator::new(
                pipeline,
                proxy_mode,
                data_pool,
                proxy_pool,
                sidecar_tx,
                threads,
                hits_tx,
                Some(pm),
            ));

            s.runner = Some(runner.clone());
            s.hits.clear();

            // Drop the lock before spawning long-running tasks
            drop(s);

            // Wrap eval_js in Arc<Mutex> so both spawned tasks can use it
            let eval_js = Arc::new(tokio::sync::Mutex::new(eval_js));

            // Notify frontend that runner has started
            {
                let ejs = eval_js.lock().await;
                let resp = IpcResponse::ok("runner_started", serde_json::json!(null));
                ejs(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
            }

            // Spawn hit collector -- streams each hit to frontend
            let state2 = state.clone();
            let eval_js2 = eval_js.clone();
            inner_handle.spawn(async move {
                while let Some(hit) = hits_rx.recv().await {
                    let hit_data = serde_json::json!({
                        "data_line": hit.data_line,
                        "captures": hit.captures,
                        "proxy": hit.proxy,
                    });
                    let resp = IpcResponse::ok("runner_hit", hit_data);
                    let ejs = eval_js2.lock().await;
                    ejs(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                    drop(ejs);
                    let mut s = state2.lock().await;
                    s.hits.push(hit);
                }
            });

            // Spawn runner
            let eval_js3 = eval_js.clone();
            inner_handle.spawn(async move {
                runner.start().await;
                // Runner finished -- notify frontend
                let ejs = eval_js3.lock().await;
                let resp = IpcResponse::ok("runner_stopped", serde_json::json!(null));
                ejs(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
            });
        });
    }
}

pub(super) fn pause_runner(
    state: Arc<Mutex<AppState>>,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let s = state.lock().await;
            if let Some(ref runner) = s.runner {
                runner.pause();
            }
        });
    }
}

pub(super) fn resume_runner(
    state: Arc<Mutex<AppState>>,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let s = state.lock().await;
            if let Some(ref runner) = s.runner {
                runner.resume();
            }
        });
    }
}

pub(super) fn stop_runner(
    state: Arc<Mutex<AppState>>,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let s = state.lock().await;
            if let Some(ref runner) = s.runner {
                runner.stop();
            }
        });
    }
}

pub(super) fn get_runner_stats(
    state: Arc<Mutex<AppState>>,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let s = state.lock().await;
            let stats = if let Some(ref runner) = s.runner {
                Some(runner.get_stats())
            } else {
                None
            };
            let resp = IpcResponse::ok("runner_stats", serde_json::to_value(&stats).unwrap_or_default());
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
        });
    }
}

pub(super) fn check_proxies(
    state: Arc<Mutex<AppState>>,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let state = state.clone();
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let s = state.lock().await;
            let mut all_sources = s.pipeline.proxy_settings.proxy_sources.clone();
            // Also gather sources from active proxy group
            if !s.pipeline.proxy_settings.active_group.is_empty() {
                if let Some(group) = s.pipeline.proxy_settings.proxy_groups.iter()
                    .find(|g| g.name == s.pipeline.proxy_settings.active_group) {
                    all_sources = group.sources.clone();
                }
            }
            drop(s);

            // Load proxies from sources
            let mut proxies: Vec<String> = Vec::new();
            for src in &all_sources {
                match src.source_type {
                    ironbullet::pipeline::ProxySourceType::File => {
                        if let Ok(content) = std::fs::read_to_string(&src.value) {
                            proxies.extend(content.lines().filter(|l| !l.trim().is_empty()).map(|l| l.trim().to_string()));
                        }
                    }
                    ironbullet::pipeline::ProxySourceType::Inline => {
                        proxies.extend(src.value.lines().filter(|l| !l.trim().is_empty()).map(|l| l.trim().to_string()));
                    }
                    ironbullet::pipeline::ProxySourceType::Url => {
                        if let Ok(resp) = reqwest::get(&src.value).await {
                            if let Ok(text) = resp.text().await {
                                proxies.extend(text.lines().filter(|l| !l.trim().is_empty()).map(|l| l.trim().to_string()));
                            }
                        }
                    }
                }
            }

            let total = proxies.len();
            let mut alive = 0u32;
            let mut dead = 0u32;

            // Simple connectivity check: try to connect through each proxy
            for proxy_str in &proxies {
                let check_result = async {
                    let proxy = reqwest::Proxy::all(proxy_str).map_err(|e| e.to_string())?;
                    let client = reqwest::Client::builder()
                        .proxy(proxy)
                        .timeout(std::time::Duration::from_secs(8))
                        .build()
                        .map_err(|e| e.to_string())?;
                    client.get("https://httpbin.org/ip")
                        .send()
                        .await
                        .map_err(|e| e.to_string())?;
                    Ok::<_, String>(())
                }.await;

                match check_result {
                    Ok(_) => alive += 1,
                    Err(_) => dead += 1,
                }
            }

            let resp = IpcResponse::ok("proxy_check_result", serde_json::json!({
                "alive": alive,
                "dead": dead,
                "total": total,
            }));
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
        });
    }
}

pub(super) fn probe_url(
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let url = data.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
            if url.is_empty() {
                let resp = IpcResponse::ok("probe_result", serde_json::json!({ "error": "No URL provided" }));
                eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                return;
            }

            let start = std::time::Instant::now();
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .redirect(reqwest::redirect::Policy::limited(5))
                .danger_accept_invalid_certs(true)
                .default_headers({
                    let mut h = reqwest::header::HeaderMap::new();
                    h.insert(reqwest::header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".parse().unwrap());
                    h.insert(reqwest::header::ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".parse().unwrap());
                    h.insert(reqwest::header::ACCEPT_LANGUAGE, "en-US,en;q=0.9".parse().unwrap());
                    h
                })
                .cookie_store(true)
                .build();

            let client = match client {
                Ok(c) => c,
                Err(e) => {
                    let resp = IpcResponse::ok("probe_result", serde_json::json!({ "error": format!("Failed to build HTTP client: {}", e) }));
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                    return;
                }
            };

            match client.get(&url).send().await {
                Ok(response) => {
                    let status_code = response.status().as_u16();
                    let final_url = response.url().to_string();
                    let timing_ms = start.elapsed().as_millis() as u64;

                    // Collect headers
                    let mut headers = serde_json::Map::new();
                    for (name, value) in response.headers().iter() {
                        if let Ok(v) = value.to_str() {
                            headers.insert(name.to_string(), serde_json::Value::String(v.to_string()));
                        }
                    }

                    // Extract cookies from set-cookie headers
                    let mut cookies = serde_json::Map::new();
                    for value in response.headers().get_all(reqwest::header::SET_COOKIE).iter() {
                        if let Ok(v) = value.to_str() {
                            if let Some(eq_pos) = v.find('=') {
                                let name = v[..eq_pos].trim().to_string();
                                let rest = &v[eq_pos + 1..];
                                let val = rest.split(';').next().unwrap_or("").trim().to_string();
                                cookies.insert(name, serde_json::Value::String(val));
                            }
                        }
                    }

                    // Read body snippet (first 2KB)
                    let body_snippet = match response.text().await {
                        Ok(text) => {
                            if text.len() > 2048 {
                                text[..2048].to_string()
                            } else {
                                text
                            }
                        }
                        Err(_) => String::new(),
                    };

                    let resp = IpcResponse::ok("probe_result", serde_json::json!({
                        "status_code": status_code,
                        "headers": headers,
                        "cookies": cookies,
                        "body_snippet": body_snippet,
                        "final_url": final_url,
                        "timing_ms": timing_ms,
                    }));
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                }
                Err(e) => {
                    let resp = IpcResponse::ok("probe_result", serde_json::json!({ "error": format!("Request failed: {}", e) }));
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                }
            }
        });
    }
}

pub(super) fn site_inspect(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let url    = data.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let method = data.get("method").and_then(|v| v.as_str()).unwrap_or("GET").to_string();
            let proxy  = data.get("proxy").and_then(|v| v.as_str()).map(|s| s.to_string());
            let body   = data.get("body").and_then(|v| v.as_str()).map(|s| s.to_string());
            let browser = data.get("browser").and_then(|v| v.as_str()).unwrap_or("chrome").to_string();
            // Extra headers the user typed in the inspector
            let extra_headers: Vec<[String; 2]> = data.get("headers")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();

            if url.is_empty() {
                let resp = IpcResponse::ok("site_inspect_result", serde_json::json!({ "error": "No URL provided" }));
                eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                return;
            }

            // Attempt to use the AzureTLS sidecar for real TLS fingerprinting
            let mut s = state.lock().await;
            let sidecar_path = resolve_sidecar_path(&s.config.sidecar_path);
            let sidecar_tx = s.sidecar.get_or_start(&sidecar_path).await.ok();
            drop(s);

            if let Some(tx) = sidecar_tx {
                let session_id = format!("__inspector_{}", uuid::Uuid::new_v4());

                // Create session
                let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
                let _ = tx.send((ironbullet::sidecar::protocol::SidecarRequest {
                    id: uuid::Uuid::new_v4().to_string(),
                    action: "new_session".into(),
                    session: session_id.clone(),
                    browser: Some(browser.clone()),
                    proxy: proxy.clone(),
                    follow_redirects: Some(true),
                    max_redirects: Some(8),
                    ssl_verify: None,
                    ja3: None, http2fp: None, url: None, method: None,
                    headers: None, body: None, timeout: None,
                    custom_ciphers: None, return_request_headers: None,
                }, resp_tx)).await;
                let _ = resp_rx.await;

                // Make the request
                let (resp_tx2, resp_rx2) = tokio::sync::oneshot::channel();
                let hdrs: Vec<Vec<String>> = extra_headers.iter()
                    .map(|h| vec![h[0].clone(), h[1].clone()])
                    .collect();
                let _ = tx.send((ironbullet::sidecar::protocol::SidecarRequest {
                    id: uuid::Uuid::new_v4().to_string(),
                    action: "request".into(),
                    session: session_id.clone(),
                    method: Some(method.clone()),
                    url: Some(url.clone()),
                    headers: if hdrs.is_empty() { None } else { Some(hdrs) },
                    body: body.clone(),
                    timeout: Some(20_000),
                    proxy: proxy.clone(),
                    browser: Some(browser.clone()),
                    follow_redirects: Some(true),
                    max_redirects: Some(8),
                    ssl_verify: None,
                    ja3: None, http2fp: None,
                    custom_ciphers: None,
                    return_request_headers: Some(true), // capture what was actually sent
                }, resp_tx2)).await;

                let sidecar_resp = resp_rx2.await.ok();

                // Close session
                let (close_tx, _) = tokio::sync::oneshot::channel();
                let _ = tx.send((ironbullet::sidecar::protocol::SidecarRequest {
                    id: uuid::Uuid::new_v4().to_string(),
                    action: "close_session".into(),
                    session: session_id,
                    method: None, url: None, headers: None, body: None,
                    timeout: None, proxy: None, browser: None,
                    ja3: None, http2fp: None, follow_redirects: None,
                    max_redirects: None, ssl_verify: None,
                    custom_ciphers: None, return_request_headers: None,
                }, close_tx)).await;

                if let Some(sr) = sidecar_resp {
                    let resp = IpcResponse::ok("site_inspect_result", serde_json::json!({
                        "status":          sr.status,
                        "final_url":       sr.final_url,
                        "timing_ms":       sr.timing_ms,
                        "headers":         sr.headers.unwrap_or_default(),
                        "request_headers": sr.request_headers.unwrap_or_default(),
                        "cookies":         sr.cookies.unwrap_or_default(),
                        "body":            sr.body,
                        "error":           sr.error,
                        "via":             "azuretls",
                        "browser":         browser,
                    }));
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                    return;
                }
            }

            // Fallback: native reqwest (no TLS fingerprinting)
            let start = std::time::Instant::now();
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(20))
                .redirect(reqwest::redirect::Policy::limited(8))
                .danger_accept_invalid_certs(true)
                .cookie_store(true)
                .build();

            let client = match client {
                Ok(c) => c,
                Err(e) => {
                    let resp = IpcResponse::ok("site_inspect_result", serde_json::json!({ "error": format!("Client error: {}", e) }));
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                    return;
                }
            };

            let mut req = client.request(reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::GET), &url);
            for h in &extra_headers {
                req = req.header(&h[0], &h[1]);
            }
            if let Some(b) = body {
                req = req.body(b);
            }
            if let Some(p) = proxy {
                if let Ok(prx) = reqwest::Proxy::all(&p) {
                    // Can't override per-request proxy in reqwest easily, skip
                    let _ = prx;
                }
            }

            match req.send().await {
                Ok(response) => {
                    let status = response.status().as_u16() as i32;
                    let final_url = response.url().to_string();
                    let timing_ms = start.elapsed().as_millis() as i64;
                    let mut headers = std::collections::HashMap::new();
                    for (k, v) in response.headers() {
                        if let Ok(val) = v.to_str() {
                            headers.insert(k.to_string(), val.to_string());
                        }
                    }
                    let body = response.text().await.unwrap_or_default();
                    let resp = IpcResponse::ok("site_inspect_result", serde_json::json!({
                        "status":          status,
                        "final_url":       final_url,
                        "timing_ms":       timing_ms,
                        "headers":         headers,
                        "request_headers": serde_json::Value::Object(serde_json::Map::new()),
                        "cookies":         serde_json::Value::Object(serde_json::Map::new()),
                        "body":            body,
                        "error":           null,
                        "via":             "reqwest",
                    }));
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                }
                Err(e) => {
                    let resp = IpcResponse::ok("site_inspect_result", serde_json::json!({ "error": format!("Request failed: {}", e) }));
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                }
            }
        });
    }
}
