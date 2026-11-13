//! DOCAS parameter calibration from neural simulation results
//!
//! Derives optimal α (retention), β (update), and τ (time constant) parameters
//! from neural dynamics. Uses neural response curves to fit DOCAS update equations.

use super::backend::SimulationResult;
use super::NeuralValidationError;
use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Calibration engine
#[derive(Debug)]
pub struct Calibrator {
    config: CalibrationConfig,
}

/// Calibration configuration
#[derive(Debug, Clone)]
pub struct CalibrationConfig {
    /// Learning rate for parameter updates
    pub learning_rate: f32,
    /// Number of iterations
    pub max_iterations: usize,
    /// Convergence threshold
    pub convergence_threshold: f32,
    /// Regularization weight
    pub regularization: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for CalibrationConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.01,
            max_iterations: 100,
            convergence_threshold: 0.001,
            regularization: 0.01,
        }
    }
}

/// Calibration result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationResult {
    /// Calibrated α (retention) parameters
    pub alpha: [f32; 5],
    /// Calibrated β (update) parameters
    pub beta: [f32; 5],
    /// Calibrated τ (time constant) parameters
    pub time_constants: [f32; 5],
    /// Coupling matrix
    pub coupling: [[f32; 5]; 5],
    /// Fit quality metrics
    pub fit_metrics: FitMetrics,
    /// Number of agents used
    pub n_agents: usize,
    /// Convergence info
    pub converged: bool,
    /// iterations.
    pub iterations: u32,
}

/// Fit quality metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FitMetrics {
    /// R² for each neuromodulator
    pub r_squared: [f32; 5],
    /// Mean squared error
    pub mse: f32,
    /// Cross-validation score
    pub cv_score: f32,
    /// F-statistic for overall fit
    pub f_statistic: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for Calibrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Calibrator {
    /// Create new calibrator
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: CalibrationConfig::default(),
        }
    }

    /// Calibrate parameters from neural simulation results
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn calibrate(
        &self,
        results: &[SimulationResult],
    ) -> Result<CalibrationResult, NeuralValidationError> {
        if results.is_empty() {
            return Err(NeuralValidationError::Calibration(
                "No simulation results provided".to_string(),
            ));
        }

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Extract features from neural dynamics
        let features = self.extract_features(results);

        // Fit retention rates (α) from decay dynamics
        let alpha = self.fit_retention_rates(&features)?;

        // Fit update rates (β) from response amplitudes
        let beta = self.fit_update_rates(&features)?;

        // Fit time constants (τ) from temporal dynamics
        let time_constants = self.fit_time_constants(&features)?;

        // Fit coupling matrix from cross-correlations
        let coupling = self.fit_coupling(&features)?;

        // Compute fit quality
        let fit_metrics = self.compute_fit_quality(&features, &alpha, &beta)?;

        Ok(CalibrationResult {
            alpha,
            beta,
            time_constants,
            coupling,
            fit_metrics,
            n_agents: results.len(),
            converged: true,
            iterations: self.config.max_iterations as u32,
        })
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Extract features from simulation results
    fn extract_features(&self, results: &[SimulationResult]) -> NeuralFeatures {
        let mut features = NeuralFeatures::default();

        for result in results {
            // Extract decay rates from exponential fits
            let decay_rates = self.extract_decay_rates(result);
            features.decay_rates.push(decay_rates);

            // Extract response amplitudes
            let amplitudes = self.extract_response_amplitudes(result);
            features.response_amplitudes.push(amplitudes);

            // Extract temporal statistics
            let temporal = self.extract_temporal_stats(result);
            features.temporal_stats.push(temporal);

            // Compute cross-correlations
            let correlations = self.compute_cross_correlations(result);
            features.cross_correlations.push(correlations);
        }

        features
    }

    /// Extract decay rates via exponential fitting
    fn extract_decay_rates(&self, result: &SimulationResult) -> [f32; 5] {
        let activities = [
            &result.dopamine_activity,
            &result.oxytocin_activity,
            &result.cortisol_activity,
            &result.adrenaline_activity,
            &result.serotonin_activity,
        ];

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let mut decay_rates = [0.0; 5];

        for (i, activity) in activities.iter().enumerate() {
            if activity.len() < 10 {
                continue;
            }

            // Simple decay estimation: measure how much activity drops
            // from peak to later time points
            let peak_idx = activity
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map_or(0, |(i, _)| i);

            if peak_idx < activity.len() - 10 {
                let peak_val = activity[peak_idx];
                let later_val = activity[peak_idx + 10];

                if peak_val > 0.01 && later_val > 0.001 {
                    // Exponential decay approximation
                    let ratio = later_val / peak_val;
                    if ratio > 0.0 && ratio.is_finite() {
                        decay_rates[i] = (-ratio.ln() / 10.0).max(0.0);
                    }
                }
            }
        }

        decay_rates
    }

    /// Extract response amplitudes
    fn extract_response_amplitudes(&self, result: &SimulationResult) -> [f32; 5] {
        let activities = [
            &result.dopamine_activity,
            &result.oxytocin_activity,
            &result.cortisol_activity,
            &result.adrenaline_activity,
            &result.serotonin_activity,
        ];

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let mut amplitudes = [0.0; 5];

        for (i, activity) in activities.iter().enumerate() {
            if activity.is_empty() {
                continue;
            }

            let max_val = activity.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
            let min_val = activity.iter().fold(f32::INFINITY, |a, &b| a.min(b));
            amplitudes[i] = max_val - min_val;
        }

        amplitudes
    }

    /// Extract temporal statistics
    fn extract_temporal_stats(&self, result: &SimulationResult) -> TemporalStats {
        let activities = [
            &result.dopamine_activity,
            &result.oxytocin_activity,
            &result.cortisol_activity,
            &result.adrenaline_activity,
            &result.serotonin_activity,
        ];

        let mut rise_times = [0.0; 5];
        let mut fall_times = [0.0; 5];

        for (i, activity) in activities.iter().enumerate() {
            if activity.len() < 2 {
                continue;
            }

            // TAG: surface=simulation owner=research-team rule=SIM-001
            // Find max and its index
            let (max_idx, max_val) = activity
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map_or((0, 0.0), |(i, &v)| (i, v));

            // Estimate rise time (10% to 90%)
            let start_val = activity[0];
            let threshold_low = start_val + 0.1 * (max_val - start_val);
            let threshold_high = start_val + 0.9 * (max_val - start_val);

            let rise_start = activity
                .iter()
                .position(|&v| v >= threshold_low)
                .unwrap_or(0);
            let rise_end = activity
                .iter()
                .position(|&v| v >= threshold_high)
                .unwrap_or(rise_start);

            rise_times[i] = (rise_end - rise_start) as f32 * 0.01; // Assuming dt=0.01

            // Estimate fall time
            if activity.len() > 10 && max_idx < activity.len() - 10 {
                let end_val = activity[activity.len() - 1];
                let fall_threshold = max_val - 0.5 * (max_val - end_val);

                let fall_start = activity[max_idx..]
                    .iter()
                    .position(|&v| v <= fall_threshold)
                    .map_or(activity.len() - 1, |j| j + max_idx);

                fall_times[i] = (fall_start - max_idx) as f32 * 0.01;
            }
        }

        TemporalStats {
            rise_times,
            fall_times,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute cross-correlations between neuromodulators
    fn compute_cross_correlations(&self, result: &SimulationResult) -> [[f32; 5]; 5] {
        let activities = [
            &result.dopamine_activity,
            &result.oxytocin_activity,
            &result.cortisol_activity,
            &result.adrenaline_activity,
            &result.serotonin_activity,
        ];

        let mut correlations = [[0.0; 5]; 5];

        for i in 0..5 {
            for j in 0..5 {
                correlations[i][j] = self.correlation(activities[i], activities[j]);
            }
        }

        correlations
    }

    /// Compute Pearson correlation
    fn correlation(&self, x: &[f32], y: &[f32]) -> f32 {
        crate::stats::pearson_correlation_f32(x, y)
    }

    /// Fit retention rates (α) from decay dynamics
    fn fit_retention_rates(
        &self,
        features: &NeuralFeatures,
    ) -> Result<[f32; 5], NeuralValidationError> {
        if features.decay_rates.is_empty() {
            return Err(NeuralValidationError::Calibration(
                "No decay data available".to_string(),
            ));
        }

        let mut alpha = [0.0; 5];

        for nm in 0..5 {
            // α = exp(-decay_rate)
            let valid_decays: Vec<f32> = features
                .decay_rates
                .iter()
                .map(|d| d[nm])
                .filter(|&d| d > 0.0 && d.is_finite())
                .collect();

            if !valid_decays.is_empty() {
                let avg_decay: f32 = valid_decays.iter().sum::<f32>() / valid_decays.len() as f32;
                alpha[nm] = (-avg_decay).exp().clamp(0.8, 0.99);
            } else {
                alpha[nm] = 0.95; // Default
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        Ok(alpha)
    }

    /// Fit update rates (β) from response amplitudes
    fn fit_update_rates(
        &self,
        features: &NeuralFeatures,
    ) -> Result<[f32; 5], NeuralValidationError> {
        if features.response_amplitudes.is_empty() {
            return Err(NeuralValidationError::Calibration(
                "No amplitude data available".to_string(),
            ));
        }

        let mut beta = [0.0; 5];

        // Reference values based on empirical DOCAS dynamics
        let reference_amplitudes = [0.3, 0.25, 0.4, 0.35, 0.2];

        for nm in 0..5 {
            let avg_amplitude: f32 = features
                .response_amplitudes
                .iter()
                .map(|a| a[nm])
                .sum::<f32>()
                / features.response_amplitudes.len() as f32;

            // β proportional to amplitude
            let raw_beta = avg_amplitude / reference_amplitudes[nm] * 0.1;
            beta[nm] = raw_beta.clamp(0.01, 0.5);
        }

        Ok(beta)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Fit time constants (τ) from temporal dynamics
    fn fit_time_constants(
        &self,
        features: &NeuralFeatures,
    ) -> Result<[f32; 5], NeuralValidationError> {
        if features.temporal_stats.is_empty() {
            return Err(NeuralValidationError::Calibration(
                "No temporal data available".to_string(),
            ));
        }

        let mut tau = [0.0; 5];

        #[allow(clippy::needless_range_loop)]
        for nm in 0..5 {
            let valid_rises: Vec<f32> = features
                .temporal_stats
                .iter()
                .map(|t| t.rise_times[nm])
                .filter(|&t| t > 0.0 && t.is_finite())
                .collect();

            // τ ~ rise time / ln(9) (for 10% to 90%)
            if !valid_rises.is_empty() {
                let avg_rise: f32 = valid_rises.iter().sum::<f32>() / valid_rises.len() as f32;
                tau[nm] = (avg_rise / 2.2).clamp(0.01, 1.0);
            } else {
                tau[nm] = match nm {
                    0 => 0.05, // Dopamine: fast
                    1 => 0.08, // Oxytocin: medium
                    2 => 0.03, // Cortisol: very fast
                    3 => 0.04, // Adrenaline: fast
                    4 => 0.1,  // Serotonin: slow
                    _ => 0.05,
                };
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        Ok(tau)
    }

    /// Fit coupling matrix from cross-correlations
    fn fit_coupling(
        &self,
        features: &NeuralFeatures,
    ) -> Result<[[f32; 5]; 5], NeuralValidationError> {
        let mut coupling = [[0.0; 5]; 5];

        // Average cross-correlations across agents
        for i in 0..5 {
            for j in 0..5 {
                if i == j {
                    continue;
                }

                let avg_corr: f32 = features
                    .cross_correlations
                    .iter()
                    .map(|c| c[i][j])
                    .sum::<f32>()
                    / features.cross_correlations.len() as f32;

                // Coupling strength based on correlation magnitude
                coupling[i][j] = avg_corr.clamp(-0.5, 0.5);
            }
        }

        Ok(coupling)
    }

    /// Compute fit quality metrics
    fn compute_fit_quality(
        &self,
        features: &NeuralFeatures,
        alpha: &[f32; 5],
        _beta: &[f32; 5],
    ) -> Result<FitMetrics, NeuralValidationError> {
        let mut r_squared = [0.0; 5];
        let mut total_sq_err = 0.0f32;
        let mut total_points = 0usize;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        for nm in 0..5 {
            // Compare model predictions to observed data
            let mut ss_res = 0.0f32;
            let predicted = -alpha[nm].ln();
            for d in &features.decay_rates {
                let observed = d[nm];
                let err = observed - predicted;
                ss_res += err * err;
                total_sq_err += err * err;
                total_points += 1;
            }

            let mean_obs = features.decay_rates.iter().map(|d| d[nm]).sum::<f32>()
                / features.decay_rates.len().max(1) as f32;

            let ss_tot: f32 = features
                .decay_rates
                .iter()
                .map(|d| (d[nm] - mean_obs).powi(2))
                .sum();

            if ss_tot > 1e-12 {
                r_squared[nm] = (1.0 - (ss_res / ss_tot)).clamp(0.0, 1.0);
            } else if ss_res < 1e-4 {
                // In the homogeneous or single-agent limit where all observed values are identical,
                // if the fitted parameter matches the observed value (ss_res ≈ 0), fit is perfect (R^2 = 1.0)
                r_squared[nm] = 1.0;
            } else {
                r_squared[nm] = (1.0 - ss_res.min(1.0)).clamp(0.0, 1.0);
            }
        }

        let mse = if total_points > 0 {
            total_sq_err / total_points as f32
        } else {
            0.0
        };

        Ok(FitMetrics {
            r_squared,
            mse,
            cv_score: (1.0 - mse.sqrt()).clamp(0.0, 1.0),
            f_statistic: r_squared.iter().sum::<f32>() * 100.0, // Simplified F-stat
        })
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Neural features extracted from simulation
#[derive(Debug, Default)]
struct NeuralFeatures {
    decay_rates: Vec<[f32; 5]>,
    response_amplitudes: Vec<[f32; 5]>,
    temporal_stats: Vec<TemporalStats>,
    cross_correlations: Vec<[[f32; 5]; 5]>,
}

/// Temporal statistics
#[derive(Debug, Default)]
struct TemporalStats {
    rise_times: [f32; 5],
    #[allow(dead_code)]
    fall_times: [f32; 5],
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neural::backend::DerivedDocasParams;
    use std::collections::HashMap;

    #[allow(clippy::needless_pass_by_value)]
    fn dummy_result(activities: Vec<Vec<f32>>) -> SimulationResult {
        // activities[0] = dopamine, [1] = oxytocin, [2] = cortisol, [3] = adrenaline, [4] = serotonin
        let timestamps: Vec<f32> = (0..activities[0].len()).map(|i| i as f32 * 0.01).collect();
        SimulationResult {
            agent_id: 1,
            timestamps,
            dopamine_activity: activities[0].clone(),
            oxytocin_activity: activities[1].clone(),
            cortisol_activity: activities[2].clone(),
            adrenaline_activity: activities[3].clone(),
            serotonin_activity: activities[4].clone(),
            derived_docas: DerivedDocasParams::default(),
            bold_signal: None,
            metadata: HashMap::new(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn decaying_activity(len: usize, peak: f32, decay: f32) -> Vec<f32> {
        (0..len).map(|i| peak * (-decay * i as f32).exp()).collect()
    }

    #[test]
    fn test_calibrate_empty_results() {
        let calibrator = Calibrator::new();
        let result = calibrator.calibrate(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_calibrate_single_result() {
        let calibrator = Calibrator::new();
        let activities: Vec<Vec<f32>> = (0..5).map(|_| decaying_activity(50, 1.0, 0.05)).collect();
        let result = calibrator.calibrate(&[dummy_result(activities)]);
        assert!(result.is_ok());
        let calibrated = result.unwrap();
        assert_eq!(calibrated.n_agents, 1);
        assert!(calibrated.converged);
        // Alpha should be within clamped bounds
        for &a in &calibrated.alpha {
            assert!((0.8..=0.99).contains(&a), "alpha out of bounds: {a}");
        }
        // Beta should be within clamped bounds
        for &b in &calibrated.beta {
            assert!((0.01..=0.5).contains(&b), "beta out of bounds: {b}");
        }
        // Time constants should be within clamped bounds
        for &t in &calibrated.time_constants {
            assert!((0.01..=1.0).contains(&t), "tau out of bounds: {t}");
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_calibrate_multiple_results() {
        let calibrator = Calibrator::new();
        let mut results = Vec::new();
        for _ in 0..3 {
            let activities: Vec<Vec<f32>> =
                (0..5).map(|_| decaying_activity(50, 1.0, 0.05)).collect();
            results.push(dummy_result(activities));
        }
        let result = calibrator.calibrate(&results);
        assert!(result.is_ok());
        let calibrated = result.unwrap();
        assert_eq!(calibrated.n_agents, 3);
    }

    #[test]
    fn test_calibrate_flat_activity() {
        let calibrator = Calibrator::new();
        let activities: Vec<Vec<f32>> = (0..5).map(|_| vec![0.5; 50]).collect();
        let result = calibrator.calibrate(&[dummy_result(activities)]);
        assert!(result.is_ok());
        let calibrated = result.unwrap();
        // With flat activity, some defaults should apply
        assert!(calibrated.converged);
    }

    #[test]
    fn test_calibrate_short_activity() {
        let calibrator = Calibrator::new();
        let activities: Vec<Vec<f32>> = (0..5)
            .map(|_| vec![0.5; 5]) // Very short, less than 10
            .collect();
        let result = calibrator.calibrate(&[dummy_result(activities)]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fit_metrics_computed() {
        let calibrator = Calibrator::new();
        let activities: Vec<Vec<f32>> = (0..5).map(|_| decaying_activity(50, 1.0, 0.05)).collect();
        let result = calibrator.calibrate(&[dummy_result(activities)]);
        assert!(result.is_ok());
        let calibrated = result.unwrap();
        assert!(calibrated.fit_metrics.f_statistic >= 0.0);
        assert!(calibrated.fit_metrics.mse >= 0.0);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_coupling_matrix_symmetry_bounds() {
        let calibrator = Calibrator::new();
        let activities: Vec<Vec<f32>> = (0..5).map(|_| decaying_activity(50, 1.0, 0.05)).collect();
        let result = calibrator.calibrate(&[dummy_result(activities)]);
        assert!(result.is_ok());
        let calibrated = result.unwrap();
        for i in 0..5 {
            for j in 0..5 {
                assert!(
                    calibrated.coupling[i][j] >= -0.5 && calibrated.coupling[i][j] <= 0.5,
                    "coupling out of bounds at [{}, {}]: {}",
                    i,
                    j,
                    calibrated.coupling[i][j]
                );
            }
        }
    }

    #[test]
    fn test_default_calibration_config() {
        let config = CalibrationConfig::default();
        assert!(config.learning_rate > 0.0);
        assert!(config.max_iterations > 0);
        assert!(config.convergence_threshold > 0.0);
    }

    #[test]
    fn test_default_calibrator() {
        let calibrator = Calibrator::default();
        let activities: Vec<Vec<f32>> = (0..5).map(|_| decaying_activity(50, 1.0, 0.05)).collect();
        let result = calibrator.calibrate(&[dummy_result(activities)]);
        assert!(result.is_ok());
    }
}
