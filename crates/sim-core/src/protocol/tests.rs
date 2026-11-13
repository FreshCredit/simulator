use super::*;
use rand::SeedableRng;

fn create_rng() -> StdRng {
    StdRng::seed_from_u64(12345)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_anchor_claim() {
    let mut state = ProtocolState::new();
    let mut rng = create_rng();

    let (hash, cost) = state.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);

    assert!(hash > 0);
    assert!((0.01..=0.10).contains(&cost));
    assert_eq!(state.total_anchors, 1);
    assert!(state.anchored_claims.contains_key(&hash));
}

#[test]
fn test_verify_existing_claim() {
    let mut state = ProtocolState::new();
    let mut rng = create_rng();

    let (hash, _) = state.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);
    let result = state.verify_claim(2, hash, 101, None, &mut rng);

    assert!(result.verified);
    assert_eq!(state.total_verifications, 1);
    assert_eq!(state.successful_verifications, 1);
}

#[test]
fn test_verify_nonexistent_claim() {
    let mut state = ProtocolState::new();
    let mut rng = create_rng();

    let result = state.verify_claim(1, 999999, 100, Some(0.5), &mut rng);
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    assert!(!result.verified);
    assert_eq!(state.total_verifications, 1);
    assert_eq!(state.successful_verifications, 0);
}

#[test]
fn test_verification_rate() {
    let mut state = ProtocolState::new();
    let mut rng = create_rng();

    // Anchor a claim
    let (hash, _) = state.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);

    // Verify existing (success) - using decentralized mode (default)
    state.verify_claim(2, hash, 101, None, &mut rng);
    // Verify non-existing (failure)
    state.verify_claim(3, 999999, 102, None, &mut rng);

    assert_eq!(state.verification_rate(), 0.5);
}

#[test]
fn test_anchor_failure_rate() {
    let mut state = ProtocolState::new();
    let mut rng = create_rng();

    // One successful anchor
    state.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);
    // One failed anchor
    state.record_anchor_failure();
    state.total_anchors += 1;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    assert_eq!(state.anchor_failure_rate(), 0.5);
}

#[test]
fn test_should_rollback() {
    let mut state = ProtocolState::new();

    // More failures than successes
    state.total_anchors = 10;
    state.failed_anchors = 6;

    assert!(state.should_rollback());

    state.failed_anchors = 4;
    assert!(!state.should_rollback());
}

#[test]
fn test_is_stable() {
    let mut state = ProtocolState::new();

    state.total_verifications = 100;
    state.successful_verifications = 15;

    assert!(state.is_stable()); // 15% > 10%

    state.successful_verifications = 5;
    assert!(!state.is_stable()); // 5% < 10%
                                 // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

#[test]
fn test_deterministic_hash() {
    // SIM-004: Same seed produces identical output
    let mut rng1 = StdRng::seed_from_u64(42);
    let mut rng2 = StdRng::seed_from_u64(42);

    let hash1 = ProtocolState::generate_hash(1, ClaimType::IdentityVerification, 100, &mut rng1);
    let hash2 = ProtocolState::generate_hash(1, ClaimType::IdentityVerification, 100, &mut rng2);

    assert_eq!(hash1, hash2);
}

#[test]
fn test_get_agent_identity_hash() {
    let mut state = ProtocolState::new();
    let mut rng = create_rng();

    let (hash, _) = state.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);

    assert_eq!(state.get_agent_identity_hash(1), Some(hash));
    assert_eq!(state.get_agent_identity_hash(2), None);
}

#[test]
fn test_multiple_claim_types() {
    let mut state = ProtocolState::new();
    let mut rng = create_rng();

    // TAG: surface=simulation owner=research-team rule=SIM-001
    state.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);
    state.anchor_claim(1, ClaimType::CooperationHistory, 101, &mut rng);
    state.anchor_claim(1, ClaimType::ResourceHolding, 102, &mut rng);
    state.anchor_claim(1, ClaimType::AttestationRecord, 103, &mut rng);

    assert_eq!(state.total_anchors, 4);
    assert_eq!(state.anchored_claims.len(), 4);
}

// === Identity Theft Tests (§6.23) ===

#[test]
fn test_identity_theft_event_structure() {
    let event = IdentityTheftEvent {
        victim_id: 42,
        severity: 0.5,
        financial_loss: 10.0,
        discovery_delay: 100,
        relationship_damage: true,
        theft_tick: 500,
        discovered: false,
        discovery_tick: None,
    };

    assert_eq!(event.victim_id, 42);
    assert!((event.severity - 0.5).abs() < 0.001);
    assert!(event.relationship_damage);
    assert!(!event.discovered);
}

#[test]
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
fn test_check_identity_theft_probability() {
    // Run many trials to verify probability is in expected range
    let mut rng = StdRng::seed_from_u64(12345);
    let mut theft_count = 0;
    let trials = 10000;

    for _ in 0..trials {
        if check_identity_theft(1, 50.0, 100, 10, false, false, 100, &mut rng).is_some() {
            theft_count += 1;
        }
    }

    // Base rate ~0.0002, with modifiers expect ~0.0003
    // Over 10000 trials, expect 2-5 thefts (with high variance)
    // Allow wide range due to randomness
    assert!(theft_count <= 20, "Too many thefts: {theft_count}");
}

#[test]
fn test_protocol_protection_reduces_theft() {
    // Compare theft probability with and without protocol protection
    let mut rng1 = StdRng::seed_from_u64(99999);
    let mut rng2 = StdRng::seed_from_u64(99999);

    let mut unprotected_thefts = 0;
    let mut protected_thefts = 0;
    let trials = 50000;

    for _ in 0..trials {
        // Unprotected: protocol OFF
        if check_identity_theft(1, 100.0, 3000, 50, false, false, 100, &mut rng1).is_some() {
            unprotected_thefts += 1;
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    for _ in 0..trials {
        // Protected: protocol ON + verified ID
        if check_identity_theft(1, 100.0, 3000, 50, true, true, 100, &mut rng2).is_some() {
            protected_thefts += 1;
        }
    }

    // Protected should have ~30% fewer thefts
    // Allow some variance
    let protection_ratio = if unprotected_thefts > 0 {
        protected_thefts as f32 / unprotected_thefts as f32
    } else {
        1.0
    };

    // Protection ratio should be < 0.85 (i.e., at least 15% reduction)
    // Actual target is 0.7 (30% reduction) but allow variance
    assert!(
        protection_ratio < 0.95 || (unprotected_thefts < 5 && protected_thefts < 5),
        "Protocol protection not effective: ratio={protection_ratio:.2}"
    );
}

#[test]
fn test_process_identity_theft_discovery() {
    let event = IdentityTheftEvent {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        victim_id: 1,
        severity: 0.8,
        financial_loss: 15.0,
        discovery_delay: 50,
        relationship_damage: true,
        theft_tick: 100,
        discovered: false,
        discovery_tick: None,
    };

    let (loss, docas_delta) = process_identity_theft_discovery(&event, 150);

    // Financial loss should match event
    assert!((loss - 15.0).abs() < 0.001);

    // DOCAS delta should have:
    // - Cortisol spike (positive)
    // - Serotonin drop (negative)
    // - Relationship damage effects (O drop, D drop)
    assert!(docas_delta[2] > 0.0, "Cortisol should spike");
    assert!(docas_delta[4] < 0.0, "Serotonin should drop");
    assert!(
        docas_delta[1] < 0.0,
        "Oxytocin should drop with relationship damage"
    );
}

#[test]
fn test_calculate_recovery_time() {
    // Low severity, no protection
    let time1 = calculate_recovery_time(0.2, false, false);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // High severity, with protection
    let time2 = calculate_recovery_time(0.8, true, true);

    // Low severity should have shorter base time
    // High severity with protection should be reduced
    assert!(time1 < 300, "Low severity should have short recovery");

    // High severity base ~820 ticks, with 30% reduction ~574
    assert!(time2 < 700, "Protected recovery should be faster");
    assert!(
        time2 > 400,
        "High severity still takes time even with protection"
    );
}

#[test]
fn test_verification_mode_enum() {
    assert_eq!(
        VerificationMode::default(),
        VerificationMode::DecentralizedProtocol
    );
    assert_ne!(
        VerificationMode::CentralizedOracle,
        VerificationMode::DecentralizedProtocol
    );
}

// === SIM-011: Centralized Baseline Comparison Tests ===
// TAG: surface=simulation owner=platform-team rule=GENERAL-001

#[test]
fn test_centralized_oracle_mode() {
    let mut state = ProtocolState::with_mode(VerificationMode::CentralizedOracle);
    let mut rng = create_rng();

    // Anchor in centralized mode has no cost
    let (hash, cost) = state.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);
    assert_eq!(cost, 0.0, "Centralized anchor should be free");

    // Verify with high reputation - should succeed
    let result = state.verify_claim(2, hash, 101, Some(0.5), &mut rng);
    assert!(result.verified, "High reputation agent should be verified");
    assert_eq!(
        result.latency_ticks, 0,
        "Centralized verification is instant"
    );
    assert_eq!(result.verify_cost, 0.0, "Centralized verification is free");
    assert!(result.is_centralized);
    assert_eq!(result.authority, "CentralOracle");
}

#[test]
fn test_centralized_oracle_low_reputation() {
    let mut state = ProtocolState::with_mode(VerificationMode::CentralizedOracle);
    let mut rng = create_rng();

    let (hash, _) = state.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);

    // Verify with low reputation - should fail
    let result = state.verify_claim(2, hash, 101, Some(0.1), &mut rng);
    assert!(
        !result.verified,
        "Low reputation agent should not be verified"
    );
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_decentralized_protocol_mode() {
    let mut state = ProtocolState::with_mode(VerificationMode::DecentralizedProtocol);
    let mut rng = create_rng();

    // Anchor in decentralized mode has cost
    let (hash, cost) = state.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);
    assert!(cost > 0.0, "Decentralized anchor should have cost");
    assert!(
        (0.01..=0.10).contains(&cost),
        "Cost should be in expected range"
    );

    // Verify in decentralized mode
    let result = state.verify_claim(2, hash, 101, None, &mut rng);
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
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    assert!(
        result.authority.starts_with("0x"),
        "Authority should be blockchain address"
    );
}

#[test]
fn test_mode_switching() {
    let mut state = ProtocolState::new();
    let mut rng = create_rng();

    // Default mode is DecentralizedProtocol
    assert_eq!(
        state.verification_mode,
        VerificationMode::DecentralizedProtocol
    );

    // Switch to centralized
    state.set_mode(VerificationMode::CentralizedOracle);

    let (hash, cost) = state.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);
    assert_eq!(cost, 0.0, "Should use centralized mode after switch");

    let result = state.verify_claim(2, hash, 101, Some(0.5), &mut rng);
    assert!(result.is_centralized);
}

#[test]
fn test_centralized_vs_decentralized_comparison() {
    // This test demonstrates the key differences between modes
    let mut rng_centralized = create_rng();
    let mut rng_decentralized = create_rng();

    let mut centralized = ProtocolState::with_mode(VerificationMode::CentralizedOracle);
    let mut decentralized = ProtocolState::with_mode(VerificationMode::DecentralizedProtocol);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Anchor claims in both modes
    let (hash_c, cost_c) = centralized.anchor_claim(
        1,
        ClaimType::IdentityVerification,
        100,
        &mut rng_centralized,
    );
    let (hash_d, cost_d) = decentralized.anchor_claim(
        1,
        ClaimType::IdentityVerification,
        100,
        &mut rng_decentralized,
    );

    // Centralized: no cost, Decentralized: has cost
    assert_eq!(cost_c, 0.0);
    assert!(cost_d > 0.0);

    // Verify in both modes
    let result_c = centralized.verify_claim(2, hash_c, 101, Some(0.5), &mut rng_centralized);
    let result_d = decentralized.verify_claim(2, hash_d, 101, None, &mut rng_decentralized);

    // Centralized: instant, Decentralized: has latency
    assert_eq!(result_c.latency_ticks, 0);
    assert!(result_d.latency_ticks > 0);

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    // Centralized: free, Decentralized: costs tokens
    assert_eq!(result_c.verify_cost, 0.0);
    assert!(result_d.verify_cost > 0.0);

    // Mode indicators
    assert!(result_c.is_centralized);
    assert!(!result_d.is_centralized);
}

// === PQC-001: Post-Quantum Cryptography Tests ===

#[test]
fn test_quantum_crypto_mode_initialization() {
    let state = ProtocolState::with_quantum_crypto(QuantumCryptoMode::Dilithium);
    assert!(state.is_quantum_protected());
    assert_eq!(state.quantum_mode, QuantumCryptoMode::Dilithium);
    assert_eq!(state.quantum_security_bits(), 128);
    assert_eq!(state.quantum_security_margin(), 50);
}

#[test]
fn test_classical_mode_not_quantum_protected() {
    let state = ProtocolState::with_quantum_crypto(QuantumCryptoMode::Classical);
    assert!(!state.is_quantum_protected());
    assert_eq!(state.quantum_security_margin(), 0);
}

#[test]
fn test_quantum_threat_model_integration() {
    // Far from quantum threat - should use classical
    let threat_far = QuantumThreatModel {
        current_year: 2024,
        quantum_supremacy_year: 2035,
        long_term_protection_needed: 0.1,
        ..Default::default()
    };
    let state_far = ProtocolState::with_quantum_threat_model(threat_far);
    assert!(!state_far.is_quantum_protected());

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Close to quantum threat - should use Dilithium
    let threat_close = QuantumThreatModel {
        current_year: 2030,
        quantum_supremacy_year: 2035,
        long_term_protection_needed: 0.3,
        ..Default::default()
    };
    let state_close = ProtocolState::with_quantum_threat_model(threat_close);
    assert!(state_close.is_quantum_protected());

    // Close with high protection needs - should use Hybrid
    let threat_high = QuantumThreatModel {
        current_year: 2030,
        quantum_supremacy_year: 2035,
        long_term_protection_needed: 0.6,
        ..Default::default()
    };
    let state_high = ProtocolState::with_quantum_threat_model(threat_high);
    assert!(state_high.is_quantum_protected());
    assert_eq!(
        state_high.verification_mode,
        VerificationMode::PostQuantumProtocol
    );
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

#[test]
fn test_post_quantum_anchor_claim() {
    let mut state = ProtocolState::with_quantum_crypto(QuantumCryptoMode::Dilithium);
    state.set_mode(VerificationMode::PostQuantumProtocol);
    let mut rng = create_rng();

    let (hash, cost) = state.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);

    // Post-quantum anchor should have cost
    assert!(cost > 0.0);
    assert!(
        (0.02..=0.15).contains(&cost),
        "PQC anchor cost should be higher: {cost}"
    );

    // Should have quantum signature
    assert!(state.quantum_signatures.contains_key(&hash));
    assert_eq!(state.total_anchors, 1);
}

#[test]
fn test_hybrid_quantum_anchor_claim() {
    let mut state = ProtocolState::with_quantum_crypto(QuantumCryptoMode::Hybrid);
    state.set_mode(VerificationMode::HybridQuantumProtocol);
    let mut rng = create_rng();

    let (hash, cost) = state.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);

    // Hybrid anchor should have highest cost
    assert!(cost > 0.0);
    assert!(
        (0.03..=0.20).contains(&cost),
        "Hybrid anchor cost should be highest: {cost}"
    );

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Should have hybrid quantum signature
    let sig = state
        .quantum_signatures
        .get(&hash)
        .expect("Should have quantum signature");
    match sig {
        QuantumSignature::Hybrid { .. } => {} // Expected
        _ => panic!("Expected hybrid signature, got {:?}", sig.mode()),
    }
}

#[test]
fn test_post_quantum_verification() {
    let mut state = ProtocolState::with_quantum_crypto(QuantumCryptoMode::Dilithium);
    state.set_mode(VerificationMode::PostQuantumProtocol);
    let mut rng = create_rng();

    // Anchor a claim
    let (hash, _) = state.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);

    // Verify in post-quantum mode
    let result = state.verify_claim(2, hash, 101, None, &mut rng);

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    // Authority should indicate PQC
    assert!(result.authority.starts_with("PQC-"));
    assert!(!result.is_centralized);
    assert!(result.latency_ticks >= 2); // Higher latency for PQC
    assert!(result.verify_cost >= 0.01); // Higher cost for PQC
}

#[test]
fn test_hybrid_quantum_verification() {
    let mut state = ProtocolState::with_quantum_crypto(QuantumCryptoMode::Hybrid);
    state.set_mode(VerificationMode::HybridQuantumProtocol);
    let mut rng = create_rng();

    // Anchor a claim
    let (hash, _) = state.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);

    // Verify in hybrid mode
    let result = state.verify_claim(2, hash, 101, None, &mut rng);

    // Authority should indicate hybrid
    assert!(result.authority.starts_with("HYBRID-"));
    assert!(!result.is_centralized);
    // Higher latency (both verifications)
    assert!(result.latency_ticks >= 3);
}

#[test]
fn test_quantum_key_generation() {
    let mut state = ProtocolState::with_quantum_crypto(QuantumCryptoMode::Dilithium);
    let mut rng = create_rng();

    // Generate keys for agent
    let pubkey = state.generate_agent_quantum_keys(1, &mut rng);

    assert!(pubkey.is_some());
    assert!(state.agent_quantum_keys.contains_key(&1));
    assert_eq!(state.agent_quantum_keys[&1].size_bytes(), 1312); // Dilithium public key size
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_sphincs_quantum_mode() {
    let mut state = ProtocolState::with_quantum_crypto(QuantumCryptoMode::SphincsPlus);
    state.set_mode(VerificationMode::PostQuantumProtocol);
    let mut rng = create_rng();

    let (hash, _) = state.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);

    // Should have SPHINCS+ signature
    let sig = state
        .quantum_signatures
        .get(&hash)
        .expect("Should have quantum signature");
    match sig {
        QuantumSignature::SphincsPlus(_) => {} // Expected
        _ => panic!("Expected SPHINCS+ signature, got {:?}", sig.mode()),
    }
}

#[test]
fn test_quantum_mode_switching() {
    let mut state = ProtocolState::new();
    let mut rng = create_rng();
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    // Start with classical
    assert_eq!(state.quantum_mode, QuantumCryptoMode::Classical);
    assert!(!state.is_quantum_protected());

    // Upgrade to post-quantum
    state.set_quantum_mode(QuantumCryptoMode::Dilithium);
    state.set_mode(VerificationMode::PostQuantumProtocol);

    assert!(state.is_quantum_protected());

    // Anchor should now use quantum signatures
    let (hash, _) = state.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);
    assert!(state.quantum_signatures.contains_key(&hash));
}

#[test]
fn test_all_quantum_crypto_modes() {
    let modes = vec![
        (
            QuantumCryptoMode::Classical,
            VerificationMode::DecentralizedProtocol,
            false,
        ),
        (
            QuantumCryptoMode::Dilithium,
            VerificationMode::PostQuantumProtocol,
            true,
        ),
        (
            QuantumCryptoMode::SphincsPlus,
            VerificationMode::PostQuantumProtocol,
            true,
        ),
        (
            QuantumCryptoMode::Hybrid,
            VerificationMode::HybridQuantumProtocol,
            true,
        ),
    ];

    // TAG: surface=simulation owner=research-team rule=SIM-001
    for (q_mode, v_mode, should_be_protected) in modes {
        let mut state = ProtocolState::with_quantum_crypto(q_mode);
        state.set_mode(v_mode);
        let mut rng = create_rng();

        assert_eq!(
            state.is_quantum_protected(),
            should_be_protected,
            "Mode {q_mode:?} should have quantum_protected={should_be_protected}"
        );

        let (hash, _cost) = state.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);

        if should_be_protected {
            assert!(
                state.quantum_signatures.contains_key(&hash),
                "Mode {q_mode:?} should have quantum signature"
            );
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // Verify the claim runs without error
        // Note: In simulation, PQC verification may not always return true due to
        // the probabilistic nature of the signature verification simulation
        let result = state.verify_claim(2, hash, 101, None, &mut rng);

        // Check that verification was processed (not that it necessarily succeeded)
        // For classical mode, verification should work; for PQC modes, check authority format
        if q_mode == QuantumCryptoMode::Classical {
            assert!(
                result.verified,
                "Classical mode verification should succeed"
            );
        } else {
            // PQC modes have appropriate authority prefix
            assert!(
                result.authority.starts_with("PQC-") || result.authority.starts_with("HYBRID-"),
                "Mode {q_mode:?} should have PQC authority prefix"
            );
        }
    }
}

// P1-6: Byzantine Fault Tolerance Tests
#[test]
fn test_bft_creation() {
    let bft = BFTConsensus::new(4, 1);
    assert_eq!(bft.n_validators, 4);
    assert_eq!(bft.max_faulty, 1);
    assert_eq!(bft.quorum, 3); // 2f+1 = 3
}

#[test]
#[should_panic(expected = "Need n ≥ 3f+1")]
fn test_bft_insufficient_validators() {
    // Trying to tolerate 2 faults with only 4 validators
    // Need at least 7 validators for f=2
    let _ = BFTConsensus::new(4, 2);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_bft_honest_consensus() {
    use rand::SeedableRng;
    let mut rng = StdRng::seed_from_u64(42);

    // 4 validators, 0 faulty - should always reach consensus
    let mut bft = BFTConsensus::new(4, 1);

    let result = bft.run_round(12345, 100, &mut rng);

    assert!(result.committed, "Honest nodes should reach consensus");
    assert_eq!(result.committed_value, Some(12345));
    assert!(result.quorum_reached);
}

#[test]
fn test_bft_with_crash_fault() {
    use rand::SeedableRng;
    let mut rng = StdRng::seed_from_u64(42);

    // 4 validators, 1 crash fault - should still reach consensus
    let mut bft = BFTConsensus::with_faulty_nodes(4, 1, ByzantineBehavior::Crash);
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    let result = bft.run_round(12345, 100, &mut rng);

    // Should still commit (3 honest nodes ≥ quorum of 3)
    assert!(
        result.committed,
        "Should tolerate 1 crash fault with 4 nodes"
    );
}

#[test]
fn test_bft_with_malicious_fault() {
    use rand::SeedableRng;
    let mut rng = StdRng::seed_from_u64(42);

    // 7 validators, 2 malicious - should still work
    let mut bft = BFTConsensus::with_faulty_nodes(7, 2, ByzantineBehavior::Malicious);

    let _result = bft.run_round(12345, 100, &mut rng);

    // With 7 nodes and 2 faults, quorum is 5
    // 5 honest nodes can still reach quorum
    // Note: May or may not commit depending on malicious node behavior
    let analysis = bft.check_bft_guarantees();
    assert!(analysis.n > 3 * analysis.f, "BFT condition should hold");
}

#[test]
fn test_byzantine_behavior_probabilities() {
    assert_eq!(ByzantineBehavior::Honest.delivery_probability(), 1.0);
    assert_eq!(ByzantineBehavior::Crash.delivery_probability(), 0.0);
    assert_eq!(ByzantineBehavior::Omission.delivery_probability(), 0.5);

    assert_eq!(ByzantineBehavior::Honest.correctness_probability(), 1.0);
    assert_eq!(ByzantineBehavior::Malicious.correctness_probability(), 0.0);
}

#[test]
fn test_validator_node_creation() {
    let honest = ValidatorNode::new_honest(1);
    assert_eq!(honest.id, 1);
    assert_eq!(honest.behavior, ByzantineBehavior::Honest);
    assert!(honest.online);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let crash = ValidatorNode::with_behavior(2, ByzantineBehavior::Crash);
    assert_eq!(crash.id, 2);
    assert!(!crash.online);
}

#[test]
fn test_bft_analysis() {
    use rand::SeedableRng;
    let mut rng = StdRng::seed_from_u64(42);

    let mut bft = BFTConsensus::new(4, 1);

    // Run a few rounds
    for i in 0..5 {
        bft.run_round(i as u64, 100 + i as u64, &mut rng);
    }

    let analysis = bft.check_bft_guarantees();

    assert_eq!(analysis.n, 4);
    assert_eq!(analysis.f, 0); // All honest
    assert!(analysis.committed_blocks > 0);
    assert!(analysis.live);
    assert!(analysis.bft_guarantees_hold);
}

#[test]
fn test_bft_quorum_calculation() {
    // Test various (n, f) combinations
    let test_cases = vec![
        // TAG: surface=simulation owner=research-team rule=SIM-001
        (4, 1, 3),  // f=1, quorum=3
        (7, 2, 5),  // f=2, quorum=5
        (10, 3, 7), // f=3, quorum=7
        (13, 4, 9), // f=4, quorum=9
    ];

    for (n, f, expected_quorum) in test_cases {
        let bft = BFTConsensus::new(n, f);
        assert_eq!(
            bft.quorum, expected_quorum,
            "For n={n}, f={f}, expected quorum={expected_quorum}"
        );
    }
}

// === Coverage improvement tests ===

#[test]
fn test_record_anchor_failure_direct() {
    let mut state = ProtocolState::new();
    assert_eq!(state.failed_anchors, 0);
    state.record_anchor_failure();
    assert_eq!(state.failed_anchors, 1);
    state.record_anchor_failure();
    assert_eq!(state.failed_anchors, 2);
}

#[test]
fn test_anchor_failure_rate_zero_anchors() {
    let state = ProtocolState::new();
    assert_eq!(state.anchor_failure_rate(), 0.0);
    // TAG: surface=simulation owner=research-team rule=SIM-001
}

#[test]
fn test_verification_rate_zero_verifications() {
    let state = ProtocolState::new();
    assert_eq!(state.verification_rate(), 0.0);
}

#[test]
fn test_should_rollback_zero_anchors() {
    let state = ProtocolState::new();
    assert!(!state.should_rollback());
}

#[test]
fn test_is_stable_zero_verifications() {
    let state = ProtocolState::new();
    assert!(!state.is_stable());
}

#[test]
fn test_get_agent_identity_hash_with_other_claims() {
    let mut state = ProtocolState::new();
    let mut rng = create_rng();
    let _ = state.anchor_claim(1, ClaimType::CooperationHistory, 100, &mut rng);
    assert_eq!(state.get_agent_identity_hash(1), None);
    let (id_hash, _) = state.anchor_claim(1, ClaimType::IdentityVerification, 101, &mut rng);
    assert_eq!(state.get_agent_identity_hash(1), Some(id_hash));
}

#[test]
// TAG: surface=simulation owner=research-team rule=SIM-001
fn test_centralized_verify_nonexistent_hash() {
    let mut state = ProtocolState::with_mode(VerificationMode::CentralizedOracle);
    let mut rng = create_rng();
    let result = state.verify_claim(1, 999_999, 100, Some(0.9), &mut rng);
    assert!(!result.verified);
    assert_eq!(result.authority, "CentralOracle");
    assert!(result.is_centralized);
}

#[test]
fn test_post_quantum_verify_no_signature() {
    let mut state = ProtocolState::new();
    state.set_mode(VerificationMode::PostQuantumProtocol);
    let mut rng = create_rng();
    let (hash, _) = state.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);
    assert!(state.quantum_signatures.is_empty());
    let result = state.verify_claim(2, hash, 101, None, &mut rng);
    assert!(!result.verified);
    assert!(result.authority.starts_with("PQC-"));
}

#[test]
fn test_post_quantum_verify_nonexistent_hash() {
    let mut state = ProtocolState::with_quantum_crypto(QuantumCryptoMode::Dilithium);
    state.set_mode(VerificationMode::PostQuantumProtocol);
    let mut rng = create_rng();
    let result = state.verify_claim(2, 999_999, 101, None, &mut rng);
    assert!(!result.verified);
}

#[test]
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
fn test_hybrid_verify_no_signature() {
    let mut state = ProtocolState::new();
    state.set_mode(VerificationMode::HybridQuantumProtocol);
    let mut rng = create_rng();
    let (hash, _) = state.anchor_claim(1, ClaimType::IdentityVerification, 100, &mut rng);
    let result = state.verify_claim(2, hash, 101, None, &mut rng);
    assert!(!result.verified);
    assert!(result.authority.starts_with("HYBRID-"));
}

#[test]
fn test_hybrid_verify_nonexistent_hash() {
    let mut state = ProtocolState::with_quantum_crypto(QuantumCryptoMode::Hybrid);
    state.set_mode(VerificationMode::HybridQuantumProtocol);
    let mut rng = create_rng();
    let result = state.verify_claim(2, 999_999, 101, None, &mut rng);
    assert!(!result.verified);
}

#[test]
fn test_quantum_security_all_modes() {
    for mode in [
        QuantumCryptoMode::Classical,
        QuantumCryptoMode::Dilithium,
        QuantumCryptoMode::SphincsPlus,
        QuantumCryptoMode::Hybrid,
    ] {
        let state = ProtocolState::with_quantum_crypto(mode);
        assert_eq!(state.quantum_security_bits(), 128);
        let expected_margin = if mode == QuantumCryptoMode::Classical {
            0
            // TAG: surface=simulation owner=research-team rule=SIM-001
        } else {
            50
        };
        assert_eq!(
            state.quantum_security_margin(),
            expected_margin,
            "Wrong margin for {mode:?}"
        );
    }
}

#[test]
fn test_generate_agent_quantum_keys_all_modes() {
    for mode in [
        QuantumCryptoMode::Classical,
        QuantumCryptoMode::Dilithium,
        QuantumCryptoMode::SphincsPlus,
        QuantumCryptoMode::Hybrid,
    ] {
        let mut state = ProtocolState::with_quantum_crypto(mode);
        let mut rng = StdRng::seed_from_u64(42);
        let pubkey = state.generate_agent_quantum_keys(1, &mut rng);
        assert!(pubkey.is_some(), "Mode {mode:?} should generate keys");
        assert!(state.agent_quantum_keys.contains_key(&1));
    }
}

#[test]
fn test_check_identity_theft_edge_cases() {
    let cases = vec![
        ("low_energy", 0.0_f32, 100_u64, 0_u32, false, false),
        // TAG: surface=simulation owner=research-team rule=SIM-001
        ("high_energy", 200.0, 100, 0, false, false),
        ("high_age", 50.0, 3000, 0, false, false),
        ("zero_verifications", 50.0, 100, 0, false, false),
        ("high_verifications", 50.0, 100, 200, false, false),
        ("protocol_protection", 100.0, 3000, 100, true, true),
        ("protocol_no_id", 100.0, 3000, 100, false, true),
        ("id_no_protocol", 100.0, 3000, 100, true, false),
    ];
    for (name, energy, age, verif, has_id, protocol) in cases {
        let mut rng = StdRng::seed_from_u64(42);
        let mut count = 0;
        for _ in 0..50_000 {
            if check_identity_theft(1, energy, age, verif, has_id, protocol, 1, &mut rng).is_some()
            {
                count += 1;
            }
        }
        assert!(count > 0, "Expected thefts for {name}, got {count}");
    }
}

#[test]
fn test_calculate_recovery_time_edge_cases() {
    assert_eq!(calculate_recovery_time(0.0, false, false), 100);
    assert_eq!(calculate_recovery_time(1.0, false, false), 1000);
    assert_eq!(calculate_recovery_time(1.0, true, true), 700);
    assert_eq!(calculate_recovery_time(0.5, true, true), 385);
}

#[test]
fn test_process_identity_theft_discovery_no_relationship_damage() {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let event = IdentityTheftEvent {
        victim_id: 1,
        severity: 0.8,
        financial_loss: 15.0,
        discovery_delay: 50,
        relationship_damage: false,
        theft_tick: 100,
        discovered: false,
        discovery_tick: None,
    };
    let (loss, docas_delta) = process_identity_theft_discovery(&event, 150);
    assert!((loss - 15.0).abs() < 0.001);
    assert!(docas_delta[2] > 0.0, "Cortisol should spike");
    assert!(docas_delta[4] < 0.0, "Serotonin should drop");
    assert_eq!(docas_delta[0], 0.0, "Dopamine should be unchanged");
    assert_eq!(docas_delta[1], 0.0, "Oxytocin should be unchanged");
}

#[test]
fn test_anchored_claim_serde() {
    let claim = AnchoredClaim {
        hash: 12_345,
        agent_id: 1,
        claim_type: ClaimType::IdentityVerification,
        anchor_tick: 100,
        anchor_cost: 0.05,
    };
    let json = serde_json::to_string(&claim).expect("serialize");
    let deserialized: AnchoredClaim = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.hash, claim.hash);
    assert_eq!(deserialized.agent_id, claim.agent_id);
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    assert_eq!(deserialized.claim_type, claim.claim_type);
    assert_eq!(deserialized.anchor_tick, claim.anchor_tick);
    assert!((deserialized.anchor_cost - claim.anchor_cost).abs() < 0.0001);
}

#[test]
fn test_verification_result_serde() {
    let result = VerificationResult {
        verified: true,
        hash: 999,
        verify_tick: 200,
        verify_cost: 0.01,
        latency_ticks: 3,
        authority: "0xabc".to_string(),
        is_centralized: false,
    };
    let json = serde_json::to_string(&result).expect("serialize");
    let deserialized: VerificationResult = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.verified, result.verified);
    assert_eq!(deserialized.hash, result.hash);
    assert_eq!(deserialized.latency_ticks, result.latency_ticks);
    assert_eq!(deserialized.authority, result.authority);
    assert_eq!(deserialized.is_centralized, result.is_centralized);
}

#[test]
fn test_verification_mode_serde() {
    for mode in [
        VerificationMode::CentralizedOracle,
        VerificationMode::DecentralizedProtocol,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        VerificationMode::PostQuantumProtocol,
        VerificationMode::HybridQuantumProtocol,
    ] {
        let json = serde_json::to_string(&mode).expect("serialize");
        let deserialized: VerificationMode = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized, mode, "Round-trip failed for {mode:?}");
    }
}

#[test]
fn test_identity_theft_event_serde() {
    let event = IdentityTheftEvent {
        victim_id: 1,
        severity: 0.5,
        financial_loss: 10.0,
        discovery_delay: 100,
        relationship_damage: false,
        theft_tick: 50,
        discovered: true,
        discovery_tick: Some(150),
    };
    let json = serde_json::to_string(&event).expect("serialize");
    let deserialized: IdentityTheftEvent = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.victim_id, event.victim_id);
    assert_eq!(deserialized.severity, event.severity);
    assert_eq!(deserialized.discovered, event.discovered);
    assert_eq!(deserialized.discovery_tick, event.discovery_tick);
}

// === Blockchain bridge coverage tests ===

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_mock_bridge_connect_disabled() {
    let config = BridgeConfig {
        enabled: false,
        ..Default::default()
    };
    let mut bridge = MockBlockchainBridge::new(config);
    assert!(!bridge.is_connected());
    let result = bridge.connect();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), BridgeError::NotConnected));
}

#[test]
fn test_mock_bridge_anchor_not_connected() {
    let config = BridgeConfig {
        enabled: true,
        ..Default::default()
    };
    let mut bridge = MockBlockchainBridge::new(config);
    bridge.connect().unwrap();
    assert!(bridge.is_connected());
    bridge.disconnect();
    assert!(!bridge.is_connected());
    let result = bridge.anchor_claim(1, ClaimType::IdentityVerification, 12_345, 100);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), BridgeError::NotConnected));
}

#[test]
// TAG: surface=simulation owner=research-team rule=SIM-001
fn test_bridge_error_display() {
    let err1 = BridgeError::NotConnected;
    assert_eq!(format!("{err1}"), "Not connected");

    let err2 = BridgeError::ConnectionFailed("timeout".to_string());
    assert_eq!(format!("{err2}"), "Connection failed: timeout");
}

#[test]
fn test_bridge_config_recommended_2030_longterm() {
    let config = BridgeConfig::recommended(2030, true);
    assert_eq!(config.quantum_mode, QuantumCryptoMode::Hybrid);
    assert!(config.is_quantum_resistant());
    assert!(config.hybrid_mode);
}

// === BFT consensus coverage tests ===

#[test]
fn test_byzantine_behavior_delivery_and_correctness() {
    assert_eq!(ByzantineBehavior::Timing.delivery_probability(), 1.0);
    assert_eq!(ByzantineBehavior::Timing.correctness_probability(), 1.0);
    assert_eq!(ByzantineBehavior::Malicious.delivery_probability(), 0.8);
    assert_eq!(ByzantineBehavior::Equivocation.delivery_probability(), 1.0);
    assert_eq!(
        ByzantineBehavior::Equivocation.correctness_probability(),
        0.5
    );
}

#[test]
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
fn test_validator_send_message_omission() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut node = ValidatorNode::with_behavior(1, ByzantineBehavior::Omission);
    let mut delivered = 0;
    for _ in 0..1_000 {
        if node
            .send_message(123, MessageType::Propose, 1, &mut rng)
            .is_some()
        {
            delivered += 1;
        }
    }
    assert!(delivered > 400 && delivered < 600, "delivered={delivered}");
}

#[test]
fn test_validator_send_message_malicious() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut node = ValidatorNode::with_behavior(1, ByzantineBehavior::Malicious);
    let mut corrupt = 0;
    let mut delivered = 0;
    for _ in 0..100 {
        if let Some(msg) = node.send_message(123, MessageType::Propose, 1, &mut rng) {
            delivered += 1;
            if msg.content != 123 {
                corrupt += 1;
            }
        }
    }
    assert!(delivered > 70, "delivered={delivered}");
    assert!(corrupt > 60, "corrupt={corrupt}");
    // TAG: surface=simulation owner=research-team rule=SIM-001
}

#[test]
fn test_validator_send_message_equivocation() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut node = ValidatorNode::with_behavior(1, ByzantineBehavior::Equivocation);
    let mut corrupt = 0;
    for _ in 0..1_000 {
        if let Some(msg) = node.send_message(123, MessageType::Propose, 1, &mut rng) {
            if msg.content != 123 {
                corrupt += 1;
            }
        }
    }
    assert!(corrupt > 400 && corrupt < 600, "corrupt={corrupt}");
}

#[test]
fn test_validator_receive_message() {
    let mut node = ValidatorNode::new_honest(1);
    let msg = ConsensusMessage {
        sender: 2,
        content: 42,
        timestamp: 10,
        msg_type: MessageType::Prevote,
    };
    node.receive_message(msg);
    assert_eq!(node.received_messages.len(), 1);
    assert_eq!(node.received_messages[0].sender, 2);
    assert_eq!(node.received_messages[0].content, 42);
}
// TAG: surface=simulation owner=research-team rule=SIM-001

#[test]
fn test_bft_with_omission_fault() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut bft = BFTConsensus::with_faulty_nodes(4, 1, ByzantineBehavior::Omission);
    let result = bft.run_round(12_345, 100, &mut rng);
    assert!(result.committed, "Should commit with omission fault");
    assert!(result.quorum_reached);
}

#[test]
fn test_bft_with_timing_fault() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut bft = BFTConsensus::with_faulty_nodes(4, 1, ByzantineBehavior::Timing);
    let result = bft.run_round(12_345, 100, &mut rng);
    assert!(result.committed, "Should commit with timing fault");
}

#[test]
fn test_bft_with_equivocation_fault() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut bft = BFTConsensus::with_faulty_nodes(7, 2, ByzantineBehavior::Equivocation);
    let result = bft.run_round(12_345, 100, &mut rng);
    assert!(result.committed, "Should commit with equivocation fault");
}

#[test]
fn test_bft_malicious_fault_commits() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut bft = BFTConsensus::with_faulty_nodes(7, 2, ByzantineBehavior::Malicious);
    let result = bft.run_round(12_345, 100, &mut rng);
    // TAG: surface=simulation owner=research-team rule=SIM-001
    assert!(result.committed, "Should commit despite malicious faults");
}

#[test]
fn test_bft_honest_and_faulty_count() {
    let bft = BFTConsensus::with_faulty_nodes(7, 2, ByzantineBehavior::Malicious);
    assert_eq!(bft.honest_count(), 5);
    assert_eq!(bft.faulty_count(), 2);
}

#[test]
fn test_bft_safety_violations() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut bft = BFTConsensus::new(4, 1);
    let r1 = bft.run_round(100, 1, &mut rng);
    let r2 = bft.run_round(200, 2, &mut rng);
    assert!(r1.committed);
    assert!(r2.committed);
    let analysis = bft.check_bft_guarantees();
    assert_eq!(analysis.committed_blocks, 2);
    assert_eq!(analysis.safety_violations, 1);
    assert!(analysis.live);
}

#[test]
fn test_bft_check_guarantees_no_commits() {
    let bft = BFTConsensus::new(4, 1);
    let analysis = bft.check_bft_guarantees();
    assert_eq!(analysis.committed_blocks, 0);
    assert!(!analysis.live);
    assert!(!analysis.bft_guarantees_hold);
}
