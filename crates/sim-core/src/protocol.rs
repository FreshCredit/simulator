//! Protocol module: Blockchain verification simulation
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION protocol, not `FreshCredit` production blockchain
//! SIM-004: Deterministic - same seed produces identical verification outcomes
// TAG: surface=simulation owner=research-team rule=SIM-001
//!
//! Per §22.1: Phase 1 MVP uses local SHA-256 hashes (no real blockchain)
//! Per §8.2.1 lines 4646-4666: Protocol Toggle Contract
//! Per §6.8.5: Fresh Protocol Intervention Analysis
//!
//! ## Key Concepts
//! - **Anchor**: Create a verification hash for a claim
//! - **Verify**: Check if a counterparty's claim has been anchored
//! - **Verification Rate**: Fraction of interactions where verification is used
//! - **Protocol Effect**: Behavioral delta between Protocol ON vs OFF

/// bft consensus module.
pub mod bft_consensus;
/// Blockchain bridge for real blockchain integration (P0-3)
pub mod blockchain_bridge;
pub use bft_consensus::*;

use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::quantum::{
    generate_quantum_keypair, QuantumCryptoMode, QuantumSignature, QuantumThreatModel,
};
use crate::types::{AgentId, Tick};

// Re-export blockchain bridge types
pub use blockchain_bridge::{BlockchainBridge, BridgeConfig, BridgeError, MockBlockchainBridge};

// === Identity Theft Constants (§6.23.1) ===
// Per BJS 2016: Annual prevalence 10% = ~0.00027 per tick at 36500 ticks/year (corrected from 7.3%)
/// Base probability of identity theft per tick
pub const IDENTITY_THEFT_BASE_RATE: f32 = 0.0002;
/// Median energy used for wealth factor calculation
pub const MEDIAN_ENERGY: f32 = 50.0;

/// Verification hash (SHA-256 simulation)
/// Per §22.1.1: Phase 1 uses local hashes with `sim_anchor_` prefix
pub type VerificationHash = u64;

// ClaimType is now unified in types.rs
// Re-export for backward compatibility
pub use crate::types::ClaimType;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Verification mode for trust anchoring and comparison experiments
/// Per SIM-011: Centralized Baseline Comparison Mode
///
/// This is the CORE research intervention for comparing:
/// - Centralized trust (traditional reputation systems)
/// - Decentralized trust (blockchain verification)
/// - Post-quantum trust (quantum-resistant cryptography)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VerificationMode {
    /// Centralized oracle (baseline)
    /// - Trust based on central authority reputation
    /// - No blockchain verification
    /// - Lower latency, lower cost
    /// - Single point of failure, censorship risk
    CentralizedOracle,

    /// Decentralized protocol (experimental)
    /// - Trust via blockchain verification
    /// - Anchored claims on Substrate
    /// - Higher latency, higher cost, censorship-resistant
    /// - Distributed consensus, no single point of failure
    #[default]
    DecentralizedProtocol,

    /// Post-quantum decentralized protocol
    /// - Uses CRYSTALS-Dilithium or SPHINCS+ signatures
    /// - Quantum-resistant anchoring
    /// - Future-proof against quantum attacks
    /// - Higher bandwidth due to larger signatures
    PostQuantumProtocol,

    /// Hybrid post-quantum protocol
    /// - Combines classical + post-quantum signatures
    /// - Defense in depth against unknown attacks
    /// - Gradual migration path to full PQC
    HybridQuantumProtocol,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Identity theft event structure
/// Per §6.23.2 lines 3101-3108
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityTheftEvent {
    /// Victim agent ID
    pub victim_id: AgentId,
    /// Severity of theft (0.1 minor to 1.0 catastrophic)
    pub severity: f32,
    /// Energy units lost
    pub financial_loss: f32,
    /// Ticks until victim discovers theft
    pub discovery_delay: u64,
    /// Whether social network is affected
    pub relationship_damage: bool,
    /// Tick when theft occurred
    pub theft_tick: Tick,
    /// Whether theft has been discovered
    pub discovered: bool,
    /// Tick when theft was discovered (if discovered)
    pub discovery_tick: Option<Tick>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// An anchored claim in the simulation
/// Per §22.1.1: Hash stored locally with timestamp + `sim_anchor`_ prefix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchoredClaim {
    /// Unique hash for this claim
    pub hash: VerificationHash,
    /// Agent who anchored the claim
    pub agent_id: AgentId,
    /// Type of claim
    pub claim_type: ClaimType,
    /// Tick when anchored
    pub anchor_tick: Tick,
    /// Cost paid for anchoring (simulated)
    /// Per §22.1.1: 0.01-0.1 SIM tokens
    pub anchor_cost: f32,
}

/// Verification request result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether the claim was found and valid
    pub verified: bool,
    /// The hash that was checked
    pub hash: VerificationHash,
    /// Tick when verification was performed
    pub verify_tick: Tick,
    /// Cost paid for verification (simulated)
    pub verify_cost: f32,
    /// Latency in ticks for this verification
    /// - `CentralizedOracle`: 0 (instant)
    /// - `DecentralizedProtocol`: 1-10 (blockchain latency)
    pub latency_ticks: u32,
    /// Authority that performed verification
    /// - `CentralizedOracle`: "`CentralOracle`"
    /// - `DecentralizedProtocol`: blockchain hash
    pub authority: String,
    /// Whether this was a centralized verification
    pub is_centralized: bool,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Protocol state for the simulation
/// Per §22.1: Tracks all anchored claims and verification history
/// Per SIM-011: Supports both `CentralizedOracle` and `DecentralizedProtocol` modes
/// Per PQC-001: Supports post-quantum cryptographic modes
#[derive(Debug, Clone, Default)]
pub struct ProtocolState {
    /// All anchored claims indexed by hash
    pub anchored_claims: HashMap<VerificationHash, AnchoredClaim>,
    /// Verification history (`agent_id` -> list of verification results)
    pub verification_history: HashMap<AgentId, Vec<VerificationResult>>,
    /// Total anchors created
    pub total_anchors: u64,
    /// Total verifications performed
    pub total_verifications: u64,
    /// Successful verifications
    pub successful_verifications: u64,
    /// Failed anchors (for rollback detection)
    pub failed_anchors: u64,
    /// Current verification mode
    /// Per SIM-011: Determines verification behavior
    pub verification_mode: VerificationMode,
    /// Pending verifications for decentralized mode (simulates blockchain latency)
    /// Stores (`completion_tick`, result) pairs
    pub pending_verifications: Vec<(Tick, VerificationResult)>,
    /// Quantum cryptography mode for post-quantum security
    /// Per PQC-001: Enables quantum-resistant signatures
    pub quantum_mode: QuantumCryptoMode,
    /// Quantum threat model for determining crypto requirements
    pub quantum_threat_model: Option<QuantumThreatModel>,
    /// Agent quantum public keys (`agent_id` -> public key)
    pub agent_quantum_keys: HashMap<AgentId, crate::quantum::QuantumPublicKey>,
    /// Quantum signatures for anchored claims (hash -> signature)
    pub quantum_signatures: HashMap<VerificationHash, QuantumSignature>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl ProtocolState {
    /// Create a new empty protocol state with default mode (`DecentralizedProtocol`)
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new protocol state with specific verification mode
    /// Per SIM-011: Enable comparison between `CentralizedOracle` and `DecentralizedProtocol`
    #[must_use]
    pub fn with_mode(mode: VerificationMode) -> Self {
        Self {
            verification_mode: mode,
            ..Self::default()
        }
    }

    /// Create a new protocol state with quantum cryptography
    /// Per PQC-001: Enable post-quantum security
    #[must_use]
    pub fn with_quantum_crypto(quantum_mode: QuantumCryptoMode) -> Self {
        Self {
            quantum_mode,
            ..Self::default()
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create a new protocol state with full post-quantum configuration
    #[must_use]
    pub fn with_quantum_threat_model(threat_model: QuantumThreatModel) -> Self {
        let recommended_mode = threat_model.recommended_mode();
        Self {
            verification_mode: if recommended_mode.is_quantum_resistant() {
                VerificationMode::PostQuantumProtocol
            } else {
                VerificationMode::DecentralizedProtocol
            },
            quantum_mode: recommended_mode,
            quantum_threat_model: Some(threat_model),
            ..Self::default()
        }
    }

    /// Set the verification mode
    pub fn set_mode(&mut self, mode: VerificationMode) {
        self.verification_mode = mode;
    }

    /// Set the quantum cryptography mode
    /// Per PQC-001: Enable post-quantum security upgrade
    pub fn set_quantum_mode(&mut self, mode: QuantumCryptoMode) {
        self.quantum_mode = mode;
    }

    /// Generate quantum-safe keys for an agent
    pub fn generate_agent_quantum_keys<R: Rng>(
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        &mut self,
        agent_id: AgentId,
        rng: &mut R,
    ) -> Option<crate::quantum::QuantumPublicKey> {
        let (pubkey, _privkey) = generate_quantum_keypair(self.quantum_mode, rng);
        self.agent_quantum_keys.insert(agent_id, pubkey.clone());
        Some(pubkey)
    }

    /// Check if quantum mode is active
    #[must_use]
    pub const fn is_quantum_protected(&self) -> bool {
        self.quantum_mode.is_quantum_resistant()
    }

    /// Get quantum security level in bits
    #[must_use]
    pub const fn quantum_security_bits(&self) -> u32 {
        match self.quantum_mode {
            QuantumCryptoMode::Classical => 128, // Vulnerable to quantum attacks
            QuantumCryptoMode::Dilithium => 128, // NIST Level 2
            QuantumCryptoMode::SphincsPlus => 128, // NIST Level 1
            QuantumCryptoMode::Hybrid => 128,    // Conservative estimate
        }
    }

    /// Estimate quantum security margin (years until vulnerable)
    #[must_use]
    pub const fn quantum_security_margin(&self) -> u32 {
        match self.quantum_mode {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            QuantumCryptoMode::Classical => 0, // Already vulnerable
            QuantumCryptoMode::Dilithium
            | QuantumCryptoMode::SphincsPlus
            | QuantumCryptoMode::Hybrid => {
                // Post-quantum algorithms are expected to remain secure
                // even after quantum computers become available
                50 // Conservative estimate
            }
        }
    }

    /// Anchor a claim for an agent
    /// Per §22.1.1: Creates SHA-256 hash (simulated as u64)
    /// Per SIM-011: Mode-aware anchoring with different cost/latency profiles
    /// Per PQC-001: Supports quantum-resistant signatures
    /// Returns the hash and cost
    /// Mode-aware anchor cost
    fn anchor_cost(&self, rng: &mut StdRng) -> f32 {
        match self.verification_mode {
            VerificationMode::CentralizedOracle => 0.0, // No cost in centralized mode
            VerificationMode::DecentralizedProtocol => 0.01 + rng.gen::<f32>() * 0.09,
            VerificationMode::PostQuantumProtocol => 0.02 + rng.gen::<f32>() * 0.13,
            VerificationMode::HybridQuantumProtocol => 0.03 + rng.gen::<f32>() * 0.17,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// anchor claim.
    pub fn anchor_claim(
        &mut self,
        agent_id: AgentId,
        claim_type: ClaimType,
        tick: Tick,
        rng: &mut StdRng,
    ) -> (VerificationHash, f32) {
        let hash = Self::generate_hash(agent_id, claim_type, tick, rng);
        let anchor_cost = self.anchor_cost(rng);

        let claim = AnchoredClaim {
            hash,
            agent_id,
            claim_type,
            anchor_tick: tick,
            anchor_cost,
        };

        self.anchored_claims.insert(hash, claim);

        // Generate quantum signature if quantum mode is enabled
        if self.is_quantum_protected() {
            let quantum_sig = self.create_quantum_signature(hash, agent_id, rng);
            self.quantum_signatures.insert(hash, quantum_sig);
        }

        self.total_anchors += 1;

        (hash, anchor_cost)
    }

    /// Create a quantum signature for an anchored claim
    fn create_quantum_signature<R: Rng>(
        &self,
        _hash: VerificationHash,
        _agent_id: AgentId,
        rng: &mut R,
    ) -> QuantumSignature {
        // P2-PERF: Removed unused format! allocation - message was never used
        let _message: Vec<u8> = Vec::new(); // Placeholder for potential future use

        match self.quantum_mode {
            QuantumCryptoMode::Classical => {
                // Classical SHA-256 hash (simulated)
                let mut hash_bytes = [0u8; 32];
                rng.fill(&mut hash_bytes[..]);
                QuantumSignature::Classical(hash_bytes)
            }
            QuantumCryptoMode::Dilithium => {
                // TAG: surface=simulation owner=research-team rule=SIM-001
                use crate::quantum::DilithiumSignature;
                QuantumSignature::Dilithium(DilithiumSignature::new(rng, 2))
            }
            QuantumCryptoMode::SphincsPlus => {
                use crate::quantum::SphincsSignature;
                use crate::quantum::SphincsVariant;
                QuantumSignature::SphincsPlus(SphincsSignature::new(rng, SphincsVariant::S128s))
            }
            QuantumCryptoMode::Hybrid => {
                use crate::quantum::DilithiumSignature;
                let mut classical_hash = [0u8; 32];
                rng.fill(&mut classical_hash[..]);
                QuantumSignature::Hybrid {
                    classical: classical_hash,
                    dilithium: DilithiumSignature::new(rng, 2),
                }
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Mode-aware verification that dispatches to appropriate method
    /// Per SIM-011: Core comparison mechanism
    /// Per PQC-001: Supports post-quantum verification modes
    pub fn verify_claim(
        &mut self,
        verifier_id: AgentId,
        hash: VerificationHash,
        tick: Tick,
        agent_reputation: Option<f32>, // Required for centralized mode
        rng: &mut StdRng,
    ) -> VerificationResult {
        match self.verification_mode {
            VerificationMode::CentralizedOracle => {
                self.verify_centralized(verifier_id, hash, tick, agent_reputation.unwrap_or(0.5))
            }
            VerificationMode::DecentralizedProtocol => {
                self.verify_decentralized(verifier_id, hash, tick, rng)
            }
            VerificationMode::PostQuantumProtocol => {
                self.verify_post_quantum(verifier_id, hash, tick, rng)
            }
            VerificationMode::HybridQuantumProtocol => {
                self.verify_hybrid_quantum(verifier_id, hash, tick, rng)
            }
        }
    }

    /// Verify claim using centralized oracle
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Per SIM-011: Centralized baseline with instant verification
    ///
    /// Characteristics:
    /// - No blockchain latency (instant)
    /// - No token cost
    /// - Trust threshold based on reputation
    /// - Single authority vulnerability
    fn verify_centralized(
        &mut self,
        verifier_id: AgentId,
        hash: VerificationHash,
        tick: Tick,
        agent_reputation: f32,
    ) -> VerificationResult {
        let trust_threshold = 0.3;
        let verified =
            agent_reputation >= trust_threshold && self.anchored_claims.contains_key(&hash);

        let result = VerificationResult {
            verified,
            hash,
            verify_tick: tick,
            verify_cost: 0.0,
            latency_ticks: 0, // Instant verification
            authority: "CentralOracle".to_string(),
            is_centralized: true,
        };

        self.record_verification(verifier_id, result)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Verify claim using decentralized protocol
    /// Per SIM-011: Blockchain verification with latency and cost
    ///
    /// Characteristics:
    /// - Blockchain latency (1-10 ticks for simulation)
    /// - Token cost for verification
    /// - Cryptographic proof
    /// - Censorship resistant
    fn verify_decentralized(
        &mut self,
        verifier_id: AgentId,
        hash: VerificationHash,
        tick: Tick,
        rng: &mut StdRng,
    ) -> VerificationResult {
        let verified = self.anchored_claims.contains_key(&hash);

        let result = VerificationResult {
            verified,
            hash,
            verify_tick: tick,
            verify_cost: 0.005 + rng.gen::<f32>() * 0.045,
            latency_ticks: rng.gen_range(1..=3),
            authority: format!("0x{hash:016x}"), // Mock blockchain address
            is_centralized: false,
        };

        self.record_verification(verifier_id, result)
    }

    /// Verify quantum signature for a given hash
    fn verify_quantum_signature(&self, hash: VerificationHash, verifier_id: AgentId) -> bool {
        if let Some(quantum_sig) = self.quantum_signatures.get(&hash) {
            // P2-PERF: Use stack buffer with write! instead of format! to avoid heap allocation
            let mut buf = [0u8; 40]; // Max u64 + ":" + max u64 = ~39 bytes
                                     // TAG: surface=simulation owner=research-team rule=SIM-001
            let msg_len = {
                use std::io::Write;
                let mut cursor = std::io::Cursor::new(&mut buf[..]);
                let _ = write!(cursor, "{verifier_id}:{hash}");
                cursor.position() as usize
            };
            quantum_sig.verify(&buf[..msg_len])
        } else {
            false
        }
    }

    /// Verify claim using post-quantum protocol
    /// Per PQC-001: Quantum-resistant verification
    ///
    /// Characteristics:
    /// - Larger signatures (2-8 KB vs 64 bytes)
    /// - Higher bandwidth requirements
    /// - Quantum-resistant security
    /// - Higher verification cost
    fn verify_post_quantum(
        &mut self,
        verifier_id: AgentId,
        hash: VerificationHash,
        tick: Tick,
        rng: &mut StdRng,
    ) -> VerificationResult {
        let hash_verified = self.anchored_claims.contains_key(&hash);
        let quantum_verified = self.verify_quantum_signature(hash, verifier_id);
        let verified = hash_verified && quantum_verified;

        let result = VerificationResult {
            verified,
            hash,
            verify_tick: tick,
            verify_cost: 0.01 + rng.gen::<f32>() * 0.04,
            latency_ticks: rng.gen_range(2..=5),
            authority: format!("PQC-0x{hash:016x}"), // Post-quantum identifier
            is_centralized: false,
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        self.record_verification(verifier_id, result)
    }

    /// Verify claim using hybrid quantum protocol
    /// Per PQC-001: Classical + Post-quantum combined verification
    fn verify_hybrid_quantum(
        &mut self,
        verifier_id: AgentId,
        hash: VerificationHash,
        tick: Tick,
        rng: &mut StdRng,
    ) -> VerificationResult {
        // Run both classical and post-quantum verification
        let classical_result = self.verify_decentralized(verifier_id, hash, tick, rng);
        let pq_result = self.verify_post_quantum(verifier_id, hash, tick, rng);

        // Combined result: both must pass for hybrid security
        let verified = classical_result.verified && pq_result.verified;

        VerificationResult {
            verified,
            hash,
            verify_tick: tick,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            verify_cost: classical_result.verify_cost + pq_result.verify_cost,
            latency_ticks: classical_result.latency_ticks + pq_result.latency_ticks,
            authority: format!("HYBRID-0x{hash:016x}"),
            is_centralized: false,
        }
    }

    /// Generate a deterministic hash for a claim
    /// Per §22.1.1: Simulates SHA-256 with `sim_anchor`_ prefix
    fn generate_hash(
        agent_id: AgentId,
        claim_type: ClaimType,
        tick: Tick,
        rng: &mut StdRng,
    ) -> VerificationHash {
        // Combine inputs into a hash-like value
        // This is deterministic given the same RNG state
        let type_val = claim_type as u64;
        let random_component: u64 = rng.gen();

        // Simple hash combination (not cryptographic, but deterministic)
        agent_id
            .wrapping_mul(31)
            .wrapping_add(type_val.wrapping_mul(17))
            .wrapping_add(tick.wrapping_mul(13))
            .wrapping_add(random_component)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get agent's verification hash if they have anchored an identity claim
    #[must_use]
    pub fn get_agent_identity_hash(&self, agent_id: AgentId) -> Option<VerificationHash> {
        self.anchored_claims
            .values()
            .find(|c| c.agent_id == agent_id && c.claim_type == ClaimType::IdentityVerification)
            .map(|c| c.hash)
    }

    /// Calculate verification rate (successful / total)
    /// Per §D.3 line 7587: Stability condition > 0.1
    #[must_use]
    pub fn verification_rate(&self) -> f32 {
        if self.total_verifications == 0 {
            0.0
        } else {
            self.successful_verifications as f32 / self.total_verifications as f32
        }
    }

    /// Calculate anchor failure rate
    /// Per §D.3 line 7587: Rollback trigger > 50%
    #[must_use]
    pub fn anchor_failure_rate(&self) -> f32 {
        if self.total_anchors == 0 {
            0.0
        } else {
            self.failed_anchors as f32 / self.total_anchors as f32
        }
    }

    /// Check if rollback should be triggered
    /// Per §D.3 line 7587: Anchor failures > 50%
    #[must_use]
    pub fn should_rollback(&self) -> bool {
        self.anchor_failure_rate() > 0.5
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if protocol is stable
    /// Per §D.3 line 7587: Verification rate > 0.1
    #[must_use]
    pub fn is_stable(&self) -> bool {
        self.verification_rate() > 0.1
    }

    /// Record an anchor failure
    pub fn record_anchor_failure(&mut self) {
        self.failed_anchors += 1;
    }

    /// Record a verification result and update counters
    fn record_verification(
        &mut self,
        verifier_id: AgentId,
        result: VerificationResult,
    ) -> VerificationResult {
        let verified = result.verified;
        self.verification_history
            .entry(verifier_id)
            .or_default()
            .push(result.clone());
        self.total_verifications += 1;
        if verified {
            self.successful_verifications += 1;
        }
        result
    }
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
/// Check if an agent becomes an identity theft victim
/// Per §6.23.3 lines 3119-3147
///
/// Risk factors (research-backed from BJS 2016):
/// - Wealth: Wealthier agents are more attractive targets
/// - Exposure: More verifications = more exposure
/// - Age: Older agents have higher risk
/// - Protocol protection: Verified identities are harder to steal
///
/// Compute the probability of identity theft for an agent
#[allow(clippy::too_many_arguments)] // P0-1: Identity theft requires multiple risk factors
fn compute_theft_probability(
    energy: f32,
    age: u64,
    verification_count: u32,
    protocol_enabled: bool,
    has_verified_id: bool,
) -> f32 {
    let base_prob = IDENTITY_THEFT_BASE_RATE;

    // Risk factors (research-backed from BJS 2016)
    let wealth_factor = (energy / MEDIAN_ENERGY).min(2.0); // Wealthier = target
    let exposure_factor = (verification_count as f32 / 100.0).min(1.0); // Activity = exposure
    let age_factor = if age > 2000 { 1.2 } else { 1.0 }; // Older = higher risk

    // Protocol protection effect (§6.23.5)
    let protocol_protection = if protocol_enabled && has_verified_id {
        0.7 // 30% reduction
    } else {
        1.0
    };

    base_prob
        * (1.0 + 0.3 * wealth_factor + 0.2 * exposure_factor)
        * age_factor
        * protocol_protection
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[allow(clippy::too_many_arguments)] // P0-1: Identity theft requires multiple risk factors
/// check identity theft.
pub fn check_identity_theft(
    agent_id: AgentId,
    energy: f32,
    age: u64,
    verification_count: u32,
    has_verified_id: bool,
    protocol_enabled: bool,
    current_tick: Tick,
    rng: &mut StdRng,
) -> Option<IdentityTheftEvent> {
    let theft_prob = compute_theft_probability(
        energy,
        age,
        verification_count,
        protocol_enabled,
        has_verified_id,
    );

    if rng.gen::<f32>() < theft_prob {
        let severity = rng.gen_range(0.1..1.0);
        let financial_loss = severity * energy * 0.2; // Up to 20% of energy

        Some(IdentityTheftEvent {
            victim_id: agent_id,
            severity,
            financial_loss,
            discovery_delay: rng.gen_range(10..500), // 10-500 ticks to discover
            relationship_damage: rng.gen::<f32>() < 0.15 * severity, // 15% × severity chance
            theft_tick: current_tick,
            discovered: false,
            discovery_tick: None,
        })
    } else {
        None
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Process identity theft discovery and apply effects
/// Per §6.23.4 lines 3152-3195
///
/// Effects:
/// 1. Financial impact (energy loss)
/// 2. DOCAS impact (cortisol spike, serotonin drop)
/// 3. Returns updated DOCAS deltas for the victim
///
/// Compute DOCAS delta for an identity theft event
#[must_use]
fn compute_identity_theft_docas(event: &IdentityTheftEvent) -> [f32; 5] {
    let cortisol_spike = 0.3 + event.severity * 0.3;
    let serotonin_drop = -0.1 - event.severity * 0.2;
    let adrenaline_spike = 0.1 + event.severity * 0.1;

    let (oxytocin_drop, dopamine_drop) = if event.relationship_damage {
        (-0.2 * event.severity, -0.1 * event.severity)
    } else {
        (0.0, 0.0)
    };

    [
        dopamine_drop,    // D: slight drop from loss
        oxytocin_drop,    // O: major drop if relationships damaged
        cortisol_spike,   // C: stress spike
        adrenaline_spike, // A: immediate stress response
        serotonin_drop,   // S: wellbeing drop
    ]
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[must_use]
/// process identity theft discovery.
pub fn process_identity_theft_discovery(
    event: &IdentityTheftEvent,
    current_tick: Tick,
) -> (f32, [f32; 5]) {
    let _ = current_tick; // Used for logging/metrics in future
    let docas_delta = compute_identity_theft_docas(event);
    (event.financial_loss, docas_delta)
}

/// Calculate recovery time for identity theft
/// Per §6.23.5: Protocol ON = 70% of baseline recovery time
#[must_use]
pub fn calculate_recovery_time(
    severity: f32,
    protocol_enabled: bool,
    has_verified_id: bool,
) -> u64 {
    // Base recovery: 100-1000 ticks based on severity
    // Per §6.23.1: 3-12 months empirical = 1095-4380 ticks at 365 ticks/month
    // Scaled down for simulation: 100-1000 ticks
    let base_recovery = (100.0 + severity * 900.0) as u64;

    // Protocol protection speeds recovery (§6.23.5)
    if protocol_enabled && has_verified_id {
        (base_recovery as f32 * 0.7) as u64 // 30% faster recovery
    } else {
        base_recovery
    }
}

#[cfg(test)]
mod tests;
