//! Variance Estimation for Reproducibility Analysis
//!
//! Tools for estimating variance components and required replication counts.

use std::f64::consts::SQRT_2;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Variance estimator for reproducibility analysis
#[derive(Debug)]
pub struct VarianceEstimator;

impl VarianceEstimator {
    /// Create new variance estimator
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate required number of replications to achieve target CV
    ///
    /// Uses the formula: `n_target` = `n_observed` * (`CV_observed` / `CV_target)^2`
    #[must_use]
    pub fn required_replications(
        &self,
        observed_cv: f64,
        observed_n: usize,
        target_cv: f64,
    ) -> usize {
        if observed_cv <= 0.0 || target_cv <= 0.0 || observed_n == 0 {
            return observed_n.max(30);
        }

        let ratio = observed_cv / target_cv;
        let required = (observed_n as f64 * ratio.powi(2)).ceil() as usize;
        required.min(1000).max(observed_n) // Cap at 1000, ensure minimum is observed_n
    }

    /// Calculate statistical power for detecting effect
    #[must_use]
    pub fn calculate_power(
        &self,
        effect_size: f64,
        std_dev: f64,
        sample_size: usize,
        alpha: f64,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    ) -> f64 {
        if std_dev <= 0.0 || sample_size == 0 {
            return 0.0;
        }

        let se = std_dev / (sample_size as f64).sqrt();
        let alpha = alpha.clamp(1e-12, 1.0 - 1e-12);
        let z_alpha = self.normal_quantile(1.0 - alpha / 2.0);
        let delta = effect_size.abs() / se;

        (self.normal_cdf(delta - z_alpha) + self.normal_cdf(-delta - z_alpha)).clamp(0.0, 1.0)
    }

    /// Standard normal CDF approximation
    fn normal_cdf(&self, z: f64) -> f64 {
        0.5 * (1.0 + self.erf(z / SQRT_2))
    }

    /// Standard normal quantile approximation
    fn normal_quantile(&self, probability: f64) -> f64 {
        // Approximation for inverse CDF
        if probability <= 0.0 {
            return -3.0;
        }
        if probability >= 1.0 {
            return 3.0;
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Beasley-Springer-Moro approximation
        let a0 = 2.50662823884;
        let a1 = -18.61500062529;
        let a2 = 41.39119773534;
        let a3 = -25.44106049637;
        let b1 = -8.47351093090;
        let b2 = 23.08336743743;
        let b3 = -21.06224101826;
        let b4 = 3.13082909833;
        let c0 = 0.3374754822726147;
        let c1 = 0.9761690190917186;
        let c2 = 0.1607979714918209;
        let c3 = 0.0276438810333863;
        let c4 = 0.0038405729373609;
        let c5 = 0.0003951896511919;
        let c6 = 0.0000321767881768;
        let c7 = 0.0000002888167364;
        let c8 = 0.0000003960315187;

        let shifted = probability - 0.5;

        if shifted.abs() < 0.42 {
            let rr = shifted * shifted;
            shifted * (((a3 * rr + a2) * rr + a1) * rr + a0)
                / ((((b4 * rr + b3) * rr + b2) * rr + b1) * rr + 1.0)
        } else {
            let tail = if shifted < 0.0 {
                probability
            } else {
                1.0 - probability
            };
            let log_log = (-tail.ln()).ln();
            let poly = c0
                + log_log
                    * (c1
                        + log_log
                            * (c2
                                + log_log
                                    * (c3
                                        + log_log
                                            * (c4
                                                + log_log
                                                    * (c5
                                                        + log_log
                                                            * (c6
                                                                + log_log
                                                                    * (c7 + log_log * c8)))))));
            if shifted < 0.0 {
                -poly
            } else {
                poly
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Error function approximation
    fn erf(&self, x: f64) -> f64 {
        let sign = if x < 0.0 { -1.0 } else { 1.0 };
        let x = x.abs();

        let a1 = 0.254829592;
        let a2 = -0.284496736;
        let a3 = 1.421413741;
        let a4 = -1.453152027;
        let a5 = 1.061405429;
        let p = 0.3275911;

        let t = 1.0 / (1.0 + p * x);
        let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

        sign * y
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for VarianceEstimator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_replications() {
        let estimator = VarianceEstimator::new();

        // If observed CV is 10% and we want 5%, we need 4x more replications
        let required = estimator.required_replications(0.10, 30, 0.05);
        assert_eq!(required, 120);

        // If CV is already at target, keep current
        let required = estimator.required_replications(0.10, 30, 0.10);
        assert_eq!(required, 30);
    }

    #[test]
    fn test_calculate_power() {
        let estimator = VarianceEstimator::new();

        // Large effect size should give high power
        let power = estimator.calculate_power(1.0, 1.0, 100, 0.05);
        assert!(power > 0.9, "Expected high power, got {power}");

        // Small effect size with small sample should give low power
        let power = estimator.calculate_power(0.1, 1.0, 10, 0.05);
        assert!(power < 0.5, "Expected low power, got {power}");
    }
    #[test]
    fn test_calculate_power_symmetric_negative() {
        let estimator = VarianceEstimator::new();
        let pos_power = estimator.calculate_power(1.0, 1.0, 100, 0.05);
        let neg_power = estimator.calculate_power(-1.0, 1.0, 100, 0.05);
        assert!(
            (pos_power - neg_power).abs() < 1e-6,
            "Power must be symmetric for two-sided tests"
        );
    }

    #[test]
    fn test_calculate_power_null_hypothesis_equals_alpha() {
        let estimator = VarianceEstimator::new();
        let alpha = 0.05;
        let power_null = estimator.calculate_power(0.0, 1.0, 100, alpha);
        assert!(
            (power_null - alpha).abs() < 1e-4,
            "Two-sided power under null effect must equal alpha ({alpha}), got {power_null}"
        );
    }
}
