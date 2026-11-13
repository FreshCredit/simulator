//! Glutamate Receptor Dynamics Module
//!
//! Implements detailed synaptic modeling with AMPA, NMDA, and `GABA_A` receptors.
//! These receptors mediate excitatory and inhibitory synaptic transmission,
//! determining synaptic strength and plasticity.
// TAG: surface=simulation owner=research-team rule=SIM-001
//!
//! # Key Concepts
//!
//! ## AMPA Receptors (Fast Excitation)
//! - Fast EPSC with ~2-10ms decay time constant
//! - Mediates most excitatory synaptic transmission in the brain
//! - Permeable to Na+ and K+ (reversal potential ~0mV)
//! - Undergoes rapid desensitization during sustained glutamate exposure
//!
//! ## NMDA Receptors (Slow, Voltage-Dependent)
//! - Slow EPSC with ~100-300ms decay time constant
//! - Requires postsynaptic depolarization to unblock Mg2+ channel block
//! - Permeable to Ca2+ (critical for synaptic plasticity/LTP)
//! - Provides coincidence detection for Hebbian learning
//!
//! ## `GABA_A` Receptors (Inhibition)
//! - Inhibitory postsynaptic current (IPSC) with ~10-50ms decay
//! - Cl- channel (reversal potential ~-70mV)
//! - Provides shunting inhibition and membrane stabilization
//!
//! # Research References
//! - Jahr & Stevens (1990) - Voltage dependence of NMDA conductance
//! - Hestrin (1992) - Activation and desensitization of AMPA receptors
//! - Mody et al. (1994) - `GABA_A` receptor physiology
//! - Lüscher et al. (2000) - Synaptic AMPA receptor dynamics

use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Glutamate receptor system for synaptic transmission
///
/// Manages populations of AMPA, NMDA, and `GABA_A` receptors at a synapse,
/// along with extracellular neurotransmitter concentrations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlutamateSystem {
    /// AMPA receptor population (fast excitation)
    pub ampa_receptors: Vec<AMPAR>,
    /// NMDA receptor population (slow, voltage-dependent)
    pub nmda_receptors: Vec<NMDAR>,
    /// `GABA_A` receptor population (inhibition)
    pub gaba_receptors: Vec<GABAR>,
    /// Extracellular glutamate concentration (arbitrary units, 0-1 normalized)
    pub glutamate_concentration: f32,
    /// Extracellular GABA concentration (arbitrary units, 0-1 normalized)
    pub gaba_concentration: f32,
    /// Synaptic cleft glutamate clearance rate
    pub glutamate_clearance_rate: f32,
    /// Synaptic cleft GABA clearance rate
    pub gaba_clearance_rate: f32,
    /// Vesicular glutamate release probability (presynaptic)
    pub glutamate_release_prob: f32,
    /// Vesicular GABA release probability (presynaptic)
    pub gaba_release_prob: f32,
}

impl GlutamateSystem {
    /// Create a new glutamate system with specified receptor counts
    ///
    /// # Arguments
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// * `n_ampa` - Number of AMPA receptors (typical: 50-200)
    /// * `n_nmda` - Number of NMDA receptors (typical: 10-40)
    /// * `n_gaba` - Number of `GABA_A` receptors (typical: 20-60)
    #[must_use]
    pub fn new(n_ampa: usize, n_nmda: usize, n_gaba: usize) -> Self {
        let ampa_receptors: Vec<AMPAR> = (0..n_ampa).map(|i| AMPAR::new(i as u64)).collect();

        let nmda_receptors: Vec<NMDAR> = (0..n_nmda).map(|i| NMDAR::new(i as u64)).collect();

        let gaba_receptors: Vec<GABAR> = (0..n_gaba).map(|i| GABAR::new(i as u64)).collect();

        Self {
            ampa_receptors,
            nmda_receptors,
            gaba_receptors,
            glutamate_concentration: 0.0,
            gaba_concentration: 0.0,
            glutamate_clearance_rate: 0.9,
            gaba_clearance_rate: 0.85,
            glutamate_release_prob: 0.3,
            gaba_release_prob: 0.25,
        }
    }

    /// Create with default receptor counts for a typical excitatory synapse
    #[must_use]
    pub fn excitatory_synapse() -> Self {
        Self::new(100, 20, 0)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create with default receptor counts for an inhibitory synapse
    #[must_use]
    pub fn inhibitory_synapse() -> Self {
        Self::new(0, 0, 30)
    }

    /// Create with default receptor counts for a mixed synapse
    #[must_use]
    pub fn mixed_synapse() -> Self {
        Self::new(100, 20, 30)
    }

    /// Release glutamate from presynaptic terminal (simulates vesicle release)
    ///
    /// # Arguments
    /// * `amount` - Amount of glutamate released (0-1, where 1 = saturating)
    pub fn release_glutamate(&mut self, amount: f32) {
        let effective_amount = amount * self.glutamate_release_prob;
        self.glutamate_concentration = (self.glutamate_concentration + effective_amount).min(1.0);

        // Bind to AMPA receptors (fast activation)
        for receptor in &mut self.ampa_receptors {
            let binding_prob = self.glutamate_concentration * 0.8;
            receptor.open_probability = (receptor.open_probability + binding_prob).min(1.0);
            receptor.desensitization = (receptor.desensitization + binding_prob * 0.3).min(1.0);
        }

        // Bind to NMDA receptors (slower activation, requires depolarization)
        for receptor in &mut self.nmda_receptors {
            let binding_prob = self.glutamate_concentration * 0.2;
            receptor.open_probability = (receptor.open_probability + binding_prob).min(1.0);
        }
    }

    /// Release GABA from presynaptic terminal (inhibitory interneuron)
    ///
    /// # Arguments
    /// * `amount` - Amount of GABA released (0-1, where 1 = saturating)
    pub fn release_gaba(&mut self, amount: f32) {
        let effective_amount = amount * self.gaba_release_prob;
        self.gaba_concentration = (self.gaba_concentration + effective_amount).min(1.0);

        for receptor in &mut self.gaba_receptors {
            let binding_prob = self.gaba_concentration * 0.6;
            receptor.open_probability = (receptor.open_probability + binding_prob).min(1.0);
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Simulate a presynaptic spike (action potential)
    ///
    /// This triggers vesicular release of neurotransmitters based on
    /// current release probabilities.
    ///
    /// # Arguments
    /// * `is_glutamatergic` - True if excitatory neuron, false if `GABAergic`
    pub fn presynaptic_spike(&mut self, is_glutamatergic: bool) {
        if is_glutamatergic {
            // Glutamatergic neurons release glutamate
            self.release_glutamate(1.0);
        } else {
            // GABAergic neurons release GABA
            self.release_gaba(1.0);
        }
    }

    /// Compute total synaptic current
    ///
    /// Combines contributions from AMPA, NMDA, and `GABA_A` receptors.
    /// NMDA current is voltage-dependent due to Mg2+ block.
    ///
    /// # Arguments
    /// * `membrane_potential` - Postsynaptic membrane potential in mV
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    ///
    /// # Returns
    /// Total synaptic current in normalized units (positive = excitatory, negative = inhibitory)
    #[must_use]
    pub fn synaptic_current(&self, membrane_potential: f32) -> f32 {
        let ampa_current: f32 = self
            .ampa_receptors
            .iter()
            .map(|r| r.current(membrane_potential))
            .sum();

        let nmda_current: f32 = self
            .nmda_receptors
            .iter()
            .map(|r| r.current(membrane_potential))
            .sum();

        let gaba_current: f32 = self
            .gaba_receptors
            .iter()
            .map(|r| r.current(membrane_potential))
            .sum();

        // Under Kirchhoff's Current Law / Hodgkin-Huxley convention,
        // membrane current is the sum of all receptor currents:
        // I_syn = I_AMPA + I_NMDA + I_GABA
        // At V_m in (-70, 0) mV, AMPA and NMDA currents are negative (inward/depolarizing)
        // while GABA current is positive (outward/hyperpolarizing), naturally opposing each other.
        ampa_current + nmda_current + gaba_current
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute excitatory current only (AMPA + NMDA)
    ///
    /// # Arguments
    /// * `membrane_potential` - Postsynaptic membrane potential in mV
    #[must_use]
    pub fn excitatory_current(&self, membrane_potential: f32) -> f32 {
        let ampa: f32 = self
            .ampa_receptors
            .iter()
            .map(|r| r.current(membrane_potential))
            .sum();
        let nmda: f32 = self
            .nmda_receptors
            .iter()
            .map(|r| r.current(membrane_potential))
            .sum();
        ampa + nmda
    }

    /// Compute inhibitory current only (`GABA_A`)
    ///
    /// # Arguments
    /// * `membrane_potential` - Postsynaptic membrane potential in mV
    #[must_use]
    pub fn inhibitory_current(&self, membrane_potential: f32) -> f32 {
        self.gaba_receptors
            .iter()
            .map(|r| r.current(membrane_potential))
            .sum()
    }

    /// Compute NMDA/AMPA ratio (measure of synaptic plasticity potential)
    ///
    /// Higher ratios indicate greater potential for Hebbian plasticity
    /// due to increased Ca2+ influx through NMDA receptors.
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[must_use]
    pub fn nmda_ampa_ratio(&self) -> f32 {
        let total_nmda_conductance: f32 = self.nmda_receptors.iter().map(|r| r.conductance).sum();
        let total_ampa_conductance: f32 = self.ampa_receptors.iter().map(|r| r.conductance).sum();

        if total_ampa_conductance > 0.0 {
            total_nmda_conductance / total_ampa_conductance
        } else if total_nmda_conductance > 0.0 {
            f32::INFINITY
        } else {
            0.0
        }
    }

    /// Update receptor kinetics (decay and desensitization recovery)
    ///
    /// # Arguments
    /// * `dt` - Time step in milliseconds
    pub fn update(&mut self, dt: f32) {
        // Clear neurotransmitters from synaptic cleft
        self.glutamate_concentration *= self.glutamate_clearance_rate.max(0.0).powf(dt / 1.0);
        self.gaba_concentration *= self.gaba_clearance_rate.max(0.0).powf(dt / 1.0);

        // Update AMPA receptor kinetics
        for r in &mut self.ampa_receptors {
            r.update(dt);
        }

        // Update NMDA receptor kinetics
        for r in &mut self.nmda_receptors {
            r.update(dt);
        }

        // Update GABA receptor kinetics
        for r in &mut self.gaba_receptors {
            r.update(dt);
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Reset all receptors to closed state
    pub fn reset(&mut self) {
        self.glutamate_concentration = 0.0;
        self.gaba_concentration = 0.0;

        for r in &mut self.ampa_receptors {
            r.open_probability = 0.0;
            r.desensitization = 0.0;
        }

        for r in &mut self.nmda_receptors {
            r.open_probability = 0.0;
            r.mg_block = 1.0; // Full block at rest
        }

        for r in &mut self.gaba_receptors {
            r.open_probability = 0.0;
        }
    }

    /// Get average open probability across receptor populations
    #[must_use]
    pub fn average_open_probabilities(&self) -> (f32, f32, f32) {
        let ampa_avg = if self.ampa_receptors.is_empty() {
            0.0
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        } else {
            self.ampa_receptors
                .iter()
                .map(|r| r.open_probability)
                .sum::<f32>()
                / self.ampa_receptors.len() as f32
        };

        let nmda_avg = if self.nmda_receptors.is_empty() {
            0.0
        } else {
            self.nmda_receptors
                .iter()
                .map(|r| r.open_probability)
                .sum::<f32>()
                / self.nmda_receptors.len() as f32
        };

        let gaba_avg = if self.gaba_receptors.is_empty() {
            0.0
        } else {
            self.gaba_receptors
                .iter()
                .map(|r| r.open_probability)
                .sum::<f32>()
                / self.gaba_receptors.len() as f32
        };

        (ampa_avg, nmda_avg, gaba_avg)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get total receptor counts
    #[must_use]
    pub fn receptor_counts(&self) -> (usize, usize, usize) {
        (
            self.ampa_receptors.len(),
            self.nmda_receptors.len(),
            self.gaba_receptors.len(),
        )
    }
}

impl Default for GlutamateSystem {
    fn default() -> Self {
        Self::new(100, 20, 30)
    }
}

/// AMPA receptor (fast excitatory synaptic transmission)
///
/// AMPA receptors mediate fast excitatory postsynaptic currents (EPSCs)
/// with rapid activation and desensitization kinetics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AMPAR {
    /// Unique receptor identifier
    pub id: u64,
    /// Single-channel conductance (normalized, typical: 10-20 pS)
    pub conductance: f32,
    /// Probability of channel being open [0, 1]
    pub open_probability: f32,
    /// Desensitization state [0, 1], reduces response to sustained glutamate
    pub desensitization: f32,
    /// Decay time constant in milliseconds (typical: 2-10ms)
    pub tau_decay: f32,
    /// Desensitization recovery time constant in milliseconds (typical: 50-200ms)
    pub tau_recovery: f32,
    /// Reversal potential in mV (near 0mV for Na+/K+ permeability)
    pub reversal_potential: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl AMPAR {
    /// Create a new AMPA receptor with default parameters
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self {
            id,
            conductance: 0.01, // Normalized conductance
            open_probability: 0.0,
            desensitization: 0.0,
            tau_decay: 5.0,          // 5ms fast decay
            tau_recovery: 100.0,     // 100ms recovery from desensitization
            reversal_potential: 0.0, // mV
        }
    }

    /// Compute receptor current
    ///
    /// # Arguments
    /// * `v` - Membrane potential in mV
    ///
    /// # Returns
    /// Current in normalized units
    #[must_use]
    pub fn current(&self, v: f32) -> f32 {
        // Effective open probability reduced by desensitization
        let effective_open = self.open_probability * (1.0 - self.desensitization);
        self.conductance * effective_open * (v - self.reversal_potential)
    }

    /// Update receptor kinetics
    ///
    /// # Arguments
    /// * `dt` - Time step in milliseconds
    pub fn update(&mut self, dt: f32) {
        // Decay open probability (channel closing)
        let decay_factor = (-dt / self.tau_decay).exp();
        self.open_probability *= decay_factor;

        // Recovery from desensitization
        let recovery_factor = (-dt / self.tau_recovery).exp();
        self.desensitization *= recovery_factor;
    }

    /// Set custom kinetics parameters
    pub fn set_kinetics(&mut self, tau_decay: f32, tau_recovery: f32) {
        self.tau_decay = tau_decay.clamp(1.0, 50.0);
        self.tau_recovery = tau_recovery.clamp(10.0, 500.0);
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for AMPAR {
    fn default() -> Self {
        Self::new(0)
    }
}

/// NMDA receptor (slow, voltage-dependent excitatory transmission)
///
/// NMDA receptors mediate slow EPSCs and are critical for synaptic plasticity.
/// They require both glutamate binding AND postsynaptic depolarization
/// (to relieve Mg2+ block) to open.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NMDAR {
    /// Unique receptor identifier
    pub id: u64,
    /// Single-channel conductance (normalized, typical: 40-50 pS)
    pub conductance: f32,
    /// Magnesium block factor [0, 1], decreases with depolarization
    pub mg_block: f32,
    /// Probability of channel being open [0, 1]
    pub open_probability: f32,
    /// Decay time constant in milliseconds (typical: 100-300ms)
    pub tau_decay: f32,
    /// Reversal potential in mV (near 0mV, but Ca2+ permeable)
    pub reversal_potential: f32,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Mg2+ concentration factor (affects voltage dependence)
    pub mg_concentration: f32,
}

impl NMDAR {
    /// Create a new NMDA receptor with default parameters
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self {
            id,
            conductance: 0.02, // Normalized conductance (higher than AMPA per channel)
            mg_block: 1.0,     // Fully blocked at rest (~-70mV)
            open_probability: 0.0,
            tau_decay: 150.0,        // 150ms slow decay
            reversal_potential: 0.0, // mV
            mg_concentration: 1.0,   // mM Mg2+
        }
    }

    /// Compute Mg2+ block factor based on membrane potential
    ///
    /// Uses the Jahr & Stevens (1990) model:
    /// block = 1 / (1 + [Mg2+] * exp(-0.062 * V) / 3.57)
    ///
    /// # Arguments
    /// * `v` - Membrane potential in mV
    fn compute_mg_block(&self, v: f32) -> f32 {
        // Jahr & Stevens model for Mg2+ block
        // block increases with hyperpolarization (negative V)
        let exp_term = (-0.062 * v).exp();
        let block = 1.0 / (1.0 + self.mg_concentration * exp_term / 3.57);
        block.clamp(0.0, 1.0)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute receptor current
    ///
    /// # Arguments
    /// * `v` - Membrane potential in mV
    ///
    /// # Returns
    /// Current in normalized units
    #[must_use]
    pub fn current(&self, v: f32) -> f32 {
        // Update Mg2+ block based on voltage
        let mg_unblock = self.compute_mg_block(v);
        // Current only flows when unblocked and open
        self.conductance * self.open_probability * mg_unblock * (v - self.reversal_potential)
    }

    /// Update receptor kinetics
    ///
    /// # Arguments
    /// * `dt` - Time step in milliseconds
    pub fn update(&mut self, dt: f32) {
        // Decay open probability (slow channel closing)
        let decay_factor = (-dt / self.tau_decay).exp();
        self.open_probability *= decay_factor;

        // Update Mg2+ block (fast, essentially instantaneous)
        // This is handled in current() for accuracy
    }

    /// Set custom kinetics parameters
    pub fn set_kinetics(&mut self, tau_decay: f32, mg_concentration: f32) {
        self.tau_decay = tau_decay.clamp(50.0, 500.0);
        self.mg_concentration = mg_concentration.clamp(0.1, 10.0);
    }

    /// Check if receptor is "unblocked" (potential for Ca2+ influx)
    ///
    /// Returns true if membrane potential is depolarized enough
    /// to significantly relieve Mg2+ block.
    ///
    /// # Arguments
    /// * `v` - Membrane potential in mV
    #[must_use]
    pub fn is_unblocked(&self, v: f32) -> bool {
        self.compute_mg_block(v) > 0.3
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for NMDAR {
    fn default() -> Self {
        Self::new(0)
    }
}

/// `GABA_A` receptor (inhibitory synaptic transmission)
///
/// `GABA_A` receptors are ligand-gated Cl- channels that mediate
/// fast inhibitory postsynaptic currents (IPSCs).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GABAR {
    /// Unique receptor identifier
    pub id: u64,
    /// Single-channel conductance (normalized, typical: 10-30 pS)
    pub conductance: f32,
    /// Cl- reversal potential in mV (typical: -70 to -65 mV)
    pub reversal_potential: f32,
    /// Probability of channel being open [0, 1]
    pub open_probability: f32,
    /// Decay time constant in milliseconds (typical: 10-50ms)
    pub tau_decay: f32,
    /// Receptor modulation factor (e.g., by benzodiazepines) [0.5, 2.0]
    pub modulation: f32,
}

impl GABAR {
    /// Create a new `GABA_A` receptor with default parameters
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self {
            id,
            conductance: 0.008,        // Normalized conductance
            reversal_potential: -70.0, // mV (typical Cl- reversal)
            open_probability: 0.0,
            tau_decay: 25.0, // 25ms intermediate decay
            modulation: 1.0, // No modulation by default
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute receptor current
    ///
    /// Note: Current is typically negative (inward Cl- flow) when
    /// membrane potential is above reversal potential.
    ///
    /// # Arguments
    /// * `v` - Membrane potential in mV
    ///
    /// # Returns
    /// Current in normalized units
    #[must_use]
    pub fn current(&self, v: f32) -> f32 {
        let effective_conductance = self.conductance * self.modulation;
        effective_conductance * self.open_probability * (v - self.reversal_potential)
    }

    /// Update receptor kinetics
    ///
    /// # Arguments
    /// * `dt` - Time step in milliseconds
    pub fn update(&mut self, dt: f32) {
        // Decay open probability
        let decay_factor = (-dt / self.tau_decay).exp();
        self.open_probability *= decay_factor;
    }

    /// Set custom kinetics parameters
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub fn set_kinetics(&mut self, tau_decay: f32, reversal_potential: f32) {
        self.tau_decay = tau_decay.clamp(5.0, 100.0);
        self.reversal_potential = reversal_potential.clamp(-90.0, -40.0);
    }

    /// Apply allosteric modulation (e.g., by benzodiazepines or neurosteroids)
    ///
    /// # Arguments
    /// * `modulation_factor` - Multiplier for conductance [0.5, 2.0]
    ///   where >1 enhances and <1 reduces inhibition
    pub fn modulate(&mut self, modulation_factor: f32) {
        self.modulation = modulation_factor.clamp(0.5, 2.0);
    }
}

impl Default for GABAR {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Synaptic plasticity state tracking
///
/// Tracks receptor dynamics relevant for long-term potentiation (LTP)
/// and long-term depression (LTD).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlasticityState {
    /// Recent NMDA receptor activation (Ca2+ proxy)
    pub nmda_activation_history: Vec<f32>,
    /// Postsynaptic depolarization history
    pub depolarization_history: Vec<f32>,
    /// Eligibility trace for plasticity
    pub eligibility_trace: f32,
    /// Last update timestamp
    pub last_update_ms: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl PlasticityState {
    /// Create a new plasticity tracker
    #[must_use]
    pub fn new() -> Self {
        Self {
            nmda_activation_history: Vec::with_capacity(100),
            depolarization_history: Vec::with_capacity(100),
            eligibility_trace: 0.0,
            last_update_ms: 0.0,
        }
    }

    /// Record NMDA activation for plasticity computation
    ///
    /// # Arguments
    /// * `nmda_current` - NMDA receptor current (proxy for Ca2+ influx)
    /// * `timestamp_ms` - Current simulation time in milliseconds
    pub fn record_nmda_activation(&mut self, nmda_current: f32, timestamp_ms: f32) {
        // NMDA current at subthreshold potentials is inward/negative;
        // record its magnitude as Ca2+ influx proxy so ltp_conditions_met evaluates correctly.
        self.nmda_activation_history.push(nmda_current.abs());
        if self.nmda_activation_history.len() > 100 {
            self.nmda_activation_history.remove(0);
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        self.last_update_ms = timestamp_ms;
    }

    /// Update eligibility trace for STDP
    ///
    /// # Arguments
    /// * `pre_post_timing` - Time difference (post - pre) in milliseconds
    /// * `learning_rate` - Learning rate for trace update
    pub fn update_eligibility_trace(&mut self, pre_post_timing: f32, learning_rate: f32) {
        // Standard STDP: positive timing = LTP, negative = LTD
        let stdp_window = if pre_post_timing > 0.0 {
            // Post after pre = LTP
            (20.0 - pre_post_timing).max(0.0) / 20.0
        } else {
            // Post before pre = LTD
            (-pre_post_timing).min(20.0) / 20.0 * -0.5
        };

        self.eligibility_trace += learning_rate * stdp_window;
        self.eligibility_trace = self.eligibility_trace.clamp(-1.0, 1.0);

        // Decay eligibility trace over time
        self.eligibility_trace *= 0.995;
    }

    /// Check if LTP conditions are met
    ///
    /// Requires sustained NMDA activation and postsynaptic depolarization
    #[must_use]
    pub fn ltp_conditions_met(&self, threshold: f32) -> bool {
        let recent_nmda: f32 = self.nmda_activation_history.iter().rev().take(10).sum();
        recent_nmda > threshold && self.eligibility_trace > 0.3
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if LTD conditions are met
    #[must_use]
    pub fn ltd_conditions_met(&self, threshold: f32) -> bool {
        let recent_nmda: f32 = self.nmda_activation_history.iter().rev().take(10).sum();
        recent_nmda < threshold && self.eligibility_trace < -0.3
    }
}

/// Statistics for glutamate system
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GlutamateStatistics {
    /// Number of AMPA receptors
    pub ampa_count: usize,
    /// Number of NMDA receptors
    pub nmda_count: usize,
    /// Number of GABA receptors
    pub gaba_count: usize,
    /// Average AMPA open probability
    pub ampa_open_avg: f32,
    /// Average NMDA open probability
    pub nmda_open_avg: f32,
    /// Average GABA open probability
    pub gaba_open_avg: f32,
    /// Current glutamate concentration
    pub glutamate_concentration: f32,
    /// Current GABA concentration
    pub gaba_concentration: f32,
    /// NMDA/AMPA conductance ratio
    pub nmda_ampa_ratio: f32,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glutamate_system_creation() {
        let system = GlutamateSystem::new(100, 20, 30);

        assert_eq!(system.ampa_receptors.len(), 100);
        assert_eq!(system.nmda_receptors.len(), 20);
        assert_eq!(system.gaba_receptors.len(), 30);
        assert_eq!(system.glutamate_concentration, 0.0);
        assert_eq!(system.gaba_concentration, 0.0);
    }

    #[test]
    fn test_excitatory_synapse() {
        let system = GlutamateSystem::excitatory_synapse();

        assert!(!system.ampa_receptors.is_empty());
        assert!(!system.nmda_receptors.is_empty());
        assert!(system.gaba_receptors.is_empty()); // Pure excitatory
    }

    #[test]
    fn test_inhibitory_synapse() {
        let system = GlutamateSystem::inhibitory_synapse();

        assert!(system.ampa_receptors.is_empty());
        assert!(system.nmda_receptors.is_empty());
        assert!(!system.gaba_receptors.is_empty());
    }

    #[test]
    fn test_glutamate_release() {
        let mut system = GlutamateSystem::excitatory_synapse();

        system.release_glutamate(0.5);

        assert!(system.glutamate_concentration > 0.0);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // AMPA receptors should have non-zero open probability
        let ampa_avg = system
            .ampa_receptors
            .iter()
            .map(|r| r.open_probability)
            .sum::<f32>()
            / system.ampa_receptors.len() as f32;
        assert!(ampa_avg > 0.0);
    }

    #[test]
    fn test_gaba_release() {
        let mut system = GlutamateSystem::inhibitory_synapse();

        system.release_gaba(0.5);

        assert!(system.gaba_concentration > 0.0);

        // GABA receptors should have non-zero open probability
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let gaba_avg = system
            .gaba_receptors
            .iter()
            .map(|r| r.open_probability)
            .sum::<f32>()
            / system.gaba_receptors.len() as f32;
        assert!(gaba_avg > 0.0);
    }

    #[test]
    fn test_presynaptic_spike() {
        let mut system = GlutamateSystem::excitatory_synapse();

        system.presynaptic_spike(true);
        assert!(system.glutamate_concentration > 0.0);

        let mut system_inh = GlutamateSystem::inhibitory_synapse();
        system_inh.presynaptic_spike(false);
        assert!(system_inh.gaba_concentration > 0.0);
    }

    #[test]
    fn test_ampa_current() {
        let receptor = AMPAR {
            open_probability: 0.5,
            ..AMPAR::default()
        };

        // At 0mV (reversal), current should be 0
        let current_0mv = receptor.current(0.0);
        assert!(current_0mv.abs() < 1e-6);

        // At -70mV, current should be inward (negative)
        let current_neg = receptor.current(-70.0);
        assert!(current_neg < 0.0);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // At +40mV, current should be outward (positive)
        let current_pos = receptor.current(40.0);
        assert!(current_pos > 0.0);
    }

    #[test]
    fn test_nmda_voltage_dependence() {
        let receptor = NMDAR::default();

        // At hyperpolarized potential, Mg2+ block should be strong
        let block_hyp = receptor.compute_mg_block(-70.0);
        assert!(block_hyp < 0.1); // Mostly blocked

        // At depolarized potential, block should be relieved
        let block_dep = receptor.compute_mg_block(0.0);
        assert!(block_dep > 0.5); // Mostly unblocked

        // At very positive potential, near complete unblock
        let block_pos = receptor.compute_mg_block(40.0);
        assert!(block_pos > 0.9);
    }

    #[test]
    fn test_nmda_unblocked() {
        let receptor = NMDAR::default();

        // Should be blocked at rest
        assert!(!receptor.is_unblocked(-70.0));

        // Should be unblocked at depolarized potentials
        assert!(receptor.is_unblocked(0.0));
        assert!(receptor.is_unblocked(-30.0));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_gaba_current() {
        let receptor = GABAR {
            open_probability: 0.5,
            ..GABAR::default()
        };

        // At reversal potential (-70mV), current should be 0
        let current_rev = receptor.current(-70.0);
        assert!(current_rev.abs() < 1e-6);

        // At 0mV (above reversal), Cl- flows in (negative current)
        let current_0mv = receptor.current(0.0);
        assert!(current_0mv > 0.0); // v - e_rev = 0 - (-70) = 70, positive current

        // At -90mV (below reversal), Cl- flows out
        let current_neg = receptor.current(-90.0);
        assert!(current_neg < 0.0);
    }

    #[test]
    fn test_gaba_modulation() {
        let mut receptor = GABAR::default();

        // Default modulation is 1.0
        assert_eq!(receptor.modulation, 1.0);

        // Enhance with benzodiazepine-like effect
        receptor.modulate(1.5);
        assert_eq!(receptor.modulation, 1.5);

        // Reduce with inverse agonist
        receptor.modulate(0.5);
        assert_eq!(receptor.modulation, 0.5);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // Test clamping
        receptor.modulate(5.0);
        assert_eq!(receptor.modulation, 2.0);
    }

    #[test]
    fn test_synapse_current_computation() {
        let mut system = GlutamateSystem::mixed_synapse();

        // Set some receptors to open
        for r in &mut system.ampa_receptors {
            r.open_probability = 0.3;
        }
        for r in &mut system.nmda_receptors {
            r.open_probability = 0.2;
        }
        for r in &mut system.gaba_receptors {
            r.open_probability = 0.4;
        }

        // Compute total current at -50mV (above GABA reversal, so inhibitory current flows)
        let current = system.synaptic_current(-50.0);

        // Should have non-zero current
        assert!(current != 0.0);

        // Should have excitatory and inhibitory components
        let exc = system.excitatory_current(-50.0);
        let inh = system.inhibitory_current(-50.0);
        assert!(exc != 0.0);
        // At -50mV (above -70mV reversal), GABA current should be non-zero
        assert!(inh != 0.0);
    }
    #[test]
    fn test_gaba_opposes_excitatory_current() {
        let mut pure_exc = GlutamateSystem::excitatory_synapse();
        for r in &mut pure_exc.ampa_receptors {
            r.open_probability = 0.5;
        }
        let exc_only_current = pure_exc.synaptic_current(-50.0);
        assert!(
            exc_only_current < 0.0,
            "Excitatory current at -50mV must be negative (inward)"
        );

        let mut mixed = GlutamateSystem::mixed_synapse();
        for r in &mut mixed.ampa_receptors {
            r.open_probability = 0.5;
        }
        for r in &mut mixed.gaba_receptors {
            r.open_probability = 0.5;
        }
        let mixed_current = mixed.synaptic_current(-50.0);
        // GABA outward current at -50mV opposes inward AMPA current, bringing total current closer to 0
        assert!(
            mixed_current > exc_only_current,
            "GABA current must oppose inward excitatory current: mixed={mixed_current}, exc={exc_only_current}"
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001

    #[test]
    fn test_nmda_ampa_ratio() {
        let system = GlutamateSystem::excitatory_synapse();

        let ratio = system.nmda_ampa_ratio();

        // Should be finite and non-negative
        assert!(ratio >= 0.0);
        assert!(ratio.is_finite());
    }

    #[test]
    fn test_ampa_update() {
        let mut receptor = AMPAR {
            open_probability: 0.8,
            desensitization: 0.5,
            ..AMPAR::default()
        };

        receptor.update(5.0); // 5ms step

        // Open probability should decay
        assert!(receptor.open_probability < 0.8);
        // Desensitization should recover
        assert!(receptor.desensitization < 0.5);
    }

    #[test]
    fn test_nmda_update() {
        let mut receptor = NMDAR {
            open_probability: 0.8,
            ..NMDAR::default()
        };
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        receptor.update(50.0); // 50ms step

        // Should decay but slower than AMPA
        assert!(receptor.open_probability < 0.8);
        assert!(receptor.open_probability > 0.4); // Still significant at 50ms
    }

    #[test]
    fn test_system_update() {
        let mut system = GlutamateSystem::excitatory_synapse();

        system.release_glutamate(1.0);
        let initial_glut = system.glutamate_concentration;

        system.update(10.0); // 10ms step

        // Glutamate should be cleared
        assert!(system.glutamate_concentration < initial_glut);
    }

    #[test]
    fn test_reset() {
        let mut system = GlutamateSystem::mixed_synapse();

        system.release_glutamate(1.0);
        system.release_gaba(1.0);
        for r in &mut system.ampa_receptors {
            r.open_probability = 0.5;
        }

        system.reset();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(system.glutamate_concentration, 0.0);
        assert_eq!(system.gaba_concentration, 0.0);
        for r in &system.ampa_receptors {
            assert_eq!(r.open_probability, 0.0);
        }
    }

    #[test]
    fn test_plasticity_state() {
        let mut state = PlasticityState::new();

        assert_eq!(state.eligibility_trace, 0.0);

        // Record some NMDA activation
        state.record_nmda_activation(0.5, 0.0);
        state.record_nmda_activation(0.6, 10.0);

        assert_eq!(state.nmda_activation_history.len(), 2);

        // Update eligibility trace (LTP - post after pre)
        state.update_eligibility_trace(5.0, 0.1);
        assert!(state.eligibility_trace > 0.0);

        // Update with LTD timing (post before pre)
        let old_trace = state.eligibility_trace;
        state.update_eligibility_trace(-10.0, 0.1);
        assert!(state.eligibility_trace < old_trace);
    }

    #[test]
    fn test_receptor_counts() {
        let system = GlutamateSystem::new(50, 10, 20);

        let (ampa, nmda, gaba) = system.receptor_counts();
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert_eq!(ampa, 50);
        assert_eq!(nmda, 10);
        assert_eq!(gaba, 20);
    }

    #[test]
    fn test_average_open_probabilities() {
        let mut system = GlutamateSystem::mixed_synapse();

        // Set known open probabilities
        for r in &mut system.ampa_receptors {
            r.open_probability = 0.5;
        }
        for r in &mut system.nmda_receptors {
            r.open_probability = 0.3;
        }
        for r in &mut system.gaba_receptors {
            r.open_probability = 0.2;
        }

        let (ampa_avg, nmda_avg, gaba_avg) = system.average_open_probabilities();

        assert!((ampa_avg - 0.5).abs() < 0.01);
        assert!((nmda_avg - 0.3).abs() < 0.01);
        assert!((gaba_avg - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_ampa_kinetics() {
        let mut receptor = AMPAR::default();
        receptor.set_kinetics(3.0, 50.0);

        assert_eq!(receptor.tau_decay, 3.0);
        assert_eq!(receptor.tau_recovery, 50.0);
        // TAG: surface=simulation owner=research-team rule=SIM-001

        // Test clamping
        receptor.set_kinetics(0.5, 5.0);
        assert_eq!(receptor.tau_decay, 1.0); // Clamped to min
        assert_eq!(receptor.tau_recovery, 10.0); // Clamped to min
    }

    #[test]
    fn test_nmda_kinetics() {
        let mut receptor = NMDAR::default();
        receptor.set_kinetics(200.0, 2.0);

        assert_eq!(receptor.tau_decay, 200.0);
        assert_eq!(receptor.mg_concentration, 2.0);

        // Test clamping
        receptor.set_kinetics(600.0, 20.0);
        assert_eq!(receptor.tau_decay, 500.0); // Clamped to max
        assert_eq!(receptor.mg_concentration, 10.0); // Clamped to max
    }

    #[test]
    fn test_gaba_kinetics() {
        let mut receptor = GABAR::default();
        receptor.set_kinetics(15.0, -65.0);

        assert_eq!(receptor.tau_decay, 15.0);
        assert_eq!(receptor.reversal_potential, -65.0);

        // Test clamping
        receptor.set_kinetics(200.0, -30.0);
        assert_eq!(receptor.tau_decay, 100.0); // Clamped to max
        assert_eq!(receptor.reversal_potential, -40.0); // Clamped to max
    }
}
