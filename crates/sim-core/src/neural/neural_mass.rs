//! Neural Mass Models
//!
//! Population-level neural dynamics using mean-field approximations.
//! Per Wilson-Cowan (1972), Jansen-Rit (1995) models.
//! Maps to DOCAS as macroscopic brain states.

use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Neural mass model parameters
/// Per Wilson-Cowan: Excitatory and inhibitory population coupling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralMassParams {
    /// Excitatory time constant (ms)
    pub tau_e: f32,
    /// Inhibitory time constant (ms)
    pub tau_i: f32,
    /// Excitatory self-coupling
    pub c_ee: f32,
    /// Inhibitory to excitatory coupling
    pub c_ie: f32,
    /// Excitatory to inhibitory coupling
    pub c_ei: f32,
    /// Inhibitory self-coupling
    pub c_ii: f32,
    /// External input to excitatory
    pub p_e: f32,
    /// External input to inhibitory
    pub p_i: f32,
    /// Sigmoid steepness
    pub r: f32,
    /// Sigmoid threshold
    pub theta: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for NeuralMassParams {
    fn default() -> Self {
        // Wilson-Cowan typical parameters
        Self {
            tau_e: 10.0,
            tau_i: 20.0,
            c_ee: 16.0,
            c_ie: 12.0,
            c_ei: 15.0,
            c_ii: 3.0,
            p_e: 1.0,
            p_i: 0.0,
            r: 1.3,
            theta: 4.0,
        }
    }
}

impl NeuralMassParams {
    /// Jansen-Rit model parameters (for alpha rhythms)
    #[must_use]
    pub const fn jansen_rit() -> Self {
        Self {
            tau_e: 10.0,
            tau_i: 20.0,
            c_ee: 3.25,
            c_ie: 2.5,
            c_ei: 22.0,
            c_ii: 1.5,
            p_e: 220.0,
            p_i: 200.0,
            r: 0.56,
            theta: 6.0,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Wilson-Cowan neural mass state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WilsonCowanState {
    /// Excitatory activity [0, 1]
    pub e: f32,
    /// Inhibitory activity [0, 1]
    pub i: f32,
    /// Time (ms)
    pub t: f32,
}

impl Default for WilsonCowanState {
    fn default() -> Self {
        Self {
            e: 0.1,
            i: 0.1,
            t: 0.0,
        }
    }
}

impl WilsonCowanState {
    /// Sigmoid response function
    fn sigmoid(x: f32, r: f32, theta: f32) -> f32 {
        1.0 / (1.0 + (-r * (x - theta)).exp())
    }

    /// Compute derivatives (for Runge-Kutta integration)
    fn derivatives(&self, params: &NeuralMassParams) -> (f32, f32) {
        let s_e = Self::sigmoid(
            params.c_ee * self.e - params.c_ie * self.i + params.p_e,
            params.r,
            params.theta,
        );
        let s_i = Self::sigmoid(
            params.c_ei * self.e - params.c_ii * self.i + params.p_i,
            params.r,
            params.theta,
        );

        let de_dt = (-self.e + s_e) / params.tau_e;
        let di_dt = (-self.i + s_i) / params.tau_i;

        (de_dt, di_dt)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Step forward using simple Euler integration
    pub fn step(&mut self, params: &NeuralMassParams, dt: f32) {
        let (de, di) = self.derivatives(params);
        self.e = (self.e + de * dt).clamp(0.0, 1.0);
        self.i = (self.i + di * dt).clamp(0.0, 1.0);
        self.t += dt;
    }

    /// Simulate for specified duration
    pub fn simulate(
        &mut self,
        params: &NeuralMassParams,
        duration_ms: f32,
        dt: f32,
    ) -> Vec<(f32, f32, f32)> {
        let steps = (duration_ms / dt) as usize;
        let mut trajectory = Vec::with_capacity(steps);

        for _ in 0..steps {
            trajectory.push((self.t, self.e, self.i));
            self.step(params, dt);
        }

        trajectory
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check for oscillatory activity
    #[must_use]
    pub fn is_oscillatory(&self) -> bool {
        // Oscillations occur when e and i are both moderate
        self.e > 0.2 && self.e < 0.8 && self.i > 0.2 && self.i < 0.8
    }

    /// Map to DOCAS-like state
    #[must_use]
    pub fn to_docas_proxy(&self) -> [f32; 5] {
        // Approximate DOCAS from mass activity
        [
            self.e * 0.8,                  // D: dopamine-like (excitatory)
            (1.0 - self.i) * 0.5,          // O: oxytocin-like (low inhibition)
            self.i * 0.6,                  // C: cortisol-like (inhibitory)
            (self.e - self.i).abs() * 0.5, // A: adrenaline-like (imbalance)
            1.0 - (self.e + self.i) / 2.0, // S: serotonin-like (stability)
        ]
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Neural mass simulator for multiple coupled populations
#[derive(Debug)]
pub struct NeuralMassNetwork {
    /// Individual masses
    pub masses: Vec<WilsonCowanState>,
    /// Coupling matrix (weights between masses)
    pub coupling: Vec<Vec<f32>>,
    /// Parameters for each mass
    pub params: Vec<NeuralMassParams>,
}

impl NeuralMassNetwork {
    /// Create network with n identical masses
    #[must_use]
    pub fn new(n: usize, params: NeuralMassParams) -> Self {
        let masses = vec![WilsonCowanState::default(); n];
        let params_vec = vec![params; n];
        // No coupling initially
        let coupling = vec![vec![0.0; n]; n];

        Self {
            masses,
            coupling,
            params: params_vec,
        }
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Set coupling between two masses
    pub fn set_coupling(&mut self, from: usize, to: usize, weight: f32) {
        if from < self.masses.len() && to < self.masses.len() {
            self.coupling[to][from] = weight;
        }
    }

    /// Step all masses forward
    pub fn step(&mut self, dt: f32) {
        let n = self.masses.len();
        let mut inputs = vec![0.0f32; n];

        // Compute coupling inputs
        for (i, inputs_i) in inputs.iter_mut().enumerate().take(n) {
            for (j, mass_j) in self.masses.iter().enumerate().take(n) {
                *inputs_i += self.coupling[i][j] * mass_j.e;
            }
        }

        // Step each mass with coupling input
        for (i, mass) in self.masses.iter_mut().enumerate().take(n) {
            // Add coupling as external input
            let mut params = self.params[i].clone();
            params.p_e += inputs[i];
            mass.step(&params, dt);
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get mean activity across all masses
    #[must_use]
    pub fn mean_activity(&self) -> (f32, f32) {
        if self.masses.is_empty() {
            return (0.0, 0.0);
        }
        let sum_e: f32 = self.masses.iter().map(|m| m.e).sum();
        let sum_i: f32 = self.masses.iter().map(|m| m.i).sum();
        (
            sum_e / self.masses.len() as f32,
            sum_i / self.masses.len() as f32,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wilson_cowan_creation() {
        let state = WilsonCowanState::default();
        assert!(state.e >= 0.0 && state.e <= 1.0);
        assert!(state.i >= 0.0 && state.i <= 1.0);
    }

    #[test]
    fn test_wilson_cowan_step() {
        let mut state = WilsonCowanState::default();
        let params = NeuralMassParams::default();

        state.step(&params, 0.1);

        // State should remain bounded
        assert!(state.e >= 0.0 && state.e <= 1.0);
        assert!(state.i >= 0.0 && state.i <= 1.0);
    }

    #[test]
    fn test_sigmoid_function() {
        let s = WilsonCowanState::sigmoid(0.0, 1.0, 0.0);
        assert!((s - 0.5).abs() < 0.01);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Large positive input → ~1
        let s_pos = WilsonCowanState::sigmoid(10.0, 1.0, 0.0);
        assert!(s_pos > 0.99);

        // Large negative input → ~0
        let s_neg = WilsonCowanState::sigmoid(-10.0, 1.0, 0.0);
        assert!(s_neg < 0.01);
    }

    #[test]
    fn test_simulation_trajectory() {
        let mut state = WilsonCowanState::default();
        let params = NeuralMassParams::default();

        let traj = state.simulate(&params, 100.0, 1.0);

        // Should have ~100 steps
        assert!(traj.len() >= 90 && traj.len() <= 110);

        // All values should be bounded
        for (_, e, i) in &traj {
            assert!((&0.0..=&1.0).contains(&e));
            assert!((&0.0..=&1.0).contains(&i));
        }
    }

    #[test]
    fn test_to_docas_proxy() {
        let state = WilsonCowanState {
            e: 0.5,
            i: 0.3,
            t: 0.0,
        };
        let docas = state.to_docas_proxy();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(docas.len(), 5);

        // All values should be in [0, 1]
        for v in &docas {
            assert!((&0.0..=&1.0).contains(&v));
        }
    }

    #[test]
    fn test_neural_mass_network() {
        let mut network = NeuralMassNetwork::new(3, NeuralMassParams::default());

        // Set some coupling
        network.set_coupling(0, 1, 0.5);
        network.set_coupling(1, 0, 0.3);

        // Step network
        network.step(0.1);

        // Get mean activity
        let (mean_e, mean_i) = network.mean_activity();
        assert!((0.0..=1.0).contains(&mean_e));
        assert!((0.0..=1.0).contains(&mean_i));
    }

    #[test]
    fn test_jansen_rit_params() {
        let params = NeuralMassParams::jansen_rit();

        // Jansen-Rit has different parameter ranges
        assert!(params.c_ei > params.c_ee); // Stronger inhibition
    }
}
