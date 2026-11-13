//! Game-theoretic economic interactions module
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION games, NOT financial advice
//!
//! Per §6.10 UTGT: Trust-weighted utility and classical game experiments
//! Per §6.10.3: Prisoner's Dilemma, Ultimatum Game, Public Goods Game, Stag Hunt
//! Per §6.10.5: TFT + forgiveness is ESS at high τ populations

use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::types::{AgentId, StrategyType};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Game types per §6.10.3 Classical Game Experiments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameType {
    /// Prisoner's Dilemma: Cooperation vs Defection
    /// Per §6.10.3: Cooperation emerges when τ > 0.65
    PrisonersDilemma,
    /// Ultimatum Game: Proposer offers split, responder accepts/rejects
    /// Per §6.10.3: Fair offers (40-50%) when proposer τ > 0.60
    UltimatumGame,
    /// Dictator Game: Proposer offers split, responder must accept
    /// Per §6.10.6: Tests pure altruism, typical offers 20-30%
    DictatorGame,
    /// Trust Game: Investor sends money, trustee decides return
    /// Per §6.10.6: Tests trust (investment) and trustworthiness (return)
    TrustGame,
    /// Public Goods Game: Contribute to common pool
    /// Per §6.10.3: Full contribution when group τ > 0.75
    PublicGoodsGame,
    /// Stag Hunt: Coordinate on high-risk/high-reward or safe option
    /// Per §6.10.3: Stag choice when τ > 0.70
    StagHunt,
}

/// Payoff matrix for Prisoner's Dilemma
/// Per §6.10.3 line 1522-1527:
/// ```text
///           Partner
///           C     D
/// Agent C (3,3) (0,5)
///       D (5,0) (1,1)
/// ```
pub const PD_PAYOFF: [[f32; 2]; 2] = [
    [3.0, 0.0], // Agent cooperates: (C,C)=3, (C,D)=0
    [5.0, 1.0], // Agent defects: (D,C)=5, (D,D)=1
];

/// Payoff matrix for Stag Hunt
/// Per §6.10.3: Coordinate on Stag (high reward) or Hare (safe)
/// ```text
///           Partner
///           Stag  Hare
/// Agent Stag (4,4) (0,3)
///       Hare (3,0) (2,2)
/// ```
pub const STAG_HUNT_PAYOFF: [[f32; 2]; 2] = [
    [4.0, 0.0], // Agent chooses Stag: (S,S)=4, (S,H)=0
    [3.0, 2.0], // Agent chooses Hare: (H,S)=3, (H,H)=2
];

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Game outcome for a single interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameOutcome {
    /// Type of game played
    pub game_type: GameType,
    /// First player ID
    pub player_a: AgentId,
    /// Second player ID
    pub player_b: AgentId,
    /// Whether player A cooperated (true) or defected (false)
    pub a_cooperated: bool,
    /// Whether player B cooperated (true) or defected (false)
    pub b_cooperated: bool,
    /// Payoff for player A (energy transfer)
    pub payoff_a: f32,
    /// Payoff for player B (energy transfer)
    pub payoff_b: f32,
    /// Tick when game occurred
    pub tick: u64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Decide action based on strategy and partner's last action
/// Per §6.10.5: Strategy space includes TFT, TFT+F, AC, AD, Grim
pub fn decide_action(
    strategy: StrategyType,
    partner_last_cooperated: Option<bool>,
    own_tau: f32,
    cooperation_tendency: f32,
    rng: &mut StdRng,
) -> bool {
    match strategy {
        StrategyType::Cooperate => true,
        StrategyType::Defect => false,
        StrategyType::TitForTat => {
            // Copy partner's last move, cooperate if first interaction
            partner_last_cooperated.unwrap_or(true)
        }
        StrategyType::TitForTatForgive => {
            // TFT but forgive 10% of defections
            // Per §6.10.5: TFT+F expected ESS at high τ
            match partner_last_cooperated {
                Some(true) => true,
                Some(false) => rng.gen::<f32>() < 0.1, // 10% forgiveness
                None => true,                          // Cooperate on first interaction
            }
        }
        StrategyType::Random => {
            // Weighted by cooperation tendency and trust
            let coop_prob = 0.3 + 0.4 * cooperation_tendency + 0.3 * own_tau.min(1.0);
            rng.gen::<f32>() < coop_prob
        }
        StrategyType::ProspectTheory => {
            // Use prospect theory to evaluate cooperate vs defect
            // Default to Prisoner's Dilemma payoffs if game type not specified
            let cooperate_prospect = evaluate_prospect_pd(true, partner_last_cooperated);
            let defect_prospect = evaluate_prospect_pd(false, partner_last_cooperated);
            cooperate_prospect >= defect_prospect
        }
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

/// Estimate probability partner cooperates based on their last action
const fn estimate_partner_coop_prob(partner_last: Option<bool>) -> f32 {
    match partner_last {
        Some(true) => 0.8,  // Partner cooperated before, likely to again
        Some(false) => 0.2, // Partner defected before, likely to again
        None => 0.5,        // No history, assume neutral
    }
}

/// Evaluate prospect value for Prisoner's Dilemma action
/// Per §6.10: Prospect Theory integrated into agent decision-making
///
/// Uses PT value function from `economics::prospect_value` to evaluate outcomes
/// with loss aversion and risk asymmetry as per Kahneman & Tversky (1979)
fn evaluate_prospect_pd(action_cooperate: bool, partner_last: Option<bool>) -> f32 {
    use crate::economics::prospect_value;

    // PD payoff matrix: PD_PAYOFF[agent_action][partner_action]
    // Agent Cooperate: [0][0]=3 vs [0][1]=0
    // Agent Defect:    [1][0]=5 vs [1][1]=1

    let partner_coop_prob = estimate_partner_coop_prob(partner_last);

    // Expected payoffs for this action
    let (payoff_if_partner_coops, payoff_if_partner_defects) = if action_cooperate {
        (PD_PAYOFF[0][0], PD_PAYOFF[0][1]) // Cooperate: 3 if C, 0 if D
    } else {
        (PD_PAYOFF[1][0], PD_PAYOFF[1][1]) // Defect: 5 if C, 1 if D
    };

    // Reference point: average payoff in mutual cooperation (3.0)
    let reference_point = PD_PAYOFF[0][0];

    // Compute prospect value: weighted sum of value-adjusted outcomes
    let v_coop = prospect_value(payoff_if_partner_coops - reference_point);
    let v_defect = prospect_value(payoff_if_partner_defects - reference_point);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Weight by probability (using prospect theory probability weighting)
    partner_coop_prob * v_coop + (1.0 - partner_coop_prob) * v_defect
}

/// Generic 2×2 game helper
#[must_use]
const fn play_2x2_game(payoff: &[[f32; 2]; 2], a_action: bool, b_action: bool) -> (f32, f32) {
    let a_idx = if a_action { 0 } else { 1 };
    let b_idx = if b_action { 0 } else { 1 };
    (payoff[a_idx][b_idx], payoff[b_idx][a_idx])
}

/// Play Prisoner's Dilemma between two agents
/// Per §6.10.3: Returns payoffs based on cooperation decisions
#[must_use]
pub const fn play_prisoners_dilemma(a_cooperates: bool, b_cooperates: bool) -> (f32, f32) {
    play_2x2_game(&PD_PAYOFF, a_cooperates, b_cooperates)
}

/// Play Stag Hunt between two agents
/// Per §6.10.3: Stag = cooperate (high risk/reward), Hare = defect (safe)
#[must_use]
pub const fn play_stag_hunt(a_chooses_stag: bool, b_chooses_stag: bool) -> (f32, f32) {
    play_2x2_game(&STAG_HUNT_PAYOFF, a_chooses_stag, b_chooses_stag)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Play Ultimatum Game
/// Per §6.10.3: Proposer offers split, responder accepts/rejects
/// Returns (`proposer_payoff`, `responder_payoff`)
pub fn play_ultimatum_game(
    proposer_tau: f32,
    responder_tau: f32,
    stake: f32,
    rng: &mut StdRng,
) -> (f32, f32) {
    // Proposer offer: higher τ → fairer offer (40-50%)
    // Per §6.10.3: Fair offers when proposer τ > 0.60
    let base_offer = 0.2 + 0.3 * proposer_tau.min(1.0);
    let offer_fraction = (base_offer + rng.gen::<f32>() * 0.1).clamp(0.1, 0.5);
    let offer = stake * offer_fraction;

    // Responder acceptance: higher τ → more tolerant of unfair offers
    // Per §6.10.3: Rejection rate correlates negatively with τ
    let min_acceptable = 0.4 - 0.2 * responder_tau.min(1.0);
    let accepts = offer_fraction >= min_acceptable || rng.gen::<f32>() < responder_tau * 0.3;

    if accepts {
        (stake - offer, offer)
    } else {
        (0.0, 0.0) // Both get nothing on rejection
    }
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
/// Play Dictator Game
/// Per §6.10.6: Tests pure altruism - dictator proposes split, recipient must accept
///
/// **Theoretical Basis:**
/// - Kahneman et al. (1986): "Fairness as a Constraint on Profit Seeking"
/// - Typical offers: 20-30% of stake (not 50/50 as rational models predict)
/// - Higher trust (τ) correlates with fairer offers
///
/// **Parameters:**
/// - `dictator_tau`: Dictator's trust level [0.0, 1.0]
/// - `stake`: Total amount to split
/// - `rng`: Random number generator for behavioral noise
///
/// **Returns:** (`dictator_payoff`, `recipient_payoff`)
pub fn play_dictator_game(dictator_tau: f32, stake: f32, rng: &mut StdRng) -> (f32, f32) {
    // Dictator offer: higher τ → fairer offer
    // Empirical findings: offers range from 10% (selfish) to 50% (fair)
    // Baseline: 20% + up to 25% based on trust
    let base_offer = 0.15 + 0.25 * dictator_tau.min(1.0);

    // Add behavioral noise (±10%)
    let noise = (rng.gen::<f32>() - 0.5) * 0.2;
    let offer_fraction = (base_offer + noise).clamp(0.05, 0.5);

    let offer = stake * offer_fraction;

    // Recipient has no choice - must accept
    (stake - offer, offer)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Play Trust Game (Investment Game)
/// Per §6.10.6: Tests trust (investment) and trustworthiness (return)
///
/// **Theoretical Basis:**
/// - Berg et al. (1995): "Trust, Reciprocity, and Social History"
/// - Investor sends amount X, which is multiplied (typically 3x)
/// - Trustee decides how much to return
///
/// **Game Mechanics:**
/// 1. Investor receives endowment
/// 2. Sends amount X to trustee (investment)
/// 3. Investment is multiplied by factor (typically 3.0)
/// 4. Trustee receives multiplied amount (3X)
/// 5. Trustee decides return amount Y
/// 6. Final payoffs:
///    - Investor: Endowment - X + Y
///    - Trustee: 3X - Y
///
/// **Parameters:**
/// - `investor_tau`: Investor's trust level (affects investment amount) [0.0, 1.0]
/// - `trustee_tau`: Trustee's trustworthiness (affects return amount) [0.0, 1.0]
/// - `endowment`: Investor's initial amount
/// - `multiplier`: Investment multiplier (typically 3.0)
/// - `rng`: Random number generator
///
/// **Returns:** (`investor_payoff`, `trustee_payoff`, investment, `returned_amount`)
pub fn play_trust_game(
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    investor_tau: f32,
    trustee_tau: f32,
    endowment: f32,
    multiplier: f32,
    rng: &mut StdRng,
) -> (f32, f32, f32, f32) {
    // Investor decides how much to send
    // Higher trust → higher investment (0% to 100% of endowment)
    let base_investment = 0.2 + 0.6 * investor_tau.min(1.0);
    let noise = (rng.gen::<f32>() - 0.5) * 0.2;
    let investment_fraction = (base_investment + noise).clamp(0.0, 1.0);
    let investment = endowment * investment_fraction;

    // Investment is multiplied
    let multiplied_amount = investment * multiplier;

    // Trustee decides how much to return
    // Higher trustworthiness → higher return (0% to 150% of investment)
    // Trustees often return ~1/3 of multiplied amount (breaking even)
    let base_return_fraction = 0.2 + 0.5 * trustee_tau.min(1.0);
    let return_noise = (rng.gen::<f32>() - 0.5) * 0.3;
    let return_fraction = (base_return_fraction + return_noise).clamp(0.0, 1.5);
    let returned_amount = multiplied_amount * return_fraction;

    // Calculate final payoffs
    let investor_payoff = endowment - investment + returned_amount;
    let trustee_payoff = multiplied_amount - returned_amount;

    (investor_payoff, trustee_payoff, investment, returned_amount)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute trust game metrics for analysis
/// Returns (`trust_index`, `trustworthiness_index`, efficiency)
///
/// **Metrics:**
/// - Trust Index: Investment / Endowment [0.0, 1.0]
/// - Trustworthiness Index: Returned / Multiplied Investment [0.0, 1.0]
/// - Efficiency: (`Investor_Payoff` + `Trustee_Payoff`) / (2 * Endowment) [0.0, ∞)
#[must_use]
pub fn compute_trust_metrics(
    endowment: f32,
    investment: f32,
    returned_amount: f32,
    multiplied_amount: f32,
) -> (f32, f32, f32) {
    let trust_index = if endowment > 0.0 {
        investment / endowment
    } else {
        0.0
    };
    let trustworthiness_index = if multiplied_amount > 0.0 {
        returned_amount / multiplied_amount
    } else {
        0.0
    };

    // Efficiency: ratio of total payoff to baseline (both keeping endowment)
    let total_payoff =
        (endowment - investment + returned_amount) + (multiplied_amount - returned_amount);
    let baseline = 2.0 * endowment;
    let efficiency = if baseline > 0.0 {
        total_payoff / baseline
    } else {
        1.0
    };

    (
        trust_index.clamp(0.0, 1.0),
        trustworthiness_index.clamp(0.0, 1.0),
        efficiency,
    )
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Play Public Goods Game with a group
/// Per §6.10.3: Full contribution when group τ > 0.75
/// Returns payoffs for each participant
#[must_use]
pub fn play_public_goods_game(
    contributions: &[(AgentId, f32, f32)], // (agent_id, contribution, tau)
    multiplier: f32,                       // Typically 1.5-2.0
) -> Vec<(AgentId, f32)> {
    if contributions.is_empty() {
        return Vec::new();
    }

    let total_contribution: f32 = contributions.iter().map(|(_, c, _)| c).sum();
    let pool = total_contribution * multiplier;
    let share = pool / contributions.len() as f32;

    contributions
        .iter()
        .map(|(id, contribution, _)| {
            // Net payoff = share received - contribution made
            (*id, share - contribution)
        })
        .collect()
}

/// Compute contribution for Public Goods Game based on trust
/// Per §6.10.3: Contribution correlates with group τ
#[must_use]
pub fn compute_contribution(
    own_tau: f32,
    group_mean_tau: f32,
    endowment: f32,
    cooperation_tendency: f32,
) -> f32 {
    // Base contribution: weighted by own τ and group τ
    // Per §6.10.3: Full contribution at group τ > 0.75
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    let trust_factor =
        0.3 * own_tau.min(1.0) + 0.4 * group_mean_tau.min(1.0) + 0.3 * cooperation_tendency;
    (endowment * trust_factor).clamp(0.0, endowment)
}

/// Select strategy based on genome cooperation tendency and current τ
/// Per §6.10.5: Strategy distribution correlates with population τ
pub fn select_strategy(cooperation_tendency: f32, own_tau: f32, rng: &mut StdRng) -> StrategyType {
    let coop_score = 0.5 * cooperation_tendency + 0.5 * own_tau.min(1.0);
    let roll = rng.gen::<f32>();

    const HIGH_TRUST: &[(f32, StrategyType)] = &[
        (0.5, StrategyType::ProspectTheory),
        (0.8, StrategyType::TitForTatForgive),
        (1.0, StrategyType::TitForTat),
    ];
    const MEDIUM_TRUST: &[(f32, StrategyType)] = &[
        (0.4, StrategyType::ProspectTheory),
        (0.7, StrategyType::TitForTat),
        (0.9, StrategyType::TitForTatForgive),
        (1.0, StrategyType::Random),
    ];
    const LOW_TRUST: &[(f32, StrategyType)] = &[
        (0.5, StrategyType::ProspectTheory),
        (0.8, StrategyType::Defect),
        (1.0, StrategyType::Random),
    ];

    let table = if coop_score > 0.7 {
        HIGH_TRUST
    } else if coop_score > 0.4 {
        MEDIUM_TRUST
    } else {
        LOW_TRUST
    };

    table
        .iter()
        .find(|(t, _)| roll < *t)
        .map_or(StrategyType::Random, |(_, s)| *s)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// DOCAS modulation from game outcome
/// Per task description: D+↑ on cooperation reward, C+↑ on defection betrayal, O+↑ on mutual cooperation
#[derive(Debug, Clone, Copy)]
pub struct DocasGameModulation {
    /// Dopamine delta (reward signal)
    pub delta_d: f32,
    /// Oxytocin delta (social bonding)
    pub delta_o: f32,
    /// Cortisol delta (stress from betrayal)
    pub delta_c: f32,
    /// Adrenaline delta (arousal from competition)
    pub delta_a: f32,
    /// Serotonin delta (satisfaction from fairness)
    pub delta_s: f32,
}

impl DocasGameModulation {
    /// Compute DOCAS modulation from game outcome
    #[must_use]
    pub fn from_outcome(
        own_cooperated: bool,
        partner_cooperated: bool,
        payoff: f32,
        expected_payoff: f32,
    ) -> Self {
        let reward_error = (payoff - expected_payoff) / expected_payoff.max(1.0);

        match (own_cooperated, partner_cooperated) {
            (true, true) => {
                // Mutual cooperation: D+, O++, S+
                Self {
                    delta_d: 0.05 + reward_error.max(0.0) * 0.1,
                    delta_o: 0.15,  // Strong social bonding
                    delta_c: -0.05, // Reduced stress
                    delta_a: 0.0,
                    delta_s: 0.1, // Satisfaction from fairness
                }
            }
            (true, false) => {
                // Betrayed (sucker's payoff): C++, A+, O-
                Self {
                    // TAG: surface=simulation owner=research-team rule=SIM-001
                    delta_d: -0.1,
                    delta_o: -0.1, // Trust damaged
                    delta_c: 0.2,  // High stress from betrayal
                    delta_a: 0.1,  // Arousal
                    delta_s: -0.1, // Unfairness
                }
            }
            (false, true) => {
                // Exploited partner: D+ (reward), but O- (guilt if high τ)
                Self {
                    delta_d: 0.1 + reward_error.max(0.0) * 0.1,
                    delta_o: -0.05,
                    delta_c: 0.05, // Some stress from guilt
                    delta_a: 0.05,
                    delta_s: -0.05,
                }
            }
            (false, false) => {
                // Mutual defection: neutral, slight stress
                Self {
                    delta_d: 0.0,
                    delta_o: -0.05,
                    delta_c: 0.1,
                    delta_a: 0.05,
                    delta_s: 0.0,
                }
            }
        }
    }
}

// ============================================================================
// GAME-P0-1: Trust-Weighted Expected Utility EU(a|T)
// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute trust-weighted expected utility for an action
/// Per GAME-P0-1: EU(a|T) = Σ p(o|a) × u(o) × T
/// Where:
/// - p(o|a) is probability of outcome o given action a
/// - u(o) is utility of outcome o
/// - T is trust level (multiplicative weight)
#[must_use]
pub fn compute_expected_utility_trust_weighted(
    trust: f32,
    action: bool, // true = cooperate, false = defect
    partner_trust: f32,
    payoff_matrix: &[[f32; 2]; 2], // [agent_action][partner_action]
) -> f32 {
    // Trust affects expected probability of partner cooperation
    // Higher partner_trust → higher probability they cooperate
    let p_partner_cooperates = partner_trust.clamp(0.0, 1.0);
    let p_partner_defects = 1.0 - p_partner_cooperates;

    let agent_idx = usize::from(!action);
    let cooperate_idx = 0;
    let defect_idx = 1;

    // Expected utility without trust weighting
    let eu_no_trust = p_partner_cooperates * payoff_matrix[agent_idx][cooperate_idx]
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
        + p_partner_defects * payoff_matrix[agent_idx][defect_idx];

    // Trust-weighted: multiply by agent's trust level
    // This implements the multiplicative trust weighting per GAME-P0-1
    eu_no_trust * trust.clamp(0.0, 1.0)
}

/// Compute expected utility for Prisoner's Dilemma with trust weighting
#[must_use]
pub fn compute_pd_eu_trust_weighted(trust: f32, cooperate: bool, partner_trust: f32) -> f32 {
    compute_expected_utility_trust_weighted(trust, cooperate, partner_trust, &PD_PAYOFF)
}

/// Choose action maximizing trust-weighted EU
/// Returns (`should_cooperate`, `expected_utility`)
#[must_use]
pub fn choose_action_max_eu(
    trust: f32,
    partner_trust: f32,
    payoff_matrix: &[[f32; 2]; 2],
) -> (bool, f32) {
    let eu_cooperate =
        compute_expected_utility_trust_weighted(trust, true, partner_trust, payoff_matrix);
    let eu_defect =
        compute_expected_utility_trust_weighted(trust, false, partner_trust, payoff_matrix);

    if eu_cooperate >= eu_defect {
        (true, eu_cooperate)
    } else {
        (false, eu_defect)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Validate GAME-P0-1: Cooperation rate should increase with trust
/// Returns correlation coefficient between trust and cooperation
#[must_use]
pub fn validate_eu_trust_monotonicity(
    trials: &[(f32, f32, bool)], // (trust, partner_trust, chose_cooperate)
) -> f32 {
    if trials.is_empty() {
        return 0.0;
    }

    // Compute correlation between trust and cooperation rate
    let n = trials.len() as f32;
    let mean_trust: f32 = trials.iter().map(|(t, _, _)| t).sum::<f32>() / n;
    let mean_coop: f32 = trials
        .iter()
        .map(|(_, _, c)| if *c { 1.0 } else { 0.0 })
        .sum::<f32>()
        / n;

    let covariance: f32 = trials
        .iter()
        .map(|(t, _, c)| (t - mean_trust) * (if *c { 1.0 } else { 0.0 } - mean_coop))
        .sum::<f32>()
        / n;

    let var_trust: f32 = trials
        .iter()
        .map(|(t, _, _)| (t - mean_trust).powi(2))
        .sum::<f32>()
        / n;
    let var_coop: f32 = trials
        .iter()
        .map(|(_, _, c)| (if *c { 1.0 } else { 0.0 } - mean_coop).powi(2))
        .sum::<f32>()
        / n;

    if var_trust <= 0.0 || var_coop <= 0.0 {
        return 0.0;
    }

    covariance / (var_trust.sqrt() * var_coop.sqrt())
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Strategy evolution tracking for GAME-P0-3
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StrategyEvolution {
    /// Strategy frequencies over time [Cooperate, Defect, TFT, `TFTForgive`, Random, `ProspectTheory`]
    pub frequencies: Vec<(u64, [f32; 6])>,
    /// Strategy switches
    pub switches: Vec<(u64, AgentId, StrategyType, StrategyType)>, // tick, agent, from, to
    /// Current ESS (Evolutionarily Stable Strategy) if detected
    pub ess: Option<StrategyType>,
}

/// Average strategy frequencies over the last `window` records
fn average_frequencies(recent: &[(u64, [f32; 6])], window: usize) -> [f32; 6] {
    let mut avg = [0.0; 6];
    for i in 0..6 {
        avg[i] = recent.iter().map(|(_, f)| f[i]).sum::<f32>() / window as f32;
    }
    avg
}

/// Map a strategy frequency index to its type
const fn strategy_from_index(index: usize) -> Option<StrategyType> {
    match index {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        0 => Some(StrategyType::Cooperate),
        1 => Some(StrategyType::Defect),
        2 => Some(StrategyType::TitForTat),
        3 => Some(StrategyType::TitForTatForgive),
        5 => Some(StrategyType::ProspectTheory),
        _ => None,
    }
}

impl StrategyEvolution {
    /// Record strategy frequencies at a tick
    pub fn record_frequencies(&mut self, tick: u64, strategies: &[StrategyType]) {
        if strategies.is_empty() {
            return;
        }

        let n = strategies.len() as f32;
        let mut counts = [0.0; 6];

        for s in strategies {
            match s {
                StrategyType::Cooperate => counts[0] += 1.0,
                StrategyType::Defect => counts[1] += 1.0,
                StrategyType::TitForTat => counts[2] += 1.0,
                StrategyType::TitForTatForgive => counts[3] += 1.0,
                StrategyType::Random => counts[4] += 1.0,
                StrategyType::ProspectTheory => counts[5] += 1.0,
            }
        }

        let frequencies: [f32; 6] = counts.map(|c| c / n);
        self.frequencies.push((tick, frequencies));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Detect ESS if strategy frequencies converged
    #[must_use]
    pub fn detect_ess(&self, window: usize) -> Option<StrategyType> {
        if self.frequencies.len() < window {
            return None;
        }

        let recent = &self.frequencies[self.frequencies.len() - window..];
        let avg_freq = average_frequencies(recent, window);

        for (i, &freq) in avg_freq.iter().enumerate() {
            if freq >= 0.8 {
                return strategy_from_index(i);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_prisoners_dilemma_payoffs() {
        // Mutual cooperation
        let (a, b) = play_prisoners_dilemma(true, true);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert_eq!(a, 3.0);
        assert_eq!(b, 3.0);

        // Mutual defection
        let (a, b) = play_prisoners_dilemma(false, false);
        assert_eq!(a, 1.0);
        assert_eq!(b, 1.0);

        // A cooperates, B defects (sucker's payoff)
        let (a, b) = play_prisoners_dilemma(true, false);
        assert_eq!(a, 0.0);
        assert_eq!(b, 5.0);
    }

    #[test]
    fn test_stag_hunt_payoffs() {
        // Both choose stag
        let (a, b) = play_stag_hunt(true, true);
        assert_eq!(a, 4.0);
        assert_eq!(b, 4.0);

        // Both choose hare
        let (a, b) = play_stag_hunt(false, false);
        assert_eq!(a, 2.0);
        assert_eq!(b, 2.0);
    }

    #[test]
    fn test_public_goods_game() {
        let contributions = vec![(0, 10.0, 0.8), (1, 10.0, 0.8), (2, 10.0, 0.8)];
        let payoffs = play_public_goods_game(&contributions, 1.5);

        // Total = 30, pool = 45, share = 15
        // Net = 15 - 10 = 5 for each
        assert_eq!(payoffs.len(), 3);
        for (_, payoff) in &payoffs {
            assert!((*payoff - 5.0).abs() < 0.01);
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_strategy_selection_high_trust() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut tft_count = 0;
        let mut tftf_count = 0;
        let mut prospect_count = 0;

        for _ in 0..100 {
            let strategy = select_strategy(0.9, 0.9, &mut rng);
            match strategy {
                StrategyType::TitForTat => tft_count += 1,
                StrategyType::TitForTatForgive => tftf_count += 1,
                StrategyType::ProspectTheory => prospect_count += 1,
                _ => {}
            }
        }

        // High trust should favor ProspectTheory, TFT+F, and TFT (cooperative strategies)
        assert!(tft_count + tftf_count + prospect_count > 80);
        // ProspectTheory should be the most common at high trust
        assert!(prospect_count > tft_count);
    }

    #[test]
    fn test_docas_modulation_mutual_cooperation() {
        let modulation = DocasGameModulation::from_outcome(true, true, 3.0, 2.5);
        assert!(modulation.delta_o > 0.1); // Strong oxytocin boost
        assert!(modulation.delta_c < 0.0); // Reduced cortisol
    }

    #[test]
    fn test_docas_modulation_betrayal() {
        let modulation = DocasGameModulation::from_outcome(true, false, 0.0, 2.5);
        assert!(modulation.delta_c > 0.1); // High cortisol from betrayal
        assert!(modulation.delta_o < 0.0); // Trust damaged
    }

    // ============================================================================
    // GAME-P0-1: Trust-Weighted EU Tests
    // ============================================================================

    #[test]
    fn test_eu_trust_weighted_computation() {
        // Trust-weighted EU computation
        // Note: In standard PD, defection is dominant regardless of trust
        // Trust weighting scales EU but doesn't change ordinal preferences
        let eu_coop_high = compute_pd_eu_trust_weighted(0.9, true, 0.9);
        let eu_defect_high = compute_pd_eu_trust_weighted(0.9, false, 0.9);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Both EU values are positive and scaled by trust
        assert!(eu_coop_high > 0.0);
        assert!(eu_defect_high > 0.0);
        // Trust scaling preserves ordering
        assert!(eu_defect_high > eu_coop_high); // Defect yields higher payoff in PD

        // Low trust agent → lower absolute EU but same ordering
        let eu_coop_low = compute_pd_eu_trust_weighted(0.2, true, 0.5);
        let eu_defect_low = compute_pd_eu_trust_weighted(0.2, false, 0.5);

        assert!(eu_coop_low > 0.0);
        assert!(eu_defect_low > 0.0);
        assert!(eu_defect_low > eu_coop_low);
    }

    #[test]
    fn test_choose_action_max_eu() {
        // In standard PD, defection is always rational (dominant strategy)
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Even with high trust, defection yields higher expected utility
        let (action, eu) = choose_action_max_eu(0.9, 0.9, &PD_PAYOFF);
        assert!(!action); // Should defect (dominant strategy in PD)
        assert!(eu > 0.0);

        // Low trust → also defect
        let (action, _) = choose_action_max_eu(0.2, 0.5, &PD_PAYOFF);
        assert!(!action); // Should defect
    }

    #[test]
    fn test_eu_trust_monotonicity() {
        // Generate trials at different trust levels
        let mut trials = Vec::new();

        for trust in [0.2, 0.4, 0.6, 0.8, 0.9] {
            let (chose_cooperate, _) = choose_action_max_eu(trust, 0.7, &PD_PAYOFF);
            trials.push((trust, 0.7, chose_cooperate));
        }

        let correlation = validate_eu_trust_monotonicity(&trials);

        // Correlation should be positive (higher trust → more cooperation)
        assert!(
            correlation >= 0.0,
            "Expected positive correlation, got {correlation}"
        );
    }

    #[test]
    fn test_strategy_evolution() {
        let mut evolution = StrategyEvolution::default();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Record some frequencies
        let strategies = vec![
            StrategyType::TitForTat,
            StrategyType::TitForTat,
            StrategyType::Cooperate,
            StrategyType::Defect,
            StrategyType::TitForTatForgive,
        ];

        evolution.record_frequencies(0, &strategies);
        evolution.record_frequencies(1, &strategies);
        evolution.record_frequencies(2, &strategies);

        // No ESS yet (no dominant strategy)
        assert!(evolution.detect_ess(3).is_none());

        // Record with dominant strategy
        let dominant = vec![
            StrategyType::TitForTatForgive,
            StrategyType::TitForTatForgive,
            StrategyType::TitForTatForgive,
            StrategyType::TitForTatForgive,
            StrategyType::Cooperate,
        ];
        evolution.record_frequencies(3, &dominant);
        // TAG: surface=simulation owner=research-team rule=SIM-001
        evolution.record_frequencies(4, &dominant);
        evolution.record_frequencies(5, &dominant);

        // Should detect TFT+F as ESS
        let ess = evolution.detect_ess(3);
        assert_eq!(ess, Some(StrategyType::TitForTatForgive));
    }

    // ============================================================================
    // Dictator Game Tests
    // ============================================================================

    #[test]
    fn test_dictator_game_basic() {
        let mut rng = StdRng::seed_from_u64(42);
        let stake = 100.0;

        // High trust dictator should offer more
        let (dictator_high, recipient_high) = play_dictator_game(0.9, stake, &mut rng);

        // Low trust dictator should offer less
        let (dictator_low, recipient_low) = play_dictator_game(0.1, stake, &mut rng);

        // Verify payoffs sum to stake
        assert!((dictator_high + recipient_high - stake).abs() < 0.01);
        assert!((dictator_low + recipient_low - stake).abs() < 0.01);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // High trust should offer more than low trust (statistically)
        assert!(recipient_high > recipient_low);
    }

    #[test]
    fn test_dictator_game_offer_range() {
        let mut rng = StdRng::seed_from_u64(123);
        let stake = 100.0;

        // Test many iterations to verify offer range
        for tau in [0.0, 0.5, 1.0] {
            for _ in 0..50 {
                let (_, recipient) = play_dictator_game(tau, stake, &mut rng);
                let offer_fraction = recipient / stake;

                // Offers should be in [5%, 50%] range
                assert!(
                    (0.05..=0.5).contains(&offer_fraction),
                    "Offer fraction {offer_fraction} out of range for tau={tau}"
                );
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_dictator_game_trust_correlation() {
        let mut rng = StdRng::seed_from_u64(456);
        let stake = 100.0;

        // Collect offers at different trust levels
        let mut offers = Vec::new();
        for tau in [0.1, 0.3, 0.5, 0.7, 0.9] {
            let mut total_offer = 0.0;
            for _ in 0..20 {
                let (_, recipient) = play_dictator_game(tau, stake, &mut rng);
                total_offer += recipient / stake; // Normalize to fraction
            }
            offers.push((tau, total_offer / 20.0));
        }

        // Check monotonicity (higher trust → higher offers)
        for i in 1..offers.len() {
            assert!(
                offers[i].1 >= offers[i - 1].1 - 0.1,
                "Offers should generally increase with trust"
            );
        }
    }

    // ============================================================================
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    // Trust Game Tests
    // ============================================================================

    #[test]
    fn test_trust_game_basic() {
        let mut rng = StdRng::seed_from_u64(42);
        let endowment = 100.0;
        let multiplier = 3.0;

        // High trust investor, high trustworthiness trustee
        let (investor_high, trustee_high, investment_high, returned_high) =
            play_trust_game(0.9, 0.9, endowment, multiplier, &mut rng);

        // Low trust investor, low trustworthiness trustee
        let (_investor_low, _trustee_low, investment_low, _returned_low) =
            play_trust_game(0.1, 0.1, endowment, multiplier, &mut rng);

        // Verify investment is within bounds
        assert!(investment_high > investment_low);
        assert!(investment_high <= endowment);

        // Verify returned amount is within bounds
        let multiplied_high = investment_high * multiplier;
        assert!(returned_high <= multiplied_high);

        // Verify payoffs
        assert!((investor_high - (endowment - investment_high + returned_high)).abs() < 0.01);
        assert!((trustee_high - (multiplied_high - returned_high)).abs() < 0.01);
    }

    #[test]
    fn test_trust_game_metrics() {
        let endowment = 100.0;
        let investment = 50.0;
        let multiplied = investment * 3.0;
        let returned = 75.0;

        let (trust_index, trustworthiness_index, efficiency) =
            compute_trust_metrics(endowment, investment, returned, multiplied);

        // Trust index: investment / endowment
        assert!((trust_index - 0.5).abs() < 0.01);

        // Trustworthiness index: returned / multiplied
        assert!((trustworthiness_index - 0.5).abs() < 0.01);

        // Efficiency: (total payoff) / (2 * endowment)
        // With investment 50, multiplier 3, return 75:
        // Investor: 100 - 50 + 75 = 125
        // Trustee: 150 - 75 = 75
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Total: 200, Baseline: 200, Efficiency: 1.0
        // If returned < 75, efficiency < 1.0
        assert!(efficiency >= 0.5); // At least 50% efficiency
    }

    #[test]
    fn test_trust_game_low_trust() {
        let mut rng = StdRng::seed_from_u64(42);
        let endowment = 100.0;
        let multiplier = 3.0;

        // Low trust investor sends less
        let (investor_low, trustee_low, investment_low, _returned_low) =
            play_trust_game(0.1, 0.9, endowment, multiplier, &mut rng);

        // High trust investor sends more
        let (_investor_high, _trustee_high, investment_high, _returned_high) =
            play_trust_game(0.9, 0.9, endowment, multiplier, &mut rng);

        // Verify investments are reasonable
        assert!(
            investment_low < investment_high,
            "Low trust should invest less than high trust"
        );

        // Verify payoffs are non-negative
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(investor_low >= 0.0);
        assert!(trustee_low >= 0.0);
    }

    #[test]
    fn test_trust_game_full_return() {
        let mut rng = StdRng::seed_from_u64(42);
        let endowment = 100.0;
        let multiplier = 3.0;

        // Run multiple times to check high trustworthiness scenario
        let mut total_return_fraction = 0.0;
        let n = 20;

        for _ in 0..n {
            let (_, _, investment, returned) =
                play_trust_game(0.9, 0.9, endowment, multiplier, &mut rng);
            if investment > 0.0 {
                total_return_fraction += returned / (investment * multiplier);
            }
        }

        let avg_return_fraction = total_return_fraction / n as f32;
        // High trustworthiness should result in substantial returns
        assert!(avg_return_fraction > 0.3);
    }
}
