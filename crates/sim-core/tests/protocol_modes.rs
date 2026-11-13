// TAG: surface=simulation owner=research-team rule=SIM-001
//! SIM-011: Protocol Modes Integration Tests
//!
//! CODE-009: EXCEPTION - Integration test for sim-core crate (tests protocol modes)
//! Tests for Centralized Oracle vs Decentralized Protocol comparison.
//! Per SIM-011: Core research intervention for trust emergence comparison.

// Numeric cast precision is acceptable in these simulation test utilities.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::float_cmp
)]

use rand::rngs::StdRng;
use rand::SeedableRng;
use sim_core::{
    protocol::{ClaimType, ProtocolState, VerificationMode},
    simulation::{Simulation, SimulationConfig},
    types::Position,
};

/// Test centralized oracle verification behavior
#[test]
fn test_centralized_oracle_verification() {
    let config = SimulationConfig {
        verification_mode: VerificationMode::CentralizedOracle,
        ..SimulationConfig::default()
    };

    let _sim = Simulation::new(config);

    // Create protocol state with centralized mode
    let mut protocol = ProtocolState::with_mode(VerificationMode::CentralizedOracle);
    let mut rng = StdRng::seed_from_u64(42);

    // Anchor a claim - should be free in centralized mode
    let (hash, cost) = protocol.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);
    assert_eq!(cost, 0.0, "Centralized anchor should be free");

    // Verify with high reputation - should succeed
    let result = protocol.verify_claim(2, hash, 101, Some(0.5), &mut rng);
    assert!(result.verified, "High reputation agent should be verified");
    assert_eq!(
        result.latency_ticks, 0,
        "Centralized verification is instant"
    );
    assert_eq!(result.verify_cost, 0.0, "Centralized verification is free");
    assert!(result.is_centralized);
    assert_eq!(result.authority, "CentralOracle");
}

/// Test decentralized protocol verification behavior
#[test]
// TAG: surface=simulation owner=research-team rule=SIM-001
fn test_decentralized_protocol_verification() {
    let config = SimulationConfig {
        verification_mode: VerificationMode::DecentralizedProtocol,
        ..SimulationConfig::default()
    };

    let _sim = Simulation::new(config);

    // Create protocol state with decentralized mode
    let mut protocol = ProtocolState::with_mode(VerificationMode::DecentralizedProtocol);
    let mut rng = StdRng::seed_from_u64(42);

    // Anchor a claim - should have cost in decentralized mode
    let (hash, cost) = protocol.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);
    assert!(cost > 0.0, "Decentralized anchor should have cost");
    assert!(
        (0.01..=0.10).contains(&cost),
        "Cost should be in expected range"
    );

    // Verify in decentralized mode
    let result = protocol.verify_claim(2, hash, 101, None, &mut rng);
    assert!(result.verified);
    assert!(
        result.latency_ticks >= 1,
        "Decentralized verification has latency"
    );
    assert!(
        result.verify_cost > 0.0,
        "Decentralized verification has cost"
    );
    assert!(!result.is_centralized);
    assert!(
        result.authority.starts_with("0x"),
        "Authority should be blockchain address"
    );
}

/// Test mode comparison - the core research question
#[test]
fn test_mode_comparison() {
    let seed = 12345;
    let ticks = 100;

    // Centralized baseline
    let centralized_config = SimulationConfig::centralized_baseline(seed, ticks);
    assert_eq!(
        centralized_config.verification_mode,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        VerificationMode::CentralizedOracle
    );

    let mut centralized_sim = Simulation::new(centralized_config);
    centralized_sim.spawn_agent(Position::new_2d(32.0, 32.0));

    // Decentralized protocol
    let decentralized_config = SimulationConfig::decentralized_protocol(seed, ticks);
    assert_eq!(
        decentralized_config.verification_mode,
        VerificationMode::DecentralizedProtocol
    );

    let mut decentralized_sim = Simulation::new(decentralized_config);
    decentralized_sim.spawn_agent(Position::new_2d(32.0, 32.0));

    // Run both simulations
    for _ in 0..ticks {
        centralized_sim.step();
        decentralized_sim.step();
    }

    // Both should complete
    assert_eq!(centralized_sim.tick, ticks);
    assert_eq!(decentralized_sim.tick, ticks);

    // Verify modes are correctly set in protocol states
    assert_eq!(
        centralized_sim.protocol.verification_mode,
        VerificationMode::CentralizedOracle
    );
    assert_eq!(
        decentralized_sim.protocol.verification_mode,
        VerificationMode::DecentralizedProtocol
    );
}

/// Test that centralized mode rejects low-reputation agents
#[test]
fn test_centralized_reputation_threshold() {
    let mut protocol = ProtocolState::with_mode(VerificationMode::CentralizedOracle);
    let mut rng = StdRng::seed_from_u64(42);

    // Anchor a claim
    let (hash, _) = protocol.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);

    // Verify with high reputation - should succeed
    let result_high = protocol.verify_claim(2, hash, 101, Some(0.5), &mut rng);
    // TAG: surface=simulation owner=research-team rule=SIM-001
    assert!(result_high.verified, "High reputation should pass");

    // Verify with low reputation - should fail
    let result_low = protocol.verify_claim(3, hash, 101, Some(0.1), &mut rng);
    assert!(!result_low.verified, "Low reputation should fail");
}

/// Test mode switching at runtime
#[test]
fn test_mode_switching() {
    let mut protocol = ProtocolState::new();

    // Default is DecentralizedProtocol
    assert_eq!(
        protocol.verification_mode,
        VerificationMode::DecentralizedProtocol
    );

    // Switch to centralized
    protocol.set_mode(VerificationMode::CentralizedOracle);

    let mut rng = StdRng::seed_from_u64(42);
    let (_, cost) = protocol.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);
    assert_eq!(cost, 0.0, "Should use centralized mode after switch");
}

/// Test cost comparison between modes
#[test]
fn test_cost_comparison() {
    let mut rng_centralized = StdRng::seed_from_u64(42);
    let mut rng_decentralized = StdRng::seed_from_u64(42);

    let mut centralized = ProtocolState::with_mode(VerificationMode::CentralizedOracle);
    let mut decentralized = ProtocolState::with_mode(VerificationMode::DecentralizedProtocol);

    // Anchor multiple claims and compare costs
    let mut total_centralized_cost = 0.0;
    let mut total_decentralized_cost = 0.0;

    for i in 0..10 {
        let (_, cost_c) = centralized.anchor_claim(
            i as u64,
            ClaimType::IdentityVerification,
            100 + i as u64,
            &mut rng_centralized,
        );
        let (_, cost_d) = decentralized.anchor_claim(
            i as u64,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            ClaimType::IdentityVerification,
            100 + i as u64,
            &mut rng_decentralized,
        );

        total_centralized_cost += cost_c;
        total_decentralized_cost += cost_d;
    }

    // Centralized should be free, decentralized should have cost
    assert_eq!(
        total_centralized_cost, 0.0,
        "Centralized total cost should be 0"
    );
    assert!(
        total_decentralized_cost > 0.0,
        "Decentralized should have cost"
    );

    println!("Centralized total cost: {total_centralized_cost}");
    println!("Decentralized total cost: {total_decentralized_cost}");
}

/// Test latency comparison between modes
#[test]
fn test_latency_comparison() {
    let mut rng = StdRng::seed_from_u64(42);

    let mut centralized = ProtocolState::with_mode(VerificationMode::CentralizedOracle);
    let mut decentralized = ProtocolState::with_mode(VerificationMode::DecentralizedProtocol);

    // Anchor claims
    let (hash_c, _) = centralized.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);
    let (hash_d, _) = decentralized.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);

    // Verify and collect latencies
    let result_c = centralized.verify_claim(2, hash_c, 101, Some(0.5), &mut rng);
    let result_d = decentralized.verify_claim(2, hash_d, 101, None, &mut rng);

    // Centralized should be instant (0 ticks)
    assert_eq!(
        result_c.latency_ticks, 0,
        "Centralized should have 0 latency"
    );

    // Decentralized should have latency (> 0 ticks)
    assert!(
        result_d.latency_ticks > 0,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        "Decentralized should have latency"
    );

    println!("Centralized latency: {} ticks", result_c.latency_ticks);
    println!("Decentralized latency: {} ticks", result_d.latency_ticks);
}

/// Test authority identification differences
#[test]
fn test_authority_identification() {
    let mut rng = StdRng::seed_from_u64(42);

    let mut centralized = ProtocolState::with_mode(VerificationMode::CentralizedOracle);
    let mut decentralized = ProtocolState::with_mode(VerificationMode::DecentralizedProtocol);

    // Anchor and verify
    let (hash_c, _) = centralized.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);
    let (hash_d, _) = decentralized.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);

    let result_c = centralized.verify_claim(2, hash_c, 101, Some(0.5), &mut rng);
    let result_d = decentralized.verify_claim(2, hash_d, 101, None, &mut rng);

    // Centralized uses "CentralOracle"
    assert_eq!(result_c.authority, "CentralOracle");

    // Decentralized uses blockchain-style address
    assert!(result_d.authority.starts_with("0x"));
    assert_ne!(result_d.authority, "CentralOracle");
}

/// Test the full comparison workflow as described in SIM-011
#[test]
fn test_full_comparison_workflow() {
    // This test demonstrates the complete research workflow
    let seed = 99999;
    let ticks = 50;

    // Step 1: Configure both modes with same parameters
    let centralized_config = SimulationConfig {
        verification_mode: VerificationMode::CentralizedOracle,
        seed,
        max_ticks: ticks,
        ..SimulationConfig::default()
    };

    let decentralized_config = SimulationConfig {
        verification_mode: VerificationMode::DecentralizedProtocol,
        seed,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        max_ticks: ticks,
        ..SimulationConfig::default()
    };

    // Step 2: Create simulations
    let mut sim_centralized = Simulation::new(centralized_config);
    let mut sim_decentralized = Simulation::new(decentralized_config);

    // Step 3: Spawn identical agents
    for i in 0..5 {
        let pos = Position::new_2d(30.0 + i as f32, 30.0);
        sim_centralized.spawn_agent(pos);
        sim_decentralized.spawn_agent(pos);
    }

    // Step 4: Run simulations
    for _ in 0..ticks {
        sim_centralized.step();
        sim_decentralized.step();
    }

    // Step 5: Compare results
    println!("=== SIM-011 Comparison Results ===");
    println!("Centralized tick: {}", sim_centralized.tick);
    println!("Decentralized tick: {}", sim_decentralized.tick);
    println!("Centralized agents: {}", sim_centralized.agents.len());
    println!("Decentralized agents: {}", sim_decentralized.agents.len());

    // Verify protocol states are tracking correctly
    println!(
        "Centralized anchors: {}",
        sim_centralized.protocol.total_anchors
    );
    println!(
        "Decentralized anchors: {}",
        sim_decentralized.protocol.total_anchors
    );
    println!(
        "Centralized verifications: {}",
        sim_centralized.protocol.total_verifications
    );
    println!(
        "Decentralized verifications: {}",
        sim_decentralized.protocol.total_verifications
    );

    // Both should have completed successfully
    assert_eq!(sim_centralized.tick, ticks);
    assert_eq!(sim_decentralized.tick, ticks);
}

/// Test that block_time, consensus_threshold, and protocol_anchor_cost_range
/// produce non-zero dynamical Jacobian sensitivity on the simulation trajectory.
#[test]
fn test_protocol_block_time_and_consensus_threshold_causal_jacobian() {
    let seed = 42;
    let ticks = 40;

    let base_toggles = sim_core::toggles::ToggleConfig::stage_5();

    // Baseline: fast block time (1 tick), low consensus threshold (0.2)
    let fast_config = SimulationConfig {
        verification_mode: VerificationMode::DecentralizedProtocol,
        seed,
        max_ticks: ticks,
        block_time: 1,
        consensus_threshold: 0.2,
        toggles: base_toggles,
        ..SimulationConfig::default()
    };

    // Slow: block time 10 ticks, high consensus threshold (0.9), higher anchor cost
    let mut slow_toggles = base_toggles;
    slow_toggles.dials.protocol_anchor_cost_range = (0.05, 0.15);

    let slow_config = SimulationConfig {
        verification_mode: VerificationMode::DecentralizedProtocol,
        seed,
        max_ticks: ticks,
        block_time: 10,
        consensus_threshold: 0.9,
        toggles: slow_toggles,
        ..SimulationConfig::default()
    };

    let mut sim_fast = Simulation::new(fast_config);
    let mut sim_slow = Simulation::new(slow_config);

    for i in 0..25 {
        let pos = Position::new_2d(30.0 + (i as f32 * 0.2), 30.0 + (i as f32 * 0.2));
        sim_fast.spawn_agent(pos);
        sim_slow.spawn_agent(pos);
    }

    for _ in 0..ticks {
        sim_fast.step();
        sim_slow.step();
    }

    // Causal Jacobian validation: block_time and consensus_threshold MUST diverge verification history
    let fast_results: Vec<_> = sim_fast
        .protocol
        .verification_history
        .values()
        .flatten()
        .collect();
    let slow_results: Vec<_> = sim_slow
        .protocol
        .verification_history
        .values()
        .flatten()
        .collect();

    assert!(
        !fast_results.is_empty(),
        "Fast simulation must record verifications"
    );
    assert_eq!(
        fast_results[0].latency_ticks, 1,
        "Fast simulation must have block_time = 1 latency"
    );

    if !slow_results.is_empty() {
        assert_eq!(
            slow_results[0].latency_ticks, 10,
            "Slow simulation must have block_time = 10 latency"
        );
    }

    // Divergence in successful verification count or trajectory demonstrates non-zero Jacobian
    assert_ne!(
        (sim_fast.protocol.successful_verifications, sim_fast.protocol.total_verifications),
        (sim_slow.protocol.successful_verifications, sim_slow.protocol.total_verifications),
        "Perturbing block_time and consensus_threshold must induce non-zero dynamical Jacobian divergence"
    );
}
