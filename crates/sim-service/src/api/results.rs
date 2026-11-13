//! Experiment Results API Module (P1-SIM-2)
//!
//! Provides REST endpoints for experiment result persistence with versioning.
//!
//! ## Routes
// TAG: surface=simulation owner=research-team rule=SIM-001
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | POST | `/api/results` | Store experiment result |
//! | GET | `/api/results` | List results with filters |
//! | GET | `/api/results/:id` | Get specific result |
//! | DELETE | `/api/results/:id` | Soft delete result |
//! | GET | `/api/results/:id/versions` | Get version history |
//! | POST | `/api/results/:id/versions` | Create new version |
//! | GET | `/api/results/export/:id` | Export result (CSV/JSON/Parquet) |

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

use crate::persistence::result_store::{ResultStatus, ResultStore, StoredResult, VersionUpdate};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Request to store a new result
#[derive(Debug, Deserialize)]
pub struct StoreResultRequest {
    /// experiment id
    pub experiment_id: String,
    /// experiment name
    pub experiment_name: Option<String>,
    /// run id
    pub run_id: String,
    /// config
    pub config: serde_json::Value,
    /// metrics
    pub metrics: serde_json::Value,
    /// artifacts
    pub artifacts: Option<Vec<String>>,
    /// figures
    pub figures: Option<Vec<String>>,
    /// created by
    pub created_by: String,
    /// duration secs
    pub duration_secs: f64,
    /// tags
    pub tags: Option<Vec<String>>,
    /// description
    pub description: Option<String>,
}

/// Request to update/create new version
#[derive(Debug, Deserialize)]
pub struct UpdateResultRequest {
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
/// Query parameters for listing results
#[derive(Debug, Deserialize)]
pub struct ListResultsQuery {
    /// experiment id
    pub experiment_id: Option<String>,
    /// status
    pub status: Option<String>,
    /// created by
    pub created_by: Option<String>,
    /// tags
    pub tags: Option<String>, // Comma-separated
    /// limit
    pub limit: Option<usize>,
    /// offset
    pub offset: Option<usize>,
}

/// Export format enum
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    /// Json
    Json,
    /// Csv
    Csv,
    /// Parquet
    Parquet,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Export query parameters
#[derive(Debug, Deserialize)]
pub struct ExportParams {
    /// format
    pub format: Option<ExportFormat>,
}

/// Result version info for API
#[derive(Debug, Clone, Serialize)]
pub struct ResultVersionView {
    /// version
    pub version: i32,
    /// created at
    pub created_at: String,
    /// created by
    pub created_by: String,
    /// changes
    pub changes: String,
}

impl From<crate::persistence::result_store::ResultVersion> for ResultVersionView {
    fn from(v: crate::persistence::result_store::ResultVersion) -> Self {
        Self {
            version: v.version,
            created_at: v.created_at,
            created_by: v.created_by,
            changes: v.changes,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Response wrapper
#[derive(Debug, Serialize)]
pub struct ResultResponse {
    /// result
    pub result: StoredResult,
}

/// List response wrapper
#[derive(Debug, Serialize)]
pub struct ResultsListResponse {
    /// results
    pub results: Vec<StoredResult>,
    /// total
    pub total: usize,
    /// offset
    pub offset: usize,
    /// limit
    pub limit: usize,
}

/// Store state containing the result store
#[derive(Clone, Debug)]
pub struct ResultsState {
    /// store
    pub store: Arc<ResultStore>,
}

/// POST /api/results - Store a new experiment result
/// # Errors
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
///
/// Returns an error if the operation fails.
pub async fn store_result(
    State(state): State<ResultsState>,
    Json(request): Json<StoreResultRequest>,
) -> Result<(StatusCode, Json<ResultResponse>), StatusCode> {
    info!(
        experiment_id = %request.experiment_id,
        run_id = %request.run_id,
        "Storing experiment result"
    );

    // Parse status from metrics if available, default to Completed
    let status = if let Some(status_str) = request.metrics.get("status").and_then(|s| s.as_str()) {
        match status_str {
            "running" => ResultStatus::Running,
            "completed" => ResultStatus::Completed,
            "failed" => ResultStatus::Failed,
            "cancelled" => ResultStatus::Cancelled,
            _ => ResultStatus::Completed,
        }
    } else {
        ResultStatus::Completed
    };

    let result = StoredResult {
        id: uuid::Uuid::new_v4().to_string(),
        experiment_id: request.experiment_id,
        experiment_name: request
            .experiment_name
// TAG: surface=simulation owner=research-team rule=SIM-001
            .unwrap_or_else(|| "Unnamed".to_string()),
        run_id: request.run_id,
        status,
        version: 1,
        config: request.config,
        metrics: request.metrics,
        artifacts: request.artifacts.unwrap_or_default(),
        figures: request.figures.unwrap_or_default(),
        created_at: chrono::Utc::now().to_rfc3339(),
        created_by: request.created_by,
        duration_secs: request.duration_secs,
        tags: request.tags.unwrap_or_default(),
        description: request.description,
        is_deleted: false,
    };

    match state.store.store(result).await {
        Ok(id) => {
            let stored = state
                .store
                .get(&id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            info!(result_id = %id, "Result stored successfully");
            Ok((StatusCode::CREATED, Json(ResultResponse { result: stored })))
        }
        Err(e) => {
            error!(error = %e, "Failed to store result");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /api/results - List results with filters
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn list_results(
    State(state): State<ResultsState>,
    Query(query): Query<ListResultsQuery>,
) -> Result<Json<ResultsListResponse>, StatusCode> {
    info!("Listing experiment results with filters");

    // Parse status filter
    let status_filter = query.status.as_ref().and_then(|s| match s.as_str() {
        "running" => Some(ResultStatus::Running),
        "completed" => Some(ResultStatus::Completed),
        "failed" => Some(ResultStatus::Failed),
        "cancelled" => Some(ResultStatus::Cancelled),
        _ => None,
    });

    // Parse tags filter
    let tags_filter = query
        .tags
        .as_ref()
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect());

    let result_query = crate::persistence::result_store::ResultQuery {
        experiment_id: query.experiment_id,
        status: status_filter,
        created_by: query.created_by,
        tags: tags_filter,
        limit: query.limit.unwrap_or(50),
        offset: query.offset.unwrap_or(0),
    };

    let (results, total) = state.store.list(result_query).await;

    Ok(Json(ResultsListResponse {
        results,
        total,
        offset: query.offset.unwrap_or(0),
        limit: query.limit.unwrap_or(50),
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /api/results/:id - Get specific result
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_result(
    State(state): State<ResultsState>,
    Path(result_id): Path<String>,
) -> Result<Json<ResultResponse>, StatusCode> {
    info!(result_id = %result_id, "Getting experiment result");

    match state.store.get(&result_id).await {
        Ok(result) => Ok(Json(ResultResponse { result })),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /api/results/:id/versions - Get version history
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_result_versions(
    State(state): State<ResultsState>,
    Path(result_id): Path<String>,
) -> Result<Json<Vec<ResultVersionView>>, StatusCode> {
    info!(result_id = %result_id, "Getting result version history");

    match state.store.get_versions(&result_id).await {
        Ok(versions) => {
            let views: Vec<_> = versions.into_iter().map(ResultVersionView::from).collect();
            Ok(Json(views))
        }
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

/// POST /api/results/:id/versions - Create new version
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn create_version(
    State(state): State<ResultsState>,
    Path(result_id): Path<String>,
    Json(request): Json<UpdateResultRequest>,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
) -> Result<(StatusCode, Json<ResultResponse>), StatusCode> {
    info!(
        result_id = %result_id,
        updated_by = %request.updated_by,
        "Creating new result version"
    );

    let update = VersionUpdate {
        config: request.config,
        metrics: request.metrics,
        artifacts: request.artifacts,
        figures: request.figures,
        tags: request.tags,
        description: request.description,
        changes: request.changes,
        updated_by: request.updated_by,
    };

    match state.store.create_version(&result_id, update).await {
        Ok(result) => Ok((StatusCode::CREATED, Json(ResultResponse { result }))),
        Err(crate::persistence::result_store::ResultStoreError::NotFound(_)) => {
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!(error = %e, "Failed to create version");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /api/results/export/:id - Export result in specified format
pub async fn export_result(
    State(state): State<ResultsState>,
    Path(result_id): Path<String>,
    Query(params): Query<ExportParams>,
) -> impl IntoResponse {
    info!(
        result_id = %result_id,
        format = ?params.format,
        "Exporting experiment result"
    );

    let format = params.format.unwrap_or(ExportFormat::Json);

    match format {
        ExportFormat::Json => match state.store.export_json(&result_id).await {
            Ok(json) => (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, "application/json")],
                json,
            )
                .into_response(),
            Err(crate::persistence::result_store::ResultStoreError::NotFound(_)) => {
                StatusCode::NOT_FOUND.into_response()
            }
            Err(e) => {
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                error!(error = %e, "Export failed");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        },
        ExportFormat::Csv => match state.store.export_csv(&result_id).await {
            Ok(csv) => (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, "text/csv")],
                csv,
            )
                .into_response(),
            Err(crate::persistence::result_store::ResultStoreError::NotFound(_)) => {
                StatusCode::NOT_FOUND.into_response()
            }
            Err(e) => {
                error!(error = %e, "Export failed");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        },
        ExportFormat::Parquet => {
            // Parquet export requires additional dependencies (arrow, parquet)
            // For now, return not implemented
            (
                StatusCode::NOT_IMPLEMENTED,
                "Parquet export requires arrow/parquet feature. Use JSON or CSV instead.",
            )
                .into_response()
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// DELETE /api/results/:id - Delete result (soft delete)
pub async fn delete_result(
    State(state): State<ResultsState>,
    Path(result_id): Path<String>,
) -> StatusCode {
    info!(result_id = %result_id, "Deleting experiment result");

    match state.store.delete(&result_id).await {
        Ok(()) => StatusCode::NO_CONTENT,
        Err(crate::persistence::result_store::ResultStoreError::NotFound(_)) => {
            StatusCode::NOT_FOUND
        }
        Err(e) => {
            error!(error = %e, "Delete failed");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

/// GET /api/results/stats - Get store statistics
pub async fn get_stats(
    State(state): State<ResultsState>,
) -> Json<crate::persistence::result_store::StoreStats> {
    let stats = state.store.stats().await;
    Json(stats)
}

#[cfg(test)]
mod tests {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    use super::*;

    fn test_results_state() -> ResultsState {
        ResultsState {
            store: Arc::new(crate::persistence::result_store::ResultStore::new()),
        }
    }

    #[tokio::test]
    async fn test_get_result_not_found() {
        let state = test_results_state();
        let response = get_result(State(state), Path("res1".to_string())).await;
        assert_eq!(response.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_result_versions_not_found() {
        let state = test_results_state();
        let response = get_result_versions(State(state), Path("res1".to_string())).await;
        assert_eq!(response.unwrap_err(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_result_not_found() {
        let state = test_results_state();
        let response = delete_result(State(state), Path("res1".to_string())).await;
        assert_eq!(response, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_stats_empty() {
        let state = test_results_state();
        let response = get_stats(State(state)).await;
        assert_eq!(response.0.total_results, 0);
    }
}
