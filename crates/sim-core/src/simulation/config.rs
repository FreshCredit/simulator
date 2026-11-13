use serde::{Deserialize, Serialize};

use crate::birthrate::BirthrateConfig;
use crate::environment::EnvironmentConfig;
use crate::generation::GenerationMode;
use crate::parameters::SimulationParameters;
use crate::protocol::VerificationMode;
use crate::toggles::ToggleConfig;
use crate::types::Tick;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Simulation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    /// Random seed for deterministic execution (SIM-004)
    pub seed: u64,
    /// Environment configuration
    pub environment: EnvironmentConfig,
    /// Toggle configuration
    pub toggles: ToggleConfig,
    /// Maximum ticks to run
    pub max_ticks: Tick,
    /// Energy consumption per tick per §7.4.0 line 3755
    pub energy_consumption_rate: f32,
    /// Movement cost per cell per §7.4.0 line 3756
    pub movement_cost: f32,
    // === Stage 1 (Biology) parameters ===
    /// Gompertz baseline mortality per §6.21.4 line 2581
    pub mortality_baseline: f32,
    /// Gompertz aging parameter β per §6.21.4 line 2582 (range [0.08, 0.12])
    pub mortality_aging_beta: f32,
    /// Reproduction energy threshold per §7.4.0 line 3757
    pub reproduction_threshold: f32,
    /// Energy cost to reproduce
    pub reproduction_cost: f32,
    /// Mutation rate per locus per §7.5 line 3860 (Drake et al. 1998: 0.001)
    pub mutation_rate: f32,
    // === Protocol Mode (P0-2) ===
    /// Verification mode: Decentralized (blockchain) vs Centralized (single authority)
    /// Per §6.23.5a: Comparison baseline for identity theft protection
    pub verification_mode: VerificationMode,
    // === Birthrate Mechanism ===
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Birthrate configuration for probabilistic reproduction
    /// When enabled, replaces fixed-threshold reproduction
    pub birthrate_config: Option<BirthrateConfig>,
    // === Wright-Fisher Discrete Generation Mode ===
    /// Generation mode for reproduction: Overlapping (default) or Discrete
    /// Discrete mode implements true Wright-Fisher non-overlapping generations
    pub generation_mode: GenerationMode,
    // === Layer 4: Neurobiology (DOCAS) ===
    /// DOCAS stimulus response rate (alpha) per §8.2.1
    pub docas_alpha_base: f32,
    /// Trust curvature window size for instability detection
    pub trust_curvature_window: usize,
    // === Layer 5: Psychology ===
    /// DMN planning horizon in ticks
    pub planning_horizon: Tick,
    /// Emotion decay rate per tick (0.0-1.0)
    pub emotion_decay_rate: f32,
    // === Layer 6: Philosophy ===
    /// Worldview drift rate per tick
    pub worldview_drift_rate: f32,
    // === Layer 7: Ecology ===
    /// Trust decay rate per tick
    pub trust_decay_rate: f32,
    /// Minimum reputation for credit access
    pub reputation_threshold: f32,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    // === Layer 8: Economy ===
    /// Maximum credit per agent
    pub credit_limit: f32,
    /// Base interest rate for lending
    pub interest_rate_base: f32,
    /// Transaction cost per economic interaction
    pub transaction_cost: f32,
    // === Layer 9: Technology ===
    /// Ticks per blockchain block
    pub block_time: Tick,
    /// Consensus threshold (fraction of attestations needed)
    pub consensus_threshold: f32,
    // === Memory Management ===
    /// Cleanup interval in ticks (0 = disabled, default: 1000)
    pub cleanup_interval: u32,
    /// Maximum dead agents before forced cleanup (default: 500)
    pub max_dead_agents: usize,
    /// Preserve lineage summaries when cleaning (default: true)
    pub preserve_lineage: bool,
    // === Movement & Social Forces ===
    /// Movement speed multiplier (units per tick)
    pub movement_speed: f32,
    /// Social force influence factor (0.0-1.0): how much social forces affect movement
    pub social_force_influence: f32,
    // === Version 4.0: Optogenetic Control & Dual-Branch Visual Cloaking ===
    /// Optogenetic causal intervention configuration
    pub optogenetics: crate::optogenetics::OptogeneticConfig,
    /// Visual credit agreement stimulus configuration (Sim D16)
    pub visual_cloaking: crate::visual_cloaking::VisualAgreementStimulus,
    /// Whether on-device Weather-Shield decision buffer is active (Sim D17)
    pub weather_shield_enabled: bool,
    /// Version 4.0: Baylor unconscious narrative processing & attractor drift (Sim D18)
    pub baylor: crate::baylor_gating::BaylorStateConfig,
    // === Civilization Challenge Levers ===
    /// Epigenetic methylation rate [0.0, 1.0]
    pub epigenetic_methylation_rate: f32,
    /// Circadian sleep deficit [0.0, 1.0]
    pub circadian_sleep_deficit: f32,
    /// Institutional wealth redistribution tax rate [0.0, 0.50]
    pub tax_redistribution_rate: f32,
    /// Bank run sunspot sensitivity [0.0, 2.0]
    pub bank_run_sensitivity: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for SimulationConfig {
    /// Default Phase 0 configuration (N=1 agent, 64×64 grid)
    fn default() -> Self {
        Self {
            seed: 42,
            environment: EnvironmentConfig::default(),
            toggles: ToggleConfig::stage_0(),
            max_ticks: 1000,
            energy_consumption_rate: 1.0, // TUNABLE: 1.0-2.0 per §7.4.0
            movement_cost: 0.5,           // TUNABLE: 0.5-1.0 per §7.4.0
            // Stage 1 defaults
            mortality_baseline: 0.00001,  // Per §6.21.4 line 2581
            mortality_aging_beta: 0.10,   // Per §6.21.4 line 2582 (β ∈ [0.08, 0.12])
            reproduction_threshold: 75.0, // TUNABLE: 50-100 per §7.4.0 line 3757
            reproduction_cost: 30.0,      // Energy transferred to offspring
            mutation_rate: 0.001,         // Per Drake et al. 1998
            verification_mode: VerificationMode::DecentralizedProtocol, // Default to blockchain
            birthrate_config: None,       // Disabled by default (use threshold-based)
            generation_mode: GenerationMode::default(), // Overlapping generations by default
            // Layer 4: Neurobiology
            docas_alpha_base: 0.5, // Mid-range stimulus response
            // TAG: surface=simulation owner=research-team rule=SIM-001
            trust_curvature_window: 10, // Default curvature detection window
            // Layer 5: Psychology
            planning_horizon: 10,     // Default DMN lookahead
            emotion_decay_rate: 0.95, // 5% emotion decay per tick
            // Layer 6: Philosophy
            worldview_drift_rate: 0.01, // Slow worldview evolution
            // Layer 7: Ecology
            trust_decay_rate: 0.001,   // Slow trust erosion
            reputation_threshold: 0.5, // Midpoint reputation requirement
            // Layer 8: Economy
            credit_limit: 100.0,      // Default credit cap
            interest_rate_base: 0.05, // 5% base interest
            transaction_cost: 0.1,    // Small transaction fee
            // Layer 9: Technology
            block_time: 10,            // Block every 10 ticks
            consensus_threshold: 0.67, // 2/3 consensus required
            // Memory management
            cleanup_interval: 0, // Disabled by default for backward compatibility
            max_dead_agents: 500, // Force cleanup if dead > 500
            preserve_lineage: true, // Keep lineage summaries
            // Movement & Social Forces
            movement_speed: 1.0,         // 1 unit per tick base speed
            social_force_influence: 0.3, // 30% social, 70% individual
            // Version 4.0 additions
            optogenetics: crate::optogenetics::OptogeneticConfig::default(),
            visual_cloaking: crate::visual_cloaking::VisualAgreementStimulus::default(),
            weather_shield_enabled: false,
            baylor: crate::baylor_gating::BaylorStateConfig::default(),
            epigenetic_methylation_rate: 0.05,
            circadian_sleep_deficit: 0.0,
            tax_redistribution_rate: 0.0,
            bank_run_sensitivity: 1.0,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl SimulationConfig {
    /// Create configuration from unified `SimulationParameters`
    /// This is the preferred way to create configs for parameter experimentation
    #[must_use]
    pub fn from_parameters(params: &SimulationParameters, seed: u64, max_ticks: Tick) -> Self {
        Self {
            seed,
            environment: EnvironmentConfig {
                width: 64, // Grid size controlled separately
                height: 64,
                depth: 1, // 2D for now
                resource_density: params.resources.density,
                initial_resource: params.resources.initial_amount,
                altitude_effects_enabled: false,
                weather_enabled: true,
            },
            toggles: ToggleConfig::stage_0(),
            max_ticks,
            energy_consumption_rate: params.energy.consumption_rate,
            movement_cost: params.energy.movement_cost,
            mortality_baseline: params.mortality.baseline,
            mortality_aging_beta: params.mortality.aging_beta,
            reproduction_threshold: params.reproduction.threshold,
            reproduction_cost: params.reproduction.cost,
            mutation_rate: params.reproduction.mutation_rate,
            verification_mode: VerificationMode::DecentralizedProtocol,
            birthrate_config: None,
            generation_mode: GenerationMode::default(),
            // Layer 4+ parameters use defaults
            ..Default::default()
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create configuration from parameters with custom toggles
    #[must_use]
    pub fn from_parameters_with_toggles(
        params: &SimulationParameters,
        seed: u64,
        max_ticks: Tick,
        toggles: ToggleConfig,
    ) -> Self {
        let mut config = Self::from_parameters(params, seed, max_ticks);
        config.toggles = toggles;
        config
    }

    /// Create a centralized verification baseline configuration
    /// Per P0-2: For comparison experiments with decentralized blockchain
    #[must_use]
    pub fn centralized_baseline(seed: u64, max_ticks: Tick) -> Self {
        let mut toggles = ToggleConfig::protocol_on(); // Protocol enabled
        toggles.protocol_enabled = true; // But using centralized verification
        Self {
            verification_mode: VerificationMode::CentralizedOracle,
            toggles,
            ..Self::default()
        }
        .with_seed_and_ticks(seed, max_ticks)
    }

    /// Create a decentralized verification configuration
    /// Per P0-2: For comparison experiments with centralized baseline
    #[must_use]
    pub fn decentralized_protocol(seed: u64, max_ticks: Tick) -> Self {
        Self {
            verification_mode: VerificationMode::DecentralizedProtocol,
            toggles: ToggleConfig::protocol_on(),
            ..Self::default()
        }
        .with_seed_and_ticks(seed, max_ticks)
    }

    /// Helper to set seed and `max_ticks`
    const fn with_seed_and_ticks(mut self, seed: u64, max_ticks: Tick) -> Self {
        self.seed = seed;
        self.max_ticks = max_ticks;
        self
    }
}
