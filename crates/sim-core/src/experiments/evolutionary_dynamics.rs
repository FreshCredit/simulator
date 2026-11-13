//! Evolutionary Dynamics Experiment (GAME-P0-3)
//!
//! Validates: "Evolutionary dynamics of trust-weighted strategies missing"
//!
//! Gap Analysis:
//! - Paper claims: Trust-based strategies evolve to equilibrium
//! - Missing: Replicator dynamics simulation
//! - Missing: Evolutionary stable strategy (ESS) identification
//!
//! Methodology:
//! 1. Initialize population with mixed strategies
//! 2. Apply replicator dynamics
//! 3. Track strategy frequencies over time
//! 4. Identify equilibrium

use crate::experiments::{Experiment, ExperimentConfig, ExperimentResult};
use anyhow::Result;

use serde::{Deserialize, Serialize};
use std::time::Instant;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Evolutionary dynamics experiment for GAME-P0-3
#[derive(Debug)]
pub struct EvolutionaryDynamicsExperiment;

/// Strategy types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Strategy {
    /// cooperator.
    Cooperator, // Always cooperate
    /// defector.
    Defector, // Always defect
    /// tit for tat.
    TitForTat, // Copy opponent's last move
    /// grim trigger.
    GrimTrigger, // Cooperate until defected, then always defect
    /// trust based.
    TrustBased, // Cooperate if trust > 0.5
}

impl Strategy {
    /// Get base payoff against other strategies
    const fn payoff_against(&self, other: Self) -> f64 {
        match (self, other) {
            // Mutual cooperation
            (Self::Cooperator, Self::Cooperator) => 3.0,
            (Self::TitForTat, Self::TitForTat) => 3.0,
            (Self::GrimTrigger, Self::GrimTrigger) => 3.0,
            (Self::TrustBased, Self::TrustBased) => 3.0,

            // Mutual defection
            (Self::Defector, Self::Defector) => 1.0,

            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            // Exploitation
            (Self::Defector, Self::Cooperator) => 5.0,
            (Self::Cooperator, Self::Defector) => 0.0,

            // TFT vs others (simplified)
            (Self::TitForTat, Self::Cooperator) => 3.0,
            (Self::Cooperator, Self::TitForTat) => 3.0,
            (Self::TitForTat, Self::Defector) => 1.0, // Gets exploited once
            (Self::Defector, Self::TitForTat) => 4.0, // Exploits once

            // Grim Trigger (simplified)
            (Self::GrimTrigger, Self::Cooperator) => 3.0,
            (Self::Cooperator, Self::GrimTrigger) => 3.0,
            (Self::GrimTrigger, Self::Defector) => 0.5, // Defected once
            (Self::Defector, Self::GrimTrigger) => 4.5, // Exploits once

            // Trust-based
            (Self::TrustBased, Self::Cooperator) => 3.0,
            (Self::Cooperator, Self::TrustBased) => 3.0,
            (Self::TrustBased, Self::Defector) => 1.0,
            (Self::Defector, Self::TrustBased) => 4.0,

            // Mixed
            (Self::TitForTat, Self::GrimTrigger) => 3.0,
            (Self::GrimTrigger, Self::TitForTat) => 3.0,
            (Self::TitForTat, Self::TrustBased) => 3.0,
            (Self::TrustBased, Self::TitForTat) => 3.0,
            (Self::GrimTrigger, Self::TrustBased) => 3.0,
            (Self::TrustBased, Self::GrimTrigger) => 3.0,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Population state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationState {
    /// generation.
    pub generation: usize,
    /// frequencies.
    pub frequencies: std::collections::HashMap<Strategy, f64>,
    /// avg payoff.
    pub avg_payoff: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl EvolutionaryDynamicsExperiment {
    /// Calculate average payoff for each strategy
    fn calculate_fitness(
        &self,
        frequencies: &std::collections::HashMap<Strategy, f64>,
    ) -> std::collections::HashMap<Strategy, f64> {
        let mut fitness = std::collections::HashMap::new();

        for (&strat, &freq) in frequencies {
            if freq < 0.001 {
                fitness.insert(strat, 0.0);
                continue;
            }

            let mut payoff = 0.0;
            for (&other, &other_freq) in frequencies {
                payoff += strat.payoff_against(other) * other_freq;
            }
            fitness.insert(strat, payoff);
        }

        fitness
    }

    /// Apply replicator dynamics
    fn evolve(
        &self,
        frequencies: &std::collections::HashMap<Strategy, f64>,
        dt: f64,
    ) -> std::collections::HashMap<Strategy, f64> {
        let fitness = self.calculate_fitness(frequencies);

        // Average fitness
        let avg_fitness: f64 = frequencies
            .iter()
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            .map(|(s, f)| fitness.get(s).unwrap_or(&0.0) * f)
            .sum();

        // Replicator equation: dx_i/dt = x_i * (f_i - avg_f)
        let mut new_freq = std::collections::HashMap::new();
        for (&strat, &freq) in frequencies {
            let fit = fitness.get(&strat).unwrap_or(&0.0);
            let change = freq * (fit - avg_fitness) * dt;
            new_freq.insert(strat, (freq + change).max(0.0));
        }

        // Normalize
        let total: f64 = new_freq.values().sum();
        if total > 0.0 {
            new_freq = new_freq.into_iter().map(|(k, v)| (k, v / total)).collect();
        }

        new_freq
    }

    /// Check for equilibrium (small changes)
    fn is_equilibrium(
        &self,
        prev: &std::collections::HashMap<Strategy, f64>,
        curr: &std::collections::HashMap<Strategy, f64>,
        threshold: f64,
    ) -> bool {
        for (strat, freq) in curr {
            let prev_freq = prev.get(strat).unwrap_or(&0.0);
            if (freq - prev_freq).abs() > threshold {
                return false;
            }
        }
        true
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Experiment for EvolutionaryDynamicsExperiment {
    fn name(&self) -> &'static str {
        "evolutionary_dynamics"
    }

    fn description(&self) -> &'static str {
        "GAME-P0-3: Evolutionary dynamics of trust-weighted strategies"
    }

    fn required_agents(&self) -> usize {
        1000
    }

    fn run(&self, config: &ExperimentConfig) -> Result<ExperimentResult> {
        let start_time = Instant::now();

        // Initialize with uniform distribution
        let strategies = [
            Strategy::Cooperator,
            Strategy::Defector,
            Strategy::TitForTat,
            Strategy::GrimTrigger,
            Strategy::TrustBased,
        ];

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let initial_freq = 1.0 / strategies.len() as f64;
        let mut frequencies: std::collections::HashMap<Strategy, f64> =
            strategies.iter().map(|&s| (s, initial_freq)).collect();

        let mut history = Vec::new();
        let dt = 0.1;
        let mut equilibrium_reached = false;
        let mut generations_to_equilibrium = 0;

        for gen in 0..config.n_timesteps {
            let fitness = self.calculate_fitness(&frequencies);
            let avg_payoff: f64 = frequencies
                .iter()
                .map(|(s, f)| fitness.get(s).unwrap_or(&0.0) * f)
                .sum();

            history.push(PopulationState {
                generation: gen,
                frequencies: frequencies.clone(),
                avg_payoff,
            });

            let new_freq = self.evolve(&frequencies, dt);

            if !equilibrium_reached && self.is_equilibrium(&frequencies, &new_freq, 0.001) {
                equilibrium_reached = true;
                generations_to_equilibrium = gen;
            }

            // TAG: surface=simulation owner=research-team rule=SIM-001
            frequencies = new_freq;
        }

        // Final state
        let final_state = history
            .last()
            .ok_or_else(|| anyhow::anyhow!("No history recorded"))?;
        let tft_freq = final_state
            .frequencies
            .get(&Strategy::TitForTat)
            .copied()
            .unwrap_or(0.0);
        let trust_freq = final_state
            .frequencies
            .get(&Strategy::TrustBased)
            .copied()
            .unwrap_or(0.0);
        let coop_freq = tft_freq + trust_freq;

        // Success: Cooperative strategies dominate
        let success = coop_freq > 0.5;
        let duration = start_time.elapsed();

        // Build CSV
        let mut csv_data = String::from(
            "generation,cooperator,defector,titfortat,grimtrigger,trustbased,avg_payoff\n",
        );
        for state in &history {
            let get_freq = |s| state.frequencies.get(&s).copied().unwrap_or(0.0);
            csv_data.push_str(&format!(
                "{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}\n",
                state.generation,
                get_freq(Strategy::Cooperator),
                get_freq(Strategy::Defector),
                get_freq(Strategy::TitForTat),
                get_freq(Strategy::GrimTrigger),
                get_freq(Strategy::TrustBased),
                state.avg_payoff
            ));
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Summary
        let summary = format!(
            "Evolutionary Dynamics Experiment (GAME-P0-3)\n\
             ============================================\n\
             Generations: {}\n\
             Equilibrium Reached: {} (gen {})\n\
             \n\
             Final Frequencies:\n\
             - Cooperative (TFT + Trust): {:.2}%\n\
             - Tit-for-Tat: {:.2}%\n\
             - Trust-Based: {:.2}%\n\
             - Defector: {:.2}%\n\
             - Cooperator (pure): {:.2}%\n\
             - Grim Trigger: {:.2}%\n\
             \n\
             Success Criterion: Cooperative strategies > 50%\n\
             Result: {}\n",
            config.n_timesteps,
            if equilibrium_reached { "Yes" } else { "No" },
            generations_to_equilibrium,
            coop_freq * 100.0,
            tft_freq * 100.0,
            trust_freq * 100.0,
            final_state
                .frequencies
                .get(&Strategy::Defector)
                .copied()
                .unwrap_or(0.0)
                * 100.0,
            final_state
                .frequencies
                // TAG: surface=simulation owner=research-team rule=SIM-001
                .get(&Strategy::Cooperator)
                .copied()
                .unwrap_or(0.0)
                * 100.0,
            final_state
                .frequencies
                .get(&Strategy::GrimTrigger)
                .copied()
                .unwrap_or(0.0)
                * 100.0,
            if success { "✓ PASSED" } else { "✗ FAILED" }
        );

        let artifacts = vec![
            ("evolution.csv".to_string(), csv_data),
            ("evolution_summary.txt".to_string(), summary),
        ];

        let metadata = serde_json::json!({
            "cooperative_frequency": coop_freq,
            "titfortat_frequency": tft_freq,
            "trust_based_frequency": trust_freq,
            "equilibrium_reached": equilibrium_reached,
            "generations_to_equilibrium": generations_to_equilibrium,
        });

        Ok(ExperimentResult {
            success,
            experiment_type: "evolutionary_dynamics".to_string(),
            gap_id: "GAME-P0-3".to_string(),
            paper: "GAME".to_string(),
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
    fn test_evolve() {
        let exp = EvolutionaryDynamicsExperiment;

        let mut freq = std::collections::HashMap::new();
        freq.insert(Strategy::Cooperator, 0.2);
        freq.insert(Strategy::Defector, 0.2);
        freq.insert(Strategy::TitForTat, 0.6);

        let new_freq = exp.evolve(&freq, 0.1);

        let total: f64 = new_freq.values().sum();
        assert!((total - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_experiment_runs() {
        let config = ExperimentConfig::quick();
        let exp = EvolutionaryDynamicsExperiment;

        let result = exp.run(&config).expect("Experiment should run");

        assert!(result.metrics.get("cooperative_frequency").is_some());
    }
}
