pub mod worker;
pub mod data_pool;
pub mod proxy_pool;
pub mod output;
pub mod job;
pub mod job_manager;

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

use crate::pipeline::Pipeline;
use crate::sidecar::protocol::{SidecarRequest, SidecarResponse};
use data_pool::DataPool;
use proxy_pool::ProxyPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerStats {
    pub total: usize,
    pub processed: usize,
    pub hits: usize,
    pub fails: usize,
    pub bans: usize,
    pub retries: usize,
    pub errors: usize,
    pub cpm: f64,
    pub active_threads: usize,
    pub elapsed_secs: f64,
}

pub struct RunnerOrchestrator {
    pipeline: Pipeline,
    data_pool: Arc<DataPool>,
    proxy_pool: Arc<ProxyPool>,
    sidecar_tx: mpsc::Sender<(SidecarRequest, oneshot::Sender<SidecarResponse>)>,
    thread_count: usize,
    running: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    stats: Arc<RunnerStatsInner>,
    hits_tx: mpsc::Sender<HitResult>,
    output_writer: Option<Arc<output::OutputWriter>>,
    plugin_manager: Option<Arc<crate::plugin::manager::PluginManager>>,
}

pub(crate) struct RunnerStatsInner {
    processed: AtomicUsize,
    hits: AtomicUsize,
    fails: AtomicUsize,
    bans: AtomicUsize,
    retries: AtomicUsize,
    errors: AtomicUsize,
    active_threads: AtomicUsize,
    start_time_ms: AtomicU64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitResult {
    pub data_line: String,
    pub captures: std::collections::HashMap<String, String>,
    pub proxy: Option<String>,
}

impl RunnerOrchestrator {
    pub fn new(
        pipeline: Pipeline,
        data_pool: DataPool,
        proxy_pool: ProxyPool,
        sidecar_tx: mpsc::Sender<(SidecarRequest, oneshot::Sender<SidecarResponse>)>,
        thread_count: usize,
        hits_tx: mpsc::Sender<HitResult>,
        plugin_manager: Option<Arc<crate::plugin::manager::PluginManager>>,
    ) -> Self {
        let ow = if pipeline.output_settings.save_to_file {
            Some(Arc::new(output::OutputWriter::new(
                &pipeline.output_settings,
                &pipeline.name,
            )))
        } else {
            None
        };
        Self {
            pipeline,
            data_pool: Arc::new(data_pool),
            proxy_pool: Arc::new(proxy_pool),
            sidecar_tx,
            thread_count,
            running: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            stats: Arc::new(RunnerStatsInner {
                processed: AtomicUsize::new(0),
                hits: AtomicUsize::new(0),
                fails: AtomicUsize::new(0),
                bans: AtomicUsize::new(0),
                retries: AtomicUsize::new(0),
                errors: AtomicUsize::new(0),
                active_threads: AtomicUsize::new(0),
                start_time_ms: AtomicU64::new(0),
            }),
            hits_tx,
            output_writer: ow,
            plugin_manager,
        }
    }

    pub async fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
        self.paused.store(false, Ordering::SeqCst);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.stats.start_time_ms.store(now, Ordering::SeqCst);

        let mut handles = Vec::new();

        for _ in 0..self.thread_count {
            let pipeline = self.pipeline.clone();
            let data_pool = self.data_pool.clone();
            let proxy_pool = self.proxy_pool.clone();
            let sidecar_tx = self.sidecar_tx.clone();
            let running = self.running.clone();
            let paused = self.paused.clone();
            let stats = self.stats.clone();
            let hits_tx = self.hits_tx.clone();
            let output_writer = self.output_writer.clone();
            let plugin_manager = self.plugin_manager.clone();

            let handle = tokio::spawn(async move {
                worker::run_worker(
                    pipeline,
                    data_pool,
                    proxy_pool,
                    sidecar_tx,
                    running,
                    paused,
                    stats,
                    hits_tx,
                    output_writer,
                    plugin_manager,
                ).await;
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.await;
        }
    }

    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
    }

    pub fn resume(&self) {
        self.paused.store(false, Ordering::SeqCst);
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn get_stats(&self) -> RunnerStats {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let start = self.stats.start_time_ms.load(Ordering::Relaxed);
        let elapsed_secs = if start > 0 { (now - start) as f64 / 1000.0 } else { 0.0 };
        let processed = self.stats.processed.load(Ordering::Relaxed);
        let cpm = if elapsed_secs > 0.0 { processed as f64 / elapsed_secs * 60.0 } else { 0.0 };

        RunnerStats {
            total: self.data_pool.total(),
            processed,
            hits: self.stats.hits.load(Ordering::Relaxed),
            fails: self.stats.fails.load(Ordering::Relaxed),
            bans: self.stats.bans.load(Ordering::Relaxed),
            retries: self.stats.retries.load(Ordering::Relaxed),
            errors: self.stats.errors.load(Ordering::Relaxed),
            cpm,
            active_threads: self.stats.active_threads.load(Ordering::Relaxed),
            elapsed_secs,
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}
