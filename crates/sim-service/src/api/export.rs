//! Result Export API Module (P1-API-4)
//!
//! Export experiment results in multiple formats: CSV, JSON, Parquet.

use axum::{
    extract::Query,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Export format
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    /// Csv
    Csv,
    /// Json
    Json,
    /// Parquet
    Parquet,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Export request query parameters
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    /// experiment id
    pub experiment_id: String,
    /// format
    pub format: ExportFormat,
    /// metrics
    pub metrics: Option<Vec<String>>,
    /// start tick
    pub start_tick: Option<u64>,
    /// end tick
    pub end_tick: Option<u64>,
}

/// Export experiment results
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn export_results(
    Query(query): Query<ExportQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    if !crate::artifacts::ArtifactStore::is_valid_run_id(&query.experiment_id) {
        return Err(StatusCode::BAD_REQUEST);
    }
    // Generate synthetic data for demo
    let data = generate_sample_data(&query.experiment_id);

    match query.format {
        ExportFormat::Csv => Ok(export_csv(&data, &query)),
        ExportFormat::Json => export_json(&data, &query),
        ExportFormat::Parquet => export_parquet(&data, &query),
    }
}

const MAX_BATCH_EXPORT_IDS: usize = 50;

fn csv_escape(cell: &str) -> String {
    let formula_prefixed = if cell.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{cell}")
    } else {
        cell.to_string()
    };
    if formula_prefixed.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", formula_prefixed.replace('"', "\"\""))
    } else {
        formula_prefixed
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Export multiple experiments
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn export_batch(
    Json(request): Json<BatchExportRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    if request.experiment_ids.is_empty()
        || request.experiment_ids.len() > MAX_BATCH_EXPORT_IDS
        || request
            .experiment_ids
            .iter()
            .any(|id| !crate::artifacts::ArtifactStore::is_valid_run_id(id))
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let mut combined_data = Vec::new();

    for exp_id in &request.experiment_ids {
        let data = generate_sample_data(exp_id);
        combined_data.push((exp_id.clone(), data));
    }

    match request.format {
        ExportFormat::Json => {
            let json = serde_json::to_string(&combined_data)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            Ok((
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/json")],
                json,
            ))
        }
        ExportFormat::Csv => {
            let csv = generate_batch_csv(&combined_data);
            Ok((StatusCode::OK, [(header::CONTENT_TYPE, "text/csv")], csv))
        }
        ExportFormat::Parquet => Err(StatusCode::NOT_IMPLEMENTED),
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

/// List available export formats
pub async fn list_export_formats() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "formats": [
            {
                "id": "csv",
                "name": "CSV",
                "description": "Comma-separated values",
                "mime_type": "text/csv",
                "extension": ".csv"
            },
            {
                "id": "json",
                "name": "JSON",
                "description": "JavaScript Object Notation",
                "mime_type": "application/json",
                "extension": ".json"
            },
            {
                "id": "parquet",
                "name": "Parquet",
                "description": "Apache Parquet columnar format",
                "mime_type": "application/octet-stream",
                "extension": ".parquet"
            }
        ]
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Export as CSV
#[allow(clippy::type_complexity)]
fn export_csv(
    data: &[MetricRow],
    query: &ExportQuery,
) -> (StatusCode, [(header::HeaderName, &'static str); 1], String) {
    let mut csv = String::from("timestamp,metric,value,agent_id\n");

    for row in data {
        if let Some(ref metrics) = query.metrics {
            if !metrics.contains(&row.metric) {
                continue;
            }
        }

        if let Some(start) = query.start_tick {
            if row.timestamp < start {
                continue;
            }
        }

        if let Some(end) = query.end_tick {
            if row.timestamp > end {
                continue;
            }
        }

        csv.push_str(&format!(
            "{},{},{},{}\n",
            row.timestamp,
            csv_escape(&row.metric),
            row.value,
            row.agent_id
        ));
    }

    (StatusCode::OK, [(header::CONTENT_TYPE, "text/csv")], csv)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Export as JSON
#[allow(clippy::type_complexity)]
fn export_json(
    data: &[MetricRow],
    query: &ExportQuery,
) -> Result<(StatusCode, [(header::HeaderName, &'static str); 1], String), StatusCode> {
    let filtered: Vec<_> = data
        .iter()
        .filter(|row| {
            if let Some(ref metrics) = query.metrics {
                metrics.contains(&row.metric)
            } else {
                true
            }
        })
        .filter(|row| {
            if let Some(start) = query.start_tick {
                row.timestamp >= start
            } else {
                true
            }
        })
        .filter(|row| {
            if let Some(end) = query.end_tick {
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                row.timestamp <= end
            } else {
                true
            }
        })
        .collect();

    let json =
        serde_json::to_string_pretty(&filtered).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        json,
    ))
}

/// Export as Parquet
#[allow(clippy::type_complexity)]
const fn export_parquet(
    _data: &[MetricRow],
    _query: &ExportQuery,
) -> Result<(StatusCode, [(header::HeaderName, &'static str); 1], String), StatusCode> {
    // Binary Parquet stream format reserved for columnar batch pipeline
    Err(StatusCode::NOT_IMPLEMENTED)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Sample data row
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricRow {
    timestamp: u64,
    metric: String,
    value: f64,
    agent_id: u64,
}

/// Generate sample data
fn generate_sample_data(experiment_id: &str) -> Vec<MetricRow> {
    let mut data = Vec::new();
    let metrics = vec!["fitness", "trust", "cooperation", "diversity"];

    for tick in 0..100 {
        for agent_id in 0..10 {
            for metric in &metrics {
                let value = match *metric {
                    "fitness" => 0.5 + (tick as f64 / 200.0),
                    "trust" => 0.6 + 0.2 * (tick as f64 / 100.0).sin(),
                    "cooperation" => 0.4 + 0.3 * (tick as f64 / 50.0).sin(),
                    "diversity" => 0.7 - (tick as f64 / 300.0),
                    _ => 0.5,
                };

                data.push(MetricRow {
                    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                    timestamp: tick as u64,
                    metric: format!("{experiment_id}_{metric}"),
                    value,
                    agent_id: agent_id as u64,
                });
            }
        }
    }

    data
}

/// Generate CSV for multiple experiments
/// # Errors
///
/// Returns an error if the operation fails.
fn generate_batch_csv(data: &[(String, Vec<MetricRow>)]) -> String {
    let mut csv = String::from("experiment_id,timestamp,metric,value,agent_id\n");

    for (exp_id, rows) in data {
        let escaped_id = csv_escape(exp_id);
        for row in rows {
            csv.push_str(&format!(
                "{},{},{},{},{}\n",
                escaped_id,
                row.timestamp,
                csv_escape(&row.metric),
                row.value,
                row.agent_id
            ));
        }
    }

    csv
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Batch export request
#[derive(Debug, Deserialize)]
pub struct BatchExportRequest {
    /// experiment ids
    pub experiment_ids: Vec<String>,
    /// format
    pub format: ExportFormat,
    /// metrics
    pub metrics: Option<Vec<String>>,
}

/// Get export preview
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn export_preview(
    Query(query): Query<ExportQuery>,
) -> Result<Json<ExportPreview>, StatusCode> {
    let data = generate_sample_data(&query.experiment_id);

    let metrics: Vec<String> = data
        .iter()
        .map(|row| row.metric.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let total_rows = data.len();
    let size_estimate_bytes = total_rows * 50; // Rough estimate

    Ok(Json(ExportPreview {
        experiment_id: query.experiment_id,
        total_rows,
        columns: vec![
            "timestamp".to_string(),
            "metric".to_string(),
            "value".to_string(),
            "agent_id".to_string(),
        ],
        available_metrics: metrics,
        size_estimate_bytes,
        sample: data.into_iter().take(5).collect(),
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Export preview response
#[derive(Debug, Serialize)]
pub struct ExportPreview {
    /// experiment id
    pub experiment_id: String,
    /// total rows
    pub total_rows: usize,
    /// columns
    pub columns: Vec<String>,
    /// available metrics
    pub available_metrics: Vec<String>,
    /// size estimate bytes
    pub size_estimate_bytes: usize,
    /// sample
    pub sample: Vec<MetricRow>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_export_formats() {
        let response = list_export_formats().await;
        let formats = response.0["formats"].as_array().unwrap();
        assert!(!formats.is_empty());
        assert_eq!(formats[0]["id"], "csv");
    }
}
