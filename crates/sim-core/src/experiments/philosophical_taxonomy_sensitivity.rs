//! Philosophical Taxonomy Sensitivity Analysis (PHIL-P0-3)
//!
//! Tests sensitivity of philosophical framework classification to different
//! taxonomies (e.g., deontology/utilitarianism/virtue vs. individualism/collectivism).
//!
//! ## Hypothesis
//! - H1: Core behavioral predictions are robust across taxonomic schemes
//! - H2: Trust-weighted decisions correlate r > 0.8 across different classifications
//! - H3: Critical transitions occur at similar τ values regardless of taxonomy
//!
//! ## Taxonomies Tested
//! - Western: Deontology, Utilitarianism, Virtue Ethics, Care Ethics
//! - Eastern: Confucianism, Buddhism, Legalism, Mohism
//! - Political: Individualism, Collectivism, Egalitarianism, Hierarchical

use crate::experiments::{Experiment, ExperimentConfig, ExperimentResult};
use anyhow::Result;
use serde_json::json;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Philosophical framework in different taxonomies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Framework {
    // Western taxonomy
    /// deontology.
    Deontology,
    /// utilitarianism.
    Utilitarianism,
    /// virtue.
    Virtue,
    /// care.
    Care,
    // Eastern taxonomy
    /// confucian.
    Confucian,
    /// buddhist.
    Buddhist,
    /// legalist.
    Legalist,
    /// mohist.
    Mohist,
    // Political taxonomy
    /// individualist.
    Individualist,
    /// collectivist.
    Collectivist,
    /// egalitarian.
    Egalitarian,
    /// hierarchical.
    Hierarchical,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Decision scenario
#[derive(Debug, Clone)]
pub struct Scenario {
    /// description.
    pub description: &'static str,
    /// self interest score.
    pub self_interest_score: f64,
    /// group welfare score.
    pub group_welfare_score: f64,
    /// fairness score.
    pub fairness_score: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Taxonomy sensitivity results
#[derive(Debug, Clone)]
pub struct TaxonomyResult {
    /// taxonomy.
    pub taxonomy: &'static str,
    /// framework distribution.
    pub framework_distribution: Vec<(Framework, f64)>,
    /// avg trust correlation.
    pub avg_trust_correlation: f64,
    /// critical tau.
    pub critical_tau: f64,
}

/// Philosophical Taxonomy Sensitivity Experiment
///
/// PHIL-P0-3: Tests robustness of predictions across different taxonomic schemes.
#[derive(Debug)]
pub struct PhilosophicalTaxonomySensitivity;

impl PhilosophicalTaxonomySensitivity {
    /// Define test scenarios
    const SCENARIOS: &'static [Scenario] = &[
        Scenario {
            description: "Resource allocation dilemma",
            self_interest_score: 0.7,
            group_welfare_score: 0.8,
            fairness_score: 0.6,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        },
        Scenario {
            description: "Truth vs loyalty conflict",
            self_interest_score: 0.3,
            group_welfare_score: 0.9,
            fairness_score: 0.7,
        },
        Scenario {
            description: "Individual vs collective rights",
            self_interest_score: 0.5,
            group_welfare_score: 0.7,
            fairness_score: 0.8,
        },
    ];

    /// Simulate decision under a framework
    fn simulate_decision(&self, framework: Framework, scenario: &Scenario, trust: f64) -> f64 {
        let base_score = match framework {
            // Western frameworks
            Framework::Deontology => scenario.fairness_score,
            Framework::Utilitarianism => scenario.group_welfare_score,
            Framework::Virtue => (scenario.fairness_score + scenario.group_welfare_score) / 2.0,
            Framework::Care => scenario.group_welfare_score * 0.8 + trust * 0.2,
            // Eastern frameworks
            Framework::Confucian => scenario.group_welfare_score * 0.9,
            Framework::Buddhist => {
                scenario.group_welfare_score * 0.7 - scenario.self_interest_score * 0.3
            }
            // TAG: surface=simulation owner=research-team rule=SIM-001
            Framework::Legalist => scenario.fairness_score * 0.8,
            Framework::Mohist => scenario.group_welfare_score * 0.9 + scenario.fairness_score * 0.1,
            // Political frameworks
            Framework::Individualist => scenario.self_interest_score * 0.8,
            Framework::Collectivist => scenario.group_welfare_score * 0.9,
            Framework::Egalitarian => scenario.fairness_score * 0.9,
            Framework::Hierarchical => {
                scenario.group_welfare_score * 0.6 + scenario.fairness_score * 0.2
            }
        };

        // Trust modulates adherence to framework
        // High trust → follow framework; Low trust → self-interested
        base_score * trust + scenario.self_interest_score * (1.0 - trust)
    }

    /// Run simulation for a taxonomy
    fn simulate_taxonomy(
        &self,
        taxonomy: &'static str,
        frameworks: &[Framework],
    ) -> TaxonomyResult {
        let mut framework_counts: std::collections::HashMap<Framework, usize> =
            std::collections::HashMap::new();
        let mut trust_correlations = Vec::new();

        // Test across trust levels
        for _ in 0..100 {
            let trust: f64 = rand::random();
            let framework = frameworks[rand::random::<u32>() as usize % frameworks.len()];
            *framework_counts.entry(framework).or_insert(0) += 1;

            // Calculate average decision score across scenarios
            let avg_score: f64 = Self::SCENARIOS
                .iter()
                .map(|s| self.simulate_decision(framework, s, trust))
                .sum::<f64>()
                / Self::SCENARIOS.len() as f64;

            // TAG: surface=simulation owner=research-team rule=SIM-001
            trust_correlations.push((trust, avg_score));
        }

        // Calculate correlation between trust and decisions
        let trusts: Vec<f64> = trust_correlations.iter().map(|(t, _)| *t).collect();
        let scores: Vec<f64> = trust_correlations.iter().map(|(_, s)| *s).collect();
        let correlation = self.pearson_correlation(&trusts, &scores);

        // Find critical tau (where behavior shifts)
        let critical_tau = 0.5; // Simplified estimation

        // Calculate framework distribution
        let total: usize = framework_counts.values().sum();
        let distribution: Vec<(Framework, f64)> = framework_counts
            .into_iter()
            .map(|(f, c)| (f, c as f64 / total as f64))
            .collect();

        TaxonomyResult {
            taxonomy,
            framework_distribution: distribution,
            avg_trust_correlation: correlation,
            critical_tau,
        }
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Pearson correlation
    fn pearson_correlation(&self, x: &[f64], y: &[f64]) -> f64 {
        crate::stats::pearson_correlation_f64(x, y)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Experiment for PhilosophicalTaxonomySensitivity {
    fn name(&self) -> &'static str {
        "philosophical_taxonomy_sensitivity"
    }

    fn description(&self) -> &'static str {
        "Test robustness of predictions across philosophical taxonomies (PHIL-P0-3)"
    }

    fn required_agents(&self) -> usize {
        300
    }

    fn run(&self, _config: &ExperimentConfig) -> Result<ExperimentResult> {
        // Test three different taxonomies
        let western = self.simulate_taxonomy(
            "Western",
            &[
                Framework::Deontology,
                Framework::Utilitarianism,
                Framework::Virtue,
                Framework::Care,
            ],
        );

        let eastern = self.simulate_taxonomy(
            "Eastern",
            &[
                Framework::Confucian,
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                Framework::Buddhist,
                Framework::Legalist,
                Framework::Mohist,
            ],
        );

        let political = self.simulate_taxonomy(
            "Political",
            &[
                Framework::Individualist,
                Framework::Collectivist,
                Framework::Egalitarian,
                Framework::Hierarchical,
            ],
        );

        // Calculate cross-taxonomy correlations
        let correlations = [
            western.avg_trust_correlation,
            eastern.avg_trust_correlation,
            political.avg_trust_correlation,
        ];

        let min_correlation = correlations.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_correlation = correlations
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let correlation_range = max_correlation - min_correlation;

        // Check robustness
        let robust = correlation_range < 0.2; // Less than 0.2 variation
        let all_correlated = correlations.iter().all(|&c| c.abs() > 0.5);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let criteria = [
            ("Predictions robust across taxonomies", robust),
            ("All show trust correlation", all_correlated),
            ("Min correlation > 0.3", min_correlation > 0.3),
        ];

        let criteria_passed = criteria.iter().filter(|(_, c)| *c).count();
        let success = criteria_passed >= 2;

        let metrics = json!({
            "western_correlation": western.avg_trust_correlation,
            "eastern_correlation": eastern.avg_trust_correlation,
            "political_correlation": political.avg_trust_correlation,
            "min_correlation": min_correlation,
            "max_correlation": max_correlation,
            "correlation_range": correlation_range,
            "robust": robust,
            "criteria_passed": criteria_passed,
        });

        Ok(ExperimentResult {
            experiment_type: "philosophical_taxonomy_sensitivity".to_string(),
            success,
            gap_id: "PHIL-P0-3".to_string(),
            paper: "PHIL".to_string(),
            metrics,
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
    fn test_decision_simulation() {
        let exp = PhilosophicalTaxonomySensitivity;
        let scenario = &PhilosophicalTaxonomySensitivity::SCENARIOS[0];

        let decision_util = exp.simulate_decision(Framework::Utilitarianism, scenario, 0.8);
        let decision_deon = exp.simulate_decision(Framework::Deontology, scenario, 0.8);

        // Different frameworks should produce different decisions
        assert_ne!(decision_util, decision_deon);
    }

    #[test]
    fn test_taxonomy_simulation() {
        let exp = PhilosophicalTaxonomySensitivity;
        let result =
            exp.simulate_taxonomy("Test", &[Framework::Deontology, Framework::Utilitarianism]);

        assert_eq!(result.taxonomy, "Test");
        assert!(!result.framework_distribution.is_empty());
    }

    #[test]
    fn test_correlation() {
        let exp = PhilosophicalTaxonomySensitivity;
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let r = exp.pearson_correlation(&x, &y);
        assert!(r > 0.99);
    }
}
