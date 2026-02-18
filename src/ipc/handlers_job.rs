use std::sync::Arc;
use tokio::sync::Mutex;

use ironbullet::export::format::RfxConfig;
use ironbullet::runner::job::Job;

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
            let mut job = if let Ok(j) = serde_json::from_value::<Job>(data.clone()) {
                j
            } else {
                Job::default()
            };
            // Override name if provided
            if let Some(name) = data.get("name").and_then(|v| v.as_str()) {
                job.name = name.to_string();
            }
            // Override config path if provided
            if let Some(path) = data.get("config_path").and_then(|v| v.as_str()) {
                job.config_path = Some(path.to_string());
                if let Ok(rfx) = RfxConfig::load_from_file(path) {
                    job.pipeline = rfx.pipeline;
                }
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

            // Start sidecar if needed
            let sidecar_path = resolve_sidecar_path(&s.config.sidecar_path);
            if !s.sidecar.is_running() {
                s.sidecar.stop().await;
                match s.sidecar.start(&sidecar_path).await {
                    Ok(_) => {}
                    Err(e) => {
                        let resp = IpcResponse::err("job_stats_update", format!("Failed to start sidecar: {}", e));
                        eval_js(format!("window.__ipc_callback({})", serde_json::to_string(&resp).unwrap_or_default()));
                        return;
                    }
                }
            }
            let sidecar_tx = match s.sidecar.start(&sidecar_path).await {
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

                // Spawn hit collector for this job
                inner_handle.spawn(async move {
                    while let Some(hit) = hits_rx.recv().await {
                        let mut s = state2.lock().await;
                        s.job_manager.add_hit(job_id, hit);
                    }
                });

                // Use a channel to relay eval_js calls from multiple tasks
                let (js_tx, mut js_rx) = tokio::sync::mpsc::channel::<String>(64);

                // eval_js consumer -- owns eval_js, calls it for each message
                inner_handle.spawn(async move {
                    while let Some(js) = js_rx.recv().await {
                        eval_js(js);
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
