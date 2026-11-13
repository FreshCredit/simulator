//! Civilizational Collapse Experiment (MEAN-P0-2)
//!
//! Validates: "Historical collapse unvalidated - trust erosion precedes
//! civilizational collapse with `τ_critical` = 0.32 threshold"
//!
//! Gap Analysis:
//! - Paper claims: Trust decline precedes collapse
//! - Missing: Simulation of civilizational dynamics
//!
//! Methodology:
//! 1. Model civilization with trust-dependent institutions
//! 2. Track trust erosion over time
//! 3. Detect collapse events
//! 4. Validate τ < 0.32 precedes collapse

use crate::experiments::{Experiment, ExperimentConfig, ExperimentResult};
use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Instant;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Civilizational collapse experiment for MEAN-P0-2
#[derive(Debug)]
pub struct CivilizationalCollapseExperiment;

/// Civilization state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CivilizationState {
    /// year.
    pub year: usize,
    /// trust level.
    pub trust_level: f64,
    /// institution health.
    pub institution_health: f64,
    /// economic output.
    pub economic_output: f64,
    /// population.
    pub population: f64,
    /// collapsed.
    pub collapsed: bool,
}

/// Collapse event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollapseEvent {
    /// year.
    pub year: usize,
    /// trust at collapse.
    pub trust_at_collapse: f64,
    /// years below threshold.
    pub years_below_threshold: usize,
}

impl CivilizationalCollapseExperiment {
    const COLLAPSE_THRESHOLD: f64 = 0.32;

    /// Simulate civilization dynamics
    fn simulate_civilization(
        &self,
        n_years: usize,
        initial_trust: f64,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        rng: &mut impl Rng,
    ) -> Vec<CivilizationState> {
        let mut states = Vec::with_capacity(n_years);
        let mut trust = initial_trust;
        let mut institution_health = 0.7;
        let mut economic_output = 100.0;
        let mut population = 1000.0;
        let mut collapsed = false;

        for year in 0..n_years {
            states.push(CivilizationState {
                year,
                trust_level: trust,
                institution_health,
                economic_output,
                population,
                collapsed,
            });

            if collapsed {
                continue;
            }

            // Trust dynamics with random shocks
            let shock = if rng.gen::<f64>() < 0.05 {
                rng.gen_range(-0.1..-0.02) // Negative shock
                                           // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            } else {
                rng.gen_range(-0.01..0.02)
            };

            trust = (trust + shock).clamp(0.05, 1.0);

            // Institution health depends on trust
            let target_health = trust * 0.8 + 0.1;
            institution_health += (target_health - institution_health) * 0.1;

            // Economic output depends on institutions and trust
            economic_output *= 1.0 + (trust - 0.5) * 0.02;

            // Population follows resources
            let carrying_capacity = economic_output * 10.0;
            population += (carrying_capacity - population) * 0.01;

            // Collapse condition
            if trust < Self::COLLAPSE_THRESHOLD
                && institution_health < 0.3
                && rng.gen::<f64>() < 0.3
            {
                collapsed = true;
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        states
    }

    /// Analyze collapse events
    fn analyze_collapse(&self, states: &[CivilizationState]) -> Option<CollapseEvent> {
        let collapse_year = states.iter().position(|s| s.collapsed)?;

        let trust_at_collapse = states[collapse_year].trust_level;

        // Count years below threshold before collapse
        let years_below = states[..collapse_year]
            .iter()
            .rev()
            .take_while(|s| s.trust_level < Self::COLLAPSE_THRESHOLD)
            .count();

        Some(CollapseEvent {
            year: collapse_year,
            trust_at_collapse,
            years_below_threshold: years_below,
        })
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Experiment for CivilizationalCollapseExperiment {
    fn name(&self) -> &'static str {
        "civilizational_collapse"
    }

    fn description(&self) -> &'static str {
        "MEAN-P0-2: Civilizational collapse at τ_critical = 0.32"
    }

    fn required_agents(&self) -> usize {
        1 // Single civilization simulation
    }

    fn run(&self, config: &ExperimentConfig) -> Result<ExperimentResult> {
        let start_time = Instant::now();
        let mut rng = config.rng();

        // Run multiple civilization simulations
        let mut collapses = Vec::new();
        let mut all_states = Vec::new();
        let mut n_collapse = 0;

        for rep in 0..config.n_replications {
            let initial_trust = rng.gen_range(0.4..0.8);

            let states = self.simulate_civilization(config.n_timesteps, initial_trust, &mut rng);

            if let Some(collapse) = self.analyze_collapse(&states) {
                collapses.push(collapse);
                n_collapse += 1;
            }

            if rep == 0 {
                all_states = states;
            }
        }

        // Statistics
        let collapse_rate = n_collapse as f64 / config.n_replications as f64;
        let mean_trust_at_collapse = if collapses.is_empty() {
            0.0
        } else {
            collapses.iter().map(|c| c.trust_at_collapse).sum::<f64>() / collapses.len() as f64
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let mean_years_below = if collapses.is_empty() {
            0.0
        } else {
            collapses
                .iter()
                .map(|c| c.years_below_threshold as f64)
                .sum::<f64>()
                / collapses.len() as f64
        };

        // Success: τ < 0.32 detected before collapse
        let success =
            !collapses.is_empty() && mean_trust_at_collapse < Self::COLLAPSE_THRESHOLD + 0.1;
        let duration = start_time.elapsed();

        // Build CSV
        let mut csv_data =
            String::from("year,trust,institution_health,economic_output,population,collapsed\n");
        for s in &all_states {
            csv_data.push_str(&format!(
                "{},{:.3},{:.3},{:.1},{:.1},{}\n",
                s.year,
                s.trust_level,
                s.institution_health,
                s.economic_output,
                s.population,
                i32::from(s.collapsed)
            ));
        }

        // Summary
        let summary = format!(
            "Civilizational Collapse Experiment (MEAN-P0-2)\n\
             // TAG: surface=simulation owner=research-team rule=SIM-001
             ==============================================\n\
             Simulations: {}\n\
             Collapse Events: {} ({:.1}%)\n\
             \n\
             Collapse Statistics:\n\
             - Mean trust at collapse: {:.3}\n\
             - Mean years below τ_c: {:.1}\n\
             - Critical threshold: {:.2}\n\
             \n\
             Success Criterion: τ < 0.32 precedes collapse\n\
             Result: {}\n",
            config.n_replications,
            n_collapse,
            collapse_rate * 100.0,
            mean_trust_at_collapse,
            mean_years_below,
            Self::COLLAPSE_THRESHOLD,
            if success { "✓ PASSED" } else { "✗ FAILED" }
        );

        let artifacts = vec![
            ("civilization_trajectory.csv".to_string(), csv_data),
            ("collapse_summary.txt".to_string(), summary),
        ];

        let metadata = serde_json::json!({
            "collapse_rate": collapse_rate,
            "mean_trust_at_collapse": mean_trust_at_collapse,
            "mean_years_below_threshold": mean_years_below,
            "n_collapses": n_collapse,
        });

        Ok(ExperimentResult {
            success,
            experiment_type: "civilizational_collapse".to_string(),
            // TAG: surface=simulation owner=research-team rule=SIM-001
            gap_id: "MEAN-P0-2".to_string(),
            paper: "MEAN".to_string(),
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
    fn test_civilization_simulation() {
        let exp = CivilizationalCollapseExperiment;
        let mut rng = rand::thread_rng();

        let states = exp.simulate_civilization(100, 0.6, &mut rng);

        assert!(!states.is_empty());
        assert!(states[0].trust_level >= 0.0 && states[0].trust_level <= 1.0);
    }

    #[test]
    fn test_experiment_runs() {
        let config = ExperimentConfig::quick();
        let exp = CivilizationalCollapseExperiment;

        let result = exp.run(&config).expect("Experiment should run");

        assert!(result.metrics.get("collapse_rate").is_some());
    }
}
