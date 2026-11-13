// TAG: surface=simulation owner=research-team rule=SIM-001
//! Heterogeneity estimators and model fit for meta-analysis

use super::dist::chi_square_cdf;
use super::types::*;

/// Variance floor constant to eliminate division-by-zero singularities in meta-analysis
pub const MIN_STUDY_VARIANCE: f64 = 1e-12;

/// Estimate between-study variance (tau²)
#[must_use]
pub fn estimate_tau_squared(
    config: &MetaAnalysisConfig,
    studies: &[Study],
    variances: &[f64],
) -> f64 {
    match config.heterogeneity_method {
        HeterogeneityMethod::DerSimonianLaird => der_simonian_laird(studies, variances),
        HeterogeneityMethod::Hedges => hedges_estimator(studies, variances),
        HeterogeneityMethod::HunterSchmidt => hunter_schmidt(studies, variances),
        HeterogeneityMethod::MaximumLikelihood => maximum_likelihood_estimator(studies, variances),
        HeterogeneityMethod::REML => reml_estimator(studies, variances),
    }
}

#[inline]
fn compute_ml_step(studies: &[Study], variances: &[f64], tau_sq: f64) -> Option<f64> {
    let count = studies.len().min(variances.len());
    if count == 0 {
        return None;
    }
    let weights: Vec<f64> = variances
        .iter()
        .take(count)
        .map(|&v| 1.0 / (v.max(MIN_STUDY_VARIANCE) + tau_sq))
        .collect();
    let sum_w: f64 = weights.iter().sum();
    if sum_w <= 1e-12 {
        return None;
    }
    let mu_hat = studies
        .iter()
        .take(count)
        .zip(weights.iter())
        .map(|(s, &w)| s.effect_size * w)
        .sum::<f64>()
        / sum_w;

    let mut score = 0.0;
    let mut info = 0.0;
    for (s, &w) in studies.iter().take(count).zip(weights.iter()) {
        let diff = s.effect_size - mu_hat;
        let w_sq = w * w;
        score += -0.5 * w + 0.5 * w_sq * diff * diff;
        info += 0.5 * w_sq;
    }

    if info <= 1e-12 {
        None
    } else {
        Some(score / info)
    }
}

/// Maximum Likelihood estimator for tau² (Viechtbauer 2005)
#[must_use]
pub fn maximum_likelihood_estimator(studies: &[Study], variances: &[f64]) -> f64 {
    let k = studies.len();
    if k < 2 {
        return 0.0;
    }

    let mut tau_sq = der_simonian_laird(studies, variances);

    for _ in 0..25 {
        let Some(step) = compute_ml_step(studies, variances, tau_sq) else {
            break;
        };
        let new_tau_sq = (tau_sq + step).max(0.0);
        if (new_tau_sq - tau_sq).abs() < 1e-8 {
            tau_sq = new_tau_sq;
            break;
        }
        tau_sq = new_tau_sq;
    }

    tau_sq
}

#[inline]
fn compute_reml_step(studies: &[Study], variances: &[f64], tau_sq: f64) -> Option<f64> {
    let count = studies.len().min(variances.len());
    if count == 0 {
        return None;
    }
    let weights: Vec<f64> = variances
        .iter()
        .take(count)
        .map(|&v| 1.0 / (v.max(MIN_STUDY_VARIANCE) + tau_sq))
        .collect();
    let sum_w: f64 = weights.iter().sum();
    if sum_w <= 1e-12 {
        return None;
    }
    let mu_hat = studies
        .iter()
        .take(count)
        .zip(weights.iter())
        .map(|(s, &w)| s.effect_size * w)
        .sum::<f64>()
        / sum_w;

    let sum_w2: f64 = weights.iter().map(|&w| w * w).sum();
    let sum_w3: f64 = weights.iter().map(|&w| w * w * w).sum();

    let mut weighted_ss_sq = 0.0;
    for (s, &w) in studies.iter().take(count).zip(weights.iter()) {
        let diff = s.effect_size - mu_hat;
        weighted_ss_sq += w * w * diff * diff;
    }

    let score = -0.5 * (sum_w - sum_w2 / sum_w) + 0.5 * weighted_ss_sq;
    let info = 0.5 * (sum_w2 - 2.0 * sum_w3 / sum_w + (sum_w2 * sum_w2) / (sum_w * sum_w));

    if info <= 1e-12 {
        None
    } else {
        Some(score / info)
    }
}

/// Restricted Maximum Likelihood (REML) estimator for tau² (Viechtbauer 2005)
#[must_use]
pub fn reml_estimator(studies: &[Study], variances: &[f64]) -> f64 {
    let k = studies.len().min(variances.len());
    if k < 2 {
        return 0.0;
    }

    let mut tau_sq = der_simonian_laird(studies, variances);

    for _ in 0..25 {
        let Some(step) = compute_reml_step(studies, variances, tau_sq) else {
            break;
        };
        let new_tau_sq = (tau_sq + step).max(0.0);
        if (new_tau_sq - tau_sq).abs() < 1e-8 {
            tau_sq = new_tau_sq;
            break;
        }
        tau_sq = new_tau_sq;
    }

    tau_sq
}

/// DerSimonian-Laird estimator
#[must_use]
pub fn der_simonian_laird(studies: &[Study], variances: &[f64]) -> f64 {
    let count = studies.len().min(variances.len());
    if count < 2 {
        return 0.0;
    }
    let k = count as f64;

    let fixed_weights: Vec<f64> = variances
        .iter()
        .take(count)
        .map(|v| 1.0 / v.max(MIN_STUDY_VARIANCE))
        .collect();
    let sum_w: f64 = fixed_weights.iter().sum();
    if sum_w <= 1e-12 {
        return 0.0;
    }

    let weighted_sum: f64 = studies
        .iter()
        .take(count)
        .zip(fixed_weights.iter())
        .map(|(s, w)| s.effect_size * w)
        .sum();
    let mean_fe = weighted_sum / sum_w;

    let q: f64 = studies
        .iter()
        .take(count)
        .zip(fixed_weights.iter())
        .map(|(s, w)| w * (s.effect_size - mean_fe).powi(2))
        .sum();

    let sum_w2: f64 = fixed_weights.iter().map(|w| w * w).sum();
    let c = sum_w - sum_w2 / sum_w;

    let num = q - (k - 1.0);
    if num > 0.0 && c > 0.0 {
        num / c
    } else {
        0.0
    }
}

/// Hedges estimator
#[must_use]
pub fn hedges_estimator(studies: &[Study], variances: &[f64]) -> f64 {
    let count = studies.len().min(variances.len());
    if count < 2 {
        return 0.0;
    }
    let k = count as f64;

    let mean_unweighted: f64 = studies
        .iter()
        .take(count)
        .map(|s| s.effect_size)
        .sum::<f64>()
        / k;
    let ssq: f64 = studies
        .iter()
        .take(count)
        .map(|s| (s.effect_size - mean_unweighted).powi(2))
        .sum();
    let expected_var: f64 = variances.iter().take(count).sum::<f64>() / k;

    let denom = (k - 1.0).max(1.0);
    let numerator = ssq - denom * expected_var;
    if numerator > 0.0 {
        numerator / denom
    } else {
        0.0
    }
}

/// Hunter-Schmidt estimator (sample-size weighted)
#[must_use]
pub fn hunter_schmidt(studies: &[Study], variances: &[f64]) -> f64 {
    let count = studies.len().min(variances.len());
    if count == 0 {
        return 0.0;
    }
    let total_n: f64 = studies
        .iter()
        .take(count)
        .map(|s| s.sample_size.max(1) as f64)
        .sum();
    if total_n <= 0.0 {
        return 0.0;
    }

    let mean_es: f64 = studies
        .iter()
        .take(count)
        .map(|s| s.effect_size * (s.sample_size.max(1) as f64))
        .sum::<f64>()
        / total_n;

    let var_observed: f64 = studies
        .iter()
        .take(count)
        .map(|s| (s.sample_size.max(1) as f64) * (s.effect_size - mean_es).powi(2))
        .sum::<f64>()
        / total_n;

    let avg_var: f64 = studies
        .iter()
        .take(count)
        .zip(variances.iter())
        .map(|(s, v)| (s.sample_size.max(1) as f64) * v.max(MIN_STUDY_VARIANCE))
        .sum::<f64>()
        / total_n;

    if var_observed > avg_var {
        var_observed - avg_var
    } else {
        0.0
    }
}

/// Calculate heterogeneity statistics
#[must_use]
pub fn calculate_heterogeneity(
    studies: &[Study],
    _weights: &[f64],
    variances: &[f64],
    tau_squared: f64,
) -> HeterogeneityStats {
    let k = studies.len().min(variances.len());
    let df = k.saturating_sub(1);

    let fixed_weights: Vec<f64> = variances
        .iter()
        .take(k)
        .map(|v| 1.0 / v.max(MIN_STUDY_VARIANCE))
        .collect();
    let sum_w: f64 = fixed_weights.iter().sum();
    let weighted_sum: f64 = studies
        .iter()
        .take(k)
        .zip(fixed_weights.iter())
        .map(|(s, w)| s.effect_size * w)
        .sum();
    let mean_fe = if sum_w > 1e-12 {
        weighted_sum / sum_w
    } else {
        0.0
    };

    let q: f64 = studies
        .iter()
        .take(k)
        .zip(fixed_weights.iter())
        .map(|(s, w)| w * (s.effect_size - mean_fe).powi(2))
        .sum();

    let q_pvalue = if df > 0 && q.is_finite() && q > 0.0 {
        let p = 1.0 - chi_square_cdf(q, df as f64);
        if p.is_finite() {
            p.clamp(0.0, 1.0)
        } else {
            1.0
        }
    } else {
        1.0
    };

    let i_squared = if q.is_finite() && q > 0.0 && k > 1 {
        ((q - df as f64) / q * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    let h_squared = if df > 0 && q.is_finite() {
        (q / df as f64).max(1.0)
    } else {
        1.0
    };
    let tau = tau_squared.max(0.0).sqrt();

    HeterogeneityStats {
        q,
        df,
        q_pvalue,
        i_squared,
        h_squared,
        tau_squared,
        tau,
    }
}

/// Calculate model fit statistics
#[must_use]
pub fn calculate_model_fit(
    studies: &[Study],
    _study_results: &[StudyResult],
    pooled_effect: f64,
    tau_squared: f64,
) -> ModelFitStats {
    let k = studies.len() as f64;
    if k < 2.0 {
        return ModelFitStats {
            log_likelihood: None,
            aic: None,
            bic: None,
            r_squared: None,
        };
    }

    let tau_squared = if tau_squared.is_finite() && tau_squared > 0.0 {
        tau_squared
    } else {
        0.0
    };
    let pooled_effect = if pooled_effect.is_finite() {
        pooled_effect
    } else {
        0.0
    };

    let mut log_lik = 0.0;
    let mean_unweighted = studies.iter().map(|s| s.effect_size).sum::<f64>() / k;

    for s in studies {
        let total_var = s.standard_error.max(1e-6).powi(2) + tau_squared;
        if total_var > 1e-12 {
            let diff = s.effect_size - pooled_effect;
            log_lik +=
                -0.5 * ((2.0 * std::f64::consts::PI * total_var).ln() + diff.powi(2) / total_var);
        }
    }

    let p = if tau_squared > 0.0 { 2.0 } else { 1.0 };
    let aic = -2.0 * log_lik + 2.0 * p;
    let bic = -2.0 * log_lik + p * k.ln();

    let r_squared = if tau_squared > 0.0 {
        let tau_total = studies
            .iter()
            .map(|s| (s.effect_size - mean_unweighted).powi(2))
            .sum::<f64>()
            / (k - 1.0).max(1.0);
        if tau_total > 1e-10 {
            Some(((tau_total - tau_squared) / tau_total).clamp(0.0, 1.0))
        } else {
            Some(0.0)
        }
    } else {
        Some(1.0)
    };

    ModelFitStats {
        log_likelihood: Some(log_lik),
        aic: Some(aic),
        bic: Some(bic),
        r_squared,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heterogeneity_empty_studies() {
        let empty_studies: Vec<Study> = Vec::new();
        let variances: Vec<f64> = Vec::new();
        let config = MetaAnalysisConfig::default();
        let tau_sq = estimate_tau_squared(&config, &empty_studies, &variances);
        assert_eq!(tau_sq, 0.0);
    }

    #[test]
    fn test_h_squared_homogeneous_lower_bound_is_one() {
        let studies = vec![
            Study {
                id: "1".to_string(),
                name: "Study 1".to_string(),
                effect_size: 0.5,
                standard_error: 0.1,
                sample_size: 100,
                confidence_interval: None,
                year: None,
                source: StudySource::Empirical,
                metadata: std::collections::HashMap::new(),
            },
            Study {
                id: "2".to_string(),
                name: "Study 2".to_string(),
                effect_size: 0.5, // Identical effect => Q = 0
                standard_error: 0.1,
                sample_size: 100,
                confidence_interval: None,
                year: None,
                source: StudySource::Empirical,
                metadata: std::collections::HashMap::new(),
            },
        ];
        let variances = vec![0.01, 0.01];
        let weights = vec![100.0, 100.0];
        let stats = calculate_heterogeneity(&studies, &weights, &variances, 0.0);
        assert_eq!(stats.q, 0.0);
        assert_eq!(stats.i_squared, 0.0);
        assert_eq!(stats.h_squared, 1.0); // Must be clamped at 1.0, never 0.0
    }

    #[test]
    fn test_zero_variance_and_mismatched_slices_do_not_produce_nan() {
        let studies = vec![
            Study {
                id: "1".to_string(),
                name: "Study 1".to_string(),
                effect_size: 0.4,
                standard_error: 0.0,
                sample_size: 50,
                confidence_interval: None,
                year: None,
                source: StudySource::Empirical,
                metadata: std::collections::HashMap::new(),
            },
            Study {
                id: "2".to_string(),
                name: "Study 2".to_string(),
                effect_size: 0.6,
                standard_error: 0.1,
                sample_size: 60,
                confidence_interval: None,
                year: None,
                source: StudySource::Empirical,
                metadata: std::collections::HashMap::new(),
            },
        ];
        // One variance is 0.0, second is 0.02, extra variance element to test mismatch
        let variances = vec![0.0, 0.02, 0.05];
        let weights = vec![1.0, 1.0];

        let tau_dl = der_simonian_laird(&studies, &variances);
        assert!(tau_dl.is_finite());
        assert!(!tau_dl.is_nan());

        let tau_ml = maximum_likelihood_estimator(&studies, &variances);
        assert!(tau_ml.is_finite());
        assert!(!tau_ml.is_nan());

        let tau_reml = reml_estimator(&studies, &variances);
        assert!(tau_reml.is_finite());
        assert!(!tau_reml.is_nan());

        let stats = calculate_heterogeneity(&studies, &weights, &variances, tau_dl);
        assert!(stats.q.is_finite());
        assert!(!stats.q.is_nan());
        assert!(stats.h_squared >= 1.0);
    }

    #[test]
    fn test_i_squared_clamped_to_unit_percent_interval() {
        let studies = vec![
            Study {
                id: "1".to_string(),
                name: "Study 1".to_string(),
                effect_size: -10.0,
                standard_error: 0.01,
                sample_size: 100,
                confidence_interval: None,
                year: None,
                source: StudySource::Empirical,
                metadata: std::collections::HashMap::new(),
            },
            Study {
                id: "2".to_string(),
                name: "Study 2".to_string(),
                effect_size: 10.0,
                standard_error: 0.01,
                sample_size: 100,
                confidence_interval: None,
                year: None,
                source: StudySource::Empirical,
                metadata: std::collections::HashMap::new(),
            },
        ];
        let variances = vec![0.0001, 0.0001];
        let weights = vec![10000.0, 10000.0];
        let stats = calculate_heterogeneity(&studies, &weights, &variances, 5.0);
        assert!(stats.i_squared >= 0.0 && stats.i_squared <= 100.0);
        assert!(stats.tau >= 0.0 && stats.tau.is_finite());
    }
}
