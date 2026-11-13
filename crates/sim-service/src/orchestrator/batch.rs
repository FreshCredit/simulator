//! Batch Job Orchestrator (B-010)
//!
//! Manages job queues, workers, and storage for batch simulation execution.
//! Supports distributed processing across multiple nodes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

use sim_core::simulation::SimulationConfig;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Batch job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchJobStatus {
    /// Queued
    Queued,
    /// Running
    Running,
    /// Paused
    Paused,
    /// Completed
    Completed,
    /// Failed
    Failed,
    /// Cancelled
    Cancelled,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Priority level for job scheduling
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum JobPriority {
    /// Low
    Low = 0,
    /// Normal
    Normal = 1,
    /// High
    High = 2,
    /// Critical
    Critical = 3,
}

/// Batch job definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchJob {
    /// id
    pub id: Uuid,
    /// name
    pub name: String,
    /// config
    pub config: SimulationConfig,
    /// n runs
    pub n_runs: u32,
    /// status
    pub status: BatchJobStatus,
    /// priority
    pub priority: JobPriority,
    /// progress
    pub progress: BatchProgress,
    /// results
    pub results: Option<BatchResults>,
    /// error message
    pub error_message: Option<String>,
    /// created at
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// started at
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// completed at
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Progress tracking for batch jobs
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BatchProgress {
    /// runs completed
    pub runs_completed: u32,
    /// runs total
    pub runs_total: u32,
    /// current run
    pub current_run: Option<u32>,
    /// current tick
    pub current_tick: Option<u64>,
    /// ticks total
    pub ticks_total: u64,
    /// percent complete
    pub percent_complete: f32,
}

/// Results from a completed batch job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResults {
    /// output dir
    pub output_dir: String,
    /// run summaries
    pub run_summaries: Vec<RunSummary>,
    /// aggregate metrics
    pub aggregate_metrics: AggregateMetrics,
    /// report path
    pub report_path: Option<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Summary of a single simulation run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummary {
    /// run number
    pub run_number: u32,
    /// seed
    pub seed: u64,
    /// duration ms
    pub duration_ms: u64,
    /// ticks completed
    pub ticks_completed: u64,
    /// final metrics
    pub final_metrics: RunMetrics,
}

/// Metrics from a single run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMetrics {
    /// coop rate
    pub coop_rate: f32,
    /// defection rate
    pub defection_rate: f32,
    /// avg trust
    pub avg_trust: f32,
    /// avg wealth
    pub avg_wealth: f32,
    /// gini
    pub gini: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Aggregate metrics across all runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateMetrics {
    /// n runs
    pub n_runs: u32,
    /// coop rate mean
    pub coop_rate_mean: f32,
    /// coop rate std
    pub coop_rate_std: f32,
    /// avg trust mean
    pub avg_trust_mean: f32,
    /// avg trust std
    pub avg_trust_std: f32,
    /// gini mean
    pub gini_mean: f32,
}

/// Worker node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerNode {
    /// id
    pub id: Uuid,
    /// hostname
    pub hostname: String,
    /// status
    pub status: WorkerStatus,
    /// capabilities
    pub capabilities: WorkerCapabilities,
    /// current job
    pub current_job: Option<Uuid>,
    /// last heartbeat
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
}

/// Worker status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerStatus {
    /// Online
    Online,
    /// Busy
    Busy,
    /// Offline
    Offline,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Worker capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCapabilities {
    /// max concurrent jobs
    pub max_concurrent_jobs: u32,
    /// max agents
    pub max_agents: u32,
    /// supports gpu
    pub supports_gpu: bool,
    /// region
    pub region: String,
}

/// Job queue manager
#[derive(Debug)]
pub struct BatchOrchestrator {
    jobs: Arc<RwLock<HashMap<Uuid, BatchJob>>>,
    workers: Arc<RwLock<HashMap<Uuid, WorkerNode>>>,
    job_tx: mpsc::Sender<OrchestratorCommand>,
}

/// Commands for the orchestrator
#[derive(Debug)]
pub enum OrchestratorCommand {
    /// `SubmitJob`
    SubmitJob(Box<BatchJob>),
    /// `CancelJob`
    CancelJob(Uuid),
    /// `PauseJob`
    PauseJob(Uuid),
    /// `ResumeJob`
    ResumeJob(Uuid),
    /// `WorkerHeartbeat`
    WorkerHeartbeat(Uuid),
    /// `WorkerOffline`
    WorkerOffline(Uuid),
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl BatchOrchestrator {
    /// Create a new batch orchestrator
    /// P0-PERF-005: Using bounded channel instead of unbounded to prevent OOM
    #[must_use]
    pub fn new() -> (Self, mpsc::Receiver<OrchestratorCommand>) {
        let (job_tx, job_rx) = mpsc::channel(10000);

        let orchestrator = Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            workers: Arc::new(RwLock::new(HashMap::new())),
            job_tx,
        };

        (orchestrator, job_rx)
    }

    /// Submit a new batch job
    pub async fn submit_job(&self, name: String, config: SimulationConfig, n_runs: u32) -> Uuid {
        let job = BatchJob {
            id: Uuid::new_v4(),
            name,
            config,
            n_runs,
            status: BatchJobStatus::Queued,
            priority: JobPriority::Normal,
            progress: BatchProgress {
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                runs_total: n_runs,
                ticks_total: 1000, // From config
                ..Default::default()
            },
            results: None,
            error_message: None,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
        };

        let id = job.id;
        {
            let mut jobs = self.jobs.write().await;
            jobs.insert(id, job.clone());
        }

        // Spawn the send to avoid blocking - fire and forget
        let job_tx = self.job_tx.clone();
        let job_box = Box::new(job);
        tokio::spawn(async move {
            let _ = job_tx.send(OrchestratorCommand::SubmitJob(job_box)).await;
        });
        id
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get job status
    pub async fn get_job(&self, id: Uuid) -> Option<BatchJob> {
        let jobs = self.jobs.read().await;
        jobs.get(&id).cloned()
    }

    /// List all jobs
    pub async fn list_jobs(&self) -> Vec<BatchJob> {
        let jobs = self.jobs.read().await;
        jobs.values().cloned().collect()
    }

    /// List active jobs (queued or running)
    pub async fn list_active_jobs(&self) -> Vec<BatchJob> {
        let jobs = self.jobs.read().await;
        jobs.values()
            .filter(|j| matches!(j.status, BatchJobStatus::Queued | BatchJobStatus::Running))
            .cloned()
            .collect()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Cancel a job
    pub async fn cancel_job(&self, id: Uuid) -> bool {
        // Fire and forget - cancellation is best effort
        let job_tx = self.job_tx.clone();
        tokio::spawn(async move {
            let _ = job_tx.send(OrchestratorCommand::CancelJob(id)).await;
        });

        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(&id) {
            job.status = BatchJobStatus::Cancelled;
            true
        } else {
            false
        }
    }

    /// Register a worker node
    pub async fn register_worker(&self, worker: WorkerNode) {
        let mut workers = self.workers.write().await;
        workers.insert(worker.id, worker);
    }

    /// Update worker heartbeat
    pub async fn worker_heartbeat(&self, worker_id: Uuid) {
        // Fire and forget heartbeat update
        let job_tx = self.job_tx.clone();
        tokio::spawn(async move {
            let _ = job_tx
                .send(OrchestratorCommand::WorkerHeartbeat(worker_id))
                .await;
        });

        let mut workers = self.workers.write().await;
        if let Some(worker) = workers.get_mut(&worker_id) {
            worker.last_heartbeat = chrono::Utc::now();
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get available workers
    pub async fn get_available_workers(&self) -> Vec<WorkerNode> {
        let workers = self.workers.read().await;
        workers
            .values()
            .filter(|w| matches!(w.status, WorkerStatus::Online))
            .cloned()
            .collect()
    }

    /// Clean up stale workers
    pub async fn cleanup_stale_workers(&self, timeout_secs: i64) {
        let now = chrono::Utc::now();
        let mut workers = self.workers.write().await;

        let stale_ids: Vec<Uuid> = workers
            .iter()
            .filter(|(_, w)| (now - w.last_heartbeat).num_seconds() > timeout_secs)
            .map(|(id, _)| *id)
            .collect();

        for id in stale_ids {
            if let Some(worker) = workers.get_mut(&id) {
                worker.status = WorkerStatus::Offline;
            }
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Job scheduler that implements priority queue
#[derive(Debug)]
pub struct JobScheduler {
    orchestrator: Arc<BatchOrchestrator>,
}

impl JobScheduler {
    #[must_use]
    /// fn
    pub const fn new(orchestrator: Arc<BatchOrchestrator>) -> Self {
        Self { orchestrator }
    }

    /// Find the next job to execute based on priority and fairness
    pub async fn next_job(&self) -> Option<Uuid> {
        let jobs = self.orchestrator.list_active_jobs().await;

        // Sort by priority (desc), then by creation time (asc)
        let mut sorted: Vec<_> = jobs
            .iter()
            .filter(|j| j.status == BatchJobStatus::Queued)
            .collect();

        sorted.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.created_at.cmp(&b.created_at))
        });
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        sorted.first().map(|j| j.id)
    }

    /// Check if any worker can handle the job
    pub async fn can_schedule(&self, _job: &BatchJob) -> bool {
        // In production, check agent count against worker capacity
        // For now, any available worker can handle jobs
        let workers = self.orchestrator.get_available_workers().await;
        !workers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_submit_and_get_job() {
        let (orchestrator, _rx) = BatchOrchestrator::new();
        let config = SimulationConfig::default();

        let job_id = orchestrator
            .submit_job("test".to_string(), config, 10)
            .await;

        let job = orchestrator.get_job(job_id).await;
        assert!(job.is_some());
        assert_eq!(job.unwrap().n_runs, 10);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[tokio::test]
    async fn test_cancel_job() {
        let (orchestrator, _rx) = BatchOrchestrator::new();
        let config = SimulationConfig::default();

        let job_id = orchestrator.submit_job("test".to_string(), config, 5).await;
        let cancelled = orchestrator.cancel_job(job_id).await;

        assert!(cancelled);

        let job = orchestrator.get_job(job_id).await.unwrap();
        assert_eq!(job.status, BatchJobStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_worker_registration() {
        let (orchestrator, _rx) = BatchOrchestrator::new();

        let worker = WorkerNode {
            id: Uuid::new_v4(),
            hostname: "test-node-1".to_string(),
            status: WorkerStatus::Online,
            capabilities: WorkerCapabilities {
                max_concurrent_jobs: 4,
                max_agents: 10000,
                supports_gpu: false,
                region: "us-central1".to_string(),
            },
            current_job: None,
            last_heartbeat: chrono::Utc::now(),
        };

        orchestrator.register_worker(worker.clone()).await;

        let available = orchestrator.get_available_workers().await;
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].hostname, "test-node-1");
    }
}
