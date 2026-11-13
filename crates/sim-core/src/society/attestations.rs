//! Attestations and debt settlement subsystem
//!
//! SIM-002: Settlement and reneging with reputation consequences
//! Per §6.22: Attestation network mechanics

use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::agent::Agent;
use crate::types::{AgentId, AttestationId, Tick, C, O};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Attestation claim types
/// Per §6.22 line 2748
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimType {
    /// identity.
    Identity,
    /// skill.
    Skill,
    /// transaction.
    Transaction,
    /// cooperation.
    Cooperation,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Blockchain anchor status
/// Per §6.22 line 2757
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnchorStatus {
    /// none.
    None,
    /// pending.
    Pending,
    /// confirmed.
    Confirmed,
}

/// Attestation structure
/// Per §6.22 lines 2744-2758
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    /// id.
    pub id: AttestationId,
    /// attestor id.
    pub attestor_id: AgentId,
    /// subject id.
    pub subject_id: AgentId,
    /// claim type.
    pub claim_type: ClaimType,
    /// value.
    pub value: f32, // -1.0 (negative) to +1.0 (positive)
    /// tick created.
    pub tick_created: Tick,
    /// tick expires.
    pub tick_expires: Option<Tick>,
    /// stake.
    pub stake: f32, // Attestor's reputation risked (0.0-0.3)
    // Blockchain anchoring (Protocol ON only)
    /// attestation hash.
    pub attestation_hash: Option<[u8; 32]>,
    /// anchor tx id.
    pub anchor_tx_id: Option<String>,
    /// anchor block.
    pub anchor_block: Option<u64>,
    /// anchor status.
    pub anchor_status: AnchorStatus,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Interaction outcome for attestation decision
/// Per §6.22.3 line 3012
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionOutcome {
    /// cooperated.
    Cooperated,
    /// defected.
    Defected,
    /// neutral.
    Neutral,
}

/// Decide whether to create an attestation after an interaction
/// Per §6.22.3 lines 3008-3040
pub fn decide_attestation(
    attestor: &Agent,
    subject_id: AgentId,
    interaction_outcome: InteractionOutcome,
    current_tick: Tick,
    next_attestation_id: AttestationId,
    rng: &mut StdRng,
) -> Option<Attestation> {
    // Threshold based on oxytocin-analog (pro-social willingness)
    // Per §6.22.3 line 3015
    let attest_threshold = 0.5 - 0.2 * attestor.docas.levels[O]; // High O → lower threshold

    let value = match interaction_outcome {
        InteractionOutcome::Cooperated => 0.5 + rng.gen::<f32>() * 0.5, // [0.5, 1.0]
        InteractionOutcome::Defected => -0.5 - rng.gen::<f32>() * 0.5,  // [-1.0, -0.5]
        InteractionOutcome::Neutral => return None, // No attestation for neutral
    };
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    // Cost-benefit: attestation costs energy, benefits subject
    // Per §6.22.3 lines 3024-3026
    let energy_cost = 0.01 * attestor.energy;
    let willingness = attestor.docas.levels[O] * (1.0 - attestor.docas.levels[C]); // High O, low C → willing

    if willingness > attest_threshold && attestor.energy > energy_cost {
        Some(Attestation {
            id: next_attestation_id,
            attestor_id: attestor.id,
            subject_id,
            claim_type: ClaimType::Cooperation,
            value,
            tick_created: current_tick,
            tick_expires: Some(current_tick + 1000), // ~20% of lifespan
            stake: 0.1 * attestor.reputation_score.abs(), // Risk 10% of own reputation
            attestation_hash: None,
            anchor_tx_id: None,
            anchor_block: None,
            anchor_status: AnchorStatus::None,
        })
    } else {
        None
    }
}

// ============================================================================
// DEBT SETTLEMENT MECHANICS (SIM-002)
// Per §6.10.3: Debt creation from game outcomes
// Per SIM-002: Settlement and reneging with reputation consequences
// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Debt settlement decision factors
/// Per SIM-002: Agents decide to settle or renege based on multiple factors
#[derive(Debug, Clone, Copy)]
pub struct DebtDecisionFactors {
    /// Agent's current trust level (τ)
    pub trust: f32,
    /// Agent's current reputation
    pub reputation: f32,
    /// Debt amount relative to agent's energy
    pub debt_to_energy_ratio: f32,
    /// Creditor's reputation
    pub creditor_reputation: f32,
    /// Whether debt is overdue
    pub is_overdue: bool,
    /// Agent's worldview (higher = more Machiavellian)
    pub worldview: f32,
}

/// Decide whether to settle or renege on a debt
/// Per SIM-002: Settlement probability based on trust, reputation, and worldview
///
/// Returns `DebtDecision::Settle` if agent chooses to pay
/// Returns `DebtDecision::Renege` if agent chooses to default
pub fn decide_debt_settlement(
    factors: &DebtDecisionFactors,
    rng: &mut StdRng,
) -> crate::types::DebtDecision {
    // Base settlement probability
    let mut settle_probability = 0.5;

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    // High trust → more likely to settle (keep word)
    settle_probability += 0.2 * (factors.trust - 1.0);

    // High reputation → more likely to maintain it
    settle_probability += 0.15 * (factors.reputation - 0.5);

    // High creditor reputation → more likely to settle (respect creditor)
    settle_probability += 0.1 * (factors.creditor_reputation - 0.5);

    // High debt-to-energy ratio → less likely to settle (can't afford)
    settle_probability -= 0.3 * factors.debt_to_energy_ratio.min(1.0);

    // Overdue → slight penalty to settlement (already late)
    if factors.is_overdue {
        settle_probability -= 0.1;
    }

    // Worldview effect: Machiavellian agents (high worldview) less likely to settle
    // unless it's strategically beneficial (high creditor reputation)
    settle_probability -= 0.15 * factors.worldview;

    // Clamp probability to [0, 1]
    settle_probability = settle_probability.clamp(0.0, 1.0);

    // Make decision
    if rng.gen::<f32>() < settle_probability {
        crate::types::DebtDecision::Settle
    } else {
        crate::types::DebtDecision::Renege
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Process debt settlement between debtor and creditor
/// Per SIM-002: Updates both agents' states and reputations
///
/// Returns (`debtor_reputation_delta`, `creditor_reputation_delta`)
pub fn process_debt_settlement(
    debtor: &mut Agent,
    creditor: &mut Agent,
    debt_amount: f32,
    current_tick: Tick,
    rng: &mut StdRng,
) -> (f32, f32) {
    // Check if debtor has enough energy to pay
    let can_pay = debtor.energy >= debt_amount;

    if can_pay {
        // Transfer energy from debtor to creditor
        debtor.consume_energy(debt_amount);
        creditor.gain_energy(debt_amount);

        // Update reputations
        // Debtor: +0.05 for honoring debt
        let debtor_delta = 0.05;
        debtor.reputation_score = (debtor.reputation_score + debtor_delta).min(1.0);

        // Creditor: +0.02 for successful collection
        let creditor_delta = 0.02;
        creditor.reputation_score = (creditor.reputation_score + creditor_delta).min(1.0);

        // Update DOCAS: settlement increases Oxytocin (trustworthiness)
        debtor.docas.levels[O] = (debtor.docas.levels[O] + 0.05).min(1.0);

        // Record positive interaction for debtor
        debtor.record_positive_interaction();

        (debtor_delta, creditor_delta)
    } else {
        // Can't pay - forced renege
        process_debt_reneging(debtor, creditor, debt_amount, current_tick, rng)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Process debt reneging (default)
/// Per SIM-002: Severe reputation penalties with kin spillover
///
/// Returns (`debtor_reputation_delta`, `creditor_reputation_delta`)
pub fn process_debt_reneging(
    debtor: &mut Agent,
    _creditor: &mut Agent,
    _debt_amount: f32,
    _current_tick: Tick,
    _rng: &mut StdRng,
) -> (f32, f32) {
    // Debtor: -0.5 for reneging (severe penalty)
    let debtor_delta = -0.5;
    debtor.reputation_score = (debtor.reputation_score + debtor_delta).max(-1.0);

    // Creditor: no direct penalty (they're the victim)
    let creditor_delta = 0.0;

    // Update DOCAS: reneging increases Cortisol (stress from guilt/recognition)
    debtor.docas.levels[C] = (debtor.docas.levels[C] + 0.3).min(1.0);
    // And decreases Oxytocin (damaged trust relationships)
    debtor.docas.levels[O] = (debtor.docas.levels[O] - 0.1).max(0.0);

    // Record negative interaction for debtor
    debtor.record_negative_interaction();

    (debtor_delta, creditor_delta)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Apply kin spillover penalty for debt reneging
/// Per SIM-005: Kin inherit partial reputation damage
///
/// `penalty_amount`: base reputation penalty (typically -0.2)
pub fn apply_kin_spillover(agent: &mut Agent, penalty_amount: f32) {
    agent.reputation_score = (agent.reputation_score + penalty_amount).max(-1.0);
}

/// Propagate reputation through kin network (SIM-005)
/// Updates local reputation of all agents who are kin to the affected agent
///
/// Returns number of agents affected
pub fn propagate_reputation_to_kin(
    agents: &mut [Agent],
    affected_agent_id: AgentId,
    reputation_delta: f32,
    kin_spillover_factor: f32, // Typically 0.4 (40% of penalty propagates)
) -> usize {
    let spillover_delta = reputation_delta * kin_spillover_factor;
    let mut affected_count = 0;

    for agent in agents.iter_mut() {
        if agent.id != affected_agent_id {
            // Check if this agent has a local reputation entry for the affected agent
            // (indicating prior interaction/kinship knowledge)
            if agent.knows_reputation_of(affected_agent_id) {
                agent.update_local_reputation(affected_agent_id, spillover_delta);
                affected_count += 1;
            }
        }
    }

    affected_count
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
    fn test_decide_attestation_cooperated() {
        let mut attestor = create_test_agent(1, Sex::Male);
        attestor.docas.levels[crate::types::O] = 0.9; // High oxytocin
        attestor.energy = 100.0;
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let result = decide_attestation(
            &attestor,
            2,
            InteractionOutcome::Cooperated,
            10,
            1,
            &mut rng,
        );
        assert!(result.is_some());
        let attestation = result.unwrap();
        assert!(attestation.value > 0.0);
        assert_eq!(attestation.attestor_id, 1);
        assert_eq!(attestation.subject_id, 2);
    }

    #[test]
    fn test_decide_attestation_neutral() {
        let attestor = create_test_agent(1, Sex::Male);
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let result = decide_attestation(&attestor, 2, InteractionOutcome::Neutral, 10, 1, &mut rng);
        assert!(result.is_none());
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_decide_attestation_defected() {
        let mut attestor = create_test_agent(1, Sex::Male);
        attestor.docas.levels[crate::types::O] = 0.9;
        attestor.energy = 100.0;
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let result =
            decide_attestation(&attestor, 2, InteractionOutcome::Defected, 10, 1, &mut rng);
        assert!(result.is_some());
        let attestation = result.unwrap();
        assert!(attestation.value < 0.0);
    }

    #[test]
    fn test_apply_kin_spillover() {
        let mut agent = create_test_agent(1, Sex::Male);
        agent.reputation_score = 0.5;
        apply_kin_spillover(&mut agent, -0.2);
        assert!(agent.reputation_score < 0.5);
    }

    #[test]
    fn test_propagate_reputation_to_kin() {
        let mut agents = vec![
            create_test_agent(1, Sex::Male),
            create_test_agent(2, Sex::Female),
        ];
        // Agent 2 knows agent 1's reputation
        agents[1].update_local_reputation(1, 0.5);
        let affected = propagate_reputation_to_kin(&mut agents, 1, -0.3, 0.4);
        assert_eq!(affected, 1);
    }
}
