//! Effect size calculations and conversions
//!
//! Implements various effect size measures and their conversions.

use super::types::*;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Trait for effect size calculations
pub trait EffectSize {
    /// Calculate effect size from data
    fn calculate(&self, data: &EffectSizeData) -> f64;

    /// Get the type of effect size
    fn effect_type(&self) -> EffectSizeType;

    /// Interpret the magnitude of the effect size
    fn interpret(&self, value: f64) -> EffectSizeInterpretation;
}

/// Data for effect size calculation
#[derive(Debug, Clone, Default)]
pub struct EffectSizeData {
    /// group1 mean
    pub group1_mean: Option<f64>,
    /// group2 mean
    pub group2_mean: Option<f64>,
    /// group1 std
    pub group1_std: Option<f64>,
    /// group2 std
    pub group2_std: Option<f64>,
    /// pooled std
    pub pooled_std: Option<f64>,
    /// n1
    pub n1: Option<usize>,
    /// n2
    pub n2: Option<usize>,
    /// correlation
    pub correlation: Option<f64>,
    /// frequencies
    pub frequencies: Option<Vec<Vec<usize>>>,
    /// ss total
    pub ss_total: Option<f64>,
    /// ss effect
    pub ss_effect: Option<f64>,
    /// ss error
    pub ss_error: Option<f64>,
    /// df effect
    pub df_effect: Option<f64>,
    /// df error
    pub df_error: Option<f64>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Cohen's d effect size
#[derive(Debug)]
pub struct CohenD;

impl EffectSize for CohenD {
    fn calculate(&self, data: &EffectSizeData) -> f64 {
        let mean1 = data.group1_mean.unwrap_or(0.0);
        let mean2 = data.group2_mean.unwrap_or(0.0);

        // Use pooled SD if available, otherwise calculate from group SDs
        let pooled_sd = if let Some(psd) = data.pooled_std {
            if psd > 0.0 && psd.is_finite() {
                psd
            } else {
                0.0
            }
        } else {
            let sd1 = data.group1_std.unwrap_or(1.0);
            let sd2 = data.group2_std.unwrap_or(1.0);
            if !sd1.is_finite() || sd1 <= 0.0 || !sd2.is_finite() || sd2 <= 0.0 {
                0.0
            } else if data.n1.is_none() && data.n2.is_none() {
                // Unweighted pooled standard deviation (Cohen 1988, Eq. 2.5.2)
                ((sd1 * sd1 + sd2 * sd2) / 2.0).sqrt()
            } else {
                let n1 = data.n1.unwrap_or(1) as f64;
                let n2 = data.n2.unwrap_or(1) as f64;
                if n1 <= 1.0 && n2 <= 1.0 {
                    // Small-sample/summary fallback
                    ((sd1 * sd1 + sd2 * sd2) / 2.0).sqrt()
                } else {
                    let denom = (n1 + n2 - 2.0).max(1.0);
                    let num = (n1 - 1.0).max(0.0) * sd1 * sd1 + (n2 - 1.0).max(0.0) * sd2 * sd2;
                    (num / denom).sqrt()
                }
            }
        };

        if pooled_sd <= 0.0 || !pooled_sd.is_finite() {
            0.0
        } else {
            (mean1 - mean2) / pooled_sd
        }
    }

    fn effect_type(&self) -> EffectSizeType {
        EffectSizeType::CohensD
    }

    fn interpret(&self, value: f64) -> EffectSizeInterpretation {
        EffectSizeInterpretation::from_cohens_d(value)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Hedge's g (corrected Cohen's d for small samples)
#[derive(Debug)]
pub struct HedgesG;

impl EffectSize for HedgesG {
    fn calculate(&self, data: &EffectSizeData) -> f64 {
        // Calculate Cohen's d first
        let cohens_d = CohenD.calculate(data);

        // Apply correction factor
        let n1 = data.n1.unwrap_or(1) as f64;
        let n2 = data.n2.unwrap_or(1) as f64;
        let df = n1 + n2 - 2.0;

        if df >= 1.0 {
            let correction = (1.0 - 3.0 / (4.0 * df - 1.0)).max(0.0);
            cohens_d * correction
        } else {
            cohens_d
        }
    }

    fn effect_type(&self) -> EffectSizeType {
        EffectSizeType::HedgesG
    }

    fn interpret(&self, value: f64) -> EffectSizeInterpretation {
        // Same interpretation as Cohen's d
        EffectSizeInterpretation::from_cohens_d(value)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Glass's delta (uses control group SD only)
#[derive(Debug)]
pub struct GlassDelta;

impl EffectSize for GlassDelta {
    fn calculate(&self, data: &EffectSizeData) -> f64 {
        let mean1 = data.group1_mean.unwrap_or(0.0);
        let mean2 = data.group2_mean.unwrap_or(0.0);
        let control_sd = data.group2_std.unwrap_or(1.0);

        if control_sd <= 0.0 || !control_sd.is_finite() {
            0.0
        } else {
            (mean1 - mean2) / control_sd
        }
    }

    fn effect_type(&self) -> EffectSizeType {
        EffectSizeType::GlassDelta
    }

    fn interpret(&self, value: f64) -> EffectSizeInterpretation {
        EffectSizeInterpretation::from_cohens_d(value)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Pearson's r correlation effect size
#[derive(Debug)]
pub struct PearsonsR;

impl EffectSize for PearsonsR {
    fn calculate(&self, data: &EffectSizeData) -> f64 {
        data.correlation.unwrap_or(0.0)
    }

    fn effect_type(&self) -> EffectSizeType {
        EffectSizeType::PearsonsR
    }

    fn interpret(&self, value: f64) -> EffectSizeInterpretation {
        EffectSizeInterpretation::from_pearsons_r(value)
    }
}

/// Eta-squared (proportion of variance explained in ANOVA)
#[derive(Debug)]
pub struct EtaSquared;

impl EffectSize for EtaSquared {
    fn calculate(&self, data: &EffectSizeData) -> f64 {
        let ss_effect = data.ss_effect.unwrap_or(0.0);
        let ss_total = data.ss_total.unwrap_or(1.0);

        if ss_total <= 0.0 || !ss_effect.is_finite() || !ss_total.is_finite() {
            0.0
        } else {
            (ss_effect / ss_total).clamp(0.0, 1.0)
        }
    }

    fn effect_type(&self) -> EffectSizeType {
        EffectSizeType::EtaSquared
    }

    fn interpret(&self, value: f64) -> EffectSizeInterpretation {
        EffectSizeInterpretation::from_eta_squared(value)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Partial eta-squared
#[derive(Debug)]
pub struct PartialEtaSquared;

impl EffectSize for PartialEtaSquared {
    fn calculate(&self, data: &EffectSizeData) -> f64 {
        let ss_effect = data.ss_effect.unwrap_or(0.0);
        let ss_error = data.ss_error.unwrap_or(0.0);
        let denom = ss_effect + ss_error;

        if denom <= 0.0 || !ss_effect.is_finite() || !ss_error.is_finite() {
            0.0
        } else {
            (ss_effect / denom).clamp(0.0, 1.0)
        }
    }

    fn effect_type(&self) -> EffectSizeType {
        EffectSizeType::PartialEtaSquared
    }

    fn interpret(&self, value: f64) -> EffectSizeInterpretation {
        EffectSizeInterpretation::from_eta_squared(value)
    }
}

/// Omega-squared (less biased than eta-squared)
#[derive(Debug)]
pub struct OmegaSquared;

impl EffectSize for OmegaSquared {
    fn calculate(&self, data: &EffectSizeData) -> f64 {
        let ss_effect = data.ss_effect.unwrap_or(0.0);
        let ss_total = data.ss_total.unwrap_or(1.0);
        let df_effect = data.df_effect.unwrap_or(1.0).max(1.0);

        let (ss_error, df_error) = match (data.ss_error, data.df_error) {
            (Some(sse), Some(dfe)) => (sse, dfe.max(1.0)),
            (Some(sse), None) => (sse, 1.0),
            (None, Some(dfe)) => ((ss_total - ss_effect).max(0.0), dfe.max(1.0)),
            (None, None) => {
                let dfe = match (data.n1, data.n2) {
                    (Some(n1), Some(n2)) => (n1 + n2).saturating_sub(2).max(1) as f64,
                    _ => 1.0,
                };
                ((ss_total - ss_effect).max(0.0), dfe)
            }
        };

        let ms_error = if df_error <= 0.0 {
            0.0
        } else {
            ss_error / df_error
        };

        let denom = ss_total + ms_error;
        if denom <= 0.0 {
            0.0
        } else {
            let omega2 = (ss_effect - df_effect * ms_error) / denom;
            omega2.clamp(0.0, 1.0)
        }
    }

    fn effect_type(&self) -> EffectSizeType {
        EffectSizeType::OmegaSquared
    }

    fn interpret(&self, value: f64) -> EffectSizeInterpretation {
        EffectSizeInterpretation::from_eta_squared(value)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Odds ratio effect size
#[derive(Debug)]
pub struct OddsRatio;

impl EffectSize for OddsRatio {
    fn calculate(&self, data: &EffectSizeData) -> f64 {
        let Some(freq) = data.frequencies.as_ref() else {
            return 1.0;
        };
        if freq.len() < 2 || freq[0].len() < 2 || freq[1].len() < 2 {
            return 1.0;
        }

        // 2x2 contingency table
        // [a, b]
        // [c, d]
        let a = freq[0][0] as f64;
        let b = freq[0][1] as f64;
        let c = freq[1][0] as f64;
        let d = freq[1][1] as f64;

        // Add 0.5 correction for zero cells
        (a + 0.5) * (d + 0.5) / ((b + 0.5) * (c + 0.5))
    }

    fn effect_type(&self) -> EffectSizeType {
        EffectSizeType::OddsRatio
    }

    fn interpret(&self, value: f64) -> EffectSizeInterpretation {
        if !value.is_finite() || value <= 0.0 {
            return EffectSizeInterpretation::Negligible;
        }
        // Convert odds ratio to Cohen's d approximation for interpretation
        let ln_or = value.ln();
        let d = ln_or * 3.0_f64.sqrt() / std::f64::consts::PI;
        EffectSizeInterpretation::from_cohens_d(d)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Cohen's h (difference between proportions)
#[derive(Debug)]
pub struct CohenH;

impl CohenH {
    /// Calculate from proportions
    #[must_use]
    pub fn from_proportions(p1: f64, p2: f64) -> f64 {
        let p1_clamped = p1.clamp(0.0, 1.0);
        let p2_clamped = p2.clamp(0.0, 1.0);
        let arcsin_p1 = p1_clamped.sqrt().asin() * 2.0;
        let arcsin_p2 = p2_clamped.sqrt().asin() * 2.0;
        (arcsin_p1 - arcsin_p2).abs()
    }
}

impl EffectSize for CohenH {
    fn calculate(&self, data: &EffectSizeData) -> f64 {
        // Calculate from frequency data
        let Some(freq) = data.frequencies.as_ref() else {
            return 0.0;
        };
        if freq.len() < 2 || freq[0].len() < 2 || freq[1].len() < 2 {
            return 0.0;
        }

        let n1 = (freq[0][0] + freq[0][1]) as f64;
        let n2 = (freq[1][0] + freq[1][1]) as f64;

        if n1 == 0.0 || n2 == 0.0 {
            return 0.0;
        }

        let p1 = freq[0][0] as f64 / n1;
        let p2 = freq[1][0] as f64 / n2;

        Self::from_proportions(p1, p2)
    }

    fn effect_type(&self) -> EffectSizeType {
        EffectSizeType::CohensH
    }

    fn interpret(&self, value: f64) -> EffectSizeInterpretation {
        EffectSizeInterpretation::from_cohens_d(value)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Cohen's f (for ANOVA)
#[derive(Debug)]
pub struct CohenF;

impl CohenF {
    /// Convert from eta-squared
    #[must_use]
    pub fn from_eta_squared(eta2: f64) -> f64 {
        let eta2_clamped = eta2.clamp(0.0, 0.999999);
        (eta2_clamped / (1.0 - eta2_clamped)).sqrt()
    }

    /// Convert to eta-squared
    #[must_use]
    pub fn to_eta_squared(f: f64) -> f64 {
        let f2 = f * f;
        f2 / (1.0 + f2)
    }
}

impl EffectSize for CohenF {
    fn calculate(&self, data: &EffectSizeData) -> f64 {
        let eta2 = EtaSquared.calculate(data);
        Self::from_eta_squared(eta2)
    }

    fn effect_type(&self) -> EffectSizeType {
        EffectSizeType::CohensF
    }

    fn interpret(&self, value: f64) -> EffectSizeInterpretation {
        // Convert to eta-squared for interpretation
        let eta2 = Self::to_eta_squared(value);
        EffectSizeInterpretation::from_eta_squared(eta2)
    }
}

/// Cramer's V (for chi-square tests)
#[derive(Debug)]
pub struct CramersV;

// TAG: surface=simulation owner=research-team rule=SIM-001
impl EffectSize for CramersV {
    fn calculate(&self, data: &EffectSizeData) -> f64 {
        let Some(freq) = data.frequencies.as_ref() else {
            return 0.0;
        };
        if freq.is_empty() || freq[0].is_empty() {
            return 0.0;
        }

        // Calculate chi-square
        let n_rows = freq.len();
        let n_cols = freq[0].len();
        if n_rows < 2 || n_cols < 2 || freq.iter().any(|r| r.len() != n_cols) {
            return 0.0;
        }

        // Calculate row and column totals
        let mut row_totals: Vec<f64> = vec![0.0; n_rows];
        let mut col_totals: Vec<f64> = vec![0.0; n_cols];
        let mut grand_total = 0.0;

        for (i, row) in freq.iter().enumerate() {
            for (j, &cell) in row.iter().enumerate() {
                row_totals[i] += cell as f64;
                col_totals[j] += cell as f64;
                grand_total += cell as f64;
            }
        }

        if grand_total == 0.0 {
            return 0.0;
        }

        // Calculate chi-square
        let mut chi2 = 0.0;
        for (i, row) in freq.iter().enumerate() {
            for (j, &observed) in row.iter().enumerate() {
                let expected = row_totals[i] * col_totals[j] / grand_total;
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                if expected > 0.0 {
                    chi2 += ((observed as f64 - expected).powi(2)) / expected;
                }
            }
        }

        // Cramer's V
        let df = (n_rows.min(n_cols).saturating_sub(1)).max(1) as f64;
        let v2 = chi2 / (grand_total * df);
        if !v2.is_finite() || v2 <= 0.0 {
            0.0
        } else {
            v2.sqrt().clamp(0.0, 1.0)
        }
    }

    fn effect_type(&self) -> EffectSizeType {
        EffectSizeType::CramersV
    }

    fn interpret(&self, value: f64) -> EffectSizeInterpretation {
        if !value.is_finite() {
            return EffectSizeInterpretation::Negligible;
        }
        // Similar to Cohen's d interpretation
        match value {
            v if v < 0.10 => EffectSizeInterpretation::Negligible,
            v if v < 0.30 => EffectSizeInterpretation::Small,
            v if v < 0.50 => EffectSizeInterpretation::Medium,
            v if v < 0.70 => EffectSizeInterpretation::Large,
            _ => EffectSizeInterpretation::VeryLarge,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Effect size converter between different types
#[derive(Debug)]
pub struct EffectSizeConverter;

impl EffectSizeConverter {
    /// Convert Cohen's d to Pearson's r
    #[must_use]
    pub fn d_to_r(d: f64) -> f64 {
        if !d.is_finite() {
            return if d > 0.0 { 1.0 } else { -1.0 };
        }
        d / (d * d + 4.0).sqrt()
    }

    /// Convert Pearson's r to Cohen's d
    #[must_use]
    pub fn r_to_d(r: f64) -> f64 {
        if !r.is_finite() {
            return 0.0;
        }
        let r_clamped = r.clamp(-0.999999, 0.999999);
        2.0 * r_clamped / (1.0 - r_clamped * r_clamped).sqrt()
    }

    /// Convert Cohen's d to eta-squared (approximate)
    #[must_use]
    pub fn d_to_eta2(d: f64) -> f64 {
        if !d.is_finite() {
            return 0.0;
        }
        let d2 = d * d;
        d2 / (d2 + 4.0)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Convert eta-squared to Cohen's d (approximate)
    #[must_use]
    pub fn eta2_to_d(eta2: f64) -> f64 {
        if !eta2.is_finite() {
            return 0.0;
        }
        let eta2_clamped = eta2.clamp(0.0, 0.999999);
        (eta2_clamped / (1.0 - eta2_clamped)).sqrt() * 2.0
    }

    /// Convert odds ratio to Cohen's d
    #[must_use]
    pub fn or_to_d(or: f64) -> f64 {
        if !or.is_finite() || or <= 0.0 {
            return 0.0;
        }
        let or_clamped = or.max(1e-12);
        or_clamped.ln() * 3.0_f64.sqrt() / std::f64::consts::PI
    }

    /// Convert Cohen's d to odds ratio
    #[must_use]
    pub fn d_to_or(d: f64) -> f64 {
        if !d.is_finite() {
            return 1.0;
        }
        (d * std::f64::consts::PI / 3.0_f64.sqrt()).exp()
    }

    /// Convert between any two effect size types
    #[must_use]
    pub fn convert(value: f64, from: EffectSizeType, to: EffectSizeType) -> f64 {
        if from == to {
            return value;
        }

        // Convert to Cohen's d as intermediate
        let d = match from {
            EffectSizeType::CohensD => value,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            EffectSizeType::HedgesG => value, // Close enough approximation
            EffectSizeType::GlassDelta => value,
            EffectSizeType::PearsonsR => Self::r_to_d(value),
            EffectSizeType::EtaSquared => Self::eta2_to_d(value),
            EffectSizeType::PartialEtaSquared => Self::eta2_to_d(value),
            EffectSizeType::OmegaSquared => Self::eta2_to_d(value),
            EffectSizeType::OddsRatio => Self::or_to_d(value),
            EffectSizeType::CohensH => value, // Same scale
            EffectSizeType::CohensF => Self::eta2_to_d(CohenF::to_eta_squared(value)),
            EffectSizeType::CramersV => value * 2.0, // Rough approximation
            EffectSizeType::RSquared => Self::eta2_to_d(value),
        };

        // Convert from Cohen's d to target
        match to {
            EffectSizeType::CohensD => d,
            EffectSizeType::HedgesG => d, // Close enough
            EffectSizeType::GlassDelta => d,
            EffectSizeType::PearsonsR => Self::d_to_r(d),
            EffectSizeType::EtaSquared => Self::d_to_eta2(d),
            EffectSizeType::PartialEtaSquared => Self::d_to_eta2(d),
            EffectSizeType::OmegaSquared => Self::d_to_eta2(d),
            EffectSizeType::OddsRatio => Self::d_to_or(d),
            EffectSizeType::CohensH => d,
            EffectSizeType::CohensF => CohenF::from_eta_squared(Self::d_to_eta2(d)),
            EffectSizeType::CramersV => d / 2.0,
            EffectSizeType::RSquared => Self::d_to_eta2(d),
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cohens_d_calculation() {
        let data = EffectSizeData {
            group1_mean: Some(100.0),
            group2_mean: Some(90.0),
            group1_std: Some(10.0),
            group2_std: Some(10.0),
            n1: Some(50),
            n2: Some(50),
            ..Default::default()
        };

        let d = CohenD.calculate(&data);
        assert!((d - 1.0).abs() < 0.1); // Should be approximately 1.0
    }

    #[test]
    fn test_hedges_g_smaller_than_cohens_d() {
        let data = EffectSizeData {
            group1_mean: Some(100.0),
            group2_mean: Some(90.0),
            group1_std: Some(10.0),
            group2_std: Some(10.0),
            n1: Some(10), // Small sample
            n2: Some(10),
            ..Default::default()
        };

        let cohens_d = CohenD.calculate(&data);
        let hedges_g = HedgesG.calculate(&data);

        assert!(hedges_g < cohens_d); // Hedges g should be slightly smaller
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_d_to_r_conversion() {
        let d = 0.5;
        let r = EffectSizeConverter::d_to_r(d);
        let d_back = EffectSizeConverter::r_to_d(r);

        assert!((d - d_back).abs() < 0.01);
    }

    #[test]
    fn test_d_to_or_roundtrip() {
        let d = 0.5;
        let or = EffectSizeConverter::d_to_or(d);
        let d_back = EffectSizeConverter::or_to_d(or);

        assert!((d - d_back).abs() < 0.01);
    }

    #[test]
    fn test_cohens_f_eta_squared_conversion() {
        let f = 0.25; // Medium effect
        let eta2 = CohenF::to_eta_squared(f);
        let f_back = CohenF::from_eta_squared(eta2);

        assert!((f - f_back).abs() < 0.001);
    }

    #[test]
    fn test_effect_size_interpretation() {
        assert_eq!(
            EffectSizeInterpretation::from_cohens_d(0.1),
            EffectSizeInterpretation::Negligible // < 0.2 is negligible
        );
        assert_eq!(
            EffectSizeInterpretation::from_cohens_d(0.3), // 0.2-0.5 is small
            // TAG: surface=simulation owner=research-team rule=SIM-001
            EffectSizeInterpretation::Small
        );
        assert_eq!(
            EffectSizeInterpretation::from_cohens_d(0.5), // 0.5 is the boundary
            EffectSizeInterpretation::Medium
        );
        assert_eq!(
            EffectSizeInterpretation::from_cohens_d(0.9), // 0.8-1.2 is large
            EffectSizeInterpretation::Large
        );
    }

    #[test]
    fn test_cramers_v_calculation() {
        let data = EffectSizeData {
            frequencies: Some(vec![vec![40, 10], vec![20, 30]]),
            ..Default::default()
        };

        let v = CramersV.calculate(&data);
        assert!(v > 0.0 && v <= 1.0);
    }

    #[test]
    fn test_omega_squared_calculation() {
        let data = EffectSizeData {
            ss_effect: Some(40.0),
            ss_error: Some(60.0),
            ss_total: Some(100.0),
            df_effect: Some(2.0),
            df_error: Some(20.0),
            ..Default::default()
        };
        // ms_error = 60 / 20 = 3.0
        // omega2 = (40 - 2 * 3.0) / (100 + 3.0) = 34 / 103 ≈ 0.330097
        let omega2 = OmegaSquared.calculate(&data);
        assert!(
            (omega2 - (34.0 / 103.0)).abs() < 1e-4,
            "Expected ~0.3301, got {omega2}"
        );
    }

    #[test]
    fn test_cohens_f_d_roundtrip() {
        let f = 0.25; // Medium effect
        let d = EffectSizeConverter::convert(f, EffectSizeType::CohensF, EffectSizeType::CohensD);
        assert!((d - 0.50).abs() < 1e-4, "Expected d=2f=0.50, got {d}");
        let f_back =
            EffectSizeConverter::convert(d, EffectSizeType::CohensD, EffectSizeType::CohensF);
        assert!(
            (f_back - f).abs() < 1e-4,
            "Expected round-trip f=0.25, got {f_back}"
        );
    }

    #[test]
    fn test_cohen_h_from_proportions() {
        let h = CohenH::from_proportions(0.6, 0.4);
        assert!(h > 0.0);

        // Equal proportions should give h ≈ 0
        let h_equal = CohenH::from_proportions(0.5, 0.5);
        assert!(h_equal.abs() < 0.01);
    }

    #[test]
    fn test_cohens_d_without_sample_sizes_computes_unweighted_pooled_sd() {
        let data = EffectSizeData {
            group1_mean: Some(100.0),
            group2_mean: Some(90.0),
            group1_std: Some(10.0),
            group2_std: Some(10.0),
            ..Default::default()
        };
        let d = CohenD.calculate(&data);
        assert!(
            (d - 1.0).abs() < 1e-4,
            "Expected d=1.0 when sample sizes omitted, got {d}"
        );

        let g = HedgesG.calculate(&data);
        assert!(
            (g - 1.0).abs() < 1e-4,
            "Expected g=1.0 when sample sizes omitted, got {g}"
        );
    }

    #[test]
    fn test_glass_delta_non_positive_control_sd_safely_zeros() {
        let data_neg = EffectSizeData {
            group1_mean: Some(100.0),
            group2_mean: Some(90.0),
            group2_std: Some(-5.0),
            ..Default::default()
        };
        assert_eq!(GlassDelta.calculate(&data_neg), 0.0);

        let data_zero = EffectSizeData {
            group1_mean: Some(100.0),
            group2_mean: Some(90.0),
            group2_std: Some(0.0),
            ..Default::default()
        };
        assert_eq!(GlassDelta.calculate(&data_zero), 0.0);
    }
}
