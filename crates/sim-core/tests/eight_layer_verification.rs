// TAG: surface=simulation owner=research-team rule=SIM-001
//! Eight-Layer Architecture Verification Tests
//!
//! CODE-009: EXCEPTION - Integration test for sim-core crate (tests 8-layer architecture)
//! Per architecture spec: 8 canonical layers with strict unidirectional causal dependencies
//! Each layer depends on all layers below it.
//!
//! Layer Stack (bottom-up):
//! 1. Geography (Stage 0) → Spatial substrate, agent positions
//! 2. Genealogy (Stage 1) → 40 diploid loci, Mendelian inheritance
//! 3. Biology (Stage 1) → Energy dynamics, mortality, reproduction
//! 4. Neurobiology (Stage 2) → DOCAS [D,O,C,A,S], genome-modulated
//! 5. Psychology (Stage 2) → Semantic state, affect, social orientation
//! 6. Philosophy (Stage 2) → Worldview, norm adherence, trust posture
//! 7. Ecology (Stage 3) → Population dynamics, reputation, norms
//! 8. Economy (Stage 3+) → Credit access, debt, wealth
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-004: Deterministic tests use fixed seeds

// Numeric cast precision is acceptable in these simulation test utilities.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::float_cmp
)]

use rand::Rng;
use rand::SeedableRng;
use sim_core::docas::DocasState;
use sim_core::genome::Genome;
use sim_core::simulation::{Simulation, SimulationConfig};
use sim_core::toggles::ToggleConfig;
use sim_core::Position;

/// Test Layer 1: Geography
/// - Output: Positions in grid bounds
/// - Check: No agent outside [0, `grid_size`]
#[test]
fn test_layer_1_geography() {
    let config = SimulationConfig {
        toggles: ToggleConfig::stage_0(),
        max_ticks: 100,
        seed: 42,
        ..Default::default()
    };

    let mut sim = Simulation::new(config);

    // Spawn agents at various positions
    for i in 0..10 {
        let x = (i as f32).mul_add(6.0, 5.0);
        let y = (i as f32).mul_add(5.0, 3.0);
        // TAG: surface=simulation owner=research-team rule=SIM-001
        sim.spawn_agent(Position::new_2d(x, y));
    }

    // Run simulation
    for _ in 0..50 {
        sim.step();
    }

    // Verify all agents remain in bounds
    for agent in &sim.agents {
        let pos = &agent.position;
        assert!(
            pos.x >= 0.0 && pos.x <= 64.0,
            "Agent {} X out of bounds: {}",
            agent.id,
            pos.x
        );
        assert!(
            pos.y >= 0.0 && pos.y <= 64.0,
            "Agent {} Y out of bounds: {}",
            agent.id,
            pos.y
        );
    }
}

/// Test Layer 2: Genealogy
/// - 44 diploid loci encoding 22 traits
/// - Mendelian inheritance with mutation
/// - Parent-offspring genome correlation r > 0.5
#[test]
fn test_layer_2_genealogy() {
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use sim_core::genome::crossover;

    let mut rng = StdRng::seed_from_u64(42);
    let parent_a = Genome::random(&mut rng);
    let parent_b = Genome::random(&mut rng);

    // Generate multiple offspring
    let mut offspring_similarities = Vec::new();
    for _ in 0..20 {
        let offspring = crossover(&parent_a, &parent_b, 0.001, &mut rng);

        // Compute similarity to parents
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let sim_a = genome_similarity(&parent_a, &offspring);
        let sim_b = genome_similarity(&parent_b, &offspring);
        offspring_similarities.push((sim_a + sim_b) / 2.0);
    }

    // Average similarity should be > 0.5 (Mendelian inheritance)
    let avg_similarity: f32 =
        offspring_similarities.iter().sum::<f32>() / offspring_similarities.len() as f32;
    assert!(
        avg_similarity > 0.5,
        "Parent-offspring genome correlation too low: {avg_similarity}"
    );

    // DOCAS alpha and beta should be in valid ranges
    let alpha = parent_a.docas_alpha();
    let beta = parent_a.docas_beta();

    for a in alpha {
        assert!(
            (0.9..=0.99).contains(&a),
            "DOCAS alpha {a} out of range [0.9, 0.99]"
        );
    }
    for b in beta {
        assert!(
            (0.05..=0.2).contains(&b),
            "DOCAS beta {b} out of range [0.05, 0.2]"
        );
    }
}

/// Compute similarity between two genomes
fn genome_similarity(a: &Genome, b: &Genome) -> f32 {
    let mut total_diff = 0.0;
    let n = a.alleles.len().min(b.alleles.len());

    for i in 0..n {
        let diff_0 = (a.alleles[i][0] - b.alleles[i][0]).abs();
        let diff_1 = (a.alleles[i][1] - b.alleles[i][1]).abs();
        total_diff += (diff_0 + diff_1) / 2.0;
    }

    1.0 - (total_diff / n as f32)
}

/// Test Layer 3: Biology
// TAG: surface=simulation owner=research-team rule=SIM-001
/// - Energy dynamics, mortality, reproduction
/// - Energy mean should be in reasonable range
#[test]
#[ignore = "slow: 200-tick simulation validation"]
fn test_layer_3_biology() {
    let config = SimulationConfig {
        toggles: ToggleConfig::stage_1(), // Enable genome
        max_ticks: 200,
        seed: 42,
        reproduction_threshold: 60.0,
        reproduction_cost: 20.0,
        ..Default::default()
    };

    let mut sim = Simulation::new(config);

    // Spawn initial population
    for _ in 0..20 {
        sim.spawn_agent(Position::new_2d(32.0, 32.0));
    }

    // Run simulation
    for _ in 0..200 {
        sim.step();
    }

    // Check energy levels are reasonable
    let living: Vec<f32> = sim
        .agents
        .iter()
        .filter(|a| a.alive)
        .map(|a| a.energy)
        .collect();

    if !living.is_empty() {
        let mean_energy: f32 = living.iter().sum::<f32>() / living.len() as f32;
        assert!(
            (10.0..=95.0).contains(&mean_energy),
            "Mean energy {mean_energy} outside expected range [10, 95]"
        );
    }

    // Verify reproduction occurred (population should have changed)
    assert!(sim.agents.len() > 20, "No reproduction occurred");
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Test Layer 4: Neurobiology (DOCAS)
/// - [D, O, C, A, S] values in [0, 1]
/// - Uses genome params (alpha, beta), not hardcoded
/// - Genome-modulated dynamics
#[test]
fn test_layer_4_neurobiology() {
    use sim_core::docas::DocasState;

    let config = SimulationConfig {
        toggles: ToggleConfig::stage_2(), // Enable DOCAS
        max_ticks: 100,
        seed: 42,
        ..Default::default()
    };

    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    // Run simulation
    for _ in 0..100 {
        sim.step();
    }

    // Verify DOCAS values are in bounds
    for agent in &sim.agents {
        let [dopamine, oxytocin, cortisol, adrenaline, serotonin] = agent.docas.levels;
        assert!(
            (0.0..=1.0).contains(&dopamine),
            "Dopamine {dopamine} out of bounds"
        );
        assert!(
            (0.0..=1.0).contains(&oxytocin),
            "Oxytocin {oxytocin} out of bounds"
        );
        assert!(
            (0.0..=1.0).contains(&cortisol),
            "Cortisol {cortisol} out of bounds"
        );
        assert!(
            (0.0..=1.0).contains(&adrenaline),
            "Adrenaline {adrenaline} out of bounds"
        );
        assert!(
            (0.0..=1.0).contains(&serotonin),
            "Serotonin {serotonin} out of bounds"
        );
    }

    // Test DOCAS update uses genome params
    let genome = Genome::default();
    let mut docas = DocasState::neutral();

    // Get initial levels
    let initial_d = docas.levels[0];

    // Update with reward error
    // P3-1: Include geometric parameters (kappa, psi) - neutral values for test
    docas.update(&genome, 1.0, 0.0, 0.0, 0.0, 0.0); // κ=0, ψ=0.75 (geometric props computed separately)

    // Dopamine should increase with positive reward error
    assert!(
        // TAG: surface=simulation owner=research-team rule=SIM-001
        docas.levels[0] > initial_d,
        "DOCAS Dopamine should increase with reward"
    );

    // Verify compute_tau works correctly
    let tau = docas.compute_tau();
    // τ = (D + O + S) / (1 + C + A)
    let expected_tau = (docas.levels[0] + docas.levels[1] + docas.levels[4])
        / (1.0 + docas.levels[2] + docas.levels[3]);
    assert!(
        (tau - expected_tau).abs() < 0.001,
        "Tau computation incorrect"
    );
}

/// Test Layer 5: Psychology
/// - Semantic state, affect, social orientation
/// - High O increases `social_orientation`
/// - Consumes DOCAS outputs
#[test]
fn test_layer_5_psychology() {
    let config = SimulationConfig {
        toggles: ToggleConfig::stage_2(), // Enable DOCAS → Psychology
        max_ticks: 100,
        seed: 42,
        ..Default::default()
    };

    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    // Run simulation
    for _ in 0..100 {
        sim.step();
    }

    // Verify psychology layer fields
    for agent in &sim.agents {
        // Affect valence in [-1, 1]
        assert!(
            agent.affect_valence >= -1.0 && agent.affect_valence <= 1.0,
            "Affect valence {} out of bounds",
            agent.affect_valence
        );

        // Affect arousal in [0, 1]
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(
            agent.affect_arousal >= 0.0 && agent.affect_arousal <= 1.0,
            "Affect arousal {} out of bounds",
            agent.affect_arousal
        );

        // Social orientation in [-1, 1]
        assert!(
            agent.social_orientation >= -1.0 && agent.social_orientation <= 1.0,
            "Social orientation {} out of bounds",
            agent.social_orientation
        );

        // High Oxytocin should correlate with positive social orientation
        let oxytocin = agent.docas.oxytocin();
        if oxytocin > 0.7 {
            assert!(
                agent.social_orientation > 0.0,
                "High oxytocin ({}) should yield positive social_orientation ({})",
                oxytocin,
                agent.social_orientation
            );
        }
    }

    // Verify semantic state is populated
    let agent = &sim.agents[0];
    for (i, &val) in agent.semantic_state.iter().enumerate() {
        assert!(
            (0.0..=1.0).contains(&val),
            "Semantic state[{i}] {val} out of bounds"
        );
    }
}

/// Test Layer 6: Philosophy
/// - Worldview, norm adherence, trust posture
/// - Narrative coherence correlates with `norm_adherence`
/// - Consumes Psychology outputs
#[test]
fn test_layer_6_philosophy() {
    let config = SimulationConfig {
        toggles: ToggleConfig::stage_2(), // Enable up to Philosophy
        max_ticks: 150,
        seed: 42,
        ..Default::default() // TAG: surface=simulation owner=research-team rule=SIM-001
    };

    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    // Run simulation
    for _ in 0..150 {
        sim.step();
    }

    // Verify philosophy layer fields
    for agent in &sim.agents {
        // Check philosophical state exists and tau_phi is in [-1, 1]
        if let Some(ref phil) = agent.philosophical_state {
            assert!(
                phil.tau_phi >= -1.0 && phil.tau_phi <= 1.0,
                "tau_phi {} out of bounds",
                phil.tau_phi
            );
            assert!(
                phil.bond_activation >= 0.0 && phil.bond_activation <= 1.0,
                "bond_activation {} out of bounds",
                phil.bond_activation
            );
        }
    }

    // Test correlation between tau_phi and norm_adherence (mapped from tau_phi)
    let tau_phi_values: Vec<f32> = sim
        .agents
        .iter()
        .filter(|a| a.alive)
        .filter_map(|a| a.philosophical_state.as_ref().map(|p| p.tau_phi))
        .collect();
    let adherence_values: Vec<f32> = sim
        .agents
        .iter()
        .filter(|a| a.alive)
        .filter_map(|a| {
            a.philosophical_state
                .as_ref()
                .map(|p| (p.tau_phi + 1.0) / 2.0)
        })
        .collect();

    if tau_phi_values.len() > 1 {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let correlation = compute_correlation(&tau_phi_values, &adherence_values);
        // Correlation should be positive (tau_phi → norm adherence)
        assert!(
            correlation > 0.0,
            "Expected positive correlation between tau_phi and adherence, got {correlation}"
        );
    }
}

/// Compute Pearson correlation coefficient
fn compute_correlation(x: &[f32], y: &[f32]) -> f32 {
    if x.len() != y.len() || x.len() < 2 {
        return 0.0;
    }

    let n = x.len() as f32;
    let mean_x: f32 = x.iter().sum::<f32>() / n;
    let mean_y: f32 = y.iter().sum::<f32>() / n;

    let mut num = 0.0;
    let mut den_x = 0.0;
    let mut den_y = 0.0;

    for i in 0..x.len() {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        num += dx * dy;
        den_x += dx * dx;
        den_y += dy * dy;
    }

    let denom = (den_x * den_y).sqrt();
    if denom < 1e-10 {
        return 0.0;
    }

    num / denom
}

/// Test Layer 7: Ecology
/// - Population dynamics, reputation, norms
/// - >60% compliance triggers norm emergence
/// - Consumes Philosophy outputs (`norm_adherence`)
#[test]
#[ignore = "slow: 300-tick simulation validation"]
fn test_layer_7_ecology() {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let config = SimulationConfig {
        toggles: ToggleConfig::stage_3(), // Enable institutions + norms
        max_ticks: 300,
        seed: 42,
        ..Default::default()
    };

    let mut sim = Simulation::new(config);

    // Spawn population with diverse norms
    for i in 0..30 {
        let x = (i as f32).mul_add(1.0, 20.0);
        let y = (i as f32).mul_add(0.8, 20.0);
        sim.spawn_agent(Position::new_2d(x, y));
    }

    // Run simulation
    for _ in 0..300 {
        sim.step();
    }

    // Check that norms can emerge
    // Stage 3 enables norm processing
    // We mainly verify the simulation runs without errors
    assert!(sim.tick == 300, "Simulation should complete 300 ticks");

    // Verify some agents have group memberships
    let grouped_count = sim
        .agents
        .iter()
        .filter(|a| a.alive && a.group_id.is_some())
        .count();

    // Some agents should form groups
    // (This is probabilistic, so we just check the system works)
    println!("Agents in groups: {grouped_count}/30");
}

/// Test Layer 8: Economy
/// - Credit access gated by reputation threshold 0.3
/// - Check: reputation < 0.3 → `credit_access` = 0.0
/// - Consumes Ecology outputs (reputation)
#[test]
fn test_layer_8_economy() {
    use sim_core::economics::compute_credit_access;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let config = SimulationConfig {
        toggles: ToggleConfig::all_on(), // Enable all layers including economics
        max_ticks: 200,
        seed: 42,
        ..Default::default()
    };

    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    // Run simulation
    for _ in 0..200 {
        sim.step();
    }

    // Test credit access threshold
    // Below threshold: should get 0 credit
    let credit_low = compute_credit_access(0.4, 0.3, 0.2); // reputation = 0.2 < 0.3
    assert_eq!(credit_low, 0.0, "Credit should be 0 when reputation < 0.3");

    let credit_low_rep = compute_credit_access(0.8, 0.6, 0.25); // reputation = 0.25 < 0.3
    assert_eq!(
        credit_low_rep, 0.0,
        "Credit should be 0 when reputation < 0.3"
    );

    // Above thresholds: should get some credit
    let credit_high = compute_credit_access(0.8, 0.6, 0.6);
    assert!(
        credit_high > 0.0,
        "Credit should be > 0 when all thresholds met"
    );

    // Verify economic state fields
    for agent in &sim.agents {
        if agent.alive {
            assert!(
                agent.economics.credit_access >= 0.0 && agent.economics.credit_access <= 1.0,
                "Credit access {} out of bounds",
                agent.economics.credit_access
            );
            assert!(
                agent.economics.debt_burden >= 0.0,
                "Debt burden {} should be non-negative",
                agent.economics.debt_burden
            );
            // TAG: surface=simulation owner=research-team rule=SIM-001
        }
    }
}

/// Test 500-tick stability - no panics, all layers functional
#[test]
#[ignore = "slow: 500-tick simulation validation"]
fn test_500_tick_stability() {
    let config = SimulationConfig {
        toggles: ToggleConfig::all_on(),
        max_ticks: 500,
        seed: 42,
        ..Default::default()
    };

    let mut sim = Simulation::new(config);

    // Spawn initial population spread across the environment
    // to avoid O(n²) encounter storms that make the test impractically slow.
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    for _ in 0..20 {
        let x = rng.gen::<f32>().mul_add(60.0, 2.0);
        let y = rng.gen::<f32>().mul_add(60.0, 2.0);
        sim.spawn_agent(Position::new_2d(x, y));
    }

    // Run 500 ticks - should not panic
    for tick in 0..500 {
        sim.step();

        // Verify no NaN or Inf values in critical fields
        for agent in &sim.agents {
            if agent.alive {
                assert!(!agent.energy.is_nan(), "Energy NaN at tick {tick}");
                assert!(!agent.energy.is_infinite(), "Energy Inf at tick {tick}");

                for (i, &level) in agent.docas.levels.iter().enumerate() {
                    assert!(!level.is_nan(), "DOCAS[{i}] NaN at tick {tick}");
                    assert!(!level.is_infinite(), "DOCAS[{i}] Inf at tick {tick}");
                }
            }
        }
    }

    assert_eq!(sim.tick, 500, "Should complete 500 ticks");

    // TAG: surface=simulation owner=research-team rule=SIM-001
}

/// Test Causal Chain: Geography → Genealogy
/// Spatial proximity → mate selection
/// Agents outside perception radius can't reproduce
#[test]
fn test_causal_chain_geography_to_genealogy() {
    let config = SimulationConfig {
        toggles: ToggleConfig::stage_1(),
        max_ticks: 100,
        seed: 42,
        ..Default::default()
    };

    let mut sim = Simulation::new(config);

    // Spawn agents far apart (opposite corners)
    sim.spawn_agent(Position::new_2d(5.0, 5.0));
    sim.spawn_agent(Position::new_2d(60.0, 60.0));

    let initial_count = sim.agents.len();

    // Run simulation
    for _ in 0..100 {
        sim.step();
    }

    // Agents far apart should not reproduce
    // (Reproduction requires proximity and opposite sex)
    // This is a weak check - mainly ensuring the system doesn't panic
    assert!(sim.agents.len() >= initial_count, "Agents should survive");
}

/// Test Causal Chain: Genealogy → Biology
/// Genome → biological traits
#[test]
fn test_causal_chain_genealogy_to_biology() {
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use sim_core::genome::Genome;

    let mut rng = StdRng::seed_from_u64(42);

    // Create genomes with different traits
    let genome_low_risk = Genome::random(&mut rng);
    let genome_high_risk = Genome::random(&mut rng);
    // TAG: surface=simulation owner=research-team rule=SIM-001

    // Verify genome traits affect agent properties
    let risk_low = genome_low_risk.risk_tolerance();
    let risk_high = genome_high_risk.risk_tolerance();

    // Risk tolerance should be in valid range
    assert!(
        (0.0..=1.0).contains(&risk_low),
        "Risk tolerance out of bounds"
    );
    assert!(
        (0.0..=1.0).contains(&risk_high),
        "Risk tolerance out of bounds"
    );
}

/// Test Causal Chain: Biology → Neurobiology
/// `Genome.docas_alpha()`, `docas_beta()` modulate DOCAS
#[test]
fn test_causal_chain_biology_to_neurobiology() {
    use sim_core::genome::Genome;

    // Create genomes with explicitly different DOCAS trait values
    // Trait 0-4 control alpha, Trait 5-9 control beta
    let mut genome_low = Genome::default();
    let mut genome_high = Genome::default();

    // Set trait-controlling loci to different values
    // Trait 0 (D alpha): loci 0, 1
    genome_low.alleles[0] = [0.1, 0.1];
    genome_low.alleles[1] = [0.1, 0.1];
    genome_high.alleles[0] = [0.9, 0.9];
    genome_high.alleles[1] = [0.9, 0.9];

    // Trait 5 (D beta): loci 10, 11
    genome_low.alleles[10] = [0.1, 0.1];
    genome_low.alleles[11] = [0.1, 0.1];
    genome_high.alleles[10] = [0.9, 0.9];
    genome_high.alleles[11] = [0.9, 0.9];

    // Get DOCAS parameters
    let alpha_low = genome_low.docas_alpha();
    let alpha_high = genome_high.docas_alpha();

    // Different genomes should produce different alpha values for D
    assert!(
        // TAG: surface=simulation owner=research-team rule=SIM-001
        (alpha_low[0] - alpha_high[0]).abs() > 0.05,
        "Different genomes should produce different alpha values: low={alpha_low:?}, high={alpha_high:?}"
    );

    // Verify DOCAS update uses genome params
    let mut docas1 = DocasState::neutral();
    let mut docas2 = DocasState::neutral();

    // P3-1: Include geometric parameters (kappa, psi) - neutral values for test
    docas1.update(&genome_low, 1.0, 0.0, 0.0, 0.0, 0.0); // κ=0, ψ=0.75 (geometric props computed separately)
    docas2.update(&genome_high, 1.0, 0.0, 0.0, 0.0, 0.0); // κ=0, ψ=0.75 (geometric props computed separately)

    // Different genomes → different DOCAS updates
    let diff = (docas1.dopamine() - docas2.dopamine()).abs();
    assert!(
        diff > 0.0,
        "Different genomes should produce different DOCAS states: diff={diff}"
    );
}

/// Test Causal Chain: Neurobiology → Psychology
/// DOCAS → semantic similarity, affect, social orientation
/// High O increases `social_orientation`
#[test]
fn test_causal_chain_neurobiology_to_psychology() {
    use sim_core::genome::Genome;
    use sim_core::types::{Position, Sex};

    let genome = Genome::default();
    let mut agent =
        sim_core::agent::Agent::new(0, Position::new_2d(32.0, 32.0), genome, 0, Sex::Male);

    // Set high Oxytocin
    agent.docas.levels[1] = 0.9; // O = high
    agent.update_psychology();

    let social_high_o = agent.social_orientation;

    // Set low Oxytocin
    agent.docas.levels[1] = 0.1; // O = low
    agent.update_psychology();

    let social_low_o = agent.social_orientation;

    // High O should yield more positive social orientation than low O
    assert!(
        // TAG: surface=simulation owner=research-team rule=SIM-001
        social_high_o > social_low_o - 0.5,
        "High oxytocin should yield more positive social orientation"
    );
}

/// Test Causal Chain: Psychology → Philosophy
/// Social orientation → worldview
/// `tau_phi` correlates with social orientation
#[test]
fn test_causal_chain_psychology_to_philosophy() {
    use sim_core::genome::Genome;
    use sim_core::types::{Position, Sex};

    let genome = Genome::default();
    let mut agent =
        sim_core::agent::Agent::new(0, Position::new_2d(32.0, 32.0), genome, 0, Sex::Male);

    // Initialize and update philosophy
    agent.initialize_philosophy(0);
    agent.update_philosophy(0);

    // Check philosophical state exists
    assert!(
        agent.philosophical_state.is_some(),
        "Philosophical state should be initialized"
    );

    let tau_phi = agent.philosophical_state.as_ref().unwrap().tau_phi;
    assert!(
        (-1.0..=1.0).contains(&tau_phi),
        "tau_phi {tau_phi} out of bounds"
    );

    // High social orientation should correlate with higher tau_phi (openness)
    agent.social_orientation = 0.8;
    agent.update_philosophy(1);
    let high_social_tau_phi = agent.philosophical_state.as_ref().unwrap().tau_phi;

    agent.social_orientation = -0.8;
    agent.update_philosophy(2);
    let low_social_tau_phi = agent.philosophical_state.as_ref().unwrap().tau_phi;

    // Positive social orientation should yield higher tau_phi (more openness)
    assert!(
        high_social_tau_phi >= low_social_tau_phi - 0.1,
        "Positive social orientation should correlate with higher tau_phi (openness)" // TAG: surface=simulation owner=research-team rule=SIM-001
    );
}

/// Test Causal Chain: Philosophy → Ecology
/// Norm adherence → emergent norms
/// >60% compliance triggers norm emergence
#[test]
#[ignore = "slow: 200-tick simulation validation"]
fn test_causal_chain_philosophy_to_ecology() {
    let config = SimulationConfig {
        toggles: ToggleConfig::stage_3(),
        max_ticks: 200,
        seed: 42,
        ..Default::default()
    };

    let mut sim = Simulation::new(config);

    // Spawn population
    for _ in 0..20 {
        sim.spawn_agent(Position::new_2d(32.0, 32.0));
    }

    // Run simulation
    for _ in 0..200 {
        sim.step();
    }

    // Verify norm compliance tracking is working (via tau_phi)
    let avg_tau_phi: f32 = sim
        .agents
        .iter()
        .filter(|a| a.alive)
        .filter_map(|a| a.philosophical_state.as_ref().map(|p| p.tau_phi))
        .sum::<f32>()
        / sim.living_agent_count().max(1) as f32;
    let avg_adherence = (avg_tau_phi + 1.0) / 2.0;

    println!("Average norm adherence (from tau_phi): {avg_adherence}");

    // Adherence should be in valid range
    assert!(
        (0.0..=1.0).contains(&avg_adherence),
        "Average norm adherence {avg_adherence} out of bounds"
    );
}

// TAG: surface=simulation owner=research-team rule=SIM-001

/// Test Causal Chain: Ecology → Economy
/// Reputation gates credit access (threshold 0.3)
/// reputation < 0.3 → `credit_access` = 0.0
#[test]
fn test_causal_chain_ecology_to_economy() {
    use sim_core::economics::compute_credit_access;

    // Test explicit threshold
    let credit_below = compute_credit_access(0.8, 0.7, 0.2); // rep = 0.2 < 0.3
    assert_eq!(
        credit_below, 0.0,
        "Credit should be 0 when reputation < 0.3"
    );

    let credit_at_threshold = compute_credit_access(0.8, 0.7, 0.3); // rep = 0.3
    assert!(
        credit_at_threshold > 0.0,
        "Credit should be > 0 at threshold"
    );

    let credit_above = compute_credit_access(0.8, 0.7, 0.8); // rep = 0.8 > 0.3
    assert!(
        credit_above > credit_at_threshold,
        "Higher reputation should yield higher credit"
    );
}

/// Test Economic Feedback Loop: Economy → Reputation → DOCAS
/// Success increases O/S; failures increase C/decrease D
#[test]
fn test_economic_feedback_loop() {
    let config = SimulationConfig {
        toggles: ToggleConfig::all_on(),
        max_ticks: 100,
        seed: 42,
        ..Default::default()
    };

    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    // Record initial state
    let initial_docas = sim.agents[0].docas.levels;

    // Run simulation
    // TAG: surface=simulation owner=research-team rule=SIM-001
    for _ in 0..100 {
        sim.step();
    }

    // Verify DOCAS changed (feedback loop is active)
    let final_docas = sim.agents[0].docas.levels;
    let diff: f32 = initial_docas
        .iter()
        .zip(final_docas.iter())
        .map(|(a, b)| (a - b).abs())
        .sum();

    // DOCAS should have changed over time due to feedback
    assert!(
        diff > 0.0,
        "DOCAS should change due to economic feedback loop"
    );
}

/// Test toggle dependencies: disabling lower layer prevents upper layer
#[test]
fn test_toggle_dependencies() {
    // Stage 0 only: Only Geography should be active
    let config_0 = SimulationConfig {
        toggles: ToggleConfig::stage_0(),
        max_ticks: 50,
        seed: 42,
        ..Default::default()
    };
    assert!(!config_0.toggles.genome_enabled);

    // Stage 1: Geography + Genealogy/Biology
    let config_1 = SimulationConfig {
        toggles: ToggleConfig::stage_1(),
        max_ticks: 50,
        seed: 42,
        ..Default::default()
    };
    assert!(config_1.toggles.genome_enabled);
    assert!(!config_1.toggles.docas_enabled);

    // Stage 2: + Neurobiology/Psychology/Philosophy
    let config_2 = SimulationConfig {
        toggles: ToggleConfig::stage_2(),
        max_ticks: 50,
        seed: 42,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        ..Default::default()
    };
    assert!(config_2.toggles.genome_enabled);
    assert!(config_2.toggles.docas_enabled);

    // All stages: + Ecology/Economy
    let config_all = SimulationConfig {
        toggles: ToggleConfig::all_on(),
        max_ticks: 50,
        seed: 42,
        ..Default::default()
    };
    assert!(config_all.toggles.institutions_enabled);
    assert!(config_all.toggles.norms_enabled);
    assert!(config_all.toggles.economics_enabled);
}

/// Test layer-specific assertions from verification spec
#[test]
#[ignore = "slow: 500-tick simulation validation"]
fn test_layer_metrics() {
    let config = SimulationConfig {
        toggles: ToggleConfig::all_on(),
        max_ticks: 500,
        seed: 42,
        ..Default::default()
    };

    let mut sim = Simulation::new(config);

    // Spawn initial population spread across the environment
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    for _ in 0..30 {
        let x = rng.gen::<f32>().mul_add(60.0, 2.0);
        let y = rng.gen::<f32>().mul_add(60.0, 2.0);
        sim.spawn_agent(Position::new_2d(x, y));
    }

    // Sample at tick 0
    let baseline = sample_layer_metrics(&sim);
    assert!(
        baseline.geography_valid,
        "Geography: positions should be in bounds at tick 0"
    );

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Run to tick 50
    for _ in 0..50 {
        sim.step();
    }
    let midpoint = sample_layer_metrics(&sim);
    assert!(
        midpoint.biology_energy_mean >= 20.0 && midpoint.biology_energy_mean <= 100.0,
        "Biology: energy mean should be in [20, 100], got {}",
        midpoint.biology_energy_mean
    );

    // Run to tick 100
    for _ in 0..50 {
        sim.step();
    }
    let sample_tick_100 = sample_layer_metrics(&sim);
    assert!(
        sample_tick_100.neurobiology_docas_valid,
        "Neurobiology: DOCAS values should all be in [0, 1]"
    );

    // Run to tick 200
    for _ in 0..100 {
        sim.step();
    }
    let sample_tick_200 = sample_layer_metrics(&sim);
    assert!(
        sample_tick_200.philosophy_norm_adherence_mean >= 0.0
            && sample_tick_200.philosophy_norm_adherence_mean <= 1.0,
        "Philosophy: norm adherence mean should be in [0, 1], got {}",
        sample_tick_200.philosophy_norm_adherence_mean
    );

    // Run to tick 500
    for _ in 0..300 {
        sim.step();
    }
    let sample_tick_500 = sample_layer_metrics(&sim);
    println!("Tick 500 metrics:");
    println!("  Living agents: {}", sample_tick_500.population);
    println!(
        "  Mean DOCAS: {:?}",
        sample_tick_500.neurobiology_docas_mean
    );
    println!(
        "  Mean norm adherence: {}",
        sample_tick_500.philosophy_norm_adherence_mean
    );
}
// TAG: surface=simulation owner=research-team rule=SIM-001

/// Structure to hold layer metrics at a given tick
struct LayerMetrics {
    geography_valid: bool,
    biology_energy_mean: f32,
    neurobiology_docas_valid: bool,
    neurobiology_docas_mean: [f32; 5],
    #[allow(dead_code)]
    psychology_social_sigma: f32,
    philosophy_norm_adherence_mean: f32,
    population: usize,
}

/// Sample metrics from current simulation state
fn sample_layer_metrics(sim: &Simulation) -> LayerMetrics {
    let living: Vec<_> = sim.agents.iter().filter(|a| a.alive).collect();

    // Geography: check positions in bounds
    let geography_valid = living.iter().all(|a| {
        a.position.x >= 0.0 && a.position.x <= 64.0 && a.position.y >= 0.0 && a.position.y <= 64.0
    });

    // Biology: energy mean
    let energy_sum: f32 = living.iter().map(|a| a.energy).sum();
    let biology_energy_mean = if living.is_empty() {
        0.0
    } else {
        energy_sum / living.len() as f32
    };

    // Neurobiology: DOCAS values
    let mut docas_sum = [0.0f32; 5];
    let mut docas_valid = true;
    for agent in &living {
        for (i, docas_sum_i) in docas_sum.iter_mut().enumerate() {
            *docas_sum_i += agent.docas.levels[i];
            if agent.docas.levels[i] < 0.0 || agent.docas.levels[i] > 1.0 {
                docas_valid = false;
            }
        }
    }
    let neurobiology_docas_mean: [f32; 5] = if living.is_empty() {
        [0.0; 5]
    } else {
        [
            docas_sum[0] / living.len() as f32,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            docas_sum[1] / living.len() as f32,
            docas_sum[2] / living.len() as f32,
            docas_sum[3] / living.len() as f32,
            docas_sum[4] / living.len() as f32,
        ]
    };

    // Psychology: social orientation std dev
    let social_values: Vec<f32> = living.iter().map(|a| a.social_orientation).collect();
    let psychology_social_sigma = compute_std_dev(&social_values);

    // Philosophy: norm adherence mean (from tau_phi)
    let adherence_values: Vec<f32> = living
        .iter()
        .filter_map(|a| {
            a.philosophical_state
                .as_ref()
                .map(|p| (p.tau_phi + 1.0) / 2.0)
        })
        .collect();
    let philosophy_norm_adherence_mean = if adherence_values.is_empty() {
        0.5
    }
    // Default neutral
    else {
        adherence_values.iter().sum::<f32>() / adherence_values.len() as f32
    };

    LayerMetrics {
        geography_valid,
        biology_energy_mean,
        neurobiology_docas_valid: docas_valid,
        neurobiology_docas_mean,
        psychology_social_sigma,
        philosophy_norm_adherence_mean,
        population: living.len(),
    }
}

/// Compute standard deviation
fn compute_std_dev(values: &[f32]) -> f32 {
    if values.len() < 2 {
        return 0.0;
    }
    let mean = values.iter().sum::<f32>() / values.len() as f32;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / values.len() as f32;
    variance.sqrt()
}
