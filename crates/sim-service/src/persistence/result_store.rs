//! Result Store with Versioning Support (P1-SIM-2)
//!
//! Provides in-memory storage for experiment results with versioning,
//! filtering, and export capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Errors that can occur in the result store
#[derive(Debug, thiserror::Error)]
pub enum ResultStoreError {
    #[error("Result not found: {0}")]
    /// `NotFound`
    NotFound(String),
    #[error("Version conflict: {0}")]
    /// `VersionConflict`
    VersionConflict(String),
    #[error("Invalid data: {0}")]
    /// `InvalidData`
    InvalidData(String),
    #[error("Export error: {0}")]
    /// `ExportError`
    ExportError(String),
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Status of an experiment result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResultStatus {
    /// Running
    Running,
    /// Completed
    Completed,
    /// Failed
    Failed,
    /// Cancelled
    Cancelled,
}

/// A stored experiment result with full metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredResult {
    /// id
    pub id: String,
    /// experiment id
    pub experiment_id: String,
    /// experiment name
    pub experiment_name: String,
    /// run id
    pub run_id: String,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// status
    pub status: ResultStatus,
    /// version
    pub version: i32,
    /// config
    pub config: serde_json::Value,
    /// metrics
    pub metrics: serde_json::Value,
    /// artifacts
    pub artifacts: Vec<String>,
    /// figures
    pub figures: Vec<String>,
    /// created at
    pub created_at: String,
    /// created by
    pub created_by: String,
    /// duration secs
    pub duration_secs: f64,
    /// tags
    pub tags: Vec<String>,
    /// description
    pub description: Option<String>,
    /// is deleted
    pub is_deleted: bool,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Version history entry
#[derive(Debug, Clone, Serialize)]
pub struct ResultVersion {
    /// version
    pub version: i32,
    /// created at
    pub created_at: String,
    /// created by
    pub created_by: String,
    /// changes
    pub changes: String,
    /// config diff
    pub config_diff: Option<serde_json::Value>,
    /// metrics diff
    pub metrics_diff: Option<serde_json::Value>,
}

/// Query filters for listing results
#[derive(Debug, Clone, Default)]
pub struct ResultQuery {
    /// experiment id
    pub experiment_id: Option<String>,
    /// status
    pub status: Option<ResultStatus>,
    /// created by
    pub created_by: Option<String>,
    /// tags
    pub tags: Option<Vec<String>>,
    /// limit
    pub limit: usize,
    /// offset
    pub offset: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// In-memory result store with versioning
#[derive(Debug, Clone)]
pub struct ResultStore {
    /// Main storage: `result_id` -> `StoredResult`
    results: Arc<RwLock<HashMap<String, StoredResult>>>,
    /// Version history: `result_id` -> Vec of versions (oldest first)
    versions: Arc<RwLock<HashMap<String, Vec<ResultVersion>>>>,
    /// Index by `experiment_id` for faster lookups
    experiment_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Index by `run_id` for lookups
    run_index: Arc<RwLock<HashMap<String, String>>>,
}

impl Default for ResultStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ResultStore {
    /// Create a new empty result store
    #[must_use]
    pub fn new() -> Self {
        Self {
            results: Arc::new(RwLock::new(HashMap::new())),
            versions: Arc::new(RwLock::new(HashMap::new())),
            experiment_index: Arc::new(RwLock::new(HashMap::new())),
            run_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Store a new experiment result
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn store(&self, result: StoredResult) -> Result<String, ResultStoreError> {
        let id = result.id.clone();
        let experiment_id = result.experiment_id.clone();
        let run_id = result.run_id.clone();

        // Check for existing result with same run_id (idempotency)
        let run_index = self.run_index.read().await;
        if let Some(existing_id) = run_index.get(&run_id) {
            warn!(
                run_id = %run_id,
                existing_id = %existing_id,
                "Result for this run already exists, returning existing ID"
            );
            return Ok(existing_id.clone());
        }
        drop(run_index);

        // Store the result
        let mut results = self.results.write().await;
        results.insert(id.clone(), result);
        drop(results);

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Update indexes
        let mut experiment_index = self.experiment_index.write().await;
        experiment_index
            .entry(experiment_id)
            .or_default()
            .push(id.clone());
        drop(experiment_index);

        let mut run_index = self.run_index.write().await;
        run_index.insert(run_id, id.clone());
        drop(run_index);

        // Initialize version history
        let mut versions = self.versions.write().await;
        let version_entry = ResultVersion {
            version: 1,
            created_at: chrono::Utc::now().to_rfc3339(),
            created_by: "system".to_string(),
            changes: "Initial creation".to_string(),
            config_diff: None,
            metrics_diff: None,
        };
        versions.insert(id.clone(), vec![version_entry]);
        drop(versions);

        info!(result_id = %id, "Result stored successfully");
        Ok(id)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get a result by ID
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn get(&self, id: &str) -> Result<StoredResult, ResultStoreError> {
        let results = self.results.read().await;
        results
            .get(id)
            .filter(|r| !r.is_deleted)
            .cloned()
            .ok_or_else(|| ResultStoreError::NotFound(id.to_string()))
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get a result by `run_id`
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn get_by_run_id(&self, run_id: &str) -> Result<StoredResult, ResultStoreError> {
        let run_index = self.run_index.read().await;
        let id = run_index
            .get(run_id)
            .cloned()
            .ok_or_else(|| ResultStoreError::NotFound(format!("run_id: {run_id}")))?;
        drop(run_index);

        self.get(&id).await
    }

    /// List results with optional filtering
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn list(&self, query: ResultQuery) -> (Vec<StoredResult>, usize) {
        let results = self.results.read().await;
        let experiment_index = self.experiment_index.read().await;

        // Determine which IDs to consider
        let candidate_ids: Vec<String> = if let Some(exp_id) = &query.experiment_id {
            experiment_index.get(exp_id).cloned().unwrap_or_default()
        } else {
            results.keys().cloned().collect()
        };

        // Apply filters
        let mut filtered: Vec<StoredResult> = candidate_ids
            .into_iter()
            .filter_map(|id| results.get(&id))
            .filter(|r| !r.is_deleted)
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            .filter(|r| {
                if let Some(ref status) = query.status {
                    r.status == *status
                } else {
                    true
                }
            })
            .filter(|r| {
                if let Some(ref created_by) = query.created_by {
                    r.created_by == *created_by
                } else {
                    true
                }
            })
            .filter(|r| {
                if let Some(ref tags) = query.tags {
                    tags.iter().all(|tag| r.tags.contains(tag))
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        let total = filtered.len();

        // Sort by created_at descending
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Apply pagination
        let paginated = filtered
            .into_iter()
            .skip(query.offset)
            .take(query.limit)
            .collect();

        (paginated, total)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create a new version of an existing result
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn create_version(
        &self,
        id: &str,
        updates: VersionUpdate,
    ) -> Result<StoredResult, ResultStoreError> {
        let mut results = self.results.write().await;

        let result = results
            .get_mut(id)
            .filter(|r| !r.is_deleted)
            .ok_or_else(|| ResultStoreError::NotFound(id.to_string()))?;

        // Record version history
        let mut versions = self.versions.write().await;
        let history = versions.entry(id.to_string()).or_default();

        let old_version = result.version;
        let new_version = old_version + 1;

        // Calculate diffs
        let config_diff = updates
            .config
            .as_ref()
            .map(|new| json_diff(&result.config, new));
        let metrics_diff = updates
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            .metrics
            .as_ref()
            .map(|new| json_diff(&result.metrics, new));

        // Add to history
        history.push(ResultVersion {
            version: new_version,
            created_at: chrono::Utc::now().to_rfc3339(),
            created_by: updates.updated_by.clone(),
            changes: updates.changes.clone(),
            config_diff,
            metrics_diff,
        });

        // Apply updates
        if let Some(config) = updates.config {
            result.config = config;
        }
        if let Some(metrics) = updates.metrics {
            result.metrics = metrics;
        }
        if let Some(artifacts) = updates.artifacts {
            result.artifacts = artifacts;
        }
        if let Some(figures) = updates.figures {
            result.figures = figures;
        }
        if let Some(tags) = updates.tags {
            result.tags = tags;
        }
        if let Some(description) = updates.description {
            result.description = Some(description);
        }
        result.version = new_version;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let updated = result.clone();
        drop(results);
        drop(versions);

        info!(
            result_id = %id,
            old_version = old_version,
            new_version = new_version,
            "Result version created"
        );

        Ok(updated)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get version history for a result
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn get_versions(&self, id: &str) -> Result<Vec<ResultVersion>, ResultStoreError> {
        let results = self.results.read().await;
        if !results.contains_key(id) {
            return Err(ResultStoreError::NotFound(id.to_string()));
        }
        drop(results);

        let versions = self.versions.read().await;
        Ok(versions.get(id).cloned().unwrap_or_default())
    }

    /// Soft delete a result
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn delete(&self, id: &str) -> Result<(), ResultStoreError> {
        let mut results = self.results.write().await;

        let result = results
            .get_mut(id)
            .ok_or_else(|| ResultStoreError::NotFound(id.to_string()))?;

        result.is_deleted = true;
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        drop(results);

        info!(result_id = %id, "Result soft-deleted");
        Ok(())
    }

    /// Permanently delete a result (use with caution)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn hard_delete(&self, id: &str) -> Result<(), ResultStoreError> {
        let mut results = self.results.write().await;
        let result = results
            .remove(id)
            .ok_or_else(|| ResultStoreError::NotFound(id.to_string()))?;
        drop(results);

        // Clean up indexes
        let mut experiment_index = self.experiment_index.write().await;
        if let Some(ids) = experiment_index.get_mut(&result.experiment_id) {
            ids.retain(|x| x != id);
        }
        drop(experiment_index);

        let mut run_index = self.run_index.write().await;
        run_index.remove(&result.run_id);
        drop(run_index);

        let mut versions = self.versions.write().await;
        versions.remove(id);
        drop(versions);

        info!(result_id = %id, "Result permanently deleted");
        Ok(())
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Export a result to JSON string
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn export_json(&self, id: &str) -> Result<String, ResultStoreError> {
        let result = self.get(id).await?;
        serde_json::to_string_pretty(&result)
            .map_err(|e| ResultStoreError::ExportError(e.to_string()))
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Export a result to CSV format (metrics only)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn export_csv(&self, id: &str) -> Result<String, ResultStoreError> {
        let result = self.get(id).await?;

        let mut csv = String::new();
        csv.push_str("result_id,experiment_id,run_id,version,status,created_at,duration_secs\n");
        csv.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            result.id,
            result.experiment_id,
            result.run_id,
            result.version,
            serde_json::to_string(&result.status).unwrap_or_default(),
            result.created_at,
            result.duration_secs
        ));

        // Add metrics as additional rows
        if let Some(metrics) = result.metrics.as_object() {
            csv.push_str("\nmetric_name,metric_value\n");
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            for (key, value) in metrics {
                let value_str = match value {
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::String(s) => s.clone(),
                    _ => value.to_string(),
                };
                csv.push_str(&format!("{key},{value_str}\n"));
            }
        }

        Ok(csv)
    }

    /// Get store statistics
    pub async fn stats(&self) -> StoreStats {
        let results = self.results.read().await;
        let active_count = results.values().filter(|r| !r.is_deleted).count();
        let deleted_count = results.values().filter(|r| r.is_deleted).count();

        StoreStats {
            total_results: results.len(),
            active_results: active_count,
            deleted_results: deleted_count,
            total_versions: self
                .versions
                .read()
                .await
                .values()
                .map(std::vec::Vec::len)
                .sum(),
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Update parameters for creating a new version
#[derive(Debug, Clone, Default)]
pub struct VersionUpdate {
    /// config
    pub config: Option<serde_json::Value>,
    /// metrics
    pub metrics: Option<serde_json::Value>,
    /// artifacts
    pub artifacts: Option<Vec<String>>,
    /// figures
    pub figures: Option<Vec<String>>,
    /// tags
    pub tags: Option<Vec<String>>,
    /// description
    pub description: Option<String>,
    /// changes
    pub changes: String,
    /// updated by
    pub updated_by: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Store statistics
#[derive(Debug, Clone, Serialize)]
pub struct StoreStats {
    /// total results
    pub total_results: usize,
    /// active results
    pub active_results: usize,
    /// deleted results
    pub deleted_results: usize,
    /// total versions
    pub total_versions: usize,
}

/// Calculate a simple JSON diff (returns the new value for changed fields)
fn json_diff(old: &serde_json::Value, new: &serde_json::Value) -> serde_json::Value {
    match (old, new) {
        (serde_json::Value::Object(old_map), serde_json::Value::Object(new_map)) => {
            let mut diff = serde_json::Map::new();
            for (key, new_val) in new_map {
                match old_map.get(key) {
                    Some(old_val) if old_val == new_val => {} // No change
                    _ => {
                        diff.insert(key.clone(), new_val.clone());
                    }
                }
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            }
            serde_json::Value::Object(diff)
        }
        _ => new.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_and_get() {
        let store = ResultStore::new();

        let result = StoredResult {
            id: "test-1".to_string(),
            experiment_id: "exp-1".to_string(),
            experiment_name: "Test Experiment".to_string(),
            run_id: "run-1".to_string(),
            status: ResultStatus::Completed,
            version: 1,
            config: serde_json::json!({"agents": 100}),
            metrics: serde_json::json!({"score": 0.95}),
            artifacts: vec![],
            figures: vec![],
            created_at: chrono::Utc::now().to_rfc3339(),
            created_by: "test".to_string(),
            duration_secs: 60.0,
            tags: vec![],
            description: None,
            is_deleted: false,
        };

        let id = store.store(result.clone()).await.unwrap();
        assert_eq!(id, "test-1");

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let retrieved = store.get(&id).await.unwrap();
        assert_eq!(retrieved.experiment_id, "exp-1");
    }

    #[tokio::test]
    async fn test_versioning() {
        let store = ResultStore::new();

        let result = StoredResult {
            id: "test-2".to_string(),
            experiment_id: "exp-2".to_string(),
            experiment_name: "Test".to_string(),
            run_id: "run-2".to_string(),
            status: ResultStatus::Completed,
            version: 1,
            config: serde_json::json!({"agents": 100}),
            metrics: serde_json::json!({"score": 0.9}),
            artifacts: vec![],
            figures: vec![],
            created_at: chrono::Utc::now().to_rfc3339(),
            created_by: "test".to_string(),
            duration_secs: 60.0,
            tags: vec![],
            description: None,
            is_deleted: false,
        };

        store.store(result).await.unwrap();

        // Create new version
        let update = VersionUpdate {
            metrics: Some(serde_json::json!({"score": 0.95})),
            changes: "Updated score".to_string(),
            updated_by: "test_user".to_string(),
            ..Default::default()
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let updated = store.create_version("test-2", update).await.unwrap();
        assert_eq!(updated.version, 2);

        let versions = store.get_versions("test-2").await.unwrap();
        assert_eq!(versions.len(), 2);
    }

    #[tokio::test]
    async fn test_soft_delete() {
        let store = ResultStore::new();

        let result = StoredResult {
            id: "test-3".to_string(),
            experiment_id: "exp-3".to_string(),
            experiment_name: "Test".to_string(),
            run_id: "run-3".to_string(),
            status: ResultStatus::Completed,
            version: 1,
            config: serde_json::json!({}),
            metrics: serde_json::json!({}),
            artifacts: vec![],
            figures: vec![],
            created_at: chrono::Utc::now().to_rfc3339(),
            created_by: "test".to_string(),
            duration_secs: 60.0,
            tags: vec![],
            description: None,
            is_deleted: false,
        };

        store.store(result).await.unwrap();
        store.delete("test-3").await.unwrap();

        // Should not be found after soft delete
        assert!(store.get("test-3").await.is_err());
    }
}
