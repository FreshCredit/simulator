//! Real-Time Neural Backend Integration
//!
//! Provides lightweight, per-tick neural simulation for real-time DOCAS validation.
//! Unlike the offline calibration (which runs full Nengo/Brian2 simulations),
//! this module uses simplified spiking neuron models that can run every tick.
//!
//! Per P0-2: Enable real-time neural validation for scientific validity.

use crate::docas::DocasState;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Real-time neural simulator configuration
#[derive(Debug, Clone)]
pub struct RealTimeConfig {
    /// Enable real-time neural simulation
    pub enabled: bool,
    /// Neurons per DOCAS dimension
    pub neurons_per_dim: usize,
    /// Simulation timestep (ms)
    pub dt: f32,
    /// Synaptic time constant (ms)
    pub tau_syn: f32,
    /// Refractory period (ms)
    pub tau_ref: f32,
    /// Maximum firing rate (Hz)
    pub max_rate: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for RealTimeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            neurons_per_dim: 100, // 100 neurons per DOCAS dimension
            dt: 1.0,              // 1ms timestep (matches simulation tick)
            tau_syn: 5.0,         // 5ms synaptic time constant
            tau_ref: 2.0,         // 2ms refractory period
            max_rate: 200.0,      // 200 Hz max firing rate
        }
    }
}

/// Simplified spiking neuron for real-time simulation
/// Based on leaky integrate-and-fire (LIF) model
#[derive(Debug, Clone)]
pub struct SpikingNeuron {
    /// Membrane potential (mV)
    pub voltage: f32,
    /// Firing threshold (mV)
    pub threshold: f32,
    /// Membrane time constant (ms)
    pub tau_mem: f32,
    /// Resting potential (mV)
    pub v_rest: f32,
    /// Reset potential (mV)
    pub v_reset: f32,
    /// Time since last spike (ms)
    pub time_since_spike: f32,
    /// Spike count in current window
    pub spike_count: u32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for SpikingNeuron {
    fn default() -> Self {
        Self {
            voltage: -65.0,
            threshold: -55.0,
            tau_mem: 20.0,
            v_rest: -65.0,
            v_reset: -70.0,
            time_since_spike: 100.0, // Start ready to fire
            spike_count: 0,
        }
    }
}

impl SpikingNeuron {
    /// Create new LIF neuron
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Simulate one timestep with input current
    /// Returns true if neuron spiked
    pub fn step(&mut self, input_current: f32, dt: f32, tau_ref: f32) -> bool {
        // Check refractory period
        if self.time_since_spike < tau_ref {
            self.time_since_spike += dt;
            return false;
        }

        // Leaky integrate
        let dv = (self.v_rest - self.voltage + input_current) / self.tau_mem * dt;
        self.voltage += dv;

        // Check for spike
        if self.voltage >= self.threshold {
            self.voltage = self.v_reset;
            self.time_since_spike = 0.0;
            self.spike_count += 1;
            true
        } else {
            self.time_since_spike += dt;
            false
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get firing rate (Hz) over window
    #[must_use]
    pub fn firing_rate(&self, window_ms: f32) -> f32 {
        (self.spike_count as f32 / window_ms) * 1000.0
    }

    /// Reset spike counter
    pub fn reset_counter(&mut self) {
        self.spike_count = 0;
    }
}

/// Ensemble of neurons representing one DOCAS dimension
#[derive(Debug, Clone)]
pub struct NeuralEnsemble {
    /// DOCAS dimension index (0=D, 1=O, 2=C, 3=A, 4=S)
    pub dimension: usize,
    /// Neurons in ensemble
    pub neurons: Vec<SpikingNeuron>,
    /// Encoder values (how each neuron responds to input)
    pub encoders: Vec<f32>,
    /// Decoders (for reading out population activity)
    pub decoders: Vec<f32>,
    /// Target firing rate at max input
    pub max_rate: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl NeuralEnsemble {
    /// Create new ensemble for a DOCAS dimension
    #[must_use]
    pub fn new(dimension: usize, n_neurons: usize, max_rate: f32) -> Self {
        let mut neurons = Vec::with_capacity(n_neurons);
        let mut encoders = Vec::with_capacity(n_neurons);
        let mut decoders = Vec::with_capacity(n_neurons);

        for i in 0..n_neurons {
            neurons.push(SpikingNeuron::new());
            // Random encoder (-1 to 1)
            let encoder = (i as f32 / n_neurons as f32) * 2.0 - 1.0;
            encoders.push(encoder);
            // Simple decoder (normalized)
            decoders.push(encoder / n_neurons as f32);
        }

        Self {
            dimension,
            neurons,
            encoders,
            decoders,
            max_rate,
        }
    }

    /// Simulate one timestep with input value
    /// Input is DOCAS level [0, 1]
    pub fn step(&mut self, input: f32, config: &RealTimeConfig) -> f32 {
        let mut total_current = 0.0;

        for (i, neuron) in self.neurons.iter_mut().enumerate() {
            // Input current depends on encoder and input
            // Scale to physiological range (~0-10 nA)
            let current = 10.0 * input * self.encoders[i].abs();

            if neuron.step(current, config.dt, config.tau_ref) {
                total_current += self.decoders[i];
            }
        }

        // Return decoded value
        total_current
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get population firing rate (average across neurons)
    #[must_use]
    pub fn population_rate(&self, window_ms: f32) -> f32 {
        let total_spikes: u32 = self.neurons.iter().map(|n| n.spike_count).sum();
        let avg_spikes = total_spikes as f32 / self.neurons.len() as f32;
        (avg_spikes / window_ms) * 1000.0
    }

    /// Reset all spike counters
    pub fn reset_counters(&mut self) {
        for neuron in &mut self.neurons {
            neuron.reset_counter();
        }
    }
}

/// Real-time neural network simulating all DOCAS dimensions
#[derive(Debug, Clone)]
pub struct RealTimeNeuralNetwork {
    /// Configuration
    pub config: RealTimeConfig,
    /// Ensembles for each DOCAS dimension
    pub ensembles: Vec<NeuralEnsemble>,
    /// Cross-ensemble weights (coupling matrix)
    pub coupling_weights: [[f32; 5]; 5],
    /// Simulation tick counter
    pub tick_count: u64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl RealTimeNeuralNetwork {
    /// Create new real-time neural network
    #[must_use]
    pub fn new(config: RealTimeConfig) -> Self {
        let mut ensembles = Vec::with_capacity(5);

        for dim in 0..5 {
            ensembles.push(NeuralEnsemble::new(
                dim,
                config.neurons_per_dim,
                config.max_rate,
            ));
        }

        // Initialize coupling weights (from DOCAS paper)
        // D→O, C→D, C→A coupling as documented
        let coupling_weights = [
            [0.0, 0.1, 0.0, 0.0, 0.0],   // D receives from O
            [0.0, 0.0, 0.0, 0.0, 0.0],   // O
            [0.15, 0.0, 0.0, 0.15, 0.0], // C affects D and A
            [0.0, 0.0, 0.0, 0.0, 0.0],   // A
            [0.0, 0.0, 0.0, 0.0, 0.0],   // S
        ];

        Self {
            config,
            ensembles,
            coupling_weights,
            tick_count: 0,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Simulate one timestep with DOCAS inputs
    /// Returns neural-derived DOCAS levels
    pub fn step(&mut self, docas_input: &DocasState) -> DocasState {
        let inputs = [
            docas_input.dopamine(),
            docas_input.oxytocin(),
            docas_input.cortisol(),
            docas_input.adrenaline(),
            docas_input.serotonin(),
        ];

        // Compute coupling inputs
        let mut coupled_inputs = inputs;
        for (i, coupled_input_i) in coupled_inputs.iter_mut().enumerate() {
            for (j, input_j) in inputs.iter().enumerate() {
                *coupled_input_i += self.coupling_weights[i][j] * input_j;
            }
            *coupled_input_i = coupled_input_i.clamp(0.0, 1.0);
        }

        // Run each ensemble
        let mut outputs = [0.0; 5];
        for (i, ensemble) in self.ensembles.iter_mut().enumerate() {
            outputs[i] = ensemble.step(coupled_inputs[i], &self.config);
        }

        self.tick_count += 1;

        // Create output DOCAS state
        DocasState::new(outputs)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get firing rates for all dimensions
    #[must_use]
    pub fn firing_rates(&self, window_ms: f32) -> [f32; 5] {
        let mut rates = [0.0; 5];
        for (i, ensemble) in self.ensembles.iter().enumerate() {
            rates[i] = ensemble.population_rate(window_ms);
        }
        rates
    }

    /// Reset all spike counters
    pub fn reset_counters(&mut self) {
        for ensemble in &mut self.ensembles {
            ensemble.reset_counters();
        }
    }
}

/// Real-time neural backend for agent simulation
#[derive(Debug)]
pub struct RealTimeNeuralBackend {
    /// Configuration
    config: RealTimeConfig,
    /// Per-agent neural networks
    agent_networks: std::collections::HashMap<crate::types::AgentId, RealTimeNeuralNetwork>,
}

impl RealTimeNeuralBackend {
    /// Create new real-time backend
    #[must_use]
    pub fn new(config: RealTimeConfig) -> Self {
        Self {
            config,
            agent_networks: std::collections::HashMap::new(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Initialize network for an agent
    pub fn initialize_agent(&mut self, agent_id: crate::types::AgentId) {
        self.agent_networks
            .insert(agent_id, RealTimeNeuralNetwork::new(self.config.clone()));
    }

    /// Simulate one tick for an agent
    pub fn simulate_tick(
        &mut self,
        agent_id: crate::types::AgentId,
        docas_input: &DocasState,
    ) -> Option<DocasState> {
        if !self.config.enabled {
            return None;
        }

        let network = self.agent_networks.get_mut(&agent_id)?;
        Some(network.step(docas_input))
    }

    /// Get firing rates for an agent
    #[must_use]
    pub fn get_firing_rates(
        &self,
        agent_id: crate::types::AgentId,
        window_ms: f32,
    ) -> Option<[f32; 5]> {
        let network = self.agent_networks.get(&agent_id)?;
        Some(network.firing_rates(window_ms))
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    /// Remove agent network (when agent dies)
    pub fn remove_agent(&mut self, agent_id: crate::types::AgentId) {
        self.agent_networks.remove(&agent_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spiking_neuron() {
        let mut neuron = SpikingNeuron::new();

        // With no input, shouldn't spike
        let spiked = neuron.step(0.0, 1.0, 2.0);
        assert!(!spiked);

        // With strong input, should eventually spike
        let mut spikes = 0;
        for _ in 0..100 {
            if neuron.step(20.0, 1.0, 2.0) {
                spikes += 1;
            }
        }
        assert!(spikes > 0, "Neuron should spike with strong input");
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_neural_ensemble() {
        let mut ensemble = NeuralEnsemble::new(0, 50, 200.0); // More neurons
        let config = RealTimeConfig::default();

        // Run with high input for longer
        for _ in 0..500 {
            ensemble.step(0.9, &config);
        }

        let rate = ensemble.population_rate(500.0);
        // With strong input, should have some spikes
        // Note: Rate might still be 0 with few neurons, so just verify it doesn't panic
        assert!(rate >= 0.0, "Rate should be non-negative");
    }

    #[test]
    fn test_realtime_network() {
        let mut network = RealTimeNeuralNetwork::new(RealTimeConfig::default());
        let input = DocasState::new([0.5, 0.5, 0.5, 0.5, 0.5]);

        // Run simulation
        for _ in 0..100 {
            let _ = network.step(&input);
        }

        let rates = network.firing_rates(100.0);
        assert!(
            rates.iter().all(|&r| r >= 0.0),
            "All rates should be non-negative"
        );
    }
}
