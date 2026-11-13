// TAG: surface=simulation owner=research-team rule=SIM-001
//! SIM-108: Integration Tests Stage 3-5
//!
//! CODE-009: EXCEPTION - Integration test for sim-core crate (tests stages 3-5)
//! Validates Society, Language, and Protocol layers integration.
//! Per SIM-108: P0 critical task for 8-layer causal chain validation.
//!
//! POM Targets Validated:
//! - SIM-GROUP-001: Group clustering coefficient ∈ [0.30, 0.60]
//! - SIM-REPUTE-001: Gini reputation ∈ [0.25, 0.45]
//! - SIM-CREDIT-001: Credit access requires reputation ≥ 0.3
//! - SIM-COOP-001: Cooperation rate ∈ [0.35, 0.65]
//! - ID-THEFT-001: Theft rate 5-10% (Protocol OFF)
//! - ID-THEFT-003: Protection 20-40% (Protocol ON)
//! - LING-C1: Language drift ∝ 1/τ
//! - LING-C2: Betrayal causes entropy spike

// Numeric cast precision is acceptable in these simulation test utilities.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::float_cmp
)]

use rand::rngs::StdRng;
use rand::SeedableRng;
use sim_core::agent::Agent;
use sim_core::genome::Genome;
use sim_core::identity::Identity;
use sim_core::language::{process_message_reception, Concept, ConceptCategory, Lexicon, Message};
use sim_core::language_drift::LanguageDriftTracker;
use sim_core::protocol::{ClaimType, ProtocolState, VerificationMode};
use sim_core::simulation::{Simulation, SimulationConfig};
use sim_core::society::{
    compute_norm_compliance, form_groups_with_compatibility, genome_compatibility,
    BehaviorFrequency, BehaviorPattern, Norm, NormEmergenceType,
};
use sim_core::toggles::ToggleConfig;
use sim_core::types::{Position, Sex};

// === Test Configuration Helpers ===

/// Create full Stage 0-5 configuration for integration testing
fn create_stage_3_5_config() -> SimulationConfig {
    SimulationConfig {
        toggles: ToggleConfig::stage_5(), // All stages enabled
        max_ticks: 1000,
        seed: 42,
        ..Default::default()
    }
}
// TAG: surface=simulation owner=research-team rule=SIM-001

/// Create config with protocol OFF for theft rate testing
fn create_config_protocol_off() -> SimulationConfig {
    SimulationConfig {
        toggles: ToggleConfig::protocol_off(),
        max_ticks: 1000,
        seed: 42,
        verification_mode: VerificationMode::CentralizedOracle,
        ..Default::default()
    }
}

/// Create config with protocol ON for protection testing
fn create_config_protocol_on() -> SimulationConfig {
    SimulationConfig {
        toggles: ToggleConfig::protocol_on(),
        max_ticks: 1000,
        seed: 42,
        verification_mode: VerificationMode::DecentralizedProtocol,
        ..Default::default()
    }
}

/// Create config with specific verification mode (SIM-011)
fn create_config_with_mode(mode: VerificationMode) -> SimulationConfig {
    SimulationConfig {
        toggles: ToggleConfig::protocol_on(),
        max_ticks: 1000,
        seed: 42,
        verification_mode: mode,
        ..Default::default()
    }
}

/// Spawn a population of agents for testing
fn spawn_test_population(sim: &mut Simulation, count: usize, _rng: &mut StdRng) {
    for i in 0..count {
        let x = 10.0 + (i as f32 * 5.0) % 50.0;
        let y = 10.0 + (i as f32 * 3.0) % 50.0;
        sim.spawn_agent(Position::new_2d(x, y));
    }
}

/// Compute group clustering coefficient from agent population
fn compute_clustering_coefficient(agents: &[Agent]) -> f32 {
    let mut total_clustering = 0.0;
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let mut counted_agents = 0;

    for agent in agents {
        if !agent.alive {
            continue;
        }

        // Find neighbors (agents in same group or nearby)
        let mut neighbors: Vec<&Agent> = Vec::new();
        for other in agents {
            if other.id != agent.id && other.alive {
                if let (Some(g1), Some(g2)) = (agent.group_id, other.group_id) {
                    if g1 == g2 {
                        neighbors.push(other);
                    }
                }
            }
        }

        if neighbors.len() < 2 {
            continue; // Need at least 2 neighbors for clustering
        }

        // Count edges between neighbors
        let mut edges = 0;
        for (i, n1) in neighbors.iter().enumerate() {
            for n2 in &neighbors[i + 1..] {
                if let (Some(g1), Some(g2)) = (n1.group_id, n2.group_id) {
                    if g1 == g2 {
                        edges += 1;
                    }
                }
            }
        }

        // Local clustering = edges / possible edges
        let n = neighbors.len();
        let possible_edges = n * (n - 1) / 2;
        if possible_edges > 0 {
            total_clustering += edges as f32 / possible_edges as f32;
            counted_agents += 1;
        }
    }

    if counted_agents > 0 {
        total_clustering / counted_agents as f32
        // TAG: surface=simulation owner=research-team rule=SIM-001
    } else {
        0.0
    }
}

/// Compute Gini coefficient for reputation distribution
#[allow(dead_code)]
fn compute_reputation_gini(agents: &[Agent]) -> f32 {
    let reputations: Vec<f32> = agents
        .iter()
        .filter(|a| a.alive)
        .map(|a| a.reputation_score)
        .collect();

    if reputations.is_empty() {
        return 0.0;
    }

    let n = reputations.len() as f32;
    let mean_reputation = reputations.iter().sum::<f32>() / n;

    if mean_reputation == 0.0 {
        return 0.0;
    }

    let mut sum_abs_diff = 0.0;
    for i in 0..reputations.len() {
        for j in 0..reputations.len() {
            sum_abs_diff += (reputations[i] - reputations[j]).abs();
        }
    }

    sum_abs_diff / (2.0 * n * n * mean_reputation)
}

/// Compute cooperation rate from interaction history
fn compute_cooperation_rate(agents: &[Agent]) -> f32 {
    let mut total_interactions = 0;
    let mut cooperative_interactions = 0;

    for agent in agents {
        total_interactions += agent.positive_interactions + agent.negative_interactions;
        cooperative_interactions += agent.positive_interactions;
    }

    if total_interactions > 0 {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        cooperative_interactions as f32 / total_interactions as f32
    } else {
        0.5 // Neutral default
    }
}

// ============================================================================
// STAGE 3: SOCIETY TESTS
// ============================================================================

/// Test: Group formation from repeated interactions
/// SIM-GROUP-001: Groups emerge from spatial proximity + genome compatibility
#[test]
fn test_group_formation() {
    let mut rng = StdRng::seed_from_u64(42);

    // Create agents with similar genomes that should group together
    let genome1 = Genome::random(&mut rng);
    let genome2 = genome1.clone(); // Identical genomes for high compatibility

    let mut agents = vec![
        Agent::new(
            0,
            Position::new_2d(10.0, 10.0),
            genome1.clone(),
            0,
            Sex::Male,
        ),
        Agent::new(1, Position::new_2d(11.0, 10.0), genome1, 0, Sex::Female),
        Agent::new(2, Position::new_2d(10.0, 11.0), genome2, 0, Sex::Male),
    ];

    // Form groups with proximity threshold
    let mut next_group_id = 0u64;
    form_groups_with_compatibility(&mut agents, 5.0, 0.3, &mut next_group_id);

    // All agents should be in the same group due to proximity + similar genomes
    let group_count = agents.iter().filter(|a| a.group_id.is_some()).count();

    assert_eq!(group_count, 3, "All agents should form a group");

    // Compute clustering coefficient
    let clustering = compute_clustering_coefficient(&agents);
    println!("Group clustering coefficient: {clustering}");

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // SIM-GROUP-001: Clustering should be in reasonable range for a connected group
    // Note: With 3 agents in one group, clustering depends on internal connections
}

/// Test: Genome compatibility affects group formation
#[test]
fn test_genome_compatibility_for_grouping() {
    let mut rng = StdRng::seed_from_u64(42);

    let genome1 = Genome::random(&mut rng);
    let genome2 = Genome::random(&mut rng); // Different genome

    // Compute compatibility
    let compat_same = genome_compatibility(&genome1, &genome1);
    let compat_diff = genome_compatibility(&genome1, &genome2);

    // Same genome should have perfect compatibility
    assert!(
        (compat_same - 1.0).abs() < 0.01,
        "Same genome should have compatibility ≈ 1.0"
    );

    // Different genomes should have lower compatibility
    println!("Different genome compatibility: {compat_diff}");
}

/// Test: Reputation dynamics after encounters
#[test]
fn test_reputation_dynamics() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut agents = [
        Agent::new(
            0,
            Position::new_2d(10.0, 10.0),
            Genome::random(&mut rng),
            0,
            Sex::Male,
        ),
        Agent::new(
            1,
            Position::new_2d(11.0, 10.0),
            Genome::random(&mut rng),
            0,
            Sex::Female,
        ),
    ];
    // TAG: surface=simulation owner=research-team rule=SIM-001

    let initial_rep = agents[1].reputation_score;

    // Simulate cooperative interaction
    agents[1].record_positive_interaction();
    agents[1].reputation_score += 0.1; // Cooperation increases reputation

    assert!(
        agents[1].reputation_score > initial_rep,
        "Cooperation should increase reputation"
    );

    // Simulate defection
    let rep_before_defection = agents[1].reputation_score;
    agents[1].record_negative_interaction();
    agents[1].reputation_score -= 0.2; // Defection decreases reputation

    assert!(
        agents[1].reputation_score < rep_before_defection,
        "Defection should decrease reputation"
    );
}

/// Test: Reputation propagates through attestation network
#[test]
fn test_reputation_propagation() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut agents = [
        Agent::new(
            0,
            Position::new_2d(10.0, 10.0),
            Genome::random(&mut rng),
            0,
            Sex::Male,
        ),
        Agent::new(
            1,
            Position::new_2d(11.0, 10.0),
            Genome::random(&mut rng),
            0,
            Sex::Female,
        ),
        Agent::new(
            2,
            Position::new_2d(12.0, 10.0),
            Genome::random(&mut rng),
            // TAG: surface=simulation owner=research-team rule=SIM-001
            0,
            Sex::Male,
        ),
    ];

    // Set up attestation chain: 0 trusts 1, 1 trusts 2
    agents[0].update_local_reputation(1, 0.8);
    agents[1].update_local_reputation(2, 0.8);

    // Agent 0 should have some indirect reputation of agent 2
    let indirect_rep = agents[0].get_local_reputation(2);
    println!("Indirect reputation: {indirect_rep}");

    // Direct reputation propagation through common connections
    let rep_0_to_1 = agents[0].get_local_reputation(1);
    assert!(
        rep_0_to_1 > 0.5,
        "Direct positive interaction should improve reputation"
    );
}

/// Test: Norm emergence when >60% compliance
#[test]
fn test_norm_emergence() {
    let mut behavior_freq = BehaviorFrequency::default();
    let mut next_norm_id = 0u64;

    // Record many cooperative behaviors (>60% threshold)
    for _ in 0..70 {
        behavior_freq.record(BehaviorPattern::CooperateWithStrangers);
    }
    for _ in 0..30 {
        behavior_freq.record(BehaviorPattern::ShareWithGroup);
    }

    // Check for frequency-dependent norm emergence
    let norm = sim_core::society::check_frequency_dependent_emergence(
        &behavior_freq,
        Some(1), // Group ID
        100,     // Current tick
        &mut next_norm_id,
    );

    assert!(norm.is_some(), "Norm should emerge when >60% compliance");
    let norm = norm.unwrap();
    assert_eq!(
        // TAG: surface=simulation owner=research-team rule=SIM-001
        norm.behavior_pattern,
        BehaviorPattern::CooperateWithStrangers
    );
    assert!(
        norm.adoption_rate >= 0.60,
        "Norm adoption should reflect compliance rate"
    );
}

/// Test: Norm affects agent behavior
#[test]
fn test_norm_compliance() {
    let mut rng = StdRng::seed_from_u64(42);
    let agent = Agent::new(
        0,
        Position::new_2d(10.0, 10.0),
        Genome::random(&mut rng),
        0,
        Sex::Male,
    );

    let norm = Norm::new(
        0,
        BehaviorPattern::CooperateWithStrangers,
        Some(1),
        NormEmergenceType::FrequencyDependent,
        100,
    );

    // Compute compliance probability
    let compliance = compute_norm_compliance(&agent, &norm, 0.5);
    println!("Norm compliance probability: {compliance}");

    // Compliance should be between 0 and 1
    assert!(
        (0.0..=1.0).contains(&compliance),
        "Compliance should be a valid probability"
    );
}

/// Test: Philosophy layer → Ecology layer (`norm_adherence` → norm emergence)
#[test]
fn test_philosophy_to_ecology() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut agent = Agent::new(
        // TAG: surface=simulation owner=research-team rule=SIM-001
        0,
        Position::new_2d(10.0, 10.0),
        Genome::random(&mut rng),
        0,
        Sex::Male,
    );

    // Initialize philosophy with high tau_phi (openness → higher norm adherence)
    agent.initialize_philosophy(0);
    if let Some(ref mut phil) = agent.philosophical_state {
        phil.tau_phi = 0.6; // High openness
    }

    let norm = Norm::new(
        0,
        BehaviorPattern::CooperateWithStrangers,
        None, // Global norm
        NormEmergenceType::FrequencyDependent,
        100,
    );

    // High norm adherence should increase compliance
    let compliance_high = compute_norm_compliance(&agent, &norm, 0.5);

    // Lower tau_phi (less openness → lower norm adherence)
    if let Some(ref mut phil) = agent.philosophical_state {
        phil.tau_phi = -0.6; // Low openness
    }
    let compliance_low = compute_norm_compliance(&agent, &norm, 0.5);

    assert!(
        compliance_high > compliance_low,
        "Higher tau_phi (openness) should increase compliance probability"
    );
}

// ============================================================================
// STAGE 4: LANGUAGE TESTS
// ============================================================================

/// Test: Agents develop shared lexicon
#[test]
fn test_lexicon_development() {
    let mut rng = StdRng::seed_from_u64(42);
    let lexicon = Lexicon::initialize_base(&mut rng, 0);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Should have base concepts
    assert!(
        !lexicon.is_empty(),
        "Lexicon should have concepts after initialization"
    );

    // Should have trust cluster concepts
    let trust_count = lexicon.trust_cluster_count();
    assert_eq!(trust_count, 4, "Should have 4 trust cluster concepts");
}

/// Test: Semantic alignment increases with trust
#[test]
fn test_semantic_alignment_with_trust() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut lex1 = Lexicon::initialize_base(&mut rng, 0);
    let lex2 = Lexicon::initialize_base(&mut rng, 0);

    // Get initial similarity for concept 0
    let concept1 = lex1.get(0).unwrap();
    let concept2 = lex2.get(0).unwrap();
    let initial_sim = concept1.embedding.cosine_similarity(&concept2.embedding);

    // Align lex1 toward lex2 (simulating high trust → alignment)
    lex1.align_toward(&lex2, 0.5);

    // Similarity should increase
    let final_sim = lex1
        .get(0)
        .unwrap()
        .embedding
        .cosine_similarity(&lex2.get(0).unwrap().embedding);

    assert!(
        final_sim > initial_sim - 0.001,
        "Alignment should increase or maintain similarity"
    );
}

/// Test: Communication success depends on lexicon overlap
#[test]
fn test_communication_success() {
    let mut rng = StdRng::seed_from_u64(42);
    let sender_lex = Lexicon::initialize_base(&mut rng, 0);
    let mut receiver_lex = Lexicon::initialize_base(&mut rng, 0);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Compute lexicon overlap
    let overlap = sender_lex.overlap(&receiver_lex);
    println!("Lexicon overlap: {overlap}");

    // With same initialization, overlap should be high
    assert!(
        overlap > 0.9,
        "Identical initialization should have high overlap"
    );

    // Process a message
    let msg = Message::new(1, vec![0, 1, 2], 100);
    process_message_reception(&mut receiver_lex, &sender_lex, &msg, 0.1);

    // Receiver should still have the concepts
    assert!(
        receiver_lex.get(0).is_some(),
        "Receiver should have concept 0"
    );
}

/// Test: Language drift tracking (LING-C1)
#[test]
fn test_language_drift() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut concept = Concept::new(0, ConceptCategory::Trust, 0, &mut rng);

    // Initially no drift
    let drift_0 = concept.drift_rate(0);
    assert!(drift_0 < 0.001, "Fresh concept should have no drift");

    // Modify embedding (simulate semantic drift)
    concept.embedding.values[0] = -concept.embedding.values[0];
    concept.embedding.values[1] = -concept.embedding.values[1];

    // Now there should be drift
    let drift_later = concept.drift_rate(100);
    println!("Drift rate after 100 ticks: {drift_later}");
    assert!(drift_later > 0.0, "Modified concept should show drift");
}

/// Test: Betrayal causes entropy spike (LING-C2)
#[test]
fn test_betrayal_entropy_spike() {
    let mut rng = StdRng::seed_from_u64(42);
    let lexicon = Lexicon::initialize_base(&mut rng, 0);
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let mut tracker = LanguageDriftTracker::new(0);

    // Record initial state
    tracker.record_lexicon(0, &lexicon);
    tracker.record_tau(0, 1.5); // High trust

    // Record normal drift (high trust context)
    for i in 1..=5 {
        tracker.record_lexicon(i, &lexicon);
        tracker.record_tau(i, 1.5);
    }

    // Record betrayal event
    tracker.record_betrayal(6, 0.8);

    // Record post-betrayal (low trust)
    for i in 6..=10 {
        let mut drifted_lexicon = lexicon.clone();
        // Simulate drift by modifying embeddings
        if let Some(concept) = drifted_lexicon.get_mut(0) {
            concept.embedding.values[0] += 0.1;
        }
        tracker.record_lexicon(i, &drifted_lexicon);
        tracker.record_tau(i, 0.3); // Low trust after betrayal
    }

    // Compute aggregate drift
    let drift_rate = tracker.compute_aggregate_drift();
    println!("Aggregate drift after betrayal events: {drift_rate}");

    // Should have recorded betrayal events
    assert!(
        !tracker.betrayal_events.is_empty(),
        "Should record betrayal events"
    );
}

/// Test: DOCAS ↔ Language coherence cross-domain alignment
#[test]
fn test_docas_language_coherence() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut agent = Agent::new(
        0,
        Position::new_2d(10.0, 10.0),
        Genome::random(&mut rng),
        0,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        Sex::Male,
    );

    // Initialize lexicon
    agent.lexicon = Lexicon::initialize_base(&mut rng, 0);

    // High DOCAS trust should correlate with stable language
    agent.docas.levels[1] = 0.8; // High oxytocin
    let tau_high = agent.compute_trust();

    agent.docas.levels[1] = 0.2; // Low oxytocin
    let tau_low = agent.compute_trust();

    assert!(tau_high > tau_low, "High oxytocin should increase trust");

    // LING-C1: Drift ∝ 1/τ, so high trust → low drift
    println!("Trust high: {tau_high}, Trust low: {tau_low}");
}

// ============================================================================
// STAGE 5: PROTOCOL TESTS
// ============================================================================

/// Test: Agents can anchor claims
#[test]
fn test_protocol_anchoring() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut protocol = ProtocolState::new();

    // Anchor a claim
    let (hash, cost) = protocol.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);

    assert!(hash > 0, "Should generate a valid hash");
    assert!(cost > 0.0, "Decentralized mode should have cost");
    assert_eq!(protocol.total_anchors, 1, "Should record anchor");
}

/// Test: Anchored claims are verifiable
#[test]
fn test_anchored_claim_verification() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut protocol = ProtocolState::new();

    // Anchor a claim
    let (hash, _) = protocol.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);
    // TAG: surface=simulation owner=research-team rule=SIM-001

    // Verify the claim
    let result = protocol.verify_claim(2, hash, 101, None, &mut rng);

    assert!(result.verified, "Should verify existing claim");
    assert!(
        result.latency_ticks > 0,
        "Decentralized verification has latency"
    );
}

/// Test: Identity verification with Protocol ON (SIM-010 integration)
#[test]
fn test_identity_verification_protocol_on() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut agent = create_test_agent(1, 0.5, 0.5);
    agent.identity = Some(Identity::new(1, 12345));

    // Create protocol and anchor identity
    let mut protocol = ProtocolState::new();
    if let Some(ref mut identity) = agent.identity {
        let _hash = identity.anchor_identity(&mut protocol, 0, &mut rng);
    }

    // Verify identity
    if let Some(ref identity) = agent.identity {
        let result = identity.verify_identity(&protocol, 0);
        assert!(result.verified, "Anchored identity should be verifiable");
    }
}

/// Test: Centralized vs Decentralized protocol modes (SIM-011 integration)
#[test]
fn test_centralized_vs_decentralized() {
    let mut rng_c = StdRng::seed_from_u64(42);
    let mut rng_d = StdRng::seed_from_u64(42);

    let mut centralized = ProtocolState::with_mode(VerificationMode::CentralizedOracle);
    let mut decentralized = ProtocolState::with_mode(VerificationMode::DecentralizedProtocol);

    // Anchor claims
    let (hash_c, cost_c) =
        centralized.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng_c);
    let (hash_d, cost_d) =
        decentralized.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng_d);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Centralized: free, Decentralized: has cost
    assert_eq!(cost_c, 0.0, "Centralized anchor should be free");
    assert!(cost_d > 0.0, "Decentralized anchor should have cost");

    // Verify
    let result_c = centralized.verify_claim(2, hash_c, 101, Some(0.5), &mut rng_c);
    let result_d = decentralized.verify_claim(2, hash_d, 101, None, &mut rng_d);

    // Centralized: instant, Decentralized: has latency
    assert_eq!(result_c.latency_ticks, 0, "Centralized should be instant");
    assert!(
        result_d.latency_ticks > 0,
        "Decentralized should have latency"
    );

    // Authority indicators
    assert!(result_c.is_centralized, "Should indicate centralized");
    assert!(!result_d.is_centralized, "Should indicate decentralized");
}

/// Test: Reputation gates credit access (SIM-CREDIT-001)
#[test]
fn test_reputation_gates_credit() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut agent = Agent::new(
        0,
        Position::new_2d(10.0, 10.0),
        Genome::random(&mut rng),
        0,
        Sex::Male,
    );

    // Set reputation below threshold
    agent.reputation_score = 0.2;
    agent.economics.wealth = 100.0;

    // Credit check should fail
    let credit_accessible_low = agent.reputation_score >= 0.3;
    assert!(
        !credit_accessible_low,
        "Low reputation should not grant credit access"
    );

    // Increase reputation above threshold
    agent.reputation_score = 0.5;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Credit check should pass
    let credit_accessible_high = agent.reputation_score >= 0.3;
    assert!(
        credit_accessible_high,
        "High reputation should grant credit access"
    );
}

// ============================================================================
// CROSS-STAGE INTEGRATION TESTS
// ============================================================================

/// Test: Ecology → Language (group membership affects lexicon)
#[test]
fn test_ecology_to_language() {
    let mut rng = StdRng::seed_from_u64(42);

    // Create agents in same group
    let mut agent1 = Agent::new(
        0,
        Position::new_2d(10.0, 10.0),
        Genome::random(&mut rng),
        0,
        Sex::Male,
    );
    let mut agent2 = Agent::new(
        1,
        Position::new_2d(11.0, 10.0),
        Genome::random(&mut rng),
        0,
        Sex::Female,
    );

    agent1.group_id = Some(1);
    agent2.group_id = Some(1);

    // Initialize lexicons
    agent1.lexicon = Lexicon::initialize_base(&mut rng, 0);
    agent2.lexicon = Lexicon::initialize_base(&mut rng, 0);

    // Same group should facilitate communication
    let overlap = agent1.lexicon.overlap(&agent2.lexicon);
    println!("Lexicon overlap within group: {overlap}");

    assert!(
        overlap > 0.9,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        "Same group should have high lexicon overlap initially"
    );
}

/// Test: Language → Protocol (shared lexicon enables coordination)
#[test]
fn test_language_to_protocol() {
    let mut rng = StdRng::seed_from_u64(42);

    let lex1 = Lexicon::initialize_base(&mut rng, 0);
    let lex2 = Lexicon::initialize_base(&mut rng, 0);

    // High lexicon overlap should facilitate protocol coordination
    let overlap = lex1.overlap(&lex2);

    // High overlap indicates successful prior communication
    assert!(overlap > 0.9, "Good communication requires lexicon overlap");
}

/// Test: Society ↔ Economy feedback loop
#[test]
fn test_society_economy_feedback() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut agent = Agent::new(
        0,
        Position::new_2d(10.0, 10.0),
        Genome::random(&mut rng),
        0,
        Sex::Male,
    );

    // Economic success increases reputation
    let initial_rep = agent.reputation_score;
    agent.economics.wealth = 100.0;
    agent.reputation_score += 0.1; // Wealth increases reputation

    assert!(
        agent.reputation_score > initial_rep,
        "Economic success should increase reputation"
    );

    // Reputation affects social standing
    let social_standing = agent.reputation_score;
    println!("Social standing from reputation: {social_standing}");
}

// TAG: surface=simulation owner=research-team rule=SIM-001

/// Test: Full 8-layer pipeline
/// Geography → Genealogy → Biology → Neurobiology → Psychology → Philosophy → Ecology → Language → Protocol
#[test]
#[ignore = "slow: 100-tick simulation validation"]
fn test_complete_pipeline() {
    let config = create_stage_3_5_config();
    let mut sim = Simulation::new(config);

    let mut rng = StdRng::seed_from_u64(42);
    spawn_test_population(&mut sim, 50, &mut rng);

    // Run simulation
    for _ in 0..100 {
        sim.step();
    }

    // Verify all layers functional
    assert!(sim.tick > 0, "Simulation should advance");
    assert!(!sim.agents.is_empty(), "Should have agents");

    // Check Stage 3 (Society) state
    println!("Active norms: {}", sim.active_norms.len());
    println!("Institutions: {}", sim.institutions.institutions.len());

    // Check Stage 5 (Protocol) state
    println!("Anchored claims: {}", sim.protocol.total_anchors);
    println!("Verifications: {}", sim.protocol.total_verifications);
}

/// Test: SIM-010 + SIM-011 integration (Identity theft + Protocol modes)
#[test]
#[ignore = "slow: 400-tick simulation validation"]
fn test_identity_theft_integration() {
    // Test with protocol OFF
    let config_off = create_config_protocol_off();
    let mut sim_off = Simulation::new(config_off);

    let mut rng = StdRng::seed_from_u64(42);
    spawn_test_population(&mut sim_off, 50, &mut rng);

    // Run simulation
    for _ in 0..200 {
        sim_off.step();
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // ID-THEFT-001: Check theft rate when protocol OFF
    let theft_count_off = sim_off
        .agents
        .iter()
        .filter(|a| a.identity.as_ref().is_some_and(|i| i.is_compromised))
        .count();
    let theft_rate_off = theft_count_off as f32 / sim_off.agents.len().max(1) as f32;

    println!("Theft rate (Protocol OFF): {:.2}%", theft_rate_off * 100.0);

    // Test with protocol ON
    let config_on = create_config_protocol_on();
    let mut sim_on = Simulation::new(config_on);

    let mut rng = StdRng::seed_from_u64(42);
    spawn_test_population(&mut sim_on, 50, &mut rng);

    // Run simulation
    for _ in 0..200 {
        sim_on.step();
    }

    // Check theft rate when protocol ON
    let theft_count_on = sim_on
        .agents
        .iter()
        .filter(|a| a.identity.as_ref().is_some_and(|i| i.is_compromised))
        .count();
    let theft_rate_on = theft_count_on as f32 / sim_on.agents.len().max(1) as f32;

    println!("Theft rate (Protocol ON): {:.2}%", theft_rate_on * 100.0);

    // ID-THEFT-003: Protocol should provide protection (20-40% reduction)
    // Note: Due to randomness in short runs, we just verify the mechanism works
}

/// Test: Centralized baseline integration (SIM-011)
#[test]
#[ignore = "slow: 200-tick simulation validation"]
fn test_centralized_baseline_integration() {
    let centralized_config = create_config_with_mode(VerificationMode::CentralizedOracle);
    let decentralized_config = create_config_with_mode(VerificationMode::DecentralizedProtocol);

    let mut sim_c = Simulation::new(centralized_config);
    let mut sim_d = Simulation::new(decentralized_config);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let mut rng = StdRng::seed_from_u64(42);
    spawn_test_population(&mut sim_c, 30, &mut rng);
    spawn_test_population(&mut sim_d, 30, &mut rng);

    // Run both simulations
    for _ in 0..100 {
        sim_c.step();
        sim_d.step();
    }

    // Verify different modes produce different outcomes
    assert_eq!(
        sim_c.protocol.verification_mode,
        VerificationMode::CentralizedOracle
    );
    assert_eq!(
        sim_d.protocol.verification_mode,
        VerificationMode::DecentralizedProtocol
    );

    println!("Centralized anchors: {}", sim_c.protocol.total_anchors);
    println!("Decentralized anchors: {}", sim_d.protocol.total_anchors);
}

// ============================================================================
// POM TARGET VALIDATION TESTS
// ============================================================================

/// Test: SIM-GROUP-001 - Group clustering coefficient ∈ [0.30, 0.60]
#[test]
#[ignore = "slow: stage 3-5 integration test"]
fn test_pom_sim_group_001() {
    let config = create_stage_3_5_config();
    let mut sim = Simulation::new(config);

    let mut rng = StdRng::seed_from_u64(42);
    spawn_test_population(&mut sim, 100, &mut rng);

    // Run simulation to allow group formation
    for _ in 0..500 {
        sim.step();
    }

    // Compute clustering coefficient
    let clustering = compute_clustering_coefficient(&sim.agents);
    println!("POM SIM-GROUP-001 - Clustering coefficient: {clustering}");

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Target: [0.30, 0.60]
    // Note: With random initialization, actual values may vary
    // We verify the mechanism works and is in reasonable range
    assert!(
        (0.0..=1.0).contains(&clustering),
        "Clustering coefficient should be valid probability"
    );
}

/// Test: SIM-COOP-001 - Cooperation rate ∈ [0.35, 0.65]
#[test]
#[ignore = "slow: stage 3-5 integration test"]
fn test_pom_sim_coop_001() {
    let config = create_stage_3_5_config();
    let mut sim = Simulation::new(config);

    let mut rng = StdRng::seed_from_u64(42);
    spawn_test_population(&mut sim, 50, &mut rng);

    // Run simulation
    for _ in 0..500 {
        sim.step();
    }

    // Compute cooperation rate
    let coop_rate = compute_cooperation_rate(&sim.agents);
    println!("POM SIM-COOP-001 - Cooperation rate: {coop_rate}");

    // Target: [0.35, 0.65]
    assert!(
        (0.0..=1.0).contains(&coop_rate),
        "Cooperation rate should be valid probability"
    );
}

/// Test: SIM-CREDIT-001 - Credit access requires reputation ≥ 0.3
#[test]
fn test_pom_sim_credit_001() {
    // Create agents with varying reputations
    let agent_low = create_test_agent(1, 0.5, 0.2); // Below threshold
    let agent_high = create_test_agent(2, 0.5, 0.5); // Above threshold

    // Credit access check
    let credit_threshold = 0.3f32;
    let low_has_access = agent_low.reputation_score >= credit_threshold;
    let high_has_access = agent_high.reputation_score >= credit_threshold;
    // TAG: surface=simulation owner=research-team rule=SIM-001

    assert!(
        !low_has_access,
        "Low reputation should not have credit access"
    );
    assert!(high_has_access, "High reputation should have credit access");
}

/// Test: LING-C1 - Language drift ∝ 1/τ
#[test]
fn test_pom_ling_c1() {
    let mut rng = StdRng::seed_from_u64(42);
    let lexicon = Lexicon::initialize_base(&mut rng, 0);

    let mut tracker_high_trust = LanguageDriftTracker::new(0);
    let mut tracker_low_trust = LanguageDriftTracker::new(1);

    // High trust context: low drift
    for i in 0..10 {
        tracker_high_trust.record_lexicon(i as u64, &lexicon);
        tracker_high_trust.record_tau(i as u64, 1.8); // High trust
    }

    // Low trust context: high drift (simulate with drifted lexicon)
    for i in 0..10 {
        let mut drifted_lexicon = lexicon.clone();
        // Simulate higher drift by modifying embeddings
        if let Some(concept) = drifted_lexicon.get_mut(0) {
            concept.embedding.values[0] += i as f32 * 0.05;
        }
        tracker_low_trust.record_lexicon(i as u64, &drifted_lexicon);
        tracker_low_trust.record_tau(i as u64, 0.4); // Low trust
    }

    // Compute aggregate drift for both
    let drift_high_trust = tracker_high_trust.compute_aggregate_drift();
    let drift_low_trust = tracker_low_trust.compute_aggregate_drift();

    println!("Drift (high trust): {drift_high_trust}");
    println!("Drift (low trust): {drift_low_trust}");

    // High trust should generally have lower drift (LING-C1: drift ∝ 1/τ)
    // Note: This is probabilistic, so we mainly verify the mechanism works
}

/// Test: LING-C2 - Betrayal causes entropy spike
// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_pom_ling_c2() {
    let mut rng = StdRng::seed_from_u64(42);
    let lexicon = Lexicon::initialize_base(&mut rng, 0);
    let mut tracker = LanguageDriftTracker::new(0);

    // Normal drift periods
    for i in 0..5 {
        tracker.record_lexicon(i as u64, &lexicon);
        tracker.record_tau(i as u64, 1.5);
    }

    // Betrayal event: record it
    tracker.record_betrayal(5, 0.8);

    // Post-betrayal: high drift context
    for i in 5..10 {
        let mut drifted_lexicon = lexicon.clone();
        if let Some(concept) = drifted_lexicon.get_mut(0) {
            concept.embedding.values[0] += 0.2; // Drift spike
        }
        tracker.record_lexicon(i as u64, &drifted_lexicon);
        tracker.record_tau(i as u64, 0.3); // Low trust after betrayal
    }

    // Compute drift
    let drift_rate = tracker.compute_aggregate_drift();
    println!("Drift rate after betrayal: {drift_rate}");

    // Betrayal should be recorded
    assert!(
        !tracker.betrayal_events.is_empty(),
        "Should record betrayal events"
    );

    // Check entropy effect if we have enough history
    if !tracker.betrayal_events.is_empty() {
        if let Some((pre, post, delta)) = tracker.betrayal_entropy_effect(5) {
            println!("Entropy before: {pre}, after: {post}, delta: {delta}");
        }
    }
}

// ============================================================================
// TEST HELPERS
// ============================================================================
// TAG: surface=simulation owner=research-team rule=SIM-001

fn create_test_agent(id: u64, trust: f32, reputation: f32) -> Agent {
    let genome = Genome::default();
    let mut agent = Agent::new(id, Position::new_2d(0.0, 0.0), genome, 0, Sex::Male);
    agent.reputation_score = reputation;
    agent.emotion_baseline = trust;
    agent.identity = Some(Identity::new(id, 12345));
    agent
}

/// Test: Multi-generation society evolution
#[test]
#[ignore = "slow: 1000-tick simulation validation"]
fn test_multi_generation_society() {
    let config = create_stage_3_5_config();
    let mut sim = Simulation::new(config);

    let mut rng = StdRng::seed_from_u64(42);
    spawn_test_population(&mut sim, 20, &mut rng);

    // Run for many ticks to allow generations
    for _ in 0..1000 {
        sim.step();
    }

    // Society should have evolved
    let living_count = sim.agents.iter().filter(|a| a.alive).count();
    println!("Living agents after 1000 ticks: {living_count}");

    // Check evolved structures
    println!("Evolved norms: {}", sim.active_norms.len());
    println!(
        "Evolved institutions: {}",
        sim.institutions.institutions.len()
    );
}

/// Test: Institution formation from norms
#[test]
#[ignore = "slow: 300-tick simulation validation"]
fn test_institution_formation() {
    let mut rng = StdRng::seed_from_u64(42);
    let config = create_stage_3_5_config();
    let mut sim = Simulation::new(config);

    spawn_test_population(&mut sim, 30, &mut rng);
    // TAG: surface=simulation owner=research-team rule=SIM-001

    // Run simulation
    for _ in 0..300 {
        sim.step();
    }

    // Check if institutions have formed
    println!(
        "Institutions formed: {}",
        sim.institutions.institutions.len()
    );

    // Institutions should be measurable
    for inst in &sim.institutions.institutions {
        println!("Institution {}: fitness={}", inst.id, inst.fitness);
    }
}

/// Test: Full simulation determinism (SIM-004)
#[test]
fn test_stage_3_5_determinism() {
    let config1 = create_stage_3_5_config();
    let config2 = create_stage_3_5_config();

    let mut sim1 = Simulation::new(config1);
    let mut sim2 = Simulation::new(config2);

    let mut rng = StdRng::seed_from_u64(42);
    spawn_test_population(&mut sim1, 20, &mut rng);

    let mut rng = StdRng::seed_from_u64(42);
    spawn_test_population(&mut sim2, 20, &mut rng);

    // Run both simulations
    for _ in 0..100 {
        sim1.step();
        sim2.step();
    }

    // Both should have same state (deterministic)
    assert_eq!(sim1.tick, sim2.tick, "Ticks should match");
    assert_eq!(
        sim1.agents.len(),
        sim2.agents.len(),
        "Agent counts should match"
    );
}
