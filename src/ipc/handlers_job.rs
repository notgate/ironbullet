use std::sync::Arc;
use tokio::sync::Mutex;

use ironbullet::export::format::RfxConfig;
use ironbullet::runner::job::{Job, JobType};

use super::{resolve_sidecar_path, AppState, IpcResponse};

pub(super) fn create_job(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let mut s = state.lock().await;

            // Try full deserialization first; fall back to default + manual patching.
            // NOTE: the frontend only sends partial data, so deserialization almost always
            // fails (missing required `proxy_source` etc.).  We therefore always patch the
            // fields we care about explicitly below.
            let mut job = if let Ok(j) = serde_json::from_value::<Job>(data.clone()) {
                j
            } else {
                Job::default()
            };

            // ── job_type (CRITICAL — was never set before this fix) ───────────
            if let Some(jt) = data.get("job_type").and_then(|v| v.as_str()) {
                job.job_type = match jt {
                    "ProxyCheck" => JobType::ProxyCheck,
                    _ => JobType::Config,
                };
            }

            // ── basic fields ──────────────────────────────────────────────────
            if let Some(name) = data.get("name").and_then(|v| v.as_str()) {
                job.name = name.to_string();
            }
            if let Some(tc) = data.get("thread_count").and_then(|v| v.as_u64()) {
                job.thread_count = tc as usize;
            }

            // ── data source ───────────────────────────────────────────────────
            if let Some(ds) = data.get("data_source") {
                if let Ok(new_ds) = serde_json::from_value::<ironbullet::runner::job::DataSource>(ds.clone()) {
                    job.data_source = new_ds;
                }
            }

            // ── pipeline snapshot from frontend (keeps proxy_settings etc. in sync) ──
            if let Some(pl) = data.get("pipeline") {
                match serde_json::from_value::<ironbullet::pipeline::Pipeline>(pl.clone()) {
                    Ok(pipeline) => {
                        eprintln!("[create_job] Pipeline OK: {} blocks, proxy_mode={:?}, proxy_sources={}",
                            pipeline.blocks.len(),
                            pipeline.proxy_settings.proxy_mode,
                            pipeline.proxy_settings.proxy_sources.len());
                        job.pipeline = pipeline;
                    }
                    Err(e) => {
                        eprintln!("[create_job] Pipeline deser FAILED: {}. Falling back to s.pipeline ({} blocks).",
                            e, s.pipeline.blocks.len());
                        job.pipeline = s.pipeline.clone();
                    }
                }
            }

            // ── config path (loads pipeline from .rfx file if provided) ───────
            if let Some(path) = data.get("config_path").and_then(|v| v.as_str()) {
                job.config_path = Some(path.to_string());
                if let Ok(rfx) = RfxConfig::load_from_file(path) {
                    job.pipeline = rfx.pipeline;
                }
            }

            // ── proxy check fields ────────────────────────────────────────────
            if let Some(url) = data.get("proxy_check_url").and_then(|v| v.as_str()) {
                if !url.is_empty() { job.proxy_check_url = url.to_string(); }
            }
            if let Some(list) = data.get("proxy_check_list").and_then(|v| v.as_str()) {
                if !list.is_empty() { job.proxy_check_list = list.to_string(); }
            }

            // Use currently loaded pipeline for Config jobs unless a snapshot was provided
            if job.job_type == JobType::Config && data.get("pipeline").is_none() {
                job.pipeline = s.pipeline.clone();
            }

            let id = s.job_manager.add_job(job);
            let resp = IpcResponse::ok("job_created", serde_json::json!({ "id": id.to_string() }));
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
        });
    }
}

pub(super) fn remove_job(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let mut s = state.lock().await;
            if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                if let Ok(uuid) = uuid::Uuid::parse_str(id) {
                    s.job_manager.remove_job(uuid);
                }
            }
            let jobs = s.job_manager.list_jobs();
            let resp = IpcResponse::ok("jobs_list", serde_json::to_value(jobs).unwrap_or_default());
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
        });
    }
}

pub(super) fn list_jobs(
    state: Arc<Mutex<AppState>>,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let s = state.lock().await;
            let jobs = s.job_manager.list_jobs();
            let resp = IpcResponse::ok("jobs_list", serde_json::to_value(jobs).unwrap_or_default());
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
        });
    }
}

pub(super) fn start_job(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        let inner_handle = handle.clone();
        handle.spawn(async move {
            let mut s = state.lock().await;
            let id_str = data.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let uuid = match uuid::Uuid::parse_str(id_str) {
                Ok(u) => u,
                Err(_) => {
                    let resp = IpcResponse::err("job_stats_update", "Invalid job ID".into());
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                    return;
                }
            };

            // Proxy check jobs bypass the sidecar and runner entirely
            let is_proxy_check = s.job_manager.get_job_mut(uuid)
                .map(|j| j.job_type == ironbullet::runner::job::JobType::ProxyCheck)
                .unwrap_or(false);

            if is_proxy_check {
                let pc_rx = s.job_manager.start_proxy_check_job(uuid, inner_handle.clone());

                // Immediate jobs_list so UI transitions Queued → Running right away
                {
                    let jobs = s.job_manager.list_jobs();
                    let resp = IpcResponse::ok("jobs_list", serde_json::to_value(jobs).unwrap_or_default());
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                }
                drop(s);

                if let Some(mut hits_rx) = pc_rx {
                    let state2 = state.clone();
                    let state3 = state.clone();
                    let state4 = state.clone();
                    let (js_tx, mut js_rx) = tokio::sync::mpsc::channel::<String>(256);
                    inner_handle.spawn(async move { while let Some(js) = js_rx.recv().await { eval_js(js); } });

                    // ── Hit receiver: runner_hit events + store hit ──────────────
                    let js_tx2 = js_tx.clone();
                    inner_handle.spawn(async move {
                        while let Some(hit) = hits_rx.recv().await {
                            let mut s = state2.lock().await;
                            s.job_manager.add_hit(uuid, hit.clone());
                            s.job_manager.update_job_stats(uuid);
                            // Push runner_hit for live append on frontend
                            let hit_resp = IpcResponse::ok("runner_hit", serde_json::to_value(&hit).unwrap_or_default());
                            let _ = js_tx2.send(format!("window.__ipc_callback({})",
                                serde_json::to_string(&hit_resp).unwrap_or_default())).await;
                            // Also update jobs_list for stat columns
                            let jobs = s.job_manager.list_jobs();
                            let resp = IpcResponse::ok("jobs_list", serde_json::to_value(jobs).unwrap_or_default());
                            let _ = js_tx2.send(format!("window.__ipc_callback({})",
                                serde_json::to_string(&resp).unwrap_or_default())).await;
                        }

                        // All tasks done — mark Completed
                        let mut s = state3.lock().await;
                        if let Some(job) = s.job_manager.get_job_mut(uuid) {
                            job.state = ironbullet::runner::job::JobState::Completed;
                            job.completed = Some(chrono::Utc::now());
                        }
                        s.job_manager.update_job_stats(uuid);
                        let jobs = s.job_manager.list_jobs();
                        let resp = IpcResponse::ok("jobs_list", serde_json::to_value(jobs).unwrap_or_default());
                        let _ = js_tx2.send(format!("window.__ipc_callback({})",
                            serde_json::to_string(&resp).unwrap_or_default())).await;
                    });

                    // ── Periodic stats push (500 ms) so dead-proxy progress shows too ──
                    let js_tx3 = js_tx.clone();
                    inner_handle.spawn(async move {
                        loop {
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                            let mut s = state4.lock().await;
                            let still_running = s.job_manager.get_job_mut(uuid)
                                .map(|j| j.state == ironbullet::runner::job::JobState::Running)
                                .unwrap_or(false);
                            if !still_running { break; }
                            s.job_manager.update_job_stats(uuid);
                            let jobs = s.job_manager.list_jobs();
                            let resp = IpcResponse::ok("jobs_list", serde_json::to_value(jobs).unwrap_or_default());
                            let _ = js_tx3.send(format!("window.__ipc_callback({})",
                                serde_json::to_string(&resp).unwrap_or_default())).await;
                        }
                    });
                }
                return;
            }

            // Get or start sidecar (reuses existing process if already running)
            let sidecar_path = resolve_sidecar_path(&s.config.sidecar_path);
            let sidecar_tx = match s.sidecar.get_or_start(&sidecar_path).await {
                Ok(tx) => tx,
                Err(e) => {
                    let resp = IpcResponse::err("job_stats_update", format!("Failed to start sidecar: {}", e));
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                    return;
                }
            };

            let pm = s.plugin_manager.clone();
            let result = s.job_manager.start_job(uuid, sidecar_tx, Some(pm));
            drop(s);

            if let Some((runner, mut hits_rx)) = result {
                let job_id = uuid;
                let state2 = state.clone();

                // Single channel → single eval_js consumer (eval_js can only be moved once)
                let (js_tx, mut js_rx) = tokio::sync::mpsc::channel::<String>(512);
                inner_handle.spawn(async move {
                    while let Some(js) = js_rx.recv().await { eval_js(js); }
                });

                // Hit collector: store hit AND broadcast runner_hit for immediate live append
                let js_hit = js_tx.clone();
                inner_handle.spawn(async move {
                    while let Some(hit) = hits_rx.recv().await {
                        let mut s = state2.lock().await;
                        s.job_manager.add_hit(job_id, hit.clone());
                        // Push runner_hit so frontend can append without polling
                        let resp = IpcResponse::ok("runner_hit", serde_json::to_value(&hit).unwrap_or_default());
                        let _ = js_hit.send(format!("window.__ipc_callback({})",
                            serde_json::to_string(&resp).unwrap_or_default())).await;
                    }
                });

                // Spawn periodic stats push
                let state3 = state.clone();
                let js_tx2 = js_tx.clone();
                let runner2 = runner.clone();
                inner_handle.spawn(async move {
                    loop {
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        let mut s = state3.lock().await;
                        s.job_manager.update_job_stats(job_id);
                        let jobs = s.job_manager.list_jobs();
                        let resp = IpcResponse::ok("jobs_list", serde_json::to_value(jobs).unwrap_or_default());
                        let _ = js_tx2.send(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default())).await;
                        if !runner2.is_running() { break; }
                    }
                });

                // Spawn runner
                let state4 = state.clone();
                inner_handle.spawn(async move {
                    runner.start().await;
                    // Mark completed
                    let mut s = state4.lock().await;
                    if let Some(job) = s.job_manager.get_job_mut(job_id) {
                        job.state = ironbullet::runner::job::JobState::Completed;
                        job.completed = Some(chrono::Utc::now());
                    }
                    s.job_manager.update_job_stats(job_id);
                    let jobs = s.job_manager.list_jobs();
                    let resp = IpcResponse::ok("jobs_list", serde_json::to_value(jobs).unwrap_or_default());
                    let _ = js_tx.send(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default())).await;
                });
            }
        });
    }
}

pub(super) fn pause_job(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let mut s = state.lock().await;
            if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                if let Ok(uuid) = uuid::Uuid::parse_str(id) {
                    s.job_manager.pause_job(uuid);
                }
            }
            let jobs = s.job_manager.list_jobs();
            let resp = IpcResponse::ok("jobs_list", serde_json::to_value(jobs).unwrap_or_default());
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
        });
    }
}

pub(super) fn resume_job(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let mut s = state.lock().await;
            if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                if let Ok(uuid) = uuid::Uuid::parse_str(id) {
                    s.job_manager.resume_job(uuid);
                }
            }
            let jobs = s.job_manager.list_jobs();
            let resp = IpcResponse::ok("jobs_list", serde_json::to_value(jobs).unwrap_or_default());
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
        });
    }
}

pub(super) fn stop_job(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let mut s = state.lock().await;
            if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                if let Ok(uuid) = uuid::Uuid::parse_str(id) {
                    s.job_manager.stop_job(uuid);
                    // Kill sidecar if no config jobs remain running.
                    // Proxy-check jobs don't use the sidecar, so only check config jobs.
                    if !s.job_manager.any_config_job_running() {
                        s.sidecar.stop().await;
                        eprintln!("[stop_job] No running config jobs — sidecar stopped.");
                    }
                }
            }
            let jobs = s.job_manager.list_jobs();
            let resp = IpcResponse::ok("jobs_list", serde_json::to_value(jobs).unwrap_or_default());
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
        });
    }
}

pub(super) fn get_job_stats(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let mut s = state.lock().await;
            if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                if let Ok(uuid) = uuid::Uuid::parse_str(id) {
                    s.job_manager.update_job_stats(uuid);
                    let stats = s.job_manager.get_job_stats(uuid);
                    let resp = IpcResponse::ok("job_stats_update", serde_json::json!({
                        "id": id,
                        "stats": serde_json::to_value(&stats).unwrap_or_default(),
                    }));
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                }
            }
        });
    }
}

pub(super) fn get_job_hits(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let s = state.lock().await;
            if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                if let Ok(uuid) = uuid::Uuid::parse_str(id) {
                    let hits = s.job_manager.get_job_hits(uuid);
                    let resp = IpcResponse::ok("job_hits", serde_json::json!({
                        "id": id,
                        "hits": serde_json::to_value(&hits).unwrap_or_default(),
                    }));
                    eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                }
            }
        });
    }
}

pub(super) fn update_job(
    state: Arc<Mutex<AppState>>,
    data: serde_json::Value,
    eval_js: impl Fn(String) + Send + 'static,
) {
    let rt = tokio::runtime::Handle::try_current();
    if let Ok(handle) = rt {
        handle.spawn(async move {
            let mut s = state.lock().await;
            let id_str = data.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            if let Ok(uuid) = uuid::Uuid::parse_str(&id_str) {
                if let Some(job) = s.job_manager.get_job_mut(uuid) {
                    if let Some(name) = data.get("name").and_then(|v| v.as_str()) {
                        job.name = name.to_string();
                    }
                    if let Some(tc) = data.get("thread_count").and_then(|v| v.as_u64()) {
                        job.thread_count = tc as usize;
                    }
                    if let Some(ds) = data.get("data_source") {
                        if let Ok(new_ds) = serde_json::from_value::<ironbullet::runner::job::DataSource>(ds.clone()) {
                            job.data_source = new_ds;
                        }
                    }
                    if let Some(url) = data.get("proxy_check_url").and_then(|v| v.as_str()) {
                        job.proxy_check_url = url.to_string();
                    }
                    if let Some(list) = data.get("proxy_check_list").and_then(|v| v.as_str()) {
                        job.proxy_check_list = list.to_string();
                    }
                }
            }
            let resp = IpcResponse::ok("job_updated", serde_json::json!({ "id": id_str }));
            eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
        });
    }
}
