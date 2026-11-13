//! Crisis Threshold Experiment (ECON-P0-2)
//!
//! Validates: "Crisis threshold unvalidated - τ < 0.68 → crisis with 85% accuracy"
//!
//! Gap Analysis:
//! - Paper claims: Trust below 0.68 predicts economic crisis
//! - Missing: Historical crisis prediction validation
//! - Missing: Threshold calibration
//!
//! Methodology:
//! 1. Simulate economic systems with varying trust
//! 2. Track crisis events
//! 3. Test τ = 0.68 as crisis predictor
//! 4. Validate 85% accuracy claim

use crate::experiments::{Experiment, ExperimentConfig, ExperimentResult};
use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Instant;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Crisis threshold experiment for ECON-P0-2
#[derive(Debug)]
pub struct CrisisThresholdExperiment;

/// Economic system state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicState {
    /// timestep.
    pub timestep: usize,
    /// trust level.
    pub trust_level: f64,
    /// gdp.
    pub gdp: f64,
    /// crisis.
    pub crisis: bool,
    /// investment.
    pub investment: f64,
    /// consumption.
    pub consumption: f64,
}

/// Crisis prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrisisPrediction {
    /// trust below threshold.
    pub trust_below_threshold: bool,
    /// crisis occurred.
    pub crisis_occurred: bool,
    /// true positive.
    pub true_positive: bool,
    /// false positive.
    pub false_positive: bool,
    /// true negative.
    pub true_negative: bool,
    /// false negative.
    pub false_negative: bool,
}

impl CrisisThresholdExperiment {
    const CRISIS_THRESHOLD: f64 = 0.68;

    /// Simulate economic dynamics
    fn simulate_economy(
        &self,
        n_steps: usize,
        base_trust: f64,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        rng: &mut impl Rng,
    ) -> Vec<EconomicState> {
        let mut states = Vec::with_capacity(n_steps);
        let mut trust = base_trust;
        let mut gdp = 100.0;
        let mut crisis_countdown = 0;

        for step in 0..n_steps {
            // Crisis dynamics
            let in_crisis = crisis_countdown > 0;

            if in_crisis {
                crisis_countdown -= 1;
                gdp *= 0.95; // GDP declines during crisis
            } else {
                // Normal growth with trust effect
                let growth = 0.02 * trust;
                gdp *= 1.0 + growth;
            }

            // Crisis trigger: low trust + random shock
            if !in_crisis && trust < 0.5 && rng.gen::<f64>() < 0.1 {
                crisis_countdown = rng.gen_range(5..15);
            }

            // Trust evolution
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            if in_crisis {
                trust *= 0.95; // Trust erodes during crisis
            } else {
                trust += rng.gen_range(-0.02..0.01);
            }
            trust = trust.clamp(0.1, 1.0);

            let investment = gdp * 0.2 * trust;
            let consumption = gdp * 0.6;

            states.push(EconomicState {
                timestep: step,
                trust_level: trust,
                gdp,
                crisis: in_crisis,
                investment,
                consumption,
            });
        }

        states
    }

    /// Evaluate crisis prediction at threshold
    fn evaluate_predictions(&self, states: &[EconomicState]) -> Vec<CrisisPrediction> {
        let mut predictions = Vec::new();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Look ahead window for crisis
        let window = 5;

        for i in 0..states.len().saturating_sub(window) {
            let trust_below = states[i].trust_level < Self::CRISIS_THRESHOLD;

            // Crisis in next window steps?
            let crisis_occurred = states[i..(i + window).min(states.len())]
                .iter()
                .any(|s| s.crisis);

            predictions.push(CrisisPrediction {
                trust_below_threshold: trust_below,
                crisis_occurred,
                true_positive: trust_below && crisis_occurred,
                false_positive: trust_below && !crisis_occurred,
                true_negative: !trust_below && !crisis_occurred,
                false_negative: !trust_below && crisis_occurred,
            });
        }

        predictions
    }

    /// Calculate accuracy metrics
    fn calculate_accuracy(&self, predictions: &[CrisisPrediction]) -> (f64, f64, f64) {
        let n = predictions.len() as f64;
        if n == 0.0 {
            return (0.0, 0.0, 0.0);
        }

        let tp = predictions.iter().filter(|p| p.true_positive).count() as f64;
        let fp = predictions.iter().filter(|p| p.false_positive).count() as f64;
        let tn = predictions.iter().filter(|p| p.true_negative).count() as f64;
        let fn_count = predictions.iter().filter(|p| p.false_negative).count() as f64;

        let accuracy = (tp + tn) / n;
        let precision = if tp + fp > 0.0 { tp / (tp + fp) } else { 0.0 };
        let recall = if tp + fn_count > 0.0 {
            tp / (tp + fn_count)
        } else {
            0.0
        };

        (accuracy, precision, recall)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Experiment for CrisisThresholdExperiment {
    fn name(&self) -> &'static str {
        "crisis_threshold"
    }

    fn description(&self) -> &'static str {
        "ECON-P0-2: Crisis threshold validation (τ < 0.68 → crisis, 85% acc)"
    }

    fn required_agents(&self) -> usize {
        100
    }

    fn run(&self, config: &ExperimentConfig) -> Result<ExperimentResult> {
        let start_time = Instant::now();
        let mut rng = config.rng();

        // Run multiple economies with different initial trust
        let mut all_predictions = Vec::new();
        let mut all_states = Vec::new();

        for rep in 0..config.n_replications {
            let initial_trust = rng.gen_range(0.3..0.9);

            let states = self.simulate_economy(config.n_timesteps, initial_trust, &mut rng);

            let predictions = self.evaluate_predictions(&states);
            all_predictions.extend(predictions);

            // TAG: surface=simulation owner=research-team rule=SIM-001
            if rep == 0 {
                all_states = states;
            }
        }

        let (accuracy, precision, recall) = self.calculate_accuracy(&all_predictions);

        // Success: accuracy >= 85%
        let success = accuracy >= 0.85;
        let duration = start_time.elapsed();

        // Build CSV
        let mut csv_data = String::from("timestep,trust,gdp,crisis,investment,consumption\n");
        for s in &all_states {
            csv_data.push_str(&format!(
                "{},{:.4},{:.2},{},{:.2},{:.2}\n",
                s.timestep,
                s.trust_level,
                s.gdp,
                i32::from(s.crisis),
                s.investment,
                s.consumption
            ));
        }

        // Summary
        let summary = format!(
            "Crisis Threshold Experiment (ECON-P0-2)\n\
             ========================================\n\
             Crisis Threshold: τ < {:.2}\n\
             Simulations: {}\n\
             Total Predictions: {}\n\
             \n\
             Accuracy: {:.2}%\n\
             Precision: {:.2}%\n\
             Recall: {:.2}%\n\
             \n\
             // TAG: surface=simulation owner=research-team rule=SIM-001
             Success Criterion: Accuracy ≥ 85%\n\
             Result: {}\n",
            Self::CRISIS_THRESHOLD,
            config.n_replications,
            all_predictions.len(),
            accuracy * 100.0,
            precision * 100.0,
            recall * 100.0,
            if success { "✓ PASSED" } else { "✗ FAILED" }
        );

        let artifacts = vec![
            ("crisis_simulation.csv".to_string(), csv_data),
            ("crisis_summary.txt".to_string(), summary),
        ];

        let metadata = serde_json::json!({
            "accuracy": accuracy,
            "precision": precision,
            "recall": recall,
            "threshold": Self::CRISIS_THRESHOLD,
            "n_predictions": all_predictions.len(),
        });

        Ok(ExperimentResult {
            success,
            experiment_type: "crisis_threshold".to_string(),
            gap_id: "ECON-P0-2".to_string(),
            paper: "ECONOMICS".to_string(),
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
    fn test_crisis_prediction() {
        let exp = CrisisThresholdExperiment;

        // Need at least window + 1 states (window = 5)
        let states: Vec<EconomicState> = (0..10)
            .map(|i| EconomicState {
                timestep: i,
                trust_level: if i == 5 { 0.4 } else { 0.7 },
                gdp: 100.0,
                crisis: i == 6,
                investment: 10.0,
                consumption: 60.0,
            })
            .collect();

        let predictions = exp.evaluate_predictions(&states);
        assert!(!predictions.is_empty());
    }

    #[test]
    fn test_experiment_runs() {
        let config = ExperimentConfig::quick();
        let exp = CrisisThresholdExperiment;

        let result = exp.run(&config).expect("Experiment should run");

        assert!(result.metrics.get("accuracy").is_some());
    }
}
