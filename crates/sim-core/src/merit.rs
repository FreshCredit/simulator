//! `MeritRegistry`: Ray Dalio-style idea meritocracy system
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION merit system, not production governance
//!
// TAG: surface=simulation owner=research-team rule=SIM-001
//! Implements believability-weighted decision making where voting power
//! is proportional to demonstrated competence (merit).
//!
//! ## Core Concepts
//! - **Merit Attributes**: Measurable competencies (reputation, cooperation, verification, contribution)
//! - **Believability-Weighting**: Voting power proportional to merit scores
//! - **Dynamic Evolution**: Weights can adjust based on prediction accuracy
//!
//! ## Ray Dalio Principles Integration
//! - "Pain + Reflection = Progress" - merit evolves from outcomes
//! - "Radical Transparency" - all merit scores are observable
//! - "Believability-Weighted Decision Making" - competence determines influence

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::agent::Agent;
use crate::types::AgentId;

/// Unique identifier for merit attributes
pub type AttributeId = u32;

/// Standard merit attribute IDs for default registry
pub const ATTR_REPUTATION: AttributeId = 0;
/// attr cooperation.
pub const ATTR_COOPERATION: AttributeId = 1;
/// attr verification.
pub const ATTR_VERIFICATION: AttributeId = 2;
/// attr contribution.
pub const ATTR_CONTRIBUTION: AttributeId = 3;

/// Lifecycle status of a merit attribute
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttributeStatus {
    /// Attribute is active and contributes to merit scores
    Active,
    /// Attribute is proposed but not yet active (pending governance)
    Proposed,
    /// Attribute is deprecated (will be removed in future tick)
    Deprecated,
    /// Attribute is removed (no longer tracked)
    Removed,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl AttributeStatus {
    /// Check if this status contributes to merit scores
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }
}

/// A single merit attribute definition
///
/// Represents one attribute used in merit score computation
/// with its weight, status, and metadata.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeritAttribute {
    /// Unique attribute identifier
    pub id: AttributeId,
    /// Human-readable name
    pub name: String,
    /// Weight in merit score computation [0.0, 1.0]
    pub weight: f32,
    /// Tick when attribute was added
    pub added_tick: u64,
    /// Current lifecycle status
    pub status: AttributeStatus,
    /// Source function name for attribute computation
    pub source_fn_name: String,
    /// Description of what this attribute measures
    pub description: String,
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
impl MeritAttribute {
    /// Create a new merit attribute
    pub fn new(
        id: AttributeId,
        name: impl Into<String>,
        weight: f32,
        added_tick: u64,
        source_fn_name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            weight: weight.clamp(0.0, 1.0),
            added_tick,
            status: AttributeStatus::Active,
            source_fn_name: source_fn_name.into(),
            description: description.into(),
        }
    }

    /// Check if this attribute depends on Protocol being enabled
    #[must_use]
    pub fn is_protocol_dependent(&self) -> bool {
        // Verification-based attributes depend on Protocol
        self.name.to_lowercase().contains("verification")
            || self.name.to_lowercase().contains("verified")
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Snapshot of weights at a specific tick (for history/auditing)
///
/// Records the complete weight configuration at a point in time
/// for auditing and historical analysis.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeightSnapshot {
    /// Tick when snapshot was taken
    pub tick: u64,
    /// Attribute weights at this tick
    pub weights: HashMap<AttributeId, f32>,
    /// Rationale for weight configuration
    pub rationale: String,
}

impl WeightSnapshot {
    /// Create a new weight snapshot
    pub fn new(
        tick: u64,
        weights: HashMap<AttributeId, f32>,
        rationale: impl Into<String>,
    ) -> Self {
        Self {
            tick,
            weights,
            rationale: rationale.into(),
        }
    }
}

/// Governance type for merit weight evolution
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MeritGovernanceType {
    /// Weights are fixed and only change through explicit governance
    #[default]
    Fixed,
    /// Weights evolve based on voting outcomes (prediction accuracy)
    DynamicVote,
    /// Weights are optimized by machine learning algorithms
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    MachineLearning,
}

/// Result of a believability-weighted vote
///
/// Aggregates weighted yes/no votes and determines pass/fail
/// based on merit-weighted participation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WeightedVoteResult {
    /// Sum of merit-weighted yes votes
    pub weighted_yes: f32,
    /// Sum of merit-weighted no votes
    pub weighted_no: f32,
    /// Total merit score of all participating voters
    pub total_merit_participating: f32,
    /// Whether the vote passed (`weighted_yes` > `weighted_no`)
    pub passed: bool,
    /// Vote margin (`weighted_yes` - `weighted_no`) / `total_merit_participating`
    pub margin: f32,
}

impl WeightedVoteResult {
    /// Create a new vote result
    #[must_use]
    pub fn new(weighted_yes: f32, weighted_no: f32, total_merit: f32) -> Self {
        let passed = weighted_yes > weighted_no;
        let margin = if total_merit > 0.0 {
            (weighted_yes - weighted_no) / total_merit
        } else {
            0.0
        };
        Self {
            weighted_yes,
            weighted_no,
            total_merit_participating: total_merit,
            passed,
            margin,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get the yes ratio (0.0 to 1.0)
    #[must_use]
    pub fn yes_ratio(&self) -> f32 {
        let total = self.weighted_yes + self.weighted_no;
        if total > 0.0 {
            self.weighted_yes / total
        } else {
            0.5
        }
    }

    /// Get participation rate relative to total possible merit
    #[must_use]
    pub fn participation_rate(&self, total_possible_merit: f32) -> f32 {
        if total_possible_merit > 0.0 {
            self.total_merit_participating / total_possible_merit
        } else {
            0.0
        }
    }
}

/// Errors that can occur in merit operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MeritError {
    /// Invalid weight value (outside [0.0, 1.0])
    InvalidWeight(f32),
    /// Attribute not found in registry
    AttributeNotFound(AttributeId),
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Duplicate attribute name
    DuplicateName(String),
    /// Weight sum mismatch (doesn't sum to ~1.0)
    WeightSumMismatch {
        /// expected.
        expected: f32,
        /// actual.
        actual: f32,
    },
    /// Invalid governance operation
    InvalidGovernanceOperation(String),
}

impl std::fmt::Display for MeritError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidWeight(w) => {
                write!(f, "Invalid weight: {w} (must be in [0.0, 1.0])")
            }
            Self::AttributeNotFound(id) => write!(f, "Attribute not found: {id}"),
            Self::DuplicateName(name) => write!(f, "Duplicate attribute name: {name}"),
            Self::WeightSumMismatch { expected, actual } => {
                write!(
                    f,
                    "Weight sum mismatch: expected ~{expected}, actual {actual}"
                )
            }
            Self::InvalidGovernanceOperation(msg) => {
                write!(f, "Invalid governance operation: {msg}")
            }
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl std::error::Error for MeritError {}

/// `MeritRegistry`: Ray Dalio-style idea meritocracy system
///
/// Implements believability-weighted decision making where voting power
/// is proportional to demonstrated competence (merit).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeritRegistry {
    /// All merit attributes indexed by ID
    pub attributes: HashMap<AttributeId, MeritAttribute>,
    /// Historical weight snapshots for audit trail
    pub weight_history: Vec<WeightSnapshot>,
    /// Next attribute ID to assign
    next_id: AttributeId,
    /// Governance mode for weight evolution
    pub governance_type: MeritGovernanceType,
    /// Minimum merit score threshold for voting (0.0 = no threshold)
    pub min_voting_threshold: f32,
}

impl MeritRegistry {
    /// Create a new empty registry
    #[must_use]
    pub fn new(governance_type: MeritGovernanceType) -> Self {
        Self {
            attributes: HashMap::new(),
            weight_history: Vec::new(),
            next_id: 0,
            governance_type,
            min_voting_threshold: 0.0,
        }
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Create default registry with standard attributes
    ///
    /// Default attributes:
    /// - reputation (0.40) - Social standing from attestations
    /// - cooperation (0.30) - Historical cooperation rate
    /// - verification (0.20) - Protocol verification count (Protocol-dependent)
    /// - contribution (0.10) - Value contributed to network
    #[must_use]
    pub fn new_default(start_tick: u64) -> Self {
        let mut registry = Self::new(MeritGovernanceType::Fixed);

        // Add default attributes with standard weights
        registry.add_attribute_internal(
            ATTR_REPUTATION,
            "reputation",
            0.40,
            start_tick,
            "compute_reputation_score",
            "Social standing derived from attestation network and interaction history",
        );

        registry.add_attribute_internal(
            ATTR_COOPERATION,
            "cooperation",
            0.30,
            start_tick,
            "compute_cooperation_rate",
            "Historical cooperation rate from game interactions and trades",
        );

        registry.add_attribute_internal(
            ATTR_VERIFICATION,
            "verification",
            0.20,
            start_tick,
            "compute_verification_score",
            "Protocol-based identity verification and claim anchoring (requires Protocol ON)",
        );

        // TAG: surface=simulation owner=research-team rule=SIM-001
        registry.add_attribute_internal(
            ATTR_CONTRIBUTION,
            "contribution",
            0.10,
            start_tick,
            "compute_contribution_score",
            "Value contributed to network through attestations and group activities",
        );

        // Record initial snapshot
        let _ =
            registry.record_snapshot(start_tick, "Initial default merit registry configuration");

        registry
    }

    /// Add a new merit attribute
    ///
    /// After adding, weights are re-normalized to sum to 1.0
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn add_attribute(
        &mut self,
        name: &str,
        weight: f32,
        tick: u64,
        description: &str,
    ) -> Result<AttributeId, MeritError> {
        // Validate weight
        if !(0.0..=1.0).contains(&weight) {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            return Err(MeritError::InvalidWeight(weight));
        }

        // Check for duplicate name
        if self.attributes.values().any(|attr| attr.name == name) {
            return Err(MeritError::DuplicateName(name.to_string()));
        }

        let id = self.next_id;
        self.next_id += 1;

        self.add_attribute_internal(
            id,
            name,
            weight,
            tick,
            &format!("compute_{name}_score"),
            description,
        );

        // Normalize weights to sum to 1.0
        self.normalize_weights()?;

        Ok(id)
    }

    /// Internal method to add an attribute with a specific ID
    fn add_attribute_internal(
        &mut self,
        id: AttributeId,
        name: &str,
        weight: f32,
        tick: u64,
        source_fn_name: &str,
        description: &str,
    ) {
        let attr = MeritAttribute::new(id, name, weight, tick, source_fn_name, description);
        self.attributes.insert(id, attr);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get an attribute by ID
    #[must_use]
    pub fn get_attribute(&self, id: AttributeId) -> Option<&MeritAttribute> {
        self.attributes.get(&id)
    }

    /// Get an attribute by name
    #[must_use]
    pub fn get_attribute_by_name(&self, name: &str) -> Option<&MeritAttribute> {
        self.attributes.values().find(|attr| attr.name == name)
    }

    /// Get all active attributes
    #[must_use]
    pub fn active_attributes(&self) -> Vec<&MeritAttribute> {
        self.attributes
            .values()
            .filter(|attr| attr.status.is_active())
            .collect()
    }

    /// Compute merit score for an agent
    ///
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Weighted sum of attributes. Skips verification-based attributes
    /// if Protocol is OFF. Normalizes weights if sum != 1.0.
    #[must_use]
    pub fn compute_merit_score(&self, agent: &Agent, protocol_enabled: bool) -> f32 {
        let active_attrs: Vec<&MeritAttribute> = self
            .attributes
            .values()
            .filter(|attr| {
                attr.status.is_active() && (protocol_enabled || !attr.is_protocol_dependent())
            })
            .collect();

        if active_attrs.is_empty() {
            return 0.5; // Neutral score if no attributes
        }

        // Compute raw weight sum for normalization
        let total_weight: f32 = active_attrs.iter().map(|attr| attr.weight).sum();
        if total_weight == 0.0 {
            return 0.5;
        }

        // Compute weighted sum of attribute values
        let mut weighted_sum = 0.0;
        for attr in &active_attrs {
            let value = self.compute_attribute_value(agent, attr);
            weighted_sum += attr.weight * value;
        }

        // Normalize by total weight
        (weighted_sum / total_weight).clamp(0.0, 1.0)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute the value of a specific attribute for an agent
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn compute_attribute_value(&self, agent: &Agent, attr: &MeritAttribute) -> f32 {
        match attr.id {
            ATTR_REPUTATION => agent.reputation_score,
            ATTR_COOPERATION => agent.interaction_reputation(),
            ATTR_VERIFICATION => {
                // Verification score based on verification count and verified ID status
                let count_factor = (agent.verification_count as f32 / 100.0).min(1.0);
                let verified_factor = if agent.has_verified_id { 1.0 } else { 0.5 };
                count_factor * verified_factor
            }
            ATTR_CONTRIBUTION => {
                // Contribution based on attestations given and economic activity
                let attestation_factor = (agent.attestations_given.len() as f32 / 50.0).min(1.0);
                let economic_factor = (agent.economics.wealth / 1000.0).min(1.0);
                (attestation_factor * 0.6 + economic_factor * 0.4).min(1.0)
            }
            _ => {
                // For custom attributes, use a default value based on agent properties
                // This would be extended with custom computation functions
                0.5
            }
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    /// Update attribute weight (governance function)
    ///
    /// After updating, weights are re-normalized to sum to 1.0
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn update_weight(
        &mut self,
        attr_id: AttributeId,
        new_weight: f32,
        tick: u64,
        rationale: &str,
    ) -> Result<(), MeritError> {
        // Validate weight
        if !(0.0..=1.0).contains(&new_weight) {
            return Err(MeritError::InvalidWeight(new_weight));
        }

        // Find attribute
        let attr = self
            .attributes
            .get_mut(&attr_id)
            .ok_or(MeritError::AttributeNotFound(attr_id))?;

        // Update weight
        attr.weight = new_weight;

        // Re-normalize weights to ensure they sum to 1.0
        self.normalize_weights()?;

        // Record snapshot
        self.record_snapshot(tick, rationale)?;

        Ok(())
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Set attribute status (e.g., deprecate, activate)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn set_attribute_status(
        &mut self,
        attr_id: AttributeId,
        status: AttributeStatus,
        tick: u64,
    ) -> Result<(), MeritError> {
        // Get attribute name first before mutable borrow
        let attr_name = self
            .attributes
            .get(&attr_id)
            .map(|attr| attr.name.clone())
            .ok_or(MeritError::AttributeNotFound(attr_id))?;

        // Now update the status
        let attr = self
            .attributes
            .get_mut(&attr_id)
            .ok_or(MeritError::AttributeNotFound(attr_id))?;
        attr.status = status;

        // Record snapshot for significant status changes
        if status == AttributeStatus::Removed || status == AttributeStatus::Deprecated {
            self.record_snapshot(
                tick,
                &format!("Attribute {attr_name} status changed to {status:?}"),
            )?;
        }

        Ok(())
    }

    /// Record a weight snapshot
    fn record_snapshot(&mut self, tick: u64, rationale: &str) -> Result<(), MeritError> {
        let weights: HashMap<AttributeId, f32> = self
            .attributes
            .iter()
            .filter(|(_, attr)| attr.status.is_active())
            .map(|(id, attr)| (*id, attr.weight))
            .collect();

        let snapshot = WeightSnapshot::new(tick, weights, rationale);
        self.weight_history.push(snapshot);

        Ok(())
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get top agents by merit (for leader selection)
    #[must_use]
    pub fn get_top_agents(
        &self,
        agents: &[Agent],
        protocol_enabled: bool,
        n: usize,
    ) -> Vec<(AgentId, f32)> {
        let mut scored: Vec<(AgentId, f32)> = agents
            .iter()
            .map(|agent| {
                let score = self.compute_merit_score(agent, protocol_enabled);
                (agent.id, score)
            })
            .collect();

        // Sort by score descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top n
        scored.into_iter().take(n).collect()
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    /// Compute believability-weighted vote
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[must_use]
    pub fn compute_weighted_vote(
        &self,
        votes: &[(AgentId, bool)], // (voter_id, vote_yes)
        agents: &HashMap<AgentId, Agent>,
        protocol_enabled: bool,
    ) -> WeightedVoteResult {
        let mut weighted_yes = 0.0;
        let mut weighted_no = 0.0;
        let mut total_merit = 0.0;

        for (agent_id, vote_yes) in votes {
            if let Some(agent) = agents.get(agent_id) {
                let merit = self.compute_merit_score(agent, protocol_enabled);

                // Apply voting threshold
                if merit >= self.min_voting_threshold {
                    if *vote_yes {
                        weighted_yes += merit;
                    } else {
                        weighted_no += merit;
                    }
                    total_merit += merit;
                }
            }
        }

        WeightedVoteResult::new(weighted_yes, weighted_no, total_merit)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Evolution: adjust weights based on prediction accuracy
    ///
    /// For `DynamicVote` governance: adjusts weights based on outcomes
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn evolve_weights(
        &mut self,
        accuracy_data: &[(AttributeId, f32)],
        tick: u64,
    ) -> Result<(), MeritError> {
        if self.governance_type != MeritGovernanceType::DynamicVote {
            return Err(MeritError::InvalidGovernanceOperation(
                "Weight evolution only available in DynamicVote mode".to_string(),
            ));
        }

        // Adjust weights based on prediction accuracy
        // Higher accuracy → higher weight (agents with this attribute made better predictions)
        for (attr_id, accuracy) in accuracy_data {
            if let Some(attr) = self.attributes.get_mut(attr_id) {
                if attr.status.is_active() {
                    // Adjust weight by small amount proportional to accuracy
                    let adjustment = (accuracy - 0.5) * 0.05; // ±2.5% max adjustment
                    attr.weight = (attr.weight + adjustment).clamp(0.01, 0.99);
                    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                }
            }
        }

        // Normalize weights to sum to 1.0
        self.normalize_weights()?;

        // Record snapshot
        self.record_snapshot(tick, "Weight evolution based on prediction accuracy")?;

        Ok(())
    }

    /// Normalize all active weights to sum to 1.0
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn normalize_weights(&mut self) -> Result<(), MeritError> {
        let active_attrs: Vec<AttributeId> = self
            .attributes
            .values()
            .filter(|attr| attr.status.is_active())
            .map(|attr| attr.id)
            .collect();

        let total: f32 = active_attrs
            .iter()
            .filter_map(|id| self.attributes.get(id).map(|a| a.weight))
            .sum();

        if total > 0.0 {
            for id in active_attrs {
                if let Some(attr) = self.attributes.get_mut(&id) {
                    attr.weight /= total;
                }
            }
        }

        Ok(())
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Validate weight distribution (must sum to ~1.0)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn validate_weights(&self) -> Result<(), MeritError> {
        let active_weights: Vec<f32> = self
            .attributes
            .values()
            .filter(|attr| attr.status.is_active())
            .map(|attr| attr.weight)
            .collect();

        let sum: f32 = active_weights.iter().sum();

        // Allow 5% tolerance
        let tolerance = 0.05;
        if (sum - 1.0).abs() > tolerance {
            return Err(MeritError::WeightSumMismatch {
                expected: 1.0,
                actual: sum,
            });
        }

        Ok(())
    }

    /// Get weight history
    #[must_use]
    pub fn weight_history(&self) -> &[WeightSnapshot] {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        &self.weight_history
    }

    /// Set minimum voting threshold
    pub fn set_voting_threshold(&mut self, threshold: f32) {
        self.min_voting_threshold = threshold.clamp(0.0, 1.0);
    }

    /// Get registry statistics
    #[must_use]
    pub fn stats(&self) -> MeritRegistryStats {
        let active_count = self
            .attributes
            .values()
            .filter(|attr| attr.status.is_active())
            .count();

        let total_weight: f32 = self
            .attributes
            .values()
            .filter(|attr| attr.status.is_active())
            .map(|attr| attr.weight)
            .sum();

        let protocol_dependent_count = self
            .attributes
            .values()
            .filter(|attr| attr.is_protocol_dependent())
            .count();

        MeritRegistryStats {
            total_attributes: self.attributes.len(),
            active_attributes: active_count,
            protocol_dependent_attributes: protocol_dependent_count,
            total_weight,
            governance_type: self.governance_type,
            history_snapshots: self.weight_history.len(),
        }
    }

    /// Get the default merit score (for agents not in registry)
    #[must_use]
    pub const fn default_merit_score(&self) -> f32 {
        0.5 // Neutral default
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for MeritRegistry {
    fn default() -> Self {
        Self::new(MeritGovernanceType::Fixed)
    }
}

/// Statistics for the merit registry
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MeritRegistryStats {
    /// Total number of attributes
    pub total_attributes: usize,
    /// Number of active attributes
    pub active_attributes: usize,
    /// Number of protocol-dependent attributes
    pub protocol_dependent_attributes: usize,
    /// Sum of active attribute weights
    pub total_weight: f32,
    /// Current governance type
    pub governance_type: MeritGovernanceType,
    /// Number of weight history snapshots
    pub history_snapshots: usize,
}
// TAG: surface=simulation owner=platform-team rule=GENERAL-001

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genome::Genome;
    use crate::types::{Position, Sex};
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    // === Test 1: Default Registry Creation ===
    #[test]
    fn test_default_registry_creation() {
        let registry = MeritRegistry::new_default(0);

        assert_eq!(registry.attributes.len(), 4);
        assert!(registry.attributes.contains_key(&ATTR_REPUTATION));
        assert!(registry.attributes.contains_key(&ATTR_COOPERATION));
        assert!(registry.attributes.contains_key(&ATTR_VERIFICATION));
        assert!(registry.attributes.contains_key(&ATTR_CONTRIBUTION));

        // Check default weights
        let rep = registry.get_attribute(ATTR_REPUTATION).unwrap();
        assert_eq!(rep.weight, 0.40);
        assert_eq!(rep.name, "reputation");

        let coop = registry.get_attribute(ATTR_COOPERATION).unwrap();
        assert_eq!(coop.weight, 0.30);

        let verif = registry.get_attribute(ATTR_VERIFICATION).unwrap();
        assert_eq!(verif.weight, 0.20);

        let contrib = registry.get_attribute(ATTR_CONTRIBUTION).unwrap();
        assert_eq!(contrib.weight, 0.10);
    }

    // === Test 2: Merit Score Computation ===
    #[test]
    fn test_merit_score_computation() {
        let registry = MeritRegistry::new_default(0);
        let genome = Genome::random(&mut StdRng::seed_from_u64(42));
        let mut agent = Agent::new(1, Position::new_2d(10.0, 10.0), genome, 0, Sex::Male);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Set agent properties
        agent.reputation_score = 0.8;
        agent.positive_interactions = 80;
        agent.negative_interactions = 20;
        agent.has_verified_id = true;
        agent.verification_count = 50;

        // Protocol ON
        let score_on = registry.compute_merit_score(&agent, true);
        assert!(score_on > 0.0 && score_on <= 1.0);

        // Protocol OFF - verification attribute should be skipped
        let score_off = registry.compute_merit_score(&agent, false);
        assert!(score_off > 0.0 && score_off <= 1.0);

        // With verification contributing, score should be different
        // (but not necessarily higher - depends on verification score)
        assert!((score_on - score_off).abs() < 1.0); // Sanity check
    }

    // === Test 3: Weight Updates ===
    #[test]
    fn test_weight_updates() {
        let mut registry = MeritRegistry::new_default(0);

        // Update reputation weight (will be normalized)
        registry
            .update_weight(
                ATTR_REPUTATION,
                0.50,
                100,
                "Increased reputation importance",
            )
            .unwrap();

        // After normalization, weights should sum to 1.0
        let total: f32 = registry.active_attributes().iter().map(|a| a.weight).sum();
        assert!(
            (total - 1.0).abs() < 0.01,
            "Weights should sum to 1.0 after normalization"
        );

        // Check that snapshot was recorded
        assert_eq!(registry.weight_history.len(), 2); // Initial + update

        // Invalid weight should fail
        let result = registry.update_weight(ATTR_REPUTATION, 1.5, 200, "Invalid");
        assert!(matches!(result, Err(MeritError::InvalidWeight(1.5))));

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Non-existent attribute should fail
        let result = registry.update_weight(999, 0.5, 200, "Invalid");
        assert!(matches!(result, Err(MeritError::AttributeNotFound(999))));
    }

    // === Test 4: Protocol ON/OFF Behavior ===
    #[test]
    fn test_protocol_on_off_behavior() {
        let registry = MeritRegistry::new_default(0);
        let genome = Genome::random(&mut StdRng::seed_from_u64(42));
        let mut agent = Agent::new(1, Position::new_2d(10.0, 10.0), genome, 0, Sex::Male);

        // High verification score, low cooperation
        agent.has_verified_id = true;
        agent.verification_count = 100;
        agent.positive_interactions = 0;
        agent.negative_interactions = 100;

        let score_on = registry.compute_merit_score(&agent, true);
        let score_off = registry.compute_merit_score(&agent, false);

        // With protocol on, verification contributes (higher score expected)
        // With protocol off, verification is skipped (lower score expected)
        assert!(
            score_on > score_off,
            "Protocol ON should include verification score"
        );
    }

    // === Test 5: Weighted Voting ===
    #[test]
    fn test_weighted_voting() {
        let registry = MeritRegistry::new_default(0);
        let genome = Genome::random(&mut StdRng::seed_from_u64(42));
        let mut agents = HashMap::new();

        // Create agents with different merit scores
        for i in 1..=5 {
            let mut agent = Agent::new(
                i,
                Position::new_2d(10.0, 10.0),
                genome.clone(),
                0,
                Sex::Male,
            );
            agent.reputation_score = i as f32 * 0.2;
            agents.insert(i, agent);
        }

        // All vote yes
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let votes: Vec<(AgentId, bool)> =
            vec![(1, true), (2, true), (3, true), (4, true), (5, true)];
        let result = registry.compute_weighted_vote(&votes, &agents, true);

        assert!(result.passed);
        assert!(result.weighted_yes > 0.0);
        assert_eq!(result.weighted_no, 0.0);

        // Split vote - high merit agents vote no
        let votes: Vec<(AgentId, bool)> =
            vec![(1, true), (2, true), (3, false), (4, false), (5, false)];
        let result = registry.compute_weighted_vote(&votes, &agents, true);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // Agents 3, 4, 5 (higher merit) vote no, should outweigh agents 1, 2
        assert!(
            !result.passed,
            "High merit agents voting no should outweigh low merit agents voting yes"
        );
    }

    // === Test 6: Top Agent Selection ===
    #[test]
    fn test_top_agent_selection() {
        let registry = MeritRegistry::new_default(0);
        let genome = Genome::random(&mut StdRng::seed_from_u64(42));
        let mut agents = Vec::new();

        // Create 10 agents with varying reputation
        for i in 1..=10 {
            let mut agent = Agent::new(
                i,
                Position::new_2d(10.0, 10.0),
                genome.clone(),
                0,
                Sex::Male,
            );
            agent.reputation_score = i as f32 * 0.1;
            agents.push(agent);
        }

        let top_3 = registry.get_top_agents(&agents, true, 3);
        assert_eq!(top_3.len(), 3);
        // Highest merit agents should be last (agent 10, 9, 8)
        assert_eq!(top_3[0].0, 10);
        assert_eq!(top_3[1].0, 9);
        assert_eq!(top_3[2].0, 8);
    }

    // === Test 7: Weight Validation ===
    #[test]
    fn test_weight_validation() {
        let mut registry = MeritRegistry::new_default(0);

        // Default weights should sum to 1.0
        assert!(registry.validate_weights().is_ok());

        // Modify weight to break sum
        if let Some(attr) = registry.attributes.get_mut(&ATTR_REPUTATION) {
            attr.weight = 0.9;
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Should fail validation (sum now exceeds 1.0)
        let result = registry.validate_weights();
        assert!(matches!(result, Err(MeritError::WeightSumMismatch { .. })));
    }

    // === Test 8: Weight Evolution ===
    #[test]
    fn test_weight_evolution() {
        let mut registry = MeritRegistry::new(MeritGovernanceType::DynamicVote);
        // Add attributes with equal weights
        registry
            .add_attribute("test_attr", 0.5, 0, "Test attribute")
            .unwrap();
        registry
            .add_attribute("test_attr2", 0.5, 0, "Test attribute 2")
            .unwrap();

        // Verify initial weights sum to 1.0 (normalized)
        let total_before: f32 = registry.active_attributes().iter().map(|a| a.weight).sum();
        assert!((total_before - 1.0).abs() < 0.01);

        let accuracy_data = vec![(0, 0.9), (1, 0.3)];
        let result = registry.evolve_weights(&accuracy_data, 100);
        assert!(result.is_ok());

        // After evolution and normalization, weights should still sum to 1.0
        let total_after: f32 = registry.active_attributes().iter().map(|a| a.weight).sum();
        assert!(
            (total_after - 1.0).abs() < 0.01,
            "Weights should sum to 1.0 after evolution"
        );

        // First attribute should have higher relative weight (high accuracy), second lower
        let attr1 = registry.get_attribute(0).unwrap();
        let attr2 = registry.get_attribute(1).unwrap();
        assert!(
            attr1.weight > attr2.weight,
            "Higher accuracy attribute should have higher weight"
        );
    }

    // === Test 9: Fixed Governance Mode ===
    #[test]
    fn test_fixed_governance_mode() {
        let mut registry = MeritRegistry::new(MeritGovernanceType::Fixed);
        registry.add_attribute("test_attr", 0.5, 0, "Test").unwrap();
        registry
            .add_attribute("test_attr2", 0.5, 0, "Test 2")
            .unwrap();

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let accuracy_data = vec![(0, 0.9), (1, 0.3)];
        let result = registry.evolve_weights(&accuracy_data, 100);

        // Should fail - evolution not allowed in Fixed mode
        assert!(matches!(
            result,
            Err(MeritError::InvalidGovernanceOperation(_))
        ));

        // But weights should sum to 1.0 (normalized during add)
        let total: f32 = registry.active_attributes().iter().map(|a| a.weight).sum();
        assert!((total - 1.0).abs() < 0.01);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    // === Test 10: Add Attribute ===
    #[test]
    fn test_add_attribute() {
        let mut registry = MeritRegistry::new(MeritGovernanceType::Fixed);

        let id = registry
            .add_attribute("custom_attr", 0.25, 0, "Custom attribute")
            .unwrap();
        assert_eq!(id, 0);

        let attr = registry.get_attribute(id).unwrap();
        assert_eq!(attr.name, "custom_attr");
        // Note: weight may be normalized if it's the only attribute
        assert!(attr.weight > 0.0 && attr.weight <= 1.0);
        assert_eq!(attr.status, AttributeStatus::Active);

        // Duplicate name should fail
        let result = registry.add_attribute("custom_attr", 0.3, 0, "Duplicate");
        assert!(matches!(result, Err(MeritError::DuplicateName(_))));

        // Invalid weight should fail
        let result = registry.add_attribute("invalid", 1.5, 0, "Invalid weight");
        assert!(matches!(result, Err(MeritError::InvalidWeight(1.5))));
    }

    // === Test 11: Attribute Status Management ===
    #[test]
    fn test_attribute_status_management() {
        let mut registry = MeritRegistry::new_default(0);

        // Deprecate an attribute
        registry
            .set_attribute_status(ATTR_VERIFICATION, AttributeStatus::Deprecated, 100)
            .unwrap();

        let attr = registry.get_attribute(ATTR_VERIFICATION).unwrap();
        assert_eq!(attr.status, AttributeStatus::Deprecated);
        assert!(!attr.status.is_active());

        // Non-existent attribute should fail
        let result = registry.set_attribute_status(999, AttributeStatus::Removed, 100);
        assert!(matches!(result, Err(MeritError::AttributeNotFound(999))));
    }

    // === Test 12: Voting Threshold ===
    #[test]
    fn test_voting_threshold() {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let mut registry = MeritRegistry::new_default(0);
        registry.set_voting_threshold(0.5);

        let genome = Genome::random(&mut StdRng::seed_from_u64(42));
        let mut agents = HashMap::new();

        // Create agents - one with low merit, one with high merit
        let mut low_merit = Agent::new(
            1,
            Position::new_2d(10.0, 10.0),
            genome.clone(),
            0,
            Sex::Male,
        );
        low_merit.reputation_score = 0.3; // Below threshold
        agents.insert(1, low_merit);

        let mut high_merit = Agent::new(2, Position::new_2d(10.0, 10.0), genome, 0, Sex::Male);
        high_merit.reputation_score = 0.8; // Above threshold
        agents.insert(2, high_merit);

        // Both vote yes, but only high_merit counts
        let votes = vec![(1, true), (2, true)];
        let result = registry.compute_weighted_vote(&votes, &agents, true);

        // Only agent 2 should contribute
        assert!(result.total_merit_participating < 1.0);
    }

    // === Test 13: Vote Result Methods ===
    #[test]
    fn test_vote_result_methods() {
        let result = WeightedVoteResult::new(70.0, 30.0, 100.0);

        assert!(result.passed);
        assert!((result.margin - 0.4).abs() < 0.01);
        assert!((result.yes_ratio() - 0.7).abs() < 0.01);
        assert!((result.participation_rate(100.0) - 1.0).abs() < 0.01);

        let result = WeightedVoteResult::new(30.0, 70.0, 100.0);
        assert!(!result.passed);
        assert!(result.margin < 0.0);
    }

    // === Test 14: Registry Statistics ===
    #[test]
    fn test_registry_stats() {
        let registry = MeritRegistry::new_default(0);
        let stats = registry.stats();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(stats.total_attributes, 4);
        assert_eq!(stats.active_attributes, 4);
        assert_eq!(stats.protocol_dependent_attributes, 1); // verification
        assert!(stats.total_weight > 0.95 && stats.total_weight < 1.05);
        assert_eq!(stats.governance_type, MeritGovernanceType::Fixed);
        assert_eq!(stats.history_snapshots, 1);
    }

    // === Test 15: Empty Registry ===
    #[test]
    fn test_empty_registry() {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let registry = MeritRegistry::new(MeritGovernanceType::Fixed);
        let genome = Genome::random(&mut StdRng::seed_from_u64(42));
        let agent = Agent::new(1, Position::new_2d(10.0, 10.0), genome, 0, Sex::Male);

        let score = registry.compute_merit_score(&agent, true);
        assert_eq!(score, 0.5); // Default neutral score
    }

    // === Test 16: Protocol Dependency Detection ===
    #[test]
    fn test_protocol_dependency_detection() {
        let attr1 =
            MeritAttribute::new(0, "reputation", 0.5, 0, "compute_reputation", "Reputation");
        let attr2 = MeritAttribute::new(
            1,
            "verification_score",
            0.3,
            0,
            "compute_verification",
            "Verification",
        );
        let attr3 = MeritAttribute::new(
            2,
            "verified_identity",
            0.2,
            0,
            "compute_identity",
            "Identity",
        );
        let attr4 = MeritAttribute::new(
            3,
            "cooperation",
            0.4,
            0,
            "compute_cooperation",
            "Cooperation",
        );

        assert!(!attr1.is_protocol_dependent());
        assert!(attr2.is_protocol_dependent());
        assert!(attr3.is_protocol_dependent());
        assert!(!attr4.is_protocol_dependent());
    }

    // === Test 17: Weight History ===
    #[test]
    fn test_weight_history() {
        let mut registry = MeritRegistry::new_default(0);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Initial snapshot exists
        assert_eq!(registry.weight_history.len(), 1);

        // Update weights to create more snapshots
        registry
            .update_weight(ATTR_REPUTATION, 0.45, 100, "Adjust reputation")
            .unwrap();
        registry
            .update_weight(ATTR_COOPERATION, 0.35, 200, "Adjust cooperation")
            .unwrap();

        // Each update records a snapshot
        assert!(registry.weight_history.len() >= 2);

        // Check snapshot content
        let snapshot = &registry.weight_history[0];
        assert_eq!(snapshot.tick, 0);
        assert!(snapshot.rationale.contains("Initial"));
    }

    // === Test 18: Vote Result Edge Cases ===
    #[test]
    fn test_vote_result_edge_cases() {
        // Empty vote
        let result = WeightedVoteResult::new(0.0, 0.0, 0.0);
        assert!(!result.passed);
        assert_eq!(result.margin, 0.0);
        assert_eq!(result.yes_ratio(), 0.5);

        // Tie vote
        let result = WeightedVoteResult::new(50.0, 50.0, 100.0);
        assert!(!result.passed); // Tie = not passed
        assert_eq!(result.margin, 0.0);

        // Unanimous
        let result = WeightedVoteResult::new(100.0, 0.0, 100.0);
        assert!(result.passed);
        assert_eq!(result.margin, 1.0);
        assert_eq!(result.yes_ratio(), 1.0);
    }

    // === Test 19: Attribute Status Variants ===
    #[test]
    fn test_attribute_status_variants() {
        assert!(AttributeStatus::Active.is_active());
        assert!(!AttributeStatus::Proposed.is_active());
        assert!(!AttributeStatus::Deprecated.is_active());
        assert!(!AttributeStatus::Removed.is_active());
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    // === Test 20: Get Attribute By Name ===
    #[test]
    fn test_get_attribute_by_name() {
        let registry = MeritRegistry::new_default(0);

        let attr = registry.get_attribute_by_name("reputation").unwrap();
        assert_eq!(attr.id, ATTR_REPUTATION);

        let attr = registry.get_attribute_by_name("verification").unwrap();
        assert_eq!(attr.id, ATTR_VERIFICATION);

        assert!(registry.get_attribute_by_name("nonexistent").is_none());
    }
}
