//! Artifact analysis API with Gemini AI integration
//!
//! Provides AI-powered insights on completed simulation runs.
//! Uses direct HTTP calls to Gemini API (SIM-001 compliant).

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::ai::{AiState, SimulationPromptBuilder};
use crate::artifacts::ArtifactStore;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// State for analysis endpoints
#[derive(Clone, Debug)]
pub struct AnalysisState {
    /// artifact store
    pub artifact_store: Arc<ArtifactStore>,
    /// ai state
    pub ai_state: Arc<AiState>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl AnalysisState {
    #[must_use]
    /// fn
    pub const fn new(artifact_store: Arc<ArtifactStore>, ai_state: Arc<AiState>) -> Self {
        Self {
            artifact_store,
            ai_state,
        }
    }
}

/// Request to query a specific artifact
#[derive(Debug, Clone, Deserialize)]
pub struct ArtifactQueryRequest {
    /// Natural language question about the artifact
    pub question: String,
}

/// Response with AI-generated insight
#[derive(Debug, Clone, Serialize)]
pub struct ArtifactInsightResponse {
    /// The AI-generated insight
    pub insight: String,
    /// Whether this is a mock response
    pub is_mock: bool,
    /// Model used for generation
    pub model: String,
    /// Run ID being analyzed
    pub run_id: String,
    /// Timestamp of generation
    pub generated_at: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /`api/analysis/{run_id}/insights`
/// Generate AI insights for a completed simulation run
/// # Errors
///
/// Returns an error if the operation fails.
/// # Panics
///
/// Panics if preconditions are not met.
pub async fn get_run_insights(
    Path(run_id): Path<String>,
    State(state): State<AnalysisState>,
) -> Result<Json<ArtifactInsightResponse>, axum::http::StatusCode> {
    use chrono::Utc;

    if !ArtifactStore::is_valid_run_id(&run_id) {
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }

    // Load manifest from artifact store
    let Some(manifest) = state.artifact_store.load_manifest(&run_id).await else {
        tracing::warn!("Run {} not found for analysis", run_id);
        return Err(axum::http::StatusCode::NOT_FOUND);
    };

    // Check if AI is available
    if !state.ai_state.is_available() {
        tracing::warn!(
            "AI not configured, returning mock insight for run {}",
            run_id // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        );
        return Ok(Json(ArtifactInsightResponse {
            insight: format!(
                "Analysis for run {}:\n\n\
                This simulation completed with status: {:?}.\n\
                - Schema version: {}\n\
                - Metrics file: {}\n\
                - Validation passed: {}\n\n\
                (Note: Gemini AI is not configured. Set GEMINI_API_KEY for AI-powered analysis.)",
                run_id,
                manifest.status,
                manifest.schema_version,
                manifest.metrics_file,
                manifest
                    .validation
                    .as_ref()
                    .map_or("N/A", |v| if v.passed { "Yes" } else { "No" })
            ),
            is_mock: true,
            model: "mock".to_string(),
            run_id,
            generated_at: Utc::now().to_rfc3339(),
        }));
    }

    // Build prompt from manifest
    let prompt = SimulationPromptBuilder::analyze_manifest(&manifest);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Call Gemini
    let Some(gemini) = state.ai_state.gemini.as_ref() else {
        return Err(axum::http::StatusCode::SERVICE_UNAVAILABLE);
    };
    match gemini
        .generate_text(prompt, Some(SimulationPromptBuilder::system_instruction()))
        .await
    {
        Ok(insight) => Ok(Json(ArtifactInsightResponse {
            insight,
            is_mock: false,
            model: gemini.model().to_string(),
            run_id,
            generated_at: Utc::now().to_rfc3339(),
        })),
        Err(e) => {
            tracing::error!("Gemini API error: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// POST /`api/analysis/{run_id}/query`
/// Answer a specific question about a simulation run
/// # Errors
///
/// Returns an error if the operation fails.
/// # Panics
///
/// Panics if preconditions are not met.
pub async fn query_artifact(
    Path(run_id): Path<String>,
    State(state): State<AnalysisState>,
    Json(request): Json<ArtifactQueryRequest>,
) -> Result<Json<ArtifactInsightResponse>, axum::http::StatusCode> {
    use chrono::Utc;

    const MAX_QUESTION_LEN: usize = 4000;
    let question = request.question.trim();
    if !ArtifactStore::is_valid_run_id(&run_id)
        || question.is_empty()
        || question.len() > MAX_QUESTION_LEN
    {
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }

    // Load manifest
    let Some(manifest) = state.artifact_store.load_manifest(&run_id).await else {
        return Err(axum::http::StatusCode::NOT_FOUND);
    };

    // Load metrics if available
    let metrics_summary = match state.artifact_store.load_metrics(&run_id).await {
        Some(metrics) => {
            let count = metrics.len();
            let avg_cooperation: f64 =
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
                metrics.iter().map(|m| m.cooperation_rate).sum::<f64>() / count.max(1) as f64;
            format!(
                "Metrics available: {} ticks, avg cooperation: {:.1}%",
                count,
                avg_cooperation * 100.0
            )
        }
        None => "No metrics available".to_string(),
    };

    // Check AI availability
    if !state.ai_state.is_available() {
        let status_str = format!("Status: {:?}", manifest.status);
        return Ok(Json(ArtifactInsightResponse {
            insight: format!(
                "Question: {}\n\n\
                Context: Run {} - {}\n\
                {}\n\n\
                (Note: Gemini AI is not configured. Set GEMINI_API_KEY for AI-powered analysis.)",
                request.question, run_id, status_str, metrics_summary
            ),
            is_mock: true,
            model: "mock".to_string(),
            run_id,
            generated_at: Utc::now().to_rfc3339(),
        }));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Build contextual prompt
    let prompt = format!(
        "You are analyzing an Agent Society Simulation run.\n\n\
        **Run Information:**\n\
        - Run ID: {}\n\
        - Status: {:?}\n\
        - Schema: {}\n\
        - {}\n\n\
        **User Question:** {}\n\n\
        Provide a clear, specific answer based on the available information.",
        run_id, manifest.status, manifest.schema_version, metrics_summary, request.question
    );

    // Call Gemini
    let Some(gemini) = state.ai_state.gemini.as_ref() else {
        return Err(axum::http::StatusCode::SERVICE_UNAVAILABLE);
    };
    match gemini
        .generate_text(prompt, Some(SimulationPromptBuilder::system_instruction()))
        .await
    {
        Ok(insight) => Ok(Json(ArtifactInsightResponse {
            insight,
            is_mock: false,
            model: gemini.model().to_string(),
            run_id,
            generated_at: Utc::now().to_rfc3339(),
        })),
        Err(e) => {
            tracing::error!("Gemini API error: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /api/analysis/{run_id}/metrics-insights
/// Generate insights from metrics time series data
/// # Errors
///
/// Returns an error if the operation fails.
/// # Panics
///
/// Panics if preconditions are not met.
pub async fn get_metrics_insights(
    Path(run_id): Path<String>,
    State(state): State<AnalysisState>,
) -> Result<Json<ArtifactInsightResponse>, axum::http::StatusCode> {
    use chrono::Utc;

    if !ArtifactStore::is_valid_run_id(&run_id) {
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }

    // Load metrics
    let Some(metrics) = state.artifact_store.load_metrics(&run_id).await else {
        return Err(axum::http::StatusCode::NOT_FOUND);
    };

    // Check AI availability
    if !state.ai_state.is_available() {
        return Ok(Json(ArtifactInsightResponse {
            insight: format!(
                "Metrics for run {}:\n\n\
                - Total ticks: {}\n\
                - Initial agents: {}\n\
                - Final agents: {}\n\n\
                (Note: Gemini AI is not configured. Set GEMINI_API_KEY for AI-powered analysis.)",
                run_id,
                metrics.len(),
                metrics.first().map_or(0, |m| m.agent_count),
                metrics.last().map_or(0, |m| m.agent_count)
            ),
            is_mock: true,
            model: "mock".to_string(),
            run_id,
            generated_at: Utc::now().to_rfc3339(),
        }));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Build prompt from metrics
    let prompt = SimulationPromptBuilder::analyze_metrics(&metrics);

    // Call Gemini
    let Some(gemini) = state.ai_state.gemini.as_ref() else {
        return Err(axum::http::StatusCode::SERVICE_UNAVAILABLE);
    };
    match gemini
        .generate_text(prompt, Some(SimulationPromptBuilder::system_instruction()))
        .await
    {
        Ok(insight) => Ok(Json(ArtifactInsightResponse {
            insight,
            is_mock: false,
            model: gemini.model().to_string(),
            run_id,
            generated_at: Utc::now().to_rfc3339(),
        })),
        Err(e) => {
            tracing::error!("Gemini API error: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analysis_rejects_traversal_run_id_and_oversized_question() {
        let dir = tempfile::tempdir().unwrap();
        let state = AnalysisState::new(
            Arc::new(ArtifactStore::new(dir.path().to_path_buf())),
            Arc::new(AiState::disabled()),
        );
        let res_bad_id = get_run_insights(Path("../etc".to_string()), State(state.clone())).await;
        assert_eq!(res_bad_id.unwrap_err(), axum::http::StatusCode::BAD_REQUEST);

        let res_big_q = query_artifact(
            Path("valid-run".to_string()),
            State(state),
            Json(ArtifactQueryRequest {
                question: "x".repeat(5000),
            }),
        )
        .await;
        assert_eq!(res_big_q.unwrap_err(), axum::http::StatusCode::BAD_REQUEST);
    }
}
