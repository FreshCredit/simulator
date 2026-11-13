//! Sample size calculator
//!
//! Calculates required sample sizes for various statistical tests
//! to achieve desired power levels.

use super::calculator::PowerCalculator;
use super::types::*;
use serde::{Deserialize, Serialize};
use tracing::debug;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Calculator for required sample sizes
#[derive(Debug)]
pub struct SampleSizeCalculator {
    power_calc: PowerCalculator,
}

impl SampleSizeCalculator {
    /// Create a new sample size calculator
    #[must_use]
    pub const fn new() -> Self {
        Self {
            power_calc: PowerCalculator::new(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate required sample size for given configuration
    pub fn calculate_sample_size(&self, config: &PowerAnalysisConfig) -> SampleSizeResult {
        let target_power = config.target_power;
        let effect_size = config.effect_size.magnitude;

        debug!(
            "Calculating sample size for target_power={}, effect_size={}",
            target_power, effect_size
        );

        // Find sample size for target power
        let n_target = self
            .find_sample_size_binary_search(config, target_power, 2, 10000)
            .unwrap_or(10000);

        // Calculate sample sizes for standard power levels ensuring monotonicity
        let n_80 = self
            .find_sample_size_binary_search(config, 0.80, 2, 10000)
            .unwrap_or(10000);
        let n_90 = self
            .find_sample_size_binary_search(config, 0.90, 2, 10000)
            .unwrap_or(10000)
            .max(n_80);
        let n_95 = self
            .find_sample_size_binary_search(config, 0.95, 2, 10000)
            .unwrap_or(10000)
            .max(n_90);

        // Generate power curve
        let power_curve = self.generate_sample_size_curve(config, n_target);

        SampleSizeResult {
            n_per_group: n_target,
            total_n: n_target * config.num_groups,
            n_for_80_power: n_80,
            n_for_90_power: n_90,
            n_for_95_power: n_95,
            power_curve,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate sample size using analytical formulas where available
    #[must_use]
    pub fn calculate_analytical(&self, config: &PowerAnalysisConfig) -> usize {
        let d = config.effect_size.magnitude;
        let alpha = config.alpha;
        let power = config.target_power;

        match config.test {
            StatisticalTest::TTestIndependent => {
                self.t_test_independent_n(d, alpha, power, config.one_tailed)
            }
            StatisticalTest::TTestOneSample => {
                self.t_test_one_sample_n(d, alpha, power, config.one_tailed)
            }
            StatisticalTest::TTestPaired => {
                self.t_test_paired_n(d, alpha, power, config.one_tailed)
            }
            StatisticalTest::AnovaOneWay => {
                let n_approx = self.anova_one_way_n(d, config.num_groups, alpha, power);
                self.find_sample_size_binary_search(
                    config,
                    power,
                    (n_approx / 2).max(2),
                    (n_approx * 2).min(10000),
                )
                .unwrap_or(n_approx)
            }
            StatisticalTest::CorrelationPearson => {
                self.correlation_n(d, alpha, power, config.one_tailed)
            }
            _ => {
                // Fall back to iterative search
                self.find_sample_size_binary_search(config, power, 2, 10000)
                    .unwrap_or(100)
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            }
        }
    }

    /// Sample size for independent samples t-test
    /// Formula: n = 2 * ((z_{1-α/2} + z_{1-β}) / d)^2
    fn t_test_independent_n(&self, d: f64, alpha: f64, power: f64, one_tailed: bool) -> usize {
        if !d.is_finite() || !alpha.is_finite() || !power.is_finite() {
            return 10000;
        }
        let d_abs = d.abs();
        if d_abs < 1e-4 {
            return 10000;
        }
        let z_alpha = if one_tailed {
            z_critical(alpha)
        } else {
            z_critical(alpha / 2.0)
        };
        let z_beta = z_critical(1.0 - power);

        let z_sum = (z_alpha + z_beta).max(0.0);
        let n: f64 = 2.0 * (z_sum / d_abs).powi(2);
        n.ceil().clamp(2.0, 10000.0) as usize
    }

    /// Sample size for one-sample t-test
    /// Formula: n = ((z_{1-α/2} + z_{1-β}) / d)^2
    fn t_test_one_sample_n(&self, d: f64, alpha: f64, power: f64, one_tailed: bool) -> usize {
        if !d.is_finite() || !alpha.is_finite() || !power.is_finite() {
            return 10000;
        }
        let d_abs = d.abs();
        if d_abs < 1e-4 {
            return 10000;
        }
        let z_alpha = if one_tailed {
            z_critical(alpha)
        } else {
            z_critical(alpha / 2.0)
        };
        let z_beta = z_critical(1.0 - power);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let z_sum = (z_alpha + z_beta).max(0.0);
        let n = (z_sum / d_abs).powi(2);
        n.ceil().clamp(2.0, 10000.0) as usize
    }

    /// Sample size for paired samples t-test
    /// Same formula as one-sample (applied to difference scores)
    fn t_test_paired_n(&self, d: f64, alpha: f64, power: f64, one_tailed: bool) -> usize {
        self.t_test_one_sample_n(d, alpha, power, one_tailed)
    }

    /// Sample size for one-way ANOVA
    /// Using Cohen's f effect size
    fn anova_one_way_n(&self, f: f64, k: usize, alpha: f64, power: f64) -> usize {
        if !f.is_finite() || !alpha.is_finite() || !power.is_finite() {
            return 10000;
        }
        let f_abs = f.abs();
        if f_abs < 1e-4 {
            return 10000;
        }
        let z_alpha = z_critical(alpha / 2.0);
        let z_beta = z_critical(1.0 - power);

        // Approximate formula for equal group sizes
        let z_sum = (z_alpha + z_beta).max(0.0);
        let numerator = z_sum.powi(2);
        let denominator = f_abs * f_abs * (k.max(1) as f64);

        let n = numerator / denominator;
        n.ceil().clamp(2.0, 10000.0) as usize
    }

    /// Sample size for Pearson correlation
    fn correlation_n(&self, r: f64, alpha: f64, power: f64, one_tailed: bool) -> usize {
        if !r.is_finite() || !alpha.is_finite() || !power.is_finite() {
            return 10000;
        }
        let r_clamped = r.clamp(-0.9999, 0.9999);
        if r_clamped.abs() < 1e-4 {
            return 10000;
        }

        // Fisher z-transformation of r
        let z_r = ((1.0 + r_clamped) / (1.0 - r_clamped)).ln() / 2.0;

        let z_alpha = if one_tailed {
            z_critical(alpha)
        } else {
            z_critical(alpha / 2.0)
        };
        let z_beta = z_critical(1.0 - power);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let z_sum = (z_alpha + z_beta).max(0.0);
        let n = (z_sum / z_r.abs()).powi(2) + 3.0;
        n.ceil().clamp(4.0, 10000.0) as usize
    }

    /// Find sample size using binary search for target power
    fn find_sample_size_binary_search(
        &self,
        config: &PowerAnalysisConfig,
        target_power: f64,
        min_n: usize,
        max_n: usize,
    ) -> Option<usize> {
        let effective_min = if matches!(config.test, StatisticalTest::CorrelationPearson) {
            min_n.max(4)
        } else {
            min_n.max(2)
        };
        let mut low = effective_min;
        let mut high = max_n;
        let mut result = None;

        while low <= high {
            let mid = (low + high) / 2;
            let power_result = self.power_calc.calculate_power(config, mid);

            if power_result.power >= target_power {
                result = Some(mid);
                high = mid - 1;
            } else {
                low = mid + 1;
            }
        }

        result
    }

    /// Generate power curve showing power at different sample sizes
    fn generate_sample_size_curve(
        &self,
        config: &PowerAnalysisConfig,
        target_n: usize,
    ) -> Vec<PowerCurvePoint> {
        let mut curve = Vec::new();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Generate curve around target sample size
        let n_min = (target_n as f64 * 0.25) as usize;
        let n_max = (target_n as f64 * 2.0) as usize;
        let step = ((n_max - n_min) / 20).max(1);

        let mut n = n_min.max(2);
        while n <= n_max {
            let power_result = self.power_calc.calculate_power(config, n);
            curve.push(PowerCurvePoint {
                n,
                power: power_result.power,
            });
            n += step;
        }

        curve
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate sample size with attrition adjustment
    #[must_use]
    pub fn calculate_with_attrition(
        &self,
        config: &PowerAnalysisConfig,
        attrition_rate: f64,
    ) -> SampleSizeResult {
        let base_result = self.calculate_sample_size(config);

        // Adjust for expected attrition with safe bounds
        let attrition_clamped = if attrition_rate.is_finite() {
            attrition_rate.clamp(0.0, 0.99)
        } else {
            0.0
        };
        let attrition_factor = 1.0 / (1.0 - attrition_clamped);

        SampleSizeResult {
            n_per_group: (base_result.n_per_group as f64 * attrition_factor).ceil() as usize,
            total_n: (base_result.total_n as f64 * attrition_factor).ceil() as usize,
            n_for_80_power: (base_result.n_for_80_power as f64 * attrition_factor).ceil() as usize,
            n_for_90_power: (base_result.n_for_90_power as f64 * attrition_factor).ceil() as usize,
            n_for_95_power: (base_result.n_for_95_power as f64 * attrition_factor).ceil() as usize,
            power_curve: base_result.power_curve,
        }
    }

    /// Calculate sample size for multiple groups with unequal allocation
    #[must_use]
    pub fn calculate_unequal_groups(
        &self,
        config: &PowerAnalysisConfig,
        allocation_ratios: &[f64],
    ) -> Vec<usize> {
        let base_n = self.calculate_sample_size(config).n_per_group;
        if allocation_ratios.is_empty() {
            return vec![base_n; config.num_groups];
        }

        // Fleiss / Cohen harmonic-mean non-centrality preservation:
        // Harmonic mean H(r) = k / sum(1 / r_i).
        // Each group size n_i = ceil(base_n * (r_i / H(r))) guarantees
        // the effective harmonic sample size >= base_n, preserving target statistical power.
        let k = allocation_ratios.len() as f64;
        let sum_inv: f64 = allocation_ratios
            .iter()
            .map(|&r| {
                if r.is_finite() && r > 1e-6 {
                    1.0 / r
                } else {
                    1.0
                }
            })
            .sum();
        let harmonic_mean = if sum_inv > 0.0 { k / sum_inv } else { 1.0 };

        allocation_ratios
            .iter()
            .map(|&ratio| {
                let r_safe = ratio.max(1e-4);
                let scaled = (base_n as f64) * (r_safe / harmonic_mean);
                scaled.ceil().max(2.0) as usize
            })
            .collect()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Validate sample size calculation
    #[must_use]
    pub fn validate_calculation(
        &self,
        config: &PowerAnalysisConfig,
        proposed_n: usize,
    ) -> SampleSizeValidation {
        let result = self.power_calc.calculate_power(config, proposed_n);
        let target_power = config.target_power;

        SampleSizeValidation {
            proposed_n,
            achieved_power: result.power,
            target_power,
            is_adequate: result.power >= target_power,
            power_deficit: if result.power < target_power {
                target_power - result.power
            } else {
                0.0
            },
            recommendation: if result.power < target_power {
                let required_n = self.calculate_sample_size(config).n_per_group;
                format!(
                    "Increase sample size to {} per group for {}% power",
                    required_n,
                    (target_power * 100.0) as i32
                )
            } else {
                "Sample size is adequate".to_string()
            },
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get sample size table for different effect sizes
    #[must_use]
    pub fn generate_effect_size_table(
        &self,
        config: &PowerAnalysisConfig,
        effect_sizes: &[f64],
    ) -> Vec<EffectSizeSampleSize> {
        let mut table = Vec::new();
        let mut test_config = config.clone();

        for &es in effect_sizes {
            test_config.effect_size.magnitude = es;
            let n = self.calculate_sample_size(&test_config).n_per_group;

            table.push(EffectSizeSampleSize {
                effect_size: es,
                n_per_group: n,
                interpretation: EffectSizeInterpretation::from_cohens_d(es),
            });
        }

        table
    }
}

/// Sample size validation result
#[derive(Debug, Clone)]
pub struct SampleSizeValidation {
    /// proposed n
    pub proposed_n: usize,
    /// achieved power
    pub achieved_power: f64,
    /// target power
    pub target_power: f64,
    /// is adequate
    pub is_adequate: bool,
    /// power deficit
    pub power_deficit: f64,
    /// recommendation
    pub recommendation: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Effect size to sample size mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectSizeSampleSize {
    /// effect size
    pub effect_size: f64,
    /// n per group
    pub n_per_group: usize,
    /// interpretation
    pub interpretation: EffectSizeInterpretation,
}

impl Default for SampleSizeCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Critical z-value for given alpha (normal distribution)
fn z_critical(alpha: f64) -> f64 {
    let alpha = alpha.clamp(1e-12, 1.0 - 1e-12);
    // Approximation of inverse normal CDF using the rational approximation
    let p = if alpha < 0.5 { alpha } else { 1.0 - alpha };

    let a1 = -3.969_683_028_665_376e1;
    let a2 = 2.209_460_984_245_205e2;
    let a3 = -2.759_285_104_469_687e2;
    let a4 = 1.383_577_518_672_69e2;
    let a5 = -3.066_479_806_614_716e1;
    let a6 = 2.506_628_277_459_239;

    let b1 = -5.447_609_879_822_406e1;
    let b2 = 1.615_858_368_580_409e2;
    let b3 = -1.556_989_798_598_866e2;
    let b4 = 6.680_131_188_771_972e1;
    let b5 = -1.328_068_155_288_572e1;

    let c1 = -7.784_894_002_430_293e-3;
    let c2 = -3.223_964_580_411_365e-1;
    let c3 = -2.400_758_277_161_838;
    let c4 = -2.549_732_539_343_734;
    let c5 = 4.374_664_141_464_968;
    let c6 = 2.938_163_982_698_783;

    let d1 = 7.784_695_709_041_462e-3;
    let d2 = 3.224_671_290_700_398e-1;
    let d3 = 2.445_134_137_142_996;
    let d4 = 3.754_408_661_907_416;

    let p_low = 0.02425;
    let p_high = 1.0 - p_low;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let z = if p < p_low {
        let q = (-2.0 * p.ln()).sqrt();
        (((((c1 * q + c2) * q + c3) * q + c4) * q + c5) * q + c6)
            / ((((d1 * q + d2) * q + d3) * q + d4) * q + 1.0)
    } else if p <= p_high {
        let q = p - 0.5;
        let r = q * q;
        ((((((a1 * r + a2) * r + a3) * r + a4) * r + a5) * r + a6) * q)
            / (((((b1 * r + b2) * r + b3) * r + b4) * r + b5) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((c1 * q + c2) * q + c3) * q + c4) * q + c5) * q + c6)
            / ((((d1 * q + d2) * q + d3) * q + d4) * q + 1.0)
    };

    if alpha < 0.5 {
        -z
    } else {
        z
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_size_calculation() {
        let calc = SampleSizeCalculator::new();
        let config = PowerAnalysisConfig::default();

        let result = calc.calculate_sample_size(&config);

        // For d=0.5, alpha=0.05, power=0.80, n should be ~64 per group
        // Using approximations, accept reasonable range
        assert!(
            result.n_per_group >= 50 && result.n_per_group <= 100,
            "Expected n around 64, got {}",
            result.n_per_group
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_analytical_vs_iterative() {
        let calc = SampleSizeCalculator::new();
        let config = PowerAnalysisConfig::default();

        let analytical = calc.calculate_analytical(&config);
        let iterative = calc.calculate_sample_size(&config).n_per_group;

        // Results should be close
        assert!(
            (analytical as i32 - iterative as i32).abs() <= 5,
            "Analytical: {analytical}, Iterative: {iterative}"
        );
    }

    #[test]
    fn test_attrition_adjustment() {
        let calc = SampleSizeCalculator::new();
        let config = PowerAnalysisConfig::default();

        let base = calc.calculate_sample_size(&config);
        let with_attrition = calc.calculate_with_attrition(&config, 0.20); // 20% attrition

        // With 20% attrition, we need 25% more participants
        let expected_increase = (base.n_per_group as f64 / 0.80).ceil() as usize;
        assert_eq!(with_attrition.n_per_group, expected_increase);
    }

    #[test]
    fn test_validation_adequate() {
        let calc = SampleSizeCalculator::new();
        let config = PowerAnalysisConfig::default();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let required_n = calc.calculate_sample_size(&config).n_per_group;
        let validation = calc.validate_calculation(&config, required_n);

        assert!(validation.is_adequate);
        assert!(validation.power_deficit == 0.0);
    }

    #[test]
    fn test_validation_inadequate() {
        let calc = SampleSizeCalculator::new();
        let config = PowerAnalysisConfig::default();

        // Use very small sample size
        let validation = calc.validate_calculation(&config, 10);

        assert!(!validation.is_adequate);
        assert!(validation.power_deficit > 0.0);
    }

    #[test]
    fn test_effect_size_table() {
        let calc = SampleSizeCalculator::new();
        let config = PowerAnalysisConfig::default();

        let effect_sizes = vec![0.2, 0.5, 0.8];
        let table = calc.generate_effect_size_table(&config, &effect_sizes);

        assert_eq!(table.len(), 3);
        // Smaller effect sizes should require larger samples
        assert!(table[0].n_per_group > table[2].n_per_group);
    }

    #[test]
    fn test_sample_size_monotonic_in_power() {
        let calc = SampleSizeCalculator::new();
        let mut config = PowerAnalysisConfig::default();

        config.target_power = 0.50;
        let n_50 = calc.calculate_analytical(&config);

        config.target_power = 0.80;
        let n_80 = calc.calculate_analytical(&config);

        config.target_power = 0.95;
        let n_95 = calc.calculate_analytical(&config);

        assert!(n_50 <= n_80, "n_50 ({n_50}) must be <= n_80 ({n_80})");
        assert!(n_80 <= n_95, "n_80 ({n_80}) must be <= n_95 ({n_95})");
    }

    #[test]
    fn test_calculate_unequal_groups_preserves_harmonic_mean() {
        let calc = SampleSizeCalculator::new();
        let config = PowerAnalysisConfig::default();

        let base_n = calc.calculate_sample_size(&config).n_per_group;
        let ratios = vec![1.0, 2.0];
        let unequal_ns = calc.calculate_unequal_groups(&config, &ratios);

        assert_eq!(unequal_ns.len(), 2);
        let harmonic_mean = 2.0 / (1.0 / unequal_ns[0] as f64 + 1.0 / unequal_ns[1] as f64);
        assert!(
            harmonic_mean >= base_n as f64 - 0.5,
            "Harmonic mean {harmonic_mean} must preserve base equal sample size {base_n}"
        );
    }

    #[test]
    fn test_sample_size_nan_guards_and_extreme_power_monotonicity() {
        let calc = SampleSizeCalculator::new();
        let mut config = PowerAnalysisConfig::default();

        // 1. Monotonicity of standard power levels even for tiny effect sizes
        config.effect_size.magnitude = 0.01;
        let res = calc.calculate_sample_size(&config);
        assert!(res.n_for_80_power <= res.n_for_90_power);
        assert!(res.n_for_90_power <= res.n_for_95_power);

        // 2. NaN effect size guards
        config.effect_size.magnitude = f64::NAN;
        let analytical_nan = calc.calculate_analytical(&config);
        assert_eq!(analytical_nan, 10000);
        assert!(analytical_nan > 0);
    }
}
