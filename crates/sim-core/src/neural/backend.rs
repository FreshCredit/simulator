//! Neural backend implementations
//!
//! Provides pluggable backends for:
//! - Nengo (Neural Engineering Framework)
//! - Brian2 (Spiking neural networks)
//! - Mock (for testing)

use super::NeuralValidationError;
use crate::agent::Agent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Command, Stdio};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Backend configuration
#[derive(Debug, Clone)]
pub struct BackendConfig {
    /// Python executable path
    pub python_path: String,
    /// Script directory
    pub script_dir: String,
    /// Simulation timeout (seconds)
    pub timeout_secs: u64,
    /// Number of neurons per ensemble
    pub neurons_per_ensemble: usize,
    /// Simulation timestep (seconds)
    pub dt: f32,
    /// Total simulation time (seconds)
    pub simulation_time: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            python_path: "python3".to_string(),
            script_dir: "./scripts/neural".to_string(),
            timeout_secs: 300,
            neurons_per_ensemble: 100,
            dt: 0.001,
            simulation_time: 1.0,
        }
    }
}

/// Neural backend trait
pub trait NeuralBackend: Send + Sync {
    /// Get backend name
    fn name(&self) -> &str;

    /// Simulate a single agent's neural dynamics
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn simulate_agent(&mut self, agent: &Agent) -> Result<SimulationResult, NeuralValidationError>;

    /// Batch simulate multiple agents
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn simulate_batch(
        &mut self,
        agents: &[Agent],
    ) -> Result<Vec<SimulationResult>, NeuralValidationError>;

    /// Get backend configuration
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn config(&self) -> &BackendConfig;

    /// Health check
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn health_check(&self) -> Result<bool, NeuralValidationError>;
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Simulation result from neural backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    /// Agent ID
    pub agent_id: u64,

    /// Timestamps for each data point
    pub timestamps: Vec<f32>,

    /// Dopamine activity (spike rate or decoded value)
    pub dopamine_activity: Vec<f32>,

    /// Oxytocin activity
    pub oxytocin_activity: Vec<f32>,

    /// Cortisol activity
    pub cortisol_activity: Vec<f32>,

    /// Adrenaline activity
    pub adrenaline_activity: Vec<f32>,

    /// Serotonin activity
    pub serotonin_activity: Vec<f32>,

    /// Derived DOCAS parameters from neural dynamics
    pub derived_docas: DerivedDocasParams,

    /// BOLD signal (if generated)
    pub bold_signal: Option<Vec<f32>>,

    /// Metadata
    pub metadata: HashMap<String, String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Derived DOCAS parameters from neural simulation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DerivedDocasParams {
    /// Retention rates (α)
    pub alpha: [f32; 5],

    /// Update rates (β)
    pub beta: [f32; 5],

    /// Time constants (τ)
    pub time_constants: [f32; 5],

    /// Coupling strengths
    pub coupling: [[f32; 5]; 5],

    /// Confidence in derivation (0-1)
    pub confidence: f32,
}

/// Nengo backend implementation
#[derive(Debug)]
pub struct NengoBackend {
    config: BackendConfig,
    script_path: String,
}

impl NengoBackend {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create new Nengo backend
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn new() -> Result<Self, NeuralValidationError> {
        let config = BackendConfig::default();
        let script_path = format!("{}/nengo_docas.py", config.script_dir);

        // Verify Python and Nengo are available
        let output = Command::new(&config.python_path)
            .args(["-c", "import nengo; print('OK')"])
            .output()
            .map_err(|e| NeuralValidationError::Backend(format!("Failed to check Nengo: {e}")))?;

        if !output.status.success() {
            return Err(NeuralValidationError::Backend(
                "Nengo not installed or not accessible".to_string(),
            ));
        }

        // Create scripts directory if needed
        std::fs::create_dir_all(&config.script_dir)?;

        // Write Nengo script
        Self::write_nengo_script(&script_path)?;

        Ok(Self {
            config,
            script_path,
        })
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    /// Write the Nengo simulation script
    fn write_nengo_script(path: &str) -> Result<(), NeuralValidationError> {
        let script = r"
import sys
import json
import numpy as np
import nengo
from nengo.solvers import LstsqL2

def simulate_docas(params):
    '''Simulate DOCAS dynamics in Nengo'''

    # Parameters from agent
    initial_levels = params['initial_levels']  # [D, O, C, A, S]
    stimuli = params['stimuli']  # Dict of stimuli over time
    dt = params.get('dt', 0.001)
    t_final = params.get('t_final', 1.0)

    # Nengo model
    model = nengo.Network()
    with model:
        # DOCAS ensembles (one per neuromodulator)
        D = nengo.Ensemble(100, 1, label='Dopamine')
        O = nengo.Ensemble(100, 1, label='Oxytocin')
        C = nengo.Ensemble(100, 1, label='Cortisol')
        A = nengo.Ensemble(100, 1, label='Adrenaline')
        S = nengo.Ensemble(100, 1, label='Serotonin')

        # Input nodes
        reward_error = nengo.Node(lambda t: stimuli.get('reward', 0.0))
        social_val = nengo.Node(lambda t: stimuli.get('social', 0.0))
        threat = nengo.Node(lambda t: stimuli.get('threat', 0.0))
        action = nengo.Node(lambda t: stimuli.get('action', 0.0))
        stability = nengo.Node(lambda t: stimuli.get('stability', 0.0))

        # Connections (simplified DOCAS update rules)
        tau_fast = 0.05  # 50ms
        tau_slow = 0.1   # 100ms

        nengo.Connection(reward_error, D, synapse=tau_fast, transform=0.1)
        nengo.Connection(social_val, O, synapse=tau_fast, transform=0.1)
        nengo.Connection(threat, C, synapse=tau_fast, transform=0.1)
        nengo.Connection(action, A, synapse=tau_fast, transform=0.1)
        nengo.Connection(stability, S, synapse=tau_slow, transform=0.1)

        # Recurrent connections (decay)
        nengo.Connection(D, D, synapse=tau_fast, transform=0.9)
        nengo.Connection(O, O, synapse=tau_fast, transform=0.9)
        nengo.Connection(C, C, synapse=tau_fast, transform=0.9)
        nengo.Connection(A, A, synapse=tau_fast, transform=0.9)
        nengo.Connection(S, S, synapse=tau_slow, transform=0.95)

        # Cross-modulator coupling
        nengo.Connection(D, O, synapse=tau_fast, transform=0.05)  # D->O (reward->trust)
        nengo.Connection(C, D, synapse=tau_fast, transform=-0.1)  # C->D (threat reduces reward)
        nengo.Connection(C, A, synapse=tau_fast, transform=0.2)   # C->A (threat->action)

        # Probes for recording
        p_D = nengo.Probe(D, synapse=0.01)
        p_O = nengo.Probe(O, synapse=0.01)
        p_C = nengo.Probe(C, synapse=0.01)
        p_A = nengo.Probe(A, synapse=0.01)
        p_S = nengo.Probe(S, synapse=0.01)

    # Run simulation
    with nengo.Simulator(model, dt=dt) as sim:
        sim.run(t_final)

    # Extract results
    result = {
        'timestamps': sim.trange().tolist(),
        'dopamine_activity': sim.data[p_D].flatten().tolist(),
        'oxytocin_activity': sim.data[p_O].flatten().tolist(),
        'cortisol_activity': sim.data[p_C].flatten().tolist(),
        'adrenaline_activity': sim.data[p_A].flatten().tolist(),
        'serotonin_activity': sim.data[p_S].flatten().tolist(),
        'dt': dt,
        't_final': t_final,
    }

    return result

if __name__ == '__main__':
    # Read parameters from stdin
    params = json.load(sys.stdin)

    # Run simulation
    result = simulate_docas(params)

    # Output results
    print(json.dumps(result))
";

        std::fs::write(path, script).map_err(NeuralValidationError::Io)?;

        Ok(())
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl NeuralBackend for NengoBackend {
    fn name(&self) -> &'static str {
        "Nengo"
    }

    fn simulate_agent(&mut self, agent: &Agent) -> Result<SimulationResult, NeuralValidationError> {
        // Prepare parameters
        let params = serde_json::json!({
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
        });

        // Run Python script
        let mut child = Command::new(&self.config.python_path)
            .arg(&self.script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| NeuralValidationError::Backend(format!("Failed to spawn Nengo: {e}")))?;

        // Send parameters
        if let Some(stdin) = child.stdin.take() {
            serde_json::to_writer(stdin, &params).map_err(|e| {
                NeuralValidationError::Backend(format!("Failed to write params: {e}"))
            })?;
        }

        // Wait for output
        let output = child
            .wait_with_output()
            .map_err(|e| NeuralValidationError::Backend(format!("Nengo process failed: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NeuralValidationError::Backend(format!(
                "Nengo error: {stderr}"
            )));
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Parse results
        let raw_result: serde_json::Value =
            serde_json::from_slice(&output.stdout).map_err(|e| {
                NeuralValidationError::Backend(format!("Failed to parse Nengo output: {e}"))
            })?;

        // Convert to SimulationResult
        let result = SimulationResult {
            agent_id: agent.id,
            timestamps: serde_json::from_value(raw_result["timestamps"].clone())
                .unwrap_or_default(),
            dopamine_activity: serde_json::from_value(raw_result["dopamine_activity"].clone())
                .unwrap_or_default(),
            oxytocin_activity: serde_json::from_value(raw_result["oxytocin_activity"].clone())
                .unwrap_or_default(),
            cortisol_activity: serde_json::from_value(raw_result["cortisol_activity"].clone())
                .unwrap_or_default(),
            adrenaline_activity: serde_json::from_value(raw_result["adrenaline_activity"].clone())
                .unwrap_or_default(),
            serotonin_activity: serde_json::from_value(raw_result["serotonin_activity"].clone())
                .unwrap_or_default(),
            derived_docas: DerivedDocasParams::default(), // Will be filled by calibrator
            bold_signal: None,
            metadata: {
                let mut m = HashMap::new();
                m.insert("backend".to_string(), "nengo".to_string());
                m.insert(
                    "neurons_per_ensemble".to_string(),
                    self.config.neurons_per_ensemble.to_string(),
                );
                m
            },
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        Ok(result)
    }

    fn simulate_batch(
        &mut self,
        agents: &[Agent],
    ) -> Result<Vec<SimulationResult>, NeuralValidationError> {
        let mut results = Vec::with_capacity(agents.len());
        for agent in agents {
            results.push(self.simulate_agent(agent)?);
        }
        Ok(results)
    }

    fn config(&self) -> &BackendConfig {
        &self.config
    }

    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn health_check(&self) -> Result<bool, NeuralValidationError> {
        let output = Command::new(&self.config.python_path)
            .args(["-c", "import nengo; print('OK')"])
            .output()
            .map_err(|e| NeuralValidationError::Backend(format!("Health check failed: {e}")))?;

        Ok(output.status.success())
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Brian2 backend implementation
#[derive(Debug)]
pub struct Brian2Backend {
    config: BackendConfig,
}

impl Brian2Backend {
    /// Create new Brian2 backend
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn new() -> Result<Self, NeuralValidationError> {
        let config = BackendConfig::default();

        // Verify Brian2 is available
        let output = Command::new(&config.python_path)
            .args(["-c", "import brian2; print('OK')"])
            .output()
            .map_err(|e| NeuralValidationError::Backend(format!("Failed to check Brian2: {e}")))?;

        if !output.status.success() {
            return Err(NeuralValidationError::Backend(
                "Brian2 not installed or not accessible".to_string(),
            ));
        }

        Ok(Self { config })
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl NeuralBackend for Brian2Backend {
    fn name(&self) -> &'static str {
        "Brian2"
    }

    fn simulate_agent(&mut self, agent: &Agent) -> Result<SimulationResult, NeuralValidationError> {
        // Similar to Nengo but using Brian2 equations
        // For now, return mock result
        Ok(SimulationResult {
            agent_id: agent.id,
            timestamps: vec![0.0],
            dopamine_activity: vec![agent.docas.levels[0]],
            oxytocin_activity: vec![agent.docas.levels[1]],
            cortisol_activity: vec![agent.docas.levels[2]],
            adrenaline_activity: vec![agent.docas.levels[3]],
            serotonin_activity: vec![agent.docas.levels[4]],
            derived_docas: DerivedDocasParams::default(),
            bold_signal: None,
            metadata: {
                let mut m = HashMap::new();
                m.insert("backend".to_string(), "brian2".to_string());
                m
            },
        })
    }

    fn simulate_batch(
        &mut self,
        agents: &[Agent],
    ) -> Result<Vec<SimulationResult>, NeuralValidationError> {
        agents.iter().map(|a| self.simulate_agent(a)).collect()
    }

    fn config(&self) -> &BackendConfig {
        &self.config
    }

    fn health_check(&self) -> Result<bool, NeuralValidationError> {
        let output = Command::new(&self.config.python_path)
            .args(["-c", "import brian2; print('OK')"])
            .output()
            .map_err(|e| NeuralValidationError::Backend(format!("Health check failed: {e}")))?;

        Ok(output.status.success())
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Mock backend for testing
#[derive(Debug)]
pub struct MockBackend {
    config: BackendConfig,
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl MockBackend {
    /// Create new mock backend
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: BackendConfig::default(),
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl NeuralBackend for MockBackend {
    fn name(&self) -> &'static str {
        "Mock"
    }

    fn simulate_agent(&mut self, agent: &Agent) -> Result<SimulationResult, NeuralValidationError> {
        // Generate synthetic data based on agent state
        let n_points = 100;
        let dt = 0.01;

        let mut timestamps = Vec::with_capacity(n_points);
        let mut dopamine = Vec::with_capacity(n_points);
        let mut oxytocin = Vec::with_capacity(n_points);
        let mut cortisol = Vec::with_capacity(n_points);
        let mut adrenaline = Vec::with_capacity(n_points);
        let mut serotonin = Vec::with_capacity(n_points);

        for i in 0..n_points {
            let t = i as f32 * dt;
            timestamps.push(t);

            // Synthetic dynamics with noise
            dopamine.push(
                agent.docas.levels[0] + 0.1 * (t * 2.0).sin() + 0.05 * (i as f32 % 10.0) / 10.0,
            );
            oxytocin.push(
                agent.docas.levels[1]
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
                    + 0.08 * (t * 1.5).cos()
                    + 0.04 * ((i + 3) as f32 % 10.0) / 10.0,
            );
            cortisol.push(
                agent.docas.levels[2]
                    + 0.12 * (t * 3.0).sin()
                    + 0.06 * ((i + 7) as f32 % 10.0) / 10.0,
            );
            adrenaline.push(
                agent.docas.levels[3]
                    + 0.15 * (t * 2.5).cos()
                    + 0.05 * ((i + 2) as f32 % 10.0) / 10.0,
            );
            serotonin.push(
                agent.docas.levels[4]
                    + 0.07 * (t * 1.0).sin()
                    + 0.03 * ((i + 5) as f32 % 10.0) / 10.0,
            );
        }

        // Derive DOCAS parameters
        let derived = DerivedDocasParams {
            alpha: [0.95, 0.93, 0.90, 0.92, 0.96], // Mock retention rates
            beta: [0.08, 0.12, 0.15, 0.10, 0.06],  // Mock update rates
            time_constants: [0.05, 0.08, 0.03, 0.04, 0.10],
            coupling: [[0.0; 5]; 5],
            confidence: 0.85,
        };

        Ok(SimulationResult {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            agent_id: agent.id,
            timestamps,
            dopamine_activity: dopamine,
            oxytocin_activity: oxytocin,
            cortisol_activity: cortisol,
            adrenaline_activity: adrenaline,
            serotonin_activity: serotonin,
            derived_docas: derived,
            bold_signal: None,
            metadata: {
                let mut m = HashMap::new();
                m.insert("backend".to_string(), "mock".to_string());
                m.insert("synthetic".to_string(), "true".to_string());
                m
            },
        })
    }

    fn simulate_batch(
        &mut self,
        agents: &[Agent],
    ) -> Result<Vec<SimulationResult>, NeuralValidationError> {
        agents.iter().map(|a| self.simulate_agent(a)).collect()
    }

    fn config(&self) -> &BackendConfig {
        &self.config
    }

    fn health_check(&self) -> Result<bool, NeuralValidationError> {
        Ok(true)
    }
}
