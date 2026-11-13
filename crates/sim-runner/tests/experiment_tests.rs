// TAG: surface=simulation owner=research-team rule=SIM-001
//! Integration tests for `sim-runner::experiment`
//!
//! Extracted from src/experiment.rs to reduce per-file CCN.
// Integration tests extracted from crates/sim-runner/src/experiment.rs

#[cfg(test)]
mod tests {
    use sim_runner::*;

    #[test]
    fn test_experiment_type_defaults() {
        let exp = ExperimentType::LambdaUniversality;
        assert_eq!(exp.paper(), "GTT");
        assert!(exp.default_replications() > 0);
        assert!(exp.default_agent_count() > 0);
    }

    #[test]
    fn test_config_creation() {
        let config = ExperimentConfig::for_experiment(ExperimentType::LambdaUniversality);
        assert_eq!(config.experiment_type.paper(), "GTT");
        assert!(!config.acceptance_criteria.is_empty());
    }

    #[test]
    fn test_acceptance_criterion_range() {
        let crit = AcceptanceCriterion::in_range("test", 0.5, 0.7, "test criterion");
        assert!(crit.expected_range.0 <= crit.expected_range.1);
        assert_eq!(crit.comparison, Comparison::InRange);
    }
}
