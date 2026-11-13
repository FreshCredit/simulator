//! Statistical power calculator
//!
//! Implements power calculations for various statistical tests using
//! normal approximations and non-central distributions.

use super::types::*;
use tracing::debug;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Calculator for statistical power
#[derive(Debug)]
pub struct PowerCalculator;

impl PowerCalculator {
    /// Create a new power calculator
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Calculate statistical power given sample size and parameters
    pub fn calculate_power(
        &self,
        config: &PowerAnalysisConfig,
        n_per_group: usize,
    ) -> PowerCalculationResult {
        let effect_size = config.effect_size.magnitude;
        let alpha = config.alpha;
        let n = n_per_group as f64;

        // Calculate power based on test type
        let power = match config.test {
            StatisticalTest::TTestIndependent => {
                self.t_test_independent_power(n, effect_size, alpha, config.one_tailed)
            }
            StatisticalTest::TTestOneSample => {
                self.t_test_one_sample_power(n, effect_size, alpha, config.one_tailed)
            }
            StatisticalTest::TTestPaired => {
                self.t_test_paired_power(n, effect_size, alpha, config.one_tailed)
            }
            StatisticalTest::AnovaOneWay => {
                self.anova_one_way_power(n, config.num_groups as f64, effect_size, alpha)
                // TAG: surface=simulation owner=research-team rule=SIM-001
            }
            StatisticalTest::CorrelationPearson => {
                self.correlation_power(n, effect_size, alpha, config.one_tailed)
            }
            StatisticalTest::ChiSquare => {
                let total_n = (n_per_group * config.num_groups.max(1)) as f64;
                let df = config.num_groups.saturating_sub(1).max(1);
                self.chi_square_power(total_n, effect_size, df, alpha)
            }
            _ => {
                // For unsupported tests, use t-test approximation
                debug!("Using t-test approximation for {:?}", config.test);
                self.t_test_independent_power(n, effect_size, alpha, config.one_tailed)
            }
        };

        let power_level = PowerLevel::from_power(power);
        let total_n = n_per_group * config.num_groups;

        let recommendations =
            self.generate_recommendations(power, config.target_power, n_per_group);

        PowerCalculationResult {
            power,
            required_n_per_group: n_per_group,
            total_required_n: total_n,
            alpha,
            effect_size,
            power_level,
            recommendations,
        }
    }

    /// Calculate power for independent samples t-test
    /// Uses normal approximation: power = Φ(δ√(n/2) - z_{1-α}) (+ Φ(-δ√(n/2) - z_{1-α/2}) for two-tailed)
    fn t_test_independent_power(&self, n: f64, d: f64, alpha: f64, one_tailed: bool) -> f64 {
        if n <= 1.0 || !d.is_finite() || !alpha.is_finite() || alpha <= 0.0 || alpha >= 1.0 {
            return alpha.clamp(0.0, 1.0);
        }
        let z_alpha = if one_tailed {
            self.z_critical(alpha)
        } else {
            self.z_critical(alpha / 2.0)
        };
        let delta = d.abs() * (n / 2.0).sqrt();
        if one_tailed {
            self.normal_cdf(delta - z_alpha).clamp(0.0, 1.0)
        } else {
            (self.normal_cdf(delta - z_alpha) + self.normal_cdf(-delta - z_alpha)).clamp(0.0, 1.0)
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate power for one-sample t-test
    fn t_test_one_sample_power(&self, n: f64, d: f64, alpha: f64, one_tailed: bool) -> f64 {
        if n <= 1.0 || !d.is_finite() || !alpha.is_finite() || alpha <= 0.0 || alpha >= 1.0 {
            return alpha.clamp(0.0, 1.0);
        }
        let z_alpha = if one_tailed {
            self.z_critical(alpha)
        } else {
            self.z_critical(alpha / 2.0)
        };
        let delta = d.abs() * n.sqrt();
        if one_tailed {
            self.normal_cdf(delta - z_alpha).clamp(0.0, 1.0)
        } else {
            (self.normal_cdf(delta - z_alpha) + self.normal_cdf(-delta - z_alpha)).clamp(0.0, 1.0)
        }
    }

    /// Calculate power for paired samples t-test
    fn t_test_paired_power(&self, n: f64, d: f64, alpha: f64, one_tailed: bool) -> f64 {
        // Paired t-test has same formula as one-sample on difference scores
        self.t_test_one_sample_power(n, d, alpha, one_tailed)
    }

    /// Calculate power for one-way ANOVA using Cohen's f and Patnaik's non-central F approximation
    fn anova_one_way_power(&self, n: f64, k: f64, f: f64, alpha: f64) -> f64 {
        if n <= 1.0
            || k <= 1.0
            || !f.is_finite()
            || !alpha.is_finite()
            || alpha <= 0.0
            || alpha >= 1.0
        {
            return alpha;
        }

        let f_abs = f.abs();
        let lambda = n * f_abs * f_abs * k;
        let df_between = k - 1.0;
        let df_within = k * (n - 1.0);

        if lambda < 1e-12 {
            return alpha;
        }

        let f_crit = self.f_critical(df_between, df_within, alpha);

        // Patnaik (1949) approximation: non-central F(nu1, nu2; lambda) ~ c * F(nu*, nu2)
        // where c = (nu1 + lambda) / nu1, nu* = (nu1 + lambda)^2 / (nu1 + 2*lambda)
        let c = (df_between + lambda) / df_between;
        let nu_star = (df_between + lambda).powi(2) / (df_between + 2.0 * lambda);
        let x = (f_crit / c).max(1e-12);

        // Paulson (1942) normal approximation of F(nu*, df_within)
        let a = 2.0 / (9.0 * nu_star);
        let b = 2.0 / (9.0 * df_within);
        let x_cbrt = x.cbrt();
        let num = (1.0 - b) * x_cbrt - (1.0 - a);
        let den = (a + b * x_cbrt * x_cbrt).sqrt().max(1e-12);
        let z = num / den;

        (1.0 - self.normal_cdf(z)).clamp(0.0, 1.0)
    }

    /// Calculate power for Pearson correlation using exact two-tailed / one-tailed normal power
    fn correlation_power(&self, n: f64, r: f64, alpha: f64, one_tailed: bool) -> f64 {
        if n <= 3.0 || !r.is_finite() || !alpha.is_finite() || alpha <= 0.0 || alpha >= 1.0 {
            return alpha;
        }

        // Fisher z-transformation
        let r_clamped = r.clamp(-0.9999, 0.9999);
        let z_r = ((1.0 + r_clamped) / (1.0 - r_clamped)).ln() / 2.0;
        let z_alpha = if one_tailed {
            self.z_critical(alpha)
        } else {
            self.z_critical(alpha / 2.0)
        };

        // Standard error of Fisher z
        let se = 1.0 / (n - 3.0).sqrt();
        let delta = z_r.abs() / se;

        if one_tailed {
            self.normal_cdf(delta - z_alpha).clamp(0.0, 1.0)
        } else {
            (self.normal_cdf(delta - z_alpha) + self.normal_cdf(-delta - z_alpha)).clamp(0.0, 1.0)
        }
    }

    /// Calculate power for chi-square test using Patnaik (1949) / Abdel-Aty (1954) cube-root approximation
    fn chi_square_power(&self, n: f64, w: f64, df: usize, alpha: f64) -> f64 {
        if n <= 0.0 || !w.is_finite() || !alpha.is_finite() || alpha <= 0.0 || alpha >= 1.0 {
            return alpha.clamp(0.0, 1.0);
        }
        let df_f = df.max(1) as f64;
        let w_abs = w.abs();
        let lambda = n * w_abs * w_abs;

        if lambda < 1e-12 {
            return alpha.clamp(0.0, 1.0);
        }

        let chi2_crit = self.chi2_critical(df_f, alpha);

        // Patnaik (1949) / Abdel-Aty (1954) two-moment non-central chi-square approximation:
        // c = (df + 2*lambda) / (df + lambda), nu* = (df + lambda)^2 / (df + 2*lambda)
        let c = (df_f + 2.0 * lambda) / (df_f + lambda);
        let nu_star = (df_f + lambda).powi(2) / (df_f + 2.0 * lambda);
        let y = (chi2_crit / (c * nu_star)).max(1e-12);

        // Wilson-Hilferty cube-root transformation
        let h = 2.0 / (9.0 * nu_star);
        let z = (y.cbrt() - (1.0 - h)) / h.sqrt();

        (1.0 - self.normal_cdf(z)).clamp(0.0, 1.0)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate power curve
    #[must_use]
    pub fn generate_power_curve(
        &self,
        config: &PowerAnalysisConfig,
        n_min: usize,
        n_max: usize,
        n_step: usize,
    ) -> Vec<PowerCurvePoint> {
        let mut curve = Vec::new();

        let mut n = n_min;
        let step = n_step.max(1);
        while n <= n_max {
            let result = self.calculate_power(config, n);
            curve.push(PowerCurvePoint {
                n,
                power: result.power,
            });
            n += step;
        }

        curve
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Find minimum sample size for target power
    #[must_use]
    pub fn find_minimum_n(
        &self,
        config: &PowerAnalysisConfig,
        target_power: f64,
        max_n: usize,
    ) -> Option<usize> {
        let start_n = if matches!(config.test, StatisticalTest::CorrelationPearson) {
            4
        } else {
            2
        };
        for n in start_n..=max_n {
            let result = self.calculate_power(config, n);
            if result.power >= target_power {
                return Some(n);
            }
        }
        None
    }

    /// Generate recommendations based on power calculation
    fn generate_recommendations(
        &self,
        power: f64,
        target_power: f64,
        current_n: usize,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if power < target_power {
            let deficit = (target_power - power) * 100.0;
            recommendations.push(format!(
                "Power is {deficit:.1}% below target. Consider increasing sample size."
            ));

            if current_n < 30 {
                recommendations.push(
                    "Small sample size may violate normality assumptions. Consider non-parametric alternatives.".to_string()
                );
            }
        } else if power > 0.99 {
            recommendations.push(
                "Power is very high (>99%). You may be able to reduce sample size to save resources.".to_string()
            );
        }

        if power >= 0.80 {
            recommendations.push("Adequate power for detecting expected effect size.".to_string());
        }

        recommendations
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Standard normal CDF approximation
    fn normal_cdf(&self, z: f64) -> f64 {
        if !z.is_finite() {
            return if z.is_sign_negative() { 0.0 } else { 1.0 };
        }
        // Abramowitz and Stegun approximation 7.1.26 via erf(z / sqrt(2))
        let a1 = 0.254829592;
        let a2 = -0.284496736;
        let a3 = 1.421413741;
        let a4 = -1.453152027;
        let a5 = 1.061405429;
        let p = 0.3275911;

        let sign = if z < 0.0 { -1.0 } else { 1.0 };
        let x = z.abs() / std::f64::consts::SQRT_2;

        let t = 1.0 / (1.0 + p * x);
        let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

        0.5 * (1.0 + sign * y)
    }

    /// Critical z-value for given alpha
    fn z_critical(&self, alpha: f64) -> f64 {
        let alpha = alpha.clamp(1e-12, 1.0 - 1e-12);
        // Approximation of inverse normal CDF
        // Using the rational approximation
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

        // TAG: surface=simulation owner=research-team rule=SIM-001
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

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Critical F-value approximation using Paulson (1942) / Wilson-Hilferty normal inversion
    fn f_critical(&self, df1: f64, df2: f64, alpha: f64) -> f64 {
        let df1 = df1.max(1.0);
        let df2 = df2.max(1.0);
        let z = self.z_critical(alpha);

        let a = 2.0 / (9.0 * df1);
        let b = 2.0 / (9.0 * df2);
        let c = 1.0 - a;
        let d = 1.0 - b;

        let radicand = c * c * b + d * d * a - z * z * a * b;
        let denom = d * d - z * z * b;

        if radicand > 0.0 && denom > 0.05 {
            let root = (c * d + z * radicand.sqrt()) / denom;
            root.powi(3).max(0.01)
        } else {
            // Asymptotic fallback for extreme degrees of freedom
            let chi2 = self.chi2_critical(df1, alpha);
            (chi2 / df1).max(0.01)
        }
    }

    /// Critical chi-square value approximation
    fn chi2_critical(&self, df: f64, alpha: f64) -> f64 {
        // Wilson-Hilferty approximation
        let z = self.z_critical(alpha);
        df * (1.0 - 2.0 / (9.0 * df) + z * (2.0 / (9.0 * df)).sqrt()).powi(3)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for PowerCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_calculation() {
        let calc = PowerCalculator::new();
        let config = PowerAnalysisConfig::default();

        let result = calc.calculate_power(&config, 64);

        // With n=64, d=0.5, alpha=0.05 (two-tailed), power should be ~0.80
        assert!(result.power > 0.75 && result.power < 0.85);
    }

    #[test]
    fn test_chi_square_power_null_identity() {
        let calc = PowerCalculator::new();
        let mut config = PowerAnalysisConfig::default();
        config.test = StatisticalTest::ChiSquare;
        config.effect_size.magnitude = 0.0;
        config.num_groups = 2;
        config.alpha = 0.05;

        let result = calc.calculate_power(&config, 50);
        assert!(
            (result.power - 0.05).abs() < 1e-4,
            "Chi-square power under null must equal alpha (0.05), got {}",
            result.power
        );
    }

    #[test]
    fn test_chi_square_power() {
        let calc = PowerCalculator::new();
        let mut config = PowerAnalysisConfig::default();
        config.test = StatisticalTest::ChiSquare;
        config.effect_size.magnitude = 0.3;
        config.num_groups = 4;

        // For w=0.3, df=3, total N = 120 (30 per group), power should be substantial (~0.85)
        let result = calc.calculate_power(&config, 30);
        assert!(result.power > 0.75 && result.power < 0.95);
        assert_eq!(result.total_required_n, 120);
    }

    #[test]
    fn test_small_effect_requires_larger_n() {
        let calc = PowerCalculator::new();
        let mut config = PowerAnalysisConfig::default();
        config.effect_size.magnitude = 0.2; // Small effect

        let small_effect_result = calc.calculate_power(&config, 64);
        config.effect_size.magnitude = 0.8; // Large effect
        let large_effect_result = calc.calculate_power(&config, 64);

        assert!(large_effect_result.power > small_effect_result.power);
    }

    #[test]
    fn test_power_curve_generation() {
        let calc = PowerCalculator::new();
        let config = PowerAnalysisConfig::default();

        let curve = calc.generate_power_curve(&config, 10, 100, 10);

        assert_eq!(curve.len(), 10);
        // Power should increase with sample size
        for i in 1..curve.len() {
            assert!(curve[i].power >= curve[i - 1].power);
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_minimum_n_finding() {
        let calc = PowerCalculator::new();
        let config = PowerAnalysisConfig::default();

        let min_n = calc.find_minimum_n(&config, 0.80, 200);

        assert!(min_n.is_some());
        assert!(min_n.unwrap() >= 2);
    }

    #[test]
    fn test_normal_cdf() {
        let calc = PowerCalculator::new();

        // Test some known values (approximate)
        assert!((calc.normal_cdf(0.0) - 0.5).abs() < 0.001);
        // CDF(1.96) ≈ 0.9750021 per Abramowitz & Stegun
        assert!((calc.normal_cdf(1.96) - 0.975002).abs() < 1e-4);
        assert!((calc.normal_cdf(-1.96) - 0.024998).abs() < 1e-4);
        assert!((calc.normal_cdf(1.96) + calc.normal_cdf(-1.96) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_z_critical() {
        let calc = PowerCalculator::new();

        // Test common critical values (approximate)
        let z_05 = calc.z_critical(0.05);
        // Should be approximately 1.645
        assert!(z_05 > 1.6 && z_05 < 1.7);

        let z_025 = calc.z_critical(0.025);
        // Should be approximately 1.96
        assert!(z_025 > 1.9 && z_025 < 2.0);
    }
}
