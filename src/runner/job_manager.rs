use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::sidecar::protocol::{SidecarRequest, SidecarResponse};
use super::data_pool::DataPool;
use super::job::{DataSourceType, Job, JobState, StartCondition};
use super::proxy_pool::ProxyPool;
use super::{HitResult, RunnerOrchestrator, RunnerStats};

pub struct JobManager {
    jobs: Vec<Job>,
    runners: HashMap<Uuid, Arc<RunnerOrchestrator>>,
    job_hits: HashMap<Uuid, Vec<HitResult>>,
}

impl JobManager {
    pub fn new() -> Self {
        Self {
            jobs: Vec::new(),
            runners: HashMap::new(),
            job_hits: HashMap::new(),
        }
    }

    pub fn add_job(&mut self, job: Job) -> Uuid {
        let id = job.id;
        self.jobs.push(job);
        self.job_hits.insert(id, Vec::new());
        id
    }

    pub fn remove_job(&mut self, id: Uuid) -> bool {
        if let Some(runner) = self.runners.get(&id) {
            runner.stop();
        }
        self.runners.remove(&id);
        self.job_hits.remove(&id);
        let len = self.jobs.len();
        self.jobs.retain(|j| j.id != id);
        self.jobs.len() < len
    }

    pub fn list_jobs(&self) -> &[Job] {
        &self.jobs
    }

    pub fn get_job_mut(&mut self, id: Uuid) -> Option<&mut Job> {
        self.jobs.iter_mut().find(|j| j.id == id)
    }

    pub fn get_job_stats(&self, id: Uuid) -> Option<RunnerStats> {
        self.runners.get(&id).map(|r| r.get_stats())
    }

    pub fn get_job_hits(&self, id: Uuid) -> Vec<HitResult> {
        self.job_hits.get(&id).cloned().unwrap_or_default()
    }

    pub fn add_hit(&mut self, id: Uuid, hit: HitResult) {
        if let Some(hits) = self.job_hits.get_mut(&id) {
            hits.push(hit);
        }
    }

    pub fn pause_job(&mut self, id: Uuid) -> bool {
        if let Some(runner) = self.runners.get(&id) {
            runner.pause();
            if let Some(job) = self.get_job_mut(id) {
                job.state = JobState::Paused;
            }
            true
        } else {
            false
        }
    }

    pub fn resume_job(&mut self, id: Uuid) -> bool {
        if let Some(runner) = self.runners.get(&id) {
            runner.resume();
            if let Some(job) = self.get_job_mut(id) {
                job.state = JobState::Running;
            }
            true
        } else {
            false
        }
    }

    pub fn stop_job(&mut self, id: Uuid) -> bool {
        if let Some(runner) = self.runners.get(&id) {
            runner.stop();
            if let Some(job) = self.get_job_mut(id) {
                job.state = JobState::Stopped;
                job.completed = Some(Utc::now());
            }
            true
        } else {
            false
        }
    }

    pub fn start_job(
        &mut self,
        id: Uuid,
        sidecar_tx: mpsc::Sender<(SidecarRequest, oneshot::Sender<SidecarResponse>)>,
        plugin_manager: Option<Arc<crate::plugin::manager::PluginManager>>,
    ) -> Option<(Arc<RunnerOrchestrator>, mpsc::Receiver<HitResult>)> {
        let job = self.jobs.iter_mut().find(|j| j.id == id)?;

        // Load data from source
        let data_lines = match job.data_source.source_type {
            DataSourceType::File => {
                std::fs::read_to_string(&job.data_source.value)
                    .unwrap_or_default()
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .map(|l| l.to_string())
                    .collect::<Vec<_>>()
            }
            DataSourceType::Folder => {
                // Read all .txt / .csv files in the folder, concatenate their lines
                let mut all_lines: Vec<String> = Vec::new();
                if let Ok(rd) = std::fs::read_dir(&job.data_source.value) {
                    let mut paths: Vec<_> = rd
                        .filter_map(|e| e.ok())
                        .map(|e| e.path())
                        .filter(|p| p.is_file() && matches!(
                            p.extension().and_then(|s| s.to_str()),
                            Some("txt") | Some("csv") | Some("lst") | Some("dat")
                        ))
                        .collect();
                    paths.sort();
                    for path in paths {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            all_lines.extend(
                                content.lines()
                                    .filter(|l| !l.trim().is_empty())
                                    .map(|l| l.to_string())
                            );
                        }
                    }
                }
                all_lines
            }
            DataSourceType::Inline => {
                job.data_source.value.lines()
                    .filter(|l| !l.trim().is_empty())
                    .map(|l| l.to_string())
                    .collect::<Vec<_>>()
            }
            _ => Vec::new(),
        };

        let data_pool = DataPool::new(data_lines);
        let proxy_pool = ProxyPool::empty();
        let (hits_tx, hits_rx) = mpsc::channel::<HitResult>(1024);

        let runner = Arc::new(RunnerOrchestrator::new(
            job.pipeline.clone(),
            data_pool,
            proxy_pool,
            sidecar_tx,
            job.thread_count,
            hits_tx,
            plugin_manager,
        ));

        job.state = JobState::Running;
        job.started = Some(Utc::now());
        self.runners.insert(id, runner.clone());

        Some((runner, hits_rx))
    }

    /// Check start conditions for queued jobs (delayed/scheduled)
    pub fn tick(&mut self) -> Vec<Uuid> {
        let now = Utc::now();
        let mut ready = Vec::new();

        for job in &mut self.jobs {
            if job.state != JobState::Queued {
                continue;
            }
            let should_start = match &job.start_condition {
                StartCondition::Immediate => true,
                StartCondition::Delayed { delay_secs } => {
                    let elapsed = (now - job.created).num_seconds();
                    elapsed >= *delay_secs as i64
                }
                StartCondition::Scheduled { at } => now >= *at,
            };
            if should_start {
                job.state = JobState::Waiting;
                ready.push(job.id);
            }
        }

        ready
    }

    pub fn update_job_stats(&mut self, id: Uuid) {
        if let Some(stats) = self.get_job_stats(id) {
            if let Some(job) = self.get_job_mut(id) {
                job.stats = stats;
            }
        }
    }
}
