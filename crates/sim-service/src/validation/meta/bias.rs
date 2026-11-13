// TAG: surface=simulation owner=research-team rule=SIM-001
//! Publication bias tests and diagnostics (Egger, Begg, Trim-and-fill)

use super::dist::{normal_cdf, student_t_cdf};
use super::heterogeneity::estimate_tau_squared;
use super::types::*;

/// Test for publication bias
#[must_use]
pub fn test_publication_bias(
    config: &MetaAnalysisConfig,
    studies: &[Study],
) -> Vec<BiasTestResult> {
    let mut results = Vec::new();

    match config.bias_test {
        PublicationBiasTest::Egger => {
            results.push(egger_test(studies));
        }
        PublicationBiasTest::Begg => {
            results.push(begg_test(studies));
        }
        PublicationBiasTest::TrimAndFill => {
            results.push(trim_and_fill(config, studies));
        }
        PublicationBiasTest::None => {}
    }

    results
}

/// Filter studies to only valid entries with finite effect size and positive standard error
fn filter_valid_studies(studies: &[Study]) -> Vec<Study> {
    studies
        .iter()
        .filter(|s| {
            s.effect_size.is_finite() && s.standard_error.is_finite() && s.standard_error > 0.0
        })
        .cloned()
        .collect()
}

/// Egger's regression test
#[must_use]
pub fn egger_test(studies: &[Study]) -> BiasTestResult {
    let valid_studies = filter_valid_studies(studies);
    let n = valid_studies.len() as f64;
    if valid_studies.len() < 3 {
        return BiasTestResult {
            test: "Egger's regression test".to_string(),
            statistic: 0.0,
            p_value: 1.0,
            interpretation: "Insufficient studies for regression test (k < 3)".to_string(),
            adjusted_estimate: None,
        };
    }

    let y: Vec<f64> = valid_studies
        .iter()
        .map(|s| s.effect_size / s.standard_error.max(1e-6))
        .collect();
    let x: Vec<f64> = valid_studies
        .iter()
        .map(|s| 1.0 / s.standard_error.max(1e-6))
        .collect();

    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;

    let ss_xy: f64 = x
        .iter()
        .zip(y.iter())
        .map(|(xi, yi)| (xi - mean_x) * (yi - mean_y))
        .sum();
    let ss_xx: f64 = x.iter().map(|xi| (xi - mean_x).powi(2)).sum();

    if ss_xx <= 1e-12 {
        return BiasTestResult {
            test: "Egger's regression test".to_string(),
            statistic: 0.0,
            p_value: 1.0,
            interpretation: "Insufficient variance in precision for regression test".to_string(),
            adjusted_estimate: None,
        };
    }

    let slope = ss_xy / ss_xx;
    let intercept = mean_y - slope * mean_x;

    let mut ss_resid = 0.0;
    for i in 0..valid_studies.len() {
        let pred = intercept + slope * x[i];
        ss_resid += (y[i] - pred).powi(2);
    }
    let mse = ss_resid / (n - 2.0).max(1.0);
    let se_intercept = (mse * (1.0 / n + mean_x.powi(2) / ss_xx)).sqrt();

    let t_stat = if se_intercept > 0.0 {
        intercept / se_intercept
    } else {
        0.0
    };

    let df = (n - 2.0).max(1.0);
    let p_value = 2.0 * (1.0 - student_t_cdf(t_stat.abs(), df));

    let interpretation = if p_value < 0.05 {
        "Significant asymmetry detected (potential publication bias)".to_string()
    } else {
        "No significant asymmetry detected".to_string()
    };

    BiasTestResult {
        test: "Egger's regression test".to_string(),
        statistic: t_stat,
        p_value,
        interpretation,
        adjusted_estimate: None,
    }
}

/// Begg & Mazumdar (1994) rank correlation test for publication bias
#[must_use]
pub fn begg_test(studies: &[Study]) -> BiasTestResult {
    let valid_studies = filter_valid_studies(studies);
    let n = valid_studies.len();
    if n < 3 {
        return BiasTestResult {
            test: "Begg's rank correlation test".to_string(),
            statistic: 0.0,
            p_value: 1.0,
            interpretation: "Insufficient studies for rank correlation".to_string(),
            adjusted_estimate: None,
        };
    }

    let inv_vars: Vec<f64> = valid_studies
        .iter()
        .map(|s| 1.0 / s.standard_error.max(1e-6).powi(2))
        .collect();
    let sum_inv_v: f64 = inv_vars.iter().sum();
    let pooled_mean = if sum_inv_v > 0.0 {
        valid_studies
            .iter()
            .zip(&inv_vars)
            .map(|(s, &w)| s.effect_size * w)
            .sum::<f64>()
            / sum_inv_v
    } else {
        0.0
    };
    let inv_sum = if sum_inv_v > 0.0 {
        1.0 / sum_inv_v
    } else {
        0.0
    };
    let std_effects: Vec<f64> = valid_studies
        .iter()
        .map(|s| {
            let v_i = s.standard_error.max(1e-6).powi(2);
            let v_star = (v_i - inv_sum).max(1e-8);
            (s.effect_size - pooled_mean) / v_star.sqrt()
        })
        .collect();

    let (concordant, discordant, ties_x, ties_y) =
        compute_kendall_concordance_and_ties(&std_effects, &valid_studies);

    let total_pairs = (n * (n - 1) / 2) as f64;
    let denom = ((total_pairs - ties_x as f64) * (total_pairs - ties_y as f64)).sqrt();
    if denom <= 1e-12 {
        return BiasTestResult {
            test: "Begg's rank correlation test".to_string(),
            statistic: 0.0,
            p_value: 1.0,
            interpretation:
                "Insufficient variance in effect sizes or precision for rank correlation"
                    .to_string(),
            adjusted_estimate: None,
        };
    }

    let tau = (concordant - discordant) as f64 / denom;
    let var_tau = (2.0 * (2.0 * n as f64 + 5.0)) / (9.0 * n as f64 * (n as f64 - 1.0));
    let z_stat = if var_tau > 0.0 {
        tau / var_tau.sqrt()
    } else {
        0.0
    };
    let p_value = 2.0 * (1.0 - normal_cdf(z_stat.abs()));

    let interpretation = if p_value < 0.05 {
        "Significant rank correlation detected (potential publication bias)".to_string()
    } else {
        "No significant rank correlation detected".to_string()
    };

    BiasTestResult {
        test: "Begg's rank correlation test".to_string(),
        statistic: tau,
        p_value,
        interpretation,
        adjusted_estimate: None,
    }
}

fn compute_kendall_concordance_and_ties(
    std_effects: &[f64],
    studies: &[Study],
) -> (i64, i64, i64, i64) {
    let n = std_effects.len();
    let mut concordant = 0i64;
    let mut discordant = 0i64;
    let mut ties_x = 0i64;
    let mut ties_y = 0i64;
    for i in 0..n {
        for j in (i + 1)..n {
            let dy = std_effects[i] - std_effects[j];
            let dx = studies[i].standard_error - studies[j].standard_error;
            let y_tie = dy.abs() < 1e-12;
            let x_tie = dx.abs() < 1e-12;
            if x_tie {
                ties_x += 1;
            }
            if y_tie {
                ties_y += 1;
            }
            if !x_tie && !y_tie {
                if (dy > 0.0 && dx > 0.0) || (dy < 0.0 && dx < 0.0) {
                    concordant += 1;
                } else {
                    discordant += 1;
                }
            }
        }
    }
    (concordant, discordant, ties_x, ties_y)
}

fn count_wilcoxon_signed_rank_sums(dev_pairs: &[(usize, f64, f64)]) -> (f64, f64) {
    let mut rank_sum_pos = 0.0;
    let mut rank_sum_neg = 0.0;
    let mut i = 0;
    while i < dev_pairs.len() {
        let mut j = i + 1;
        while j < dev_pairs.len() && (dev_pairs[j].2 - dev_pairs[i].2).abs() < 1e-12 {
            j += 1;
        }
        let avg_rank = (i + 1 + j) as f64 / 2.0;
        for k_idx in i..j {
            let dev = dev_pairs[k_idx].1;
            if dev > 1e-12 {
                rank_sum_pos += avg_rank;
            } else if dev < -1e-12 {
                rank_sum_neg += avg_rank;
            } else {
                // Zero deviation: split rank equally to conserve total rank sum T_+ + T_- = k(k+1)/2
                rank_sum_pos += avg_rank * 0.5;
                rank_sum_neg += avg_rank * 0.5;
            }
        }
        i = j;
    }
    (rank_sum_pos, rank_sum_neg)
}

fn compute_trimmed_and_imputed(
    studies: &[Study],
    initial_mean: f64,
    trim_right: bool,
    k0: usize,
) -> (f64, Vec<f64>, Vec<f64>) {
    let mut sorted: Vec<&Study> = studies.iter().collect();
    if trim_right {
        sorted.sort_by(|a, b| {
            let diff_a = a.effect_size - initial_mean;
            let diff_b = b.effect_size - initial_mean;
            diff_b
                .is_finite()
                .cmp(&diff_a.is_finite())
                .then_with(|| diff_b.total_cmp(&diff_a))
                .then_with(|| a.id.cmp(&b.id))
        });
    } else {
        sorted.sort_by(|a, b| {
            let diff_a = a.effect_size - initial_mean;
            let diff_b = b.effect_size - initial_mean;
            diff_a
                .is_finite()
                .cmp(&diff_b.is_finite())
                .reverse()
                .then_with(|| diff_a.total_cmp(&diff_b))
                .then_with(|| a.id.cmp(&b.id))
        });
    }

    let trimmed = &sorted[k0..];
    let trimmed_weights: Vec<f64> = trimmed
        .iter()
        .map(|s| 1.0 / s.standard_error.max(1e-6).powi(2))
        .collect();
    let trimmed_sum_w: f64 = trimmed_weights.iter().sum();
    let trimmed_mean = if trimmed_sum_w > 0.0 {
        trimmed
            .iter()
            .zip(trimmed_weights.iter())
            .map(|(s, w)| s.effect_size * w)
            .sum::<f64>()
            / trimmed_sum_w
    } else {
        initial_mean
    };

    let mut all_effects: Vec<f64> = studies.iter().map(|s| s.effect_size).collect();
    let mut all_variances: Vec<f64> = studies
        .iter()
        .map(|s| s.standard_error.max(1e-6).powi(2))
        .collect();

    for study in sorted.iter().take(k0) {
        let imputed_effect = 2.0 * trimmed_mean - study.effect_size;
        let imputed_var = study.standard_error.max(1e-6).powi(2);
        all_effects.push(imputed_effect);
        all_variances.push(imputed_var);
    }

    (trimmed_mean, all_effects, all_variances)
}

/// Duval & Tweedie (2000) non-parametric Trim-and-Fill estimator for publication bias
#[must_use]
pub fn trim_and_fill(config: &MetaAnalysisConfig, studies: &[Study]) -> BiasTestResult {
    let valid_studies = filter_valid_studies(studies);
    let k = valid_studies.len();
    if k < 3 {
        return BiasTestResult {
            test: "Trim-and-fill".to_string(),
            statistic: 0.0,
            p_value: 1.0,
            interpretation: "Insufficient studies for trim-and-fill (k < 3)".to_string(),
            adjusted_estimate: None,
        };
    }

    let fixed_weights: Vec<f64> = valid_studies
        .iter()
        .map(|s| 1.0 / s.standard_error.max(1e-6).powi(2))
        .collect();
    let sum_w: f64 = fixed_weights.iter().sum();
    let initial_mean = if sum_w > 0.0 {
        valid_studies
            .iter()
            .zip(fixed_weights.iter())
            .map(|(s, w)| s.effect_size * w)
            .sum::<f64>()
            / sum_w
    } else {
        0.0
    };

    let mut dev_pairs: Vec<(usize, f64, f64)> = valid_studies
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let dev = s.effect_size - initial_mean;
            (i, dev, dev.abs())
        })
        .collect();

    dev_pairs.sort_by(|a, b| a.2.total_cmp(&b.2));

    let (rank_sum_pos, rank_sum_neg) = count_wilcoxon_signed_rank_sums(&dev_pairs);

    let (trim_right, t_stat) = if rank_sum_pos >= rank_sum_neg {
        (true, rank_sum_pos)
    } else {
        (false, rank_sum_neg)
    };

    let k_f = k as f64;
    let l0_raw = (4.0 * t_stat - k_f * (k_f + 1.0)) / (2.0 * k_f - 1.0).max(1.0);
    let k0 = (l0_raw.round() as isize).clamp(0, (k / 2) as isize) as usize;

    if k0 == 0 {
        return BiasTestResult {
            test: "Trim-and-fill (Duval & Tweedie)".to_string(),
            statistic: 0.0,
            p_value: 1.0,
            interpretation: "Funnel plot is symmetric (0 missing studies estimated)".to_string(),
            adjusted_estimate: Some(initial_mean),
        };
    }

    let (trimmed_mean, all_effects, all_variances) =
        compute_trimmed_and_imputed(&valid_studies, initial_mean, trim_right, k0);

    let tau_adj = match config.model {
        MetaAnalysisModel::FixedEffect => 0.0,
        MetaAnalysisModel::RandomEffects | MetaAnalysisModel::Mixed => {
            let pseudo_studies: Vec<Study> = all_effects
                .iter()
                .zip(all_variances.iter())
                .map(|(&eff, &v)| Study {
                    id: String::new(),
                    name: String::new(),
                    effect_size: eff,
                    standard_error: v.sqrt(),
                    sample_size: 1,
                    confidence_interval: None,
                    year: None,
                    source: StudySource::Simulation,
                    metadata: std::collections::HashMap::new(),
                })
                .collect();
            estimate_tau_squared(config, &pseudo_studies, &all_variances)
        }
    };

    let final_weights: Vec<f64> = all_variances.iter().map(|&v| 1.0 / (v + tau_adj)).collect();
    let total_adjusted_w: f64 = final_weights.iter().sum();
    let adjusted_estimate = if total_adjusted_w > 0.0 {
        all_effects
            .iter()
            .zip(final_weights.iter())
            .map(|(eff, w)| eff * w)
            .sum::<f64>()
            / total_adjusted_w
    } else {
        trimmed_mean
    };

    let se_l0 = ((k_f * (k_f + 1.0) * (2.0 * k_f + 1.0)) / 24.0).sqrt() / (k_f - 0.5).max(1.0);
    let z_l0 = if se_l0 > 0.0 { l0_raw / se_l0 } else { 0.0 };
    let p_value = 2.0 * (1.0 - normal_cdf(z_l0.abs()));

    BiasTestResult {
        test: "Trim-and-fill (Duval & Tweedie)".to_string(),
        statistic: k0 as f64,
        p_value,
        interpretation: format!(
            "Estimated {} missing studies imputed (adjusted estimate: {:.3} vs unadjusted: {:.3})",
            k0, adjusted_estimate, initial_mean
        ),
        adjusted_estimate: Some(adjusted_estimate),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bias_empty_and_insufficient_studies() {
        let empty_studies: Vec<Study> = Vec::new();
        let egger = egger_test(&empty_studies);
        assert_eq!(egger.p_value, 1.0);
        let begg = begg_test(&empty_studies);
        assert_eq!(begg.p_value, 1.0);
        let config = MetaAnalysisConfig::default();
        let tf = trim_and_fill(&config, &empty_studies);
        assert_eq!(tf.p_value, 1.0);
    }

    #[test]
    fn test_bias_tests_with_nan_studies() {
        let nan_study = Study {
            id: "nan".to_string(),
            name: "nan".to_string(),
            effect_size: f64::NAN,
            standard_error: f64::NAN,
            sample_size: 10,
            confidence_interval: None,
            year: None,
            source: StudySource::Simulation,
            metadata: std::collections::HashMap::new(),
        };
        let studies = vec![nan_study; 5];
        let egger = egger_test(&studies);
        assert_eq!(egger.p_value, 1.0);
        let begg = begg_test(&studies);
        assert_eq!(begg.p_value, 1.0);
        let config = MetaAnalysisConfig::default();
        let tf = trim_and_fill(&config, &studies);
        assert_eq!(tf.p_value, 1.0);
    }
}
