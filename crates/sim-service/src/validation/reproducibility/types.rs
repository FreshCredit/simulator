//! Types for Reproducibility Validation (P1-VAL-5)
//!
//! Core type definitions for seed variation testing and stability metrics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Configuration for reproducibility validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReproducibilityConfig {
    /// base seed
    pub base_seed: u64,
    /// n replications
    pub n_replications: usize,
    /// stability threshold
    pub stability_threshold: f64,
    /// convergence tolerance
    pub convergence_tolerance: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for ReproducibilityConfig {
    fn default() -> Self {
        Self {
            base_seed: 42,
            n_replications: 30,
            stability_threshold: 0.10,
            convergence_tolerance: 0.01,
        }
    }
}

/// Individual replication result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationResult {
    /// index
    pub index: usize,
    /// seed
    pub seed: u64,
    /// metrics
    pub metrics: HashMap<String, f64>,
    /// execution time secs
    pub execution_time_secs: f64,
    /// success
    pub success: bool,
    /// error
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub error: Option<String>,
}

/// Stability analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReproducibilityResult {
    /// is stable
    pub is_stable: bool,
    /// stability score
    pub stability_score: f64,
    /// coefficient of variation
    pub coefficient_of_variation: HashMap<String, f64>,
    /// mean values
    pub mean_values: HashMap<String, f64>,
    /// std dev values
    pub std_dev_values: HashMap<String, f64>,
    /// confidence intervals 95
    pub confidence_intervals_95: HashMap<String, (f64, f64)>,
    /// outliers
    pub outliers: HashMap<String, Vec<usize>>,
    /// convergence
    pub convergence: ConvergenceStatus,
    /// recommendations
    pub recommendations: Vec<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Convergence status across replications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceStatus {
    /// has converged
    pub has_converged: bool,
    /// replication at convergence
    pub replication_at_convergence: Option<usize>,
    /// r hat
    pub r_hat: HashMap<String, f64>,
}

/// Seed sensitivity analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedSensitivityResult {
    /// base result
    pub base_result: HashMap<String, f64>,
    /// perturbed results
    pub perturbed_results: Vec<HashMap<String, f64>>,
    /// max deviation
    pub max_deviation: HashMap<String, f64>,
    /// is robust
    pub is_robust: bool,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Variance components for ANOVA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarianceComponents {
    /// within variance
    pub within_variance: f64,
    /// between variance
    pub between_variance: f64,
    /// total variance
    pub total_variance: f64,
    /// icc
    pub icc: f64,
}

/// Variance analysis response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarianceAnalysisResponse {
    /// components
    pub components: VarianceComponents,
    /// recommendations
    pub recommendations: Vec<String>,
}

/// Reproducibility report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReproducibilityReport {
    /// summary
    pub summary: String,
    /// metrics summary
    pub metrics_summary: HashMap<String, MetricReport>,
    /// overall stability
    pub overall_stability: StabilityLevel,
    /// convergence assessment
    pub convergence_assessment: String,
    /// recommendations
    pub recommendations: Vec<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Individual metric report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricReport {
    /// mean
    pub mean: f64,
    /// std dev
    pub std_dev: f64,
    /// cv
    pub cv: f64,
    /// ci lower
    pub ci_lower: f64,
    /// ci upper
    pub ci_upper: f64,
    /// stability
    pub stability: StabilityLevel,
}

/// Stability level classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StabilityLevel {
    /// High
    High,
    /// Moderate
    Moderate,
    /// Low
    Low,
    /// Unstable
    Unstable,
}

impl StabilityLevel {
    #[must_use]
    /// from cv
    pub fn from_cv(cv: f64) -> Self {
        if !cv.is_finite() || cv < 0.0 {
            Self::Unstable
        } else if cv < 0.05 {
            Self::High
        } else if cv < 0.10 {
            Self::Moderate
        } else if cv < 0.20 {
            Self::Low
        } else {
            Self::Unstable
        }
    }
}
