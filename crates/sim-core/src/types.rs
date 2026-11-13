//! Core type definitions for the simulation.
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - these are SIMULATION types, not `FreshCredit` product types

use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Unique identifier for agents (per §7.5.1 line 3884)
pub type AgentId = u64;

/// Unique identifier for groups (per §7.5.1 line 3901)
pub type GroupId = u64;

/// Unique identifier for norms (per §7.5.1 line 3902)
pub type NormId = u64;

/// Unique identifier for institutions (per §4311-4338)
pub type InstitutionId = u64;

/// Unique identifier for concepts in lexicon (per §7.5.1 line 3911)
pub type ConceptId = u64;

/// Unique identifier for attestations (per §7.5.1 lines 3907-3908)
pub type AttestationId = u64;

/// Simulation tick counter (per §7.5.1 line 3890)
pub type Tick = u64;

/// Energy level for agents (per §7.5.1 line 3889)
pub type Energy = f32;

/// Altitude layer type for 3D environment
pub type AltitudeLayer = usize;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Grid position (x, y, z) - Full 3D support with altitude layer
/// Per §7.5.1 line 3886: position: (f32, f32, f32)
/// Supports 3D spatial navigation with altitude effects
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    /// x.
    pub x: f32,
    /// y.
    pub y: f32,
    /// Altitude/elevation (z-coordinate)
    /// Higher altitude = lower temperature, different resources
    pub z: f32,
}

impl Position {
    /// Create a new 3D position
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Create a new 2D position (z defaults to 0.0 for backward compatibility)
    #[must_use]
    pub const fn new_2d(x: f32, y: f32) -> Self {
        Self { x, y, z: 0.0 }
    }

    /// Get the altitude layer (discretized z for grid-based operations)
    #[must_use]
    pub fn altitude_layer(&self) -> AltitudeLayer {
        self.z.floor() as AltitudeLayer
    }

    /// Manhattan distance for grid-based movement (3D)
    #[must_use]
    pub fn distance(&self, other: &Self) -> f32 {
        (self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Manhattan distance in 2D (ignores z, for backward compatibility)
    #[must_use]
    pub fn distance_2d(&self, other: &Self) -> f32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    /// Euclidean distance for continuous space (3D)
    #[must_use]
    pub fn euclidean_distance(&self, other: &Self) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2))
            .sqrt()
    }

    /// Euclidean distance in 2D (ignores z, for backward compatibility)
    #[must_use]
    pub fn euclidean_distance_2d(&self, other: &Self) -> f32 {
        (self.x - other.x).hypot(self.y - other.y)
    }

    /// Calculate horizontal distance (x-y plane only)
    #[must_use]
    pub fn horizontal_distance(&self, other: &Self) -> f32 {
        self.euclidean_distance_2d(other)
    }

    /// Calculate vertical distance (z-axis only)
    #[must_use]
    pub fn vertical_distance(&self, other: &Self) -> f32 {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        (self.z - other.z).abs()
    }
}

/// 2D Vector for force calculations and spatial math
///
/// Used for Barnes-Hut social forces, movement calculations, and
/// grid-based physics simulations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector {
    /// x.
    pub x: f32,
    /// y.
    pub y: f32,
}

impl Vector {
    /// Create a new 2D vector
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Zero vector
    #[must_use]
    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Vector magnitude
    #[must_use]
    pub fn magnitude(&self) -> f32 {
        self.x.hypot(self.y)
    }

    /// Normalize to unit vector
    #[must_use]
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Self {
                x: self.x / mag,
                y: self.y / mag,
            }
        } else {
            Self::zero()
        }
    }

    /// Scale vector by scalar
    #[must_use]
    pub fn scale(&self, s: f32) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
        }
    }
}

impl std::ops::Add for Vector {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl std::ops::Sub for Vector {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl std::ops::AddAssign for Vector {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

/// Direction for agent movement (3D)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Direction {
    /// dx.
    pub dx: f32,
    /// dy.
    pub dy: f32,
    /// Vertical direction component (altitude change)
    pub dz: f32,
}

impl Direction {
    /// Create a new 3D direction
    #[must_use]
    pub const fn new(dx: f32, dy: f32, dz: f32) -> Self {
        Self { dx, dy, dz }
    }

    /// Create a new 2D direction (dz defaults to 0.0 for backward compatibility)
    #[must_use]
    pub const fn new_2d(dx: f32, dy: f32) -> Self {
        Self { dx, dy, dz: 0.0 }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Normalize to unit vector (3D)
    #[must_use]
    pub fn normalize(&self) -> Self {
        let magnitude = (self.dx.powi(2) + self.dy.powi(2) + self.dz.powi(2)).sqrt();
        if magnitude > 0.0 {
            Self {
                dx: self.dx / magnitude,
                dy: self.dy / magnitude,
                dz: self.dz / magnitude,
            }
        } else {
            Self {
                dx: 0.0,
                dy: 0.0,
                dz: 0.0,
            }
        }
    }

    /// Normalize to unit vector in 2D (dz set to 0)
    #[must_use]
    pub fn normalize_2d(&self) -> Self {
        let magnitude = self.dx.hypot(self.dy);
        if magnitude > 0.0 {
            Self {
                dx: self.dx / magnitude,
                dy: self.dy / magnitude,
                dz: 0.0,
            }
        } else {
            Self {
                dx: 0.0,
                dy: 0.0,
                dz: 0.0,
            }
        }
    }

    /// Get horizontal magnitude (x-y plane)
    #[must_use]
    pub fn horizontal_magnitude(&self) -> f32 {
        self.dx.hypot(self.dy)
    }

    /// Get vertical component (z-axis)
    #[must_use]
    pub const fn vertical_component(&self) -> f32 {
        self.dz
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if direction has vertical movement
    #[must_use]
    pub fn has_vertical(&self) -> bool {
        self.dz != 0.0
    }
}

/// Action outcome for memory buffer (per §7.5.1 line 3935)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionOutcome {
    /// success.
    Success,
    /// failure.
    Failure,
    /// neutral.
    Neutral,
}

/// Strategy type for game-theoretic behavior (per §7.5.1 line 3920)
/// Per §6.10 UTGT: TFT emergence expected at τ > 0.60
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum StrategyType {
    /// Always cooperate
    Cooperate,
    /// Always defect
    Defect,
    /// Tit-for-tat (per GTT-C5: ESS at equilibrium)
    TitForTat,
    /// Tit-for-tat with forgiveness (per §6.10.5)
    TitForTatForgive,
    /// Random choice
    Random,
    /// Prospect Theory: Evaluate choices using behavioral economics
    /// Per §6.10: Agents use prospect value to evaluate cooperate vs defect
    #[default]
    ProspectTheory,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Communication intent for language-based interactions
/// Phase 2.5: Used to select appropriate concepts from lexicon
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CommunicationIntent {
    /// Social bonding greeting
    #[default]
    Greeting,
    /// Warning or defensive threat
    Threat,
    /// Asking for resources or help
    Request,
    /// Signal trustworthiness
    ShareTrust,
    /// Pre-game negotiation
    Negotiate,
}

/// Unified claim type for identity, protocol, and society modules
///
/// Consolidates claims from:
/// - protocol.rs: Identity, `CooperationHistory`, `ResourceHolding`, `AttestationRecord`
/// - identity.rs: Reputation, Wealth, `SocialConnections`, `GroupMembership`, `CooperationHistory`, `IdentityVerification`
/// - society.rs: Identity, Skill, Transaction, Cooperation
///
/// This unified type eliminates duplication and provides a single source of truth
/// for all claim types in the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
pub enum ClaimType {
    // === Identity Layer Claims ===
    /// Identity verification claim (who the agent is)
    IdentityVerification,

    // === Society Layer Claims ===
    /// Reputation claim (social standing)
    Reputation,
    /// Cooperation history claim (past behavior)
    CooperationHistory,
    /// Social connections claim (network position)
    SocialConnections,
    /// Group membership claim (affiliation)
    GroupMembership,

    // === Resource Layer Claims ===
    /// Resource/wealth claim (current holdings)
    ResourceHolding,
    /// Financial wealth claim (derived from resources)
    Wealth,

    // === Protocol Layer Claims ===
    /// Attestation record claim (received attestations)
    AttestationRecord,
    /// Transaction claim (economic activity)
    Transaction,
    /// Skill claim (ability/capability)
    Skill,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl ClaimType {
    /// Get category of claim for grouping/UI purposes
    #[must_use]
    pub const fn category(&self) -> ClaimCategory {
        match self {
            Self::IdentityVerification => ClaimCategory::Identity,
            Self::Reputation
            | Self::CooperationHistory
            | Self::SocialConnections
            | Self::GroupMembership => ClaimCategory::Social,
            Self::ResourceHolding | Self::Wealth => ClaimCategory::Resource,
            Self::AttestationRecord | Self::Transaction | Self::Skill => ClaimCategory::Protocol,
        }
    }

    /// Check if this claim can be stolen during identity theft
    #[must_use]
    pub const fn is_stealable(&self) -> bool {
        matches!(
            self,
            Self::Reputation
                | Self::Wealth
                | Self::SocialConnections
                | Self::GroupMembership
                | Self::IdentityVerification
        )
    }

    /// Check if this claim can be anchored to blockchain
    #[must_use]
    pub const fn is_anchorable(&self) -> bool {
        matches!(
            self,
            Self::IdentityVerification
                | Self::CooperationHistory
                | Self::ResourceHolding
                | Self::AttestationRecord
                | Self::Reputation
                | Self::Wealth
        )
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Category of claim for grouping purposes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClaimCategory {
    /// identity.
    Identity,
    /// social.
    Social,
    /// resource.
    Resource,
    /// protocol.
    Protocol,
}

/// Sex enum for reproduction biology (per SIM-001)
/// Required for Stage 1: Biology reproduction pairing rules
/// Two-variant enum: Male, Female
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Sex {
    #[default]
    /// male.
    Male,
    /// female.
    Female,
}

impl Sex {
    /// Returns true if opposite sex (potential reproduction pair)
    #[must_use]
    pub fn can_reproduce_with(&self, other: &Self) -> bool {
        self != other
    }

    /// Random sex assignment with 50/50 probability
    pub fn random(rng: &mut impl rand::Rng) -> Self {
        if rng.gen::<bool>() {
            Self::Male
        } else {
            Self::Female
        }
    }
}

/// Cognitive system for DMN planning modes (per SIM-008)
/// Default planning horizon (ticks)
pub const H_DEFAULT: u32 = 5;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Per §6.2.1: Dual-process cognition (System 1 / System 2)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CognitiveSystem {
    /// Fast, heuristic-based, low energy cost (95-99% of agents)
    #[default]
    System1,
    /// Slow, analytical, high energy cost (1-5% of agents)
    System2,
}

impl CognitiveSystem {
    /// Energy cost per tick for this cognitive system
    /// Per §6.2.1: System 2 has metabolic cost proportional to planning depth
    #[must_use]
    pub fn energy_cost(&self, k_rollouts: u32, h_horizon: u32) -> f32 {
        match self {
            Self::System1 => 1.0, // Base cost
            Self::System2 => {
                // High-fidelity planning: base + (K × H × cost_per_rollout)
                // Per §6.2.1: cost_per_rollout = 0.5
                1.0 + (k_rollouts as f32 * h_horizon as f32 * 0.5)
            }
        }
    }

    /// Select cognitive system based on genome propensity and energy
    /// Per §6.2.1: Energy-gated, probabilistic selection
    pub fn select(
        system2_propensity: f32,
        energy: f32,
        energy_threshold: f32,
        rng: &mut impl rand::Rng,
    ) -> Self {
        // Cannot use System 2 if energy too low
        if energy < energy_threshold {
            return Self::System1;
        }

        // Probabilistic selection based on propensity (max 5% System 2)
        if rng.gen::<f32>() < system2_propensity * 0.05 {
            Self::System2
        } else {
            Self::System1
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Debt record for tracking obligations between agents (per SIM-002)
///
/// Required for Stage 7: Economy debt mechanics. Tracks creditor-debtor
/// relationships with creation time, due dates, and settlement status.
///
/// ## Fields
///
/// - `creditor_id`: Agent who is owed the debt
/// - `amount`: Energy units owed
/// - `tick_created`: When the debt was created
/// - `tick_due`: Optional deadline for repayment
/// - `settled`: Whether debt has been paid
/// - `tick_settled`: When settlement occurred
/// - `paid_in_full`: Whether full amount was paid
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DebtRecord {
    /// Creditor agent ID (who is owed)
    pub creditor_id: AgentId,
    /// Debt amount (energy units)
    pub amount: f32,
    /// Tick when debt was created
    pub tick_created: Tick,
    /// Tick when debt is due (None = no deadline)
    pub tick_due: Option<Tick>,
    /// Whether debt has been settled
    pub settled: bool,
    /// Tick when settled (None if not settled)
    pub tick_settled: Option<Tick>,
    /// Whether creditor was paid in full
    pub paid_in_full: bool,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl DebtRecord {
    /// Create a new debt record
    #[must_use]
    pub const fn new(
        creditor_id: AgentId,
        amount: f32,
        tick_created: Tick,
        tick_due: Option<Tick>,
    ) -> Self {
        Self {
            creditor_id,
            amount,
            tick_created,
            tick_due,
            settled: false,
            tick_settled: None,
            paid_in_full: false,
        }
    }

    /// Mark debt as settled
    pub fn settle(&mut self, tick: Tick, paid_in_full: bool) {
        self.settled = true;
        self.tick_settled = Some(tick);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        self.paid_in_full = paid_in_full;
    }

    /// Check if debt is overdue
    #[must_use]
    pub fn is_overdue(&self, current_tick: Tick) -> bool {
        if self.settled {
            return false;
        }
        self.tick_due.is_some_and(|due| current_tick > due)
    }

    /// Get remaining amount (full amount if not settled, 0 if settled)
    #[must_use]
    pub const fn remaining_amount(&self) -> f32 {
        if self.settled {
            0.0
        } else {
            self.amount
        }
    }

    /// Check if debt is fully paid
    #[must_use]
    pub const fn is_fully_paid(&self) -> bool {
        self.settled && self.paid_in_full
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Record a partial or full payment
    /// Returns true if payment was successful
    pub fn record_payment(&mut self, amount: f32) -> bool {
        if self.settled {
            return false;
        }

        // Note: This is a simplified implementation
        // In a full implementation, we'd track partial payments
        // For now, any payment settles the debt
        if amount >= self.amount {
            self.settled = true;
            self.paid_in_full = true;
            true
        } else {
            // Partial payment - track remaining (simplified)
            // In full implementation, add to partial_payments vector
            false
        }
    }
}

/// Debt settlement decision outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebtDecision {
    /// Agent chooses to pay the debt
    Settle,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Agent chooses to renege (default)
    Renege,
    /// Agent requests extension
    RequestExtension,
}

/// DOCAS component indices (per §6.1 lines 617-660)
/// D=Dopamine, O=Oxytocin, C=Cortisol, A=Adrenaline, S=Serotonin
pub const D: usize = 0; // Dopamine - Expectation
/// o.
pub const O: usize = 1; // Oxytocin - Social Salience
/// c.
pub const C: usize = 2; // Cortisol - Recognition/Threat
/// a.
pub const A: usize = 3; // Adrenaline - Action
/// s.
pub const S: usize = 4; // Serotonin - Stabilization

/// Number of DOCAS components
pub const DOCAS_SIZE: usize = 5;

/// Number of genome loci (per §6.6.5: 44 loci for 22 traits including PDE4B)
/// Extended to 48 loci for 24 traits including neural maturation
pub const GENOME_LOCI: usize = 48;

/// Number of traits (per §6.6.5: 22 traits including PDE4B)
/// Extended to 24 traits including neural maturation controls
pub const TRAIT_COUNT: usize = 24;

/// Temperature modifier per altitude level
/// Higher altitude = lower temperature
pub const ALTITUDE_TEMPERATURE_MODIFIER: f32 = -0.5; // degrees per unit altitude

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Base temperature at sea level (z = 0)
pub const BASE_TEMPERATURE: f32 = 20.0; // degrees Celsius

/// Movement cost multiplier for vertical movement
/// Going up costs more energy than going down
pub const VERTICAL_MOVEMENT_COST_FACTOR: f32 = 1.5;

/// Resource availability modifier per altitude
/// Higher altitude = different resource distribution
pub const ALTITUDE_RESOURCE_MODIFIER: f32 = 0.8; // resources decrease with altitude

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_position_distance_3d() {
        let p1 = Position::new(0.0, 0.0, 0.0);
        let p2 = Position::new(3.0, 4.0, 0.0);
        // 3D Manhattan distance
        assert_eq!(p1.distance(&p2), 7.0);
        // 3D Euclidean distance (same as 2D when z=0)
        assert!((p1.euclidean_distance(&p2) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_position_distance_2d_backward_compat() {
        let p1 = Position::new_2d(0.0, 0.0);
        let p2 = Position::new_2d(3.0, 4.0);
        // 2D Manhattan distance
        assert_eq!(p1.distance_2d(&p2), 7.0);
        // 2D Euclidean distance
        assert!((p1.euclidean_distance_2d(&p2) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_position_distance_full_3d() {
        let p1 = Position::new(0.0, 0.0, 0.0);
        let p2 = Position::new(1.0, 2.0, 2.0);
        // 3D Manhattan: |1| + |2| + |2| = 5
        assert_eq!(p1.distance(&p2), 5.0);
        // 3D Euclidean: sqrt(1 + 4 + 4) = sqrt(9) = 3
        assert!((p1.euclidean_distance(&p2) - 3.0).abs() < 0.001);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_position_altitude_layer() {
        let p1 = Position::new(10.0, 20.0, 3.7);
        assert_eq!(p1.altitude_layer(), 3);

        // Test with ground level
        let p2 = Position::new(10.0, 20.0, 0.0);
        assert_eq!(p2.altitude_layer(), 0);

        // Negative z floors to 0 for usize compatibility
        let p3 = Position::new(10.0, 20.0, -1.2);
        // Negative floats cast to usize wrap around (implementation defined)
        // We just verify it doesn't panic
        let _layer = p3.altitude_layer();
    }

    #[test]
    fn test_position_vertical_horizontal_distance() {
        let p1 = Position::new(0.0, 0.0, 0.0);
        let p2 = Position::new(3.0, 4.0, 10.0);

        // Horizontal distance (x-y plane)
        assert!((p1.horizontal_distance(&p2) - 5.0).abs() < 0.001);

        // Vertical distance (z-axis)
        assert!((p1.vertical_distance(&p2) - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_direction_normalize_3d() {
        let d = Direction::new(1.0, 2.0, 2.0);
        let n = d.normalize();
        // Magnitude should be 1: sqrt(1 + 4 + 4) = 3
        let magnitude = (n.dx.powi(2) + n.dy.powi(2) + n.dz.powi(2)).sqrt();
        assert!((magnitude - 1.0).abs() < 0.001);
        // Components should be 1/3, 2/3, 2/3
        assert!((n.dx - 1.0 / 3.0).abs() < 0.001);
        assert!((n.dy - 2.0 / 3.0).abs() < 0.001);
        assert!((n.dz - 2.0 / 3.0).abs() < 0.001);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_direction_normalize_2d() {
        let d = Direction::new_2d(3.0, 4.0);
        let n = d.normalize();
        // Magnitude should be 1
        let magnitude = (n.dx.powi(2) + n.dy.powi(2) + n.dz.powi(2)).sqrt();
        assert!((magnitude - 1.0).abs() < 0.001);
        // dz should be 0 for normalized 2D
        assert_eq!(n.dz, 0.0);
    }

    #[test]
    fn test_direction_normalize_2d_variant() {
        let d = Direction::new(3.0, 4.0, 0.0);
        let n = d.normalize_2d();
        // dz should be explicitly set to 0
        assert_eq!(n.dz, 0.0);
        // x,y should be normalized
        assert!((n.dx - 0.6).abs() < 0.001);
        assert!((n.dy - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_direction_horizontal_vertical_components() {
        let d = Direction::new(3.0, 4.0, 5.0);
        assert!((d.horizontal_magnitude() - 5.0).abs() < 0.001);
        assert!((d.vertical_component() - 5.0).abs() < 0.001);
        assert!(d.has_vertical());

        let d2 = Direction::new_2d(3.0, 4.0);
        assert!((d2.horizontal_magnitude() - 5.0).abs() < 0.001);
        assert_eq!(d2.vertical_component(), 0.0);
        assert!(!d2.has_vertical());
    }

    #[test]
    fn test_backward_compat_new_2d() {
        // Ensure 2D API works (z defaults to 0)
        let p = Position::new_2d(10.0, 20.0);
        assert_eq!(p.x, 10.0);
        assert_eq!(p.y, 20.0);
        assert_eq!(p.z, 0.0);

        let d = Direction::new_2d(1.0, 0.0);
        assert_eq!(d.dx, 1.0);
        assert_eq!(d.dy, 0.0);
        assert_eq!(d.dz, 0.0);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_sex_enum_variants() {
        assert_ne!(Sex::Male, Sex::Female);
        assert_eq!(Sex::Male, Sex::Male);
        assert_eq!(Sex::Female, Sex::Female);
    }

    #[test]
    fn test_sex_can_reproduce_with() {
        assert!(Sex::Male.can_reproduce_with(&Sex::Female));
        assert!(Sex::Female.can_reproduce_with(&Sex::Male));
        assert!(!Sex::Male.can_reproduce_with(&Sex::Male));
        assert!(!Sex::Female.can_reproduce_with(&Sex::Female));
    }

    #[test]
    fn test_sex_random_distribution() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut male_count = 0;
        let mut female_count = 0;

        for _ in 0..1000 {
            match Sex::random(&mut rng) {
                Sex::Male => male_count += 1,
                Sex::Female => female_count += 1,
            }
        }

        // With 1000 samples, should be roughly 50/50 (allowing 40-60% range)
        assert!(
            male_count > 400 && male_count < 600,
            "Male count {male_count} should be roughly 50%"
        );
        assert!(
            female_count > 400 && female_count < 600,
            "Female count {female_count} should be roughly 50%"
        );
    }

    // === DebtRecord Tests (SIM-002) ===

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_debt_record_creation() {
        let debt = DebtRecord::new(1, 10.0, 100, Some(200));

        assert_eq!(debt.creditor_id, 1);
        assert_eq!(debt.amount, 10.0);
        assert_eq!(debt.tick_created, 100);
        assert_eq!(debt.tick_due, Some(200));
        assert!(!debt.settled);
        assert_eq!(debt.tick_settled, None);
        assert!(!debt.paid_in_full);
    }

    #[test]
    fn test_debt_record_settle() {
        let mut debt = DebtRecord::new(1, 10.0, 100, Some(200));

        debt.settle(150, true);

        assert!(debt.settled);
        assert_eq!(debt.tick_settled, Some(150));
        assert!(debt.paid_in_full);
    }

    #[test]
    fn test_debt_record_is_overdue() {
        let debt = DebtRecord::new(1, 10.0, 100, Some(200));

        assert!(!debt.is_overdue(100)); // Same tick as created
        assert!(!debt.is_overdue(200)); // Exactly at due date
        assert!(debt.is_overdue(201)); // Past due date
        assert!(debt.is_overdue(250)); // Well past due date

        // Settled debt is never overdue
        let mut settled_debt = DebtRecord::new(1, 10.0, 100, Some(200));
        settled_debt.settle(150, true);
        assert!(!settled_debt.is_overdue(300));
    }

    #[test]
    fn test_debt_record_remaining_amount() {
        let mut debt = DebtRecord::new(1, 10.0, 100, None);

        assert_eq!(debt.remaining_amount(), 10.0);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        debt.settle(150, true);
        assert_eq!(debt.remaining_amount(), 0.0);
    }

    #[test]
    fn test_debt_record_no_due_date() {
        let debt = DebtRecord::new(1, 10.0, 100, None);

        assert!(!debt.is_overdue(1000)); // Never overdue without due date
        assert_eq!(debt.remaining_amount(), 10.0);
    }

    // === 3D Movement Tests ===

    #[test]
    fn test_3d_movement_calculation() {
        // Agent moving diagonally in 3D space
        let start = Position::new(0.0, 0.0, 0.0);
        let end = Position::new(3.0, 4.0, 5.0);

        // Total 3D distance
        let distance = start.euclidean_distance(&end);
        // sqrt(9 + 16 + 25) = sqrt(50) ≈ 7.07
        assert!((distance - 7.071).abs() < 0.01);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // Horizontal only
        let horizontal = start.horizontal_distance(&end);
        assert!((horizontal - 5.0).abs() < 0.001); // 3-4-5 triangle

        // Vertical only
        let vertical = start.vertical_distance(&end);
        assert!((vertical - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_direction_3d_movement() {
        // Moving up and northeast
        let dir = Direction::new(1.0, 1.0, 1.0);
        let normalized = dir.normalize();

        // All components should be equal (1/sqrt(3))
        let expected = 1.0 / (3.0_f32).sqrt();
        assert!((normalized.dx - expected).abs() < 0.001);
        assert!((normalized.dy - expected).abs() < 0.001);
        assert!((normalized.dz - expected).abs() < 0.001);
    }

    #[test]
    fn test_altitude_constants() {
        // Verify altitude-related constants are defined
        assert_eq!(ALTITUDE_TEMPERATURE_MODIFIER, -0.5);
        assert_eq!(BASE_TEMPERATURE, 20.0);
        assert_eq!(VERTICAL_MOVEMENT_COST_FACTOR, 1.5);
        assert_eq!(ALTITUDE_RESOURCE_MODIFIER, 0.8);
    }
}
