//! # sim-core: Deterministic Simulation Engine Kernel
//!
//! SIM-001: Isolation verified - ZERO product dependencies
//! Per §3 lines 560-575: This crate MUST NOT import web, DB, auth, blockchain,
//! payments, network, UI, or product crates.
// TAG: surface=simulation owner=research-team rule=SIM-001
//!
//! ## Purpose
//! Research-grade agent-based model (ABM) for testing trust emergence across
//! six developmental stages: Geography → Biology → Cognition → Society → Language → Protocol.
//!
//! ## Key Design Principles
//! - Trust is NEVER hardcoded - it emerges from behavior (§1 lines 82-99)
//! - Deterministic execution with seeded RNG (SIM-004)
//! - 9 toggleable mechanisms for factorial experiments (§0.4 lines 54)
//!
//! ## Module Structure (per §3 lines 560-575)
//! - `agent`: Agent state and behavior (§7.5.1)
//! - `environment`: Grid-based world with resources (Stage 0)
//! - `genome`: Heritable traits with diploid encoding (§7.5)
//! - `docas`: Neuromodulator-inspired state vector (§6.1)
//! - `music`: Music as universal trust indicator (§6.16)
//! - `toggles`: 10 mechanism toggle contracts (§8.2.1)
//! - `simulation`: Main simulation loop and tick execution

// This is a simulation research crate with heavy numeric conversions,
// long setup functions, and domain-specific naming conventions.
// The following pedantic lints are allowed globally to keep the code
// readable and maintainable without sacrificing research flexibility.
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::suboptimal_flops)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::unused_self)]
#![allow(clippy::format_push_string)]
#![allow(clippy::significant_drop_tightening)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::module_inception)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::similar_names)]
// TAG: surface=simulation owner=research-team rule=SIM-001
#![allow(clippy::unreadable_literal)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::no_effect_underscore_binding)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::too_long_first_doc_paragraph)]
#![allow(clippy::match_wildcard_for_single_variants)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::float_cmp)]
#![allow(clippy::large_stack_arrays)]
// SIM-003: Terminology disambiguation
// "Agent" here refers to SIMULATION agents, NOT FreshCredit AI agents (AGENT-* rules)
#![forbid(unsafe_code)]

pub mod agent;
pub mod barnes_hut;
pub mod baylor_gating;

pub mod batch;
pub mod biomarkers;

pub mod birthrate;
pub mod choice_architecture;
pub mod circadian;
pub mod clics3_data;
pub mod cognition;
pub mod collective;
pub mod communication;
pub mod cross_domain;
pub mod culture;
pub mod dmn;
pub mod docas;
pub mod ecology;
pub mod economics;
/// emotion module.
pub mod emotion;
pub mod emotion_dynamics;
pub mod encounter;
// TAG: surface=simulation owner=research-team rule=SIM-001
pub mod encounter_games;
pub mod environment;
pub mod epigenome;
/// Experiments for P0 gap resolution (47 critical publication blockers)
pub mod experiments;
pub mod extended_papers;
pub mod feedback;
/// Explicit fitness testing protocol for research (complements implicit fitness)
pub mod fitness;
pub mod games;
pub mod genealogy;
pub mod generation;
pub mod genetics;
pub mod genome;
pub mod heritability;
pub mod identity;
pub mod issue_log;
pub mod language;
pub mod language_drift;
pub mod lineage;
pub mod meaning;
pub mod memory_consolidation;
pub mod merit;
pub mod meta_analysis;
pub mod metrics;
pub mod music;
pub mod neural;
pub mod optogenetics;

pub mod paradigm;
pub mod parameters;
pub mod perception;
pub mod performance;
pub mod pharmacology;
pub mod philosophy;
pub mod pom;
pub mod privacy;
pub mod protocol;
// TAG: surface=simulation owner=research-team rule=SIM-001
pub mod visual_cloaking;

pub mod credit_structural;
pub mod eu_bridge;
pub mod kuramoto_leakage;
pub mod master_law;
pub mod observation_dissipation;
pub mod psychology;
pub mod quantum;
pub mod reputation;
pub mod research;
pub mod sensitivity;
pub mod simulation;
pub mod social;
pub mod society;
pub mod stats;
pub mod stigmergy;
pub mod temporal;
pub mod theory_of_mind;
pub mod wetlab_calibration;

pub mod toggles;
pub mod types;
/// UnifiedAgent for 12-paper P0 gap resolution (CRITICAL PATH)
pub mod unified_agent;
pub mod v42_batteries;
pub mod vec2vec_training;
pub mod visualization;
pub use v42_batteries::*;

// Re-export core types for convenience
pub use agent::Agent;
pub use birthrate::{BirthrateConfig, BirthrateSystem};
pub use circadian::{AgentCircadian, CircadianModulations, CircadianSystem};
pub use cognition::{
    ActionCapabilities,
    ActionType,
    Affordance,
    BodyImage,
    BodyPart,
    BodyPartState,
    BodyRequirement,
    BodySchema,
    ChangeDirection,
    CognitiveScaffold,
    CoordinationPattern,
    CoupledSystem,
    // TAG: surface=simulation owner=research-team rule=SIM-001
    DetectedObject,
    // Embodied cognition
    EmbodiedCognition,
    ExtendedMind,
    ExternalRepresentation,
    HandRequirement,
    ObjectId,
    ObjectType,
    PerceptualMode,
    PerceptualSalience,
    ProprioceptiveMap,
    RepresentationType,
    ScaffoldType,
    SensorimotorContingency,
    SensoryInput,
    SensoryModality,
    SpatialLayout,
    SystemComponent,
    TactileEvent,
};
pub use collective::{
    CollectiveGroup, CollectiveIntentionality, CollectiveIntentionalityStats, ConstitutiveRule,
    Goal, WeIntention,
};
pub use cross_domain::{
    CrossDomainAlignment, CrossDomainTrust, DomainAlignment, PopulationCoherence, DOMAIN_DOCAS,
    DOMAIN_EMOTION, DOMAIN_LANGUAGE, DOMAIN_MUSIC,
};
pub use docas::{
    DocasState, TrustCurvature, TrustViscosity, ViscosityRegime, CURVATURE_WINDOW_SIZE,
};
pub use ecology::{AlleeComponent, AlleeConfig, AlleeEffect, AlleeMetrics};
pub use economics::banking::{Bank, BankHealth, BankId, BankingSystem, Loan, SystemicRiskMetrics};
pub use economics::{EconomicState, GroupEconomics};
// TAG: surface=simulation owner=research-team rule=SIM-001
pub use biomarkers::*;
pub use emotion::{
    arousal_trajectory, classify_emotion, compute_intensity, compute_valence, detect_transitions,
    emotion_intensity_series, emotion_trajectory, emotional_stability, is_in_crisis,
    predict_crisis_probability, valence_trajectory, validate_emo_c1, validate_emo_c2,
    validate_emo_c3, Emotion, EmotionCategory, EmotionClassification, EmotionClassifier,
    EmotionState,
};
pub use environment::Environment;
pub use epigenome::{EpigeneticStats, Epigenome, MethylationCause, MethylationMarker};
pub use feedback::{
    BaseballCard, BelievabilityVoteResult, DimensionScore, Dot, DotCollector, DotCollectorRegistry,
    FeedbackDimension, InteractionType, PopulationStats, RatingBias,
};
pub use genealogy::{
    build_coalescent_from_lineage, find_all_pairwise_mrcas, find_mrca, validate_coalescent_tree,
    CoalescentNode, CoalescentStats, KingmanCoalescent, NodeId,
};
pub use generation::{
    compute_allele_frequencies, compute_heterozygosity, sample_offspring_genomes,
    DiscreteGenerationManager, FixationEvent, FixationStatus, GenerationCohort, GenerationMode,
    WrightFisherDiscreteModel,
};
pub use genetics::{
    calculate_offspring_f, can_reproduce_inbreeding, compute_population_inbreeding_stats,
    describe_relationship, is_inbred, is_severe_inbreeding, predict_offspring_fitness,
    InbreedingStats, InbreedingTracker, INBREEDING_AVOIDANCE_THRESHOLD, MIN_REPRODUCTION_FITNESS,
};
pub use genome::Genome;
pub use heritability::{HeritabilityAnalysis, HeritabilityEstimate};
pub use identity::{
    attempt_identity_theft, calculate_theft_metrics, compare_protection,
    process_identity_theft_population, Claim, ClaimType, Identity, IdentityClaim, IdentityMetrics,
    IdentityTheft, IdentityTheftStats, PomValidation, TheftFailureReason, TheftMethod,
    TheftMetrics, TheftResult,
};
pub use issue_log::{
    // TAG: surface=simulation owner=research-team rule=SIM-001
    Issue,
    IssueCategory,
    IssueLog,
    IssueLogError,
    IssueLogStats,
    IssueSeverity,
    IssueStatus,
    Lesson,
    RecurringPattern,
};
pub use language_drift::{
    validate_ling_c1, validate_ling_c2, LanguageDriftTracker, PopulationLanguageDrift,
};
pub use lineage::{BrainEvolutionStats, LineageNode, LineageTree, TraitSnapshot};
pub use meaning::{
    CollapseDynamics, CollapseEvent, MeaningPercolation, MeaningState, PopulationMeaning,
    TAU_CRITICAL,
};
pub use merit::{
    AttributeId, AttributeStatus, MeritAttribute, MeritError, MeritGovernanceType, MeritRegistry,
    MeritRegistryStats, WeightSnapshot, ATTR_CONTRIBUTION, ATTR_COOPERATION, ATTR_REPUTATION,
    ATTR_VERIFICATION,
};
pub use meta_analysis::{
    CrossCuttingTheme, CrossPaperComparison, MetaAggregator, MetaAnalysisConfig, PaperResults,
    PublicationReadiness, ReproducibilityStatus, ResearchGap,
};
pub use music::{MusicGenre, MusicMetrics, MusicState, SyncPerformance};
pub use neural::{
    ActivityHistory, Astrocyte, AxonSegmentId, AxonType, GlialStatistics, GlialSystem,
    Oligodendrocyte,
};
pub use neural::{
    AggregationStrategy, ConvergenceTracker, FederatedCoordinator, FederatedLearning,
    FederatedLearningError, FederatedRoundStats, LocalParticipant, TrainingSample,
};
pub use neural::{
    AttackMetrics, AttackModel, AuditReport, BackdoorAttack, ByzantineAttack, ByzantineType,
    FLAttack, FLAttackSimulator, GradientLeakageAttack, MembershipInferenceAttack, ModelAPI,
    ModelInversionAttack, PrivacyAuditor, SimpleModel,
};
pub use neural::{
    BackendType,
    NeuralValidationConfig,
    NeuralValidationEngine,
    PomValidationResult,
    // TAG: surface=simulation owner=research-team rule=SIM-001
    ValidationMetrics,
};
pub use neural::{
    CalciumParams, ConsolidationParams, HomeostaticParams, MetaplasticityParams, NeuronId,
    PlasticityBuilder, PlasticityStats, PlasticityTag, StdpParams, Synapse, SynapticPlasticity,
};
pub use neural::{GlutamateStatistics, GlutamateSystem, PlasticityState, AMPAR, GABAR, NMDAR};
pub use optogenetics::{OpsinWavelength, OptogeneticConfig, OptogeneticScope};
pub use paradigm::{
    AgentParadigmState, Anomaly, Paradigm, ParadigmId, ParadigmShift, ParadigmState,
    ParadigmSystem, PopulationParadigmStats, ShiftCause,
};
pub use parameters::SimulationParameters;
pub use philosophy::{PhilosophicalOrientation, PhilosophicalState, PopulationPhilosophy};
pub use pom::{
    validate_all_targets, validate_simulation, CategoryResults, PomCategory, PomMetrics,
    PomPriority, PomTargetResult, PomValidationSuite, PomValidator,
};
pub use privacy::{
    BudgetStatus, DifferentialPrivacy, NoiseMechanism, PrivacyAccountant, PrivacyError,
    PrivatePopulationStats,
};
pub use protocol::{ProtocolState, VerificationMode};
pub use psychology::{
    compute_big5_heritability, loci as personality_loci, EarlyLifeEnvironment,
    PersonalityHeritability, PersonalityTraits, TRAIT_AGREEABLENESS, TRAIT_CONSCIENTIOUSNESS,
    TRAIT_EXTRAVERSION, TRAIT_NEUROTICISM, TRAIT_OPENNESS,
};
pub use quantum::{
    generate_quantum_keypair,
    Complex,
    CryptoPerformanceMetrics,
    DilithiumSignature,
    GroverSearch,
    KeyGenerationResult,
    MeasurementResult,
    MigrationStrategy,
    // TAG: surface=simulation owner=research-team rule=SIM-001
    NoiseModel,
    PostQuantumSecurity,
    QuantumAlgorithmType,
    QuantumAttackResult,
    QuantumAttackSimulator,
    QuantumAttackType,
    QuantumCryptoMode,
    QuantumGate,
    QuantumPrivateKey,
    QuantumPublicKey,
    QuantumSignature,
    // Quantum simulator
    QuantumSimulator,
    QuantumThreatModel,
    ResistanceReport,
    ShorAlgorithm,
    SphincsSignature,
    SphincsVariant,
    ThreatLevel,
};
pub use reputation::{
    DamageEvent, DamageEventType, EraComparison, JudgmentSource, PopulationVolatilityMetrics,
    ReputationConfusionMatrix, ReputationVolatilityTracker,
};
pub use research::{
    BayesianModel, BayesianValidator, CausalConfig, CausalEdge, CausalEffect, CausalGraph,
    CausalInference, CausalRelationship, EdgeType, Evidence, GrangerResult, Hypothesis,
    HypothesisConfig, HypothesisGenerator, HypothesisPriority, HypothesisStatus,
    LikelihoodFunction, LiteratureConfig, LiteratureEntry, LiteratureReview, McmcConfig,
    ModelComparison, ModelFit, Parameter, PosteriorApproximation, PpcResult, PriorDistribution,
    RelationshipType, ResearchConfig, ResearchEngine, ResearchReport, Synthesis, Theme,
};
pub use simulation::{
    EncounterEvent, EventStore, EventSummary, GameEvent, Simulation, SimulationReport,
};
pub use stigmergy::{
    // TAG: surface=simulation owner=research-team rule=SIM-001
    PheromoneGrid,
    PheromoneStats,
    PheromoneType,
    StigmergyConfig,
    PHEROMONE_TYPE_COUNT,
};
pub use temporal::{
    AdoptionCurve, BassDiffusionModel, EraTracker, EraTransitionEvent, SimulationEra,
    TemporalConfig,
};
pub use toggles::{DialConfig, DialValidationError, ToggleConfig};
pub use types::*;
pub use vec2vec_training::{
    BufferStats, FederatedDomain, FederatedVec2VecTrainer, OnlineTrainer, TrainingBuffer,
    TrainingStats,
};
pub use visual_cloaking::{
    BrainItViewMode, ContractCategory, DecileVulnerabilityGradient, VisualAgreementStimulus,
};
pub use visualization::{
    ChartConfig, ChartDimensions, ChartGenerator, ChartPoint, ChartSeries, ChartType, ColorScheme,
    DataAdapter,
};
pub use wetlab_calibration::*;

/// Simulation version for artifact compatibility (per Appendix B lines 7131-7195)
pub const SIM_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Schema version for manifest.json compatibility (§22.3.1 lines 6949)
pub const SCHEMA_VERSION: u32 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_exists() {
        // Verify version string is set by comparing to expected value
        assert_eq!(SIM_VERSION, "0.1.0");
    }

    #[test]
    fn test_schema_version() {
        assert_eq!(SCHEMA_VERSION, 1);
    }
}
