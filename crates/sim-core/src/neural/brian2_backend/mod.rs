//! Brian2 Backend Implementation
//!
//! Full implementation of the `NeuralBackend` trait using Brian2 spiking neural
//! network simulator. Provides biologically-detailed DOCAS dynamics.
//!
// TAG: surface=simulation owner=research-team rule=SIM-001
//! # Architecture
//!
//! This backend generates Python scripts that use Brian2's equation-based modeling
//! to simulate spiking neuron populations for each DOCAS neuromodulator.
//!
//! # Features
//!
//! - **Connection Pooling**: Reuses Python processes for better performance
//! - **Result Caching**: Caches simulation results for identical agent states
//! - **Timeout Handling**: Prevents hung processes
//!
//! # Dependencies
//!
//! - Python 3.8+
//! - Brian2 2.5.0+
//! - `NumPy`, Matplotlib (optional, for debugging)
//!
//! # Example
//!
//! ```rust,no_run
//! use sim_core::neural::{Brian2Backend, NeuralBackend, BackendConfig};
//! use sim_core::agent::Agent;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut backend = Brian2Backend::new()?;
//! // Assuming agent is an existing Agent instance
//! // let result = backend.simulate_agent(&agent)?;
//! # Ok(())
//! # }
//! ```

use super::backend::{BackendConfig, DerivedDocasParams, NeuralBackend, SimulationResult};
use super::glial::{AxonSegmentId, GlialStatistics, GlialSystem, NeuralPosition};
use super::glutamate::GlutamateSystem;
use super::NeuralValidationError;
use crate::agent::Agent;
// DocasState imported when needed
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub mod pool;
pub mod script;
#[cfg(test)]
pub mod tests;

pub use pool::*;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Brian2 backend for spiking neural network simulation
#[derive(Debug)]
pub struct Brian2Backend {
    config: BackendConfig,
    script_path: String,
    neuron_equations: HashMap<Neuromodulator, String>,
    coupling_matrix: [[f32; 5]; 5],
    /// Connection pool for process reuse
    connection_pool: Option<Brian2ConnectionPool>,
    /// Result cache for identical inputs
    result_cache: Arc<Mutex<HashMap<u64, SimulationResult>>>,
    /// Cache hit counter for metrics
    cache_hits: Arc<std::sync::atomic::AtomicU64>,
    /// Glial system for neural modulation
    glial_system: GlialSystem,
}

/// Neuromodulator types for Brian2 modeling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Neuromodulator {
    /// dopamine.
    Dopamine = 0,
    /// oxytocin.
    Oxytocin = 1,
    /// cortisol.
    Cortisol = 2,
    /// adrenaline.
    Adrenaline = 3,
    /// serotonin.
    Serotonin = 4,
}

impl Brian2Backend {
    /// Create a new Brian2 backend with connection pooling
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn new() -> Result<Self, NeuralValidationError> {
        let config = BackendConfig::default();
        let script_path = format!("{}/brian2_docas.py", config.script_dir);

        // Verify Brian2 is available
        let output = Command::new(&config.python_path)
            .args(["-c", "import brian2; print('OK')"])
            .output()
            .map_err(|e| NeuralValidationError::Backend(format!("Failed to check Brian2: {e}")))?;

        if !output.status.success() {
            return Err(NeuralValidationError::Backend(
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                "Brian2 not installed or not accessible. Install with: pip install brian2"
                    .to_string(),
            ));
        }

        // Create scripts directory if needed
        std::fs::create_dir_all(&config.script_dir)?;

        // Initialize default neuron equations
        let neuron_equations = Self::default_neuron_equations();

        // Write Brian2 script
        Self::write_brian2_script(&script_path, &neuron_equations)?;

        // Initialize connection pool (4 connections, 5 min timeout)
        let connection_pool = Brian2ConnectionPool::new(
            config.python_path.clone(),
            script_path.clone(),
            4,
            Duration::from_secs(300),
        );

        // Initialize glial system for neural modulation
        // Create astrocytes and oligodendrocytes based on world size
        let glial_system = GlialSystem::new(20, 15, 100.0);

        Ok(Self {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            config,
            script_path,
            neuron_equations,
            coupling_matrix: [[0.0; 5]; 5],
            connection_pool: Some(connection_pool),
            result_cache: Arc::new(Mutex::new(HashMap::new())),
            cache_hits: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            glial_system,
        })
    }

    /// Compute cache key from agent state
    fn compute_cache_key(&self, agent: &Agent) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        // Hash DOCAS levels (convert f32 to bits for hashing)
        for level in &agent.docas.levels {
            level.to_bits().hash(&mut hasher);
        }
        // Hash genome alpha values
        for alpha in agent.genome.docas_alpha() {
            alpha.to_bits().hash(&mut hasher);
        }
        hasher.finish()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check cache for existing result
    fn check_cache(&self, agent: &Agent) -> Option<SimulationResult> {
        let key = self.compute_cache_key(agent);
        let cache = self.result_cache.lock().unwrap_or_else(|e| e.into_inner());

        if let Some(result) = cache.get(&key) {
            self.cache_hits
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            return Some(result.clone());
        }
        None
    }

    /// Store result in cache
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    fn store_cache(&self, agent: &Agent, result: &SimulationResult) {
        let key = self.compute_cache_key(agent);
        let mut cache = self.result_cache.lock().unwrap_or_else(|e| e.into_inner());

        // Limit cache size to 1000 entries
        if cache.len() >= 1000 {
            cache.clear(); // Simple eviction: clear all
        }

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        cache.insert(key, result.clone());
    }

    /// Get cache statistics
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[must_use]
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn cache_stats(&self) -> (usize, u64) {
        let cache = self.result_cache.lock().unwrap_or_else(|e| e.into_inner());
        let hits = self.cache_hits.load(std::sync::atomic::Ordering::SeqCst);
        (cache.len(), hits)
    }

    /// Set custom neuron equations for a neuromodulator
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn set_neuron_equations(
        &mut self,
        nm: Neuromodulator,
        equations: &str,
    ) -> Result<(), NeuralValidationError> {
        self.neuron_equations.insert(nm, equations.to_string());
        // Regenerate script with new equations
        Self::write_brian2_script(&self.script_path, &self.neuron_equations)?;
        Ok(())
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Set coupling strength between neuromodulators
    pub fn set_coupling(&mut self, from: Neuromodulator, to: Neuromodulator, strength: f32) {
        self.coupling_matrix[from as usize][to as usize] = strength;
    }

    /// Generate Hodgkin-Huxley style neuron equations for Brian2
    ///
    /// Creates detailed biophysical neuron equations with Na+, K+, and leak currents
    /// based on the classic Hodgkin-Huxley (1952) formalism.
    #[allow(clippy::uninlined_format_args)]
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[must_use]
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn hh_neuron_equations(neuromodulator: Neuromodulator) -> String {
        // Base HH parameters adjusted for each neuromodulator type
        let (g_na, g_k, g_l, e_na, e_k, e_l, cm) = match neuromodulator {
            Neuromodulator::Dopamine => (120.0, 36.0, 0.3, 55.0, -90.0, -70.0, 1.0), // VTA/SNc
            Neuromodulator::Oxytocin => (100.0, 40.0, 0.25, 50.0, -95.0, -75.0, 1.0), // Hypothalamic
            Neuromodulator::Cortisol => (140.0, 30.0, 0.35, 60.0, -85.0, -65.0, 0.9), // Fast/stress
            Neuromodulator::Adrenaline => (130.0, 35.0, 0.3, 55.0, -90.0, -70.0, 1.0), // LC/NA
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            Neuromodulator::Serotonin => (110.0, 38.0, 0.28, 55.0, -92.0, -72.0, 1.1), // Raphe (slow)
        };

        let nm_type = format!("{neuromodulator:?}");
        format!(
            // TAG: surface=simulation owner=research-team rule=SIM-001
            r"
# Hodgkin-Huxley neuron model ({nm_type})
# Classic squid axon dynamics with voltage-gated Na+ and K+ channels

dv/dt = (I_ext - g_Na * m**3 * h * (v - E_Na) - g_K * n**4 * (v - E_K) - g_L * (v - E_L)) / C_m : mV

# Gating variables (HH rate equations)
dm/dt = alpha_m * (1 - m) - beta_m * m : 1
dh/dt = alpha_h * (1 - h) - beta_h * h : 1
dn/dt = alpha_n * (1 - n) - beta_n * n : 1

# Voltage-dependent rate constants (temperature-corrected for 37C)
alpha_m = 0.32 * (v + 45) / (1 - exp(-(v + 45) / 4)) / ms : Hz
beta_m = 0.28 * (v + 18) / (exp((v + 18) / 5) - 1) / ms : Hz
alpha_h = 0.128 * exp(-(v + 43) / 18) / ms : Hz
beta_h = 4 / (1 + exp(-(v + 20) / 5)) / ms : Hz
alpha_n = 0.032 * (v + 45) / (1 - exp(-(v + 45) / 5)) / ms : Hz
beta_n = 0.5 * exp(-(v + 50) / 40) / ms : Hz

# Parameters
C_m = {cm}*uF/cm**2 : uF/cm**2
g_Na = {g_na}*mS/cm**2 : mS/cm**2
g_K = {g_k}*mS/cm**2 : mS/cm**2
g_L = {g_l}*mS/cm**2 : mS/cm**2
E_Na = {e_na}*mV : mV
E_K = {e_k}*mV : mV
E_L = {e_l}*mV : mV

# External input current
I_ext : uA/cm**2

# Spike threshold for monitoring
v_thresh = -20*mV : mV
",
            nm_type = nm_type,
            g_na = g_na,
            g_k = g_k,
            g_l = g_l,
            e_na = e_na,
            e_k = e_k,
            e_l = e_l,
            cm = cm,
        )
    }

    /// Enable Hodgkin-Huxley style neuron models in the backend
    ///
    /// Replaces the simplified LIF neurons with detailed HH models
    /// # Errors
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    ///
    /// Returns an error if the operation fails.
    pub fn enable_hodgkin_huxley_models(&mut self) -> Result<(), NeuralValidationError> {
        for nm in [
            Neuromodulator::Dopamine,
            Neuromodulator::Oxytocin,
            Neuromodulator::Cortisol,
            Neuromodulator::Adrenaline,
            Neuromodulator::Serotonin,
        ] {
            let hh_eqs = Self::hh_neuron_equations(nm);
            self.neuron_equations.insert(nm, hh_eqs);
        }

        // Regenerate script with HH equations
        Self::write_brian2_script(&self.script_path, &self.neuron_equations)?;
        Ok(())
    }

    /// Extract DOCAS parameters from Brian2 spike rates
    fn extract_docas_params(&self, result: &SimulationResult) -> DerivedDocasParams {
        // Calculate decay rates from activity traces
        let calc_decay = |activity: &[f32]| -> f32 {
            if activity.len() < 10 {
                // TAG: surface=simulation owner=research-team rule=SIM-001
                return 0.95;
            }
            let peak = activity.iter().fold(0.0f32, |a, &b| a.max(b));
            let late = activity[activity.len() - 10..].iter().sum::<f32>() / 10.0;
            if peak > 0.01 {
                (late / peak).clamp(0.8, 0.99)
            } else {
                0.95
            }
        };

        // Calculate response amplitude
        let calc_amplitude = |activity: &[f32]| -> f32 {
            let max_val = activity.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
            let min_val = activity.iter().fold(f32::INFINITY, |a, &b| a.min(b));
            max_val - min_val
        };

        let alpha = [
            calc_decay(&result.dopamine_activity),
            calc_decay(&result.oxytocin_activity),
            calc_decay(&result.cortisol_activity),
            calc_decay(&result.adrenaline_activity),
            calc_decay(&result.serotonin_activity),
        ];

        let amplitudes = [
            calc_amplitude(&result.dopamine_activity),
            calc_amplitude(&result.oxytocin_activity),
            calc_amplitude(&result.cortisol_activity),
            calc_amplitude(&result.adrenaline_activity),
            calc_amplitude(&result.serotonin_activity),
        ];

        // Beta proportional to amplitude
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let beta = [
            amplitudes[0].clamp(0.01, 0.5),
            amplitudes[1].clamp(0.01, 0.5),
            amplitudes[2].clamp(0.01, 0.5),
            amplitudes[3].clamp(0.01, 0.5),
            amplitudes[4].clamp(0.01, 0.5),
        ];

        // Time constants based on neuromodulator properties
        let time_constants = [0.05, 0.08, 0.03, 0.04, 0.10];

        DerivedDocasParams {
            alpha,
            beta,
            time_constants,
            coupling: self.coupling_matrix,
            confidence: 0.85,
        }
    }

    /// Apply glial cell modulation to simulation results
    ///
    /// This integrates astrocyte synaptic modulation and oligodendrocyte
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// conduction velocity effects into the neural simulation output
    fn apply_glial_modulation(&mut self, result: &mut SimulationResult) {
        // Record neural activity for oligodendrocyte myelination updates
        for (i, activity) in result.dopamine_activity.iter().enumerate() {
            let segment = AxonSegmentId::new(i as u64);
            self.glial_system.record_activity(segment, *activity);
        }

        // Update glial system
        self.glial_system.tick();
        self.glial_system
            .update_astrocytes(&result.dopamine_activity, 0.001);
        self.glial_system.update_myelination(0.001);

        // Apply astrocyte modulation to activity traces
        // This simulates the tripartite synapse effect
        let position = NeuralPosition::new_2d(0.0, 0.0);
        for activity in &mut result.dopamine_activity {
            *activity = self
                .glial_system
                .modulate_synaptic_strength(*activity, &position);
            // TAG: surface=simulation owner=research-team rule=SIM-001
        }

        // Modulate derived DOCAS parameters based on glial state
        let timing_modifier = self.glial_system.trust_timing_modifier();
        for tc in &mut result.derived_docas.time_constants {
            *tc *= timing_modifier;
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get glial system statistics
    #[must_use]
    pub fn glial_statistics(&self) -> GlialStatistics {
        self.glial_system.statistics()
    }

    /// Update glial system with external activity
    pub fn update_glial(&mut self, neural_activity: &[f32], dt: f32) {
        self.glial_system.update_astrocytes(neural_activity, dt);
        self.glial_system.update_myelination(dt);
        self.glial_system.tick();
    }

    /// Get conduction delay for myelinated vs unmyelinated axons
    #[must_use]
    pub fn conduction_delay(&self, distance: f32, myelinated: bool) -> f32 {
        use super::glial::AxonType;
        let axon_type = if myelinated {
            AxonType::Myelinated
        } else {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            AxonType::Unmyelinated
        };
        self.glial_system.conduction_delay(distance, axon_type)
    }

    /// Create a glutamatergic synapse with AMPA and NMDA receptors
    ///
    /// This creates a detailed synapse model for computing excitatory currents
    /// that can be integrated into Brian2 neuron models.
    ///
    /// # Arguments
    /// * `n_ampa` - Number of AMPA receptors (typical: 50-200)
    /// * `n_nmda` - Number of NMDA receptors (typical: 10-40)
    #[must_use]
    pub fn create_glutamatergic_synapse(&self, n_ampa: usize, n_nmda: usize) -> GlutamateSystem {
        GlutamateSystem::new(n_ampa, n_nmda, 0)
    }

    /// Create an inhibitory `GABAergic` synapse
    ///
    /// # Arguments
    /// * `n_gaba` - Number of `GABA_A` receptors (typical: 20-60)
    #[must_use]
    pub fn create_gabaergic_synapse(&self, n_gaba: usize) -> GlutamateSystem {
        GlutamateSystem::new(0, 0, n_gaba)
    }

    /// Create a mixed synapse with all receptor types
    ///
    /// Useful for modeling synapses with spillover or for comprehensive
    /// synaptic current computations.
    #[must_use]
    pub fn create_mixed_synapse(
        &self,
        n_ampa: usize,
        n_nmda: usize,
        n_gaba: usize,
    ) -> GlutamateSystem {
        GlutamateSystem::new(n_ampa, n_nmda, n_gaba)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute synaptic currents for a neuron using detailed receptor dynamics
    ///
    /// This method integrates the glutamate receptor system with the glial
    /// modulation system to compute realistic synaptic currents.
    ///
    /// # Arguments
    /// * `synapse` - Glutamate system with receptor populations
    /// * `membrane_potential` - Current membrane potential in mV
    /// * `position` - Spatial position for glial modulation lookup
    ///
    /// # Returns
    /// Total synaptic current (positive = excitatory, negative = inhibitory)
    pub fn compute_synapse_current(
        &self,
        synapse: &mut GlutamateSystem,
        membrane_potential: f32,
        position: &NeuralPosition,
    ) -> f32 {
        // Apply glial modulation to synaptic strength
        let modulation_factor = self.glial_system.modulate_synaptic_strength(1.0, position);

        // Compute base synaptic current
        let base_current = synapse.synaptic_current(membrane_potential);

        // Apply glial modulation
        base_current * modulation_factor
    }

    /// Update synapse with time step (receptor kinetics and neurotransmitter clearance)
    ///
    /// # Arguments
    /// * `synapse` - Glutamate system to update
    /// * `dt` - Time step in milliseconds
    pub fn update_synapse(&self, synapse: &mut GlutamateSystem, dt: f32) {
        // Update receptor kinetics
        synapse.update(dt);

        // Apply astrocyte-mediated glutamate clearance
        let clearance_rate = self.glial_system.statistics().glutamate_clearance;
        synapse.glutamate_concentration *= clearance_rate.powf(dt / 10.0);
    }

    /// Trigger presynaptic spike in synapse
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    ///
    /// # Arguments
    /// * `synapse` - Glutamate system representing the synapse
    /// * `is_excitatory` - True for glutamatergic, false for `GABAergic`
    pub fn trigger_presynaptic_spike(&self, synapse: &mut GlutamateSystem, is_excitatory: bool) {
        synapse.presynaptic_spike(is_excitatory);
    }

    /// Apply synaptic plasticity (LTP/LTD) to synapses based on activity
    ///
    /// This method integrates the synaptic plasticity system with the Brian2 backend,
    /// allowing for learning and memory formation during simulation.
    ///
    /// # Arguments
    /// * `plasticity` - Synaptic plasticity system to update
    /// * `pre_spikes` - List of pre-synaptic neuron IDs that spiked
    /// * `post_spikes` - List of post-synaptic neuron IDs that spiked
    /// * `current_tick` - Current simulation tick for timing
    ///
    /// # Returns
    /// Statistics about plasticity changes
    pub fn apply_plasticity(
        &self,
        plasticity: &mut super::plasticity::SynapticPlasticity,
        pre_spikes: &[super::plasticity::NeuronId],
        post_spikes: &[super::plasticity::NeuronId],
        current_tick: crate::types::Tick,
    ) -> super::plasticity::PlasticityStats {
        // Record pre-synaptic spikes
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        for &pre_id in pre_spikes {
            plasticity.record_pre_spike(pre_id, current_tick);
        }

        // Record post-synaptic spikes
        for &post_id in post_spikes {
            plasticity.record_post_spike(post_id, current_tick);
        }

        // Apply STDP to synapses with recorded spike pairs
        for i in 0..plasticity.synapses.len() {
            plasticity.apply_stdp_recorded(i, current_tick);
        }

        // Apply calcium-dependent plasticity
        plasticity.apply_calcium_plasticity_all();

        // Update metaplasticity periodically
        if current_tick % 100 == 0 {
            plasticity.update_metaplasticity();
        }

        // Consolidate early-phase to late-phase periodically
        if current_tick % plasticity.consolidation_params.consolidation_ticks == 0 {
            plasticity.consolidate();
        }

        // Decay calcium levels
        plasticity.decay_all_calcium();

        // Update tick counter
        plasticity.tick();

        // Return statistics
        plasticity.stats()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create a new synaptic plasticity system for learning
    ///
    /// This creates a plasticity system pre-configured with standard
    /// parameters for DOCAS neuromodulator learning.
    ///
    /// # Arguments
    /// * `agent_id` - Optional agent ID to associate with this system
    #[must_use]
    pub fn create_plasticity_system(
        &self,
        agent_id: Option<crate::types::AgentId>,
    ) -> super::plasticity::SynapticPlasticity {
        use super::plasticity::{CalciumParams, StdpParams, SynapticPlasticity};

        let mut plasticity = if let Some(id) = agent_id {
            SynapticPlasticity::for_agent(id)
        } else {
            SynapticPlasticity::new()
        };

        // Configure for DOCAS timescales (slower than typical hippocampal)
        plasticity.stdp_params = StdpParams {
            tau_plus: 50.0,  // ms - slower potentiation window
            tau_minus: 50.0, // ms - slower depression window
            max_window_ms: 200.0,
            ..Default::default()
        };

        // Configure calcium parameters for neuromodulator-driven plasticity
        plasticity.calcium_params = CalciumParams {
            theta_ltp: 0.7, // Slightly lower for social learning
            theta_ltd: 0.25,
            calcium_decay: 0.005, // Slower decay for sustained effects
            ..Default::default()
        };

        plasticity
    }

    /// Simulate experience-dependent plasticity based on DOCAS activity
    ///
    /// This connects plasticity to DOCAS neuromodulator levels, allowing
    /// dopamine (reward), oxytocin (social), and cortisol (stress) to
    /// modulate synaptic learning.
    ///
    /// # Arguments
    /// * `plasticity` - Plasticity system to modulate
    /// * `docas_levels` - Current DOCAS neuromodulator levels [D, O, C, A, S]
    /// * `learning_signal` - Additional learning signal (e.g., prediction error)
    // TAG: surface=simulation owner=research-team rule=SIM-001
    pub fn modulate_plasticity_by_docas(
        &self,
        plasticity: &mut super::plasticity::SynapticPlasticity,
        docas_levels: &[f32; 5],
        learning_signal: f32,
    ) {
        // Dopamine modulates learning rate (reward prediction error)
        let dopamine_factor = 1.0 + docas_levels[0]; // Dopamine index = 0

        // Oxytocin enhances social learning (trust signals)
        let oxytocin_factor = 1.0 + 0.5 * docas_levels[1]; // Oxytocin index = 1

        // Cortisol impairs learning under stress (but enhances threat learning)
        let cortisol_factor = 1.0 - 0.3 * docas_levels[2].clamp(0.0, 1.0); // Cortisol index = 2

        // Serotonin stabilizes learning (prevents runaway plasticity)
        let serotonin_factor = 1.0 / (1.0 + docas_levels[4]); // Serotonin index = 4

        // Combined modulation
        let modulation = dopamine_factor * oxytocin_factor * cortisol_factor * serotonin_factor;

        // Apply to effective learning rate
        let base_rate = plasticity.learning_rate;
        plasticity.effective_learning_rate =
            (base_rate * modulation * (1.0 + learning_signal)).clamp(0.0001, 0.5);
    }

    fn prepare_simulation_params(&self, agent: &Agent) -> serde_json::Value {
        serde_json::json!({
            "initial_levels": agent.docas.levels,
            "stimuli": {
                "reward": 0.5,
                "social": 0.5,
                "threat": 0.3,
                "action": 0.4,
                "stability": 0.6,
            },
            "dt": self.config.dt,
            "t_final": self.config.simulation_time,
            "n_neurons": self.config.neurons_per_ensemble,
        })
    }

    fn run_brian2_process(
        &self,
        params: &serde_json::Value,
    ) -> Result<std::process::Output, NeuralValidationError> {
        let output = if let Some(ref pool) = self.connection_pool {
            let mut conn = pool.acquire()?;
            let mut child = conn.take_child().ok_or_else(|| {
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                NeuralValidationError::Backend("No child process available".to_string())
            })?;
            if let Some(stdin) = child.stdin.take() {
                serde_json::to_writer(stdin, params).map_err(|e| {
                    NeuralValidationError::Backend(format!("Failed to write params: {e}"))
                })?;
            }
            child.wait_with_output().map_err(|e| {
                NeuralValidationError::Backend(format!("Brian2 process failed: {e}"))
            })?
        } else {
            let mut child = Command::new(&self.config.python_path)
                .arg(&self.script_path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| {
                    NeuralValidationError::Backend(format!("Failed to spawn Brian2: {e}"))
                })?;
            if let Some(stdin) = child.stdin.take() {
                serde_json::to_writer(stdin, params).map_err(|e| {
                    NeuralValidationError::Backend(format!("Failed to write params: {e}"))
                })?;
            }
            // TAG: surface=simulation
            child.wait_with_output().map_err(|e| {
                NeuralValidationError::Backend(format!("Brian2 process failed: {e}"))
            })?
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NeuralValidationError::Backend(format!(
                "Brian2 error: {stderr}"
            )));
        }

        Ok(output)
    }

    fn parse_simulation_result(
        &self,
        agent: &Agent,
        raw_result: &serde_json::Value,
    ) -> SimulationResult {
        let parse_vec = |k: &str| serde_json::from_value(raw_result[k].clone()).unwrap_or_default();
        SimulationResult {
            agent_id: agent.id,
            timestamps: parse_vec("timestamps"),
            dopamine_activity: parse_vec("dopamine_activity"),
            oxytocin_activity: parse_vec("oxytocin_activity"),
            cortisol_activity: parse_vec("cortisol_activity"),
            adrenaline_activity: parse_vec("adrenaline_activity"),
            serotonin_activity: parse_vec("serotonin_activity"),
            derived_docas: DerivedDocasParams::default(),
            bold_signal: None,
            metadata: {
                let mut m = HashMap::new();
                m.insert("backend".to_string(), "brian2".to_string());
                m.insert(
                    // TAG: surface=simulation
                    "spike_counts".to_string(),
                    raw_result["spike_counts"].to_string(),
                );
                m.insert(
                    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                    "neurons_per_ensemble".to_string(),
                    self.config.neurons_per_ensemble.to_string(),
                );
                m
            },
        }
    }
}

impl NeuralBackend for Brian2Backend {
    fn name(&self) -> &'static str {
        "Brian2"
    }

    fn simulate_agent(&mut self, agent: &Agent) -> Result<SimulationResult, NeuralValidationError> {
        if let Some(cached_result) = self.check_cache(agent) {
            return Ok(cached_result);
        }

        let params = self.prepare_simulation_params(agent);
        let output = self.run_brian2_process(&params)?;
        let raw_result: serde_json::Value =
            serde_json::from_slice(&output.stdout).map_err(|e| {
                NeuralValidationError::Backend(format!("Failed to parse Brian2 output: {e}"))
            })?;

        let mut result = self.parse_simulation_result(agent, &raw_result);
        result.derived_docas = self.extract_docas_params(&result);
        self.apply_glial_modulation(&mut result);
        self.store_cache(agent, &result);

        Ok(result)
    }

    fn simulate_batch(
        &mut self,
        agents: &[Agent],
    ) -> Result<Vec<SimulationResult>, NeuralValidationError> {
        agents
            .iter()
            .map(|agent| self.simulate_agent(agent))
            .collect()
    }

    fn config(&self) -> &BackendConfig {
        &self.config
    }

    fn health_check(&self) -> Result<bool, NeuralValidationError> {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let output = Command::new(&self.config.python_path)
            .args(["-c", "import brian2; print('OK')"])
            .output()
            .map_err(|e| NeuralValidationError::Backend(format!("Health check failed: {e}")))?;

        Ok(output.status.success())
    }
}

impl Default for Brian2Backend {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            config: BackendConfig::default(),
            script_path: String::new(),
            neuron_equations: Self::default_neuron_equations(),
            coupling_matrix: [[0.0; 5]; 5],
            connection_pool: None,
            result_cache: Arc::new(Mutex::new(HashMap::new())),
            cache_hits: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            glial_system: GlialSystem::default(),
        })
    }
}
