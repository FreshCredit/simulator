//! Pattern API - REST endpoints for macro experiment patterns

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::patterns::{
    templates::{PatternTemplates, TemplateInfo, ValidationLevel},
    types::*,
    PatternExecutor,
};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Pattern API state
#[derive(Clone, Debug)]
pub struct PatternState {
    executor: Arc<PatternExecutor>,
}

impl PatternState {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[must_use]
    /// new
    pub fn new() -> Self {
        Self {
            executor: Arc::new(PatternExecutor::new()),
        }
    }
}

impl Default for PatternState {
    fn default() -> Self {
        Self::new()
    }
}

/// List available pattern templates
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn list_templates() -> Result<Json<TemplateListResponse>, StatusCode> {
    let templates = PatternTemplates::all();

    Ok(Json(TemplateListResponse { templates }))
}

/// Get a specific template
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_template(
    Path(template_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match PatternTemplates::get(&template_id) {
        Some(template) => Ok(Json(serde_json::json!(template))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Execute a pattern from a template
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn execute_from_template(
    Path(template_id): Path<String>,
    Json(request): Json<ExecuteTemplateRequest>,
) -> Result<Json<ExecutePatternResponse>, StatusCode> {
    // Get base template
    let mut config = PatternTemplates::get(&template_id).ok_or(StatusCode::NOT_FOUND)?;

    // Apply overrides
    if let Some(ref overrides) = request.overrides {
        apply_overrides(&mut config, overrides);
    }

    // Validate
    let validation_messages = crate::patterns::templates::PatternValidator::validate(&config);
    if validation_messages
        .iter()
        .any(|m| m.level == ValidationLevel::Error)
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Execute
    let exec_request = ExecutePatternRequest {
        name: request.name,
        description: request.description,
        config,
    };

    let response = PatternState::new().executor.execute(exec_request).await;

    Ok(Json(response))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Execute a custom pattern
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn execute_pattern(
    State(state): State<PatternState>,
    Json(request): Json<ExecutePatternRequest>,
) -> Result<Json<ExecutePatternResponse>, StatusCode> {
    // Validate
    let validation_messages =
        crate::patterns::templates::PatternValidator::validate(&request.config);
    if validation_messages
        .iter()
        .any(|m| m.level == ValidationLevel::Error)
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    let response = state.executor.execute(request).await;

    Ok(Json(response))
}

/// Get pattern execution status
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_pattern_status(
    State(state): State<PatternState>,
    Path(pattern_id): Path<String>,
) -> Result<Json<PatternStatusResponse>, StatusCode> {
    match state.executor.get_status(&pattern_id).await {
        Some(status) => Ok(Json(status)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get pattern execution results
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_pattern_results(
    State(state): State<PatternState>,
    Path(pattern_id): Path<String>,
) -> Result<Json<PatternResult>, StatusCode> {
    match state.executor.get_result(&pattern_id).await {
        Some(result) => Ok(Json(result)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// List all pattern executions
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn list_patterns(
    State(state): State<PatternState>,
) -> Result<Json<PatternListResponse>, StatusCode> {
    let patterns = state.executor.list_executions().await;

    Ok(Json(PatternListResponse { patterns }))
}

/// Cancel a pattern execution
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn cancel_pattern(
    State(state): State<PatternState>,
    Path(pattern_id): Path<String>,
) -> StatusCode {
    if state.executor.cancel(&pattern_id).await {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Validate a pattern configuration
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn validate_pattern(
    Json(request): Json<ValidatePatternRequest>,
) -> Result<Json<ValidationResponse>, StatusCode> {
    let messages = crate::patterns::templates::PatternValidator::validate(&request.config);

    let is_valid = messages.iter().all(|m| m.level != ValidationLevel::Error);

    Ok(Json(ValidationResponse {
        valid: is_valid,
        messages,
    }))
}

/// Preview what runs a pattern would generate
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn preview_pattern(
    Json(request): Json<PreviewPatternRequest>,
) -> Result<Json<PatternPreview>, StatusCode> {
    // Generate run configurations based on pattern type
    let (run_count, example_configs) = match &request.config.pattern {
        ExperimentPattern::ParameterSweep(sweep) => {
            // Calculate total combinations
            let combinations: usize = if sweep.sampling == SamplingMethod::Grid {
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                sweep
                    .parameters
                    .iter()
                    .map(|p| {
                        if let Some(ref values) = p.values {
                            values.len()
                        } else {
                            let step = p.step.unwrap_or(1.0);
                            ((p.end - p.start) / step + 1.0) as usize
                        }
                    })
                    .product()
            } else {
                sweep.sample_count.unwrap_or(10)
            };
            let total = combinations * sweep.replications as usize;

            // Generate example configs
            let examples = vec![serde_json::json!({
                "example": "Generated configuration will vary based on sampling method"
            })];

            (total, examples)
        }
        ExperimentPattern::ABTest(ab) => {
            let total = ab.replications as usize * 2; // Control + treatment
            let examples = vec![
                serde_json::json!({
                                    "variant": "control",
                                    "replication": 0,
                // TAG: surface=simulation owner=research-team rule=SIM-001
                                }),
                serde_json::json!({
                    "variant": "treatment",
                    "replication": 0,
                }),
            ];
            (total, examples)
        }
        ExperimentPattern::Factorial(factorial) => {
            let combinations: usize = factorial
                .factors
                .iter()
                .map(|f| f.levels.len())
                .product::<usize>();
            let total = combinations * factorial.replications as usize;
            let examples = vec![serde_json::json!({
                "example": format!("Full factorial with {} combinations", combinations)
            })];
            (total, examples)
        }
        ExperimentPattern::Scenario(scenario) => {
            let total = (1 + scenario.scenarios.len()) * scenario.replications as usize;
            let examples = vec![serde_json::json!({
                "scenario": "base",
                "replication": 0,
            })];
            (total, examples)
        }
        ExperimentPattern::Baseline(baseline) => {
            let total = (1 + baseline.alternatives.len()) * baseline.replications as usize;
            let examples = vec![serde_json::json!({
                "variant": "baseline",
                "replication": 0,
            })];
            (total, examples)
        }
    };

    // Estimate duration
    let estimated_duration_secs = run_count as u64 * request.config.global_params.ticks / 10;

    Ok(Json(PatternPreview {
        total_runs: run_count,
        estimated_duration_secs,
        estimated_storage_mb: (run_count as f64 * 0.1) as u64, // Rough estimate
        example_configurations: example_configs,
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Export pattern results
/// # Panics
///
/// Panics if preconditions are not met.
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn export_pattern_results(
    State(state): State<PatternState>,
    Path(pattern_id): Path<String>,
    Json(request): Json<ExportRequest>,
) -> Result<Json<ExportResponse>, StatusCode> {
    let result = state
        .executor
        .get_result(&pattern_id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    let payload = serde_json::to_vec(&result).unwrap_or_default();
    let size_bytes = payload.len() as u64;

    Ok(Json(ExportResponse {
        url: format!("/api/patterns/{pattern_id}/export/download"),
        format: request.format,
        size_bytes,
        expires_at: chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(24))
            .unwrap_or_else(chrono::Utc::now)
            .to_rfc3339(),
    }))
}

// Request/Response types

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Deserialize)]
/// `ExecuteTemplateRequest` data
pub struct ExecuteTemplateRequest {
    /// name
    pub name: String,
    /// description
    pub description: Option<String>,
    /// overrides
    pub overrides: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
/// `TemplateListResponse` data
pub struct TemplateListResponse {
    /// templates
    pub templates: Vec<TemplateInfo>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Deserialize)]
/// `ValidatePatternRequest` data
pub struct ValidatePatternRequest {
    /// config
    pub config: PatternConfig,
}

#[derive(Debug, Serialize)]
/// `ValidationResponse` data
pub struct ValidationResponse {
    /// valid
    pub valid: bool,
    /// messages
    pub messages: Vec<crate::patterns::templates::ValidationMessage>,
}

#[derive(Debug, Deserialize)]
/// `PreviewPatternRequest` data
pub struct PreviewPatternRequest {
    /// config
    pub config: PatternConfig,
}

#[derive(Debug, Serialize)]
/// `PatternPreview` data
pub struct PatternPreview {
    /// total runs
    pub total_runs: usize,
    /// estimated duration secs
    pub estimated_duration_secs: u64,
    /// estimated storage mb
    pub estimated_storage_mb: u64,
    /// example configurations
    pub example_configurations: Vec<serde_json::Value>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Deserialize)]
/// `ExportRequest` data
pub struct ExportRequest {
    /// format
    pub format: String,
    /// include raw
    pub include_raw: Option<bool>,
}

#[derive(Debug, Serialize)]
/// `ExportResponse` data
pub struct ExportResponse {
    /// url
    pub url: String,
    /// format
    pub format: String,
    /// size bytes
    pub size_bytes: u64,
    /// expires at
    pub expires_at: String,
}

#[derive(Debug, Serialize)]
/// `PatternListResponse` data
pub struct PatternListResponse {
    /// patterns
    pub patterns: Vec<PatternSummary>,
}

/// Apply overrides to a pattern configuration
fn apply_overrides(config: &mut PatternConfig, overrides: &serde_json::Value) {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    if let Some(obj) = overrides.as_object() {
        // Apply global params overrides
        if let Some(global) = obj.get("global_params") {
            if let Some(ticks) = global.get("ticks").and_then(serde_json::Value::as_u64) {
                config.global_params.ticks = ticks;
            }
            if let Some(metrics) = global.get("metrics").and_then(|v| v.as_array()) {
                config.global_params.metrics = metrics
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
            }
        }

        // Apply constraints overrides
        if let Some(constraints) = obj.get("constraints") {
            if let Some(parallelism) = constraints
                .get("max_parallelism")
                .and_then(serde_json::Value::as_u64)
            {
                config.constraints.max_parallelism = parallelism as usize;
            }
            if let Some(duration) = constraints
                .get("max_duration_secs")
                .and_then(serde_json::Value::as_u64)
            {
                config.constraints.max_duration_secs = duration;
            }
        }
    }
}
