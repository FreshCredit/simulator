//! Statistics and correlation utilities for experiments

pub(crate) fn compute_correlation(data: &[(f32, f32)]) -> f32 {
    let clean: Vec<(f32, f32)> = data
        .iter()
        .copied()
        .filter(|&(x, y)| x.is_finite() && y.is_finite())
        .collect();

    if clean.len() < 2 {
        return 0.0;
    }

    let n = clean.len() as f32;
    let mean_x = clean.iter().map(|(x, _)| x).sum::<f32>() / n;
    let mean_y = clean.iter().map(|(_, y)| y).sum::<f32>() / n;

    let num: f32 = clean.iter().map(|(x, y)| (x - mean_x) * (y - mean_y)).sum();
    let denom_x: f32 = clean.iter().map(|(x, _)| (x - mean_x).powi(2)).sum();
    let denom_y: f32 = clean.iter().map(|(_, y)| (y - mean_y).powi(2)).sum();

    if denom_x > 0.0 && denom_y > 0.0 && denom_x.is_finite() && denom_y.is_finite() {
        let denom = denom_x.sqrt() * denom_y.sqrt();
        if denom > 0.0 && denom.is_finite() {
            (num / denom).clamp(-1.0, 1.0)
        } else {
            0.0
        }
    } else {
        0.0
    }
}

/// Compute Pearson correlation between two vectors
pub(crate) fn compute_correlation_single(x: &[f32], y: &[f32]) -> f32 {
    if x.len() != y.len() || x.len() < 2 {
        return 0.0;
    }

    let clean: Vec<(f32, f32)> = x
        .iter()
        .zip(y.iter())
        .filter(|&(&xi, &yi)| xi.is_finite() && yi.is_finite())
        .map(|(&xi, &yi)| (xi, yi))
        .collect();

    if clean.len() < 2 {
        return 0.0;
    }

    let n = clean.len() as f32;
    let mean_x = clean.iter().map(|(xi, _)| xi).sum::<f32>() / n;
    let mean_y = clean.iter().map(|(_, yi)| yi).sum::<f32>() / n;

    let num: f32 = clean
        .iter()
        .map(|(xi, yi)| (xi - mean_x) * (yi - mean_y))
        .sum();
    let denom_x: f32 = clean.iter().map(|(xi, _)| (xi - mean_x).powi(2)).sum();
    let denom_y: f32 = clean.iter().map(|(_, yi)| (yi - mean_y).powi(2)).sum();

    if denom_x > 0.0 && denom_y > 0.0 && denom_x.is_finite() && denom_y.is_finite() {
        let denom = denom_x.sqrt() * denom_y.sqrt();
        if denom > 0.0 && denom.is_finite() {
            (num / denom).clamp(-1.0, 1.0)
        } else {
            0.0
        }
    } else {
        0.0
    }
}

/// Compute R² coefficient of determination
/// # Errors
///
/// Returns an error if the operation fails.
pub(crate) fn compute_r2(actual: &[f32], predicted: &[f32]) -> Option<f32> {
    if actual.len() != predicted.len() || actual.is_empty() {
        return None;
    }

    let clean: Vec<(f32, f32)> = actual
        .iter()
        .zip(predicted.iter())
        .filter(|&(&a, &p)| a.is_finite() && p.is_finite())
        .map(|(&a, &p)| (a, p))
        .collect();

    if clean.is_empty() {
        return None;
    }

    let n = clean.len() as f32;
    let mean_actual = clean.iter().map(|(a, _)| a).sum::<f32>() / n;

    let ss_tot: f32 = clean.iter().map(|(a, _)| (a - mean_actual).powi(2)).sum();
    let ss_res: f32 = clean.iter().map(|(a, p)| (a - p).powi(2)).sum();

    if ss_tot > 1e-12 && ss_tot.is_finite() && ss_res.is_finite() {
        Some((1.0 - ss_res / ss_tot).clamp(0.0f32, 1.0f32))
    } else if ss_res <= 1e-12 {
        Some(1.0)
    } else {
        Some(0.0)
    }
}

pub(crate) const fn quadrant_to_f64(quad: sim_core::agent::CulturalQuadrant) -> f64 {
    match quad {
        sim_core::agent::CulturalQuadrant::HighBondHighTrust => 1.0,
        sim_core::agent::CulturalQuadrant::HighBondLowTrust => 2.0,
        sim_core::agent::CulturalQuadrant::LowBondHighTrust => 3.0,
        sim_core::agent::CulturalQuadrant::LowBondLowTrust => 4.0,
    }
}

pub(crate) fn compute_r2_f64(actual: &[f64], predicted: &[f64]) -> Option<f64> {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    if actual.len() != predicted.len() || actual.is_empty() {
        return None;
    }

    let clean: Vec<(f64, f64)> = actual
        .iter()
        .zip(predicted.iter())
        .filter(|&(&a, &p)| a.is_finite() && p.is_finite())
        .map(|(&a, &p)| (a, p))
        .collect();

    if clean.is_empty() {
        return None;
    }

    let n = clean.len() as f64;
    let mean_actual = clean.iter().map(|(a, _)| a).sum::<f64>() / n;
    let ss_tot: f64 = clean.iter().map(|(a, _)| (a - mean_actual).powi(2)).sum();
    let ss_res: f64 = clean.iter().map(|(a, p)| (a - p).powi(2)).sum();

    if ss_tot > 1e-12 && ss_tot.is_finite() && ss_res.is_finite() {
        Some((1.0 - ss_res / ss_tot).clamp(0.0, 1.0))
    } else if ss_res <= 1e-12 {
        Some(1.0)
    } else {
        Some(0.0)
    }
}

pub(crate) fn mean_f32(values: &[f32]) -> f32 {
    let clean: Vec<f32> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if clean.is_empty() {
        return 0.0;
    }
    clean.iter().sum::<f32>() / clean.len() as f32
}

pub(crate) fn std_f32(values: &[f32]) -> f32 {
    let clean: Vec<f32> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if clean.len() < 2 {
        return 0.0;
    }
    let mean = clean.iter().sum::<f32>() / clean.len() as f32;
    let denom = (clean.len() - 1) as f32;
    let var = clean.iter().map(|&v| (v - mean).powi(2)).sum::<f32>() / denom;
    if var > 0.0 && var.is_finite() {
        var.sqrt()
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_std_f32_bessel_correction() {
        let vals = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        // Mean = 5.0, sum of squares = 9 + 1 + 1 + 1 + 0 + 0 + 4 + 16 = 32
        // Sample variance = 32 / 7 ≈ 4.5714 => sample SD ≈ 2.138
        let s = std_f32(&vals);
        assert!((s - (32.0f32 / 7.0).sqrt()).abs() < 1e-5);
    }

    #[test]
    fn test_compute_r2_constant_actual() {
        let actual = vec![5.0, 5.0, 5.0];
        let predicted_match = vec![5.0, 5.0, 5.0];
        assert_eq!(compute_r2(&actual, &predicted_match), Some(1.0));

        let predicted_diff = vec![1.0, 2.0, 3.0];
        assert_eq!(compute_r2(&actual, &predicted_diff), Some(0.0));
    }

    #[test]
    fn test_stats_nan_and_inf_resilience() {
        let x = [1.0f32, 2.0, f32::NAN, 4.0, f32::INFINITY];
        let y = [2.0f32, 4.0, 6.0, 8.0, 10.0];
        let r = compute_correlation_single(&x, &y);
        assert!(
            (r - 1.0).abs() < 1e-5,
            "Expected 1.0 on clean pairs, got {r}"
        );

        let pairs = [(1.0f32, 2.0f32), (2.0, 4.0), (f32::NAN, 6.0), (4.0, 8.0)];
        let r_pairs = compute_correlation(&pairs);
        assert!((r_pairs - 1.0).abs() < 1e-5);

        let actual = [1.0f32, 2.0, 3.0];
        let predicted = [1.0f32, f32::NAN, 3.0];
        let r2 = compute_r2(&actual, &predicted);
        assert!(r2.is_some());
        let val = r2.unwrap();
        assert!(val.is_finite() && val >= 0.0 && val <= 1.0);

        let m = mean_f32(&[1.0, f32::NAN, 3.0, f32::INFINITY]);
        assert_eq!(m, 2.0);
    }
}
