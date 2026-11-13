//! Research API (P2-RES)
//!
//! REST endpoints for advanced research capabilities.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use sim_core::research::{BayesianModel, LiteratureEntry, ResearchEngine};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Shared research state
#[derive(Clone, Debug)]
pub struct ResearchState {
    /// engine
    pub engine: Arc<RwLock<ResearchEngine>>,
}

impl ResearchState {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[must_use]
    /// new
    pub fn new() -> Self {
        Self {
            engine: Arc::new(RwLock::new(ResearchEngine::default())),
        }
    }
}

impl Default for ResearchState {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== HYPOTHESIS ENDPOINTS ====================

/// List all hypotheses
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn list_hypotheses(
    State(state): State<ResearchState>,
) -> Result<Json<Vec<HypothesisResponse>>, StatusCode> {
    let engine = state.engine.read().await;

    let hypotheses: Vec<HypothesisResponse> = engine
        .hypothesis_gen
        .hypotheses
        .iter()
        .map(|h| HypothesisResponse {
            id: h.id.clone(),
            statement: h.statement.clone(),
            status: format!("{:?}", h.status),
            priority: format!("{:?}", h.priority),
            confidence: h.confidence,
            variables: h.variables.clone(),
            generated_at: h.generated_at,
        })
        .collect();

    Ok(Json(hypotheses))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get high priority hypotheses
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn high_priority_hypotheses(
    State(state): State<ResearchState>,
) -> Result<Json<Vec<HypothesisResponse>>, StatusCode> {
    let engine = state.engine.read().await;

    let hypotheses: Vec<HypothesisResponse> = engine
        .hypothesis_gen
        .high_priority()
        .into_iter()
        .map(|h| HypothesisResponse {
            id: h.id.clone(),
            statement: h.statement.clone(),
            status: format!("{:?}", h.status),
            priority: format!("{:?}", h.priority),
            confidence: h.confidence,
            variables: h.variables.clone(),
            generated_at: h.generated_at,
        })
        .collect();

    Ok(Json(hypotheses))
}

/// Generate hypotheses from correlations
#[derive(Debug, Deserialize)]
pub struct GenerateHypothesesRequest {
    /// correlations
    pub correlations: Vec<(String, String, f32)>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Serialize)]
/// `GenerateHypothesesResponse` data
pub struct GenerateHypothesesResponse {
    /// generated
    pub generated: usize,
    /// hypotheses
    pub hypotheses: Vec<HypothesisResponse>,
}

/// # Errors
///
/// Returns an error if the operation fails.
pub async fn generate_hypotheses(
    State(state): State<ResearchState>,
    Json(request): Json<GenerateHypothesesRequest>,
) -> Result<Json<GenerateHypothesesResponse>, StatusCode> {
    let mut engine = state.engine.write().await;

    let new_hypotheses = engine.hypothesis_gen.generate_from_correlations(
        &request.correlations,
        0, // tick
    );

    let response = GenerateHypothesesResponse {
        generated: new_hypotheses.len(),
        hypotheses: new_hypotheses
            .iter()
            .map(|h| HypothesisResponse {
                id: h.id.clone(),
                statement: h.statement.clone(),
                status: format!("{:?}", h.status),
                priority: format!("{:?}", h.priority),
                confidence: h.confidence,
                variables: h.variables.clone(),
                generated_at: h.generated_at,
            })
            .collect(),
    };

    Ok(Json(response))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Hypothesis response
#[derive(Debug, Serialize, Clone)]
pub struct HypothesisResponse {
    /// id
    pub id: String,
    /// statement
    pub statement: String,
    /// status
    pub status: String,
    /// priority
    pub priority: String,
    /// confidence
    pub confidence: f32,
    /// variables
    pub variables: Vec<String>,
    /// generated at
    pub generated_at: u64,
}

// ==================== LITERATURE ENDPOINTS ====================

/// Search literature
#[derive(Debug, Deserialize)]
pub struct LiteratureSearchRequest {
    /// query
    pub query: String,
    /// limit
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
/// `LiteratureSearchResponse` data
pub struct LiteratureSearchResponse {
    /// results
    pub results: Vec<LiteratureEntryResponse>,
    /// total
    pub total: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Serialize)]
/// `LiteratureEntryResponse` data
pub struct LiteratureEntryResponse {
    /// id
    pub id: String,
    /// title
    pub title: String,
    /// authors
    pub authors: Vec<String>,
    /// year
    pub year: u16,
    /// relevance score
    pub relevance_score: f32,
}

/// # Errors
///
/// Returns an error if the operation fails.
pub async fn search_literature(
    State(state): State<ResearchState>,
    Query(params): Query<LiteratureSearchRequest>,
) -> Result<Json<LiteratureSearchResponse>, StatusCode> {
    let engine = state.engine.read().await;

    let results: Vec<&LiteratureEntry> = engine.literature.search(&params.query);
    let limit = params.limit.unwrap_or(10);

    let response = LiteratureSearchResponse {
        total: results.len(),
        results: results
            .iter()
            .take(limit)
            .map(|e| LiteratureEntryResponse {
                id: e.id.clone(),
                title: e.title.clone(),
                authors: e.authors.clone(),
                year: e.year,
                relevance_score: e.relevance_score,
            })
            .collect(),
    };

    Ok(Json(response))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Generate literature synthesis
#[derive(Debug, Serialize)]
pub struct SynthesisResponse {
    /// topic
    pub topic: String,
    /// papers reviewed
    pub papers_reviewed: usize,
    /// major themes
    pub major_themes: Vec<ThemeResponse>,
    /// gaps
    pub gaps: Vec<String>,
}

#[derive(Debug, Serialize)]
/// `ThemeResponse` data
pub struct ThemeResponse {
    /// name
    pub name: String,
    /// paper count
    pub paper_count: usize,
}

/// # Errors
///
/// Returns an error if the operation fails.
pub async fn generate_synthesis(
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    State(state): State<ResearchState>,
    Path(topic): Path<String>,
) -> Result<Json<SynthesisResponse>, StatusCode> {
    let mut engine = state.engine.write().await;

    let synthesis = engine.literature.generate_synthesis(&topic);

    let response = SynthesisResponse {
        topic: synthesis.topic,
        papers_reviewed: synthesis.papers_reviewed,
        major_themes: synthesis
            .major_themes
            .iter()
            .map(|t| ThemeResponse {
                name: t.name.clone(),
                paper_count: t.paper_count,
            })
            .collect(),
        gaps: synthesis.gaps_identified,
    };

    Ok(Json(response))
}

// ==================== CAUSAL INFERENCE ENDPOINTS ====================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Discover causal relationships
#[derive(Debug, Deserialize)]
pub struct CausalDiscoveryRequest {
    /// time series
    pub time_series: HashMap<String, Vec<f32>>,
}

#[derive(Debug, Serialize)]
/// `CausalRelationshipResponse` data
pub struct CausalRelationshipResponse {
    /// cause
    pub cause: String,
    /// effect
    pub effect: String,
    /// strength
    pub strength: f32,
    /// confidence
    pub confidence: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn discover_causal(
    State(state): State<ResearchState>,
    Json(request): Json<CausalDiscoveryRequest>,
) -> Result<Json<Vec<CausalRelationshipResponse>>, StatusCode> {
    let engine = state.engine.read().await;

    let relationships = engine
        .causal
        .discover_causal_relationships(&request.time_series);

    let response: Vec<CausalRelationshipResponse> = relationships
        .iter()
        .map(|r| CausalRelationshipResponse {
            cause: r.cause.clone(),
            effect: r.effect.clone(),
            strength: r.strength,
            confidence: r.confidence,
        })
        .collect();

    Ok(Json(response))
}

// ==================== BAYESIAN VALIDATION ENDPOINTS ====================

/// Compare models using Bayesian methods
#[derive(Debug, Deserialize)]
pub struct ModelComparisonRequest {
    /// models
    pub models: Vec<BayesianModelRequest>,
    /// data
    pub data: Vec<f64>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Deserialize)]
/// `BayesianModelRequest` data
pub struct BayesianModelRequest {
    /// name
    pub name: String,
    /// parameters
    pub parameters: Vec<ParameterRequest>,
}

#[derive(Debug, Deserialize)]
/// `ParameterRequest` data
pub struct ParameterRequest {
    /// name
    pub name: String,
    /// prior mean
    pub prior_mean: f64,
    /// prior std
    pub prior_std: f64,
}

#[derive(Debug, Serialize)]
/// `ModelComparisonResponse` data
pub struct ModelComparisonResponse {
    /// fits
    pub fits: Vec<ModelFitResponse>,
    /// best model
    pub best_model: String,
}

#[derive(Debug, Serialize)]
/// `ModelFitResponse` data
pub struct ModelFitResponse {
    /// model name
    pub model_name: String,
    /// bic
    pub bic: f64,
    /// n parameters
    pub n_parameters: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn compare_models(
    State(state): State<ResearchState>,
    Json(request): Json<ModelComparisonRequest>,
) -> Result<Json<ModelComparisonResponse>, StatusCode> {
    let engine = state.engine.read().await;

    let models: Vec<BayesianModel> = request
        .models
        .iter()
        .map(|m| BayesianModel {
            name: m.name.clone(),
            parameters: m
                .parameters
                .iter()
                .map(|p| sim_core::research::Parameter {
                    name: p.name.clone(),
                    prior_mean: p.prior_mean,
                    prior_std: p.prior_std,
                    posterior_mean: None,
                    posterior_std: None,
                })
                .collect(),
            likelihood: sim_core::research::LikelihoodFunction::Gaussian,
            prior: sim_core::research::PriorDistribution::Gaussian,
        })
        .collect();

    let comparison = engine.bayesian.compare_models(&models, &request.data);

    let response = ModelComparisonResponse {
        fits: comparison
            .fits
            .iter()
            .map(|f| ModelFitResponse {
                model_name: f.model_name.clone(),
                bic: f.bic,
                n_parameters: f.n_parameters,
            })
            .collect(),
        best_model: comparison.best_model,
    };

    Ok(Json(response))
}

// ==================== RESEARCH REPORT ENDPOINT ====================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get research report
#[derive(Debug, Serialize)]
pub struct ResearchReportResponse {
    /// active hypotheses
    pub active_hypotheses: usize,
    /// supported hypotheses
    pub supported_hypotheses: usize,
    /// literature entries
    pub literature_entries: usize,
    /// causal relationships
    pub causal_relationships: usize,
}

/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_research_report(
    State(state): State<ResearchState>,
) -> Result<Json<ResearchReportResponse>, StatusCode> {
    let engine = state.engine.read().await;
    let report = engine.generate_report();

    let response = ResearchReportResponse {
        active_hypotheses: report.active_hypotheses,
        supported_hypotheses: report.supported_hypotheses,
        literature_entries: report.literature_entries,
        causal_relationships: report.causal_relationships,
    };

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_research_state() {
        let state = ResearchState::new();
        let engine = state.engine.read().await;
        let report = engine.generate_report();
        assert_eq!(report.active_hypotheses, 0);
    }
}
