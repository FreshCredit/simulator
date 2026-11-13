//! GTT Hypothesis Test Runner
//!
//! B-001: Critical blocker for research validity
//!
//! Core hypothesis: "Protocol ON increases trust (τ) compared to Protocol OFF"
//!
//! This module provides automated hypothesis testing for the General Theory of Trust (GTT)
//! using the `FreshCredit` Agent Society Simulator. It compares trust emergence between
//! Protocol ON (decentralized verification) and Protocol OFF (centralized oracle) conditions.
//!
//! ## Hypothesis Framework
//!
//! **Null Hypothesis (H₀):** `μ_protocol` ≤ `μ_control`
//! - Protocol ON does not increase trust compared to Protocol OFF
//!
//! **Alternative Hypothesis (H₁):** `μ_protocol` > `μ_control`
//! - Protocol ON increases trust compared to Protocol OFF
//!
//! **Falsification Criteria:**
//! - Cohen's d < 0.15 (small effect)
//! - p-value > 0.05 (not significant)
//! - 95% CI includes zero
//!
//! ## Statistical Methods
//!
//! - **Effect Size:** Cohen's d with 95% CI
//! - **Significance:** Welch's t-test (unequal variances)
//! - **Power Analysis:** Minimum n=20 per condition for 80% power
//!
//! ## Example
//!
//! ```rust,no_run
//! use sim_runner::{RunConfig, hypothesis::GttHypothesisRunner};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = RunConfig::default();
//! let runner = GttHypothesisRunner::new(config);
//! let result = runner.run(20, 42)?;
//!
//! if result.hypothesis_supported {
//!     println!("GTT hypothesis supported! Effect size: {:.2}", result.effect_size);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-004: Deterministic - fixed seeds for reproducibility

use crate::batch::{BatchError, BatchRunner};
use crate::config::RunConfig;
use serde::{Deserialize, Serialize};
// TAG: surface=simulation owner=research-team rule=SIM-001
use sim_core::{
    protocol::VerificationMode,
    stats::{cohens_d_with_ci, mean_std, welch_t_test},
};

/// GTT Hypothesis Test Runner
///
/// Automates the core research question: Does Protocol ON increase trust?
#[derive(Debug)]
pub struct GttHypothesisRunner {
    base_config: RunConfig,
}

impl GttHypothesisRunner {
    /// Create a new hypothesis runner with base configuration
    #[must_use]
    pub const fn new(base_config: RunConfig) -> Self {
        Self { base_config }
    }

    /// Run the GTT hypothesis test
    ///
    /// # Arguments
    /// * `n_runs` - Number of simulation runs per condition (minimum 20 for statistical power)
    /// * `seed` - Master random seed for reproducibility
    ///
    /// # Returns
    /// Complete hypothesis test results with statistical analysis
    ///
    /// # Example
    /// ```rust,no_run
    /// use sim_runner::{RunConfig, hypothesis::GttHypothesisRunner};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = RunConfig::default();
    /// let runner = GttHypothesisRunner::new(config);
    /// let result = runner.run(20, 42)?;
    ///
    /// debug_assert!(result.hypothesis_supported, "GTT hypothesis should be supported");
    /// # Ok(())
    /// # }
    /// ```
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn run(&self, n_runs: u32, seed: u64) -> Result<GttHypothesisResult, BatchError> {
        if n_runs < 10 {
            eprintln!("Warning: n_runs < 10 may result in underpowered tests");
        }

        // Run Protocol ON condition (DecentralizedProtocol)
        let protocol_on_metrics = self.run_condition(
            n_runs,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            seed,
            VerificationMode::DecentralizedProtocol,
            "Protocol ON",
        )?;

        // Run Protocol OFF condition (CentralizedOracle)
        let protocol_off_metrics = self.run_condition(
            n_runs,
            seed + 10000, // Different seed offset
            VerificationMode::CentralizedOracle,
            "Protocol OFF",
        )?;

        // Calculate statistics
        let result = self.analyze_results(protocol_on_metrics, protocol_off_metrics, n_runs, seed);

        Ok(result)
    }

    /// Run a single condition and collect trust metrics
    fn run_condition(
        &self,
        n_runs: u32,
        seed: u64,
        mode: VerificationMode,
        label: &str,
    ) -> Result<Vec<f32>, BatchError> {
        println!("Running {label} condition (n={n_runs})...");

        let mut config = self.base_config.clone();
        config.simulation.verification_mode = mode;
        config.simulation.seed = seed;

        let runner = BatchRunner::new(config);
        let batch_result = runner.run(n_runs)?;

        // Extract trust metrics from batch results
        // Using avg_docas_magnitude as a proxy for trust (tau)
        // DOCAS magnitude correlates with trust state per simulation-plan.md
        let trust_values: Vec<f32> = batch_result
            .individual_results
            .iter()
            .map(|run| run.avg_docas_magnitude as f32)
            .collect();
        // TAG: surface=simulation owner=research-team rule=SIM-001

        let (mean, std) = mean_std(&trust_values);
        println!("  {label}: μ={mean:.3}, σ={std:.3}");

        Ok(trust_values)
    }

    /// Statistical analysis of results
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn analyze_results(
        &self,
        protocol_on: Vec<f32>,
        protocol_off: Vec<f32>,
        n_runs: u32,
        seed: u64,
    ) -> GttHypothesisResult {
        // Calculate means and standard deviations
        let (mean_on, std_on) = mean_std(&protocol_on);
        let (mean_off, std_off) = mean_std(&protocol_off);

        // Effect size (Cohen's d)
        let (cohens_d, d_ci_lower, d_ci_upper) = cohens_d_with_ci(&protocol_on, &protocol_off);

        // Statistical significance (Welch's t-test)
        let (t_stat, p_value) = welch_t_test(&protocol_on, &protocol_off);

        // Determine hypothesis support
        // H₁ supported if: d >= 0.15, p < 0.05, CI excludes 0
        let hypothesis_supported = cohens_d >= 0.15 && p_value < 0.05 && d_ci_lower > 0.0;

        // Effect size interpretation
        let effect_size_interpretation = interpret_cohens_d(cohens_d);

        GttHypothesisResult {
            n_runs,
            seed,
            protocol_on: ConditionResults {
                n: protocol_on.len(),
                mean: mean_on,
                std: std_on,
                values: protocol_on,
            },
            protocol_off: ConditionResults {
                // TAG: surface=simulation owner=research-team rule=SIM-001
                n: protocol_off.len(),
                mean: mean_off,
                std: std_off,
                values: protocol_off,
            },
            effect_size: cohens_d,
            effect_size_ci_lower: d_ci_lower,
            effect_size_ci_upper: d_ci_upper,
            effect_size_interpretation,
            t_statistic: t_stat,
            p_value,
            hypothesis_supported,
            confidence_level: 0.95,
        }
    }

    /// Run a quick test with reduced parameters for validation
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn quick_test(&self, seed: u64) -> Result<GttHypothesisResult, BatchError> {
        println!("Running quick GTT hypothesis test (n=5)...");
        self.run(5, seed)
    }
}

/// Results from a single condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionResults {
    /// Number of observations
    pub n: usize,
    /// Mean trust value
    pub mean: f32,
    /// Standard deviation
    pub std: f32,
    /// Individual values (for further analysis)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub values: Vec<f32>,
}
// TAG: surface=simulation owner=research-team rule=SIM-001

/// Complete GTT hypothesis test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GttHypothesisResult {
    /// Number of runs per condition
    pub n_runs: u32,
    /// Master random seed
    pub seed: u64,
    /// Protocol ON results
    pub protocol_on: ConditionResults,
    /// Protocol OFF results
    pub protocol_off: ConditionResults,
    /// Cohen's d effect size
    pub effect_size: f32,
    /// Effect size CI lower bound
    pub effect_size_ci_lower: f32,
    /// Effect size CI upper bound
    pub effect_size_ci_upper: f32,
    /// Human-readable effect size interpretation
    pub effect_size_interpretation: String,
    /// t-statistic from Welch's test
    pub t_statistic: f32,
    /// p-value from Welch's test
    pub p_value: f32,
    /// Whether the hypothesis is supported
    pub hypothesis_supported: bool,
    /// Confidence level used (typically 0.95)
    pub confidence_level: f32,
}

impl GttHypothesisResult {
    /// Generate a human-readable summary report
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[must_use]
    pub fn summary_report(&self) -> String {
        let mut report = String::new();

        report.push_str("=================================\n");
        report.push_str("GTT Hypothesis Test Results\n");
        report.push_str("=================================\n\n");

        report.push_str("Configuration:\n");
        report.push_str(&format!("  Runs per condition: {}\n", self.n_runs));
        report.push_str(&format!("  Master seed: {}\n", self.seed));
        report.push_str(&format!(
            "  Confidence level: {}%\n\n",
            (self.confidence_level * 100.0) as u32
        ));
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        report.push_str("Protocol ON (Decentralized):\n");
        report.push_str(&format!("  n = {}\n", self.protocol_on.n));
        report.push_str(&format!("  μ = {:.4}\n", self.protocol_on.mean));
        report.push_str(&format!("  σ = {:.4}\n\n", self.protocol_on.std));

        report.push_str("Protocol OFF (Centralized):\n");
        report.push_str(&format!("  n = {}\n", self.protocol_off.n));
        report.push_str(&format!("  μ = {:.4}\n", self.protocol_off.mean));
        report.push_str(&format!("  σ = {:.4}\n\n", self.protocol_off.std));

        report.push_str("Statistical Analysis:\n");
        report.push_str(&format!(
            "  Effect size (Cohen's d): {:.3}\n",
            self.effect_size
        ));
        report.push_str(&format!(
            "  95% CI: [{:.3}, {:.3}]\n",
            self.effect_size_ci_lower, self.effect_size_ci_upper
        ));
        report.push_str(&format!(
            "  Interpretation: {}\n",
            self.effect_size_interpretation
        ));
        report.push_str(&format!("  t-statistic: {:.3}\n", self.t_statistic));
        report.push_str(&format!("  p-value: {:.4}\n\n", self.p_value));

        report.push_str("Conclusion:\n");
        if self.hypothesis_supported {
            report.push_str("  ✅ GTT HYPOTHESIS SUPPORTED\n");
            report.push_str(
                "  Protocol ON significantly increases trust compared to Protocol OFF.\n",
            );
        } else {
            report.push_str("  ❌ GTT HYPOTHESIS NOT SUPPORTED\n");
            report.push_str("  Insufficient evidence that Protocol ON increases trust.\n");
        }

        report.push_str("\n=================================\n");

        report
    }

    /// Generate JSON report
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Check if result meets research-grade criteria
    ///
    /// Research-grade requires:
    /// - n >= 20 per condition
    /// - p < 0.05
    /// - |d| >= 0.2 (at least small-medium effect)
    #[must_use]
    pub fn is_research_grade(&self) -> bool {
        self.protocol_on.n >= 20
            && self.protocol_off.n >= 20
            && self.p_value < 0.05
            && self.effect_size.abs() >= 0.2
    }
}

/// Interpret Cohen's d effect size
/// # Errors
///
/// Returns an error if the operation fails.
fn interpret_cohens_d(d: f32) -> String {
    let abs_d = d.abs();
    let direction = if d > 0.0 { "increase" } else { "decrease" };

    let magnitude = if abs_d < 0.2 {
        "negligible"
    } else if abs_d < 0.5 {
        "small"
    } else if abs_d < 0.8 {
        "medium"
    } else {
        "large"
    };

    format!("{magnitude} {direction} (d={d:.2})")
}

/// Convenience function for quick hypothesis testing
///
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
/// # Example
/// ```rust,no_run
/// use sim_runner::hypothesis::quick_gtt_test;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let result = quick_gtt_test(42)?;
/// println!("{}", result.summary_report());
/// # Ok(())
/// # }
/// ```
/// # Errors
///
/// Returns an error if the operation fails.
pub fn quick_gtt_test(seed: u64) -> Result<GttHypothesisResult, BatchError> {
    let config = RunConfig::default();
    let runner = GttHypothesisRunner::new(config);
    runner.run(20, seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpret_cohens_d() {
        assert_eq!(interpret_cohens_d(0.1), "negligible increase (d=0.10)");
        assert_eq!(interpret_cohens_d(0.3), "small increase (d=0.30)");
        assert_eq!(interpret_cohens_d(0.6), "medium increase (d=0.60)");
        assert_eq!(interpret_cohens_d(0.9), "large increase (d=0.90)");
        assert_eq!(interpret_cohens_d(-0.3), "small decrease (d=-0.30)");
    }

    #[test]
    fn test_gtt_result_research_grade() {
        let result = GttHypothesisResult {
            n_runs: 20,
            seed: 42,
            protocol_on: ConditionResults {
                n: 20,
                mean: 0.7,
                std: 0.1,
                values: vec![],
            },
            protocol_off: ConditionResults {
                n: 20,
                mean: 0.6,
                std: 0.1,
                values: vec![],
                // TAG: surface=simulation owner=research-team rule=SIM-001
            },
            effect_size: 0.5,
            effect_size_ci_lower: 0.1,
            effect_size_ci_upper: 0.9,
            effect_size_interpretation: "medium".to_string(),
            t_statistic: 2.5,
            p_value: 0.01,
            hypothesis_supported: true,
            confidence_level: 0.95,
        };

        assert!(result.is_research_grade());
        assert!(result.summary_report().contains("GTT HYPOTHESIS SUPPORTED"));
    }

    #[test]
    fn test_gtt_result_not_research_grade_low_n() {
        let result = GttHypothesisResult {
            n_runs: 5,
            seed: 42,
            protocol_on: ConditionResults {
                n: 5, // Too low
                mean: 0.7,
                std: 0.1,
                values: vec![],
            },
            protocol_off: ConditionResults {
                n: 5,
                mean: 0.6,
                std: 0.1,
                values: vec![],
            },
            effect_size: 0.5,
            effect_size_ci_lower: 0.1,
            effect_size_ci_upper: 0.9,
            effect_size_interpretation: "medium".to_string(),
            t_statistic: 2.5,
            p_value: 0.01,
            hypothesis_supported: true,
            confidence_level: 0.95,
        };

        assert!(!result.is_research_grade());
    }
}
