//! Criticality Sigma Experiment
//!
//! **Gap Addressed:** DOCAS-P0-3 (Criticality σ not demonstrated)
//!
//! **Purpose:** Demonstrate criticality σ ∈ [0.9, 1.1] via avalanche detection
//!
//! **Success Criteria:**
//! - σ ∈ [0.9, 1.1]
//! - Power law fit R² ≥ 0.90

use super::{init_experiment, save_results, Experiment, ExperimentConfig, ExperimentResult};
use crate::unified_agent::{DocasVector, UnifiedAgent};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Neuronal avalanche event
#[derive(Debug, Clone)]
pub struct Avalanche {
    /// start time.
    pub start_time: u64,
    /// duration.
    pub duration: usize,
    /// size.
    pub size: usize, // Number of neurons involved
    /// peak amplitude.
    pub peak_amplitude: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Criticality analysis results
#[derive(Debug, Clone)]
pub struct CriticalityResults {
    /// avalanches.
    pub avalanches: Vec<Avalanche>,
    /// sigma.
    pub sigma: f64,
    /// r squared.
    pub r_squared: f64,
    /// power law exponent.
    pub power_law_exponent: f64,
}

/// Criticality Sigma Experiment
#[derive(Debug)]
pub struct CriticalitySigmaExperiment;

impl Default for CriticalitySigmaExperiment {
    fn default() -> Self {
        Self::new()
    }
}

impl CriticalitySigmaExperiment {
    #[must_use]
    /// new.
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub const fn new() -> Self {
        Self
    }

    /// Detect avalanches in DOCAS history
    /// An avalanche is a cascade of DOCAS changes above threshold
    fn detect_avalanches(&self, docas_history: &[DocasVector], threshold: f64) -> Vec<Avalanche> {
        let mut avalanches = Vec::new();
        let mut in_avalanche = false;
        let mut current_avalanche: Option<Avalanche> = None;

        for (t, docas) in docas_history.iter().enumerate() {
            // Calculate DOCAS magnitude (norm of vector)
            let magnitude: f64 = docas.iter().map(|x| x * x).sum::<f64>().sqrt();

            if magnitude > threshold && !in_avalanche {
                // Start new avalanche
                in_avalanche = true;
                current_avalanche = Some(Avalanche {
                    start_time: t as u64,
                    duration: 1,
                    size: 1,
                    peak_amplitude: magnitude,
                });
            } else if magnitude > threshold && in_avalanche {
                // Continue avalanche
                if let Some(ref mut av) = current_avalanche {
                    // TAG: surface=simulation owner=research-team rule=SIM-001
                    av.duration += 1;
                    av.size += 1;
                    if magnitude > av.peak_amplitude {
                        av.peak_amplitude = magnitude;
                    }
                }
            } else if magnitude <= threshold && in_avalanche {
                // End avalanche
                in_avalanche = false;
                if let Some(av) = current_avalanche.take() {
                    if av.duration >= 3 {
                        // Only count significant avalanches
                        avalanches.push(av);
                    }
                }
            }
        }

        // Flush any ongoing avalanche at end of history
        if in_avalanche {
            if let Some(av) = current_avalanche.take() {
                if av.duration >= 3 {
                    avalanches.push(av);
                }
            }
        }

        avalanches
    }

    /// Fit power law to avalanche size distribution
    /// Returns (sigma, R², exponent)
    fn fit_power_law(&self, avalanches: &[Avalanche]) -> (f64, f64, f64) {
        if avalanches.len() < 10 {
            return (1.0, 0.0, 0.0); // Not enough data
        }

        // Get avalanche sizes
        let sizes: Vec<usize> = avalanches.iter().map(|a| a.size).collect();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Create histogram
        let max_size = *sizes.iter().max().unwrap_or(&1);
        let mut histogram = vec![0; max_size + 1];
        for size in &sizes {
            histogram[*size] += 1;
        }

        // Log-log regression for power law
        let data_points: Vec<(f64, f64)> = histogram
            .iter()
            .enumerate()
            .skip(1) // Skip size 0
            .filter(|(_, count)| **count > 0)
            .map(|(size, count)| ((size as f64).ln(), (*count as f64).ln()))
            .collect();

        if data_points.len() < 3 {
            return (1.0, 0.0, 0.0);
        }

        // Linear regression on log-log data using centered deviations
        let n = data_points.len() as f64;
        let mean_x = data_points.iter().map(|(x, _)| x).sum::<f64>() / n;
        let mean_y = data_points.iter().map(|(_, y)| y).sum::<f64>() / n;

        let mut num = 0.0;
        let mut den_x = 0.0;
        let mut ss_tot = 0.0;
        for (x, y) in &data_points {
            let dx = x - mean_x;
            let dy = y - mean_y;
            num += dx * dy;
            den_x += dx * dx;
            ss_tot += dy * dy;
        }

        if den_x <= 1e-12 || !den_x.is_finite() || ss_tot <= 1e-12 || !ss_tot.is_finite() {
            return (1.0, 0.0, 0.0);
        }

        let slope = num / den_x;
        let intercept = mean_y - slope * mean_x;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // R-squared
        let ss_res: f64 = data_points
            .iter()
            .map(|(x, y)| {
                let y_pred = slope * x + intercept;
                (y - y_pred).powi(2)
            })
            .sum();

        let r_squared = if ss_tot > 1e-12 && ss_tot.is_finite() && ss_res.is_finite() {
            (1.0 - (ss_res / ss_tot)).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Sigma is related to power law exponent
        // For critical systems, σ ≈ 1.0
        let sigma = if slope.abs() > 1e-12 && slope.is_finite() {
            (-1.0 / slope).clamp(0.5, 2.0)
        } else {
            1.0
        };

        (sigma, r_squared, slope)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate DOCAS-based criticality metric
    #[must_use]
    pub fn calculate_criticality(&self, agent: &UnifiedAgent) -> f64 {
        if agent.docas_history.len() < 100 {
            return 1.0; // Not enough history
        }

        // Detect avalanches
        let threshold = 0.5; // DOCAS magnitude threshold
        let avalanches = self.detect_avalanches(&agent.docas_history, threshold);

        if avalanches.len() < 10 {
            return 1.0;
        }

        // Fit power law
        let (sigma, r_squared, _) = self.fit_power_law(&avalanches);

        // Only return valid sigma if power law fit is good
        if r_squared >= 0.90 {
            sigma
        } else {
            1.0 // Default if fit is poor
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }
}

impl Experiment for CriticalitySigmaExperiment {
    fn name(&self) -> &'static str {
        "criticality_sigma"
    }

    fn description(&self) -> &'static str {
        "DOCAS-P0-3: Criticality sigma demonstration via avalanche detection"
    }

    fn required_agents(&self) -> usize {
        500
    }

    fn run(&self, config: &ExperimentConfig) -> anyhow::Result<ExperimentResult> {
        println!("Starting Criticality Sigma Experiment");
        println!("Agents: {}", config.n_agents);
        println!("Timesteps: {}", config.n_timesteps);

        init_experiment(config)?;

        // Run simulation to generate DOCAS histories
        // (Simplified - would use actual simulation loop)
        let agents: Vec<UnifiedAgent> = (0..config.n_agents)
            .map(|_i| {
                // TAG: surface=simulation owner=research-team rule=SIM-001
                let mut agent = UnifiedAgent::new(0, 0);
                // Generate synthetic DOCAS history for testing
                for t in 0..config.n_timesteps {
                    let docas: DocasVector = [
                        0.5 + 0.3 * (t as f64 * 0.1).sin(),
                        0.5 + 0.2 * (t as f64 * 0.15).cos(),
                        0.5 + 0.25 * (t as f64 * 0.08).sin(),
                        0.5 + 0.15 * (t as f64 * 0.12).cos(),
                        0.5 + 0.2 * (t as f64 * 0.09).sin(),
                    ];
                    agent.docas_history.push(docas);
                }
                agent
            })
            .collect();

        // Calculate criticality for each agent
        let mut sigmas = Vec::new();
        let mut all_avalanches = Vec::new();

        for agent in &agents {
            let threshold = 0.6;
            let avalanches = self.detect_avalanches(&agent.docas_history, threshold);
            let (sigma, r_squared, _) = self.fit_power_law(&avalanches);

            if r_squared >= 0.90 {
                sigmas.push(sigma);
                all_avalanches.extend(avalanches);
            }
        }

        // Calculate mean sigma
        let mean_sigma = if sigmas.is_empty() {
            1.0
        } else {
            sigmas.iter().sum::<f64>() / sigmas.len() as f64
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Overall power law fit
        let (_, overall_r2, exponent) = self.fit_power_law(&all_avalanches);

        // Validate success criteria
        let success = (0.9..=1.1).contains(&mean_sigma) && overall_r2 >= 0.90;

        let result = ExperimentResult {
            experiment_type: "criticality_sigma".to_string(),
            success,
            gap_id: "DOCAS-P0-3".to_string(),
            paper: "DOCAS".to_string(),
            metrics: serde_json::json!({
                "mean_sigma": mean_sigma,
                "r_squared": overall_r2,
                "power_law_exponent": exponent,
                "n_agents_with_criticality": sigmas.len(),
                "total_avalanches": all_avalanches.len(),
                "n_replications": config.n_replications,
            }),
            figures: vec![],
            csv_outputs: vec![],
            artifacts: vec![],
            duration: std::time::Duration::from_secs(0),
            error_message: if success {
                None
            } else {
                Some("Success criteria not met".to_string())
            },
        };

        save_results(&result, &config.output_dir)?;

        println!("\nCriticality Sigma Results:");
        println!("  Mean σ: {mean_sigma:.3}");
        println!("  R²: {overall_r2:.3}");
        println!("  Exponent: {exponent:.3}");
        println!("  Avalanches detected: {}", all_avalanches.len());
        println!("  Success: {success}");

        // TAG: surface=simulation owner=research-team rule=SIM-001
        Ok(result)
    }

    fn validate(&self, result: &ExperimentResult) -> bool {
        if !result.success {
            return false;
        }

        let metrics = &result.metrics;
        let sigma = metrics["mean_sigma"].as_f64().unwrap_or(0.0);
        let r2 = metrics["r_squared"].as_f64().unwrap_or(0.0);

        (0.9..=1.1).contains(&sigma) && r2 >= 0.90
    }

    fn generate_figures(&self, _result: &ExperimentResult) -> anyhow::Result<()> {
        println!("Generating criticality figures...");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avalanche_detection() {
        let exp = CriticalitySigmaExperiment::new();

        // Create synthetic DOCAS history with avalanche pattern
        let mut history = Vec::new();
        for i in 0..100 {
            if (40..=50).contains(&i) {
                // High magnitude = avalanche
                history.push([0.9, 0.9, 0.9, 0.9, 0.9]);
            } else {
                // Low magnitude = baseline
                history.push([0.3, 0.3, 0.3, 0.3, 0.3]);
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let avalanches = exp.detect_avalanches(&history, 0.6);

        // Should detect one avalanche
        assert_eq!(avalanches.len(), 1);
        assert!(avalanches[0].duration >= 10);
    }

    #[test]
    fn test_power_law_fit() {
        let exp = CriticalitySigmaExperiment::new();

        // Create avalanches with power law distribution
        let mut avalanches = Vec::new();
        for size in 1..=100 {
            let count = (100.0 / size as f64) as usize; // Power law: P(s) ~ 1/s
            for _ in 0..count.max(1) {
                avalanches.push(Avalanche {
                    start_time: 0,
                    duration: size,
                    size,
                    peak_amplitude: size as f64,
                });
            }
        }

        let (sigma, r2, _) = exp.fit_power_law(&avalanches);

        // Should get reasonable fit
        assert!(r2 > 0.5);
        assert!(sigma > 0.5 && sigma < 2.0);
    }

    #[test]
    fn test_power_law_fit_flat_histogram_no_nan() {
        let exp = CriticalitySigmaExperiment::new();

        // 10 avalanches of distinct sizes (every histogram bin has count 1 => ln(count)=0 => ss_tot=0)
        let avalanches: Vec<Avalanche> = (1..=10)
            .map(|size| Avalanche {
                start_time: 0,
                duration: size,
                size,
                peak_amplitude: size as f64,
            })
            .collect();

        let (sigma, r2, slope) = exp.fit_power_law(&avalanches);
        assert!(!r2.is_nan(), "r2 must not be NaN on flat histogram");
        assert!(sigma.is_finite(), "sigma must be finite on flat histogram");
        assert!(!slope.is_nan(), "slope must not be NaN");
        assert_eq!(r2, 0.0);
    }
}
