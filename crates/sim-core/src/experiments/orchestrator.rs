//! Experiment Orchestrator
//!
//! Runs all P0 gap validation experiments and generates summary report

use super::auc_validation::AucValidationExperiment;
use super::bond_mechanism_formalization::BondMechanismFormalization;
// TAG: surface=simulation owner=research-team rule=SIM-001
use super::civilizational_collapse::CivilizationalCollapseExperiment;
use super::civilizational_scale_protocol::CivilizationalScaleProtocol;
use super::cost_reduction::CostReductionExperiment;
use super::crisis_threshold::CrisisThresholdExperiment;
use super::criticality_sigma::CriticalitySigmaExperiment;
use super::cross_cultural_validation::CrossCulturalValidation;
use super::cross_domain_correlation::CrossDomainCorrelationExperiment;
use super::cultural_quadrants::CulturalQuadrantsExperiment;
use super::docas_emotion_mapping::DOCASEmotionMapping;
use super::economic_model_comparison::EconomicModelComparison;
use super::emotion_mapping::EmotionMappingExperiment;
use super::emotion_pharmacology::EmotionPharmacologyExperiment;
use super::evolutionary_dynamics::EvolutionaryDynamicsExperiment;
use super::gdp_prediction::GdpPredictionExperiment;
use super::gene_scoring_rubric::GeneScoringRubric;
use super::genomic_sem_specification::GenomicSEMSpecification;
use super::historical_collapse_validation::HistoricalCollapseValidation;
use super::hofstede_prediction::HofstedePredictionExperiment;
use super::lambda_universality::LambdaUniversalityExperiment;
use super::music_auc_claim::MusicAucClaim;
use super::music_trust_correlation::MusicTrustCorrelationExperiment;
use super::pde4b_behavioral_trust::PDE4BBehavioralTrust;
use super::pde4b_eeg_criticality::PDE4BEegCriticality;
use super::pde4b_genetic_analysis::Pde4bGeneticAnalysisExperiment;
use super::percolation_threshold::PercolationThresholdExperiment;
use super::philosophical_oscillation::PhilosophicalOscillationExperiment;
use super::philosophical_taxonomy_sensitivity::PhilosophicalTaxonomySensitivity;
use super::semantic_drift::SemanticDriftExperiment;
use super::semantic_network::SemanticNetworkExperiment;
use super::spectral_coherence::SpectralCoherenceExperiment;
use super::stability_law::StabilityLawExperiment;
use super::tau_critical_measurement::TauCriticalMeasurement;
use super::tau_phi_measurement::TauPhiMeasurement;
use super::trust_measurement_specification::TrustMeasurementSpecification;
use super::trust_weighted_pd::TrustWeightedPDExperiment;
use super::trust_weighted_ultimatum::TrustWeightedUltimatumExperiment;
use super::twin_heritability::TwinHeritabilityExperiment;
use super::{Experiment, ExperimentConfig, ExperimentResult, ExperimentType};

use std::collections::HashMap;
use std::time::Instant;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Experiment registry
pub struct ExperimentOrchestrator {
    experiments: HashMap<String, Box<dyn Experiment>>,
    results: Vec<ExperimentResult>,
}

impl std::fmt::Debug for ExperimentOrchestrator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExperimentOrchestrator")
            .field("results", &self.results)
            .finish_non_exhaustive()
    }
}

/// Summary of all experiments
#[derive(Debug)]
pub struct ExperimentSummary {
    /// total.
    pub total: usize,
    /// passed.
    pub passed: usize,
    /// failed.
    pub failed: usize,
    /// skipped.
    pub skipped: usize,
    /// total duration secs.
    pub total_duration_secs: f64,
    /// by paper.
    pub by_paper: HashMap<String, PaperSummary>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug)]
/// paper summary.
pub struct PaperSummary {
    /// name.
    pub name: String,
    /// total.
    pub total: usize,
    /// passed.
    pub passed: usize,
    /// gaps addressed.
    pub gaps_addressed: Vec<String>,
}

impl Default for ExperimentOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl ExperimentOrchestrator {
    #[must_use]
    /// new.
    pub fn new() -> Self {
        let mut experiments: HashMap<String, Box<dyn Experiment>> = HashMap::new();

        // Register all experiments
        experiments.insert(
            "lambda_universality".to_string(),
            Box::new(LambdaUniversalityExperiment::new()),
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        );
        experiments.insert(
            "twin_heritability".to_string(),
            Box::new(TwinHeritabilityExperiment::new(500, 500)),
        );
        experiments.insert(
            "criticality_sigma".to_string(),
            Box::new(CriticalitySigmaExperiment::new()),
        );
        experiments.insert(
            "auc_validation".to_string(),
            Box::new(AucValidationExperiment::new()),
        );
        experiments.insert(
            "emotion_mapping".to_string(),
            Box::new(EmotionMappingExperiment::new()),
        );
        experiments.insert(
            "cultural_quadrants".to_string(),
            Box::new(CulturalQuadrantsExperiment::new()),
        );
        experiments.insert(
            "gdp_prediction".to_string(),
            Box::new(GdpPredictionExperiment::new()),
        );
        experiments.insert(
            "semantic_network".to_string(),
            Box::new(SemanticNetworkExperiment),
        );
        experiments.insert(
            // TAG: surface=simulation owner=research-team rule=SIM-001
            "philosophical_oscillation".to_string(),
            Box::new(PhilosophicalOscillationExperiment),
        );
        experiments.insert(
            "trust_weighted_pd".to_string(),
            Box::new(TrustWeightedPDExperiment),
        );
        experiments.insert(
            "percolation_threshold".to_string(),
            Box::new(PercolationThresholdExperiment),
        );
        experiments.insert(
            "spectral_coherence".to_string(),
            Box::new(SpectralCoherenceExperiment),
        );
        experiments.insert(
            "stability_law".to_string(),
            Box::new(StabilityLawExperiment),
        );
        experiments.insert(
            "semantic_drift".to_string(),
            Box::new(SemanticDriftExperiment),
        );
        experiments.insert(
            "crisis_threshold".to_string(),
            Box::new(CrisisThresholdExperiment),
        );
        experiments.insert(
            "hofstede_prediction".to_string(),
            Box::new(HofstedePredictionExperiment),
        );
        experiments.insert(
            "evolutionary_dynamics".to_string(),
            Box::new(EvolutionaryDynamicsExperiment),
        );
        experiments.insert(
            "pde4b_genetic_analysis".to_string(),
            Box::new(Pde4bGeneticAnalysisExperiment),
        );
        // TAG: surface=simulation owner=research-team rule=SIM-001
        experiments.insert(
            "cross_domain_correlation".to_string(),
            Box::new(CrossDomainCorrelationExperiment),
        );
        experiments.insert(
            "cost_reduction".to_string(),
            Box::new(CostReductionExperiment),
        );
        experiments.insert(
            "emotion_pharmacology".to_string(),
            Box::new(EmotionPharmacologyExperiment),
        );
        experiments.insert(
            "civilizational_collapse".to_string(),
            Box::new(CivilizationalCollapseExperiment),
        );
        experiments.insert(
            "music_trust_correlation".to_string(),
            Box::new(MusicTrustCorrelationExperiment),
        );
        experiments.insert(
            "trust_weighted_ultimatum".to_string(),
            Box::new(TrustWeightedUltimatumExperiment),
        );
        experiments.insert(
            "tau_critical_measurement".to_string(),
            Box::new(TauCriticalMeasurement),
        );
        experiments.insert(
            "historical_collapse_validation".to_string(),
            Box::new(HistoricalCollapseValidation),
        );
        experiments.insert(
            "tau_phi_measurement".to_string(),
            Box::new(TauPhiMeasurement),
        );
        experiments.insert(
            "docas_emotion_mapping".to_string(),
            Box::new(DOCASEmotionMapping),
        );
        experiments.insert(
            "philosophical_taxonomy_sensitivity".to_string(),
            Box::new(PhilosophicalTaxonomySensitivity),
        );
        experiments.insert(
            "bond_mechanism_formalization".to_string(),
            Box::new(BondMechanismFormalization),
        );
        experiments.insert(
            "trust_measurement_specification".to_string(),
            Box::new(TrustMeasurementSpecification),
        );
        experiments.insert(
            "pde4b_behavioral_trust".to_string(),
            Box::new(PDE4BBehavioralTrust),
        );
        experiments.insert(
            "pde4b_eeg_criticality".to_string(),
            Box::new(PDE4BEegCriticality),
        );
        experiments.insert(
            "civilizational_scale_protocol".to_string(),
            Box::new(CivilizationalScaleProtocol),
        );
        experiments.insert(
            "cross_cultural_validation".to_string(),
            Box::new(CrossCulturalValidation),
        );
        experiments.insert(
            "gene_scoring_rubric".to_string(),
            Box::new(GeneScoringRubric),
        );
        experiments.insert(
            "genomic_sem_specification".to_string(),
            Box::new(GenomicSEMSpecification),
        );
        experiments.insert(
            "economic_model_comparison".to_string(),
            Box::new(EconomicModelComparison::new()),
        );
        experiments.insert(
            "music_auc_claim".to_string(),
            Box::new(MusicAucClaim::new()),
        );

        Self {
            experiments,
            results: Vec::new(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Run all experiments
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn run_all(&mut self, base_config: &ExperimentConfig) -> anyhow::Result<ExperimentSummary> {
        println!("========================================");
        println!("Running All P0 Gap Validation Experiments");
        println!("========================================");
        println!();

        let start_time = Instant::now();
        let mut passed = 0;
        let mut failed = 0;
        let mut by_paper: HashMap<String, PaperSummary> = HashMap::new();

        let experiment_order = vec![
            ("lambda_universality", "GTT", vec!["GTT-P0-1"]),
            ("twin_heritability", "DOCAS", vec!["DOCAS-P0-2"]),
            ("criticality_sigma", "DOCAS", vec!["DOCAS-P0-3"]),
            ("cultural_quadrants", "CULTURE", vec!["CULT-P0-1"]),
            ("gdp_prediction", "ECONOMICS", vec!["ECON-P0-1"]),
            ("semantic_network", "LING", vec!["LING-P0-1"]),
            ("philosophical_oscillation", "PHIL", vec!["PHIL-P0-2"]),
            ("trust_weighted_pd", "GAME", vec!["GAME-P0-1"]),
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            ("percolation_threshold", "MEAN", vec!["MEAN-P0-3"]),
            ("spectral_coherence", "MUSIC", vec!["MUSIC-P0-2"]),
            ("auc_validation", "TFP", vec!["TFP-P0-3", "TFP-P0-1"]),
            ("emotion_mapping", "EMOTION", vec!["EMOT-P0-1"]),
            ("stability_law", "GTT", vec!["GTT-P0-6"]),
            ("semantic_drift", "LING", vec!["LING-P0-2"]),
            ("crisis_threshold", "ECONOMICS", vec!["ECON-P0-2"]),
            ("hofstede_prediction", "CULTURE", vec!["CULT-P0-3"]),
            ("evolutionary_dynamics", "GAME", vec!["GAME-P0-3"]),
            ("pde4b_genetic_analysis", "DOCAS", vec!["DOCAS-P0-1"]),
            ("cross_domain_correlation", "GTT", vec!["GTT-P0-5"]),
            ("cost_reduction", "TFP", vec!["TFP-P0-4"]),
            ("emotion_pharmacology", "EMOTION", vec!["EMOT-P0-3"]),
            ("civilizational_collapse", "MEAN", vec!["MEAN-P0-2"]),
            ("music_trust_correlation", "MUSIC", vec!["MUSIC-P0-3"]),
            ("trust_weighted_ultimatum", "GAME", vec!["GAME-P0-1-ext"]),
            ("tau_critical_measurement", "MEAN", vec!["MEAN-P0-1"]),
            (
                "historical_collapse_validation",
                "CULTURE",
                vec!["CULT-P0-2", "MEAN-P0-2"],
            ),
            ("tau_phi_measurement", "PHIL", vec!["PHIL-P0-1"]),
            ("docas_emotion_mapping", "EMOTION", vec!["EMOT-P0-2"]),
            (
                "philosophical_taxonomy_sensitivity",
                "PHIL",
                vec!["PHIL-P0-3"],
            ),
            ("bond_mechanism_formalization", "PHIL", vec!["PHIL-P0-4"]),
            ("trust_measurement_specification", "GAME", vec!["GAME-P0-2"]),
            ("pde4b_behavioral_trust", "PDE4B", vec!["PDE4B-P0-1"]),
            ("pde4b_eeg_criticality", "PDE4B", vec!["PDE4B-P0-2"]),
            ("civilizational_scale_protocol", "MEAN", vec!["MEAN-P0-4"]),
            ("cross_cultural_validation", "LING", vec!["LING-P0-3"]),
            ("gene_scoring_rubric", "DOCAS", vec!["DOCAS-P0-4"]),
            ("genomic_sem_specification", "DOCAS", vec!["DOCAS-P0-5"]),
            ("economic_model_comparison", "ECONOMICS", vec!["ECON-P0-4"]),
            ("music_auc_claim", "MUSIC", vec!["MUSIC-P0-1"]),
        ];
        let total_experiments = experiment_order.len();

        for (name, paper, gaps) in experiment_order {
            print!("Running {name}... ");

            let config = ExperimentConfig {
                experiment_type: match name {
                    "lambda_universality" => ExperimentType::LambdaUniversality,
                    "twin_heritability" => ExperimentType::TwinHeritability {
                        n_mz: 500,
                        n_dz: 500,
                    },
                    "criticality_sigma" => ExperimentType::CriticalitySigma,
                    "auc_validation" => ExperimentType::PsiTrustAuc,
                    "emotion_mapping" => ExperimentType::EmotionMapping,
                    "cultural_quadrants" => ExperimentType::CulturalQuadrants,
                    "gdp_prediction" => ExperimentType::GdpPrediction,
                    "semantic_network" => ExperimentType::SemanticNetwork,
                    "philosophical_oscillation" => ExperimentType::PhilosophicalOscillation,
                    "trust_weighted_pd" => ExperimentType::TrustWeightedPd,
                    "percolation_threshold" => ExperimentType::PercolationThreshold,
                    "spectral_coherence" => ExperimentType::SpectralCoherence,
                    "stability_law" => ExperimentType::StabilityLaw,
                    "semantic_drift" => ExperimentType::SemanticDrift,
                    "crisis_threshold" => ExperimentType::CrisisThreshold,
                    "hofstede_prediction" => ExperimentType::HofstedePrediction,
                    "evolutionary_dynamics" => ExperimentType::EvolutionaryDynamics,
                    "pde4b_genetic_analysis" => ExperimentType::Pde4bGeneticAnalysis,
                    "cross_domain_correlation" => ExperimentType::CrossDomainCorrelation,
                    "cost_reduction" => ExperimentType::CostReduction,
                    "emotion_pharmacology" => ExperimentType::EmotionPharmacology,
                    "civilizational_collapse" => ExperimentType::CivilizationalCollapse,
                    "music_trust_correlation" => ExperimentType::MusicTrustCorrelation,
                    "trust_weighted_ultimatum" => ExperimentType::TrustWeightedUltimatum,
                    "tau_critical_measurement" => ExperimentType::TauCriticalMeasurement,
                    "historical_collapse_validation" => {
                        ExperimentType::HistoricalCollapseValidation
                    }
                    "tau_phi_measurement" => ExperimentType::TauPhiMeasurement,
                    "docas_emotion_mapping" => ExperimentType::DOCASEmotionMapping,
                    "philosophical_taxonomy_sensitivity" => {
                        ExperimentType::PhilosophicalTaxonomySensitivity
                    }
                    "bond_mechanism_formalization" => ExperimentType::BondMechanismFormalization,
                    "trust_measurement_specification" => {
                        ExperimentType::TrustMeasurementSpecification
                    }
                    "pde4b_behavioral_trust" => ExperimentType::PDE4BBehavioralTrust,
                    "pde4b_eeg_criticality" => ExperimentType::PDE4BEegCriticality,
                    "civilizational_scale_protocol" => ExperimentType::CivilizationalScaleProtocol,
                    "cross_cultural_validation" => ExperimentType::CrossCulturalValidation,
                    "gene_scoring_rubric" => ExperimentType::GeneScoringRubric,
                    "genomic_sem_specification" => ExperimentType::GenomicSEMSpecification,
                    "economic_model_comparison" => ExperimentType::EconomicModelComparison,
                    "music_auc_claim" => ExperimentType::MusicAUCClaim,
                    _ => ExperimentType::LambdaUniversality,
                },
                n_agents: base_config.n_agents,
                n_timesteps: base_config.n_timesteps,
                n_replications: base_config.n_replications,
                seed: base_config.seed,
                output_dir: format!("{}/{}", base_config.output_dir, name),
            };

            match self.experiments.get(name) {
                Some(experiment) => {
                    match experiment.run(&config) {
                        Ok(result) => {
                            let success = experiment.validate(&result);
                            if success {
                                println!("✅ PASSED");
                                passed += 1;
                            } else {
                                println!("❌ FAILED (criteria not met)");
                                failed += 1;
                            }

                            // Update paper summary
                            let paper_summary =
                                by_paper
                                    .entry(paper.to_string())
                                    .or_insert_with(|| PaperSummary {
                                        name: paper.to_string(),
                                        total: 0,
                                        passed: 0,
                                        gaps_addressed: Vec::new(),
                                    });
                            paper_summary.total += 1;
                            if success {
                                paper_summary.passed += 1;
                            }
                            paper_summary
                                .gaps_addressed
                                .extend(gaps.iter().map(std::string::ToString::to_string));

                            // TAG: surface=simulation owner=research-team rule=SIM-001
                            self.results.push(result);
                        }
                        Err(e) => {
                            println!("❌ ERROR: {e}");
                            failed += 1;
                        }
                    }
                }
                None => {
                    println!("⚠️  NOT FOUND");
                }
            }
        }

        let duration = start_time.elapsed().as_secs_f64();

        println!();
        println!("========================================");
        println!("Experiment Summary");
        println!("========================================");
        println!("Total: {total_experiments}");
        println!("Passed: {passed}");
        println!("Failed: {failed}");
        println!("Duration: {duration:.2}s");
        println!();

        println!("By Paper:");
        for (paper, summary) in &by_paper {
            println!("  {}: {}/{} passed", paper, summary.passed, summary.total);
            println!("    Gaps: {}", summary.gaps_addressed.join(", "));
        }

        Ok(ExperimentSummary {
            total: total_experiments,
            passed,
            failed,
            skipped: 0,
            total_duration_secs: duration,
            by_paper,
        })
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Run single experiment by name
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn run_one(
        &self,
        name: &str,
        config: &ExperimentConfig,
    ) -> anyhow::Result<ExperimentResult> {
        match self.experiments.get(name) {
            Some(experiment) => experiment.run(config),
            None => anyhow::bail!("Experiment '{name}' not found"),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// List all available experiments
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[must_use]
    pub fn list_experiments(&self) -> Vec<&str> {
        self.experiments
            .keys()
            .map(std::string::String::as_str)
            .collect()
    }

    /// Generate comprehensive report
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn generate_report(&self, output_path: &str) -> anyhow::Result<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(output_path)?;

        writeln!(file, "# P0 Gap Validation Experiment Report")?;
        writeln!(file)?;
        writeln!(
            file,
            "Date: {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        )?;
        writeln!(file)?;

        writeln!(file, "## Results Summary")?;
        writeln!(file)?;
        writeln!(file, "| Experiment | Status | Gap ID |")?;
        writeln!(file, "|------------|--------|--------|")?;

        for res in &self.results {
            let status = if res.success { "PASSED" } else { "FAILED" };
            writeln!(
                file,
                "| {:?} | {} | {:?} |",
                res.experiment_type, status, res.metrics
            )?;
        }

        Ok(())
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// CLI entry point for running all experiments
pub fn run_all_experiments_cli() {
    let config = ExperimentConfig {
        experiment_type: ExperimentType::LambdaUniversality,
        n_agents: 1000,
        n_timesteps: 365,
        n_replications: 100,
        seed: 42,
        output_dir: "./results".to_string(),
    };

    let mut orchestrator = ExperimentOrchestrator::new();

    match orchestrator.run_all(&config) {
        Ok(summary) => {
            println!();
            println!(
                "Overall Success Rate: {:.1}%",
                (summary.passed as f64 / summary.total as f64) * 100.0
            );

            if summary.failed == 0 {
                println!("🎉 All experiments passed!");
            } else {
                println!("⚠️  Some experiments failed. Check logs for details.");
                std::process::exit(1);
            }
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        }
        Err(e) => {
            tracing::error!(error = %e, "Error running experiments");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_creation() {
        let orchestrator = ExperimentOrchestrator::new();
        let experiments = orchestrator.list_experiments();

        assert!(!experiments.is_empty());
        assert!(experiments.contains(&"lambda_universality"));
    }

    #[test]
    fn test_orchestrator_registry_size() {
        let orchestrator = ExperimentOrchestrator::new();
        let experiments = orchestrator.list_experiments();
        assert_eq!(
            experiments.len(),
            39,
            "Expected exactly 39 experiments, got {}",
            experiments.len()
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_orchestrator_all_ids_unique() {
        let orchestrator = ExperimentOrchestrator::new();
        let experiments = orchestrator.list_experiments();
        let unique: std::collections::HashSet<_> = experiments.iter().collect();
        assert_eq!(unique.len(), experiments.len());
    }

    #[test]
    fn test_run_one_missing_experiment() {
        let orchestrator = ExperimentOrchestrator::new();
        let config = ExperimentConfig::quick();
        let result = orchestrator.run_one("nonexistent_experiment", &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_orchestrator_contains_key_experiments() {
        let orchestrator = ExperimentOrchestrator::new();
        let experiments = orchestrator.list_experiments();
        for key in ["lambda_universality", "stability_law", "cultural_quadrants"] {
            assert!(experiments.contains(&key), "Missing key experiment: {key}");
        }
    }
}
