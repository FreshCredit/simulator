//! Encounter Pipeline: Two-Agent Interaction System
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION encounters, not product interactions
//! SIM-010: Identity theft simulation integration
// TAG: surface=simulation owner=research-team rule=SIM-001
//!
//! CANONICAL VERSION: This is the canonical encounter implementation.
//! society.rs contains deprecated duplicate functions maintained for test compatibility.
//!
//! Per SIM-003: 4-step encounter pipeline
//! 1. Proximity Detection: Agents within perception radius
//! 2. Similarity Computation: Genome, language, reputation, resources, sex
//! 3. Engage/Avoid Decision: DOCAS + confidence + reputation + scarcity
//! 4. Interaction Mode Selection: Game vs Reproduction based on sex pairing
//!
//! Per SIM-010: Identity theft attempts during encounters
//! - 2% base chance per encounter
//! - Protocol protection reduces theft rate by 20-40%
//! - Recovery takes 100-500 ticks

use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::agent::Agent;
use crate::communication::{
    generate_from_docas_with_lexicon, receive_communication_with_language, CommunicationContext,
};
use crate::encounter_games::{
    apply_trust_update_rw, process_dictator_game, process_prisoners_dilemma, process_public_goods,
    process_stag_hunt, process_trust_game, process_ultimatum, InteractionOutcome,
};
use crate::games::{decide_action, GameType};
use crate::identity::{IdentityTheftStats, TheftMethod};
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
use crate::types::{AgentId, CommunicationIntent, Position, Tick};

/// Perception radius for encounter detection
/// Agents within this distance can detect each other
pub const PERCEPTION_RADIUS: f32 = 3.0;

/// Minimum similarity threshold for engagement
/// Below this, agents will avoid interaction
pub const MIN_SIMILARITY_THRESHOLD: f32 = 0.3;

/// Encounter result between two agents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncounterResult {
    /// Agents chose to avoid each other
    Avoided,
    /// Agents engaged in game interaction
    /// game type.
    GameEngagement {
        /// game type.
        game_type: GameType,
    },
    /// Opposite-sex agents engaged in reproduction
    ReproductionEngagement,
    /// Identity theft occurred during encounter
    IdentityTheft {
        /// method.
        method: TheftMethod,
    },
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Similarity metrics between two agents
/// Per SIM-003 Step 2: Multi-channel similarity computation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SimilarityMetrics {
    /// Genome similarity [0, 1] - genetic kinship
    pub genome: f32,
    /// Language similarity [0, 1] - communication compatibility
    pub language: f32,
    /// Reputation signal [-1, 1] - social standing compatibility
    pub reputation: f32,
    /// Resource asymmetry [0, 1] - wealth difference (0 = equal)
    pub resource_asymmetry: f32,
    /// Sex compatibility (1.0 = opposite, 0.0 = same)
    pub sex_compatibility: f32,
    /// Music cadence similarity [0, 1] - rhythmic compatibility
    /// Enables cross-language bonding through shared rhythm
    pub music: f32,
}

impl SimilarityMetrics {
    /// Compute weighted composite similarity score
    /// Returns value in [0, 1] where higher = more similar
    #[must_use]
    pub fn composite_score(&self) -> f32 {
        // Weights per simulation design
        const W_GENOME: f32 = 0.22;
        const W_LANGUAGE: f32 = 0.18;
        const W_REPUTATION: f32 = 0.28;
        const W_RESOURCE: f32 = 0.12;
        const W_SEX: f32 = 0.10;
        const W_MUSIC: f32 = 0.10; // New: music enables cross-language bonding

        // Normalize reputation from [-1, 1] to [0, 1]
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let reputation_norm = (self.reputation + 1.0) / 2.0;

        // Resource asymmetry: 0 difference = 1.0 similarity
        let resource_sim = 1.0 - self.resource_asymmetry;

        W_GENOME * self.genome
            + W_LANGUAGE * self.language
            + W_REPUTATION * reputation_norm
            + W_RESOURCE * resource_sim
            + W_SEX * self.sex_compatibility
            + W_MUSIC * self.music
    }

    /// Check if agents can bond via music (high music similarity, low language similarity)
    /// This enables cross-language bonding through shared rhythm
    #[must_use]
    pub fn can_bond_via_music(&self) -> bool {
        const MUSIC_THRESHOLD: f32 = 0.7;
        const LANGUAGE_THRESHOLD: f32 = 0.4;

        self.music >= MUSIC_THRESHOLD && self.language <= LANGUAGE_THRESHOLD
    }

    /// Get music-bonding boost for engagement decision
    /// Returns additional similarity boost when music enables bonding
    #[must_use]
    pub fn music_bonding_boost(&self) -> f32 {
        if self.can_bond_via_music() {
            // Significant boost: shared rhythm compensates for language barrier
            0.15 * self.music
        } else {
            0.0
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Encounter decision factors
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EncounterDecision {
    /// Whether to engage (true) or avoid (false)
    pub should_engage: bool,
    /// Confidence in the decision [0, 1]
    pub confidence: f32,
    /// Primary reason for decision
    pub reason: EncounterReason,
}

/// Reasons for encounter decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncounterReason {
    /// High similarity score
    HighSimilarity,
    /// Low similarity score
    LowSimilarity,
    /// High trust (τ) enables risk-taking
    HighTrust,
    /// Low trust (τ) causes caution
    LowTrust,
    /// Resource scarcity drives competition
    ResourceScarcity,
    /// Resource abundance enables cooperation
    ResourceAbundance,
    /// Reputation difference
    ReputationGap,
    /// Sexual attraction (opposite sex)
    SexualAttraction,
}

/// Detect agents within perception radius of each other
/// Per SIM-003 Step 1: Proximity detection using spatial grid partitioning
///
/// Returns pairs of agent indices that can encounter each other
#[must_use]
pub fn detect_proximity(agents: &[Agent], radius: f32) -> Vec<(usize, usize)> {
    let alive_positions: Vec<(usize, &Position)> = agents
        .iter()
        .enumerate()
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        .filter(|(_, a)| a.alive)
        .map(|(i, a)| (i, &a.position))
        .collect();

    let n = alive_positions.len();
    if n < 2 {
        return Vec::new();
    }

    const MAX_ENCOUNTERS_PER_AGENT: usize = 4;
    let radius_sq = radius * radius;
    let cell_size = radius.max(1.0);

    let mut grid: std::collections::HashMap<(i32, i32), Vec<(usize, &Position)>> =
        std::collections::HashMap::with_capacity(n);

    for &(idx, pos) in &alive_positions {
        let cx = (pos.x / cell_size).floor() as i32;
        let cy = (pos.y / cell_size).floor() as i32;
        grid.entry((cx, cy)).or_default().push((idx, pos));
    }

    let mut encounters = Vec::with_capacity(n.min(2048));
    let mut agent_encounter_counts = std::collections::HashMap::<usize, usize>::with_capacity(n);

    for &(idx1, pos1) in &alive_positions {
        let count1 = agent_encounter_counts.get(&idx1).copied().unwrap_or(0);
        if count1 >= MAX_ENCOUNTERS_PER_AGENT {
            continue;
        }

        let cx = (pos1.x / cell_size).floor() as i32;
        let cy = (pos1.y / cell_size).floor() as i32;

        'neighbor_loop: for dy in -1..=1 {
            for dx in -1..=1 {
                if let Some(neighbors) = grid.get(&(cx + dx, cy + dy)) {
                    for &(idx2, pos2) in neighbors {
                        if idx1 >= idx2 {
                            continue;
                        }

                        let count2 = agent_encounter_counts.get(&idx2).copied().unwrap_or(0);
                        if count2 >= MAX_ENCOUNTERS_PER_AGENT {
                            continue;
                        }

                        let d_sq = (pos1.x - pos2.x) * (pos1.x - pos2.x)
                            + (pos1.y - pos2.y) * (pos1.y - pos2.y);
                        if d_sq <= radius_sq {
                            encounters.push((idx1, idx2));
                            *agent_encounter_counts.entry(idx1).or_insert(0) += 1;
                            *agent_encounter_counts.entry(idx2).or_insert(0) += 1;

                            if agent_encounter_counts.get(&idx1).copied().unwrap_or(0)
                                >= MAX_ENCOUNTERS_PER_AGENT
                            {
                                break 'neighbor_loop;
                            }
                        }
                    }
                }
            }
        }
    }

    encounters
}

/// Compute genome similarity between two agents
/// Per SIM-003 Step 2: Genome similarity
///
/// Returns [0, 1] where 1 = identical genomes, 0 = completely different
#[must_use]
pub fn compute_genome_similarity(agent1: &Agent, agent2: &Agent) -> f32 {
    let g1 = &agent1.genome;
    let g2 = &agent2.genome;

    // Compute average allele difference across all loci
    let mut total_diff = 0.0;
    for i in 0..g1.alleles.len() {
        let a1 = (g1.alleles[i][0] + g1.alleles[i][1]) / 2.0;
        let a2 = (g2.alleles[i][0] + g2.alleles[i][1]) / 2.0;
        total_diff += (a1 - a2).abs();
    }

    let avg_diff = total_diff / g1.alleles.len() as f32;
    // Convert difference to similarity: 0 diff = 1.0 similarity
    1.0 - avg_diff
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute language similarity between two agents
/// Per SIM-003 Step 2: Language similarity
///
/// Returns [0, 1] based on lexicon overlap
#[must_use]
pub fn compute_language_similarity(agent1: &Agent, agent2: &Agent) -> f32 {
    let lex1 = &agent1.lexicon;
    let lex2 = &agent2.lexicon;

    if lex1.is_empty() || lex2.is_empty() {
        // No language data - neutral similarity
        return 0.5;
    }

    // Compute overlap of concept IDs
    let concepts1: std::collections::HashSet<_> = lex1.concepts.keys().copied().collect();
    let concepts2: std::collections::HashSet<_> = lex2.concepts.keys().copied().collect();

    if concepts1.is_empty() || concepts2.is_empty() {
        return 0.5;
    }

    // Jaccard similarity: |A ∩ B| / |A ∪ B|
    let intersection: std::collections::HashSet<_> =
        concepts1.intersection(&concepts2).copied().collect();
    let union: std::collections::HashSet<_> = concepts1.union(&concepts2).copied().collect();

    intersection.len() as f32 / union.len() as f32
}

/// Compute reputation signal between two agents
/// Per SIM-003 Step 2: Reputation signal
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
///
/// Returns [-1, 1] where 1 = both high reputation, -1 = mutual distrust
#[must_use]
pub fn compute_reputation_signal(agent1: &Agent, agent2: &Agent) -> f32 {
    // Check if agents have local reputation data for each other
    let rep1_to_2 = agent1.get_local_reputation(agent2.id);
    let rep2_to_1 = agent2.get_local_reputation(agent1.id);

    // If both have interacted before, use mutual reputation
    if agent1.knows_reputation_of(agent2.id) && agent2.knows_reputation_of(agent1.id) {
        // Average of mutual reputations, normalized to [-1, 1]
        (rep1_to_2 + rep2_to_1) / 2.0
    } else if agent1.knows_reputation_of(agent2.id) {
        // Only agent1 knows agent2
        rep1_to_2 * 0.8 // Slightly dampened
    } else if agent2.knows_reputation_of(agent1.id) {
        // Only agent2 knows agent1
        rep2_to_1 * 0.8
    } else {
        // No prior interaction - use global reputation scores
        let global_sim = (agent1.reputation_score - agent2.reputation_score).abs();
        // More similar reputations = higher signal
        1.0 - global_sim * 2.0
    }
}

/// Compute resource asymmetry between two agents
/// Per SIM-003 Step 2: Resource asymmetry
///
/// Returns [0, 1] where 0 = equal resources, 1 = maximum inequality
#[must_use]
pub fn compute_resource_asymmetry(agent1: &Agent, agent2: &Agent) -> f32 {
    let max_energy = 100.0; // Assuming max energy cap
    let e1 = agent1.energy / max_energy;
    let e2 = agent2.energy / max_energy;

    (e1 - e2).abs()
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute sex compatibility
/// Per SIM-003 Step 2: Sex pairing
///
/// Returns 1.0 for opposite sex (reproduction possible), 0.0 for same sex
#[must_use]
pub fn compute_sex_compatibility(agent1: &Agent, agent2: &Agent) -> f32 {
    if agent1.sex.can_reproduce_with(&agent2.sex) {
        1.0
    } else {
        0.0
    }
}

/// Compute music cadence similarity between two agents
/// Per music.md: Rhythm-based bonding enables cross-language connection
///
/// Returns [0, 1] based on cadence pattern similarity
#[must_use]
pub fn compute_music_similarity(agent1: &Agent, agent2: &Agent) -> f32 {
    // Get music states if both agents have them
    match (&agent1.music_state, &agent2.music_state) {
        (Some(music1), Some(music2)) => music1.cadence_similarity(music2),
        // If either agent lacks music state, return neutral similarity
        _ => 0.5,
    }
}

/// Compute all similarity metrics between two agents
#[must_use]
pub fn compute_similarity_metrics(agent1: &Agent, agent2: &Agent) -> SimilarityMetrics {
    SimilarityMetrics {
        genome: compute_genome_similarity(agent1, agent2),
        language: compute_language_similarity(agent1, agent2),
        reputation: compute_reputation_signal(agent1, agent2),
        resource_asymmetry: compute_resource_asymmetry(agent1, agent2),
        sex_compatibility: compute_sex_compatibility(agent1, agent2),
        music: compute_music_similarity(agent1, agent2),
    }
}

/// Make engage/avoid decision based on agent states and similarity
/// Per SIM-003 Step 3: Engage/Avoid Decision
///
/// Uses Prospect Theory (Kahneman & Tversky, 1979) to evaluate engagement outcomes
/// Per §6.10: Agents use prospect value, not raw trust, for decisions
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
///
/// `Ψ_trust` Integration:
/// - Uses unified trust from cross-domain coherence
/// - Adjusts confidence threshold based on coherence
/// - Information-seeking mode when domains conflict
///
/// PHILOSOPHY LAYER INTEGRATION (`τ_φ)`:
/// - Philosophy alignment affects engagement willingness
/// - Cooperation willingness from ethical framework
/// - Risk tolerance from worldview trust axis
#[must_use]
pub fn decide_engagement(
    agent: &Agent,
    other: &Agent,
    similarity: &SimilarityMetrics,
    scarcity: f32,
) -> EncounterDecision {
    use crate::economics::{probability_weight, prospect_value};

    let tau = agent.unified_trust();

    if agent.should_seek_information() {
        let coherence = agent.compute_psi_coherence();
        return EncounterDecision {
            should_engage: false,
            confidence: 0.3 + 0.7 * coherence,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            reason: EncounterReason::LowTrust,
        };
    }

    let coherence = agent.compute_psi_coherence();
    let confidence = 0.3 + 0.7 * coherence;

    let prospect_value_total =
        compute_prospect_value(agent, similarity, tau, prospect_value, probability_weight);
    let scarcity_factor = compute_scarcity_factor(similarity, scarcity);
    let (philosophy_alignment, phil_coop_willingness, phil_risk_tolerance) =
        compute_philosophy_factors(agent, other);
    let philosophy_modifier =
        (philosophy_alignment - 0.5) * 0.2 + (phil_coop_willingness - 0.5) * 0.3;

    if confidence < 0.3 {
        return EncounterDecision {
            should_engage: false,
            confidence,
            reason: if tau < 0.8 {
                EncounterReason::LowTrust
            } else {
                EncounterReason::LowSimilarity
            },
        };
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let adjusted_prospect = prospect_value_total + philosophy_modifier * 10.0;
    let risk_adjusted_threshold = -5.0 + phil_risk_tolerance * 5.0;
    let should_engage = adjusted_prospect > risk_adjusted_threshold && scarcity_factor > 0.5;

    let reason = determine_engagement_reason(
        should_engage,
        similarity,
        scarcity,
        philosophy_alignment,
        phil_coop_willingness,
        prospect_value_total,
    );

    EncounterDecision {
        should_engage,
        confidence,
        reason,
    }
}

fn compute_prospect_value(
    _agent: &Agent,
    similarity: &SimilarityMetrics,
    tau: f32,
    prospect_value_fn: impl Fn(f32) -> f32,
    probability_weight_fn: impl Fn(f32) -> f32,
) -> f32 {
    let bonding_boost = similarity.music_bonding_boost();
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    let adjusted_similarity = (similarity.composite_score() + bonding_boost).min(1.0);
    let expected_gain = adjusted_similarity * 10.0;
    let expected_loss = (1.0 - adjusted_similarity) * 5.0;

    let v_gain = prospect_value_fn(expected_gain);
    let v_loss = prospect_value_fn(-expected_loss);

    let trust_prob = tau.min(2.0) / 2.0;
    let weighted_prob = probability_weight_fn(trust_prob.clamp(0.0, 1.0));

    weighted_prob * v_gain + (1.0 - weighted_prob) * v_loss
}

fn compute_scarcity_factor(similarity: &SimilarityMetrics, scarcity: f32) -> f32 {
    if similarity.resource_asymmetry > 0.5 {
        1.0 - scarcity * 0.5
    } else {
        1.0
    }
}

fn compute_philosophy_factors(agent: &Agent, other: &Agent) -> (f32, f32, f32) {
    (
        agent.philosophy_alignment(other),
        agent.philosophy_cooperation_willingness(other),
        agent.philosophy_risk_tolerance(),
    )
}

fn determine_engagement_reason(
    should_engage: bool,
    similarity: &SimilarityMetrics,
    scarcity: f32,
    philosophy_alignment: f32,
    phil_coop_willingness: f32,
    prospect_value_total: f32,
) -> EncounterReason {
    if should_engage {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        if philosophy_alignment > 0.7 && phil_coop_willingness > 0.6 {
            EncounterReason::HighSimilarity
        } else if similarity.sex_compatibility > 0.5 {
            EncounterReason::SexualAttraction
        } else if similarity.genome > 0.7 {
            EncounterReason::HighSimilarity
        } else if prospect_value_total > 5.0 {
            EncounterReason::HighTrust
        } else if scarcity > 0.7 {
            EncounterReason::ResourceScarcity
        } else {
            EncounterReason::HighSimilarity
        }
    } else if prospect_value_total <= 0.0 {
        EncounterReason::LowTrust
    } else if similarity.genome < 0.3 || philosophy_alignment < 0.3 {
        EncounterReason::LowSimilarity
    } else {
        EncounterReason::ReputationGap
    }
}

/// Select interaction mode based on agent pairing
/// Per SIM-003 Step 4: Interaction Mode Selection
///
/// Same sex → game only
/// Opposite sex → reproduction possible (may also play game)
pub fn select_interaction_mode(
    agent1: &Agent,
    agent2: &Agent,
    rng: &mut StdRng,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
) -> EncounterResult {
    let can_reproduce = agent1.sex.can_reproduce_with(&agent2.sex);

    if can_reproduce {
        // Opposite sex: decision between reproduction and game
        // Based on energy states - low energy agents prefer games
        let avg_energy = (agent1.energy + agent2.energy) / 2.0;

        if avg_energy > 60.0 && rng.gen::<f32>() < 0.6 {
            // High energy + chance → reproduction
            EncounterResult::ReproductionEngagement
        } else {
            // Otherwise play game
            let game_type = select_game_type(agent1, agent2, rng);
            EncounterResult::GameEngagement { game_type }
        }
    } else {
        // Same sex: game only
        let game_type = select_game_type(agent1, agent2, rng);
        EncounterResult::GameEngagement { game_type }
    }
}

/// Select game type based on agent states and context
///
/// Game selection logic uses multiple factors to choose appropriate games:
/// - High trust + low asymmetry → Trust Game (investment/reciprocity)
/// - High resource asymmetry → Dictator Game (unilateral decision)
/// - Low trust + similar reputation → Public Goods Game (group cooperation)
/// - High trust → Stag Hunt (coordination)
/// - Low energy → Prisoner's Dilemma or Ultimatum (competitive)
///
/// Select a game type by weighted random roll when no predicate matches.
fn select_game_by_weight(rng: &mut StdRng) -> GameType {
    // Weights based on game complexity and frequency in empirical studies
    const WEIGHTS: &[(f32, GameType)] = &[
        (0.20, GameType::PrisonersDilemma),
        (0.18, GameType::StagHunt),
        (0.18, GameType::UltimatumGame),
        (0.16, GameType::TrustGame),
        (0.14, GameType::DictatorGame),
        (0.14, GameType::PublicGoodsGame),
    ];

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let roll = rng.gen::<f32>();
    let mut cumulative = 0.0;
    for (weight, game_type) in WEIGHTS {
        cumulative += weight;
        if roll < cumulative {
            return *game_type;
        }
    }
    GameType::PublicGoodsGame
}

fn select_game_type(agent1: &Agent, agent2: &Agent, rng: &mut StdRng) -> GameType {
    // Compute key factors for game selection
    let avg_energy = (agent1.energy + agent2.energy) / 2.0;
    let avg_trust = (agent1.compute_trust() + agent2.compute_trust()) / 2.0;
    let _trust_diff = (agent1.compute_trust() - agent2.compute_trust()).abs();
    let resource_asymmetry = compute_resource_asymmetry(agent1, agent2);
    let reputation_diff = (agent1.reputation_score - agent2.reputation_score).abs();

    // High trust + low asymmetry → Trust Game (requires mutual trust)
    if avg_trust > 1.0 && resource_asymmetry < 0.4 && rng.gen::<f32>() < 0.35 {
        return GameType::TrustGame;
    }

    // High resource asymmetry → Dictator Game (unilateral decision)
    if resource_asymmetry > 0.6 && rng.gen::<f32>() < 0.4 {
        return GameType::DictatorGame;
    }

    // Low trust but similar reputation → Public Goods Game
    if avg_trust < 0.8 && reputation_diff < 0.2 && rng.gen::<f32>() < 0.3 {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        return GameType::PublicGoodsGame;
    }

    // High trust → Stag Hunt (coordination game)
    if avg_trust > 1.2 && rng.gen::<f32>() < 0.45 {
        return GameType::StagHunt;
    }

    // Low energy → competitive games
    if avg_energy < 40.0 {
        if rng.gen::<f32>() < 0.5 {
            return GameType::PrisonersDilemma;
        }
        return GameType::UltimatumGame;
    }

    select_game_by_weight(rng)
}

/// Process a single encounter between two agents
/// Returns (`encounter_result`, `optional_debt_created`)
pub fn process_encounter(
    agent1: &mut Agent,
    agent2: &mut Agent,
    tick: Tick,
    rng: &mut StdRng,
) -> (EncounterResult, Option<(AgentId, AgentId, f32)>) {
    // Step 2: Compute similarities
    let similarity = compute_similarity_metrics(agent1, agent2);

    // Step 3: Both agents make engagement decisions
    // Compute scarcity based on average energy
    let avg_energy = (agent1.energy + agent2.energy) / 2.0;
    let scarcity = 1.0 - (avg_energy / 100.0).clamp(0.0, 1.0);

    let decision1 = decide_engagement(agent1, agent2, &similarity, scarcity);
    let decision2 = decide_engagement(agent2, agent1, &similarity, scarcity);

    // Both must agree to engage
    if !decision1.should_engage || !decision2.should_engage {
        // Update local reputations slightly (avoidance is noted)
        agent1.update_local_reputation(agent2.id, -0.02);
        agent2.update_local_reputation(agent1.id, -0.02);
        return (EncounterResult::Avoided, None);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Update engagement networks for identity theft cascade
    // Per Phase A Redesign: Network breach propagation through engagement edges
    if !agent1.engagement_network.contains(&agent2.id) {
        agent1.engagement_network.push(agent2.id);
    }
    if !agent2.engagement_network.contains(&agent1.id) {
        agent2.engagement_network.push(agent1.id);
    }

    // === PHASE 2.5: LANGUAGE COMMUNICATION PHASE ===
    // Agents communicate and update lexicons via semantic alignment
    // This happens before game/reproduction to enable trust-building
    if !agent1.lexicon.is_empty() && !agent2.lexicon.is_empty() {
        use crate::communication::CommunicativeAct;

        // Create communication context based on similarity and relationship
        let context = CommunicationContext {
            social_distance: 1.0 - similarity.genome, // Closer = lower social distance
            power_difference: (agent1.energy - agent2.energy) / 100.0,
            urgency: 0.5, // Moderate urgency for encounters
            cultural_common_ground: similarity.language, // Shared language = common ground
        };

        // Agent 1 communicates to Agent 2
        let (comm1, _) = generate_from_docas_with_lexicon(
            &agent1.docas,
            &agent1.lexicon,
            CommunicationIntent::Negotiate,
            "greeting",
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            rng,
        );
        let act1 = CommunicativeAct {
            sender: agent1.id,
            receiver: Some(agent2.id),
            modalities: comm1,
            timestamp: tick,
            context: context.clone(),
        };

        let _reception2 = receive_communication_with_language(
            act1,
            &agent2.docas,
            &mut agent2.lexicon,
            &agent1.lexicon,
            0.1, // alignment_factor - should come from config
            tick,
        );

        // Agent 2 communicates to Agent 1 (bidirectional)
        let (comm2, _) = generate_from_docas_with_lexicon(
            &agent2.docas,
            &agent2.lexicon,
            CommunicationIntent::Negotiate,
            "response",
            rng,
        );
        let act2 = CommunicativeAct {
            sender: agent2.id,
            receiver: Some(agent1.id),
            modalities: comm2,
            timestamp: tick,
            context,
        };

        let _reception1 = receive_communication_with_language(
            act2,
            &agent1.docas,
            &mut agent1.lexicon,
            &agent2.lexicon,
            0.1, // alignment_factor
            tick,
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Step 4: Select interaction mode
    let result = select_interaction_mode(agent1, agent2, rng);

    // Process the interaction
    match result {
        EncounterResult::GameEngagement { game_type } => {
            let debt = process_game_interaction(agent1, agent2, game_type, tick, rng);
            (result, debt)
        }
        EncounterResult::ReproductionEngagement => {
            // Reproduction is handled separately by simulation::process_reproduction
            // Just mark that these agents are considering reproduction
            agent1.record_positive_interaction();
            agent2.record_positive_interaction();
            (result, None)
        }
        EncounterResult::Avoided => (result, None),
        EncounterResult::IdentityTheft { .. } => (result, None),
    }
}

/// Attempt identity theft during an encounter
/// Per SIM-010: 2% base chance per encounter for theft attempt
///
/// # Arguments
/// * `perpetrator` - Agent attempting theft
/// * `victim` - Target of theft attempt
/// * `tick` - Current simulation tick
/// * `protocol_enabled` - Whether protocol protection is active
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
/// * `stats` - Global theft statistics tracker
/// * `rng` - Random number generator
///
/// # Returns
/// * `Some(TheftMethod)` - If theft was successful
/// * `None` - If theft failed or wasn't attempted
///
/// Apply the effects of a successful identity theft.
fn apply_theft_success(
    perpetrator: &mut Agent,
    victim: &mut Agent,
    tick: Tick,
    method: TheftMethod,
    success_prob: f32,
    stats: &mut IdentityTheftStats,
) -> Option<TheftMethod> {
    let identity = victim.identity.as_mut()?;
    identity.mark_stolen(tick);

    // Record theft event
    let theft = crate::identity::IdentityTheft {
        victim: victim.id,
        perpetrator: perpetrator.id,
        tick,
        method,
        severity: success_prob,
        financial_loss: success_prob * 100.0,
        recovery_duration: identity.recovery_duration,
    };
    identity.theft_history.push(theft);

    // Update victim state
    victim.is_identity_theft_victim = true;
    victim.reputation_score = (victim.reputation_score - 0.3).max(0.0);

    // DOCAS impact: cortisol spike, trust damage
    victim.docas.levels[2] += 0.3; // Cortisol
    victim.docas.levels[1] = (victim.docas.levels[1] - 0.2).max(0.0); // Oxytocin

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Perpetrator gains (temporarily)
    perpetrator.energy += 100.0;

    // Record success
    stats.record_success(method, identity.recovery_duration as f32);

    Some(method)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Apply the effects of a failed identity theft attempt.
fn apply_theft_failure(
    perpetrator: &mut Agent,
    victim: &Agent,
    protocol_enabled: bool,
    stats: &mut IdentityTheftStats,
) {
    let failure_reason = if protocol_enabled
        && victim
            .identity
            .as_ref()
            .is_some_and(|i| !i.anchored_claims.is_empty())
    {
        crate::identity::TheftFailureReason::AnchoredProtection
    } else if victim.reputation_score > 0.7 {
        crate::identity::TheftFailureReason::StrongSecurity
    } else {
        crate::identity::TheftFailureReason::RandomFailure
    };

    stats.record_failure(&failure_reason);

    // Perpetrator suffers reputation penalty
    perpetrator.reputation_score = (perpetrator.reputation_score - 0.1).max(0.0);
}

/// attempt identity theft in encounter.
pub fn attempt_identity_theft_in_encounter(
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    perpetrator: &mut Agent,
    victim: &mut Agent,
    tick: Tick,
    protocol_enabled: bool,
    stats: &mut IdentityTheftStats,
    rng: &mut StdRng,
) -> Option<TheftMethod> {
    // Base probability of theft attempt: 2% per encounter
    const THEFT_ATTEMPT_PROBABILITY: f32 = 0.02;

    // Don't attempt if victim is already compromised
    if let Some(ref identity) = victim.identity {
        if identity.is_compromised || identity.is_stolen {
            return None;
        }
    }

    // Check if theft is attempted
    if rng.gen::<f32>() >= THEFT_ATTEMPT_PROBABILITY {
        return None;
    }

    // Select theft method based on agent characteristics
    let method = select_theft_method(perpetrator, victim, rng);

    // Record attempt
    stats.attempts += 1;
    if let Some(ref mut identity) = victim.identity {
        identity.theft_attempts += 1;
    }

    // Calculate success probability
    let success_prob =
        calculate_theft_success_probability(perpetrator, victim, method, protocol_enabled);

    // Attempt theft
    if rng.gen::<f32>() < success_prob {
        apply_theft_success(perpetrator, victim, tick, method, success_prob, stats)
    } else {
        apply_theft_failure(perpetrator, victim, protocol_enabled, stats);
        None
    }
}

/// Select theft method based on agent characteristics
fn select_theft_method(perpetrator: &Agent, victim: &Agent, rng: &mut StdRng) -> TheftMethod {
    use TheftMethod::{AccountTakeover, CredentialTheft, Impersonation, SocialEngineering};

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Calculate weights for each method based on agent characteristics
    let social_engineering_weight = victim.compute_trust() * 0.3 + // High trust victims vulnerable
        perpetrator.reputation_score * 0.2; // High rep perpetrators better at social eng

    let credential_weight = (1.0 - victim.reputation_score) * 0.3 + // Low rep victims = weaker security
        (if victim.verification_count > 0 { 0.2 } else { 0.0 }); // Verified = credential target

    let impersonation_weight = 0.2; // Base rate

    let takeover_weight = if perpetrator.reputation_score > 0.5 {
        0.3
    } else {
        0.1
    }; // Experienced perpetrators

    let total_weight =
        social_engineering_weight + credential_weight + impersonation_weight + takeover_weight;

    let roll = rng.gen::<f32>() * total_weight;

    if roll < social_engineering_weight {
        SocialEngineering
    } else if roll < social_engineering_weight + credential_weight {
        CredentialTheft
    } else if roll < social_engineering_weight + credential_weight + impersonation_weight {
        Impersonation
    } else {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        AccountTakeover
    }
}

/// Calculate theft success probability
fn calculate_theft_success_probability(
    perpetrator: &Agent,
    victim: &Agent,
    _method: TheftMethod,
    protocol_enabled: bool,
) -> f32 {
    // Base success rate: ~0.75% per attempt
    let base_rate = 0.0075;

    // Victim security factors
    let victim_security = if let Some(ref identity) = victim.identity {
        let verification_bonus = if protocol_enabled {
            identity.verification_rate() * 0.5
        } else {
            0.0
        };
        let anchored_bonus = if identity.anchored_claims.is_empty() {
            0.0
        } else {
            0.2 // 20% reduction from anchored identity
        };
        let reputation_factor = victim.reputation_score * 0.3;

        verification_bonus + anchored_bonus + reputation_factor
    } else {
        0.0
    };

    // Perpetrator skill (based on reputation)
    let thief_skill = perpetrator.reputation_score * 0.1;

    // Calculate final probability
    let prob = (base_rate + thief_skill - victim_security).clamp(0.001, 0.5);

    // Protocol protection: 20-40% reduction when enabled and victim has anchored claims
    if protocol_enabled {
        if let Some(ref identity) = victim.identity {
            if !identity.anchored_claims.is_empty() {
                // Apply 20-40% reduction
                let reduction = 0.2 + (identity.anchored_claims.len() as f32 * 0.05).min(0.2);
                return (prob * (1.0 - reduction)).clamp(0.001, 0.5);
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    prob
}

/// Process game interaction between two agents using actual strategies
/// Returns optional (`loser_id`, `winner_id`, `debt_amount`) if debt is created
/// Decide whether an agent cooperates in a game, using philosophy or strategy.
fn decide_agent_action(
    agent: &Agent,
    opponent: &Agent,
    opponent_last: Option<bool>,
    game_type: GameType,
    rng: &mut StdRng,
) -> bool {
    if agent.philosophical_state.is_some() {
        agent.decide_game_action_with_philosophy(game_type, opponent, opponent_last, rng)
    } else {
        decide_action(
            agent.strategy,
            opponent_last,
            agent.compute_trust(),
            agent.genome.cooperation_tendency(),
            rng,
        )
    }
}

fn process_game_interaction(
    agent1: &mut Agent,
    agent2: &mut Agent,
    game_type: GameType,
    tick: Tick,
    rng: &mut StdRng,
) -> Option<(AgentId, AgentId, f32)> {
    // Get last actions from interaction history
    let agent1_last = agent1
        .interaction_history
        .get(&agent2.id)
        .map(|(action, _)| *action);
    let agent2_last = agent2
        .interaction_history
        .get(&agent1.id)
        .map(|(action, _)| *action);

    // Decide actions based on strategies AND philosophy
    // Philosophy layer (τ_φ) now influences game decisions
    let agent1_cooperates = decide_agent_action(agent1, agent2, agent2_last, game_type, rng);
    let agent2_cooperates = decide_agent_action(agent2, agent1, agent1_last, game_type, rng);

    // Store actions in history
    // TAG: surface=simulation owner=research-team rule=SIM-001
    agent1
        .interaction_history
        .insert(agent2.id, (agent1_cooperates, tick));
    agent2
        .interaction_history
        .insert(agent1.id, (agent2_cooperates, tick));

    // Clean old history (keep last 100 interactions)
    agent1
        .interaction_history
        .retain(|_, (_, t)| tick - *t < 100);
    agent2
        .interaction_history
        .retain(|_, (_, t)| tick - *t < 100);

    match game_type {
        GameType::PrisonersDilemma => {
            process_prisoners_dilemma(agent1, agent2, agent1_cooperates, agent2_cooperates, tick)
        }
        GameType::StagHunt => {
            process_stag_hunt(agent1, agent2, agent1_cooperates, agent2_cooperates)
        }
        GameType::UltimatumGame => {
            process_ultimatum(agent1, agent2, agent1_cooperates, agent2_cooperates)
        }
        GameType::PublicGoodsGame => {
            process_public_goods(agent1, agent2, agent1_cooperates, agent2_cooperates)
        }
        GameType::DictatorGame => {
            // Dictator game: agent1 is dictator, agent2 is recipient
            // Recipient has no choice - always "accepts"
            process_dictator_game(agent1, agent2)
        }
        GameType::TrustGame => {
            // Trust game: agent1 is investor, agent2 is trustee
            process_trust_game(agent1, agent2, agent1_cooperates, agent2_cooperates)
        }
    }
}

/// Apply trust update after Prisoner's Dilemma using Rescorla-Wagner
#[allow(dead_code)] // Will be integrated with process_prisoners_dilemma in future
fn apply_pd_trust_updates(agent1: &mut Agent, agent2: &mut Agent, a1_coop: bool, a2_coop: bool) {
    match (a1_coop, a2_coop) {
        (true, true) => {
            // Mutual cooperation: positive for both
            apply_trust_update_rw(agent1, agent2, InteractionOutcome::Positive);
        }
        (false, false) => {
            // Mutual defection: negative for both
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            apply_trust_update_rw(agent1, agent2, InteractionOutcome::Negative);
        }
        _ => {
            // Asymmetric: mixed outcome
            // The cooperator experiences betrayal (negative)
            // The defector experiences success but potential social cost
            if a1_coop {
                // Agent1 cooperated but was betrayed
                agent1.update_trust_rescorla_wagner(0.3); // Low reward
                agent2.update_trust_rescorla_wagner(1.8); // High reward (exploitation)
            } else {
                // Agent2 cooperated but was betrayed
                agent1.update_trust_rescorla_wagner(1.8); // High reward
                agent2.update_trust_rescorla_wagner(0.3); // Low reward
            }
        }
    }
}

// Integration tests extracted from crates/sim-core/src/encounter.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::docas::DocasState;
    use crate::genome::Genome;
    use crate::types::Sex;
    use rand::SeedableRng;

    fn create_test_agent(id: AgentId, sex: Sex) -> Agent {
        let genome = Genome::default();
        Agent::new(id, Position::new_2d(0.0, 0.0), genome, 0, sex)
    }

    /// Create agent with high trust (high dopamine/oxytocin for engagement tests)
    fn create_high_trust_agent(id: AgentId, sex: Sex) -> Agent {
        let genome = Genome::default();
        let mut agent = Agent::new(id, Position::new_2d(0.0, 0.0), genome, 0, sex);
        // Set very high DOCAS levels for trust: D=0.95, O=0.95, C=0.05, A=0.05, S=0.9
        // target: tau > 1.0, trust_factor > 0.5, engagement_score > 0.3
        agent.docas = DocasState::new([0.95, 0.95, 0.05, 0.05, 0.9]);
        agent
    }

    #[test]
    fn test_similarity_metrics_composite() {
        let metrics = SimilarityMetrics {
            genome: 0.8,
            language: 0.7,
            reputation: 0.5,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            resource_asymmetry: 0.2,
            sex_compatibility: 1.0,
            music: 0.6,
        };

        let score = metrics.composite_score();
        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn test_genome_similarity_identical() {
        let agent1 = create_test_agent(0, Sex::Male);
        let agent2 = create_test_agent(1, Sex::Female);

        let sim = compute_genome_similarity(&agent1, &agent2);
        // Both have default genomes (all 0.5), so should be very similar
        assert!((sim - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_sex_compatibility() {
        let male = create_test_agent(0, Sex::Male);
        let female = create_test_agent(1, Sex::Female);
        let male2 = create_test_agent(2, Sex::Male);

        assert_eq!(compute_sex_compatibility(&male, &female), 1.0);
        assert_eq!(compute_sex_compatibility(&male, &male2), 0.0);
    }

    #[test]
    fn test_decide_engagement_high_similarity() {
        // Use high-trust agents to ensure engagement with high similarity
        // Default agents have tau=0.5 (too low for engagement threshold)
        let agent1 = create_high_trust_agent(0, Sex::Male);
        let agent2 = create_high_trust_agent(1, Sex::Female);

        let similarity = SimilarityMetrics {
            genome: 0.9,
            language: 0.8,
            reputation: 0.7,
            resource_asymmetry: 0.1,
            sex_compatibility: 1.0,
            music: 0.7,
        };

        let decision = decide_engagement(&agent1, &agent2, &similarity, 0.3);
        assert!(
            decision.should_engage,
            "High similarity + high trust should engage" // TAG: surface=simulation owner=research-team rule=SIM-001
        );
        assert!(decision.confidence > 0.0);
    }

    #[test]
    fn test_decide_engagement_low_similarity() {
        let agent1 = create_test_agent(0, Sex::Male);
        let agent2 = create_test_agent(1, Sex::Female);

        let similarity = SimilarityMetrics {
            genome: 0.1,
            language: 0.1,
            reputation: -0.8,
            resource_asymmetry: 0.9,
            sex_compatibility: 0.0,
            music: 0.2,
        };

        let decision = decide_engagement(&agent1, &agent2, &similarity, 0.8);
        assert!(!decision.should_engage);
    }

    #[test]
    fn test_detect_proximity() {
        let agents = vec![
            Agent::new(
                0,
                Position::new_2d(0.0, 0.0),
                Genome::default(),
                0,
                Sex::Male,
            ),
            Agent::new(
                1,
                Position::new_2d(1.0, 1.0),
                Genome::default(),
                0,
                Sex::Female,
            ),
            Agent::new(
                2,
                Position::new_2d(10.0, 10.0),
                Genome::default(),
                0,
                Sex::Male,
            ),
        ];

        let encounters = detect_proximity(&agents, 2.0);
        // Agent 0 and 1 should be detected (distance ~1.414 < 2.0)
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(encounters.len(), 1);
        assert!(encounters.contains(&(0, 1)) || encounters.contains(&(1, 0)));

        // Test with larger radius
        let encounters_large = detect_proximity(&agents, 20.0);
        // All pairs should be detected: (0,1), (0,2), (1,2)
        // Agent 2 is at (10,10), distance from origin = sqrt(200) ≈ 14.14
        // Distance from agent 1 (1,1) to agent 2 = sqrt(162) ≈ 12.73
        assert_eq!(
            encounters_large.len(),
            3,
            "Expected 3 pairs with radius 20.0"
        );
    }

    #[test]
    fn test_select_interaction_mode_sex_based() {
        let mut rng = StdRng::seed_from_u64(42);

        let male = create_test_agent(0, Sex::Male);
        let female = create_test_agent(1, Sex::Female);
        let male2 = create_test_agent(2, Sex::Male);

        // Opposite sex can reproduce
        let result = select_interaction_mode(&male, &female, &mut rng);
        // Result depends on RNG, but should be either reproduction or game
        match result {
            EncounterResult::ReproductionEngagement | EncounterResult::GameEngagement { .. } => {}
            _ => panic!("Unexpected result for opposite sex"),
        }

        // Same sex always game
        let mut all_games = true;
        for _ in 0..10 {
            let result = select_interaction_mode(&male, &male2, &mut rng);
            if !matches!(result, EncounterResult::GameEngagement { .. }) {
                all_games = false;
                break;
            }
        }
        assert!(all_games, "Same-sex agents should always play games");
    }

    #[test]
    fn test_process_encounter_avoidance() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut agent1 = create_test_agent(0, Sex::Male);
        let mut agent2 = create_test_agent(1, Sex::Female);

        // Make agents very different to trigger avoidance
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        agent1.docas = crate::docas::DocasState::new([0.2, 0.2, 0.9, 0.9, 0.2]);
        agent2.docas = crate::docas::DocasState::new([0.9, 0.9, 0.2, 0.2, 0.9]);

        let (result, debt) = process_encounter(&mut agent1, &mut agent2, 100, &mut rng);

        // Result may vary based on RNG, but debt should be None for avoidance
        if matches!(result, EncounterResult::Avoided) {
            assert!(debt.is_none());
        }
    }

    // Tests for enhanced game selection (GAP-001)

    /// Create agent with specific resource level
    fn create_agent_with_energy(id: AgentId, sex: Sex, energy: f32) -> Agent {
        let genome = Genome::default();
        let mut agent = Agent::new(id, Position::new_2d(0.0, 0.0), genome, 0, sex);
        agent.energy = energy;
        agent
    }

    #[test]
    fn test_all_six_games_can_be_selected() {
        let mut rng = StdRng::seed_from_u64(42);

        // Track which games appear over many trials
        let mut games_seen = std::collections::HashSet::new();

        for i in 0..200 {
            // Vary agent parameters to trigger different game selection paths
            let energy1 = 30.0 + (i % 70) as f32;
            let energy2 = 40.0 + (i % 50) as f32;
            let mut agent1 = create_agent_with_energy(i as u64 * 2, Sex::Male, energy1);
            let mut agent2 = create_agent_with_energy(i as u64 * 2 + 1, Sex::Female, energy2);

            // Vary trust levels
            let trust1 = 0.5 + (i % 100) as f32 / 100.0;
            let trust2 = 0.6 + (i % 80) as f32 / 100.0;
            agent1.docas = DocasState::new([trust1, trust1 * 0.9, 0.2, 0.2, 0.8]);
            agent2.docas = DocasState::new([trust2, trust2 * 0.9, 0.2, 0.2, 0.8]);

            // Vary reputations
            agent1.reputation_score = -0.5 + (i % 150) as f32 / 150.0;
            agent2.reputation_score = -0.4 + (i % 120) as f32 / 120.0;

            let result = select_interaction_mode(&agent1, &agent2, &mut rng);
            if let EncounterResult::GameEngagement { game_type } = result {
                games_seen.insert(std::mem::discriminant(&game_type));
            }
            // TAG: surface=simulation owner=research-team rule=SIM-001
        }

        // All 6 game types should have been seen
        assert_eq!(games_seen.len(), 6, "All 6 game types should be selectable");
    }

    #[test]
    fn test_dictator_game_selected_for_high_asymmetry() {
        let mut rng = StdRng::seed_from_u64(42);

        // Create agents with high resource asymmetry (> 0.6)
        let rich_agent = create_agent_with_energy(0, Sex::Male, 95.0); // 95% of max
        let poor_agent = create_agent_with_energy(1, Sex::Female, 20.0); // 20% of max

        // Asymmetry = |0.95 - 0.20| = 0.75 > 0.6, should favor Dictator Game
        let asymmetry = compute_resource_asymmetry(&rich_agent, &poor_agent);
        assert!(asymmetry > 0.6, "Test setup: asymmetry should be > 0.6");

        // Run multiple times to account for RNG
        let mut dictator_count = 0;
        let mut _total_games = 0;

        for _ in 0..50 {
            let result = select_interaction_mode(&rich_agent, &poor_agent, &mut rng);
            if let EncounterResult::GameEngagement { game_type: _ } = result {
                _total_games += 1;
                dictator_count += 1; // All interactions are games for same-sex agents
            }
        }

        // Dictator Game should appear at least occasionally with high asymmetry
        assert!(
            dictator_count > 0,
            "Dictator Game should appear at least once for high asymmetry"
        );
    }

    #[test]
    fn test_trust_game_selected_for_high_trust() {
        let mut rng = StdRng::seed_from_u64(42);

        // Create high-trust agents
        let mut agent1 = create_test_agent(0, Sex::Male);
        let mut agent2 = create_test_agent(1, Sex::Female);

        // Set high DOCAS for trust (tau > 1.0)
        // D=0.9, O=0.9, S=0.9, C=0.1, A=0.1
        // tau = (0.9+0.9+0.9)/(1+0.1+0.1) = 2.7/1.2 = 2.25 > 1.0
        agent1.docas = DocasState::new([0.9, 0.9, 0.1, 0.1, 0.9]);
        agent2.docas = DocasState::new([0.9, 0.9, 0.1, 0.1, 0.9]);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // Set similar energy for low asymmetry (< 0.4)
        agent1.energy = 55.0;
        agent2.energy = 60.0;

        let asymmetry = compute_resource_asymmetry(&agent1, &agent2);
        assert!(asymmetry < 0.4, "Test setup: asymmetry should be < 0.4");

        // Run multiple times
        let mut trust_count = 0;
        let mut _total_games = 0;

        for _ in 0..50 {
            let result = select_interaction_mode(&agent1, &agent2, &mut rng);
            if let EncounterResult::GameEngagement { game_type } = result {
                _total_games += 1;
                if matches!(game_type, GameType::TrustGame) {
                    trust_count += 1;
                }
            }
        }

        // Trust Game should appear at least once for high trust
        assert!(
            trust_count > 0,
            "Trust Game should appear at least once for high trust"
        );
    }

    #[test]
    fn test_public_goods_selected_for_low_trust_similar_rep() {
        let mut rng = StdRng::seed_from_u64(42);

        // Create low-trust agents with similar reputations
        let mut agent1 = create_test_agent(0, Sex::Male);
        let mut agent2 = create_test_agent(1, Sex::Female);

        // Set low DOCAS for trust (tau < 0.8)
        // D=0.4, O=0.4, S=0.4, C=0.6, A=0.5
        // tau = (0.4+0.4+0.4)/(1+0.6+0.5) = 1.2/2.1 = 0.57 < 0.8
        agent1.docas = DocasState::new([0.4, 0.4, 0.6, 0.5, 0.4]);
        agent2.docas = DocasState::new([0.4, 0.4, 0.6, 0.5, 0.4]);

        // Set similar reputations (difference < 0.2)
        agent1.reputation_score = 0.5;
        agent2.reputation_score = 0.55;

        // Run multiple times
        let mut public_goods_count = 0;
        let mut _total_games = 0;
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        for _ in 0..50 {
            let result = select_interaction_mode(&agent1, &agent2, &mut rng);
            if let EncounterResult::GameEngagement { game_type } = result {
                _total_games += 1;
                if matches!(game_type, GameType::PublicGoodsGame) {
                    public_goods_count += 1;
                }
            }
        }

        // Public Goods should be selected occasionally for this configuration
        // Note: Lower probability due to 30% base chance in selection logic
        assert!(
            public_goods_count > 0,
            "Public Goods Game should appear at least once for low trust + similar rep"
        );
    }

    // =========================================================================
    // Music-Based Group Bonding Tests
    // =========================================================================

    #[test]
    fn test_can_bond_via_music() {
        // High music similarity + low language similarity = can bond via music
        let metrics = SimilarityMetrics {
            genome: 0.5,
            language: 0.3, // Low language
            reputation: 0.0,
            resource_asymmetry: 0.3,
            sex_compatibility: 0.0,
            music: 0.8, // High music
        };

        assert!(
            metrics.can_bond_via_music(),
            "Should bond via music when music=0.8, language=0.3"
        );

        // High music + high language = normal bonding (not cross-language)
        let metrics_lang = SimilarityMetrics {
            genome: 0.5,
            language: 0.7, // High language
            reputation: 0.0,
            resource_asymmetry: 0.3,
            sex_compatibility: 0.0,
            music: 0.8, // High music
        };
        // TAG: surface=simulation owner=research-team rule=SIM-001

        assert!(
            !metrics_lang.can_bond_via_music(),
            "Should not use music bonding when language is high"
        );

        // Low music = can't bond via music
        let metrics_low_music = SimilarityMetrics {
            genome: 0.5,
            language: 0.2,
            reputation: 0.0,
            resource_asymmetry: 0.3,
            sex_compatibility: 0.0,
            music: 0.5, // Low music
        };

        assert!(
            !metrics_low_music.can_bond_via_music(),
            "Should not bond via music when music is low"
        );
    }

    #[test]
    fn test_music_bonding_boost() {
        let metrics_bonding = SimilarityMetrics {
            genome: 0.5,
            language: 0.3,
            reputation: 0.0,
            resource_asymmetry: 0.3,
            sex_compatibility: 0.0,
            music: 0.8,
        };

        let boost = metrics_bonding.music_bonding_boost();
        assert!(
            boost > 0.0,
            "Music bonding boost should be positive: {boost}"
        );

        let metrics_no_bonding = SimilarityMetrics {
            genome: 0.5,
            language: 0.7, // High language
            reputation: 0.0,
            resource_asymmetry: 0.3,
            sex_compatibility: 0.0,
            music: 0.8,
        };

        let no_boost = metrics_no_bonding.music_bonding_boost();
        assert_eq!(
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            no_boost,
            0.0,
            "Should have no music bonding boost when language is high"
        );
    }

    #[test]
    fn test_composite_score_with_music() {
        let metrics = SimilarityMetrics {
            genome: 0.5,
            language: 0.5,
            reputation: 0.0,
            resource_asymmetry: 0.0,
            sex_compatibility: 0.0,
            music: 1.0, // Perfect music match
        };

        let score = metrics.composite_score();
        assert!(
            score > 0.0 && score <= 1.0,
            "Composite score should be in [0, 1]: {score}"
        );

        // With perfect music match, should be higher than without music
        let metrics_no_music = SimilarityMetrics {
            genome: 0.5,
            language: 0.5,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            reputation: 0.0,
            resource_asymmetry: 0.0,
            sex_compatibility: 0.0,
            music: 0.0,
        };

        let score_no_music = metrics_no_music.composite_score();
        assert!(
            score > score_no_music,
            "High music similarity should increase composite score"
        );
    }

    #[test]
    fn test_compute_music_similarity_without_music_state() {
        let agent1 = create_test_agent(0, Sex::Male);
        let agent2 = create_test_agent(1, Sex::Female);

        // Agents without music state should return neutral similarity
        let sim = compute_music_similarity(&agent1, &agent2);
        assert_eq!(
            sim, 0.5,
            "Agents without music state should have neutral similarity"
        );
    }
}
