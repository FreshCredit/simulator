//! Batch Experiment API Module (P1-SIM-1)
//!
//! Provides REST endpoints for executing multiple experiments in parallel.
//!
//! ## Routes
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | POST | `/api/batch` | Submit batch execution |
//! | GET | `/api/batch/:id` | Get batch status |
//! | GET | `/api/batch/:id/progress` | SSE progress stream |
//! | POST | `/api/batch/:id/cancel` | Cancel batch execution |

// TAG: surface=simulation owner=research-team rule=SIM-001
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{sse::Event, Sse},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use crate::executor::ExecutorConfig;
use crate::orchestrator::batch_executor::{
    BatchConfig, BatchExecutionHandle, BatchProgress, ExperimentTask, ParallelBatchExecutor,
};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Request to submit a batch of experiments
#[derive(Debug, Deserialize)]
pub struct SubmitBatchRequest {
    /// List of experiments to run
    pub experiments: Vec<ExperimentTaskRequest>,
    /// Maximum parallelism (default: 4)
    pub max_parallelism: Option<usize>,
    /// Continue on error (default: true)
    pub continue_on_error: Option<bool>,
}

/// Individual experiment task request
#[derive(Debug, Deserialize)]
pub struct ExperimentTaskRequest {
    /// Experiment ID from catalog
    pub experiment_id: String,
    /// Number of agents (default: 1000)
    pub agents: Option<usize>,
    /// Number of ticks (default: 365)
    pub ticks: Option<u64>,
    /// Number of replications (default: 10)
    pub replications: Option<u32>,
    /// Random seed (default: random)
    pub seed: Option<u64>,
    /// PDE4B variance level (default: "medium")
    pub pde4b_variance: Option<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl From<ExperimentTaskRequest> for ExperimentTask {
    fn from(req: ExperimentTaskRequest) -> Self {
        Self {
            run_id: uuid::Uuid::new_v4().to_string(),
            experiment_id: req.experiment_id,
            agents: req.agents.unwrap_or(1000),
            ticks: req.ticks.unwrap_or(365),
            replications: req.replications.unwrap_or(10),
            seed: req.seed.unwrap_or_else(rand::random),
            pde4b_variance: req.pde4b_variance.unwrap_or_else(|| "medium".to_string()),
        }
    }
}

/// Batch submission response
#[derive(Debug, Serialize)]
pub struct SubmitBatchResponse {
    /// batch id
    pub batch_id: String,
    /// status
    pub status: String,
    /// total tasks
    pub total_tasks: usize,
    /// estimated duration secs
    pub estimated_duration_secs: f64,
}

/// Batch status response
#[derive(Debug, Serialize)]
pub struct BatchStatusResponse {
    /// batch id
    pub batch_id: String,
    /// status
    pub status: String,
    /// progress
    pub progress: BatchProgressView,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Simplified progress view for API
#[derive(Debug, Serialize)]
pub struct BatchProgressView {
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
}

impl From<BatchProgress> for BatchProgressView {
    fn from(p: BatchProgress) -> Self {
        Self {
            total_tasks: p.total_tasks,
            completed_tasks: p.completed_tasks,
            failed_tasks: p.failed_tasks,
            running_tasks: p.running_tasks,
            pending_tasks: p.pending_tasks,
            percent_complete: p.percent_complete,
            started_at: p.started_at,
            estimated_completion: p.estimated_completion,
        }
    }
}

/// Active batch executions store
pub type BatchStore = Arc<Mutex<HashMap<String, BatchExecutionHandle>>>;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Create a new batch store
#[must_use]
pub fn create_batch_store() -> BatchStore {
    Arc::new(Mutex::new(HashMap::new()))
}

/// Combined state for batch operations
#[derive(Clone, Debug)]
pub struct BatchState {
    /// store
    pub store: BatchStore,
    /// config
    pub config: Arc<ExecutorConfig>,
}

/// POST /api/batch - Submit batch execution
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn submit_batch(
    State(state): State<BatchState>,
    Json(request): Json<SubmitBatchRequest>,
) -> Result<(StatusCode, Json<SubmitBatchResponse>), StatusCode> {
    info!(
        experiment_count = request.experiments.len(),
        max_parallelism = request.max_parallelism.unwrap_or(4),
        "Submitting batch execution"
    );

    // Convert requests to tasks
    let tasks: Vec<ExperimentTask> = request.experiments.into_iter().map(Into::into).collect();
    let total_tasks = tasks.len();
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    // Create batch config
    let batch_config = BatchConfig {
        max_parallelism: request.max_parallelism.unwrap_or(4),
        executor_config: (*state.config).clone(),
        continue_on_error: request.continue_on_error.unwrap_or(true),
        progress_interval_secs: 5,
    };

    // Create executor and start batch
    let batch_id = uuid::Uuid::new_v4().to_string();
    let executor = ParallelBatchExecutor::new(batch_config);

    let handle = executor.execute_batch(batch_id.clone(), tasks).await;

    // Estimate duration (rough heuristic)
    let estimated_duration_secs =
        60.0 * total_tasks as f64 / request.max_parallelism.unwrap_or(4) as f64;

    // Store handle
    state.store.lock().await.insert(batch_id.clone(), handle);

    let response = SubmitBatchResponse {
        batch_id: batch_id.clone(),
        status: "running".to_string(),
        total_tasks,
        estimated_duration_secs,
    };

    info!(batch_id = %batch_id, "Batch execution started");
    Ok((StatusCode::CREATED, Json(response)))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /api/batch/:id - Get batch status
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_batch_status(
    State(state): State<BatchState>,
    Path(batch_id): Path<String>,
) -> Result<Json<BatchStatusResponse>, StatusCode> {
    let mut store_guard = state.store.lock().await;

    let handle = store_guard
        .get_mut(&batch_id)
        .ok_or(StatusCode::NOT_FOUND)?;

    // Try to get latest progress
    let progress = match handle.try_recv_progress().await {
        Some(p) => p,
        None => {
            // Return a default pending progress if no update yet
            BatchProgress {
                batch_id: batch_id.clone(),
                status: crate::orchestrator::batch_executor::BatchStatus::Pending,
                total_tasks: 0,
                completed_tasks: 0,
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                failed_tasks: 0,
                running_tasks: 0,
                pending_tasks: 0,
                percent_complete: 0.0,
                started_at: chrono::Utc::now().to_rfc3339(),
                estimated_completion: None,
                results: vec![],
            }
        }
    };

    let status = match progress.status {
        crate::orchestrator::batch_executor::BatchStatus::Pending => "pending",
        crate::orchestrator::batch_executor::BatchStatus::Running => "running",
        crate::orchestrator::batch_executor::BatchStatus::Completed => "completed",
        crate::orchestrator::batch_executor::BatchStatus::Failed => "failed",
        crate::orchestrator::batch_executor::BatchStatus::Cancelled => "cancelled",
    }
    .to_string();

    Ok(Json(BatchStatusResponse {
        batch_id,
        status,
        progress: progress.into(),
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /api/batch/:id/progress - SSE progress stream
pub async fn stream_batch_progress(
    State(state): State<BatchState>,
    Path(batch_id): Path<String>,
) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    use futures::stream;
    use std::time::Duration;

    info!(batch_id = %batch_id, "Starting SSE progress stream");

    let batch_id_clone = batch_id;

    let stream = stream::unfold(
        (state.store, batch_id_clone, 0u64, false),
        |(store, batch_id, last_update, is_done)| async move {
            if is_done {
                // Clean up batch from store
                let mut store_guard = store.lock().await;
                store_guard.remove(&batch_id);
                return None;
            }

            // Wait a bit between updates
            tokio::time::sleep(Duration::from_secs(2)).await;

            // Clone store before locking to avoid borrow issues
            let store_clone = store.clone();

            // Check batch progress
            let result = {
                let mut store_guard = store.lock().await;
                if let Some(handle) = store_guard.get_mut(&batch_id) {
                    if let Some(progress) = handle.try_recv_progress().await {
                        let complete = matches!(
                            progress.status,
                            crate::orchestrator::batch_executor::BatchStatus::Completed
                                | crate::orchestrator::batch_executor::BatchStatus::Failed
                                | crate::orchestrator::batch_executor::BatchStatus::Cancelled
                        );

                        let event = Event::default().event("progress").data(
                            serde_json::to_string(&BatchProgressView::from(progress))
                                .unwrap_or_default(),
                        );

                        // TAG: surface=simulation owner=research-team rule=SIM-001
                        Some((
                            Ok(event),
                            (store_clone, batch_id, last_update + 1, complete),
                        ))
                    } else {
                        // No new progress, send heartbeat
                        let event = Event::default().event("heartbeat").data("{}");
                        Some((Ok(event), (store_clone, batch_id, last_update, false)))
                    }
                } else {
                    // Batch not found
                    None
                }
            };

            result
        },
    );

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// POST /api/batch/:id/cancel - Cancel batch execution
pub async fn cancel_batch(
    State(state): State<BatchState>,
    Path(batch_id): Path<String>,
) -> StatusCode {
    info!(batch_id = %batch_id, "Cancelling batch execution");

    // Clone the Arc to the store for the async block
    let store_for_check = state.store.clone();
    let batch_id_for_check = batch_id.clone();

    // Check if batch exists first
    let exists = {
        let store_guard = store_for_check.lock().await;
        store_guard.contains_key(&batch_id_for_check)
    };

    if !exists {
        return StatusCode::NOT_FOUND;
    }

    // Cancel the batch
    let cancel_result = {
        let mut store_guard = state.store.lock().await;
        if let Some(handle) = store_guard.get_mut(&batch_id) {
            handle.cancel().await;
            true
        } else {
            false
        }
    };

    if cancel_result {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// List all active batches
pub async fn list_active_batches(State(state): State<BatchState>) -> Json<Vec<String>> {
    let store_guard = state.store.lock().await;
    let batch_ids: Vec<String> = store_guard.keys().cloned().collect();
    Json(batch_ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_batch_state() -> BatchState {
        BatchState {
            store: create_batch_store(),
            config: Arc::new(crate::executor::ExecutorConfig::default()),
        }
    }

    #[tokio::test]
    async fn test_get_batch_status_not_found() {
        let state = test_batch_state();
        let response = get_batch_status(State(state), Path("batch1".to_string())).await;
        assert_eq!(response.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_cancel_batch_not_found() {
        let state = test_batch_state();
        let response = cancel_batch(State(state), Path("batch1".to_string())).await;
        assert_eq!(response, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_active_batches_empty() {
        let state = test_batch_state();
        let response = list_active_batches(State(state)).await;
        assert!(response.0.is_empty());
    }
}
