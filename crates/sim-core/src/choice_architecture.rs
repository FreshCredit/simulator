//! Choice Architecture Module - Thaler & Sunstein Behavioral Economics
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION choice architecture, not product features
//!
//! Per §6.4 lines 815-832: Implements behavioral economics effects without
//! directly modifying agent trust/honesty/cooperation values.
//!
//! Based on:
//! - Default effects (Johnson & Goldstein 2003)
//! - Framing effects (Kahneman & Tversky 1979)
//! - Cognitive load effects (Sweller 1988)
//! - Loss aversion / Endowment effect (Thaler & Sunstein §6.4)

use crate::agent::Agent;
use crate::dmn::Action;
use crate::types::D;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Nudge configuration for choice architecture
/// Per §8.2.1 lines 4866-4893: Choice architecture toggle contract
#[derive(Debug, Clone, Copy)]
pub struct NudgeConfig {
    /// Default boost: +60-90% weight for default actions
    pub default_boost: f32,
    /// Friction penalty: -10-30% per friction unit
    pub friction_penalty: f32,
    /// Salience boost: +20-50% for salient options
    pub salience_boost: f32,
    /// Cognitive cost multiplier for DMN budget
    pub cognitive_cost_factor: f32,
    /// Whether choice architecture is enabled
    pub enabled: bool,
}

impl Default for NudgeConfig {
    fn default() -> Self {
        Self {
            default_boost: 0.7,    // +70% for defaults (mid-range)
            friction_penalty: 0.2, // -20% per friction unit
            salience_boost: 0.35,  // +35% for salient (mid-range)
            cognitive_cost_factor: 1.0,
            enabled: true,
        }
    }
}

impl NudgeConfig {
    /// Create config with choice architecture disabled (OFF behavior)
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Create config with choice architecture enabled (ON behavior)
    #[must_use]
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }

    /// Create config from simulation dial parameters
    #[must_use]
    pub fn from_dials(dials: &crate::toggles::DialConfig) -> Self {
        Self {
            default_boost: (0.5 + dials.choice_default_bias).clamp(0.1, 1.5),
            friction_penalty: (dials.choice_friction_units as f32 * 0.1).clamp(0.0, 0.8),
            salience_boost: dials.choice_anchor_adjustment.clamp(0.0, 1.0),
            cognitive_cost_factor: (dials.choice_loss_aversion_lambda / 2.25).clamp(0.1, 5.0),
            enabled: true,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Types of framing for utility computation
/// Per Thaler & Sunstein §6.4: Same outcome framed differently changes preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FrameType {
    /// Neutral framing (no effect)
    #[default]
    Neutral,
    /// Loss framing ("you will lose X if...")
    Loss,
    /// Gain framing ("you will gain X if...")
    Gain,
    /// Mixed framing (both loss and gain aspects)
    Mixed,
}

/// Nudge type for choice architecture interventions
/// Per Thaler & Sunstein §6.4: Different nudge strategies
#[derive(Debug, Clone, PartialEq, Default)]
pub enum NudgeType {
    /// Default option bias - agents tend to stick with default
    /// default action.
    DefaultBias {
        /// default action.
        default_action: Action,
    },

    /// Simplify complex choices - limit options presented
    /// max options.
    Simplification {
        /// max options.
        max_options: usize,
    },

    /// Social proof - "most agents choose X"
    SocialProof {
        /// popular action.
        popular_action: Action,
        /// adoption rate.
        adoption_rate: f32,
    },

    /// Loss framing - "you will lose X if..."
    /// reference point.
    LossFraming {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        /// reference point.
        reference_point: f32,
    },

    /// Gain framing - "you will gain X if..."
    /// reference point.
    GainFraming {
        /// reference point.
        reference_point: f32,
    },

    /// Commitment device - pre-commit to action with penalty for deviation
    CommitmentDevice {
        /// committed action.
        committed_action: Action,
        /// penalty.
        penalty: f32,
    },

    /// Combination of multiple nudges
    Combined(Vec<NudgeType>),

    /// No nudge (neutral)
    #[default]
    None,
}

impl NudgeType {
    /// Get the default action for this nudge type, if any
    #[must_use]
    pub fn default_action(&self) -> Option<Action> {
        match self {
            Self::DefaultBias { default_action } => Some(*default_action),
            Self::SocialProof { popular_action, .. } => Some(*popular_action),
            Self::CommitmentDevice {
                committed_action, ..
            } => Some(*committed_action),
            Self::Combined(nudges) => {
                // Return first available default from combined nudges
                nudges.iter().find_map(Self::default_action)
            }
            _ => None,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if this nudge applies loss framing
    #[must_use]
    pub fn is_loss_framed(&self) -> bool {
        match self {
            Self::LossFraming { .. } => true,
            Self::Combined(nudges) => nudges.iter().any(Self::is_loss_framed),
            _ => false,
        }
    }

    /// Check if this nudge applies gain framing
    #[must_use]
    pub fn is_gain_framed(&self) -> bool {
        match self {
            Self::GainFraming { .. } => true,
            Self::Combined(nudges) => nudges.iter().any(Self::is_gain_framed),
            _ => false,
        }
    }

    /// Get the frame type for this nudge
    #[must_use]
    pub fn frame_type(&self) -> FrameType {
        match self {
            Self::LossFraming { .. } => FrameType::Loss,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            Self::GainFraming { .. } => FrameType::Gain,
            Self::Combined(nudges) => {
                let has_loss = nudges.iter().any(Self::is_loss_framed);
                let has_gain = nudges.iter().any(Self::is_gain_framed);
                match (has_loss, has_gain) {
                    (true, true) => FrameType::Mixed,
                    (true, false) => FrameType::Loss,
                    (false, true) => FrameType::Gain,
                    (false, false) => FrameType::Neutral,
                }
            }
            _ => FrameType::Neutral,
        }
    }
}

/// Context for choice architecture decisions
/// Contains all relevant state for computing nudge effects
#[derive(Debug, Clone, Default)]
pub struct Context {
    /// Current tick in simulation
    pub tick: u64,
    /// Current energy level
    pub energy: f32,
    /// Current DOCAS D (dopamine) level
    pub dopamine: f32,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Current DOCAS C (cortisol) level
    pub cortisol: f32,
    /// Social context: number of nearby agents
    pub nearby_agents: usize,
    /// Environmental richness at current position
    pub local_resources: f32,
    /// Previous action taken
    pub previous_action: Option<Action>,
    /// Whether agent is in a group
    pub in_group: bool,
    /// Current trust level (tau)
    pub trust_level: f32,
}

impl Context {
    /// Create a new context from an agent
    #[must_use]
    pub fn from_agent(agent: &Agent) -> Self {
        Self {
            tick: agent.age,
            energy: agent.energy,
            dopamine: agent.docas.levels[D],
            cortisol: agent.docas.levels[2], // C index
            nearby_agents: 0,                // Would need environment query
            local_resources: 0.0,            // Would need environment query
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            previous_action: None, // Would need agent history
            in_group: agent.group_id.is_some(),
            trust_level: agent.compute_trust(),
        }
    }

    /// Create a context with specified values
    #[must_use]
    pub const fn new(
        tick: u64,
        energy: f32,
        dopamine: f32,
        cortisol: f32,
        trust_level: f32,
    ) -> Self {
        Self {
            tick,
            energy,
            dopamine,
            cortisol,
            nearby_agents: 0,
            local_resources: 0.0,
            previous_action: None,
            in_group: false,
            trust_level,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Set nearby agents count
    #[must_use]
    pub const fn with_nearby_agents(mut self, count: usize) -> Self {
        self.nearby_agents = count;
        self
    }

    /// Set local resources
    #[must_use]
    pub const fn with_local_resources(mut self, resources: f32) -> Self {
        self.local_resources = resources;
        self
    }

    /// Set previous action
    #[must_use]
    pub const fn with_previous_action(mut self, action: Action) -> Self {
        self.previous_action = Some(action);
        self
    }

    /// Set group membership
    #[must_use]
    pub const fn with_in_group(mut self, in_group: bool) -> Self {
        self.in_group = in_group;
        self
    }
}

/// Thaler behavioral economics effects parameters
/// Per §6.4 lines 815-832: Core parameters for choice architecture
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThalerEffects {
    /// Loss aversion lambda (λ): losses hurt ~2.0-2.5x more than gains feel good
    /// Per Thaler & Sunstein §6.4 and Kahneman & Tversky 1979
    pub loss_aversion_lambda: f32,

    /// Default bias strength: how strongly agents favor default options
    /// Range 0.6-0.9 per §8.2.1
    pub default_bias_strength: f32,

    /// Social proof strength: how much agents follow majority behavior
    /// Range 0.2-0.5
    pub social_proof_strength: f32,

    /// Framing effect size: how much framing changes utility perception
    /// Range 0.3-0.6
    pub framing_effect_size: f32,

    /// Cognitive load impact: how cognitive load affects decision quality
    /// Range 0.5-1.0
    pub cognitive_load_impact: f32,
}

impl Default for ThalerEffects {
    /// Default Thaler effects per §6.4
    fn default() -> Self {
        Self {
            loss_aversion_lambda: 2.25,  // Classic Kahneman & Tversky value
            default_bias_strength: 0.75, // Mid-range default bias
            social_proof_strength: 0.35, // Moderate social influence
            framing_effect_size: 0.45,   // Moderate framing effect
            cognitive_load_impact: 0.75, // Significant cognitive impact
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl ThalerEffects {
    /// Create effects with minimal nudge impact (for testing)
    #[must_use]
    pub const fn minimal() -> Self {
        Self {
            loss_aversion_lambda: 1.0,
            default_bias_strength: 0.0,
            social_proof_strength: 0.0,
            framing_effect_size: 0.0,
            cognitive_load_impact: 0.0,
        }
    }

    /// Create effects with maximum nudge impact (for testing boundary conditions)
    #[must_use]
    pub const fn maximal() -> Self {
        Self {
            loss_aversion_lambda: 3.0,
            default_bias_strength: 1.0,
            social_proof_strength: 1.0,
            framing_effect_size: 1.0,
            cognitive_load_impact: 1.0,
        }
    }

    /// Validate parameters are in reasonable ranges
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn validate(&self) -> Result<(), &'static str> {
        if !(1.0..=5.0).contains(&self.loss_aversion_lambda) {
            return Err("loss_aversion_lambda should be in [1.0, 5.0]");
        }
        if !(0.0..=2.0).contains(&self.default_bias_strength) {
            return Err("default_bias_strength should be in [0.0, 2.0]");
        }
        if !(0.0..=1.0).contains(&self.social_proof_strength) {
            return Err("social_proof_strength should be in [0.0, 1.0]");
        }
        if !(0.0..=1.0).contains(&self.framing_effect_size) {
            return Err("framing_effect_size should be in [0.0, 1.0]");
        }
        if !(0.0..=2.0).contains(&self.cognitive_load_impact) {
            return Err("cognitive_load_impact should be in [0.0, 2.0]");
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        }
        Ok(())
    }
}

/// Choice Architecture Adapter trait
/// Per §6.4 lines 815-832: Complete interface for choice architecture
///
/// This trait defines how choice architecture interfaces with the DMN
/// to modify action selection through nudges rather than hard constraints.
pub trait ChoiceArchitectureAdapter {
    /// Filter available actions based on context
    /// Returns the subset of actions that should be considered
    fn available_actions(&self, agent: &Agent, context: &Context) -> Vec<Action>;

    /// Add friction to certain actions (0.0 = no friction, 1.0 = blocked)
    /// Friction reduces the probability of selecting an action
    fn action_friction(&self, action: &Action, context: &Context) -> f32;

    /// Get default action (what happens if agent doesn't choose)
    /// Per Johnson & Goldstein 2003: Default effect is powerful
    fn default_action(&self, context: &Context) -> Option<Action>;

    /// Compute salience weight for an action (attention-grabbing)
    /// Dopamine (D) modulates attention to rewarding actions
    fn salience(&self, action: &Action, context: &Context) -> f32;

    /// Compute cognitive cost of an action (affects DMN budget)
    /// Higher cost = more DMN budget consumed
    fn cognitive_cost(&self, action: &Action, context: &Context) -> f32;

    /// Apply framing effects to utility computation
    /// Same outcome framed as loss vs gain changes preference
    fn frame_utility(&self, utility: f32, context: &Context, frame: FrameType) -> f32;

    /// Apply endowment effect (loss aversion)
    /// Losses hurt more than equivalent gains feel good
    /// Per Thaler & Sunstein §6.4: λ ≈ 2.25
    fn apply_endowment_effect(&self, value: f32, is_loss: bool) -> f32;

    /// Get the Thaler effects parameters
    fn thaler_effects(&self) -> &ThalerEffects;

    /// Get the current nudge type
    fn nudge_type(&self) -> &NudgeType;

    /// Check if choice architecture is enabled
    fn is_enabled(&self) -> bool;
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Standard implementation of `ChoiceArchitectureAdapter`
/// Per §6.4: Default choice architecture with configurable nudges
#[derive(Debug, Clone)]
pub struct StandardChoiceArchitecture {
    /// Current nudge configuration
    pub nudge: NudgeType,
    /// Thaler effects parameters
    pub effects: ThalerEffects,
    /// Whether choice architecture is enabled
    pub enabled: bool,
    /// Configuration parameters
    pub config: NudgeConfig,
}

impl Default for StandardChoiceArchitecture {
    fn default() -> Self {
        Self {
            nudge: NudgeType::DefaultBias {
                default_action: Action::Stay,
            },
            effects: ThalerEffects::default(),
            enabled: true,
            config: NudgeConfig::default(),
        }
    }
}

impl StandardChoiceArchitecture {
    /// Create a new default choice architecture with built-in nudge
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with specific nudge type
    #[must_use]
    pub fn with_nudge(nudge: NudgeType) -> Self {
        Self {
            nudge,
            ..Default::default()
        }
    }

    /// Create with specific Thaler effects
    #[must_use]
    pub fn with_effects(effects: ThalerEffects) -> Self {
        Self {
            effects,
            ..Default::default()
        }
    }

    /// Create disabled (OFF behavior)
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Enable choice architecture
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Disable choice architecture
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Set nudge type
    pub fn set_nudge(&mut self, nudge: NudgeType) {
        self.nudge = nudge;
    }

    /// Compute effective friction considering agent susceptibility
    #[allow(dead_code)]
    fn effective_friction(&self, base_friction: f32, _agent: &Agent) -> f32 {
        if !self.enabled {
            return 0.0;
        }
        // Agent's genome affects friction sensitivity
        let susceptibility = _agent.genome.default_susceptibility();
        base_friction * susceptibility
    }

    /// Compute effective salience considering agent state
    #[allow(dead_code)]
    fn effective_salience(&self, base_salience: f32, _agent: &Agent, context: &Context) -> f32 {
        if !self.enabled {
            return 1.0; // Uniform salience when disabled
        }
        // Dopamine modulates attention
        let dopamine_boost = context.dopamine * 0.3;
        // Cortisol reduces attention to non-threats
        let cortisol_penalty = context.cortisol * 0.2;
        (base_salience + dopamine_boost - cortisol_penalty).clamp(0.1, 2.0)
    }
}

impl ChoiceArchitectureAdapter for StandardChoiceArchitecture {
    fn available_actions(&self, agent: &Agent, _context: &Context) -> Vec<Action> {
        use crate::dmn::Direction;

        let mut actions = vec![Action::Stay];

        // Movement actions always available
        for dir in Direction::all() {
            actions.push(Action::Move(dir));
        }

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Reproduction only if energy sufficient
        if agent.energy >= 50.0 {
            actions.push(Action::Reproduce);
        }

        // Apply simplification nudge if present
        match &self.nudge {
            NudgeType::Simplification { max_options } if self.enabled => {
                // Keep default action and top salient actions
                let default = self.default_action(_context);
                if actions.len() > *max_options {
                    // Simple heuristic: keep default + random sample
                    let mut simplified = Vec::new();
                    if let Some(d) = default {
                        simplified.push(d);
                    }
                    // Add some movement options
                    simplified.push(Action::Move(Direction::North));
                    simplified.push(Action::Move(Direction::East));
                    simplified.push(Action::Stay);
                    if agent.energy >= 50.0 && simplified.len() < *max_options {
                        simplified.push(Action::Reproduce);
                    }
                    simplified
                } else {
                    actions
                    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                }
            }
            _ => actions,
        }
    }

    fn action_friction(&self, action: &Action, _context: &Context) -> f32 {
        if !self.enabled {
            return 0.0;
        }

        match action {
            // Stay has no friction (default-like behavior)
            Action::Stay => 0.0,
            // Movement has low friction
            Action::Move(_) => 0.2,
            // Reproduction has high friction (costly decision)
            Action::Reproduce => 0.8,
        }
    }

    fn default_action(&self, _context: &Context) -> Option<Action> {
        if !self.enabled {
            return None;
        }

        // Use nudge's default if available
        self.nudge.default_action().or(Some(Action::Stay))
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn salience(&self, action: &Action, context: &Context) -> f32 {
        if !self.enabled {
            return 1.0; // Uniform salience when disabled
        }

        match action {
            // Reproduction is salient when energy high and D high
            Action::Reproduce if context.energy > 80.0 => 0.5 + context.dopamine * 0.3,
            Action::Reproduce => 0.3,
            // Movement toward resources is salient with high D
            Action::Move(_) => 0.4 + context.dopamine * 0.2,
            // Stay is low salience (boring)
            Action::Stay => 0.2,
        }
    }

    fn cognitive_cost(&self, action: &Action, _context: &Context) -> f32 {
        match action {
            // Stay requires minimal cognition
            Action::Stay => 0.5,
            // Movement requires some planning
            Action::Move(_) => 1.0,
            // Reproduction requires significant deliberation
            Action::Reproduce => 2.0,
        }
    }

    fn frame_utility(&self, utility: f32, _context: &Context, frame: FrameType) -> f32 {
        if !self.enabled {
            return utility;
        }

        let effect_size = self.effects.framing_effect_size;

        match frame {
            FrameType::Neutral => utility,
            FrameType::Loss => {
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                // Loss framing amplifies negative utilities
                if utility < 0.0 {
                    utility * (1.0 + effect_size)
                } else {
                    utility * (1.0 - effect_size * 0.5)
                }
            }
            FrameType::Gain => {
                // Gain framing amplifies positive utilities
                if utility > 0.0 {
                    utility * (1.0 + effect_size)
                } else {
                    utility * (1.0 - effect_size * 0.5)
                }
            }
            FrameType::Mixed => {
                // Mixed framing has reduced effect
                utility * (1.0 + effect_size * 0.3)
            }
        }
    }

    fn apply_endowment_effect(&self, value: f32, is_loss: bool) -> f32 {
        if !self.enabled {
            return value;
        }

        // Per Thaler & Sunstein §6.4: losses hurt λx more than gains feel good
        let lambda = self.effects.loss_aversion_lambda;
        if is_loss {
            value * lambda
        } else {
            value // Gains valued at face value
        }
    }

    fn thaler_effects(&self) -> &ThalerEffects {
        &self.effects
    }

    fn nudge_type(&self) -> &NudgeType {
        &self.nudge
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get friction level for an action (legacy function)
/// Per §6.4 line 828: `action_friction()` -> f32, 0.0 = no friction
#[must_use]
pub const fn action_friction(action: &Action) -> f32 {
    match action {
        // Stay has no friction (default-like behavior)
        Action::Stay => 0.0,
        // Movement has low friction
        Action::Move(_) => 0.2,
        // Reproduction has high friction (costly decision)
        Action::Reproduce => 0.8,
    }
}

/// Get default action for a context (legacy function)
/// Per §6.4 line 829: `default_action()` -> `Option<Action>`
/// Default is to Stay (status quo bias)
#[must_use]
pub const fn default_action() -> Action {
    Action::Stay
}

/// Get salience of an action given agent state (legacy function)
/// Per §6.4 line 830: `salience()` -> f32, attention weight
/// Dopamine (D) modulates attention to rewarding actions
#[must_use]
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
pub fn action_salience(action: &Action, agent: &Agent) -> f32 {
    let d_level = agent.docas.levels[D];

    match action {
        // Reproduction is salient when energy high and D high
        Action::Reproduce if agent.energy > 80.0 => 0.5 + d_level * 0.3,
        Action::Reproduce => 0.3,
        // Movement toward resources is salient with high D
        Action::Move(_) => 0.4 + d_level * 0.2,
        // Stay is low salience (boring)
        Action::Stay => 0.2,
    }
}

/// Get cognitive cost of an action for DMN budget (legacy function)
/// Per §6.4 line 831: `cognitive_cost()` -> f32, DMN budget modifier
#[must_use]
pub const fn cognitive_cost(action: &Action) -> f32 {
    match action {
        // Stay requires minimal cognition
        Action::Stay => 0.5,
        // Movement requires some planning
        Action::Move(_) => 1.0,
        // Reproduction requires significant deliberation
        Action::Reproduce => 2.0,
    }
}

/// Compute DMN budget reduction from cognitive costs
/// Per §6.4: Cognitive costs reduce available DMN budget
pub fn compute_dmn_budget_reduction(
    actions: &[Action],
    adapter: &dyn ChoiceArchitectureAdapter,
    context: &Context,
) -> f32 {
    actions
        .iter()
        .map(|a| adapter.cognitive_cost(a, context))
        .sum()
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Apply Thaler nudges to weight action candidates
/// Per §8.2.1 lines 4866-4893: `choice_arch_on` implementation
#[must_use]
pub fn apply_nudges(
    candidates: &[(Action, f32)],
    agent: &Agent,
    config: &NudgeConfig,
) -> Vec<(Action, f32)> {
    let default = default_action();
    let susceptibility = agent.genome.default_susceptibility();

    candidates
        .iter()
        .map(|&(action, base_value)| {
            let mut weight = base_value;

            // Default effect: boost default action
            // Per §4877-4878: +60-90% weight
            if action == default {
                weight *= 1.0 + config.default_boost * susceptibility;
            }

            // Friction effect: penalize high-friction actions
            // Per §4881-4883: -10-30% per friction unit
            let friction = action_friction(&action);
            weight *= (1.0 - config.friction_penalty * friction).max(0.1);

            // Salience effect: boost salient actions
            // Per §4885-4888: D modulates attention
            let salience = action_salience(&action, agent);
            weight *= 1.0 + salience * config.salience_boost;

            (action, weight)
        })
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
        .collect()
}

/// Apply Thaler nudges using the full adapter interface
/// Per §8.2.1 lines 4866-4893: Full choice architecture integration
pub fn apply_nudges_with_adapter(
    candidates: &[(Action, f32)],
    agent: &Agent,
    context: &Context,
    adapter: &dyn ChoiceArchitectureAdapter,
) -> Vec<(Action, f32)> {
    if !adapter.is_enabled() {
        return uniform_weights(candidates);
    }

    let default = adapter.default_action(context);
    let susceptibility = agent.genome.default_susceptibility();
    let effects = adapter.thaler_effects();

    candidates
        .iter()
        .map(|&(action, base_value)| {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            let mut weight = base_value;

            // Apply framing effects to utility
            let frame = adapter.nudge_type().frame_type();
            weight = adapter.frame_utility(weight, context, frame);

            // Default effect: boost default action
            if Some(action) == default {
                weight *= 1.0 + effects.default_bias_strength * susceptibility;
            }

            // Friction effect: penalize high-friction actions
            let friction = adapter.action_friction(&action, context);
            weight *= (1.0 - 0.2 * friction).max(0.1);

            // Salience effect: boost salient actions
            let salience = adapter.salience(&action, context);
            weight *= 1.0 + salience * 0.35;

            // Apply endowment effect for actions with potential losses
            if weight < 0.0 {
                weight = adapter.apply_endowment_effect(weight, true);
            }

            (action, weight)
        })
        .collect()
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// OFF behavior: uniform action distribution
/// Per §8.2.1 line 4129: Choice Architecture OFF = uniform weights
#[must_use]
pub fn uniform_weights(candidates: &[(Action, f32)]) -> Vec<(Action, f32)> {
    candidates
        .iter()
        // TAG: surface=simulation owner=research-team rule=SIM-001
        .map(|&(action, _)| (action, 1.0))
        .collect()
}

/// Compute loss aversion adjusted utility
/// Per Thaler & Sunstein §6.4: Losses weighted by λ
#[must_use]
pub fn loss_aversion_utility(gain: f32, loss: f32, lambda: f32) -> f32 {
    gain - lambda * loss
}

/// Choice Architecture Manager
/// Central coordinator for choice architecture effects
#[derive(Debug, Clone)]
pub struct ChoiceArchitectureManager {
    /// Current adapter implementation
    adapter: StandardChoiceArchitecture,
    /// History of nudges applied
    nudge_history: Vec<(u64, NudgeType)>,
    /// Statistics tracking
    stats: ChoiceArchitectureStats,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

/// Statistics for choice architecture
#[derive(Debug, Clone, Default)]
pub struct ChoiceArchitectureStats {
    /// Number of times default was selected
    pub default_selections: u64,
    /// Number of times friction prevented an action
    pub friction_blocks: u64,
    /// Total cognitive cost expended
    pub total_cognitive_cost: f32,
    /// Number of nudges applied
    pub nudges_applied: u64,
}

impl ChoiceArchitectureManager {
    /// Create a new manager with default adapter
    #[must_use]
    pub fn new() -> Self {
        Self {
            adapter: StandardChoiceArchitecture::default(),
            nudge_history: Vec::new(),
            stats: ChoiceArchitectureStats::default(),
        }
    }

    /// Create with specific adapter
    #[must_use]
    pub fn with_adapter(adapter: StandardChoiceArchitecture) -> Self {
        Self {
            adapter,
            nudge_history: Vec::new(),
            stats: ChoiceArchitectureStats::default(),
        }
    }

    /// Enable choice architecture
    pub fn enable(&mut self) {
        self.adapter.enable();
    }

    /// Disable choice architecture
    pub fn disable(&mut self) {
        self.adapter.disable();
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if enabled
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.adapter.is_enabled()
    }

    /// Set nudge type
    pub fn set_nudge(&mut self, nudge: NudgeType, tick: u64) {
        self.adapter.set_nudge(nudge.clone());
        self.nudge_history.push((tick, nudge));
    }

    /// Get current nudge type
    #[must_use]
    pub const fn current_nudge(&self) -> &NudgeType {
        &self.adapter.nudge
    }

    /// Get adapter reference
    #[must_use]
    pub const fn adapter(&self) -> &StandardChoiceArchitecture {
        &self.adapter
    }

    /// Get mutable adapter reference
    pub fn adapter_mut(&mut self) -> &mut StandardChoiceArchitecture {
        &mut self.adapter
    }

    /// Get statistics
    #[must_use]
    pub const fn stats(&self) -> &ChoiceArchitectureStats {
        &self.stats
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    /// Record a default selection
    pub fn record_default_selection(&mut self) {
        self.stats.default_selections += 1;
    }

    /// Record a friction block
    pub fn record_friction_block(&mut self) {
        self.stats.friction_blocks += 1;
    }

    /// Record cognitive cost
    pub fn record_cognitive_cost(&mut self, cost: f32) {
        self.stats.total_cognitive_cost += cost;
    }

    /// Record nudge application
    pub fn record_nudge_applied(&mut self) {
        self.stats.nudges_applied += 1;
    }

    /// Apply choice architecture to action candidates
    pub fn apply_to_candidates(
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        &mut self,
        candidates: &[(Action, f32)],
        agent: &Agent,
        context: &Context,
    ) -> Vec<(Action, f32)> {
        let result = apply_nudges_with_adapter(candidates, agent, context, &self.adapter);
        self.record_nudge_applied();
        result
    }

    /// Get the best action considering choice architecture
    pub fn select_action(
        &mut self,
        candidates: &[(Action, f32)],
        agent: &Agent,
        context: &Context,
    ) -> Action {
        let weighted = self.apply_to_candidates(candidates, agent, context);

        // Sort by weight descending
        let mut sorted = weighted;
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return best or default
        sorted
            .first()
            .map(|(a, _)| *a)
            .or_else(|| self.adapter.default_action(context))
            .unwrap_or(Action::Stay)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for ChoiceArchitectureManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::Agent;
    use crate::genome::Genome;
    use crate::types::{Position, Sex};

    fn create_test_agent() -> Agent {
        Agent::new(
            1,
            Position::new_2d(32.0, 32.0),
            Genome::default(),
            0,
            Sex::Male,
        )
    }

    fn create_test_context() -> Context {
        Context::new(0, 75.0, 0.5, 0.3, 0.8)
    }

    // === Legacy Function Tests ===

    #[test]
    fn test_action_friction_levels() {
        assert_eq!(action_friction(&Action::Stay), 0.0);
        assert!(action_friction(&Action::Reproduce) > action_friction(&Action::Stay));
    }

    #[test]
    fn test_default_action_is_stay() {
        assert_eq!(default_action(), Action::Stay);
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[test]
    fn test_apply_nudges_boosts_default() {
        let agent = create_test_agent();
        let config = NudgeConfig::default();
        let candidates = vec![(Action::Stay, 1.0), (Action::Reproduce, 1.0)];

        let nudged = apply_nudges(&candidates, &agent, &config);

        // Default (Stay) should have higher weight after nudges
        // ERR-001: Test code - unwrap() exempt for test assertions
        let stay_weight = nudged.iter().find(|(a, _)| *a == Action::Stay).unwrap().1;
        let repro_weight = nudged
            .iter()
            .find(|(a, _)| *a == Action::Reproduce)
            .unwrap()
            .1;

        assert!(
            stay_weight > repro_weight,
            "Default action should be boosted"
        );
    }

    #[test]
    fn test_uniform_weights() {
        let candidates = vec![(Action::Stay, 5.0), (Action::Reproduce, 0.1)];

        let uniform = uniform_weights(&candidates);

        assert_eq!(uniform[0].1, 1.0);
        assert_eq!(uniform[1].1, 1.0);
    }

    // === FrameType Tests ===

    #[test]
    fn test_frame_type_default() {
        let frame: FrameType = FrameType::default();
        assert_eq!(frame, FrameType::Neutral);
    }

    // === NudgeType Tests ===

    #[test]
    fn test_nudge_type_default_action() {
        let nudge = NudgeType::DefaultBias {
            default_action: Action::Stay,
        };
        assert_eq!(nudge.default_action(), Some(Action::Stay));
        // TAG: surface=simulation owner=research-team rule=SIM-001

        let nudge = NudgeType::SocialProof {
            popular_action: Action::Reproduce,
            adoption_rate: 0.7,
        };
        assert_eq!(nudge.default_action(), Some(Action::Reproduce));

        let nudge = NudgeType::LossFraming {
            reference_point: 50.0,
        };
        assert_eq!(nudge.default_action(), None);
    }

    #[test]
    fn test_nudge_type_is_loss_framed() {
        let loss_nudge = NudgeType::LossFraming {
            reference_point: 50.0,
        };
        assert!(loss_nudge.is_loss_framed());

        let gain_nudge = NudgeType::GainFraming {
            reference_point: 50.0,
        };
        assert!(!gain_nudge.is_loss_framed());
    }

    #[test]
    fn test_nudge_type_is_gain_framed() {
        let gain_nudge = NudgeType::GainFraming {
            reference_point: 50.0,
        };
        assert!(gain_nudge.is_gain_framed());

        let loss_nudge = NudgeType::LossFraming {
            reference_point: 50.0,
        };
        assert!(!loss_nudge.is_gain_framed());
    }

    #[test]
    fn test_nudge_type_frame_type() {
        assert_eq!(
            NudgeType::LossFraming {
                reference_point: 50.0
            }
            .frame_type(),
            FrameType::Loss
        );
        assert_eq!(
            NudgeType::GainFraming {
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                reference_point: 50.0
            }
            .frame_type(),
            FrameType::Gain
        );
        assert_eq!(
            NudgeType::DefaultBias {
                default_action: Action::Stay
            }
            .frame_type(),
            FrameType::Neutral
        );
    }

    #[test]
    fn test_combined_nudge_frame_type() {
        let combined = NudgeType::Combined(vec![
            NudgeType::LossFraming {
                reference_point: 50.0,
            },
            NudgeType::GainFraming {
                reference_point: 60.0,
            },
        ]);
        assert_eq!(combined.frame_type(), FrameType::Mixed);

        let combined_loss_only = NudgeType::Combined(vec![
            NudgeType::LossFraming {
                reference_point: 50.0,
            },
            // TAG: surface=simulation owner=research-team rule=SIM-001
            NudgeType::DefaultBias {
                default_action: Action::Stay,
            },
        ]);
        assert_eq!(combined_loss_only.frame_type(), FrameType::Loss);
    }

    // === Context Tests ===

    #[test]
    fn test_context_from_agent() {
        let agent = create_test_agent();
        let context = Context::from_agent(&agent);

        assert_eq!(context.tick, agent.age);
        assert_eq!(context.energy, agent.energy);
        assert_eq!(context.dopamine, agent.docas.levels[D]);
    }

    #[test]
    fn test_context_builder_methods() {
        let context = Context::new(10, 80.0, 0.6, 0.2, 0.9)
            .with_nearby_agents(5)
            .with_local_resources(100.0)
            .with_in_group(true);

        assert_eq!(context.tick, 10);
        assert_eq!(context.nearby_agents, 5);
        assert_eq!(context.local_resources, 100.0);
        assert!(context.in_group);
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    // === ThalerEffects Tests ===

    #[test]
    fn test_thaler_effects_default() {
        let effects = ThalerEffects::default();

        // Per Thaler & Sunstein §6.4: λ ≈ 2.25
        assert!((effects.loss_aversion_lambda - 2.25).abs() < 0.01);
        assert!(effects.default_bias_strength > 0.0);
        assert!(effects.social_proof_strength >= 0.0);
        assert!(effects.framing_effect_size >= 0.0);
        assert!(effects.cognitive_load_impact >= 0.0);
    }

    #[test]
    fn test_thaler_effects_validate() {
        let valid = ThalerEffects::default();
        assert!(valid.validate().is_ok());

        let invalid = ThalerEffects {
            loss_aversion_lambda: 10.0, // Too high
            ..Default::default()
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_thaler_effects_minimal() {
        let minimal = ThalerEffects::minimal();
        assert_eq!(minimal.loss_aversion_lambda, 1.0);
        assert_eq!(minimal.default_bias_strength, 0.0);
    }

    #[test]
    fn test_thaler_effects_maximal() {
        let maximal = ThalerEffects::maximal();
        assert_eq!(maximal.loss_aversion_lambda, 3.0);
        assert_eq!(maximal.default_bias_strength, 1.0);
    }

    // === StandardChoiceArchitecture Tests ===

    #[test]
    fn test_adapter_default_action_when_enabled() {
        let adapter = StandardChoiceArchitecture::new();
        let context = create_test_context();

        assert!(adapter.is_enabled());
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(adapter.default_action(&context), Some(Action::Stay));
    }

    #[test]
    fn test_adapter_default_action_when_disabled() {
        let adapter = StandardChoiceArchitecture::disabled();
        let context = create_test_context();

        assert!(!adapter.is_enabled());
        assert_eq!(adapter.default_action(&context), None);
    }

    #[test]
    fn test_adapter_action_friction() {
        let adapter = StandardChoiceArchitecture::new();
        let context = create_test_context();

        // Stay has no friction
        assert_eq!(adapter.action_friction(&Action::Stay, &context), 0.0);
        // Reproduction has high friction
        assert!(
            adapter.action_friction(&Action::Reproduce, &context)
                > adapter.action_friction(&Action::Stay, &context)
        );
    }

    #[test]
    fn test_adapter_salience_uniform_when_disabled() {
        let disabled = StandardChoiceArchitecture::disabled();
        let context = create_test_context();

        // When disabled, all actions have uniform salience
        assert_eq!(disabled.salience(&Action::Stay, &context), 1.0);
        assert_eq!(disabled.salience(&Action::Reproduce, &context), 1.0);
    }

    #[test]
    fn test_adapter_salience_varies_when_enabled() {
        let adapter = StandardChoiceArchitecture::new();
        let context = create_test_context();

        // When enabled, salience varies by action
        let stay_salience = adapter.salience(&Action::Stay, &context);
        let move_salience = adapter.salience(&Action::Move(crate::dmn::Direction::North), &context);

        assert_ne!(stay_salience, move_salience);
    }

    #[test]
    fn test_adapter_cognitive_cost() {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let adapter = StandardChoiceArchitecture::new();
        let context = create_test_context();

        let stay_cost = adapter.cognitive_cost(&Action::Stay, &context);
        let move_cost =
            adapter.cognitive_cost(&Action::Move(crate::dmn::Direction::North), &context);
        let repro_cost = adapter.cognitive_cost(&Action::Reproduce, &context);

        // Reproduction costs more than movement, which costs more than staying
        assert!(repro_cost > move_cost);
        assert!(move_cost > stay_cost);
    }

    #[test]
    fn test_adapter_frame_utility_neutral() {
        let adapter = StandardChoiceArchitecture::new();
        let context = create_test_context();

        let utility = 10.0;
        let framed = adapter.frame_utility(utility, &context, FrameType::Neutral);

        // Neutral framing should not change utility
        assert_eq!(framed, utility);
    }

    #[test]
    fn test_adapter_frame_utility_loss() {
        let adapter = StandardChoiceArchitecture::new();
        let context = create_test_context();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Negative utility with loss framing should be amplified
        let loss_utility = -10.0;
        let framed_loss = adapter.frame_utility(loss_utility, &context, FrameType::Loss);
        assert!(framed_loss < loss_utility);

        // Positive utility with loss framing should be reduced
        let gain_utility = 10.0;
        let framed_gain = adapter.frame_utility(gain_utility, &context, FrameType::Loss);
        assert!(framed_gain < gain_utility);
    }

    #[test]
    fn test_adapter_frame_utility_gain() {
        let adapter = StandardChoiceArchitecture::new();
        let context = create_test_context();

        // Positive utility with gain framing should be amplified
        let gain_utility = 10.0;
        let framed_gain = adapter.frame_utility(gain_utility, &context, FrameType::Gain);
        assert!(framed_gain > gain_utility);

        // Negative utility with gain framing should be reduced
        let loss_utility = -10.0;
        let framed_loss = adapter.frame_utility(loss_utility, &context, FrameType::Gain);
        assert!(framed_loss > loss_utility);
    }

    #[test]
    fn test_adapter_apply_endowment_effect() {
        let adapter = StandardChoiceArchitecture::new();

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let value = 10.0;
        let loss_value = adapter.apply_endowment_effect(-value, true);
        let gain_value = adapter.apply_endowment_effect(value, false);

        // Losses should be amplified by lambda
        assert!(loss_value.abs() > value);
        // Gains should be at face value
        assert_eq!(gain_value, value);

        // Loss should hurt more than equivalent gain feels good
        assert!(loss_value.abs() > gain_value);
    }

    #[test]
    fn test_adapter_apply_endowment_effect_disabled() {
        let adapter = StandardChoiceArchitecture::disabled();

        let value = 10.0;
        let loss_value = adapter.apply_endowment_effect(-value, true);

        // When disabled, endowment effect should not apply
        assert_eq!(loss_value, -value);
    }

    #[test]
    fn test_adapter_available_actions() {
        let adapter = StandardChoiceArchitecture::new();
        let mut agent = create_test_agent();
        // Set energy high enough for reproduction (requires >= 50.0)
        agent.energy = 75.0;
        let context = Context::from_agent(&agent);

        let actions = adapter.available_actions(&agent, &context);

        // Should include Stay, all 8 Move directions, and Reproduce
        assert!(actions.contains(&Action::Stay));
        assert!(actions.iter().any(|a| matches!(a, Action::Move(_))));
        assert!(actions.contains(&Action::Reproduce));
    }

    #[test]
    fn test_adapter_with_nudge() {
        let adapter = StandardChoiceArchitecture::with_nudge(NudgeType::DefaultBias {
            default_action: Action::Reproduce,
        });
        let context = create_test_context();

        assert_eq!(adapter.default_action(&context), Some(Action::Reproduce));
    }
    // TAG: surface=simulation owner=research-team rule=SIM-001

    #[test]
    fn test_adapter_enable_disable() {
        let mut adapter = StandardChoiceArchitecture::new();

        assert!(adapter.is_enabled());

        adapter.disable();
        assert!(!adapter.is_enabled());

        adapter.enable();
        assert!(adapter.is_enabled());
    }

    // === ChoiceArchitectureManager Tests ===

    #[test]
    fn test_manager_new() {
        let manager = ChoiceArchitectureManager::new();

        assert!(manager.is_enabled());
        assert_eq!(manager.stats().nudges_applied, 0);
    }

    #[test]
    fn test_manager_enable_disable() {
        let mut manager = ChoiceArchitectureManager::new();

        assert!(manager.is_enabled());

        manager.disable();
        assert!(!manager.is_enabled());

        manager.enable();
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_manager_set_nudge() {
        let mut manager = ChoiceArchitectureManager::new();

        manager.set_nudge(
            NudgeType::SocialProof {
                popular_action: Action::Stay,
                adoption_rate: 0.8,
            },
            10,
        );

        match manager.current_nudge() {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            NudgeType::SocialProof { adoption_rate, .. } => {
                assert!((*adoption_rate - 0.8).abs() < 0.01);
            }
            _ => panic!("Expected SocialProof nudge"),
        }
    }

    #[test]
    fn test_manager_record_statistics() {
        let mut manager = ChoiceArchitectureManager::new();

        manager.record_default_selection();
        manager.record_friction_block();
        manager.record_cognitive_cost(2.5);
        manager.record_nudge_applied();

        assert_eq!(manager.stats().default_selections, 1);
        assert_eq!(manager.stats().friction_blocks, 1);
        assert!((manager.stats().total_cognitive_cost - 2.5).abs() < 0.01);
        assert_eq!(manager.stats().nudges_applied, 1);
    }

    #[test]
    fn test_manager_apply_to_candidates() {
        let mut manager = ChoiceArchitectureManager::new();
        let agent = create_test_agent();
        let context = create_test_context();

        let candidates = vec![(Action::Stay, 1.0), (Action::Reproduce, 1.0)];

        let result = manager.apply_to_candidates(&candidates, &agent, &context);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(result.len(), 2);
        assert_eq!(manager.stats().nudges_applied, 1);
    }

    #[test]
    fn test_manager_select_action() {
        let mut manager = ChoiceArchitectureManager::new();
        let agent = create_test_agent();
        let context = create_test_context();

        let candidates = vec![(Action::Stay, 10.0), (Action::Reproduce, 1.0)];

        let selected = manager.select_action(&candidates, &agent, &context);

        // Stay should be selected due to higher base weight + default boost
        assert_eq!(selected, Action::Stay);
    }

    // === Loss Aversion Tests ===

    #[test]
    fn test_loss_aversion_utility() {
        let lambda = 2.25;

        // Pure gain
        let gain_only = loss_aversion_utility(10.0, 0.0, lambda);
        assert_eq!(gain_only, 10.0);

        // Pure loss
        let loss_only = loss_aversion_utility(0.0, 10.0, lambda);
        assert_eq!(loss_only, -22.5); // -2.25 * 10

        // Mixed
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let mixed = loss_aversion_utility(10.0, 5.0, lambda);
        assert_eq!(mixed, 10.0 - 2.25 * 5.0); // 10 - 11.25 = -1.25
    }

    // === NudgeConfig Tests ===

    #[test]
    fn test_nudge_config_default() {
        let config = NudgeConfig::default();

        assert!(config.enabled);
        assert!(config.default_boost > 0.0);
        assert!(config.friction_penalty > 0.0);
        assert!(config.salience_boost > 0.0);
    }

    #[test]
    fn test_nudge_config_disabled() {
        let config = NudgeConfig::disabled();

        assert!(!config.enabled);
    }

    #[test]
    fn test_nudge_config_enabled() {
        let config = NudgeConfig::enabled();

        assert!(config.enabled);
    }

    // === Integration Tests ===

    #[test]
    fn test_apply_nudges_with_adapter_enabled() {
        let adapter = StandardChoiceArchitecture::new();
        let agent = create_test_agent();
        let context = create_test_context();

        let candidates = vec![(Action::Stay, 1.0), (Action::Reproduce, 1.0)];

        let result = apply_nudges_with_adapter(&candidates, &agent, &context, &adapter);

        // Default (Stay) should be boosted
        // ERR-001: Test code - unwrap() exempt for test assertions
        let stay_weight = result.iter().find(|(a, _)| *a == Action::Stay).unwrap().1;
        let repro_weight = result
            .iter()
            .find(|(a, _)| *a == Action::Reproduce)
            .unwrap()
            // TAG: surface=simulation owner=research-team rule=SIM-001
            .1;

        assert!(stay_weight > repro_weight);
    }

    #[test]
    fn test_apply_nudges_with_adapter_disabled() {
        let adapter = StandardChoiceArchitecture::disabled();
        let agent = create_test_agent();
        let context = create_test_context();

        let candidates = vec![(Action::Stay, 5.0), (Action::Reproduce, 1.0)];

        let result = apply_nudges_with_adapter(&candidates, &agent, &context, &adapter);

        // When disabled, should return uniform weights
        assert_eq!(result[0].1, 1.0);
        assert_eq!(result[1].1, 1.0);
    }

    #[test]
    fn test_compute_dmn_budget_reduction() {
        let adapter = StandardChoiceArchitecture::new();
        let context = create_test_context();

        let actions = vec![
            Action::Stay,
            Action::Move(crate::dmn::Direction::North),
            Action::Reproduce,
        ];

        let reduction = compute_dmn_budget_reduction(&actions, &adapter, &context);

        // Stay: 0.5, Move: 1.0, Reproduce: 2.0 = 3.5 total
        assert!((reduction - 3.5).abs() < 0.01);
    }

    #[test]
    fn test_endowment_effect_classic_value() {
        // Test the classic 2.25 lambda value
        let effects = ThalerEffects::default();
        let adapter = StandardChoiceArchitecture::with_effects(effects);

        let gain_value = adapter.apply_endowment_effect(100.0, false);
        let loss_value = adapter.apply_endowment_effect(-100.0, true);

        // Gain at face value
        assert_eq!(gain_value, 100.0);
        // Loss amplified by lambda
        assert!((loss_value.abs() - 225.0).abs() < 0.1);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // Loss hurts ~2.25x more than equivalent gain
        let ratio = loss_value.abs() / gain_value;
        assert!((ratio - 2.25).abs() < 0.01);
    }

    #[test]
    fn test_commitment_device_nudge() {
        let nudge = NudgeType::CommitmentDevice {
            committed_action: Action::Stay,
            penalty: 0.5,
        };

        assert_eq!(nudge.default_action(), Some(Action::Stay));
        assert!(!nudge.is_loss_framed());
        assert!(!nudge.is_gain_framed());
    }

    #[test]
    fn test_simplification_nudge() {
        let adapter =
            StandardChoiceArchitecture::with_nudge(NudgeType::Simplification { max_options: 3 });
        let agent = create_test_agent();
        let context = Context::from_agent(&agent);

        let actions = adapter.available_actions(&agent, &context);

        // Simplification should limit options
        assert!(actions.len() <= 4); // max_options + some flexibility
    }

    #[test]
    fn test_framing_effect_size_impact() {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let high_effect = StandardChoiceArchitecture::with_effects(ThalerEffects {
            framing_effect_size: 0.6,
            ..Default::default()
        });
        let low_effect = StandardChoiceArchitecture::with_effects(ThalerEffects {
            framing_effect_size: 0.1,
            ..Default::default()
        });

        let context = create_test_context();
        let utility = -10.0;

        let high_framed = high_effect.frame_utility(utility, &context, FrameType::Loss);
        let low_framed = low_effect.frame_utility(utility, &context, FrameType::Loss);

        // Higher effect size should produce more extreme framing
        assert!(high_framed < low_framed);
    }

    #[test]
    fn test_social_proof_nudge() {
        let nudge = NudgeType::SocialProof {
            popular_action: Action::Reproduce,
            adoption_rate: 0.75,
        };

        assert_eq!(nudge.default_action(), Some(Action::Reproduce));

        let adapter = StandardChoiceArchitecture::with_nudge(nudge);
        let context = create_test_context();

        assert_eq!(adapter.default_action(&context), Some(Action::Reproduce));
    }
}
