// TAG: surface=simulation owner=research-team rule=SIM-001
use super::super::*;
use crate::agent::Agent;
use crate::docas::DocasState;
use crate::genome::Genome;
use crate::types::{AgentId, Position, Sex};
use rand::rngs::StdRng;
use rand::SeedableRng;
fn create_test_agent(id: AgentId, x: f32, y: f32) -> Agent {
    Agent {
        id,
        position: Position::new_2d(x, y),
        energy: 100.0,
        age: 0,
        sex: Sex::Male,
        genome: Genome::default(),
        vision_range: 6,
        metabolic_rate: 1.5,
        docas: DocasState::default(),
        dmn_budget: 100,
        criticality_sigma: 1.0,
        emotion_baseline: 0.5,
        reputation_score: 0.5,
        cognitive_bandwidth: 0.5,

        group_id: None,
        cultural_traits: Vec::new(),
        isolation_score: 0.0,
        attestations_given: Vec::new(),
        attestations_received: Vec::new(),
        lexicon: crate::language::Lexicon::default(),
        has_verified_id: false,
        verification_count: 0,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        strategy: crate::types::StrategyType::TitForTat,
        is_identity_theft_victim: false,
        identity_theft_event: None,
        identity: None,
        theft_recovery_tick: None,
        credit_report: crate::identity::CreditReport::clean(),
        credit_error_recovery_tick: None,
        risk_profile: crate::identity::RiskProfile::default(),
        engagement_network: Vec::new(),
        birth_tick: 0,
        alive: true,
        positive_interactions: 0,
        negative_interactions: 0,
        interaction_history: std::collections::HashMap::new(),
        economics: crate::economics::EconomicState::default(),
        debt_obligations: Vec::new(),
        local_reputation: std::collections::HashMap::new(),
        cognitive_system: crate::types::CognitiveSystem::System1,
        dmn_horizon: crate::types::H_DEFAULT,
        music_state: None,
        // Emotion state
        emotion_state: crate::emotion::EmotionState::default(),
        // Language drift tracking
        language_drift: crate::language_drift::LanguageDriftTracker::new(id),
        // Trust curvature fields (§6.6.2)
        docas_history: std::collections::VecDeque::new(),
        trust_curvature: crate::docas::TrustCurvature::new(),
        // Viscosity fields (§6.6.3)
        tau_history: Vec::new(),
        trust_viscosity: crate::docas::TrustViscosity::default(),
        adaptation_rate: 1.0,
        // Psychology Layer (Stage 2)
        semantic_state: [0.5; 8],
        affect_valence: 0.0,
        affect_arousal: 0.5,
        social_orientation: 0.0,
        // Philosophy Layer (Stage 2) - new fields
        philosophical_state: None,
        // Meaning Layer
        meaning_state: None,
        // GTT Fields
        lambda: 0.61,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        expectation: 0.5,
        validation: 0.5,
        prediction_error: 0.0,
        stability: 1.0,
        // TFP Fields
        psi_bio: 0.5,
        psi_fin: 0.5,
        psi_ling: 0.5,
        psi_trust: 0.5,
        // Emotion Fields
        dominance: 0.5,
        // Linguistics Fields
        trust_in_meaning: 0.5,
        semantic_entropy: 0.5,
        semantic_coherence: 0.5,
        // Culture Fields
        cultural_quadrant: None,
        hofstede_dimensions: None,
        cultural_orientation: None,
        // Game Theory Fields
        game_history: Vec::new(),
        trust_in_partner: std::collections::HashMap::new(),
        cumulative_payoff: 0.0,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // PDE4B Fields
        pde4b_expression: 1.0,
        camp_level: 1.0,
        learning_rate_modifier: 0.61,
        // Neural Maturation Fields
        prefrontal_cortex: crate::agent::PrefrontalCortex::default(),
        // Epigenetics Fields
        epigenome: crate::epigenome::Epigenome::new(),
        // Big Five Personality
        personality: crate::psychology::PersonalityTraits::default(),
        // Working Memory
        working_memory: crate::cognition::WorkingMemory::new(0.0, 0),
        // Inbreeding
        inbreeding: crate::genetics::InbreedingTracker::new(),
        // Movement direction for visualization
        last_dx: 1.0,
        last_dy: 0.0,
        // Stigmergy fields
        pheromone_sensitivity: [1.0, 1.0, 1.0, 0.5, 0.8],
        chemotaxis_direction: None,
        last_pheromone_deposit_tick: 0,
        // Social forces (Barnes-Hut)
        social_force: crate::types::Vector::zero(),
        // Time preference
        time_preference: crate::economics::TimePreferenceProfile::default(),
        biomarkers: crate::biomarkers::AgentBiomarkers::default(),
    }
}

#[test]
fn test_build_social_network() {
    let mut agent_a = create_test_agent(0, 0.0, 0.0);
    let mut agent_b = create_test_agent(1, 5.0, 0.0);
    let agent_c = create_test_agent(2, 10.0, 0.0);

    // A and B have interacted (local reputation)
    agent_a.update_local_reputation(1, 0.5);
    agent_b.update_local_reputation(0, 0.5);
    // TAG: surface=simulation owner=research-team rule=SIM-001

    let agents = vec![agent_a, agent_b, agent_c];
    let attestations: Vec<Attestation> = vec![];

    let network = build_social_network(&agents, &attestations);

    // Should have edges between A and B
    assert!(network.contains_key(&0));
    assert!(network.contains_key(&1));

    // ERR-001: Test code - unwrap() exempt for test assertions
    assert!(network.get(&0).unwrap().iter().any(|(id, _)| *id == 1));
}

#[test]
fn test_propagate_reputation_network() {
    let mut agent_a = create_test_agent(0, 0.0, 0.0);
    let mut agent_b = create_test_agent(1, 5.0, 0.0);
    let mut agent_c = create_test_agent(2, 10.0, 0.0);

    // Set up reputation values
    agent_a.reputation_score = 0.8; // High reputation
    agent_b.reputation_score = 0.0;
    agent_c.reputation_score = 0.0;

    // Create interaction network: A-B, B-C
    agent_a.update_local_reputation(1, 0.5);
    agent_b.update_local_reputation(0, 0.5);
    agent_b.update_local_reputation(2, 0.5);
    agent_c.update_local_reputation(1, 0.5);

    let mut agents = vec![agent_a, agent_b, agent_c];
    let attestations: Vec<Attestation> = vec![];
    let config = ReputationPropagationConfig::default();

    let updates = propagate_reputation_network(&mut agents, &attestations, &config);

    // Should have some updates
    assert!(updates > 0, "Expected reputation updates to propagate");

    // C should now have some reputation information about A (through B)
    let c_rep_a = agents[2].get_local_reputation(0);
    assert!(
        c_rep_a != 0.5,
        "C's reputation of A should have been updated from neutral"
    );
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
#[test]
fn test_reputation_propagation_decay() {
    // Test that reputation decays with distance
    let mut agents: Vec<Agent> = (0..5)
        .map(|i| create_test_agent(i, i as f32 * 5.0, 0.0))
        .collect();

    // Set up chain: 0-1-2-3-4
    for i in 0..4 {
        agents[i].update_local_reputation(i as u64 + 1, 0.8);
        agents[i + 1].update_local_reputation(i as u64, 0.8);
    }

    // Agent 0 has high reputation
    agents[0].reputation_score = 1.0;

    let attestations: Vec<Attestation> = vec![];
    let config = ReputationPropagationConfig {
        decay_factor: 0.5,
        max_hops: 3,
        min_reputation_threshold: 0.1,
        attestation_boost: 1.0,
    };

    let updates = propagate_reputation_network(&mut agents, &attestations, &config);
    assert!(updates > 0);

    // Agent 4 should have weaker signal than agent 1
    let rep_1 = agents[1].get_local_reputation(0);
    let rep_4 = agents[4].get_local_reputation(0);

    // With 50% decay per hop, rep_4 should be much weaker than rep_1
    // But due to BFS ordering, exact values depend on propagation path
    assert!(
        rep_4.abs() <= rep_1.abs() || rep_4 == 0.5,
        "Distant agent should have weaker or neutral reputation signal" // TAG: surface=simulation owner=research-team rule=SIM-001
    );
}

#[test]
fn test_compute_reputation_network_metrics() {
    let mut agent_a = create_test_agent(0, 0.0, 0.0);
    let mut agent_b = create_test_agent(1, 5.0, 0.0);
    let agent_c = create_test_agent(2, 10.0, 0.0);

    // Create connections
    agent_a.update_local_reputation(1, 0.5);
    agent_b.update_local_reputation(0, 0.5);

    let agents = vec![agent_a, agent_b, agent_c];
    let metrics = compute_reputation_network_metrics(&agents);

    // 2 connections among 3 agents = 0.667 avg
    assert!(metrics.avg_connections > 0.0);
    assert_eq!(metrics.total_connections, 2);
}

#[test]
fn test_attestation_boosts_propagation() {
    let mut agent_a = create_test_agent(0, 0.0, 0.0);
    let agent_b = create_test_agent(1, 5.0, 0.0);
    let agent_c = create_test_agent(2, 10.0, 0.0);

    agent_a.reputation_score = 0.8;

    // No direct interaction between A and B
    // But there's an attestation
    let attestations = vec![Attestation {
        id: 0,
        attestor_id: 0,
        subject_id: 1,
        claim_type: ClaimType::Cooperation,
        value: 0.8,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        tick_created: 10,
        tick_expires: None,
        stake: 0.2,
        attestation_hash: None,
        anchor_tx_id: None,
        anchor_block: None,
        anchor_status: AnchorStatus::None,
    }];

    let mut agents = vec![agent_a, agent_b, agent_c];
    let config = ReputationPropagationConfig::default();

    let updates = propagate_reputation_network(&mut agents, &attestations, &config);

    // Should propagate through attestation network
    assert!(updates > 0);
}

// =================================================================
// Institution Evolution Tests (§6.18)
// Selection-Amplification-Variation Dynamics
// =================================================================

#[test]
fn test_institution_with_extended_fields() {
    let inst = Institution::new_with_name(
        1,
        "TestCoop".to_string(),
        RuleType::ResourceSharing,
        Some(5),
        100,
    );

    assert_eq!(inst.id, 1);
    assert_eq!(inst.name, "TestCoop");
    assert_eq!(inst.rule_type, RuleType::ResourceSharing);
    assert_eq!(inst.group_id, Some(5));

    // Extended fields
    assert_eq!(inst.member_count, 0);
    assert_eq!(inst.total_resources_accumulated, 0.0);
    assert_eq!(inst.successful_cooperations, 0);
    assert_eq!(inst.failed_cooperations, 0);
    assert!((inst.efficiency_score - 0.5).abs() < 0.01);
    assert!((inst.mutation_rate - 0.05).abs() < 0.01);
    assert!(inst.parent_institution.is_none());
}

#[test]
// TAG: surface=simulation owner=research-team rule=SIM-001
fn test_institution_record_cooperation() {
    let mut inst = Institution::new(1, RuleType::TradeRules, None, 100);

    // Record successful cooperation
    inst.record_cooperation(true, 50.0);
    assert_eq!(inst.successful_cooperations, 1);
    assert_eq!(inst.failed_cooperations, 0);
    assert_eq!(inst.total_resources_accumulated, 50.0);

    // Record failed cooperation
    inst.record_cooperation(false, 0.0);
    assert_eq!(inst.successful_cooperations, 1);
    assert_eq!(inst.failed_cooperations, 1);

    // Efficiency should be computed
    inst.compute_efficiency();
    assert!(inst.efficiency_score > 0.0 && inst.efficiency_score <= 1.0);
}

#[test]
fn test_institution_create_variant() {
    let parent = Institution::new_with_name(
        1,
        "ParentInst".to_string(),
        RuleType::ResourceSharing,
        Some(5),
        100,
    );

    let variant = parent.create_variant(2, 150, None);

    assert_eq!(variant.id, 2);
    assert_eq!(variant.parent_institution, Some(1));
    assert!(variant.name.contains("Variant"));
    assert_eq!(variant.rule_type, parent.rule_type);
    assert!(variant.fitness < parent.fitness); // Reduced fitness
}

#[test]
fn test_institution_apply_mutation() {
    let mut inst = Institution::new(1, RuleType::ResourceSharing, None, 100);

    // Test name change
    inst.apply_mutation(InstitutionMutation::NameChange {
        new_name: "NewName".to_string(),
    });
    assert_eq!(inst.name, "NewName");

    // Test membership criteria change
    inst.apply_mutation(InstitutionMutation::MembershipCriteriaChange { new_criteria: 0.8 });
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    assert!((inst.membership_criteria - 0.8).abs() < 0.01);

    // Test governance change
    inst.apply_mutation(InstitutionMutation::GovernanceStructureChange {
        new_structure: GovernanceType::Meritocratic,
    });
    assert_eq!(inst.governance_type, GovernanceType::Meritocratic);
}

#[test]
fn test_institution_selection_score() {
    let mut inst = Institution::new(1, RuleType::ResourceSharing, None, 100);

    // Set up performance data
    inst.successful_cooperations = 80;
    inst.failed_cooperations = 20;
    inst.total_resources_accumulated = 5000.0;
    inst.member_count = 50;
    inst.compute_efficiency();

    // Test efficiency-based selection
    let eff_score = inst.selection_score(SelectionCriterion::EfficiencyBased);
    assert!((0.0..=1.0).contains(&eff_score));

    // Test resource-based selection
    let res_score = inst.selection_score(SelectionCriterion::ResourceBased);
    assert!((0.0..=1.0).contains(&res_score));

    // Test growth-based selection
    let growth_score = inst.selection_score(SelectionCriterion::GrowthBased);
    assert!((0.0..=1.0).contains(&growth_score));

    // Test combined selection
    let combined_score = inst.selection_score(SelectionCriterion::Combined);
    assert!((0.0..=1.0).contains(&combined_score));
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_selection_amplification_engine_default() {
    let engine = SelectionAmplificationEngine::default();

    assert!(engine.selection_pressure >= 0.0 && engine.selection_pressure <= 1.0);
    assert!(engine.amplification_rate >= 0.0 && engine.amplification_rate <= 1.0);
    assert!(engine.mutation_rate >= 0.0 && engine.mutation_rate <= 1.0);
    assert_eq!(engine.selection_criterion, SelectionCriterion::Combined);
}

#[test]
fn test_selection_amplification_engine_new() {
    let engine =
        SelectionAmplificationEngine::new(0.5, 0.3, 0.1, SelectionCriterion::EfficiencyBased);

    assert!((engine.selection_pressure - 0.5).abs() < 0.01);
    assert!((engine.amplification_rate - 0.3).abs() < 0.01);
    assert!((engine.mutation_rate - 0.1).abs() < 0.01);
    assert_eq!(
        engine.selection_criterion,
        SelectionCriterion::EfficiencyBased
    );
}

#[test]
fn test_selection_phase() {
    let mut memory = InstitutionalMemory::default();

    // Add institutions with varying fitness
    let id1 = memory.add(RuleType::ResourceSharing, None, 100);
    let id2 = memory.add(RuleType::TradeRules, None, 100);
    let id3 = memory.add(RuleType::DecisionMaking, None, 100);

    // Set different efficiency scores
    if let Some(inst) = memory.get_mut(id1) {
        inst.efficiency_score = 0.8; // High - should be selected
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    if let Some(inst) = memory.get_mut(id2) {
        inst.efficiency_score = 0.4; // Low - should not be selected
    }
    if let Some(inst) = memory.get_mut(id3) {
        inst.efficiency_score = 0.9; // Very high - should be selected
    }

    let engine = SelectionAmplificationEngine::new(
        0.5,
        0.0, // No amplification for this test
        0.0,
        SelectionCriterion::EfficiencyBased,
    );
    memory.evolution_engine = engine;

    // Run selection
    let selected = memory
        .evolution_engine
        .select_institutions(&memory.institutions);

    // Should select institutions with efficiency >= 0.6
    assert!(selected.contains(&id1));
    assert!(!selected.contains(&id2));
    assert!(selected.contains(&id3));
}

#[test]
fn test_amplification_phase() {
    let mut rng = StdRng::seed_from_u64(42);

    // Create parent institution
    let parent = Institution::new_with_name(
        1,
        "Parent".to_string(),
        RuleType::ResourceSharing,
        None,
        100,
    );

    let engine = SelectionAmplificationEngine::new(
        0.5,
        1.0, // Always amplify
        0.0, // No mutation
        SelectionCriterion::EfficiencyBased,
    );

    // Run amplification
    let parent_institutions = vec![parent];
    let new_institutions = engine.create_variants(&parent_institutions, 2, 200, &mut rng);
    // TAG: surface=simulation owner=research-team rule=SIM-001

    // Should create a variant
    assert!(!new_institutions.is_empty());
    assert_eq!(new_institutions.len(), 1);

    // Check lineage
    let variant = &new_institutions[0];
    assert_eq!(variant.parent_institution, Some(1));
    assert_eq!(variant.id, 2);
}

#[test]
fn test_evolve_institutions_cycle() {
    let mut rng = StdRng::seed_from_u64(42);

    // Configure engine for efficiency-based selection
    let engine = SelectionAmplificationEngine::new(
        0.5,
        1.0, // Always amplify when selected
        0.0, // No mutation
        SelectionCriterion::EfficiencyBased,
    );
    let mut memory = InstitutionalMemory {
        evolution_engine: engine,
        ..InstitutionalMemory::default()
    };
    memory.evolution_engine.selection_threshold = 0.6;

    // Create parent institution with high fitness
    let parent_id = memory.add(RuleType::ResourceSharing, None, 100);
    if let Some(inst) = memory.get_mut(parent_id) {
        inst.efficiency_score = 0.9; // High - should be selected
        inst.fitness = 0.9;
        inst.salience = 0.9;
        inst.successful_cooperations = 90;
        inst.failed_cooperations = 10;
    }

    // Run evolution cycle
    let result = evolve_institutions(&mut memory, 200, 0.01, &mut rng);

    // Check results
    assert!(
        result.selected_count > 0,
        "Expected at least one institution to be selected"
    );
    assert!(result.new_institutions > 0 || result.extinct_count > 0);

    // Memory should have recorded the cycle
    assert_eq!(memory.evolution_history.len(), 1);
    // TAG: surface=simulation owner=research-team rule=SIM-001
}

#[test]
fn test_institutional_memory_diversity_metrics() {
    let mut memory = InstitutionalMemory::default();

    // Add institutions of different types
    let _ = memory.add(RuleType::ResourceSharing, None, 100);
    let _ = memory.add(RuleType::ResourceSharing, None, 100);
    let _ = memory.add(RuleType::TradeRules, None, 100);
    let _ = memory.add(RuleType::DecisionMaking, None, 100);
    let _ = memory.add(RuleType::ConflictResolution, None, 100);

    let rule_counts = memory.count_by_rule_type();

    assert_eq!(rule_counts.get(&RuleType::ResourceSharing), Some(&2));
    assert_eq!(rule_counts.get(&RuleType::TradeRules), Some(&1));
    assert_eq!(rule_counts.get(&RuleType::DecisionMaking), Some(&1));
    assert_eq!(rule_counts.get(&RuleType::ConflictResolution), Some(&1));
}

#[test]
fn test_compute_institutional_metrics_extended() {
    let mut memory = InstitutionalMemory::default();

    // Add institutions with different properties
    let id1 = memory.add(RuleType::ResourceSharing, None, 100);
    let id2 = memory.add(RuleType::TradeRules, None, 150);
    let id3 = memory.add(RuleType::DecisionMaking, None, 200);

    // Set extended properties
    if let Some(inst) = memory.get_mut(id1) {
        inst.efficiency_score = 0.8;
        inst.member_count = 20;
        inst.governance_type = GovernanceType::Hierarchical;
    }
    if let Some(inst) = memory.get_mut(id2) {
        inst.efficiency_score = 0.6;
        inst.member_count = 15;
        inst.governance_type = GovernanceType::Egalitarian;
        // TAG: surface=simulation owner=research-team rule=SIM-001
    }
    if let Some(inst) = memory.get_mut(id3) {
        inst.efficiency_score = 0.9;
        inst.member_count = 25;
        inst.governance_type = GovernanceType::Meritocratic;
        inst.parent_institution = Some(999); // Mark as variant
    }

    let metrics = compute_institutional_metrics(&memory, 300);

    // Basic metrics
    assert_eq!(metrics.active_count, 3);
    assert!(metrics.mean_age > 0.0);
    assert!(metrics.mean_fitness > 0.0);

    // Extended metrics
    assert_eq!(metrics.institution_diversity, 3); // 3 rule types
    assert!(metrics.institution_concentration > 0.0 && metrics.institution_concentration <= 1.0);
    assert!(metrics.mean_efficiency_score > 0.0);
    assert!(metrics.mean_member_count > 0.0);
    assert_eq!(metrics.total_members, 60);
    assert_eq!(metrics.root_institution_count, 2); // id1, id2
    assert_eq!(metrics.variant_count, 1); // id3

    // Distribution checks
    assert_eq!(
        metrics
            .governance_distribution
            .get(&GovernanceType::Hierarchical),
        Some(&1)
    );
    assert_eq!(
        metrics
            .governance_distribution
            .get(&GovernanceType::Egalitarian),
        Some(&1)
    );
    assert_eq!(
        metrics
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            .governance_distribution
            .get(&GovernanceType::Meritocratic),
        Some(&1)
    );
}

#[test]
fn test_compute_evolution_rates() {
    let mut memory = InstitutionalMemory::default();

    // Simulate evolution history
    memory.evolution_history.push(EvolutionCycleResult {
        selected_count: 2,
        new_institutions: 1,
        mutations_applied: 0,
        extinct_count: 0,
    });
    memory.evolution_history.push(EvolutionCycleResult {
        selected_count: 3,
        new_institutions: 2,
        mutations_applied: 1,
        extinct_count: 1,
    });
    memory.evolution_history.push(EvolutionCycleResult {
        selected_count: 4,
        new_institutions: 1,
        mutations_applied: 1,
        extinct_count: 0,
    });

    let rates = compute_evolution_rates(&memory, 10);

    assert_eq!(rates.innovation_rate, 4.0 / 3.0); // (1+2+1)/3
    assert_eq!(rates.extinction_rate, 1.0 / 3.0); // (0+1+0)/3
    assert_eq!(rates.mutation_rate, 2.0 / 3.0); // (0+1+1)/3
    assert_eq!(rates.window_size, 3);
}

#[test]
fn test_lineage_tracking() {
    let mut memory = InstitutionalMemory::default();

    let parent_id = memory.add(RuleType::ResourceSharing, None, 100);
    let child_id = memory.add(RuleType::ResourceSharing, None, 150);
    let grandchild_id = memory.add(RuleType::ResourceSharing, None, 200);

    // Record lineage
    memory.record_lineage(parent_id, child_id);
    memory.record_lineage(child_id, grandchild_id);
    // TAG: surface=simulation owner=research-team rule=SIM-001

    // Check descendants
    let children = memory.get_descendants(parent_id);
    assert!(children.contains(&child_id));

    let grandchildren = memory.get_descendants(child_id);
    assert!(grandchildren.contains(&grandchild_id));
}

#[test]
fn test_institution_niche_capacity() {
    // Test that the engine's niche capacity setting exists and can be configured
    let mut engine =
        SelectionAmplificationEngine::new(0.5, 1.0, 0.0, SelectionCriterion::EfficiencyBased);

    // Verify niche capacity can be set
    engine.max_institutions_per_niche = 3;
    assert_eq!(engine.max_institutions_per_niche, 3);

    // Test variant creation respects the amplification rate (which limits creation)
    let mut rng = StdRng::seed_from_u64(42);
    let parent = Institution::new(1, RuleType::ResourceSharing, None, 100);

    // With 0% amplification rate, no variants should be created
    engine.amplification_rate = 0.0;
    let variants = engine.create_variants(&[parent], 2, 200, &mut rng);
    assert!(variants.is_empty());

    // With 100% amplification rate, variants should be created
    engine.amplification_rate = 1.0;
    let parent2 = Institution::new(1, RuleType::ResourceSharing, None, 100);
    let variants = engine.create_variants(&[parent2], 2, 200, &mut rng);
    assert_eq!(variants.len(), 1);
}

#[test]
fn test_governance_type_variants() {
    let mut inst = Institution::new(1, RuleType::ResourceSharing, None, 100);

    // Test all governance types
    inst.governance_type = GovernanceType::Hierarchical;
    assert_eq!(inst.governance_type, GovernanceType::Hierarchical);

    inst.governance_type = GovernanceType::Egalitarian;
    assert_eq!(inst.governance_type, GovernanceType::Egalitarian);

    inst.governance_type = GovernanceType::Meritocratic;
    assert_eq!(inst.governance_type, GovernanceType::Meritocratic);

    inst.governance_type = GovernanceType::Consensus;
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    assert_eq!(inst.governance_type, GovernanceType::Consensus);
}

#[test]
fn test_mutation_clamping() {
    let mut inst = Institution::new(1, RuleType::ResourceSharing, None, 100);

    // Test membership criteria clamping
    inst.apply_mutation(InstitutionMutation::MembershipCriteriaChange {
        new_criteria: 1.5, // Above 1.0
    });
    assert!(inst.membership_criteria <= 1.0);

    inst.apply_mutation(InstitutionMutation::MembershipCriteriaChange {
        new_criteria: -0.5, // Below 0.0
    });
    assert!(inst.membership_criteria >= 0.0);

    // Test mutation rate clamping
    inst.apply_mutation(InstitutionMutation::MutationRateChange {
        new_rate: 0.8, // Above max
    });
    assert!(inst.mutation_rate <= 0.5);
}

#[test]
fn test_evolution_result_default() {
    let result = EvolutionCycleResult::default();
    assert_eq!(result.selected_count, 0);
    assert_eq!(result.new_institutions, 0);
    assert_eq!(result.mutations_applied, 0);
    assert_eq!(result.extinct_count, 0);
}

#[test]
fn test_efficiency_score_computation() {
    let mut inst = Institution::new(1, RuleType::ResourceSharing, None, 100);

    // Initially no cooperations
    inst.compute_efficiency();
    assert!((inst.efficiency_score - 0.5).abs() < 0.01);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Add some successful cooperations
    for _ in 0..8 {
        inst.record_cooperation(true, 100.0);
    }
    for _ in 0..2 {
        inst.record_cooperation(false, 0.0);
    }

    // 80% success rate should give high efficiency
    assert!(inst.efficiency_score > 0.5);
    assert!(inst.efficiency_score <= 1.0);
}

#[test]
fn test_variant_inherits_properties() {
    let mut parent = Institution::new_with_name(
        1,
        "Parent".to_string(),
        RuleType::ResourceSharing,
        Some(5),
        100,
    );
    parent.membership_criteria = 0.7;
    parent.resource_distribution = 0.3;
    parent.governance_type = GovernanceType::Meritocratic;
    parent.mutation_rate = 0.1;

    let variant = parent.create_variant(2, 150, None);

    // Check inheritance
    assert_eq!(variant.rule_type, parent.rule_type);
    assert_eq!(variant.group_id, parent.group_id);
    assert_eq!(variant.membership_criteria, parent.membership_criteria);
    assert_eq!(variant.resource_distribution, parent.resource_distribution);
    assert_eq!(variant.governance_type, parent.governance_type);
    assert_eq!(variant.mutation_rate, parent.mutation_rate);

    // Check modifications
    assert!(variant.fitness < parent.fitness);
    assert!(variant.salience < parent.salience);
}
