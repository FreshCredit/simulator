//! External Data Import API Module (P1-API-5)
//!
//! Import real datasets for validation.

use axum::{extract::Query, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Supported import formats
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImportFormat {
    /// Csv
    Csv,
    /// Json
    Json,
    /// Xlsx
    Xlsx,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Import request
#[derive(Debug, Deserialize)]
pub struct ImportRequest {
    /// dataset name
    pub dataset_name: String,
    /// format
    pub format: ImportFormat,
    /// data type
    pub data_type: DataType,
    /// description
    pub description: Option<String>,
}

/// Type of data being imported
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    /// `SurveyResponses`
    SurveyResponses,
    /// `ExperimentalData`
    ExperimentalData,
    /// `ObservationalData`
    ObservationalData,
    /// `EconomicIndicators`
    EconomicIndicators,
    /// `DemographicData`
    DemographicData,
}

/// Import response
#[derive(Debug, Serialize)]
pub struct ImportResponse {
    /// import id
    pub import_id: String,
    /// status
    pub status: String,
    /// records imported
    pub records_imported: usize,
    /// validation errors
    pub validation_errors: Vec<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Import external dataset
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn import_dataset(
    Json(_request): Json<ImportRequest>,
) -> Result<Json<ImportResponse>, StatusCode> {
    // Fail closed rather than returning fabricated 1000-record mock success
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// Upload file for import (placeholder - multipart not enabled)
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn upload_dataset() -> Result<Json<ImportResponse>, StatusCode> {
    Ok(Json(ImportResponse {
        import_id: format!("import_{}", uuid::Uuid::new_v4()),
        status: "pending".to_string(),
        records_imported: 0,
        validation_errors: vec!["Multipart upload not enabled".to_string()],
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// List imported datasets
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn list_imported_datasets() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "datasets": [],
        "not_implemented": true
    }))
}

/// Get dataset details
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_dataset(
    Query(_query): Query<DatasetQuery>,
) -> Result<Json<DatasetInfo>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Delete imported dataset
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn delete_dataset(Query(_query): Query<DatasetQuery>) -> Result<StatusCode, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Validate dataset against schema
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn validate_dataset(
    Json(_request): Json<ValidationRequest>,
) -> Result<Json<ValidationResult>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// Map dataset to experiment parameters
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn map_to_experiment(
    Json(_request): Json<MappingRequest>,
) -> Result<Json<MappingResult>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Query parameters
#[derive(Debug, Deserialize)]
pub struct DatasetQuery {
    /// dataset id
    pub dataset_id: String,
}

/// Dataset information
#[derive(Debug, Serialize)]
pub struct DatasetInfo {
    /// id
    pub id: String,
    /// name
    pub name: String,
    /// data type
    pub data_type: DataType,
    /// records
    pub records: usize,
    /// schema
    pub schema: Vec<ColumnInfo>,
    /// summary
    pub summary: DatasetSummary,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Column information
#[derive(Debug, Serialize)]
pub struct ColumnInfo {
    /// name
    pub name: String,
    /// data type
    pub data_type: String,
    /// nullable
    pub nullable: bool,
}

/// Dataset summary
#[derive(Debug, Serialize)]
pub struct DatasetSummary {
    /// completeness
    pub completeness: f32,
    /// numeric columns
    pub numeric_columns: Vec<String>,
    /// categorical columns
    pub categorical_columns: Vec<String>,
}

/// Validation request
#[derive(Debug, Deserialize)]
pub struct ValidationRequest {
    /// dataset id
    pub dataset_id: String,
    /// schema
    pub schema: Option<serde_json::Value>,
}

/// Validation result
#[derive(Debug, Serialize)]
pub struct ValidationResult {
    /// valid
    pub valid: bool,
    /// errors
    pub errors: Vec<String>,
    /// warnings
    pub warnings: Vec<String>,
    /// stats
    pub stats: ValidationStats,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Validation statistics
#[derive(Debug, Serialize)]
pub struct ValidationStats {
    /// total records
    pub total_records: usize,
    /// valid records
    pub valid_records: usize,
    /// invalid records
    pub invalid_records: usize,
}

/// Mapping request
#[derive(Debug, Deserialize)]
pub struct MappingRequest {
    /// dataset id
    pub dataset_id: String,
    /// mappings
    pub mappings: std::collections::HashMap<String, f32>,
}

/// Mapping result
#[derive(Debug, Serialize)]
pub struct MappingResult {
    /// mapping id
    pub mapping_id: String,
    /// experiment config
    pub experiment_config: serde_json::Value,
    /// coverage
    pub coverage: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_upload_dataset() {
        let response = upload_dataset().await.unwrap();
        assert_eq!(response.status, "pending");
    }

    #[tokio::test]
    async fn test_list_imported_datasets() {
        let response = list_imported_datasets().await;
        let datasets = response.0["datasets"].as_array().unwrap();
        assert!(datasets.is_empty());
        assert_eq!(response.0["not_implemented"], true);
    }

    #[tokio::test]
    async fn test_import_returns_not_implemented() {
        assert_eq!(
            delete_dataset(Query(DatasetQuery {
                dataset_id: "x".to_string()
            }))
            .await
            .unwrap_err(),
            StatusCode::NOT_IMPLEMENTED
        );
    }
}
