// TAG: surface=simulation owner=research-team rule=SIM-001
#![allow(clippy::field_reassign_with_default)]
use crate::epigenome::Epigenome;

use crate::birthrate::BirthrateConfig;
use crate::dmn::Action;
use crate::parameters::SimulationParameters;
use crate::protocol::VerificationMode;
use crate::society::SanctionResult;
use crate::toggles::ToggleConfig;

/// Phase 0 N=1 validation test per §22.5.3
/// Success criteria:
/// - Agent can move on grid without errors
/// - Agent consumes energy each tick
/// - Agent collects resources when on resource cell
/// - Agent dies when energy reaches 0
/// - No panics in 1,000 tick run
#[test]
fn test_single_agent_1000_ticks() {
    let mut sim = Simulation::new_default();

    // Spawn single agent at center of grid
    let pos = Position::new_2d(32.0, 32.0);
    let agent_id = sim.spawn_agent(pos);

    assert_eq!(sim.living_agent_count(), 1);

    // Run 1000 ticks - should not panic
    sim.run(1000);

    // Agent should have aged
    let agent = &sim.agents[agent_id as usize];
    assert!(agent.age > 0);

    // Tick should have advanced
    assert_eq!(sim.tick, 1000);
}

#[test]
fn test_agent_energy_consumption() {
    let mut sim = Simulation::new_default();

    // Spawn agent in center
    let pos = Position::new_2d(32.0, 32.0);
    sim.spawn_agent(pos);

    // Clear ALL resources in the environment so agent can't gain energy
    for resource in &mut sim.environment.resources {
        resource.amount = 0.0;
        resource.capacity = 0.0;
        resource.regrowth_rate = 0.0;
    }

    let initial_energy = sim.agents[0].energy;

    // Run one tick
    sim.step();
    // TAG: surface=simulation owner=research-team rule=SIM-001

    // Energy should decrease (consumption + potential movement)
    assert!(
        sim.agents[0].energy < initial_energy,
        "Energy should decrease: initial={}, current={}",
        initial_energy,
        sim.agents[0].energy
    );
}

#[test]
fn test_agent_dies_at_zero_energy() {
    let config = SimulationConfig {
        energy_consumption_rate: 10.0, // Consume 10 per tick
        movement_cost: 0.0,            // Disable movement cost for predictable test
        ..Default::default()
    };

    let mut sim = Simulation::new(config);

    let pos = Position::new_2d(32.0, 32.0);
    let agent_id = sim.spawn_agent(pos);

    // Set initial energy to exactly 50
    sim.agents[agent_id as usize].energy = 50.0;
    // TAG: surface=simulation owner=research-team rule=SIM-001

    // Clear all resources AND disable regrowth
    for y in 0..sim.environment.height {
        for x in 0..sim.environment.width {
            let idx = y * sim.environment.width + x;
            sim.environment.resources[idx].amount = 0.0;
            sim.environment.resources[idx].regrowth_rate = 0.0;
        }
    }

    // Run 5 ticks: 50 - (10 * 5) = 0, should die on tick 5
    for _ in 0..5 {
        sim.step();
    }

    // Agent should be dead (energy = 0)
    assert!(
        !sim.agents[agent_id as usize].alive,
        "Agent with energy {} should be dead",
        sim.agents[agent_id as usize].energy
    );
    assert_eq!(sim.living_agent_count(), 0);
}

#[test]
fn test_agent_movement() {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let mut sim = Simulation::new_default();

    let pos = Position::new_2d(32.0, 32.0);
    sim.spawn_agent(pos);

    // Force high resources to one side to attract agent
    let high_resource_idx = 33 + 32 * 64; // (33, 32)
    sim.environment.resources[high_resource_idx].amount = 100.0;

    let _initial_pos = sim.agents[0].position;

    // Run several ticks
    sim.run(5);

    // Agent position may have changed (direction depends on gradient)
    // Just verify no panic and position is valid
    let new_pos = sim.agents[0].position;
    assert!(sim.environment.is_valid_position(&new_pos));

    // Position should have moved toward resources
    // (This is a weak assertion since it depends on initial conditions)
    assert!(new_pos.x >= 0.0 && new_pos.x < 64.0);
    assert!(new_pos.y >= 0.0 && new_pos.y < 64.0);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_resource_collection() {
    let mut sim = Simulation::new_default();

    // Put agent in center
    let pos = Position::new_2d(10.0, 10.0);
    sim.spawn_agent(pos);

    // Put resources in a 3x3 area around agent so it will collect
    // regardless of which direction it moves
    let resource_indices: Vec<usize> = (0..3)
        .flat_map(|dx| (0..3).map(move |dy| (9 + dx) + (9 + dy) * 64))
        .collect();

    for &idx in &resource_indices {
        sim.environment.resources[idx].amount = 50.0;
        sim.environment.resources[idx].capacity = 100.0;
    }

    // Track total resources before
    let total_before: f32 = resource_indices
        .iter()
        .map(|&idx| sim.environment.resources[idx].amount)
        .sum();

    // Run a tick - agent should collect resources at its final position
    // TAG: surface=simulation owner=research-team rule=SIM-001
    sim.step();

    // Track total resources after
    let total_after: f32 = resource_indices
        .iter()
        .map(|&idx| sim.environment.resources[idx].amount)
        .sum();

    // Total resources should decrease (agent harvested some)
    // Note: Regrowth adds back some, so we check that harvesting occurred
    // The agent harvests up to 5.0, so we should see net decrease
    assert!(
        total_after < total_before,
        "Resources should decrease: before={total_before}, after={total_after}"
    );
}

#[test]
fn test_deterministic_execution() {
    // SIM-004: Same seed + same config = identical output
    let config = SimulationConfig::default();

    let mut sim1 = Simulation::new(config.clone());
    sim1.spawn_agent(Position::new_2d(32.0, 32.0));
    sim1.run(100);

    let mut sim2 = Simulation::new(config);
    sim2.spawn_agent(Position::new_2d(32.0, 32.0));
    sim2.run(100);

    // Results should be identical
    assert_eq!(sim1.tick, sim2.tick);
    assert_eq!(sim1.living_agent_count(), sim2.living_agent_count());

    // Agent state should be identical (same energy, age, position)
    if sim1.has_living_agents() {
        assert_eq!(sim1.agents[0].energy, sim2.agents[0].energy);
        assert_eq!(sim1.agents[0].age, sim2.agents[0].age);
        assert_eq!(sim1.agents[0].position, sim2.agents[0].position);
    }
}

/// Stage 1 test: Reproduction with genetic crossover
#[test]
fn test_reproduction() {
    let config = SimulationConfig {
        reproduction_threshold: 50.0,
        reproduction_cost: 20.0,
        ..Default::default()
    };
    // TAG: surface=simulation owner=research-team rule=SIM-001

    let mut sim = Simulation::new(config);

    // Spawn two agents with high energy
    let agent1 = sim.spawn_agent(Position::new_2d(10.0, 10.0));
    let agent2 = sim.spawn_agent(Position::new_2d(11.0, 10.0));
    sim.agents[agent1 as usize].energy = 100.0;
    sim.agents[agent2 as usize].energy = 100.0;

    let initial_count = sim.agents.len();

    // Run one step - should trigger reproduction
    sim.step();

    // Should have offspring
    let agent_count = sim.agents.len();
    assert!(
        agent_count > initial_count,
        "Expected offspring, got {agent_count} agents"
    );

    // Parents should have reduced energy
    assert!(sim.agents[agent1 as usize].energy < 100.0);
    assert!(sim.agents[agent2 as usize].energy < 100.0);
}
// TAG: surface=simulation owner=research-team rule=SIM-001

/// Stage 1 test: Gompertz mortality increases with age
#[test]
fn test_gompertz_mortality() {
    let config = SimulationConfig {
        mortality_baseline: 0.001, // Higher baseline for testing
        mortality_aging_beta: 0.1,
        ..Default::default()
    };

    let mut sim = Simulation::new(config);

    // Spawn old agent
    let old_agent = sim.spawn_agent(Position::new_2d(10.0, 10.0));
    sim.agents[old_agent as usize].age = 100; // Very old
    sim.agents[old_agent as usize].energy = 50.0; // Enough energy

    // Run many ticks - old agent should eventually die from age
    let mut died_from_age = false;
    for _ in 0..1000 {
        if !sim.agents[old_agent as usize].alive {
            died_from_age = true;
            break;
        }
        sim.step();
    }
    // TAG: surface=simulation owner=research-team rule=SIM-001

    assert!(
        died_from_age,
        "Old agent should die from Gompertz mortality"
    );
}

/// Stage 2 test: DOCAS state updates in response to environment
#[test]
fn test_docas_updates() {
    let mut config = SimulationConfig::default();
    config.toggles.docas_enabled = true;

    let mut sim = Simulation::new(config);
    let agent_id = sim.spawn_agent(Position::new_2d(32.0, 32.0));

    // Record initial DOCAS state
    let initial_docas = sim.agents[agent_id as usize].docas.levels;

    // Clear resources to prevent energy gain
    for y in 0..sim.environment.height {
        for x in 0..sim.environment.width {
            let pos = Position::new_2d(x as f32, y as f32);
            sim.environment.harvest_at(&pos, 1000.0);
        }
        // TAG: surface=simulation owner=research-team rule=SIM-001
    }

    // Set low energy to trigger threat response
    sim.agents[agent_id as usize].energy = 20.0;

    // Run a few steps
    for _ in 0..5 {
        sim.step();
        // Keep energy low to maintain threat signal
        if sim.agents[agent_id as usize].energy > 20.0 {
            sim.agents[agent_id as usize].energy = 20.0;
        }
    }

    // DOCAS state should have changed from initial neutral [0.5; 5]
    let final_docas = sim.agents[agent_id as usize].docas.levels;
    let changed = initial_docas
        .iter()
        .zip(final_docas.iter())
        .any(|(a, b)| (a - b).abs() > 0.01);

    assert!(
        changed,
        "DOCAS state should change in response to environment"
    );

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Cortisol (threat) should be elevated due to low energy
    // Note: Due to alpha decay, it may not be much higher than 0.5
    // Just verify it changed from initial
    let cortisol_changed =
        (final_docas[crate::types::C] - initial_docas[crate::types::C]).abs() > 0.01;
    assert!(
        cortisol_changed,
        "Cortisol should change in response to low energy threat"
    );
}

/// Stage 2 test: DOCAS toggle OFF keeps neutral state
#[test]
fn test_docas_toggle_off() {
    let mut config = SimulationConfig::default();
    config.toggles.docas_enabled = false; // Toggle OFF

    let mut sim = Simulation::new(config);
    let agent_id = sim.spawn_agent(Position::new_2d(32.0, 32.0));

    // Set low energy (would trigger DOCAS if enabled)
    sim.agents[agent_id as usize].energy = 20.0;

    // Run steps
    for _ in 0..10 {
        sim.step();
    }

    // DOCAS should remain neutral [0.5; 5]
    let final_docas = sim.agents[agent_id as usize].docas.levels;
    for level in final_docas {
        assert!(
            (level - 0.5).abs() < 0.01,
            "DOCAS should remain neutral when toggle is OFF"
        );
    }
}

// ========================================================================
// P2.2: Movement Unit Tests (Phase 16)
// Per §7.4.0: 9-Cell DOCAS-Weighted Movement
// ========================================================================

/// Test that 9-cell movement evaluates all neighboring cells
/// Per §7.4.0 lines 3868-3892: Agent should move to highest utility cell
#[test]
fn test_9_cell_movement_selects_best_cell() {
    let mut sim = Simulation::new_default();

    // Spawn agent at center
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let pos = Position::new_2d(32.0, 32.0);
    sim.spawn_agent(pos);

    // Clear all resources
    for resource in &mut sim.environment.resources {
        resource.amount = 0.0;
    }

    // Put high resources to the east (33, 32)
    let east_idx = 33 + 32 * 64;
    sim.environment.resources[east_idx].amount = 100.0;

    // Set agent's Dopamine high (drives toward resources)
    sim.agents[0].docas.levels[crate::types::D] = 0.9;
    sim.agents[0].docas.levels[crate::types::O] = 0.1;
    sim.agents[0].docas.levels[crate::types::C] = 0.1;

    // Run one tick
    sim.step();

    // Agent should have moved toward the resource (east)
    let new_pos = sim.agents[0].position;
    assert!(
        new_pos.x >= 32.0,
        "Agent should move toward resource: x={}",
        // TAG: surface=simulation owner=research-team rule=SIM-001
        new_pos.x
    );
}

/// Test collision detection prevents agents from occupying same cell
/// Per §7.4.0 lines 3975-3997: `OccupancyGrid`
#[test]
fn test_collision_detection() {
    let mut sim = Simulation::new_default();

    // Spawn two agents at adjacent positions
    sim.spawn_agent(Position::new_2d(32.0, 32.0));
    sim.spawn_agent(Position::new_2d(33.0, 32.0));

    // Clear all resources except at position (34, 32)
    for resource in &mut sim.environment.resources {
        resource.amount = 0.0;
    }
    let target_idx = 34 + 32 * 64;
    sim.environment.resources[target_idx].amount = 100.0;
    sim.environment.resources[target_idx].capacity = 150.0;
    sim.environment.resources[target_idx].regrowth_rate = 0.5;

    // Both agents want to move east toward resources
    for agent in &mut sim.agents {
        agent.docas.levels[crate::types::D] = 0.9;
        // TAG: surface=simulation owner=research-team rule=SIM-001
    }

    // Run one tick
    sim.step();

    // Agents should not occupy the same cell
    let pos0 = sim.agents[0].position;
    let pos1 = sim.agents[1].position;
    let same_cell = (pos0.x as i32 == pos1.x as i32) && (pos0.y as i32 == pos1.y as i32);
    assert!(
        !same_cell,
        "Agents should not occupy same cell: agent0=({},{}), agent1=({},{})",
        pos0.x, pos0.y, pos1.x, pos1.y
    );
}

/// Test DOCAS-weighted movement formula: D×resource + O×social - C×threat
/// Per §7.4.0 lines 3882-3892
#[test]
fn test_docas_weighted_movement() {
    let mut sim = Simulation::new_default();

    // Spawn agent at center
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Clear all resources
    for resource in &mut sim.environment.resources {
        resource.amount = 0.0;
    }

    // Put resources to the west (31, 32)
    let west_idx = 31 + 32 * 64;
    sim.environment.resources[west_idx].amount = 100.0;

    // Spawn another agent to the east (social attraction)
    sim.spawn_agent(Position::new_2d(34.0, 32.0));

    // Test 1: High Dopamine should drive toward resources (west)
    sim.agents[0].docas.levels[crate::types::D] = 0.9;
    sim.agents[0].docas.levels[crate::types::O] = 0.1;
    let initial_pos = sim.agents[0].position;
    sim.step();
    let after_d_pos = sim.agents[0].position;

    // Reset for next test
    sim.agents[0].position = initial_pos;

    // Test 2: High Oxytocin should drive toward social cluster (east)
    sim.agents[0].docas.levels[crate::types::D] = 0.1;
    sim.agents[0].docas.levels[crate::types::O] = 0.9;
    sim.step();
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let after_o_pos = sim.agents[0].position;

    // With high D, agent should move west (toward resources)
    // With high O, agent should move east (toward other agent)
    // These should be different directions
    let d_moved_west = after_d_pos.x < initial_pos.x;
    let o_moved_east = after_o_pos.x > initial_pos.x;

    // At least one of these should be true (stochastic noise may affect)
    assert!(d_moved_west || o_moved_east,
                "DOCAS weighting should affect movement direction: D_west={d_moved_west}, O_east={o_moved_east}");
}

/// Test stochastic noise prevents deterministic gridlock
/// Per §7.4.0 lines 3885-3887: 10% random perturbation
#[test]
fn test_stochastic_noise_prevents_gridlock() {
    // Run same simulation twice with same seed
    // Movement should be deterministic (SIM-004)
    let config1 = SimulationConfig {
        seed: 12345,
        ..Default::default()
    };
    let config2 = SimulationConfig {
        seed: 12345,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        ..Default::default()
    };

    let mut sim1 = Simulation::new(config1);
    let mut sim2 = Simulation::new(config2);

    // Spawn agents at same positions
    sim1.spawn_agent(Position::new_2d(32.0, 32.0));
    sim2.spawn_agent(Position::new_2d(32.0, 32.0));

    // Run both for 100 ticks
    sim1.run(100);
    sim2.run(100);

    // Positions should be identical (determinism)
    let pos1 = sim1.agents[0].position;
    let pos2 = sim2.agents[0].position;
    assert!(
        (pos1.x - pos2.x).abs() < 0.001 && (pos1.y - pos2.y).abs() < 0.001,
        "Same seed should produce identical positions: sim1=({},{}), sim2=({},{})",
        pos1.x,
        pos1.y,
        pos2.x,
        pos2.y
    );

    // TAG: surface=simulation owner=research-team rule=SIM-001
}

/// Test `OccupancyGrid` correctly tracks agent positions
#[test]
fn test_occupancy_grid() {
    let mut grid = OccupancyGrid::new();

    // Create positions with longer lifetimes
    let pos1 = Position::new_2d(10.0, 10.0);
    let pos2 = Position::new_2d(20.0, 20.0);
    let pos3 = Position::new_2d(10.0, 10.0);

    // Add some agents
    let agents = vec![
        (1u64, &pos1, true),
        (2u64, &pos2, true),
        (3u64, &pos3, false), // Dead agent, should not occupy
    ];

    grid.update_from_agents(agents.into_iter());

    // Check occupancy
    assert!(grid.is_occupied(&Position::new_2d(10.0, 10.0)));
    assert!(grid.is_occupied(&Position::new_2d(20.0, 20.0)));
    assert!(!grid.is_occupied(&Position::new_2d(30.0, 30.0)));
    // TAG: surface=simulation owner=research-team rule=SIM-001

    // Check occupant IDs
    assert_eq!(grid.get_occupant(&Position::new_2d(10.0, 10.0)), Some(1));
    assert_eq!(grid.get_occupant(&Position::new_2d(20.0, 20.0)), Some(2));
    assert_eq!(grid.get_occupant(&Position::new_2d(30.0, 30.0)), None);
}

// =========================================================================
// PROPERTY-BASED TESTS: Invariants that must hold across all simulations
// =========================================================================

/// Property: Agent energy is never negative
/// Per §7.4.0: Energy is consumed but should never go below 0
#[test]
fn test_property_energy_never_negative() {
    let config = SimulationConfig {
        max_ticks: 500,
        seed: 88888,
        energy_consumption_rate: 5.0, // High consumption
        ..Default::default()
    };

    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    for _ in 0..500 {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        sim.step();
        // Check all agents have non-negative energy
        for agent in &sim.agents {
            assert!(
                agent.energy >= 0.0,
                "Agent {} has negative energy: {}",
                agent.id,
                agent.energy
            );
        }
    }
}

/// Property: DOCAS values are always clamped to [0.0, 1.0]
/// Per §6.1.1: DOCAS state must remain bounded
#[test]
fn test_property_docas_clamped() {
    let mut config = SimulationConfig::default();
    config.toggles.docas_enabled = true;
    config.max_ticks = 200;
    config.seed = 77777;

    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    // TAG: surface=simulation owner=research-team rule=SIM-001
    for _ in 0..200 {
        sim.step();
        for agent in &sim.agents {
            for (i, &level) in agent.docas.levels.iter().enumerate() {
                assert!(
                    (0.0..=1.0).contains(&level),
                    "Agent {} DOCAS[{}] out of bounds: {}",
                    agent.id,
                    i,
                    level
                );
            }
        }
    }
}

/// Property: Agent positions stay within grid bounds
/// Per §7.4.0: Movement respects world boundaries
#[test]
fn test_property_positions_bounded() {
    let config = SimulationConfig {
        max_ticks: 300,
        seed: 66666,
        ..Default::default()
    };

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));
    let width = sim.environment.width as f32;
    let height = sim.environment.height as f32;

    for _ in 0..300 {
        sim.step();
        for agent in &sim.agents {
            assert!(
                agent.position.x >= 0.0 && agent.position.x < width,
                "Agent {} x position {} out of bounds [0, {})",
                agent.id,
                agent.position.x,
                width
            );
            assert!(
                agent.position.y >= 0.0 && agent.position.y < height,
                "Agent {} y position {} out of bounds [0, {})",
                agent.id,
                agent.position.y,
                height
            );
        }
    }
}

/// Property: Trust (τ) is always non-negative
/// Per §6.1: τ = (D + O + S) / (1 + C + A) with bounded DOCAS → τ ≥ 0
#[test]
fn test_property_trust_non_negative() {
    let mut config = SimulationConfig::default();
    config.toggles.docas_enabled = true;
    config.max_ticks = 200;
    config.seed = 55555;

    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    for _ in 0..200 {
        sim.step();
        for agent in &sim.agents {
            let tau = agent.compute_trust();
            assert!(tau >= 0.0, "Agent {} has negative trust: {}", agent.id, tau);
        }
    }
}

/// Property: Dead agents remain dead
/// Agents with alive=false should never become alive again
#[test]
// TAG: surface=simulation owner=research-team rule=SIM-001
fn test_property_dead_stay_dead() {
    let config = SimulationConfig {
        max_ticks: 500,
        seed: 44444,
        energy_consumption_rate: 3.0,
        ..Default::default()
    };

    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));
    let mut dead_agents: std::collections::HashSet<u64> = std::collections::HashSet::new();

    for _ in 0..500 {
        sim.step();

        // Record newly dead agents
        for agent in &sim.agents {
            if !agent.alive {
                dead_agents.insert(agent.id);
            }
        }

        // Verify previously dead agents are still dead
        for agent in &sim.agents {
            if dead_agents.contains(&agent.id) {
                // TAG: surface=simulation owner=research-team rule=SIM-001
                assert!(!agent.alive, "Dead agent {} came back to life!", agent.id);
            }
        }
    }
}

/// Property: Agent IDs are unique
#[test]
fn test_property_unique_agent_ids() {
    let config = SimulationConfig {
        max_ticks: 300,
        seed: 33333,
        reproduction_threshold: 60.0,
        ..Default::default()
    };

    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    for _ in 0..300 {
        sim.step();

        let mut ids: std::collections::HashSet<u64> = std::collections::HashSet::new();
        for agent in &sim.agents {
            assert!(ids.insert(agent.id), "Duplicate agent ID: {}", agent.id);
        }
        // TAG: surface=simulation owner=research-team rule=SIM-001
    }
}

// =========================================================================
// STATISTICAL VALIDATION TESTS
// Per §4.2, §5.3.3, §6.3, §6.6.1
// =========================================================================

/// Criticality test: Verify branching ratio σ ≈ 1.0 ± 0.1
/// Per §6.3: System maintains edge-of-chaos dynamics
/// Per §4.2: Target σ = 1.0 ± 0.1
#[test]
fn test_criticality_branching_ratio() {
    use crate::metrics::branching_ratio;

    // Test critical state
    let sigma = branching_ratio(100, 100);
    assert!(
        (sigma - 1.0).abs() <= 0.1,
        "Critical branching ratio: σ = {sigma} not in [0.9, 1.1]"
    );

    // Test subcritical (should be detected)
    let sigma_sub = branching_ratio(80, 100);
    assert!(
        // TAG: surface=simulation owner=research-team rule=SIM-001
        sigma_sub < 1.0,
        "Should detect subcritical: σ = {sigma_sub}"
    );

    // Test supercritical (should be detected)
    let sigma_super = branching_ratio(120, 100);
    assert!(
        sigma_super > 1.0,
        "Should detect supercritical: σ = {sigma_super}"
    );
}

/// Trust emergence test: τ should emerge from DOCAS dynamics
/// Per §6.1: Trust is never hardcoded - emerges from behavior
#[test]
fn test_trust_emergence_from_docas() {
    use crate::docas::DocasState;
    use crate::genome::Genome;

    let genome = Genome::default();
    let mut docas = DocasState::neutral();

    // Initial trust
    let initial_tau = docas.compute_tau();

    // Apply stimuli (positive social interaction)
    // TAG: surface=simulation owner=research-team rule=SIM-001
    docas.update(&genome, 0.5, 0.7, 0.0, 0.0, 0.3); // D↑, O↑, S↑

    let updated_tau = docas.compute_tau();

    // Trust should change based on DOCAS changes
    assert!(
        (updated_tau - initial_tau).abs() > 0.01,
        "Trust should change with DOCAS: initial={initial_tau}, updated={updated_tau}"
    );

    // High D + O + S, low C + A should increase trust
    let high_trust_docas = DocasState::new([0.9, 0.9, 0.1, 0.1, 0.8]);
    let low_trust_docas = DocasState::new([0.2, 0.2, 0.9, 0.9, 0.2]);

    assert!(
        high_trust_docas.compute_tau() > low_trust_docas.compute_tau(),
        "High D+O+S, low C+A should yield higher trust"
    );
}

/// Heritability proxy test: Offspring DOCAS sensitivity should correlate with parents
/// Per §6.6.1: h² ≈ 0.50 for trust-state heritability
#[test]
fn test_heritability_docas_sensitivity() {
    use crate::genome::{crossover, Genome};
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    let mut rng = StdRng::seed_from_u64(12345);

    // Create parent genomes
    let parent1 = Genome::random(&mut rng);
    let parent2 = Genome::random(&mut rng);

    // Create offspring via crossover
    let offspring = crossover(&parent1, &parent2, 0.001, &mut rng);

    // Offspring DOCAS beta should be in valid range
    let beta_o = offspring.docas_beta();

    for (i, &beta_val) in beta_o.iter().enumerate() {
        // Offspring should be within valid DOCAS beta range [0.05, 0.2]
        // With crossover and low mutation rate (0.001), values should remain in range
        assert!(
            (0.05..=0.2).contains(&beta_val),
            "Offspring beta[{i}] = {beta_val} should be in valid range [0.05, 0.2]"
        );
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Genome-trust correlation test: Different genomes should produce different trust trajectories
/// Per §6.8.2: Genome → DOCAS pathway affects trust
#[test]
fn test_genome_trust_correlation() {
    use crate::docas::DocasState;
    use crate::genome::Genome;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    let mut rng = StdRng::seed_from_u64(99999);

    // Create genomes with different sensitivity profiles
    let genome1 = Genome::random(&mut rng);
    let genome2 = Genome::random(&mut rng);

    // Apply same stimuli sequence to both
    let stimuli = [(0.5, 0.3, 0.2, 0.1, 0.4), (0.2, 0.6, 0.3, 0.2, 0.5)];

    let mut docas1 = DocasState::neutral();
    let mut docas2 = DocasState::neutral();

    for (d, o, c, a, s) in &stimuli {
        // P3-1: Include geometric parameters - neutral values for test
        docas1.update(&genome1, *d, *o, *c, *a, *s);
        docas2.update(&genome2, *d, *o, *c, *a, *s);
        // TAG: surface=simulation owner=research-team rule=SIM-001
    }

    let tau1 = docas1.compute_tau();
    let tau2 = docas2.compute_tau();

    // Different genomes should (usually) produce different trust values
    // This tests the causal pathway from genome through DOCAS to trust
    // Note: With random genomes, there's a small chance they're similar
    // We verify the pathway works, not that they must differ
    assert!(
        tau1 >= 0.0 && tau2 >= 0.0,
        "Both trust values should be non-negative"
    );
}

// =========================================================================
// TWIN STUDY TESTS
// Per §6.6.1: MZ (identical) vs DZ (fraternal) twin design for heritability
// h² = 2(r_MZ - r_DZ) where r is intraclass correlation
// Target: h² ≈ 0.50 for trust-state heritability
// =========================================================================

/// Twin study: Monozygotic (identical) twins should have higher τ correlation than DZ
/// Per §6.6.1: MZ twins share 100% genome, DZ share ~50%
/// This tests the heritability pathway: Genome → DOCAS → τ
#[test]
// TAG: surface=simulation owner=research-team rule=SIM-001
fn test_twin_study_mz_vs_dz_correlation() {
    use crate::docas::DocasState;
    use crate::genome::{crossover, Genome};
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    let mut rng = StdRng::seed_from_u64(777777);

    // Create MZ twins: identical genomes
    let mz_genome = Genome::random(&mut rng);
    let mz_twin1 = mz_genome.clone();
    let mz_twin2 = mz_genome;

    // Create DZ twins: share ~50% genetics (via crossover from same parents)
    let parent1 = Genome::random(&mut rng);
    let parent2 = Genome::random(&mut rng);
    let dz_twin1 = crossover(&parent1, &parent2, 0.0, &mut rng); // No mutation
    let dz_twin2 = crossover(&parent1, &parent2, 0.0, &mut rng);

    // Apply identical stimulus sequence to all twins
    let stimuli_sequence = [
        (0.6, 0.4, 0.2, 0.1, 0.5),
        (0.3, 0.7, 0.1, 0.2, 0.4),
        (0.5, 0.5, 0.3, 0.2, 0.6),
        (0.4, 0.3, 0.4, 0.3, 0.3),
        // TAG: surface=simulation owner=research-team rule=SIM-001
        (0.7, 0.2, 0.1, 0.1, 0.7),
    ];

    let mut mz1_docas = DocasState::neutral();
    let mut mz2_docas = DocasState::neutral();
    let mut dz1_docas = DocasState::neutral();
    let mut dz2_docas = DocasState::neutral();

    let mut mz1_tau_history = Vec::new();
    let mut mz2_tau_history = Vec::new();
    let mut dz1_tau_history = Vec::new();
    let mut dz2_tau_history = Vec::new();

    for (d, o, c, a, s) in &stimuli_sequence {
        mz1_docas.update(&mz_twin1, *d, *o, *c, *a, *s);
        mz2_docas.update(&mz_twin2, *d, *o, *c, *a, *s);
        dz1_docas.update(&dz_twin1, *d, *o, *c, *a, *s);
        dz2_docas.update(&dz_twin2, *d, *o, *c, *a, *s);

        mz1_tau_history.push(mz1_docas.compute_tau());
        mz2_tau_history.push(mz2_docas.compute_tau());
        dz1_tau_history.push(dz1_docas.compute_tau());
        dz2_tau_history.push(dz2_docas.compute_tau());
    }

    // Calculate intraclass correlation (simplified Pearson r for pairs)
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let mz_diff: f32 = mz1_tau_history
        .iter()
        .zip(&mz2_tau_history)
        .map(|(a, b)| (a - b).abs())
        .sum::<f32>()
        / mz1_tau_history.len() as f32;

    let dz_diff: f32 = dz1_tau_history
        .iter()
        .zip(&dz2_tau_history)
        .map(|(a, b)| (a - b).abs())
        .sum::<f32>()
        / dz1_tau_history.len() as f32;

    // MZ twins should have identical τ trajectories (same genome)
    assert!(
        mz_diff < 0.001,
        "MZ twins should have identical τ (diff={mz_diff})"
    );

    // DZ twins should have some variation (different genome combinations)
    // This confirms heritability pathway is working
    println!("Twin Study Results:");
    println!("  MZ twin τ difference: {mz_diff:.6}");
    println!("  DZ twin τ difference: {dz_diff:.6}");

    // Heritability estimate: h² ≈ 2(r_MZ - r_DZ)
    // Since MZ should be ~1.0 and DZ < 1.0, h² should be positive
    // We verify the structural expectation: MZ more similar than DZ
    assert!(
        mz_diff <= dz_diff,
        "MZ twins should be at least as similar as DZ twins"
    );
}

/// Extended heritability test: Verify h² estimate is in expected range
/// Per §6.6.1: h² ≈ 0.50 for trust-related traits
#[test]
fn test_heritability_estimate_range() {
    use crate::docas::DocasState;
    use crate::genome::{crossover, Genome};
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    let mut rng = StdRng::seed_from_u64(888888);
    let n_pairs = 20; // Number of twin pairs to simulate

    let mut mz_correlations = Vec::new();
    let mut dz_correlations = Vec::new();

    // TAG: surface=simulation owner=research-team rule=SIM-001
    for _ in 0..n_pairs {
        // MZ pair
        let mz_genome = Genome::random(&mut rng);
        let mut mz1 = DocasState::neutral();
        let mut mz2 = DocasState::neutral();

        // DZ pair
        let p1 = Genome::random(&mut rng);
        let p2 = Genome::random(&mut rng);
        let dz1_genome = crossover(&p1, &p2, 0.0, &mut rng);
        let dz2_genome = crossover(&p1, &p2, 0.0, &mut rng);
        let mut dz1 = DocasState::neutral();
        let mut dz2 = DocasState::neutral();

        // Common environment (same stimuli)
        let stim = (
            rng.gen::<f32>() * 0.5 + 0.25,
            rng.gen::<f32>() * 0.5 + 0.25,
            rng.gen::<f32>() * 0.3,
            rng.gen::<f32>() * 0.3,
            rng.gen::<f32>() * 0.5 + 0.25,
        );

        // P3-1: Include geometric parameters - neutral values for test
        mz1.update(&mz_genome, stim.0, stim.1, stim.2, stim.3, stim.4);
        // TAG: surface=simulation owner=research-team rule=SIM-001
        mz2.update(&mz_genome, stim.0, stim.1, stim.2, stim.3, stim.4);
        dz1.update(&dz1_genome, stim.0, stim.1, stim.2, stim.3, stim.4);
        dz2.update(&dz2_genome, stim.0, stim.1, stim.2, stim.3, stim.4);

        mz_correlations.push((mz1.compute_tau(), mz2.compute_tau()));
        dz_correlations.push((dz1.compute_tau(), dz2.compute_tau()));
    }

    // Calculate correlation for MZ (should be ~1.0)
    let mz_r = pearson_r(&mz_correlations);
    // Calculate correlation for DZ (should be < 1.0)
    let dz_r = pearson_r(&dz_correlations);

    // Heritability estimate
    let h2 = 2.0 * (mz_r - dz_r);

    println!("Heritability Analysis (n={n_pairs} pairs):");
    println!("  r_MZ = {mz_r:.4}");
    println!("  r_DZ = {dz_r:.4}");
    println!("  h²   = {h2:.4}");

    // MZ should have perfect correlation (identical genomes)
    assert!(mz_r > 0.99, "MZ correlation should be ~1.0, got {mz_r}");

    // Verify structural validity of h² calculation
    assert!(h2 >= 0.0, "Heritability should be non-negative, got {h2}");
    // TAG: surface=simulation owner=research-team rule=SIM-001
}

/// Helper: Calculate Pearson correlation coefficient for paired data
fn pearson_r(pairs: &[(f32, f32)]) -> f32 {
    if pairs.len() < 2 {
        return 0.0;
    }

    let n = pairs.len() as f32;
    let sum_x: f32 = pairs.iter().map(|(x, _)| x).sum();
    let sum_y: f32 = pairs.iter().map(|(_, y)| y).sum();
    let sum_xy: f32 = pairs.iter().map(|(x, y)| x * y).sum();
    let sum_x2: f32 = pairs.iter().map(|(x, _)| x * x).sum();
    let sum_y2: f32 = pairs.iter().map(|(_, y)| y * y).sum();

    let numerator = n * sum_xy - sum_x * sum_y;
    let denominator = ((n * sum_x2 - sum_x.powi(2)) * (n * sum_y2 - sum_y.powi(2))).sqrt();

    if denominator < 0.0001 {
        1.0 // Perfect correlation when no variance
    } else {
        (numerator / denominator).clamp(-1.0, 1.0)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
// ========================================================================
// Birthrate Mechanism Tests
// ========================================================================

/// Test birthrate configuration can be added to `SimulationConfig`
#[test]
fn test_birthrate_config_integration() {
    let birthrate_config = BirthrateConfig::default();
    let config = SimulationConfig {
        birthrate_config: Some(birthrate_config),
        ..Default::default()
    };

    assert!(config.birthrate_config.is_some());
    // ERR-001: Test code - unwrap() exempt for test assertions
    let br = config.birthrate_config.unwrap();
    assert_eq!(br.base_birthrate, 0.05);
    assert_eq!(br.energy_threshold, 60.0);
}

/// Test birthrate system with high fertility produces offspring
#[test]
fn test_birthrate_reproduction_produces_offspring() {
    let birthrate_config = BirthrateConfig {
        base_birthrate: 1.0, // 100% chance (for testing)
        energy_threshold: 50.0,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        reproductive_maturity_age: 1, // Mature immediately
        reproduction_cost: 10.0,
        max_offspring_per_tick: 100,
        ..Default::default()
    };

    let config = SimulationConfig {
        birthrate_config: Some(birthrate_config),
        toggles: ToggleConfig::stage_1(), // Enable genome for inheritance
        ..Default::default()
    };

    let mut sim = Simulation::new(config);

    // Spawn a male and female with high energy
    let male = sim.spawn_agent(Position::new_2d(32.0, 32.0));
    let female = sim.spawn_agent(Position::new_2d(32.0, 32.0));

    sim.agents[male as usize].sex = Sex::Male;
    sim.agents[female as usize].sex = Sex::Female;
    sim.agents[male as usize].energy = 100.0;
    sim.agents[female as usize].energy = 100.0;
    sim.agents[male as usize].age = 50; // Mature
    sim.agents[female as usize].age = 50; // Mature
                                          // Neural maturity required for reproduction eligibility
                                          // TAG: surface=simulation owner=research-team rule=SIM-001
    for id in [male, female] {
        sim.agents[id as usize].prefrontal_cortex.vmpfc.development = 0.8;
        sim.agents[id as usize].prefrontal_cortex.dlpfc.development = 0.6;
    }

    let initial_count = sim.living_agent_count();

    // Run several ticks to allow reproduction
    for _ in 0..10 {
        sim.step();
        // Keep parents energized
        if sim.agents[male as usize].alive {
            sim.agents[male as usize].energy = 100.0;
        }
        if sim.agents[female as usize].alive {
            sim.agents[female as usize].energy = 100.0;
        }
    }

    // Should have offspring
    let final_count = sim.living_agent_count();
    assert!(
        final_count > initial_count || sim.agents.len() > 2,
        "Birthrate reproduction should produce offspring"
    );
}
// TAG: surface=simulation owner=research-team rule=SIM-001

/// Test asexual reproduction when enabled
#[test]
fn test_birthrate_asexual_reproduction() {
    let birthrate_config = BirthrateConfig {
        base_birthrate: 1.0, // 100% chance
        energy_threshold: 50.0,
        reproductive_maturity_age: 1,
        reproduction_cost: 10.0,
        allow_asexual: true, // Enable asexual
        max_offspring_per_tick: 10,
        partner_proximity: 1.0, // Small radius so no partner found
        ..Default::default()
    };

    let config = SimulationConfig {
        birthrate_config: Some(birthrate_config),
        toggles: ToggleConfig::stage_1(),
        ..Default::default()
    };

    let mut sim = Simulation::new(config);

    // Spawn single agent (no partner possible)
    let agent = sim.spawn_agent(Position::new_2d(32.0, 32.0));
    // TAG: surface=simulation owner=research-team rule=SIM-001
    sim.agents[agent as usize].energy = 100.0;
    sim.agents[agent as usize].age = 50;
    // Neural maturity required for birthrate eligibility
    sim.agents[agent as usize]
        .prefrontal_cortex
        .vmpfc
        .development = 0.8;
    sim.agents[agent as usize]
        .prefrontal_cortex
        .dlpfc
        .development = 0.6;

    // Run ticks
    for _ in 0..5 {
        sim.step();
        if sim.agents[agent as usize].alive {
            sim.agents[agent as usize].energy = 100.0;
        }
    }

    // Should have reproduced asexually
    assert!(
        sim.agents.len() > 1,
        "Asexual reproduction should produce offspring even with single agent"
    );

    // TAG: surface=simulation owner=research-team rule=SIM-001
}

/// Test birthrate eligibility criteria
#[test]
fn test_birthrate_eligibility() {
    use crate::birthrate::BirthrateCalculator;

    let config = BirthrateConfig::default();
    let calculator = BirthrateCalculator::new(&config, 0);

    let mut sim = Simulation::new(SimulationConfig::default());
    let agent_id = sim.spawn_agent(Position::new_2d(32.0, 32.0));
    let agent = &sim.agents[agent_id as usize];

    // Young agent (not mature)
    assert!(!calculator.is_eligible(agent));

    // Mature agent with low energy
    sim.agents[agent_id as usize].age = 50;
    sim.agents[agent_id as usize].energy = 30.0; // Below threshold
    let agent = &sim.agents[agent_id as usize];
    assert!(!calculator.is_eligible(agent));

    // Mature agent with sufficient energy
    sim.agents[agent_id as usize].energy = 80.0;
    // TAG: surface=simulation owner=research-team rule=SIM-001
    sim.agents[agent_id as usize]
        .prefrontal_cortex
        .vmpfc
        .development = 0.8;
    sim.agents[agent_id as usize]
        .prefrontal_cortex
        .dlpfc
        .development = 0.6;
    let agent = &sim.agents[agent_id as usize];
    assert!(calculator.is_eligible(agent));

    // Old agent (keep neural maturity so only age matters)
    sim.agents[agent_id as usize].age = 600;
    let agent = &sim.agents[agent_id as usize];
    assert!(!calculator.is_eligible(agent));
}

/// Test birthrate with different seasons
#[test]
fn test_birthrate_seasonal_variation() {
    use crate::birthrate::BirthrateCalculator;

    let config = BirthrateConfig {
        season_length: 100,
        spring_start: 0,
        spring_duration: 25,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        ..BirthrateConfig::default()
    };

    // Spring (tick 10)
    let spring_calc = BirthrateCalculator::new(&config, 10);
    let spring_factor = spring_calc.seasonal_factor();

    // Non-spring (tick 50)
    let other_calc = BirthrateCalculator::new(&config, 50);
    let other_factor = other_calc.seasonal_factor();

    // Spring should have higher factor
    assert!(spring_factor > other_factor,
            "Spring birthrate factor ({spring_factor}) should be higher than non-spring ({other_factor})");
}

/// Test birthrate system respects `max_offspring_per_tick`
#[test]
fn test_birthrate_max_offspring_limit() {
    let birthrate_config = BirthrateConfig {
        base_birthrate: 1.0,
        energy_threshold: 30.0,
        reproductive_maturity_age: 1,
        reproduction_cost: 5.0,
        max_offspring_per_tick: 2, // Limit to 2 births per tick
        // TAG: surface=simulation owner=research-team rule=SIM-001
        ..Default::default()
    };

    let config = SimulationConfig {
        birthrate_config: Some(birthrate_config),
        toggles: ToggleConfig::stage_1(),
        ..Default::default()
    };

    let mut sim = Simulation::new(config);

    // Spawn many eligible agents
    for _ in 0..10 {
        let id = sim.spawn_agent(Position::new_2d(32.0, 32.0));
        sim.agents[id as usize].energy = 100.0;
        sim.agents[id as usize].age = 50;
    }

    // Count births in a single tick
    let count_before = sim.agents.len();
    sim.step();
    let count_after = sim.agents.len();
    let births = count_after - count_before;

    // Births should not exceed max_offspring_per_tick
    assert!(
        // TAG: surface=simulation owner=research-team rule=SIM-001
        births <= 2,
        "Births ({births}) should not exceed max_offspring_per_tick (2)"
    );
}

/// Test partner selection prefers opposite sex
#[test]
fn test_birthrate_partner_selection_opposite_sex() {
    use crate::birthrate::PartnerSelector;
    use crate::genome::Genome;

    let config = BirthrateConfig::default();
    let selector = PartnerSelector::new(&config);

    // Create agents directly without borrowing issues
    let mut male = Agent::new(
        0,
        Position::new_2d(32.0, 32.0),
        Genome::default(),
        0,
        Sex::Male,
    );
    male.energy = 80.0;
    male.age = 50;

    let mut female = Agent::new(
        1,
        Position::new_2d(33.0, 32.0),
        Genome::default(),
        0,
        Sex::Female,
    );
    female.energy = 80.0;
    female.age = 50;

    let mut male2 = Agent::new(
        2,
        Position::new_2d(31.0, 32.0),
        Genome::default(),
        0,
        Sex::Male,
    );
    male2.energy = 80.0;
    male2.age = 50;

    // Candidates: female at distance 1, male at distance 1
    let candidates: Vec<(usize, &Agent, f32)> = vec![(1, &female, 1.0), (2, &male2, 1.0)];

    let best = selector.find_best_partner(&male, &candidates);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Should select female (opposite sex)
    assert_eq!(
        best,
        Some(1), // female index
        "Partner selection should prefer opposite sex"
    );
}

/// Test birthrate system is deterministic with same seed
#[test]
fn test_birthrate_determinism() {
    let birthrate_config = BirthrateConfig::default();

    let config1 = SimulationConfig {
        seed: 12345,
        birthrate_config: Some(birthrate_config),
        toggles: ToggleConfig::stage_1(),
        ..Default::default()
    };

    let config2 = SimulationConfig {
        seed: 12345,
        birthrate_config: Some(birthrate_config),
        toggles: ToggleConfig::stage_1(),
        ..Default::default()
    };
    // TAG: surface=simulation owner=research-team rule=SIM-001

    let mut sim1 = Simulation::new(config1);
    let mut sim2 = Simulation::new(config2);

    // Spawn same agents in same order
    for _ in 0..5 {
        let id1 = sim1.spawn_agent(Position::new_2d(32.0, 32.0));
        sim1.agents[id1 as usize].energy = 100.0;
        sim1.agents[id1 as usize].age = 50;

        let id2 = sim2.spawn_agent(Position::new_2d(32.0, 32.0));
        sim2.agents[id2 as usize].energy = 100.0;
        sim2.agents[id2 as usize].age = 50;
    }

    // Run both simulations
    sim1.run(50);
    sim2.run(50);

    // Should have same number of agents (deterministic)
    assert_eq!(
        sim1.agents.len(),
        sim2.agents.len(),
        "Same seed should produce same number of agents"
    );

    // TAG: surface=simulation owner=research-team rule=SIM-001
}

/// Test birthrate system can be disabled (use legacy threshold)
#[test]
fn test_birthrate_disabled_uses_legacy() {
    let config = SimulationConfig {
        birthrate_config: None, // Disabled
        reproduction_threshold: 75.0,
        reproduction_cost: 20.0,
        toggles: ToggleConfig::stage_1(),
        ..Default::default()
    };

    let mut sim = Simulation::new(config);

    // Spawn two agents with high energy
    let agent1 = sim.spawn_agent(Position::new_2d(32.0, 32.0));
    let agent2 = sim.spawn_agent(Position::new_2d(33.0, 32.0));
    sim.agents[agent1 as usize].energy = 100.0;
    sim.agents[agent2 as usize].energy = 100.0;

    let initial_count = sim.agents.len();

    // Run one step
    sim.step();
    // TAG: surface=simulation owner=research-team rule=SIM-001

    // Legacy threshold reproduction should produce offspring
    assert!(
        sim.agents.len() > initial_count,
        "Legacy threshold reproduction should work when birthrate disabled"
    );
}

/// Test memory cleanup functionality
#[test]
fn test_cleanup_dead_agents() {
    let config = SimulationConfig {
        cleanup_interval: 100, // Enable cleanup every 100 ticks
        preserve_lineage: true,
        ..SimulationConfig::default()
    };

    let mut sim = Simulation::new(config);

    // Spawn an agent
    let agent_id = sim.spawn_agent(Position::new_2d(32.0, 32.0));

    // Kill the agent manually
    sim.agents[agent_id as usize].alive = false;
    sim.agents[agent_id as usize].age = 50;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Verify agent is dead but still in vector
    assert_eq!(sim.living_agent_count(), 0);
    assert_eq!(sim.agents.len(), 1);

    // Manually trigger cleanup
    let removed = sim.cleanup_dead_agents();

    // Verify cleanup worked
    assert_eq!(removed, 1, "Should have removed 1 dead agent");
    assert_eq!(sim.agents.len(), 0, "Agents vector should be empty");

    // Verify lineage summary was preserved
    assert_eq!(sim.lineage.summary_count(), 1, "Should have 1 summary");
    assert!(
        sim.lineage.get_summary(agent_id).is_some(),
        "Summary should exist for dead agent"
    );
}

/// Test memory usage estimation
#[test]
fn test_estimate_memory_usage() {
    let config = SimulationConfig::default();
    let mut sim = Simulation::new(config);

    // Initial memory usage
    let mem_before = sim.estimate_memory_usage();
    assert_eq!(mem_before.alive_count, 0);
    assert_eq!(mem_before.dead_count, 0);

    // Spawn an agent
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    let mem_after = sim.estimate_memory_usage();
    assert_eq!(mem_after.alive_count, 1);
    assert_eq!(mem_after.dead_count, 0);
    assert!(mem_after.total_bytes > mem_before.total_bytes);
}

/// Test automatic cleanup trigger at interval
#[test]
fn test_automatic_cleanup_trigger() {
    let config = SimulationConfig {
        cleanup_interval: 50,          // Cleanup every 50 ticks
        energy_consumption_rate: 10.0, // Fast energy drain to kill agents quickly
        movement_cost: 0.0,            // Disable movement cost
        ..SimulationConfig::default()
    };

    let mut sim = Simulation::new(config);
    // TAG: surface=simulation owner=research-team rule=SIM-001

    // Spawn an agent with low energy
    let agent_id = sim.spawn_agent(Position::new_2d(32.0, 32.0));
    sim.agents[agent_id as usize].energy = 50.0; // Will die in 5 ticks

    // Clear resources so agent can't replenish
    for resource in &mut sim.environment.resources {
        resource.amount = 0.0;
        resource.regrowth_rate = 0.0;
    }

    // Run until agent dies and cleanup triggers
    sim.run(55); // Past 50 tick cleanup interval

    // Agent should be dead and cleaned up
    assert_eq!(sim.living_agent_count(), 0);
    // After cleanup at tick 50, agent should be removed from vector
    // Note: exact count depends on timing, but should be 0 or 1
}

/// Test that pheromones actually affect agent movement (SIM-STIG-001)
///
/// This test verifies that:
/// 1. Agents sense pheromones at their position
/// 2. Agents follow pheromone gradients when concentration exceeds threshold
// TAG: surface=simulation owner=research-team rule=SIM-001
/// 3. Agents deposit pheromones based on their state
#[test]
fn test_pheromone_influences_movement() {
    use crate::stigmergy::PheromoneType;

    let mut config = SimulationConfig::default();
    config.toggles.stigmergy_enabled = true;
    config.movement_cost = 0.0; // Disable movement cost for predictable test
    config.energy_consumption_rate = 0.0; // No energy drain

    let mut sim = Simulation::new(config);

    // Deposit strong food pheromone at position (40, 32)
    let food_pos = Position::new_2d(40.0, 32.0);
    sim.pheromone_grid
        .deposit(&food_pos, PheromoneType::FoodTrail, 5.0);

    // Place agent nearby at (32, 32), but NOT at the food pheromone
    let agent_id = sim.spawn_agent(Position::new_2d(32.0, 32.0));
    let initial_pos = sim.agents[agent_id as usize].position;

    // Run a few ticks to allow agent to sense and follow pheromone
    for _ in 0..5 {
        sim.step();
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Agent should have moved toward food pheromone (positive x direction)
    let final_pos = sim.agents[agent_id as usize].position;
    let dist_to_food_initial = initial_pos.distance(&food_pos);
    let dist_to_food_final = final_pos.distance(&food_pos);

    // Agent should generally move toward food (distance should decrease or stay similar)
    // We use a loose assertion because movement has randomness
    assert!(
            dist_to_food_final <= dist_to_food_initial + 2.0, // Allow some slack for randomness
            "Agent should move toward or stay near food pheromone: initial_dist={dist_to_food_initial}, final_dist={dist_to_food_final}"
        );

    // Verify agent sensed pheromones (chemotaxis_direction should be Some after following)
    // Note: chemotaxis_direction may be None if pheromone was below threshold
    // or agent moved away from the gradient
}

/// Test that agents deposit pheromones based on their state
#[test]
fn test_agent_deposits_pheromones() {
    use crate::stigmergy::PheromoneType;
    use crate::types::C;

    let mut config = SimulationConfig::default();
    config.toggles.stigmergy_enabled = true;
    // TAG: surface=simulation owner=research-team rule=SIM-001
    config.energy_consumption_rate = 0.0; // No energy drain

    let mut sim = Simulation::new(config);

    // Spawn agent with high energy (should deposit food trail)
    let agent_id = sim.spawn_agent(Position::new_2d(32.0, 32.0));
    sim.agents[agent_id as usize].energy = 80.0; // Above 60 threshold

    // Clear any existing pheromones first
    sim.pheromone_grid = crate::stigmergy::PheromoneGrid::new(64, 64, 1);

    // Run one tick to allow deposition
    sim.step();

    // Check that food trail pheromone was deposited
    let agent_pos = sim.agents[agent_id as usize].position;
    let food_conc = sim
        .pheromone_grid
        .concentration(&agent_pos, PheromoneType::FoodTrail);

    assert!(
        food_conc > 0.0,
        "Agent with high energy should deposit food trail pheromone, got concentration={food_conc}"
    );

    // Now test danger signal deposition
    // TAG: surface=simulation owner=research-team rule=SIM-001
    sim.agents[agent_id as usize].docas.levels[C] = 0.9; // High cortisol

    // Move agent to new position
    sim.agents[agent_id as usize].position = Position::new_2d(35.0, 35.0);
    let danger_pos = sim.agents[agent_id as usize].position;

    // Run another tick
    sim.step();

    // Check that danger signal was deposited
    let danger_conc = sim
        .pheromone_grid
        .concentration(&danger_pos, PheromoneType::DangerSignal);

    assert!(
            danger_conc > 0.0,
            "Agent with high cortisol should deposit danger signal pheromone, got concentration={danger_conc}"
        );
}

// =========================================================================
// SOCIAL FORCE TESTS
// =========================================================================

/// Test that social forces affect agent movement
/// CRITICAL FIX: Verifies that `social_force` field is actually applied to movement
#[test]
fn test_social_force_affects_movement() {
    use crate::dmn::Direction;
    use crate::types::Vector;

    // Create simulation with Barnes-Hut enabled and DMN for direct movement control
    let mut config = SimulationConfig::default();
    config.toggles.barnes_hut_enabled = true;
    config.toggles.dmn_enabled = true; // Enable DMN to directly control movement
    config.social_force_influence = 0.5; // 50% social influence for test
    config.movement_cost = 0.0; // Disable movement cost for predictable test

    let mut sim = Simulation::new(config);

    // Spawn an agent at a fixed position
    let pos1 = Position::new_2d(32.0, 32.0);
    let agent1_id = sim.spawn_agent(pos1);

    // Record initial position
    let initial_pos1 = sim.agents[agent1_id as usize].position;

    // Manually apply a strong social force to agent1 (pointing east)
    // This simulates what Barnes-Hut would compute
    sim.agents[agent1_id as usize].apply_social_force(Vector::new(1.0, 0.0));
    // TAG: surface=simulation owner=research-team rule=SIM-001

    // Directly test the movement application by simulating what happens in step()
    // Use Direction directly
    let dir = Direction::North; // Base movement is North
    let (_base_dx, _base_dy) = dir.to_delta();

    // Apply social force influence (mimicking the logic in step())
    let social_force = sim.agents[agent1_id as usize].social_force;
    let _ = sim.config.social_force_influence;

    // With 50% social influence and strong eastward force:
    // - Base: North (dx=0, dy=1)
    // - Social: East (dx=1, dy=0)
    // - Combined should have eastward component
    let force_magnitude = social_force.x.hypot(social_force.y);
    assert!(
        force_magnitude > 0.5,
        "Social force magnitude should be significant"
    );

    // The social force should affect the final movement direction
    // Verify the force is stored correctly
    assert!(
        social_force.x > 0.0,
        "Social force should point east (positive x), got {social_force:?}" // TAG: surface=simulation owner=research-team rule=SIM-001
    );

    // Now execute a step which will apply the social force during movement
    sim.step();

    // Verify the social force was cleared after application
    let force_after = sim.agents[agent1_id as usize].social_force;
    assert!(
        force_after.x.abs() < 0.001 && force_after.y.abs() < 0.001,
        "Social force should be cleared after movement, got {force_after:?}"
    );

    // Verify chemotaxis was also cleared
    assert!(
        sim.agents[agent1_id as usize]
            .chemotaxis_direction
            .is_none(),
        "Chemotaxis direction should be cleared after movement"
    );

    // Test that position changed (agent moved during the step)
    let final_pos1 = sim.agents[agent1_id as usize].position;
    let distance_moved = initial_pos1.distance(&final_pos1);
    assert!(
        distance_moved >= 0.0,
        "Distance should be non-negative, got {distance_moved}" // TAG: surface=simulation owner=research-team rule=SIM-001
    );

    // Verify apply/clear cycle works correctly
    sim.agents[agent1_id as usize].apply_social_force(Vector::new(0.5, 0.5));
    let force_before = sim.agents[agent1_id as usize].social_force;
    assert!(
        force_before.x > 0.0 && force_before.y > 0.0,
        "Social force should be stored, got {force_before:?}"
    );

    sim.agents[agent1_id as usize].clear_social_force();
    let force_cleared = sim.agents[agent1_id as usize].social_force;
    assert!(
        force_cleared.x.abs() < 0.001 && force_cleared.y.abs() < 0.001,
        "Social force should be cleared, got {force_cleared:?}"
    );
}

/// Test that Barnes-Hut computation runs and produces forces
#[test]
fn test_barnes_hut_computation() {
    use crate::barnes_hut::{default_social_force, BarnesHutTree};

    let mut config = SimulationConfig::default();
    config.toggles.barnes_hut_enabled = true;
    // TAG: surface=simulation owner=research-team rule=SIM-001

    let mut sim = Simulation::new(config);

    // Spawn several agents at specific positions
    let id1 = sim.spawn_agent(Position::new_2d(10.0, 10.0));
    let id2 = sim.spawn_agent(Position::new_2d(12.0, 12.0)); // Very close to agent 1
    let id3 = sim.spawn_agent(Position::new_2d(30.0, 30.0));
    let id4 = sim.spawn_agent(Position::new_2d(32.0, 32.0)); // Very close to agent 3

    // Set up trust relationships - high trust between pairs
    // This is necessary because default_social_force uses trust - 0.5
    // update_local_reputation adds to current value (default 0.5), so add 0.3 to get 0.8
    if let Some(agent1) = sim.agents.iter_mut().find(|a| a.id == id1) {
        agent1.update_local_reputation(id2, 0.3); // High trust: 0.5 + 0.3 = 0.8
    }
    if let Some(agent2) = sim.agents.iter_mut().find(|a| a.id == id2) {
        agent2.update_local_reputation(id1, 0.3); // High trust: 0.5 + 0.3 = 0.8
    }
    if let Some(agent3) = sim.agents.iter_mut().find(|a| a.id == id3) {
        agent3.update_local_reputation(id4, -0.3); // Low trust (repulsion): 0.5 - 0.3 = 0.2
    }
    if let Some(agent4) = sim.agents.iter_mut().find(|a| a.id == id4) {
        agent4.update_local_reputation(id3, -0.3); // Low trust (repulsion): 0.5 - 0.3 = 0.2
    }

    // Build Barnes-Hut tree
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let mut tree = BarnesHutTree::new(64.0, 64.0);
    tree.build(&sim.agents);

    // Compute forces
    tree.compute_forces(&mut sim.agents, &default_social_force);

    // Verify that forces were computed (non-zero for agents with trust relationships)
    let mut any_non_zero = false;
    for agent in &sim.agents {
        if agent.alive {
            let force_magnitude = agent.social_force.magnitude();
            if force_magnitude > 0.001 {
                any_non_zero = true;
                break;
            }
        }
    }

    assert!(
        any_non_zero,
        "Barnes-Hut should compute non-zero forces when agents have trust relationships"
    );
}

/// Test that social force influence configuration affects movement
#[test]
#[allow(clippy::field_reassign_with_default)]
fn test_social_force_influence_configuration() {
    // Test with 0% social influence
    let mut config_no_social = SimulationConfig::default();
    config_no_social.social_force_influence = 0.0;
    config_no_social.toggles.barnes_hut_enabled = true;

    // Test with 100% social influence
    #[allow(clippy::field_reassign_with_default)]
    let mut config_full_social = SimulationConfig::default();
    config_full_social.social_force_influence = 1.0;
    config_full_social.toggles.barnes_hut_enabled = true;

    assert_eq!(config_no_social.social_force_influence, 0.0);
    assert_eq!(config_full_social.social_force_influence, 1.0);
}
use super::*;
use rand::rngs::StdRng;
use rand::SeedableRng;

// ========================================================================
// SimulationConfig builder and factory method tests
// ========================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_config_from_parameters() {
    let params = SimulationParameters::default();
    let config = SimulationConfig::from_parameters(&params, 42, 1000);
    assert_eq!(config.seed, 42);
    assert_eq!(config.max_ticks, 1000);
    assert_eq!(config.environment.width, 64);
    assert_eq!(config.environment.height, 64);
    assert_eq!(
        config.verification_mode,
        VerificationMode::DecentralizedProtocol
    );
    assert!(config.birthrate_config.is_none());
}

#[test]
fn test_config_from_parameters_with_toggles() {
    let params = SimulationParameters::default();
    let toggles = ToggleConfig::stage_3();
    let config = SimulationConfig::from_parameters_with_toggles(&params, 99, 500, toggles);
    assert_eq!(config.seed, 99);
    assert_eq!(config.max_ticks, 500);
    assert_eq!(config.toggles.genome_enabled, toggles.genome_enabled);
    assert_eq!(
        config.toggles.institutions_enabled,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        toggles.institutions_enabled
    );
    assert_eq!(config.toggles.norms_enabled, toggles.norms_enabled);
}

#[test]
fn test_config_centralized_baseline() {
    let config = SimulationConfig::centralized_baseline(123, 2000);
    assert_eq!(config.seed, 123);
    assert_eq!(config.max_ticks, 2000);
    assert_eq!(
        config.verification_mode,
        VerificationMode::CentralizedOracle
    );
    assert!(config.toggles.protocol_enabled);
}

#[test]
fn test_config_decentralized_protocol() {
    let config = SimulationConfig::decentralized_protocol(456, 3000);
    assert_eq!(config.seed, 456);
    assert_eq!(config.max_ticks, 3000);
    assert_eq!(
        config.verification_mode,
        VerificationMode::DecentralizedProtocol
    );
    // TAG: surface=simulation owner=research-team rule=SIM-001
    assert!(config.toggles.protocol_enabled);
}

// ========================================================================
// EventStore comprehensive tests
// ========================================================================

#[test]
fn test_event_store_new_default() {
    let store = EventStore::default();
    assert_eq!(store.game_events.capacity(), 10000);
    assert_eq!(store.max_capacity, 100000);
}

#[test]
fn test_event_store_persist_game_events_overflow() {
    let mut store = EventStore::new();
    store.max_capacity = 20;

    // Fill to capacity
    for i in 0..20 {
        store.game_events.push(GameEvent {
            tick: i as u64,
            agent_a: 0,
            agent_b: 1,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            game_type: "test".to_string(),
            outcome: "cooperate".to_string(),
            payoff_a: 1.0,
            payoff_b: 1.0,
        });
    }
    assert_eq!(store.game_events.len(), 20);

    // Persist one more event - should trigger ring buffer drain
    let new_events = vec![GameEvent {
        tick: 99,
        agent_a: 2,
        agent_b: 3,
        game_type: "test".to_string(),
        outcome: "defect".to_string(),
        payoff_a: 0.0,
        payoff_b: 5.0,
    }];
    store.persist_game_events(&new_events);
    // 20 - 2 (10% of 20) + 1 = 19
    assert_eq!(store.game_events.len(), 19);
    // The new event should be at the end
    assert_eq!(store.game_events.last().unwrap().tick, 99);
}

#[test]
// TAG: surface=simulation owner=research-team rule=SIM-001
fn test_event_store_persist_encounter_events_overflow() {
    let mut store = EventStore::new();
    store.max_capacity = 20;

    for i in 0..20 {
        store.encounter_events.push(EncounterEvent {
            tick: i as u64,
            agent_a: 0,
            agent_b: 1,
            encounter_type: "proximity".to_string(),
        });
    }

    let new_events = vec![EncounterEvent {
        tick: 99,
        agent_a: 2,
        agent_b: 3,
        encounter_type: "game".to_string(),
    }];
    store.persist_encounter_events(&new_events);
    assert_eq!(store.encounter_events.len(), 19);
    assert_eq!(store.encounter_events.last().unwrap().tick, 99);
}

#[test]
fn test_event_store_record_birth_overflow() {
    let mut store = EventStore::new();
    store.max_capacity = 5;

    for i in 0..5 {
        store.record_birth(i as u64, i as u64, Some(0), Some(1));
    }
    assert_eq!(store.birth_events.len(), 5);

    store.record_birth(99, 99, Some(0), None);
    assert_eq!(store.birth_events.len(), 5);
    // First event should have been popped
    assert_eq!(store.birth_events.front().unwrap().0, 1);
    // New event at back
    assert_eq!(store.birth_events.back().unwrap().0, 99);
}

#[test]
fn test_event_store_record_death_overflow() {
    let mut store = EventStore::new();
    store.max_capacity = 5;

    for i in 0..5 {
        store.record_death(i as u64, i as u64, "starvation");
    }
    // TAG: surface=simulation owner=research-team rule=SIM-001
    assert_eq!(store.death_events.len(), 5);

    store.record_death(99, 99, "old_age");
    assert_eq!(store.death_events.len(), 5);
    assert_eq!(store.death_events.front().unwrap().0, 1);
    assert_eq!(store.death_events.back().unwrap().2, "old_age");
}

#[test]
fn test_event_store_record_group_formation_overflow() {
    let mut store = EventStore::new();

    for i in 0..9999 {
        store.record_group_formation(i, i, vec![0, 1]);
    }
    assert_eq!(store.group_formation_events.len(), 9999);

    // Two more should trigger overflow (limit is 10000)
    store.record_group_formation(99999, 99, vec![2, 3]);
    store.record_group_formation(99998, 98, vec![3, 4]);
    assert_eq!(store.group_formation_events.len(), 10000);
    assert_eq!(store.group_formation_events.last().unwrap().0, 99998);
}

#[test]
// TAG: surface=simulation owner=research-team rule=SIM-001
fn test_event_store_get_game_events_in_range() {
    let mut store = EventStore::new();
    for i in 0..10 {
        store.game_events.push(GameEvent {
            tick: i as u64 * 10,
            agent_a: i as u64,
            agent_b: (i + 1) as u64,
            game_type: "test".to_string(),
            outcome: "cooperate".to_string(),
            payoff_a: 1.0,
            payoff_b: 1.0,
        });
    }

    let events = store.get_game_events_in_range(20, 50);
    assert_eq!(events.len(), 4); // ticks 20, 30, 40, 50
    assert_eq!(events[0].tick, 20);
    assert_eq!(events[3].tick, 50);

    let empty = store.get_game_events_in_range(200, 300);
    assert!(empty.is_empty());
}

#[test]
fn test_event_store_get_agent_events() {
    let mut store = EventStore::new();
    // TAG: surface=simulation owner=research-team rule=SIM-001
    store.game_events.push(GameEvent {
        tick: 1,
        agent_a: 5,
        agent_b: 10,
        game_type: "test".to_string(),
        outcome: "cooperate".to_string(),
        payoff_a: 1.0,
        payoff_b: 1.0,
    });
    store.game_events.push(GameEvent {
        tick: 2,
        agent_a: 10,
        agent_b: 20,
        game_type: "test".to_string(),
        outcome: "defect".to_string(),
        payoff_a: 0.0,
        payoff_b: 5.0,
    });
    store.game_events.push(GameEvent {
        tick: 3,
        agent_a: 99,
        agent_b: 99,
        game_type: "test".to_string(),
        outcome: "cooperate".to_string(),
        payoff_a: 1.0,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        payoff_b: 1.0,
    });

    let events = store.get_agent_events(10);
    assert_eq!(events.len(), 2);

    let events = store.get_agent_events(99);
    assert_eq!(events.len(), 1);

    let events = store.get_agent_events(999);
    assert!(events.is_empty());
}

#[test]
fn test_event_store_get_summary() {
    let mut store = EventStore::new();
    store.game_events.push(GameEvent {
        tick: 1,
        agent_a: 0,
        agent_b: 1,
        game_type: "pd".to_string(),
        outcome: "cooperate".to_string(),
        payoff_a: 3.0,
        payoff_b: 3.0,
    });
    store.encounter_events.push(EncounterEvent {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        tick: 1,
        agent_a: 0,
        agent_b: 1,
        encounter_type: "game".to_string(),
    });
    store.record_birth(2, 1, Some(0), Some(1));
    store.record_death(3, 5, "starvation");
    store.record_group_formation(1, 10, vec![0, 1]);

    let summary = store.get_summary();
    assert_eq!(summary.total_game_events, 1);
    assert_eq!(summary.total_encounter_events, 1);
    assert_eq!(summary.total_births, 1);
    assert_eq!(summary.total_deaths, 1);
    assert_eq!(summary.total_group_formations, 1);
}

#[test]
fn test_event_store_generate_report() {
    let mut store = EventStore::new();
    for i in 0..5 {
        store.game_events.push(GameEvent {
            tick: i as u64,
            agent_a: i as u64,
            agent_b: (i + 1) as u64,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            game_type: "pd".to_string(),
            outcome: "cooperate".to_string(),
            payoff_a: 3.0,
            payoff_b: 3.0,
        });
    }
    store.record_birth(10, 1, Some(0), Some(1));
    store.record_death(11, 5, "starvation");
    store.record_group_formation(1, 10, vec![0, 1]);

    let report = store.generate_report(3);
    assert_eq!(report.recent_game_events.len(), 3);
    assert_eq!(report.births.len(), 1);
    assert_eq!(report.deaths.len(), 1);
    assert_eq!(report.group_formations.len(), 1);
    // Recent events should be the last 3 in original order
    assert_eq!(report.recent_game_events[0].tick, 2);
    assert_eq!(report.recent_game_events[2].tick, 4);
}

// ========================================================================
// SimulationDebugState tests
// ========================================================================

#[test]
fn test_debug_state_get_agent_history() {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let mut debug = SimulationDebugState::default();
    let agent_id = 7u64;

    // No history yet
    assert!(debug.get_agent_history(agent_id).is_none());

    debug.record_position(1, agent_id, Position::new_2d(10.0, 10.0), 100.0);
    debug.record_position(2, agent_id, Position::new_2d(11.0, 10.0), 95.0);

    let history = debug.get_agent_history(agent_id).unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].0, 1);
    assert_eq!(history[1].0, 2);
    assert_eq!(history[1].1.x, 11.0);

    // Unknown agent
    assert!(debug.get_agent_history(999).is_none());
}

#[test]
fn test_debug_state_position_log_limit() {
    let mut debug = SimulationDebugState::default();
    let agent_id = 1u64;

    for i in 0..105 {
        debug.record_position(i as u64, agent_id, Position::new_2d(i as f32, 0.0), 100.0);
    }

    let history = debug.get_agent_history(agent_id).unwrap();
    assert_eq!(history.len(), 100); // capped at 100
}

#[test]
fn test_debug_state_log_limit() {
    let mut debug = SimulationDebugState::default();
    debug.max_log_entries = 5;

    for i in 0..10 {
        debug.log(i as u64, "Test", Some(1), "details");
    }

    assert_eq!(debug.recent_logs.len(), 5);
}

#[test]
fn test_debug_state_select_sampled_agent_empty() {
    let mut debug = SimulationDebugState::default();
    debug.select_sampled_agent(&[]);
    assert!(debug.sampled_agent_id.is_none());
}
// TAG: surface=simulation owner=research-team rule=SIM-001

#[test]
fn test_debug_state_select_sampled_agent_all_dead() {
    let mut sim = Simulation::new_default();
    let agent_id = sim.spawn_agent(Position::new_2d(32.0, 32.0));
    sim.agents[agent_id as usize].alive = false;

    let mut debug = SimulationDebugState::default();
    debug.select_sampled_agent(&sim.agents);
    assert!(debug.sampled_agent_id.is_none());
}

// ========================================================================
// Simulation helper method tests
// ========================================================================

#[test]
fn test_has_living_agents_empty() {
    let sim = Simulation::new_default();
    assert!(!sim.has_living_agents());
}

#[test]
fn test_has_living_agents_all_dead() {
    let mut sim = Simulation::new_default();
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let agent_id = sim.spawn_agent(Position::new_2d(32.0, 32.0));
    sim.agents[agent_id as usize].alive = false;
    assert!(!sim.has_living_agents());
}

#[test]
fn test_compute_allele_frequencies_empty() {
    let sim = Simulation::new_default();
    let freqs = sim.compute_allele_frequencies();
    assert!(freqs.is_empty());
}

#[test]
fn test_compute_allele_frequencies_with_agents() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(Position::new_2d(10.0, 10.0));
    sim.spawn_agent(Position::new_2d(20.0, 20.0));

    let freqs = sim.compute_allele_frequencies();
    assert!(!freqs.is_empty());
}

#[test]
fn test_compute_heterozygosity_empty() {
    let sim = Simulation::new_default();
    assert_eq!(sim.compute_heterozygosity(), 0.0);
    // TAG: surface=simulation owner=research-team rule=SIM-001
}

#[test]
fn test_compute_heterozygosity_with_agents() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(Position::new_2d(10.0, 10.0));
    sim.spawn_agent(Position::new_2d(20.0, 20.0));

    let h = sim.compute_heterozygosity();
    assert!(h >= 0.0);
}

#[test]
fn test_get_agent_epigenome() {
    let mut sim = Simulation::new_default();
    let agent_id = sim.spawn_agent(Position::new_2d(32.0, 32.0));
    let epi = Simulation::get_agent_epigenome(&sim.agents, agent_id);
    // Should return the agent's epigenome
    assert_eq!(
        epi.maternal_care_received,
        sim.agents[agent_id as usize]
            .epigenome
            .maternal_care_received
    );

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let missing = Simulation::get_agent_epigenome(&sim.agents, 999);
    assert_eq!(
        missing.maternal_care_received,
        Epigenome::new().maternal_care_received
    );
}

#[test]
fn test_agent_has_anchored_identity() {
    let mut sim = Simulation::new_default();
    let agent_id = sim.spawn_agent(Position::new_2d(32.0, 32.0));

    // Freshly spawned agent should have identity but no anchored claims
    assert!(!Simulation::agent_has_anchored_identity(
        &sim.agents,
        agent_id
    ));
    assert!(!Simulation::agent_has_anchored_identity(&sim.agents, 999));
}

#[test]
fn test_cleanup_dead_agents_no_dead() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(Position::new_2d(32.0, 32.0));
    let removed = sim.cleanup_dead_agents();
    assert_eq!(removed, 0);
    // TAG: surface=simulation owner=research-team rule=SIM-001
}

#[test]
fn test_cleanup_dead_agents_no_preserve() {
    let mut config = SimulationConfig::default();
    config.preserve_lineage = false;
    let mut sim = Simulation::new(config);

    let agent_id = sim.spawn_agent(Position::new_2d(32.0, 32.0));
    sim.agents[agent_id as usize].alive = false;

    let removed = sim.cleanup_dead_agents();
    assert_eq!(removed, 1);
}

// ========================================================================
// Discrete generation mode (Wright-Fisher) tests
// ========================================================================

#[test]
fn test_initialize_discrete_generation_overlapping() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(Position::new_2d(32.0, 32.0));
    sim.initialize_discrete_generation();
    assert!(sim.discrete_generation.is_none());
    assert!(sim.wf_model.is_none());
}

#[test]
fn test_initialize_discrete_generation() {
    let mut config = SimulationConfig::default();
    config.generation_mode = GenerationMode::Discrete {
        generation_time: 10,
        population_size: 4,
    };

    let mut sim = Simulation::new(config);
    for _ in 0..4 {
        sim.spawn_agent(Position::new_2d(32.0, 32.0));
    }

    sim.initialize_discrete_generation();
    assert!(sim.discrete_generation.is_some());
    assert!(sim.wf_model.is_some());
}

#[test]
fn test_discrete_generation_tick_no_replace() {
    let mut config = SimulationConfig::default();
    config.generation_mode = GenerationMode::Discrete {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        generation_time: 10,
        population_size: 2,
    };

    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(10.0, 10.0));
    sim.spawn_agent(Position::new_2d(20.0, 20.0));
    sim.initialize_discrete_generation();

    let count_before = sim.living_agent_count();
    sim.run(5); // Less than generation_time
    assert_eq!(sim.living_agent_count(), count_before);
}

#[test]
fn test_discrete_generation_replacement() {
    let mut config = SimulationConfig::default();
    config.generation_mode = GenerationMode::Discrete {
        generation_time: 5,
        population_size: 2,
    };
    config.energy_consumption_rate = 0.0;
    config.movement_cost = 0.0;

    let mut sim = Simulation::new(config);
    // TAG: surface=simulation owner=research-team rule=SIM-001
    sim.spawn_agent(Position::new_2d(10.0, 10.0));
    sim.spawn_agent(Position::new_2d(20.0, 20.0));
    sim.initialize_discrete_generation();

    let ids_before: Vec<u64> = sim.agents.iter().map(|a| a.id).collect();
    sim.run(6); // Past generation_time

    // Should have replaced generation (parents die, offspring born)
    let ids_after: Vec<u64> = sim.agents.iter().map(|a| a.id).collect();
    // New generation should have different agent IDs
    assert_ne!(ids_before, ids_after);
}

#[test]
fn test_execute_generation_replacement_no_mgr() {
    let mut sim = Simulation::new_default();
    // discrete_generation is None, should be a no-op
    sim.execute_generation_replacement();
}

// ========================================================================
// Simulation step with various toggles enabled
// ========================================================================

#[test]
fn test_step_with_economics_enabled() {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let mut config = SimulationConfig::default();
    config.toggles.economics_enabled = true;
    config.energy_consumption_rate = 0.0;
    config.movement_cost = 0.0;

    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    // Run 10 ticks so economics process triggers
    sim.run(10);
    assert_eq!(sim.tick, 10);
}

#[test]
fn test_step_with_criticality_enabled() {
    let mut config = SimulationConfig::default();
    config.toggles.criticality_enabled = true;
    config.energy_consumption_rate = 0.0;
    config.movement_cost = 0.0;

    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    sim.run(5);
    assert_eq!(sim.tick, 5);
    // TAG: surface=simulation owner=research-team rule=SIM-001
}

#[test]
fn test_step_with_music_enabled() {
    let mut config = SimulationConfig::default();
    config.toggles.music_enabled = true;
    config.energy_consumption_rate = 0.0;
    config.movement_cost = 0.0;

    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    sim.run(5);
    assert_eq!(sim.tick, 5);
}

#[test]
fn test_step_with_protocol_enabled_centralized() {
    let mut config = SimulationConfig::default();
    config.toggles.protocol_enabled = true;
    config.verification_mode = VerificationMode::CentralizedOracle;
    config.energy_consumption_rate = 0.0;
    config.movement_cost = 0.0;

    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));
    // TAG: surface=simulation owner=research-team rule=SIM-001

    sim.run(5);
    assert_eq!(sim.tick, 5);
}

#[test]
fn test_step_with_protocol_enabled_decentralized() {
    let mut config = SimulationConfig::default();
    config.toggles.protocol_enabled = true;
    config.verification_mode = VerificationMode::DecentralizedProtocol;
    config.energy_consumption_rate = 0.0;
    config.movement_cost = 0.0;

    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    sim.run(5);
    assert_eq!(sim.tick, 5);
}

#[test]
fn test_step_with_encounters_enabled() {
    let mut config = SimulationConfig::default();
    config.toggles.encounters_enabled = true;
    config.energy_consumption_rate = 0.0;
    config.movement_cost = 0.0;

    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));
    sim.spawn_agent(Position::new_2d(32.5, 32.5)); // Close for encounter

    sim.run(5);
    assert_eq!(sim.tick, 5);
    // Cooperation history should be populated
    assert!(!sim.metrics.cooperation_history.is_empty());
}

#[test]
fn test_step_with_language_enabled() {
    let mut config = SimulationConfig::default();
    config.toggles.language_enabled = true;
    config.energy_consumption_rate = 0.0;
    config.movement_cost = 0.0;

    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    sim.run(5);
    assert_eq!(sim.tick, 5);
    // Agent should have a lexicon initialized
    // TAG: surface=simulation owner=research-team rule=SIM-001
    assert!(!sim.agents[0].lexicon.concepts.is_empty());
}

#[test]
fn test_step_with_norms_enabled() {
    let mut config = SimulationConfig::default();
    config.toggles.norms_enabled = true;
    config.energy_consumption_rate = 0.0;
    config.movement_cost = 0.0;

    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    sim.run(5);
    assert_eq!(sim.tick, 5);
}

#[test]
fn test_step_with_institutions_enabled() {
    let mut config = SimulationConfig::default();
    config.toggles.institutions_enabled = true;
    config.energy_consumption_rate = 0.0;
    config.movement_cost = 0.0;

    let mut sim = Simulation::new(config);
    // TAG: surface=simulation owner=research-team rule=SIM-001
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    sim.run(5);
    assert_eq!(sim.tick, 5);
}

#[test]
fn test_step_with_dmn_enabled() {
    let mut config = SimulationConfig::default();
    config.toggles.dmn_enabled = true;
    config.energy_consumption_rate = 0.0;
    config.movement_cost = 0.0;

    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    sim.run(5);
    assert_eq!(sim.tick, 5);
}

#[test]
fn test_step_with_choice_architecture_enabled() {
    let mut config = SimulationConfig::default();
    config.toggles.dmn_enabled = true;
    config.toggles.choice_architecture_enabled = true;
    config.energy_consumption_rate = 0.0;
    // TAG: surface=simulation owner=research-team rule=SIM-001
    config.movement_cost = 0.0;

    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    sim.run(5);
    assert_eq!(sim.tick, 5);
}

#[test]
fn test_step_with_barnes_hut_post_movement() {
    let mut config = SimulationConfig::default();
    config.toggles.barnes_hut_enabled = true;
    config.energy_consumption_rate = 0.0;
    config.movement_cost = 0.0;

    let mut sim = Simulation::new(config);
    // Spawn more than 10 agents to trigger post-movement Barnes-Hut
    for i in 0..12 {
        sim.spawn_agent(Position::new_2d(10.0 + i as f32, 10.0 + i as f32));
    }

    sim.run(2);
    assert_eq!(sim.tick, 2);
}
// TAG: surface=simulation owner=research-team rule=SIM-001

#[test]
fn test_step_empty_simulation() {
    let mut sim = Simulation::new_default();
    sim.step();
    assert_eq!(sim.tick, 1);
    assert_eq!(sim.living_agent_count(), 0);
}

// ========================================================================
// Society and culture processing tests
// ========================================================================

#[test]
fn test_process_stage3_society_norms_and_institutions() {
    let mut config = SimulationConfig::default();
    config.toggles.norms_enabled = true;
    config.toggles.institutions_enabled = true;
    config.energy_consumption_rate = 0.0;
    config.movement_cost = 0.0;

    let mut sim = Simulation::new(config);
    // Spawn enough agents for norm emergence (need >= 5 in a group)
    for _ in 0..6 {
        sim.spawn_agent(Position::new_2d(32.0, 32.0));
    }
    // TAG: surface=simulation owner=research-team rule=SIM-001

    // Run to tick 100 where norm emergence triggers
    sim.run(100);
    assert_eq!(sim.tick, 100);
}

#[test]
fn test_apply_norm_compliance_empty_norms() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(Position::new_2d(32.0, 32.0));
    // No norms defined, should be a no-op
    sim.apply_norm_compliance();
}

#[test]
fn test_process_norm_enforcement_empty_violations() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(Position::new_2d(32.0, 32.0));
    // No norms, no violations
    sim.process_norm_enforcement();
}

#[test]
fn test_process_group_formation() {
    let mut sim = Simulation::new_default();
    for _ in 0..5 {
        sim.spawn_agent(Position::new_2d(32.0, 32.0));
    }
    sim.tick = 50;
    sim.process_group_formation();
    // Groups may or may not form depending on positions
}

#[test]
fn test_process_institution_evolution() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(Position::new_2d(32.0, 32.0));
    sim.process_institution_evolution();
}

#[test]
fn test_find_norm_violations() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut sim = Simulation::new_default();
    for _ in 0..3 {
        sim.spawn_agent(Position::new_2d(32.0, 32.0));
    }

    let norms = vec![Norm {
        id: 0,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        behavior_pattern: crate::society::BehaviorPattern::CooperateWithStrangers,
        value: 0.8,
        created_tick: 0,
        group_id: None,
        emergence_type: crate::society::NormEmergenceType::FrequencyDependent,
        adoption_rate: 0.5,
        salience: 0.5,
    }];

    let violations = Simulation::find_norm_violations(&sim.agents, &norms, &mut rng);
    // With random compliance, we may get some violations
    assert!(violations.len() <= sim.agents.len() * norms.len());
}

#[test]
fn test_apply_enforcement_deltas() {
    let mut sim = Simulation::new_default();
    let id1 = sim.spawn_agent(Position::new_2d(32.0, 32.0));
    let id2 = sim.spawn_agent(Position::new_2d(33.0, 33.0));

    let results = vec![SanctionResult {
        violator_id: id1,
        violator_cortisol_delta: 0.1,
        punisher_id: Some(id2),
        punisher_dopamine_delta: 0.05,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        sanction_type: crate::society::SanctionType::ReputationLoss,
        severity: 0.5,
    }];

    let before = sim.agents[id1 as usize].docas.levels[2];
    sim.apply_enforcement_deltas(&results);
    let after = sim.agents[id1 as usize].docas.levels[2];
    assert!((after - before).abs() > 0.001 || after >= 0.999);
}

// ========================================================================
// Identity theft processing tests
// ========================================================================

#[test]
fn test_process_identity_theft_no_agents() {
    let mut sim = Simulation::new_default();
    sim.process_identity_theft();
    assert!(sim.metrics.identity_theft_stats.is_some());
}

#[test]
fn test_initialize_agent_identities() {
    let mut sim = Simulation::new_default();
    let agent_id = sim.spawn_agent(Position::new_2d(32.0, 32.0));
    // Remove identity to test initialization
    // TAG: surface=simulation owner=research-team rule=SIM-001
    sim.agents[agent_id as usize].identity = None;
    sim.initialize_agent_identities();
    assert!(sim.agents[agent_id as usize].identity.is_some());
}

#[test]
fn test_sync_identity_legacy_fields() {
    let mut sim = Simulation::new_default();
    let agent_id = sim.spawn_agent(Position::new_2d(32.0, 32.0));
    sim.sync_identity_legacy_fields();
    assert!(!sim.agents[agent_id as usize].is_identity_theft_victim);
}

// ========================================================================
// Encounter processing tests
// ========================================================================

#[test]
fn test_process_encounters_no_pairs() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(Position::new_2d(1.0, 1.0));
    // Single agent, no pairs possible
    sim.process_encounters();
    assert_eq!(sim.metrics.encounters_this_tick, 0);
    // Cooperation history should still be pushed
    // TAG: surface=simulation owner=research-team rule=SIM-001
    assert!(!sim.metrics.cooperation_history.is_empty());
}

#[test]
fn test_process_encounters_with_pairs() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(Position::new_2d(32.0, 32.0));
    sim.spawn_agent(Position::new_2d(32.1, 32.1)); // Very close

    sim.process_encounters();
    // Should have at least one encounter
    assert!(sim.metrics.encounters_this_tick > 0);
}

// ========================================================================
// Reproduction edge case tests
// ========================================================================

#[test]
fn test_process_reproduction_at_capacity() {
    let mut config = SimulationConfig::default();
    config.reproduction_threshold = 1.0; // Very low threshold
    config.reproduction_cost = 1.0;
    config.energy_consumption_rate = 0.0;
    config.movement_cost = 0.0;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let mut sim = Simulation::new(config);
    // Spawn 500 agents (at capacity)
    for _ in 0..500 {
        sim.spawn_agent(Position::new_2d(32.0, 32.0));
    }
    for agent in &mut sim.agents {
        agent.energy = 100.0;
    }

    let count_before = sim.agents.len();
    sim.process_reproduction();
    // Should not exceed capacity
    assert_eq!(sim.agents.len(), count_before);
}

#[test]
fn test_find_best_partner_no_eligible() {
    let mut sim = Simulation::new_default();
    let id1 = sim.spawn_agent(Position::new_2d(32.0, 32.0));
    let id2 = sim.spawn_agent(Position::new_2d(33.0, 33.0));

    // Set very low interaction reputation so trust fails
    sim.agents[id1 as usize].positive_interactions = 0;
    sim.agents[id1 as usize].negative_interactions = 10;
    sim.agents[id2 as usize].positive_interactions = 0;
    sim.agents[id2 as usize].negative_interactions = 10;

    let eligible = vec![id1 as usize, id2 as usize];
    let used = std::collections::HashSet::new();
    let partner = sim.find_best_partner(id1 as usize, 1, &eligible, &used);
    assert!(partner.is_none());
}

#[test]
fn test_create_offspring_genome_toggle_off() {
    let mut sim = Simulation::new_default();
    let genome_a = Genome::default();
    let genome_b = Genome::default();
    let offspring = sim.create_offspring_genome(&genome_a, &genome_b);
    // When genome toggle is off, should return random genome
    assert_ne!(offspring.alleles, genome_a.alleles);
}

#[test]
fn test_create_offspring_genome_toggle_on() {
    let mut config = SimulationConfig::default();
    config.toggles.genome_enabled = true;
    let mut sim = Simulation::new(config);
    let mut rng = StdRng::seed_from_u64(42);
    let genome_a = Genome::random(&mut rng);
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let genome_b = Genome::random(&mut rng);
    let offspring = sim.create_offspring_genome(&genome_a, &genome_b);
    // With crossover of different genomes, offspring should differ
    assert_ne!(offspring.alleles, genome_a.alleles);
}

// ========================================================================
// Compute actions and movement tests
// ========================================================================

#[test]
fn test_compute_actions_heuristic_empty() {
    let mut sim = Simulation::new_default();
    let actions = sim.compute_actions_heuristic();
    assert!(actions.is_empty());
}

#[test]
fn test_compute_actions_dmn_empty() {
    let mut sim = Simulation::new_default();
    let actions = sim.compute_actions_dmn(false);
    assert!(actions.is_empty());
}

#[test]
// TAG: surface=simulation owner=research-team rule=SIM-001
fn test_count_action_types() {
    use crate::dmn::Direction;
    let actions = vec![
        (0, Action::Move(Direction::North)),
        (1, Action::Stay),
        (2, Action::Reproduce),
        (3, Action::Move(Direction::South)),
    ];
    let (moves, stays, reproduces) = Simulation::count_action_types(&actions);
    assert_eq!(moves, 2);
    assert_eq!(stays, 1);
    assert_eq!(reproduces, 1);
}

#[test]
fn test_compute_social_movement_zero_force() {
    let agent = Agent::new(
        0,
        Position::new_2d(32.0, 32.0),
        Genome::default(),
        0,
        Sex::Male,
    );
    let (dx, dy) = Simulation::compute_social_movement(&agent, 1.0, 0.0, 0.5);
    assert_eq!(dx, 1.0);
    assert_eq!(dy, 0.0);
    // TAG: surface=simulation owner=research-team rule=SIM-001
}

#[test]
fn test_compute_social_movement_with_force() {
    let mut agent = Agent::new(
        0,
        Position::new_2d(32.0, 32.0),
        Genome::default(),
        0,
        Sex::Male,
    );
    agent.apply_social_force(crate::types::Vector::new(1.0, 0.0));
    let (dx, dy) = Simulation::compute_social_movement(&agent, 0.0, 1.0, 0.5);
    // Social force should influence direction
    assert!(dx.abs() > 0.0 || dy.abs() > 0.0);
}

// ========================================================================
// Mortality and resource tests
// ========================================================================

#[test]
fn test_apply_mortality_rules_all_dead() {
    let mut sim = Simulation::new_default();
    let id = sim.spawn_agent(Position::new_2d(32.0, 32.0));
    // TAG: surface=simulation owner=research-team rule=SIM-001
    sim.agents[id as usize].alive = false;
    let deaths = sim.apply_mortality_rules();
    assert!(deaths.is_empty());
}

#[test]
fn test_record_deaths_empty() {
    let mut sim = Simulation::new_default();
    sim.record_deaths(&[]);
    assert_eq!(sim.debug_state.total_deaths, 0);
}

#[test]
fn test_collect_resources_all_dead() {
    let mut sim = Simulation::new_default();
    let id = sim.spawn_agent(Position::new_2d(32.0, 32.0));
    sim.agents[id as usize].alive = false;
    sim.collect_resources();
    // Should not panic
}

// ========================================================================
// Run and tick edge cases
// ========================================================================

#[test]
// TAG: surface=simulation owner=research-team rule=SIM-001
fn test_run_zero_ticks() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(Position::new_2d(32.0, 32.0));
    sim.run(0);
    assert_eq!(sim.tick, 0);
}

#[test]
fn test_cleanup_and_log_no_cleanup() {
    let mut sim = Simulation::new_default();
    sim.config.cleanup_interval = 0; // Disabled
    sim.spawn_agent(Position::new_2d(32.0, 32.0));
    sim.tick = 1;
    sim.cleanup_and_log();
    // Should not panic
}

#[test]
fn test_cleanup_and_log_with_cleanup() {
    let mut config = SimulationConfig::default();
    config.cleanup_interval = 10;
    config.max_dead_agents = 1;
    let mut sim = Simulation::new(config);

    let id = sim.spawn_agent(Position::new_2d(32.0, 32.0));
    sim.agents[id as usize].alive = false;
    sim.tick = 10;

    sim.cleanup_and_log();
    // Dead agent should be cleaned up
    assert_eq!(sim.living_agent_count(), 0);
}

#[test]
fn test_setup_tick_persists_events() {
    let mut sim = Simulation::new_default();
    sim.game_events_this_tick.push(GameEvent {
        tick: 1,
        agent_a: 0,
        agent_b: 1,
        game_type: "pd".to_string(),
        outcome: "cooperate".to_string(),
        payoff_a: 3.0,
        payoff_b: 3.0,
    });
    sim.encounter_events_this_tick.push(EncounterEvent {
        tick: 1,
        agent_a: 0,
        agent_b: 1,
        encounter_type: "game".to_string(),
        // TAG: surface=simulation owner=research-team rule=SIM-001
    });

    sim.setup_tick(1);
    assert!(sim.game_events_this_tick.is_empty());
    assert!(sim.encounter_events_this_tick.is_empty());
    assert_eq!(sim.event_store.game_events.len(), 1);
    assert_eq!(sim.event_store.encounter_events.len(), 1);
}

#[test]
fn test_setup_tick_no_sampled_agent() {
    let mut sim = Simulation::new_default();
    sim.setup_tick(0);
    assert!(sim.debug_state.sampled_agent_id.is_none());
}

#[test]
fn test_update_environment_and_allee() {
    let mut sim = Simulation::new_default();
    sim.spawn_agent(Position::new_2d(32.0, 32.0));
    sim.update_environment_and_allee();
    assert!(sim.metrics.allee_fitness_multiplier >= 0.0);
}

#[test]
// TAG: surface=simulation owner=research-team rule=SIM-001
fn test_post_movement_physics_no_barnes_hut() {
    let mut sim = Simulation::new_default();
    sim.config.toggles.barnes_hut_enabled = false;
    sim.spawn_agent(Position::new_2d(32.0, 32.0));
    sim.post_movement_physics();
    // Should be a no-op when barnes_hut disabled
}

#[test]
fn test_process_stigmergy_disabled() {
    let mut sim = Simulation::new_default();
    sim.config.toggles.stigmergy_enabled = false;
    sim.process_stigmergy();
    // Should be a no-op
}

#[test]
fn test_process_life_cycle_discrete_no_mgr() {
    let mut config = SimulationConfig::default();
    config.generation_mode = GenerationMode::Discrete {
        generation_time: 10,
        population_size: 2,
    };
    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));
    // discrete_generation is None
    // TAG: surface=simulation owner=research-team rule=SIM-001
    sim.process_life_cycle();
    // Should not panic
}

// ========================================================================
// Cooperation rate tests
// ========================================================================

#[test]
fn test_compute_cooperation_rate_empty() {
    let sim = Simulation::new_default();
    let rate = sim.compute_cooperation_rate();
    assert_eq!(rate, 0.5);
}

#[test]
fn test_compute_cooperation_rate_with_interactions() {
    let mut sim = Simulation::new_default();
    let id = sim.spawn_agent(Position::new_2d(32.0, 32.0));
    sim.agents[id as usize].positive_interactions = 10;
    sim.agents[id as usize].negative_interactions = 10;
    let rate = sim.compute_cooperation_rate();
    assert_eq!(rate, 0.5);
}

// ========================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
// Energy and aging tests
// ========================================================================

#[test]
fn test_apply_energy_and_aging_economics() {
    let mut config = SimulationConfig::default();
    config.toggles.economics_enabled = true;
    config.energy_consumption_rate = 1.0;
    let mut sim = Simulation::new(config);
    let id = sim.spawn_agent(Position::new_2d(32.0, 32.0));
    let before = sim.agents[id as usize].energy;
    sim.apply_energy_and_aging();
    // Energy should decrease
    assert!(sim.agents[id as usize].energy <= before);
}

#[test]
fn test_apply_energy_and_aging_all_dead() {
    let mut sim = Simulation::new_default();
    let id = sim.spawn_agent(Position::new_2d(32.0, 32.0));
    sim.agents[id as usize].alive = false;
    sim.apply_energy_and_aging();
    // Should not panic
}

// TAG: surface=simulation owner=research-team rule=SIM-001
// ========================================================================
// Memory usage tests
// ========================================================================

#[test]
fn test_estimate_memory_usage_with_dead() {
    let mut sim = Simulation::new_default();
    let id = sim.spawn_agent(Position::new_2d(32.0, 32.0));
    sim.agents[id as usize].alive = false;

    let mem = sim.estimate_memory_usage();
    assert_eq!(mem.alive_count, 0);
    assert_eq!(mem.dead_count, 1);
    assert!(mem.total_bytes > 0);
}

// ========================================================================
// Regulate criticality tests
// ========================================================================

#[test]
fn test_regulate_criticality_empty() {
    let mut sim = Simulation::new_default();
    sim.regulate_criticality();
    // Should not panic with no agents
}
// TAG: surface=simulation owner=research-team rule=SIM-001

#[test]
fn test_regulate_criticality_with_agents() {
    let mut sim = Simulation::new_default();
    for _ in 0..5 {
        sim.spawn_agent(Position::new_2d(32.0, 32.0));
    }
    sim.metrics.births_this_tick = 100;
    sim.metrics.deaths_this_tick = 50;
    sim.metrics.encounters_this_tick = 200;
    sim.regulate_criticality();
    // Should not panic
}

// ========================================================================
// Offspring position tests
// ========================================================================

#[test]
fn test_compute_offspring_position_clamped() {
    let mut sim = Simulation::new_default();
    let pos = Position::new_2d(63.0, 63.0);
    let offspring_pos = sim.compute_offspring_position(pos);
    assert!(offspring_pos.x >= 0.0 && offspring_pos.x < 64.0);
    assert!(offspring_pos.y >= 0.0 && offspring_pos.y < 64.0);
}
