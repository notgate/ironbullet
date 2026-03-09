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
            let chrome_exe = {
                let cfg_path = s.config.chrome_executable_path.clone();
                if !cfg_path.is_empty() {
                    let p = std::path::PathBuf::from(&cfg_path);
                    if p.exists() { Some(p) } else { super::find_chrome_executable() }
                } else {
                    super::find_chrome_executable()
                }
            };
            drop(s); // Release lock before async execution

            let native_tx = ironbullet::sidecar::native::create_native_backend();

            let mut ctx = ExecutionContext::new(uuid::Uuid::new_v4().to_string());
            ctx.plugin_manager = Some(pm);
            ctx.chrome_executable_path = chrome_exe;

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

            // Use frontend pipeline snapshot if provided so proxy_settings is current.
            let pipeline_for_proxy = data.get("pipeline")
                .and_then(|v| serde_json::from_value::<ironbullet::pipeline::Pipeline>(v.clone()).ok())
                .unwrap_or_else(|| s.pipeline.clone());
            let ban_secs = pipeline_for_proxy.proxy_settings.ban_duration_secs as u64;

            // Load proxies:
            // 1. Explicit proxy_path from the job (flat file) — highest priority
            // 2. Active proxy group's sources from pipeline proxy_settings
            // 3. Global proxy_sources from pipeline proxy_settings
            // 4. Empty pool (proxyless)
            let proxy_path = data.get("proxy_path").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let proxy_pool = if !proxy_path.is_empty() {
                match ProxyPool::from_file(&proxy_path, ban_secs) {
                    Ok(pp) => pp,
                    Err(e) => {
                        let resp = IpcResponse::err("runner_error", format!("Failed to load proxies '{}': {}", proxy_path, e));
                        eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                        return;
                    }
                }
            } else {
                // Try to load from pipeline proxy groups / sources
                let ps = &pipeline_for_proxy.proxy_settings;
                // Gather all source paths from the active group (or all groups if no active)
                let sources: Vec<_> = if !ps.active_group.is_empty() {
                    ps.proxy_groups.iter()
                        .filter(|g| g.name == ps.active_group)
                        .flat_map(|g| g.sources.iter())
                        .collect()
                } else if !ps.proxy_groups.is_empty() {
                    ps.proxy_groups.iter().flat_map(|g| g.sources.iter()).collect()
                } else {
                    ps.proxy_sources.iter().collect()
                };

                if !sources.is_empty() {
                    use ironbullet::pipeline::ProxySourceType;
                    let mut pool = ProxyPool::empty_with_ban(ban_secs);
                    for src in sources {
                        if matches!(src.source_type, ProxySourceType::File) {
                            let default_type = src.default_proxy_type.as_deref();
                            if let Err(e) = pool.load_from_file(&src.value, default_type) {
                                eprintln!("[runner] warning: failed to load proxy source '{}': {}", src.value, e);
                            }
                        }
                        // URL sources could be fetched here in future — skip for now
                    }
                    pool
                } else {
                    ProxyPool::empty()
                }
            };

            let (hits_tx, mut hits_rx) = tokio::sync::mpsc::channel::<HitResult>(1024);

            // Use frontend pipeline snapshot if provided (keeps runner_settings, proxy_settings, etc. in sync)
            let pipeline = data.get("pipeline")
                .and_then(|v| serde_json::from_value::<ironbullet::pipeline::Pipeline>(v.clone()).ok())
                .unwrap_or_else(|| s.pipeline.clone());
            let proxy_mode = pipeline.proxy_settings.proxy_mode.clone();
            let pm = s.plugin_manager.clone();
            // Resolve chrome executable: prefer user-configured path → auto-discovery
            let chrome_exe = {
                let cfg_path = s.config.chrome_executable_path.clone();
                if !cfg_path.is_empty() {
                    let p = std::path::PathBuf::from(&cfg_path);
                    if p.exists() { Some(p) } else { super::find_chrome_executable() }
                } else {
                    super::find_chrome_executable()
                }
            };
            let runner = Arc::new(RunnerOrchestrator::new(
                pipeline,
                proxy_mode,
                data_pool,
                proxy_pool,
                sidecar_tx,
                threads,
                hits_tx,
                Some(pm),
                chrome_exe,
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

// ── Browser capture (Inspector panel) ──────────────────────────────────────────

pub(super) fn inspect_browser_open(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    use chromiumoxide::browser::Browser;
    use chromiumoxide::cdp::browser_protocol::network::{
        EnableParams, EventRequestWillBeSent, EventResponseReceived,
        EventLoadingFinished, GetRequestPostDataParams, GetResponseBodyParams,
    };
    use futures::StreamExt;

    // Read chrome_exe path synchronously with try_lock — avoids an await inside
    // the async task that can deadlock if another handler holds the mutex.
    // Falls back to None if the lock is contended; find_chrome_executable() below handles that.
    let chrome_exe_from_config: Option<std::path::PathBuf> = state.try_lock()
        .map(|s| {
            let p = s.config.chrome_executable_path.clone();
            if !p.is_empty() { let pb = std::path::PathBuf::from(&p); if pb.exists() { Some(pb) } else { None } } else { None }
        })
        .ok()
        .flatten();

    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        let (js_tx, mut js_rx) = tokio::sync::mpsc::channel::<String>(1024);
        handle.spawn(async move { while let Some(js) = js_rx.recv().await { eval_js(js); } });

        let js = js_tx.clone();

        // Generate the profile dir UUID here so it can be stored in AppState
        // (for cleanup on next launch) AND passed into the async task.
        // UUID-per-launch prevents Chrome profile-dir locking when a prior
        // Chrome process was orphaned by a timed-out launch — a URL-derived
        // fixed name would be locked by the zombie, causing infinite failures.
        let tmp_profile = std::env::temp_dir()
            .join(format!("ib-chrome-{}", uuid::Uuid::new_v4()));
        let tmp_profile_task = tmp_profile.clone();

        let state_for_abort = state.clone();
        let capture_task = handle.spawn(async move {
            let tmp_profile = tmp_profile_task;
            let chrome_exe_cfg = chrome_exe_from_config;

            fn emit_sync(tx: &tokio::sync::mpsc::Sender<String>, payload: serde_json::Value) {
                let resp = IpcResponse::ok("inspector_browser_event", payload);
                let _ = tx.try_send(format!("window.__ipc_callback({})",
                    serde_json::to_string(&resp).unwrap_or_default()));
            }

            // Abort any prior capture session and clean up its stale Chrome
            // profile before launching. The old profile dir is locked by the
            // previous Chrome process if it didn't exit cleanly (e.g. launch
            // timeout). Removing it prevents the next launch from failing with
            // a "profile in use" error.
            let stale_profile = {
                let mut s = state.lock().await;
                if let Some(h) = s.browser_capture_abort.take() { h.abort(); }
                s.browser_capture_profile.take()
            };

            if let Some(old_dir) = stale_profile {
                tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                // Use a timeout — on Windows, remove_dir_all can hang if Chrome
                // left file locks behind. Give it 3s max then skip.
                let _ = tokio::time::timeout(
                    std::time::Duration::from_secs(3),
                    tokio::fs::remove_dir_all(&old_dir),
                ).await;
            }

            let raw_url = data.get("url").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
            if raw_url.is_empty() { return; }
            let url = if raw_url.starts_with("http://") || raw_url.starts_with("https://") {
                raw_url
            } else {
                format!("https://{}", raw_url)
            };

            // chrome_exe was read synchronously at handler entry (no mutex await needed)
            let chrome_exe = chrome_exe_cfg.or_else(|| super::find_chrome_executable());

            if chrome_exe.is_none() {
                emit_sync(&js, serde_json::json!({
                    "type": "error",
                    "message": "Google Chrome or Chromium is not installed or not found.\n\nPlease set the Chrome executable path in Settings → Paths, or install Chrome from https://www.google.com/chrome/"
                }));
                return;
            }

            // ── Chrome launch via manual spawn + CDP HTTP poll ──────────────────
            // Browser::launch reads Chrome's stderr for the WS URL, which is
            // unreliable on Windows (Chrome may not flush stderr, or may write
            // to stdout instead). We instead:
            //   1. Pick a free port
            //   2. Spawn Chrome via std::process::Command (synchronous, reliable)
            //   3. Poll http://localhost:{port}/json/version until Chrome responds
            //   4. Connect via Browser::connect(ws_url) — pure async, no blocking
            //
            // This mirrors how Playwright launches Chrome and is far more reliable
            // across platforms.

            // Pick a free port by binding to :0 and reading the assigned port.
            let cdp_port = {
                match std::net::TcpListener::bind("127.0.0.1:0") {
                    Ok(l) => l.local_addr().map(|a| a.port()).unwrap_or(9222),
                    Err(_) => 9222,
                }
            };

            let chrome_exe = chrome_exe.unwrap(); // already checked above
            let profile_dir = tmp_profile.clone();
            let mut chrome_args = vec![
                format!("--remote-debugging-port={}", cdp_port),
                format!("--user-data-dir={}", profile_dir.display()),
                "--no-first-run".to_string(),
                "--no-default-browser-check".to_string(),
                "--disable-sync".to_string(),
                "--disable-translate".to_string(),
                "--disable-extensions".to_string(),
                "--disable-component-extensions-with-background-pages".to_string(),
                "--disable-background-networking".to_string(),
                "--disable-backgrounding-occluded-windows".to_string(),
                "--disable-device-discovery-notifications".to_string(),
                "--disable-client-side-phishing-detection".to_string(),
                "--no-sandbox".to_string(),
            ];
            // Navigate to the target URL directly on launch
            chrome_args.push(url.clone());

            let mut cmd = std::process::Command::new(&chrome_exe);
            cmd.args(&chrome_args);

            // Redirect stdout/stderr to null — prevents Chrome from blocking on
            // pipe buffers filling up (especially on Windows)
            cmd.stdout(std::process::Stdio::null());
            cmd.stderr(std::process::Stdio::null());
            cmd.stdin(std::process::Stdio::null());

            eprintln!("[inspector] spawning Chrome: {:?} with args: {:?}", chrome_exe, chrome_args);

            // Keep chrome_child alive for the duration of the capture session.
            // Using a non-underscore name ensures Rust doesn't drop it immediately.
            let mut chrome_child = match cmd.spawn() {
                Ok(child) => child,
                Err(e) => {
                    emit_sync(&js, serde_json::json!({
                        "type": "error",
                        "message": format!("Failed to spawn Chrome process: {}\n\nChrome path: {:?}\n\nMake sure Chrome is installed and the path is set correctly in Settings → Paths.", e, chrome_exe)
                    }));
                    return;
                }
            };
            emit_sync(&js, serde_json::json!({
                "type": "status",
                "message": format!("Chrome started, connecting on port {}...", cdp_port)
            }));

            // Poll /json/version until Chrome is ready (up to 15s)
            let version_url = format!("http://127.0.0.1:{}/json/version", cdp_port);
            let http_client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(2))
                .build()
                .unwrap_or_default();

            let ws_url: String = {
                let mut found = None;
                let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(15);
                loop {
                    if tokio::time::Instant::now() > deadline {
                        emit_sync(&js, serde_json::json!({
                            "type": "error",
                            "message": "Chrome started but did not open its debug port within 15 seconds."
                        }));
                        return;
                    }
                    match http_client.get(&version_url).send().await {
                        Ok(resp) => {
                            if let Ok(json) = resp.json::<serde_json::Value>().await {
                                if let Some(ws) = json.get("webSocketDebuggerUrl").and_then(|v| v.as_str()) {
                                    found = Some(ws.to_string());
                                    break;
                                }
                            }
                        }
                        Err(_) => { /* Chrome not ready yet, keep polling */ }
                    }

                    // Every 3 seconds emit a diagnostic so the user sees progress
                    let elapsed = tokio::time::Instant::now().duration_since(deadline - std::time::Duration::from_secs(15));
                    if elapsed.as_millis() % 3000 < 250 {
                        emit_sync(&js, serde_json::json!({
                            "type": "diagnostic",
                            "message": format!("Waiting for Chrome CDP on port {} ({:.0}s elapsed)...", cdp_port, elapsed.as_secs_f32())
                        }));
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                }
                match found {
                    Some(ws) => ws,
                    None => {
                        emit_sync(&js, serde_json::json!({ "type": "error", "message": "Chrome debug port ready but no WS URL returned." }));
                        return;
                    }
                }
            };

            // Connect to already-running Chrome via WS URL — pure async, no blocking
            let connect_result = tokio::time::timeout(
                std::time::Duration::from_secs(10),
                Browser::connect(ws_url),
            ).await;

            let (browser, mut handler) = match connect_result {
                Ok(Ok(pair)) => pair,
                Ok(Err(e)) => {
                    emit_sync(&js, serde_json::json!({
                        "type": "error",
                        "message": format!("CDP connect failed: {}", e)
                    }));
                    return;
                }
                Err(_) => {
                    emit_sync(&js, serde_json::json!({
                        "type": "error",
                        "message": "CDP websocket connect timed out (10s)."
                    }));
                    return;
                }
            };

            // Chrome process is up — clear the loading state in the frontend
            // immediately so the 25s safety timer never fires. CDP network capture
            // setup continues below; any failure after this emits an "error" event.
            emit_sync(&js, serde_json::json!({ "type": "opened", "url": url }));

            // Spawn CDP handler; send a signal when Chrome's connection closes so
            // the event loop below can break immediately instead of hanging.
            let (died_tx, mut died_rx) = tokio::sync::oneshot::channel::<()>();
            tokio::spawn(async move {
                while handler.next().await.is_some() {}
                let _ = died_tx.send(());
            });

            // Chrome was launched with the URL as a CLI arg, so it already has an
            // open tab. Grab that existing page instead of creating a new one —
            // new_page("about:blank") on Windows can block or race with Chrome's
            // initialization of the first tab.
            let page = {
                // Give Chrome a moment to register the initial tab with CDP
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                let pages_result = tokio::time::timeout(
                    std::time::Duration::from_secs(10),
                    browser.pages(),
                ).await;
                match pages_result {
                    Ok(Ok(mut pages)) if !pages.is_empty() => pages.remove(0),
                    Ok(Ok(_)) | Ok(Err(_)) => {
                        // No existing pages — fall back to opening a new one
                        match tokio::time::timeout(
                            std::time::Duration::from_secs(15),
                            browser.new_page("about:blank"),
                        ).await {
                            Ok(Ok(p)) => p,
                            Ok(Err(e)) => {
                                emit_sync(&js, serde_json::json!({ "type": "error", "message": format!("Page open failed: {}", e) }));
                                return;
                            }
                            Err(_) => {
                                emit_sync(&js, serde_json::json!({ "type": "error", "message": "Chrome page open timed out (15s)" }));
                                return;
                            }
                        }
                    }
                    Err(_) => {
                        emit_sync(&js, serde_json::json!({ "type": "error", "message": "Timed out getting Chrome pages (10s)" }));
                        return;
                    }
                }
            };

            // Enable CDP Network domain with generous buffers.
            match tokio::time::timeout(std::time::Duration::from_secs(10), page.execute(EnableParams {
                max_total_buffer_size:    Some(100 * 1024 * 1024),
                max_resource_buffer_size: Some(5   * 1024 * 1024),
                max_post_data_size:       Some(5   * 1024 * 1024),
                ..Default::default()
            })).await {
                Ok(Ok(_)) => {}
                _ => { emit_sync(&js, serde_json::json!({ "type": "error", "message": "CDP Network.enable failed or timed out" })); return; }
            }

            // Register all listeners BEFORE navigating so no requests are missed.
            let mut ev_req = match tokio::time::timeout(
                std::time::Duration::from_secs(10),
                page.event_listener::<EventRequestWillBeSent>(),
            ).await {
                Ok(Ok(e)) => e,
                Ok(Err(e)) => { emit_sync(&js, serde_json::json!({ "type": "error", "message": format!("{e}") })); return; }
                Err(_)    => { emit_sync(&js, serde_json::json!({ "type": "error", "message": "Listener setup timed out" })); return; }
            };
            let mut ev_resp = match tokio::time::timeout(
                std::time::Duration::from_secs(10),
                page.event_listener::<EventResponseReceived>(),
            ).await { Ok(Ok(e)) => e, _ => return };
            let mut ev_done = match tokio::time::timeout(
                std::time::Duration::from_secs(10),
                page.event_listener::<EventLoadingFinished>(),
            ).await { Ok(Ok(e)) => e, _ => return };

            // Chrome was already launched with the URL as a CLI arg — no goto needed.
            // The page is already navigating; listeners are attached and will capture requests.

            let page_req  = page.clone();
            let page_body = page.clone();

            loop {
                tokio::select! {
                    Some(event) = ev_req.next() => {
                        let req = &event.request;
                        let headers: std::collections::HashMap<String, String> =
                            req.headers.inner().as_object()
                                .map(|obj| obj.iter()
                                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                                    .collect())
                                .unwrap_or_default();

                        let post_data: Option<String> = if req.has_post_data.unwrap_or(false) {
                            page_req.execute(GetRequestPostDataParams { request_id: event.request_id.clone() })
                                .await.ok().map(|r| r.result.post_data)
                        } else { None };

                        let resource_type = event.r#type.as_ref()
                            .map(|t| format!("{t:?}"))
                            .unwrap_or_else(|| "Other".to_string());

                        emit_sync(&js, serde_json::json!({
                            "type":          "request",
                            "id":            event.request_id.inner().clone(),
                            "url":           req.url,
                            "method":        req.method,
                            "headers":       headers,
                            "post_data":     post_data,
                            "resource_type": resource_type,
                        }));
                    }
                    Some(event) = ev_resp.next() => {
                        let resp = &event.response;
                        let headers: std::collections::HashMap<String, String> =
                            resp.headers.inner().as_object()
                                .map(|obj| obj.iter()
                                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                                    .collect())
                                .unwrap_or_default();

                        emit_sync(&js, serde_json::json!({
                            "type":        "response_meta",
                            "id":          event.request_id.inner().clone(),
                            "status":      resp.status,
                            "status_text": resp.status_text,
                            "mime_type":   resp.mime_type,
                            "headers":     headers,
                        }));
                    }
                    Some(event) = ev_done.next() => {
                        // Fetch response body in a separate task so we don't block
                        // the event loop while waiting for CDP to return the body.
                        // This prevents missing requests on pages with many resources.
                        let within_limit = event.encoded_data_length < 0.0
                            || event.encoded_data_length < 4_194_304.0;
                        if within_limit {
                            let page_body2 = page_body.clone();
                            let js2 = js.clone();
                            let req_id = event.request_id.clone();
                            tokio::spawn(async move {
                                if let Ok(r) = page_body2
                                    .execute(GetResponseBodyParams { request_id: req_id.clone() })
                                    .await
                                {
                                    let body_text = if r.result.base64_encoded {
                                        #[allow(unused_imports)]
                                        use std::io::Read;
                                        match base64::Engine::decode(
                                            &base64::engine::general_purpose::STANDARD,
                                            r.result.body.trim(),
                                        ) {
                                            Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                                            Err(_) => r.result.body,
                                        }
                                    } else {
                                        r.result.body
                                    };
                                    if !body_text.is_empty() {
                                        emit_sync(&js2, serde_json::json!({
                                            "type": "response_body",
                                            "id":   req_id.inner().clone(),
                                            "body": body_text,
                                        }));
                                    }
                                }
                            });
                        }
                    }
                    // Chrome process closed — CDP handler task exited and fired
                    // this signal. Break immediately so we don't hang forever
                    // waiting for stream events that will never arrive.
                    _ = &mut died_rx => break,
                    else => break,
                }
            }

            emit_sync(&js, serde_json::json!({ "type": "closed" }));

            // Clean up Chrome process when capture session ends
            let _ = chrome_child.kill();
        });

        let abort = capture_task.abort_handle();
        handle.spawn(async move {
            let mut s = state_for_abort.lock().await;
            s.browser_capture_abort = Some(abort);
            // Store the profile path so the NEXT launch can clean it up.
            s.browser_capture_profile = Some(tmp_profile);
        });
    }
}

pub(super) fn inspect_browser_close(
    state: Arc<Mutex<AppState>>,
    _eval_js: impl Fn(String) + Send + 'static,
) {
    // The frontend already updates browserOpen/browserLoading synchronously in
    // closeBrowser() before this IPC even arrives. Emitting a "closed" event here
    // would hit the *next* session's listener (registered moments later) and
    // incorrectly reset its loading state. Just abort the task and clean up.
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let profile = {
                let mut s = state.lock().await;
                if let Some(h) = s.browser_capture_abort.take() { h.abort(); }
                s.browser_capture_profile.take()
            };
            // Give Chrome a moment to release file locks, then remove its profile dir.
            if let Some(dir) = profile {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                let _ = tokio::fs::remove_dir_all(&dir).await;
            }
        });
    }
}

// ── Local HTTP Proxy Capture (Inspector panel alternative) ────────────────────
//
// Binds a TCP listener on a user-specified port (default 8877).
// The user configures their browser proxy to 127.0.0.1:<port>.
// Supports:
//   - Plain HTTP: read the full request, forward to the real server, capture
//     both request and response, emit inspector_proxy_event to the frontend.
//   - HTTPS (CONNECT tunnel): respond 200, then relay raw bytes bidirectionally
//     so the browser's TLS works end-to-end. We emit a minimal capture event
//     showing the CONNECT target host + URL; we cannot decrypt HTTPS without
//     a MitM CA (not implemented).
//
// Each accepted connection runs in its own tokio task. The listener task holds
// an AbortHandle stored in AppState so the frontend can stop the proxy cleanly.

pub(super) fn inspect_proxy_start(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    #[allow(unused_imports)]
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    #[allow(unused_imports)]
    use tokio::net::{TcpListener, TcpStream};

    let rt = tokio::runtime::Handle::try_current();
    let Ok(handle) = rt else { return };

    let port = data.get("port")
        .and_then(|v| v.as_u64())
        .unwrap_or(8877) as u16;

    let (js_tx, mut js_rx) = tokio::sync::mpsc::channel::<String>(512);
    handle.spawn(async move { while let Some(js) = js_rx.recv().await { eval_js(js); } });

    let js = js_tx.clone();

    fn emit(tx: &tokio::sync::mpsc::Sender<String>, payload: serde_json::Value) {
        let resp = IpcResponse::ok("inspector_proxy_event", payload);
        let _ = tx.try_send(format!("window.__ipc_callback({})",
            serde_json::to_string(&resp).unwrap_or_default()));
    }

    let state2 = state.clone();
    let js2 = js.clone();

    let task = handle.spawn(async move {
        // Abort any previous proxy
        {
            let mut s = state2.lock().await;
            if let Some(h) = s.inspect_proxy_abort.take() { h.abort(); }
            s.inspect_proxy_port = None;
        }

        let listener = match TcpListener::bind(("127.0.0.1", port)).await {
            Ok(l) => l,
            Err(e) => {
                emit(&js2, serde_json::json!({
                    "type": "error",
                    "message": format!("Failed to bind proxy on port {}: {}", port, e)
                }));
                return;
            }
        };

        {
            let mut s = state2.lock().await;
            s.inspect_proxy_port = Some(port);
        }

        emit(&js2, serde_json::json!({
            "type": "ready",
            "port": port,
            "message": format!("Proxy listening on 127.0.0.1:{}", port)
        }));

        loop {
            let (stream, _peer) = match listener.accept().await {
                Ok(v) => v,
                Err(_) => break,
            };
            let js_conn = js2.clone();
            tokio::spawn(async move {
                let _ = handle_proxy_connection(stream, js_conn).await;
            });
        }
    });

    let abort = task.abort_handle();
    let state3 = state.clone();
    tokio::runtime::Handle::try_current().ok().map(|h| h.spawn(async move {
        let mut s = state3.lock().await;
        s.inspect_proxy_abort = Some(abort);
    }));
}

async fn handle_proxy_connection(
    mut stream: tokio::net::TcpStream,
    js: tokio::sync::mpsc::Sender<String>,
) -> std::io::Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    fn emit_conn(tx: &tokio::sync::mpsc::Sender<String>, payload: serde_json::Value) {
        let resp = IpcResponse::ok("inspector_proxy_event", payload);
        let _ = tx.try_send(format!("window.__ipc_callback({})",
            serde_json::to_string(&resp).unwrap_or_default()));
    }

    // Read request header (stop at \r\n\r\n, max 64KB)
    let mut buf = Vec::with_capacity(8192);
    loop {
        let mut byte = [0u8; 1];
        let n = stream.read(&mut byte).await?;
        if n == 0 { return Ok(()); }
        buf.push(byte[0]);
        if buf.len() >= 4 && &buf[buf.len()-4..] == b"\r\n\r\n" { break; }
        if buf.len() > 65536 { return Ok(()); }
    }

    let header_str = String::from_utf8_lossy(&buf);
    let first_line = header_str.lines().next().unwrap_or("").to_string();
    let parts: Vec<&str> = first_line.splitn(3, ' ').collect();
    if parts.len() < 2 { return Ok(()); }

    let method = parts[0];
    let target = parts[1];

    if method == "CONNECT" {
        // HTTPS tunnel — respond 200, relay bytes, emit minimal event
        stream.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await?;
        emit_conn(&js, serde_json::json!({
            "type": "request",
            "id": uuid::Uuid::new_v4().to_string(),
            "method": "CONNECT",
            "url": format!("https://{}", target),
            "host": target,
            "resource_type": "tunnel",
            "headers": {},
            "note": "HTTPS tunnel (encrypted — cannot inspect body)"
        }));
        // Relay raw bytes bidirectionally
        let host = target.to_string();
        if let Ok(mut upstream) = tokio::net::TcpStream::connect(&host).await {
            let (mut cr, mut cw) = stream.split();
            let (mut ur, mut uw) = upstream.split();
            tokio::select! {
                _ = tokio::io::copy(&mut cr, &mut uw) => {}
                _ = tokio::io::copy(&mut ur, &mut cw) => {}
            }
        }
        return Ok(());
    }

    // Plain HTTP — parse headers, read body if present, forward, capture response
    let mut headers: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut content_length: usize = 0;
    for line in header_str.lines().skip(1) {
        if line.is_empty() { break; }
        if let Some(colon) = line.find(':') {
            let k = line[..colon].trim().to_lowercase();
            let v = line[colon+1..].trim().to_string();
            if k == "content-length" {
                content_length = v.parse().unwrap_or(0);
            }
            headers.insert(k, v);
        }
    }

    // Read body
    let mut body_bytes = vec![0u8; content_length.min(1024 * 1024)];
    if content_length > 0 {
        let _ = stream.read_exact(&mut body_bytes).await;
    }
    let body_str = String::from_utf8_lossy(&body_bytes).to_string();

    // Determine upstream host + port
    let host_hdr = headers.get("host").cloned().unwrap_or_default();
    let (upstream_host, upstream_port) = if host_hdr.contains(':') {
        let mut it = host_hdr.rsplitn(2, ':');
        let p: u16 = it.next().unwrap_or("80").parse().unwrap_or(80);
        (it.next().unwrap_or(&host_hdr).to_string(), p)
    } else {
        (host_hdr.clone(), 80u16)
    };

    let req_id = uuid::Uuid::new_v4().to_string();
    emit_conn(&js, serde_json::json!({
        "type": "request",
        "id": req_id,
        "method": method,
        "url": if target.starts_with("http") { target.to_string() } else { format!("http://{}{}", host_hdr, target) },
        "host": host_hdr,
        "resource_type": "fetch",
        "headers": headers,
        "post_data": if body_str.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(body_str.clone()) }
    }));

    // Forward to upstream
    let upstream_addr = format!("{}:{}", upstream_host, upstream_port);
    let mut upstream = match tokio::net::TcpStream::connect(&upstream_addr).await {
        Ok(u) => u,
        Err(e) => {
            let err_resp = format!("HTTP/1.1 502 Bad Gateway\r\nContent-Length: {}\r\n\r\n{}", e.to_string().len(), e);
            let _ = stream.write_all(err_resp.as_bytes()).await;
            return Ok(());
        }
    };

    // Forward request (rebuild it)
    upstream.write_all(&buf).await?;
    if !body_bytes.is_empty() {
        upstream.write_all(&body_bytes).await?;
    }

    // Read response header from upstream
    let mut resp_buf = Vec::with_capacity(8192);
    loop {
        let mut byte = [0u8; 1];
        let n = upstream.read(&mut byte).await?;
        if n == 0 { break; }
        resp_buf.push(byte[0]);
        if resp_buf.len() >= 4 && &resp_buf[resp_buf.len()-4..] == b"\r\n\r\n" { break; }
        if resp_buf.len() > 65536 { break; }
    }

    let resp_header_str = String::from_utf8_lossy(&resp_buf);
    let resp_first = resp_header_str.lines().next().unwrap_or("");
    let resp_parts: Vec<&str> = resp_first.splitn(3, ' ').collect();
    let resp_status: u16 = resp_parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let resp_status_text = resp_parts.get(2).unwrap_or(&"").to_string();

    let mut resp_headers: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut resp_cl: usize = 0;
    let mut chunked = false;
    for line in resp_header_str.lines().skip(1) {
        if line.is_empty() { break; }
        if let Some(colon) = line.find(':') {
            let k = line[..colon].trim().to_lowercase();
            let v = line[colon+1..].trim().to_string();
            if k == "content-length" { resp_cl = v.parse().unwrap_or(0); }
            if k == "transfer-encoding" && v.contains("chunked") { chunked = true; }
            resp_headers.insert(k, v);
        }
    }

    // Read response body (cap at 2MB for capture)
    let mut resp_body_bytes = Vec::new();
    if resp_cl > 0 {
        resp_body_bytes.resize(resp_cl.min(2 * 1024 * 1024), 0);
        let _ = upstream.read_exact(&mut resp_body_bytes).await;
    } else if chunked {
        // Simple chunked read — read until 0-size chunk
        let mut chunk_buf = vec![0u8; 65536];
        loop {
            let n = upstream.read(&mut chunk_buf).await.unwrap_or(0);
            if n == 0 { break; }
            resp_body_bytes.extend_from_slice(&chunk_buf[..n]);
            if resp_body_bytes.len() > 2 * 1024 * 1024 { break; }
        }
    }

    // Forward response back to client
    stream.write_all(&resp_buf).await?;
    stream.write_all(&resp_body_bytes).await?;

    let resp_body_str = String::from_utf8_lossy(&resp_body_bytes);
    let mime = resp_headers.get("content-type").cloned().unwrap_or_default();

    emit_conn(&js, serde_json::json!({
        "type": "response",
        "id": req_id,
        "resp_status": resp_status,
        "resp_status_text": resp_status_text,
        "resp_mime": mime,
        "resp_headers": resp_headers,
        "resp_body": resp_body_str.chars().take(65536).collect::<String>()
    }));

    Ok(())
}

pub(super) fn inspect_proxy_stop(
    state: Arc<Mutex<AppState>>,
    _eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let mut s = state.lock().await;
            if let Some(h) = s.inspect_proxy_abort.take() { h.abort(); }
            s.inspect_proxy_port = None;
        });
    }
}
