//! Stability Law Experiment (GTT-P0-6)
//!
//! Validates: "Stability law fit not tested" - R² ≥ 0.60 for trust stability
//!
//! Gap Analysis:
//! - Paper claims: Trust stability follows power law decay
//! - Missing: Explicit curve fitting validation
//! - Missing: R² calculation for stability model
//!
//! Methodology:
//! 1. Simulate trust evolution over time
//! 2. Measure stability metric S(t) = 1 / (1 + variance(τ))
//! 3. Fit power law: S(t) = S₀ × t^(-β)
//! 4. Validate R² ≥ 0.60

use crate::experiments::{Experiment, ExperimentConfig, ExperimentResult};
use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Instant;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Stability law experiment for GTT-P0-6
#[derive(Debug)]
pub struct StabilityLawExperiment;

/// Stability trajectory measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilityMeasurement {
    /// time.
    pub time: f64,
    /// stability.
    pub stability: f64,
    /// trust mean.
    pub trust_mean: f64,
    /// trust variance.
    pub trust_variance: f64,
}

/// Power law fit parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerLawFit {
    /// s0.
    pub s0: f64, // Initial stability
    /// beta.
    pub beta: f64, // Decay exponent
    /// r squared.
    pub r_squared: f64,
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
impl StabilityLawExperiment {
    /// Simulate trust stability decay
    fn simulate_stability_decay(
        &self,
        n_agents: usize,
        n_steps: usize,
        rng: &mut impl Rng,
    ) -> Vec<StabilityMeasurement> {
        let mut measurements = Vec::with_capacity(n_steps);

        // Initialize agents with varying trust
        let mut trusts: Vec<f64> = (0..n_agents).map(|_| rng.gen_range(0.3..0.7)).collect();

        let master_law = crate::master_law::MasterLawConfig::for_preset(
            crate::master_law::MasterLawPreset::BiologyHomeostatic,
        );
        let dissipation = crate::observation_dissipation::ObservationDissipationConfig::default();
        let mut epsilon = 0.0;

        for step in 0..n_steps {
            let time = step as f64;
            epsilon = dissipation.step_exact(epsilon, rng);

            // Update trust via Master Update Law with dissipative innovation
            for trust in &mut trusts {
                *trust = master_law.step(*trust, epsilon).clamp(0.0, 1.0);
            }

            // Calculate statistics
            let mean = trusts.iter().sum::<f64>() / n_agents as f64;
            let variance = trusts.iter().map(|t| (t - mean).powi(2)).sum::<f64>() / n_agents as f64;

            // Stability metric: inverse of variance
            let stability = 1.0 / (1.0 + variance * 10.0);

            // TAG: surface=simulation owner=research-team rule=SIM-001
            measurements.push(StabilityMeasurement {
                time,
                stability,
                trust_mean: mean,
                trust_variance: variance,
            });
        }

        measurements
    }

    /// Fit power law: S(t) = S₀ × t^(-β)
    fn fit_power_law(&self, measurements: &[StabilityMeasurement]) -> PowerLawFit {
        // Use log-log linear regression
        // ln(S) = ln(S₀) - β × ln(t)

        let data: Vec<(f64, f64)> = measurements.iter()
            .skip(1) // Skip t=0
            .filter(|m| m.stability > 0.01)
            .map(|m| (m.time.ln(), m.stability.ln()))
            .collect();

        if data.len() < 3 {
            return PowerLawFit {
                s0: 1.0,
                beta: 0.0,
                r_squared: 0.0,
            };
        }

        let n = data.len() as f64;
        let mean_x = data.iter().map(|(x, _)| x).sum::<f64>() / n;
        let mean_y = data.iter().map(|(_, y)| y).sum::<f64>() / n;

        let mut num = 0.0;
        let mut den = 0.0;
        let mut ss_res = 0.0;
        let mut ss_tot = 0.0;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        for (x, y) in &data {
            num += (x - mean_x) * (y - mean_y);
            den += (x - mean_x).powi(2);
        }

        let beta = if den > 1e-10 { -num / den } else { 0.0 };
        let ln_s0 = mean_y + beta * mean_x;
        let s0 = ln_s0.exp();

        // Calculate R²
        for (x, y) in &data {
            let y_pred = ln_s0 - beta * x;
            ss_res += (y - y_pred).powi(2);
            ss_tot += (y - mean_y).powi(2);
        }

        let r_squared = if ss_tot > 1e-10 && ss_tot.is_finite() && ss_res.is_finite() {
            (1.0 - ss_res / ss_tot).clamp(0.0, 1.0)
        } else {
            0.0
        };

        PowerLawFit {
            s0,
            beta,
            r_squared,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Experiment for StabilityLawExperiment {
    fn name(&self) -> &'static str {
        "stability_law"
    }

    fn description(&self) -> &'static str {
        "GTT-P0-6: Stability law fit validation (R² ≥ 0.60)"
    }

    fn required_agents(&self) -> usize {
        500
    }

    fn run(&self, config: &ExperimentConfig) -> Result<ExperimentResult> {
        let start_time = Instant::now();
        let mut rng = config.rng();

        // Run multiple replications
        let mut fits = Vec::new();
        let mut all_measurements = Vec::new();

        for rep in 0..config.n_replications {
            let measurements =
                self.simulate_stability_decay(config.n_agents, config.n_timesteps, &mut rng);

            let fit = self.fit_power_law(&measurements);
            fits.push(fit);

            if rep == 0 {
                all_measurements = measurements;
            }
        }

        // Aggregate results
        let mean_r2: f64 = fits.iter().map(|f| f.r_squared).sum::<f64>() / fits.len() as f64;
        let mean_beta: f64 = fits.iter().map(|f| f.beta).sum::<f64>() / fits.len() as f64;
        let mean_s0: f64 = fits.iter().map(|f| f.s0).sum::<f64>() / fits.len() as f64;

        // Success criterion: R² ≥ 0.60
        let success = mean_r2 >= 0.60;
        let duration = start_time.elapsed();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Build CSV
        let mut csv_data = String::from("time,stability,trust_mean,trust_variance\n");
        for m in &all_measurements {
            csv_data.push_str(&format!(
                "{:.2},{:.4},{:.4},{:.6}\n",
                m.time, m.stability, m.trust_mean, m.trust_variance
            ));
        }

        // Summary
        let summary = format!(
            "Stability Law Experiment (GTT-P0-6)\n\
             ===================================\n\
             Agents: {}\n\
             Timesteps: {}\n\
             Replications: {}\n\
             \n\
             Power Law Fit: S(t) = S₀ × t^(-β)\n\
             - S₀ (initial): {:.4}\n\
             - β (decay): {:.4}\n\
             - R²: {:.4}\n\
             \n\
             Success Criterion: R² ≥ 0.60\n\
             Result: {}\n",
            config.n_agents,
            config.n_timesteps,
            config.n_replications,
            mean_s0,
            mean_beta,
            mean_r2,
            if success { "✓ PASSED" } else { "✗ FAILED" }
        );

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let artifacts = vec![
            ("stability_trajectory.csv".to_string(), csv_data),
            ("stability_summary.txt".to_string(), summary),
        ];

        let metadata = serde_json::json!({
            "mean_r_squared": mean_r2,
            "mean_beta": mean_beta,
            "mean_s0": mean_s0,
            "n_replications": config.n_replications,
        });

        Ok(ExperimentResult {
            success,
            experiment_type: "stability_law".to_string(),
            gap_id: "GTT-P0-6".to_string(),
            paper: "GTT".to_string(),
            figures: vec![],
            csv_outputs: vec![],
            metrics: metadata,
            artifacts,
            duration,
            error_message: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_law_fit() {
        let exp = StabilityLawExperiment;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Generate synthetic power law data
        let measurements: Vec<StabilityMeasurement> = (1..=100)
            .map(|i| {
                let t = i as f64;
                let s = 0.8 * t.powf(-0.3); // S(t) = 0.8 * t^(-0.3)
                StabilityMeasurement {
                    time: t,
                    stability: s,
                    trust_mean: 0.5,
                    trust_variance: 0.1,
                }
            })
            .collect();

        let fit = exp.fit_power_law(&measurements);

        // Should recover approximately S₀ ≈ 0.8, β ≈ 0.3
        assert!(fit.s0 > 0.7 && fit.s0 < 0.9);
        assert!(fit.beta > 0.2 && fit.beta < 0.4);
        assert!(fit.r_squared > 0.95); // Should fit very well
    }

    #[test]
    fn test_experiment_runs() {
        let config = ExperimentConfig::quick();
        let exp = StabilityLawExperiment;

        let result = exp.run(&config).expect("Experiment should run");

        assert!(result.metrics.get("mean_r_squared").is_some());
        assert!(result.duration.as_secs() < 10);
    }
}
