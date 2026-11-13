//! Trust-Weighted Ultimatum Game Experiment
//!
//! Validates GAME-P0-1 extension: Trust-weighted expected utility in ultimatum game.
//! Tests whether Ψ-Trust (τ) predicts proposer offers and responder acceptance thresholds.
//!
//! ## Hypothesis
//! - H1: Higher τ proposers make fairer offers (closer to 50/50)
//! - H2: Higher τ responders accept lower offers (more trust in fairness)
//! - H3: τ predicts rejection of unfair offers (< 20%)
//!
//! ## References
//! - Güth et al. (1982): Original ultimatum game
//! - Nowak et al. (2000): Fairness in evolutionary context
//! - GTT Paper: Ψ-Trust as decision weight

use crate::experiments::{Experiment, ExperimentConfig, ExperimentResult};
use anyhow::Result;
use serde_json::json;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Trust-Weighted Ultimatum Game Experiment
///
/// GAME-P0-1 extension: Validates trust-weighted expected utility
/// in ultimatum bargaining context.
#[derive(Debug)]
pub struct TrustWeightedUltimatumExperiment;

impl TrustWeightedUltimatumExperiment {
    /// Calculate trust-weighted offer
    ///
    /// Proposers with higher τ make fairer offers (closer to 50/50)
    fn calculate_offer(proposer_tau: f64, endowment: f64) -> f64 {
        let fair_offer = 0.5 * endowment;
        let selfish_offer = 0.15 * endowment;
        let weight = proposer_tau.clamp(0.0, 1.0);
        selfish_offer + weight * (fair_offer - selfish_offer)
    }

    /// Calculate acceptance threshold for responder
    ///
    /// Higher τ responders have lower thresholds (more trusting)
    fn calculate_threshold(responder_tau: f64, endowment: f64) -> f64 {
        let max_threshold = 0.4 * endowment;
        let min_threshold = 0.1 * endowment;
        let weight = 1.0 - responder_tau.clamp(0.0, 1.0);
        min_threshold + weight * (max_threshold - min_threshold)
    }

    /// Run single ultimatum game
    fn run_game(proposer_tau: f64, responder_tau: f64, endowment: f64) -> (f64, bool, f64) {
        let offer = Self::calculate_offer(proposer_tau, endowment);
        let threshold = Self::calculate_threshold(responder_tau, endowment);
        let offer_fraction = offer / endowment;
        let threshold_fraction = threshold / endowment;
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let accepted = offer >= threshold;

        (offer_fraction, accepted, threshold_fraction)
    }

    /// Calculate Pearson correlation coefficient
    fn pearson_correlation(x: &[f64], y: &[f64]) -> Option<f64> {
        crate::stats::pearson_r_f64(x, y)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Experiment for TrustWeightedUltimatumExperiment {
    fn name(&self) -> &'static str {
        "trust_weighted_ultimatum"
    }

    fn description(&self) -> &'static str {
        "Trust-weighted ultimatum game behavior (GAME-P0-1 extension)"
    }

    fn required_agents(&self) -> usize {
        200
    }

    fn run(&self, config: &ExperimentConfig) -> Result<ExperimentResult> {
        let n_pairs = config.n_agents / 2;
        let endowment = 100.0;
        let n_rounds = 10;

        let mut outcomes = Vec::new();

        // Run games with agents of varying trust levels
        for _ in 0..n_pairs {
            let proposer_tau: f64 = (0.5 + 0.15 * rand::random::<f64>()).clamp(0.0, 1.0);
            let responder_tau: f64 = (0.5 + 0.15 * rand::random::<f64>()).clamp(0.0, 1.0);

            for _ in 0..n_rounds {
                let (offer_frac, accepted, threshold_frac) =
                    Self::run_game(proposer_tau, responder_tau, endowment);
                outcomes.push((
                    proposer_tau,
                    responder_tau,
                    offer_frac,
                    accepted,
                    threshold_frac,
                ));
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Calculate statistics
        let high_tau_offers: Vec<f64> = outcomes
            .iter()
            .filter(|(p_tau, _, _, _, _)| *p_tau > 0.6)
            .map(|(_, _, offer, _, _)| *offer)
            .collect();

        let low_tau_offers: Vec<f64> = outcomes
            .iter()
            .filter(|(p_tau, _, _, _, _)| *p_tau < 0.3)
            .map(|(_, _, offer, _, _)| *offer)
            .collect();

        let low_offers_high_tau: Vec<bool> = outcomes
            .iter()
            .filter(|(_, r_tau, offer, _, _)| *offer < 0.2 && *r_tau > 0.6)
            .map(|(_, _, _, accepted, _)| *accepted)
            .collect();

        let low_offers_low_tau: Vec<bool> = outcomes
            .iter()
            .filter(|(_, r_tau, offer, _, _)| *offer < 0.2 && *r_tau < 0.3)
            .map(|(_, _, _, accepted, _)| *accepted)
            .collect();

        let taus: Vec<f64> = outcomes.iter().map(|(t, _, _, _, _)| *t).collect();
        let offers: Vec<f64> = outcomes.iter().map(|(_, _, o, _, _)| *o).collect();
        let tau_offer_correlation = Self::pearson_correlation(&taus, &offers).unwrap_or(0.0);

        let responder_taus: Vec<f64> = outcomes.iter().map(|(_, t, _, _, _)| *t).collect();
        let thresholds: Vec<f64> = outcomes.iter().map(|(_, _, _, _, th)| *th).collect();
        let tau_threshold_correlation =
            Self::pearson_correlation(&responder_taus, &thresholds).unwrap_or(0.0);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let high_tau_offer_mean = if high_tau_offers.is_empty() {
            0.0
        } else {
            high_tau_offers.iter().sum::<f64>() / high_tau_offers.len() as f64
        };

        let low_tau_offer_mean = if low_tau_offers.is_empty() {
            0.0
        } else {
            low_tau_offers.iter().sum::<f64>() / low_tau_offers.len() as f64
        };

        let high_tau_accept_low = if low_offers_high_tau.is_empty() {
            0.0
        } else {
            low_offers_high_tau.iter().filter(|&&b| b).count() as f64
                / low_offers_high_tau.len() as f64
        };

        let low_tau_accept_low = if low_offers_low_tau.is_empty() {
            0.0
        } else {
            low_offers_low_tau.iter().filter(|&&b| b).count() as f64
                / low_offers_low_tau.len() as f64
        };

        // Validate success criteria
        let criteria_passed = [
            high_tau_offer_mean > low_tau_offer_mean,
            high_tau_offer_mean > 0.40,
            high_tau_accept_low > low_tau_accept_low,
            tau_offer_correlation > 0.3,
            tau_threshold_correlation < -0.2,
        ]
        .iter()
        .filter(|&&c| c)
        .count();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let success = criteria_passed >= 4; // Allow 1 failure

        let metrics = json!({
            "high_tau_offer_mean": high_tau_offer_mean,
            "low_tau_offer_mean": low_tau_offer_mean,
            "high_tau_accept_low": high_tau_accept_low,
            "low_tau_accept_low": low_tau_accept_low,
            "tau_offer_correlation": tau_offer_correlation,
            "tau_threshold_correlation": tau_threshold_correlation,
            "n_outcomes": outcomes.len(),
            "criteria_passed": criteria_passed,
            "criteria_total": 5,
        });

        Ok(ExperimentResult {
            experiment_type: "trust_weighted_ultimatum".to_string(),
            success,
            gap_id: "GAME-P0-1-EXT".to_string(),
            paper: "GAME".to_string(),
            metrics,
            figures: vec![],
            csv_outputs: vec![],
            artifacts: vec![],
            duration: std::time::Duration::from_secs(0),
            error_message: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offer_calculation() {
        let high_tau_offer = TrustWeightedUltimatumExperiment::calculate_offer(0.9, 100.0);
        let low_tau_offer = TrustWeightedUltimatumExperiment::calculate_offer(0.1, 100.0);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(high_tau_offer > low_tau_offer);
        assert!(high_tau_offer > 40.0);
        assert!(low_tau_offer < 25.0);
    }

    #[test]
    fn test_threshold_calculation() {
        let high_tau_thresh = TrustWeightedUltimatumExperiment::calculate_threshold(0.9, 100.0);
        let low_tau_thresh = TrustWeightedUltimatumExperiment::calculate_threshold(0.1, 100.0);

        assert!(high_tau_thresh < low_tau_thresh);
    }

    #[test]
    fn test_game_outcomes() {
        let (offer_frac, accepted, _) = TrustWeightedUltimatumExperiment::run_game(0.9, 0.1, 100.0);
        assert!(offer_frac >= 0.4);
        assert!(accepted);

        let (offer_frac, accepted, _) = TrustWeightedUltimatumExperiment::run_game(0.1, 0.1, 100.0);
        assert!(offer_frac < 0.25);
        assert!(!accepted);
    }

    #[test]
    fn test_experiment_run() {
        let exp = TrustWeightedUltimatumExperiment;
        let config = ExperimentConfig::quick();
        let result = exp.run(&config).unwrap();

        assert!(!result.experiment_type.is_empty());
        assert_eq!(result.gap_id, "GAME-P0-1-EXT");
    }
}
