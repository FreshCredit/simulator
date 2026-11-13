//! Parallel Batch Experiment Executor (P1-SIM-1)
//!
//! Executes multiple experiments in parallel with configurable concurrency.
//! Provides progress tracking and result aggregation.

use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, Mutex, Semaphore};
use tokio::task::JoinHandle;
use tracing::{error, info};

use crate::executor::{execute_experiment, ExecutorConfig};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Batch execution configuration
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum number of concurrent experiments
    pub max_parallelism: usize,
    /// Executor configuration
    pub executor_config: ExecutorConfig,
    /// Whether to continue on individual experiment failure
    pub continue_on_error: bool,
    /// Progress update interval (seconds)
    pub progress_interval_secs: u64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_parallelism: 4,
            executor_config: ExecutorConfig::default(),
            continue_on_error: true,
            progress_interval_secs: 5,
        }
    }
}

/// Single experiment task within a batch
#[derive(Debug, Clone)]
pub struct ExperimentTask {
    /// Experiment ID from catalog
    pub experiment_id: String,
    /// Unique run ID for this execution
    pub run_id: String,
    /// Number of agents
    pub agents: usize,
    /// Number of ticks
    pub ticks: u64,
    /// Number of replications
    pub replications: u32,
    /// Random seed
    pub seed: u64,
    /// PDE4B variance level
    pub pde4b_variance: String,
}

/// Batch execution status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchStatus {
    /// Pending
    Pending,
    /// Running
    Running,
    /// Completed
    Completed,
    /// Failed
    Failed,
    /// Cancelled
    Cancelled,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Individual experiment result
#[derive(Debug, Clone)]
pub struct ExperimentResult {
    /// run id
    pub run_id: String,
    /// experiment id
    pub experiment_id: String,
    /// status
    pub status: ExperimentStatus,
    /// started at
    pub started_at: String,
    /// completed at
    pub completed_at: Option<String>,
    /// duration secs
    pub duration_secs: Option<f64>,
    /// error message
    pub error_message: Option<String>,
    /// artifacts path
    pub artifacts_path: Option<String>,
}

/// Experiment execution status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExperimentStatus {
    /// Pending
    Pending,
    /// Running
    Running,
    /// Completed
    Completed,
    /// Failed
    Failed,
    /// Cancelled
    Cancelled,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Batch execution progress update
#[derive(Debug, Clone)]
pub struct BatchProgress {
    /// batch id
    pub batch_id: String,
    /// status
    pub status: BatchStatus,
    /// total tasks
    pub total_tasks: usize,
    /// completed tasks
    pub completed_tasks: usize,
    /// failed tasks
    pub failed_tasks: usize,
    /// running tasks
    pub running_tasks: usize,
    /// pending tasks
    pub pending_tasks: usize,
    /// percent complete
    pub percent_complete: f64,
    /// started at
    pub started_at: String,
    /// estimated completion
    pub estimated_completion: Option<String>,
    /// results
    pub results: Vec<ExperimentResult>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Batch execution handle for monitoring and control
#[derive(Debug)]
pub struct BatchExecutionHandle {
    /// batch id
    pub batch_id: String,
    progress_rx: mpsc::Receiver<BatchProgress>,
    cancel_tx: broadcast::Sender<()>,
    task_handles: Vec<JoinHandle<()>>,
}

impl BatchExecutionHandle {
    /// Get the next progress update (non-blocking)
    pub async fn try_recv_progress(&mut self) -> Option<BatchProgress> {
        tracing::trace!(batch_id = %self.batch_id, "Polling batch progress");
        self.progress_rx.try_recv().ok()
    }

    /// Get the next progress update (blocking)
    pub async fn recv_progress(&mut self) -> Option<BatchProgress> {
        tracing::trace!(batch_id = %self.batch_id, "Awaiting batch progress");
        self.progress_rx.recv().await
    }

    /// Cancel the batch execution
    pub async fn cancel(&self) {
        tracing::info!(batch_id = %self.batch_id, "Cancelling batch execution");
        let _ = self.cancel_tx.send(());
    }

    /// Wait for all tasks to complete
    pub async fn wait_for_completion(self) {
        tracing::info!(batch_id = %self.batch_id, "Awaiting batch tasks completion");
        futures::future::join_all(self.task_handles).await;
    }
}

/// Parallel batch experiment executor
#[derive(Debug)]
pub struct ParallelBatchExecutor {
    config: BatchConfig,
    semaphore: Arc<Semaphore>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl ParallelBatchExecutor {
    /// Create a new batch executor with the given configuration
    #[must_use]
    pub fn new(config: BatchConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_parallelism));
        Self { config, semaphore }
    }

    /// Execute a batch of experiments in parallel
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub async fn execute_batch(
        &self,
        batch_id: String,
        tasks: Vec<ExperimentTask>,
    ) -> BatchExecutionHandle {
        info!(
            batch_id = %batch_id,
            task_count = tasks.len(),
            max_parallelism = self.config.max_parallelism,
            "Starting parallel batch execution"
        );

        let (progress_tx, progress_rx) = mpsc::channel(100);
        let (cancel_tx, cancel_rx) = broadcast::channel(1);
        let cancel_rx = cancel_rx;

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let total_tasks = tasks.len();
        let results = Arc::new(Mutex::new(Vec::with_capacity(total_tasks)));
        let mut task_handles = Vec::new();

        let started_at = chrono::Utc::now().to_rfc3339();

        // Spawn progress reporter
        let progress_reporter = self.spawn_progress_reporter(
            batch_id.clone(),
            total_tasks,
            results.clone(),
            progress_tx,
            started_at.clone(),
        );
        task_handles.push(progress_reporter);

        // Spawn experiment tasks
        for task in tasks {
            let sem = self.semaphore.clone();
            let config = self.config.clone();
            let results = results.clone();
            let executor_config = config.executor_config.clone();
            let artifacts_dir = executor_config.artifacts_dir.clone();
            let mut cancel_rx = cancel_rx.resubscribe();

            let handle = tokio::spawn(async move {
                let Ok(permit) = sem.acquire_owned().await else {
                    tracing::warn!("Semaphore closed while awaiting batch task permit");
                    return;
                };

                let run_id = task.run_id.clone();
                let experiment_id = task.experiment_id.clone();

                // TAG: surface=simulation owner=research-team rule=SIM-001
                // Record start
                {
                    let mut results_guard = results.lock().await;
                    results_guard.push(ExperimentResult {
                        run_id: run_id.clone(),
                        experiment_id: experiment_id.clone(),
                        status: ExperimentStatus::Running,
                        started_at: chrono::Utc::now().to_rfc3339(),
                        completed_at: None,
                        duration_secs: None,
                        error_message: None,
                        artifacts_path: None,
                    });
                }

                // Execute experiment
                let start_time = std::time::Instant::now();

                let result = tokio::select! {
                    result = execute_experiment(
                        experiment_id.clone(),
                        run_id.clone(),
                        task.agents,
                        task.ticks,
                        task.replications,
                        task.seed,
                        task.pde4b_variance,
                        executor_config,
                    ) => {
                        result
                    }
                    _ = cancel_rx.recv() => {
                        info!(run_id = %run_id, "Experiment cancelled");
                        Err("Cancelled".to_string())
                    }
                };

                // TAG: surface=simulation owner=research-team rule=SIM-001
                let duration = start_time.elapsed().as_secs_f64();

                // Update result
                {
                    let mut results_guard = results.lock().await;
                    if let Some(existing) = results_guard.iter_mut().find(|r| r.run_id == run_id) {
                        existing.completed_at = Some(chrono::Utc::now().to_rfc3339());
                        existing.duration_secs = Some(duration);

                        match result {
                            Ok(()) => {
                                existing.status = ExperimentStatus::Completed;
                                existing.artifacts_path =
                                    Some(format!("{}/{}", artifacts_dir.display(), run_id));
                                info!(run_id = %run_id, duration = %duration, "Experiment completed successfully");
                            }
                            Err(e) => {
                                existing.status = ExperimentStatus::Failed;
                                existing.error_message = Some(e.clone());
                                error!(run_id = %run_id, error = %e, "Experiment failed");
                            }
                        }
                    }
                }

                // Release permit
                drop(permit);
            });

            task_handles.push(handle);
        }

        BatchExecutionHandle {
            batch_id,
            progress_rx,
            cancel_tx,
            task_handles,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Spawn a background task that reports progress
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    fn spawn_progress_reporter(
        &self,
        batch_id: String,
        total_tasks: usize,
        results: Arc<Mutex<Vec<ExperimentResult>>>,
        progress_tx: mpsc::Sender<BatchProgress>,
        started_at: String,
    ) -> JoinHandle<()> {
        let interval = tokio::time::Duration::from_secs(self.config.progress_interval_secs);
        let max_parallelism = self.config.max_parallelism;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);

            loop {
                interval.tick().await;

                let results_guard = results.lock().await;
                let completed = results_guard
                    .iter()
                    .filter(|r| {
                        matches!(
                            r.status,
                            ExperimentStatus::Completed
                                | ExperimentStatus::Failed
                                | ExperimentStatus::Cancelled // TAG: surface=simulation owner=research-team rule=SIM-001
                        )
                    })
                    .count();
                let failed = results_guard
                    .iter()
                    .filter(|r| matches!(r.status, ExperimentStatus::Failed))
                    .count();
                let running = results_guard
                    .iter()
                    .filter(|r| matches!(r.status, ExperimentStatus::Running))
                    .count();
                let pending = total_tasks - results_guard.len();

                let percent_complete = if total_tasks > 0 {
                    (completed as f64 / total_tasks as f64) * 100.0
                } else {
                    0.0
                };

                // Calculate estimated completion
                let estimated_completion = if completed > 0 {
                    let avg_duration: f64 = results_guard
                        .iter()
                        .filter_map(|r| r.duration_secs)
                        .sum::<f64>()
                        / completed as f64;
                    let remaining_tasks = total_tasks - completed;
                    let estimated_remaining_secs =
                        avg_duration * remaining_tasks as f64 / max_parallelism as f64;

                    chrono::DateTime::parse_from_rfc3339(&started_at)
                        .ok()
                        .map(|start| {
                            let estimated =
                                start + chrono::Duration::seconds(estimated_remaining_secs as i64);
                            estimated.to_rfc3339()
                        })
                } else {
                    None
                };

                // TAG: surface=simulation owner=research-team rule=SIM-001
                let status = if completed == total_tasks {
                    if failed > 0 {
                        BatchStatus::Failed
                    } else {
                        BatchStatus::Completed
                    }
                } else {
                    BatchStatus::Running
                };

                let progress = BatchProgress {
                    batch_id: batch_id.clone(),
                    status,
                    total_tasks,
                    completed_tasks: completed,
                    failed_tasks: failed,
                    running_tasks: running,
                    pending_tasks: pending,
                    percent_complete,
                    started_at: started_at.clone(),
                    estimated_completion,
                    results: results_guard.clone(),
                };

                drop(results_guard);

                if progress_tx.send(progress).await.is_err() {
                    break;
                }

                if completed == total_tasks {
                    break;
                }
            }
        })
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Execute multiple experiments and wait for all to complete
/// # Panics
///
/// Panics if preconditions are not met.
pub async fn execute_experiments_parallel(
    tasks: Vec<ExperimentTask>,
    config: BatchConfig,
) -> Vec<ExperimentResult> {
    let batch_id = uuid::Uuid::new_v4().to_string();
    let executor = ParallelBatchExecutor::new(config);

    let mut handle = executor.execute_batch(batch_id.clone(), tasks).await;

    // Wait for completion and collect results
    let mut final_progress = None;
    while let Some(progress) = handle.recv_progress().await {
        let is_done = matches!(
            progress.status,
            BatchStatus::Completed | BatchStatus::Failed
        );
        final_progress = Some(progress);
        if is_done {
            break;
        }
    }

    handle.wait_for_completion().await;

    final_progress.map(|p| p.results).unwrap_or_default()
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_batch_config_default() {
        let config = BatchConfig::default();
        assert_eq!(config.max_parallelism, 4);
        assert!(config.continue_on_error);
    }

    #[test]
    fn test_batch_progress_calculation() {
        let progress = BatchProgress {
            batch_id: "test".to_string(),
            status: BatchStatus::Running,
            total_tasks: 10,
            completed_tasks: 5,
            failed_tasks: 1,
            running_tasks: 2,
            pending_tasks: 2,
            percent_complete: 50.0,
            started_at: chrono::Utc::now().to_rfc3339(),
            estimated_completion: None,
            results: vec![],
        };

        assert_eq!(progress.percent_complete, 50.0);
    }

    #[tokio::test]
    async fn test_parallel_executor_creation() {
        let config = BatchConfig::default();
        let executor = ParallelBatchExecutor::new(config);
        assert_eq!(executor.config.max_parallelism, 4);
    }
}
