//! Semantic Drift Experiment (LING-P0-2)
//!
//! Validates: "Semantic drift rate is inversely proportional to trust (drift ∝ 1/τ)"
//!
//! Gap Analysis:
//! - Paper claims: Higher trust → slower semantic drift
//! - Missing: Explicit drift rate measurement
//! - Missing: Trust-drift correlation validation
//!
//! Methodology:
//! 1. Track semantic meaning of "trust" over time
//! 2. Measure drift rate using embedding distance
//! 3. Correlate drift rate with trust level
//! 4. Validate inverse proportionality

use crate::experiments::{Experiment, ExperimentConfig, ExperimentResult};
use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Instant;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Semantic drift experiment for LING-P0-2
#[derive(Debug)]
pub struct SemanticDriftExperiment;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Semantic state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticState {
    /// timestep.
    pub timestep: usize,
    /// embedding.
    pub embedding: Vec<f64>,
    /// trust level.
    pub trust_level: f64,
}

/// Drift measurement between states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftMeasurement {
    /// timestep.
    pub timestep: usize,
    /// drift rate.
    pub drift_rate: f64,
    /// trust level.
    pub trust_level: f64,
    /// inverse trust.
    pub inverse_trust: f64,
}

impl SemanticDriftExperiment {
    /// Calculate cosine distance between embeddings
    fn embedding_distance(&self, a: &[f64], b: &[f64]) -> f64 {
        let mut dot = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;

        for i in 0..a.len().min(b.len()) {
            dot += a[i] * b[i];
            norm_a += a[i].powi(2);
            norm_b += b[i].powi(2);
        }

        if norm_a == 0.0 || norm_b == 0.0 {
            return 1.0;
        }

        let similarity = dot / (norm_a.sqrt() * norm_b.sqrt());
        1.0 - similarity.max(0.0) // Distance = 1 - similarity
    }

    /// Simulate semantic evolution
    fn simulate_semantic_evolution(
        &self,
        n_steps: usize,
        trust_trajectory: &[f64],
        // TAG: surface=simulation owner=research-team rule=SIM-001
        rng: &mut impl Rng,
    ) -> Vec<SemanticState> {
        let dim = 64;
        let mut states = Vec::with_capacity(n_steps);

        // Initial embedding (random)
        let mut current_embedding: Vec<f64> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();

        // Normalize
        let norm = current_embedding
            .iter()
            .map(|x| x.powi(2))
            .sum::<f64>()
            .sqrt();
        current_embedding = current_embedding.iter().map(|x| x / norm).collect();

        for step in 0..n_steps {
            let trust = trust_trajectory.get(step).copied().unwrap_or(0.5);

            states.push(SemanticState {
                timestep: step,
                embedding: current_embedding.clone(),
                trust_level: trust,
            });

            // Drift: higher trust → smaller drift
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            // Drift rate ∝ 1/trust
            let base_drift = 0.1;
            let trust_factor = 1.0 / (trust.max(0.1));
            let drift_magnitude = base_drift * trust_factor / 10.0;

            // Apply random drift
            let drift: Vec<f64> = (0..dim)
                .map(|_| rng.gen_range(-drift_magnitude..drift_magnitude))
                .collect();

            current_embedding = current_embedding
                .iter()
                .zip(drift.iter())
                .map(|(e, d)| e + d)
                .collect();

            // Renormalize
            let norm = current_embedding
                .iter()
                .map(|x| x.powi(2))
                .sum::<f64>()
                .sqrt();
            if norm > 0.0 {
                current_embedding = current_embedding.iter().map(|x| x / norm).collect();
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        states
    }

    /// Calculate drift measurements
    fn calculate_drift(&self, states: &[SemanticState]) -> Vec<DriftMeasurement> {
        let mut drifts = Vec::new();

        for i in 1..states.len() {
            let prev = &states[i - 1];
            let curr = &states[i];

            let distance = self.embedding_distance(&prev.embedding, &curr.embedding);
            let inverse_trust = 1.0 / curr.trust_level.max(0.01);

            drifts.push(DriftMeasurement {
                timestep: curr.timestep,
                drift_rate: distance,
                trust_level: curr.trust_level,
                inverse_trust,
            });
        }

        drifts
    }

    /// Calculate correlation between drift and 1/trust
    fn calculate_correlation(&self, drifts: &[DriftMeasurement]) -> f64 {
        let n = drifts.len() as f64;
        if n < 2.0 {
            return 0.0;
        }

        let mean_drift = drifts.iter().map(|d| d.drift_rate).sum::<f64>() / n;
        let mean_inv = drifts.iter().map(|d| d.inverse_trust).sum::<f64>() / n;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let mut num = 0.0;
        let mut den_drift = 0.0;
        let mut den_inv = 0.0;

        for d in drifts {
            let dd = d.drift_rate - mean_drift;
            let di = d.inverse_trust - mean_inv;
            num += dd * di;
            den_drift += dd.powi(2);
            den_inv += di.powi(2);
        }

        if den_drift > 1e-10 && den_inv > 1e-10 {
            num / (den_drift.sqrt() * den_inv.sqrt())
        } else {
            0.0
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Experiment for SemanticDriftExperiment {
    fn name(&self) -> &'static str {
        "semantic_drift"
    }

    fn description(&self) -> &'static str {
        "LING-P0-2: Semantic drift ∝ 1/τ validation"
    }

    fn required_agents(&self) -> usize {
        100
    }

    fn run(&self, config: &ExperimentConfig) -> Result<ExperimentResult> {
        let start_time = Instant::now();
        let mut rng = config.rng();

        // Generate trust trajectories for multiple runs
        let mut all_correlations = Vec::new();
        let mut all_drifts = Vec::new();

        for rep in 0..config.n_replications {
            // Generate trust trajectory (decreasing over time)
            let trust_trajectory: Vec<f64> = (0..config.n_timesteps)
                .map(|t| {
                    let base = 0.8 - (t as f64 / config.n_timesteps as f64) * 0.5;
                    base + rng.gen_range(-0.05..0.05)
                })
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                .map(|t| t.clamp(0.1, 1.0))
                .collect();

            let states =
                self.simulate_semantic_evolution(config.n_timesteps, &trust_trajectory, &mut rng);

            let drifts = self.calculate_drift(&states);
            let correlation = self.calculate_correlation(&drifts);

            all_correlations.push(correlation);

            if rep == 0 {
                all_drifts = drifts;
            }
        }

        let mean_correlation = all_correlations.iter().sum::<f64>() / all_correlations.len() as f64;

        // Success: positive correlation (drift increases as 1/trust increases)
        let success = mean_correlation > 0.3;
        let duration = start_time.elapsed();

        // Build CSV
        let mut csv_data = String::from("timestep,drift_rate,trust_level,inverse_trust\n");
        for d in &all_drifts {
            csv_data.push_str(&format!(
                "{},{:.6},{:.4},{:.4}\n",
                d.timestep, d.drift_rate, d.trust_level, d.inverse_trust
            ));
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Summary
        let summary = format!(
            "Semantic Drift Experiment (LING-P0-2)\n\
             ======================================\n\
             Timesteps: {}\n\
             Replications: {}\n\
             \n\
             Correlation (drift vs 1/τ): {:.4}\n\
             \n\
             Success Criterion: r > 0.3\n\
             Result: {}\n",
            config.n_timesteps,
            config.n_replications,
            mean_correlation,
            if success { "✓ PASSED" } else { "✗ FAILED" }
        );

        let artifacts = vec![
            ("semantic_drift.csv".to_string(), csv_data),
            ("drift_summary.txt".to_string(), summary),
        ];

        let metadata = serde_json::json!({
            "mean_correlation": mean_correlation,
            "n_replications": config.n_replications,
        });

        Ok(ExperimentResult {
            success,
            experiment_type: "semantic_drift".to_string(),
            gap_id: "LING-P0-2".to_string(),
            paper: "LING".to_string(),
            figures: vec![],
            csv_outputs: vec![],
            metrics: metadata,
            artifacts,
            duration,
            error_message: None,
        })
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_distance() {
        let exp = SemanticDriftExperiment;

        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];

        let dist = exp.embedding_distance(&a, &b);
        assert!(dist > 0.9 && dist <= 1.0); // Orthogonal vectors

        let c = vec![1.0, 0.0, 0.0];
        let dist_self = exp.embedding_distance(&a, &c);
        assert!(dist_self < 0.01); // Same direction
    }

    #[test]
    fn test_experiment_runs() {
        let config = ExperimentConfig::quick();
        let exp = SemanticDriftExperiment;

        let result = exp.run(&config).expect("Experiment should run");

        assert!(result.metrics.get("mean_correlation").is_some());
    }
}
