//! Norm emergence, compliance, and sanctions subsystem
//!
//! Per §4264-4282: Four complementary emergence processes
//! Per §4284-4309: Punishment mechanisms for norm violations

use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::agent::Agent;
use crate::types::{AgentId, GroupId, NormId, Tick, A, C, D, O};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Norm emergence mechanism types
/// Per §4264-4282: Four complementary emergence processes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NormEmergenceType {
    /// Frequency-dependent: Behaviors adopted by >60% of group become normative
    /// Per §4268-4270: threshold freq(behavior) > 0.60 → behavior becomes "expected"
    FrequencyDependent,

    /// Reputation-based: High-trust (τ > 0.70) agents' behaviors weighted more heavily
    /// Per §4272-4274: `behavior_weight` = `base_weight` × (1 + agent.τ)
    ReputationBased,

    /// Coordination-based: Norms solve repeated coordination failures
    /// Per §4276-4278: Coordination failure rate > 0.30 over 50 ticks → norm proposal
    CoordinationBased,

    /// Threshold dynamics: Critical mass required for norm activation
    /// Per §4280-4282: 20-30% early adopters trigger cascade (S-curve)
    ThresholdDynamics,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Norm behavior pattern identifiers
/// Per §4264-4282: Common behavioral patterns that can become norms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BehaviorPattern {
    /// Share resources with group members
    ShareWithGroup,
    /// Cooperate with strangers
    CooperateWithStrangers,
    /// Punish defectors
    PunishDefectors,
    /// Reciprocate cooperation
    Reciprocate,
    /// Verify before trusting
    VerifyFirst,
    /// Help group members in need
    HelpInNeed,
    /// Exclude known defectors
    ExcludeDefectors,
    /// Signal group membership
    SignalMembership,
}

/// Norm structure representing emerged behavioral rules
/// Per §4264-4282: Norms emerge from group dynamics and enforce behavior
/// Per §8.2.1 lines 4687-4708: Norms toggle contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Norm {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Unique identifier
    pub id: NormId,

    /// The behavior pattern this norm prescribes
    pub behavior_pattern: BehaviorPattern,

    /// Strength/compliance level [0.0, 1.0]
    /// Higher value = stronger enforcement, more compliance expected
    pub value: f32,

    /// Tick when this norm emerged
    pub created_tick: Tick,

    /// Group that owns this norm (None = global/population-wide norm)
    pub group_id: Option<GroupId>,

    /// How this norm emerged
    pub emergence_type: NormEmergenceType,

    /// Current adoption rate within the group [0.0, 1.0]
    pub adoption_rate: f32,

    /// Salience: how visible/prominent this norm is [0.0, 1.0]
    /// Per §4352-4354: High-salience norms are followed more
    pub salience: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Norm {
    /// Create a new norm
    #[must_use]
    pub const fn new(
        id: NormId,
        behavior_pattern: BehaviorPattern,
        group_id: Option<GroupId>,
        emergence_type: NormEmergenceType,
        created_tick: Tick,
    ) -> Self {
        Self {
            id,
            behavior_pattern,
            value: 0.5, // Initial moderate strength
            created_tick,
            group_id,
            emergence_type,
            adoption_rate: 0.0,
            salience: 0.5, // Initial moderate salience
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if an action violates this norm
    /// Per §4176-4189: Cultural norm filter (only active, salient norms can be violated)
    #[must_use]
    pub fn violated_by(&self, action_pattern: BehaviorPattern) -> bool {
        if self.salience <= 0.0 || self.value <= 0.0 {
            return false;
        }
        // A norm is violated when the opposite behavior is exhibited
        match self.behavior_pattern {
            BehaviorPattern::CooperateWithStrangers => {
                action_pattern == BehaviorPattern::ExcludeDefectors
            }
            BehaviorPattern::PunishDefectors => false, // Punishing is always allowed if norm exists
            BehaviorPattern::ShareWithGroup => false,  // Sharing never violates share norm
            BehaviorPattern::Reciprocate => false,     // Reciprocation is norm-compliant
            BehaviorPattern::VerifyFirst => false,     // Verification is always acceptable
            _ => false,                                // Default: no violation
        }
    }

    /// Update norm value based on enforcement success
    /// Per §4330-4333: Salience decays when not invoked
    pub fn decay(&mut self, decay_rate: f32) {
        self.salience = (self.salience - decay_rate).max(0.0);
        if self.salience < 0.1 {
            self.value = (self.value - decay_rate * 0.5).max(0.0);
        }
    }

    /// Reinforce norm when successfully enforced
    pub fn reinforce(&mut self, amount: f32) {
        self.value = (self.value + amount).min(1.0);
        self.salience = (self.salience + amount * 0.5).min(1.0);
    }
}

// ============================================================================
// NORM EMERGENCE AND COMPLIANCE
// Per §4264-4282: Four complementary emergence processes
// ============================================================================

/// Behavior frequency tracking for norm emergence
/// Per §4268-4270: Rolling window of 100 ticks per group
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BehaviorFrequency {
    /// Count of each behavior pattern observed
    pub counts: std::collections::HashMap<BehaviorPattern, u32>,
    /// Total observations
    pub total: u32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl BehaviorFrequency {
    /// Record a behavior observation
    pub fn record(&mut self, pattern: BehaviorPattern) {
        *self.counts.entry(pattern).or_insert(0) += 1;
        self.total += 1;
    }

    /// Get frequency of a specific behavior
    #[must_use]
    pub fn frequency(&self, pattern: BehaviorPattern) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        self.counts.get(&pattern).copied().unwrap_or(0) as f32 / self.total as f32
    }

    /// Reset for new window
    pub fn reset(&mut self) {
        self.counts.clear();
        self.total = 0;
    }
}

/// Check for frequency-dependent norm emergence
/// Per §4268-4270: Behaviors adopted by >60% of group become normative
pub fn check_frequency_dependent_emergence(
    behavior_freq: &BehaviorFrequency,
    group_id: Option<GroupId>,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    current_tick: Tick,
    next_norm_id: &mut NormId,
) -> Option<Norm> {
    const FREQUENCY_THRESHOLD: f32 = 0.60;

    for &pattern in &[
        BehaviorPattern::ShareWithGroup,
        BehaviorPattern::CooperateWithStrangers,
        BehaviorPattern::Reciprocate,
        BehaviorPattern::VerifyFirst,
        BehaviorPattern::PunishDefectors,
    ] {
        let freq = behavior_freq.frequency(pattern);
        if freq > FREQUENCY_THRESHOLD {
            let norm_id = *next_norm_id;
            *next_norm_id += 1;
            return Some(Norm {
                id: norm_id,
                behavior_pattern: pattern,
                value: freq, // Stronger norm if higher frequency
                created_tick: current_tick,
                group_id,
                emergence_type: NormEmergenceType::FrequencyDependent,
                adoption_rate: freq,
                salience: 0.5 + 0.3 * freq, // Higher frequency → more salient
            });
        }
    }
    None
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Check for reputation-based norm emergence
/// Per §4272-4274: High-trust agents' behaviors weighted more heavily
pub fn check_reputation_based_emergence(
    agents: &[Agent],
    group_id: Option<GroupId>,
    behavior_freq: &BehaviorFrequency,
    current_tick: Tick,
    next_norm_id: &mut NormId,
) -> Option<Norm> {
    const HIGH_TRUST_THRESHOLD: f32 = 0.70;
    const LEADER_ADOPTION_THRESHOLD: f32 = 0.50;

    // Filter to group members and high-trust agents
    let group_agents: Vec<_> = agents
        .iter()
        .filter(|a| a.alive && a.group_id == group_id)
        .collect();

    if group_agents.is_empty() {
        return None;
    }

    // Find top 10% by reputation (leaders)
    let mut reputations: Vec<_> = group_agents.iter().map(|a| a.reputation_score).collect();
    reputations.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    let leader_threshold_idx = (reputations.len() as f32 * 0.1).ceil() as usize;
    let leader_threshold = reputations
        .get(leader_threshold_idx.saturating_sub(1))
        .copied()
        .unwrap_or(HIGH_TRUST_THRESHOLD);

    let leaders: Vec<_> = group_agents
        .iter()
        .filter(|a| a.reputation_score >= leader_threshold.max(HIGH_TRUST_THRESHOLD))
        .collect();

    if leaders.is_empty() {
        return None;
    }

    // Check what behavior leaders exhibit most
    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Per §4272-4274: Leaders have 2× influence on norm formation
    for &pattern in &[
        BehaviorPattern::CooperateWithStrangers,
        BehaviorPattern::ShareWithGroup,
        BehaviorPattern::Reciprocate,
    ] {
        let freq = behavior_freq.frequency(pattern);
        // Leaders adopting a behavior at >50% rate can establish a norm
        if freq > LEADER_ADOPTION_THRESHOLD && !leaders.is_empty() {
            let norm_id = *next_norm_id;
            *next_norm_id += 1;
            return Some(Norm {
                id: norm_id,
                behavior_pattern: pattern,
                value: 0.6 + 0.2 * freq, // Leader-backed norms start stronger
                created_tick: current_tick,
                group_id,
                emergence_type: NormEmergenceType::ReputationBased,
                adoption_rate: freq,
                salience: 0.7, // Leader norms are highly salient
            });
        }
    }
    None
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Coordination failure tracking for norm proposals
/// Per §4276-4278: Coordination failure rate > 0.30 over 50 ticks → norm proposal
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoordinationTracker {
    /// Number of coordination attempts
    pub attempts: u32,
    /// Number of coordination failures
    pub failures: u32,
    /// Tick window start
    pub window_start: Tick,
}

impl CoordinationTracker {
    /// Record a coordination outcome
    pub fn record(&mut self, success: bool) {
        self.attempts += 1;
        if !success {
            self.failures += 1;
        }
    }

    /// Get failure rate
    #[must_use]
    pub fn failure_rate(&self) -> f32 {
        if self.attempts == 0 {
            return 0.0;
        }
        self.failures as f32 / self.attempts as f32
    }

    /// Reset for new window
    pub fn reset(&mut self, current_tick: Tick) {
        self.attempts = 0;
        self.failures = 0;
        self.window_start = current_tick;
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Check for coordination-based norm emergence
/// Per §4276-4278: Norms solve repeated coordination failures
pub fn check_coordination_based_emergence(
    tracker: &CoordinationTracker,
    group_id: Option<GroupId>,
    current_tick: Tick,
    next_norm_id: &mut NormId,
) -> Option<Norm> {
    const FAILURE_THRESHOLD: f32 = 0.30;
    const MIN_WINDOW_TICKS: Tick = 50;
    const MIN_ATTEMPTS: u32 = 10;

    // Need enough observations
    if tracker.attempts < MIN_ATTEMPTS {
        return None;
    }

    // Check if window is long enough
    if current_tick.saturating_sub(tracker.window_start) < MIN_WINDOW_TICKS {
        return None;
    }

    // High failure rate triggers norm proposal
    if tracker.failure_rate() > FAILURE_THRESHOLD {
        let norm_id = *next_norm_id;
        *next_norm_id += 1;
        // Propose a coordination-solving norm
        return Some(Norm {
            id: norm_id,
            behavior_pattern: BehaviorPattern::Reciprocate, // Default coordination solution
            value: 0.5,
            created_tick: current_tick,
            group_id,
            emergence_type: NormEmergenceType::CoordinationBased,
            adoption_rate: 0.0, // Just proposed, not yet adopted
            salience: 0.6,      // Moderate salience for new proposals
        });
    }
    None
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Check for threshold dynamics norm emergence
/// Per §4280-4282: Critical mass triggers cascade (S-curve adoption)
pub fn check_threshold_dynamics_emergence(norm: &mut Norm, agents: &[Agent]) -> bool {
    const EARLY_ADOPTER_THRESHOLD: f32 = 0.25; // 20-30% early adopters
    const CASCADE_ACCELERATION: f32 = 1.5; // Adoption rate multiplier post-threshold

    // Count adopters in the group
    let group_agents: Vec<_> = agents
        .iter()
        .filter(|a| a.alive && a.group_id == norm.group_id)
        .collect();

    if group_agents.is_empty() {
        return false;
    }

    let adopters = group_agents
        .iter()
        .filter(|a| a.cultural_traits.contains(&norm.id))
        .count();

    let adoption_rate = adopters as f32 / group_agents.len() as f32;
    norm.adoption_rate = adoption_rate;

    // Check if threshold crossed → trigger cascade
    if adoption_rate > EARLY_ADOPTER_THRESHOLD && norm.value < 0.8 {
        // S-curve acceleration: strengthen norm rapidly
        norm.value = (norm.value * CASCADE_ACCELERATION).min(1.0);
        norm.salience = (norm.salience * 1.2).min(1.0);
        return true; // Cascade triggered
    }

    false
}

/// Compute norm compliance probability for an agent
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
/// Per §4176-4189: Cultural norm filter
/// Per §8.2.1 lines 4687-4708: Norms toggle contract
///
/// LAYER DEPENDENCY: Consumes Philosophy Layer (`norm_adherence`)
/// Per Layer 6→7: Philosophy → Ecology (norm emergence)
#[must_use]
pub fn compute_norm_compliance(
    agent: &Agent,
    norm: &Norm,
    group_tau: f32, // Group's average trust level
) -> f32 {
    // Base compliance from norm strength
    let base_compliance = norm.value;

    // Modulated by agent's DOCAS state
    // High O (oxytocin) → more compliant (pro-social)
    // High C (cortisol) → less compliant (stress reduces cooperation)
    let docas_modulation = 1.0 + 0.3 * agent.docas.levels[O] - 0.2 * agent.docas.levels[C];

    // Salience effect: high-salience norms are followed more
    // Per §4352-4354
    let salience_effect = 1.0 + 0.5 * (norm.salience - 0.5);

    // Group trust effect: higher group trust → more norm compliance
    let trust_effect = 0.7 + 0.3 * group_tau;

    // PHILOSOPHY LAYER: Agent's intrinsic norm adherence
    // Per Layer 6: Philosophy (norm_adherence) → Ecology (norm compliance)
    // Higher norm_adherence → higher compliance probability
    // Map tau_phi to norm_adherence: higher openness (tau_phi) → higher adherence
    let norm_adherence = agent
        .philosophical_state
        .as_ref()
        .map_or(0.5, |p| (p.tau_phi + 1.0) / 2.0);
    let adherence_effect = 0.5 + 0.5 * norm_adherence;

    // Combined probability
    (base_compliance * docas_modulation * salience_effect * trust_effect * adherence_effect)
        .clamp(0.0, 1.0)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Check if an action violates any active norms and compute punishment probability
/// Per §4176-4189: Cultural norm filter
#[must_use]
pub fn check_norm_violation(
    action_pattern: BehaviorPattern,
    active_norms: &[Norm],
    agent: &Agent,
    group_tau: f32,
) -> Option<(NormId, f32)> {
    for norm in active_norms {
        // Check if this norm applies to the agent's group
        if norm.group_id.is_some() && norm.group_id != agent.group_id {
            continue;
        }

        if norm.violated_by(action_pattern) {
            let compliance_prob = compute_norm_compliance(agent, norm, group_tau);
            let violation_prob = 1.0 - compliance_prob;
            return Some((norm.id, violation_prob));
        }
    }
    None
}

// ============================================================================
// NORM ENFORCEMENT
// Per §4284-4309: Punishment mechanisms for norm violations
// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Sanction types for norm enforcement
/// Per §4297-4309: Punishment types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SanctionType {
    /// Reputation reduction
    /// Per §4302: Punished agent loses reputation
    ReputationLoss,
    /// Resource/energy penalty
    /// Per §4305: Economic sanction
    ResourcePenalty,
    /// Social exclusion (temporary group removal)
    /// Per §4307: Ostracism
    SocialExclusion,
}

/// Sanction result from norm enforcement
#[derive(Debug, Clone)]
pub struct SanctionResult {
    /// Agent being sanctioned
    pub violator_id: AgentId,
    /// Agent applying sanction (None for collective punishment)
    pub punisher_id: Option<AgentId>,
    /// Type of sanction applied
    pub sanction_type: SanctionType,
    /// Severity [0.0, 1.0]
    pub severity: f32,
    /// DOCAS effect on violator: Cortisol increase
    pub violator_cortisol_delta: f32,
    /// DOCAS effect on punisher: Dopamine from enforcement
    pub punisher_dopamine_delta: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute punishment probability for a norm violation
/// Per §4284-4297: Punishment probability calculation
#[must_use]
pub fn compute_punishment_probability(
    norm: &Norm,
    violator: &Agent,
    potential_punisher: &Agent,
    group_tau: f32,
) -> f32 {
    // Base punishment probability from norm strength
    // Per §4286: P(punishment) = norm.value × group_tau
    let base_prob = norm.value * group_tau;

    // Modulated by punisher's DOCAS state
    // High O (oxytocin) on punisher → more willing to enforce group norms
    // High A (adrenaline) → more reactive to violations
    let punisher_modulation =
        1.0 + 0.2 * potential_punisher.docas.levels[O] + 0.1 * potential_punisher.docas.levels[A];

    // Modulated by violator's reputation
    // Per §4290: Low-reputation violators punished more
    let reputation_factor = 1.0 - 0.3 * violator.reputation_score.max(0.0);

    // Salience effect: salient norms more likely to be enforced
    let salience_factor = 0.7 + 0.3 * norm.salience;

    // Combined probability
    (base_prob * punisher_modulation * reputation_factor * salience_factor).clamp(0.0, 0.95)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Apply sanction to a norm violator
/// Per §4297-4309: Sanction application
pub fn apply_sanction(
    violator: &mut Agent,
    punisher: Option<&mut Agent>,
    norm: &Norm,
    sanction_type: SanctionType,
    rng: &mut StdRng,
) -> SanctionResult {
    // Severity based on norm strength and violation context
    let base_severity = norm.value * 0.5;
    let random_factor = 0.8 + 0.4 * rng.gen::<f32>(); // [0.8, 1.2]
    let severity = (base_severity * random_factor).clamp(0.1, 0.8);

    // Apply sanction effects
    match sanction_type {
        SanctionType::ReputationLoss => {
            // Per §4302: Punished: reputation -= severity × 0.3
            violator.reputation_score = (violator.reputation_score - severity * 0.3).max(-1.0);
        }
        SanctionType::ResourcePenalty => {
            // Per §4305: Punished: energy -= severity × 10.0
            violator.energy = (violator.energy - severity * 10.0).max(0.0);
        }
        SanctionType::SocialExclusion => {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            // Per §4307: Temporary group removal (handled by caller)
            // Mark for exclusion by setting group_id to None temporarily
            violator.group_id = None;
            violator.isolation_score = (violator.isolation_score + 0.2).min(1.0);
        }
    }

    // DOCAS effects
    // Per §4307-4309: Emotional consequences
    // Violator: C += 0.15 (stress from punishment)
    let violator_cortisol_delta = 0.15 * severity;
    violator.docas.levels[C] = (violator.docas.levels[C] + violator_cortisol_delta).min(1.0);

    // Punisher: D += 0.1 (satisfaction from enforcement)
    let punisher_dopamine_delta = 0.1 * severity;
    let punisher_id = if let Some(p) = punisher {
        p.docas.levels[D] = (p.docas.levels[D] + punisher_dopamine_delta).min(1.0);
        Some(p.id)
    } else {
        None
    };

    SanctionResult {
        violator_id: violator.id,
        punisher_id,
        sanction_type,
        severity,
        violator_cortisol_delta,
        punisher_dopamine_delta,
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Select appropriate sanction type based on violation context
/// Per §4297-4309: Sanction selection
pub fn select_sanction_type(
    norm: &Norm,
    _violator: &Agent, // Reserved for future violation-context-aware selection
    is_repeat_offender: bool,
    rng: &mut StdRng,
) -> SanctionType {
    // Escalation for repeat offenders
    if is_repeat_offender {
        return SanctionType::SocialExclusion;
    }

    // Based on norm type and violator state
    let roll = rng.gen::<f32>();

    // Strong norms → harsher sanctions
    if norm.value > 0.7 {
        if roll < 0.5 {
            SanctionType::ReputationLoss
        } else if roll < 0.8 {
            SanctionType::ResourcePenalty
        } else {
            SanctionType::SocialExclusion
        }
    } else {
        // Weaker norms → milder sanctions
        if roll < 0.7 {
            SanctionType::ReputationLoss
        } else {
            SanctionType::ResourcePenalty
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Process norm enforcement for an entire group
/// Per §4284-4309: Group-level enforcement
pub fn enforce_group_norms(
    agents: &mut [Agent],
    active_norms: &[Norm],
    violations: &[(AgentId, NormId)],
    group_tau: f32,
    rng: &mut StdRng,
) -> Vec<SanctionResult> {
    let mut results = Vec::new();

    for (violator_id, norm_id) in violations {
        // Find the norm
        let Some(norm) = active_norms.iter().find(|n| n.id == *norm_id) else {
            continue;
        };

        // Find violator index
        let Some(violator_idx) = agents.iter().position(|a| a.id == *violator_id && a.alive) else {
            continue;
        };

        // Find a potential punisher (random group member with high O)
        let group_id = agents[violator_idx].group_id;
        let potential_punishers: Vec<_> = agents
            .iter()
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            .enumerate()
            .filter(|(i, a)| {
                *i != violator_idx && a.alive && a.group_id == group_id && a.docas.levels[O] > 0.4
                // Pro-social agents punish
            })
            .map(|(i, _)| i)
            .collect();

        if potential_punishers.is_empty() {
            continue; // No one to enforce
        }

        // Select random punisher
        let punisher_idx = potential_punishers[rng.gen_range(0..potential_punishers.len())];

        // Compute punishment probability
        let punishment_prob = compute_punishment_probability(
            norm,
            &agents[violator_idx],
            &agents[punisher_idx],
            group_tau,
        );

        // Decide if punishment occurs
        if rng.gen::<f32>() < punishment_prob {
            // Check if repeat offender (simplified: low reputation = repeat)
            let is_repeat = agents[violator_idx].reputation_score < 0.3;
            let sanction_type = select_sanction_type(norm, &agents[violator_idx], is_repeat, rng);

            // Apply sanction (need to handle borrow checker)
            // We'll apply directly without intermediate steps
            let violator = &mut agents[violator_idx];
            let severity = norm.value * 0.5;

            // TAG: surface=simulation owner=research-team rule=SIM-001
            match sanction_type {
                SanctionType::ReputationLoss => {
                    violator.reputation_score =
                        (violator.reputation_score - severity * 0.3).max(-1.0);
                }
                SanctionType::ResourcePenalty => {
                    violator.energy = (violator.energy - severity * 10.0).max(0.0);
                }
                SanctionType::SocialExclusion => {
                    violator.group_id = None;
                    violator.isolation_score = (violator.isolation_score + 0.2).min(1.0);
                }
            }

            // DOCAS effects
            violator.docas.levels[C] = (violator.docas.levels[C] + 0.15 * severity).min(1.0);

            let punisher = &mut agents[punisher_idx];
            punisher.docas.levels[D] = (punisher.docas.levels[D] + 0.1 * severity).min(1.0);

            results.push(SanctionResult {
                violator_id: *violator_id,
                punisher_id: Some(agents[punisher_idx].id),
                sanction_type,
                severity,
                violator_cortisol_delta: 0.15 * severity,
                punisher_dopamine_delta: 0.1 * severity,
            });
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::Agent;
    use crate::genome::Genome;
    use crate::types::{Position, Sex};
    use rand::SeedableRng;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn create_test_agent(id: u64, sex: Sex) -> Agent {
        let genome = Genome::default();
        Agent::new(id, Position::new_2d(0.0, 0.0), genome, 0, sex)
    }

    #[test]
    fn test_compute_norm_compliance() {
        let agent = create_test_agent(1, Sex::Male);
        let norm = Norm::new(
            1,
            BehaviorPattern::ShareWithGroup,
            None,
            NormEmergenceType::FrequencyDependent,
            0,
        );
        let compliance = compute_norm_compliance(&agent, &norm, 0.7);
        assert!((0.0..=1.0).contains(&compliance));
    }

    #[test]
    fn test_norm_violated_by() {
        let norm = Norm::new(
            2,
            BehaviorPattern::CooperateWithStrangers,
            None,
            NormEmergenceType::CoordinationBased,
            0,
        );
        assert!(norm.violated_by(BehaviorPattern::ExcludeDefectors));
        assert!(!norm.violated_by(BehaviorPattern::CooperateWithStrangers));
    }

    #[test]
    fn test_check_norm_violation_found() {
        let agent = create_test_agent(1, Sex::Male);
        let norm = Norm::new(
            2,
            BehaviorPattern::CooperateWithStrangers,
            None,
            NormEmergenceType::CoordinationBased,
            0,
        );
        let result = check_norm_violation(BehaviorPattern::ExcludeDefectors, &[norm], &agent, 0.7);
        assert!(result.is_some());
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_check_norm_violation_none() {
        let agent = create_test_agent(1, Sex::Male);
        let norm = Norm::new(
            2,
            BehaviorPattern::CooperateWithStrangers,
            None,
            NormEmergenceType::CoordinationBased,
            0,
        );
        let result = check_norm_violation(
            BehaviorPattern::CooperateWithStrangers,
            &[norm],
            &agent,
            0.7,
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_compute_punishment_probability() {
        let violator = create_test_agent(1, Sex::Male);
        let punisher = create_test_agent(2, Sex::Female);
        let norm = Norm::new(
            1,
            BehaviorPattern::ShareWithGroup,
            None,
            NormEmergenceType::FrequencyDependent,
            0,
        );
        let prob = compute_punishment_probability(&norm, &violator, &punisher, 0.8);
        assert!((0.0..=1.0).contains(&prob));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_select_sanction_type_repeat() {
        let agent = create_test_agent(1, Sex::Male);
        let norm = Norm::new(
            1,
            BehaviorPattern::ShareWithGroup,
            None,
            NormEmergenceType::FrequencyDependent,
            0,
        );
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let sanction = select_sanction_type(&norm, &agent, true, &mut rng);
        assert_eq!(sanction, SanctionType::SocialExclusion);
    }

    #[test]
    fn test_select_sanction_type_first_offense() {
        let agent = create_test_agent(1, Sex::Male);
        let norm = Norm::new(
            1,
            BehaviorPattern::ShareWithGroup,
            None,
            NormEmergenceType::FrequencyDependent,
            0,
        );
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let sanction = select_sanction_type(&norm, &agent, false, &mut rng);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert!(
            sanction == SanctionType::ReputationLoss || sanction == SanctionType::ResourcePenalty
        );
    }

    #[test]
    fn test_apply_sanction_reputation() {
        let mut agent = create_test_agent(1, Sex::Male);
        agent.reputation_score = 0.8;
        let norm = Norm::new(
            1,
            BehaviorPattern::ShareWithGroup,
            None,
            NormEmergenceType::FrequencyDependent,
            0,
        );
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let result = apply_sanction(
            &mut agent,
            None,
            &norm,
            SanctionType::ReputationLoss,
            &mut rng,
        );
        assert!(agent.reputation_score < 0.8);
        assert_eq!(result.violator_id, 1);
    }
}
