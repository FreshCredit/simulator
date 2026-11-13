// TAG: surface=simulation owner=research-team rule=SIM-001
//! Sampling matrix generation and bootstrap inference for Sobol sensitivity analysis

use super::types::*;
use rand::Rng;
use std::collections::HashMap;

#[inline]
fn sample_distribution<R: Rng>(
    param: &SensitivityParameter,
    lower: f64,
    upper: f64,
    rng: &mut R,
) -> f64 {
    let safe_lower = if lower.is_finite() { lower } else { 0.0 };
    let safe_upper = if upper.is_finite() {
        upper.max(safe_lower)
    } else {
        safe_lower + 1.0
    };
    if (safe_upper - safe_lower).abs() < 1e-12 {
        return safe_lower;
    }

    match param.distribution {
        DistributionType::Uniform => rng.gen_range(safe_lower..=safe_upper),
        DistributionType::Normal => {
            let mean = (safe_lower + safe_upper) / 2.0;
            let std = ((safe_upper - safe_lower) / 6.0).max(0.001);
            let normal = rand_distr::Normal::new(mean, std);
            let v: f64 = match normal {
                Ok(dist) => rng.sample(dist),
                Err(_) => mean,
            };
            v.clamp(safe_lower, safe_upper)
        }
        DistributionType::LogNormal => {
            let pos_lower = safe_lower.max(1e-6);
            let pos_upper = safe_upper.max(pos_lower + 1e-6);
            let log_mean = (pos_lower.ln() + pos_upper.ln()) / 2.0;
            let log_std = ((pos_upper.ln() - pos_lower.ln()) / 6.0).max(0.001);
            match rand_distr::LogNormal::new(log_mean, log_std) {
                Ok(dist) => rng.sample(dist).clamp(pos_lower, pos_upper),
                Err(_) => (pos_lower + pos_upper) / 2.0,
            }
        }
        DistributionType::Triangular => {
            let mode = if param.baseline.is_finite() {
                param.baseline.clamp(safe_lower, safe_upper)
            } else {
                (safe_lower + safe_upper) / 2.0
            };
            match rand_distr::Triangular::new(safe_lower, safe_upper, mode) {
                Ok(dist) => rng.sample(dist),
                Err(_) => (safe_lower + safe_upper) / 2.0,
            }
        }
        DistributionType::Beta => match rand_distr::Beta::new(2.0, 2.0) {
            Ok(dist) => {
                let v: f64 = rng.sample(dist);
                safe_lower + (safe_upper - safe_lower) * v
            }
            Err(_) => (safe_lower + safe_upper) / 2.0,
        },
    }
}

/// Generate sample matrix (N x k)
#[must_use]
pub fn generate_sample_matrix<R: Rng>(
    n: usize,
    parameters: &[SensitivityParameter],
    rng: &mut R,
) -> Vec<HashMap<String, f64>> {
    let mut matrix = Vec::with_capacity(n);

    for _ in 0..n {
        let mut sample = HashMap::new();
        for param in parameters {
            let (lower, upper) = if param.lower <= param.upper {
                (param.lower, param.upper)
            } else {
                (param.upper, param.lower)
            };
            let value = sample_distribution(param, lower, upper, rng);
            sample.insert(param.name.clone(), value);
        }
        matrix.push(sample);
    }

    matrix
}

/// Evaluate model for each sample in matrix
#[must_use]
pub fn evaluate_matrix<F>(matrix: &[HashMap<String, f64>], model: &F) -> Vec<HashMap<String, f64>>
where
    F: Fn(&HashMap<String, f64>) -> HashMap<String, f64>,
{
    matrix.iter().map(model).collect()
}

/// Compute first-order Sobol index (S1) using Saltelli (2002, 2010) & Owen (2013) centered estimator
#[must_use]
pub fn compute_s1(b: &[f64], a: &[f64], c: &[f64], n: usize) -> f64 {
    if n == 0 {
        return 0.0;
    }
    let valid_triples: Vec<(f64, f64, f64)> = a
        .iter()
        .zip(b.iter().zip(c.iter()))
        .filter_map(|(&ak, (&bk, &ck))| {
            if ak.is_finite() && bk.is_finite() && ck.is_finite() {
                Some((ak, bk, ck))
            } else {
                None
            }
        })
        .collect();

    let n_valid = valid_triples.len();
    if n_valid < 2 {
        return 0.0;
    }
    let n_f = n_valid as f64;
    let mean_a: f64 = valid_triples.iter().map(|t| t.0).sum::<f64>() / n_f;
    let mean_b: f64 = valid_triples.iter().map(|t| t.1).sum::<f64>() / n_f;
    let mean_y = (mean_a + mean_b) / 2.0;

    let denom = n_f - 1.0;
    let var_a: f64 = valid_triples
        .iter()
        .map(|t| (t.0 - mean_y).powi(2))
        .sum::<f64>()
        / denom;
    let var_b: f64 = valid_triples
        .iter()
        .map(|t| (t.1 - mean_y).powi(2))
        .sum::<f64>()
        / denom;
    let var_y = (var_a + var_b) / 2.0;
    if var_y <= 1e-12 || !var_y.is_finite() {
        return 0.0;
    }

    let v_cond: f64 = valid_triples
        .iter()
        .map(|t| (t.1 - mean_y) * (t.2 - t.0))
        .sum::<f64>()
        / denom;

    let ratio = v_cond / var_y;
    if ratio.is_finite() {
        ratio.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Compute total-order Sobol index (ST) using Jansen (1999) / Saltelli et al. (2010, eq. f) estimator
#[must_use]
pub fn compute_st(a: &[f64], b: &[f64], c: &[f64], n: usize) -> f64 {
    if n == 0 {
        return 0.0;
    }
    let valid_triples: Vec<(f64, f64, f64)> = a
        .iter()
        .zip(b.iter().zip(c.iter()))
        .filter_map(|(&ak, (&bk, &ck))| {
            if ak.is_finite() && bk.is_finite() && ck.is_finite() {
                Some((ak, bk, ck))
            } else {
                None
            }
        })
        .collect();

    let n_valid = valid_triples.len();
    if n_valid < 2 {
        return 0.0;
    }
    let n_f = n_valid as f64;
    let mean_a: f64 = valid_triples.iter().map(|t| t.0).sum::<f64>() / n_f;
    let mean_b: f64 = valid_triples.iter().map(|t| t.1).sum::<f64>() / n_f;
    let mean_y = (mean_a + mean_b) / 2.0;

    let denom = n_f - 1.0;
    let var_a: f64 = valid_triples
        .iter()
        .map(|t| (t.0 - mean_y).powi(2))
        .sum::<f64>()
        / denom;
    let var_b: f64 = valid_triples
        .iter()
        .map(|t| (t.1 - mean_y).powi(2))
        .sum::<f64>()
        / denom;
    let var_y = (var_a + var_b) / 2.0;
    if var_y <= 1e-12 || !var_y.is_finite() {
        return 0.0;
    }

    let v_tot: f64 = valid_triples
        .iter()
        .map(|t| (t.0 - t.2).powi(2))
        .sum::<f64>()
        / (2.0 * denom);

    let ratio = v_tot / var_y;
    if ratio.is_finite() {
        ratio.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Generate C matrix (A with column i from B)
#[must_use]
pub fn generate_c_matrix(
    matrix_a: &[HashMap<String, f64>],
    matrix_b: &[HashMap<String, f64>],
    param_name: &str,
) -> Vec<HashMap<String, f64>> {
    matrix_a
        .iter()
        .zip(matrix_b.iter())
        .map(|(a, b)| {
            let mut c = a.clone();
            if let Some(&b_val) = b.get(param_name) {
                c.insert(param_name.to_string(), b_val);
            }
            c
        })
        .collect()
}

/// Bootstrap confidence intervals with seeded RNG
#[must_use]
pub fn bootstrap_confidence_interval<R: Rng, F: Fn(&[f64], &[f64], &[f64], usize) -> f64>(
    b: &[f64],
    a: &[f64],
    c: &[f64],
    n: usize,
    num_bootstrap: usize,
    compute_fn: F,
    rng: &mut R,
) -> (f64, f64) {
    let eff_n = n.min(b.len()).min(a.len()).min(c.len());
    if eff_n == 0 || num_bootstrap == 0 {
        let s1 = if eff_n == 0 {
            0.0
        } else {
            compute_fn(b, a, c, eff_n)
        };
        return (s1, s1);
    }

    let mut estimates = Vec::with_capacity(num_bootstrap);

    for _ in 0..num_bootstrap {
        let idx: Vec<usize> = (0..eff_n).map(|_| rng.gen_range(0..eff_n)).collect();
        let b_boot: Vec<f64> = idx.iter().map(|&i| b[i]).collect();
        let a_boot: Vec<f64> = idx.iter().map(|&i| a[i]).collect();
        let c_boot: Vec<f64> = idx.iter().map(|&i| c[i]).collect();

        let s1 = compute_fn(&b_boot, &a_boot, &c_boot, eff_n);
        if s1.is_finite() {
            estimates.push(s1);
        }
    }

    if estimates.is_empty() {
        return (0.0, 0.0);
    }

    estimates.sort_by(f64::total_cmp);

    let n_est = estimates.len();
    let lower_idx = ((n_est - 1) as f64 * 0.025).round() as usize;
    let upper_idx = ((n_est - 1) as f64 * 0.975).round() as usize;

    (
        estimates[lower_idx.min(n_est - 1)],
        estimates[upper_idx.min(n_est - 1)],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_sobol_sampling_empty_bootstrap() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let (lo, hi) =
            bootstrap_confidence_interval(&[], &[], &[], 0, 0, |_, _, _, _| 0.5, &mut rng);
        assert_eq!(lo, 0.0);
        assert_eq!(hi, 0.0);
    }

    #[test]
    fn test_bootstrap_confidence_interval_handles_oversized_n_gracefully() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let b = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let a = vec![1.5, 2.5, 3.5, 4.5, 5.5];
        let c = vec![1.2, 2.2, 3.2, 4.2, 5.2];
        // Nominal n = 100 exceeds slice length 5; must not panic
        let (lo, hi) = bootstrap_confidence_interval(
            &b,
            &a,
            &c,
            100,
            10,
            |_b, _a, _c, n| {
                assert!(n <= 5);
                0.25
            },
            &mut rng,
        );
        assert_eq!(lo, 0.25);
        assert_eq!(hi, 0.25);
    }
}
