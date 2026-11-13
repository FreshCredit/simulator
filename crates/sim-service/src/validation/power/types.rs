//! Types for statistical power analysis

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Statistical test type for power analysis
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StatisticalTest {
    /// One-sample t-test
    TTestOneSample,
    /// Independent samples t-test
    TTestIndependent,
    /// Paired samples t-test
    TTestPaired,
    /// One-way ANOVA
    AnovaOneWay,
    /// Two-way ANOVA
    AnovaTwoWay,
    /// Repeated measures ANOVA
    AnovaRepeatedMeasures,
    /// Chi-square test
    ChiSquare,
    /// Fisher's exact test
    FishersExact,
    /// Pearson correlation
    CorrelationPearson,
    /// Spearman correlation
    CorrelationSpearman,
    /// Linear regression
    RegressionLinear,
    /// Logistic regression
    RegressionLogistic,
    /// Mann-Whitney U test
    MannWhitney,
    /// Wilcoxon signed-rank test
    Wilcoxon,
    /// Kruskal-Wallis test
    KruskalWallis,
    /// Binomial test
    BinomialTest,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Power analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerAnalysisConfig {
    /// Target statistical power (typically 0.80)
    pub target_power: f64,
    /// Significance level alpha (typically 0.05)
    pub alpha: f64,
    /// Statistical test to use
    pub test: StatisticalTest,
    /// Expected effect size
    pub effect_size: EffectSizeConfig,
    /// Number of groups/conditions
    pub num_groups: usize,
    /// Allocation ratio between groups (for unequal sample sizes)
    pub allocation_ratio: Option<f64>,
    /// Whether test is one-tailed
    pub one_tailed: bool,
    /// Minimum detectable effect size
    pub min_detectable_effect: Option<f64>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for PowerAnalysisConfig {
    fn default() -> Self {
        Self {
            target_power: 0.80,
            alpha: 0.05,
            test: StatisticalTest::TTestIndependent,
            effect_size: EffectSizeConfig::default(),
            num_groups: 2,
            allocation_ratio: None,
            one_tailed: false,
            min_detectable_effect: None,
        }
    }
}

/// Effect size configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectSizeConfig {
    /// Type of effect size
    pub effect_type: EffectSizeType,
    /// Expected magnitude
    pub magnitude: f64,
    /// Source of estimate (prior study, pilot, convention)
    pub source: EffectSizeSource,
    /// Confidence interval for effect size estimate
    pub ci: Option<(f64, f64)>,
}

impl Default for EffectSizeConfig {
    fn default() -> Self {
        Self {
            effect_type: EffectSizeType::CohensD,
            magnitude: 0.5, // Medium effect
            source: EffectSizeSource::Convention,
            ci: None,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Type of effect size measure
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EffectSizeType {
    /// Cohen's d
    CohensD,
    /// Hedge's g (corrected for small samples)
    HedgesG,
    /// Glass's delta
    GlassDelta,
    /// Pearson's r
    PearsonsR,
    /// R-squared
    RSquared,
    /// Eta-squared
    EtaSquared,
    /// Partial eta-squared
    PartialEtaSquared,
    /// Omega-squared
    OmegaSquared,
    /// Odds ratio
    OddsRatio,
    /// Cohen's h
    CohensH,
    /// Cohen's f (for ANOVA)
    CohensF,
    /// Cramer's V
    CramersV,
}

/// Source of effect size estimate
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EffectSizeSource {
    /// Prior published study
    PriorStudy,
    /// Pilot data
    PilotData,
    /// Researcher convention (small=0.2, medium=0.5, large=0.8)
    Convention,
    /// Meta-analysis estimate
    MetaAnalysis,
    /// Conservative estimate
    Conservative,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Result of a power calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerCalculationResult {
    /// Calculated statistical power
    pub power: f64,
    /// Required sample size per group
    pub required_n_per_group: usize,
    /// Total required sample size
    pub total_required_n: usize,
    /// Actual alpha used
    pub alpha: f64,
    /// Effect size used in calculation
    pub effect_size: f64,
    /// Achieved power level
    pub power_level: PowerLevel,
    /// Recommendations based on calculation
    pub recommendations: Vec<String>,
}

/// Power level classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PowerLevel {
    /// Power < 0.60 (inadequate)
    Inadequate,
    /// Power 0.60-0.70 (suboptimal)
    Suboptimal,
    /// Power 0.70-0.80 (acceptable)
    Acceptable,
    /// Power 0.80-0.90 (good)
    Good,
    /// Power 0.90-0.95 (very good)
    VeryGood,
    /// Power >= 0.95 (excellent)
    Excellent,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl PowerLevel {
    /// Get power level from numeric value
    #[must_use]
    pub fn from_power(power: f64) -> Self {
        if !power.is_finite() {
            return Self::Inadequate;
        }
        match power {
            p if p < 0.60 => Self::Inadequate,
            p if p < 0.70 => Self::Suboptimal,
            p if p < 0.80 => Self::Acceptable,
            p if p < 0.90 => Self::Good,
            p if p < 0.95 => Self::VeryGood,
            _ => Self::Excellent,
        }
    }

    /// Get description of power level
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Inadequate => "Power is inadequate (< 0.60). High risk of Type II error.",
            Self::Suboptimal => "Power is suboptimal (0.60-0.70). Consider increasing sample size.",
            Self::Acceptable => "Power is acceptable (0.70-0.80). Marginal but may be sufficient.",
            Self::Good => "Power is good (0.80-0.90). Standard for most research.",
            Self::VeryGood => "Power is very good (0.90-0.95). High confidence in results.",
            Self::Excellent => "Power is excellent (>= 0.95). Very high confidence in results.",
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Sample size analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleSizeResult {
    /// Recommended sample size per group
    pub n_per_group: usize,
    /// Total recommended sample size
    pub total_n: usize,
    /// Sample size for 80% power
    pub n_for_80_power: usize,
    /// Sample size for 90% power
    pub n_for_90_power: usize,
    /// Sample size for 95% power
    pub n_for_95_power: usize,
    /// Power curve data points
    pub power_curve: Vec<PowerCurvePoint>,
}

/// Single point on a power curve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerCurvePoint {
    /// Sample size per group
    pub n: usize,
    /// Statistical power at this sample size
    pub power: f64,
}

/// Effect size analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectSizeResult {
    /// Calculated effect size
    pub effect_size: f64,
    /// Type of effect size
    pub effect_type: EffectSizeType,
    /// Interpretation (small, medium, large)
    pub interpretation: EffectSizeInterpretation,
    /// Confidence interval
    pub confidence_interval: Option<(f64, f64)>,
    /// Magnitude descriptor
    pub magnitude: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Effect size interpretation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EffectSizeInterpretation {
    /// Negligible
    Negligible,
    /// Small
    Small,
    /// Medium
    Medium,
    /// Large
    Large,
    /// `VeryLarge`
    VeryLarge,
}

impl EffectSizeInterpretation {
    /// Get interpretation for Cohen's d
    #[must_use]
    pub fn from_cohens_d(d: f64) -> Self {
        if !d.is_finite() {
            return Self::Negligible;
        }
        let abs_d = d.abs();
        match abs_d {
            d if d < 0.2 => Self::Negligible,
            d if d < 0.5 => Self::Small,
            d if d < 0.8 => Self::Medium,
            d if d < 1.2 => Self::Large,
            _ => Self::VeryLarge,
        }
    }

    /// Get interpretation for Pearson's r
    #[must_use]
    pub fn from_pearsons_r(r: f64) -> Self {
        if !r.is_finite() {
            return Self::Negligible;
        }
        let abs_r = r.abs();
        match abs_r {
            r if r < 0.1 => Self::Negligible,
            r if r < 0.3 => Self::Small,
            r if r < 0.5 => Self::Medium,
            r if r < 0.7 => Self::Large,
            _ => Self::VeryLarge,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get interpretation for eta-squared
    #[must_use]
    pub fn from_eta_squared(eta2: f64) -> Self {
        if !eta2.is_finite() {
            return Self::Negligible;
        }
        match eta2 {
            e if e < 0.01 => Self::Negligible,
            e if e < 0.06 => Self::Small,
            e if e < 0.14 => Self::Medium,
            e if e < 0.25 => Self::Large,
            _ => Self::VeryLarge,
        }
    }
}

/// Power analysis request for API
#[derive(Debug, Deserialize)]
pub struct PowerAnalysisRequest {
    /// config
    pub config: PowerAnalysisConfig,
    /// available n
    pub available_n: Option<usize>,
    /// metric name
    pub metric_name: Option<String>,
}

/// Power analysis response for API
#[derive(Debug, Serialize)]
pub struct PowerAnalysisResponse {
    /// calculation
    pub calculation: PowerCalculationResult,
    /// sample size
    pub sample_size: SampleSizeResult,
    /// recommendations
    pub recommendations: Vec<String>,
    /// is adequately powered
    pub is_adequately_powered: bool,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Batch power analysis for multiple metrics
#[derive(Debug, Deserialize)]
pub struct BatchPowerAnalysisRequest {
    /// analyses
    pub analyses: Vec<PowerAnalysisRequest>,
}

#[derive(Debug, Serialize)]
/// `BatchPowerAnalysisResponse` data
pub struct BatchPowerAnalysisResponse {
    /// results
    pub results: HashMap<String, PowerAnalysisResponse>,
    /// overall powered
    pub overall_powered: bool,
    /// weakest metric
    pub weakest_metric: Option<String>,
    /// recommendations
    pub recommendations: Vec<String>,
}

/// Power curve request
#[derive(Debug, Deserialize)]
pub struct PowerCurveRequest {
    /// config
    pub config: PowerAnalysisConfig,
    /// n min
    pub n_min: usize,
    /// n max
    pub n_max: usize,
    /// n step
    pub n_step: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Serialize)]
/// `PowerCurveResponse` data
pub struct PowerCurveResponse {
    /// curve
    pub curve: Vec<PowerCurvePoint>,
    /// target n
    pub target_n: usize,
    /// target power
    pub target_power: f64,
}

/// Prior power analysis record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorPowerAnalysis {
    /// id
    pub id: String,
    /// experiment name
    pub experiment_name: String,
    /// config
    pub config: PowerAnalysisConfig,
    /// result
    pub result: PowerCalculationResult,
    /// created at
    pub created_at: String,
    /// updated at
    pub updated_at: String,
}
// TAG: surface=simulation owner=platform-team rule=GENERAL-001

/// Convention-based effect size magnitudes
pub mod conventions {
    use super::EffectSizeType;

    /// Get small effect size for given type
    #[must_use]
    pub const fn small(effect_type: EffectSizeType) -> f64 {
        match effect_type {
            EffectSizeType::CohensD => 0.2,
            EffectSizeType::HedgesG => 0.2,
            EffectSizeType::GlassDelta => 0.2,
            EffectSizeType::PearsonsR => 0.1,
            EffectSizeType::RSquared => 0.01,
            EffectSizeType::EtaSquared => 0.01,
            EffectSizeType::PartialEtaSquared => 0.01,
            EffectSizeType::OmegaSquared => 0.01,
            EffectSizeType::OddsRatio => 1.5,
            EffectSizeType::CohensH => 0.2,
            EffectSizeType::CohensF => 0.1,
            EffectSizeType::CramersV => 0.1,
        }
    }

    /// Get medium effect size for given type
    #[must_use]
    pub const fn medium(effect_type: EffectSizeType) -> f64 {
        match effect_type {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            EffectSizeType::CohensD => 0.5,
            EffectSizeType::HedgesG => 0.5,
            EffectSizeType::GlassDelta => 0.5,
            EffectSizeType::PearsonsR => 0.3,
            EffectSizeType::RSquared => 0.09,
            EffectSizeType::EtaSquared => 0.06,
            EffectSizeType::PartialEtaSquared => 0.06,
            EffectSizeType::OmegaSquared => 0.06,
            EffectSizeType::OddsRatio => 3.0,
            EffectSizeType::CohensH => 0.5,
            EffectSizeType::CohensF => 0.25,
            EffectSizeType::CramersV => 0.3,
        }
    }

    /// Get large effect size for given type
    #[must_use]
    pub const fn large(effect_type: EffectSizeType) -> f64 {
        match effect_type {
            EffectSizeType::CohensD => 0.8,
            EffectSizeType::HedgesG => 0.8,
            EffectSizeType::GlassDelta => 0.8,
            EffectSizeType::PearsonsR => 0.5,
            EffectSizeType::RSquared => 0.25,
            EffectSizeType::EtaSquared => 0.14,
            EffectSizeType::PartialEtaSquared => 0.14,
            EffectSizeType::OmegaSquared => 0.14,
            EffectSizeType::OddsRatio => 5.0,
            EffectSizeType::CohensH => 0.8,
            EffectSizeType::CohensF => 0.4,
            EffectSizeType::CramersV => 0.5,
        }
    }
}
