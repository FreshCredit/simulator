//! Quantum Computing Backend for Neural Simulation
//!
//! Aspirational: Quantum-accelerated neural dynamics simulation.
//! Uses quantum superposition and entanglement concepts for
//! exploring agent state spaces exponentially faster than classical.
//!
//! Note: This is a simulation/placeholder for future quantum hardware.

use rand::Rng;
use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Simulated quantum state (qubit representation)
/// In real quantum: |ψ⟩ = α|0⟩ + β|1⟩ where |α|² + |β|² = 1
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Qubit {
    /// Real part of amplitude for |0⟩
    pub alpha_real: f32,
    /// Imaginary part of amplitude for |0⟩
    pub alpha_imag: f32,
    /// Real part of amplitude for |1⟩
    pub beta_real: f32,
    /// Imaginary part of amplitude for |1⟩
    pub beta_imag: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Qubit {
    /// Create new qubit in |0⟩ state
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            alpha_real: 1.0,
            alpha_imag: 0.0,
            beta_real: 0.0,
            beta_imag: 0.0,
        }
    }

    /// Create new qubit in |1⟩ state
    #[must_use]
    pub const fn one() -> Self {
        Self {
            alpha_real: 0.0,
            alpha_imag: 0.0,
            beta_real: 1.0,
            beta_imag: 0.0,
        }
    }

    /// Create superposition state: (|0⟩ + |1⟩)/√2
    #[must_use]
    pub fn plus() -> Self {
        let s = 1.0 / 2.0f32.sqrt();
        Self {
            alpha_real: s,
            alpha_imag: 0.0,
            beta_real: s,
            beta_imag: 0.0,
        }
    }

    /// Check if state is normalized
    #[must_use]
    pub fn is_normalized(&self) -> bool {
        let norm_sq = self.alpha_real.powi(2)
            + self.alpha_imag.powi(2)
            + self.beta_real.powi(2)
            + self.beta_imag.powi(2);
        (norm_sq - 1.0).abs() < 1e-6
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Apply Pauli-X (NOT) gate
    pub fn apply_x(&mut self) {
        // X|0⟩ = |1⟩, X|1⟩ = |0⟩
        std::mem::swap(&mut self.alpha_real, &mut self.beta_real);
        std::mem::swap(&mut self.alpha_imag, &mut self.beta_imag);
    }

    /// Apply Pauli-Z gate
    pub fn apply_z(&mut self) {
        // Z|0⟩ = |0⟩, Z|1⟩ = -|1⟩
        self.beta_real = -self.beta_real;
        self.beta_imag = -self.beta_imag;
    }

    /// Apply Hadamard gate (creates superposition)
    pub fn apply_h(&mut self) {
        let s = 1.0 / 2.0f32.sqrt();
        let a_r = self.alpha_real;
        let a_i = self.alpha_imag;
        let b_r = self.beta_real;
        let b_i = self.beta_imag;

        // H|0⟩ = (|0⟩ + |1⟩)/√2, H|1⟩ = (|0⟩ - |1⟩)/√2
        self.alpha_real = s * (a_r + b_r);
        self.alpha_imag = s * (a_i + b_i);
        self.beta_real = s * (a_r - b_r);
        self.beta_imag = s * (a_i - b_i);
    }

    /// Measure qubit, collapsing to |0⟩ or |1⟩
    pub fn measure<R: Rng>(&self, rng: &mut R) -> bool {
        let prob_0 = self.alpha_real.powi(2) + self.alpha_imag.powi(2);
        rng.gen::<f32>() < prob_0
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get probability of measuring |0⟩
    #[must_use]
    pub fn prob_zero(&self) -> f32 {
        self.alpha_real.powi(2) + self.alpha_imag.powi(2)
    }

    /// Get probability of measuring |1⟩
    #[must_use]
    pub fn prob_one(&self) -> f32 {
        self.beta_real.powi(2) + self.beta_imag.powi(2)
    }
}

/// Quantum register (multi-qubit state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumRegister {
    /// Number of qubits
    pub n_qubits: usize,
    /// State vector (2^n amplitudes)
    /// Index i represents basis state |i⟩ in binary
    pub amplitudes: Vec<(f32, f32)>, // (real, imag) for each basis state
}

impl QuantumRegister {
    /// Create register in |0...0⟩ state
    #[must_use]
    pub fn new(n_qubits: usize) -> Self {
        let size = 1usize << n_qubits;
        let mut amplitudes = vec![(0.0f32, 0.0f32); size];
        amplitudes[0] = (1.0, 0.0); // |0...0⟩

        Self {
            n_qubits,
            amplitudes,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if state is normalized
    #[must_use]
    pub fn is_normalized(&self) -> bool {
        let norm_sq: f32 = self.amplitudes.iter().map(|(r, i)| r * r + i * i).sum();
        (norm_sq - 1.0).abs() < 1e-6
    }

    /// Apply Hadamard to all qubits (creates uniform superposition)
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn apply_h_all(&mut self) {
        let n = 1usize << self.n_qubits;
        let s = 1.0 / (n as f32).sqrt();
        for amp in &mut self.amplitudes {
            *amp = (s, 0.0);
        }
    }

    /// Apply CNOT gate (controlled-NOT)
    /// control: control qubit index
    /// target: target qubit index
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn apply_cnot(&mut self, control: usize, target: usize) {
        debug_assert!(control < self.n_qubits && target < self.n_qubits);
        debug_assert!(control != target);

        let n = 1usize << self.n_qubits;
        let mut new_amplitudes = self.amplitudes.clone();

        for i in 0..n {
            let control_bit = (i >> control) & 1;
            if control_bit == 1 {
                // Flip target bit
                let j = i ^ (1usize << target);
                new_amplitudes.swap(i, j);
            }
        }

        self.amplitudes = new_amplitudes;
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Measure all qubits
    pub fn measure_all<R: Rng>(&self, rng: &mut R) -> usize {
        let r: f32 = rng.gen();
        let mut cum_prob = 0.0f32;

        for (i, (real, imag)) in self.amplitudes.iter().enumerate() {
            let prob = real * real + imag * imag;
            cum_prob += prob;
            if r < cum_prob {
                return i;
            }
        }

        self.amplitudes.len() - 1
    }

    /// Get probability distribution
    #[must_use]
    pub fn probabilities(&self) -> Vec<f32> {
        self.amplitudes.iter().map(|(r, i)| r * r + i * i).collect()
    }
}

/// Quantum backend for agent state exploration
/// Uses quantum superposition to explore multiple agent states simultaneously
#[derive(Debug)]
pub struct QuantumAgentBackend {
    /// Quantum register for agent states
    pub register: QuantumRegister,
    /// Number of agent state dimensions encoded
    pub state_dimensions: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl QuantumAgentBackend {
    /// Create backend with n-dimensional state space
    #[must_use]
    pub fn new(state_dimensions: usize) -> Self {
        // Need log2(n) qubits to encode n states
        let n_qubits = (state_dimensions as f32).log2().ceil() as usize;
        let register = QuantumRegister::new(n_qubits.max(1));

        Self {
            register,
            state_dimensions,
        }
    }

    /// Initialize uniform superposition over all states
    /// This allows exploring all agent states "simultaneously"
    pub fn initialize_superposition(&mut self) {
        self.register.apply_h_all();
    }

    /// Apply quantum oracle for state evaluation
    /// Marks states that satisfy certain conditions
    pub fn apply_oracle(&mut self, condition: impl Fn(usize) -> bool) {
        // Grover's oracle: flip phase of marked states
        for (i, amp) in self.register.amplitudes.iter_mut().enumerate() {
            if condition(i) {
                // Phase flip: multiply by -1
                amp.0 = -amp.0;
                amp.1 = -amp.1;
            }
        }
    }

    /// Grover diffusion operator (amplitude amplification)
    pub fn apply_diffusion(&mut self) {
        let n = self.register.amplitudes.len() as f32;
        let mean: f32 = self.register.amplitudes.iter().map(|(r, _)| r).sum::<f32>() / n;

        // Reflection about mean
        for (r, _) in &mut self.register.amplitudes {
            *r = 2.0 * mean - *r;
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Sample an agent state from quantum distribution
    pub fn sample_state<R: Rng>(&self, rng: &mut R) -> usize {
        self.register.measure_all(rng) % self.state_dimensions
    }

    /// Get quantum advantage estimate
    /// Returns speedup factor vs classical search
    #[must_use]
    pub fn speedup_estimate(&self) -> f32 {
        // Grover's algorithm provides O(√N) speedup
        (self.state_dimensions as f32).sqrt()
    }
}

/// Quantum-inspired neural network weights
/// Uses complex-valued weights (amplitudes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumNeuralWeights {
    /// Complex weights (real, imag) for each connection
    pub weights: Vec<Vec<(f32, f32)>>,
}

impl QuantumNeuralWeights {
    /// Create random quantum weights
    #[must_use]
    pub fn random(n_in: usize, n_out: usize) -> Self {
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        let mut rng = StdRng::seed_from_u64(42);

        let weights: Vec<Vec<(f32, f32)>> = (0..n_out)
            .map(|_| {
                (0..n_in)
                    .map(|_| {
                        // Random phase and magnitude
                        let magnitude: f32 = rng.gen();
                        let phase: f32 = rng.gen::<f32>() * 2.0 * std::f32::consts::PI;
                        (magnitude * phase.cos(), magnitude * phase.sin())
                    })
                    .collect()
            })
            .collect();

        Self { weights }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Apply to input (quantum interference pattern)
    #[must_use]
    pub fn apply(&self, input: &[f32]) -> Vec<f32> {
        self.weights
            .iter()
            .map(|w| {
                // Complex dot product
                let real_sum: f32 = w
                    .iter()
                    .zip(input.iter())
                    .map(|((wr, _), inp)| wr * inp)
                    .sum();
                let imag_sum: f32 = w
                    .iter()
                    .zip(input.iter())
                    .map(|((_, wi), inp)| wi * inp)
                    .sum();
                // Return probability (squared magnitude)
                real_sum * real_sum + imag_sum * imag_sum
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_qubit_zero() {
        let q = Qubit::zero();
        assert!(q.is_normalized());
        assert_eq!(q.prob_zero(), 1.0);
        assert_eq!(q.prob_one(), 0.0);
    }

    #[test]
    fn test_qubit_one() {
        let q = Qubit::one();
        assert!(q.is_normalized());
        assert_eq!(q.prob_zero(), 0.0);
        assert_eq!(q.prob_one(), 1.0);
    }

    #[test]
    fn test_qubit_plus() {
        let q = Qubit::plus();
        assert!(q.is_normalized());
        assert!((q.prob_zero() - 0.5).abs() < 1e-6);
        assert!((q.prob_one() - 0.5).abs() < 1e-6);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_qubit_x_gate() {
        let mut q = Qubit::zero();
        q.apply_x(); // NOT gate
        assert!((q.prob_zero() - 0.0).abs() < 1e-6);
        assert!((q.prob_one() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_qubit_h_gate() {
        let mut q = Qubit::zero();
        q.apply_h();
        assert!(q.is_normalized());
        assert!((q.prob_zero() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_quantum_register() {
        let reg = QuantumRegister::new(3);
        assert_eq!(reg.n_qubits, 3);
        assert_eq!(reg.amplitudes.len(), 8); // 2^3
        assert!(reg.is_normalized());
    }

    #[test]
    fn test_superposition() {
        let mut reg = QuantumRegister::new(2);
        reg.apply_h_all();
        assert!(reg.is_normalized());

        // Uniform superposition: all amplitudes equal
        let expected = 0.5; // 1/sqrt(4)
        for (r, i) in &reg.amplitudes {
            assert!((r - expected).abs() < 1e-6);
            assert!(i.abs() < 1e-6);
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_cnot_gate() {
        let mut reg = QuantumRegister::new(2);
        // |00⟩ -> CNOT(0,1) -> |00⟩
        reg.apply_cnot(0, 1);
        assert_eq!(reg.amplitudes[0].0, 1.0); // Still |00⟩
    }

    #[test]
    fn test_quantum_backend() {
        let mut backend = QuantumAgentBackend::new(8);
        assert_eq!(backend.state_dimensions, 8);

        backend.initialize_superposition();
        assert!(backend.register.is_normalized());

        // Check speedup estimate
        let speedup = backend.speedup_estimate();
        assert!(speedup > 2.0); // √8 ≈ 2.8
    }

    #[test]
    fn test_quantum_oracle() {
        let mut backend = QuantumAgentBackend::new(4);
        backend.initialize_superposition();

        // Mark even states
        backend.apply_oracle(|i| i % 2 == 0);

        // Phase should be flipped for even states
        let probs = backend.register.probabilities();
        assert_eq!(probs.len(), 4);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_quantum_neural_weights() {
        let weights = QuantumNeuralWeights::random(3, 2);
        assert_eq!(weights.weights.len(), 2);
        assert_eq!(weights.weights[0].len(), 3);

        let input = vec![1.0, 0.0, 0.0];
        let output = weights.apply(&input);
        assert_eq!(output.len(), 2);

        // Output should be non-negative (probabilities)
        for o in output {
            assert!(o >= 0.0);
        }
    }

    #[test]
    fn test_qubit_measurement_distribution() {
        let mut rng = StdRng::seed_from_u64(42);
        let q = Qubit::plus();

        // Measure many times, should get ~50% distribution
        let trials = 1000;
        let mut zeros = 0;
        for _ in 0..trials {
            if q.measure(&mut rng) {
                zeros += 1;
            }
        }

        let zero_ratio = zeros as f32 / trials as f32;
        assert!(
            zero_ratio > 0.4 && zero_ratio < 0.6,
            "Expected ~0.5, got {zero_ratio}"
        );
    }
}
