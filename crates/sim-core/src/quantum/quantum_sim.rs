//! Quantum-Resistant Cryptography Module
//!
//! Implements post-quantum cryptographic options for post-quantum security.
//! Current SHA-256 is vulnerable to quantum attacks (Grover's algorithm).
//! This module provides lattice-based and hash-based signatures for quantum resistance.
// TAG: surface=simulation owner=research-team rule=SIM-001
//!
//! ## Post-Quantum Algorithms
//!
//! - **CRYSTALS-Dilithium**: Lattice-based digital signature scheme (NIST PQC Standard)
//!   - Security based on module-LWE and module-SIS problems
//!   - Fast signature generation and verification
//!   - Relatively small signatures and public keys
//!
//! - **SPHINCS+**: Stateless hash-based signature scheme (NIST PQC Standard)
//!   - Conservative security based purely on hash function properties
//!   - Larger signatures but well-understood security assumptions
//!   - No structural mathematical assumptions
//!
//! - **Hybrid Mode**: Combines classical + post-quantum for defense in depth
//!   - Provides protection if either algorithm class remains secure

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt::Write as FmtWrite;

/// Complex number for quantum state representation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Complex {
    /// Real component
    pub re: f32,
    /// Imaginary component
    pub im: f32,
}

impl Complex {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Create a new complex number
    #[must_use]
    pub const fn new(re: f32, im: f32) -> Self {
        Self { re, im }
    }

    /// Create a complex number from real part only
    #[must_use]
    pub const fn real(re: f32) -> Self {
        Self { re, im: 0.0 }
    }

    /// Create the imaginary unit i
    #[must_use]
    pub const fn i() -> Self {
        Self { re: 0.0, im: 1.0 }
    }

    /// Zero complex number
    #[must_use]
    pub const fn zero() -> Self {
        Self { re: 0.0, im: 0.0 }
    }

    /// One complex number
    #[must_use]
    pub const fn one() -> Self {
        Self { re: 1.0, im: 0.0 }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Add two complex numbers
    #[allow(clippy::should_implement_trait)]
    #[must_use]
    pub fn add(self, other: Self) -> Self {
        Self {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }

    /// Multiply two complex numbers
    #[allow(clippy::should_implement_trait)]
    #[must_use]
    pub fn mul(self, other: Self) -> Self {
        Self {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }

    /// Scale by a real number
    #[must_use]
    pub fn scale(self, scalar: f32) -> Self {
        Self {
            re: self.re * scalar,
            im: self.im * scalar,
        }
    }

    /// Complex conjugate
    #[must_use]
    pub fn conj(self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    /// Magnitude squared (probability)
    #[must_use]
    pub fn norm_sqr(&self) -> f32 {
        self.re * self.re + self.im * self.im
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Magnitude
    #[must_use]
    pub fn norm(&self) -> f32 {
        self.norm_sqr().sqrt()
    }
}

impl Default for Complex {
    fn default() -> Self {
        Self::zero()
    }
}

/// Quantum gate types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum QuantumGate {
    /// Hadamard gate - creates superposition
    /// H|0⟩ = (|0⟩ + |1⟩)/√2, H|1⟩ = (|0⟩ - |1⟩)/√2
    Hadamard(usize),
    /// Pauli-X gate (NOT gate)
    /// X|0⟩ = |1⟩, X|1⟩ = |0⟩
    PauliX(usize),
    /// Pauli-Y gate
    /// Y|0⟩ = i|1⟩, Y|1⟩ = -i|0⟩
    PauliY(usize),
    /// Pauli-Z gate (phase flip)
    /// Z|0⟩ = |0⟩, Z|1⟩ = -|1⟩
    PauliZ(usize),
    /// CNOT gate (controlled-NOT)
    /// |00⟩→|00⟩, |01⟩→|01⟩, |10⟩→|11⟩, |11⟩→|10⟩
    CNOT(usize, usize),
    /// Toffoli gate (controlled-controlled-NOT)
    Toffoli(usize, usize, usize),
    /// Phase gate (S gate)
    /// S|0⟩ = |0⟩, S|1⟩ = i|1⟩
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    Phase(usize),
    /// T gate (π/8 gate)
    /// T|0⟩ = |0⟩, T|1⟩ = e^(iπ/4)|1⟩
    T(usize),
    /// Rotation around X axis
    RotationX(usize, f32),
    /// Rotation around Y axis
    RotationY(usize, f32),
    /// Rotation around Z axis
    RotationZ(usize, f32),
    /// Swap gate
    Swap(usize, usize),
}

/// Noise model for quantum simulation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum NoiseModel {
    /// No noise (ideal quantum computer)
    #[default]
    None,
    /// Depolarizing noise - qubit becomes maximally mixed with probability p
    Depolarizing(f32),
    /// Dephasing noise - loss of phase coherence
    Dephasing(f32),
    /// Amplitude damping - energy dissipation
    AmplitudeDamping(f32),
    /// Combined noise model
    Combined {
        /// depolarizing.
        depolarizing: f32,
        /// dephasing.
        dephasing: f32,
        /// amplitude damping.
        amplitude_damping: f32,
    },
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl NoiseModel {
    /// Check if this noise model represents a fault-tolerant regime
    #[must_use]
    pub fn is_fault_tolerant(&self) -> bool {
        match self {
            Self::None => true,
            Self::Depolarizing(p) => *p < 0.001,
            Self::Dephasing(p) => *p < 0.001,
            Self::AmplitudeDamping(p) => *p < 0.001,
            Self::Combined {
                depolarizing,
                dephasing,
                amplitude_damping,
            } => *depolarizing < 0.001 && *dephasing < 0.001 && *amplitude_damping < 0.001,
        }
    }

    /// Get total error probability
    #[must_use]
    pub fn total_error_probability(&self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Depolarizing(p) => *p,
            Self::Dephasing(p) => *p,
            Self::AmplitudeDamping(p) => *p,
            Self::Combined {
                depolarizing,
                dephasing,
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                amplitude_damping,
            } => (depolarizing + dephasing + amplitude_damping).min(1.0),
        }
    }
}

/// Quantum measurement result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeasurementResult {
    /// Measured |0⟩
    Zero,
    /// Measured |1⟩
    One,
}

impl MeasurementResult {
    /// Convert to boolean
    #[must_use]
    pub const fn as_bool(&self) -> bool {
        matches!(self, Self::One)
    }

    /// Convert from boolean
    #[must_use]
    pub const fn from_bool(val: bool) -> Self {
        if val {
            Self::One
        } else {
            Self::Zero
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Quantum computing simulator for cryptographic analysis
///
/// Simulates a quantum computer using state vector representation.
/// Used to test post-quantum cryptographic resistance against quantum attacks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumSimulator {
    /// Number of qubits in the system
    pub num_qubits: u32,
    /// State vector representing quantum state
    /// Length is 2^n where n is `num_qubits`
    pub state_vector: Vec<Complex>,
    /// History of applied gates
    pub gates: Vec<QuantumGate>,
    /// Noise model for realistic simulation
    pub noise_model: NoiseModel,
    /// Number of measurements performed
    pub measurement_count: usize,
    /// Random seed for deterministic simulation
    pub seed: u64,
}

impl QuantumSimulator {
    /// Create a new quantum simulator with n qubits initialized to |0...0⟩
    #[must_use]
    pub fn new(num_qubits: u32) -> Self {
        let safe_qubits = num_qubits.min(24);
        let size = 1usize << safe_qubits; // 2^n
        let mut state = vec![Complex::zero(); size];
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        state[0] = Complex::one(); // |0...0⟩ state

        Self {
            num_qubits: safe_qubits,
            state_vector: state,
            gates: Vec::new(),
            noise_model: NoiseModel::None,
            measurement_count: 0,
            seed: 0,
        }
    }

    /// Create a simulator with specific seed for deterministic testing
    #[must_use]
    pub fn with_seed(num_qubits: u32, seed: u64) -> Self {
        let mut sim = Self::new(num_qubits);
        sim.seed = seed;
        sim
    }

    /// Set the noise model
    #[must_use]
    pub const fn with_noise(mut self, noise: NoiseModel) -> Self {
        self.noise_model = noise;
        self
    }

    /// Get the dimension of the state space (2^n)
    #[must_use]
    pub const fn dimension(&self) -> usize {
        1usize << self.num_qubits
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if state vector is normalized (sum of probabilities = 1)
    #[must_use]
    pub fn is_normalized(&self) -> bool {
        let total_prob: f32 = self.state_vector.iter().map(Complex::norm_sqr).sum();
        (total_prob - 1.0).abs() < 1e-5
    }

    /// Apply a quantum gate to the state
    pub fn apply_gate(&mut self, gate: QuantumGate) {
        match gate {
            QuantumGate::Hadamard(q) => self.apply_hadamard(q),
            QuantumGate::PauliX(q) => self.apply_pauli_x(q),
            QuantumGate::PauliY(q) => self.apply_pauli_y(q),
            QuantumGate::PauliZ(q) => self.apply_pauli_z(q),
            QuantumGate::CNOT(c, t) => self.apply_cnot(c, t),
            QuantumGate::Toffoli(c1, c2, t) => self.apply_toffoli(c1, c2, t),
            QuantumGate::Phase(q) => self.apply_phase(q),
            QuantumGate::T(q) => self.apply_t(q),
            QuantumGate::RotationX(q, angle) => self.apply_rotation_x(q, angle),
            QuantumGate::RotationY(q, angle) => self.apply_rotation_y(q, angle),
            QuantumGate::RotationZ(q, angle) => self.apply_rotation_z(q, angle),
            QuantumGate::Swap(q1, q2) => self.apply_swap(q1, q2),
        }
        self.gates.push(gate);
    }

    /// Apply Hadamard gate to create superposition
    fn apply_hadamard(&mut self, qubit: usize) {
        let h = 1.0 / 2.0_f32.sqrt();
        let n = self.num_qubits as usize;

        if qubit >= n {
            return;
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        }

        let mut new_state = self.state_vector.clone();
        let mask = 1usize << qubit;

        for i in 0..self.dimension() {
            let j = i ^ mask; // Flip the qubit

            if i < j {
                let ai = self.state_vector[i];
                let aj = self.state_vector[j];

                // H|0⟩ = (|0⟩ + |1⟩)/√2, H|1⟩ = (|0⟩ - |1⟩)/√2
                if (i & mask) == 0 {
                    // |0⟩ component
                    new_state[i] = Complex::new(h * (ai.re + aj.re), h * (ai.im + aj.im));
                    new_state[j] = Complex::new(h * (ai.re - aj.re), h * (ai.im - aj.im));
                }
            }
        }

        self.state_vector = new_state;
    }

    /// Apply Pauli-X (NOT) gate
    fn apply_pauli_x(&mut self, qubit: usize) {
        let n = self.num_qubits as usize;
        if qubit >= n {
            return;
        }

        let mask = 1usize << qubit;
        let mut new_state = self.state_vector.clone();

        for i in 0..self.dimension() {
            let j = i ^ mask;
            if i < j {
                new_state.swap(i, j);
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        self.state_vector = new_state;
    }

    /// Apply Pauli-Y gate
    fn apply_pauli_y(&mut self, qubit: usize) {
        let n = self.num_qubits as usize;
        if qubit >= n {
            return;
        }

        let mask = 1usize << qubit;
        let mut new_state = self.state_vector.clone();

        for i in 0..self.dimension() {
            let j = i ^ mask;
            if i < j {
                let ai = self.state_vector[i];
                let aj = self.state_vector[j];

                // Y|0⟩ = i|1⟩, Y|1⟩ = -i|0⟩
                new_state[i] = Complex::new(aj.im, -aj.re); // i * aj
                new_state[j] = Complex::new(-ai.im, ai.re); // -i * ai
            }
        }

        self.state_vector = new_state;
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    /// Apply Pauli-Z (phase flip) gate
    fn apply_pauli_z(&mut self, qubit: usize) {
        let n = self.num_qubits as usize;
        if qubit >= n {
            return;
        }

        let mask = 1usize << qubit;

        for i in 0..self.dimension() {
            if (i & mask) != 0 {
                // Z|1⟩ = -|1⟩
                self.state_vector[i] = self.state_vector[i].scale(-1.0);
            }
        }
    }

    /// Apply CNOT (controlled-NOT) gate
    fn apply_cnot(&mut self, control: usize, target: usize) {
        let n = self.num_qubits as usize;
        if control >= n || target >= n || control == target {
            return;
        }

        let control_mask = 1usize << control;
        let target_mask = 1usize << target;
        let mut new_state = self.state_vector.clone();

        for (i, new_state_i) in new_state.iter_mut().enumerate().take(self.dimension()) {
            if (i & control_mask) != 0 {
                // Control is |1⟩, flip target
                let j = i ^ target_mask;
                *new_state_i = self.state_vector[j];
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        self.state_vector = new_state;
    }

    /// Apply Toffoli (CCNOT) gate
    fn apply_toffoli(&mut self, control1: usize, control2: usize, target: usize) {
        let n = self.num_qubits as usize;
        if control1 >= n
            || control2 >= n
            || target >= n
            || control1 == control2
            || control1 == target
            || control2 == target
        {
            return;
        }

        let c1_mask = 1usize << control1;
        let c2_mask = 1usize << control2;
        let target_mask = 1usize << target;
        let mut new_state = self.state_vector.clone();

        for (i, new_state_i) in new_state.iter_mut().enumerate().take(self.dimension()) {
            if (i & c1_mask) != 0 && (i & c2_mask) != 0 {
                // Both controls are |1⟩, flip target
                let j = i ^ target_mask;
                *new_state_i = self.state_vector[j];
            }
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        }

        self.state_vector = new_state;
    }

    /// Apply Phase (S) gate
    fn apply_phase(&mut self, qubit: usize) {
        let n = self.num_qubits as usize;
        if qubit >= n {
            return;
        }

        let mask = 1usize << qubit;

        for i in 0..self.dimension() {
            if (i & mask) != 0 {
                // S|1⟩ = i|1⟩
                let c = self.state_vector[i];
                self.state_vector[i] = Complex::new(-c.im, c.re);
            }
        }
    }

    /// Apply T gate
    fn apply_t(&mut self, qubit: usize) {
        let n = self.num_qubits as usize;
        if qubit >= n {
            return;
        }

        let mask = 1usize << qubit;
        let phase = Complex::new(1.0 / 2.0_f32.sqrt(), 1.0 / 2.0_f32.sqrt());

        // TAG: surface=simulation owner=research-team rule=SIM-001
        for i in 0..self.dimension() {
            if (i & mask) != 0 {
                let c = self.state_vector[i];
                self.state_vector[i] = c.mul(phase);
            }
        }
    }

    /// Apply rotation around X axis
    fn apply_rotation_x(&mut self, qubit: usize, angle: f32) {
        let n = self.num_qubits as usize;
        if qubit >= n {
            return;
        }

        let mask = 1usize << qubit;
        let cos = (angle / 2.0).cos();
        let sin = (angle / 2.0).sin();
        let mut new_state = self.state_vector.clone();

        for i in 0..self.dimension() {
            let j = i ^ mask;
            if i < j {
                let ai = self.state_vector[i];
                let aj = self.state_vector[j];

                // R_x(theta) = cos(theta/2) * I - i * sin(theta/2) * X (Nielsen & Chuang Eq. 4.4)
                new_state[i] = Complex::new(cos * ai.re + sin * aj.im, cos * ai.im - sin * aj.re);
                new_state[j] = Complex::new(cos * aj.re + sin * ai.im, cos * aj.im - sin * ai.re);
            }
        }

        self.state_vector = new_state;
    }

    /// Apply rotation around Y axis
    fn apply_rotation_y(&mut self, qubit: usize, angle: f32) {
        let n = self.num_qubits as usize;
        if qubit >= n {
            return;
        }

        let mask = 1usize << qubit;
        let cos = (angle / 2.0).cos();
        let sin = (angle / 2.0).sin();
        let mut new_state = self.state_vector.clone();

        for i in 0..self.dimension() {
            let j = i ^ mask;
            if i < j {
                let ai = self.state_vector[i];
                let aj = self.state_vector[j];

                new_state[i] = Complex::new(cos * ai.re - sin * aj.re, cos * ai.im - sin * aj.im);
                new_state[j] = Complex::new(sin * ai.re + cos * aj.re, sin * ai.im + cos * aj.im);
            }
        }

        self.state_vector = new_state;
    }

    /// Apply rotation around Z axis
    fn apply_rotation_z(&mut self, qubit: usize, angle: f32) {
        let n = self.num_qubits as usize;
        if qubit >= n {
            return;
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let mask = 1usize << qubit;
        let cos = (angle / 2.0).cos();
        let sin = (angle / 2.0).sin();

        for i in 0..self.dimension() {
            let c = self.state_vector[i];
            if (i & mask) == 0 {
                // |0⟩ gets e^(-iθ/2)
                self.state_vector[i] =
                    Complex::new(cos * c.re + sin * c.im, cos * c.im - sin * c.re);
            } else {
                // |1⟩ gets e^(iθ/2)
                self.state_vector[i] =
                    Complex::new(cos * c.re - sin * c.im, cos * c.im + sin * c.re);
            }
        }
    }

    /// Apply SWAP gate
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    fn apply_swap(&mut self, qubit1: usize, qubit2: usize) {
        let n = self.num_qubits as usize;
        if qubit1 >= n || qubit2 >= n || qubit1 == qubit2 {
            return;
        }

        // SWAP is equivalent to 3 CNOTs
        self.apply_cnot(qubit1, qubit2);
        self.apply_cnot(qubit2, qubit1);
        self.apply_cnot(qubit1, qubit2);
    }

    /// Apply noise to the state (if noise model is set)
    pub fn apply_noise(&mut self) {
        match self.noise_model {
            NoiseModel::None => {}
            NoiseModel::Depolarizing(p) => self.apply_depolarizing(p),
            NoiseModel::Dephasing(p) => self.apply_dephasing(p),
            NoiseModel::AmplitudeDamping(p) => self.apply_amplitude_damping(p),
            NoiseModel::Combined {
                depolarizing,
                dephasing,
                amplitude_damping,
            } => {
                self.apply_depolarizing(depolarizing);
                self.apply_dephasing(dephasing);
                self.apply_amplitude_damping(amplitude_damping);
            }
        }
    }

    /// Apply depolarizing noise
    fn apply_depolarizing(&mut self, p: f32) {
        if p <= 0.0 {
            return;
            // TAG: surface=simulation owner=research-team rule=SIM-001
        }
        let p_clamped = p.clamp(0.0, 1.0);
        let d = self.dimension() as f32;
        let target_prob = p_clamped / d;

        for c in &mut self.state_vector {
            let current_prob = c.norm_sqr();
            let new_prob = (1.0 - p_clamped) * current_prob + target_prob;
            let current_mag = current_prob.sqrt();
            if current_mag > 1e-12 {
                let scale = new_prob.sqrt() / current_mag;
                c.re *= scale;
                c.im *= scale;
            } else {
                c.re = new_prob.sqrt();
                c.im = 0.0;
            }
        }
        self.normalize();
    }

    /// Apply dephasing noise (phase damping):
    /// Relative phase is damped toward the zero-phase axis while strictly conserving each basis state's probability |ψ_k|².
    fn apply_dephasing(&mut self, p: f32) {
        if p <= 0.0 {
            return;
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let p_clamped = p.clamp(0.0, 1.0);
        let factor = 1.0 - p_clamped;
        for c in &mut self.state_vector {
            let magnitude = (c.re * c.re + c.im * c.im).sqrt();
            if magnitude > 1e-12 {
                let current_phase = c.im.atan2(c.re);
                let damped_phase = current_phase * factor;
                c.re = magnitude * damped_phase.cos();
                c.im = magnitude * damped_phase.sin();
            }
        }
    }

    /// Apply amplitude damping
    fn apply_amplitude_damping(&mut self, p: f32) {
        if p <= 0.0 {
            return;
        }
        let p_clamped = p.clamp(0.0, 1.0);
        let mut dissipated_prob = 0.0f32;
        for i in 1..self.dimension() {
            let orig_norm_sq = self.state_vector[i].norm_sqr();
            let factor = (1.0 - p_clamped).sqrt();
            self.state_vector[i] = self.state_vector[i].scale(factor);
            dissipated_prob += orig_norm_sq * p_clamped;
        }
        let ground_re = (self.state_vector[0].norm_sqr() + dissipated_prob).sqrt();
        self.state_vector[0] = Complex::real(ground_re);
        self.normalize();
    }

    /// Measure a qubit, collapsing the superposition
    /// Returns true for |1⟩, false for |0⟩
    pub fn measure(&mut self, qubit: usize) -> bool {
        if qubit >= self.num_qubits as usize {
            return false;
        }
        let prob_one = self.probability_one(qubit);

        // Deterministic measurement based on seed and count
        let threshold =
            ((self.seed.wrapping_add(self.measurement_count as u64)) % 1000) as f32 / 1000.0;
        let result = threshold < prob_one;

        self.collapse(qubit, result);
        self.measurement_count += 1;

        result
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Probabilistic measurement (uses RNG)
    pub fn measure_probabilistic<R: Rng>(&mut self, qubit: usize, rng: &mut R) -> bool {
        if qubit >= self.num_qubits as usize {
            return false;
        }
        let prob_one = self.probability_one(qubit);
        let result: f32 = rng.gen();
        let measured = result < prob_one;

        self.collapse(qubit, measured);
        self.measurement_count += 1;

        measured
    }

    /// Calculate probability of measuring |1⟩ on a qubit
    #[must_use]
    pub fn probability_one(&self, qubit: usize) -> f32 {
        if qubit >= self.num_qubits as usize {
            return 0.0;
        }
        let mask = 1usize << qubit;
        let mut prob = 0.0f32;

        for (i, amp) in self.state_vector.iter().enumerate() {
            if (i & mask) != 0 {
                prob += amp.norm_sqr();
            }
        }

        prob
    }

    /// Calculate probability of measuring |0⟩ on a qubit
    #[must_use]
    pub fn probability_zero(&self, qubit: usize) -> f32 {
        if qubit >= self.num_qubits as usize {
            return 1.0;
        }
        1.0 - self.probability_one(qubit)
    }

    /// Collapse state after measurement
    fn collapse(&mut self, qubit: usize, result: bool) {
        if qubit >= self.num_qubits as usize {
            return;
        }
        let mask = 1usize << qubit;

        // Zero out states that are inconsistent with measurement
        for (i, amp) in self.state_vector.iter_mut().enumerate() {
            let bit_set = (i & mask) != 0;
            if bit_set != result {
                *amp = Complex::zero();
            }
        }

        // Renormalize
        self.normalize();
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Normalize the state vector
    pub fn normalize(&mut self) {
        let norm_sq: f32 = self.state_vector.iter().map(Complex::norm_sqr).sum();

        if norm_sq > 0.0 {
            let norm = norm_sq.sqrt();
            for c in &mut self.state_vector {
                *c = c.scale(1.0 / norm);
            }
        }
    }

    /// Reset to |0...0⟩ state
    pub fn reset(&mut self) {
        for c in &mut self.state_vector {
            *c = Complex::zero();
        }
        self.state_vector[0] = Complex::one();
        self.gates.clear();
        self.measurement_count = 0;
    }

    /// Create Bell state (|00⟩ + |11⟩)/√2 between two qubits
    pub fn create_bell_state(&mut self, qubit1: usize, qubit2: usize) {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        self.reset();
        self.apply_gate(QuantumGate::Hadamard(qubit1));
        self.apply_gate(QuantumGate::CNOT(qubit1, qubit2));
    }

    /// Create GHZ state (|0...0⟩ + |1...1⟩)/√2
    pub fn create_ghz_state(&mut self) {
        self.reset();
        self.apply_gate(QuantumGate::Hadamard(0));
        for i in 1..self.num_qubits as usize {
            self.apply_gate(QuantumGate::CNOT(0, i));
        }
    }

    /// Get the most probable state
    #[must_use]
    pub fn most_probable_state(&self) -> (usize, f32) {
        let mut max_prob = 0.0f32;
        let mut max_idx = 0usize;

        for (i, amp) in self.state_vector.iter().enumerate() {
            let prob = amp.norm_sqr();
            if prob > max_prob {
                max_prob = prob;
                max_idx = i;
            }
        }

        (max_idx, max_prob)
    }

    /// Calculate entanglement entropy (von Neumann entropy for a qubit)
    #[must_use]
    pub fn entanglement_entropy(&self, qubit: usize) -> f32 {
        if qubit >= self.num_qubits as usize {
            return 0.0;
        }
        let p1 = self.probability_one(qubit);
        let p0 = 1.0 - p1;

        // S = -p0*log2(p0) - p1*log2(p1)
        let s0 = if p0 > 0.0 { -p0 * p0.log2() } else { 0.0 };
        let s1 = if p1 > 0.0 { -p1 * p1.log2() } else { 0.0 };

        s0 + s1
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get state as a string representation
    #[must_use]
    pub fn state_string(&self) -> String {
        // P2-PERF: Pre-size vector - at most 2^num_qubits entries, but often much fewer
        let estimated_entries = (1usize << self.num_qubits.min(10)).min(1024);
        let mut parts = Vec::with_capacity(estimated_entries);

        // P2-PERF: Reuse buffer for binary string formatting to avoid allocations
        let width = self.num_qubits as usize;
        let mut bits_buf = vec![b'0'; width];

        for (i, amp) in self.state_vector.iter().enumerate() {
            let prob = amp.norm_sqr();
            if prob > 0.001 {
                // Format index as binary directly into buffer
                let mut idx = i;
                for j in (0..width).rev() {
                    bits_buf[j] = if idx & 1 == 1 { b'1' } else { b'0' };
                    idx >>= 1;
                }
                let bits_str = std::str::from_utf8(&bits_buf).unwrap_or("0");

                // P2-PERF: Use write! to String instead of format! to reduce allocations
                let mut part = String::with_capacity(32);
                let _ = write!(part, "{}|{}⟩", amp.norm(), bits_str);
                parts.push(part);
            }
        }
        if parts.is_empty() {
            "|0...0⟩".to_string()
        } else {
            parts.join(" + ")
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(dead_code, unused_imports)]
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn create_rng() -> StdRng {
        StdRng::seed_from_u64(12345)
    }

    #[test]
    fn test_complex_number_operations() {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let a = Complex::new(3.0, 4.0);
        let b = Complex::new(1.0, 2.0);

        // Test addition
        let sum = a.add(b);
        assert_eq!(sum.re, 4.0);
        assert_eq!(sum.im, 6.0);

        // Test multiplication
        let prod = a.mul(b);
        assert_eq!(prod.re, 3.0 * 1.0 - 4.0 * 2.0); // -5
        assert_eq!(prod.im, 3.0 * 2.0 + 4.0 * 1.0); // 10

        // Test norm
        assert_eq!(a.norm_sqr(), 25.0);
        assert_eq!(a.norm(), 5.0);

        // Test conjugate
        let conj = a.conj();
        assert_eq!(conj.re, 3.0);
        assert_eq!(conj.im, -4.0);
    }

    #[test]
    fn test_quantum_simulator_initialization() {
        let sim = QuantumSimulator::new(3);

        assert_eq!(sim.num_qubits, 3);
        assert_eq!(sim.dimension(), 8); // 2^3
        assert!(sim.is_normalized());
        assert_eq!(sim.state_vector[0], Complex::one());

        // All other states should be zero
        for i in 1..8 {
            assert_eq!(sim.state_vector[i], Complex::zero());
        }
    }

    #[test]
    fn test_hadamard_gate() {
        let mut sim = QuantumSimulator::new(1);

        // Apply Hadamard to |0⟩
        sim.apply_gate(QuantumGate::Hadamard(0));

        // Should be (|0⟩ + |1⟩)/√2
        let h = 1.0 / 2.0_f32.sqrt();
        assert!((sim.state_vector[0].re - h).abs() < 1e-5);
        assert!((sim.state_vector[1].re - h).abs() < 1e-5);
        assert!(sim.is_normalized());
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    #[test]
    fn test_pauli_x_gate() {
        let mut sim = QuantumSimulator::new(1);

        // |0⟩ -> |1⟩
        sim.apply_gate(QuantumGate::PauliX(0));
        assert_eq!(sim.state_vector[0], Complex::zero());
        assert_eq!(sim.state_vector[1], Complex::one());

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // |1⟩ -> |0⟩
        sim.apply_gate(QuantumGate::PauliX(0));
        assert_eq!(sim.state_vector[0], Complex::one());
        assert_eq!(sim.state_vector[1], Complex::zero());
    }

    #[test]
    fn test_pauli_z_gate() {
        let mut sim = QuantumSimulator::new(1);

        // Start with |1⟩
        sim.apply_gate(QuantumGate::PauliX(0));

        // Apply Z gate: |1⟩ -> -|1⟩
        sim.apply_gate(QuantumGate::PauliZ(0));
        assert_eq!(sim.state_vector[1].re, -1.0);
    }

    #[test]
    fn test_cnot_gate() {
        let mut sim = QuantumSimulator::new(2);

        // Create |10⟩ (qubit 0 = |0⟩, qubit 1 = |1⟩)
        // Index: qubit 0 is LSB, so |10⟩ = index 2
        sim.apply_gate(QuantumGate::PauliX(1));

        // Apply CNOT(0, 1): control=0, target=1
        // |10⟩ -> |11⟩ (control is 0, so no change)
        sim.apply_gate(QuantumGate::CNOT(0, 1));
        assert!(sim.state_vector[2].norm_sqr() > 0.99); // Still |10⟩ since control is 0

        // Reset and test with control=1
        let mut sim2 = QuantumSimulator::new(2);
        sim2.apply_gate(QuantumGate::PauliX(0)); // |01⟩ - qubit 0 is 1
        sim2.apply_gate(QuantumGate::CNOT(0, 1)); // |01⟩ -> |11⟩ (flip target)
        assert!(sim2.state_vector[3].norm_sqr() > 0.99); // Should be |11⟩ = index 3
    }

    #[test]
    fn test_bell_state_creation() {
        let mut sim = QuantumSimulator::new(2);

        sim.create_bell_state(0, 1);

        // Bell state: (|00⟩ + |11⟩)/√2
        let h = 1.0 / 2.0_f32.sqrt();
        assert!((sim.state_vector[0].re - h).abs() < 1e-5);
        assert!((sim.state_vector[3].re - h).abs() < 1e-5);
        assert!(sim.is_normalized());
        // TAG: surface=simulation owner=research-team rule=SIM-001

        // Qubits should be entangled
        let p0 = sim.probability_one(0);
        let p1 = sim.probability_one(1);
        assert!((p0 - 0.5).abs() < 1e-5);
        assert!((p1 - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_measurement() {
        let mut sim = QuantumSimulator::new(1);

        // Apply Hadamard to create superposition
        sim.apply_gate(QuantumGate::Hadamard(0));

        // Verify that probability is 0.5
        let prob_one = sim.probability_one(0);
        assert!((prob_one - 0.5).abs() < 0.001);

        // Test measurement collapses state
        let mut test_sim = sim.clone();
        let result = test_sim.measure(0);

        // After measurement, state should be collapsed
        if result {
            assert!(test_sim.probability_one(0) > 0.99);
        } else {
            assert!(test_sim.probability_zero(0) > 0.99);
        }
    }
    #[test]
    fn test_noise_model_properties() {
        let none = NoiseModel::None;
        assert!(none.is_fault_tolerant());
        assert_eq!(none.total_error_probability(), 0.0);

        let depol = NoiseModel::Depolarizing(0.0005);
        assert!(depol.is_fault_tolerant());

        let high_noise = NoiseModel::Depolarizing(0.01);
        assert!(!high_noise.is_fault_tolerant());
    }

    #[test]
    fn test_quantum_simulator_reset() {
        let mut sim = QuantumSimulator::new(2);

        // Apply some gates
        sim.apply_gate(QuantumGate::Hadamard(0));
        sim.apply_gate(QuantumGate::PauliX(1));
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // Reset
        sim.reset();

        // Should be back to |00⟩
        assert_eq!(sim.state_vector[0], Complex::one());
        assert!(sim.gates.is_empty());
        assert!(sim.is_normalized());
    }

    #[test]
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    fn test_quantum_simulator_ghz_state() {
        let mut sim = QuantumSimulator::new(3);

        sim.create_ghz_state();

        // GHZ: (|000⟩ + |111⟩)/√2
        let h = 1.0 / 2.0_f32.sqrt();
        assert!((sim.state_vector[0].re - h).abs() < 1e-5);
        assert!((sim.state_vector[7].re - h).abs() < 1e-5);
        assert!(sim.is_normalized());
    }

    #[test]
    fn test_entanglement_entropy() {
        let mut sim = QuantumSimulator::new(2);

        // Product state has zero entanglement entropy
        let entropy0 = sim.entanglement_entropy(0);
        assert_eq!(entropy0, 0.0);

        // Bell state has maximum entanglement entropy
        sim.create_bell_state(0, 1);
        let entropy_bell = sim.entanglement_entropy(0);
        assert!((entropy_bell - 1.0).abs() < 0.01); // 1 bit of entropy
    }

    #[test]
    fn test_pauli_y_gate() {
        let mut sim = QuantumSimulator::new(1);
        sim.apply_gate(QuantumGate::PauliY(0));
        assert!((sim.state_vector[0].re - 0.0).abs() < 1e-5);
        assert!((sim.state_vector[1].re - 0.0).abs() < 1e-5);
        assert!((sim.state_vector[1].im - 1.0).abs() < 1e-5);
        assert!(sim.is_normalized());
        // Y^2 = I, so applying twice returns to |0⟩
        sim.apply_gate(QuantumGate::PauliY(0));
        assert!((sim.state_vector[0].re - 1.0).abs() < 1e-5);
        assert!((sim.state_vector[0].im - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_phase_gate() {
        let mut sim = QuantumSimulator::new(1);
        sim.apply_gate(QuantumGate::PauliX(0));
        sim.apply_gate(QuantumGate::Phase(0));
        assert!((sim.state_vector[1].re - 0.0).abs() < 1e-5);
        assert!((sim.state_vector[1].im - 1.0).abs() < 1e-5);
        assert!(sim.is_normalized());
    }
    // TAG: surface=simulation owner=research-team rule=SIM-001

    #[test]
    fn test_t_gate() {
        let mut sim = QuantumSimulator::new(1);
        sim.apply_gate(QuantumGate::PauliX(0));
        sim.apply_gate(QuantumGate::T(0));
        let h = 1.0 / 2.0_f32.sqrt();
        assert!((sim.state_vector[1].re - h).abs() < 1e-5);
        assert!((sim.state_vector[1].im - h).abs() < 1e-5);
        assert!(sim.is_normalized());
    }

    #[test]
    fn test_rotation_x_gate() {
        let mut sim = QuantumSimulator::new(1);
        sim.apply_gate(QuantumGate::RotationX(0, std::f32::consts::PI));
        // At theta = pi: |0> -> -i|1>, so real is 0, imaginary is -1.0
        assert!((sim.state_vector[0].norm_sqr()).abs() < 1e-5);
        assert!((sim.state_vector[1].re).abs() < 1e-5);
        assert!((sim.state_vector[1].im - (-1.0)).abs() < 1e-5);
        assert!(sim.is_normalized());

        // Lie-algebra identity: H * R_z(theta) * H = R_x(theta) for any angle
        let theta = 1.2345f32;
        let mut sim_decomp = QuantumSimulator::new(1);
        sim_decomp.apply_gate(QuantumGate::Hadamard(0));
        sim_decomp.apply_gate(QuantumGate::RotationZ(0, theta));
        sim_decomp.apply_gate(QuantumGate::Hadamard(0));

        let mut sim_rx = QuantumSimulator::new(1);
        sim_rx.apply_gate(QuantumGate::RotationX(0, theta));

        for k in 0..2 {
            assert!((sim_decomp.state_vector[k].re - sim_rx.state_vector[k].re).abs() < 1e-5);
            assert!((sim_decomp.state_vector[k].im - sim_rx.state_vector[k].im).abs() < 1e-5);
        }
    }

    #[test]
    fn test_rotation_y_gate() {
        let mut sim = QuantumSimulator::new(1);
        sim.apply_gate(QuantumGate::RotationY(0, std::f32::consts::PI));
        assert!((sim.state_vector[1].norm_sqr() - 1.0).abs() < 1e-5);
        assert!(sim.is_normalized());
    }

    #[test]
    fn test_rotation_z_gate() {
        let mut sim = QuantumSimulator::new(1);
        sim.apply_gate(QuantumGate::PauliX(0));
        sim.apply_gate(QuantumGate::RotationZ(0, 2.0 * std::f32::consts::PI));
        assert!((sim.state_vector[1].norm_sqr() - 1.0).abs() < 1e-5);
        assert!(sim.is_normalized());
    }

    #[test]
    fn test_swap_gate() {
        let mut sim = QuantumSimulator::new(2);
        sim.apply_gate(QuantumGate::PauliX(0));
        sim.apply_gate(QuantumGate::Swap(0, 1));
        assert!((sim.state_vector[2].norm_sqr() - 1.0).abs() < 1e-5);
        assert!(sim.is_normalized());
    }

    #[test]
    fn test_toffoli_gate() {
        let mut sim = QuantumSimulator::new(3);
        sim.apply_gate(QuantumGate::PauliX(0));
        // TAG: surface=simulation owner=research-team rule=SIM-001
        sim.apply_gate(QuantumGate::PauliX(1));
        sim.apply_gate(QuantumGate::Toffoli(0, 1, 2));
        assert!((sim.state_vector[7].norm_sqr() - 1.0).abs() < 1e-5);
        assert!(sim.is_normalized());
    }

    #[test]
    fn test_gate_out_of_bounds() {
        let mut sim = QuantumSimulator::new(1);
        sim.apply_gate(QuantumGate::Hadamard(5));
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        sim.apply_gate(QuantumGate::PauliX(5));
        sim.apply_gate(QuantumGate::CNOT(0, 5));
        sim.apply_gate(QuantumGate::CNOT(5, 0));
        sim.apply_gate(QuantumGate::Swap(0, 5));
        assert!(sim.is_normalized());
        assert_eq!(sim.state_vector[0], Complex::one());
    }

    #[test]
    fn test_measure_probabilistic() {
        let mut sim = QuantumSimulator::new(1);
        sim.apply_gate(QuantumGate::Hadamard(0));
        let mut rng = create_rng();
        let result = sim.measure_probabilistic(0, &mut rng);
        if result {
            assert!(sim.probability_one(0) > 0.99);
        } else {
            assert!(sim.probability_zero(0) > 0.99);
        }
    }

    #[test]
    fn test_most_probable_state() {
        let mut sim = QuantumSimulator::new(2);
        sim.apply_gate(QuantumGate::PauliX(1));
        let (idx, prob) = sim.most_probable_state();
        assert_eq!(idx, 2);
        assert!((prob - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_state_string() {
        let mut sim = QuantumSimulator::new(1);
        let s = sim.state_string();
        assert!(s.contains("|0⟩"));
        sim.apply_gate(QuantumGate::Hadamard(0));
        let s2 = sim.state_string();
        assert!(s2.contains("|0⟩") || s2.contains("|1⟩"));
    }

    #[test]
    fn test_probability_zero() {
        let mut sim = QuantumSimulator::new(1);
        assert_eq!(sim.probability_zero(0), 1.0);
        sim.apply_gate(QuantumGate::Hadamard(0));
        assert!((sim.probability_zero(0) - 0.5).abs() < 1e-5);
    }

    #[test]
    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn test_with_seed_and_noise() {
        let sim = QuantumSimulator::with_seed(2, 42).with_noise(NoiseModel::Depolarizing(0.01));
        assert_eq!(sim.seed, 42);
        assert_eq!(sim.noise_model, NoiseModel::Depolarizing(0.01));
    }

    #[test]
    fn test_dephasing_noise() {
        let mut sim = QuantumSimulator::new(1);
        sim.apply_gate(QuantumGate::Hadamard(0));
        sim.noise_model = NoiseModel::Dephasing(0.1);
        sim.apply_noise();
        assert!(sim.is_normalized());
    }

    #[test]
    fn test_amplitude_damping_noise() {
        let mut sim = QuantumSimulator::new(1);
        sim.apply_gate(QuantumGate::PauliX(0)); // |1⟩
        sim.noise_model = NoiseModel::AmplitudeDamping(0.1);
        sim.apply_noise();
        // Amplitude damping reduces |1⟩ amplitude (energy dissipation)
        assert!(sim.state_vector[1].norm_sqr() < 1.0);
    }

    #[test]
    fn test_combined_noise() {
        let mut sim = QuantumSimulator::new(1);
        sim.apply_gate(QuantumGate::Hadamard(0));
        let before = sim.state_vector.clone();
        sim.noise_model = NoiseModel::Combined {
            depolarizing: 0.01,
            dephasing: 0.01,
            amplitude_damping: 0.01,
        };
        sim.apply_noise();
        // State should change after noise application
        assert_ne!(sim.state_vector, before);
    }

    #[test]
    fn test_noise_model_error_probability() {
        let combined = NoiseModel::Combined {
            depolarizing: 0.1,
            dephasing: 0.2,
            amplitude_damping: 0.3,
        };
        assert_eq!(combined.total_error_probability(), 0.6);
        let depol = NoiseModel::Depolarizing(0.05);
        assert_eq!(depol.total_error_probability(), 0.05);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let none = NoiseModel::None;
        assert_eq!(none.total_error_probability(), 0.0);
    }

    #[test]
    fn test_measurement_result_conversions() {
        assert!(!MeasurementResult::Zero.as_bool());
        assert!(MeasurementResult::One.as_bool());
        assert_eq!(MeasurementResult::from_bool(false), MeasurementResult::Zero);
        assert_eq!(MeasurementResult::from_bool(true), MeasurementResult::One);
    }

    #[test]
    fn test_toffoli_gate_rejects_aliased_qubits() {
        let mut sim = QuantumSimulator::new(3);
        sim.apply_gate(QuantumGate::PauliX(0));
        // Control and target aliased: must be a no-op
        sim.apply_gate(QuantumGate::Toffoli(0, 1, 1));
        assert_eq!(sim.state_vector[1].norm_sqr(), 1.0);
        // Duplicate controls: must be a no-op
        sim.apply_gate(QuantumGate::Toffoli(0, 0, 2));
        assert_eq!(sim.state_vector[1].norm_sqr(), 1.0);
    }

    #[test]
    fn test_dephasing_conserves_basis_probabilities() {
        let mut sim = QuantumSimulator::new(1);
        sim.apply_gate(QuantumGate::Hadamard(0));
        sim.apply_gate(QuantumGate::Phase(0)); // (|0⟩ + i|1⟩)/√2
        let p0_before = sim.probability_zero(0);
        let p1_before = sim.probability_one(0);

        sim.noise_model = NoiseModel::Dephasing(0.5);
        sim.apply_noise();

        assert!((sim.probability_zero(0) - p0_before).abs() < 1e-5);
        assert!((sim.probability_one(0) - p1_before).abs() < 1e-5);
        assert!(sim.is_normalized());
    }
}
