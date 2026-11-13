//! Identity Theft Simulation Module (SIM-010)
//!
//! Implements identity theft mechanics with:
//! - 5-10% annual prevalence rate (BJS 2016)
//! - Protocol protection reducing theft by 20-40%
//! - Recovery mechanics (100-500 ticks to 50% restoration)
//! - Four theft methods: `SocialEngineering`, `CredentialTheft`, Impersonation, `AccountTakeover`
//!
//! Reference: simulation-plan.md §6.22, §6.23
//! POM Targets: ID-THEFT-001 through ID-THEFT-004

use crate::protocol::{AnchoredClaim, ProtocolState, VerificationResult};
use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Simulation tick type re-export for clarity
pub type Tick = crate::types::Tick;
/// agent id.
pub type AgentId = crate::types::AgentId;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Identity theft event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityTheft {
    /// Victim agent ID
    pub victim: AgentId,
    /// Perpetrator agent ID
    pub perpetrator: AgentId,
    /// Tick when theft occurred
    pub tick: Tick,
    /// Method used for theft
    pub method: TheftMethod,
    /// Severity of theft (0.0 - 1.0)
    pub severity: f32,
    /// Financial loss incurred
    pub financial_loss: f32,
    /// Recovery duration in ticks
    pub recovery_duration: Tick,
}

/// Methods of identity theft
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TheftMethod {
    /// Exploit trust to manipulate victim
    SocialEngineering,
    /// Steal keys/credentials directly
    CredentialTheft,
    /// Create fake identity to deceive
    Impersonation,
    /// Direct control of victim's account
    AccountTakeover,
}

impl TheftMethod {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Get all theft methods
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::SocialEngineering,
            Self::CredentialTheft,
            Self::Impersonation,
            Self::AccountTakeover,
        ]
    }

    /// Get base success probability for this method
    #[must_use]
    pub const fn base_success_rate(&self) -> f32 {
        match self {
            Self::SocialEngineering => 0.02, // 2% base - requires trust
            Self::CredentialTheft => 0.01,   // 1% base - technical difficulty
            Self::Impersonation => 0.015,    // 1.5% base - moderate difficulty
            Self::AccountTakeover => 0.005,  // 0.5% base - hardest
        }
    }

    /// Get string representation
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::SocialEngineering => "social_engineering",
            Self::CredentialTheft => "credential_theft",
            Self::Impersonation => "impersonation",
            Self::AccountTakeover => "account_takeover",
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Identity claim for blockchain anchoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityClaim {
    /// Agent ID
    pub agent_id: AgentId,
    /// Public key (simulated hash)
    pub public_key: String,
    /// Tick when claim was created
    pub timestamp: Tick,
}

/// Agent identity state with theft/compromise/recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    /// Unique agent identifier (hash of public key)
    pub hash: String,
    /// Agent ID this identity belongs to
    pub agent_id: AgentId,
    /// Public key for verification
    pub public_key: String,
    /// Whether identity is currently compromised
    pub is_compromised: bool,
    /// Tick when compromise occurred
    pub compromise_tick: Option<Tick>,
    /// Tick when recovery will complete
    pub recovery_tick: Option<Tick>,
    /// Recovery duration in ticks (100-500)
    pub recovery_duration: Tick,
    /// History of theft events
    pub theft_history: Vec<IdentityTheft>,
    /// Blockchain-anchored claims for protection
    pub anchored_claims: Vec<AnchoredClaim>,
    /// Verifiable claims about the agent
    pub claims: Vec<Claim>,
    /// Whether identity is currently stolen/active theft
    pub is_stolen: bool,
    /// Timestamp (tick) when theft occurred
    pub theft_tick: Option<Tick>,
    /// Recovery progress (0.0 = just stolen, 1.0 = fully recovered)
    pub recovery_progress: f32,
    /// Number of theft attempts faced
    pub theft_attempts: u32,
    /// Number of successful thefts
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub successful_thefts: u32,
    /// Blockchain verification count (for Protocol ON)
    pub verification_count: u32,
}

/// A verifiable claim about an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    /// Type of claim
    pub claim_type: ClaimType,
    /// Claim value
    pub value: f32,
    /// Tick when claim was made
    pub tick_created: Tick,
    /// Whether claim is verified (blockchain anchored)
    pub is_verified: bool,
    /// Blockchain hash if verified
    pub blockchain_hash: Option<String>,
}

// ClaimType is now unified in types.rs
// Re-export for backward compatibility
pub use crate::types::ClaimType;

/// Result of an identity theft attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TheftResult {
    /// Theft succeeded
    Success {
        /// What was stolen
        stolen_claims: Vec<ClaimType>,
        /// Damage to victim's reputation
        reputation_damage: f32,
        /// Method used
        method: TheftMethod,
    },
    /// Theft failed (caught or prevented)
    Failed {
        /// Reason for failure
        reason: TheftFailureReason,
        /// Reputation damage to would-be thief
        thief_penalty: f32,
    },
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Reasons why theft might fail
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TheftFailureReason {
    /// Protocol verification caught the attempt
    ProtocolVerification,
    /// Victim had strong security (high reputation)
    StrongSecurity,
    /// Random chance (failed attack)
    RandomFailure,
    /// Already compromised
    AlreadyCompromised,
    /// Victim had anchored identity protection
    AnchoredProtection,
}

/// Identity theft statistics for a simulation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IdentityTheftStats {
    /// Total theft attempts
    pub attempts: u32,
    /// Successful thefts
    pub successful: u32,
    /// Failed thefts
    pub failed: u32,
    /// Thefts prevented by Protocol
    pub prevented_by_protocol: u32,
    /// Average recovery time (ticks)
    pub avg_recovery_time: f32,
    /// Total reputation damage from theft
    pub total_reputation_damage: f32,
    /// Thefts by method
    pub thefts_by_method: HashMap<String, u32>,
    /// Current theft rate (% of agents affected)
    pub current_theft_rate: f32,
    /// Protocol protection effectiveness (0.0 - 1.0)
    pub protocol_protection_rate: f32,
}

pub mod credit_report;
pub mod metrics;
pub mod theft_engine;

pub use credit_report::*;
pub use metrics::*;
pub use theft_engine::*;

/// Types of identity theft mechanisms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TheftMechanism {
    /// Network breach propagation (20% of thefts)
    /// Spreads through engagement network edges
    NetworkPropagation,

    /// Opportunistic theft (60% of thefts)
    /// Risk-weighted random selection
    Opportunistic,

    /// Mega-breach residual (20% of thefts)
    /// Large-scale centralized breach
    MegaBreach,
}

impl TheftMechanism {
    /// Get base probability weight for this mechanism
    #[must_use]
    pub const fn weight(&self) -> f32 {
        match self {
            Self::NetworkPropagation => 0.20,
            Self::Opportunistic => 0.60,
            Self::MegaBreach => 0.20,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Network breach event for cascade propagation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkBreachEvent {
    /// Tick when breach occurred
    pub tick: u64,
    /// Origin agent of the breach
    pub origin_agent: AgentId,
    /// Agents affected by this breach
    pub affected_agents: Vec<AgentId>,
    /// Propagation depth (hops from origin)
    pub propagation_depth: u32,
    /// Whether this breach has been processed
    pub processed: bool,
}

impl IdentityTheftStats {
    /// Create new stats
    #[must_use]
    pub fn new() -> Self {
        Self {
            thefts_by_method: HashMap::new(),
            ..Default::default()
        }
    }

    /// Record a successful theft
    pub fn record_success(&mut self, method: TheftMethod, recovery_time: f32) {
        self.attempts += 1;
        self.successful += 1;
        *self
            .thefts_by_method
            .entry(method.as_str().to_string())
            .or_insert(0) += 1;

        // Update average recovery time
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let total_successful = self.successful as f32;
        self.avg_recovery_time =
            (self.avg_recovery_time * (total_successful - 1.0) + recovery_time) / total_successful;
    }

    /// Record a failed theft
    pub fn record_failure(&mut self, reason: &TheftFailureReason) {
        self.attempts += 1;
        self.failed += 1;
        if matches!(
            reason,
            TheftFailureReason::ProtocolVerification | TheftFailureReason::AnchoredProtection
        ) {
            self.prevented_by_protocol += 1;
        }
    }

    /// Compute theft rate for a population
    #[must_use]
    pub fn compute_theft_rate(&self, population: usize) -> f32 {
        let total = self.attempts.max(self.successful + self.failed);
        if population == 0 || total == 0 {
            return 0.0;
        }
        self.successful as f32 / population as f32
    }

    /// Compute protocol protection effectiveness
    #[must_use]
    pub fn compute_protection_effectiveness(&self) -> f32 {
        let total = self.attempts.max(self.successful + self.failed);
        if self.failed == 0 || total == 0 {
            return 0.0;
        }
        self.prevented_by_protocol as f32 / self.failed as f32
    }

    /// Update current theft rate
    pub fn update_current_rate(&mut self, affected_agents: usize, total_population: usize) {
        if total_population == 0 || self.attempts == 0 {
            self.current_theft_rate = 0.0;
        } else {
            self.current_theft_rate = affected_agents as f32 / total_population as f32;
        }
    }
}

impl Identity {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create a new identity for an agent
    #[must_use]
    pub fn new(agent_id: AgentId, seed: u64) -> Self {
        // Generate deterministic hash from agent_id and seed
        let hash_input = format!("agent_{agent_id}_{seed}");
        // P0-SEC-011: Using mock_hash for deterministic simulation identifiers (not cryptographic)
        let hash = format!("0x{}", mock_hash::compute(&hash_input))[2..].to_string();
        let public_key = format!("pk_{}_{}", agent_id, &hash[..16]);

        Self {
            hash,
            agent_id,
            public_key,
            is_compromised: false,
            compromise_tick: None,
            recovery_tick: None,
            recovery_duration: 0,
            theft_history: Vec::new(),
            anchored_claims: Vec::new(),
            claims: Vec::new(),
            is_stolen: false,
            theft_tick: None,
            recovery_progress: 1.0,
            theft_attempts: 0,
            successful_thefts: 0,
            verification_count: 0,
        }
    }

    /// Add a claim to the identity
    pub fn add_claim(&mut self, claim_type: ClaimType, value: f32, tick: Tick) {
        if self.is_compromised {
            return;
        }
        self.claims.push(Claim {
            claim_type,
            value,
            tick_created: tick,
            is_verified: false,
            blockchain_hash: None,
        });
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Verify a claim (anchor to blockchain)
    pub fn verify_claim(&mut self, claim_type: ClaimType, tick: Tick, protocol_enabled: bool) {
        if !protocol_enabled || self.is_compromised {
            return;
        }

        if let Some(claim) = self
            .claims
            .iter_mut()
            .find(|c| c.claim_type == claim_type && !c.is_verified)
        {
            claim.is_verified = true;
            // Generate mock blockchain hash
            let hash_input = format!("{}:{}:{}", self.hash, claim_type as u8, tick);
            // SEC-008: EXCEPTION - MD5 used only for mock blockchain hashes in simulation, not security
            claim.blockchain_hash = Some(format!("0x{}", mock_hash::compute(&hash_input)));
            self.verification_count += 1;
        }
    }

    /// Get verification rate (for merit calculation)
    #[must_use]
    pub fn verification_rate(&self) -> f32 {
        if self.claims.is_empty() {
            return 0.0;
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let verified = self.claims.iter().filter(|c| c.is_verified).count();
        verified as f32 / self.claims.len() as f32
    }

    /// Check if identity is vulnerable to theft
    #[must_use]
    pub fn is_vulnerable(&self) -> bool {
        !self.is_stolen && self.recovery_progress >= 1.0 && !self.is_compromised
    }

    /// Mark identity as stolen/compromised
    pub fn mark_stolen(&mut self, tick: Tick) {
        self.is_stolen = true;
        self.is_compromised = true;
        self.theft_tick = Some(tick);
        self.compromise_tick = Some(tick);
        self.recovery_progress = 0.0;
        self.successful_thefts += 1;

        // Set recovery duration: 100-500 ticks
        // Use hash for deterministic randomness
        let hash_value = u64::from_str_radix(&self.hash[..16], 16).unwrap_or(tick);
        self.recovery_duration = 100 + (hash_value % 401) as Tick; // 100-500
        self.recovery_tick = Some(tick + self.recovery_duration);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update recovery progress
    pub fn update_recovery(&mut self, current_tick: Tick) {
        if !self.is_stolen && !self.is_compromised {
            return;
        }

        if let Some(theft_tick) = self.theft_tick {
            let ticks_since_theft = current_tick - theft_tick;
            // Recovery takes 100-500 ticks to reach 50%
            // Full recovery at 500-1000 ticks
            let recovery_rate = 0.002; // ~500 ticks for full recovery
            self.recovery_progress = (ticks_since_theft as f32 * recovery_rate).min(1.0);

            if self.recovery_progress >= 0.5 {
                self.is_stolen = false; // Can be targeted again after partial recovery
            }

            // Full recovery
            if self.recovery_progress >= 1.0 {
                self.is_compromised = false;
                self.recovery_tick = None;
            }
        }
    }

    /// Calculate vulnerability to theft
    /// Factors:
    /// - Trust level (higher trust = more vulnerable to social engineering)
    /// - Protocol usage (anchored = less vulnerable)
    /// - Reputation (higher rep = bigger target)
    #[must_use]
    pub fn theft_vulnerability(&self, trust: f32, reputation: f32) -> f32 {
        let base_vulnerability = 0.1;
        let trust_factor = trust * 0.3; // High trust = vulnerable to social engineering
        let reputation_factor = reputation * 0.2; // High rep = bigger target

        let protection_factor = if self.anchored_claims.is_empty() {
            0.0 // No protection
        } else {
            -0.2 // Blockchain protection reduces vulnerability by 20%
        };

        (base_vulnerability + trust_factor + reputation_factor + protection_factor).clamp(0.01, 0.9)
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Attempt identity theft
    pub fn attempt_theft(
        &mut self,
        perpetrator: &crate::agent::Agent,
        method: TheftMethod,
        current_tick: Tick,
        rng: &mut impl Rng,
    ) -> bool {
        let vulnerability =
            self.theft_vulnerability(perpetrator.compute_trust(), perpetrator.reputation_score);

        let method_probability = method.base_success_rate();
        let combined_probability = (vulnerability * method_probability * 10.0).clamp(0.001, 0.5);

        let success = rng.gen::<f32>() < combined_probability;

        if success {
            self.is_compromised = true;
            self.compromise_tick = Some(current_tick);
            self.is_stolen = true;
            self.theft_tick = Some(current_tick);
            self.successful_thefts += 1;

            // Set recovery duration: 100-500 ticks
            self.recovery_duration = rng.gen_range(100..=500);
            self.recovery_tick = Some(current_tick + self.recovery_duration);
            self.recovery_progress = 0.0;

            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            // Record theft event
            let theft = IdentityTheft {
                victim: self.agent_id,
                perpetrator: perpetrator.id,
                tick: current_tick,
                method,
                severity: vulnerability,
                financial_loss: vulnerability * 50.0,
                recovery_duration: self.recovery_duration,
            };
            self.theft_history.push(theft);
        }

        success
    }

    /// Recover from theft
    pub fn recover(&mut self, _current_tick: Tick) {
        if self.is_compromised {
            self.is_compromised = false;
            self.is_stolen = false;
            self.recovery_progress = 1.0;
            self.recovery_tick = None;

            // If anchored, reputation restoration is faster
            if !self.anchored_claims.is_empty() {
                // Reputation will be restored by caller
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Anchor identity claim to blockchain
    pub fn anchor_identity(
        &mut self,
        protocol: &mut ProtocolState,
        tick: Tick,
        rng: &mut StdRng,
    ) -> Option<crate::protocol::VerificationHash> {
        let _claim = IdentityClaim {
            agent_id: self.agent_id,
            public_key: self.public_key.clone(),
            timestamp: tick,
        };

        let (hash, _cost) = protocol.anchor_claim(
            self.agent_id,
            crate::protocol::ClaimType::IdentityVerification,
            tick,
            rng,
        );

        // Create anchored claim record
        let anchored = AnchoredClaim {
            hash,
            agent_id: self.agent_id,
            claim_type: crate::protocol::ClaimType::IdentityVerification,
            anchor_tick: tick,
            anchor_cost: 0.05,
        };

        self.anchored_claims.push(anchored);
        self.verification_count += 1;

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        Some(hash)
    }

    /// Verify identity against blockchain
    #[must_use]
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn verify_identity(
        &self,
        protocol: &ProtocolState,
        current_tick: Tick,
    ) -> VerificationResult {
        if self.anchored_claims.is_empty() {
            return VerificationResult {
                verified: false,
                hash: 0,
                verify_tick: current_tick,
                verify_cost: 0.0,
                latency_ticks: 0,
                authority: "none".to_string(),
                is_centralized: false,
            };
        }

        // Verify most recent anchor exists in protocol
        let Some(latest) = self.anchored_claims.last() else {
            return VerificationResult {
                verified: false,
                hash: 0,
                verify_tick: current_tick,
                verify_cost: 0.0,
                latency_ticks: 0,
                authority: "unknown".to_string(),
                is_centralized: false,
            };
        };
        let verified = protocol.anchored_claims.contains_key(&latest.hash);

        VerificationResult {
            verified,
            hash: latest.hash,
            verify_tick: current_tick,
            verify_cost: 0.0,
            latency_ticks: 0,
            authority: if verified {
                format!("0x{:016x}", latest.hash)
            } else {
                "none".to_string()
            },
            is_centralized: false,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if currently recovering
    #[must_use]
    pub fn is_recovering(&self) -> bool {
        self.is_compromised && self.recovery_progress < 1.0
    }

    /// Get time until recovery complete
    #[must_use]
    pub fn ticks_until_recovery(&self, current_tick: Tick) -> Option<Tick> {
        self.recovery_tick.map(|rt| rt.saturating_sub(current_tick))
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
