// TAG: surface=simulation owner=research-team rule=SIM-001
//! Social encounters, proximity checks, similarity metrics, and isolation scoring.

use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::agent::Agent;
use crate::types::{AgentId, Tick, C};

/// Compute social isolation score from interaction patterns.
#[must_use]
pub fn compute_social_isolation(
    agent: &Agent,
    interaction_history: &[(AgentId, Tick)],
    current_tick: Tick,
) -> f32 {
    const ISOLATION_WINDOW: Tick = 200; // Ticks to look back
    const MAX_EXPECTED_NETWORK: f32 = 10.0; // Dunbar-inspired max unique partners
                                            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    const MAX_EXPECTED_FREQ: f32 = 30.0; // Expected interactions per window

    // Filter recent interactions within window
    let recent: Vec<_> = interaction_history
        .iter()
        .filter(|(_, tick)| current_tick.saturating_sub(*tick) <= ISOLATION_WINDOW)
        .collect();

    // Objective measures (NASEM-inspired: Duke Social Support Index)
    let network_size = recent
        .iter()
        .map(|(id, _)| id)
        .collect::<std::collections::HashSet<_>>()
        .len() as f32;
    let interaction_frequency = recent.len() as f32;
    let group_membership = if agent.group_id.is_some() { 1.0 } else { 0.0 };

    // Normalize and combine
    let size_score = 1.0 - (network_size / MAX_EXPECTED_NETWORK).min(1.0);
    let freq_score = 1.0 - (interaction_frequency / MAX_EXPECTED_FREQ).min(1.0);

    // Weighted average (Duke Social Support Index inspired weights)
    let raw_isolation = 0.4 * size_score + 0.4 * freq_score + 0.2 * (1.0 - group_membership);

    // Genome modulation: isolation_resilience attenuates effect
    let resilience = agent.genome.isolation_resilience();
    raw_isolation * (1.0 - 0.5 * resilience) // Max 50% attenuation
}

// ============================================================================
// ENCOUNTER PIPELINE (SIM-003)
// Per §3.1: 4-step encounter decision tree
// Per SIM-003: Two-agent engagement with sex-based branching
// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Encounter decision outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncounterDecision {
    /// Agents decide to engage in interaction
    Engage,
    /// Agents decide to avoid each other
    Avoid,
}

/// Interaction mode after engagement decision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionMode {
    /// Economic game only (same sex or insufficient resources)
    EconomicGame,
    /// Reproduction possible (opposite sex + conditions met)
    Reproduction,
    /// Communication/semantic alignment check
    Communication,
}

/// Encounter context for decision making
#[derive(Debug, Clone, Copy)]
pub struct EncounterContext {
    /// Distance between agents
    pub distance: f32,
    /// Agent A's trust level
    pub tau_a: f32,
    /// Agent B's trust level
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub tau_b: f32,
    /// Mean reputation in the area
    pub local_reputation: f32,
    /// Resource scarcity indicator [0, 1]
    pub resource_scarcity: f32,
    /// Current tick
    pub current_tick: Tick,
}

/// Similarity metrics between two agents
///
/// DEPRECATED: Use `crate::encounter::SimilarityMetrics` instead.  // CLASS-001: Migration documentation
/// This version is maintained for backward compatibility in society module tests.
/// The encounter.rs version is the canonical implementation used in simulation.
#[derive(Debug, Clone, Copy)]
pub struct SimilarityMetrics {
    /// Genome similarity [0, 1]
    pub genome: f32,
    /// Reputation similarity [0, 1]
    pub reputation: f32,
    /// Resource asymmetry [0, 1], 0 = equal, 1 = max difference
    pub resource_asymmetry: f32,
    /// Sex compatibility (true if opposite sex)
    pub sex_compatible: bool,
    /// Combined similarity score [0, 1]
    pub combined: f32,
}

/// Step 1: Check if agents are within perception radius
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
///
/// DEPRECATED: Use `crate::encounter::detect_proximity` instead.  // CLASS-001: Migration documentation
/// This function is maintained for backward compatibility in society module tests.
#[must_use]
pub fn check_proximity(agent_a: &Agent, agent_b: &Agent, perception_radius: f32) -> bool {
    agent_a.position.distance(&agent_b.position) <= perception_radius
}

// CODE-001: Deprecated with migration path
// Migration: Use crate::encounter::compute_similarity_metrics instead.
// This function is maintained for backward compatibility in society module tests.
// The encounter.rs version is the canonical implementation used in simulation.
#[deprecated(
    since = "0.1.0",
    note = "Use crate::encounter::compute_similarity_metrics instead"
)]
#[must_use]
/// compute similarity metrics.
pub fn compute_similarity_metrics(agent_a: &Agent, agent_b: &Agent) -> SimilarityMetrics {
    // Genome similarity (using cooperation tendency as proxy)
    let coop_a = agent_a.genome.cooperation_tendency();
    let coop_b = agent_b.genome.cooperation_tendency();
    let genome_sim = 1.0 - (coop_a - coop_b).abs();

    // Reputation similarity
    let rep_a = agent_a.reputation_score;
    let rep_b = agent_b.reputation_score;
    let reputation_sim = 1.0 - (rep_a - rep_b).abs();
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    // Resource asymmetry (normalized)
    let max_energy = 100.0;
    let resource_diff = (agent_a.energy - agent_b.energy).abs();
    let resource_asymmetry = (resource_diff / max_energy).min(1.0);

    // Sex compatibility (opposite sex can reproduce)
    let sex_compatible = agent_a.sex.can_reproduce_with(&agent_b.sex);

    // Combined similarity: weighted average
    // High genome similarity + high reputation similarity = more likely to engage
    // But inverted-U for genome: too similar or too dissimilar both reduce engagement
    let genome_engagement = if genome_sim > 0.8 {
        // Too similar - slight penalty (avoid inbreeding)
        genome_sim * 0.8
    } else {
        genome_sim
    };

    let combined =
        0.4 * genome_engagement + 0.3 * reputation_sim + 0.3 * (1.0 - resource_asymmetry); // Lower asymmetry = higher similarity

    SimilarityMetrics {
        genome: genome_sim,
        reputation: reputation_sim,
        resource_asymmetry,
        sex_compatible,
        combined,
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Step 3: Decide whether to engage or avoid
/// Per SIM-003: Function of DOCAS, criticality, reputation, scarcity, recognition gap
pub fn decide_encounter(
    agent_a: &Agent,
    agent_b: &Agent,
    metrics: &SimilarityMetrics,
    context: &EncounterContext,
    rng: &mut StdRng,
) -> EncounterDecision {
    // Base engagement probability from similarity
    let mut engage_prob = metrics.combined;

    // Modulate by agent trust levels
    // High τ agents more willing to engage
    let mean_tau = (context.tau_a + context.tau_b) / 2.0;
    engage_prob *= 0.7 + 0.3 * (mean_tau / 2.0).min(1.0);

    // Resource scarcity effect
    // High scarcity → more cautious about engaging (avoid competition)
    engage_prob *= 1.0 - 0.2 * context.resource_scarcity;

    // Recognition gap (Cortisol) effect
    // High C in either agent → more likely to avoid (threat detection)
    let max_cortisol = agent_a.docas.levels[C].max(agent_b.docas.levels[C]);
    engage_prob *= 1.0 - 0.3 * max_cortisol;

    // Reputation threshold
    // If either agent has very low reputation, likely to avoid
    let min_reputation = agent_a.reputation_score.min(agent_b.reputation_score);
    if min_reputation < -0.3 {
        engage_prob *= 0.5;
    }

    // Clamp to [0, 1]
    engage_prob = engage_prob.clamp(0.0, 1.0);

    if rng.gen::<f32>() < engage_prob {
        EncounterDecision::Engage
    } else {
        EncounterDecision::Avoid
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

/// Step 4: Determine interaction mode based on sex and conditions
/// Per SIM-003: Same sex → economic game; Opposite sex → reproduction possible
#[must_use]
pub fn determine_interaction_mode(
    agent_a: &Agent,
    agent_b: &Agent,
    min_reproduction_energy: f32,
    min_reproduction_reputation: f32,
) -> InteractionMode {
    // Check sex compatibility
    if !agent_a.can_reproduce_with(agent_b) {
        // Same sex - economic game only
        return InteractionMode::EconomicGame;
    }

    // Opposite sex - check reproduction conditions
    let energy_sufficient =
        agent_a.energy >= min_reproduction_energy && agent_b.energy >= min_reproduction_energy;

    let reputation_sufficient = agent_a.reputation_score >= min_reproduction_reputation
        && agent_b.reputation_score >= min_reproduction_reputation;

    // Check criticality confidence (simplified: use current sigma)
    let criticality_sufficient = agent_a.criticality_sigma >= 0.9
        && agent_a.criticality_sigma <= 1.1
        && agent_b.criticality_sigma >= 0.9
        && agent_b.criticality_sigma <= 1.1;

    if energy_sufficient && reputation_sufficient && criticality_sufficient {
        InteractionMode::Reproduction
    } else {
        // Conditions not met - fall back to economic game
        InteractionMode::EconomicGame
    }
}

// CODE-001: Deprecated with migration path
// Migration: Use crate::encounter::process_encounter instead.
// This function is maintained for backward compatibility in society module tests.
// The encounter.rs version is the canonical implementation used in simulation.
#[deprecated(
    since = "0.1.0",
    note = "Use crate::encounter::process_encounter instead"
)]
// TAG: surface=simulation owner=research-team rule=SIM-001
#[allow(clippy::too_many_arguments)]
/// process encounter.
pub fn process_encounter(
    agent_a: &Agent,
    agent_b: &Agent,
    perception_radius: f32,
    resource_scarcity: f32,
    min_reproduction_energy: f32,
    min_reproduction_reputation: f32,
    current_tick: Tick,
    rng: &mut StdRng,
) -> Option<(EncounterDecision, InteractionMode)> {
    // Step 1: Proximity check
    if !check_proximity(agent_a, agent_b, perception_radius) {
        return None;
    }

    // Step 2: Similarity computation
    // Use the canonical implementation from encounter.rs
    #[allow(deprecated)]
    let metrics = compute_similarity_metrics(agent_a, agent_b);

    // Step 3: Engage/avoid decision
    let context = EncounterContext {
        distance: agent_a.position.distance(&agent_b.position),
        tau_a: agent_a.compute_trust(),
        tau_b: agent_b.compute_trust(),
        local_reputation: (agent_a.reputation_score + agent_b.reputation_score) / 2.0,
        resource_scarcity,
        current_tick,
    };

    let decision = decide_encounter(agent_a, agent_b, &metrics, &context, rng);

    // Step 4: Determine interaction mode (only if engaging)
    let mode = if decision == EncounterDecision::Engage {
        determine_interaction_mode(
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            agent_a,
            agent_b,
            min_reproduction_energy,
            min_reproduction_reputation,
        )
    } else {
        InteractionMode::EconomicGame // Default, won't be used if avoiding
    };

    Some((decision, mode))
}

// Integration tests extracted from crates/sim-core/src/society.rs
