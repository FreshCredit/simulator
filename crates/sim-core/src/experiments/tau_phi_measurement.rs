//! `τ_φ` (Philosophical Oscillation) Measurement Experiment (PHIL-P0-1)
//!
//! Defines and validates the `τ_φ` metric for measuring philosophical oscillation
//! between different philosophical frameworks (e.g., deontology vs utilitarianism).
//!
//! ## Hypothesis
//! - H1: Philosophical frameworks oscillate with measurable periodicity
//! - H2: `τ_φ` = 0.32 represents typical oscillation amplitude
//! - H3: Critical slowing down precedes philosophical paradigm shifts
//!
//! ## References
//! - Kuhn (1962): Structure of Scientific Revolutions
//! - Page (2008): Difference
//! - Nowak (2011): `SuperCooperators`

use crate::experiments::{Experiment, ExperimentConfig, ExperimentResult};
use anyhow::Result;
use serde_json::json;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Philosophical framework types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhilosophicalFramework {
    /// deontology.
    Deontology, // Duty-based ethics
    /// utilitarianism.
    Utilitarianism, // Consequence-based ethics
    /// virtue.
    Virtue, // Character-based ethics
    /// care.
    Care, // Relationship-based ethics
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Philosophical position over time
#[derive(Debug, Clone)]
pub struct PhilosophicalPosition {
    /// Time tick
    pub tick: u32,
    /// Position on deontology-utilitarianism axis (-1 to 1)
    pub utilitarian_score: f64,
    /// Position on virtue-care axis (-1 to 1)
    pub care_score: f64,
    /// Current dominant framework
    pub dominant_framework: PhilosophicalFramework,
}

/// Oscillation metrics
#[derive(Debug, Clone)]
pub struct OscillationMetrics {
    /// Measured oscillation period
    pub period: f64,
    /// Oscillation amplitude (`τ_φ`)
    pub amplitude: f64,
    /// Number of paradigm shifts detected
    pub paradigm_shifts: u32,
    /// Average time between shifts
    pub avg_time_between_shifts: f64,
    /// Critical slowing down detected
    pub critical_slowing: bool,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// `τ_φ` Measurement Experiment
///
/// PHIL-P0-1: Defines and validates the philosophical oscillation metric.
#[derive(Debug)]
pub struct TauPhiMeasurement;

impl TauPhiMeasurement {
    /// Simulate philosophical oscillation dynamics
    fn simulate_oscillation(&self, n_ticks: usize) -> Vec<PhilosophicalPosition> {
        let mut positions = Vec::new();
        let mut utilitarian_score = 0.0;
        let mut care_score = 0.0;
        let mut velocity_u = 0.1;
        let mut velocity_c = 0.05;

        for tick in 0..n_ticks {
            // Oscillator with damping and periodic forcing
            // Models attraction to current paradigm + external influence
            let attraction_u = -0.01 * utilitarian_score; // Return to center
            let attraction_c = -0.01 * care_score;

            // Periodic forcing (social/political cycles)
            let forcing_u = 0.1 * (tick as f64 / 100.0).sin();
            let forcing_c = 0.08 * ((tick as f64 / 150.0) + 1.0).sin();

            // Noise
            let noise_u = (rand::random::<f64>() - 0.5) * 0.05;
            let noise_c = (rand::random::<f64>() - 0.5) * 0.05;

            velocity_u += attraction_u + forcing_u + noise_u;
            velocity_c += attraction_c + forcing_c + noise_c;

            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            // Damping
            velocity_u *= 0.95;
            velocity_c *= 0.95;

            utilitarian_score += velocity_u * 0.1;
            care_score += velocity_c * 0.1;

            // Clamp
            utilitarian_score = utilitarian_score.clamp(-1.0, 1.0);
            care_score = care_score.clamp(-1.0, 1.0);

            // Determine dominant framework
            let dominant = if utilitarian_score.abs() > care_score.abs() {
                if utilitarian_score > 0.0 {
                    PhilosophicalFramework::Utilitarianism
                } else {
                    PhilosophicalFramework::Deontology
                }
            } else if care_score > 0.0 {
                PhilosophicalFramework::Care
            } else {
                PhilosophicalFramework::Virtue
            };

            positions.push(PhilosophicalPosition {
                tick: tick as u32,
                utilitarian_score,
                care_score,
                dominant_framework: dominant,
            });
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        positions
    }

    /// Calculate oscillation metrics from position history
    #[allow(clippy::collection_is_never_read)]
    fn calculate_metrics(&self, positions: &[PhilosophicalPosition]) -> OscillationMetrics {
        if positions.len() < 10 {
            return OscillationMetrics {
                period: 0.0,
                amplitude: 0.0,
                paradigm_shifts: 0,
                avg_time_between_shifts: 0.0,
                critical_slowing: false,
            };
        }

        // Find peaks and troughs in utilitarian_score
        let mut peaks = Vec::new();
        let mut troughs = Vec::new();

        for window in positions.windows(3) {
            if window[1].utilitarian_score > window[0].utilitarian_score
                && window[1].utilitarian_score > window[2].utilitarian_score
            {
                peaks.push(window[1].tick);
            }
            if window[1].utilitarian_score < window[0].utilitarian_score
                && window[1].utilitarian_score < window[2].utilitarian_score
            {
                troughs.push(window[1].tick);
            }
        }

        // Calculate period from peak spacing
        let period = if peaks.len() >= 2 {
            let total_spacing: u32 = peaks.windows(2).map(|w| w[1] - w[0]).sum();
            total_spacing as f64 / (peaks.len() - 1) as f64
        } else {
            100.0 // Default
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Calculate amplitude (τ_φ)
        let max_u = positions
            .iter()
            .map(|p| p.utilitarian_score)
            .fold(f64::MIN, f64::max);
        let min_u = positions
            .iter()
            .map(|p| p.utilitarian_score)
            .fold(f64::MAX, f64::min);
        let amplitude = (max_u - min_u) / 2.0;

        // Count paradigm shifts (framework changes)
        let mut shifts = 0;
        let mut last_framework = positions[0].dominant_framework;
        let mut shift_times = Vec::new();

        for pos in positions.iter().skip(1) {
            if pos.dominant_framework != last_framework {
                shifts += 1;
                shift_times.push(pos.tick);
                last_framework = pos.dominant_framework;
            }
        }

        // Calculate average time between shifts
        let avg_time = if shift_times.len() >= 2 {
            let total: u32 = shift_times.windows(2).map(|w| w[1] - w[0]).sum();
            total as f64 / (shift_times.len() - 1) as f64
        } else {
            0.0
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Detect critical slowing down (increasing period before shift)
        let critical_slowing = if shift_times.len() >= 3 {
            let intervals: Vec<u32> = shift_times.windows(2).map(|w| w[1] - w[0]).collect();
            let early_avg: f64 = intervals[..intervals.len() / 2].iter().sum::<u32>() as f64
                / (intervals.len() / 2).max(1) as f64;
            let late_avg: f64 = intervals[intervals.len() / 2..].iter().sum::<u32>() as f64
                / (intervals.len() - intervals.len() / 2).max(1) as f64;
            late_avg > early_avg * 1.5
        } else {
            false
        };

        OscillationMetrics {
            period,
            amplitude,
            paradigm_shifts: shifts,
            avg_time_between_shifts: avg_time,
            critical_slowing,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Experiment for TauPhiMeasurement {
    fn name(&self) -> &'static str {
        "tau_phi_measurement"
    }

    fn description(&self) -> &'static str {
        "Measure philosophical oscillation amplitude and period (PHIL-P0-1)"
    }

    fn required_agents(&self) -> usize {
        100
    }

    fn run(&self, config: &ExperimentConfig) -> Result<ExperimentResult> {
        let n_ticks = config.n_timesteps;
        let positions = self.simulate_oscillation(n_ticks);
        let metrics = self.calculate_metrics(&positions);

        // Validation criteria
        let amplitude_in_range = metrics.amplitude > 0.2 && metrics.amplitude < 0.5;
        let period_reasonable = metrics.period > 50.0 && metrics.period < 200.0;
        let oscillation_detected = metrics.paradigm_shifts > 0;
        let tau_phi_estimate = metrics.amplitude;

        let criteria = [
            ("τ_φ in [0.20, 0.50]", amplitude_in_range),
            ("Period reasonable", period_reasonable),
            ("Oscillation detected", oscillation_detected),
            ("τ_φ ≈ 0.32", (tau_phi_estimate - 0.32).abs() < 0.1),
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        ];

        let criteria_passed = criteria.iter().filter(|(_, c)| *c).count();
        let success = criteria_passed >= 3;

        let json_metrics = json!({
            "tau_phi": metrics.amplitude,
            "period": metrics.period,
            "paradigm_shifts": metrics.paradigm_shifts,
            "avg_time_between_shifts": metrics.avg_time_between_shifts,
            "critical_slowing": metrics.critical_slowing,
            "amplitude_in_range": amplitude_in_range,
            "period_reasonable": period_reasonable,
            "criteria_passed": criteria_passed,
            "criteria_total": criteria.len(),
        });

        Ok(ExperimentResult {
            experiment_type: "tau_phi_measurement".to_string(),
            success,
            gap_id: "PHIL-P0-1".to_string(),
            paper: "PHIL".to_string(),
            metrics: json_metrics,
            figures: vec![],
            csv_outputs: vec![],
            artifacts: vec![],
            duration: std::time::Duration::from_secs(0),
            error_message: None,
        })
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_runs() {
        let exp = TauPhiMeasurement;
        let positions = exp.simulate_oscillation(1000);
        assert_eq!(positions.len(), 1000);
    }

    #[test]
    fn test_metrics_calculation() {
        let exp = TauPhiMeasurement;
        let positions = exp.simulate_oscillation(1000);
        let metrics = exp.calculate_metrics(&positions);

        assert!(metrics.amplitude >= 0.0 && metrics.amplitude <= 1.0);
        assert!(metrics.period >= 0.0);
    }

    #[test]
    fn test_experiment_run() {
        let exp = TauPhiMeasurement;
        let config = ExperimentConfig::quick();
        let result = exp.run(&config).unwrap();

        assert_eq!(result.experiment_type, "tau_phi_measurement");
        assert_eq!(result.gap_id, "PHIL-P0-1");
    }
}
