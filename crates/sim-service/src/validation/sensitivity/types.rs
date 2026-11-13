//! Types for sensitivity analysis

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Sensitivity analysis method
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SensitivityMethod {
    /// Local sensitivity (one-at-a-time)
    Local,
    /// Morris screening method
    Morris,
    /// Sobol variance-based decomposition
    Sobol,
    /// Tornado diagram (range-based)
    Tornado,
}

/// Configuration for sensitivity analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivityConfig {
    /// Analysis method
    pub method: SensitivityMethod,
    /// Parameters to analyze
    pub parameters: Vec<SensitivityParameter>,
    /// Output metrics to measure sensitivity for
    pub outputs: Vec<String>,
    /// Number of samples/replications
    pub samples: usize,
    /// Random seed for reproducibility
    pub seed: Option<u64>,
    /// Confidence level for intervals (default: 0.95)
    pub confidence_level: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for SensitivityConfig {
    fn default() -> Self {
        Self {
            method: SensitivityMethod::Local,
            parameters: Vec::new(),
            outputs: vec!["output".to_string()],
            samples: 100,
            seed: None,
            confidence_level: 0.95,
        }
    }
}

/// Parameter configuration for sensitivity analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivityParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: ParameterType,
    /// Baseline/nominal value
    pub baseline: f64,
    /// Lower bound
    pub lower: f64,
    /// Upper bound
    pub upper: f64,
    /// Distribution type
    pub distribution: DistributionType,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Parameter type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ParameterType {
    /// Continuous
    Continuous,
    /// Integer
    Integer,
    /// Categorical
    Categorical,
    /// Binary
    Binary,
}

/// Distribution for parameter sampling
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DistributionType {
    /// Uniform
    Uniform,
    /// Normal
    Normal,
    /// `LogNormal`
    LogNormal,
    /// Triangular
    Triangular,
    /// Beta
    Beta,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Sensitivity analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivityResult {
    /// Analysis method used
    pub method: SensitivityMethod,
    /// Results for each output
    pub output_results: HashMap<String, OutputSensitivity>,
    /// Overall ranking of parameters
    pub parameter_ranking: Vec<ParameterRank>,
    /// Total computation time (seconds)
    pub computation_time_secs: f64,
    /// Number of model runs
    pub total_runs: usize,
}

/// Sensitivity results for a single output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSensitivity {
    /// Output name
    pub output_name: String,
    /// Sensitivity indices for each parameter
    pub parameter_indices: HashMap<String, SensitivityIndex>,
    /// R-squared (variance explained) if available
    pub r_squared: Option<f64>,
    /// Confidence intervals
    pub confidence_intervals: HashMap<String, (f64, f64)>,
}

/// Sensitivity index for a parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivityIndex {
    /// Parameter name
    pub parameter: String,
    /// Primary sensitivity measure
    pub value: f64,
    /// Type of sensitivity index
    pub index_type: IndexType,
    /// Interpretation
    pub interpretation: SensitivityLevel,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Type of sensitivity index
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IndexType {
    /// Local sensitivity coefficient
    LocalCoefficient,
    /// Morris elementary effect mean (μ)
    MorrisMu,
    /// Morris elementary effect std dev (σ)
    MorrisSigma,
    /// First-order Sobol index (S1)
    SobolFirstOrder,
    /// Total-order Sobol index (ST)
    SobolTotal,
    /// Second-order Sobol index (S2)
    SobolSecondOrder,
    /// Tornado correlation coefficient
    TornadoCorrelation,
    /// Tornado regression coefficient
    TornadoRegression,
    /// Partial correlation coefficient
    PartialCorrelation,
    /// Standardized regression coefficient
    StandardizedRegression,
}

/// Sensitivity level classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SensitivityLevel {
    /// Negligible effect (< 0.05 for normalized indices)
    Negligible,
    /// Low sensitivity (0.05 - 0.15)
    Low,
    /// Medium sensitivity (0.15 - 0.30)
    Medium,
    /// High sensitivity (> 0.30)
    High,
    /// Very high sensitivity (> 0.50)
    VeryHigh,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl SensitivityLevel {
    /// Classify sensitivity value (assumes normalized 0-1 scale)
    #[must_use]
    pub fn from_value(value: f64) -> Self {
        if !value.is_finite() {
            return Self::Negligible;
        }
        let abs_val = value.abs();
        match abs_val {
            v if v < 0.05 => Self::Negligible,
            v if v < 0.15 => Self::Low,
            v if v < 0.30 => Self::Medium,
            v if v < 0.50 => Self::High,
            _ => Self::VeryHigh,
        }
    }

    /// Get description of sensitivity level
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Negligible => "Parameter has negligible effect on output",
            Self::Low => "Parameter has low influence on output",
            Self::Medium => "Parameter has moderate influence on output",
            Self::High => "Parameter has strong influence on output",
            Self::VeryHigh => "Parameter dominates output variation",
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Parameter ranking by importance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterRank {
    /// Parameter name
    pub parameter: String,
    /// Rank (1 = most important)
    pub rank: usize,
    /// Average sensitivity across outputs
    pub avg_sensitivity: f64,
    /// Number of outputs where this parameter is significant
    pub significant_outputs: usize,
}

/// Compute parameter ranking across sensitivity outputs
#[must_use]
pub fn rank_parameters(output_results: &HashMap<String, OutputSensitivity>) -> Vec<ParameterRank> {
    if output_results.is_empty() {
        return Vec::new();
    }
    debug_assert!(!output_results.is_empty(), "ranking inputs non-empty");
    let mut param_values: std::collections::BTreeMap<String, Vec<f64>> =
        std::collections::BTreeMap::new();

    // Collect index values across outputs in deterministic output and parameter order (SIM-004)
    let mut sorted_outputs: Vec<&String> = output_results.keys().collect();
    sorted_outputs.sort();
    for output in sorted_outputs {
        if let Some(output_sens) = output_results.get(output) {
            let mut sorted_params: Vec<&String> = output_sens.parameter_indices.keys().collect();
            sorted_params.sort();
            for param_name in sorted_params {
                if let Some(index) = output_sens.parameter_indices.get(param_name) {
                    param_values
                        .entry(param_name.clone())
                        .or_default()
                        .push(index.value);
                }
            }
        }
    }

    // Compute averages
    let mut ranks: Vec<ParameterRank> = param_values
        .into_iter()
        .map(|(param, values)| {
            let clean: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
            let avg = if clean.is_empty() {
                0.0
            } else {
                clean.iter().sum::<f64>() / clean.len() as f64
            };
            let significant = clean
                .iter()
                .filter(|&&v| SensitivityLevel::from_value(v) != SensitivityLevel::Negligible)
                .count();
            ParameterRank {
                parameter: param,
                rank: 0,
                avg_sensitivity: avg,
                significant_outputs: significant,
            }
        })
        .collect();

    // Sort by average sensitivity (descending) with deterministic parameter tie-breaking (SIM-004)
    ranks.sort_by(|a, b| {
        b.avg_sensitivity
            .partial_cmp(&a.avg_sensitivity)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.parameter.cmp(&b.parameter))
    });

    // Assign ranks
    for (i, rank) in ranks.iter_mut().enumerate() {
        rank.rank = i + 1;
    }

    ranks
}

/// Compute parameter ranking across sensitivity outputs, filtering out keys with the given suffix (e.g. "_sigma", "_S1")
#[must_use]
pub fn rank_parameters_excluding(
    output_results: &HashMap<String, OutputSensitivity>,
    exclude_suffix: &str,
) -> Vec<ParameterRank> {
    if output_results.is_empty() {
        return Vec::new();
    }
    let mut param_values: std::collections::BTreeMap<String, Vec<f64>> =
        std::collections::BTreeMap::new();

    let mut sorted_outputs: Vec<&String> = output_results.keys().collect();
    sorted_outputs.sort();
    for output in sorted_outputs {
        if let Some(output_sens) = output_results.get(output) {
            let mut sorted_params: Vec<&String> = output_sens.parameter_indices.keys().collect();
            sorted_params.sort();
            for param_name in sorted_params {
                if !param_name.ends_with(exclude_suffix) {
                    if let Some(index) = output_sens.parameter_indices.get(param_name) {
                        param_values
                            .entry(param_name.clone())
                            .or_default()
                            .push(index.value);
                    }
                }
            }
        }
    }

    let mut ranks: Vec<ParameterRank> = param_values
        .into_iter()
        .map(|(param, values)| {
            let clean: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
            let avg = if clean.is_empty() {
                0.0
            } else {
                clean.iter().sum::<f64>() / clean.len() as f64
            };
            let significant = clean
                .iter()
                .filter(|&&v| SensitivityLevel::from_value(v) != SensitivityLevel::Negligible)
                .count();
            ParameterRank {
                parameter: param,
                rank: 0,
                avg_sensitivity: avg,
                significant_outputs: significant,
            }
        })
        .collect();

    ranks.sort_by(|a, b| {
        b.avg_sensitivity
            .partial_cmp(&a.avg_sensitivity)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.parameter.cmp(&b.parameter))
    });

    for (i, rank) in ranks.iter_mut().enumerate() {
        rank.rank = i + 1;
    }

    ranks
}

/// Local sensitivity configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSensitivityConfig {
    /// Perturbation fraction (e.g., 0.10 for 10%)
    pub perturbation: f64,
    /// Number of points for finite differences
    pub num_points: usize,
    /// Use central differences
    pub central_differences: bool,
}

impl Default for LocalSensitivityConfig {
    fn default() -> Self {
        Self {
            perturbation: 0.10,
            num_points: 3,
            central_differences: true,
        }
    }
}

/// Morris method configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorrisConfig {
    /// Number of trajectories
    pub num_trajectories: usize,
    /// Number of levels in grid
    pub num_levels: usize,
    /// Delta (perturbation size)
    pub delta: Option<f64>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for MorrisConfig {
    fn default() -> Self {
        Self {
            num_trajectories: 10,
            num_levels: 4,
            delta: None,
        }
    }
}

/// Sobol method configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SobolConfig {
    /// Sample size (N)
    pub n: usize,
    /// Calculate second-order indices
    pub calc_second_order: bool,
    /// Number of bootstrap resamples for confidence intervals
    pub num_bootstrap: usize,
}

impl Default for SobolConfig {
    fn default() -> Self {
        Self {
            n: 1024,
            calc_second_order: false,
            num_bootstrap: 100,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Tornado diagram data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TornadoData {
    /// Output name
    pub output_name: String,
    /// Bars for each parameter
    pub bars: Vec<TornadoBar>,
    /// Baseline output value
    pub baseline_value: f64,
    /// Units for output
    pub units: Option<String>,
}

/// Single bar in tornado diagram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TornadoBar {
    /// Parameter name
    pub parameter: String,
    /// Low value (parameter at lower bound)
    pub low_value: f64,
    /// High value (parameter at upper bound)
    pub high_value: f64,
    /// Swing (max - min)
    pub swing: f64,
    /// Relative swing (normalized)
    pub relative_swing: f64,
}

/// Simulation run result for sensitivity analysis
#[derive(Debug, Clone)]
pub struct SimulationRun {
    /// Parameter values
    pub parameters: HashMap<String, f64>,
    /// Output values
    pub outputs: HashMap<String, f64>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Sensitivity analysis request for API
#[derive(Debug, Deserialize)]
pub struct SensitivityAnalysisRequest {
    /// config
    pub config: SensitivityConfig,
    /// experiment id
    pub experiment_id: Option<String>,
    /// name
    pub name: Option<String>,
}

/// Sensitivity analysis response for API
#[derive(Debug, Serialize)]
pub struct SensitivityAnalysisResponse {
    /// analysis id
    pub analysis_id: String,
    /// status
    pub status: String,
    /// method
    pub method: String,
    /// estimated duration secs
    pub estimated_duration_secs: u64,
}

/// Sensitivity analysis status response
#[derive(Debug, Serialize)]
pub struct SensitivityStatusResponse {
    /// analysis id
    pub analysis_id: String,
    /// status
    pub status: String,
    /// progress percent
    pub progress_percent: f64,
    /// completed runs
    pub completed_runs: usize,
    /// total runs
    pub total_runs: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Detailed sensitivity result response
#[derive(Debug, Serialize)]
pub struct SensitivityResultResponse {
    /// analysis id
    pub analysis_id: String,
    /// result
    pub result: SensitivityResult,
    /// visualizations
    pub visualizations: Vec<SensitivityVisualization>,
}

/// Sensitivity visualization data
#[derive(Debug, Serialize)]
pub struct SensitivityVisualization {
    /// viz type
    pub viz_type: String,
    /// output
    pub output: String,
    /// data
    pub data: serde_json::Value,
}

/// Compare sensitivity across scenarios
#[derive(Debug, Deserialize)]
pub struct ScenarioComparisonRequest {
    /// base config
    pub base_config: SensitivityConfig,
    /// scenario configs
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub scenario_configs: Vec<(String, SensitivityConfig)>,
}

#[derive(Debug, Serialize)]
/// `ScenarioComparisonResponse` data
pub struct ScenarioComparisonResponse {
    /// base result
    pub base_result: SensitivityResult,
    /// scenario results
    pub scenario_results: HashMap<String, SensitivityResult>,
    /// stability analysis
    pub stability_analysis: HashMap<String, ParameterStability>,
}

/// Parameter stability across scenarios
#[derive(Debug, Serialize)]
pub struct ParameterStability {
    /// parameter
    pub parameter: String,
    /// base rank
    pub base_rank: usize,
    /// rank variance
    pub rank_variance: f64,
    /// sensitivity range
    pub sensitivity_range: (f64, f64),
    /// is stable
    pub is_stable: bool,
}
