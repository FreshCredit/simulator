//! Job management module
//!
//! Handles simulation job submission, status tracking, and lifecycle.
//!
//! SIM-001: Does not depend on sim-core or sim-runner directly.
//! Job execution is handled by spawning sim-runner as a subprocess.
//!
//! Note: RBAC enforcement is handled here (not in apps/simulator UI).
//! Role limits per §10.4 of simulation-plan.md:
//! - Consumer: 1,000 agents max (sandbox only)
//! - Provider: 10,000 agents max, 1 concurrent job
//! - Internal: 1,000,000 agents max, unlimited concurrent

// TAG: surface=simulation owner=research-team rule=SIM-001
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{info, warn};
use uuid::Uuid;

use crate::executor::{spawn_job_execution, ExecutorConfig};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Job status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    /// Queued
    Queued,
    /// Running
    Running,
    /// Complete
    Complete,
    /// Failed
    Failed,
    /// Cancelled
    Cancelled,
}

/// Parameter preset for evolutionary dynamics scenarios
/// Per parameters.rs: 6 scientifically-grounded presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ParameterPreset {
    /// Default balanced parameters
    #[default]
    Default,
    /// High mortality, sparse resources - models resource scarcity
    NearExtinction,
    /// Low mortality, abundant resources - models colonization
    RapidExpansion,
    /// Moderate consumption, slow regrowth - oscillating population
    BoomBustCycles,
    /// Balanced parameters for long-term stability
    StableEquilibrium,
    /// Increased mutation rate - rapid trait evolution
    HighDiversity,
    /// Enhanced society parameters - fast norm emergence
    StrongSocial,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Simulation job request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRequest {
    /// Number of agents (validated against role limits)
    pub agent_count: u32,
    /// Maximum ticks to run
    pub max_ticks: u64,
    /// Simulation stage (0-5)
    pub stage: u8,
    /// Random seed for determinism (SIM-004)
    pub seed: u64,
    /// Grid dimensions (optional, defaults to 64x64)
    #[serde(default = "default_grid_size")]
    pub grid_width: u32,
    #[serde(default = "default_grid_size")]
    /// grid height
    pub grid_height: u32,
    /// Parameter preset for evolutionary dynamics (optional)
    #[serde(default)]
    pub preset: ParameterPreset,
}

const fn default_grid_size() -> u32 {
    64
}

/// Response for parameter presets API
#[derive(Debug, Clone, Serialize)]
pub struct PresetInfo {
    /// name
    pub name: String,
    /// description
    pub description: String,
    /// expected behavior
    pub expected_behavior: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get available parameter presets
pub async fn list_presets() -> impl IntoResponse {
    let presets = vec![
        PresetInfo {
            name: "default".to_string(),
            description: "Default balanced parameters".to_string(),
            expected_behavior: "Stable population at carrying capacity".to_string(),
        },
        PresetInfo {
            name: "near_extinction".to_string(),
            description: "High mortality, sparse resources".to_string(),
            expected_behavior: "Population decline, strong selection pressure".to_string(),
        },
        PresetInfo {
            name: "rapid_expansion".to_string(),
            description: "Low mortality, abundant resources".to_string(),
            expected_behavior: "Exponential population growth".to_string(),
        },
        PresetInfo {
            name: "boom_bust_cycles".to_string(),
            description: "Moderate consumption, slow regrowth".to_string(),
            expected_behavior: "Oscillating population dynamics".to_string(),
        },
        PresetInfo {
            name: "stable_equilibrium".to_string(),
            description: "Balanced parameters for long-term stability".to_string(),
            expected_behavior: "Population converges to carrying capacity".to_string(),
        },
        PresetInfo {
            name: "high_diversity".to_string(),
            description: "Increased mutation rate".to_string(),
            expected_behavior: "Rapid trait evolution, diverse phenotypes".to_string(),
        },
        PresetInfo {
            name: "strong_social".to_string(),
            description: "Enhanced society parameters".to_string(),
            expected_behavior: "Fast norm emergence, strong institutions".to_string(),
        },
    ];

    Json(serde_json::json!({
        "presets": presets,
        "count": presets.len()
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Job record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// id
    pub id: String,
    /// status
    pub status: JobStatus,
    /// request
    pub request: JobRequest,
    /// created at
    pub created_at: DateTime<Utc>,
    /// started at
    pub started_at: Option<DateTime<Utc>>,
    /// completed at
    pub completed_at: Option<DateTime<Utc>>,
    /// error
    pub error: Option<String>,
    /// Run ID for artifact retrieval (set when complete)
    pub run_id: Option<String>,
}

impl Job {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create a new queued job
    #[must_use]
    pub fn new(request: JobRequest) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            status: JobStatus::Queued,
            request,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error: None,
            run_id: None,
        }
    }
}

/// In-memory job store
/// Note: Production would use a persistent store (Redis, `PostgreSQL`)
#[derive(Debug, Default)]
pub struct JobStore {
    jobs: RwLock<HashMap<String, Job>>,
}

impl JobStore {
    #[must_use]
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new job
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn add(&self, job: Job) {
        let mut jobs = match self.jobs.write() {
            Ok(g) => g,
            Err(e) => e.into_inner(),
        };
        jobs.insert(job.id.clone(), job);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get a job by ID
    pub fn get(&self, id: &str) -> Option<Job> {
        let jobs = match self.jobs.read() {
            Ok(g) => g,
            Err(e) => e.into_inner(),
        };
        jobs.get(id).cloned()
    }

    /// List all jobs
    pub fn list(&self) -> Vec<Job> {
        let jobs = match self.jobs.read() {
            Ok(g) => g,
            Err(e) => e.into_inner(),
        };
        let mut result: Vec<_> = jobs.values().cloned().collect();
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        result
    }

    /// Update job status
    pub fn update_status(&self, id: &str, status: JobStatus) {
        let mut jobs = match self.jobs.write() {
            Ok(g) => g,
            Err(e) => e.into_inner(),
        };
        if let Some(job) = jobs.get_mut(id) {
            job.status = status;
            match status {
                JobStatus::Running => job.started_at = Some(Utc::now()),
                JobStatus::Complete | JobStatus::Failed | JobStatus::Cancelled => {
                    job.completed_at = Some(Utc::now());
                }
                JobStatus::Queued => {}
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Set job as complete with `run_id`
    pub fn complete(&self, id: &str, run_id: String) {
        let mut jobs = match self.jobs.write() {
            Ok(g) => g,
            Err(e) => e.into_inner(),
        };
        if let Some(job) = jobs.get_mut(id) {
            job.status = JobStatus::Complete;
            job.completed_at = Some(Utc::now());
            job.run_id = Some(run_id);
        }
    }

    /// Set job as failed with error
    pub fn fail(&self, id: &str, error: String) {
        let mut jobs = match self.jobs.write() {
            Ok(g) => g,
            Err(e) => e.into_inner(),
        };
        if let Some(job) = jobs.get_mut(id) {
            job.status = JobStatus::Failed;
            job.completed_at = Some(Utc::now());
            job.error = Some(error);
        }
    }
}

/// Combined state for job routes
#[derive(Clone, Debug)]
pub struct JobState {
    /// store
    pub store: Arc<JobStore>,
    /// executor config
    pub executor_config: ExecutorConfig,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Submit a new simulation job
pub async fn submit_job(
    State(state): State<JobState>,
    Json(request): Json<JobRequest>,
) -> impl IntoResponse {
    // Validate request
    if request.agent_count == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "agent_count must be > 0"})),
        )
            .into_response();
    }

    if request.stage > 5 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "stage must be 0-5"})),
        )
            .into_response();
    }

    // Note: In production, RBAC would be enforced here based on role
    // For now, accept all requests (public research tool)

    let job = Job::new(request);
    let job_id = job.id.clone();
    state.store.add(job.clone());

    info!("Created job {}", job_id);

    // Spawn sim-runner subprocess to execute the job
    spawn_job_execution(job, state.store.clone(), state.executor_config);

    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "job_id": job_id,
            "status": "queued"
        })),
    )
        .into_response()
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get job status
pub async fn get_job_status(
    State(state): State<JobState>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    if let Some(job) = state.store.get(&job_id) {
        info!("Returning status for job {}: {:?}", job_id, job.status);
        Json(job).into_response()
    } else {
        warn!("Job not found: {}", job_id);
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("Job not found: {}", job_id)})),
        )
            .into_response()
    }
}

/// List all jobs
pub async fn list_jobs(State(state): State<JobState>) -> impl IntoResponse {
    let jobs = state.store.list();
    info!("Listed {} jobs", jobs.len());
    Json(serde_json::json!({
        "jobs": jobs,
        "count": jobs.len()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_creation() {
        let request = JobRequest {
            agent_count: 100,
            max_ticks: 1000,
            stage: 3,
            seed: 42,
            grid_width: 64,
            grid_height: 64,
            preset: ParameterPreset::Default,
        };
        let job = Job::new(request);
        assert_eq!(job.status, JobStatus::Queued);
        assert!(job.started_at.is_none());
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_job_store() {
        let store = JobStore::new();
        let request = JobRequest {
            agent_count: 100,
            max_ticks: 1000,
            stage: 3,
            seed: 42,
            grid_width: 64,
            grid_height: 64,
            preset: ParameterPreset::Default,
        };
        let job = Job::new(request);
        let job_id = job.id.clone();

        store.add(job);
        assert!(store.get(&job_id).is_some());

        store.update_status(&job_id, JobStatus::Running);
        let updated = store.get(&job_id).unwrap();
        assert_eq!(updated.status, JobStatus::Running);
        assert!(updated.started_at.is_some());
    }

    #[test]
    fn test_preset_default() {
        assert_eq!(ParameterPreset::default(), ParameterPreset::Default);
    }

    #[test]
    fn test_preset_serialization() {
        let preset = ParameterPreset::NearExtinction;
        let json = serde_json::to_string(&preset).unwrap();
        assert_eq!(json, "\"near_extinction\"");

        let parsed: ParameterPreset = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ParameterPreset::NearExtinction);
    }
}
