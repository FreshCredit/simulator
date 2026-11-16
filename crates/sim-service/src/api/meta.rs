//! Meta-Analysis API Module (P1-VAL-4)
//!
//! REST endpoints for meta-analysis and simulation-empirical comparison.

use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::validation::meta::{
    analysis::MetaAnalyzer, forest::ForestPlot, funnel::FunnelPlot, types::*,
};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Run meta-analysis
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn run_meta_analysis(
    Json(request): Json<RunMetaAnalysisRequest>,
) -> Result<Json<MetaAnalysisResult>, StatusCode> {
    if request.studies.len() > 5_000 {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    tokio::task::spawn_blocking(move || {
        let analyzer = MetaAnalyzer::new(request.config);
        Json(analyzer.analyze(&request.studies))
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compare simulation vs empirical studies
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn compare_simulation_empirical(
    Json(request): Json<SimulationEmpiricalRequest>,
) -> Result<Json<SimulationEmpiricalComparison>, StatusCode> {
    if request.empirical_studies.len() > 5_000 || request.simulation_studies.len() > 5_000 {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    tokio::task::spawn_blocking(move || {
        let analyzer = MetaAnalyzer::new(MetaAnalysisConfig::default());
        Json(
            analyzer.compare_simulation_empirical(
                &request.empirical_studies,
                &request.simulation_studies,
            ),
        )
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Generate forest plot data
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn generate_forest_plot(
    Json(request): Json<ForestPlotRequest>,
) -> Result<Json<ForestPlotData>, StatusCode> {
    let analyzer = MetaAnalyzer::new(request.config);
    let result = analyzer.analyze(&request.studies);

    let plot = ForestPlot::new();
    let data = plot.generate(&result);

    Ok(Json(data))
}

/// Generate funnel plot data
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn generate_funnel_plot(
    Json(request): Json<FunnelPlotRequest>,
) -> Result<Json<FunnelPlotData>, StatusCode> {
    let analyzer = MetaAnalyzer::new(request.config);
    let result = analyzer.analyze(&request.studies);

    let plot = FunnelPlot::new();
    let data = plot.generate(&result);

    Ok(Json(data))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get meta-analysis models
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn list_models() -> Result<Json<ModelsResponse>, StatusCode> {
    let models = vec![
        ModelInfo {
            id: "fixed_effect".to_string(),
            name: "Fixed-Effect Model".to_string(),
            description: "Assumes single true effect across all studies".to_string(),
            use_case: "When studies are functionally identical".to_string(),
        },
        ModelInfo {
            id: "random_effects".to_string(),
            name: "Random-Effects Model".to_string(),
            description: "Assumes distribution of true effects".to_string(),
            use_case: "When studies estimate different but related effects".to_string(),
        },
        ModelInfo {
            id: "mixed".to_string(),
            name: "Mixed-Effects Model".to_string(),
            description: "Combines fixed and random effects with moderators".to_string(),
            use_case: "When exploring moderators/subgroups".to_string(),
        },
    ];

    Ok(Json(ModelsResponse { models }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get heterogeneity measures info
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn list_heterogeneity_methods() -> Result<Json<HeterogeneityMethodsResponse>, StatusCode>
{
    let methods = vec![
        HeterogeneityMethodInfo {
            id: "dl".to_string(),
            name: "DerSimonian-Laird".to_string(),
            description: "Non-iterative method-of-moments estimator".to_string(),
        },
        HeterogeneityMethodInfo {
            id: "reml".to_string(),
            name: "REML".to_string(),
            description: "Restricted Maximum Likelihood".to_string(),
        },
        HeterogeneityMethodInfo {
            id: "ml".to_string(),
            name: "Maximum Likelihood".to_string(),
            description: "Full Maximum Likelihood".to_string(),
        },
        HeterogeneityMethodInfo {
            id: "hs".to_string(),
            name: "Hunter-Schmidt".to_string(),
            description: "Observed variance minus sampling variance".to_string(),
        },
    ];

    Ok(Json(HeterogeneityMethodsResponse { methods }))
}

/// Subgroup analysis
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn subgroup_analysis(
    Json(request): Json<SubgroupAnalysisRequest>,
) -> Result<Json<SubgroupResult>, StatusCode> {
    let config = MetaAnalysisConfig::default();
    let analyzer = MetaAnalyzer::new(config);

    // Group studies by variable
    let mut categories: HashMap<String, Vec<Study>> = HashMap::new();
    for study in &request.studies {
        if let Some(value) = study.metadata.get(&request.variable) {
            categories
                .entry(value.clone())
                .or_default()
                .push(study.clone());
        }
    }

    // Analyze each category with deterministic sorting by category name (SIM-004)
    let mut category_names: Vec<String> = categories.keys().cloned().collect();
    category_names.sort();

    let mut category_results = Vec::new();
    let mut sum_subgroup_q = 0.0;
    for category in &category_names {
        let studies = &categories[category];
        let result = analyzer.analyze(studies);
        sum_subgroup_q += result.heterogeneity.q;
        category_results.push(SubgroupCategory {
            category: category.clone(),
            k: result.k,
            pooled_effect: result.pooled_effect,
            ci_lower: result.confidence_interval.0,
            ci_upper: result.confidence_interval.1,
            i_squared: result.heterogeneity.i_squared,
        });
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Calculate between-group Q strictly over categorized studies
    let categorized_studies: Vec<Study> = categories.values().flatten().cloned().collect();
    let overall = analyzer.analyze(&categorized_studies);
    let between_q = (overall.heterogeneity.q - sum_subgroup_q).max(0.0);
    let df_between = category_results.len().saturating_sub(1);

    // Approximate upper-tail p-value via Wilson-Hilferty (1931) chi-square normal transformation
    let between_p = if df_between > 0 && between_q > 0.0 {
        let df = df_between as f64;
        let factor = 2.0 / (9.0 * df);
        let z = ((between_q / df).cbrt() - (1.0 - factor)) / factor.sqrt();
        (1.0 - normal_cdf_approx(z)).clamp(0.0, 1.0)
    } else {
        1.0
    };

    Ok(Json(SubgroupResult {
        variable: request.variable,
        categories: category_results,
        between_group_q: between_q,
        between_group_p: between_p.min(1.0),
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Simple normal CDF approximation using Abramowitz and Stegun 7.1.26 via erf(z / sqrt(2))
/// # Errors
///
/// Returns an error if the operation fails.
pub(crate) fn normal_cdf_approx(z: f64) -> f64 {
    if !z.is_finite() {
        return if z.is_sign_negative() { 0.0 } else { 1.0 };
    }
    // Abramowitz and Stegun approximation
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if z < 0.0 { -1.0 } else { 1.0 };
    let x = z.abs() / std::f64::consts::SQRT_2;

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    0.5 * (1.0 + sign * y)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Cumulative meta-analysis
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn cumulative_meta_analysis(
    Json(request): Json<CumulativeMetaRequest>,
) -> Result<Json<CumulativeMetaResponse>, StatusCode> {
    let config = MetaAnalysisConfig::default();
    let analyzer = MetaAnalyzer::new(config);

    // Sort studies by year with deterministic ID tie-breaking (SIM-004)
    let mut sorted_studies = request.studies;
    sorted_studies.sort_by(|a, b| a.year.cmp(&b.year).then_with(|| a.id.cmp(&b.id)));

    let mut cumulative_results = Vec::new();
    let mut cumulative_n = 0;

    for i in 0..sorted_studies.len() {
        let subset: Vec<Study> = sorted_studies[..=i].to_vec();
        cumulative_n += sorted_studies[i].sample_size;

        let result = analyzer.analyze(&subset);

        cumulative_results.push(CumulativeMetaResult {
            after_study: sorted_studies[i].id.clone(),
            cumulative_n,
            pooled_effect: result.pooled_effect,
            ci_lower: result.confidence_interval.0,
            ci_upper: result.confidence_interval.1,
        });
    }

    Ok(Json(CumulativeMetaResponse {
        final_pooled_effect: cumulative_results.last().map_or(0.0, |r| r.pooled_effect),
        results: cumulative_results,
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Leave-one-out sensitivity analysis
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn leave_one_out_analysis(
    Json(request): Json<LeaveOneOutRequest>,
) -> Result<Json<LeaveOneOutResponse>, StatusCode> {
    let config = MetaAnalysisConfig::default();
    let analyzer = MetaAnalyzer::new(config);

    let full_result = analyzer.analyze(&request.studies);

    let mut results = Vec::new();

    for i in 0..request.studies.len() {
        let mut subset = request.studies.clone();
        let removed = subset.remove(i);

        let result = analyzer.analyze(&subset);

        results.push(LeaveOneOutResult {
            study_removed: removed.id,
            pooled_effect: result.pooled_effect,
            ci_lower: result.confidence_interval.0,
            ci_upper: result.confidence_interval.1,
            i_squared: result.heterogeneity.i_squared,
        });
    }

    Ok(Json(LeaveOneOutResponse {
        full_result,
        leave_one_out_results: results,
    }))
}

// Request/Response types

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Deserialize)]
/// `RunMetaAnalysisRequest` data
pub struct RunMetaAnalysisRequest {
    /// config
    pub config: MetaAnalysisConfig,
    /// studies
    pub studies: Vec<Study>,
}

#[derive(Debug, Deserialize)]
/// `SimulationEmpiricalRequest` data
pub struct SimulationEmpiricalRequest {
    /// empirical studies
    pub empirical_studies: Vec<Study>,
    /// simulation studies
    pub simulation_studies: Vec<Study>,
}

#[derive(Debug, Deserialize)]
/// `ForestPlotRequest` data
pub struct ForestPlotRequest {
    /// config
    pub config: MetaAnalysisConfig,
    /// studies
    pub studies: Vec<Study>,
}

#[derive(Debug, Deserialize)]
/// `FunnelPlotRequest` data
pub struct FunnelPlotRequest {
    /// config
    pub config: MetaAnalysisConfig,
    /// studies
    pub studies: Vec<Study>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Serialize)]
/// `ModelInfo` data
pub struct ModelInfo {
    /// id
    pub id: String,
    /// name
    pub name: String,
    /// description
    pub description: String,
    /// Re-export
    pub use_case: String,
}

#[derive(Debug, Serialize)]
/// `ModelsResponse` data
pub struct ModelsResponse {
    /// models
    pub models: Vec<ModelInfo>,
}

#[derive(Debug, Serialize)]
/// `HeterogeneityMethodInfo` data
pub struct HeterogeneityMethodInfo {
    /// id
    pub id: String,
    /// name
    pub name: String,
    /// description
    pub description: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Serialize)]
/// `HeterogeneityMethodsResponse` data
pub struct HeterogeneityMethodsResponse {
    /// methods
    pub methods: Vec<HeterogeneityMethodInfo>,
}

#[derive(Debug, Deserialize)]
/// `SubgroupAnalysisRequest` data
pub struct SubgroupAnalysisRequest {
    /// studies
    pub studies: Vec<Study>,
    /// variable
    pub variable: String,
}

#[derive(Debug, Deserialize)]
/// `CumulativeMetaRequest` data
pub struct CumulativeMetaRequest {
    /// studies
    pub studies: Vec<Study>,
}

#[derive(Debug, Serialize)]
/// `CumulativeMetaResponse` data
pub struct CumulativeMetaResponse {
    /// results
    pub results: Vec<CumulativeMetaResult>,
    /// final pooled effect
    pub final_pooled_effect: f64,
}
// TAG: surface=simulation owner=platform-team rule=GENERAL-001

#[derive(Debug, Deserialize)]
/// `LeaveOneOutRequest` data
pub struct LeaveOneOutRequest {
    /// studies
    pub studies: Vec<Study>,
}

#[derive(Debug, Serialize)]
/// `LeaveOneOutResponse` data
pub struct LeaveOneOutResponse {
    /// full result
    pub full_result: MetaAnalysisResult,
    /// leave one out results
    pub leave_one_out_results: Vec<LeaveOneOutResult>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_models() {
        let response = list_models().await.unwrap();
        assert!(!response.models.is_empty());
        assert_eq!(response.models[0].id, "fixed_effect");
    }

    #[tokio::test]
    async fn test_list_heterogeneity_methods() {
        let response = list_heterogeneity_methods().await.unwrap();
        assert!(!response.methods.is_empty());
    }
}
