//! Hodgkin-Huxley Ion Channel Models
//!
//! Detailed biophysical neuron models based on Hodgkin-Huxley (1952) formalism.
//! Provides voltage-gated Na+, K+, Ca2+ channels and additional currents for
//! realistic neuronal spiking dynamics in the Brian2 backend.
// TAG: surface=simulation owner=research-team rule=SIM-001
//!
//! # Architecture
//!
//! - `HodgkinHuxleyModel`: Core Na+/K+ model with leak current
//! - `ExtendedHHModel`: Adds Ca2+, A-type K+, M-type K+, and `KCa` currents
//! - Channel gating dynamics follow original HH rate equations
//! - Temperature-corrected kinetics (Q10 = 3.0)
//!
//! # References
//!
//! - Hodgkin & Huxley (1952): "A quantitative description of membrane current
//!   and its application to conduction and excitation in nerve"
//! - Traub & Miles (1991): Neuronal Networks of the Hippocampus
//! - Destexhe et al. (1998): Kinetic models of synaptic transmission

use serde::{Deserialize, Serialize};

/// Standard temperature for HH kinetics (Celsius)
pub const HH_TEMPERATURE: f32 = 6.3;
/// Q10 temperature coefficient
pub const HH_Q10: f32 = 3.0;
/// Faraday constant (C/mol)
pub const FARADAY: f32 = 96485.0;
/// Gas constant (J/(mol*K))
pub const GAS_CONSTANT: f32 = 8.314;

/// Gating variable type for channel kinetics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GatingType {
    /// Activation (m-like)
    Activation,
    /// Inactivation (h-like)
    Inactivation,
    /// Slow activation (n-like)
    SlowActivation,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Single ion channel type with HH gating
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IonChannel {
    /// Maximum conductance (mS/cm^2)
    pub g_max: f32,
    /// Reversal potential (mV)
    pub e_rev: f32,
    /// Gating power (e.g., m^3 for Na+ activation)
    pub gating_power: i32,
    /// Activation variable (0-1)
    pub m: f32,
    /// Inactivation variable (0-1) - optional
    pub h: Option<f32>,
}

impl IonChannel {
    /// Create new voltage-gated channel
    #[must_use]
    pub const fn new(g_max: f32, e_rev: f32, gating_power: i32, has_inactivation: bool) -> Self {
        Self {
            g_max,
            e_rev,
            gating_power,
            m: 0.0,
            h: if has_inactivation { Some(1.0) } else { None },
        }
    }

    /// Compute current through channel
    #[must_use]
    pub fn current(&self, v: f32) -> f32 {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let g = if let Some(h) = self.h {
            self.g_max * self.m.powi(self.gating_power) * h
        } else {
            self.g_max * self.m.powi(self.gating_power)
        };
        g * (v - self.e_rev)
    }

    /// Update activation gate
    pub fn update_activation(&mut self, dt: f32, alpha: f32, beta: f32) {
        let dm = alpha * (1.0 - self.m) - beta * self.m;
        self.m += dm * dt;
        self.m = self.m.clamp(0.0, 1.0);
    }

    /// Update inactivation gate
    pub fn update_inactivation(&mut self, dt: f32, alpha: f32, beta: f32) {
        if let Some(ref mut h) = self.h {
            let dh = alpha * (1.0 - *h) - beta * *h;
            *h += dh * dt;
            *h = h.clamp(0.0, 1.0);
        }
    }
}

/// Temperature-corrected rate constant
#[must_use]
pub fn temperature_correction(rate: f32, temp_c: f32, base_temp_c: f32) -> f32 {
    let q10_factor = HH_Q10.powf((temp_c - base_temp_c) / 10.0);
    rate * q10_factor
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Hodgkin-Huxley ion channel model
///
/// Classic squid giant axon model with fast Na+, delayed rectifier K+,
/// and leak currents. Supports temperature correction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HodgkinHuxleyModel {
    // Membrane properties
    /// Membrane potential (mV)
    pub membrane_potential: f32,
    /// Membrane capacitance (uF/cm^2)
    pub capacitance: f32,
    /// Temperature (Celsius)
    pub temperature: f32,

    // Conductances (mS/cm^2)
    /// Sodium maximum conductance
    pub g_na: f32,
    /// Potassium maximum conductance
    pub g_k: f32,
    /// Leak conductance
    pub g_l: f32,

    // Reversal potentials (mV)
    /// Sodium reversal potential
    pub e_na: f32,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Potassium reversal potential
    pub e_k: f32,
    /// Leak reversal potential
    pub e_l: f32,

    // Gating variables (0-1)
    /// Na+ activation (fast)
    pub m: f32,
    /// Na+ inactivation
    pub h: f32,
    /// K+ activation (slow, n^4)
    pub n: f32,

    // Spike detection
    /// Last membrane potential for threshold detection
    last_v: f32,
    /// Spike threshold (mV)
    spike_threshold: f32,
}

impl Default for HodgkinHuxleyModel {
    fn default() -> Self {
        Self::new()
    }
}

impl HodgkinHuxleyModel {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create new HH model with standard squid axon parameters
    #[must_use]
    pub fn new() -> Self {
        let mut model = Self {
            membrane_potential: -65.0,
            capacitance: 1.0,
            temperature: HH_TEMPERATURE,
            g_na: 120.0,
            g_k: 36.0,
            g_l: 0.3,
            e_na: 55.0,
            e_k: -90.0,
            e_l: -70.0,
            m: 0.0,
            h: 0.0,
            n: 0.0,
            last_v: -65.0,
            spike_threshold: -20.0,
        };
        // Initialize gating variables to steady state at resting potential
        model.initialize_gates();
        model
    }

    /// Create model with cortical pyramidal neuron parameters
    #[must_use]
    pub fn cortical_pyramidal() -> Self {
        let mut model = Self {
            membrane_potential: -70.0,
            capacitance: 1.0,
            temperature: 37.0, // Body temperature
            g_na: 100.0,
            g_k: 40.0,
            g_l: 0.25,
            e_na: 60.0,
            e_k: -90.0,
            e_l: -70.0,
            m: 0.0,
            h: 0.0,
            n: 0.0,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            last_v: -70.0,
            spike_threshold: -30.0,
        };
        model.initialize_gates();
        model
    }

    /// Create model with thalamic relay neuron parameters
    #[must_use]
    pub fn thalamic_relay() -> Self {
        let mut model = Self {
            membrane_potential: -65.0,
            capacitance: 1.0,
            temperature: 37.0,
            g_na: 90.0,
            g_k: 20.0,
            g_l: 0.15,
            e_na: 50.0,
            e_k: -95.0,
            e_l: -75.0,
            m: 0.0,
            h: 0.0,
            n: 0.0,
            last_v: -65.0,
            spike_threshold: -35.0,
        };
        model.initialize_gates();
        model
    }

    /// Initialize gating variables to steady state at current V
    fn initialize_gates(&mut self) {
        let v = self.membrane_potential;
        self.m = self.m_inf(v);
        self.h = self.h_inf(v);
        self.n = self.n_inf(v);
    }

    /// Steady-state value for m gate
    fn m_inf(&self, v: f32) -> f32 {
        let alpha = self.alpha_m(v);
        let beta = self.beta_m(v);
        alpha / (alpha + beta)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Steady-state value for h gate
    fn h_inf(&self, v: f32) -> f32 {
        let alpha = self.alpha_h(v);
        let beta = self.beta_h(v);
        alpha / (alpha + beta)
    }

    /// Steady-state value for n gate
    fn n_inf(&self, v: f32) -> f32 {
        let alpha = self.alpha_n(v);
        let beta = self.beta_n(v);
        alpha / (alpha + beta)
    }

    /// Alpha rate for m gate (Na+ activation)
    fn alpha_m(&self, v: f32) -> f32 {
        let base_rate = if (v + 40.0).abs() < 0.001 {
            1.0 // Limit as v -> -40
        } else {
            0.1 * (v + 40.0) / (1.0 - (-(v + 40.0) / 10.0).exp())
        };
        temperature_correction(base_rate, self.temperature, HH_TEMPERATURE)
    }

    /// Beta rate for m gate (Na+ activation)
    fn beta_m(&self, v: f32) -> f32 {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let base_rate = 4.0 * (-(v + 65.0) / 18.0).exp();
        temperature_correction(base_rate, self.temperature, HH_TEMPERATURE)
    }

    /// Alpha rate for h gate (Na+ inactivation)
    fn alpha_h(&self, v: f32) -> f32 {
        let base_rate = 0.07 * (-(v + 65.0) / 20.0).exp();
        temperature_correction(base_rate, self.temperature, HH_TEMPERATURE)
    }

    /// Beta rate for h gate (Na+ inactivation)
    fn beta_h(&self, v: f32) -> f32 {
        let base_rate = 1.0 / (1.0 + (-(v + 35.0) / 10.0).exp());
        temperature_correction(base_rate, self.temperature, HH_TEMPERATURE)
    }

    /// Alpha rate for n gate (K+ activation)
    fn alpha_n(&self, v: f32) -> f32 {
        let base_rate = if (v + 55.0).abs() < 0.001 {
            0.1 // Limit as v -> -55
        } else {
            0.01 * (v + 55.0) / (1.0 - (-(v + 55.0) / 10.0).exp())
        };
        temperature_correction(base_rate, self.temperature, HH_TEMPERATURE)
    }

    /// Beta rate for n gate (K+ activation)
    fn beta_n(&self, v: f32) -> f32 {
        let base_rate = 0.125 * (-(v + 65.0) / 80.0).exp();
        temperature_correction(base_rate, self.temperature, HH_TEMPERATURE)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute ionic currents (`I_Na`, `I_K`, `I_L`) in uA/cm^2
    #[must_use]
    pub fn ionic_currents(&self) -> (f32, f32, f32) {
        let v = self.membrane_potential;

        // I_Na = g_Na * m^3 * h * (V - E_Na)
        let i_na = self.g_na * self.m.powi(3) * self.h * (v - self.e_na);

        // I_K = g_K * n^4 * (V - E_K)
        let i_k = self.g_k * self.n.powi(4) * (v - self.e_k);

        // I_L = g_L * (V - E_L)
        let i_l = self.g_l * (v - self.e_l);

        (i_na, i_k, i_l)
    }

    /// Total ionic current
    #[must_use]
    pub fn total_ionic_current(&self) -> f32 {
        let (i_na, i_k, i_l) = self.ionic_currents();
        i_na + i_k + i_l
    }

    /// Update gating variables using forward Euler
    pub fn update_gating(&mut self, dt: f32) {
        let v = self.membrane_potential;

        // m gate (Na+ activation)
        let alpha_m = self.alpha_m(v);
        let beta_m = self.beta_m(v);
        let dm = alpha_m * (1.0 - self.m) - beta_m * self.m;
        self.m += dm * dt;
        self.m = self.m.clamp(0.0, 1.0);

        // h gate (Na+ inactivation)
        let alpha_h = self.alpha_h(v);
        let beta_h = self.beta_h(v);
        let dh = alpha_h * (1.0 - self.h) - beta_h * self.h;
        self.h += dh * dt;
        self.h = self.h.clamp(0.0, 1.0);

        // n gate (K+ activation)
        let alpha_n = self.alpha_n(v);
        let beta_n = self.beta_n(v);
        let dn = alpha_n * (1.0 - self.n) - beta_n * self.n;
        self.n += dn * dt;
        self.n = self.n.clamp(0.0, 1.0);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Single integration step with external current
    ///
    /// # Arguments
    /// * `dt` - Time step (ms)
    /// * `i_ext` - External current (uA/cm^2)
    pub fn step(&mut self, dt: f32, i_ext: f32) {
        // dV/dt = (I_ext - I_ionic) / C_m
        let i_ionic = self.total_ionic_current();
        let dv = (i_ext - i_ionic) / self.capacitance;
        self.last_v = self.membrane_potential;
        self.membrane_potential += dv * dt;

        // Update gating variables
        self.update_gating(dt);
    }

    /// Check if neuron is currently spiking (crossing threshold upwards)
    #[must_use]
    pub fn is_spiking(&self) -> bool {
        self.last_v < self.spike_threshold && self.membrane_potential >= self.spike_threshold
    }

    /// Detect if spike occurred (threshold crossing)
    #[must_use]
    pub fn detect_spike(&self) -> bool {
        self.is_spiking()
    }

    /// Get current firing rate estimate based on state
    #[must_use]
    pub fn firing_rate_estimate(&self) -> f32 {
        // Simple estimate based on membrane potential and gating
        let v_factor = (self.membrane_potential - self.spike_threshold + 30.0).max(0.0) / 30.0;
        let h_factor = 1.0 - self.h; // Higher firing when h is low (more available Na+)
        v_factor * h_factor * 100.0 // Hz estimate
    }

    /// Reset after spike (for simple refractory model)
    pub fn reset(&mut self) {
        self.membrane_potential = self.e_l;
        self.h *= 0.95; // Increased refractoriness
        self.n *= 1.05; // More K+ channels open
        self.h = self.h.clamp(0.0, 1.0);
        self.n = self.n.clamp(0.0, 1.0);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Set membrane potential and reinitialize gates
    pub fn set_voltage(&mut self, v: f32) {
        self.membrane_potential = v;
        self.last_v = v;
        self.initialize_gates();
    }

    /// Get current state summary
    #[must_use]
    pub fn state_summary(&self) -> HHStateSummary {
        let (i_na, i_k, i_l) = self.ionic_currents();
        HHStateSummary {
            membrane_potential: self.membrane_potential,
            m_gate: self.m,
            h_gate: self.h,
            n_gate: self.n,
            i_na,
            i_k,
            i_l,
            total_current: i_na + i_k + i_l,
        }
    }
}

/// State summary for monitoring
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HHStateSummary {
    /// Membrane potential (mV)
    pub membrane_potential: f32,
    /// Na+ activation gate
    pub m_gate: f32,
    /// Na+ inactivation gate
    pub h_gate: f32,
    /// K+ activation gate
    pub n_gate: f32,
    /// Na+ current (uA/cm^2)
    pub i_na: f32,
    /// K+ current (uA/cm^2)
    pub i_k: f32,
    /// Leak current (uA/cm^2)
    pub i_l: f32,
    /// Total ionic current (uA/cm^2)
    pub total_current: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Extended HH model with additional ionic currents
///
/// Includes:
/// - A-type transient K+ current (`I_A`)
/// - M-type slow K+ current (`I_M`)
/// - High-voltage activated Ca2+ current (`I_Ca,L`)
/// - Ca2+-activated K+ current (`I_KCa`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedHHModel {
    /// Base HH model
    pub base: HodgkinHuxleyModel,

    // Additional conductances (mS/cm^2)
    /// A-type K+ conductance (fast inactivating)
    pub g_a: f32,
    /// M-type K+ conductance (slow, muscarinic)
    pub g_m: f32,
    /// L-type Ca2+ conductance
    pub g_ca: f32,
    /// Ca2+-activated K+ conductance
    pub g_kca: f32,

    // Additional reversal potentials (mV)
    /// Ca2+ reversal potential
    pub e_ca: f32,

    // Additional gating variables
    /// A-type K+ activation
    pub a: f32,
    /// A-type K+ inactivation
    pub b: f32,
    /// M-type K+ activation
    pub p: f32,
    /// Ca2+ channel activation
    pub q: f32,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Ca2+ channel inactivation
    pub r: f32,

    // Calcium dynamics
    /// Intracellular Ca2+ concentration (mM)
    pub ca_concentration: f32,
    /// Ca2+ decay time constant (ms)
    pub ca_tau: f32,
    /// Ca2+ influx scaling
    pub ca_influx_scale: f32,
    /// Resting Ca2+ concentration (mM)
    pub ca_rest: f32,
}

impl Default for ExtendedHHModel {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtendedHHModel {
    /// Create new extended HH model with default parameters
    #[must_use]
    pub fn new() -> Self {
        let base = HodgkinHuxleyModel::cortical_pyramidal();
        let mut model = Self {
            base,
            g_a: 10.0,
            g_m: 0.5,
            g_ca: 1.0,
            g_kca: 0.5,
            e_ca: 120.0,
            a: 0.0,
            b: 1.0,
            p: 0.0,
            q: 0.0,
            r: 1.0,
            ca_concentration: 0.0001, // 100 nM resting
            ca_tau: 100.0,
            ca_influx_scale: 0.01,
            ca_rest: 0.0001,
        };
        model.initialize_extended_gates();
        model
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create model with hippocampal CA1 pyramidal parameters
    #[must_use]
    pub fn ca1_pyramidal() -> Self {
        let base = HodgkinHuxleyModel::cortical_pyramidal();
        let mut model = Self {
            base,
            g_a: 15.0,  // Strong A-current in CA1
            g_m: 1.0,   // Prominent M-current
            g_ca: 2.0,  // Substantial Ca2+ current
            g_kca: 1.0, // Strong KCa
            e_ca: 130.0,
            a: 0.0,
            b: 1.0,
            p: 0.0,
            q: 0.0,
            r: 1.0,
            ca_concentration: 0.0001,
            ca_tau: 80.0, // Faster Ca2+ clearance in CA1
            ca_influx_scale: 0.015,
            ca_rest: 0.0001,
        };
        model.initialize_extended_gates();
        model
    }

    /// Initialize extended gating variables to steady state at resting membrane potential
    pub fn initialize_extended_gates(&mut self) {
        let v = self.base.membrane_potential;
        let aa = self.alpha_a(v);
        let ba = self.beta_a(v);
        self.a = if (aa + ba).abs() > 1e-6 {
            aa / (aa + ba)
        } else {
            0.0
        };

        let ab = self.alpha_b(v);
        let bb = self.beta_b(v);
        self.b = if (ab + bb).abs() > 1e-6 {
            ab / (ab + bb)
        } else {
            1.0
        };

        let ap = self.alpha_p(v);
        let bp = self.beta_p(v);
        self.p = if (ap + bp).abs() > 1e-6 {
            ap / (ap + bp)
        } else {
            0.0
        };

        let aq = self.alpha_q(v);
        let bq = self.beta_q(v);
        self.q = if (aq + bq).abs() > 1e-6 {
            aq / (aq + bq)
        } else {
            0.0
        };

        let ar = self.alpha_r(v);
        let br = self.beta_r(v);
        self.r = if (ar + br).abs() > 1e-6 {
            ar / (ar + br)
        } else {
            1.0
        };
    }

    /// A-current alpha rate for a gate (with L'Hôpital guard as v -> -45)
    fn alpha_a(&self, v: f32) -> f32 {
        let x = v + 45.0;
        let base_rate = if x.abs() < 0.001 {
            0.1 // 0.02 * 5.0
        } else {
            0.02 * x / (1.0 - (-x / 5.0).exp())
        };
        temperature_correction(base_rate, self.base.temperature, HH_TEMPERATURE)
    }

    /// A-current beta rate for a gate
    fn beta_a(&self, v: f32) -> f32 {
        let base_rate = 0.1 * (-(v + 15.0) / 10.0).exp();
        temperature_correction(base_rate, self.base.temperature, HH_TEMPERATURE)
    }

    /// A-current alpha rate for b gate
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    fn alpha_b(&self, v: f32) -> f32 {
        let base_rate = 0.08 * (-(v + 50.0) / 10.0).exp();
        temperature_correction(base_rate, self.base.temperature, HH_TEMPERATURE)
    }

    /// A-current beta rate for b gate
    fn beta_b(&self, v: f32) -> f32 {
        let base_rate = 0.5 / (1.0 + (-(v + 15.0) / 10.0).exp());
        temperature_correction(base_rate, self.base.temperature, HH_TEMPERATURE)
    }

    /// M-current alpha rate (with L'Hôpital guard as v -> -30)
    fn alpha_p(&self, v: f32) -> f32 {
        let x = v + 30.0;
        let base_rate = if x.abs() < 0.001 {
            0.009 // 0.001 * 9.0
        } else {
            0.001 * x / (1.0 - (-x / 9.0).exp())
        };
        temperature_correction(base_rate, self.base.temperature, HH_TEMPERATURE)
    }

    /// M-current beta rate
    fn beta_p(&self, v: f32) -> f32 {
        let base_rate = 0.001 * (-(v + 30.0) / 9.0).exp();
        temperature_correction(base_rate, self.base.temperature, HH_TEMPERATURE)
    }

    /// Ca2+ channel alpha for q gate (with L'Hôpital guard as v -> -27)
    fn alpha_q(&self, v: f32) -> f32 {
        let x = v + 27.0;
        let base_rate = if x.abs() < 0.001 {
            0.209 // 0.055 * 3.8
        } else {
            0.055 * x / (1.0 - (-x / 3.8).exp())
        };
        temperature_correction(base_rate, self.base.temperature, HH_TEMPERATURE)
    }

    /// Ca2+ channel beta for q gate
    fn beta_q(&self, v: f32) -> f32 {
        let base_rate = 0.94 * (-(v + 65.0) / 17.0).exp();
        temperature_correction(base_rate, self.base.temperature, HH_TEMPERATURE)
    }

    /// Ca2+ channel alpha for r gate
    fn alpha_r(&self, v: f32) -> f32 {
        let base_rate = 0.000457 * (-(v + 13.0) / 50.0).exp();
        temperature_correction(base_rate, self.base.temperature, HH_TEMPERATURE)
    }

    /// Ca2+ channel beta for r gate
    fn beta_r(&self, v: f32) -> f32 {
        let base_rate = 0.0065 / (1.0 + (-(v + 15.0) / 28.0).exp());
        temperature_correction(base_rate, self.base.temperature, HH_TEMPERATURE)
    }

    /// `KCa` activation (instantaneous, depends on Ca2+)
    fn kca_activation(&self) -> f32 {
        let ca = self.ca_concentration.max(0.0);
        ca / (ca + 0.001) // Hill function
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute additional currents
    #[must_use]
    pub fn extended_currents(&self) -> (f32, f32, f32, f32) {
        let v = self.base.membrane_potential;

        // I_A = g_A * a^3 * b * (V - E_K)
        let i_a = self.g_a * self.a.powi(3) * self.b * (v - self.base.e_k);

        // I_M = g_M * p * (V - E_K)
        let i_m = self.g_m * self.p * (v - self.base.e_k);

        // I_Ca = g_Ca * q^2 * r * (V - E_Ca)
        let i_ca = self.g_ca * self.q.powi(2) * self.r * (v - self.e_ca);

        // I_KCa = g_KCa * kca_act * (V - E_K)
        let kca_act = self.kca_activation();
        let i_kca = self.g_kca * kca_act * (v - self.base.e_k);

        (i_a, i_m, i_ca, i_kca)
    }

    /// Total ionic current including extended currents
    #[must_use]
    pub fn total_ionic_current(&self) -> f32 {
        let base_current = self.base.total_ionic_current();
        let (i_a, i_m, i_ca, i_kca) = self.extended_currents();
        base_current + i_a + i_m + i_ca + i_kca
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update additional gating variables
    pub fn update_extended_gating(&mut self, dt: f32) {
        let v = self.base.membrane_potential;

        // A-current gates
        let alpha_a = self.alpha_a(v);
        let beta_a = self.beta_a(v);
        let da = alpha_a * (1.0 - self.a) - beta_a * self.a;
        self.a += da * dt;
        self.a = self.a.clamp(0.0, 1.0);

        let alpha_b = self.alpha_b(v);
        let beta_b = self.beta_b(v);
        let db = alpha_b * (1.0 - self.b) - beta_b * self.b;
        self.b += db * dt;
        self.b = self.b.clamp(0.0, 1.0);

        // M-current gate
        let alpha_p = self.alpha_p(v);
        let beta_p = self.beta_p(v);
        let dp = alpha_p * (1.0 - self.p) - beta_p * self.p;
        self.p += dp * dt;
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        self.p = self.p.clamp(0.0, 1.0);

        // Ca2+ channel gates
        let alpha_q = self.alpha_q(v);
        let beta_q = self.beta_q(v);
        let dq = alpha_q * (1.0 - self.q) - beta_q * self.q;
        self.q += dq * dt;
        self.q = self.q.clamp(0.0, 1.0);

        let alpha_r = self.alpha_r(v);
        let beta_r = self.beta_r(v);
        let dr = alpha_r * (1.0 - self.r) - beta_r * self.r;
        self.r += dr * dt;
        self.r = self.r.clamp(0.0, 1.0);
    }

    /// Update calcium concentration
    pub fn update_calcium(&mut self, dt: f32) {
        let (_, _, i_ca, _) = self.extended_currents();

        // d[Ca]/dt = -I_Ca * scale - ([Ca] - [Ca]_rest) / tau_Ca
        // Negative I_Ca means Ca2+ influx (inward current positive convention)
        let ca_influx = if i_ca < 0.0 {
            -i_ca * self.ca_influx_scale
        } else {
            0.0 // No influx during outward Ca2+ current
        };

        let ca_efflux = -(self.ca_concentration - self.ca_rest) / self.ca_tau;

        self.ca_concentration += (ca_influx + ca_efflux) * dt;
        self.ca_concentration = self.ca_concentration.max(0.0);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Single integration step with external current
    pub fn step(&mut self, dt: f32, i_ext: f32) {
        // Update base model's last_v for spike detection
        self.base.last_v = self.base.membrane_potential;

        // dV/dt = (I_ext - I_ionic) / C_m
        let i_ionic = self.total_ionic_current();
        let dv = (i_ext - i_ionic) / self.base.capacitance;
        self.base.membrane_potential += dv * dt;

        // Update all gating variables
        self.base.update_gating(dt);
        self.update_extended_gating(dt);
        self.update_calcium(dt);
    }

    /// Check if spiking (delegates to base model)
    #[must_use]
    pub fn is_spiking(&self) -> bool {
        self.base.is_spiking()
    }

    /// Detect spike (threshold crossing)
    #[must_use]
    pub fn detect_spike(&self) -> bool {
        self.base.detect_spike()
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    /// Get firing rate estimate
    #[must_use]
    pub fn firing_rate_estimate(&self) -> f32 {
        self.base.firing_rate_estimate()
    }

    /// Reset after spike
    pub fn reset(&mut self) {
        self.base.reset();
        // Also reduce Ca2+ to prevent hyperexcitability
        self.ca_concentration = self.ca_rest * 1.5;
    }

    /// Set membrane potential and reinitialize all gates
    pub fn set_voltage(&mut self, v: f32) {
        self.base.set_voltage(v);
        self.initialize_extended_gates();
    }

    /// Get full state summary including extended currents
    #[must_use]
    pub fn state_summary(&self) -> ExtendedHHStateSummary {
        let base_summary = self.base.state_summary();
        let (i_a, i_m, i_ca, i_kca) = self.extended_currents();

        ExtendedHHStateSummary {
            base: base_summary,
            a_gate: self.a,
            b_gate: self.b,
            p_gate: self.p,
            q_gate: self.q,
            r_gate: self.r,
            ca_concentration: self.ca_concentration,
            i_a,
            i_m,
            i_ca,
            i_kca,
            extended_current_total: i_a + i_m + i_ca + i_kca,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Extended state summary
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ExtendedHHStateSummary {
    /// Base HH state
    pub base: HHStateSummary,
    /// A-current activation
    pub a_gate: f32,
    /// A-current inactivation
    pub b_gate: f32,
    /// M-current activation
    pub p_gate: f32,
    /// Ca2+ channel activation
    pub q_gate: f32,
    /// Ca2+ channel inactivation
    pub r_gate: f32,
    /// Ca2+ concentration (mM)
    pub ca_concentration: f32,
    /// A-type K+ current
    pub i_a: f32,
    /// M-type K+ current
    pub i_m: f32,
    /// Ca2+ current
    pub i_ca: f32,
    /// `KCa` current
    pub i_kca: f32,
    /// Total extended current
    pub extended_current_total: f32,
}

/// HH model wrapper for neuromodulator-specific parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuromodulatorHHModel {
    /// Extended HH model
    pub model: ExtendedHHModel,
    /// Neuromodulator type identifier
    pub modulator_type: String,
    /// Modulation factors for conductances
    pub g_modulation: [f32; 5], // For g_na, g_k, g_a, g_m, g_ca
    /// Baseline firing rate (Hz)
    pub baseline_rate: f32,
    /// Burst threshold (mV)
    pub burst_threshold: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl NeuromodulatorHHModel {
    /// Create dopaminergic neuron model (VTA/SNc type)
    #[must_use]
    pub fn dopaminergic() -> Self {
        let mut model = ExtendedHHModel::new();
        // DA neurons have specific properties
        model.base.g_na = 90.0;
        model.base.g_k = 30.0;
        model.g_a = 5.0; // Lower A-current
        model.g_m = 2.0; // Strong M-current for pacemaking
        model.g_ca = 3.0; // Strong Ca2+ for bursting
        model.g_kca = 2.0; // Strong KCa
        model.ca_tau = 50.0; // Faster Ca2+ dynamics

        Self {
            model,
            modulator_type: "dopamine".to_string(),
            g_modulation: [1.0, 1.0, 0.8, 1.2, 1.5],
            baseline_rate: 5.0,
            burst_threshold: -40.0,
        }
    }

    /// Create serotonergic neuron model (raphe nucleus type)
    #[must_use]
    pub fn serotonergic() -> Self {
        let mut model = ExtendedHHModel::new();
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // 5-HT neurons have slower dynamics
        model.base.g_na = 80.0;
        model.base.g_k = 40.0;
        model.g_a = 8.0;
        model.g_m = 1.5;
        model.g_ca = 1.5;
        model.g_kca = 1.0;
        model.ca_tau = 120.0; // Slower Ca2+ dynamics

        Self {
            model,
            modulator_type: "serotonin".to_string(),
            g_modulation: [0.9, 1.1, 1.0, 1.1, 0.9],
            baseline_rate: 3.0,
            burst_threshold: -35.0,
        }
    }

    /// Apply neuromodulator-specific step
    pub fn step(&mut self, dt: f32, i_ext: f32) {
        // Apply modulation to conductances
        let orig_g_na = self.model.base.g_na;
        let orig_g_k = self.model.base.g_k;
        let orig_g_a = self.model.g_a;
        let orig_g_m = self.model.g_m;
        let orig_g_ca = self.model.g_ca;

        self.model.base.g_na *= self.g_modulation[0];
        self.model.base.g_k *= self.g_modulation[1];
        self.model.g_a *= self.g_modulation[2];
        self.model.g_m *= self.g_modulation[3];
        self.model.g_ca *= self.g_modulation[4];

        // Step the model
        self.model.step(dt, i_ext);

        // Restore original conductances
        self.model.base.g_na = orig_g_na;
        self.model.base.g_k = orig_g_k;
        self.model.g_a = orig_g_a;
        self.model.g_m = orig_g_m;
        self.model.g_ca = orig_g_ca;
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check for burst firing pattern
    #[must_use]
    pub fn is_bursting(&self) -> bool {
        self.model.base.membrane_potential > self.burst_threshold
            && self.model.ca_concentration > 0.0005
    }
}

/// Simulate HH neuron response to current pulse
pub fn simulate_current_pulse(
    model: &mut HodgkinHuxleyModel,
    dt: f32,
    total_time: f32,
    pulse_start: f32,
    pulse_end: f32,
    pulse_amplitude: f32,
) -> Vec<HHStateSummary> {
    let steps = (total_time / dt) as usize;
    let start_step = (pulse_start / dt) as usize;
    let end_step = (pulse_end / dt) as usize;

    let mut results = Vec::with_capacity(steps);

    for step in 0..steps {
        let i_ext = if step >= start_step && step < end_step {
            pulse_amplitude
        } else {
            0.0
        };

        model.step(dt, i_ext);
        results.push(model.state_summary());
    }

    results
}

/// Simulate extended HH neuron response
// TAG: surface=simulation owner=research-team rule=SIM-001
pub fn simulate_extended_pulse(
    model: &mut ExtendedHHModel,
    dt: f32,
    total_time: f32,
    pulse_start: f32,
    pulse_end: f32,
    pulse_amplitude: f32,
) -> Vec<ExtendedHHStateSummary> {
    let steps = (total_time / dt) as usize;
    let start_step = (pulse_start / dt) as usize;
    let end_step = (pulse_end / dt) as usize;

    let mut results = Vec::with_capacity(steps);

    for step in 0..steps {
        let i_ext = if step >= start_step && step < end_step {
            pulse_amplitude
        } else {
            0.0
        };

        model.step(dt, i_ext);
        results.push(model.state_summary());
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hh_model_creation() {
        let model = HodgkinHuxleyModel::new();
        assert!((model.membrane_potential - (-65.0)).abs() < 0.1);
        assert!(model.m > 0.0 && model.m < 1.0);
        assert!(model.h > 0.0 && model.h < 1.0);
        assert!(model.n > 0.0 && model.n < 1.0);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    #[test]
    fn test_cortical_model_creation() {
        let model = HodgkinHuxleyModel::cortical_pyramidal();
        assert!((model.membrane_potential - (-70.0)).abs() < 0.1);
        assert!(model.temperature > HH_TEMPERATURE); // 37C > 6.3C
    }

    #[test]
    fn test_gating_rates() {
        let model = HodgkinHuxleyModel::new();
        let alpha_m = model.alpha_m(-65.0);
        let beta_m = model.beta_m(-65.0);
        assert!(alpha_m > 0.0);
        assert!(beta_m > 0.0);
    }

    #[test]
    fn test_steady_state_gates() {
        let model = HodgkinHuxleyModel::new();
        // At resting potential (-65 mV), m should be small, h should be large, n should be moderate
        // HH steady-state values at -65 mV: m_inf ≈ 0.05, h_inf ≈ 0.6, n_inf ≈ 0.32
        assert!(model.m < 0.1, "m should be small at rest, got {}", model.m);
        assert!(model.h > 0.5, "h should be large at rest, got {}", model.h);
        assert!(
            model.n > 0.25 && model.n < 0.4,
            "n should be moderate at rest (0.25-0.4), got {}",
            model.n
        );
    }

    #[test]
    fn test_ionic_currents() {
        let model = HodgkinHuxleyModel::new();
        let (i_na, i_k, i_l) = model.ionic_currents();
        // At rest, currents should be small
        assert!(i_na.abs() < 10.0, "I_Na should be small at rest");
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(i_k.abs() < 10.0, "I_K should be small at rest");
        assert!(i_l.abs() < 5.0, "I_L should be small at rest");
    }

    #[test]
    fn test_action_potential_generation() {
        let mut model = HodgkinHuxleyModel::new();
        let dt = 0.025; // 0.025 ms timestep (standard for HH)
        let pulse_amplitude = 25.0; // Strong depolarizing current

        // Run simulation with current pulse
        let mut spike_detected = false;
        let mut max_v = model.membrane_potential;

        for _ in 0..2000 {
            model.step(dt, pulse_amplitude);
            max_v = max_v.max(model.membrane_potential);
            if model.detect_spike() {
                spike_detected = true;
            }
        }

        assert!(spike_detected, "Should generate action potential");
        assert!(
            max_v > 30.0,
            "Should reach >30mV during spike (got max {max_v} mV)"
        );
    }

    #[test]
    fn test_spike_threshold() {
        let mut weak_model = HodgkinHuxleyModel::new();
        let mut strong_model = HodgkinHuxleyModel::new();
        let dt = 0.01;

        // Weak current should not spike
        let mut weak_spike = false;
        for _ in 0..5000 {
            weak_model.step(dt, 2.0);
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            if weak_model.detect_spike() {
                weak_spike = true;
                break;
            }
        }

        // Strong current should spike
        let mut strong_spike = false;
        for _ in 0..5000 {
            strong_model.step(dt, 15.0);
            if strong_model.detect_spike() {
                strong_spike = true;
                break;
            }
        }

        assert!(!weak_spike, "Weak current should not cause spike");
        assert!(strong_spike, "Strong current should cause spike");
    }

    #[test]
    fn test_refractory_period() {
        let mut model = HodgkinHuxleyModel::new();
        let dt = 0.01;
        let pulse_amplitude = 25.0;

        // Generate first spike
        let mut first_spike_time = None;
        for step in 0..5000 {
            model.step(dt, pulse_amplitude);
            if model.detect_spike() && first_spike_time.is_none() {
                first_spike_time = Some(step);
            }
        }

        assert!(first_spike_time.is_some(), "Should have first spike");

        // Check that h is reduced (inactivation) and n is elevated after spike
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(
            model.h < 0.5,
            "h should be reduced after spike (refractory)"
        );
        assert!(
            model.n > 0.5,
            "n should be elevated after spike (K+ activation)"
        );
    }

    #[test]
    fn test_extended_model_creation() {
        let model = ExtendedHHModel::new();
        assert!(model.g_a > 0.0);
        assert!(model.g_m > 0.0);
        assert!(model.g_ca > 0.0);
        assert!(model.g_kca > 0.0);
        assert!(model.ca_concentration >= 0.0);
    }

    #[test]
    fn test_calcium_dynamics() {
        let mut model = ExtendedHHModel::new();
        let initial_ca = model.ca_concentration;
        let dt = 0.01;

        // Strong depolarization causes Ca2+ influx
        for _ in 0..10000 {
            model.step(dt, 30.0);
        }

        // Ca2+ should have changed
        assert!(
            (model.ca_concentration - initial_ca).abs() > 0.000001,
            "Ca2+ concentration should change during simulation"
        );
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[test]
    fn test_temperature_correction() {
        let rate_at_6c = 1.0;
        let rate_at_37c = temperature_correction(rate_at_6c, 37.0, 6.3);
        // Higher temperature should increase rate
        assert!(rate_at_37c > rate_at_6c);
        // Q10 = 3, so rate should roughly triple every 10C
        assert!(rate_at_37c > 5.0); // Should be significantly higher
    }

    #[test]
    fn test_neuromodulator_models() {
        let da_model = NeuromodulatorHHModel::dopaminergic();
        let ser_model = NeuromodulatorHHModel::serotonergic();

        assert_eq!(da_model.modulator_type, "dopamine");
        assert_eq!(ser_model.modulator_type, "serotonin");

        // DA neurons have higher baseline rate
        assert!(da_model.baseline_rate > ser_model.baseline_rate);
    }

    #[test]
    fn test_simulate_current_pulse() {
        let mut model = HodgkinHuxleyModel::new();
        let results = simulate_current_pulse(&mut model, 0.01, 10.0, 2.0, 8.0, 20.0);

        assert!(!results.is_empty());
        // Should have at least one spike during the pulse
        let max_v = results
            .iter()
            .map(|s| s.membrane_potential)
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(max_v > 0.0, "Should reach positive voltage during spike");
    }

    #[test]
    fn test_state_summary() {
        let model = HodgkinHuxleyModel::new();
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let summary = model.state_summary();

        assert!(summary.membrane_potential < -50.0);
        assert!(summary.m_gate >= 0.0 && summary.m_gate <= 1.0);
        assert!(summary.h_gate >= 0.0 && summary.h_gate <= 1.0);
        assert!(summary.n_gate >= 0.0 && summary.n_gate <= 1.0);
    }

    #[test]
    fn test_extended_state_summary() {
        let model = ExtendedHHModel::new();
        let summary = model.state_summary();

        assert!(summary.ca_concentration >= 0.0);
        assert!(summary.a_gate >= 0.0 && summary.a_gate <= 1.0);
        assert!(summary.q_gate >= 0.0 && summary.q_gate <= 1.0);
    }

    #[test]
    fn test_ca1_pyramidal_model() {
        let model = ExtendedHHModel::ca1_pyramidal();
        // CA1 has stronger A-current
        assert!(model.g_a > 10.0);
        // Faster Ca2+ clearance
        assert!(model.ca_tau < 100.0);
    }

    #[test]
    fn test_firing_rate_estimate() {
        let mut model = HodgkinHuxleyModel::new();
        let baseline_estimate = model.firing_rate_estimate();

        // Depolarize the model
        model.set_voltage(-30.0);
        let depolarized_estimate = model.firing_rate_estimate();

        // Depolarized state should have higher firing rate estimate
        assert!(depolarized_estimate > baseline_estimate);
    }
}
