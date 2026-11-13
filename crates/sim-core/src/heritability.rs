//! Heritability Analysis Module
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-004: Deterministic - all analysis is deterministic
//!
//! Per P2-7: MZ vs DZ twin correlation analysis using Falconer's formula.
//! h² = `2(r_MZ` - `r_DZ`) where:
//! - `r_MZ` = correlation between monozygotic (identical) twins
//! - `r_DZ` = correlation between dizygotic (fraternal) twins
//!
//! ## Methodology
//! - MZ twins: same genome, different environmental exposure (DOCAS paths)
//! - DZ twins: ~50% shared genetics (via crossover), different DOCAS paths
//! - Heritability measured via tau (τ) correlation after identical stimuli

use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Heritability analysis results for a trait
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HeritabilityEstimate {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Trait name (e.g., "tau", "cooperation", "`risk_tolerance`")
    pub trait_name: String,
    /// Correlation between MZ twins (`r_MZ`)
    pub mz_correlation: f32,
    /// Correlation between DZ twins (`r_DZ`)
    pub dz_correlation: f32,
    /// Heritability estimate h² = `2(r_MZ` - `r_DZ`)
    pub heritability: f32,
    /// Number of MZ twin pairs analyzed
    pub mz_pairs: usize,
    /// Number of DZ twin pairs analyzed
    pub dz_pairs: usize,
    /// Standard error of heritability estimate
    pub se_heritability: f32,
}

impl HeritabilityEstimate {
    /// Create new estimate from twin correlations
    /// Uses Falconer's formula: h² = `2(r_MZ` - `r_DZ`)
    #[must_use]
    pub fn from_correlations(
        trait_name: &str,
        mz_values: &[(f32, f32)], // (twin1, twin2) pairs
        dz_values: &[(f32, f32)],
    ) -> Self {
        let mz_correlation = compute_correlation(mz_values);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let dz_correlation = compute_correlation(dz_values);

        // Falconer's formula
        let heritability = 2.0 * (mz_correlation - dz_correlation);
        let heritability = heritability.clamp(0.0, 1.0); // Bound to valid range

        // Standard error approximation via quadrature: SE(2(r_MZ - r_DZ)) = 2 * sqrt(SE_MZ^2 + SE_DZ^2)
        let n_mz = mz_values.len() as f32;
        let n_dz = dz_values.len() as f32;
        let se_heritability = if n_mz > 2.0 && n_dz > 2.0 {
            let se_mz = (1.0 - mz_correlation.powi(2)).max(0.0) / (n_mz - 2.0).sqrt();
            let se_dz = (1.0 - dz_correlation.powi(2)).max(0.0) / (n_dz - 2.0).sqrt();
            2.0 * (se_mz.powi(2) + se_dz.powi(2)).sqrt()
        } else {
            0.0
        };

        Self {
            trait_name: trait_name.to_string(),
            mz_correlation,
            dz_correlation,
            heritability,
            mz_pairs: mz_values.len(),
            dz_pairs: dz_values.len(),
            se_heritability,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if heritability estimate is statistically meaningful
    #[must_use]
    pub fn is_significant(&self) -> bool {
        self.mz_pairs >= 5
            && self.dz_pairs >= 5
            && self.heritability > 0.0
            && self.se_heritability.is_finite()
            && self.se_heritability > 0.0
            && self.heritability >= 1.96 * self.se_heritability
    }
}

/// Complete heritability analysis results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HeritabilityAnalysis {
    /// Heritability of trust (τ)
    pub tau: HeritabilityEstimate,
    /// Heritability of cooperation tendency
    pub cooperation: HeritabilityEstimate,
    /// Heritability of risk tolerance
    pub risk_tolerance: HeritabilityEstimate,
    /// Heritability of planning depth
    pub planning_depth: HeritabilityEstimate,
    /// Heritability of DOCAS sensitivity (average across D/O/C/A/S)
    pub docas_sensitivity: HeritabilityEstimate,
    /// Tick when analysis was performed
    pub analysis_tick: u64,
}

/// Compute Pearson correlation coefficient for twin pairs
fn compute_correlation(pairs: &[(f32, f32)]) -> f32 {
    let finite_pairs: Vec<(f32, f32)> = pairs
        .iter()
        .copied()
        .filter(|&(x, y)| x.is_finite() && y.is_finite())
        .collect();
    if finite_pairs.len() < 2 {
        return 0.0;
    }

    let n = finite_pairs.len() as f32;
    let mean_x: f32 = finite_pairs.iter().map(|(x, _)| *x).sum::<f32>() / n;
    let mean_y: f32 = finite_pairs.iter().map(|(_, y)| *y).sum::<f32>() / n;

    let mut var_x = 0.0f32;
    let mut var_y = 0.0f32;
    let mut cov_xy = 0.0f32;

    for (x, y) in &finite_pairs {
        let dx = x - mean_x;
        let dy = y - mean_y;
        var_x += dx * dx;
        var_y += dy * dy;
        cov_xy += dx * dy;
    }

    let denominator = (var_x * var_y).sqrt();
    if denominator < 1e-10 {
        return 0.0;
    }

    (cov_xy / denominator).clamp(-1.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_perfect() {
        let pairs = vec![(1.0, 1.0), (2.0, 2.0), (3.0, 3.0)];
        let r = compute_correlation(&pairs);
        assert!(
            (r - 1.0).abs() < 0.001,
            "Perfect correlation should be ~1.0"
        );
    }

    #[test]
    fn test_correlation_negative() {
        let pairs = vec![(1.0, 3.0), (2.0, 2.0), (3.0, 1.0)];
        let r = compute_correlation(&pairs);
        assert!(
            (r - (-1.0)).abs() < 0.001,
            "Perfect negative correlation should be ~-1.0"
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_heritability_from_correlations() {
        // MZ twins: high correlation (0.8)
        let mz_pairs = vec![
            (0.5, 0.52),
            (0.7, 0.72),
            (0.3, 0.28),
            (0.6, 0.62),
            (0.4, 0.38),
        ];
        // DZ twins: moderate correlation (0.4)
        let dz_pairs = vec![(0.5, 0.6), (0.7, 0.5), (0.3, 0.5), (0.6, 0.4), (0.4, 0.6)];

        let estimate = HeritabilityEstimate::from_correlations("test_trait", &mz_pairs, &dz_pairs);

        // MZ should have higher correlation
        assert!(estimate.mz_correlation > estimate.dz_correlation);
        // Heritability should be positive
        assert!(estimate.heritability > 0.0);
        assert!(estimate.heritability <= 1.0);
    }

    #[test]
    fn test_heritability_bounds() {
        // Edge case: MZ < DZ (would give negative h², should clamp to 0)
        let mz_pairs = vec![(0.5, 0.3), (0.7, 0.4)];
        let dz_pairs = vec![(0.5, 0.52), (0.7, 0.72)];

        let estimate = HeritabilityEstimate::from_correlations("edge", &mz_pairs, &dz_pairs);
        assert!(estimate.heritability >= 0.0, "h² should be clamped to >= 0");
        assert!(
            !estimate.is_significant(),
            "Zero heritability must not be significant"
        );
    }

    #[test]
    fn test_correlation_ignores_nan() {
        let pairs = vec![(1.0, 1.0), (f32::NAN, 2.0), (2.0, 2.0), (3.0, 3.0)];
        let r = compute_correlation(&pairs);
        assert!((r - 1.0).abs() < 0.001);
    }
}
