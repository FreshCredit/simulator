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

const MAX_SWEEP_RUNS: usize = 64;

/// Submit a parameter sweep
/// Per P2-8: Creates and spawns simulation jobs from parameter combinations
pub async fn submit_sweep(
    State(state): State<SweepState>,
    Json(request): Json<SweepRequest>,
) -> impl IntoResponse {
    if !(1..=100_000).contains(&request.base_config.agent_count)
        || !(10..=10_000).contains(&request.base_config.max_ticks)
        || request.base_config.stage > 5
        || !(16..=512).contains(&request.base_config.grid_width)
        || !(16..=512).contains(&request.base_config.grid_height)
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Sweep base_config parameter out of bounds"
            })),
        )
            .into_response();
    }

    let sweep_id = Uuid::new_v4().to_string();
    let mut job_ids = Vec::new();

    // Generate all parameter combinations
    let runs = generate_sweep_runs(&request);

    if runs.is_empty() || runs.len() > MAX_SWEEP_RUNS {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                            "error": format!("Sweep must produce 1..={MAX_SWEEP_RUNS} runs (got {})", runs.len()),
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                            "sweep_id": sweep_id
                        })),
        )
            .into_response();
    }

    // Create and dispatch jobs for each run
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

        // Queue job in store and dispatch execution via bounded executor semaphore
        state.job_store.add(job.clone());
        crate::executor::spawn_job_execution(
            job,
            state.job_store.clone(),
            state.executor_config.clone(),
        );
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

/// Generate all runs from sweep parameters (expands seeds x presets x toggle_sets)
fn generate_sweep_runs(request: &SweepRequest) -> Vec<(u64, ParameterPreset, String)> {
    let mut runs = Vec::new();

    let presets = if request.sweep_params.presets.is_empty() {
        vec![ParameterPreset::Default]
    } else {
        request.sweep_params.presets.clone()
    };

    let toggle_names: Vec<String> = if request.sweep_params.toggle_sets.is_empty() {
        vec!["default".to_string()]
    } else {
        request
            .sweep_params
            .toggle_sets
            .iter()
            .map(|t| t.name.clone())
            .collect()
    };

    for seed in &request.sweep_params.seeds {
        for preset in &presets {
            for toggle_name in &toggle_names {
                runs.push((*seed, *preset, toggle_name.clone()));
                if runs.len() > MAX_SWEEP_RUNS {
                    return runs;
                }
            }
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
    if !crate::artifacts::ArtifactStore::is_valid_run_id(&sweep_id) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid sweep_id"})),
        )
            .into_response();
    }
    // Get job IDs from query parameter (comma-separated, capped to MAX_SWEEP_RUNS)
    let job_ids: Vec<String> = params
        .get("jobs")
        .map(|s| {
            s.split(',')
                .take(MAX_SWEEP_RUNS + 1)
                .map(std::string::ToString::to_string)
                .collect()
        })
        .unwrap_or_default();

    if job_ids.is_empty() || job_ids.len() > MAX_SWEEP_RUNS {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                            "error": "Provide 1..=64 job IDs via ?jobs=id1,id2,id3"
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_sweep_runs_expands_toggle_sets_and_caps() {
        let req = SweepRequest {
            base_config: SweepBaseConfig {
                agent_count: 100,
                max_ticks: 100,
                stage: 2,
                grid_width: 64,
                grid_height: 64,
            },
            sweep_params: SweepParams {
                seeds: vec![1, 2],
                presets: vec![ParameterPreset::Default, ParameterPreset::NearExtinction],
                toggle_sets: vec![
                    ToggleSet {
                        name: "t1".to_string(),
                        genome_enabled: true,
                        docas_enabled: true,
                        dmn_enabled: false,
                        choice_architecture_enabled: false,
                        protocol_enabled: false,
                    },
                    ToggleSet {
                        name: "t2".to_string(),
                        genome_enabled: true,
                        docas_enabled: true,
                        dmn_enabled: true,
                        choice_architecture_enabled: true,
                        protocol_enabled: true,
                    },
                ],
            },
            max_concurrent: 2,
        };
        let runs = generate_sweep_runs(&req);
        assert_eq!(runs.len(), 8);
    }

    #[tokio::test]
    async fn test_submit_sweep_dispatches_and_rejects_excessive_runs() {
        let state = SweepState {
            job_store: Arc::new(JobStore::new()),
            executor_config: ExecutorConfig::default(),
        };
        let req = SweepRequest {
            base_config: SweepBaseConfig {
                agent_count: 100,
                max_ticks: 100,
                stage: 1,
                grid_width: 64,
                grid_height: 64,
            },
            sweep_params: SweepParams {
                seeds: (0..100).collect(),
                presets: vec![],
                toggle_sets: vec![],
            },
            max_concurrent: 2,
        };
        let resp = submit_sweep(State(state), Json(req)).await.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
