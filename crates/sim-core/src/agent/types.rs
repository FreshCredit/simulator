//! Agent types: Cultural, game theory, and neural state types
//!
//! SIM-001: Isolation verified - no product dependencies

use serde::{Deserialize, Serialize};

use crate::types::{AgentId, Tick};

// === Cultural Types (Paper 7) ===

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Cultural quadrant from b × `τ_φ` matrix
/// Per Paper 7: Four cultural regimes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CulturalQuadrant {
    /// High bond density, high trust (Cooperative-Open)
    HighBondHighTrust,
    /// High bond density, low trust (Cooperative-Skeptical)
    HighBondLowTrust,
    /// Low bond density, high trust (Individualist-Open)
    LowBondHighTrust,
    /// Low bond density, low trust (Individualist-Skeptical)
    LowBondLowTrust,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Hofstede cultural dimensions
pub use crate::culture::individual::IndividualCulturalOrientation as HofstedeDimensions;

// === Game Theory Types (Paper 9) ===

/// Outcome of a single game interaction
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GameOutcome {
    /// Partner agent ID
    pub partner_id: AgentId,
    /// Game type played
    pub game_type: crate::games::GameType,
    /// Action taken by this agent
    pub my_action: bool, // true = cooperate, false = defect
    /// Action taken by partner
    pub partner_action: bool,
    /// Payoff received
    pub my_payoff: f32,
    /// Partner's payoff
    pub partner_payoff: f32,
    /// Tick when game occurred
    pub tick: Tick,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Prefrontal Cortex State - Neural maturation tracking
/// Per Research: `MATURITY_NEURAL_DEVELOPMENT_RESEARCH.md`
/// Models dlPFC (executive function) and vmPFC (value-based decisions) development
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefrontalCortex {
    /// Dorsolateral Prefrontal Cortex - Executive function, working memory
    pub dlpfc: DLPFCState,
    /// Ventromedial Prefrontal Cortex - Value-based decision making, social evaluation
    pub vmpfc: VMPFCState,
    /// Synaptic pruning progress (0.0 = start, 1.0 = complete)
    pub pruning_progress: f32,
    /// Myelination progress (0.0 = start, 1.0 = complete)
    pub myelination_progress: f32,
    /// Age when pruning completes (genome-controlled)
    pub pruning_completion_age: Tick,
    /// Age when myelination completes (genome-controlled)
    pub myelination_completion_age: Tick,
    /// Current maturation rate (affected by PDE4B, stress, resources)
    pub current_maturation_rate: f32,
    /// Chronic cortisol average (for stress effects)
    pub chronic_cortisol: f32,
}

impl Default for PrefrontalCortex {
    fn default() -> Self {
        Self {
            dlpfc: DLPFCState::default(),
            vmpfc: VMPFCState::default(),
            pruning_progress: 0.0,
            myelination_progress: 0.0,
            pruning_completion_age: 70, // Default: ~7 years human equivalent
            myelination_completion_age: 400, // Default: ~40 years human equivalent
            current_maturation_rate: 1.0,
            chronic_cortisol: 0.0,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Dorsolateral Prefrontal Cortex State
/// Responsible for executive function, working memory, cognitive control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DLPFCState {
    /// Development level (0.0 = neonate, 1.0 = fully mature)
    pub development: f32,
    /// Working memory capacity (increases with development)
    pub working_memory_capacity: u32,
    /// Cognitive control strength (0.0-1.0)
    pub cognitive_control_strength: f32,
    /// Age when dlPFC reaches functional maturity (genome-controlled)
    pub age_mature: Tick,
    /// Executive function efficiency (0.0-1.0)
    pub executive_efficiency: f32,
}

impl Default for DLPFCState {
    fn default() -> Self {
        Self {
            development: 0.0,
            working_memory_capacity: 1,      // Start minimal
            cognitive_control_strength: 0.1, // Start low
            age_mature: 300,                 // ~30 years human equivalent
            executive_efficiency: 0.1,
        }
    }
}
// TAG: surface=simulation owner=platform-team rule=GENERAL-001

/// Ventromedial Prefrontal Cortex State
/// Responsible for value-based decision making, social evaluation, risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VMPFCState {
    /// Development level (0.0 = neonate, 1.0 = fully mature)
    pub development: f32,
    /// Social evaluation accuracy (0.0-1.0)
    pub social_evaluation_accuracy: f32,
    /// Risk assessment calibration (0.0-1.0)
    pub risk_assessment_calibration: f32,
    /// Age when vmPFC reaches functional maturity (genome-controlled)
    pub age_mature: Tick,
    /// Value integration strength (0.0-1.0)
    pub value_integration_strength: f32,
}

impl Default for VMPFCState {
    fn default() -> Self {
        Self {
            development: 0.0,
            social_evaluation_accuracy: 0.2, // Some innate social capacity
            risk_assessment_calibration: 0.1,
            age_mature: 150, // ~15 years human equivalent
            value_integration_strength: 0.1,
        }
    }
}
