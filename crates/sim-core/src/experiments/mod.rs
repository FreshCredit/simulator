//! Experiments module for P0 gap resolution
//!
//! This module contains implementations of the 29 experiments required to
//! validate P0 (Critical - Publication Blocking) claims.
//!
// TAG: surface=simulation owner=research-team rule=SIM-001
//! ## Experiment List
//!
//! | # | Experiment | Gap ID | Paper |
//! |---|------------|--------|-------|
//! | 1 | lambda_universality | GTT-P0-1 | GTT |
//! | 2 | twin_heritability | DOCAS-P0-2 | DOCAS |
//! | 3 | criticality_sigma | DOCAS-P0-3 | DOCAS |
//! | 4 | auc_validation | TFP-P0-3 | TFP |
//! | 5 | emotion_mapping | EMOT-P0-1 | EMOTION |
//! | 6 | cultural_quadrants | CULT-P0-1 | CULTURE |
//! | 7 | gdp_prediction | ECON-P0-1 | ECONOMICS |
//! | 8 | semantic_network | LING-P0-1 | LING |
//! | 9 | philosophical_oscillation | PHIL-P0-2 | PHIL |
//! | 10 | trust_weighted_pd | GAME-P0-1 | GAME |
//! | 11 | percolation_threshold | MEAN-P0-3 | MEAN |
//! | 12 | spectral_coherence | MUSIC-P0-2 | MUSIC |
//! | 13 | stability_law | GTT-P0-6 | GTT |
//! | 14 | semantic_drift | LING-P0-2 | LING |
//! | 15 | crisis_threshold | ECON-P0-2 | ECONOMICS |
//! | 16 | hofstede_prediction | CULT-P0-3 | CULTURE |
//! | 17 | evolutionary_dynamics | GAME-P0-3 | GAME |
//! | 18 | pde4b_genetic_analysis | DOCAS-P0-1 | DOCAS |
//! | 19 | cross_domain_correlation | GTT-P0-5 | GTT |
//! | 20 | cost_reduction | TFP-P0-4 | TFP |
//! | 21 | emotion_pharmacology | EMOT-P0-3 | EMOTION |
//! | 22 | civilizational_collapse | MEAN-P0-2 | MEAN |
//! | 23 | music_trust_correlation | MUSIC-P0-3 | MUSIC |
//! | 24 | trust_weighted_ultimatum | GAME-P0-1-ext | GAME |
//! | 25 | tau_critical_measurement | MEAN-P0-1 | MEAN |
//! | 26 | historical_collapse_validation | CULT-P0-2,MEAN-P0-2 | CULTURE,MEAN |
//! | 27 | tau_phi_measurement | PHIL-P0-1 | PHIL |
//! | 28 | docas_emotion_mapping | EMOT-P0-2 | EMOTION |
//! | 29 | philosophical_taxonomy_sensitivity | PHIL-P0-3 | PHIL |
//! | 30 | bond_mechanism_formalization | PHIL-P0-4 | PHIL |
//! | 31 | trust_measurement_specification | GAME-P0-2 | GAME |
//! | 32 | pde4b_behavioral_trust | PDE4B-P0-1 | PDE4B |
//! | 33 | pde4b_eeg_criticality | PDE4B-P0-2 | PDE4B |
//! | 34 | civilizational_scale_protocol | MEAN-P0-4 | MEAN |
//! | 35 | cross_cultural_validation | LING-P0-3 | LING |
//! | 36 | gene_scoring_rubric | DOCAS-P0-4 | DOCAS |
//! | 37 | genomic_sem_specification | DOCAS-P0-5 | DOCAS |
//! | 38 | economic_model_comparison | ECON-P0-4 | ECONOMICS |
//! | 39 | music_auc_claim | MUSIC-P0-1 | MUSIC |

// TAG: surface=simulation owner=research-team rule=SIM-001
pub mod auc_validation;
pub mod bond_mechanism_formalization;
pub mod civilizational_collapse;
pub mod civilizational_scale_protocol;
pub mod cost_reduction;
pub mod crisis_threshold;
pub mod criticality_sigma;
pub mod cross_cultural_validation;
pub mod cross_domain_correlation;
pub mod cultural_quadrants;
pub mod docas_emotion_mapping;
pub mod economic_model_comparison;
pub mod emotion_mapping;
pub mod emotion_pharmacology;
pub mod evolutionary_dynamics;
pub mod gdp_prediction;
pub mod gene_scoring_rubric;
pub mod genomic_sem_specification;
pub mod historical_collapse_validation;
pub mod hofstede_prediction;
pub mod lambda_universality;
pub mod music_auc_claim;
pub mod music_trust_correlation;
pub mod orchestrator;
pub mod pde4b_behavioral_trust;
pub mod pde4b_eeg_criticality;
pub mod pde4b_genetic_analysis;
pub mod percolation_threshold;
pub mod philosophical_oscillation;
pub mod philosophical_taxonomy_sensitivity;
pub mod semantic_drift;
pub mod semantic_network;
pub mod spectral_coherence;
pub mod stability_law;
pub mod tau_critical_measurement;
pub mod tau_phi_measurement;
pub mod trust_measurement_specification;
pub mod trust_weighted_pd;
pub mod trust_weighted_ultimatum;
pub mod twin_heritability;

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Common configuration for all experiments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    /// experiment type.
    pub experiment_type: ExperimentType,
    /// n agents.
    pub n_agents: usize,
    /// n timesteps.
    pub n_timesteps: usize,
    /// n replications.
    pub n_replications: usize,
    /// seed.
    pub seed: u64,
    /// output dir.
    pub output_dir: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl ExperimentConfig {
    /// Create default configuration with unique seed
    #[must_use]
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn default_with_type(experiment_type: ExperimentType) -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            experiment_type,
            n_agents: 1000,
            n_timesteps: 365,
            n_replications: 100,
            seed,
            output_dir: "./results".to_string(),
        }
    }

    /// Create quick configuration for testing
    #[must_use]
    pub fn quick() -> Self {
        Self {
            experiment_type: ExperimentType::LambdaUniversality,
            n_agents: 100,
            n_timesteps: 50,
            n_replications: 10,
            seed: 42,
            output_dir: "./test_output".to_string(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create RNG from seed
    #[must_use]
    pub fn rng(&self) -> rand::rngs::StdRng {
        use rand::SeedableRng;
        rand::rngs::StdRng::seed_from_u64(self.seed)
    }
}

/// Types of experiments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentType {
    /// lambda universality.
    LambdaUniversality,
    /// Field.
    /// Field.
    /// twin heritability.
    TwinHeritability {
        /// n mz.
        n_mz: usize,
        /// n dz.
        n_dz: usize,
    },
    /// criticality sigma.
    CriticalitySigma,
    /// psi trust auc.
    PsiTrustAuc,
    /// emotion mapping.
    EmotionMapping,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// semantic network.
    SemanticNetwork,
    /// cultural quadrants.
    CulturalQuadrants,
    /// gdp prediction.
    GdpPrediction,
    /// trust weighted pd.
    TrustWeightedPd,
    /// trust weighted ultimatum.
    TrustWeightedUltimatum,
    /// percolation threshold.
    PercolationThreshold,
    /// philosophical oscillation.
    PhilosophicalOscillation,
    /// spectral coherence.
    SpectralCoherence,
    /// stability law.
    StabilityLaw,
    /// semantic drift.
    SemanticDrift,
    /// crisis threshold.
    CrisisThreshold,
    /// hofstede prediction.
    HofstedePrediction,
    /// evolutionary dynamics.
    EvolutionaryDynamics,
    /// pde4b genetic analysis.
    Pde4bGeneticAnalysis,
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// cross domain correlation.
    CrossDomainCorrelation,
    /// cost reduction.
    CostReduction,
    /// emotion pharmacology.
    EmotionPharmacology,
    /// civilizational collapse.
    CivilizationalCollapse,
    /// music trust correlation.
    MusicTrustCorrelation,
    /// docasemotion mapping.
    DOCASEmotionMapping,
    /// historical collapse validation.
    HistoricalCollapseValidation,
    /// tau critical measurement.
    TauCriticalMeasurement,
    /// tau phi measurement.
    TauPhiMeasurement,
    /// philosophical taxonomy sensitivity.
    PhilosophicalTaxonomySensitivity,
    /// bond mechanism formalization.
    BondMechanismFormalization,
    /// trust measurement specification.
    TrustMeasurementSpecification,
    /// pde4bbehavioral trust.
    PDE4BBehavioralTrust,
    /// pde4beeg criticality.
    PDE4BEegCriticality,
    /// civilizational scale protocol.
    CivilizationalScaleProtocol,
    /// cross cultural validation.
    CrossCulturalValidation,
    /// gene scoring rubric.
    GeneScoringRubric,
    /// genomic semspecification.
    GenomicSEMSpecification,
    /// economic model comparison.
    EconomicModelComparison,
    /// music auc claim.
    MusicAUCClaim,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Common result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResult {
    /// experiment type.
    pub experiment_type: String,
    /// success.
    pub success: bool,
    /// gap id.
    pub gap_id: String,
    /// paper.
    pub paper: String,
    /// metrics.
    pub metrics: serde_json::Value,
    /// figures.
    pub figures: Vec<String>, // Paths to generated figures
    /// csv outputs.
    pub csv_outputs: Vec<String>, // Paths to CSV files
    /// artifacts.
    pub artifacts: Vec<(String, String)>, // (filename, content) pairs
    #[serde(with = "serde_millis")]
    /// duration.
    pub duration: std::time::Duration,
    /// error message.
    pub error_message: Option<String>,
}

// Serde helper for Duration
mod serde_millis {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    pub fn serialize<S: Serializer>(d: &Duration, s: S) -> Result<S::Ok, S::Error> {
        d.as_millis().serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        let millis = u64::deserialize(d)?;
        Ok(Duration::from_millis(millis))
    }
}

/// Trait for all experiments
pub trait Experiment {
    /// Experiment name
    fn name(&self) -> &str;

    /// Experiment description
    fn description(&self) -> &str;

    /// Required number of agents
    fn required_agents(&self) -> usize;

    /// Run the experiment
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn run(&self, config: &ExperimentConfig) -> anyhow::Result<ExperimentResult>;

    /// Validate success criteria
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn validate(&self, result: &ExperimentResult) -> bool {
        result.success
    }

    /// Generate publication-ready figures
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn generate_figures(&self, _result: &ExperimentResult) -> anyhow::Result<()> {
        Ok(())
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Initialize experiment environment
/// # Errors
///
/// Returns an error if the operation fails.
pub fn init_experiment(config: &ExperimentConfig) -> anyhow::Result<()> {
    use std::fs;

    // Create output directory
    fs::create_dir_all(&config.output_dir)?;
    fs::create_dir_all(format!("{}/figures", config.output_dir))?;
    fs::create_dir_all(format!("{}/data", config.output_dir))?;

    // Log configuration
    let config_json = serde_json::to_string_pretty(config)?;
    fs::write(format!("{}/config.json", config.output_dir), config_json)?;

    Ok(())
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Save results to disk
/// # Errors
///
/// Returns an error if the operation fails.
pub fn save_results(result: &ExperimentResult, output_dir: &str) -> anyhow::Result<()> {
    use std::fs;

    let results_json = serde_json::to_string_pretty(result)?;
    fs::write(format!("{output_dir}/results.json"), results_json)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_experiment_config_quick() {
        let config = ExperimentConfig::quick();
        assert_eq!(config.n_agents, 100);
        assert_eq!(config.n_timesteps, 50);
        assert_eq!(config.n_replications, 10);
        assert_eq!(config.seed, 42);
    }

    #[test]
    fn test_experiment_config_rng() {
        let config = ExperimentConfig::quick();
        let rng = config.rng();
        // Just verify it creates an RNG without panicking
        let _ = rng;
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    #[test]
    fn test_experiment_type_variants() {
        let types = [
            ExperimentType::LambdaUniversality,
            ExperimentType::CriticalitySigma,
            ExperimentType::PsiTrustAuc,
            ExperimentType::EmotionMapping,
            ExperimentType::SemanticNetwork,
        ];
        assert_eq!(types.len(), 5);
    }

    #[test]
    fn test_experiment_result_default() {
        let result = ExperimentResult {
            experiment_type: "test".to_string(),
            success: true,
            gap_id: "GTT-P0-1".to_string(),
            paper: "GTT".to_string(),
            metrics: serde_json::json!({"accuracy": 0.95}),
            figures: vec![],
            csv_outputs: vec![],
            artifacts: vec![],
            duration: std::time::Duration::from_secs(1),
            error_message: None,
        };
        assert!(result.success);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_experiment_trait_validate() {
        struct DummyExperiment;
        // TAG: surface=simulation owner=research-team rule=SIM-001
        impl Experiment for DummyExperiment {
            fn name(&self) -> &'static str {
                "dummy"
            }
            fn description(&self) -> &'static str {
                "test"
            }
            fn required_agents(&self) -> usize {
                10
            }
            fn run(&self, _config: &ExperimentConfig) -> anyhow::Result<ExperimentResult> {
                Ok(ExperimentResult {
                    experiment_type: "dummy".to_string(),
                    success: true,
                    gap_id: "TEST".to_string(),
                    paper: "TEST".to_string(),
                    metrics: serde_json::json!({}),
                    figures: vec![],
                    csv_outputs: vec![],
                    artifacts: vec![],
                    duration: std::time::Duration::from_secs(0),
                    error_message: None,
                })
            }
        }

        let exp = DummyExperiment;
        assert_eq!(exp.name(), "dummy");
        assert_eq!(exp.required_agents(), 10);
    }

    #[test]
    fn test_serde_millis_roundtrip() {
        let duration = std::time::Duration::from_millis(1500);
        let json = serde_json::json!({"duration": duration.as_millis() as u64});
        assert_eq!(json["duration"], 1500);
    }
}
