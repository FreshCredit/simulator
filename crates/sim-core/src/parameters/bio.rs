use super::{validate_min_max_range, ValidationError};
use crate::types::Tick;
use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Genome trait expression ranges
/// Per §7.5 genome specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeRangeParameters {
    /// DOCAS alpha (decay) range [min, max] per Dayan & Abbott 2001
    pub docas_alpha_range: (f32, f32),
    /// DOCAS beta (sensitivity) range [min, max] per Montague et al. 1996
    pub docas_beta_range: (f32, f32),
    /// Planning depth K range [min, max]
    pub planning_depth_range: (u32, u32),
    /// Planning horizon H range [min, max]
    pub planning_horizon_range: (u32, u32),
    /// Default susceptibility range [min, max]
    pub susceptibility_range: (f32, f32),
}

impl Default for GenomeRangeParameters {
    fn default() -> Self {
        Self {
            docas_alpha_range: (0.9, 0.99),
            docas_beta_range: (0.05, 0.2),
            planning_depth_range: (3, 20),
            planning_horizon_range: (1, 5),
            susceptibility_range: (0.6, 0.9),
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// DOCAS neuromodulator parameters
/// Per §6.1 lines 617-660
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocasParameters {
    /// Neutral state value for all components [0.0, 1.0]
    pub neutral_state: f32,
}

impl Default for DocasParameters {
    fn default() -> Self {
        Self { neutral_state: 0.5 }
    }
}

/// Decision-Making Network (DMN) parameters
/// Per §6.2 line 4481
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmnParameters {
    /// Computational budget per tick (ops/tick)
    pub budget: u32,
    /// Target criticality σ
    pub target_criticality: f32,
    /// Emotion baseline τ₀
    pub emotion_baseline: f32,
}

impl Default for DmnParameters {
    fn default() -> Self {
        Self {
            budget: 100,
            target_criticality: 1.0,
            emotion_baseline: 0.5,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Society and social dynamics parameters
/// Per §6.21-6.23 social dynamics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocietyParameters {
    // === Social Isolation (§6.21) ===
    /// Window for isolation calculation
    pub isolation_window: Tick,
    /// Maximum expected unique partners (Dunbar-inspired)
    pub max_expected_network: f32,
    /// Expected interactions per window
    pub max_expected_freq: f32,

    // === Attestation (§6.22) ===
    /// Recency decay rate (half-life ~700 ticks at 0.001)
    pub recency_decay_rate: f32,
    /// Base threshold for attestation
    pub attestation_threshold_base: f32,
    /// Oxytocin modulation factor
    pub attestation_oxytocin_factor: f32,
    /// Energy cost factor (fraction of attestor energy)
    pub attestation_energy_cost_factor: f32,
    /// Stake factor (fraction of reputation)
    pub attestation_stake_factor: f32,
    /// Attestation expiry duration
    pub attestation_expiry_ticks: Tick,

    // === Group Formation ===
    /// Minimum agents for group formation
    pub min_group_size: usize,

    // === Norm Emergence ===
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Frequency threshold for norm adoption (0.0-1.0)
    pub frequency_threshold: f32,
    /// High-trust agent threshold
    pub high_trust_threshold: f32,
    /// Leader adoption threshold
    pub leader_adoption_threshold: f32,
    /// Coordination failure threshold
    pub failure_threshold: f32,
    /// Minimum observation window
    pub min_window_ticks: Tick,
    /// Minimum coordination attempts
    pub min_attempts: u32,
    /// Early adopter fraction (20-30%)
    pub early_adopter_threshold: f32,
    /// Adoption rate multiplier at cascade
    pub cascade_acceleration: f32,

    // === Institution Emergence ===
    /// Institution failure threshold
    pub institution_failure_threshold: f32,
    /// Institution minimum window
    pub institution_min_window_ticks: Tick,
    /// Institution minimum attempts
    pub institution_min_attempts: u32,
    /// Fitness update learning rate
    pub institution_learning_rate: f32,
    /// Copied fitness decay factor
    pub institution_copy_fitness_factor: f32,
    /// Copied salience decay factor
    pub institution_copy_salience_factor: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for SocietyParameters {
    fn default() -> Self {
        Self {
            isolation_window: 200,
            max_expected_network: 10.0,
            max_expected_freq: 30.0,
            recency_decay_rate: 0.001,
            attestation_threshold_base: 0.5,
            attestation_oxytocin_factor: 0.2,
            attestation_energy_cost_factor: 0.01,
            attestation_stake_factor: 0.1,
            attestation_expiry_ticks: 1000,
            min_group_size: 2,
            frequency_threshold: 0.60,
            high_trust_threshold: 0.70,
            leader_adoption_threshold: 0.50,
            failure_threshold: 0.30,
            min_window_ticks: 50,
            min_attempts: 10,
            early_adopter_threshold: 0.25,
            cascade_acceleration: 1.5,
            institution_failure_threshold: 0.40,
            institution_min_window_ticks: 100,
            institution_min_attempts: 20,
            institution_learning_rate: 0.1,
            institution_copy_fitness_factor: 0.7,
            institution_copy_salience_factor: 0.8,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Language and semantic communication parameters
/// Per §7.5.1 line 4014 lexicon specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageParameters {
    /// Semantic embedding dimension
    pub embedding_dim: usize,
    /// Alignment factor for gradual concept alignment
    pub alignment_factor: f32,
    /// Lexicon overlap stability threshold (per §7586)
    pub overlap_stability_threshold: f32,
}

impl Default for LanguageParameters {
    fn default() -> Self {
        Self {
            embedding_dim: 8,
            alignment_factor: 0.1,
            overlap_stability_threshold: 0.3,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Blockchain protocol parameters
/// Per §D.3 verification economics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolParameters {
    /// Anchor cost range [min, max] in SIM tokens
    pub anchor_cost_range: (f32, f32),
    /// Verification cost range [min, max] in SIM tokens
    pub verify_cost_range: (f32, f32),
    /// Verification rate stability threshold (per §D.3 line 7587)
    pub verification_stability_threshold: f32,
    /// Rollback trigger threshold (anchor failure rate)
    pub rollback_failure_threshold: f32,
}

impl Default for ProtocolParameters {
    fn default() -> Self {
        Self {
            anchor_cost_range: (0.01, 0.1),
            verify_cost_range: (0.005, 0.05),
            verification_stability_threshold: 0.1,
            rollback_failure_threshold: 0.5,
        }
    }
}

impl GenomeRangeParameters {
    /// Validate genome range parameters
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_min_max_range(
            self.docas_alpha_range,
            "genome_ranges.docas_alpha_range",
            (0.8, 1.0),
        )?;
        validate_min_max_range(
            self.docas_beta_range,
            "genome_ranges.docas_beta_range",
            (0.0, 0.5),
        )?;
        Ok(())
    }
}
