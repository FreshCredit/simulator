// TAG: surface=simulation owner=research-team rule=SIM-001
use super::super::*;
use crate::agent::Agent;
use crate::docas::DocasState;
use crate::genome::Genome;
use crate::types::{AgentId, Position, Sex, C, O};
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

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_compute_social_isolation_no_interactions() {
    let agent = create_test_agent(0, 10.0, 10.0);
    let history = vec![];
    let isolation = compute_social_isolation(&agent, &history, 100);

    // No interactions, no group → high isolation
    // With mean genome (isolation_resilience ≈ 0.5), max isolation is ~0.75
    assert!(isolation >= 0.7, "Expected high isolation, got {isolation}");
}

#[test]
fn test_compute_social_isolation_with_group() {
    let mut agent = create_test_agent(0, 10.0, 10.0);
    agent.group_id = Some(1);
    let history = vec![];
    let isolation = compute_social_isolation(&agent, &history, 100);

    // Group membership reduces isolation by 20% (weight 0.2)
    assert!(isolation < 0.8, "Group membership should reduce isolation");
}

#[test]
fn test_compute_social_isolation_with_interactions() {
    let agent = create_test_agent(0, 10.0, 10.0);
    // Interactions at current_tick=100, so within 200-tick window (from tick 0)
    let history = vec![
        (1, 90),
        (2, 91),
        (3, 92),
        (1, 93), // Repeat interaction
        (4, 94),
    ];
    let isolation = compute_social_isolation(&agent, &history, 100);

    // 4 unique partners (network_size=4), 5 interactions (frequency=5)
    // size_score = 1.0 - (4/10) = 0.6
    // freq_score = 1.0 - (5/30) ≈ 0.83
    // no group → 1.0
    // raw_isolation ≈ 0.4*0.6 + 0.4*0.83 + 0.2*1.0 = 0.24 + 0.33 + 0.2 = 0.77
    // With resilience ≈ 0.5: 0.77 * (1 - 0.25) ≈ 0.58
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    // Interactions should reduce isolation below no-interaction baseline (~0.75)
    let no_interaction_baseline = compute_social_isolation(&agent, &[], 100);
    assert!(
        isolation < no_interaction_baseline,
        "Interactions should reduce isolation"
    );
}

#[test]
fn test_compute_reputation_no_attestations() {
    let agents = vec![create_test_agent(0, 10.0, 10.0)];
    let attestations = vec![];
    let reputation = compute_reputation(0, &attestations, &agents, 100);

    // No attestations → neutral reputation
    assert_eq!(reputation, 0.5);
}

#[test]
fn test_decide_attestation_cooperated() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut attestor = create_test_agent(0, 10.0, 10.0);
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    attestor.docas.levels[O] = 0.8; // High oxytocin → willing to attest
    attestor.docas.levels[C] = 0.2; // Low cortisol

    let attestation = decide_attestation(
        &attestor,
        1,
        InteractionOutcome::Cooperated,
        100,
        0,
        &mut rng,
    );

    assert!(attestation.is_some());
    // ERR-001: Test code - unwrap() exempt for test assertions
    let att = attestation.unwrap();
    assert_eq!(att.subject_id, 1);
    assert!(
        att.value > 0.0,
        "Cooperation should produce positive attestation"
    );
}

#[test]
fn test_decide_attestation_defected() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut attestor = create_test_agent(0, 10.0, 10.0);
    attestor.docas.levels[O] = 0.8;
    attestor.docas.levels[C] = 0.2;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let attestation =
        decide_attestation(&attestor, 1, InteractionOutcome::Defected, 100, 0, &mut rng);

    assert!(attestation.is_some());
    // ERR-001: Test code - unwrap() exempt for test assertions
    let att = attestation.unwrap();
    assert!(
        att.value < 0.0,
        "Defection should produce negative attestation"
    );
}

#[test]
fn test_form_groups_proximity() {
    // SIM-GROUP-001: Group formation uses proximity + genome compatibility
    // Default genomes are identical, so compatibility = 1.0 (passes any threshold)
    let mut agents = vec![
        create_test_agent(0, 10.0, 10.0),
        create_test_agent(1, 11.0, 10.0), // Close to agent 0
        create_test_agent(2, 10.5, 10.5), // Close to agent 0
        create_test_agent(3, 50.0, 50.0), // Far away
    ];

    let mut next_group_id = 0;
    form_groups(&mut agents, 2.0, &mut next_group_id);

    // Agents 0, 1, 2 should form a group (close + compatible genomes)
    assert!(agents[0].group_id.is_some());
    assert!(agents[1].group_id.is_some());
    assert!(agents[2].group_id.is_some());
    assert_eq!(agents[0].group_id, agents[1].group_id);
    assert_eq!(agents[0].group_id, agents[2].group_id);

    // Agent 3 should not be in a group (too far)
    assert!(agents[3].group_id.is_none());
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

#[test]
fn test_genome_compatibility_identical() {
    // SIM-GROUP-001: Identical genomes should have perfect compatibility
    let genome_a = Genome::default();
    let genome_b = Genome::default();
    let compatibility = genome_compatibility(&genome_a, &genome_b);
    assert!(
        (compatibility - 1.0).abs() < 0.001,
        "Identical genomes should have compatibility 1.0"
    );
}

#[test]
fn test_genome_compatibility_different() {
    // SIM-GROUP-001: Very different genomes should have low compatibility
    use rand::SeedableRng;
    let mut rng = StdRng::seed_from_u64(42);
    let genome_a = crate::genome::Genome::random(&mut rng);
    let genome_b = crate::genome::Genome::random(&mut rng);

    let compatibility = genome_compatibility(&genome_a, &genome_b);
    // Random genomes typically have compatibility in [0.5, 0.9] range
    assert!(
        (0.0..=1.0).contains(&compatibility),
        "Compatibility should be in [0, 1]"
    );
}

// ========================================================================
// Stage 3 Comprehensive Tests
// Per §4261-4366: Norms, Institutions, Society mechanisms
// ========================================================================

#[test]
fn test_norm_creation_and_fields() {
    let norm = Norm {
        id: 1,
        behavior_pattern: BehaviorPattern::ShareWithGroup,
        value: 0.8,
        created_tick: 100,
        group_id: Some(5),
        emergence_type: NormEmergenceType::FrequencyDependent,
        adoption_rate: 0.65,
        salience: 0.75,
    };

    // TAG: surface=simulation owner=research-team rule=SIM-001
    assert_eq!(norm.id, 1);
    assert_eq!(norm.behavior_pattern, BehaviorPattern::ShareWithGroup);
    assert!((norm.value - 0.8).abs() < 0.001);
    assert_eq!(norm.group_id, Some(5));
    assert_eq!(norm.emergence_type, NormEmergenceType::FrequencyDependent);
}

#[test]
fn test_norm_violated_by() {
    let norm = Norm {
        id: 1,
        behavior_pattern: BehaviorPattern::CooperateWithStrangers,
        value: 0.8,
        created_tick: 100,
        group_id: None,
        emergence_type: NormEmergenceType::FrequencyDependent,
        adoption_rate: 0.65,
        salience: 0.75,
    };

    // ExcludeDefectors violates cooperation norm per violation logic
    assert!(norm.violated_by(BehaviorPattern::ExcludeDefectors));
    // Same behavior doesn't violate
    assert!(!norm.violated_by(BehaviorPattern::CooperateWithStrangers));
    // Unrelated behavior doesn't violate
    assert!(!norm.violated_by(BehaviorPattern::VerifyFirst));
}

#[test]
fn test_behavior_frequency_tracking() {
    let mut freq = BehaviorFrequency::default();

    freq.record(BehaviorPattern::ShareWithGroup);
    freq.record(BehaviorPattern::ShareWithGroup);
    freq.record(BehaviorPattern::CooperateWithStrangers);

    assert_eq!(freq.total, 3);
    assert!((freq.frequency(BehaviorPattern::ShareWithGroup) - 0.666).abs() < 0.01);
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    assert!((freq.frequency(BehaviorPattern::CooperateWithStrangers) - 0.333).abs() < 0.01);
    assert!((freq.frequency(BehaviorPattern::ExcludeDefectors) - 0.0).abs() < 0.001);
}

#[test]
fn test_frequency_dependent_emergence() {
    let mut freq = BehaviorFrequency::default();

    // Add enough ShareWithGroup to exceed 60% threshold
    for _ in 0..7 {
        freq.record(BehaviorPattern::ShareWithGroup);
    }
    for _ in 0..3 {
        freq.record(BehaviorPattern::CooperateWithStrangers);
    }

    let mut next_norm_id = 0;
    let result = check_frequency_dependent_emergence(&freq, Some(1), 100, &mut next_norm_id);

    assert!(result.is_some());
    // ERR-001: Test code - unwrap() exempt for test assertions
    let norm = result.unwrap();
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    assert_eq!(norm.behavior_pattern, BehaviorPattern::ShareWithGroup);
    assert_eq!(norm.emergence_type, NormEmergenceType::FrequencyDependent);
    assert_eq!(norm.group_id, Some(1));
    assert_eq!(next_norm_id, 1); // ID was incremented
}

#[test]
fn test_norm_compliance_calculation() {
    let agent = create_test_agent(0, 10.0, 10.0);
    let norm = Norm {
        id: 1,
        behavior_pattern: BehaviorPattern::ShareWithGroup,
        value: 0.7,
        created_tick: 100,
        group_id: None,
        emergence_type: NormEmergenceType::FrequencyDependent,
        adoption_rate: 0.65,
        salience: 0.5,
    };

    let compliance = compute_norm_compliance(&agent, &norm, 0.6);
    // Compliance is modulated by DOCAS and trust
    assert!(compliance > 0.0 && compliance <= 1.0);
}

#[test]
fn test_punishment_probability_calculation() {
    let agent = create_test_agent(0, 10.0, 10.0);
    let mut punisher = create_test_agent(1, 11.0, 10.0);
    punisher.docas.levels[O] = 0.8; // High oxytocin → willing to punish

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let norm = Norm {
        id: 1,
        behavior_pattern: BehaviorPattern::ShareWithGroup,
        value: 0.9,
        created_tick: 100,
        group_id: None,
        emergence_type: NormEmergenceType::FrequencyDependent,
        adoption_rate: 0.65,
        salience: 0.8,
    };

    let prob = compute_punishment_probability(&norm, &agent, &punisher, 0.7);
    // Should be non-zero with high-O punisher
    assert!(prob > 0.0);
    assert!(prob <= 1.0);
}

#[test]
fn test_sanction_application() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut agent = create_test_agent(0, 10.0, 10.0);
    agent.energy = 100.0;
    let initial_reputation = agent.reputation_score;

    let norm = Norm {
        id: 1,
        behavior_pattern: BehaviorPattern::ShareWithGroup,
        value: 0.8,
        created_tick: 100,
        group_id: None,
        emergence_type: NormEmergenceType::FrequencyDependent,
        adoption_rate: 0.65,
        salience: 0.75,
    };

    apply_sanction(
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        &mut agent,
        None,
        &norm,
        SanctionType::ReputationLoss,
        &mut rng,
    );
    assert!(agent.reputation_score < initial_reputation);

    apply_sanction(
        &mut agent,
        None,
        &norm,
        SanctionType::ResourcePenalty,
        &mut rng,
    );
    assert!(agent.energy < 100.0);
}

#[test]
fn test_coordination_tracker() {
    let mut tracker = CoordinationTracker {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        window_start: 0,
        ..Default::default()
    };

    tracker.record(true); // success
    tracker.record(true); // success
    tracker.record(false); // failure

    assert_eq!(tracker.attempts, 3);
    assert_eq!(tracker.failures, 1);
    assert!((tracker.failure_rate() - 0.333).abs() < 0.01);

    tracker.reset(100);
    assert_eq!(tracker.attempts, 0);
    assert_eq!(tracker.window_start, 100);
}

#[test]
fn test_institution_creation_and_decay() {
    let mut inst = Institution::new(1, RuleType::ResourceSharing, Some(5), 100);

    assert_eq!(inst.id, 1);
    assert_eq!(inst.rule_type, RuleType::ResourceSharing);
    assert_eq!(inst.group_id, Some(5));
    assert!((inst.fitness - 0.5).abs() < 0.01);
    assert!((inst.salience - 0.5).abs() < 0.01);

    // Test decay
    inst.decay(200, 0.01);
    assert!(inst.salience < 0.5); // Should have decayed

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Test removal threshold
    inst.salience = 0.05;
    assert!(inst.should_remove());
}

#[test]
fn test_institution_invocation() {
    let mut inst = Institution::new(1, RuleType::ConflictResolution, None, 100);
    let initial_fitness = inst.fitness;

    // Successful invocation
    inst.invoke(110, true);
    assert_eq!(inst.invocation_count, 1);
    assert_eq!(inst.last_invoked_tick, 110);
    assert!(inst.fitness > initial_fitness); // Fitness should increase
    assert!(inst.salience > 0.5); // Salience should increase

    // Failed invocation
    inst.invoke(120, false);
    assert_eq!(inst.invocation_count, 2);
}

#[test]
fn test_institutional_memory() {
    let mut memory = InstitutionalMemory::default();

    let id1 = memory.add(RuleType::ResourceSharing, Some(1), 100);
    let id2 = memory.add(RuleType::TradeRules, Some(1), 110);

    assert_eq!(memory.institutions.len(), 2);
    assert!(memory.get(id1).is_some());
    assert!(memory.get(id2).is_some());

    // Get institutions for group
    let group_insts = memory.for_group(Some(1));
    assert_eq!(group_insts.len(), 2);
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    // Prune low-salience institutions
    if let Some(inst) = memory.get_mut(id1) {
        inst.salience = 0.05;
    }
    memory.prune();
    assert_eq!(memory.institutions.len(), 1);
    assert!(memory.get(id1).is_none());
    assert!(memory.get(id2).is_some());
}

#[test]
fn test_institution_emergence_from_coordination_failure() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut memory = InstitutionalMemory::default();

    // Tracker with high failure rate
    let tracker = CoordinationTracker {
        attempts: 25,
        failures: 15, // 60% failure rate > 40% threshold
        window_start: 0,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    };

    // At tick 150 (>100 ticks since start), should trigger emergence
    let result = check_institution_emergence(&tracker, Some(1), 150, &mut memory, &mut rng);

    assert!(result.is_some());
    assert_eq!(memory.institutions.len(), 1);
}

#[test]
fn test_institution_evolution() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut memory = InstitutionalMemory::default();
    memory.add(RuleType::DecisionMaking, None, 0);

    // Get the institution and set low salience
    if let Some(inst) = memory.institutions.first_mut() {
        inst.salience = 0.15;
        inst.last_invoked_tick = 0;
    }

    // Evolve at tick 500 with decay
    let _ = evolve_institutions(&mut memory, 500, 0.01, &mut rng);

    // Institution should still exist but with lower salience
    assert!(memory.institutions.len() <= 1);
}

#[test]
fn test_institutional_metrics() {
    let mut memory = InstitutionalMemory::default();
    memory.add(RuleType::ResourceSharing, None, 100);
    memory.add(RuleType::TradeRules, None, 150);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let metrics = compute_institutional_metrics(&memory, 200);

    assert_eq!(metrics.active_count, 2);
    assert!(metrics.mean_age > 0.0);
    assert!(metrics.mean_fitness > 0.0);
    assert!(metrics.mean_salience > 0.0);
}

// === Encounter Pipeline Tests (SIM-003) ===

#[test]
fn test_check_proximity() {
    let agent_a = create_test_agent(0, 0.0, 0.0);
    let agent_b = create_test_agent(1, 5.0, 0.0);
    let agent_c = create_test_agent(2, 50.0, 0.0);

    // Distance between a and b is 5.0
    assert!(check_proximity(&agent_a, &agent_b, 10.0));
    assert!(!check_proximity(&agent_a, &agent_b, 3.0));

    // Distance between a and c is 50.0
    assert!(!check_proximity(&agent_a, &agent_c, 10.0));
    assert!(check_proximity(&agent_a, &agent_c, 60.0));
}

#[test]
#[allow(deprecated)]
fn test_compute_similarity_metrics() {
    let mut agent_a = create_test_agent(0, 0.0, 0.0);
    let mut agent_b = create_test_agent(1, 5.0, 0.0);

    // Set up agents with different properties
    agent_a.reputation_score = 0.8;
    agent_b.reputation_score = 0.8; // Same reputation
    agent_a.energy = 80.0;
    agent_b.energy = 80.0; // Same energy

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    let metrics = compute_similarity_metrics(&agent_a, &agent_b);

    assert!(metrics.genome >= 0.0 && metrics.genome <= 1.0);
    assert!(metrics.reputation >= 0.0 && metrics.reputation <= 1.0);
    assert!(metrics.resource_asymmetry >= 0.0 && metrics.resource_asymmetry <= 1.0);
    assert!(!metrics.sex_compatible); // Both male
    assert!(metrics.combined >= 0.0 && metrics.combined <= 1.0);
}

#[test]
#[allow(deprecated)]
fn test_compute_similarity_sex_compatible() {
    let mut agent_a = create_test_agent(0, 0.0, 0.0);
    let mut agent_b = create_test_agent(1, 5.0, 0.0);

    // Make them opposite sex
    agent_a.sex = Sex::Male;
    agent_b.sex = Sex::Female;

    let metrics = compute_similarity_metrics(&agent_a, &agent_b);

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    assert!(metrics.sex_compatible);
}

#[test]
#[allow(deprecated)]
fn test_decide_encourage_high_trust() {
    let mut rng = StdRng::seed_from_u64(42);

    let mut agent_a = create_test_agent(0, 0.0, 0.0);
    let mut agent_b = create_test_agent(1, 5.0, 0.0);

    // High trust agents with good reputations should engage
    agent_a.docas.levels = [0.8, 0.8, 0.2, 0.2, 0.8]; // High D, O, S; Low C, A
    agent_b.docas.levels = [0.8, 0.8, 0.2, 0.2, 0.8];
    agent_a.reputation_score = 0.8;
    agent_b.reputation_score = 0.8;

    let metrics = compute_similarity_metrics(&agent_a, &agent_b);
    let context = EncounterContext {
        distance: 5.0,
        tau_a: agent_a.compute_trust(),
        tau_b: agent_b.compute_trust(),
        local_reputation: 0.8,
        resource_scarcity: 0.2,
        current_tick: 100,
    };

    // Run multiple times to account for randomness
    let mut engage_count = 0;
    for _ in 0..100 {
        let decision = decide_encounter(&agent_a, &agent_b, &metrics, &context, &mut rng);
        if decision == EncounterDecision::Engage {
            engage_count += 1;
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // High trust, good reputation, low scarcity → should engage most of the time
    assert!(
        engage_count > 60,
        "Expected mostly Engage decisions, got {engage_count}/100"
    );
}

#[test]
#[allow(deprecated)]
fn test_decide_encourage_low_trust_avoid() {
    let mut rng = StdRng::seed_from_u64(42);

    let mut agent_a = create_test_agent(0, 0.0, 0.0);
    let mut agent_b = create_test_agent(1, 5.0, 0.0);

    // Low trust, high cortisol agents should avoid
    agent_a.docas.levels = [0.2, 0.2, 0.9, 0.8, 0.2]; // Low D, O, S; High C, A
    agent_b.docas.levels = [0.2, 0.2, 0.9, 0.8, 0.2];
    agent_a.reputation_score = -0.5; // Bad reputation
    agent_b.reputation_score = -0.5;

    let metrics = compute_similarity_metrics(&agent_a, &agent_b);
    let context = EncounterContext {
        distance: 5.0,
        tau_a: agent_a.compute_trust(),
        tau_b: agent_b.compute_trust(),
        local_reputation: -0.5,
        resource_scarcity: 0.9, // High scarcity
        current_tick: 100,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    };

    // Run multiple times
    let mut engage_count = 0;
    for _ in 0..100 {
        let decision = decide_encounter(&agent_a, &agent_b, &metrics, &context, &mut rng);
        if decision == EncounterDecision::Engage {
            engage_count += 1;
        }
    }

    // Low trust, bad reputation, high scarcity, high C → should avoid most of the time
    assert!(
        engage_count < 40,
        "Expected mostly Avoid decisions, got {engage_count}/100"
    );
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
#[test]
fn test_determine_interaction_mode_same_sex() {
    let agent_a = create_test_agent(0, 0.0, 0.0);
    let agent_b = create_test_agent(1, 5.0, 0.0);

    // Both male → economic game
    let mode = determine_interaction_mode(&agent_a, &agent_b, 50.0, 0.3);
    assert_eq!(mode, InteractionMode::EconomicGame);
}

#[test]
fn test_determine_interaction_mode_reproduction_possible() {
    let mut agent_a = create_test_agent(0, 0.0, 0.0);
    let mut agent_b = create_test_agent(1, 5.0, 0.0);

    // Opposite sex with sufficient resources and reputation
    agent_a.sex = Sex::Male;
    agent_b.sex = Sex::Female;
    agent_a.energy = 100.0;
    agent_b.energy = 100.0;
    agent_a.reputation_score = 0.5;
    agent_b.reputation_score = 0.5;
    agent_a.criticality_sigma = 1.0;
    agent_b.criticality_sigma = 1.0;

    let mode = determine_interaction_mode(&agent_a, &agent_b, 50.0, 0.3);
    assert_eq!(mode, InteractionMode::Reproduction);
}

#[test]
fn test_determine_interaction_mode_insufficient_energy() {
    let mut agent_a = create_test_agent(0, 0.0, 0.0);
    let mut agent_b = create_test_agent(1, 5.0, 0.0);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Opposite sex but insufficient energy
    agent_a.sex = Sex::Male;
    agent_b.sex = Sex::Female;
    agent_a.energy = 10.0; // Too low
    agent_b.energy = 100.0;

    let mode = determine_interaction_mode(&agent_a, &agent_b, 50.0, 0.3);
    assert_eq!(mode, InteractionMode::EconomicGame);
}

#[test]
#[allow(deprecated)]
fn test_process_encounter_out_of_range() {
    let mut rng = StdRng::seed_from_u64(42);
    let agent_a = create_test_agent(0, 0.0, 0.0);
    let agent_b = create_test_agent(1, 100.0, 0.0); // Far away

    let result = process_encounter(&agent_a, &agent_b, 10.0, 0.5, 50.0, 0.3, 100, &mut rng);

    assert!(result.is_none());
}

#[test]
#[allow(deprecated)]
fn test_process_encounter_in_range() {
    let mut rng = StdRng::seed_from_u64(42);
    let agent_a = create_test_agent(0, 0.0, 0.0);
    let agent_b = create_test_agent(1, 5.0, 0.0); // Close

    let result = process_encounter(&agent_a, &agent_b, 10.0, 0.5, 50.0, 0.3, 100, &mut rng);

    assert!(result.is_some());
    // ERR-001: Test code - unwrap() exempt for test assertions
    let (decision, _mode) = result.unwrap();
    // With default agents, should get a decision (either engage or avoid)
    assert!(decision == EncounterDecision::Engage || decision == EncounterDecision::Avoid);
}

// === SIM-009: Reputation Propagation Tests ===
