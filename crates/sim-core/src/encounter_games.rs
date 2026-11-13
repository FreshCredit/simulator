//! Encounter-level game theory processing
//!
//! This module contains the encounter-level game processing functions that
//! apply game-theoretic payoffs to agent state (energy, reputation, debt).
//! Pure game-theory logic lives in `crate::games`.

use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::agent::Agent;
use crate::games::{
    compute_trust_metrics, play_dictator_game, play_trust_game, PD_PAYOFF, STAG_HUNT_PAYOFF,
};
use crate::types::{AgentId, Tick};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Interaction outcome types for Rescorla-Wagner learning
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionOutcome {
    /// Positive interaction (cooperation, successful exchange)
    Positive,
    /// Negative interaction (defection, betrayal, conflict)
    Negative,
    /// Neutral interaction (no significant outcome)
    Neutral,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Apply Rescorla-Wagner trust updates based on interaction outcome
///
/// This function updates both agents' trust levels using the Rescorla-Wagner
/// learning rule: dτ/dt = α(R - τ)
///
/// # Arguments
/// * `agent1` - First agent
/// * `agent2` - Second agent
/// * `outcome` - Type of interaction outcome
pub fn apply_trust_update_rw(agent1: &mut Agent, agent2: &mut Agent, outcome: InteractionOutcome) {
    // Map outcome to reward signal (0-2 scale)
    let reward = match outcome {
        InteractionOutcome::Positive => 1.5, // High reward
        InteractionOutcome::Negative => 0.5, // Low reward
        InteractionOutcome::Neutral => 1.0,  // Neutral
    };

    // Apply RW update to both agents
    // Each agent learns based on their own experience of the interaction
    agent1.update_trust_rescorla_wagner(reward);
    agent2.update_trust_rescorla_wagner(reward);
}

/// Process Prisoner's Dilemma with actual strategies
pub(crate) fn process_prisoners_dilemma(
    agent1: &mut Agent,
    agent2: &mut Agent,
    a1_coop: bool,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    a2_coop: bool,
    tick: Tick,
) -> Option<(AgentId, AgentId, f32)> {
    // Apply payoffs from PD_PAYOFF matrix
    let payoff_idx_1 = usize::from(!a1_coop);
    let payoff_idx_2 = usize::from(!a2_coop);

    // PD_PAYOFF[our_action][partner_action]
    let payoff_1 = PD_PAYOFF[payoff_idx_1][payoff_idx_2];
    let payoff_2 = PD_PAYOFF[payoff_idx_2][payoff_idx_1];

    // Apply energy changes
    agent1.energy = (agent1.energy + payoff_1).min(100.0);
    agent2.energy = (agent2.energy + payoff_2).min(100.0);

    // Track interactions and update reputation
    match (a1_coop, a2_coop) {
        (true, true) => {
            // Both cooperate: (3,3)
            agent1.record_positive_interaction();
            agent2.record_positive_interaction();
            agent1.update_local_reputation(agent2.id, 0.1);
            agent2.update_local_reputation(agent1.id, 0.1);
            None
        }
        (false, true) => {
            // Agent1 defects, agent2 cooperates: (5,0)
            agent1.record_negative_interaction();
            agent2.record_positive_interaction();
            // TAG: surface=simulation owner=research-team rule=SIM-001
            agent1.update_local_reputation(agent2.id, 0.05);
            agent2.update_local_reputation(agent1.id, -0.2);
            // Defector creates debt to cooperator
            let debt_amount = 5.0;
            agent1.add_debt(agent2.id, debt_amount, tick, Some(tick + 100));
            Some((agent1.id, agent2.id, debt_amount))
        }
        (true, false) => {
            // Agent1 cooperates, agent2 defects: (0,5)
            agent1.record_positive_interaction();
            agent2.record_negative_interaction();
            agent1.update_local_reputation(agent2.id, -0.2);
            agent2.update_local_reputation(agent1.id, 0.05);
            let debt_amount = 5.0;
            agent2.add_debt(agent1.id, debt_amount, tick, Some(tick + 100));
            Some((agent2.id, agent1.id, debt_amount))
        }
        (false, false) => {
            // Both defect: (1,1)
            agent1.record_negative_interaction();
            agent2.record_negative_interaction();
            agent1.update_local_reputation(agent2.id, -0.15);
            agent2.update_local_reputation(agent1.id, -0.15);
            None
        }
    }
}

/// Process Stag Hunt with actual strategies
pub(crate) fn process_stag_hunt(
    agent1: &mut Agent,
    agent2: &mut Agent,
    a1_stag: bool, // true = Stag (cooperate), false = Hare (defect)
    a2_stag: bool,
) -> Option<(AgentId, AgentId, f32)> {
    // Apply payoffs
    let payoff_idx_1 = usize::from(!a1_stag);
    let payoff_idx_2 = usize::from(!a2_stag);

    let payoff_1 = STAG_HUNT_PAYOFF[payoff_idx_1][payoff_idx_2];
    let payoff_2 = STAG_HUNT_PAYOFF[payoff_idx_2][payoff_idx_1];

    // TAG: surface=simulation owner=research-team rule=SIM-001
    agent1.energy = (agent1.energy + payoff_1).min(100.0);
    agent2.energy = (agent2.energy + payoff_2).min(100.0);

    // Track interactions
    match (a1_stag, a2_stag) {
        (true, true) => {
            // Both hunt Stag: (4,4) - best outcome
            agent1.record_positive_interaction();
            agent2.record_positive_interaction();
            agent1.update_local_reputation(agent2.id, 0.15);
            agent2.update_local_reputation(agent1.id, 0.15);
        }
        (true, false) => {
            // Agent1 hunts Stag, Agent2 hunts Hare: (0,3)
            agent1.record_positive_interaction(); // Tried to cooperate
            agent2.record_negative_interaction(); // Played safe
            agent1.update_local_reputation(agent2.id, -0.1);
            agent2.update_local_reputation(agent1.id, 0.0);
        }
        (false, true) => {
            // Agent1 hunts Hare, Agent2 hunts Stag: (3,0)
            agent1.record_negative_interaction();
            agent2.record_positive_interaction();
            agent1.update_local_reputation(agent2.id, 0.0);
            agent2.update_local_reputation(agent1.id, -0.1);
        }
        (false, false) => {
            // Both hunt Hare: (2,2) - safe but suboptimal
            agent1.record_positive_interaction(); // Not a betrayal
            agent2.record_positive_interaction();
            agent1.update_local_reputation(agent2.id, 0.05);
            agent2.update_local_reputation(agent1.id, 0.05);
        }
    }

    None // No debt in Stag Hunt
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Process Ultimatum Game
pub(crate) fn process_ultimatum(
    agent1: &mut Agent, // Proposer
    agent2: &mut Agent, // Responder
    a1_fair: bool,      // true = fair offer (40-50%), false = unfair
    a2_accepts: bool,   // true = accepts, false = rejects
) -> Option<(AgentId, AgentId, f32)> {
    // Standard ultimatum: 10 energy to split
    // Pot = 10.0 (used implicitly in share calculations)

    if a2_accepts {
        // Offer accepted
        let (share1, share2) = if a1_fair {
            (5.0, 5.0) // Fair split
        } else {
            (7.0, 3.0) // Unfair split
        };

        agent1.energy = (agent1.energy + share1).min(100.0);
        agent2.energy = (agent2.energy + share2).min(100.0);

        agent1.record_positive_interaction();
        agent2.record_positive_interaction();

        if a1_fair {
            agent1.update_local_reputation(agent2.id, 0.15);
            agent2.update_local_reputation(agent1.id, 0.15);
        } else {
            agent1.update_local_reputation(agent2.id, 0.05); // Recipient accepted offer
            agent2.update_local_reputation(agent1.id, -0.05); // Recipient penalizes unfair split
        }
    } else {
        // Offer rejected - both get nothing
        agent1.record_negative_interaction();
        agent2.record_negative_interaction();
        agent1.update_local_reputation(agent2.id, -0.1);
        agent2.update_local_reputation(agent1.id, -0.1);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    None
}

/// Process Public Goods Game
pub(crate) fn process_public_goods(
    agent1: &mut Agent,
    agent2: &mut Agent,
    a1_contrib: bool, // true = contributes, false = free rides
    a2_contrib: bool,
) -> Option<(AgentId, AgentId, f32)> {
    // 2-player public goods: each can contribute 2 energy
    // Total contribution × 1.5 is split equally
    const CONTRIBUTION: f32 = 2.0;
    const MULTIPLIER: f32 = 1.5;

    let total_contrib =
        if a1_contrib { CONTRIBUTION } else { 0.0 } + if a2_contrib { CONTRIBUTION } else { 0.0 };

    let public_good = total_contrib * MULTIPLIER;
    let share = public_good / 2.0;

    // Apply payoffs
    let payoff_1 = share - if a1_contrib { CONTRIBUTION } else { 0.0 };
    let payoff_2 = share - if a2_contrib { CONTRIBUTION } else { 0.0 };

    agent1.energy = (agent1.energy + payoff_1).min(100.0);
    agent2.energy = (agent2.energy + payoff_2).min(100.0);

    // Track interactions
    match (a1_contrib, a2_contrib) {
        (true, true) => {
            agent1.record_positive_interaction();
            agent2.record_positive_interaction();
            agent1.update_local_reputation(agent2.id, 0.1);
            agent2.update_local_reputation(agent1.id, 0.1);
        }
        (false, true) => {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            agent1.record_negative_interaction(); // Free rider
            agent2.record_positive_interaction();
            agent1.update_local_reputation(agent2.id, 0.05);
            agent2.update_local_reputation(agent1.id, -0.15);
        }
        (true, false) => {
            agent1.record_positive_interaction();
            agent2.record_negative_interaction(); // Free rider
            agent1.update_local_reputation(agent2.id, -0.15);
            agent2.update_local_reputation(agent1.id, 0.05);
        }
        (false, false) => {
            agent1.record_negative_interaction();
            agent2.record_negative_interaction();
            agent1.update_local_reputation(agent2.id, -0.05);
            agent2.update_local_reputation(agent1.id, -0.05);
        }
    }

    None
}

/// Process Dictator Game
/// Per §6.10.6: Dictator proposes split, recipient has no choice
pub(crate) fn process_dictator_game(
    dictator: &mut Agent,
    recipient: &mut Agent,
) -> Option<(AgentId, AgentId, f32)> {
    let stake = 10.0;
    let mut rng = StdRng::seed_from_u64(dictator.id.wrapping_add(recipient.id));

    let tau = dictator.compute_trust();
    let (dictator_payoff, recipient_payoff) = play_dictator_game(tau, stake, &mut rng);

    // Apply payoffs
    dictator.energy = (dictator.energy + dictator_payoff).min(100.0);
    recipient.energy = (recipient.energy + recipient_payoff).min(100.0);

    // Record interaction
    let offer_fraction = recipient_payoff / stake;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    if offer_fraction >= 0.4 {
        // Fair offer: positive interaction for both
        dictator.record_positive_interaction();
        recipient.record_positive_interaction();
        dictator.update_local_reputation(recipient.id, 0.1);
        recipient.update_local_reputation(dictator.id, 0.1);
    } else if offer_fraction >= 0.2 {
        // Moderate offer: neutral
        dictator.record_positive_interaction();
        recipient.update_local_reputation(dictator.id, 0.05);
    } else {
        // Selfish offer: negative for recipient's view
        dictator.record_negative_interaction();
        recipient.update_local_reputation(dictator.id, -0.1);
    }

    None
}

/// Process Trust Game (Investment Game)
/// Per §6.10.6: Investor sends money (multiplied), trustee decides return
pub(crate) fn process_trust_game(
    investor: &mut Agent,
    trustee: &mut Agent,
    _investor_cooperates: bool, // true = invest, false = keep
    _trustee_cooperates: bool,  // true = return fair amount, false = keep all
) -> Option<(AgentId, AgentId, f32)> {
    let endowment = 10.0;
    let multiplier = 3.0;
    let mut rng = StdRng::seed_from_u64(investor.id.wrapping_add(trustee.id));

    let investor_tau = investor.compute_trust();
    let trustee_tau = trustee.compute_trust();

    let (inv_payoff, trustee_payoff, investment, returned) =
        play_trust_game(investor_tau, trustee_tau, endowment, multiplier, &mut rng);

    // Apply payoffs
    investor.energy = (investor.energy + inv_payoff - endowment).min(100.0);
    trustee.energy = (trustee.energy + trustee_payoff).min(100.0);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Compute metrics for tracking
    let (_trust_index, trustworthiness, efficiency) =
        compute_trust_metrics(endowment, investment, returned, investment * multiplier);

    // Record interactions based on trustworthiness
    if trustworthiness >= 0.5 {
        // High return: mutual trust
        investor.record_positive_interaction();
        trustee.record_positive_interaction();
        investor.update_local_reputation(trustee.id, 0.15);
        trustee.update_local_reputation(investor.id, 0.1);
    } else if trustworthiness >= 0.2 {
        // Moderate return: acceptable
        investor.record_positive_interaction();
        trustee.record_positive_interaction();
        investor.update_local_reputation(trustee.id, 0.05);
    } else {
        // Low return: betrayal
        investor.record_negative_interaction();
        trustee.record_negative_interaction();
        investor.update_local_reputation(trustee.id, -0.2);
        trustee.update_local_reputation(investor.id, -0.1);
    }

    // Track gains from trade
    if efficiency > 1.0 {
        // Both benefited from trust
        investor.gain_energy(0.5);
        trustee.gain_energy(0.5);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::docas::DocasState;
    use crate::genome::Genome;
    use crate::types::{Position, Sex};

    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn create_test_agent(id: AgentId, sex: Sex) -> Agent {
        let genome = Genome::default();
        Agent::new(id, Position::new_2d(0.0, 0.0), genome, 0, sex)
    }

    fn create_high_trust_agent(id: AgentId, sex: Sex) -> Agent {
        let genome = Genome::default();
        let mut agent = Agent::new(id, Position::new_2d(0.0, 0.0), genome, 0, sex);
        agent.docas = DocasState::new([0.95, 0.95, 0.05, 0.05, 0.9]);
        agent
    }

    #[test]
    fn test_pd_both_cooperate() {
        let mut a1 = create_test_agent(1, Sex::Male);
        let mut a2 = create_test_agent(2, Sex::Female);
        let e1 = a1.energy;
        let e2 = a2.energy;

        let debt = process_prisoners_dilemma(&mut a1, &mut a2, true, true, 10);

        assert!(debt.is_none());
        assert_eq!(a1.energy, (e1 + 3.0).min(100.0));
        assert_eq!(a2.energy, (e2 + 3.0).min(100.0));
    }

    #[test]
    fn test_pd_a1_defects() {
        let mut a1 = create_test_agent(1, Sex::Male);
        let mut a2 = create_test_agent(2, Sex::Female);
        let e1 = a1.energy;
        let e2 = a2.energy;

        let debt = process_prisoners_dilemma(&mut a1, &mut a2, false, true, 10);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(debt.is_some());
        let (debtor, creditor, amount) = debt.unwrap();
        assert_eq!(debtor, a1.id);
        assert_eq!(creditor, a2.id);
        assert_eq!(amount, 5.0);
        assert_eq!(a1.energy, (e1 + 5.0).min(100.0));
        assert_eq!(a2.energy, e2); // 0 payoff
    }

    #[test]
    fn test_pd_a2_defects() {
        let mut a1 = create_test_agent(1, Sex::Male);
        let mut a2 = create_test_agent(2, Sex::Female);
        let e1 = a1.energy;
        let e2 = a2.energy;

        let debt = process_prisoners_dilemma(&mut a1, &mut a2, true, false, 10);

        assert!(debt.is_some());
        let (debtor, creditor, amount) = debt.unwrap();
        assert_eq!(debtor, a2.id);
        assert_eq!(creditor, a1.id);
        assert_eq!(amount, 5.0);
        assert_eq!(a1.energy, e1); // 0 payoff
        assert_eq!(a2.energy, (e2 + 5.0).min(100.0));
    }

    #[test]
    fn test_pd_both_defect() {
        let mut a1 = create_test_agent(1, Sex::Male);
        let mut a2 = create_test_agent(2, Sex::Female);
        let e1 = a1.energy;
        let e2 = a2.energy;

        let debt = process_prisoners_dilemma(&mut a1, &mut a2, false, false, 10);

        assert!(debt.is_none());
        assert_eq!(a1.energy, (e1 + 1.0).min(100.0));
        assert_eq!(a2.energy, (e2 + 1.0).min(100.0));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_stag_hunt_both_stag() {
        let mut a1 = create_test_agent(1, Sex::Male);
        let mut a2 = create_test_agent(2, Sex::Female);
        let e1 = a1.energy;
        let e2 = a2.energy;

        let debt = process_stag_hunt(&mut a1, &mut a2, true, true);

        assert!(debt.is_none());
        assert_eq!(a1.energy, (e1 + 4.0).min(100.0));
        assert_eq!(a2.energy, (e2 + 4.0).min(100.0));
    }

    #[test]
    fn test_stag_hunt_a1_hare() {
        let mut a1 = create_test_agent(1, Sex::Male);
        let mut a2 = create_test_agent(2, Sex::Female);
        let e1 = a1.energy;
        let e2 = a2.energy;

        let debt = process_stag_hunt(&mut a1, &mut a2, false, true);

        assert!(debt.is_none());
        assert_eq!(a1.energy, (e1 + 3.0).min(100.0));
        assert_eq!(a2.energy, e2); // 0 payoff
    }

    #[test]
    fn test_stag_hunt_both_hare() {
        let mut a1 = create_test_agent(1, Sex::Male);
        let mut a2 = create_test_agent(2, Sex::Female);
        let e1 = a1.energy;
        let e2 = a2.energy;

        let debt = process_stag_hunt(&mut a1, &mut a2, false, false);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(debt.is_none());
        assert_eq!(a1.energy, (e1 + 2.0).min(100.0));
        assert_eq!(a2.energy, (e2 + 2.0).min(100.0));
    }

    #[test]
    fn test_ultimatum_fair_accepted() {
        let mut a1 = create_test_agent(1, Sex::Male);
        let mut a2 = create_test_agent(2, Sex::Female);
        let e1 = a1.energy;
        let e2 = a2.energy;

        let debt = process_ultimatum(&mut a1, &mut a2, true, true);

        assert!(debt.is_none());
        assert_eq!(a1.energy, (e1 + 5.0).min(100.0));
        assert_eq!(a2.energy, (e2 + 5.0).min(100.0));
    }

    #[test]
    fn test_ultimatum_unfair_accepted() {
        let mut a1 = create_test_agent(1, Sex::Male);
        let mut a2 = create_test_agent(2, Sex::Female);
        let e1 = a1.energy;
        let e2 = a2.energy;

        let debt = process_ultimatum(&mut a1, &mut a2, false, true);

        assert!(debt.is_none());
        assert_eq!(a1.energy, (e1 + 7.0).min(100.0));
        assert_eq!(a2.energy, (e2 + 3.0).min(100.0));
    }

    #[test]
    fn test_ultimatum_rejected() {
        let mut a1 = create_test_agent(1, Sex::Male);
        let mut a2 = create_test_agent(2, Sex::Female);
        let e1 = a1.energy;
        let e2 = a2.energy;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let debt = process_ultimatum(&mut a1, &mut a2, true, false);

        assert!(debt.is_none());
        assert_eq!(a1.energy, e1);
        assert_eq!(a2.energy, e2);
    }

    #[test]
    fn test_public_goods_both_contribute() {
        let mut a1 = create_test_agent(1, Sex::Male);
        let mut a2 = create_test_agent(2, Sex::Female);
        let e1 = a1.energy;
        let e2 = a2.energy;

        // Both contribute 2, total = 4, public good = 6, share = 3
        // payoff = 3 - 2 = 1 each
        let debt = process_public_goods(&mut a1, &mut a2, true, true);

        assert!(debt.is_none());
        assert_eq!(a1.energy, (e1 + 1.0).min(100.0));
        assert_eq!(a2.energy, (e2 + 1.0).min(100.0));
    }

    #[test]
    fn test_public_goods_free_rider() {
        let mut a1 = create_test_agent(1, Sex::Male);
        let mut a2 = create_test_agent(2, Sex::Female);
        let e1 = a1.energy;
        let e2 = a2.energy;

        // A1 free rides, A2 contributes
        // total = 2, public good = 3, share = 1.5
        // A1 payoff = 1.5 - 0 = 1.5
        // A2 payoff = 1.5 - 2 = -0.5
        let debt = process_public_goods(&mut a1, &mut a2, false, true);

        assert!(debt.is_none());
        assert_eq!(a1.energy, (e1 + 1.5).min(100.0));
        assert_eq!(a2.energy, (e2 - 0.5).min(100.0));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_public_goods_neither_contributes() {
        let mut a1 = create_test_agent(1, Sex::Male);
        let mut a2 = create_test_agent(2, Sex::Female);
        let e1 = a1.energy;
        let e2 = a2.energy;

        let debt = process_public_goods(&mut a1, &mut a2, false, false);

        assert!(debt.is_none());
        assert_eq!(a1.energy, e1);
        assert_eq!(a2.energy, e2);
    }

    #[test]
    fn test_dictator_game_runs() {
        let mut dictator = create_high_trust_agent(1, Sex::Male);
        let mut recipient = create_test_agent(2, Sex::Female);
        let e1 = dictator.energy;
        let e2 = recipient.energy;

        let debt = process_dictator_game(&mut dictator, &mut recipient);

        assert!(debt.is_none());
        assert!(dictator.energy >= e1);
        assert!(recipient.energy >= e2);
    }

    #[test]
    fn test_trust_game_runs() {
        let mut investor = create_high_trust_agent(1, Sex::Male);
        let mut trustee = create_high_trust_agent(2, Sex::Female);
        let _e1 = investor.energy;

        let debt = process_trust_game(&mut investor, &mut trustee, true, true);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(debt.is_none());
        // Energy may go up or down depending on random outcomes
        assert!(investor.energy >= 0.0);
        assert!(trustee.energy >= 0.0);
    }

    #[test]
    fn test_apply_trust_update_rw_positive() {
        let mut a1 = create_test_agent(1, Sex::Male);
        let mut a2 = create_test_agent(2, Sex::Female);
        let prev_tau = a1.compute_trust();

        apply_trust_update_rw(&mut a1, &mut a2, InteractionOutcome::Positive);

        assert!(
            a1.compute_trust() >= prev_tau,
            "positive reward must increase or maintain trust"
        );
        assert!(!a1.tau_history.is_empty(), "tau history must be recorded");
    }

    #[test]
    fn test_apply_trust_update_rw_negative() {
        let mut a1 = create_test_agent(1, Sex::Male);
        let mut a2 = create_test_agent(2, Sex::Female);
        let prev_tau = a1.compute_trust();

        apply_trust_update_rw(&mut a1, &mut a2, InteractionOutcome::Negative);

        assert!(
            a1.compute_trust() <= prev_tau,
            "negative reward must decrease or maintain trust"
        );
        assert!(!a1.tau_history.is_empty(), "tau history must be recorded");
    }

    #[test]
    fn test_apply_trust_update_rw_neutral() {
        let mut a1 = create_test_agent(1, Sex::Male);
        let mut a2 = create_test_agent(2, Sex::Female);

        apply_trust_update_rw(&mut a1, &mut a2, InteractionOutcome::Neutral);

        assert!(a1.compute_trust().is_finite());
        assert!(!a1.tau_history.is_empty(), "tau history must be recorded");
    }

    #[test]
    fn test_pd_and_public_goods_betrayed_cooperator_downgrades_defector() {
        let mut cooperator = create_test_agent(1, Sex::Male);
        let mut defector = create_test_agent(2, Sex::Female);

        // In PD: cooperator plays true, defector plays false
        process_prisoners_dilemma(&mut cooperator, &mut defector, true, false, 1);

        // Cooperator must downgrade opinion of defector
        let coop_view_of_def = cooperator.get_local_reputation(defector.id);
        let def_view_of_coop = defector.get_local_reputation(cooperator.id);
        assert!(
            coop_view_of_def < 0.5,
            "Betrayed cooperator must downgrade defector reputation: got {coop_view_of_def}"
        );
        assert!(
            def_view_of_coop > 0.5,
            "Defector must observe cooperative behavior: got {def_view_of_coop}"
        );

        // In Public Goods: contributor contributes, free-rider free-rides
        let mut contributor = create_test_agent(3, Sex::Male);
        let mut freerider = create_test_agent(4, Sex::Female);
        process_public_goods(&mut contributor, &mut freerider, true, false);

        let contrib_view = contributor.get_local_reputation(freerider.id);
        let freerider_view = freerider.get_local_reputation(contributor.id);
        assert!(
            contrib_view < 0.5,
            "Exploited contributor must downgrade free rider: got {contrib_view}"
        );
        assert!(
            freerider_view > 0.5,
            "Free rider must observe cooperative contribution: got {freerider_view}"
        );
    }
}
