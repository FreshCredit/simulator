//! Artifact serving module
//!
//! Serves simulation artifacts (manifests, metrics, replay frames) from
//! local filesystem or GCS bucket.
//!
//! SIM-001: Only uses sim-artifacts for schema types.
//! SIM-002: Read-only access to artifacts.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::task;
use tracing::{info, warn};

use sim_artifacts::{Manifest, ReplayReader};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Artifact store configuration
#[derive(Debug, Clone)]
pub struct ArtifactStore {
    /// Base directory for artifact storage
    pub base_dir: PathBuf,
}

impl ArtifactStore {
    /// Create a new artifact store
    #[must_use]
    pub const fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Get the path to a run's directory
    #[must_use]
    pub fn run_dir(&self, run_id: &str) -> PathBuf {
        self.base_dir.join(run_id)
    }

    /// Get the path to a run's manifest
    #[must_use]
    pub fn manifest_path(&self, run_id: &str) -> PathBuf {
        self.run_dir(run_id).join("manifest.json")
    }

    /// Get the path to a run's replay directory
    #[must_use]
    pub fn replay_dir(&self, run_id: &str) -> PathBuf {
        self.run_dir(run_id).join("replay")
    }

    /// Get the path to a specific replay tick file
    #[must_use]
    pub fn replay_tick_path(&self, run_id: &str, tick: u64) -> PathBuf {
        self.replay_dir(run_id).join(format!("{tick:06}.jsonl"))
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get the path to a run's metrics file
    #[must_use]
    pub fn metrics_path(&self, run_id: &str) -> PathBuf {
        self.run_dir(run_id).join("metrics.jsonl")
    }

    /// Load manifest for a specific run
    pub async fn load_manifest(&self, run_id: &str) -> Option<Manifest> {
        let manifest_path = self.manifest_path(run_id);

        // P0-PERF: Use spawn_blocking for blocking file I/O
        let path_clone = manifest_path.clone();
        task::spawn_blocking(move || {
            if !path_clone.exists() {
                return None;
            }
            match std::fs::read_to_string(&path_clone) {
                Ok(contents) => Manifest::from_json(&contents).ok(),
                Err(_) => None,
            }
        })
        .await
        .ok()?
    }

    /// Load metrics for a specific run
    pub async fn load_metrics(&self, run_id: &str) -> Option<Vec<sim_artifacts::TickMetrics>> {
        let metrics_path = self.metrics_path(run_id);

        // P0-PERF: Use spawn_blocking for blocking file I/O
        let path_clone = metrics_path.clone();
        task::spawn_blocking(move || {
            if !path_clone.exists() {
                return None;
            }
            match std::fs::read_to_string(&path_clone) {
                Ok(contents) => contents
                    .lines()
                    .filter(|line| !line.is_empty())
                    .filter_map(|line| serde_json::from_str(line).ok())
                    .collect::<Vec<sim_artifacts::TickMetrics>>()
                    .into(),
                Err(_) => None,
            }
        })
        .await
        .ok()?
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// List all available runs
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn list_runs(&self) -> std::io::Result<Vec<RunSummary>> {
        let base_dir = self.base_dir.clone();

        // P0-PERF: Use spawn_blocking for blocking directory I/O
        task::spawn_blocking(move || {
            let mut runs = Vec::new();

            if !base_dir.exists() {
                return Ok(runs);
            }

            for entry in std::fs::read_dir(&base_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    let manifest_path = path.join("manifest.json");
                    if manifest_path.exists() {
                        if let Ok(contents) = std::fs::read_to_string(&manifest_path) {
                            if let Ok(manifest) = Manifest::from_json(&contents) {
                                runs.push(RunSummary {
                                    run_id: manifest.run_id.clone(),
                                    timestamp: manifest.timestamp.to_rfc3339(),
                                    status: format!("{:?}", manifest.status),
                                    agent_count: manifest.config.agent_count,
                                    max_ticks: manifest.config.max_ticks,
                                    stage: manifest.config.stage.clone(),
                                });
                            }
                        }
                    }
                }
            }

            // Sort by timestamp (newest first)
            runs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            Ok(runs)
        })
        .await
        .map_err(std::io::Error::other)?
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Summary of a simulation run for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummary {
    /// run id
    pub run_id: String,
    /// timestamp
    pub timestamp: String,
    /// status
    pub status: String,
    /// agent count
    pub agent_count: u32,
    /// max ticks
    pub max_ticks: u64,
    /// stage
    pub stage: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// List all available simulation runs
pub async fn list_runs(State(store): State<Arc<ArtifactStore>>) -> impl IntoResponse {
    match store.list_runs().await {
        Ok(runs) => {
            info!("Listed {} runs", runs.len());
            Json(serde_json::json!({
                "runs": runs,
                "count": runs.len()
            }))
            .into_response()
        }
        Err(e) => {
            warn!("Failed to list runs: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to list runs: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// Get manifest for a specific run
pub async fn get_manifest(
    State(store): State<Arc<ArtifactStore>>,
    Path(run_id): Path<String>,
) -> impl IntoResponse {
    let manifest_path = store.manifest_path(&run_id);

    // P0-PERF: Use spawn_blocking for blocking file I/O
    if let Ok(Ok(contents)) =
        task::spawn_blocking(move || std::fs::read_to_string(&manifest_path)).await
    {
        info!("Serving manifest for run {}", run_id);
        (StatusCode::OK, contents).into_response()
    } else {
        warn!("Manifest not found for run {}", run_id);
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("Run not found: {}", run_id)
            })),
        )
            .into_response()
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get replay frame for a specific tick
pub async fn get_replay_frame(
    State(store): State<Arc<ArtifactStore>>,
    Path((run_id, tick)): Path<(String, u64)>,
) -> impl IntoResponse {
    let replay_path = store.replay_tick_path(&run_id, tick);

    match ReplayReader::open(&replay_path) {
        Ok(mut reader) => match reader.read_all() {
            Ok(frames) => {
                info!(
                    "Serving {} frames for run {} tick {}",
                    frames.len(),
                    run_id,
                    tick
                );
                Json(serde_json::json!({
                    "tick": tick,
                    "frames": frames
                }))
                .into_response()
            }
            Err(e) => {
                warn!(
                    "Failed to read replay for run {} tick {}: {}",
                    run_id, tick, e
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": format!("Failed to read replay: {}", e)
                    })),
                )
                    .into_response()
            }
        },
        Err(e) => {
            warn!("Replay not found for run {} tick {}: {}", run_id, tick, e);
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": format!("Replay frame not found: tick {}", tick)
                })),
            )
                .into_response()
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get metrics for a specific run
pub async fn get_metrics(
    State(store): State<Arc<ArtifactStore>>,
    Path(run_id): Path<String>,
) -> impl IntoResponse {
    let metrics_path = store.metrics_path(&run_id);

    // P0-PERF: Use spawn_blocking for blocking file I/O
    if let Ok(Ok(contents)) =
        task::spawn_blocking(move || std::fs::read_to_string(&metrics_path)).await
    {
        // Parse JSONL and return as JSON array
        let metrics: Vec<serde_json::Value> = contents
            .lines()
            .filter(|line| !line.is_empty())
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();

        info!(
            "Serving {} metrics records for run {}",
            metrics.len(),
            run_id
        );
        Json(serde_json::json!({
            "run_id": run_id,
            "count": metrics.len(),
            "metrics": metrics
        }))
        .into_response()
    } else {
        warn!("Metrics not found for run {}", run_id);
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("Metrics not found for run: {}", run_id)
            })),
        )
            .into_response()
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_artifact_store_paths() {
        let dir = tempdir().unwrap();
        let store = ArtifactStore::new(dir.path().to_path_buf());

        let run_id = "test-run-123";
        assert!(store.run_dir(run_id).ends_with("test-run-123"));
        assert!(store.manifest_path(run_id).ends_with("manifest.json"));
        assert!(store.replay_dir(run_id).ends_with("replay"));
        assert!(store
            .replay_tick_path(run_id, 100)
            .ends_with("000100.jsonl"));
    }

    #[tokio::test]
    async fn test_list_runs_empty() {
        let dir = tempdir().unwrap();
        let store = ArtifactStore::new(dir.path().to_path_buf());
        let runs = store.list_runs().await.unwrap();
        assert!(runs.is_empty());
    }
}
