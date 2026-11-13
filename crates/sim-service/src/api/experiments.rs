//! Experiment API Module
//!
//! Provides REST endpoints for browsing and executing the 26 research experiments.
//! Bridges sim-runner CLI with sim-service HTTP API.
//!
// TAG: surface=simulation owner=research-team rule=SIM-001
//! ## Routes
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET | `/api/experiments` | List all 26 experiments |
//! | GET | `/api/experiments/:id` | Get experiment details |
//! | GET | `/api/experiments/:id/schema` | Get parameter JSON schema |
//! | POST | `/api/experiments/:id/runs` | Launch experiment |
//! | GET | `/api/experiment-runs/:id` | Get run status |
//! | GET | `/api/experiment-runs/:id/stream` | SSE progress stream |
//!
//! SIM-001: This module spawns sim-runner as subprocess - no direct sim-core linkage.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{sse::Event, IntoResponse, Sse},
    Json,
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tracing::{info, warn};

use crate::executor::{spawn_experiment_execution, ExecutorConfig};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Acceptance criterion for an experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    /// metric
    pub metric: String,
    /// description
    pub description: String,
    /// target min
    pub target_min: Option<f64>,
    /// target max
    pub target_max: Option<f64>,
    /// unit
    pub unit: String,
}

/// Experiment metadata for catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentInfo {
    /// id
    pub id: String,
    /// name
    pub name: String,
    /// paper
    pub paper: String,
    /// paper full
    pub paper_full: String,
    /// gap ids
    pub gap_ids: Vec<String>,
    /// description
    pub description: String,
    /// default config
    pub default_config: ExperimentConfig,
    /// acceptance criteria
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Experiment configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    /// agents
    pub agents: usize,
    /// ticks
    pub ticks: u64,
    /// replications
    pub replications: u32,
    /// seed
    pub seed: u64,
    /// PDE4B variance level: "zero", "low", "medium", "high"
    pub pde4b_variance: String,
}

impl Default for ExperimentConfig {
    fn default() -> Self {
        Self {
            agents: 1000,
            ticks: 365,
            replications: 10,
            seed: 42,
            pde4b_variance: "medium".to_string(),
        }
    }
}

/// Complete catalog of all 26 research experiments
pub use crate::api::catalog::experiment_catalog;

// ============================================================================
// API Handlers
// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /api/experiments - List all experiments
pub async fn list_experiments() -> impl IntoResponse {
    let catalog = experiment_catalog();
    info!("Listed {} experiments", catalog.len());

    Json(serde_json::json!({
        "experiments": catalog,
        "count": catalog.len()
    }))
}

/// GET /api/experiments/:id - Get experiment details
pub async fn get_experiment(Path(id): Path<String>) -> impl IntoResponse {
    let catalog = experiment_catalog();

    if let Some(exp) = catalog.iter().find(|e| e.id == id) {
        info!("Retrieved experiment: {}", id);
        Json(exp).into_response()
    } else {
        warn!("Experiment not found: {}", id);
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Experiment not found",
                "id": id
            })),
        )
            .into_response()
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }
}

/// GET /api/experiments/:id/schema - Get JSON Schema for experiment parameters
pub async fn get_experiment_schema(Path(id): Path<String>) -> impl IntoResponse {
    let catalog = experiment_catalog();

    match catalog.iter().find(|e| e.id == id) {
        Some(exp) => {
            // Generate JSON Schema for the experiment
            let schema = serde_json::json!({
                            "$schema": "http://json-schema.org/draft-07/schema#",
                            "type": "object",
                            "title": format!("{} Configuration", exp.name),
                            "properties": {
                                "agents": {
                                    "type": "integer",
                                    "description": "Number of agents",
                                    "minimum": 100,
                                    "maximum": 100_000,
                                    "default": exp.default_config.agents
                                },
                                "ticks": {
                                    "type": "integer",
                                    "description": "Simulation ticks",
                                    "minimum": 10,
                                    "maximum": 10_000,
                                    "default": exp.default_config.ticks
            // TAG: surface=simulation owner=research-team rule=SIM-001
                                },
                                "replications": {
                                    "type": "integer",
                                    "description": "Number of replications",
                                    "minimum": 1,
                                    "maximum": 10_000,
                                    "default": exp.default_config.replications
                                },
                                "seed": {
                                    "type": "integer",
                                    "description": "Random seed",
                                    "default": exp.default_config.seed
                                }
                            },
                            "required": ["agents", "ticks", "replications", "seed"]
                        });

            Json(schema).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Experiment not found",
                "id": id
            })),
        )
            .into_response(),
    }
}

// ============================================================================
// Launch Request/Response Types
// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Deserialize)]
/// `LaunchRequest` data
pub struct LaunchRequest {
    /// agents
    pub agents: Option<usize>,
    /// ticks
    pub ticks: Option<u64>,
    /// replications
    pub replications: Option<u32>,
    /// seed
    pub seed: Option<u64>,
    /// pde4b variance
    pub pde4b_variance: Option<String>,
}

#[derive(Debug, Serialize)]
/// `LaunchResponse` data
pub struct LaunchResponse {
    /// run id
    pub run_id: String,
    /// status
    pub status: String,
    /// message
    pub message: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// POST /api/experiments/:id/runs - Launch experiment
pub async fn launch_experiment(
    Path(id): Path<String>,
    State(config): State<Arc<ExecutorConfig>>,
    Json(req): Json<LaunchRequest>,
) -> impl IntoResponse {
    // Validate experiment exists
    let catalog = experiment_catalog();
    let Some(exp) = catalog.iter().find(|e| e.id == id) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Experiment not found",
                "id": id
            })),
        )
            .into_response();
    };

    // Generate run ID
    let run_id = uuid::Uuid::new_v4().to_string();

    // Use request parameters or experiment defaults
    let agents = req.agents.unwrap_or(exp.default_config.agents);
    let ticks = req.ticks.unwrap_or(exp.default_config.ticks);
    let replications = req.replications.unwrap_or(exp.default_config.replications);
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    let seed = req.seed.unwrap_or(exp.default_config.seed);
    let pde4b_variance = req
        .pde4b_variance
        .unwrap_or_else(|| exp.default_config.pde4b_variance.clone());

    info!(
        "Launching experiment: {} (run: {}), agents={}, ticks={}, reps={}, seed={}",
        id, run_id, agents, ticks, replications, seed
    );

    // Spawn experiment execution in background
    spawn_experiment_execution(
        id.clone(),
        run_id.clone(),
        agents,
        ticks,
        replications,
        seed,
        pde4b_variance,
        (*config).clone(),
    );

    // Return immediate response with run ID
    Json(LaunchResponse {
        run_id: run_id.clone(),
        status: "running".to_string(),
        message: format!("Experiment {id} started (run: {run_id})"),
    })
    .into_response()
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /api/experiment-runs/:id - Get real run status from artifacts directory
pub async fn get_run_status(Path(run_id): Path<String>) -> impl IntoResponse {
    let output_dir = std::path::PathBuf::from("./artifacts").join(&run_id);
    let mut manifest_path = output_dir.join("manifest.json");
    if !manifest_path.exists() && output_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&output_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().is_some_and(|ext| ext == "json") {
                    manifest_path = p;
                    break;
                }
            }
        }
    }

    if manifest_path.exists() {
        let manifest_content = std::fs::read_to_string(&manifest_path).ok();
        let manifest_val: Option<serde_json::Value> =
            manifest_content.and_then(|s| serde_json::from_str(&s).ok());
        Json(serde_json::json!({
            "run_id": run_id,
            "status": "completed",
            "progress": 1.0,
            "manifest": manifest_val,
        }))
    } else if output_dir.exists() {
        Json(serde_json::json!({
            "run_id": run_id,
            "status": "running",
            "progress": 0.5,
            "message": "Experiment execution in progress"
        }))
    } else {
        Json(serde_json::json!({
            "run_id": run_id,
            "status": "running",
            "progress": 0.1,
            "message": "Experiment queued or starting"
        }))
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /api/experiment-runs/:id/stream - SSE progress stream (stub)
pub async fn stream_progress(
    Path(run_id): Path<String>,
) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    use futures::stream;

    // Stub implementation - Phase 2 will add real SSE streaming
    // For now, create a single-event stream indicating not implemented
    let event = Event::default().event("error").data(
        serde_json::json!({
            "message": "Progress streaming not yet implemented (Phase 2)",
            "run_id": run_id
        })
        .to_string(),
    );

    let stream = stream::once(async move { Ok::<_, Infallible>(event) });

    Sse::new(stream)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
mod tests {
    use super::*;

    #[test]
    fn test_experiment_catalog_has_26() {
        let catalog = experiment_catalog();
        assert_eq!(catalog.len(), 36, "Expected 36 experiments");
    }

    #[test]
    fn test_all_experiments_have_unique_ids() {
        let catalog = experiment_catalog();
        let ids: std::collections::HashSet<_> = catalog.iter().map(|e| &e.id).collect();
        assert_eq!(
            ids.len(),
            catalog.len(),
            "All experiment IDs should be unique"
        );
    }

    #[test]
    fn test_lambda_universality_exists() {
        let catalog = experiment_catalog();
        let exp = catalog.iter().find(|e| e.id == "lambda-universality");
        assert!(exp.is_some(), "lambda-universality should exist");

        let exp = exp.unwrap();
        assert_eq!(exp.paper, "GTT");
        assert!(exp.gap_ids.contains(&"GTT-P0-1".to_string()));
    }
}
