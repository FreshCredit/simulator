//! Stability Analysis for Reproducibility Validation
//!
//! Core analysis engine for assessing result stability across random seeds.

use std::collections::HashMap;

use super::types::{
    ConvergenceStatus, MetricReport, ReplicationResult, ReproducibilityConfig,
    ReproducibilityReport, ReproducibilityResult, StabilityLevel,
};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Analyzer for stability across replications
#[derive(Debug)]
pub struct StabilityAnalyzer {
    config: ReproducibilityConfig,
}

impl StabilityAnalyzer {
    /// Create new stability analyzer
    #[must_use]
    pub const fn new(config: ReproducibilityConfig) -> Self {
        Self { config }
    }

    /// Compute descriptive statistics for a single metric.
    fn compute_metric_statistics(
        &self,
        values: &[f64],
    ) -> (f64, f64, f64, f64, f64, Vec<usize>, f64) {
        if values.is_empty() {
            return (0.0, 0.0, 0.0, 0.0, 0.0, Vec::new(), 1.0);
        }
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let denom = (values.len().saturating_sub(1)).max(1) as f64;
        let variance = values.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / denom;
        let std = variance.sqrt();
        let cv = if !mean.is_finite() || !std.is_finite() {
            0.0
        } else if mean.abs() < 1e-12 {
            if std <= 1e-12 {
                0.0
            } else {
                10.0
            }
        } else {
            (std / mean.abs()).clamp(0.0, 10.0)
        };

        // 95% confidence interval using Student's t distribution for small samples
        let se = std / (values.len() as f64).sqrt();
        let df = values.len().saturating_sub(1);
        let t_crit = match df {
            0 => 1.96,
            1 => 12.706,
            2 => 4.303,
            3 => 3.182,
            4 => 2.776,
            5 => 2.571,
            6 => 2.447,
            7 => 2.365,
            8 => 2.306,
            9 => 2.262,
            10..=14 => 2.201,
            15..=19 => 2.110,
            20..=29 => 2.064,
            _ => 1.96,
        };
        let margin = t_crit * se;
        let ci_lower = mean - margin;
        let ci_upper = mean + margin;

        // Detect outliers using standard continuous quantile interpolation for IQR
        let mut sorted_values = values.to_vec();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let compute_quantile = |q: f64| -> f64 {
            if sorted_values.len() == 1 {
                return sorted_values[0];
            }
            let index = q * (sorted_values.len() - 1) as f64;
            let lo = index.floor() as usize;
            let hi = index.ceil() as usize;
            let frac = index - lo as f64;
            sorted_values[lo] * (1.0 - frac) + sorted_values[hi] * frac
        };
        let q1 = compute_quantile(0.25);
        let q3 = compute_quantile(0.75);
        let iqr = q3 - q1;
        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let outliers: Vec<usize> = values
            .iter()
            .enumerate()
            .filter(|(_, &v)| v < lower_bound || v > upper_bound)
            .map(|(i, _)| i)
            .collect();

        let r_hat = self.calculate_r_hat(values);

        (mean, std, cv, ci_lower, ci_upper, outliers, r_hat)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Analyze replication results for stability
    #[must_use]
    pub fn analyze(&self, results: &[ReplicationResult]) -> ReproducibilityResult {
        let successful: Vec<_> = results.iter().filter(|r| r.success).collect();

        if successful.is_empty() {
            return ReproducibilityResult {
                is_stable: false,
                stability_score: 0.0,
                coefficient_of_variation: HashMap::new(),
                mean_values: HashMap::new(),
                std_dev_values: HashMap::new(),
                confidence_intervals_95: HashMap::new(),
                outliers: HashMap::new(),
                convergence: ConvergenceStatus {
                    has_converged: false,
                    replication_at_convergence: None,
                    r_hat: HashMap::new(),
                },
                recommendations: vec![
                    "No successful replications found. Minimum 2 replications required for stability analysis."
                        .to_string(),
                ],
            };
        }

        // Collect all metric names
        let mut metric_names: Vec<_> = successful
            .iter()
            .flat_map(|r| r.metrics.keys().cloned())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        metric_names.sort();

        let mut cv_map = HashMap::new();
        let mut mean_map = HashMap::new();
        let mut std_map = HashMap::new();
        let mut ci_map = HashMap::new();
        let mut outliers_map = HashMap::new();
        let mut r_hat_map = HashMap::new();

        for metric in &metric_names {
            let values: Vec<f64> = successful
                .iter()
                .filter_map(|r| r.metrics.get(metric).copied())
                .filter(|v| v.is_finite())
                .collect();

            if values.len() < 2 {
                continue;
            }

            let (mean, std, cv, ci_lower, ci_upper, outliers, r_hat) =
                self.compute_metric_statistics(&values);

            cv_map.insert(metric.clone(), cv);
            mean_map.insert(metric.clone(), mean);
            std_map.insert(metric.clone(), std);
            ci_map.insert(metric.clone(), (ci_lower, ci_upper));
            outliers_map.insert(metric.clone(), outliers);
            r_hat_map.insert(metric.clone(), r_hat);
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Calculate overall stability score
        let avg_cv = if cv_map.is_empty() {
            0.0
        } else {
            cv_map.values().sum::<f64>() / cv_map.len() as f64
        };

        let is_stable = !cv_map.is_empty() && avg_cv < self.config.stability_threshold;
        let stability_score = if is_stable {
            (1.0 - avg_cv.min(1.0)).clamp(0.0, 1.0)
        } else if cv_map.is_empty() {
            0.0
        } else {
            (1.0 - avg_cv.min(1.0)).max(0.0).min(0.5)
        };

        // Check convergence
        let has_converged = !r_hat_map.is_empty() && r_hat_map.values().all(|&r| r < 1.05);
        let replication_at_convergence = if has_converged {
            Some(successful.len())
        } else {
            None
        };

        // Generate recommendations
        let recommendations = self.generate_recommendations(&cv_map, &r_hat_map, &outliers_map);

        ReproducibilityResult {
            is_stable,
            stability_score,
            coefficient_of_variation: cv_map,
            mean_values: mean_map,
            std_dev_values: std_map,
            confidence_intervals_95: ci_map,
            outliers: outliers_map,
            convergence: ConvergenceStatus {
                has_converged,
                replication_at_convergence,
                r_hat: r_hat_map,
            },
            recommendations,
        }
    }

    /// Calculate simplified R-hat (Gelman-Rubin diagnostic)
    fn calculate_r_hat(&self, values: &[f64]) -> f64 {
        if values.len() < 4 {
            return 1.0;
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Split into two equal-length chains
        let mid = values.len() / 2;
        let chain1 = &values[..mid];
        let chain2 = &values[mid..2 * mid];
        let n = mid as f64;

        let mean1 = chain1.iter().sum::<f64>() / n;
        let mean2 = chain2.iter().sum::<f64>() / n;

        let var1 = chain1.iter().map(|&v| (v - mean1).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
        let var2 = chain2.iter().map(|&v| (v - mean2).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);

        let var_within = (var1 + var2) / 2.0;
        let mean_overall = (mean1 + mean2) / 2.0;
        let var_between_mean = (mean1 - mean_overall).powi(2) + (mean2 - mean_overall).powi(2);
        let b = var_between_mean * n;

        // Gelman & Rubin (1992): var^+ = ((n - 1) / n) * W + (1 / n) * B
        let var_plus = ((n - 1.0) / n) * var_within + (1.0 / n) * b;

        if var_within > 1e-12 {
            (var_plus / var_within).sqrt()
        } else if b > 1e-12 {
            f64::INFINITY
        } else {
            1.0
        }
    }

    /// Generate recommendations based on analysis
    fn generate_recommendations(
        &self,
        cv_map: &HashMap<String, f64>,
        r_hat_map: &HashMap<String, f64>,
        outliers: &HashMap<String, Vec<usize>>,
    ) -> Vec<String> {
        let threshold = self.config.stability_threshold.max(1e-4);
        let mut recommendations = Vec::new();

        let mut sorted_metrics: Vec<_> = cv_map.keys().cloned().collect();
        sorted_metrics.sort();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Check CV values against configured threshold
        for metric in &sorted_metrics {
            if let Some(&cv) = cv_map.get(metric) {
                if cv > 2.0 * threshold {
                    recommendations.push(format!(
                        "{}: High variability (CV={:.1}%). Consider more replications.",
                        metric,
                        cv * 100.0
                    ));
                } else if cv > threshold {
                    recommendations.push(format!(
                        "{}: Moderate variability (CV={:.1}%). Monitor this metric.",
                        metric,
                        cv * 100.0
                    ));
                }
            }
        }

        // Check convergence in deterministic metric order
        for metric in &sorted_metrics {
            if let Some(&r_hat) = r_hat_map.get(metric) {
                if r_hat > 1.05 {
                    recommendations.push(format!(
                        "{metric}: Not converged (R-hat={r_hat:.3}). More replications recommended."
                    ));
                }
            }
        }

        // Check outliers in deterministic metric order
        for metric in &sorted_metrics {
            if let Some(outlier_indices) = outliers.get(metric) {
                if !outlier_indices.is_empty() {
                    recommendations.push(format!(
                        "{}: {} outlier(s) detected. Review replication parameters.",
                        metric,
                        outlier_indices.len()
                    ));
                }
            }
        }

        if cv_map.is_empty() {
            recommendations.push(
                "Insufficient valid replications (N < 2). Additional replications required for stability."
                    .to_string(),
            );
        } else if recommendations.is_empty() {
            recommendations.push("Results show good reproducibility across seeds".to_string());
        }

        recommendations
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate detailed report
    #[must_use]
    pub fn generate_report(&self, result: &ReproducibilityResult) -> ReproducibilityReport {
        let mut metric_reports = HashMap::new();

        for (metric, &mean) in &result.mean_values {
            let std = result.std_dev_values.get(metric).copied().unwrap_or(0.0);
            let cv = result
                .coefficient_of_variation
                .get(metric)
                .copied()
                .unwrap_or(0.0);
            let (ci_lower, ci_upper) = result
                .confidence_intervals_95
                .get(metric)
                .copied()
                .unwrap_or((0.0, 0.0));

            metric_reports.insert(
                metric.clone(),
                MetricReport {
                    mean,
                    std_dev: std,
                    cv,
                    ci_lower,
                    ci_upper,
                    stability: StabilityLevel::from_cv(cv),
                },
            );
        }

        let (overall_cv, overall_stability) =
            if result.coefficient_of_variation.is_empty() || !result.is_stable {
                let cv_val = if result.coefficient_of_variation.is_empty() {
                    1.0
                } else {
                    result.coefficient_of_variation.values().sum::<f64>()
                        / result.coefficient_of_variation.len() as f64
                };
                (cv_val, StabilityLevel::Unstable)
            } else {
                let cv_val = result.coefficient_of_variation.values().sum::<f64>()
                    / result.coefficient_of_variation.len().max(1) as f64;
                (cv_val, StabilityLevel::from_cv(cv_val))
            };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let summary = if result.is_stable {
            format!("Results are stable (avg CV={:.1}%)", overall_cv * 100.0)
        } else {
            format!(
                "Results show instability (avg CV={:.1}%). Additional replications recommended.",
                overall_cv * 100.0
            )
        };

        let convergence_assessment = if result.convergence.has_converged {
            format!(
                "Converged at {} replications",
                result.convergence.replication_at_convergence.unwrap_or(0)
            )
        } else {
            "Not converged. Consider increasing replications.".to_string()
        };

        ReproducibilityReport {
            summary,
            metrics_summary: metric_reports,
            overall_stability,
            convergence_assessment,
            recommendations: result.recommendations.clone(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compare two result sets for statistical equivalence using Schuirmann's (1987)
    /// Two One-Sided Welch t-Tests (TOST, alpha=0.05 -> 90% Welch-Satterthwaite CI inside [-delta, +delta]).
    #[must_use]
    pub fn compare_for_equivalence(
        &self,
        results_a: &[ReplicationResult],
        results_b: &[ReplicationResult],
        metric: &str,
        threshold: f64,
    ) -> bool {
        let eff_threshold = if threshold > 0.0 {
            threshold
        } else {
            self.config.stability_threshold
        };
        let values_a: Vec<f64> = results_a
            .iter()
            .filter(|r| r.success)
            .filter_map(|r| r.metrics.get(metric).copied())
            .filter(|v| v.is_finite())
            .collect();

        let values_b: Vec<f64> = results_b
            .iter()
            .filter(|r| r.success)
            .filter_map(|r| r.metrics.get(metric).copied())
            .filter(|v| v.is_finite())
            .collect();

        if values_a.len() < 2 || values_b.len() < 2 {
            return false;
        }

        let n_a = values_a.len() as f64;
        let n_b = values_b.len() as f64;
        let mean_a = values_a.iter().sum::<f64>() / n_a;
        let mean_b = values_b.iter().sum::<f64>() / n_b;

        let var_a = values_a.iter().map(|&v| (v - mean_a).powi(2)).sum::<f64>() / (n_a - 1.0);
        let var_b = values_b.iter().map(|&v| (v - mean_b).powi(2)).sum::<f64>() / (n_b - 1.0);

        let scale = mean_a.abs().max(mean_b.abs()).max(1e-12);
        let delta_abs = eff_threshold * scale;
        let diff = (mean_a - mean_b).abs();

        let se_sq = (var_a / n_a) + (var_b / n_b);
        if se_sq <= 1e-18 {
            return diff < delta_abs;
        }

        let se = se_sq.sqrt();
        // Welch-Satterthwaite effective degrees of freedom
        let t1 = var_a / n_a;
        let t2 = var_b / n_b;
        let df_den = (t1 * t1) / (n_a - 1.0) + (t2 * t2) / (n_b - 1.0);
        let df = if df_den > 1e-18 {
            (se_sq * se_sq) / df_den
        } else {
            n_a + n_b - 2.0
        };

        // Two-sided 90% Student-t critical value (alpha = 0.05 per one-sided TOST bound)
        let t_crit_90 = Self::student_t_90_crit(df);
        (diff + t_crit_90 * se) < delta_abs
    }

    fn student_t_90_crit(df: f64) -> f64 {
        let nu = df.max(1.0);
        let d_int = nu.round() as usize;
        if (nu - d_int as f64).abs() < 0.25 && d_int <= 10 {
            match d_int {
                1 => 6.313_751_514_800_932,
                2 => 2.919_985_580_355_516,
                3 => 2.353_363_434_801_826_4,
                4 => 2.131_846_781_336_29,
                5 => 2.015_048_372_669_157,
                6 => 1.943_180_280_392_781_6,
                7 => 1.894_578_605_061_305,
                8 => 1.859_548_037_522_842_1,
                9 => 1.833_112_932_653_633_5,
                _ => 1.812_461_122_810_733_5,
            }
        } else {
            let z = 1.644_853_626_951_472_2_f64;
            let z3 = z.powi(3);
            let z5 = z.powi(5);
            z + (z3 + z) / (4.0 * nu) + (5.0 * z5 + 16.0 * z3 + 3.0 * z) / (96.0 * nu * nu)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_results(n: usize, seed_base: u64) -> Vec<ReplicationResult> {
        (0..n)
            .map(|i| {
                let noise = (i as f64 - n as f64 / 2.0) * 0.01;
                let mut metrics = HashMap::new();
                metrics.insert("fitness".to_string(), 0.85 + noise);
                metrics.insert("diversity".to_string(), 0.45 + noise * 0.5);

                // TAG: surface=simulation owner=research-team rule=SIM-001
                ReplicationResult {
                    index: i,
                    seed: seed_base + i as u64,
                    metrics,
                    execution_time_secs: 1.0,
                    success: true,
                    error: None,
                }
            })
            .collect()
    }

    #[test]
    fn test_stability_analyzer() {
        let config = ReproducibilityConfig::default();
        let analyzer = StabilityAnalyzer::new(config);
        let results = create_test_results(30, 42);

        let result = analyzer.analyze(&results);

        assert!(result.stability_score > 0.0);
        assert!(!result.mean_values.is_empty());
        assert!(!result.recommendations.is_empty());
    }

    #[test]
    fn test_equivalence_check() {
        let config = ReproducibilityConfig::default();
        let analyzer = StabilityAnalyzer::new(config);
        let results_a = create_test_results(30, 42);
        let results_b = create_test_results(30, 100);

        let is_equivalent =
            analyzer.compare_for_equivalence(&results_a, &results_b, "fitness", 0.05);
        assert!(is_equivalent); // Should be equivalent with small variations

        // Strict symmetry: compare_for_equivalence(A, B) == compare_for_equivalence(B, A)
        let is_equivalent_rev =
            analyzer.compare_for_equivalence(&results_b, &results_a, "fitness", 0.05);
        assert_eq!(is_equivalent, is_equivalent_rev);

        // Noisy small-sample N=2 series with close means must FAIL Welch TOST equivalence
        let mut m1 = HashMap::new();
        m1.insert("fitness".to_string(), 0.0);
        let mut m2 = HashMap::new();
        m2.insert("fitness".to_string(), 20.0);
        let noisy_a = vec![
            ReplicationResult {
                index: 0,
                seed: 1,
                metrics: m1,
                execution_time_secs: 1.0,
                success: true,
                error: None,
            },
            ReplicationResult {
                index: 1,
                seed: 2,
                metrics: m2,
                execution_time_secs: 1.0,
                success: true,
                error: None,
            },
        ];
        let mut m3 = HashMap::new();
        m3.insert("fitness".to_string(), 0.1);
        let mut m4 = HashMap::new();
        m4.insert("fitness".to_string(), 20.1);
        let noisy_b = vec![
            ReplicationResult {
                index: 0,
                seed: 3,
                metrics: m3,
                execution_time_secs: 1.0,
                success: true,
                error: None,
            },
            ReplicationResult {
                index: 1,
                seed: 4,
                metrics: m4,
                execution_time_secs: 1.0,
                success: true,
                error: None,
            },
        ];
        assert!(
            !analyzer.compare_for_equivalence(&noisy_a, &noisy_b, "fitness", 0.05),
            "High-variance N=2 series must NOT pass Welch TOST equivalence"
        );
    }

    #[test]
    fn test_empty_cohort_recommendations_not_vacuously_good() {
        let config = ReproducibilityConfig::default();
        let analyzer = StabilityAnalyzer::new(config);

        // Case 1: 0 replications
        let empty_results = Vec::<ReplicationResult>::new();
        let report_empty = analyzer.generate_report(&analyzer.analyze(&empty_results));
        assert!(report_empty
            .recommendations
            .iter()
            .any(|r| r.contains("No successful replications found")));
        assert!(!report_empty
            .recommendations
            .iter()
            .any(|r| r.contains("good reproducibility")));

        // Case 2: 1 replication (insufficient for variance estimation)
        let single_result = create_test_results(1, 42);
        let report_single = analyzer.generate_report(&analyzer.analyze(&single_result));
        assert!(report_single
            .recommendations
            .iter()
            .any(|r| r.contains("Insufficient valid replications")));
        assert!(!report_single
            .recommendations
            .iter()
            .any(|r| r.contains("good reproducibility")));
    }
}
