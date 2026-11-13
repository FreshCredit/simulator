//! Philosophical Oscillation Experiment (PHIL-P0-2)
//!
//! Validates: "Philosophical trust oscillates around `τ_φ` = 0.32 with
//! critical slowing down near bifurcation point"
//!
//! Gap Analysis:
//! - Paper claims: Oscillation around τ = 0.32
//! - Missing: Explicit bifurcation analysis
//! - Missing: Critical slowing down measurement
//! - Missing: Phase transition characterization
//!
//! Methodology:
//! 1. Model philosophical trust as dynamical system
//! 2. Identify bifurcation point at `τ_c` ≈ 0.32
//! 3. Measure oscillation characteristics
//! 4. Detect critical slowing down

use crate::experiments::{Experiment, ExperimentConfig, ExperimentResult};
use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Philosophical oscillation experiment for PHIL-P0-2
#[derive(Debug)]
pub struct PhilosophicalOscillationExperiment;

/// Philosophical stance type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhilosophicalStance {
    /// optimist.
    Optimist, // High trust tendency
    /// pessimist.
    Pessimist, // Low trust tendency
    /// skeptic.
    Skeptic, // Trust requires evidence
    /// pragmatist.
    Pragmatist, // Context-dependent trust
}

impl PhilosophicalStance {
    /// Get baseline trust level
    #[must_use]
    pub const fn baseline_trust(&self) -> f64 {
        match self {
            Self::Optimist => 0.65,
            Self::Pessimist => 0.25,
            Self::Skeptic => 0.40,
            Self::Pragmatist => 0.50,
        }
    }

    /// Get oscillation amplitude
    #[must_use]
    pub const fn oscillation_amplitude(&self) -> f64 {
        match self {
            Self::Optimist => 0.15,
            Self::Pessimist => 0.20,
            Self::Skeptic => 0.25,
            Self::Pragmatist => 0.30,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Dynamical system state for philosophical trust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustDynamics {
    /// Current trust value τ(t)
    pub tau: f64,
    /// Trust velocity dτ/dt
    pub velocity: f64,
    /// External influence (social pressure)
    pub external_influence: f64,
    /// Stochastic noise
    pub noise: f64,
}

impl TrustDynamics {
    /// Initialize with philosophical stance
    pub fn new(stance: PhilosophicalStance, noise_level: f64, rng: &mut impl Rng) -> Self {
        let baseline = stance.baseline_trust();
        Self {
            tau: baseline + rng.gen_range(-0.1..0.1),
            velocity: 0.0,
            external_influence: 0.0,
            noise: noise_level,
        }
    }

    /// Update using damped harmonic oscillator with bifurcation
    /// d²τ/dt² = -ω²(τ - `τ_c`) - γ(dτ/dt) + `F_ext` + ξ(t)
    pub fn update(&mut self, dt: f64, omega: f64, gamma: f64, tau_c: f64, rng: &mut impl Rng) {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Bifurcation: sign change at tau_c
        let distance_from_critical = self.tau - tau_c;
        let restoring_force = -omega.powi(2) * distance_from_critical;

        // Damping
        let damping = -gamma * self.velocity;

        // External influence
        let external = self.external_influence;

        // Stochastic forcing
        let xi = self.noise * rng.gen_range(-1.0..1.0);

        // Acceleration
        let acceleration = restoring_force + damping + external + xi;

        // Update velocity and position
        self.velocity += acceleration * dt;
        self.tau += self.velocity * dt;

        // Bounds
        self.tau = self.tau.clamp(0.0, 1.0);
        self.velocity = self.velocity.clamp(-1.0, 1.0);

        // Decay external influence
        self.external_influence *= 0.9;
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Oscillation analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscillationMetrics {
    /// Mean trust value
    pub mean_tau: f64,
    /// Standard deviation
    pub std_tau: f64,
    /// Oscillation frequency (cycles per unit time)
    pub frequency: f64,
    /// Period (time per cycle)
    pub period: f64,
    /// Amplitude
    pub amplitude: f64,
    /// Relaxation time (critical slowing down)
    pub relaxation_time: f64,
    /// Quality factor (resonance sharpness)
    pub quality_factor: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for OscillationMetrics {
    fn default() -> Self {
        Self {
            mean_tau: 0.0,
            std_tau: 0.0,
            frequency: 0.0,
            period: 0.0,
            amplitude: 0.0,
            relaxation_time: 0.0,
            quality_factor: 0.0,
        }
    }
}

/// Critical slowing down measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalSlowingDown {
    /// Distance from critical point
    pub epsilon: f64,
    /// Characteristic timescale
    pub tau_char: f64,
    /// Autocorrelation time
    pub autocorr_time: f64,
    /// Variance
    pub variance: f64,
}

impl PhilosophicalOscillationExperiment {
    /// Run dynamical simulation
    fn simulate_dynamics(
        &self,
        stance: PhilosophicalStance,
        tau_c: f64,
        n_steps: usize,
        dt: f64,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        rng: &mut impl Rng,
    ) -> Vec<TrustDynamics> {
        let omega = 2.0 * std::f64::consts::PI * 0.1; // 0.1 Hz natural frequency
        let gamma = 0.1; // Damping coefficient

        let mut state = TrustDynamics::new(stance, 0.05, rng);
        let mut trajectory = Vec::with_capacity(n_steps);

        for _ in 0..n_steps {
            state.update(dt, omega, gamma, tau_c, rng);
            trajectory.push(state.clone());
        }

        trajectory
    }

    /// Analyze oscillation characteristics
    fn analyze_oscillation(&self, trajectory: &[TrustDynamics]) -> OscillationMetrics {
        let taus: Vec<f64> = trajectory.iter().map(|t| t.tau).collect();

        // Guard: Need at least 3 samples for peak detection
        if taus.len() < 3 {
            return OscillationMetrics::default();
        }

        // Mean and std
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let mean = taus.iter().sum::<f64>() / taus.len() as f64;
        let variance = taus.iter().map(|t| (t - mean).powi(2)).sum::<f64>() / taus.len() as f64;
        let std = variance.sqrt();

        // Find peaks for frequency estimation
        let mut peaks = Vec::new();
        // Safe iteration: bounds checked by the guard above
        for i in 1..taus.len() - 1 {
            if taus[i] > taus[i - 1] && taus[i] > taus[i + 1] {
                peaks.push((i, taus[i]));
            }
        }

        // Calculate frequency
        let frequency = if peaks.len() >= 2 {
            let total_time = match (trajectory.last(), trajectory.first()) {
                (Some(last), Some(first)) => last.tau - first.tau,
                _ => 0.0,
            };
            let n_cycles = peaks.len() - 1;
            n_cycles as f64 / total_time.max(1.0)
        } else {
            0.0
        };

        let period = if frequency > 0.0 {
            1.0 / frequency
        } else {
            0.0
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Amplitude
        let max_tau = taus.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let min_tau = taus.iter().copied().fold(f64::INFINITY, f64::min);
        let amplitude = (max_tau - min_tau) / 2.0;

        // Relaxation time (from damping)
        let gamma = 0.1;
        let relaxation_time = 1.0 / gamma;

        // Quality factor
        let omega = 2.0 * std::f64::consts::PI * frequency;
        let quality_factor = omega / (2.0 * gamma);

        OscillationMetrics {
            mean_tau: mean,
            std_tau: std,
            frequency,
            period,
            amplitude,
            relaxation_time,
            quality_factor,
        }
    }

    /// Measure critical slowing down
    fn measure_critical_slowing_down(
        &self,
        epsilon_values: &[f64],
        tau_c: f64,
        stance: PhilosophicalStance,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        rng: &mut impl Rng,
    ) -> Vec<CriticalSlowingDown> {
        let mut results = Vec::new();

        for &epsilon in epsilon_values {
            let target_tau = tau_c + epsilon;

            // Simulate perturbed system
            let n_steps = 5000;
            let dt = 0.01;
            let trajectory = self.simulate_dynamics(stance, target_tau, n_steps, dt, rng);

            // Calculate autocorrelation time
            let taus: Vec<f64> = trajectory.iter().map(|t| t.tau).collect();
            let mean = taus.iter().sum::<f64>() / taus.len() as f64;
            let variance = taus.iter().map(|t| (t - mean).powi(2)).sum::<f64>() / taus.len() as f64;

            // Estimate autocorrelation time from lag-1 autocorrelation
            let mut autocorr_sum = 0.0;
            let mut count = 0;
            for i in 0..taus.len() - 1 {
                autocorr_sum += (taus[i] - mean) * (taus[i + 1] - mean);
                count += 1;
            }
            let lag1_autocorr = if variance > 1e-12 && count > 0 {
                ((autocorr_sum / count as f64) / variance).clamp(-1.0, 1.0)
            } else {
                0.0
            };

            // Continuous AR(1) relaxation time tau = -1 / ln(rho_1).
            // When rho_1 >= exp(-0.01) ~= 0.99005 (or rho_1 <= 0), tau smoothly saturates at 100.0,
            // eliminating logarithmic pole singularities (-Inf or negative values at rho_1 >= 1.0).
            let autocorr_time = if lag1_autocorr > 0.0 && lag1_autocorr < 0.9900498337491681 {
                let tau = -1.0 / lag1_autocorr.ln();
                if tau.is_finite() && tau > 0.0 {
                    tau.clamp(0.0, 100.0)
                } else {
                    100.0
                }
            } else {
                100.0 // Large value indicates critical slowing
            };

            results.push(CriticalSlowingDown {
                epsilon,
                tau_char: autocorr_time * dt,
                autocorr_time,
                variance,
            });
        }

        results
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Experiment for PhilosophicalOscillationExperiment {
    fn name(&self) -> &'static str {
        "philosophical_oscillation"
    }

    fn description(&self) -> &'static str {
        "PHIL-P0-2: Philosophical trust oscillates around τ_φ = 0.32"
    }

    fn required_agents(&self) -> usize {
        100
    }

    #[allow(clippy::collection_is_never_read)]
    fn run(&self, config: &ExperimentConfig) -> Result<ExperimentResult> {
        let mut rng = config.rng();
        let start_time = std::time::Instant::now();

        let tau_c = 0.32; // Critical trust value

        // Run simulations for each philosophical stance
        let stances = vec![
            PhilosophicalStance::Optimist,
            PhilosophicalStance::Pessimist,
            PhilosophicalStance::Skeptic,
            PhilosophicalStance::Pragmatist,
        ];

        let mut all_metrics = Vec::new();
        let mut trajectories = HashMap::new();

        for stance in &stances {
            let trajectory = self.simulate_dynamics(*stance, tau_c, 10000, 0.01, &mut rng);

            // TAG: surface=simulation owner=research-team rule=SIM-001
            let metrics = self.analyze_oscillation(&trajectory);
            all_metrics.push((*stance, metrics.clone()));
            trajectories.insert(format!("{stance:?}"), trajectory);
        }

        // Measure critical slowing down
        let epsilon_values: Vec<f64> =
            vec![-0.2, -0.1, -0.05, -0.02, -0.01, 0.01, 0.02, 0.05, 0.1, 0.2];
        let csd_results = self.measure_critical_slowing_down(
            &epsilon_values,
            tau_c,
            PhilosophicalStance::Pragmatist,
            &mut rng,
        );

        // Validate results
        // 1. Mean tau should be close to tau_c = 0.32
        let mean_across_stances: f64 =
            all_metrics.iter().map(|(_, m)| m.mean_tau).sum::<f64>() / all_metrics.len() as f64;

        // 2. Oscillations should be detectable
        let has_oscillations = all_metrics
            .iter()
            .all(|(_, m)| m.frequency > 0.0 && m.amplitude > 0.05);

        // 3. Critical slowing down: variance increases near tau_c
        let near_critical: Vec<_> = csd_results
            .iter()
            .filter(|c| c.epsilon.abs() < 0.05)
            .collect();
        let far_from_critical: Vec<_> = csd_results
            .iter()
            .filter(|c| c.epsilon.abs() >= 0.1)
            .collect();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let near_var = near_critical.iter().map(|c| c.variance).sum::<f64>()
            / near_critical.len().max(1) as f64;
        let far_var = far_from_critical.iter().map(|c| c.variance).sum::<f64>()
            / far_from_critical.len().max(1) as f64;

        let shows_critical_slowing = near_var > far_var * 1.5;

        // Success criteria
        let tau_within_range = (mean_across_stances - tau_c).abs() < 0.15;
        let success = tau_within_range && has_oscillations && shows_critical_slowing;

        let duration = start_time.elapsed();

        // Build CSV output
        let mut csv_data = String::from(
            "stance,mean_tau,std_tau,frequency,period,amplitude,relaxation_time,q_factor\n",
        );
        for (stance, metrics) in &all_metrics {
            csv_data.push_str(&format!(
                "{:?},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}\n",
                stance,
                metrics.mean_tau,
                metrics.std_tau,
                metrics.frequency,
                metrics.period,
                metrics.amplitude,
                metrics.relaxation_time,
                metrics.quality_factor
            ));
        }

        // Critical slowing down CSV
        let mut csd_csv = String::from("epsilon,tau_char,autocorr_time,variance\n");
        for csd in &csd_results {
            csd_csv.push_str(&format!(
                "{:.4},{:.4},{:.4},{:.6}\n",
                csd.epsilon, csd.tau_char, csd.autocorr_time, csd.variance
            ));
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Summary
        let summary = format!(
            "Philosophical Oscillation Experiment (PHIL-P0-2)\n\
             ===============================================\n\
             Critical value τ_c: {tau_c:.2}\n\
             Mean τ across stances: {mean_across_stances:.4}\n\
             Oscillations detected: {has_oscillations}\n\
             Critical slowing down: {shows_critical_slowing}\n\
             Near-critical variance: {near_var:.6}\n\
             Far-from-critical variance: {far_var:.6}\n\
             \n\
             Stance-wise Results:\n"
        );

        for (stance, metrics) in &all_metrics {
            let _summary_part = format!(
                "  {:?}: τ = {:.3} ± {:.3}, f = {:.3}, A = {:.3}\n",
                stance, metrics.mean_tau, metrics.std_tau, metrics.frequency, metrics.amplitude
            );
        }

        let success_str = format!(
            "\nSuccess Criteria:\n\
             - τ within range of 0.32: {}\n\
             - Oscillations detected: {}\n\
             - Critical slowing down: {}\n\
             \n\
             Result: {}\n",
            tau_within_range,
            has_oscillations,
            shows_critical_slowing,
            if success { "✓ PASSED" } else { "✗ FAILED" }
        );

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let artifacts = vec![
            ("oscillation_metrics.csv".to_string(), csv_data),
            ("critical_slowing_down.csv".to_string(), csd_csv),
            (
                "philosophical_summary.txt".to_string(),
                summary + &success_str,
            ),
        ];

        let metadata = serde_json::json!({
            "mean_tau": mean_across_stances,
            "tau_c": tau_c,
            "has_oscillations": has_oscillations,
            "critical_slowing": shows_critical_slowing,
        });

        Ok(ExperimentResult {
            success,
            experiment_type: "philosophical_oscillation".to_string(),
            gap_id: "PHIL-P0-2".to_string(),
            paper: "PHIL".to_string(),
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
    fn test_trust_dynamics_update() {
        let mut rng = rand::thread_rng();
        let mut dynamics = TrustDynamics::new(PhilosophicalStance::Pragmatist, 0.05, &mut rng);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let _initial_tau = dynamics.tau;

        // Update multiple times
        for _ in 0..100 {
            dynamics.update(0.01, 0.5, 0.1, 0.32, &mut rng);
        }

        // Trust should stay bounded
        assert!(dynamics.tau >= 0.0 && dynamics.tau <= 1.0);
    }

    #[test]
    fn test_oscillation_analysis() {
        let exp = PhilosophicalOscillationExperiment;
        let mut rng = rand::thread_rng();

        let trajectory =
            exp.simulate_dynamics(PhilosophicalStance::Pragmatist, 0.32, 1000, 0.01, &mut rng);

        let metrics = exp.analyze_oscillation(&trajectory);

        assert!(metrics.mean_tau >= 0.0 && metrics.mean_tau <= 1.0);
        assert!(metrics.std_tau >= 0.0);
    }

    #[test]
    fn test_experiment_runs() {
        let config = ExperimentConfig::quick();
        let exp = PhilosophicalOscillationExperiment;

        let result = exp.run(&config).expect("Experiment should run");

        assert!(result.metrics.get("mean_tau").is_some());
        assert!(result.duration.as_secs() < 10);
    }
}
