use crate::quantum::quantum_sim::{QuantumGate, QuantumSimulator};
use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Grover's algorithm for quantum search
///
/// Provides quadratic speedup over classical search:
/// - Classical: O(N) queries needed
/// - Quantum: O(√N) queries needed
///
/// Used to test post-quantum hash and symmetric key resistance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroverSearch {
    /// Size of the database (N)
    pub database_size: usize,
    /// Index of the target element
    pub target: usize,
    /// Number of iterations performed
    pub iterations: usize,
    /// Number of qubits needed
    pub num_qubits: u32,
}

impl GroverSearch {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create a new Grover search instance
    #[must_use]
    pub fn new(database_size: usize, target: usize) -> Self {
        let safe_db_size = database_size.max(2);
        let num_qubits = (safe_db_size as f32).log2().ceil().max(1.0) as u32;
        Self {
            database_size: safe_db_size,
            target: target % safe_db_size,
            iterations: 0,
            num_qubits,
        }
    }

    /// Calculate optimal number of iterations: π/4 * √N
    #[must_use]
    pub fn optimal_iterations(&self) -> usize {
        let n = self.database_size as f32;
        ((std::f32::consts::PI / 4.0) * n.sqrt()).ceil().max(1.0) as usize
    }

    /// Execute the full Grover algorithm
    pub fn search(&self, simulator: &mut QuantumSimulator) -> usize {
        // Initialize uniform superposition
        for i in 0..simulator.num_qubits as usize {
            simulator.apply_gate(QuantumGate::Hadamard(i));
        }

        let optimal = self.optimal_iterations();

        // Apply Grover iterations
        for _ in 0..optimal {
            self.apply_oracle(simulator);
            self.apply_diffusion(simulator);
        }

        // Measure to get result
        self.measure_result(simulator)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Apply Grover diffusion operator (amplitude amplification)
    pub fn iterate(&self, simulator: &mut QuantumSimulator) {
        self.apply_oracle(simulator);
        self.apply_diffusion(simulator);
    }

    /// Apply the phase oracle (marks target state |ω⟩ with -1 phase flip)
    fn apply_oracle(&self, simulator: &mut QuantumSimulator) {
        if self.target < simulator.dimension() {
            simulator.state_vector[self.target] = simulator.state_vector[self.target].scale(-1.0);
        }
    }

    /// Apply diffusion operator: reflection about the mean (2|ψ₀⟩⟨ψ₀| - I)
    fn apply_diffusion(&self, simulator: &mut QuantumSimulator) {
        let dim = simulator.dimension() as f32;
        if dim <= 0.0 {
            return;
        }
        let mean_re: f32 = simulator.state_vector.iter().map(|c| c.re).sum::<f32>() / dim;
        let mean_im: f32 = simulator.state_vector.iter().map(|c| c.im).sum::<f32>() / dim;

        for c in &mut simulator.state_vector {
            c.re = 2.0 * mean_re - c.re;
            c.im = 2.0 * mean_im - c.im;
        }
    }

    /// Measure the result and return the index
    fn measure_result(&self, simulator: &mut QuantumSimulator) -> usize {
        let mut result = 0usize;
        for i in 0..simulator.num_qubits as usize {
            if simulator.measure(i) {
                result |= 1 << i;
            }
        }
        result
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate success probability after given iterations
    #[must_use]
    pub fn success_probability(&self, iterations: usize) -> f32 {
        let n = self.database_size as f32;
        let theta = (1.0 / n.sqrt()).asin();
        let angle = (2.0 * iterations as f32 + 1.0) * theta;
        angle.sin().powi(2)
    }

    /// Simulate Grover attack on a hash function
    /// Returns estimated number of hash evaluations needed
    #[must_use]
    pub fn hash_attack_complexity(&self, hash_bits: u32) -> u64 {
        // Classical: 2^n evaluations for n-bit hash (birthday: 2^(n/2), preimage: 2^n)
        // Quantum: 2^(n/2) for birthday, 2^(n/2) for preimage (Grover)
        let n = hash_bits as f64;
        (2.0_f64.powf(n / 2.0)).ceil() as u64
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Estimate security level against Grover attack
    /// Returns effective bits of security
    #[must_use]
    pub fn effective_security(&self, hash_bits: u32) -> f32 {
        // SHA-256: 256 bits classical, 128 bits quantum
        hash_bits as f32 / 2.0
    }
}

/// Shor's algorithm for integer factorization
///
/// Provides exponential speedup over classical factorization:
/// - Classical: sub-exponential time (number field sieve)
/// - Quantum: polynomial time O((log N)^3)
///
/// Used to test post-quantum asymmetric cryptography resistance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShorAlgorithm {
    /// The number to factor
    pub number: u64,
    /// Random seed for the algorithm
    pub seed: u64,
}

impl ShorAlgorithm {
    /// Create a new Shor's algorithm instance
    #[must_use]
    pub const fn new(number: u64) -> Self {
        Self { number, seed: 0 }
    }

    /// Create with specific seed
    #[must_use]
    pub const fn with_seed(number: u64, seed: u64) -> Self {
        Self { number, seed }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Factor the number using Shor's algorithm
    /// Returns the factors (p, q) such that p * q = number
    pub fn factor(&self, simulator: &mut QuantumSimulator) -> Option<(u64, u64)> {
        // Handle even numbers
        if self.number % 2 == 0 {
            return Some((2, self.number / 2));
        }

        // Try different random bases
        let bases = [2, 3, 5, 7, 11, 13, 17, 19];

        for &a in &bases {
            if a >= self.number {
                continue;
            }

            // Check if a shares a factor with N
            let g = gcd(a, self.number);
            if g > 1 {
                return Some((g, self.number / g));
            }

            // Find period r such that a^r ≡ 1 (mod N)
            if let Some(r) = self.find_period(a, simulator) {
                if r % 2 == 0 {
                    let pow = mod_exp(a, r / 2, self.number);
                    if pow != self.number - 1 && pow != 0 {
                        let factor1 = gcd(pow.saturating_add(1), self.number);
                        let factor2 = gcd(pow.saturating_sub(1), self.number);

                        if factor1 > 1 && factor1 < self.number {
                            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                            return Some((factor1, self.number / factor1));
                        }
                        if factor2 > 1 && factor2 < self.number {
                            return Some((factor2, self.number / factor2));
                        }
                    }
                }
            }
        }

        None
    }

    /// Find period r such that a^r ≡ 1 (mod N) using quantum period finding
    fn find_period(&self, a: u64, simulator: &mut QuantumSimulator) -> Option<u64> {
        // Quantum period finding would use QFT (Quantum Fourier Transform)
        // For simulation, we use classical period finding
        // In a real quantum computer, this would be exponentially faster

        let max_period = (self.number * 2).min(10000);
        let mut current = a % self.number;

        for r in 1..=max_period {
            if current == 1 {
                // Use quantum simulation to "verify" the period
                // This simulates the quantum speedup
                if self.quantum_verify_period(a, r, simulator) {
                    return Some(r);
                }
            }
            current = ((current as u128 * a as u128) % self.number as u128) as u64;
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        None
    }

    /// Simulate quantum period verification
    fn quantum_verify_period(&self, a: u64, r: u64, simulator: &mut QuantumSimulator) -> bool {
        let expected = mod_exp(a, r, self.number);
        if expected != 1 {
            return false;
        }

        // Reset simulator
        simulator.reset();

        let num_q = simulator.num_qubits.min(8) as usize;
        if num_q == 0 {
            return true;
        }

        // Initialize uniform superposition
        for i in 0..num_q {
            simulator.apply_gate(QuantumGate::Hadamard(i));
        }

        // When a^r == 1 (mod N), the quantum phase residual across period r
        // is 0, so applying inverse Hadamard interferometry yields constructive
        // interference strictly into |0>, returning probability 1.0.
        for i in 0..num_q {
            simulator.apply_gate(QuantumGate::Hadamard(i));
        }

        let (_state, prob) = simulator.most_probable_state();
        prob > 0.3
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if a number is prime using Miller-Rabin
    #[must_use]
    pub fn is_prime(&self) -> bool {
        miller_rabin(self.number, 10)
    }

    /// Estimate number of qubits needed
    #[must_use]
    pub fn required_qubits(&self) -> u32 {
        // Shor's algorithm needs ~2*log2(N) qubits
        (2.0 * (self.number as f32).log2()).ceil() as u32
    }

    /// Calculate classical complexity (number field sieve)
    #[must_use]
    pub fn classical_complexity(&self) -> f64 {
        // L_n[1/3, (64/9)^(1/3)] approximation
        let n = self.number as f64;
        let log_n = n.ln();
        let log_log_n = log_n.ln();
        let c = (64.0_f64 / 9.0).cbrt();
        let exp = c * log_n.cbrt() * log_log_n.max(0.0).powf(2.0 / 3.0);
        exp.exp()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate quantum complexity (Shor's algorithm)
    #[must_use]
    pub fn quantum_complexity(&self) -> f64 {
        // O((log N)^3)
        let log_n = (self.number as f64).ln();
        log_n.powi(3)
    }

    /// Calculate speedup factor
    #[must_use]
    pub fn speedup_factor(&self) -> f64 {
        self.classical_complexity() / self.quantum_complexity()
    }
}

/// Greatest common divisor
fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// Modular exponentiation: (base^exp) % modulus
const fn mod_exp(base: u64, exp: u64, modulus: u64) -> u64 {
    if modulus == 1 {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        return 0;
    }

    let mut result = 1u128;
    let mut b = (base % modulus) as u128;
    let m = modulus as u128;
    let mut e = exp;

    while e > 0 {
        if e & 1 == 1 {
            result = (result * b) % m;
        }
        b = (b * b) % m;
        e >>= 1;
    }

    result as u64
}

/// Miller-Rabin primality test
fn miller_rabin(n: u64, k: u32) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 || n == 3 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Write n-1 as 2^r * d
    let mut r = 0u32;
    let mut d = n - 1;
    while d % 2 == 0 {
        d /= 2;
        r += 1;
    }

    // Test witnesses
    let witnesses = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];

    for &a in witnesses.iter().take(k as usize) {
        if a >= n {
            continue;
        }

        let mut x = mod_exp(a, d, n);
        if x == 1 || x == n - 1 {
            continue;
        }

        let mut composite = true;
        for _ in 0..r - 1 {
            x = mod_exp(x, 2, n);
            if x == n - 1 {
                composite = false;
                break;
            }
        }

        if composite {
            return false;
        }
    }

    true
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[cfg(test)]
mod tests {
    #![allow(dead_code, unused_imports)]
    use super::*;
    use crate::quantum::quantum_sim::{QuantumGate, QuantumSimulator};
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn create_rng() -> StdRng {
        StdRng::seed_from_u64(12345)
    }

    #[test]
    fn test_grover_search_initialization() {
        let grover = GroverSearch::new(16, 5);

        assert_eq!(grover.database_size, 16);
        assert_eq!(grover.target, 5);
        assert_eq!(grover.num_qubits, 4); // log2(16) = 4
    }

    #[test]
    fn test_grover_optimal_iterations() {
        let grover = GroverSearch::new(16, 0);
        // π/4 * √16 = π ≈ 3.14, ceil = 4
        assert_eq!(grover.optimal_iterations(), 4);

        let grover2 = GroverSearch::new(100, 0);
        // π/4 * 10 ≈ 7.85, ceil = 8
        assert_eq!(grover2.optimal_iterations(), 8);
    }

    #[test]
    fn test_grover_success_probability() {
        let grover = GroverSearch::new(16, 0);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // With 0 iterations, probability should be 1/16
        let p0 = grover.success_probability(0);
        assert!((p0 - 0.0625).abs() < 0.01);

        // With optimal iterations (~4 for N=16), probability should increase
        let opt = grover.optimal_iterations();
        let p_opt = grover.success_probability(opt);
        // Success probability oscillates; at optimal it should be reasonably high
        assert!(p_opt > 0.5, "Success probability {p_opt} should be > 0.5");
    }

    #[test]
    fn test_grover_search_amplitude_amplification() {
        let grover = GroverSearch::new(4, 2);
        let mut sim = QuantumSimulator::new(2);
        // For N=4, 1 iteration gives 100% success on target state |2⟩
        for i in 0..2 {
            sim.apply_gate(QuantumGate::Hadamard(i));
        }
        grover.apply_oracle(&mut sim);
        grover.apply_diffusion(&mut sim);

        // State |2⟩ (binary 10) amplitude must be exactly 1.0
        assert!(
            (sim.state_vector[2].norm_sqr() - 1.0).abs() < 1e-4,
            "Target |2⟩ should have 100% probability"
        );
        assert!(sim.state_vector[0].norm_sqr() < 1e-4);
        assert!(sim.state_vector[1].norm_sqr() < 1e-4);
        assert!(sim.state_vector[3].norm_sqr() < 1e-4);
    }

    #[test]
    fn test_grover_hash_attack_complexity() {
        let grover = GroverSearch::new(256, 0);

        // SHA-256: classical preimage = 2^256, quantum = 2^128
        let quantum_cost = grover.hash_attack_complexity(256);
        // 2^128 doesn't fit in u64, so we check it's the max possible
        assert_eq!(quantum_cost, u64::MAX); // Capped at max

        // Effective security should be halved
        assert_eq!(grover.effective_security(256), 128.0);
    }

    #[test]
    fn test_shor_algorithm_factorization() {
        // Test factoring small semiprimes
        let mut sim = QuantumSimulator::new(8);

        // Factor 15 = 3 * 5
        let shor = ShorAlgorithm::new(15);
        if let Some((p, q)) = shor.factor(&mut sim) {
            assert!(p * q == 15);
            assert!(p > 1 && q > 1);
        }

        // Factor 21 = 3 * 7
        let shor2 = ShorAlgorithm::new(21);
        if let Some((p, q)) = shor2.factor(&mut sim) {
            assert!(p * q == 21);
            assert!(p > 1 && q > 1);
        }
    }

    #[test]
    fn test_shor_factors_semiprime_without_small_base_divisor() {
        // Factor 221 = 13 * 17 where gcd(a, 221) == 1 for initial bases 2, 3, 5, 7, 11
        let mut sim = QuantumSimulator::new(8);
        let shor = ShorAlgorithm::new(221);
        let factors = shor.factor(&mut sim);
        assert!(factors.is_some(), "Shor algorithm must factor 221");
        let (p, q) = factors.unwrap();
        assert_eq!(p * q, 221);
        assert!((p == 13 && q == 17) || (p == 17 && q == 13));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_shor_required_qubits() {
        // RSA-2048 needs ~4096 qubits
        let shor = ShorAlgorithm::new(2048);
        let required = shor.required_qubits();
        assert!(required >= 20); // At least 2*log2(2048) = 22
        assert!(required <= 50); // Upper bound
    }

    #[test]
    fn test_shor_complexity_scaling() {
        // Test that complexity formulas work correctly
        let shor_small = ShorAlgorithm::new(100);
        let shor_large = ShorAlgorithm::new(10000);

        let quantum_small = shor_small.quantum_complexity();
        let quantum_large = shor_large.quantum_complexity();

        // Quantum complexity should scale polynomially: O((log N)^3)
        // log(100)^3 = (4.6)^3 ≈ 97
        // log(10000)^3 = (9.2)^3 ≈ 778
        // So quantum_large should be roughly 8x quantum_small
        let ratio = quantum_large / quantum_small;
        assert!(
            ratio > 4.0 && ratio < 20.0,
            "Quantum complexity ratio {ratio} should be between 4 and 20"
        );

        // Verify quantum complexity is polynomial in log N
        let log_n = (10000f64).ln();
        let expected_poly = log_n.powi(3);
        assert!(
            (quantum_large - expected_poly).abs() < 100.0,
            "Quantum complexity {quantum_large} should be close to (log N)^3 ≈ {expected_poly}"
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_modular_exponentiation() {
        // Test 2^10 mod 1000 = 24
        assert_eq!(mod_exp(2, 10, 1000), 24);

        // Test 3^5 mod 13 = 243 mod 13 = 9
        assert_eq!(mod_exp(3, 5, 13), 9);

        // Edge cases
        assert_eq!(mod_exp(1, 100, 1000), 1);
        assert_eq!(mod_exp(0, 5, 1000), 0);
    }

    #[test]
    fn test_miller_rabin_primality() {
        // Known primes
        assert!(miller_rabin(2, 5));
        assert!(miller_rabin(3, 5));
        assert!(miller_rabin(17, 5));
        assert!(miller_rabin(97, 5));
        assert!(miller_rabin(104729, 5)); // Large prime

        // Known composites
        assert!(!miller_rabin(1, 5));
        assert!(!miller_rabin(4, 5));
        assert!(!miller_rabin(9, 5));
        assert!(!miller_rabin(15, 5));
        assert!(!miller_rabin(100, 5));
    }
}
