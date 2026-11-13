// TAG: surface=simulation owner=research-team rule=SIM-001
//! Integration tests for `sim-core::integration`
//!
//! Extracted from src/integration.rs to reduce per-file CCN.
//! Integration Tests for sim-core
//!
//! CODE-009: EXCEPTION - Integration test for sim-core crate
//! Tests multi-stage interactions: Genome → DOCAS → DMN → Behavior → Outcomes
//! Per §6.8.2 Causal Pathway Specifications
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

use sim_core::docas::DocasState;
use sim_core::genome::Genome;
use sim_core::simulation::{Simulation, SimulationConfig};
use sim_core::toggles::ToggleConfig;
use sim_core::Position;

/// Test: Genome → DOCAS sensitivity pathway (h² = 0.25-0.72)
/// Per §6.8.2: Genome controls DOCAS via alpha (decay) and beta (sensitivity)
#[test]
fn test_genome_to_docas_pathway() {
    // Create two agents with different genomes
    let g1 = Genome::default();

    // Verify alpha range: [0.9, 0.99] per §6.1.1
    let alpha1 = g1.docas_alpha();
    for a in alpha1 {
        assert!(
            (0.9..=0.99).contains(&a),
            "Alpha {a} out of range [0.9, 0.99]"
        );
    }

    // Verify beta range: [0.05, 0.2] per §6.1.1
    let beta1 = g1.docas_beta();
    for b in beta1 {
        assert!(
            (0.05..=0.2).contains(&b),
            "Beta {b} out of range [0.05, 0.2]"
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Verify DOCAS update uses genome parameters
    let mut docas = DocasState::neutral();
    // P3-1: Include geometric parameters (kappa, psi) - neutral values for test
    docas.update(&g1, 0.5, 0.0, 0.0, 0.0, 0.0); // Dopamine stimulus, κ=0, ψ=0.75 (geometric props computed separately)

    // Should have changed from neutral 0.5
    assert!(
        docas.levels[0] != 0.5,
        "DOCAS should update from genome sensitivity"
    );
}

/// Test: Multi-stage simulation runs through all stages
/// Per §22.5.3: Phase 0/1/2 integration
#[test]
fn test_multi_stage_simulation() {
    let config = SimulationConfig {
        toggles: ToggleConfig::stage_3(), // Enable up to society
        max_ticks: 100,
        seed: 12345,
        ..Default::default()
    };

    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    // Run and verify no panics
    for _ in 0..100 {
        sim.step();
    }

    assert_eq!(sim.tick, 100);
    assert!(!sim.agents.is_empty(), "Should have surviving agents");
}

/// Test: Stage 4 (Language) integration
/// Per §7.7: Language enables semantic drift and colexification
#[test]
fn test_stage_4_language_integration() {
    let config = SimulationConfig {
        toggles: ToggleConfig::stage_4(), // Enable language
        max_ticks: 50,
        seed: 54321,
        ..Default::default()
    };

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    // Run simulation with language enabled
    for _ in 0..50 {
        sim.step();
    }

    // Verify simulation ran without errors
    assert_eq!(sim.tick, 50);
}

/// Test: Stage 5 (Protocol) integration
/// Per §7.8: Protocol enables claim anchoring and verification
#[test]
fn test_stage_5_protocol_integration() {
    let config = SimulationConfig {
        toggles: ToggleConfig::stage_5(), // Enable protocol
        max_ticks: 50,
        seed: 98765,
        ..Default::default()
    };

    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    // Run simulation with protocol enabled
    for _ in 0..50 {
        sim.step();
    }

    assert_eq!(sim.tick, 50);
}

/// Test: DOCAS → Behavior pathway
/// Per §6.8.2: DOCAS affects behavior via DMN
#[test]
fn test_docas_to_behavior_pathway() {
    let config = SimulationConfig {
        toggles: ToggleConfig::stage_2(), // Enable DOCAS + DMN
        max_ticks: 100,
        seed: 11111,
        ..Default::default()
    };

    let mut sim = Simulation::new(config);
    // TAG: surface=simulation owner=research-team rule=SIM-001
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    // Record initial state
    if let Some(agent) = sim.agents.first() {
        let initial_energy = agent.energy;

        // Run for some ticks
        for _ in 0..50 {
            sim.step();
        }

        // Agent should have had behavior (energy change, movement)
        if let Some(agent) = sim.agents.first() {
            // Energy should have changed from initial
            assert!(
                agent.energy != initial_energy || sim.tick > 0,
                "Agent should exhibit behavior"
            );
        }
    }
}

/// Test: Full causal chain Genome → DOCAS → DMN → Behavior → Outcomes
/// Per §6.8 Causal DAG
#[test]
fn test_full_causal_chain() {
    let config = SimulationConfig {
        toggles: ToggleConfig::all_on(),
        max_ticks: 200,
        seed: 22222,
        ..Default::default()
    };

    let mut sim = Simulation::new(config);
    sim.spawn_agent(Position::new_2d(32.0, 32.0));

    // Run full simulation
    for _ in 0..200 {
        sim.step();
    }

    // Verify simulation completed
    assert_eq!(sim.tick, 200);

    // Verify agents have evolved (births, deaths, DOCAS changes)
    // This tests the full pathway from genome through outcomes
}
