//! Long-Term Potentiation (LTP) and Long-Term Depression (LTD) Implementation
//!
//! Activity-dependent synaptic plasticity for learning and memory.
//! Implements Hebbian learning, STDP, and calcium-dependent plasticity rules.
//!
// TAG: surface=simulation owner=research-team rule=SIM-001
//! # Background
//!
//! LTP and LTD are the cellular basis of learning and memory:
//! - **LTP**: Long-term potentiation - synaptic strengthening
//! - **LTD**: Long-term depression - synaptic weakening
//!
//! Key principles:
//! - Hebbian rule: "Neurons that fire together, wire together"
//! - STDP: Timing-dependent plasticity based on spike order
//! - Calcium threshold: High Ca2+ → LTP, moderate → LTD (Lisman hypothesis)
//!
//! # References
//!
//! - Hebb, D.O. (1949). The Organization of Behavior
//! - Bi & Poo (1998). Synaptic modifications in cultured hippocampal neurons
//! - Lisman (1989). A mechanism for the Hebb and the anti-Hebb processes

use crate::types::{AgentId, Tick};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::VecDeque;

/// Unique identifier for neurons in the plasticity system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NeuronId(pub u64);

impl NeuronId {
    /// Create new neuron ID
    #[must_use]
    pub const fn new(id: u64) -> Self {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        Self(id)
    }
}

/// Synapse with plasticity-capable weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synapse {
    /// Pre-synaptic neuron ID
    pub pre_id: NeuronId,
    /// Post-synaptic neuron ID
    pub post_id: NeuronId,
    /// Current synaptic weight
    pub weight: f32,
    /// Initial weight (for reset/reference)
    pub initial_weight: f32,
    /// Maximum allowed weight
    pub max_weight: f32,
    /// Minimum allowed weight
    pub min_weight: f32,
    /// Timestamp of last pre-synaptic spike
    pub last_pre_spike: Option<Tick>,
    /// Timestamp of last post-synaptic spike
    pub last_post_spike: Option<Tick>,
    /// Plasticity tag for protein synthesis marker
    pub tag: PlasticityTag,
    /// Calcium concentration at synapse (for calcium-dependent plasticity)
    pub calcium_level: f32,
    /// Age of synapse in ticks (for developmental scaling)
    pub age_ticks: u64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Synapse {
    /// Create a new synapse with default weight
    #[must_use]
    pub const fn new(pre_id: NeuronId, post_id: NeuronId, initial_weight: f32) -> Self {
        Self {
            pre_id,
            post_id,
            weight: initial_weight,
            initial_weight,
            max_weight: 2.0,
            min_weight: 0.0,
            last_pre_spike: None,
            last_post_spike: None,
            tag: PlasticityTag::None,
            calcium_level: 0.0,
            age_ticks: 0,
        }
    }

    /// Create a new synapse with bounded weights
    #[must_use]
    pub fn with_bounds(
        pre_id: NeuronId,
        post_id: NeuronId,
        initial_weight: f32,
        min_weight: f32,
        max_weight: f32,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    ) -> Self {
        Self {
            pre_id,
            post_id,
            weight: initial_weight.clamp(min_weight, max_weight),
            initial_weight,
            max_weight,
            min_weight,
            last_pre_spike: None,
            last_post_spike: None,
            tag: PlasticityTag::None,
            calcium_level: 0.0,
            age_ticks: 0,
        }
    }

    /// Record a pre-synaptic spike
    pub fn record_pre_spike(&mut self, tick: Tick) {
        self.last_pre_spike = Some(tick);
        // Calcium influx from pre-synaptic activity
        self.calcium_level = (self.calcium_level + 0.1).min(1.0);
    }

    /// Record a post-synaptic spike
    pub fn record_post_spike(&mut self, tick: Tick) {
        self.last_post_spike = Some(tick);
        // Voltage-gated calcium channel opening
        self.calcium_level = (self.calcium_level + 0.2).min(1.0);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Decay calcium level (called each tick)
    pub fn decay_calcium(&mut self, decay_rate: f32) {
        self.calcium_level *= 1.0 - decay_rate;
        self.age_ticks += 1;
    }

    /// Update synaptic weight based on current calcium level (Graupner & Brunel, 2012)
    pub fn update_weight_from_calcium(&mut self, dt: f32) {
        const THETA_P: f32 = 0.5;
        const THETA_D: f32 = 0.2;
        const GAMMA_P: f32 = 0.1;
        const GAMMA_D: f32 = 0.05;

        if self.calcium_level > THETA_P {
            self.weight += GAMMA_P * (self.max_weight - self.weight) * dt;
        } else if self.calcium_level > THETA_D {
            self.weight -= GAMMA_D * (self.weight - self.min_weight) * dt;
        }
        self.clamp_weight();
    }

    /// Get weight change relative to initial
    #[must_use]
    pub fn weight_change(&self) -> f32 {
        self.weight - self.initial_weight
    }

    /// Check if synapse is potentiated (LTP occurred)
    #[must_use]
    pub fn is_potentiated(&self) -> bool {
        self.tag == PlasticityTag::LateLTPTag || self.weight > self.initial_weight * 1.1
    }

    /// Check if synapse is depressed (LTD occurred)
    #[must_use]
    pub fn is_depressed(&self) -> bool {
        self.tag == PlasticityTag::LateLTDTag || self.weight < self.initial_weight * 0.9
    }

    /// Reset weight to initial value
    pub fn reset_weight(&mut self) {
        self.weight = self.initial_weight;
        self.tag = PlasticityTag::None;
    }

    /// Clamp weight to bounds
    pub fn clamp_weight(&mut self) {
        self.weight = self.weight.clamp(self.min_weight, self.max_weight);
    }
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
/// Plasticity tags for late-phase consolidation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlasticityTag {
    /// No tag
    None,
    /// Early-phase LTP tag (~1 hour duration)
    EarlyLTPTag,
    /// Late-phase LTP tag (requires protein synthesis, persistent)
    LateLTPTag,
    /// Early-phase LTD tag
    EarlyLTDTag,
    /// Late-phase LTD tag (persistent)
    LateLTDTag,
}

impl PlasticityTag {
    /// Check if this is an LTP tag
    #[must_use]
    pub const fn is_ltp(&self) -> bool {
        matches!(self, Self::EarlyLTPTag | Self::LateLTPTag)
    }

    /// Check if this is an LTD tag
    #[must_use]
    pub const fn is_ltd(&self) -> bool {
        matches!(self, Self::EarlyLTDTag | Self::LateLTDTag)
    }

    /// Check if this is a late-phase tag
    #[must_use]
    pub const fn is_late_phase(&self) -> bool {
        matches!(self, Self::LateLTPTag | Self::LateLTDTag)
    }

    /// Check if this is an early-phase tag
    #[must_use]
    pub const fn is_early_phase(&self) -> bool {
        matches!(self, Self::EarlyLTPTag | Self::EarlyLTDTag)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// STDP (Spike-Timing Dependent Plasticity) parameters
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StdpParams {
    /// LTP amplitude (pre before post)
    pub a_plus: f32,
    /// LTD amplitude (post before pre)
    pub a_minus: f32,
    /// LTP time constant (ms)
    pub tau_plus: f32,
    /// LTD time constant (ms)
    pub tau_minus: f32,
    /// Maximum time window for STDP (ms)
    pub max_window_ms: f32,
}

impl Default for StdpParams {
    fn default() -> Self {
        Self {
            a_plus: 0.01,
            a_minus: 0.012,
            tau_plus: 20.0,
            tau_minus: 20.0,
            max_window_ms: 100.0,
        }
    }
}

/// Calcium-dependent plasticity parameters (Lisman hypothesis)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CalciumParams {
    /// High threshold for LTP induction
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub theta_ltp: f32,
    /// Moderate threshold for LTD induction
    pub theta_ltd: f32,
    /// LTP learning rate multiplier
    pub ltp_multiplier: f32,
    /// LTD learning rate multiplier
    pub ltd_multiplier: f32,
    /// Calcium decay rate per tick
    pub calcium_decay: f32,
}

impl Default for CalciumParams {
    fn default() -> Self {
        Self {
            theta_ltp: 0.8,
            theta_ltd: 0.3,
            ltp_multiplier: 2.0,
            ltd_multiplier: 1.5,
            calcium_decay: 0.01,
        }
    }
}

/// Homeostatic scaling parameters
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HomeostaticParams {
    /// Target firing rate (Hz)
    pub target_rate: f32,
    /// Scaling strength
    pub scaling_factor: f32,
    /// Time window for averaging (ticks)
    pub averaging_window: usize,
    /// Minimum allowed scaling factor
    pub min_scale: f32,
    /// Maximum allowed scaling factor
    pub max_scale: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for HomeostaticParams {
    fn default() -> Self {
        Self {
            target_rate: 10.0,
            scaling_factor: 0.1,
            averaging_window: 1000,
            min_scale: 0.5,
            max_scale: 2.0,
        }
    }
}

/// Metaplasticity parameters (plasticity of plasticity)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MetaplasticityParams {
    /// High activity threshold for threshold increase
    pub high_activity_threshold: f32,
    /// Low activity threshold for threshold decrease
    pub low_activity_threshold: f32,
    /// Learning rate increase factor
    pub increase_factor: f32,
    /// Learning rate decrease factor
    pub decrease_factor: f32,
    /// Maximum learning rate
    pub max_learning_rate: f32,
    /// Minimum learning rate
    pub min_learning_rate: f32,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

impl Default for MetaplasticityParams {
    fn default() -> Self {
        Self {
            high_activity_threshold: 0.7,
            low_activity_threshold: 0.3,
            increase_factor: 1.05,
            decrease_factor: 0.95,
            max_learning_rate: 0.1,
            min_learning_rate: 0.001,
        }
    }
}

/// Consolidation parameters for late-phase plasticity
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ConsolidationParams {
    /// Number of ticks for early-phase to late-phase transition
    pub consolidation_ticks: u64,
    /// Protein synthesis threshold (tag strength required)
    pub protein_synthesis_threshold: f32,
    /// Decay rate for early-phase tags (per tick)
    pub early_tag_decay: f32,
}

impl Default for ConsolidationParams {
    fn default() -> Self {
        Self {
            consolidation_ticks: 3600, // ~1 hour at 1 tick/second
            protein_synthesis_threshold: 0.005,
            early_tag_decay: 0.0001,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Statistics for plasticity system
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlasticityStats {
    /// Total number of synapses
    pub total_synapses: usize,
    /// Number of potentiated synapses
    pub potentiated_count: usize,
    /// Number of depressed synapses
    pub depressed_count: usize,
    /// Number with early LTP tag
    pub early_ltp_count: usize,
    /// Number with late LTP tag
    pub late_ltp_count: usize,
    /// Number with early LTD tag
    pub early_ltd_count: usize,
    /// Number with late LTD tag
    pub late_ltd_count: usize,
    /// Average weight change
    pub avg_weight_change: f32,
    /// Average calcium level
    pub avg_calcium: f32,
    /// Current learning rate
    pub current_learning_rate: f32,
    /// Total LTP events
    pub ltp_events: u64,
    /// Total LTD events
    pub ltd_events: u64,
}

/// Long-term potentiation/depression system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynapticPlasticity {
    /// Synapses undergoing plasticity
    pub synapses: Vec<Synapse>,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Calcium history for metaplasticity
    pub calcium_history: VecDeque<f32>,
    /// Base learning rate
    pub learning_rate: f32,
    /// Current effective learning rate (with metaplasticity)
    pub effective_learning_rate: f32,
    /// STDP parameters
    pub stdp_params: StdpParams,
    /// Calcium-dependent plasticity parameters
    pub calcium_params: CalciumParams,
    /// Homeostatic scaling parameters
    pub homeostatic_params: HomeostaticParams,
    /// Metaplasticity parameters
    pub metaplasticity_params: MetaplasticityParams,
    /// Consolidation parameters
    pub consolidation_params: ConsolidationParams,
    /// Agent ID this plasticity system belongs to
    pub agent_id: Option<AgentId>,
    /// Firing rate history for homeostatic scaling
    firing_rate_history: VecDeque<f32>,
    /// Current tick
    current_tick: Tick,
}

impl SynapticPlasticity {
    /// Create new synaptic plasticity system
    #[must_use]
    pub fn new() -> Self {
        Self {
            synapses: Vec::new(),
            calcium_history: VecDeque::with_capacity(1000),
            learning_rate: 0.01,
            effective_learning_rate: 0.01,
            stdp_params: StdpParams::default(),
            calcium_params: CalciumParams::default(),
            homeostatic_params: HomeostaticParams::default(),
            metaplasticity_params: MetaplasticityParams::default(),
            consolidation_params: ConsolidationParams::default(),
            agent_id: None,
            firing_rate_history: VecDeque::with_capacity(10000),
            current_tick: 0,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create new plasticity system for specific agent
    #[must_use]
    pub fn for_agent(agent_id: AgentId) -> Self {
        let mut s = Self::new();
        s.agent_id = Some(agent_id);
        s
    }

    /// Add a synapse to the system
    pub fn add_synapse(&mut self, synapse: Synapse) -> usize {
        self.synapses.push(synapse);
        self.synapses.len() - 1
    }

    /// Create and add a new synapse
    pub fn create_synapse(
        &mut self,
        pre_id: NeuronId,
        post_id: NeuronId,
        initial_weight: f32,
    ) -> usize {
        self.add_synapse(Synapse::new(pre_id, post_id, initial_weight))
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Get synapse by index
    #[must_use]
    pub fn get_synapse(&self, idx: usize) -> Option<&Synapse> {
        self.synapses.get(idx)
    }

    /// Get mutable synapse by index
    pub fn get_synapse_mut(&mut self, idx: usize) -> Option<&mut Synapse> {
        self.synapses.get_mut(idx)
    }

    /// Get synapse index by neuron IDs
    #[must_use]
    pub fn find_synapse(&self, pre_id: NeuronId, post_id: NeuronId) -> Option<usize> {
        self.synapses
            .iter()
            .position(|s| s.pre_id == pre_id && s.post_id == post_id)
    }

    /// Remove a synapse
    pub fn remove_synapse(&mut self, idx: usize) -> Option<Synapse> {
        if idx < self.synapses.len() {
            Some(self.synapses.remove(idx))
        } else {
            None
        }
    }

    /// Record pre-synaptic spike
    pub fn record_pre_spike(&mut self, pre_id: NeuronId, tick: Tick) {
        for synapse in &mut self.synapses {
            if synapse.pre_id == pre_id {
                synapse.record_pre_spike(tick);
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Record post-synaptic spike
    pub fn record_post_spike(&mut self, post_id: NeuronId, tick: Tick) {
        for synapse in &mut self.synapses {
            if synapse.post_id == post_id {
                synapse.record_post_spike(tick);
            }
        }
    }

    /// Hebbian LTP: Correlated pre/post activity strengthens synapse
    ///
    /// `Delta_w` = eta * `pre_activity` * `post_activity`
    pub fn apply_hebbian(&mut self, synapse_idx: usize, pre_activity: f32, post_activity: f32) {
        let Some(synapse) = self.synapses.get_mut(synapse_idx) else {
            return;
        };

        // Hebbian update rule
        let delta = self.effective_learning_rate * pre_activity * post_activity;
        synapse.weight += delta;
        synapse.clamp_weight();

        // Tag for late-phase if strong potentiation
        if delta > self.consolidation_params.protein_synthesis_threshold {
            synapse.tag = PlasticityTag::EarlyLTPTag;
        }
    }

    /// Apply Hebbian learning to all synapses between pre and post neurons
    pub fn apply_hebbian_bulk(
        &mut self,
        pre_neurons: &[NeuronId],
        post_neurons: &[NeuronId],
        pre_activity: &[f32],
        post_activity: &[f32],
    ) {
        for (i, &pre_id) in pre_neurons.iter().enumerate() {
            for (j, &post_id) in post_neurons.iter().enumerate() {
                if let Some(idx) = self.find_synapse(pre_id, post_id) {
                    let pre_act = pre_activity.get(i).copied().unwrap_or(0.0);
                    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                    let post_act = post_activity.get(j).copied().unwrap_or(0.0);
                    self.apply_hebbian(idx, pre_act, post_act);
                }
            }
        }
    }

    /// Spike-timing dependent plasticity (STDP)
    ///
    /// # Arguments
    /// * `synapse_idx` - Index of synapse to update
    /// * `pre_time` - Time of pre-synaptic spike
    /// * `post_time` - Time of post-synaptic spike
    ///
    /// # STDP Window
    /// - Pre before post (`delta_t` > 0): LTP
    /// - Post before pre (`delta_t` < 0): LTD
    pub fn apply_stdp(&mut self, synapse_idx: usize, pre_time: Tick, post_time: Tick) {
        let Some(synapse) = self.synapses.get_mut(synapse_idx) else {
            return;
        };

        let delta_t = post_time as i64 - pre_time as i64;
        let delta_t_ms = delta_t as f32; // Assume 1 tick = 1 ms

        // Check if within STDP window
        if delta_t_ms.abs() > self.stdp_params.max_window_ms {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            return;
        }

        // Calculate weight change based on STDP curve
        let tau_plus = self.stdp_params.tau_plus.max(0.1);
        let tau_minus = self.stdp_params.tau_minus.max(0.1);
        let delta_w = match delta_t.cmp(&0) {
            Ordering::Greater => {
                // Pre before post: LTP (causal)
                self.stdp_params.a_plus * (-delta_t_ms / tau_plus).exp()
            }
            Ordering::Less => {
                // Post before pre: LTD (anti-causal)
                -self.stdp_params.a_minus * (delta_t_ms / tau_minus).exp()
            }
            Ordering::Equal => 0.0,
        };

        synapse.weight += delta_w;
        synapse.clamp_weight();

        // Tag for late-phase consolidation
        if delta_w > self.consolidation_params.protein_synthesis_threshold {
            synapse.tag = PlasticityTag::EarlyLTPTag;
        } else if delta_w < -self.consolidation_params.protein_synthesis_threshold {
            synapse.tag = PlasticityTag::EarlyLTDTag;
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Apply STDP to synapse based on recorded spike times
    pub fn apply_stdp_recorded(&mut self, synapse_idx: usize, current_tick: Tick) {
        let Some(synapse) = self.synapses.get(synapse_idx) else {
            return;
        };

        // Need both spike times
        let (Some(pre_time), Some(post_time)) = (synapse.last_pre_spike, synapse.last_post_spike)
        else {
            return;
        };

        // Only apply if recent enough
        let time_since = current_tick.saturating_sub(pre_time.max(post_time));
        if time_since > self.stdp_params.max_window_ms as u64 {
            return;
        }

        self.apply_stdp(synapse_idx, pre_time, post_time);
    }

    /// Calcium-dependent plasticity (Lisman hypothesis)
    ///
    /// High Ca2+ → LTP, moderate → LTD, low → no change
    ///
    /// # Arguments
    /// * `synapse_idx` - Index of synapse to update
    pub fn apply_calcium_plasticity(&mut self, synapse_idx: usize) {
        let Some(synapse) = self.synapses.get_mut(synapse_idx) else {
            return;
        };

        let calcium = synapse.calcium_level;

        if calcium > self.calcium_params.theta_ltp {
            // High calcium: LTP
            let delta = self.effective_learning_rate * self.calcium_params.ltp_multiplier;
            synapse.weight += delta;
            synapse.clamp_weight();
            synapse.tag = PlasticityTag::EarlyLTPTag;
        } else if calcium > self.calcium_params.theta_ltd {
            // Moderate calcium: LTD
            let delta = -self.effective_learning_rate * self.calcium_params.ltd_multiplier;
            synapse.weight += delta;
            synapse.clamp_weight();
            synapse.tag = PlasticityTag::EarlyLTDTag;
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Apply calcium-dependent plasticity to all synapses
    pub fn apply_calcium_plasticity_all(&mut self) {
        for i in 0..self.synapses.len() {
            self.apply_calcium_plasticity(i);
        }
    }

    /// Consolidate early-phase tags to late-phase (protein synthesis)
    ///
    /// Called periodically to convert early tags to late tags.
    /// Late-phase plasticity is protein synthesis-dependent and persistent.
    pub fn consolidate(&mut self) {
        for synapse in &mut self.synapses {
            match synapse.tag {
                PlasticityTag::EarlyLTPTag => {
                    // With protein synthesis, become permanent
                    if synapse.age_ticks >= self.consolidation_params.consolidation_ticks {
                        synapse.tag = PlasticityTag::LateLTPTag;
                        synapse.initial_weight = synapse.weight; // Reset baseline
                    }
                }
                PlasticityTag::EarlyLTDTag => {
                    if synapse.age_ticks >= self.consolidation_params.consolidation_ticks {
                        synapse.tag = PlasticityTag::LateLTDTag;
                        synapse.initial_weight = synapse.weight;
                    }
                }
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                _ => {}
            }
        }
    }

    /// Homeostatic scaling (maintain stable firing rates)
    ///
    /// Scales all synaptic weights to maintain target firing rate.
    /// This prevents runaway excitation or silencing.
    ///
    /// # Arguments
    /// * `measured_rate` - Actual firing rate (Hz)
    pub fn apply_homeostatic_scaling(&mut self, measured_rate: f32) {
        if self.synapses.is_empty() || measured_rate <= 0.0 {
            return;
        }

        let target = self.homeostatic_params.target_rate;
        let scale_factor = target / measured_rate;

        // Apply bounded scaling
        let bounded_scale = scale_factor.clamp(
            self.homeostatic_params.min_scale,
            self.homeostatic_params.max_scale,
        );

        for synapse in &mut self.synapses {
            synapse.weight *= bounded_scale;
            synapse.clamp_weight();
        }
    }

    /// Update homeostatic scaling based on firing rate history
    pub fn update_homeostatic_scaling(&mut self) {
        if self.firing_rate_history.len() < self.homeostatic_params.averaging_window {
            return;
        }

        let avg_rate: f32 =
            self.firing_rate_history.iter().sum::<f32>() / self.firing_rate_history.len() as f32;

        self.apply_homeostatic_scaling(avg_rate);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Record firing rate for homeostatic scaling
    pub fn record_firing_rate(&mut self, rate: f32) {
        self.firing_rate_history.push_back(rate);
        if self.firing_rate_history.len() > self.homeostatic_params.averaging_window {
            self.firing_rate_history.pop_front();
        }
    }

    /// Metaplasticity (plasticity of plasticity)
    ///
    /// Prior activity history affects plasticity threshold.
    /// High activity → raise threshold (protect from saturation)
    /// Low activity → lower threshold (increase plasticity)
    pub fn update_metaplasticity(&mut self) {
        if self.calcium_history.is_empty() {
            return;
        }

        let recent_activity: f32 =
            self.calcium_history.iter().sum::<f32>() / self.calcium_history.len() as f32;

        if recent_activity > self.metaplasticity_params.high_activity_threshold {
            // High activity history → raise threshold (protect from saturation)
            self.effective_learning_rate *= self.metaplasticity_params.decrease_factor;
        } else if recent_activity < self.metaplasticity_params.low_activity_threshold {
            // Low activity → lower threshold (increase plasticity)
            self.effective_learning_rate = (self.effective_learning_rate
                * self.metaplasticity_params.increase_factor)
                .min(self.metaplasticity_params.max_learning_rate);
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        }

        // Clamp learning rate
        self.effective_learning_rate = self
            .effective_learning_rate
            .max(self.metaplasticity_params.min_learning_rate);
    }

    /// Record calcium level for metaplasticity
    pub fn record_calcium(&mut self, calcium: f32) {
        self.calcium_history.push_back(calcium);
        if self.calcium_history.len() > 1000 {
            self.calcium_history.pop_front();
        }
    }

    /// Update all synapse calcium levels (call each tick)
    pub fn decay_all_calcium(&mut self) {
        for synapse in &mut self.synapses {
            synapse.decay_calcium(self.calcium_params.calcium_decay);
        }
    }

    /// Process one simulation tick
    ///
    /// Applies decay (which advances synapse age), updates metaplasticity, and consolidates tags
    pub fn tick(&mut self) {
        self.current_tick += 1;

        // Decay calcium levels (and advance synapse age_ticks)
        self.decay_all_calcium();

        // Update metaplasticity periodically
        if self.current_tick % 100 == 0 {
            self.update_metaplasticity();
        }

        // Consolidate periodically
        if self.consolidation_params.consolidation_ticks > 0
            && self.current_tick % self.consolidation_params.consolidation_ticks == 0
        {
            self.consolidate();
        }

        // Update homeostatic scaling periodically
        if self.current_tick % 1000 == 0 {
            self.update_homeostatic_scaling();
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get current statistics
    #[must_use]
    pub fn stats(&self) -> PlasticityStats {
        let total = self.synapses.len();
        if total == 0 {
            return PlasticityStats::default();
        }

        let potentiated = self.synapses.iter().filter(|s| s.is_potentiated()).count();
        let depressed = self.synapses.iter().filter(|s| s.is_depressed()).count();
        let early_ltp = self
            .synapses
            .iter()
            .filter(|s| s.tag == PlasticityTag::EarlyLTPTag)
            .count();
        let late_ltp = self
            .synapses
            .iter()
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            .filter(|s| s.tag == PlasticityTag::LateLTPTag)
            .count();
        let early_ltd = self
            .synapses
            .iter()
            .filter(|s| s.tag == PlasticityTag::EarlyLTDTag)
            .count();
        let late_ltd = self
            .synapses
            .iter()
            .filter(|s| s.tag == PlasticityTag::LateLTDTag)
            .count();

        let avg_change: f32 = self
            .synapses
            .iter()
            .map(Synapse::weight_change)
            .sum::<f32>()
            / total as f32;
        let avg_calcium: f32 =
            self.synapses.iter().map(|s| s.calcium_level).sum::<f32>() / total as f32;

        PlasticityStats {
            total_synapses: total,
            potentiated_count: potentiated,
            depressed_count: depressed,
            early_ltp_count: early_ltp,
            late_ltp_count: late_ltp,
            early_ltd_count: early_ltd,
            late_ltd_count: late_ltd,
            avg_weight_change: avg_change,
            avg_calcium,
            current_learning_rate: self.effective_learning_rate,
            ltp_events: self.synapses.iter().filter(|s| s.tag.is_ltp()).count() as u64,
            ltd_events: self.synapses.iter().filter(|s| s.tag.is_ltd()).count() as u64,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Reset all synapses to initial weights
    pub fn reset_all_weights(&mut self) {
        for synapse in &mut self.synapses {
            synapse.reset_weight();
        }
    }

    /// Clear all synapses
    pub fn clear_synapses(&mut self) {
        self.synapses.clear();
    }

    /// Get total synaptic weight (for normalization)
    #[must_use]
    pub fn total_weight(&self) -> f32 {
        self.synapses.iter().map(|s| s.weight).sum()
    }

    /// Get average weight
    #[must_use]
    pub fn average_weight(&self) -> f32 {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        if self.synapses.is_empty() {
            0.0
        } else {
            self.total_weight() / self.synapses.len() as f32
        }
    }

    /// Normalize all weights to sum to target
    pub fn normalize_weights(&mut self, target_sum: f32) {
        let current = self.total_weight();
        if current > 0.0 {
            let scale = target_sum / current;
            for synapse in &mut self.synapses {
                synapse.weight *= scale;
                synapse.clamp_weight();
            }
        }
    }

    /// Prune weak synapses (weights below threshold)
    pub fn prune_weak_synapses(&mut self, threshold: f32) -> usize {
        let initial_count = self.synapses.len();
        self.synapses.retain(|s| s.weight >= threshold);
        initial_count - self.synapses.len()
    }

    /// Get synapses for a specific post-synaptic neuron
    #[must_use]
    pub fn get_synapses_to(&self, post_id: NeuronId) -> Vec<&Synapse> {
        self.synapses
            .iter()
            .filter(|s| s.post_id == post_id)
            .collect()
    }

    /// Get synapses from a specific pre-synaptic neuron
    #[must_use]
    pub fn get_synapses_from(&self, pre_id: NeuronId) -> Vec<&Synapse> {
        self.synapses
            .iter()
            .filter(|s| s.pre_id == pre_id)
            .collect()
    }

    /// Set learning rate
    pub fn set_learning_rate(&mut self, rate: f32) {
        self.learning_rate = rate.max(0.0);
        self.effective_learning_rate = self.learning_rate;
    }
    // TAG: surface=simulation owner=research-team rule=SIM-001

    /// Get effective learning rate
    #[must_use]
    pub const fn get_effective_learning_rate(&self) -> f32 {
        self.effective_learning_rate
    }
}

impl Default for SynapticPlasticity {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating synaptic plasticity systems
#[derive(Debug)]
pub struct PlasticityBuilder {
    plasticity: SynapticPlasticity,
}

impl PlasticityBuilder {
    /// Create new builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            plasticity: SynapticPlasticity::new(),
        }
    }

    /// Set agent ID
    #[must_use]
    pub const fn for_agent(mut self, agent_id: AgentId) -> Self {
        self.plasticity.agent_id = Some(agent_id);
        self
    }

    /// Set learning rate
    #[must_use]
    pub fn with_learning_rate(mut self, rate: f32) -> Self {
        self.plasticity.set_learning_rate(rate);
        self
    }

    /// Set STDP parameters
    #[must_use]
    pub const fn with_stdp_params(mut self, params: StdpParams) -> Self {
        self.plasticity.stdp_params = params;
        self
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Set calcium parameters
    #[must_use]
    pub const fn with_calcium_params(mut self, params: CalciumParams) -> Self {
        self.plasticity.calcium_params = params;
        self
    }

    /// Set homeostatic parameters
    #[must_use]
    pub const fn with_homeostatic_params(mut self, params: HomeostaticParams) -> Self {
        self.plasticity.homeostatic_params = params;
        self
    }

    /// Set metaplasticity parameters
    #[must_use]
    pub const fn with_metaplasticity_params(mut self, params: MetaplasticityParams) -> Self {
        self.plasticity.metaplasticity_params = params;
        self
    }

    /// Set consolidation parameters
    #[must_use]
    pub const fn with_consolidation_params(mut self, params: ConsolidationParams) -> Self {
        self.plasticity.consolidation_params = params;
        self
    }

    /// Add a synapse
    #[must_use]
    pub fn add_synapse(mut self, synapse: Synapse) -> Self {
        self.plasticity.add_synapse(synapse);
        self
    }

    /// Build the plasticity system
    #[must_use]
    pub fn build(self) -> SynapticPlasticity {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        self.plasticity
    }
}

impl Default for PlasticityBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Basic Synapse Tests ===

    #[test]
    fn test_synapse_creation() {
        let synapse = Synapse::new(NeuronId::new(1), NeuronId::new(2), 0.5);
        assert_eq!(synapse.pre_id, NeuronId::new(1));
        assert_eq!(synapse.post_id, NeuronId::new(2));
        assert_eq!(synapse.weight, 0.5);
        assert_eq!(synapse.initial_weight, 0.5);
        assert_eq!(synapse.tag, PlasticityTag::None);
    }

    #[test]
    fn test_synapse_with_bounds() {
        let synapse = Synapse::with_bounds(NeuronId::new(1), NeuronId::new(2), 0.5, 0.1, 1.0);
        assert_eq!(synapse.min_weight, 0.1);
        assert_eq!(synapse.max_weight, 1.0);
    }

    #[test]
    fn test_synapse_spike_recording() {
        let mut synapse = Synapse::new(NeuronId::new(1), NeuronId::new(2), 0.5);

        synapse.record_pre_spike(10);
        assert_eq!(synapse.last_pre_spike, Some(10));
        assert!(synapse.calcium_level > 0.0);

        synapse.record_post_spike(15);
        assert_eq!(synapse.last_post_spike, Some(15));
        assert!(synapse.calcium_level > 0.1);
    }

    #[test]
    fn test_synapse_calcium_decay() {
        let mut synapse = Synapse::new(NeuronId::new(1), NeuronId::new(2), 0.5);
        // TAG: surface=simulation owner=research-team rule=SIM-001
        synapse.calcium_level = 1.0;

        synapse.decay_calcium(0.1);
        assert!(synapse.calcium_level < 1.0);
        assert!(synapse.calcium_level > 0.8);
    }

    #[test]
    fn test_synapse_weight_change() {
        let mut synapse = Synapse::new(NeuronId::new(1), NeuronId::new(2), 0.5);
        synapse.weight = 0.7;

        assert!((synapse.weight_change() - 0.2).abs() < 0.0001);
        assert!(synapse.is_potentiated());
        assert!(!synapse.is_depressed());

        synapse.weight = 0.4;
        assert!(synapse.is_depressed());
        assert!(!synapse.is_potentiated());
    }

    #[test]
    fn test_synapse_clamp() {
        let mut synapse = Synapse::with_bounds(NeuronId::new(1), NeuronId::new(2), 0.5, 0.1, 1.0);

        synapse.weight = 2.0;
        synapse.clamp_weight();
        assert_eq!(synapse.weight, 1.0);

        synapse.weight = -0.5;
        synapse.clamp_weight();
        assert_eq!(synapse.weight, 0.1);
    }

    // === PlasticityTag Tests ===

    #[test]
    fn test_plasticity_tag_classification() {
        assert!(PlasticityTag::EarlyLTPTag.is_ltp());
        assert!(PlasticityTag::LateLTPTag.is_ltp());
        assert!(!PlasticityTag::EarlyLTPTag.is_ltd());

        assert!(PlasticityTag::EarlyLTDTag.is_ltd());
        assert!(PlasticityTag::LateLTDTag.is_ltd());
        assert!(!PlasticityTag::EarlyLTDTag.is_ltp());

        assert!(PlasticityTag::LateLTPTag.is_late_phase());
        assert!(PlasticityTag::LateLTDTag.is_late_phase());
        assert!(!PlasticityTag::EarlyLTPTag.is_late_phase());

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert!(PlasticityTag::EarlyLTPTag.is_early_phase());
        assert!(!PlasticityTag::LateLTPTag.is_early_phase());
    }

    // === SynapticPlasticity Tests ===

    #[test]
    fn test_plasticity_creation() {
        let plasticity = SynapticPlasticity::new();
        assert!(plasticity.synapses.is_empty());
        assert_eq!(plasticity.learning_rate, 0.01);
        assert_eq!(plasticity.effective_learning_rate, 0.01);
    }

    #[test]
    fn test_plasticity_for_agent() {
        let plasticity = SynapticPlasticity::for_agent(42);
        assert_eq!(plasticity.agent_id, Some(42));
    }

    #[test]
    fn test_add_and_find_synapse() {
        let mut plasticity = SynapticPlasticity::new();
        let pre = NeuronId::new(1);
        let post = NeuronId::new(2);

        let idx = plasticity.create_synapse(pre, post, 0.5);
        assert_eq!(idx, 0);

        let found = plasticity.find_synapse(pre, post);
        assert_eq!(found, Some(0));

        let not_found = plasticity.find_synapse(NeuronId::new(99), post);
        assert_eq!(not_found, None);
    }

    #[test]
    fn test_hebbian_ltp() {
        let mut plasticity = SynapticPlasticity::new();
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        plasticity.set_learning_rate(0.1); // Higher learning rate for test
        let idx = plasticity.create_synapse(NeuronId::new(1), NeuronId::new(2), 0.5);

        // Apply multiple times to exceed potentiation threshold (>10% increase)
        for _ in 0..10 {
            plasticity.apply_hebbian(idx, 1.0, 1.0);
        }

        let synapse = plasticity.get_synapse(idx).unwrap();
        assert!(synapse.weight > 0.5);
        assert!(synapse.is_potentiated());
    }

    #[test]
    fn test_hebbian_no_activity() {
        let mut plasticity = SynapticPlasticity::new();
        let idx = plasticity.create_synapse(NeuronId::new(1), NeuronId::new(2), 0.5);

        plasticity.apply_hebbian(idx, 0.0, 0.0);

        let synapse = plasticity.get_synapse(idx).unwrap();
        assert_eq!(synapse.weight, 0.5);
    }

    #[test]
    fn test_stdp_ltp() {
        let mut plasticity = SynapticPlasticity::new();
        let idx = plasticity.create_synapse(NeuronId::new(1), NeuronId::new(2), 0.5);

        // Pre before post = LTP
        plasticity.apply_stdp(idx, 10, 30);

        let synapse = plasticity.get_synapse(idx).unwrap();
        assert!(synapse.weight > 0.5);
    }

    #[test]
    fn test_stdp_ltd() {
        let mut plasticity = SynapticPlasticity::new();
        let idx = plasticity.create_synapse(NeuronId::new(1), NeuronId::new(2), 0.5);

        // Post before pre = LTD
        plasticity.apply_stdp(idx, 30, 10);

        let synapse = plasticity.get_synapse(idx).unwrap();
        assert!(synapse.weight < 0.5);
    }

    #[test]
    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn test_stdp_no_change_outside_window() {
        let mut plasticity = SynapticPlasticity::new();
        let idx = plasticity.create_synapse(NeuronId::new(1), NeuronId::new(2), 0.5);

        // Outside STDP window
        plasticity.apply_stdp(idx, 0, 200);

        let synapse = plasticity.get_synapse(idx).unwrap();
        assert_eq!(synapse.weight, 0.5);
    }

    #[test]
    fn test_calcium_ltp() {
        let mut plasticity = SynapticPlasticity::new();
        let idx = plasticity.create_synapse(NeuronId::new(1), NeuronId::new(2), 0.5);

        // Set high calcium for LTP
        plasticity.synapses[idx].calcium_level = 0.9;
        plasticity.apply_calcium_plasticity(idx);

        let synapse = plasticity.get_synapse(idx).unwrap();
        assert!(synapse.weight > 0.5);
        assert!(synapse.tag.is_ltp());
    }

    #[test]
    fn test_calcium_ltd() {
        let mut plasticity = SynapticPlasticity::new();
        let idx = plasticity.create_synapse(NeuronId::new(1), NeuronId::new(2), 0.5);

        // Set moderate calcium for LTD
        plasticity.synapses[idx].calcium_level = 0.5;
        plasticity.apply_calcium_plasticity(idx);

        let synapse = plasticity.get_synapse(idx).unwrap();
        assert!(synapse.weight < 0.5);
        assert!(synapse.tag.is_ltd());
    }

    #[test]
    fn test_calcium_no_change_low() {
        let mut plasticity = SynapticPlasticity::new();
        let idx = plasticity.create_synapse(NeuronId::new(1), NeuronId::new(2), 0.5);

        // Low calcium - no change
        plasticity.synapses[idx].calcium_level = 0.1;
        plasticity.apply_calcium_plasticity(idx);

        let synapse = plasticity.get_synapse(idx).unwrap();
        assert_eq!(synapse.weight, 0.5);
        // TAG: surface=simulation owner=research-team rule=SIM-001
    }

    #[test]
    fn test_consolidation() {
        let mut plasticity = SynapticPlasticity::new();
        let idx = plasticity.create_synapse(NeuronId::new(1), NeuronId::new(2), 0.5);

        // Tag as early LTP and age sufficiently
        plasticity.synapses[idx].tag = PlasticityTag::EarlyLTPTag;
        plasticity.synapses[idx].age_ticks = plasticity.consolidation_params.consolidation_ticks;
        plasticity.synapses[idx].weight = 0.7;

        plasticity.consolidate();

        let synapse = plasticity.get_synapse(idx).unwrap();
        assert_eq!(synapse.tag, PlasticityTag::LateLTPTag);
        assert_eq!(synapse.initial_weight, 0.7); // Baseline reset
    }

    #[test]
    fn test_homeostatic_scaling() {
        let mut plasticity = SynapticPlasticity::new();
        plasticity.homeostatic_params.target_rate = 10.0;
        plasticity.homeostatic_params.min_scale = 0.5;
        plasticity.homeostatic_params.max_scale = 2.0;

        // Add synapses with known weights
        plasticity.create_synapse(NeuronId::new(1), NeuronId::new(2), 1.0);
        plasticity.create_synapse(NeuronId::new(2), NeuronId::new(3), 1.0);

        // Scale for measured rate of 5 Hz (should double weights)
        plasticity.apply_homeostatic_scaling(5.0);

        assert!(plasticity.synapses[0].weight > 1.5);
        assert!(plasticity.synapses[1].weight > 1.5);
    }

    #[test]
    fn test_metaplasticity_high_activity() {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let mut plasticity = SynapticPlasticity::new();
        plasticity.learning_rate = 0.01;
        plasticity.effective_learning_rate = 0.01;

        // Simulate high activity
        for _ in 0..100 {
            plasticity.record_calcium(0.9);
        }

        plasticity.update_metaplasticity();

        // Should decrease learning rate
        assert!(plasticity.effective_learning_rate < 0.01);
    }

    #[test]
    fn test_metaplasticity_low_activity() {
        let mut plasticity = SynapticPlasticity::new();
        plasticity.learning_rate = 0.01;
        plasticity.effective_learning_rate = 0.01;

        // Simulate low activity
        for _ in 0..100 {
            plasticity.record_calcium(0.1);
        }

        plasticity.update_metaplasticity();

        // Should increase learning rate
        assert!(plasticity.effective_learning_rate > 0.01);
    }

    #[test]
    fn test_tick_decay() {
        let mut plasticity = SynapticPlasticity::new();
        let idx = plasticity.create_synapse(NeuronId::new(1), NeuronId::new(2), 0.5);
        plasticity.synapses[idx].calcium_level = 1.0;

        plasticity.tick();

        assert!(plasticity.synapses[idx].calcium_level < 1.0);
    }

    #[test]
    fn test_stats() {
        let mut plasticity = SynapticPlasticity::new();

        // Create synapses with different states
        let idx1 = plasticity.create_synapse(NeuronId::new(1), NeuronId::new(2), 0.5);
        // TAG: surface=simulation owner=research-team rule=SIM-001
        plasticity.synapses[idx1].weight = 0.7; // Potentiated
        plasticity.synapses[idx1].tag = PlasticityTag::EarlyLTPTag;

        let idx2 = plasticity.create_synapse(NeuronId::new(2), NeuronId::new(3), 0.5);
        plasticity.synapses[idx2].weight = 0.3; // Depressed
        plasticity.synapses[idx2].tag = PlasticityTag::LateLTDTag;

        let stats = plasticity.stats();

        assert_eq!(stats.total_synapses, 2);
        assert_eq!(stats.potentiated_count, 1);
        assert_eq!(stats.depressed_count, 1);
        assert_eq!(stats.early_ltp_count, 1);
        assert_eq!(stats.late_ltd_count, 1);
    }

    #[test]
    fn test_weight_normalization() {
        let mut plasticity = SynapticPlasticity::new();
        plasticity.create_synapse(NeuronId::new(1), NeuronId::new(2), 1.0);
        plasticity.create_synapse(NeuronId::new(2), NeuronId::new(3), 1.0);

        plasticity.normalize_weights(1.0);

        // Total weight should be 1.0
        let total = plasticity.total_weight();
        assert!((total - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_prune_weak_synapses() {
        let mut plasticity = SynapticPlasticity::new();
        plasticity.create_synapse(NeuronId::new(1), NeuronId::new(2), 0.1);
        plasticity.create_synapse(NeuronId::new(2), NeuronId::new(3), 0.5);
        plasticity.create_synapse(NeuronId::new(3), NeuronId::new(4), 0.05);

        let pruned = plasticity.prune_weak_synapses(0.1);

        assert_eq!(pruned, 1);
        assert_eq!(plasticity.synapses.len(), 2);
    }

    #[test]
    fn test_get_synapses_to_from() {
        let mut plasticity = SynapticPlasticity::new();
        let n1 = NeuronId::new(1);
        let n2 = NeuronId::new(2);
        let n3 = NeuronId::new(3);

        plasticity.create_synapse(n1, n2, 0.5);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        plasticity.create_synapse(n1, n3, 0.6);
        plasticity.create_synapse(n2, n3, 0.7);

        let to_n3 = plasticity.get_synapses_to(n3);
        assert_eq!(to_n3.len(), 2);

        let from_n1 = plasticity.get_synapses_from(n1);
        assert_eq!(from_n1.len(), 2);
    }

    #[test]
    fn test_plasticity_builder() {
        let plasticity = PlasticityBuilder::new()
            .for_agent(42)
            .with_learning_rate(0.05)
            .add_synapse(Synapse::new(NeuronId::new(1), NeuronId::new(2), 0.5))
            .build();

        assert_eq!(plasticity.agent_id, Some(42));
        assert_eq!(plasticity.learning_rate, 0.05);
        assert_eq!(plasticity.synapses.len(), 1);
    }

    #[test]
    fn test_apply_hebbian_bulk() {
        let mut plasticity = SynapticPlasticity::new();
        let pre1 = NeuronId::new(1);
        let pre2 = NeuronId::new(2);
        let post1 = NeuronId::new(3);
        let post2 = NeuronId::new(4);

        plasticity.create_synapse(pre1, post1, 0.5);
        plasticity.create_synapse(pre1, post2, 0.5);
        plasticity.create_synapse(pre2, post1, 0.5);

        plasticity.apply_hebbian_bulk(&[pre1, pre2], &[post1, post2], &[1.0, 0.5], &[1.0, 0.5]);

        // synapse pre1->post1: 1.0 * 1.0 = 1.0 contribution
        assert!(plasticity.get_synapse(0).unwrap().weight > 0.5);
    }
}
