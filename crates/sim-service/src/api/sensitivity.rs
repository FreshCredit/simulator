//! Sensitivity Analysis API Module (P1-VAL-2)
//!
//! REST endpoints for parameter sensitivity analysis using various methods.

use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::validation::sensitivity::{
    local::LocalSensitivityAnalyzer, morris::MorrisAnalyzer, sobol::SobolAnalyzer,
    tornado::TornadoDiagram, types::*,
};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Sensitivity API state
#[derive(Clone, Debug)]
pub struct SensitivityState;

impl SensitivityState {
    #[must_use]
    /// fn
    pub const fn new() -> Self {
        Self
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for SensitivityState {
    fn default() -> Self {
        Self::new()
    }
}

// Helper function to create a simple model for demo purposes
fn create_model(output_name: String) -> impl Fn(&HashMap<String, f64>) -> HashMap<String, f64> {
    move |inputs: &HashMap<String, f64>| -> HashMap<String, f64> {
        let mut outputs = HashMap::new();
        let mut sorted_keys: Vec<&String> = inputs.keys().collect();
        sorted_keys.sort();
        let mut y = 0.0;
        for name in sorted_keys {
            let value = inputs.get(name).copied().unwrap_or(0.0);
            let coeff = match name.as_str() {
                "learning_rate" => 2.0,
                "mutation_rate" => 1.5,
                "population_size" => 0.5,
                _ => 1.0,
            };
            y += coeff * value;
        }
        outputs.insert(output_name.clone(), y);
        outputs
    }
}

/// Run local sensitivity analysis
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn run_local_analysis(
    Json(request): Json<LocalAnalysisRequest>,
) -> Result<Json<SensitivityResult>, StatusCode> {
    if request.parameters.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    if request.parameters.len() > 32 {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    tokio::task::spawn_blocking(move || {
        let analyzer = LocalSensitivityAnalyzer::new(request.config);
        let model = create_model(request.output.clone());
        Json(analyzer.analyze(&request.parameters, &[request.output], model))
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Run Morris screening analysis
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn run_morris_analysis(
    Json(request): Json<MorrisAnalysisRequest>,
) -> Result<Json<SensitivityResult>, StatusCode> {
    if request.parameters.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    if request.parameters.len() > 32 || request.config.num_trajectories > 1_000 {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    tokio::task::spawn_blocking(move || {
        let analyzer = MorrisAnalyzer::new(request.config);
        let model = create_model(request.output.clone());
        Json(analyzer.analyze(&request.parameters, &[request.output], model, request.seed))
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Run Sobol variance-based analysis
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn run_sobol_analysis(
    Json(request): Json<SobolAnalysisRequest>,
) -> Result<Json<SensitivityResult>, StatusCode> {
    if request.parameters.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    if request.parameters.len() > 32
        || request.config.n > 4_096
        || request.config.num_bootstrap > 1_000
    {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    tokio::task::spawn_blocking(move || {
        let analyzer = SobolAnalyzer::new(request.config);
        let model = create_model(request.output.clone());
        Json(analyzer.analyze(&request.parameters, &[request.output], model, request.seed))
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Generate tornado diagram data
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn generate_tornado(
    Json(request): Json<TornadoRequest>,
) -> Result<Json<TornadoResponse>, StatusCode> {
    let tornado = TornadoDiagram::new();
    let model = create_model(request.output.clone());

    let tornado_data = tornado.generate(
        &request.parameters,
        std::slice::from_ref(&request.output),
        model,
    );

    // Convert to sensitivity result for consistent response
    let result = tornado.to_sensitivity_result(tornado_data, 0.0, request.parameters.len() * 2);

    Ok(Json(TornadoResponse {
        tornado_data: result,
        output: request.output,
    }))
}

/// Generate spider plot data
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn generate_spider_plot(
    Json(request): Json<SpiderPlotRequest>,
) -> Result<Json<SpiderPlotResponse>, StatusCode> {
    let tornado = TornadoDiagram::new();
    let model = create_model(request.output.clone());

    let spider_data = tornado.generate_spider_data(
        &request.parameters,
        std::slice::from_ref(&request.output),
        model,
        request.num_points,
    );

    Ok(Json(SpiderPlotResponse {
        curves: spider_data,
        output: request.output,
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get sensitivity methods available
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn list_methods() -> Result<Json<MethodsResponse>, StatusCode> {
    let methods = vec![
        MethodInfo {
            id: "local".to_string(),
            name: "Local Sensitivity".to_string(),
            description: "One-at-a-time perturbation analysis".to_string(),
            complexity: "O(k)".to_string(),
            best_for: "Linear models, screening".to_string(),
        },
        MethodInfo {
            id: "morris".to_string(),
            name: "Morris Screening".to_string(),
            description: "Elementary effects method for screening".to_string(),
            complexity: "O(r(k+1))".to_string(),
            best_for: "Nonlinear models, interactions".to_string(),
        },
        MethodInfo {
            id: "sobol".to_string(),
            name: "Sobol Indices".to_string(),
            description: "Variance-based global sensitivity".to_string(),
            complexity: "O(N(k+2))".to_string(),
            best_for: "Accurate decomposition, interactions".to_string(),
        },
        MethodInfo {
            id: "tornado".to_string(),
            name: "Tornado Diagram".to_string(),
            description: "Range-based sensitivity visualization".to_string(),
            complexity: "O(2k)".to_string(),
            best_for: "Visualization, ranking".to_string(),
        },
    ];

    Ok(Json(MethodsResponse { methods }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get default parameter templates for sensitivity
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn get_parameter_templates() -> Result<Json<ParameterTemplatesResponse>, StatusCode> {
    let templates = vec![
        ParameterTemplate {
            name: "Learning Rate".to_string(),
            param_type: ParameterType::Continuous,
            default_range: (0.001, 0.1),
            typical_baseline: 0.01,
        },
        ParameterTemplate {
            name: "Mutation Rate".to_string(),
            param_type: ParameterType::Continuous,
            default_range: (0.001, 0.1),
            typical_baseline: 0.05,
        },
        ParameterTemplate {
            name: "Population Size".to_string(),
            param_type: ParameterType::Integer,
            default_range: (50.0, 500.0),
            typical_baseline: 100.0,
        },
        ParameterTemplate {
            name: "Selection Pressure".to_string(),
            param_type: ParameterType::Continuous,
            default_range: (1.5, 5.0),
            typical_baseline: 2.0,
        },
    ];

    Ok(Json(ParameterTemplatesResponse { templates }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compare sensitivity across methods
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn compare_methods(
    Json(request): Json<MethodComparisonRequest>,
) -> Result<Json<MethodComparisonResponse>, StatusCode> {
    if request.parameters.is_empty() || request.methods.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    if request.parameters.len() > 32 || request.methods.len() > 8 {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    tokio::task::spawn_blocking(move || {
        let mut results = HashMap::new();
        let output = request.output.clone();

        for method in &request.methods {
            let result = match method.as_str() {
                "local" => {
                    let analyzer = LocalSensitivityAnalyzer::default();
                    let model = create_model(output.clone());
                    analyzer.analyze(&request.parameters, std::slice::from_ref(&output), model)
                }
                "morris" => {
                    let analyzer = MorrisAnalyzer::default();
                    let model = create_model(output.clone());
                    analyzer.analyze(
                        &request.parameters,
                        std::slice::from_ref(&output),
                        model,
                        Some(42),
                    )
                }
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                "sobol" => {
                    let config = SobolConfig {
                        n: 512,
                        calc_second_order: false,
                        num_bootstrap: 0,
                    };
                    let analyzer = SobolAnalyzer::new(config);
                    let model = create_model(output.clone());
                    analyzer.analyze(
                        &request.parameters,
                        std::slice::from_ref(&output),
                        model,
                        Some(42),
                    )
                }
                "tornado" => {
                    let tornado = TornadoDiagram::new();
                    let model = create_model(output.clone());
                    let data =
                        tornado.generate(&request.parameters, std::slice::from_ref(&output), model);
                    tornado.to_sensitivity_result(data, 0.0, request.parameters.len() * 2)
                }
                _ => continue,
            };

            results.insert(method.clone(), result);
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Compare rankings with deterministic method ordering
        let mut agreement = HashMap::new();
        let mut method_names: Vec<_> = results.keys().cloned().collect();
        method_names.sort();

        for i in 0..method_names.len() {
            for j in (i + 1)..method_names.len() {
                let m1 = &method_names[i];
                let m2 = &method_names[j];

                if let (Some(r1), Some(r2)) = (results.get(m1), results.get(m2)) {
                    let kendall_tau =
                        compute_kendall_tau(&r1.parameter_ranking, &r2.parameter_ranking);
                    agreement.insert(format!("{m1}_vs_{m2}"), kendall_tau);
                }
            }
        }

        Json(MethodComparisonResponse {
            results,
            ranking_agreement: agreement,
        })
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Compute Kendall's tau rank correlation with exact tie handling
fn compute_kendall_tau(ranks1: &[ParameterRank], ranks2: &[ParameterRank]) -> f64 {
    let n = ranks1.len();
    if n < 2 {
        return 0.0;
    }

    let mut concordant = 0;
    let mut discordant = 0;

    for i in 0..n {
        for j in (i + 1)..n {
            let param_i = &ranks1[i].parameter;
            let param_j = &ranks1[j].parameter;

            // TAG: surface=simulation owner=research-team rule=SIM-001
            let rank1_i = ranks1
                .iter()
                .position(|r| &r.parameter == param_i)
                .unwrap_or(n);
            let rank1_j = ranks1
                .iter()
                .position(|r| &r.parameter == param_j)
                .unwrap_or(n);
            let rank2_i = ranks2
                .iter()
                .position(|r| &r.parameter == param_i)
                .unwrap_or(n);
            let rank2_j = ranks2
                .iter()
                .position(|r| &r.parameter == param_j)
                .unwrap_or(n);

            let sign1 = (rank1_i as i32 - rank1_j as i32).signum();
            let sign2 = (rank2_i as i32 - rank2_j as i32).signum();

            if sign1 != 0 && sign2 != 0 {
                if sign1 == sign2 {
                    concordant += 1;
                } else {
                    discordant += 1;
                }
            }
        }
    }

    let total = concordant + discordant;
    if total == 0 {
        0.0
    } else {
        (concordant as f64 - discordant as f64) / total as f64
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[tokio::test]
    async fn test_list_methods() {
        let response = list_methods().await.unwrap();
        assert_eq!(response.0.methods.len(), 4);
        assert_eq!(response.0.methods[0].id, "local");
        assert_eq!(response.0.methods[1].id, "morris");
        assert_eq!(response.0.methods[2].id, "sobol");
        assert_eq!(response.0.methods[3].id, "tornado");
    }

    #[tokio::test]
    async fn test_get_parameter_templates() {
        let response = get_parameter_templates().await.unwrap();
        assert_eq!(response.0.templates.len(), 4);
        assert_eq!(response.0.templates[0].name, "Learning Rate");
        assert_eq!(response.0.templates[1].name, "Mutation Rate");
        assert_eq!(response.0.templates[2].name, "Population Size");
        assert_eq!(response.0.templates[3].name, "Selection Pressure");
    }

    #[tokio::test]
    async fn test_compute_kendall_tau_perfect_agreement() {
        let ranks1 = vec![
            ParameterRank {
                parameter: "a".to_string(),
                rank: 1,
                avg_sensitivity: 0.5,
                significant_outputs: 1,
            },
            ParameterRank {
                parameter: "b".to_string(),
                rank: 2,
                avg_sensitivity: 0.3,
                significant_outputs: 1,
            },
            ParameterRank {
                parameter: "c".to_string(),
                rank: 3,
                avg_sensitivity: 0.1,
                significant_outputs: 1,
            },
        ];
        let tau = compute_kendall_tau(&ranks1, &ranks1);
        assert_eq!(tau, 1.0);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[tokio::test]
    async fn test_compute_kendall_tau_perfect_disagreement() {
        let ranks1 = vec![
            ParameterRank {
                parameter: "a".to_string(),
                rank: 1,
                avg_sensitivity: 0.5,
                significant_outputs: 1,
            },
            ParameterRank {
                parameter: "b".to_string(),
                rank: 2,
                avg_sensitivity: 0.3,
                significant_outputs: 1,
            },
        ];
        let ranks2 = vec![
            ParameterRank {
                parameter: "b".to_string(),
                rank: 1,
                avg_sensitivity: 0.3,
                significant_outputs: 1,
            },
            ParameterRank {
                parameter: "a".to_string(),
                rank: 2,
                avg_sensitivity: 0.5,
                significant_outputs: 1,
            },
        ];
        let tau = compute_kendall_tau(&ranks1, &ranks2);
        assert_eq!(tau, -1.0);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[tokio::test]
    async fn test_compute_kendall_tau_insufficient_data() {
        let ranks1 = vec![ParameterRank {
            parameter: "a".to_string(),
            rank: 1,
            avg_sensitivity: 0.5,
            significant_outputs: 1,
        }];
        let tau = compute_kendall_tau(&ranks1, &ranks1);
        assert_eq!(tau, 0.0);
    }
}

// Request/Response types

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Deserialize)]
/// `LocalAnalysisRequest` data
pub struct LocalAnalysisRequest {
    /// config
    pub config: LocalSensitivityConfig,
    /// parameters
    pub parameters: Vec<SensitivityParameter>,
    /// output
    pub output: String,
}

#[derive(Debug, Deserialize)]
/// `MorrisAnalysisRequest` data
pub struct MorrisAnalysisRequest {
    /// config
    pub config: MorrisConfig,
    /// parameters
    pub parameters: Vec<SensitivityParameter>,
    /// output
    pub output: String,
    /// seed
    pub seed: Option<u64>,
}

#[derive(Debug, Deserialize)]
/// `SobolAnalysisRequest` data
pub struct SobolAnalysisRequest {
    /// config
    pub config: SobolConfig,
    /// parameters
    pub parameters: Vec<SensitivityParameter>,
    /// output
    pub output: String,
    /// seed
    pub seed: Option<u64>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Deserialize)]
/// `TornadoRequest` data
pub struct TornadoRequest {
    /// parameters
    pub parameters: Vec<SensitivityParameter>,
    /// output
    pub output: String,
}

#[derive(Debug, Serialize)]
/// `TornadoResponse` data
pub struct TornadoResponse {
    /// tornado data
    pub tornado_data: SensitivityResult,
    /// output
    pub output: String,
}

#[derive(Debug, Deserialize)]
/// `SpiderPlotRequest` data
pub struct SpiderPlotRequest {
    /// parameters
    pub parameters: Vec<SensitivityParameter>,
    /// output
    pub output: String,
    /// num points
    pub num_points: usize,
}

#[derive(Debug, Serialize)]
/// `SpiderPlotResponse` data
pub struct SpiderPlotResponse {
    /// curves
    pub curves: HashMap<String, HashMap<String, Vec<(f64, f64)>>>,
    /// output
    pub output: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Serialize)]
/// `MethodInfo` data
pub struct MethodInfo {
    /// id
    pub id: String,
    /// name
    pub name: String,
    /// description
    pub description: String,
    /// complexity
    pub complexity: String,
    /// best for
    pub best_for: String,
}

#[derive(Debug, Serialize)]
/// `MethodsResponse` data
pub struct MethodsResponse {
    /// methods
    pub methods: Vec<MethodInfo>,
}

#[derive(Debug, Serialize)]
/// `ParameterTemplate` data
pub struct ParameterTemplate {
    /// name
    pub name: String,
    /// param type
    pub param_type: ParameterType,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// default range
    pub default_range: (f64, f64),
    /// typical baseline
    pub typical_baseline: f64,
}

#[derive(Debug, Serialize)]
/// `ParameterTemplatesResponse` data
pub struct ParameterTemplatesResponse {
    /// templates
    pub templates: Vec<ParameterTemplate>,
}

#[derive(Debug, Deserialize)]
/// `MethodComparisonRequest` data
pub struct MethodComparisonRequest {
    /// methods
    pub methods: Vec<String>,
    /// parameters
    pub parameters: Vec<SensitivityParameter>,
    /// output
    pub output: String,
}

#[derive(Debug, Serialize)]
/// `MethodComparisonResponse` data
pub struct MethodComparisonResponse {
    /// results
    pub results: HashMap<String, SensitivityResult>,
    /// ranking agreement
    pub ranking_agreement: HashMap<String, f64>,
}
