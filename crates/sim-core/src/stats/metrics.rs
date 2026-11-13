// TAG: surface=simulation owner=research-team rule=SIM-001
//! Correlation coefficients, ranks, Fisher transformations, entropy, and Gini inequality

use super::{pearson_r, DataQualityReport};
/// Returns the active statistical metrics module version.
#[inline]
#[must_use]
pub const fn stats_metrics_version() -> &'static str {
    "0.1.0"
}

/// Metric bounds check for correlation coefficients
#[inline]
#[must_use]
pub fn is_valid_correlation(r: f32) -> bool {
    r.is_finite() && (-1.0..=1.0).contains(&r)
}

/// Metric bounds check for correlation coefficients f64
#[inline]
#[must_use]
pub fn is_valid_correlation_f64(r: f64) -> bool {
    r.is_finite() && (-1.0..=1.0).contains(&r)
}

#[inline]
fn filter_finite_pairs_f64(x: &[f64], y: &[f64]) -> Vec<(f64, f64)> {
    x.iter()
        .zip(y.iter())
        .filter(|(&a, &b)| a.is_finite() && b.is_finite())
        .map(|(&a, &b)| (a, b))
        .collect()
}

#[inline]
fn filter_finite_pairs_f32(x: &[f32], y: &[f32]) -> Vec<(f32, f32)> {
    x.iter()
        .zip(y.iter())
        .filter(|(&a, &b)| a.is_finite() && b.is_finite())
        .map(|(&a, &b)| (a, b))
        .collect()
}

/// Calculate Pearson correlation coefficient for f64 slices.
///
/// Returns 0.0 if lengths mismatch, sample size < 2, or denominator is zero.
#[must_use]
pub fn pearson_correlation_f64(x: &[f64], y: &[f64]) -> f64 {
    let clean = filter_finite_pairs_f64(x, y);

    if clean.len() < 2 {
        return 0.0;
    }

    let n = clean.len() as f64;
    let mean_x = clean.iter().map(|p| p.0).sum::<f64>() / n;
    let mean_y = clean.iter().map(|p| p.1).sum::<f64>() / n;

    let mut num = 0.0;
    let mut den_x = 0.0;
    let mut den_y = 0.0;

    for (xi, yi) in clean {
        let dx = xi - mean_x;
        let dy = yi - mean_y;
        num += dx * dy;
        den_x += dx * dx;
        den_y += dy * dy;
    }

    let den = (den_x * den_y).sqrt();
    if den == 0.0 || !den.is_finite() {
        0.0
    } else {
        (num / den).clamp(-1.0, 1.0)
    }
}

/// Convert values to mid-ranks (handling ties by assigning average rank).
#[must_use]
pub fn rank_data_f64(data: &[f64]) -> Vec<f64> {
    if data.is_empty() {
        return Vec::new();
    }
    let mut indexed: Vec<(usize, f64)> = data.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| {
        a.1.is_finite()
            .cmp(&b.1.is_finite())
            .reverse()
            .then_with(|| a.1.total_cmp(&b.1))
            .then_with(|| a.0.cmp(&b.0))
    });

    let mut ranks = vec![0.0; data.len()];
    let mut i = 0;

    while i < indexed.len() {
        let mut j = i + 1;
        while j < indexed.len()
            && (indexed[j].1.total_cmp(&indexed[i].1).is_eq()
                || (indexed[j].1 - indexed[i].1).abs() < f64::EPSILON)
        {
            j += 1;
        }

        let avg_rank = (i + 1 + j) as f64 / 2.0;
        for k in i..j {
            ranks[indexed[k].0] = avg_rank;
        }
        i = j;
    }

    ranks
}

/// Calculate Spearman rank correlation coefficient for f64 slices.
#[must_use]
pub fn spearman_correlation_f64(x: &[f64], y: &[f64]) -> f64 {
    if x.len() != y.len() || x.len() < 2 {
        return 0.0;
    }

    let clean = filter_finite_pairs_f64(x, y);
    if clean.len() < 2 {
        return 0.0;
    }
    let (clean_x, clean_y): (Vec<f64>, Vec<f64>) = clean.into_iter().unzip();

    let x_ranks = rank_data_f64(&clean_x);
    let y_ranks = rank_data_f64(&clean_y);

    pearson_correlation_f64(&x_ranks, &y_ranks)
}

/// Calculate Pearson correlation coefficient for f32 slices.
///
/// Returns 0.0 if lengths mismatch, sample size < 2, or denominator is zero.
#[must_use]
pub fn pearson_correlation_f32(x: &[f32], y: &[f32]) -> f32 {
    let clean = filter_finite_pairs_f32(x, y);

    if clean.len() < 2 {
        return 0.0;
    }

    let n = clean.len() as f32;
    let mean_x = clean.iter().map(|p| p.0).sum::<f32>() / n;
    let mean_y = clean.iter().map(|p| p.1).sum::<f32>() / n;

    let mut num = 0.0;
    let mut den_x = 0.0;
    let mut den_y = 0.0;

    for (xi, yi) in clean {
        let dx = xi - mean_x;
        let dy = yi - mean_y;
        num += dx * dy;
        den_x += dx * dx;
        den_y += dy * dy;
    }

    let den = (den_x * den_y).sqrt();
    if den == 0.0 || !den.is_finite() {
        0.0
    } else {
        (num / den).clamp(-1.0, 1.0)
    }
}

/// Pearson correlation with data quality report
#[must_use]
pub fn pearson_r_with_validation(
    x: &[f32],
    y: &[f32],
) -> (Option<f32>, DataQualityReport, DataQualityReport) {
    let report_x = DataQualityReport::from_data(x, 0.5);
    let report_y = DataQualityReport::from_data(y, 0.5);

    let r = pearson_r(x, y);

    (r, report_x, report_y)
}

/// Calculate Pearson correlation coefficient for f64 slices, returning Option<f64>.
#[must_use]
pub fn pearson_r_f64(x: &[f64], y: &[f64]) -> Option<f64> {
    let clean = filter_finite_pairs_f64(x, y);

    if clean.len() < 2 {
        return None;
    }

    let n = clean.len() as f64;
    let mean_x = clean.iter().map(|p| p.0).sum::<f64>() / n;
    let mean_y = clean.iter().map(|p| p.1).sum::<f64>() / n;

    let mut num = 0.0;
    let mut den_x = 0.0;
    let mut den_y = 0.0;

    for (xi, yi) in clean {
        let dx = xi - mean_x;
        let dy = yi - mean_y;
        num += dx * dy;
        den_x += dx * dx;
        den_y += dy * dy;
    }

    let den = (den_x * den_y).sqrt();
    if den == 0.0 || !den.is_finite() {
        None
    } else {
        Some((num / den).clamp(-1.0, 1.0))
    }
}

/// Fisher z-transformation for correlation confidence intervals
#[must_use]
pub fn fisher_z_ci(r: f32, n: usize) -> (f32, f32) {
    if !r.is_finite() || n < 4 {
        return (r, r);
    }

    let r_clamped = r.clamp(-0.9999, 0.9999);
    let z = 0.5 * ((1.0 + r_clamped) / (1.0 - r_clamped)).ln();
    let se_z = 1.0 / ((n as f32 - 3.0).sqrt());

    let z_lower = z - 1.96 * se_z;
    let z_upper = z + 1.96 * se_z;

    let r_lower = z_lower.tanh().clamp(-1.0, 1.0);
    let r_upper = z_upper.tanh().clamp(-1.0, 1.0);

    (r_lower, r_upper)
}

/// Shannon entropy with Miller-Madow correction
#[must_use]
pub fn shannon_entropy_mm(counts: &[u32]) -> f32 {
    let n: u32 = counts.iter().sum();
    if n == 0 {
        return 0.0;
    }

    let n_f = n as f32;
    let mut entropy = 0.0;
    let k = counts.iter().filter(|&&c| c > 0).count() as f32;
    if k <= 1.0 {
        return 0.0;
    }

    for &count in counts {
        if count > 0 {
            let p = count as f32 / n_f;
            entropy -= p * p.log2();
        }
    }

    let correction = (k - 1.0) / (2.0 * n_f * 2.0_f32.ln());
    (entropy + correction).max(0.0)
}

/// Gini coefficient for inequality measurement
#[must_use]
pub fn gini_coefficient(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }

    let mut clean_values: Vec<f64> = values
        .iter()
        .copied()
        .filter(|&v| v.is_finite())
        .map(|v| f64::from(v.max(0.0)))
        .collect();

    if clean_values.len() < 2 {
        return 0.0;
    }

    clean_values.sort_by(|a, b| a.total_cmp(b));

    let n = clean_values.len() as f64;
    let total: f64 = clean_values.iter().sum();

    if total <= 0.0 {
        return 0.0;
    }

    let mut rank_weighted_sum = 0.0f64;
    for (idx, &val) in clean_values.iter().enumerate() {
        let i = (idx + 1) as f64;
        rank_weighted_sum += (2.0 * i - n - 1.0) * val;
    }

    let gini = rank_weighted_sum / (n * total);
    (gini as f32).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spearman_correlation_purges_nan_pairs() {
        let x = vec![1.0, 2.0, 3.0, f64::NAN, 4.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 8.0];
        let r = spearman_correlation_f64(&x, &y);
        // Clean pairs: (1,2), (2,4), (3,6), (4,8) - monotonically increasing
        assert!(
            (r - 1.0).abs() < 1e-6,
            "Spearman r must equal 1.0 on cleaned monotonic pairs, got {r}"
        );
    }

    #[test]
    fn test_shannon_entropy_mm_single_category() {
        let counts = vec![100, 0, 0];
        let h = shannon_entropy_mm(&counts);
        assert_eq!(h, 0.0);
    }

    #[test]
    fn test_rank_data_f64_tie_reflexivity_and_rank_sum() {
        let sample = vec![1.0, 2.0, 2.0, f64::INFINITY, f64::INFINITY];
        let n = sample.len();
        let expected_sum = (n * (n + 1)) as f64 / 2.0;
        let ranks = rank_data_f64(&sample);
        let actual_sum: f64 = ranks.iter().sum();
        assert!(
            (actual_sum - expected_sum).abs() < 1e-9,
            "Rank sum must be conserved: expected {expected_sum}, got {actual_sum}"
        );
        assert_eq!(ranks[1], ranks[2]);
        assert_eq!(ranks[3], ranks[4]);
    }
}
