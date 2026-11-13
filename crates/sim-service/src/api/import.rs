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
    // Demo implementation - would actually parse data
    let import_id = format!("import_{}", uuid::Uuid::new_v4());

    Ok(Json(ImportResponse {
        import_id,
        status: "completed".to_string(),
        records_imported: 1000,
        validation_errors: vec![],
    }))
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
        "datasets": [
            {
                "id": "dataset_001",
                "name": "Trust Game Survey",
                "data_type": "experimental_data",
                "records": 500,
                "imported_at": "2026-02-16T10:00:00Z",
                "validation_status": "valid"
            },
            {
                "id": "dataset_002",
                "name": "Economic Indicators 2025",
                "data_type": "economic_indicators",
                "records": 120,
                "imported_at": "2026-02-15T14:30:00Z",
                "validation_status": "valid"
            }
        ]
    }))
}

/// Get dataset details
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_dataset(
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    Query(query): Query<DatasetQuery>,
) -> Result<Json<DatasetInfo>, StatusCode> {
    Ok(Json(DatasetInfo {
        id: query.dataset_id,
        name: "Sample Dataset".to_string(),
        data_type: DataType::ExperimentalData,
        records: 1000,
        schema: vec![
            ColumnInfo {
                name: "participant_id".to_string(),
                data_type: "string".to_string(),
                nullable: false,
            },
            ColumnInfo {
                name: "trust_score".to_string(),
                data_type: "float".to_string(),
                nullable: true,
            },
            ColumnInfo {
                name: "cooperation_rate".to_string(),
                data_type: "float".to_string(),
                nullable: true,
            },
        ],
        summary: DatasetSummary {
            completeness: 0.95,
            numeric_columns: vec!["trust_score".to_string(), "cooperation_rate".to_string()],
            categorical_columns: vec!["participant_id".to_string()],
        },
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Delete imported dataset
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn delete_dataset(Query(_query): Query<DatasetQuery>) -> Result<StatusCode, StatusCode> {
    // Would actually delete from storage
    Ok(StatusCode::NO_CONTENT)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Validate dataset against schema
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn validate_dataset(
    Json(request): Json<ValidationRequest>,
) -> Result<Json<ValidationResult>, StatusCode> {
    // Demo validation
    let errors = if request.dataset_id == "invalid" {
        vec!["Missing required column: participant_id".to_string()]
    } else {
        vec![]
    };

    Ok(Json(ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings: vec!["Some values outside expected range".to_string()],
        stats: ValidationStats {
            total_records: 1000,
            valid_records: 995,
            invalid_records: 5,
        },
    }))
}

/// Map dataset to experiment parameters
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn map_to_experiment(
    Json(request): Json<MappingRequest>,
) -> Result<Json<MappingResult>, StatusCode> {
    Ok(Json(MappingResult {
        mapping_id: format!("map_{}", uuid::Uuid::new_v4()),
        experiment_config: serde_json::json!({
            "agent_count": 1000,
            "parameters": {
                "initial_trust": request.mappings.get("trust_score").copied().unwrap_or(0.5),
                "cooperation_bias": request.mappings.get("cooperation_rate").copied().unwrap_or(0.5),
            }
        }),
        coverage: 0.85,
    }))
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
        assert_eq!(datasets.len(), 2);
    }
}
