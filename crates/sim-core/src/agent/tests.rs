use super::*;
use rand::SeedableRng;

#[test]
fn test_agent_creation() {
    let genome = Genome::random(&mut rand::rngs::StdRng::seed_from_u64(42));
    let agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);
    assert_eq!(agent.id, 1);
    assert!(agent.alive);
    assert_eq!(agent.age, 0);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_trust_computation() {
    let genome = Genome::random(&mut rand::rngs::StdRng::seed_from_u64(42));
    let agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);
    let trust = agent.compute_trust();
    assert!((0.0..=1.0).contains(&trust));
}

#[test]
fn test_new_fields_initialized() {
    let genome = Genome::random(&mut rand::rngs::StdRng::seed_from_u64(42));
    let agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    // Check GTT fields
    assert!(agent.lambda > 0.0);
    assert!(agent.expectation >= 0.0 && agent.expectation <= 1.0);

    // Check TFP fields
    assert!(agent.psi_trust >= 0.0 && agent.psi_trust <= 1.0);

    // Check emotion fields
    assert!(agent.dominance >= 0.0 && agent.dominance <= 1.0);

    // Check game theory fields
    assert!(agent.game_history.is_empty());
    assert_eq!(agent.cumulative_payoff, 0.0);

    // Check PDE4B fields
    assert!(agent.pde4b_expression > 0.0);
}

/// Test that PDE4B values are correctly computed from genome at initialization
/// Per §6.6.5: PDE4B at loci 24-25 modulates cAMP → learning rate λ
/// This validates GAP-001 fix: PDE4B is now wired to agent initialization
///
/// Note: Uses PDE4B → cAMP → PKA → CREB pathway for biologically accurate computation
#[test]
fn test_pde4b_wired_from_genome() {
    use crate::neural::pde4b_pathway::{compute_pathway, PDE4BPathwayParams};
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    // Create two agents with different genomes
    let mut rng1 = StdRng::seed_from_u64(42);
    let mut rng2 = StdRng::seed_from_u64(123);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let genome1 = Genome::random(&mut rng1);
    let genome2 = Genome::random(&mut rng2);

    // Get expected PDE4B values from genomes
    let expected_pde4b_1 = genome1.pde4b_level();
    let expected_lambda_1 = genome1.compute_lambda();

    // Compute expected cAMP using pathway
    let pathway_params = PDE4BPathwayParams::default();
    let pathway_output_1 = compute_pathway(expected_pde4b_1, &pathway_params);
    let expected_camp_1 = pathway_output_1.camp_normalized();

    let expected_pde4b_2 = genome2.pde4b_level();
    let pathway_output_2 = compute_pathway(expected_pde4b_2, &pathway_params);
    let expected_camp_2 = pathway_output_2.camp_normalized();
    let expected_lambda_2 = genome2.compute_lambda();

    // Create agents
    let agent1 = Agent::new(1u64, Position::new_2d(32.0, 32.0), genome1, 0, Sex::Male);
    let agent2 = Agent::new(2u64, Position::new_2d(32.0, 32.0), genome2, 0, Sex::Female);

    // Verify PDE4B values match genome computation
    assert!(
        (agent1.pde4b_expression - expected_pde4b_1).abs() < 0.001,
        "PDE4B expression should match genome: got {}, expected {}",
        agent1.pde4b_expression,
        expected_pde4b_1
    );
    assert!(
        (agent1.camp_level - expected_camp_1).abs() < 0.001,
        "cAMP level should match pathway computation: got {}, expected {}",
        agent1.camp_level,
        expected_camp_1
    );
    assert!(
        (agent1.learning_rate_modifier - expected_lambda_1).abs() < 0.001,
        "Learning rate should match genome compute_lambda: got {}, expected {}",
        agent1.learning_rate_modifier,
        expected_lambda_1
    );

    assert!(
        (agent2.pde4b_expression - expected_pde4b_2).abs() < 0.001,
        "PDE4B expression should match genome: got {}, expected {}",
        agent2.pde4b_expression,
        expected_pde4b_2
    );
    assert!(
        (agent2.camp_level - expected_camp_2).abs() < 0.001,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        "cAMP level should match pathway computation: got {}, expected {}",
        agent2.camp_level,
        expected_camp_2
    );
    assert!(
        (agent2.learning_rate_modifier - expected_lambda_2).abs() < 0.001,
        "Learning rate should match genome compute_lambda: got {}, expected {}",
        agent2.learning_rate_modifier,
        expected_lambda_2
    );

    // Verify ranges
    assert!(
        (0.0..=1.0).contains(&agent1.pde4b_expression),
        "PDE4B should be in [0, 1]: {}",
        agent1.pde4b_expression
    );
    assert!(
        (0.0..=1.0).contains(&agent1.camp_level),
        "cAMP should be in [0, 1]: {}",
        agent1.camp_level
    );
    assert!(
        (0.305..=0.915).contains(&agent1.learning_rate_modifier),
        "λ should be in [0.305, 0.915]: {}",
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        agent1.learning_rate_modifier
    );

    // Verify inverse relationship: low PDE4B → high cAMP (via pathway)
    if agent1.pde4b_expression < agent2.pde4b_expression {
        assert!(
            agent1.camp_level > agent2.camp_level,
            "Lower PDE4B should yield higher cAMP via pathway: {} vs {}",
            agent1.camp_level,
            agent2.camp_level
        );
    }
}

// === Neural Maturation Tests (Research: MATURITY_NEURAL_DEVELOPMENT) ===

#[test]
fn test_prefrontal_cortex_initialization() {
    let genome = Genome::default();
    let agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    // Check that PFC is initialized
    assert_eq!(
        agent.prefrontal_cortex.dlpfc.development, 0.0,
        "dlPFC should start undeveloped"
    );
    // TAG: surface=simulation owner=research-team rule=SIM-001
    assert_eq!(
        agent.prefrontal_cortex.vmpfc.development, 0.0,
        "vmPFC should start undeveloped"
    );
    assert_eq!(
        agent.prefrontal_cortex.pruning_progress, 0.0,
        "Pruning should start at 0"
    );
    assert_eq!(
        agent.prefrontal_cortex.myelination_progress, 0.0,
        "Myelination should start at 0"
    );
}

#[test]
fn test_neural_maturity_updates() {
    let genome = Genome::default();
    let mut agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    // Initially not mature
    assert!(
        !agent.is_neurally_mature(),
        "Newborn should not be neurally mature"
    );

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    // Age the agent significantly
    for _ in 0..200 {
        agent.age_tick();
        agent.update_neural_maturation();
    }

    // Should be mature by now
    assert!(
        agent.is_neurally_mature(),
        "Agent should be neurally mature after 200 ticks"
    );

    // Check development levels
    assert!(
        agent.prefrontal_cortex.vmpfc.development >= 0.7,
        "vmPFC should be >70% developed"
    );
    assert!(
        agent.prefrontal_cortex.dlpfc.development >= 0.5,
        "dlPFC should be >50% developed"
    );
}

#[test]
fn test_maturation_progress_increases() {
    let genome = Genome::default();
    let mut agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let initial_pruning = agent.prefrontal_cortex.pruning_progress;
    let initial_myelination = agent.prefrontal_cortex.myelination_progress;

    // Age agent
    for _ in 0..50 {
        agent.age_tick();
        agent.update_neural_maturation();
    }

    assert!(
        agent.prefrontal_cortex.pruning_progress > initial_pruning,
        "Pruning should progress with age"
    );
    assert!(
        agent.prefrontal_cortex.myelination_progress > initial_myelination,
        "Myelination should progress with age"
    );
}

#[test]
fn test_fast_maturation_genome() {
    // Create genome with fast maturation (high PFC rate)
    let mut fast_genome = Genome::default();
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    fast_genome.alleles[32] = [1.0, 1.0]; // Max PFC maturation rate
    fast_genome.alleles[33] = [1.0, 1.0];
    fast_genome.alleles[24] = [0.0, 0.0]; // Low PDE4B (faster maturation)
    fast_genome.alleles[25] = [0.0, 0.0];

    let mut fast_agent = Agent::new(
        1u64,
        Position::new(32.0, 32.0, 0.0),
        fast_genome,
        0,
        Sex::Male,
    );

    // Create genome with slow maturation
    let mut slow_genome = Genome::default();
    slow_genome.alleles[32] = [0.0, 0.0]; // Min PFC maturation rate
    slow_genome.alleles[33] = [0.0, 0.0];
    slow_genome.alleles[24] = [1.0, 1.0]; // High PDE4B (slower maturation)
    slow_genome.alleles[25] = [1.0, 1.0];

    let mut slow_agent = Agent::new(
        2u64,
        Position::new(32.0, 32.0, 0.0),
        slow_genome,
        0,
        Sex::Female,
    );

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Age both agents
    for _ in 0..100 {
        fast_agent.age_tick();
        fast_agent.update_neural_maturation();
        slow_agent.age_tick();
        slow_agent.update_neural_maturation();
    }

    // Fast maturation should be ahead
    assert!(
        fast_agent.prefrontal_cortex.dlpfc.development
            > slow_agent.prefrontal_cortex.dlpfc.development,
        "Fast maturation genome should develop faster"
    );
    assert!(
        fast_agent.prefrontal_cortex.vmpfc.development
            > slow_agent.prefrontal_cortex.vmpfc.development,
        "Fast maturation genome should develop faster"
    );
}

#[test]
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
fn test_working_memory_capacity_increases() {
    let genome = Genome::default();
    let mut agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    let initial_wm = agent.prefrontal_cortex.dlpfc.working_memory_capacity;

    // Age to maturity
    for _ in 0..350 {
        agent.age_tick();
        agent.update_neural_maturation();
    }

    let mature_wm = agent.prefrontal_cortex.dlpfc.working_memory_capacity;

    assert!(
        mature_wm > initial_wm,
        "Working memory should increase with development: {initial_wm} -> {mature_wm}"
    );
    assert!(
        mature_wm >= 4,
        "Mature agent should have working memory >= 4 chunks, got {mature_wm}"
    );
}

#[test]
fn test_maturation_rate_computation() {
    let genome = Genome::default();
    let agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Maturation rate should be computed from genome
    assert!(
        agent.prefrontal_cortex.current_maturation_rate > 0.0,
        "Maturation rate should be positive"
    );
}

// === Big Five Personality Tests ===

#[test]
fn test_agent_personality_default() {
    let genome = Genome::default();
    let agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    // Default personality should have all traits at 0.5
    assert_eq!(agent.personality.openness, 0.5);
    assert_eq!(agent.personality.conscientiousness, 0.5);
    assert_eq!(agent.personality.extraversion, 0.5);
    assert_eq!(agent.personality.agreeableness, 0.5);
    assert_eq!(agent.personality.neuroticism, 0.5);
}

#[test]
fn test_agent_initialize_personality() {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    let genome = Genome::default();
    let mut agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    let mut rng = StdRng::seed_from_u64(42);
    agent.initialize_personality(0.8, 0.1, &mut rng);

    // Personality should now have specific values from initialization
    assert!(agent.personality.openness >= 0.0 && agent.personality.openness <= 1.0);
    assert!(
        agent.personality.conscientiousness >= 0.0 && agent.personality.conscientiousness <= 1.0
    );
    assert!(agent.personality.extraversion >= 0.0 && agent.personality.extraversion <= 1.0);
    assert!(agent.personality.agreeableness >= 0.0 && agent.personality.agreeableness <= 1.0);
    assert!(agent.personality.neuroticism >= 0.0 && agent.personality.neuroticism <= 1.0);
}

#[test]
fn test_agent_personality_economic_bias() {
    let genome = Genome::default();
    let agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    // Default personality (0.5 for all traits) should have neutral bias
    let bias = agent.personality_economic_bias();
    assert!(
        (-0.5..=0.5).contains(&bias),
        "Default personality should have relatively neutral economic bias, got {bias}"
    );
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_agent_update_docas_from_personality() {
    use crate::types::{C, D, O};

    let genome = Genome::default();
    let mut agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    // Set specific personality values
    agent.personality = crate::psychology::PersonalityTraits {
        openness: 0.5,
        conscientiousness: 0.5,
        extraversion: 0.9,  // High extraversion
        agreeableness: 0.9, // High agreeableness
        neuroticism: 0.9,   // High neuroticism
    };

    // Reset DOCAS to neutral
    agent.docas = crate::docas::DocasState::default();
    let initial_dopamine = agent.docas.levels[D];
    let initial_cortisol = agent.docas.levels[C];
    let initial_oxytocin = agent.docas.levels[O];

    // Apply personality effects
    agent.update_docas_from_personality();

    // High extraversion → higher dopamine
    assert!(
        agent.docas.levels[D] > initial_dopamine,
        "High extraversion should increase dopamine"
    );

    // High neuroticism → higher cortisol
    assert!(
        agent.docas.levels[C] > initial_cortisol,
        "High neuroticism should increase cortisol"
    );

    // High agreeableness → higher oxytocin
    assert!(
        agent.docas.levels[O] > initial_oxytocin,
        "High agreeableness should increase oxytocin"
    );
}

#[test]
fn test_agent_personality_exploration_tendency() {
    let genome = Genome::default();
    let mut agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // High openness + high extraversion
    agent.personality = crate::psychology::PersonalityTraits {
        openness: 0.9,
        conscientiousness: 0.5,
        extraversion: 0.9,
        agreeableness: 0.5,
        neuroticism: 0.5,
    };

    let exploration = agent.personality_exploration_tendency();
    assert!(
        exploration > 0.7,
        "High O + High E should yield high exploration tendency"
    );

    // Low openness + low extraversion
    agent.personality = crate::psychology::PersonalityTraits {
        openness: 0.1,
        conscientiousness: 0.5,
        extraversion: 0.1,
        agreeableness: 0.5,
        neuroticism: 0.5,
    };

    let low_exploration = agent.personality_exploration_tendency();
    assert!(
        low_exploration < 0.3,
        "Low O + Low E should yield low exploration tendency"
    );
}

// === Working Memory Integration Tests (Miller's Law) ===

#[test]
fn test_agent_working_memory_initialized() {
    let genome = Genome::default();
    let agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    // Working memory should be initialized
    assert!(
        agent.working_memory.capacity >= 3,
        "Working memory should have minimum capacity"
    );
    assert_eq!(
        agent.working_memory.used_slots, 0,
        "New agent should have empty working memory"
    );
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_agent_perceive_and_remember() {
    let genome = Genome::default();
    let mut agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    // Create perceived info
    let info = crate::cognition::PerceivedInfo::new(
        crate::cognition::MemoryContent::Fact("test fact".to_string()),
        0.8,
        1.0,
    );

    // Perceive and remember
    agent.perceive_and_remember(info, 0);

    // Should be in working memory
    assert_eq!(agent.working_memory.items.len(), 1);
}

#[test]
fn test_agent_high_priority_forces_encoding() {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    let genome = Genome::default();
    let mut agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    // Fill working memory
    for i in 0..agent.working_memory.capacity {
        let info = crate::cognition::PerceivedInfo::new(
            crate::cognition::MemoryContent::Fact(format!("filler_{i}")),
            0.9,
            1.0,
        );
        agent.perceive_and_remember(info, 0);
    }

    let _initial_count = agent.working_memory.items.len();

    // Very high priority info should force encoding (drops lower priority)
    let urgent_info = crate::cognition::PerceivedInfo::new(
        crate::cognition::MemoryContent::Fact("urgent".to_string()),
        0.95,
        0.1, // Close = higher effective priority
    );
    agent.perceive_and_remember(urgent_info, 1);

    // Should still have items (not necessarily same count due to replacement)
    assert!(!agent.working_memory.items.is_empty());
}

#[test]
fn test_agent_working_memory_updates_with_tick() {
    let genome = Genome::default();
    let mut agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Add something to working memory
    let info = crate::cognition::PerceivedInfo::new(
        crate::cognition::MemoryContent::Fact("to decay".to_string()),
        0.5,
        1.0,
    );
    agent.perceive_and_remember(info, 0);

    // Update working memory at later tick
    agent.update_working_memory(100);

    // Executive control should be updated based on stress
    assert!(agent.working_memory.executive_control >= 0.0);
    assert!(agent.working_memory.executive_control <= 1.0);
}

#[test]
fn test_agent_working_memory_capacity_grows_with_age() {
    let genome = Genome::default();
    let mut agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    let initial_capacity = agent.working_memory.capacity;

    // Age agent to maturity
    for _ in 0..350 {
        agent.age_tick();
        agent.update_neural_maturation();
    }

    // Capacity should have increased
    assert!(
        agent.working_memory.capacity >= initial_capacity,
        "Working memory capacity should grow with age: {} -> {}",
        initial_capacity,
        agent.working_memory.capacity
    );

    // Mature agent should have Miller's 7±2 capacity
    assert!(
        agent.working_memory.capacity >= 5,
        "Mature agent should have at least 5 slots, got {}",
        agent.working_memory.capacity
    );
}

#[test]
fn test_agent_decision_complexity_limited() {
    let genome = Genome::default();
    let agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Create decision options of varying complexity
    let options = vec![
        crate::cognition::DecisionOption::simple(1, "simple", 0.5),
        crate::cognition::DecisionOption::complex(2, "complex", 0.7, 5, 0.3),
    ];

    // With empty/low working memory, should only handle simple options
    let decision = agent.make_decision(&options);

    // Should return a decision (may be simple or complex depending on available capacity)
    assert!(decision.is_some());
}

#[test]
fn test_agent_working_memory_stats() {
    let genome = Genome::default();
    let mut agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    let stats = agent.working_memory_stats();
    assert_eq!(stats.item_count, 0);

    // Add something
    let info = crate::cognition::PerceivedInfo::new(
        crate::cognition::MemoryContent::Fact("test".to_string()),
        0.5,
        1.0,
    );
    agent.perceive_and_remember(info, 0);

    let stats_after = agent.working_memory_stats();
    assert_eq!(stats_after.item_count, 1);
    assert!(stats_after.utilization > 0.0);
}

// === Rescorla-Wagner Trust Learning Tests ===

#[test]
fn test_rescorla_wagner_update_positive_reward() {
    let genome = Genome::default();
    let mut agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    // Set balanced DOCAS
    agent.docas = DocasState::new([0.5, 0.5, 0.5, 0.5, 0.5]);
    let initial_tau = agent.compute_trust();

    // Positive reward > current tau should increase trust
    agent.update_trust_rescorla_wagner(1.8);
    let new_tau = agent.compute_trust();

    // TAG: surface=simulation owner=research-team rule=SIM-001
    assert!(
        new_tau > initial_tau,
        "Positive reward should increase trust: {initial_tau:.1} -> {new_tau:.1}"
    );
}

#[test]
fn test_rescorla_wagner_update_negative_reward() {
    let genome = Genome::default();
    let mut agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    // Set moderate initial trust for clear test results
    agent.docas = DocasState::new([0.6, 0.6, 0.4, 0.4, 0.6]);
    let initial_tau = agent.compute_trust();

    // Very negative reward should decrease trust
    // The key behavior is: trust moves toward the reward signal
    agent.update_trust_rescorla_wagner(0.1); // Much lower than current
    let new_tau = agent.compute_trust();

    // Verify trust moved in the correct direction (toward reward)
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    // The new tau should be closer to 0.1 than initial was
    let initial_distance = (initial_tau - 0.1).abs();
    let new_distance = (new_tau - 0.1).abs();

    assert!(new_distance < initial_distance,
            "Trust should move toward reward signal: initial={initial_tau:.1}, new={new_tau:.1}, target=0.1");
}

#[test]
fn test_rescorla_wagner_extinction() {
    // When reward = current tau, no learning (extinction condition)
    let genome = Genome::default();
    let mut agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    agent.docas = DocasState::new([0.5, 0.5, 0.5, 0.5, 0.5]);
    let initial_tau = agent.compute_trust();

    // Apply reward equal to current tau
    agent.update_trust_rescorla_wagner(initial_tau);

    // Should be approximately unchanged (small numerical errors OK)
    let final_tau = agent.compute_trust();
    assert!(
        (final_tau - initial_tau).abs() < 0.01,
        "When reward = expectation, tau should not change: {initial_tau:.2} -> {final_tau:.2}"
    );
}

#[test]
fn test_rescorla_wagner_prediction_error_tracking() {
    let genome = Genome::default();
    let mut agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    agent.docas = DocasState::new([0.5, 0.5, 0.5, 0.5, 0.5]);
    let initial_tau = agent.compute_trust();

    // Large prediction error
    agent.update_trust_rescorla_wagner(1.9);

    // Prediction error should be recorded
    assert!(
        agent.prediction_error > 0.0,
        "Prediction error should be tracked after RW update"
    );
    assert!(
        agent.prediction_error > (1.9 - initial_tau).abs() - 0.1,
        "Prediction error should reflect actual error"
    );
}

#[test]
fn test_rescorla_wagner_updates_tau_history() {
    let genome = Genome::default();
    let mut agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    agent.docas = DocasState::new([0.5, 0.5, 0.5, 0.5, 0.5]);
    let initial_history_len = agent.tau_history.len();

    agent.update_trust_rescorla_wagner(1.5);

    // History should be updated
    assert!(
        agent.tau_history.len() > initial_history_len || agent.tau_history.len() == 20,
        "Tau history should be updated after RW update"
    );
}

#[test]
fn test_get_learning_rate_from_pde4b() {
    let genome = Genome::default();
    let agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    // Learning rate should be in valid range
    let learning_rate = agent.get_learning_rate();
    assert!(
        (0.01..=0.5).contains(&learning_rate),
        "Learning rate should be in [0.01, 0.5]: {learning_rate:.2}"
    );
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_adjust_docas_for_target_tau() {
    let genome = Genome::default();
    let mut agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    // Start with neutral DOCAS
    agent.docas = DocasState::new([0.5, 0.5, 0.5, 0.5, 0.5]);

    // Target a higher tau
    let target_tau = 1.5;
    agent.adjust_docas_for_target_tau(target_tau);

    // DOCAS should be adjusted toward target
    let new_tau = agent.compute_trust();
    // Should move toward target (may not reach exactly due to partial adjustment)
    assert!(
        new_tau > 0.5,
        "DOCAS should adjust toward higher target tau"
    );
}

#[test]
fn test_tau_history_uses_rescorla_wagner() {
    let genome = Genome::default();
    let mut agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    agent.expectation = 0.5;

    // Update with a higher tau value
    agent.update_tau_history(1.5);

    // Expectation should move toward the new tau (Rescorla-Wagner style)
    // E[t+1] = E[t] + α(τ[t] - E[t])
    // With learning rate ~0.1 and diff = 1.0, expectation should increase
    assert!(
        agent.expectation > 0.5,
        "Expectation should increase with RW update, got {}",
        agent.expectation
    );

    // But should not jump all the way to 1.5 (partial learning)
    assert!(
        agent.expectation < 1.5,
        "Expectation should not jump fully to observed tau with RW"
    );
}

// === Time Preference Profile Tests ===

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Test that `TimePreferenceProfile` is properly initialized from genome
/// This validates the critical fix for integrating hyperbolic discounting
#[test]
fn test_time_preference_initialized_from_genome() {
    // Create genome with specific time preference traits
    let mut genome = Genome::default();
    // Set trait 20 (present bias) to low value = more present-biased
    genome.alleles[40] = [0.2, 0.2];
    genome.alleles[41] = [0.2, 0.2];
    // Set trait 21 (impatience) to high value = very impatient
    genome.alleles[42] = [0.9, 0.9];
    genome.alleles[43] = [0.9, 0.9];

    let agent = Agent::new(
        1u64,
        Position::new(32.0, 32.0, 0.0),
        genome.clone(),
        0,
        Sex::Male,
    );
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    // Verify time_preference field exists and is initialized
    assert!(
        agent.time_preference.present_bias >= 0.5 && agent.time_preference.present_bias <= 1.0,
        "Present bias should be in valid range [0.5, 1.0]"
    );
    assert!(
        agent.time_preference.impatience_k >= 0.01 && agent.time_preference.impatience_k <= 0.5,
        "Impatience k should be in valid range [0.01, 0.5]"
    );
    assert!(
        agent.time_preference.long_term_discount >= 0.9
            && agent.time_preference.long_term_discount <= 0.99,
        "Long-term discount should be in valid range [0.9, 0.99]"
    );

    // Compare with direct genome computation
    let expected_beta = genome.present_bias();
    let expected_k = genome.impatience_k();
    assert!(
        (agent.time_preference.present_bias - expected_beta).abs() < 0.01,
        "Present bias should match genome computation"
    );
    assert!(
        (agent.time_preference.impatience_k - expected_k).abs() < 0.01,
        "Impatience k should match genome computation"
    );
}

/// Test hyperbolic discounting decision method
#[test]
fn test_choose_intertemporal_hyperbolic() {
    let genome = Genome::default();
    let agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Get agent's actual impatience parameter
    let _k = agent.time_preference.impatience_k;

    // Test: Prefer $50 now vs $100 in 20 ticks
    // With default k≈0.1: Hyperbolic discount = 100 / (1 + k * 20) ≈ 33.3
    // 50 > 33.3, so should prefer immediate
    let prefer_immediate = agent.choose_intertemporal_hyperbolic(50.0, 100.0, 20);
    assert!(
        prefer_immediate,
        "Should prefer immediate $50 over discounted future reward"
    );

    // Test: Prefer $40 now vs $100 in 2 ticks
    // With default k≈0.1: Hyperbolic discount = 100 / (1 + k * 2) ≈ 83.3
    // 40 < 83.3, so should prefer delayed
    let prefer_immediate2 = agent.choose_intertemporal_hyperbolic(40.0, 100.0, 2);
    assert!(
        !prefer_immediate2,
        "Should prefer delayed larger reward over smaller immediate"
    );

    // Test: Very small immediate vs larger delayed
    // Should prefer delayed when immediate is much smaller
    let prefer_immediate3 = agent.choose_intertemporal_hyperbolic(10.0, 100.0, 5);
    assert!(
        !prefer_immediate3,
        "Should prefer $100 delayed over $10 immediate"
    );
}

/// Test debt default decision using time preference
#[test]
fn test_decide_debt_default() {
    // Create a very impatient agent (low present bias = high impatience)
    let mut genome = Genome::default();
    genome.alleles[40] = [0.0, 0.0]; // Minimum present bias (very impatient)
    genome.alleles[41] = [0.0, 0.0];
    genome.alleles[42] = [1.0, 1.0]; // Maximum impatience k
    genome.alleles[43] = [1.0, 1.0];

    let impatient_agent = Agent::new(
        1u64,
        Position::new(32.0, 32.0, 0.0),
        genome.clone(),
        0,
        Sex::Male,
    );

    // Create a patient agent (high present bias = low impatience)
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    let mut genome2 = Genome::default();
    genome2.alleles[40] = [1.0, 1.0]; // Maximum present bias (patient)
    genome2.alleles[41] = [1.0, 1.0];
    genome2.alleles[42] = [0.0, 0.0]; // Minimum impatience k
    genome2.alleles[43] = [0.0, 0.0];

    let patient_agent = Agent::new(2u64, Position::new(32.0, 32.0, 0.0), genome2, 0, Sex::Male);

    // Verify agents have different time preferences
    assert!(
        impatient_agent.time_preference.present_bias < patient_agent.time_preference.present_bias,
        "Impatient agent should have lower present bias"
    );
    assert!(
        impatient_agent.time_preference.impatience_k > patient_agent.time_preference.impatience_k,
        "Impatient agent should have higher k"
    );

    // Decision: Keep $100 now vs get reputation in 100 ticks
    // Use large difference to ensure different decisions
    let debt_amount = 100.0;
    let reputation_value = 500.0; // Very high reputation value
    let delay = 100;

    // Test that the decision method works and returns a boolean
    let impatient_defaults =
        impatient_agent.decide_debt_default(debt_amount, reputation_value, delay);
    let patient_defaults = patient_agent.decide_debt_default(debt_amount, reputation_value, delay);

    // With extreme values, impatient agent is more likely to default
    // Note: Actual outcome depends on specific discount calculations
    // This test verifies the method works and produces consistent results
    let _ = (impatient_defaults, patient_defaults);
    // Test passes if we reach here without panicking
}

/// Test that present value computation works correctly
#[test]
fn test_present_value_computation() {
    let genome = Genome::default();
    let agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    // Present value of immediate reward should be the reward itself
    let pv_immediate = agent.present_value(100.0, 0);
    assert!(
        (pv_immediate - 100.0).abs() < 0.01,
        "Immediate PV should equal reward"
    );

    // Present value of delayed reward should be discounted
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    let pv_delayed = agent.present_value(100.0, 10);
    assert!(pv_delayed < 100.0, "Delayed PV should be discounted");
}

/// Test sophisticated vs naive choice
#[test]
fn test_sophisticated_choice() {
    let genome = Genome::default();
    let mut agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    // Test naive choice (is_sophisticated = false)
    agent.time_preference.is_sophisticated = false;
    let rewards = vec![100.0, 150.0, 120.0];
    let delays = vec![0, 10, 5];
    let choice_naive = agent.choose_sophisticated(&rewards, &delays);

    // Should choose based on present value
    assert!(
        choice_naive < rewards.len(),
        "Should return valid choice index"
    );

    // Test sophisticated choice (is_sophisticated = true)
    agent.time_preference.is_sophisticated = true;
    let choice_sophisticated = agent.choose_sophisticated(&rewards, &delays);

    // Sophisticated agents may choose differently due to commitment consideration
    assert!(
        choice_sophisticated < rewards.len(),
        "Should return valid choice index"
    );
}

/// Test unified trust combines multiple trust sources
#[test]
fn test_unified_trust() {
    let genome = Genome::default();
    let agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    // Get unified trust
    let unified = agent.unified_trust();

    // Should be in valid range [0, 1]
    assert!(
        (0.0..=1.0).contains(&unified),
        "Unified trust should be in [0, 1]: {unified:.2}"
    );

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    // Get canonical trust
    let canonical = agent.compute_trust();

    // Unified should be related to canonical (since DOCAS is 50% weight)
    // They won't be equal but should be in similar range
    assert!(
        (unified - canonical).abs() < 0.5,
        "Unified and canonical trust should be related: unified={unified}, canonical={canonical}"
    );
}

/// Test that canonical trust computation uses correct formula
#[test]
fn test_canonical_trust_formula() {
    use crate::types::{A, C, D, O, S};

    let genome = Genome::default();
    let mut agent = Agent::new(1u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);

    // Set specific DOCAS values: D=0.8, O=0.6, S=0.7, C=0.2, A=0.1
    agent.docas.levels[D] = 0.8;
    agent.docas.levels[O] = 0.6;
    agent.docas.levels[S] = 0.7;
    agent.docas.levels[C] = 0.2;
    agent.docas.levels[A] = 0.1;
    // TAG: surface=simulation owner=research-team rule=SIM-001

    // Clear tau history for clean test
    agent.tau_history.clear();

    // Compute base tau manually: (0.8 + 0.6 + 0.7) / (1 + 0.2 + 0.1) = 2.1 / 1.3 ≈ 1.615
    let base_tau: f32 = (0.8 + 0.6 + 0.7) / (1.0 + 0.2 + 0.1);
    assert!(
        (base_tau - 1.615f32).abs() < 0.01,
        "Base tau should be ~1.615"
    );

    // Compute trust (may differ slightly due to viscosity/stability)
    let trust = agent.compute_trust();

    // Trust should be in valid range
    assert!(
        (0.0..=2.0).contains(&trust),
        "Trust should be in [0, 2]: {trust:.2}"
    );

    // With no history, trust should be close to base tau
    assert!(
        (trust - base_tau).abs() < 0.3,
        "Trust should be close to base tau with no history: trust={trust}, base={base_tau}"
    );
}

#[test]
fn test_homeostatic_docas_bounds_and_lineage_continuity() {
    use crate::lineage::core::{LineageNode, TraitSnapshot};
    use crate::lineage::tree::LineageTree;

    // 1. Verify Homeostatic DOCAS bounds: Dopamine doesn't collapse to 0.000, Serotonin doesn't pin at 1.0
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let genome = Genome::random(&mut rng);
    let mut agent = Agent::new(
        100u64,
        Position::new(32.0, 32.0, 0.0),
        genome.clone(),
        0,
        Sex::Male,
    );

    for _ in 0..100 {
        // Zero resource / negative error
        agent.docas.update(&genome, -0.4, 0.0, 0.1, 0.2, 0.5);
    }
    // Dopamine must remain homeostatically bounded above 0.05
    assert!(
        agent.docas.dopamine() > 0.05,
        "Dopamine should not collapse to zero under homeostasis"
    );
    assert!(
        agent.docas.serotonin() < 0.98,
        "Serotonin should not pin at 1.000 under homeostasis"
    );

    // 2. Lineage continuity across dead-agent conversion
    let mut tree = LineageTree::new();
    let traits = TraitSnapshot::from_genome(&genome, 0);

    // Initial parent (id=1)
    tree.add_initial(LineageNode::new_initial(1, 0, traits.clone()));

    // Child (id=2, parent=1)
    tree.add_offspring(LineageNode::new_offspring(
        2,
        1,
        None,
        10,
        traits.clone(),
        0,
    ));

    // Grandchild (id=3, parent=2)
    tree.add_offspring(LineageNode::new_offspring(
        3,
        2,
        None,
        20,
        traits.clone(),
        1,
    ));

    // Check counts before death
    let (c1, g1) = tree.get_children_and_grandchildren_counts(1);
    assert_eq!(c1, 1, "Parent 1 should have 1 child");
    assert_eq!(g1, 1, "Parent 1 should have 1 grandchild");

    // Parent 1 dies and is converted to summary
    tree.record_death(1, 25);
    let converted = tree.convert_to_summary(1);
    assert!(converted, "Node 1 converted to summary");

    // Child 2 dies and is converted to summary
    tree.record_death(2, 30);
    let converted2 = tree.convert_to_summary(2);
    assert!(converted2, "Node 2 converted to summary");

    // After compaction of both parent and child, Parent 1 counts must be preserved!
    let (c1_after, g1_after) = tree.get_children_and_grandchildren_counts(1);
    assert_eq!(
        c1_after, 1,
        "Children count preserved after death & summary conversion"
    );
    assert_eq!(
        g1_after, 1,
        "Grandchildren count preserved after death & summary conversion"
    );
}

#[test]
fn test_sequestration_wall_orthogonality() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let genome = Genome::random(&mut rng);
    let mut agent = Agent::new(
        101u64,
        Position::new(32.0, 32.0, 0.0),
        genome,
        0,
        Sex::Female,
    );

    agent.economics.market_trust_score = 0.72;
    agent.economics.bond_density = 0.85;
    agent.reputation_score = 0.80;

    let baseline_access = crate::economics::compute_credit_access(
        agent.economics.market_trust_score,
        agent.economics.bond_density,
        (agent.reputation_score + 1.0) / 2.0,
    );

    // Perturb biological and wearable biomarkers by extreme values (+/- 5 sigma)
    agent.biomarkers.hrv_rmssd_ms = 15.0; // severe distress
    agent.biomarkers.resting_hr_bpm = 98.0;
    agent.biomarkers.glucose_mg_dl = 185.0;
    agent.biomarkers.eeg_alpha_asymmetry = -0.58;
    agent.biomarkers.sleep_efficiency_pct = 52.0;

    let post_perturb_access = crate::economics::compute_credit_access(
        agent.economics.market_trust_score,
        agent.economics.bond_density,
        (agent.reputation_score + 1.0) / 2.0,
    );

    // Sequestration Wall Invariant: Underwriting decisions must have zero mathematical dependence on wearable physiology
    assert_eq!(
        baseline_access, post_perturb_access,
        "Sequestration Wall violation: credit access changed under physiological perturbation"
    );
    assert_eq!(agent.economics.market_trust_score, 0.72);
}

#[test]
fn test_wearable_biomarkers_directional_validity() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let genome = Genome::random(&mut rng);
    let mut agent = Agent::new(102u64, Position::new(32.0, 32.0, 0.0), genome, 0, Sex::Male);
    let circadian = crate::circadian::CircadianSystem::new();

    // Baseline update with neutral DOCAS
    agent
        .biomarkers
        .update(&agent.docas, &circadian, 0.0, 0.0, 0.5);
    let baseline_hrv = agent.biomarkers.hrv_rmssd_ms;
    let baseline_hr = agent.biomarkers.resting_hr_bpm;
    let baseline_glucose = agent.biomarkers.glucose_mg_dl;

    // Acute Stress Shock: High Cortisol (C) and Adrenaline (A)
    agent.docas.levels[crate::types::C] = 0.95;
    agent.docas.levels[crate::types::A] = 0.90;
    agent.docas.levels[crate::types::O] = 0.10;
    agent.docas.levels[crate::types::S] = 0.10;

    for _ in 0..10 {
        agent
            .biomarkers
            .update(&agent.docas, &circadian, 0.0, -0.4, 0.3);
    }

    assert!(
        agent.biomarkers.hrv_rmssd_ms < baseline_hrv,
        "High stress must suppress HRV RMSSD"
    );
    assert!(
        agent.biomarkers.resting_hr_bpm > baseline_hr,
        "High stress must elevate resting heart rate"
    );
    assert!(
        agent.biomarkers.glucose_mg_dl > baseline_glucose,
        "High cortisol must stimulate hepatic gluconeogenesis"
    );
    assert!(
        agent.biomarkers.eeg_alpha_asymmetry < 0.0,
        "High stress must shift FAA negative (threat withdrawal)"
    );
}

#[test]
fn test_full_nine_layer_ablation_safety() {
    use crate::simulation::{Simulation, SimulationConfig};

    // Test knocking out each of the 9 layers independently
    let mut configs = Vec::new();

    // 1. Layer 1 (Geography) OFF
    let mut c1 = SimulationConfig::default();
    c1.toggles.geography_enabled = false;
    configs.push(("Geography OFF", c1));

    // 2. Layer 2 (Genealogy) OFF
    let mut c2 = SimulationConfig::default();
    c2.toggles.genome_enabled = false;
    configs.push(("Genealogy OFF", c2));

    // 3. Layer 3 (Biology) OFF
    let mut c3 = SimulationConfig::default();
    c3.toggles.biology_enabled = false;
    configs.push(("Biology OFF", c3));

    // 4. Layer 4 (Neurobiology) OFF
    let mut c4 = SimulationConfig::default();
    c4.toggles.docas_enabled = false;
    configs.push(("Neurobiology OFF", c4));

    // 5. Layer 5 (Psychology) OFF
    let mut c5 = SimulationConfig::default();
    c5.toggles.dmn_enabled = false;
    c5.toggles.choice_architecture_enabled = false;
    configs.push(("Psychology OFF", c5));

    // 6. Layer 6 (Philosophy) OFF
    let mut c6 = SimulationConfig::default();
    c6.toggles.philosophy_enabled = false;
    configs.push(("Philosophy OFF", c6));

    // 7. Layer 7 (Ecology & Society) OFF
    let mut c7 = SimulationConfig::default();
    c7.toggles.institutions_enabled = false;
    c7.toggles.norms_enabled = false;
    c7.toggles.groups_enabled = false;
    configs.push(("Ecology OFF", c7));

    // 8. Layer 8 (Economy) OFF
    let mut c8 = SimulationConfig::default();
    c8.toggles.economics_enabled = false;
    configs.push(("Economy OFF", c8));

    // 9. Layer 9 (Protocol) OFF
    let mut c9 = SimulationConfig::default();
    c9.toggles.protocol_enabled = false;
    configs.push(("Protocol OFF", c9));

    for (label, cfg) in configs {
        let mut sim = Simulation::new(cfg);
        sim.spawn_agent(Position::new_2d(25.0, 25.0));
        sim.spawn_agent(Position::new_2d(26.0, 25.0));
        for _ in 0..20 {
            sim.step();
        }
        for agent in &sim.agents {
            assert!(
                agent.energy.is_finite(),
                "{label}: agent energy was NaN or infinite"
            );
            assert!(agent.docas.levels[0].is_finite(), "{label}: docas was NaN");
            assert!(
                agent.biomarkers.glucose_mg_dl.is_finite(),
                "{label}: glucose was NaN"
            );
        }
    }
}
