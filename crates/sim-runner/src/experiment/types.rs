//! Experiment types, configurations, and results

use serde::{Deserialize, Serialize};
use sim_core::simulation::SimulationConfig;
use sim_core::ToggleConfig;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

/// Experiment types covering all 26 experiments from §2.4
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExperimentType {
    /// GTT-P0-1: Lambda universality validation
    LambdaUniversality,
    /// GTT-P0-6: Stability law validation
    StabilityLaw,
    /// GTT-P0-5: Cross-domain correlation
    CrossDomainCorrelation,
    /// DOCAS-P0-2: Twin heritability
    TwinHeritability {
        /// Number of monozygotic twin pairs
        n_mz: usize,
        /// Number of dizygotic twin pairs
        n_dz: usize,
    },
    /// DOCAS-P0-3: Criticality sigma
    CriticalitySigma,
    /// PDE4B-P0-1: PDE4B-lambda correlation
    Pde4bLambdaCorrelation,
    /// PDE4B-P0-2: PDE4B pharmacology
    Pde4bPharmacology,
    /// EMOT-P0-1: Emotion mapping
    EmotionMapping,
    /// EMOT-P0-3: Emotion pharmacology
    EmotionPharmacology,
    /// LING-P0-1: Semantic network (trust centrality)
    SemanticNetwork,
    /// LING-P0-2: Semantic drift
    SemanticDrift,
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// PHIL-P0-2: Philosophical oscillation
    PhilosophicalOscillation,
    /// CULT-P0-1: Cultural quadrants
    CulturalQuadrants,
    /// CULT-P0-3: Hofstede prediction
    HofstedePrediction,
    /// ECON-P0-1: GDP prediction
    GdpPrediction,
    /// ECON-P0-2: Crisis threshold
    CrisisThreshold,
    /// GAME-P0-1: Trust-weighted prisoner's dilemma
    TrustWeightedPD,
    /// Game Theory: Trust-weighted ultimatum
    TrustWeightedUltimatum,
    /// GAME-P0-3: Evolutionary dynamics
    EvolutionaryDynamics,
    /// TFP-P0-3: Psi-trust AUC
    PsiTrustAUC,
    /// TFP: Entropy reduction
    EntropyReduction,
    /// TFP-P0-4: Cost model
    CostModel,
    /// MEAN-P0-3: Percolation threshold
    PercolationThreshold,
    /// MEAN-P0-2: Civilizational collapse
    CivilizationalCollapse,
    /// MUSIC-P0-2: Spectral analysis
    SpectralCoherence,
    /// MUSIC-P0-3: Music-trust correlation
    MusicTrustCorrelation,
    /// Sim O1: Dopamine Lesion (ArchT 566nm)
    SimO1DopamineLesion,
    /// Sim O2: Cortisol Vulnerability (ChR2 470nm)
    SimO2InducedVulnerability,
    /// Sim O3: Kuramoto Mutant Fragmentation (O/S Muted)
    SimO3KuramotoFragmentation,
    /// Sim D16: Dual-Branch Visual Cloaking
    SimD16VisualCloaking,
    /// Sim D17: Weather Gating & Weather-Shield
    SimD17WeatherGating,
    /// Sim D18: Baylor Unconscious Narrative Gating & Attractor Drift
    SimD18BaylorUnconsciousGating,
}

impl ExperimentType {
    /// Get the paper this experiment belongs to
    #[must_use]
    pub const fn paper(&self) -> &'static str {
        match self {
            Self::LambdaUniversality | Self::StabilityLaw | Self::CrossDomainCorrelation => "GTT",
            Self::TwinHeritability { .. }
            | Self::CriticalitySigma
            | Self::SimO1DopamineLesion
            | Self::SimO2InducedVulnerability
            | Self::SimD16VisualCloaking
            | Self::SimD17WeatherGating
            | Self::SimD18BaylorUnconsciousGating => "DOCAS",
            Self::Pde4bLambdaCorrelation | Self::Pde4bPharmacology => "PDE4B",
            Self::EmotionMapping | Self::EmotionPharmacology => "Emotion",
            Self::SemanticNetwork | Self::SemanticDrift => "Linguistics",
            Self::PhilosophicalOscillation => "Philosophy",
            Self::CulturalQuadrants | Self::HofstedePrediction => "Culture",
            Self::GdpPrediction | Self::CrisisThreshold => "Economics",
            Self::TrustWeightedPD | Self::TrustWeightedUltimatum | Self::EvolutionaryDynamics => {
                "GameTheory"
            }
            Self::PsiTrustAUC
            | Self::EntropyReduction
            | Self::CostModel
            | Self::SimO3KuramotoFragmentation => "TFP",
            Self::PercolationThreshold | Self::CivilizationalCollapse => "Meaning",
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            Self::SpectralCoherence | Self::MusicTrustCorrelation => "Music",
        }
    }

    /// Get the P0 gap IDs this experiment addresses
    #[must_use]
    pub fn gap_ids(&self) -> Vec<&'static str> {
        match self {
            Self::LambdaUniversality => vec!["GTT-P0-1"],
            Self::StabilityLaw => vec!["GTT-P0-6"],
            Self::CrossDomainCorrelation => vec!["GTT-P0-5"],
            Self::TwinHeritability { .. } => vec!["DOCAS-P0-2"],
            Self::CriticalitySigma => vec!["DOCAS-P0-3"],
            Self::Pde4bLambdaCorrelation => vec!["PDE4B-P0-1"],
            Self::Pde4bPharmacology => vec!["PDE4B-P0-2"],
            Self::EmotionMapping => vec!["EMOT-P0-1"],
            Self::EmotionPharmacology => vec!["EMOT-P0-3"],
            Self::SemanticNetwork => vec!["LING-P0-1"],
            Self::SemanticDrift => vec!["LING-P0-2"],
            Self::PhilosophicalOscillation => vec!["PHIL-P0-2"],
            Self::CulturalQuadrants => vec!["CULT-P0-1"],
            Self::HofstedePrediction => vec!["CULT-P0-3"],
            Self::GdpPrediction => vec!["ECON-P0-1"],
            Self::CrisisThreshold => vec!["ECON-P0-2"],
            Self::TrustWeightedPD => vec!["GAME-P0-1"],
            Self::TrustWeightedUltimatum => vec![],
            Self::EvolutionaryDynamics => vec!["GAME-P0-3"],
            Self::PsiTrustAUC => vec!["TFP-P0-3"],
            Self::EntropyReduction => vec![],
            Self::CostModel => vec!["TFP-P0-4"],
            Self::PercolationThreshold => vec!["MEAN-P0-3"],
            Self::CivilizationalCollapse => vec!["MEAN-P0-2"],
            Self::SpectralCoherence => vec!["MUSIC-P0-2"],
            Self::MusicTrustCorrelation => vec!["MUSIC-P0-3"],
            Self::SimO1DopamineLesion => vec!["DOCAS-v4-O1"],
            Self::SimO2InducedVulnerability => vec!["DOCAS-v4-O2"],
            Self::SimO3KuramotoFragmentation => vec!["TFP-v4-O3"],
            Self::SimD16VisualCloaking => vec!["DOCAS-v4-D16"],
            Self::SimD17WeatherGating => vec!["DOCAS-v4-D17"],
            Self::SimD18BaylorUnconsciousGating => vec!["DOCAS-v4-D18"],
        }
    }

    /// Default agent count for this experiment
    #[must_use]
    pub fn default_agent_count(&self) -> usize {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        match self {
            Self::LambdaUniversality => 10_000,
            Self::StabilityLaw => 5_000,
            Self::CrossDomainCorrelation => 10_000,
            Self::TwinHeritability { n_mz, n_dz } => (n_mz + n_dz) * 2,
            Self::CriticalitySigma => 5_000,
            Self::Pde4bLambdaCorrelation => 5_000,
            Self::Pde4bPharmacology => 5_000,
            Self::EmotionMapping => 5_000,
            Self::EmotionPharmacology => 5_000,
            Self::SemanticNetwork => 10_000,
            Self::SemanticDrift => 10_000,
            Self::PhilosophicalOscillation => 10_000,
            Self::CulturalQuadrants => 10_000,
            Self::HofstedePrediction => 10_000,
            Self::GdpPrediction => 100_000,
            Self::CrisisThreshold => 100_000,
            Self::TrustWeightedPD => 5_000,
            Self::TrustWeightedUltimatum => 5_000,
            Self::EvolutionaryDynamics => 5_000,
            Self::PsiTrustAUC => 10_000,
            Self::EntropyReduction => 10_000,
            Self::CostModel => 100_000,
            Self::PercolationThreshold => 10_000,
            Self::CivilizationalCollapse => 10_000,
            Self::SpectralCoherence => 5_000,
            Self::MusicTrustCorrelation => 5_000,
            Self::SimO1DopamineLesion
            | Self::SimO2InducedVulnerability
            | Self::SimO3KuramotoFragmentation
            | Self::SimD16VisualCloaking
            | Self::SimD17WeatherGating
            | Self::SimD18BaylorUnconsciousGating => 1_000,
        }
    }

    /// Default tick count for this experiment
    #[must_use]
    pub const fn default_tick_count(&self) -> u64 {
        match self {
            Self::LambdaUniversality => 365,
            Self::StabilityLaw => 365,
            Self::CrossDomainCorrelation => 365,
            Self::TwinHeritability { .. } => 365,
            Self::CriticalitySigma => 365,
            Self::Pde4bLambdaCorrelation => 365,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            Self::Pde4bPharmacology => 365,
            Self::EmotionMapping => 365,
            Self::EmotionPharmacology => 365,
            Self::SemanticNetwork => 730,
            Self::SemanticDrift => 730,
            Self::PhilosophicalOscillation => 500 * 365, // 500 years
            Self::CulturalQuadrants => 500 * 365,
            Self::HofstedePrediction => 365,
            Self::GdpPrediction => 365,
            Self::CrisisThreshold => 365,
            Self::TrustWeightedPD => 100,
            Self::TrustWeightedUltimatum => 100,
            Self::EvolutionaryDynamics => 1_000,
            Self::PsiTrustAUC => 365,
            Self::EntropyReduction => 365,
            Self::CostModel => 365,
            Self::PercolationThreshold => 500 * 365,
            Self::CivilizationalCollapse => 500 * 365,
            Self::SpectralCoherence => 365,
            Self::MusicTrustCorrelation => 365,
            Self::SimO1DopamineLesion
            | Self::SimO2InducedVulnerability
            | Self::SimO3KuramotoFragmentation
            | Self::SimD16VisualCloaking
            | Self::SimD17WeatherGating
            | Self::SimD18BaylorUnconsciousGating => 100,
        }
    }

    /// Default replication count for this experiment
    #[must_use]
    pub const fn default_replications(&self) -> usize {
        match self {
            Self::LambdaUniversality => 10_000,
            Self::StabilityLaw => 5_000,
            Self::CrossDomainCorrelation => 10_000,
            Self::TwinHeritability { .. } => 2_000,
            Self::CriticalitySigma => 5_000,
            Self::Pde4bLambdaCorrelation => 5_000,
            Self::Pde4bPharmacology => 2_000,
            Self::EmotionMapping => 5_000,
            Self::EmotionPharmacology => 2_000,
            Self::SemanticNetwork => 5_000,
            Self::SemanticDrift => 5_000,
            Self::PhilosophicalOscillation => 1_000,
            Self::CulturalQuadrants => 1_000,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            Self::HofstedePrediction => 5_000,
            Self::GdpPrediction => 1_000,
            Self::CrisisThreshold => 1_000,
            Self::TrustWeightedPD => 10_000,
            Self::TrustWeightedUltimatum => 10_000,
            Self::EvolutionaryDynamics => 5_000,
            Self::PsiTrustAUC => 10_000,
            Self::EntropyReduction => 10_000,
            Self::CostModel => 1_000,
            Self::PercolationThreshold => 1_000,
            Self::CivilizationalCollapse => 1_000,
            Self::SpectralCoherence => 5_000,
            Self::MusicTrustCorrelation => 5_000,
            Self::SimO1DopamineLesion
            | Self::SimO2InducedVulnerability
            | Self::SimO3KuramotoFragmentation
            | Self::SimD16VisualCloaking
            | Self::SimD17WeatherGating
            | Self::SimD18BaylorUnconsciousGating => 1,
        }
    }
}

/// Comparison operators for acceptance criteria
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Comparison {
    /// Metric must be within range [low, high]
    InRange,
    /// Metric must be greater than threshold
    GreaterThan,
    /// Metric must be less than threshold
    LessThan,
    /// Correlation must be within range
    CorrelationIn,
}

/// Single acceptance criterion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    /// Metric name (e.g., "lambda", "heritability", "auc")
    pub metric_name: String,
    /// Expected range or threshold
    pub expected_range: (f64, f64),
    /// Confidence level for statistical validation
    pub confidence_level: f64,
    /// Comparison type
    pub comparison: Comparison,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Human-readable description
    pub description: String,
}

impl AcceptanceCriterion {
    /// Create a new range criterion
    #[must_use]
    pub fn in_range(name: &str, low: f64, high: f64, description: &str) -> Self {
        Self {
            metric_name: name.to_string(),
            expected_range: (low, high),
            confidence_level: 0.95,
            comparison: Comparison::InRange,
            description: description.to_string(),
        }
    }

    /// Create a new greater-than criterion
    #[must_use]
    pub fn greater_than(name: &str, threshold: f64, description: &str) -> Self {
        Self {
            metric_name: name.to_string(),
            expected_range: (threshold, f64::INFINITY),
            confidence_level: 0.95,
            comparison: Comparison::GreaterThan,
            description: description.to_string(),
        }
    }

    /// Create a new less-than criterion
    #[must_use]
    pub fn less_than(name: &str, threshold: f64, description: &str) -> Self {
        Self {
            metric_name: name.to_string(),
            expected_range: (f64::NEG_INFINITY, threshold),
            confidence_level: 0.95,
            comparison: Comparison::LessThan,
            description: description.to_string(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
}

/// Experiment configuration matching §2.4 Experiment Matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    /// Experiment name (e.g., "`lambda_universality`")
    pub name: String,
    /// Experiment type
    pub experiment_type: ExperimentType,
    /// Human-readable description
    pub description: String,
    /// Number of agents
    pub agent_count: usize,
    /// Number of ticks per run
    pub tick_count: u64,
    /// Number of replications
    pub replications: usize,
    /// Random seed base (replications use `seed_base` + i)
    pub seed_base: u64,
    /// Simulation configuration
    pub sim_config: SimulationConfig,
    /// Toggle configuration
    pub toggles: ToggleConfig,
    /// Acceptance criteria for validation
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    /// Output directory
    pub output_dir: PathBuf,
    /// Whether to generate CSV output
    pub output_csv: bool,
    /// Whether to generate JSON manifest
    pub output_json: bool,
    /// Whether to generate PNG visualizations
    pub output_png: bool,
}

impl ExperimentConfig {
    /// Create default configuration for an experiment type
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn for_experiment(exp_type: ExperimentType) -> Self {
        let name = format!("{exp_type:?}").to_lowercase();
        let agent_count = exp_type.default_agent_count();
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let tick_count = exp_type.default_tick_count();
        let replications = exp_type.default_replications();

        // Build acceptance criteria based on experiment type
        let acceptance_criteria = Self::default_criteria(&exp_type);

        let mut sim_config = SimulationConfig::default();
        match &exp_type {
            ExperimentType::SimO1DopamineLesion => {
                sim_config.optogenetics =
                    sim_core::optogenetics::OptogeneticConfig::dopamine_lesion();
            }
            ExperimentType::SimO2InducedVulnerability => {
                sim_config.optogenetics =
                    sim_core::optogenetics::OptogeneticConfig::cortisol_excitation(
                        sim_core::optogenetics::OptogeneticScope::EntirePopulation,
                    );
            }
            ExperimentType::SimO3KuramotoFragmentation => {
                sim_config.optogenetics =
                    sim_core::optogenetics::OptogeneticConfig::kuramoto_os_mutant_subpop(0.10);
            }
            ExperimentType::SimD16VisualCloaking => {
                sim_config.visual_cloaking =
                    sim_core::visual_cloaking::VisualAgreementStimulus::predatory_benchmark();
            }
            ExperimentType::SimD17WeatherGating => {
                sim_config.environment.weather_enabled = true;
                sim_config.visual_cloaking =
                    sim_core::visual_cloaking::VisualAgreementStimulus::predatory_benchmark();
            }
            ExperimentType::SimD18BaylorUnconsciousGating => {
                sim_config.baylor = sim_core::baylor_gating::BaylorStateConfig::unconscious();
                sim_config.visual_cloaking =
                    sim_core::visual_cloaking::VisualAgreementStimulus::predatory_benchmark();
            }
            _ => {}
        }

        let mut toggles = ToggleConfig::default();
        match &exp_type {
            ExperimentType::SimO1DopamineLesion
            | ExperimentType::SimO2InducedVulnerability
            | ExperimentType::SimO3KuramotoFragmentation
            | ExperimentType::SimD16VisualCloaking
            | ExperimentType::SimD17WeatherGating
            | ExperimentType::SimD18BaylorUnconsciousGating => {
                toggles.docas_enabled = true;
                toggles.genome_enabled = true;
            }
            _ => {
                if exp_type.paper() == "DOCAS" || exp_type.paper() == "Emotion" {
                    toggles.docas_enabled = true;
                }
            }
        }
        sim_config.toggles = toggles;

        Self {
            name,
            experiment_type: exp_type.clone(),
            description: format!("{exp_type:?} experiment for {}", exp_type.paper()),
            agent_count,
            tick_count,
            replications,
            seed_base: 42,
            sim_config,
            toggles,
            acceptance_criteria,
            output_dir: PathBuf::from("output"),
            output_csv: true,
            output_json: true,
            output_png: true,
        }
    }

    /// Default acceptance criteria for each experiment type
    fn default_criteria(exp_type: &ExperimentType) -> Vec<AcceptanceCriterion> {
        match exp_type.paper() {
            "GTT" => Self::default_criteria_gtt(exp_type),
            "DOCAS" => Self::default_criteria_docas(exp_type),
            "PDE4B" => Self::default_criteria_pde4b(exp_type),
            "Emotion" => Self::default_criteria_emotion(exp_type),
            "Linguistics" => Self::default_criteria_linguistics(exp_type),
            "Philosophy" => Self::default_criteria_philosophy(exp_type),
            "Culture" => Self::default_criteria_culture(exp_type),
            "Economics" => Self::default_criteria_economics(exp_type),
            "GameTheory" => Self::default_criteria_gametheory(exp_type),
            "TFP" => Self::default_criteria_tfp(exp_type),
            "Meaning" => Self::default_criteria_meaning(exp_type),
            "Music" => Self::default_criteria_music(exp_type),
            _ => vec![],
            // TAG: surface=simulation owner=research-team rule=SIM-001
        }
    }

    fn default_criteria_gtt(exp_type: &ExperimentType) -> Vec<AcceptanceCriterion> {
        match exp_type {
            ExperimentType::LambdaUniversality => vec![AcceptanceCriterion::in_range(
                "lambda",
                0.53,
                0.69,
                "Lambda must be in universal range [0.53, 0.69]",
            )],
            ExperimentType::StabilityLaw => vec![AcceptanceCriterion::greater_than(
                "stability_r2",
                0.60,
                "Stability law R² must be ≥ 0.60",
            )],
            ExperimentType::CrossDomainCorrelation => vec![AcceptanceCriterion::in_range(
                "cross_domain_r",
                0.40,
                0.60,
                "Cross-domain correlation must be in [0.40, 0.60]",
            )],
            _ => vec![],
        }
    }

    fn default_criteria_docas(exp_type: &ExperimentType) -> Vec<AcceptanceCriterion> {
        match exp_type {
            ExperimentType::TwinHeritability { .. } => vec![AcceptanceCriterion::in_range(
                "heritability",
                0.42,
                0.56,
                "Heritability h² must be in [0.42, 0.56]",
            )],
            ExperimentType::CriticalitySigma => vec![AcceptanceCriterion::in_range(
                "sigma",
                0.9,
                1.1,
                "Criticality σ must be in [0.9, 1.1]",
            )],
            ExperimentType::SimO1DopamineLesion => vec![AcceptanceCriterion::greater_than(
                "dopamine_lesion_suppression",
                0.90,
                "Dopamine lesion must suppress trust accumulation by >= 90%",
            )],
            ExperimentType::SimO2InducedVulnerability => vec![AcceptanceCriterion::greater_than(
                "stress_relative_risk",
                1.20,
                "Cortisol excitation relative risk >= 1.20x",
            )],
            ExperimentType::SimD16VisualCloaking => vec![AcceptanceCriterion::greater_than(
                "decile_odds_ratio",
                3.00,
                "Lowest vs highest decile odds ratio >= 3.0x",
            )],
            ExperimentType::SimD17WeatherGating => vec![AcceptanceCriterion::greater_than(
                "heatwave_relative_risk",
                1.02,
                "Heatwave stress relative risk >= 1.02x",
            )],
            ExperimentType::SimD18BaylorUnconsciousGating => vec![
                AcceptanceCriterion::greater_than(
                    "cortisol_friction_reduction",
                    0.75,
                    "Baylor unconscious Cortisol threat bypassing must collapse cognitive friction by >= 75%",
                ),
                AcceptanceCriterion::less_than(
                    "syntax_difference_ratio",
                    0.05,
                    "Syntactic sequence schema perplexity must remain within 5% under total dopaminergic silencing (D=0)",
                ),
            ],


            _ => vec![],
            // TAG: surface=simulation owner=research-team rule=SIM-001
        }
    }

    fn default_criteria_pde4b(exp_type: &ExperimentType) -> Vec<AcceptanceCriterion> {
        match exp_type {
            ExperimentType::Pde4bLambdaCorrelation => vec![AcceptanceCriterion::less_than(
                "pde4b_tau_correlation",
                -0.30,
                "PDE4B-trust correlation must be < -0.30",
            )],
            ExperimentType::Pde4bPharmacology => vec![AcceptanceCriterion::greater_than(
                "pharm_effect_size",
                0.50,
                "Pharmacological effect size d > 0.50",
            )],
            _ => vec![],
        }
    }

    fn default_criteria_emotion(exp_type: &ExperimentType) -> Vec<AcceptanceCriterion> {
        match exp_type {
            ExperimentType::EmotionMapping => vec![AcceptanceCriterion::greater_than(
                "emotion_r2",
                0.75,
                "Emotion mapping R² must be > 0.75",
            )],
            ExperimentType::EmotionPharmacology => vec![AcceptanceCriterion::greater_than(
                "emotion_pharm_r2",
                0.40,
                "Emotion-pharmacology R² > 0.40",
            )],
            _ => vec![],
        }
    }

    fn default_criteria_linguistics(exp_type: &ExperimentType) -> Vec<AcceptanceCriterion> {
        match exp_type {
            ExperimentType::SemanticNetwork => vec![AcceptanceCriterion::greater_than(
                "trust_centrality",
                2.0,
                // TAG: surface=simulation owner=research-team rule=SIM-001
                "Trust betweenness centrality must be > 2σ above mean",
            )],
            ExperimentType::SemanticDrift => vec![AcceptanceCriterion::less_than(
                "drift_trust_correlation",
                -0.30,
                "Semantic drift anticorrelates with trust (r < -0.30)",
            )],
            _ => vec![],
        }
    }

    fn default_criteria_philosophy(exp_type: &ExperimentType) -> Vec<AcceptanceCriterion> {
        match exp_type {
            ExperimentType::PhilosophicalOscillation => vec![AcceptanceCriterion::in_range(
                "oscillation_period",
                200.0,
                300.0,
                "Philosophical oscillation period ≈ 250 years",
            )],
            _ => vec![],
        }
    }

    fn default_criteria_culture(exp_type: &ExperimentType) -> Vec<AcceptanceCriterion> {
        match exp_type {
            ExperimentType::CulturalQuadrants => vec![AcceptanceCriterion::greater_than(
                "quadrant_r2",
                0.80,
                "Cultural quadrant model R² must be > 0.80",
            )],
            ExperimentType::HofstedePrediction => vec![AcceptanceCriterion::greater_than(
                "hofstede_r2",
                0.60,
                "Hofstede prediction R² must be > 0.60",
            )],
            _ => vec![],
        }
    }

    fn default_criteria_economics(exp_type: &ExperimentType) -> Vec<AcceptanceCriterion> {
        match exp_type {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            ExperimentType::GdpPrediction => vec![AcceptanceCriterion::greater_than(
                "gdp_r2",
                0.85,
                "GDP prediction R² must be > 0.85",
            )],
            ExperimentType::CrisisThreshold => vec![AcceptanceCriterion::in_range(
                "crisis_threshold",
                0.25,
                0.35,
                "Crisis threshold τ ∈ [0.25, 0.35]",
            )],
            _ => vec![],
        }
    }

    fn default_criteria_gametheory(exp_type: &ExperimentType) -> Vec<AcceptanceCriterion> {
        match exp_type {
            ExperimentType::EvolutionaryDynamics => vec![AcceptanceCriterion::greater_than(
                "ess_stability",
                0.80,
                "ESS must be stable (>80% after convergence)",
            )],
            ExperimentType::TrustWeightedPD => vec![AcceptanceCriterion::greater_than(
                "trust_eu_correlation",
                0.50,
                "Trust-EU correlation must be > 0.50",
            )],
            ExperimentType::TrustWeightedUltimatum => vec![AcceptanceCriterion::greater_than(
                "trust_offer_correlation",
                0.40,
                "Trust-offer correlation r > 0.40",
            )],
            _ => vec![],
        }
    }

    fn default_criteria_tfp(exp_type: &ExperimentType) -> Vec<AcceptanceCriterion> {
        match exp_type {
            ExperimentType::PsiTrustAUC => vec![AcceptanceCriterion::greater_than(
                "auc",
                // TAG: surface=simulation owner=research-team rule=SIM-001
                0.80,
                "Ψ_trust AUC must be ≥ 0.80",
            )],
            ExperimentType::EntropyReduction => vec![AcceptanceCriterion::less_than(
                "entropy_reduction",
                -0.10,
                "Entropy must decrease over time (ΔH < -0.10)",
            )],
            ExperimentType::CostModel => vec![AcceptanceCriterion::in_range(
                "marginal_cost",
                0.01,
                0.05,
                "Marginal Ψ_trust cost ∈ [0.01, 0.05]",
            )],
            ExperimentType::SimO3KuramotoFragmentation => vec![AcceptanceCriterion::less_than(
                "coherence_collapse",
                0.40,
                "10% mutant subpopulation coherence drop < 40%",
            )],

            _ => vec![],
        }
    }

    fn default_criteria_meaning(exp_type: &ExperimentType) -> Vec<AcceptanceCriterion> {
        match exp_type {
            ExperimentType::PercolationThreshold => vec![AcceptanceCriterion::in_range(
                "tau_critical",
                0.29,
                0.35,
                "Percolation threshold must be ≈ 0.32",
            )],
            ExperimentType::CivilizationalCollapse => vec![AcceptanceCriterion::greater_than(
                "collapse_accuracy",
                0.70,
                "Collapse prediction accuracy must be > 70%",
            )],
            _ => vec![],
        }
    }

    fn default_criteria_music(exp_type: &ExperimentType) -> Vec<AcceptanceCriterion> {
        match exp_type {
            ExperimentType::SpectralCoherence => vec![
                AcceptanceCriterion::greater_than(
                    "spectral_coherence",
                    0.30,
                    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                    "Spectral coherence must be > 0.30",
                ),
                AcceptanceCriterion::in_range(
                    "spectral_exponent",
                    0.8,
                    1.2,
                    "Spectral exponent must be ~1.0 (1/f)",
                ),
            ],
            ExperimentType::MusicTrustCorrelation => vec![
                AcceptanceCriterion::greater_than(
                    "music_lead_fraction",
                    0.10,
                    "Music as leading indicator for >10% population",
                ),
                AcceptanceCriterion::greater_than(
                    "mean_lead_time",
                    2.0,
                    "Music leads trust by >2 ticks",
                ),
            ],
            _ => vec![],
        }
    }

    /// Set custom output directory
    #[must_use]
    pub fn with_output_dir(mut self, dir: impl AsRef<Path>) -> Self {
        self.output_dir = dir.as_ref().to_path_buf();
        self
    }
}

/// Result of a single replication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationResult {
    /// Replication ID (0-indexed)
    pub rep_id: usize,
    /// Random seed used
    pub seed: u64,
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Collected metrics
    pub metrics: HashMap<String, f64>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Whether replication completed successfully
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Single criterion validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriterionResult {
    /// Criterion that was checked
    pub criterion: AcceptanceCriterion,
    /// Observed value
    pub observed_value: f64,
    /// Whether criterion passed
    pub passed: bool,
    /// 95% confidence interval
    pub confidence_interval: (f64, f64),
}

/// Validation results for an experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether all criteria passed
    pub passed: bool,
    /// Individual criterion results
    pub criteria_results: Vec<CriterionResult>,
    /// Confidence intervals for all metrics
    pub confidence_intervals: HashMap<String, (f64, f64)>,
}

/// Experiment summary statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExperimentSummary {
    /// Mean of each metric across replications
    pub means: HashMap<String, f64>,
    /// Standard deviation of each metric
    pub stds: HashMap<String, f64>,
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Median of each metric
    pub medians: HashMap<String, f64>,
    /// Min of each metric
    pub mins: HashMap<String, f64>,
    /// Max of each metric
    pub maxs: HashMap<String, f64>,
    /// Number of successful replications
    pub successful_replications: usize,
    /// Total replications attempted
    pub total_replications: usize,
}

/// Complete experiment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResult {
    /// Configuration used
    pub config: ExperimentConfig,
    /// Individual replication results
    pub replications: Vec<ReplicationResult>,
    /// Summary statistics
    pub summary: ExperimentSummary,
    /// Validation results
    pub validation: ValidationResult,
    /// Overall pass/fail
    pub passed: bool,
    /// Output file paths
    pub csv_path: Option<PathBuf>,
    /// Manifest file path
    pub manifest_path: Option<PathBuf>,
    /// Generated PNG file paths
    pub png_paths: Vec<PathBuf>,
}

impl ExperimentResult {
    /// Generate experiment ID based on type and timestamp
    #[must_use]
    pub fn experiment_id(&self) -> String {
        format!(
            "{}-{}",
            self.config.name.to_lowercase().replace(' ', "-"),
            // TAG: surface=simulation owner=research-team rule=SIM-001
            chrono::Utc::now().format("%Y%m%d-%H%M%S")
        )
    }

    /// Export results to CSV format
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn to_csv(&self) -> Result<String, ExperimentError> {
        if self.config.replications == 0 || self.replications.is_empty() {
            return Ok(String::new());
        }
        let mut lines = Vec::new();

        // Collect all metric keys from first replication
        let metric_keys: Vec<String> = if let Some(first) = self.replications.first() {
            first.metrics.keys().cloned().collect()
        } else {
            Vec::new()
        };

        // Header
        let mut header = vec!["rep_id".to_string(), "seed".to_string()];
        header.extend(metric_keys.clone());
        header.push("duration_ms".to_string());
        lines.push(header.join(","));

        // Data rows
        for rep in &self.replications {
            let mut row = vec![rep.rep_id.to_string(), rep.seed.to_string()];
            for key in &metric_keys {
                let value = rep.metrics.get(key).copied().unwrap_or(f64::NAN);
                row.push(format!("{value:.6}"));
            }
            row.push(rep.duration_ms.to_string());
            lines.push(row.join(","));
        }

        Ok(lines.join("\n"))
    }

    /// Export results to JSON manifest
    /// # Errors
    ///
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Returns an error if the operation fails.
    pub fn to_json(&self) -> Result<String, ExperimentError> {
        let manifest = serde_json::json!({
            "experiment_id": self.experiment_id(),
            "experiment_type": format!("{:?}", self.config.experiment_type),
            "paper": self.config.experiment_type.paper(),
            "gap_ids": self.config.experiment_type.gap_ids(),
            "config": {
                "name": self.config.name,
                "agent_count": self.config.agent_count,
                "tick_count": self.config.tick_count,
                "replications": self.config.replications,
                "seed_base": self.config.seed_base,
            },
            "summary": {
                "means": self.summary.means,
                "stds": self.summary.stds,
            },
            "validation": {
                "passed": self.validation.passed,
                "criteria": self.validation.criteria_results.iter().map(|c| {
                    serde_json::json!({
                        "name": c.criterion.metric_name,
                        "passed": c.passed,
                        "actual": c.observed_value,
                        "expected": (c.criterion.expected_range.0, c.criterion.expected_range.1),
                    })
                }).collect::<Vec<_>>(),
            },
            "passed": self.passed,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        serde_json::to_string_pretty(&manifest)
            .map_err(|e| ExperimentError::Serialization(e.to_string()))
    }

    /// Generate markdown report
    /// # Errors
    ///
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Returns an error if the operation fails.
    pub fn to_markdown(&self) -> Result<String, ExperimentError> {
        let mut md = Vec::new();
        md.extend(self.md_header());
        md.extend(self.md_config());
        md.extend(self.md_summary());
        md.extend(self.md_validation());
        md.push(format!(
            "\n---\n*Generated: {}*\n",
            chrono::Utc::now().to_rfc3339()
        ));
        Ok(md.join(""))
    }

    fn md_header(&self) -> Vec<String> {
        vec![
            format!("# Experiment Report: {}\n", self.config.name),
            format!("**Experiment ID:** {}\n", self.experiment_id()),
            format!("**Paper:** {}\n", self.config.experiment_type.paper()),
            format!(
                "**Gaps Addressed:** {:?}\n\n",
                self.config.experiment_type.gap_ids()
            ),
        ]
    }

    fn md_config(&self) -> Vec<String> {
        vec![
            "## Configuration\n".to_string(),
            format!("- **Agents:** {}\n", self.config.agent_count),
            format!("- **Ticks:** {}\n", self.config.tick_count),
            format!("- **Replications:** {}\n", self.config.replications),
            format!("- **Seed:** {}\n\n", self.config.seed_base),
        ]
    }

    fn md_summary(&self) -> Vec<String> {
        let mut md = vec!["## Summary Statistics\n".to_string()];
        if self.summary.means.is_empty() {
            md.push("*No summary statistics available.*\n\n".to_string());
            return md;
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        }
        md.push("| Metric | Mean | Std |\n".to_string());
        md.push("|--------|------|-----|\n".to_string());
        for (metric, mean) in &self.summary.means {
            let std = self.summary.stds.get(metric).copied().unwrap_or(0.0);
            md.push(format!("| {metric} | {mean:.4} | {std:.4} |\n"));
        }
        md.push("\n".to_string());
        md
    }

    fn md_validation(&self) -> Vec<String> {
        let mut md = vec!["## Validation Results\n".to_string()];
        if self.validation.criteria_results.is_empty() {
            md.push("*No acceptance criteria defined for this experiment.*\n\n".to_string());
            return md;
        }
        let overall = if self.validation.passed {
            "✅ PASSED"
        } else {
            "❌ FAILED"
        };
        md.push(format!("**Overall:** {overall}\n\n"));
        md.push("| Criterion | Expected | Actual | Status |\n".to_string());
        md.push("|-----------|----------|--------|--------|\n".to_string());
        for c in &self.validation.criteria_results {
            let status = if c.passed { "✓" } else { "✗" };
            md.push(format!(
                "| {} | [{:.2}, {:.2}] | {:.4} | {} |\n",
                c.criterion.metric_name,
                c.criterion.expected_range.0,
                c.criterion.expected_range.1,
                c.observed_value,
                status
            ));
        }
        md.push("\n".to_string());
        md
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get criterion results for CLI display
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[must_use]
    pub const fn acceptance_results(&self) -> &Vec<CriterionResult> {
        &self.validation.criteria_results
    }
}

/// Experiment runner error types
#[derive(Debug, thiserror::Error)]
pub enum ExperimentError {
    /// I/O error during experiment execution
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    /// Serialization or deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    /// Experiment result validation failed
    #[error("Validation failed: {0}")]
    Validation(String),
    /// Simulation execution error
    #[error("Simulation error: {0}")]
    Simulation(String),
}

/// Main experiment runner
#[derive(Debug)]
pub struct ExperimentRunner;
