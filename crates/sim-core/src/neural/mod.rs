//! Neural Validation Module
//!
//! Provides computational neuroscience integration for validating DOCAS parameters
//! against realistic neural dynamics. Uses Nengo/Brian2 as backends for offline
//! calibration (not real-time simulation).
//!
//! Architecture:
//! - Backend trait for pluggable neural simulators
//! - Representative agent sampling
//! - DOCAS parameter calibration
//! - BOLD signal generation for fMRI validation
//!
//! Per Phase B Research: Neural validation improves population accuracy through
//! cross-scale validation and parameter generalization.

// TAG: surface=simulation owner=research-team rule=SIM-001
pub mod backend;
pub mod bold;
pub mod brain_mapping;
pub mod brian2_backend;
pub mod calibration;
pub mod federated_learning;
pub mod fl_attacks;
pub mod fmri_feedback;
pub mod glial;
pub mod glutamate;
pub mod hodgkin_huxley;
pub mod multiscale;
pub mod neural_mass;
pub mod pde4b_pathway;
pub mod plasticity;
pub mod prediction_market;

pub mod quantum;
pub mod realtime;
pub mod sampling;
pub mod visualization;

pub use backend::{BackendConfig, DerivedDocasParams, NeuralBackend, SimulationResult};
pub use bold::{BoldConfig, BoldGenerator};
pub use brain_mapping::{
    BrainRegionActivation, BrainRegionDatabase, DocasBrainMapper, MniCoordinates,
    NeuralSocialBridge, NeuromodulatorType, SocialBehaviorPrediction, SocialFunction,
};
pub use brian2_backend::{Brian2Backend, Neuromodulator};
pub use calibration::{CalibrationResult, Calibrator};
pub use federated_learning::{
    // TAG: surface=simulation owner=research-team rule=SIM-001
    AggregationStrategy,
    ConvergenceTracker,
    FederatedCoordinator,
    FederatedLearning,
    FederatedLearningError,
    FederatedRoundStats,
    LocalParticipant,
    TrainingSample,
};
pub use fl_attacks::{
    AttackMetrics, AttackModel, AuditReport, BackdoorAttack, ByzantineAttack, ByzantineType,
    FLAttack, FLAttackSimulator, GradientLeakageAttack, MembershipInferenceAttack, ModelAPI,
    ModelInversionAttack, PrivacyAuditor, SimpleModel,
};
pub use fmri_feedback::{
    FeedbackOutput, FeedbackRegion, FmriFeedbackController, FmriParams, NeurofeedbackProtocol,
};
pub use glial::{
    ActivityHistory, Astrocyte, AxonSegmentId, AxonType, GlialStatistics, GlialSystem,
    NeuralPosition, Oligodendrocyte,
};
pub use glutamate::{GlutamateStatistics, GlutamateSystem, PlasticityState, AMPAR, GABAR, NMDAR};
pub use hodgkin_huxley::{
    simulate_current_pulse,
    simulate_extended_pulse,
    temperature_correction,
    ExtendedHHModel,
    ExtendedHHStateSummary,
    GatingType,
    HHStateSummary,
    HodgkinHuxleyModel,
    IonChannel,
    // TAG: surface=simulation owner=research-team rule=SIM-001
    NeuromodulatorHHModel,
    FARADAY,
    GAS_CONSTANT,
    HH_Q10,
    HH_TEMPERATURE,
};
pub use multiscale::{
    BrainRegion, BrainScale, CellularNeuron, CorticalColumn, MolecularState, MultiscaleBridge,
    WholeBrainModel,
};
pub use neural_mass::{NeuralMassNetwork, NeuralMassParams, WilsonCowanState};
pub use pde4b_pathway::{
    compute_camp, compute_camp_explicit, compute_maturation_rate_pathway, compute_modulation,
    compute_modulation_explicit, compute_pathway, compute_pcreb, compute_pcreb_explicit,
    compute_pde4b_modulation, compute_pka_activation, compute_pka_activation_explicit,
    PDE4BPathwayParams, PathwayOutput, PathwayValidationError,
};
pub use plasticity::{
    CalciumParams, ConsolidationParams, HomeostaticParams, MetaplasticityParams, NeuronId,
    PlasticityBuilder, PlasticityStats, PlasticityTag, StdpParams, Synapse, SynapticPlasticity,
};
pub use quantum::{QuantumAgentBackend, QuantumNeuralWeights, QuantumRegister, Qubit};
pub use realtime::{
    NeuralEnsemble, RealTimeConfig, RealTimeNeuralBackend, RealTimeNeuralNetwork, SpikingNeuron,
};
pub use sampling::{AgentSampler, SamplingStrategy};
pub use visualization::{NeuralVisualizer, VisualizationOverlay};

use crate::agent::Agent;
use rand::rngs::StdRng;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Configuration for neural validation
#[derive(Debug, Clone)]
pub struct NeuralValidationConfig {
    /// Backend to use
    pub backend: BackendType,
    /// Enable/disable validation
    pub enabled: bool,
    /// Real-time mode (lightweight per-tick simulation)
    pub realtime_mode: bool,
    /// Real-time configuration (when `realtime_mode` is true)
    pub realtime_config: RealTimeConfig,
    /// Sampling strategy
    pub sampling: SamplingStrategy,
    /// Calibration interval (ticks)
    pub calibration_interval: u64,
    /// BOLD signal generation
    pub generate_bold: bool,
    /// Visualization overlays
    pub visualization: bool,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for NeuralValidationConfig {
    fn default() -> Self {
        Self {
            backend: BackendType::Nengo,
            enabled: false,       // Disabled by default (offline only)
            realtime_mode: false, // Real-time disabled by default
            realtime_config: RealTimeConfig::default(),
            sampling: SamplingStrategy::Representative { count: 10 },
            calibration_interval: 10000, // Every 10k ticks
            generate_bold: true,
            visualization: true,
        }
    }
}

/// Available neural backends
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    /// Nengo (Neural Engineering Framework)
    Nengo,
    /// Brian2 (Spiking neural networks)
    Brian2,
    /// Mock backend for testing
    Mock,
}

/// Neural validation engine
pub struct NeuralValidationEngine {
    config: NeuralValidationConfig,
    backend: Box<dyn NeuralBackend>,
    calibrator: Calibrator,
    bold_generator: BoldGenerator,
    last_calibration_tick: u64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl std::fmt::Debug for NeuralValidationEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NeuralValidationEngine")
            .field("config", &self.config)
            .field("calibrator", &self.calibrator)
            .field("bold_generator", &self.bold_generator)
            .field("last_calibration_tick", &self.last_calibration_tick)
            .finish_non_exhaustive()
    }
}

impl NeuralValidationEngine {
    /// Create new validation engine
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn new(config: NeuralValidationConfig) -> Result<Self, NeuralValidationError> {
        if !config.enabled {
            return Err(NeuralValidationError::Disabled);
        }

        let backend: Box<dyn NeuralBackend> = match config.backend {
            BackendType::Nengo => Box::new(backend::NengoBackend::new()?),
            BackendType::Brian2 => Box::new(backend::Brian2Backend::new()?),
            BackendType::Mock => Box::new(backend::MockBackend::new()),
        };

        let calibrator = Calibrator::new();
        let bold_generator = BoldGenerator::new(BoldConfig::default());

        Ok(Self {
            config,
            backend,
            calibrator,
            bold_generator,
            last_calibration_tick: 0,
        })
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Run calibration if due
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn maybe_calibrate(
        &mut self,
        agents: &[Agent],
        current_tick: u64,
        rng: &mut StdRng,
    ) -> Result<Option<CalibrationResult>, NeuralValidationError> {
        if current_tick.saturating_sub(self.last_calibration_tick)
            < self.config.calibration_interval
        {
            return Ok(None);
        }

        let result = self.calibrate(agents, rng)?;
        self.last_calibration_tick = current_tick;

        Ok(Some(result))
    }

    /// Run full calibration cycle
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn calibrate(
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        &mut self,
        agents: &[Agent],
        rng: &mut StdRng,
    ) -> Result<CalibrationResult, NeuralValidationError> {
        // Step 1: Sample representative agents
        let sampled_agents = AgentSampler::sample(agents, &self.config.sampling, rng);

        // Step 2: Run neural simulation for each sampled agent
        let mut neural_results = Vec::new();
        for agent in &sampled_agents {
            let result = self.backend.simulate_agent(agent)?;
            neural_results.push(result);
        }

        // Step 3: Calibrate DOCAS parameters
        let calibration = self.calibrator.calibrate(&neural_results)?;

        // Step 4: Generate BOLD signals if enabled
        if self.config.generate_bold {
            for result in &neural_results {
                let _bold = self.bold_generator.generate(result)?;
                // Store for visualization
            }
        }

        Ok(calibration)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get backend name
    #[must_use]
    pub fn backend_name(&self) -> &str {
        self.backend.name()
    }

    /// Check if enabled
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

/// Neural validation errors
#[derive(Debug)]
pub enum NeuralValidationError {
    /// disabled.
    Disabled,
    /// backend.
    Backend(String),
    /// calibration.
    Calibration(String),
    /// bold generation.
    BoldGeneration(String),
    /// sampling.
    Sampling(String),
    /// io.
    Io(std::io::Error),
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl std::fmt::Display for NeuralValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disabled => write!(f, "Neural validation is disabled"),
            Self::Backend(s) => write!(f, "Backend error: {s}"),
            Self::Calibration(s) => write!(f, "Calibration error: {s}"),
            Self::BoldGeneration(s) => write!(f, "BOLD generation error: {s}"),
            Self::Sampling(s) => write!(f, "Sampling error: {s}"),
            Self::Io(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl std::error::Error for NeuralValidationError {}

impl From<std::io::Error> for NeuralValidationError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// Validation metrics for POM
#[derive(Debug, Clone, Default)]
pub struct ValidationMetrics {
    /// Neural-DOCAS correlation (should be > 0.5)
    pub neural_docas_correlation: f32,
    /// Prediction accuracy for social behavior
    pub behavior_prediction_auc: f32,
    /// BOLD signal realism (vs empirical fMRI)
    pub bold_realism_score: f32,
    /// Calibration convergence
    pub calibration_converged: bool,
    /// Number of calibration iterations
    pub calibration_iterations: u32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl ValidationMetrics {
    /// Validate against POM targets
    #[must_use]
    pub fn validate_pom(&self) -> PomValidationResult {
        PomValidationResult {
            neural_docas_valid: self.neural_docas_correlation > 0.5,
            behavior_prediction_valid: self.behavior_prediction_auc > 0.7,
            bold_realism_valid: self.bold_realism_score > 0.6,
            all_valid: self.neural_docas_correlation > 0.5
                && self.behavior_prediction_auc > 0.7
                && self.bold_realism_score > 0.6,
        }
    }
}

/// POM validation result
#[derive(Debug, Clone)]
pub struct PomValidationResult {
    /// neural docas valid.
    pub neural_docas_valid: bool,
    /// behavior prediction valid.
    pub behavior_prediction_valid: bool,
    /// bold realism valid.
    pub bold_realism_valid: bool,
    /// all valid.
    pub all_valid: bool,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neural_validation_config_default() {
        let config = NeuralValidationConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.backend, BackendType::Nengo);
        assert!(!config.realtime_mode);
        assert!(config.generate_bold);
        assert!(config.visualization);
    }

    #[test]
    fn test_backend_type_equality() {
        assert_eq!(BackendType::Nengo, BackendType::Nengo);
        assert_ne!(BackendType::Nengo, BackendType::Mock);
    }

    #[test]
    fn test_neural_validation_engine_disabled() {
        let config = NeuralValidationConfig::default();
        let result = NeuralValidationEngine::new(config);
        assert!(matches!(result, Err(NeuralValidationError::Disabled)));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_neural_validation_engine_mock() {
        let config = NeuralValidationConfig {
            enabled: true,
            backend: BackendType::Mock,
            ..Default::default()
        };
        let engine = NeuralValidationEngine::new(config).unwrap();
        assert_eq!(engine.backend_name(), "Mock");
        assert!(engine.is_enabled());
    }

    #[test]
    fn test_validation_metrics_all_valid() {
        let metrics = ValidationMetrics {
            neural_docas_correlation: 0.6,
            behavior_prediction_auc: 0.8,
            bold_realism_score: 0.7,
            calibration_converged: true,
            calibration_iterations: 10,
        };
        let result = metrics.validate_pom();
        assert!(result.neural_docas_valid);
        assert!(result.behavior_prediction_valid);
        assert!(result.bold_realism_valid);
        assert!(result.all_valid);
    }

    #[test]
    fn test_validation_metrics_invalid() {
        let metrics = ValidationMetrics {
            neural_docas_correlation: 0.4,
            behavior_prediction_auc: 0.6,
            bold_realism_score: 0.5,
            calibration_converged: false,
            calibration_iterations: 0,
        };
        let result = metrics.validate_pom();
        assert!(!result.neural_docas_valid);
        assert!(!result.behavior_prediction_valid);
        assert!(!result.bold_realism_valid);
        assert!(!result.all_valid);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_validation_metrics_partial() {
        let metrics = ValidationMetrics {
            neural_docas_correlation: 0.6,
            behavior_prediction_auc: 0.6,
            bold_realism_score: 0.7,
            calibration_converged: true,
            calibration_iterations: 5,
        };
        let result = metrics.validate_pom();
        assert!(result.neural_docas_valid);
        assert!(!result.behavior_prediction_valid);
        assert!(result.bold_realism_valid);
        assert!(!result.all_valid);
    }

    #[test]
    fn test_neural_validation_error_display() {
        let err = NeuralValidationError::Disabled;
        assert_eq!(format!("{err}"), "Neural validation is disabled");

        let err = NeuralValidationError::Backend("test".to_string());
        assert_eq!(format!("{err}"), "Backend error: test");

        let err = NeuralValidationError::Calibration("fail".to_string());
        assert_eq!(format!("{err}"), "Calibration error: fail");
    }
}
