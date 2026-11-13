// TAG: surface=simulation owner=research-team rule=SIM-001
//! GDP Model Validation Tests
//!
//! Validates ECON-P0-1: GDP = `f(bond_density` × trust) with R² > 0.85
//! Per §6.23: Economic model validation against empirical correlation

// Numeric cast precision is acceptable in these simulation test utilities.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::float_cmp
)]

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use sim_core::economics::compute_gdp_contribution;

/// Test ECON-P0-1: GDP model R² > 0.85
///
/// Runs multiple simulations with varying `bond_density` and trust values,
/// computes GDP, and validates that the model explains >85% of variance.
#[test]
fn test_gdp_model_r_squared() {
    let mut rng = StdRng::seed_from_u64(42);

    // Generate test data across parameter space
    let n_samples = 1000;
    let mut bond_trust_products = Vec::new();
    let mut gdp_values = Vec::new();

    for _ in 0..n_samples {
        // Random bond density and trust in realistic ranges
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let bond_density: f32 = rng.gen_range(0.1..0.9);
        let trust: f32 = rng.gen_range(0.2..0.9);
        let product = bond_density * trust;

        // Simulate agents with these parameters
        let n_agents = 100;
        let mut total_gdp = 0.0;

        for _ in 0..n_agents {
            // GDP contribution per agent
            let contribution = compute_gdp_contribution(bond_density, trust, 50.0, 0.3, &mut rng);
            total_gdp += contribution;
        }

        bond_trust_products.push(product);
        gdp_values.push(total_gdp);
    }

    // Compute R²
    let r_squared = compute_r_squared(&bond_trust_products, &gdp_values);

    println!("GDP Model R² = {r_squared:.4}");
    println!("Target: R² > 0.85");

    // ECON-P0-1: R² must exceed 0.85
    assert!(
        // TAG: surface=simulation owner=research-team rule=SIM-001
        r_squared > 0.85,
        "ECON-P0-1 FAILED: GDP model R² = {r_squared:.4}, required > 0.85"
    );

    // Also check that correlation is positive (as documented)
    let correlation = compute_correlation(&bond_trust_products, &gdp_values);
    assert!(
        correlation > 0.0,
        "GDP model correlation should be positive, got {correlation:.4}"
    );
}

/// Test GDP components are in reasonable ranges
#[test]
fn test_gdp_components_reasonable() {
    let mut rng = StdRng::seed_from_u64(42);

    // Test various economic scenarios
    let scenarios = vec![
        (0.3, 0.4, "low_bond_low_trust"),
        (0.7, 0.8, "high_bond_high_trust"),
        (0.5, 0.5, "medium"),
    ];

    for (bond, trust, name) in scenarios {
        let gdp = compute_gdp_contribution(bond, trust, 50.0, 0.3, &mut rng);
        // TAG: surface=simulation owner=research-team rule=SIM-001

        // GDP should be positive and finite
        assert!(gdp > 0.0, "{name}: GDP should be positive");
        assert!(gdp.is_finite(), "{name}: GDP should be finite");

        println!("{name}: bond={bond}, trust={trust}, GDP={gdp:.2}");
    }
}

/// Compute R-squared coefficient of determination
fn compute_r_squared(x: &[f32], y: &[f32]) -> f32 {
    assert_eq!(x.len(), y.len());

    let n = x.len() as f32;
    let mean_y = y.iter().sum::<f32>() / n;

    // Total sum of squares
    let ss_tot: f32 = y.iter().map(|&yi| (yi - mean_y).powi(2)).sum();

    // Compute linear regression: y = a + bx
    let mean_x = x.iter().sum::<f32>() / n;

    let numerator: f32 = x
        .iter()
        .zip(y.iter())
        .map(|(&xi, &yi)| (xi - mean_x) * (yi - mean_y))
        .sum();
    // TAG: surface=simulation owner=research-team rule=SIM-001

    let denominator: f32 = x.iter().map(|&xi| (xi - mean_x).powi(2)).sum();

    if denominator == 0.0 {
        return 0.0;
    }

    let slope = numerator / denominator;
    let intercept = slope.mul_add(-mean_x, mean_y);

    // Residual sum of squares
    let ss_res: f32 = x
        .iter()
        .zip(y.iter())
        .map(|(&xi, &yi)| {
            let predicted = slope.mul_add(xi, intercept);
            (yi - predicted).powi(2)
        })
        .sum();

    if ss_tot == 0.0 {
        return 1.0;
    }

    1.0 - (ss_res / ss_tot)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute Pearson correlation coefficient
fn compute_correlation(x: &[f32], y: &[f32]) -> f32 {
    assert_eq!(x.len(), y.len());

    let n = x.len() as f32;
    let mean_x = x.iter().sum::<f32>() / n;
    let mean_y = y.iter().sum::<f32>() / n;

    let mut num = 0.0;
    let mut den_x = 0.0;
    let mut den_y = 0.0;

    for i in 0..x.len() {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        num += dx * dy;
        den_x += dx * dx;
        den_y += dy * dy;
    }

    let den = (den_x * den_y).sqrt();
    if den == 0.0 {
        0.0
    } else {
        num / den
    }
}
