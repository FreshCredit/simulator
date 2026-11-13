//! Layer 2 Statistics for sim-runner (§5.3.3)
//!
//! Aggregation statistics computed per-run for POM validation and confidence intervals.
//!
//! # Functions
//!
//! | Function | Purpose | Phase |
//! |----------|---------|-------|
//! | `normal_cdf` | 95% CI construction | Phase 1 |
//! | `erf` | Required for `normal_cdf` | Phase 1 |
//! | `gini_coefficient` | Wealth inequality | Phase 1 |
//! | `cohens_d_with_ci` | POM Validator effect sizes | Phase 1 |
//! | `shannon_entropy_mm` | Miller-Madow corrected entropy | Phase 2 |
//! | `pearson_r` | Cross-domain coherence | Phase 2 |
//! | `fisher_z_ci` | Correlation CIs | Phase 2 |
//! | `welch_t_test` | Unequal variance comparison | Phase 2 |
//! | `mann_whitney_u` | Rank test with tie correction | Phase 2 |
//!
//! # SIM-001 Compliance
//!
//! All functions use only primitive types and standard library.
//! No external dependencies beyond std.

use std::f64::consts::{PI, SQRT_2};

/// Abramowitz-Stegun coefficients for erf approximation (7.1.26)
/// Reference: Abramowitz & Stegun, Handbook of Mathematical Functions, 1972
const ERF_A1: f64 = 0.254829592;
const ERF_A2: f64 = -0.284496736;
const ERF_A3: f64 = 1.421413741;
const ERF_A4: f64 = -1.453152027;
const ERF_A5: f64 = 1.061405429;
const ERF_P: f64 = 0.3275911;

/// Error function using Abramowitz-Stegun 7.1.26 approximation
///
/// erf(x) ≈ 1 - (a₁t + a₂t² + a₃t³ + a₄t⁴ + a₅t⁵) × e^(-x²)
/// where t = 1/(1 + px)
///
/// Maximum error: |ε| < 1.5 × 10⁻⁷
///
/// # Example
///
/// ```
/// use sim_runner::erf;
/// debug_assert!((erf(0.0)).abs() < 1e-6);  // Max error is 1.5e-7
/// debug_assert!((erf(1.0) - 0.8427).abs() < 0.001);
/// ```
#[must_use]
pub fn erf(x: f64) -> f64 {
    // Handle sign
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let sign = if x >= 0.0 { 1.0 } else { -1.0 };
    let x = x.abs();

    // Horner's method for polynomial evaluation
    let t = 1.0 / (1.0 + ERF_P * x);
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    let t5 = t4 * t;

    let poly = ERF_A1 * t + ERF_A2 * t2 + ERF_A3 * t3 + ERF_A4 * t4 + ERF_A5 * t5;
    let result = 1.0 - poly * (-x * x).exp();

    sign * result
}

/// Normal CDF (cumulative distribution function)
///
/// Φ(x) = 0.5 × (1 + erf(x/√2))
///
/// Used for 95% confidence interval construction.
///
/// # Example
///
/// ```
/// use sim_runner::normal_cdf;
/// debug_assert!((normal_cdf(0.0) - 0.5).abs() < 1e-6);  // Based on erf approximation
/// debug_assert!((normal_cdf(1.96) - 0.975).abs() < 0.001);
/// ```
#[must_use]
pub fn normal_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x / SQRT_2))
}

/// Inverse normal CDF (quantile function) using Newton-Raphson
///
/// Used to find z-scores for confidence intervals.
/// For example, `normal_cdf_inv(0.975)` ≈ 1.96
#[must_use]
pub fn normal_cdf_inv(p: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    if (p - 0.5).abs() < 1e-10 {
        return 0.0;
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    // Initial guess using Hastings approximation
    let sign = if p < 0.5 { -1.0 } else { 1.0 };
    let p_adj = if p < 0.5 { p } else { 1.0 - p };

    let t = (-2.0 * p_adj.ln()).sqrt();
    let c0 = 2.515517;
    let c1 = 0.802853;
    let c2 = 0.010328;
    let d1 = 1.432788;
    let d2 = 0.189269;
    let d3 = 0.001308;

    let mut x =
        sign * (t - (c0 + c1 * t + c2 * t * t) / (1.0 + d1 * t + d2 * t * t + d3 * t * t * t));

    // Newton-Raphson refinement (2 iterations usually sufficient)
    for _ in 0..3 {
        let cdf = normal_cdf(x);
        let pdf = (-(x * x) / 2.0).exp() / (2.0 * PI).sqrt();
        if pdf.abs() < 1e-15 {
            break;
        }
        x -= (cdf - p) / pdf;
    }

    x
}

/// Gini coefficient for wealth/resource inequality
///
/// G = Σᵢ Σⱼ |xᵢ - xⱼ| / (2n²μ)
///
/// Returns value in [0, 1] where 0 = perfect equality, 1 = maximum inequality.
///
/// # Example
///
/// ```
/// use sim_runner::gini_coefficient;
/// // Perfect equality
/// debug_assert!((gini_coefficient(&[10.0, 10.0, 10.0, 10.0])).abs() < 1e-10);
// TAG: surface=simulation owner=research-team rule=SIM-001
/// // Some inequality
/// let g = gini_coefficient(&[1.0, 2.0, 3.0, 4.0]);
/// debug_assert!(g > 0.0 && g < 1.0);
/// ```
#[must_use]
pub fn gini_coefficient(values: &[f64]) -> f64 {
    let mut clean: Vec<f64> = values
        .iter()
        .copied()
        .filter(|&v| v.is_finite())
        .map(|v| v.max(0.0))
        .collect();

    let n = clean.len();
    if n <= 1 {
        return 0.0;
    }

    let sum: f64 = clean.iter().sum();
    if sum <= 0.0 {
        return 0.0;
    }

    clean.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // Exact O(n log n) Lorenz rank-sum identity: G = Σ (2i - n + 1) * x_{(i)} / (n * Σ x_{(i)})
    let n_f = n as f64;
    let mut rank_sum = 0.0f64;
    for (i, &x) in clean.iter().enumerate() {
        let weight = 2.0 * i as f64 - n_f + 1.0;
        rank_sum += weight * x;
    }

    (rank_sum / (n_f * sum)).clamp(0.0, 1.0)
}

/// Cohen's d effect size with 95% confidence interval
///
/// d = (μ₁ - μ₂) / `s_pooled`
/// `SE_d` = √((n₁+n₂)/(n₁n₂) + d²/(2(n₁+n₂)))
///
/// Returns (d, `ci_lower`, `ci_upper`)
///
/// # Example
///
/// ```
/// use sim_runner::cohens_d_with_ci;
// TAG: surface=simulation owner=research-team rule=SIM-001
/// let g1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// let g2 = vec![2.0, 3.0, 4.0, 5.0, 6.0];
/// let (d, ci_low, ci_high) = cohens_d_with_ci(&g1, &g2);
/// debug_assert!(d < 0.0); // g2 has higher mean
/// ```
#[must_use]
pub fn cohens_d_with_ci(group1: &[f64], group2: &[f64]) -> (f64, f64, f64) {
    let clean1: Vec<f64> = group1.iter().copied().filter(|v| v.is_finite()).collect();
    let clean2: Vec<f64> = group2.iter().copied().filter(|v| v.is_finite()).collect();
    let n1 = clean1.len() as f64;
    let n2 = clean2.len() as f64;

    if n1 < 2.0 || n2 < 2.0 {
        return (0.0, 0.0, 0.0);
    }

    // Compute means
    let mean1: f64 = clean1.iter().sum::<f64>() / n1;
    let mean2: f64 = clean2.iter().sum::<f64>() / n2;

    // Compute variances
    let var1: f64 = clean1.iter().map(|x| (x - mean1).powi(2)).sum::<f64>() / (n1 - 1.0);
    let var2: f64 = clean2.iter().map(|x| (x - mean2).powi(2)).sum::<f64>() / (n2 - 1.0);

    // Pooled standard deviation
    let s_pooled = ((((n1 - 1.0) * var1) + ((n2 - 1.0) * var2)) / (n1 + n2 - 2.0)).sqrt();

    if s_pooled < 1e-15 {
        return (0.0, 0.0, 0.0);
    }

    // Cohen's d
    let d = (mean1 - mean2) / s_pooled;

    // Standard error of d
    let se_d = ((n1 + n2) / (n1 * n2) + (d * d) / (2.0 * (n1 + n2))).sqrt();

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // 95% CI (z = 1.96)
    let z = 1.96;
    let ci_lower = d - z * se_d;
    let ci_upper = d + z * se_d;

    (d, ci_lower, ci_upper)
}

/// Shannon entropy with Miller-Madow bias correction (§5207)
///
/// H = -Σ pᵢ log₂(pᵢ) + (k-1)/(2N ln2)
///
/// where k = number of non-zero bins, N = total count
///
/// # Example
///
/// ```
/// use sim_runner::shannon_entropy_mm;
/// // Maximum entropy for 4 equal bins
/// let h = shannon_entropy_mm(&[25, 25, 25, 25]);
/// debug_assert!((h - 2.0).abs() < 0.1); // log2(4) = 2
/// ```
#[must_use]
pub fn shannon_entropy_mm(counts: &[u64]) -> f64 {
    let total: u64 = counts.iter().sum();
    if total == 0 {
        return 0.0;
    }

    let n = total as f64;
    let k = counts.iter().filter(|&&c| c > 0).count() as f64;

    // Raw entropy
    let mut h = 0.0;
    for &count in counts {
        if count > 0 {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            let p = count as f64 / n;
            h -= p * p.log2();
        }
    }

    // Miller-Madow correction
    let correction = (k - 1.0) / (2.0 * n * 2.0_f64.ln());
    h + correction
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001

/// Pearson correlation coefficient
///
/// r = Σ(xᵢ - x̄)(yᵢ - ȳ) / √(Σ(xᵢ - x̄)² Σ(yᵢ - ȳ)²)
///
/// # Example
///
/// ```
/// use sim_runner::pearson_r;
/// // Perfect positive correlation
/// let r = pearson_r(&[1.0, 2.0, 3.0], &[2.0, 4.0, 6.0]);
/// debug_assert!((r - 1.0).abs() < 1e-10);
/// ```
#[must_use]
pub fn pearson_r(x: &[f64], y: &[f64]) -> f64 {
    let clean: Vec<(f64, f64)> = x
        .iter()
        .zip(y.iter())
        .filter(|(&a, &b)| a.is_finite() && b.is_finite())
        .map(|(&a, &b)| (a, b))
        .collect();

    let n = clean.len();
    if n < 2 {
        return 0.0;
    }

    let n_f = n as f64;
    let mean_x: f64 = clean.iter().map(|p| p.0).sum::<f64>() / n_f;
    let mean_y: f64 = clean.iter().map(|p| p.1).sum::<f64>() / n_f;

    let mut cov = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;

    for (xi, yi) in clean {
        let dx = xi - mean_x;
        let dy = yi - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    let denom = (var_x * var_y).sqrt();
    if denom < 1e-15 {
        return 0.0;
    }
    // TAG: surface=simulation owner=research-team rule=SIM-001

    cov / denom
}

/// Fisher Z transformation with confidence interval
///
/// z = 0.5 ln((1+r)/(1-r))
/// `SE_z` = 1/√(n-3)
///
/// Returns (r, `ci_lower`, `ci_upper`) in correlation scale
///
/// # Example
///
/// ```
/// use sim_runner::fisher_z_ci;
/// let (r, ci_low, ci_high) = fisher_z_ci(0.5, 100);
/// debug_assert!(ci_low < 0.5 && ci_high > 0.5);
/// ```
#[must_use]
pub fn fisher_z_ci(r: f64, n: usize) -> (f64, f64, f64) {
    if !r.is_finite() || n <= 3 {
        return (r, r, r);
    }

    // Clamp r to avoid infinity
    let r_clamped = r.clamp(-0.9999, 0.9999);

    // Fisher Z transformation
    let z = 0.5 * ((1.0 + r_clamped) / (1.0 - r_clamped)).ln();

    // Standard error in z-space
    let se_z = 1.0 / ((n - 3) as f64).sqrt();

    // 95% CI in z-space
    let z_crit = 1.96;
    let z_lower = z - z_crit * se_z;
    let z_upper = z + z_crit * se_z;

    // Transform back to r-space
    let r_lower = z_lower.tanh().clamp(-1.0, 1.0);
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    let r_upper = z_upper.tanh().clamp(-1.0, 1.0);

    (r, r_lower, r_upper)
}

/// Compute mean of a slice
#[must_use]
pub fn mean(values: &[f64]) -> f64 {
    let clean: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if clean.is_empty() {
        return 0.0;
    }
    clean.iter().sum::<f64>() / clean.len() as f64
}

/// Compute standard deviation of a slice (sample std, N-1 denominator)
#[must_use]
pub fn std_dev(values: &[f64]) -> f64 {
    let clean: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if clean.len() < 2 {
        return 0.0;
    }
    let m = mean(&clean);
    let variance: f64 =
        clean.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (clean.len() - 1) as f64;
    variance.sqrt()
}

/// Compute 95% confidence interval for mean
///
/// Returns (mean, `ci_lower`, `ci_upper`)
#[must_use]
pub fn mean_with_ci(values: &[f64]) -> (f64, f64, f64) {
    let clean: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if clean.len() < 2 {
        let m = mean(&clean);
        return (m, m, m);
    }
    let m = mean(&clean);
    let s = std_dev(&clean);
    let se = s / (clean.len() as f64).sqrt();
    let z = 1.96;
    (m, m - z * se, m + z * se)
    // TAG: surface=simulation owner=research-team rule=SIM-001
}

/// Efron's Bias-Corrected and Accelerated (BCa) bootstrap confidence interval (Sim 7 rigor battery)
/// Computes non-parametric (ci_lower, ci_upper) confidence interval for a statistic.
#[must_use]
pub fn bca_bootstrap_ci<F>(
    data: &[f64],
    stat_fn: F,
    n_resamples: usize,
    alpha: f64,
    seed: u64,
) -> (f64, f64)
where
    F: Fn(&[f64]) -> f64,
{
    let clean: Vec<f64> = data.iter().copied().filter(|v| v.is_finite()).collect();
    let n = clean.len();
    if n < 3 || n_resamples == 0 {
        let theta = stat_fn(&clean);
        return (theta, theta);
    }

    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    let mut rng = StdRng::seed_from_u64(seed);

    let theta_hat = stat_fn(&clean);

    let mut boots = Vec::with_capacity(n_resamples);
    let mut sample = vec![0.0f64; n];
    for _ in 0..n_resamples {
        for item in &mut sample {
            let idx = rng.gen_range(0..n);
            *item = clean[idx];
        }
        boots.push(stat_fn(&sample));
    }
    boots.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let less_count = boots.iter().filter(|&&t| t < theta_hat).count() as f64;
    let prop_less = (less_count / n_resamples as f64).clamp(1e-5, 1.0 - 1e-5);
    let z0 = normal_cdf_inv(prop_less);

    let mut jack_thetas = Vec::with_capacity(n);
    let mut jack_buf = Vec::with_capacity(n - 1);
    for i in 0..n {
        jack_buf.clear();
        for (j, &val) in clean.iter().enumerate() {
            if i != j {
                jack_buf.push(val);
            }
        }
        jack_thetas.push(stat_fn(&jack_buf));
    }
    let jack_mean = jack_thetas.iter().sum::<f64>() / n as f64;
    let mut num = 0.0f64;
    let mut den = 0.0f64;
    for &t_i in &jack_thetas {
        let diff = jack_mean - t_i;
        num += diff.powi(3);
        den += diff.powi(2);
    }
    let a = if den > 1e-12 {
        num / (6.0 * den.max(0.0).powf(1.5))
    } else {
        0.0
    };

    let z_alpha2 = normal_cdf_inv(alpha / 2.0);
    let z_1_alpha2 = normal_cdf_inv(1.0 - alpha / 2.0);

    let calc_pct = |z_val: f64| -> f64 {
        let num_z = z0 + z_val;
        let denom = 1.0 - a * num_z;
        if denom <= 0.0 {
            1.0
        } else if denom.abs() < 1e-5 {
            if num_z > 0.0 {
                1.0
            } else {
                0.0
            }
        } else {
            normal_cdf(z0 + num_z / denom)
        }
    };

    let p1 = calc_pct(z_alpha2).clamp(0.0, 1.0);
    let p2 = calc_pct(z_1_alpha2).clamp(0.0, 1.0);

    let idx1 = ((n_resamples as f64 - 1.0) * p1).round() as usize;
    let idx2 = ((n_resamples as f64 - 1.0) * p2).round() as usize;

    (
        boots[idx1.min(n_resamples - 1)],
        boots[idx2.min(n_resamples - 1)],
    )
}

/// Exact two-sample permutation test (Criterion C4)
/// Returns (observed_difference, empirical_p_value)
#[must_use]
pub fn permutation_test(group1: &[f64], group2: &[f64], n_perms: usize, seed: u64) -> (f64, f64) {
    let clean1: Vec<f64> = group1.iter().copied().filter(|v| v.is_finite()).collect();
    let clean2: Vec<f64> = group2.iter().copied().filter(|v| v.is_finite()).collect();

    if clean1.is_empty() || clean2.is_empty() || n_perms == 0 {
        return (0.0, 1.0);
    }

    let m1 = mean(&clean1);
    let m2 = mean(&clean2);
    let obs_diff = m1 - m2;

    use rand::rngs::StdRng;
    use rand::seq::SliceRandom;
    use rand::SeedableRng;
    let mut rng = StdRng::seed_from_u64(seed);

    let n1 = clean1.len();
    let mut pooled = Vec::with_capacity(n1 + clean2.len());
    pooled.extend_from_slice(&clean1);
    pooled.extend_from_slice(&clean2);

    let mut extreme_count = 0usize;
    for _ in 0..n_perms {
        pooled.shuffle(&mut rng);
        let null_m1 = mean(&pooled[..n1]);
        let null_m2 = mean(&pooled[n1..]);
        let null_diff = null_m1 - null_m2;
        if null_diff.abs() >= obs_diff.abs() {
            extreme_count += 1;
        }
    }

    let p_val = (extreme_count as f64 + 1.0) / (n_perms as f64 + 1.0);
    (obs_diff, p_val)
}

/// Two-sample Welch's t-test for unequal variances (§5.3.3)
///
/// Computes (t_statistic, degrees_of_freedom, two_tailed_p_value)
/// using the Welch-Satterthwaite approximation and Wallace (1959) t-distribution CDF.
#[must_use]
pub fn welch_t_test(group1: &[f64], group2: &[f64]) -> (f64, f64, f64) {
    let clean1: Vec<f64> = group1.iter().copied().filter(|v| v.is_finite()).collect();
    let clean2: Vec<f64> = group2.iter().copied().filter(|v| v.is_finite()).collect();
    let n1 = clean1.len() as f64;
    let n2 = clean2.len() as f64;

    if n1 < 2.0 || n2 < 2.0 {
        return (0.0, 1.0, 1.0);
    }

    let m1 = mean(&clean1);
    let m2 = mean(&clean2);

    let var1 = clean1.iter().map(|x| (x - m1).powi(2)).sum::<f64>() / (n1 - 1.0);
    let var2 = clean2.iter().map(|x| (x - m2).powi(2)).sum::<f64>() / (n2 - 1.0);

    let v1 = var1 / n1;
    let v2 = var2 / n2;
    let se = (v1 + v2).sqrt();

    if se < 1e-15 {
        let diff = (m1 - m2).abs();
        if diff <= 1e-12 * m1.abs().max(m2.abs()).max(1.0) {
            return (0.0, (n1 + n2 - 2.0).max(1.0), 1.0);
        }
        let t = if m1 > m2 {
            f64::INFINITY
        } else {
            f64::NEG_INFINITY
        };
        return (t, (n1 + n2 - 2.0).max(1.0), 0.0);
    }

    let t = (m1 - m2) / se;

    // Welch-Satterthwaite degrees of freedom
    let num = (v1 + v2).powi(2);
    let den = (v1.powi(2) / (n1 - 1.0)) + (v2.powi(2) / (n2 - 1.0));
    let df = if den > 1e-15 {
        (num / den).max(1.0)
    } else {
        (n1 + n2 - 2.0).max(1.0)
    };

    let t_abs = t.abs();
    if t_abs < 1e-15 {
        return (0.0, df, 1.0);
    }

    // Two-tailed p-value with Wallace (1959) t-distribution normal approximation
    let z_equiv = (df * (1.0 + (t_abs * t_abs) / df).ln()).sqrt() * (1.0 - 0.25 / df);
    let p = 2.0 * (1.0 - normal_cdf(z_equiv));

    (t, df, p.clamp(0.0, 1.0))
}

/// Mann-Whitney U test with mid-ranks for ties and Lehmann variance correction
///
/// Computes (u_statistic, two_tailed_p_value). Satisfies U1 + U2 = n1 * n2.
#[must_use]
pub fn mann_whitney_u(group1: &[f64], group2: &[f64]) -> (f64, f64) {
    let clean1: Vec<f64> = group1.iter().copied().filter(|v| v.is_finite()).collect();
    let clean2: Vec<f64> = group2.iter().copied().filter(|v| v.is_finite()).collect();
    let n1 = clean1.len() as f64;
    let n2 = clean2.len() as f64;

    if n1 < 1.0 || n2 < 1.0 {
        return (0.0, 1.0);
    }

    let mut combined: Vec<(f64, usize)> = clean1
        .iter()
        .map(|&x| (x, 0))
        .chain(clean2.iter().map(|&x| (x, 1)))
        .collect();

    combined.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let n_total = combined.len();
    let mut sum_ranks1 = 0.0f64;
    let mut sum_ranks2 = 0.0f64;
    let mut tie_sum = 0.0f64;

    let mut i = 0;
    while i < n_total {
        let mut j = i + 1;
        while j < n_total && (combined[j].0 - combined[i].0).abs() <= 1e-12 {
            j += 1;
        }
        let tie_count = (j - i) as f64;
        let mid_rank = (i + 1 + j) as f64 / 2.0;

        if tie_count > 1.0 {
            tie_sum += tie_count.powi(3) - tie_count;
        }

        for item in &combined[i..j] {
            if item.1 == 0 {
                sum_ranks1 += mid_rank;
            } else {
                sum_ranks2 += mid_rank;
            }
        }
        i = j;
    }

    let u1 = sum_ranks1 - (n1 * (n1 + 1.0)) / 2.0;
    let u2 = sum_ranks2 - (n2 * (n2 + 1.0)) / 2.0;
    let u = u1.min(u2);

    let mean_u = (n1 * n2) / 2.0;
    let n = n1 + n2;
    let tie_correction = if n > 1.0 {
        tie_sum / (n * (n - 1.0))
    } else {
        0.0
    };
    let var_u = ((n1 * n2) / 12.0) * (n + 1.0 - tie_correction);

    if var_u <= 1e-12 {
        return (u, 1.0);
    }

    let std_u = var_u.sqrt();
    let diff = (u - mean_u).abs();
    // Continuity correction
    let z = if diff > 0.5 {
        (diff - 0.5) / std_u
    } else {
        0.0
    };

    let p = 2.0 * (1.0 - normal_cdf(z));
    (u, p.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_erf_zero() {
        // Abramowitz-Stegun has max error 1.5e-7, so we use 1e-6 tolerance
        assert!(erf(0.0).abs() < 1e-6);
    }

    #[test]
    fn test_erf_one() {
        // erf(1) ≈ 0.8427
        assert!((erf(1.0) - 0.8427).abs() < 0.001);
    }

    #[test]
    fn test_erf_negative() {
        // erf(-x) = -erf(x)
        assert!((erf(-1.0) + erf(1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_normal_cdf_zero() {
        // Φ(0) = 0.5, using 1e-6 for Abramowitz-Stegun approximation
        assert!((normal_cdf(0.0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_normal_cdf_196() {
        // Φ(1.96) ≈ 0.975
        assert!((normal_cdf(1.96) - 0.975).abs() < 0.001);
    }

    #[test]
    fn test_normal_cdf_inv() {
        // Inverse of 0.975 ≈ 1.96
        assert!((normal_cdf_inv(0.975) - 1.96).abs() < 0.01);
        // TAG: surface=simulation owner=research-team rule=SIM-001
    }

    #[test]
    fn test_gini_equal() {
        // Perfect equality
        assert!(gini_coefficient(&[10.0, 10.0, 10.0, 10.0]).abs() < 1e-10);
    }

    #[test]
    fn test_gini_unequal() {
        let g = gini_coefficient(&[1.0, 2.0, 3.0, 4.0]);
        assert!(g > 0.0 && g < 1.0);
    }

    #[test]
    fn test_cohens_d_equal_groups() {
        let g1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let g2 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (d, _, _) = cohens_d_with_ci(&g1, &g2);
        assert!(d.abs() < 1e-10);
    }

    #[test]
    fn test_cohens_d_different_groups() {
        let g1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let g2 = vec![6.0, 7.0, 8.0, 9.0, 10.0];
        let (d, ci_low, ci_high) = cohens_d_with_ci(&g1, &g2);
        assert!(d < 0.0); // g2 has higher mean
        assert!(ci_low < d && d < ci_high);
    }

    #[test]
    fn test_shannon_entropy_equal_bins() {
        // 4 equal bins -> H ≈ log2(4) = 2
        let h = shannon_entropy_mm(&[100, 100, 100, 100]);
        assert!((h - 2.0).abs() < 0.1);
    }

    #[test]
    fn test_shannon_entropy_single_bin() {
        // All in one bin -> H = 0
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let h = shannon_entropy_mm(&[100, 0, 0, 0]);
        assert!(h.abs() < 0.1);
    }

    #[test]
    fn test_pearson_r_perfect() {
        let r = pearson_r(&[1.0, 2.0, 3.0], &[2.0, 4.0, 6.0]);
        assert!((r - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_pearson_r_negative() {
        let r = pearson_r(&[1.0, 2.0, 3.0], &[6.0, 4.0, 2.0]);
        assert!((r + 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_fisher_z_ci() {
        let (r, ci_low, ci_high) = fisher_z_ci(0.5, 100);
        assert!((r - 0.5).abs() < 1e-10);
        assert!(ci_low < 0.5);
        assert!(ci_high > 0.5);
    }

    #[test]
    fn test_mean_with_ci() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (m, ci_low, ci_high) = mean_with_ci(&values);
        assert!((m - 3.0).abs() < 1e-10);
        assert!(ci_low < 3.0);
        assert!(ci_high > 3.0);
    }

    #[test]
    fn test_std_dev() {
        let values = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let s = std_dev(&values);
        // Expected std dev ≈ 2.138
        assert!((s - 2.138).abs() < 0.01);
    }
    #[test]
    fn test_bca_bootstrap_ci() {
        let sample = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let (low, high) = bca_bootstrap_ci(&sample, mean, 200, 0.05, 42);
        assert!(low < 5.5 && high > 5.5);
        assert!(low >= 1.0 && high <= 10.0);
    }

    #[test]
    fn test_permutation_test_significance() {
        let g1 = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let g2 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (diff, p_val) = permutation_test(&g1, &g2, 200, 42);
        assert_eq!(diff, 9.0);
        assert!(p_val < 0.05);

        // Group swap symmetry (negative effect detected with identical significance)
        let (diff_rev, p_val_rev) = permutation_test(&g2, &g1, 200, 42);
        assert_eq!(diff_rev, -9.0);
        assert!((p_val - p_val_rev).abs() < 1e-6);
    }

    #[test]
    fn test_welch_t_test() {
        let g1 = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let g2 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (t, df, p) = welch_t_test(&g1, &g2);
        assert!(t > 0.0);
        assert!(df >= 4.0);
        assert!(p < 0.001);

        // Identical groups should yield t = 0, p = 1
        let (t_eq, _, p_eq) = welch_t_test(&g1, &g1);
        assert_eq!(t_eq, 0.0);
        assert_eq!(p_eq, 1.0);
    }

    #[test]
    fn test_mann_whitney_u() {
        let g1 = vec![1.0, 2.0, 3.0];
        let g2 = vec![4.0, 5.0, 6.0];
        let (u, p) = mann_whitney_u(&g1, &g2);
        // All elements in g1 are smaller than g2, so U = 0
        assert_eq!(u, 0.0);
        assert!(p < 0.1);

        // Test with ties
        let tied1 = vec![2.0, 2.0, 4.0];
        let tied2 = vec![2.0, 3.0, 5.0];
        let (u_tied, p_tied) = mann_whitney_u(&tied1, &tied2);
        assert!(u_tied >= 0.0 && u_tied <= 9.0);
        assert!(p_tied <= 1.0);
    }
}
