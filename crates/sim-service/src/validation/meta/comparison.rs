// TAG: surface=simulation owner=research-team rule=SIM-001
//! Simulation vs empirical meta-analysis concordance comparison

use super::dist::{normal_cdf, z_critical};
use super::types::*;

#[inline]
fn compute_comparison_stats(diff: f64, diff_se: f64) -> ComparisonStats {
    let diff = if diff.is_finite() { diff } else { 0.0 };
    let diff_se = if diff_se.is_finite() && diff_se > 0.0 {
        diff_se
    } else {
        0.0
    };
    let z_score = if diff_se > 1e-12 { diff / diff_se } else { 0.0 };
    let p_value = if diff_se > 1e-12 {
        (2.0 * (1.0 - normal_cdf(z_score.abs()))).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let z_crit = z_critical(0.95);
    ComparisonStats {
        effect_difference: diff,
        difference_se: diff_se,
        z_score,
        p_value,
        confidence_interval: (diff - z_crit * diff_se, diff + z_crit * diff_se),
    }
}

#[inline]
fn check_prediction_interval(emp_effect: f64, sim_meta: &MetaAnalysisResult) -> bool {
    if let Some((pi_lower, pi_upper)) = sim_meta.prediction_interval {
        emp_effect >= pi_lower && emp_effect <= pi_upper
    } else {
        let pi_lower = sim_meta.pooled_effect - 1.96 * sim_meta.pooled_se;
        let pi_upper = sim_meta.pooled_effect + 1.96 * sim_meta.pooled_se;
        emp_effect >= pi_lower && emp_effect <= pi_upper
    }
}

#[inline]
fn evaluate_agreement_level(diff: f64, diff_se: f64) -> AgreementLevel {
    let sd_diff = if diff_se > 0.0 {
        diff.abs() / diff_se
    } else if diff.abs() < 1e-12 {
        0.0
    } else {
        f64::INFINITY
    };
    match sd_diff {
        d if d < 0.2 => AgreementLevel::Excellent,
        d if d < 0.5 => AgreementLevel::Good,
        d if d < 1.0 => AgreementLevel::Moderate,
        _ => AgreementLevel::Poor,
    }
}

#[inline]
fn compute_ci_overlap(emp_ci: (f64, f64), sim_ci: (f64, f64)) -> f64 {
    if !emp_ci.0.is_finite()
        || !emp_ci.1.is_finite()
        || !sim_ci.0.is_finite()
        || !sim_ci.1.is_finite()
    {
        return 0.0;
    }
    let (emp_min, emp_max) = (emp_ci.0.min(emp_ci.1), emp_ci.0.max(emp_ci.1));
    let (sim_min, sim_max) = (sim_ci.0.min(sim_ci.1), sim_ci.0.max(sim_ci.1));
    let overlap_lower = emp_min.max(sim_min);
    let overlap_upper = emp_max.min(sim_max);
    let overlap = if overlap_upper > overlap_lower {
        overlap_upper - overlap_lower
    } else {
        0.0
    };
    let max_range = emp_max.max(sim_max) - emp_min.min(sim_min);
    if max_range > 1e-12 {
        (100.0 * overlap / max_range).clamp(0.0, 100.0)
    } else {
        0.0
    }
}

/// Compare simulation vs empirical meta-analyses
#[must_use]
pub fn compare_simulation_empirical(
    analyzer: &crate::validation::meta::analysis::MetaAnalyzer,
    empirical_studies: &[Study],
    simulation_studies: &[Study],
) -> SimulationEmpiricalComparison {
    let empirical_meta = analyzer.analyze(empirical_studies);
    let simulation_meta = analyzer.analyze(simulation_studies);

    let diff = empirical_meta.pooled_effect - simulation_meta.pooled_effect;
    let diff_se = empirical_meta.pooled_se.hypot(simulation_meta.pooled_se);
    let comparison = compute_comparison_stats(diff, diff_se);
    let in_pi = check_prediction_interval(empirical_meta.pooled_effect, &simulation_meta);
    let agreement = evaluate_agreement_level(diff, diff_se);
    let ci_overlap_percent = compute_ci_overlap(
        empirical_meta.confidence_interval,
        simulation_meta.confidence_interval,
    );

    let concordance = ConcordanceAssessment {
        in_prediction_interval: in_pi,
        agreement,
        ci_overlap_percent,
    };

    SimulationEmpiricalComparison {
        empirical_meta,
        simulation_meta,
        comparison,
        concordance,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_ci_overlap() {
        let pct = compute_ci_overlap((0.0, 1.0), (0.5, 1.5));
        assert!((pct - 33.33).abs() < 0.1);
        assert_eq!(compute_ci_overlap((0.0, 1.0), (2.0, 3.0)), 0.0);
    }
}
