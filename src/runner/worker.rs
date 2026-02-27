use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::pipeline::engine::ExecutionContext;
use crate::pipeline::{BotStatus, Pipeline};
use crate::sidecar::protocol::{SidecarRequest, SidecarResponse};
use super::data_pool::DataPool;
use super::proxy_pool::ProxyPool;
use super::output::OutputWriter;
use super::{HitResult, RunnerStatsInner};

pub(crate) async fn run_worker(
    pipeline: Pipeline,
    data_pool: Arc<DataPool>,
    proxy_pool: Arc<ProxyPool>,
    sidecar_tx: mpsc::Sender<(SidecarRequest, oneshot::Sender<SidecarResponse>)>,
    running: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    stats: Arc<RunnerStatsInner>,
    hits_tx: mpsc::Sender<HitResult>,
    output_writer: Option<Arc<OutputWriter>>,
    plugin_manager: Option<Arc<crate::plugin::manager::PluginManager>>,
) {
    stats.active_threads.fetch_add(1, Ordering::Relaxed);

    // Create a session for this worker
    let session_id = Uuid::new_v4().to_string();
    let new_session_req = SidecarRequest {
        id: Uuid::new_v4().to_string(),
        action: "new_session".into(),
        session: session_id.clone(),
        method: None,
        url: None,
        headers: None,
        body: None,
        timeout: None,
        proxy: None,
        browser: Some(pipeline.browser_settings.browser.clone()),
        ja3: pipeline.browser_settings.ja3.clone(),
        http2fp: pipeline.browser_settings.http2_fingerprint.clone(),
        follow_redirects: Some(true),
        max_redirects: Some(8),
        ssl_verify: None,

    };
    let (resp_tx, _) = oneshot::channel();
    let _ = sidecar_tx.send((new_session_req, resp_tx)).await;

    while running.load(Ordering::Relaxed) {
        // Check pause
        while paused.load(Ordering::Relaxed) && running.load(Ordering::Relaxed) {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        if !running.load(Ordering::Relaxed) {
            break;
        }

        // Get next data line
        let data_line = match data_pool.next_line() {
            Some(line) => line,
            None => break, // All data consumed
        };

        // Get proxy if needed
        let proxy = proxy_pool.next_proxy();

        // Set up execution context
        let mut ctx = ExecutionContext::new(session_id.clone());
        ctx.proxy = proxy.clone();
        ctx.plugin_manager = plugin_manager.clone();

        // Parse data line into input variables
        let parts: Vec<&str> = data_line.split(pipeline.data_settings.separator).collect();
        for (i, slice_name) in pipeline.data_settings.slices.iter().enumerate() {
            if let Some(part) = parts.get(i) {
                ctx.variables.set_input(slice_name, part.to_string());
            }
        }

        // Execute pipeline
        let result = ctx.execute_blocks(&pipeline.blocks, &sidecar_tx).await;

        // Record stats
        stats.processed.fetch_add(1, Ordering::Relaxed);
        match ctx.status {
            BotStatus::Success => {
                stats.hits.fetch_add(1, Ordering::Relaxed);
                let hit = HitResult {
                    data_line: data_line.clone(),
                    captures: ctx.variables.captures(),
                    proxy: proxy.clone(),
                };
                if let Some(ref ow) = output_writer {
                    ow.write_hit(&hit, BotStatus::Success);
                }
                let _ = hits_tx.send(hit).await;
            }
            BotStatus::Fail => {
                stats.fails.fetch_add(1, Ordering::Relaxed);
            }
            BotStatus::Ban => {
                stats.bans.fetch_add(1, Ordering::Relaxed);
                if let Some(ref p) = ctx.proxy {
                    proxy_pool.ban_proxy(p);
                }
            }
            BotStatus::Retry => {
                stats.retries.fetch_add(1, Ordering::Relaxed);
                data_pool.return_line(data_line);
            }
            BotStatus::Error => {
                stats.errors.fetch_add(1, Ordering::Relaxed);
            }
            _ => {
                if result.is_err() {
                    stats.errors.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    }

    // Close session
    let close_req = SidecarRequest {
        id: Uuid::new_v4().to_string(),
        action: "close_session".into(),
        session: session_id,
        method: None, url: None, headers: None, body: None, timeout: None,
        proxy: None, browser: None, ja3: None, http2fp: None,
        follow_redirects: None, max_redirects: None,
        ssl_verify: None,

    };
    let (resp_tx, _) = oneshot::channel();
    let _ = sidecar_tx.send((close_req, resp_tx)).await;

    stats.active_threads.fetch_sub(1, Ordering::Relaxed);
}
