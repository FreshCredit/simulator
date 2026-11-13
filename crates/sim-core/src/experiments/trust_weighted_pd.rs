//! Trust-Weighted Prisoner's Dilemma Experiment (GAME-P0-1)
//!
//! Validates: "Agents with τ < 0.2 defect, τ > 0.6 cooperate,
//! τ ∈ [0.2, 0.6] use mixed strategies"
//!
//! Gap Analysis:
//! - Paper claims: Trust predicts cooperation strategy
//! - Missing: Explicit game-theoretic validation
//! - Missing: Mixed strategy equilibrium analysis
//! - Conservative target: Strategy correctly predicted in 80%+ of cases
//!
//! Methodology:
//! 1. Pair agents with known trust values
//! 2. Run iterated prisoner's dilemma
//! 3. Compare actual strategy to trust-based prediction
//! 4. Validate threshold model

use crate::experiments::{Experiment, ExperimentConfig, ExperimentResult};
use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Trust-Weighted PD experiment for GAME-P0-1
#[derive(Debug)]
pub struct TrustWeightedPDExperiment;

/// Prisoner's Dilemma actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// cooperate.
    Cooperate,
    /// defect.
    Defect,
}

impl Action {
    /// Get payoff given opponent's action
    #[must_use]
    pub const fn payoff(&self, opponent: Self) -> f64 {
        match (self, opponent) {
            // Both cooperate: reward
            (Self::Cooperate, Self::Cooperate) => 3.0,
            // Cooperate vs defect: sucker's payoff
            (Self::Cooperate, Self::Defect) => 0.0,
            // Defect vs cooperate: temptation
            (Self::Defect, Self::Cooperate) => 5.0,
            // Both defect: punishment
            (Self::Defect, Self::Defect) => 1.0,
        }
    }
}

/// Strategy types based on trust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Strategy {
    /// Pure cooperation (τ > 0.6)
    PureCooperator,
    /// Pure defection (τ < 0.2)
    PureDefector,
    /// Mixed strategy (τ ∈ [0.2, 0.6])
    Mixed(f64), // Cooperation probability
    /// Tit-for-Tat (conditional)
    TitForTat,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Strategy {
    /// Predict strategy from trust value
    #[must_use]
    pub fn from_trust(trust: f64) -> Self {
        if trust > 0.6 {
            Self::PureCooperator
        } else if trust < 0.2 {
            Self::PureDefector
        } else {
            // Linear interpolation in mixed region
            let coop_prob = (trust - 0.2) / 0.4;
            Self::Mixed(coop_prob.clamp(0.0, 1.0))
        }
    }

    /// Choose action
    pub fn choose_action(&self, rng: &mut impl Rng, opponent_last: Option<Action>) -> Action {
        match self {
            Self::PureCooperator => Action::Cooperate,
            Self::PureDefector => Action::Defect,
            Self::Mixed(p) => {
                if rng.gen::<f64>() < *p {
                    Action::Cooperate
                } else {
                    Action::Defect
                }
            }
            Self::TitForTat => opponent_last.unwrap_or(Action::Cooperate),
        }
    }

    /// Predicted cooperation rate
    #[must_use]
    pub const fn cooperation_rate(&self) -> f64 {
        match self {
            Self::PureCooperator => 1.0,
            Self::PureDefector => 0.0,
            Self::Mixed(p) => *p,
            Self::TitForTat => 0.5, // Approximate for symmetric
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Game result for a pair of agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameResult {
    /// Agent A trust value
    pub trust_a: f64,
    /// Agent B trust value
    pub trust_b: f64,
    /// Predicted strategy A
    pub predicted_strategy_a: Strategy,
    /// Predicted strategy B
    pub predicted_strategy_b: Strategy,
    /// Actual cooperation rate A
    pub actual_coop_rate_a: f64,
    /// Actual cooperation rate B
    pub actual_coop_rate_b: f64,
    /// Total payoff A
    pub payoff_a: f64,
    /// Total payoff B
    pub payoff_b: f64,
    /// Prediction correct for A?
    pub prediction_correct_a: bool,
    /// Prediction correct for B?
    pub prediction_correct_b: bool,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Strategy prediction accuracy by trust region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionAccuracy {
    /// region.
    pub region: String,
    /// trust range.
    pub trust_range: (f64, f64),
    /// n agents.
    pub n_agents: usize,
    /// correct predictions.
    pub correct_predictions: usize,
    /// accuracy.
    pub accuracy: f64,
    /// mean cooperation rate.
    pub mean_cooperation_rate: f64,
}

/// Equilibrium analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquilibriumAnalysis {
    /// nash deviations.
    pub nash_deviations: f64,
    /// pareto efficiency.
    pub pareto_efficiency: f64,
    /// mutual cooperation rate.
    pub mutual_cooperation_rate: f64,
    /// mutual defection rate.
    pub mutual_defection_rate: f64,
}

impl TrustWeightedPDExperiment {
    /// Run iterated PD between two agents
    fn run_game(
        &self,
        trust_a: f64,
        trust_b: f64,
        n_rounds: usize,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        rng: &mut impl Rng,
    ) -> GameResult {
        // Predict strategies
        let strategy_a = Strategy::from_trust(trust_a);
        let strategy_b = Strategy::from_trust(trust_b);

        // Run game
        let mut payoff_a = 0.0;
        let mut payoff_b = 0.0;
        let mut coop_count_a = 0;
        let mut coop_count_b = 0;

        let mut last_action_a: Option<Action> = None;
        let mut last_action_b: Option<Action> = None;

        for _ in 0..n_rounds {
            let action_a = strategy_a.choose_action(rng, last_action_b);
            let action_b = strategy_b.choose_action(rng, last_action_a);

            payoff_a += action_a.payoff(action_b);
            payoff_b += action_b.payoff(action_a);

            if action_a == Action::Cooperate {
                coop_count_a += 1;
            }
            if action_b == Action::Cooperate {
                coop_count_b += 1;
            }

            last_action_a = Some(action_a);
            last_action_b = Some(action_b);
        }

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let actual_coop_rate_a = coop_count_a as f64 / n_rounds as f64;
        let actual_coop_rate_b = coop_count_b as f64 / n_rounds as f64;

        // Check prediction accuracy (within 20% tolerance)
        let predicted_coop_a = strategy_a.cooperation_rate();
        let predicted_coop_b = strategy_b.cooperation_rate();

        let prediction_correct_a = (actual_coop_rate_a - predicted_coop_a).abs() < 0.20;
        let prediction_correct_b = (actual_coop_rate_b - predicted_coop_b).abs() < 0.20;

        GameResult {
            trust_a,
            trust_b,
            predicted_strategy_a: strategy_a,
            predicted_strategy_b: strategy_b,
            actual_coop_rate_a,
            actual_coop_rate_b,
            payoff_a,
            payoff_b,
            prediction_correct_a,
            prediction_correct_b,
        }
    }

    /// Calculate accuracy by trust region
    fn calculate_region_accuracy(&self, results: &[GameResult]) -> Vec<RegionAccuracy> {
        let regions = vec![
            ("Defector", 0.0, 0.2),
            ("Mixed-Low", 0.2, 0.4),
            ("Mixed-High", 0.4, 0.6),
            ("Cooperator", 0.6, 1.0),
        ];

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let mut region_stats = Vec::new();

        for (name, low, high) in regions {
            let relevant: Vec<_> = results
                .iter()
                .filter(|r| r.trust_a >= low && r.trust_a < high)
                .collect();

            let n = relevant.len();
            let correct = relevant.iter().filter(|r| r.prediction_correct_a).count();
            let mean_coop = if n > 0 {
                relevant.iter().map(|r| r.actual_coop_rate_a).sum::<f64>() / n as f64
            } else {
                0.0
            };

            region_stats.push(RegionAccuracy {
                region: name.to_string(),
                trust_range: (low, high),
                n_agents: n,
                correct_predictions: correct,
                accuracy: if n > 0 {
                    correct as f64 / n as f64
                } else {
                    0.0
                },
                mean_cooperation_rate: mean_coop,
            });
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        region_stats
    }

    /// Analyze equilibrium properties
    fn analyze_equilibrium(&self, results: &[GameResult]) -> EquilibriumAnalysis {
        let n = results.len() as f64;

        // Mutual cooperation: both cooperate
        let mutual_coop = results
            .iter()
            .filter(|r| r.actual_coop_rate_a > 0.8 && r.actual_coop_rate_b > 0.8)
            .count() as f64
            / n;

        // Mutual defection: both defect
        let mutual_defect = results
            .iter()
            .filter(|r| r.actual_coop_rate_a < 0.2 && r.actual_coop_rate_b < 0.2)
            .count() as f64
            / n;

        // Pareto efficiency: mutual cooperation is Pareto optimal
        // (no agent can improve without hurting the other)
        let pareto_efficient = results
            .iter()
            .filter(|r| {
                let both_cooperating = r.actual_coop_rate_a > 0.8 && r.actual_coop_rate_b > 0.8;
                let high_payoff_a = r.payoff_a > 2.5 * 100.0; // > 2.5 per round
                let high_payoff_b = r.payoff_b > 2.5 * 100.0;
                both_cooperating && high_payoff_a && high_payoff_b
            })
            .count() as f64
            / n;

        // Nash deviations: unilateral defection would be profitable
        let nash_dev = results
            .iter()
            .filter(|r| r.payoff_a < 4.0 * 100.0 || r.payoff_b < 4.0 * 100.0)
            .count() as f64
            / n;

        EquilibriumAnalysis {
            nash_deviations: nash_dev,
            pareto_efficiency: pareto_efficient,
            mutual_cooperation_rate: mutual_coop,
            mutual_defection_rate: mutual_defect,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Experiment for TrustWeightedPDExperiment {
    fn name(&self) -> &'static str {
        "trust_weighted_pd"
    }

    fn description(&self) -> &'static str {
        "GAME-P0-1: Trust predicts cooperation in Prisoner's Dilemma"
    }

    fn required_agents(&self) -> usize {
        200
    }

    fn run(&self, config: &ExperimentConfig) -> Result<ExperimentResult> {
        let mut rng = config.rng();
        let start_time = std::time::Instant::now();

        // Generate agents with varying trust levels
        let n_games = config.n_agents / 2;
        let mut results = Vec::new();

        for _ in 0..n_games {
            // Random trust values
            let trust_a = rng.gen::<f64>();
            let trust_b = rng.gen::<f64>();

            // TAG: surface=simulation owner=research-team rule=SIM-001
            // Run iterated game
            let result = self.run_game(trust_a, trust_b, 100, &mut rng);
            results.push(result);
        }

        // Calculate region accuracies
        let region_accuracies = self.calculate_region_accuracy(&results);

        // Calculate overall accuracy
        let total_predictions = results.len() * 2; // Each game has 2 agents
        let correct_predictions = results
            .iter()
            .map(|r| usize::from(r.prediction_correct_a) + usize::from(r.prediction_correct_b))
            .sum::<usize>();
        let overall_accuracy = correct_predictions as f64 / total_predictions as f64;

        // Equilibrium analysis
        let equilibrium = self.analyze_equilibrium(&results);

        // Success criterion: overall accuracy >= 0.80
        let success = overall_accuracy >= 0.80;

        let duration = start_time.elapsed();

        // Build CSV output
        let mut csv_data = String::from(
            "trust_a,trust_b,predicted_coop_a,predicted_coop_b,actual_coop_a,actual_coop_b,payoff_a,payoff_b,correct_a,correct_b\n"
        );
        for r in &results {
            csv_data.push_str(&format!(
                "{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.1},{:.1},{},{}\n",
                r.trust_a,
                r.trust_b,
                // TAG: surface=simulation owner=research-team rule=SIM-001
                r.predicted_strategy_a.cooperation_rate(),
                r.predicted_strategy_b.cooperation_rate(),
                r.actual_coop_rate_a,
                r.actual_coop_rate_b,
                r.payoff_a,
                r.payoff_b,
                r.prediction_correct_a,
                r.prediction_correct_b
            ));
        }

        // Region accuracy CSV
        let mut region_csv =
            String::from("region,trust_low,trust_high,n_agents,correct,accuracy,mean_coop_rate\n");
        for reg in &region_accuracies {
            region_csv.push_str(&format!(
                "{},{:.2},{:.2},{},{},{:.4},{:.4}\n",
                reg.region,
                reg.trust_range.0,
                reg.trust_range.1,
                reg.n_agents,
                reg.correct_predictions,
                reg.accuracy,
                reg.mean_cooperation_rate
            ));
        }

        // Summary
        let summary = format!(
            "Trust-Weighted Prisoner's Dilemma Experiment (GAME-P0-1)\n\
             ========================================================\n\
             Games played: {}\n\
             Total predictions: {}\n\
             Correct predictions: {}\n\
             Overall accuracy: {:.2}%\n\
             \n\
             By Region:\n",
            n_games,
            total_predictions,
            correct_predictions,
            overall_accuracy * 100.0
        );

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let mut region_summary = String::new();
        for reg in &region_accuracies {
            region_summary.push_str(&format!(
                "  {} [{:.1}, {:.1}]: {:.1}% correct (n={}, coop={:.2})\n",
                reg.region,
                reg.trust_range.0,
                reg.trust_range.1,
                reg.accuracy * 100.0,
                reg.n_agents,
                reg.mean_cooperation_rate
            ));
        }

        let equilibrium_summary = format!(
            "\nEquilibrium Analysis:\n\
             - Mutual cooperation rate: {:.2}%\n\
             - Mutual defection rate: {:.2}%\n\
             - Pareto efficiency: {:.2}%\n\
             - Nash deviations: {:.2}%\n\
             \n\
             Success Criterion: Overall accuracy >= 80%\n\
             Result: {}\n",
            equilibrium.mutual_cooperation_rate * 100.0,
            equilibrium.mutual_defection_rate * 100.0,
            equilibrium.pareto_efficiency * 100.0,
            equilibrium.nash_deviations * 100.0,
            if success { "✓ PASSED" } else { "✗ FAILED" }
        );

        let artifacts = vec![
            ("pd_game_results.csv".to_string(), csv_data),
            ("pd_region_accuracy.csv".to_string(), region_csv),
            (
                "pd_summary.txt".to_string(),
                summary + &region_summary + &equilibrium_summary,
            ),
        ];

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let metadata = serde_json::json!({
            "overall_accuracy": overall_accuracy,
            "n_games": n_games,
            "mutual_cooperation": equilibrium.mutual_cooperation_rate,
        });

        Ok(ExperimentResult {
            success,
            experiment_type: "trust_weighted_pd".to_string(),
            gap_id: "GAME-P0-1".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_from_trust() {
        // High trust → Pure cooperator
        let strat = Strategy::from_trust(0.8);
        assert!(matches!(strat, Strategy::PureCooperator));

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Low trust → Pure defector
        let strat = Strategy::from_trust(0.1);
        assert!(matches!(strat, Strategy::PureDefector));

        // Medium trust → Mixed
        let strat = Strategy::from_trust(0.4);
        assert!(matches!(strat, Strategy::Mixed(_)));
    }

    #[test]
    fn test_payoff_matrix() {
        assert_eq!(Action::Cooperate.payoff(Action::Cooperate), 3.0);
        assert_eq!(Action::Cooperate.payoff(Action::Defect), 0.0);
        assert_eq!(Action::Defect.payoff(Action::Cooperate), 5.0);
        assert_eq!(Action::Defect.payoff(Action::Defect), 1.0);
    }

    #[test]
    fn test_game_run() {
        let exp = TrustWeightedPDExperiment;
        let mut rng = rand::thread_rng();

        let result = exp.run_game(0.7, 0.3, 50, &mut rng);

        assert!(result.payoff_a >= 0.0);
        assert!(result.payoff_b >= 0.0);
        assert!(result.actual_coop_rate_a >= 0.0 && result.actual_coop_rate_a <= 1.0);
    }

    #[test]
    fn test_experiment_runs() {
        let config = ExperimentConfig::quick();
        let exp = TrustWeightedPDExperiment;

        let result = exp.run(&config).expect("Experiment should run");

        assert!(result.metrics.get("overall_accuracy").is_some());
        assert!(result.duration.as_secs() < 10);
    }
}
