//! Types for meta-analysis

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Meta-analysis model type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MetaAnalysisModel {
    /// Fixed-effect model (single true effect)
    FixedEffect,
    /// Random-effects model (distribution of effects)
    RandomEffects,
    /// Mixed model
    Mixed,
}

/// Configuration for meta-analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaAnalysisConfig {
    /// Analysis model
    pub model: MetaAnalysisModel,
    /// Confidence level
    pub confidence_level: f64,
    /// Method for heterogeneity
    pub heterogeneity_method: HeterogeneityMethod,
    /// Include prediction interval
    pub include_prediction_interval: bool,
    /// Publication bias test
    pub bias_test: PublicationBiasTest,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for MetaAnalysisConfig {
    fn default() -> Self {
        Self {
            model: MetaAnalysisModel::RandomEffects,
            confidence_level: 0.95,
            heterogeneity_method: HeterogeneityMethod::DerSimonianLaird,
            include_prediction_interval: true,
            bias_test: PublicationBiasTest::Egger,
        }
    }
}

/// Heterogeneity estimation method
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HeterogeneityMethod {
    /// DerSimonian-Laird estimator
    DerSimonianLaird,
    /// Hedges estimator
    Hedges,
    /// Hunter-Schmidt estimator
    HunterSchmidt,
    /// Restricted Maximum Likelihood (REML)
    REML,
    /// Maximum Likelihood (ML)
    MaximumLikelihood,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Publication bias test
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PublicationBiasTest {
    /// Egger's regression test
    Egger,
    /// Begg's rank test
    Begg,
    /// Trim-and-fill method
    TrimAndFill,
    /// None
    None,
}

/// A single study in meta-analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Study {
    /// Study identifier
    pub id: String,
    /// Study name/title
    pub name: String,
    /// Effect size
    pub effect_size: f64,
    /// Standard error
    pub standard_error: f64,
    /// Sample size
    pub sample_size: usize,
    /// Confidence interval (optional)
    pub confidence_interval: Option<(f64, f64)>,
    /// Year of publication
    pub year: Option<i32>,
    /// Study source (simulation, empirical, etc.)
    pub source: StudySource,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Source of study data
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StudySource {
    /// Simulation result
    Simulation,
    /// Empirical/observational study
    Empirical,
    /// Experimental study
    Experimental,
    /// Meta-analysis
    MetaAnalysis,
}

/// Meta-analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaAnalysisResult {
    /// Configuration used
    pub config: MetaAnalysisConfig,
    /// Number of studies
    pub k: usize,
    /// Total sample size
    pub total_n: usize,
    /// Pooled effect size
    pub pooled_effect: f64,
    /// Standard error of pooled effect
    pub pooled_se: f64,
    /// Confidence interval for pooled effect
    pub confidence_interval: (f64, f64),
    /// Prediction interval (random-effects only)
    pub prediction_interval: Option<(f64, f64)>,
    /// Heterogeneity statistics
    pub heterogeneity: HeterogeneityStats,
    /// Publication bias test results
    pub bias_tests: Vec<BiasTestResult>,
    /// Individual study results
    pub study_results: Vec<StudyResult>,
    /// Model fit statistics
    pub model_fit: ModelFitStats,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Heterogeneity statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeterogeneityStats {
    /// Q statistic (Cochran's Q)
    pub q: f64,
    /// Degrees of freedom
    pub df: usize,
    /// p-value for Q
    pub q_pvalue: f64,
    /// I-squared (%)
    pub i_squared: f64,
    /// H-squared
    pub h_squared: f64,
    /// Tau-squared (between-study variance)
    pub tau_squared: f64,
    /// Tau (SD of true effects)
    pub tau: f64,
}

/// Publication bias test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiasTestResult {
    /// Test name
    pub test: String,
    /// Test statistic
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub statistic: f64,
    /// p-value
    pub p_value: f64,
    /// Interpretation
    pub interpretation: String,
    /// Trim-and-fill adjusted estimate (if applicable)
    pub adjusted_estimate: Option<f64>,
}

/// Result for individual study
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyResult {
    /// Study reference
    pub study_id: String,
    /// Effect size
    pub effect_size: f64,
    /// Standard error
    pub standard_error: f64,
    /// Weight in meta-analysis
    pub weight: f64,
    /// Weight percentage
    pub weight_percent: f64,
    /// Study contribution to Q
    pub q_contribution: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Model fit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFitStats {
    /// Log-likelihood
    pub log_likelihood: Option<f64>,
    /// AIC
    pub aic: Option<f64>,
    /// BIC
    pub bic: Option<f64>,
    /// R-squared analog
    pub r_squared: Option<f64>,
}

/// Comparison between simulation and empirical results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationEmpiricalComparison {
    /// Meta-analysis for empirical studies
    pub empirical_meta: MetaAnalysisResult,
    /// Meta-analysis for simulation studies
    pub simulation_meta: MetaAnalysisResult,
    /// Direct comparison statistics
    pub comparison: ComparisonStats,
    /// Concordance assessment
    pub concordance: ConcordanceAssessment,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Comparison statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonStats {
    /// Difference in pooled effects
    pub effect_difference: f64,
    /// Standard error of difference
    pub difference_se: f64,
    /// Z-statistic for difference
    pub z_score: f64,
    /// p-value for difference
    pub p_value: f64,
    /// 95% CI for difference
    pub confidence_interval: (f64, f64),
}

/// Concordance assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcordanceAssessment {
    /// Empirical effect within simulation prediction interval
    pub in_prediction_interval: bool,
    /// Agreement category
    pub agreement: AgreementLevel,
    /// Percentage overlap of CIs
    pub ci_overlap_percent: f64,
}

/// Level of agreement between simulation and empirical
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgreementLevel {
    /// Excellent agreement (< 0.2 SD difference)
    Excellent,
    /// Good agreement (0.2-0.5 SD difference)
    Good,
    /// Moderate agreement (0.5-1.0 SD difference)
    Moderate,
    /// Poor agreement (> 1.0 SD difference)
    Poor,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Meta-analysis request for API
#[derive(Debug, Deserialize)]
pub struct MetaAnalysisRequest {
    /// config
    pub config: MetaAnalysisConfig,
    /// studies
    pub studies: Vec<Study>,
    /// name
    pub name: Option<String>,
}

/// Meta-analysis response
#[derive(Debug, Serialize)]
pub struct MetaAnalysisResponse {
    /// analysis id
    pub analysis_id: String,
    /// status
    pub status: String,
    /// result
    pub result: Option<MetaAnalysisResult>,
}

/// Forest plot data
#[derive(Debug, Serialize)]
pub struct ForestPlotData {
    /// studies
    pub studies: Vec<ForestStudy>,
    /// pooled effect
    pub pooled_effect: ForestPooledEffect,
    /// heterogeneity
    pub heterogeneity: HeterogeneityStats,
    /// model
    pub model: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Study data for forest plot
#[derive(Debug, Serialize)]
pub struct ForestStudy {
    /// id
    pub id: String,
    /// name
    pub name: String,
    /// effect size
    pub effect_size: f64,
    /// ci lower
    pub ci_lower: f64,
    /// ci upper
    pub ci_upper: f64,
    /// weight
    pub weight: f64,
    /// sample size
    pub sample_size: usize,
}

/// Pooled effect for forest plot
#[derive(Debug, Serialize)]
pub struct ForestPooledEffect {
    /// effect size
    pub effect_size: f64,
    /// ci lower
    pub ci_lower: f64,
    /// ci upper
    pub ci_upper: f64,
    /// pi lower
    pub pi_lower: Option<f64>,
    /// pi upper
    pub pi_upper: Option<f64>,
}

/// Funnel plot data
#[derive(Debug, Serialize)]
pub struct FunnelPlotData {
    /// studies
    pub studies: Vec<FunnelStudy>,
    /// pooled effect
    pub pooled_effect: f64,
    /// pseudo ci lower
    pub pseudo_ci_lower: Vec<(f64, f64)>,
    /// pseudo ci upper
    pub pseudo_ci_upper: Vec<(f64, f64)>,
    /// asymmetry test
    pub asymmetry_test: Option<BiasTestResult>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Study data for funnel plot
#[derive(Debug, Clone, Serialize)]
pub struct FunnelStudy {
    /// id
    pub id: String,
    /// effect size
    pub effect_size: f64,
    /// standard error
    pub standard_error: f64,
    /// precision
    pub precision: f64, // 1/SE
}

/// Leave-one-out sensitivity data
#[derive(Debug, Serialize)]
pub struct LeaveOneOutResult {
    /// study removed
    pub study_removed: String,
    /// pooled effect
    pub pooled_effect: f64,
    /// ci lower
    pub ci_lower: f64,
    /// ci upper
    pub ci_upper: f64,
    /// i squared
    pub i_squared: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Cumulative meta-analysis data
#[derive(Debug, Serialize)]
pub struct CumulativeMetaResult {
    /// after study
    pub after_study: String,
    /// cumulative n
    pub cumulative_n: usize,
    /// pooled effect
    pub pooled_effect: f64,
    /// ci lower
    pub ci_lower: f64,
    /// ci upper
    pub ci_upper: f64,
}

/// Subgroup analysis configuration
#[derive(Debug, Deserialize)]
pub struct SubgroupConfig {
    /// variable
    pub variable: String,
    /// categories
    pub categories: Vec<String>,
}

/// Subgroup analysis result
#[derive(Debug, Serialize)]
pub struct SubgroupResult {
    /// variable
    pub variable: String,
    /// categories
    pub categories: Vec<SubgroupCategory>,
    /// between group q
    pub between_group_q: f64,
    /// between group p
    pub between_group_p: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Single category in subgroup analysis
#[derive(Debug, Serialize)]
pub struct SubgroupCategory {
    /// category
    pub category: String,
    /// k
    pub k: usize,
    /// pooled effect
    pub pooled_effect: f64,
    /// ci lower
    pub ci_lower: f64,
    /// ci upper
    pub ci_upper: f64,
    /// i squared
    pub i_squared: f64,
}

/// Meta-regression result
#[derive(Debug, Serialize)]
pub struct MetaRegressionResult {
    /// coefficients
    pub coefficients: Vec<RegressionCoefficient>,
    /// r squared
    pub r_squared: f64,
    /// f statistic
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub f_statistic: f64,
    /// p value
    pub p_value: f64,
    /// q model
    pub q_model: f64,
    /// q residual
    pub q_residual: f64,
}

/// Regression coefficient
#[derive(Debug, Serialize)]
pub struct RegressionCoefficient {
    /// predictor
    pub predictor: String,
    /// coefficient
    pub coefficient: f64,
    /// standard error
    pub standard_error: f64,
    /// z score
    pub z_score: f64,
    /// p value
    pub p_value: f64,
    /// ci lower
    pub ci_lower: f64,
    /// ci upper
    pub ci_upper: f64,
}
