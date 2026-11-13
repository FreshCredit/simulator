//! Parameter Sweep API
//!
//! Per P2-8: Expose /api/sweep endpoint for batch simulation runs.
//! Allows running multiple simulations with varied parameters to explore
//! the parameter space efficiently.
//!
//! SIM-001: Does not depend on sim-core directly.
//! SIM-006: RBAC limits enforced (Internal role only for sweeps).

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::executor::ExecutorConfig;
use crate::jobs::{Job, JobRequest, JobStore, ParameterPreset};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Parameter sweep request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepRequest {
    /// Base configuration for all runs
    pub base_config: SweepBaseConfig,
    /// Parameters to sweep over
    pub sweep_params: SweepParams,
    /// Maximum concurrent jobs (limited by role)
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: u32,
}

const fn default_max_concurrent() -> u32 {
    4
}

/// Base configuration for sweep runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepBaseConfig {
    /// Number of agents per run
    pub agent_count: u32,
    /// Maximum ticks per run
    pub max_ticks: u64,
    /// Simulation stage (0-5)
    pub stage: u8,
    /// Grid dimensions
    #[serde(default = "default_grid_size")]
    pub grid_width: u32,
    #[serde(default = "default_grid_size")]
    /// grid height
    pub grid_height: u32,
}

const fn default_grid_size() -> u32 {
    64
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Parameters to sweep
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepParams {
    /// Seeds to run (each creates a separate run)
    pub seeds: Vec<u64>,
    /// Presets to test (each creates a separate run)
    #[serde(default)]
    pub presets: Vec<ParameterPreset>,
    /// Toggle combinations to test
    #[serde(default)]
    pub toggle_sets: Vec<ToggleSet>,
}

/// Toggle configuration for a sweep run
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleSet {
    /// Name for this toggle configuration
    pub name: String,
    /// Genome toggle
    pub genome_enabled: bool,
    /// DOCAS toggle
    pub docas_enabled: bool,
    /// DMN toggle
    pub dmn_enabled: bool,
    /// Choice architecture toggle
    pub choice_architecture_enabled: bool,
    /// Protocol toggle
    pub protocol_enabled: bool,
}

/// Sweep job response
#[derive(Debug, Clone, Serialize)]
pub struct SweepResponse {
    /// Sweep ID
    pub sweep_id: String,
    /// Total runs queued
    pub total_runs: usize,
    /// Individual job IDs
    pub job_ids: Vec<String>,
    /// Status
    pub status: &'static str,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Sweep state for tracking batch jobs
#[derive(Clone, Debug)]
pub struct SweepState {
    /// job store
    pub job_store: Arc<JobStore>,
    /// executor config
    pub executor_config: ExecutorConfig,
}

/// Submit a parameter sweep
/// Per P2-8: Creates multiple simulation jobs from parameter combinations
pub async fn submit_sweep(
    State(state): State<SweepState>,
    Json(request): Json<SweepRequest>,
) -> impl IntoResponse {
    let sweep_id = Uuid::new_v4().to_string();
    let mut job_ids = Vec::new();

    // Generate all parameter combinations
    let runs = generate_sweep_runs(&request);

    if runs.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                            "error": "No runs generated. Provide at least one seed.",
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                            "sweep_id": sweep_id
                        })),
        )
            .into_response();
    }

    // Create jobs for each run
    for (seed, preset, _toggle_set_name) in &runs {
        let job_request = JobRequest {
            agent_count: request.base_config.agent_count,
            max_ticks: request.base_config.max_ticks,
            stage: request.base_config.stage,
            seed: *seed,
            grid_width: request.base_config.grid_width,
            grid_height: request.base_config.grid_height,
            preset: *preset,
        };

        // Use Job::new() which generates UUID internally
        let job = Job::new(job_request);
        job_ids.push(job.id.clone());

        // Queue job in store
        state.job_store.add(job);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    (
        StatusCode::ACCEPTED,
        Json(SweepResponse {
            sweep_id,
            total_runs: job_ids.len(),
            job_ids,
            status: "queued",
        }),
    )
        .into_response()
}

/// Generate all runs from sweep parameters
fn generate_sweep_runs(request: &SweepRequest) -> Vec<(u64, ParameterPreset, String)> {
    let mut runs = Vec::new();

    let presets = if request.sweep_params.presets.is_empty() {
        vec![ParameterPreset::Default]
    } else {
        request.sweep_params.presets.clone()
    };

    for seed in &request.sweep_params.seeds {
        for preset in &presets {
            runs.push((*seed, *preset, "default".to_string()));
        }
    }

    runs
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Sweep status response
#[derive(Debug, Clone, Serialize)]
pub struct SweepStatusResponse {
    /// Sweep ID
    pub sweep_id: String,
    /// Total runs
    pub total_runs: usize,
    /// Completed runs
    pub completed: usize,
    /// Failed runs
    pub failed: usize,
    /// In progress runs
    pub in_progress: usize,
    /// Pending runs
    pub pending: usize,
    /// Overall status
    pub status: &'static str,
    /// Job details
    pub jobs: Vec<SweepJobStatus>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Individual job status in a sweep
#[derive(Debug, Clone, Serialize)]
pub struct SweepJobStatus {
    /// Job ID
    pub job_id: String,
    /// Seed used
    pub seed: u64,
    /// Preset used
    pub preset: String,
    /// Current status
    pub status: String,
    /// Error message if failed
    pub error: Option<String>,
}

/// Get sweep status
/// Returns the status of all jobs in a sweep
#[allow(clippy::implicit_hasher)]
pub async fn get_sweep_status(
    State(state): State<SweepState>,
    axum::extract::Path(sweep_id): axum::extract::Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    // Get job IDs from query parameter (comma-separated)
    let job_ids: Vec<String> = params
        .get("jobs")
        .map(|s| s.split(',').map(std::string::ToString::to_string).collect())
        .unwrap_or_default();

    if job_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                            "error": "No job IDs provided. Use ?jobs=id1,id2,id3"
                        })),
        )
            .into_response();
    }

    let mut jobs = Vec::new();
    let mut completed = 0;
    let mut failed = 0;
    let mut in_progress = 0;
    let mut pending = 0;

    for job_id in &job_ids {
        if let Some(job) = state.job_store.get(job_id) {
            let status_str = format!("{:?}", job.status);

            // Count by status
            match job.status {
                crate::jobs::JobStatus::Complete => completed += 1,
                crate::jobs::JobStatus::Failed => failed += 1,
                crate::jobs::JobStatus::Running => in_progress += 1,
                _ => pending += 1,
            }

            jobs.push(SweepJobStatus {
                job_id: job_id.clone(),
                seed: job.request.seed,
                preset: format!("{:?}", job.request.preset),
                status: status_str,
                error: job.error.clone(),
            });
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let total = jobs.len();
    let status = if failed > 0 {
        "completed_with_errors"
    } else if completed == total {
        "completed"
    } else if in_progress > 0 {
        "running"
    } else {
        "queued"
    };

    (
        StatusCode::OK,
        Json(SweepStatusResponse {
            sweep_id,
            total_runs: total,
            completed,
            failed,
            in_progress,
            pending,
            status,
            jobs,
        }),
    )
        .into_response()
}
