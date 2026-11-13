//! Glial Cell Modeling Module
//!
//! Implements astrocytes and oligodendrocytes as active modulators of neural function.
//! Glial cells were traditionally thought of as "glue" but are now known to actively
//! modulate synaptic transmission, energy metabolism, and signal conduction.
// TAG: surface=simulation owner=research-team rule=SIM-001
//!
//! # Key Concepts
//!
//! ## Astrocytes
//! - Regulate synaptic transmission via tripartite synapse (presynaptic + postsynaptic + astrocyte)
//! - Control glutamate clearance from synaptic cleft
//! - Release gliotransmitters (ATP, D-serine) that modulate synaptic strength
//! - Calcium waves propagate signaling across neural networks
//!
//! ## Oligodendrocytes
//! - Form myelin sheaths around axons
//! - Myelination affects conduction velocity (0.5-100 m/s range)
//! - Activity-dependent myelination (Fields 2008)
//! - Thicker myelin = faster conduction
//!
//! # Research References
//! - Volterra & Meldolesi (2005) - Astrocytes as communication elements
//! - Fields (2008) - White matter in learning and cognition
//! - Araque et al. (2014) - Tripartite synapse concept

use crate::types::Position;
use serde::{Deserialize, Serialize};

/// Re-export types for use within the module
pub type NeuralPosition = Position;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Unique identifier for axon segments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AxonSegmentId(pub u64);

impl AxonSegmentId {
    /// Create new axon segment ID
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Activity history for myelination updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityHistory {
    /// Activity samples per segment
    pub segment_activity: std::collections::HashMap<AxonSegmentId, Vec<f32>>,
    /// Maximum history length
    pub max_history: usize,
    /// Current tick
    pub current_tick: u64,
}

impl ActivityHistory {
    /// Create new activity history
    #[must_use]
    pub fn new(max_history: usize) -> Self {
        Self {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            segment_activity: std::collections::HashMap::new(),
            max_history,
            current_tick: 0,
        }
    }

    /// Record activity for a segment
    pub fn record(&mut self, segment: AxonSegmentId, activity: f32) {
        let entries = self.segment_activity.entry(segment).or_default();
        entries.push(activity);
        if entries.len() > self.max_history {
            entries.remove(0);
        }
        if self.current_tick == 0 && !entries.is_empty() {
            self.current_tick = 1;
        }
    }

    /// Get average activity for a single segment
    #[must_use]
    pub fn average_for_segment(&self, segment: &AxonSegmentId) -> f32 {
        if let Some(activities) = self.segment_activity.get(segment) {
            if activities.is_empty() {
                return 0.0;
            }
            activities.iter().sum::<f32>() / activities.len() as f32
        } else {
            0.0
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get average activity across multiple segments
    #[must_use]
    pub fn average_for_segments(&self, segments: &[AxonSegmentId]) -> f32 {
        if segments.is_empty() {
            return 0.0;
        }
        let sum: f32 = segments.iter().map(|s| self.average_for_segment(s)).sum();
        sum / segments.len() as f32
    }

    /// Advance to next tick
    pub fn tick(&mut self) {
        self.current_tick += 1;
    }
}

impl Default for ActivityHistory {
    fn default() -> Self {
        Self::new(1000) // Default: 1000 ticks of history
    }
}

/// Axon type for conduction velocity calculation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AxonType {
    /// Myelinated axons with saltatory conduction (fast)
    #[default]
    Myelinated,
    /// Unmyelinated axons with continuous conduction (slow)
    Unmyelinated,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Astrocyte regulating synaptic environment
///
/// Astrocytes are star-shaped glial cells that:
/// - Regulate extracellular ion concentrations
/// - Control neurotransmitter uptake (especially glutamate)
/// - Release gliotransmitters that modulate synaptic transmission
/// - Participate in tripartite synapses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Astrocyte {
    /// Spatial position in neural tissue
    pub position: NeuralPosition,
    /// Calcium activity level [0, 1] - propagates as waves for signaling
    pub calcium_activity: f32,
    /// Glutamate uptake rate via GLT-1 transporters [0, 1]
    pub glutamate_uptake: f32,
    /// Gliotransmitter release level (ATP, D-serine) [0, 1]
    /// Modulates synaptic strength in nearby synapses
    pub gliotransmitter_release: f32,
    /// Spatial coverage radius for synaptic modulation
    pub coverage_radius: f32,
    /// Energy available for astrocyte function (lactate production)
    pub energy_level: f32,
    /// Unique identifier
    pub id: u64,
}

impl Astrocyte {
    /// Create a new astrocyte at the given position
    #[must_use]
    pub const fn new(id: u64, position: NeuralPosition, coverage_radius: f32) -> Self {
        Self {
            id,
            position,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            calcium_activity: 0.1, // Baseline calcium level
            glutamate_uptake: 0.5, // Normal uptake
            gliotransmitter_release: 0.0,
            coverage_radius,
            energy_level: 1.0,
        }
    }

    /// Update calcium dynamics based on local neural activity
    ///
    /// Calcium waves in astrocytes are triggered by:
    /// - Neuronal activity (glutamate release)
    /// - Gap junction coupling with other astrocytes
    /// - Intrinsic calcium oscillations
    pub fn update_calcium(&mut self, local_activity: f32, dt: f32) {
        // Calcium dynamics: leaky integration with activity-driven input
        let decay = 0.9f32.max(0.0).powf(dt);
        let input = local_activity * 0.1 * dt;

        self.calcium_activity = (self.calcium_activity * decay + input).clamp(0.0, 1.0);

        // Update glutamate uptake based on activity
        // Higher activity = more uptake needed
        self.glutamate_uptake = (local_activity * 0.5 + 0.3).clamp(0.0, 1.0);

        // Gliotransmitter release triggered by high calcium
        // Threshold-based release (typically > 0.6)
        if self.calcium_activity > 0.6 {
            self.gliotransmitter_release =
                ((self.calcium_activity - 0.6) * 0.5 * dt).clamp(0.0, 1.0);
        } else {
            self.gliotransmitter_release = (self.gliotransmitter_release * decay).clamp(0.0, 1.0);
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate synaptic modulation factor for a given position
    ///
    /// Returns modulation multiplier [0.5, 1.5] where:
    /// - > 1.0 = synaptic potentiation (enhanced transmission)
    /// - < 1.0 = synaptic depression (reduced transmission)
    /// - = 1.0 = no modulation
    #[must_use]
    pub fn modulate_synaptic_strength(
        &self,
        synapse_position: &NeuralPosition,
        base_strength: f32,
    ) -> f32 {
        if self.coverage_radius <= 0.0 {
            return base_strength;
        }
        let distance = self.position.euclidean_distance(synapse_position);

        if distance > self.coverage_radius {
            // Outside coverage - no modulation
            return base_strength;
        }

        // Spatial falloff (gaussian-like)
        let spatial_factor = (-distance.powi(2) / (2.0 * self.coverage_radius.powi(2))).exp();

        // Gliotransmitter effect: D-serine enhances NMDA receptor function
        // ATP can have various effects depending on receptor type
        let modulation = 1.0 + self.gliotransmitter_release * spatial_factor * 0.3;

        base_strength * modulation.clamp(0.5, 1.5)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Consume energy to support neural function
    ///
    /// Astrocytes provide lactate to neurons as energy substrate
    /// Returns the amount of lactate supplied
    pub fn supply_energy(&mut self, demand: f32) -> f32 {
        let uptake_factor = (self.glutamate_uptake * 0.5 + 0.5).clamp(0.1, 1.0);
        let effective_level = self.energy_level * uptake_factor;
        let supply = demand.min(effective_level);
        self.energy_level -= supply * 0.1; // Partial depletion
        self.energy_level = self.energy_level.clamp(0.0, 1.0);
        supply
    }

    /// Replenish energy stores
    pub fn replenish_energy(&mut self, amount: f32) {
        self.energy_level = (self.energy_level + amount).clamp(0.0, 1.0);
    }
}

impl Default for Astrocyte {
    fn default() -> Self {
        Self::new(0, NeuralPosition::new_2d(0.0, 0.0), 5.0)
    }
}

/// Oligodendrocyte producing myelin
///
/// Oligodendrocytes are myelin-producing cells that:
/// - Wrap axons in myelin sheaths (insulation)
/// - Enable saltatory conduction (fast signal propagation)
/// - Form nodes of Ranvier (gaps in myelin)
/// - Modulate conduction velocity based on myelination level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Oligodendrocyte {
    /// Axon segments myelinated by this cell
    pub axon_segments: Vec<AxonSegmentId>,
    /// Myelin thickness relative to optimal [0, 1]
    /// Thicker myelin = faster conduction
    pub myelin_thickness: f32,
    /// Distance between nodes of Ranvier (internodal distance)
    pub node_spacing: f32,
    /// Current conduction velocity in m/s [0.5, 100]
    pub conduction_velocity: f32,
    /// Unique identifier
    pub id: u64,
    /// Age of myelination (ticks)
    pub age: u64,
    /// Activity history for adaptive myelination
    pub activity_level: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Oligodendrocyte {
    /// Create a new oligodendrocyte for given axon segments
    #[must_use]
    pub fn new(id: u64, axon_segments: Vec<AxonSegmentId>) -> Self {
        let initial_thickness = 0.1; // Start with thin myelin
        let initial_velocity = Self::compute_velocity(initial_thickness);

        Self {
            id,
            axon_segments,
            myelin_thickness: initial_thickness,
            node_spacing: 100.0, // micrometers
            conduction_velocity: initial_velocity,
            age: 0,
            activity_level: 0.0,
        }
    }

    /// Compute conduction velocity from myelin thickness
    ///
    /// Formula: v = v0 + k * sqrt(thickness)
    /// where v0 = 0.5 m/s (unmyelinated baseline)
    /// and k = 10.0 (empirical scaling factor)
    #[must_use]
    pub fn compute_velocity(thickness: f32) -> f32 {
        let v0 = 0.5; // m/s - unmyelinated baseline
                      // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let k = 10.0; // scaling factor
        let velocity = v0 + k * thickness.max(0.0).sqrt();
        velocity.clamp(0.5, 100.0) // Physiological range
    }

    /// Update myelination based on activity history
    ///
    /// Activity-dependent myelination (Fields 2008):
    /// - High activity promotes myelination
    /// - Myelination increases conduction velocity
    /// - This creates a positive feedback loop for frequently used pathways
    pub fn update_myelination(&mut self, avg_activity: f32, dt: f32) {
        self.activity_level = avg_activity;

        // Activity threshold for myelination growth
        const ACTIVITY_THRESHOLD: f32 = 0.3;
        const MAX_THICKNESS: f32 = 1.0;
        const GROWTH_RATE: f32 = 0.01;

        if avg_activity > ACTIVITY_THRESHOLD && self.myelin_thickness < MAX_THICKNESS {
            // Activity promotes myelination
            let growth = GROWTH_RATE * avg_activity * dt;
            self.myelin_thickness = (self.myelin_thickness + growth).min(MAX_THICKNESS);
        }

        // Update conduction velocity based on new thickness
        self.conduction_velocity = Self::compute_velocity(self.myelin_thickness);
        self.age += 1;
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate conduction delay for a given distance
    ///
    /// For myelinated axons: delay = distance / velocity
    /// For unmyelinated: much slower conduction
    #[must_use]
    pub fn conduction_delay(&self, distance: f32) -> f32 {
        if self.conduction_velocity <= 0.0 {
            return 0.0;
        }
        // distance in micrometers, velocity in m/s
        // Convert: 1 um = 1e-6 m
        let distance_meters = distance.max(0.0) * 1e-6;
        distance_meters / self.conduction_velocity
    }

    /// Calculate effective conduction delay considering node spacing
    ///
    /// Saltatory conduction: signal jumps between nodes of Ranvier
    /// This is much faster than continuous conduction
    #[must_use]
    pub fn saltatory_delay(&self, distance: f32) -> f32 {
        if self.node_spacing <= 0.0 || self.conduction_velocity <= 0.0 {
            return self.conduction_delay(distance);
        }
        let distance = distance.max(0.0);

        // Number of nodes crossed
        let num_nodes = (distance / self.node_spacing).floor() as u32;

        // Delay at each node (very short - nanosecond scale)
        let node_delay = 1e-9; // 1 nanosecond per node

        // Propagation time between nodes
        let internode_time = self.node_spacing * 1e-6 / self.conduction_velocity;

        // Total delay
        (num_nodes as f32 * node_delay) + (distance / self.node_spacing) * internode_time
    }

    /// Get myelination efficiency score [0, 1]
    #[must_use]
    pub const fn myelination_efficiency(&self) -> f32 {
        self.myelin_thickness
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if this oligodendrocyte myelinates a specific segment
    #[must_use]
    pub fn has_segment(&self, segment: &AxonSegmentId) -> bool {
        self.axon_segments.contains(segment)
    }
}

impl Default for Oligodendrocyte {
    fn default() -> Self {
        Self::new(0, vec![])
    }
}

/// Glial cell system for neural modulation
///
/// The glial system manages astrocytes and oligodendrocytes that:
/// - Modulate synaptic transmission
/// - Regulate energy metabolism
/// - Control conduction velocity
/// - Affect trust decision timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlialSystem {
    /// Astrocyte population regulating synaptic environment
    pub astrocytes: Vec<Astrocyte>,
    /// Oligodendrocyte population producing myelin
    pub oligodendrocytes: Vec<Oligodendrocyte>,
    /// Global glutamate clearance rate (astrocyte reuptake efficiency)
    pub glutamate_clearance: f32,
    /// Lactate supply rate to neurons (energy metabolism)
    pub lactate_supply: f32,
    /// Global myelination level affecting conduction velocity
    pub myelination_level: f32,
    /// Activity history for adaptive myelination
    pub activity_history: ActivityHistory,
    /// Next unique ID for new cells
    next_id: u64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl GlialSystem {
    /// Create a new glial system with specified populations
    #[must_use]
    pub fn new(n_astrocytes: usize, n_oligodendrocytes: usize, world_size: f32) -> Self {
        let mut system = Self {
            astrocytes: Vec::with_capacity(n_astrocytes),
            oligodendrocytes: Vec::with_capacity(n_oligodendrocytes),
            glutamate_clearance: 0.5,
            lactate_supply: 1.0,
            myelination_level: 0.1,
            activity_history: ActivityHistory::default(),
            next_id: 0,
        };

        // Initialize astrocytes with distributed positions
        for i in 0..n_astrocytes {
            let x = (i as f32 / n_astrocytes as f32) * world_size;
            let y = (i * 7) as f32 % world_size; // Pseudo-random distribution
            let position = NeuralPosition::new_2d(x, y);
            let astrocyte = Astrocyte::new(
                system.next_id(),
                position,
                world_size / (n_astrocytes as f32).sqrt(), // Coverage based on density
            );
            system.astrocytes.push(astrocyte);
        }

        // Initialize oligodendrocytes
        for i in 0..n_oligodendrocytes {
            let segments = vec![AxonSegmentId::new(i as u64)];
            let oligodendrocyte = Oligodendrocyte::new(system.next_id(), segments);
            system.oligodendrocytes.push(oligodendrocyte);
        }

        system
    }

    /// Generate next unique ID
    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update astrocyte calcium signaling
    ///
    /// Tripartite synapse: astrocytes respond to neuronal activity
    /// and modulate synaptic transmission through gliotransmitter release
    pub fn update_astrocytes(&mut self, neural_activity: &[f32], dt: f32) {
        if self.astrocytes.is_empty() {
            self.lactate_supply = 0.0;
            return;
        }
        for (i, astrocyte) in self.astrocytes.iter_mut().enumerate() {
            // Sample local activity (cycling through activity array)
            let activity = if neural_activity.is_empty() {
                0.0
            } else {
                neural_activity[i % neural_activity.len()]
            };

            // Update calcium dynamics
            astrocyte.update_calcium(activity, dt);

            // Update global clearance based on collective uptake
            self.glutamate_clearance = (self.glutamate_clearance * 0.99
                + astrocyte.glutamate_uptake * 0.01)
                .clamp(0.0, 1.0);
        }

        // Update lactate supply based on collective energy production
        let total_energy: f32 = self.astrocytes.iter().map(|a| a.energy_level).sum();
        self.lactate_supply = (total_energy / self.astrocytes.len() as f32).clamp(0.0, 2.0);
    }

    /// Compute effective synaptic strength with astrocyte modulation
    ///
    /// This implements the tripartite synapse concept where astrocytes
    /// actively modulate synaptic transmission
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[must_use]
    pub fn modulate_synaptic_strength(&self, base_strength: f32, position: &NeuralPosition) -> f32 {
        if let Some(astro) = self.find_nearby_astrocyte(position) {
            astro.modulate_synaptic_strength(position, base_strength)
        } else {
            base_strength
        }
    }

    /// Find the nearest astrocyte to a given position
    fn find_nearby_astrocyte(&self, position: &NeuralPosition) -> Option<&Astrocyte> {
        self.astrocytes
            .iter()
            .min_by(|a, b| {
                let da = a.position.euclidean_distance(position);
                let db = b.position.euclidean_distance(position);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .filter(|a| a.position.euclidean_distance(position) <= a.coverage_radius)
    }

    /// Find the nearest mutable astrocyte
    #[allow(dead_code)]
    fn find_nearby_astrocyte_mut(&mut self, position: &NeuralPosition) -> Option<&mut Astrocyte> {
        let mut min_dist = f32::INFINITY;
        let mut idx = None;

        for (i, a) in self.astrocytes.iter().enumerate() {
            let dist = a.position.euclidean_distance(position);
            if dist < min_dist && dist <= a.coverage_radius {
                min_dist = dist;
                idx = Some(i);
            }
        }

        idx.map(|i| &mut self.astrocytes[i])
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update myelination based on neural activity patterns
    ///
    /// Activity-dependent myelination (Fields 2008):
    /// High activity promotes myelination, increasing conduction velocity
    pub fn update_myelination(&mut self, dt: f32) {
        let mut total_myelination = 0.0;
        // Modulate myelination rate by astrocytic lactate availability (Fields 2008)
        let effective_dt = dt * (0.5 + 0.5 * self.lactate_supply.clamp(0.0, 2.0));

        for oligodendrocyte in &mut self.oligodendrocytes {
            // Get average activity for myelinated segments
            let avg_activity = self
                .activity_history
                .average_for_segments(&oligodendrocyte.axon_segments);

            // Update myelination
            oligodendrocyte.update_myelination(avg_activity, effective_dt);

            total_myelination += oligodendrocyte.myelin_thickness;
        }

        // Update global myelination level
        if !self.oligodendrocytes.is_empty() {
            self.myelination_level =
                (total_myelination / self.oligodendrocytes.len() as f32).clamp(0.0, 1.0);
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute effective conduction delay with myelination
    ///
    /// Returns delay in seconds for signal propagation
    #[must_use]
    pub fn conduction_delay(&self, distance: f32, axon_type: AxonType) -> f32 {
        match axon_type {
            AxonType::Myelinated => {
                // Myelinated: saltatory conduction, very fast
                let velocity = self.average_myelin_velocity();
                let distance_meters = distance * 1e-6; // Convert um to m
                distance_meters / velocity
            }
            AxonType::Unmyelinated => {
                // Unmyelinated: slow, continuous conduction (0.5 m/s)
                let distance_meters = distance * 1e-6;
                distance_meters / 0.5
            }
        }
    }

    /// Get average myelin conduction velocity across all oligodendrocytes
    #[must_use]
    pub fn average_myelin_velocity(&self) -> f32 {
        if self.oligodendrocytes.is_empty() {
            return 0.5; // Default unmyelinated velocity
        }

        let sum: f32 = self
            .oligodendrocytes
            .iter()
            .map(|o| o.conduction_velocity)
            .sum();
        sum / self.oligodendrocytes.len() as f32
    }

    /// Record activity for myelination updates
    pub fn record_activity(&mut self, segment: AxonSegmentId, activity: f32) {
        self.activity_history.record(segment, activity);
    }

    /// Advance simulation tick and synchronize metabolic state
    pub fn tick(&mut self) {
        self.activity_history.tick();
        if !self.astrocytes.is_empty() {
            self.lactate_supply = (self.lactate_supply * 0.99
                + (self.total_energy_supply() / self.astrocytes.len() as f32) * 0.01)
                .clamp(0.0, 2.0);
        }
        if !self.oligodendrocytes.is_empty() {
            let total_m: f32 = self
                .oligodendrocytes
                .iter()
                .map(|o| o.myelin_thickness)
                .sum();
            self.myelination_level = (total_m / self.oligodendrocytes.len() as f32).clamp(0.0, 1.0);
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get total energy supply from astrocytes (lactate)
    #[must_use]
    pub fn total_energy_supply(&self) -> f32 {
        self.astrocytes.iter().map(|a| a.energy_level).sum()
    }

    /// Add a new astrocyte at the specified position
    pub fn add_astrocyte(&mut self, position: NeuralPosition, coverage_radius: f32) -> u64 {
        let id = self.next_id();
        let astrocyte = Astrocyte::new(id, position, coverage_radius);
        self.astrocytes.push(astrocyte);
        id
    }

    /// Add a new oligodendrocyte for given axon segments
    #[must_use]
    pub fn add_oligodendrocyte(&mut self, segments: Vec<AxonSegmentId>) -> u64 {
        let id = self.next_id();
        let oligodendrocyte = Oligodendrocyte::new(id, segments);
        self.oligodendrocytes.push(oligodendrocyte);
        id
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    /// Get system statistics
    #[must_use]
    pub fn statistics(&self) -> GlialStatistics {
        let n_astro = self.astrocytes.len();
        let n_oligo = self.oligodendrocytes.len();

        let avg_calcium = if n_astro > 0 {
            self.astrocytes
                .iter()
                .map(|a| a.calcium_activity)
                .sum::<f32>()
                / n_astro as f32
        } else {
            0.0
        };

        let avg_velocity = self.average_myelin_velocity();

        GlialStatistics {
            astrocyte_count: n_astro,
            oligodendrocyte_count: n_oligo,
            average_calcium_activity: avg_calcium,
            average_conduction_velocity: avg_velocity,
            myelination_level: self.myelination_level,
            glutamate_clearance: self.glutamate_clearance,
            lactate_supply: self.lactate_supply,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute trust decision timing modifier
    ///
    /// Myelination affects how quickly trust decisions can be made
    /// by modulating signal propagation speed
    #[must_use]
    pub fn trust_timing_modifier(&self) -> f32 {
        // Faster conduction = faster decisions
        // Normalized around 1.0 (baseline myelination)
        let baseline_velocity = 10.0; // m/s
        let current_velocity = self.average_myelin_velocity();
        (baseline_velocity / current_velocity.max(0.5)).clamp(0.5, 2.0)
    }

    /// Apply DOCAS dynamics modulation
    ///
    /// Glial cells modulate DOCAS dynamics by:
    /// - Astrocytes modulating synaptic plasticity
    /// - Energy availability affecting neural excitability
    pub fn modulate_docas(&self, docas_levels: &mut [f32; 5], dt: f32) {
        // Dopamine modulation via astrocyte glutamate clearance
        // Better clearance = cleaner dopamine signals
        let dopamine_mod = 1.0 + (self.glutamate_clearance - 0.5) * 0.2 * dt;
        docas_levels[0] *= dopamine_mod;

        // Serotonin stabilization via gliotransmitter modulation
        let serotonin_mod = 1.0 + (self.lactate_supply - 1.0) * 0.1 * dt;
        docas_levels[4] *= serotonin_mod;

        // Clamp to valid range
        for level in docas_levels.iter_mut() {
            *level = level.clamp(0.0, 1.0);
        }
    }
}

impl Default for GlialSystem {
    fn default() -> Self {
        Self::new(10, 10, 100.0)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Statistics for the glial system
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GlialStatistics {
    /// Number of astrocytes
    pub astrocyte_count: usize,
    /// Number of oligodendrocytes
    pub oligodendrocyte_count: usize,
    /// Average calcium activity across astrocytes
    pub average_calcium_activity: f32,
    /// Average conduction velocity in m/s
    pub average_conduction_velocity: f32,
    /// Overall myelination level [0, 1]
    pub myelination_level: f32,
    /// Glutamate clearance efficiency [0, 1]
    pub glutamate_clearance: f32,
    /// Lactate supply rate [0, 2]
    pub lactate_supply: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_astrocyte_creation() {
        let position = NeuralPosition::new_2d(10.0, 20.0);
        let astrocyte = Astrocyte::new(1, position, 5.0);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        assert_eq!(astrocyte.id, 1);
        assert_eq!(astrocyte.position.x, 10.0);
        assert_eq!(astrocyte.position.y, 20.0);
        assert_eq!(astrocyte.coverage_radius, 5.0);
        assert!(astrocyte.calcium_activity >= 0.0 && astrocyte.calcium_activity <= 1.0);
    }

    #[test]
    fn test_astrocyte_calcium_dynamics() {
        let position = NeuralPosition::new_2d(0.0, 0.0);
        let mut astrocyte = Astrocyte::new(1, position, 5.0);

        let initial_calcium = astrocyte.calcium_activity;

        // Update with high activity
        astrocyte.update_calcium(1.0, 1.0);

        // Calcium should increase with activity
        assert!(astrocyte.calcium_activity >= initial_calcium);

        // Test gliotransmitter release threshold
        for _ in 0..100 {
            astrocyte.update_calcium(1.0, 1.0);
        }

        // After sustained high calcium, gliotransmitter should be released
        assert!(astrocyte.gliotransmitter_release > 0.0 || astrocyte.calcium_activity < 0.6);
    }

    #[test]
    fn test_astrocyte_synaptic_modulation() {
        let position = NeuralPosition::new_2d(0.0, 0.0);
        let mut astrocyte = Astrocyte::new(1, position, 10.0);

        // Force gliotransmitter release
        astrocyte.gliotransmitter_release = 0.5;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let base_strength = 1.0;
        let synapse_pos = NeuralPosition::new_2d(1.0, 0.0);
        let modulated = astrocyte.modulate_synaptic_strength(&synapse_pos, base_strength);

        // Modulation should enhance synapse within coverage
        assert!(modulated > base_strength);

        // Outside coverage, no modulation
        let far_pos = NeuralPosition::new_2d(100.0, 100.0);
        let unmodulated = astrocyte.modulate_synaptic_strength(&far_pos, base_strength);
        assert_eq!(unmodulated, base_strength);
    }

    #[test]
    fn test_oligodendrocyte_creation() {
        let segments = vec![AxonSegmentId::new(1), AxonSegmentId::new(2)];
        let oligodendrocyte = Oligodendrocyte::new(1, segments.clone());

        assert_eq!(oligodendrocyte.id, 1);
        assert_eq!(oligodendrocyte.axon_segments, segments);
        assert!(oligodendrocyte.myelin_thickness > 0.0 && oligodendrocyte.myelin_thickness <= 1.0);
        assert!(
            oligodendrocyte.conduction_velocity >= 0.5
                && oligodendrocyte.conduction_velocity <= 100.0
        );
    }

    #[test]
    fn test_oligodendrocyte_velocity_computation() {
        // Test velocity at different thicknesses
        let v_thin = Oligodendrocyte::compute_velocity(0.1);
        let v_medium = Oligodendrocyte::compute_velocity(0.5);
        let v_thick = Oligodendrocyte::compute_velocity(1.0);

        // Thicker myelin should give faster conduction
        assert!(v_medium > v_thin);
        assert!(v_thick > v_medium);

        // All velocities should be in physiological range
        assert!((0.5..=100.0).contains(&v_thin));
        assert!((0.5..=100.0).contains(&v_thick));
    }

    #[test]
    fn test_activity_dependent_myelination() {
        let segments = vec![AxonSegmentId::new(1)];
        let mut oligodendrocyte = Oligodendrocyte::new(1, segments);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let initial_thickness = oligodendrocyte.myelin_thickness;

        // Apply high activity
        for _ in 0..100 {
            oligodendrocyte.update_myelination(0.8, 1.0);
        }

        // Myelination should increase with high activity
        assert!(oligodendrocyte.myelin_thickness > initial_thickness);
        assert!(oligodendrocyte.conduction_velocity > 0.5);
    }

    #[test]
    fn test_conduction_delay_calculation() {
        let segments = vec![AxonSegmentId::new(1)];
        let oligodendrocyte = Oligodendrocyte::new(1, segments);

        // Test delay calculation
        let distance = 1000.0; // micrometers = 1 mm
        let delay = oligodendrocyte.conduction_delay(distance);

        // Delay should be positive and reasonable
        assert!(delay > 0.0);

        // 1mm at ~3-10 m/s should be ~0.1-0.3 ms
        assert!(delay < 0.001); // Less than 1ms for 1mm
    }

    #[test]
    fn test_glial_system_creation() {
        let system = GlialSystem::new(5, 3, 100.0);

        assert_eq!(system.astrocytes.len(), 5);
        assert_eq!(system.oligodendrocytes.len(), 3);
        assert!(system.glutamate_clearance >= 0.0 && system.glutamate_clearance <= 1.0);
        assert!(system.lactate_supply >= 0.0);
    }

    #[test]
    fn test_glial_system_astrocyte_update() {
        let mut system = GlialSystem::new(3, 2, 100.0);

        let activity = vec![0.5, 0.7, 0.3];
        system.update_astrocytes(&activity, 1.0);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Astrocytes should have updated calcium levels
        for astrocyte in &system.astrocytes {
            assert!(astrocyte.calcium_activity >= 0.0 && astrocyte.calcium_activity <= 1.0);
        }
    }

    #[test]
    fn test_glial_system_synaptic_modulation() {
        let system = GlialSystem::new(1, 0, 100.0);

        let position = NeuralPosition::new_2d(0.0, 0.0);
        let base_strength = 1.0;
        let modulated = system.modulate_synaptic_strength(base_strength, &position);

        // Modulation should return a reasonable value
        assert!(modulated > 0.0);
        // With default gliotransmitter=0, modulation should be close to base
        assert!((modulated - base_strength).abs() < 0.1);
    }

    #[test]
    fn test_activity_history() {
        let mut history = ActivityHistory::new(10);
        let segment = AxonSegmentId::new(1);

        // Record some activity
        for i in 0..5 {
            history.record(segment, i as f32 * 0.1);
        }

        let avg = history.average_for_segment(&segment);
        assert!(avg > 0.0);

        // Test max history limit
        for _ in 0..20 {
            history.record(segment, 1.0);
        }

        // History should be capped at max_history
        let activities = history.segment_activity.get(&segment).unwrap();
        assert_eq!(activities.len(), 10);
    }

    #[test]
    fn test_trust_timing_modifier() {
        let mut system = GlialSystem::new(2, 2, 100.0);

        // With default myelination, modifier should be near 1.0
        let modifier_default = system.trust_timing_modifier();
        assert!(modifier_default > 0.0);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // Increase myelination
        for oligodendrocyte in &mut system.oligodendrocytes {
            oligodendrocyte.myelin_thickness = 1.0;
            oligodendrocyte.conduction_velocity = Oligodendrocyte::compute_velocity(1.0);
        }

        let modifier_fast = system.trust_timing_modifier();
        // Faster conduction should give lower modifier (faster decisions)
        assert!(modifier_fast <= modifier_default);
    }

    #[test]
    fn test_docas_modulation() {
        let system = GlialSystem::new(2, 2, 100.0);
        let mut docas = [0.5; 5];

        system.modulate_docas(&mut docas, 1.0);

        // Levels should remain in valid range
        for level in &docas {
            assert!(*level >= 0.0 && *level <= 1.0);
        }
    }

    #[test]
    fn test_glial_statistics() {
        let system = GlialSystem::new(5, 3, 100.0);
        let stats = system.statistics();

        assert_eq!(stats.astrocyte_count, 5);
        assert_eq!(stats.oligodendrocyte_count, 3);
        assert!(stats.average_calcium_activity >= 0.0);
        assert!(stats.average_conduction_velocity >= 0.5);
        assert!(stats.myelination_level >= 0.0 && stats.myelination_level <= 1.0);
    }

    #[test]
    fn test_add_cells() {
        let mut system = GlialSystem::new(0, 0, 100.0);

        let astro_id = system.add_astrocyte(NeuralPosition::new_2d(10.0, 10.0), 5.0);
        assert_eq!(system.astrocytes.len(), 1);
        assert_eq!(system.astrocytes[0].id, astro_id);

        let oligo_id = system.add_oligodendrocyte(vec![AxonSegmentId::new(1)]);
        assert_eq!(system.oligodendrocytes.len(), 1);
        assert_eq!(system.oligodendrocytes[0].id, oligo_id);
    }
}
