//! Hypothesis testing types
//!
//! These types are duplicated from sim-runner to avoid circular dependencies.
//! In a production system, these would be in a shared types crate.

use serde::{Deserialize, Serialize};
use std::fmt::Write;

/// GTT Hypothesis Test Results
///
/// This struct is duplicated from `sim-runner::hypothesis` to avoid
/// circular dependencies between sim-report and sim-runner.
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

/// Results from a single condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionResults {
    /// Number of observations
    pub n: usize,
    /// Mean trust value
    pub mean: f32,
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Standard deviation
    pub std: f32,
    /// Individual values (for further analysis)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub values: Vec<f32>,
}

fn format_condition_stats(label: &str, stats: &ConditionResults) -> String {
    format!(
        "{label}:\n  n = {}\n  μ = {:.4}\n  σ = {:.4}\n\n",
        stats.n, stats.mean, stats.std
    )
}

fn format_statistical_analysis(result: &GttHypothesisResult) -> String {
    format!(
        "Statistical Analysis:\n  Effect size (Cohen's d): {:.3}\n  95% CI: [{:.3}, {:.3}]\n  Interpretation: {}\n  t-statistic: {:.3}\n  p-value: {:.4}\n\n",
        result.effect_size,
        result.effect_size_ci_lower,
        result.effect_size_ci_upper,
        result.effect_size_interpretation,
        result.t_statistic,
        result.p_value
    )
}

impl GttHypothesisResult {
    /// Generate a human-readable summary report
    #[must_use]
    pub fn summary_report(&self) -> String {
        let mut report = String::new();

        report.push_str("=================================\n");
        report.push_str("GTT Hypothesis Test Results\n");
        report.push_str("=================================\n\n");

        report.push_str("Configuration:\n");
        let _ = writeln!(report, "  Runs per condition: {}", self.n_runs);
        let _ = writeln!(report, "  Master seed: {}", self.seed);
        let _ = write!(
            report,
            "  Confidence level: {:.0}%\n\n",
            (self.confidence_level * 100.0).floor()
        );

        report.push_str(&format_condition_stats(
            "Protocol ON (Decentralized)",
            &self.protocol_on,
        ));
        report.push_str(&format_condition_stats(
            "Protocol OFF (Centralized)",
            &self.protocol_off,
        ));
        report.push_str(&format_statistical_analysis(self));

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
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_result() -> GttHypothesisResult {
        GttHypothesisResult {
            n_runs: 100,
            seed: 42,
            protocol_on: ConditionResults {
                n: 100,
                mean: 0.75,
                std: 0.15,
                values: vec![],
            },
            protocol_off: ConditionResults {
                n: 100,
                mean: 0.60,
                std: 0.20,
                values: vec![],
            },
            effect_size: 0.85,
            effect_size_ci_lower: 0.60,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            effect_size_ci_upper: 1.10,
            effect_size_interpretation: "Large effect".to_string(),
            t_statistic: 5.42,
            p_value: 0.001,
            hypothesis_supported: true,
            confidence_level: 0.95,
        }
    }

    #[test]
    fn test_summary_report_contains_data() {
        let result = test_result();
        let report = result.summary_report();
        assert!(report.contains("GTT Hypothesis Test Results"));
        assert!(report.contains("100"));
        assert!(report.contains("0.7500"));
        assert!(report.contains("SUPPORTED"));
    }

    #[test]
    fn test_summary_report_not_supported() {
        let mut result = test_result();
        result.hypothesis_supported = false;
        let report = result.summary_report();
        assert!(report.contains("NOT SUPPORTED"));
    }

    #[test]
    fn test_to_json_roundtrip() {
        let result = test_result();
        let json = result.to_json().unwrap();
        let decoded: GttHypothesisResult = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.n_runs, 100);
        assert!((decoded.effect_size - 0.85).abs() < f32::EPSILON);
    }

    #[test]
    fn test_condition_results_default() {
        let cr = ConditionResults {
            n: 50,
            mean: 0.5,
            std: 0.1,
            values: vec![1.0, 2.0, 3.0],
        };
        assert_eq!(cr.values.len(), 3);
    }
}
