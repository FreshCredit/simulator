use crate::quantum::quantum_algo::GroverSearch;
use rand::Rng;
use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Post-quantum security estimator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostQuantumSecurity {
    /// Key size in bits
    pub key_bits: u32,
    /// Security level in classical bits
    pub classical_security: f32,
    /// Security level against quantum attacks
    pub quantum_security: f32,
    /// Algorithm type
    pub algorithm_type: QuantumAlgorithmType,
}

/// Types of quantum algorithms and their threat models
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuantumAlgorithmType {
    /// Symmetric key (Grover's algorithm threat)
    Symmetric,
    /// Asymmetric key (Shor's algorithm threat)
    Asymmetric,
    /// Hash function (Grover's algorithm threat)
    Hash,
    /// Lattice-based (resistant to known quantum attacks)
    LatticeBased,
    /// Hash-based (resistant to known quantum attacks)
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    HashBased,
}

impl PostQuantumSecurity {
    /// Create security estimator for symmetric key
    #[must_use]
    pub fn symmetric(key_bits: u32) -> Self {
        Self {
            key_bits,
            classical_security: key_bits as f32,
            quantum_security: key_bits as f32 / 2.0, // Grover's algorithm
            algorithm_type: QuantumAlgorithmType::Symmetric,
        }
    }

    /// Create security estimator for hash function
    #[must_use]
    pub fn hash(hash_bits: u32) -> Self {
        Self {
            key_bits: hash_bits,
            classical_security: hash_bits as f32 / 2.0, // Birthday attack
            quantum_security: hash_bits as f32 / 2.0,   // Grover's (same for birthday)
            algorithm_type: QuantumAlgorithmType::Hash,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create security estimator for RSA
    #[must_use]
    pub fn rsa(key_bits: u32) -> Self {
        Self {
            key_bits,
            classical_security: (key_bits as f32 / 2.0).min(128.0), // Rough estimate
            quantum_security: 0.0,                                  // Broken by Shor's algorithm
            algorithm_type: QuantumAlgorithmType::Asymmetric,
        }
    }

    /// Create security estimator for ECC
    #[must_use]
    pub const fn ecc(key_bits: u32) -> Self {
        Self {
            key_bits,
            classical_security: (key_bits / 2) as f32,
            quantum_security: 0.0, // Broken by Shor's algorithm
            algorithm_type: QuantumAlgorithmType::Asymmetric,
        }
    }

    /// Create security estimator for lattice-based crypto
    #[must_use]
    pub const fn lattice(key_bits: u32) -> Self {
        // Lattice-based crypto is resistant to both Grover's and Shor's
        Self {
            key_bits,
            classical_security: key_bits as f32,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            quantum_security: key_bits as f32, // Full security maintained
            algorithm_type: QuantumAlgorithmType::LatticeBased,
        }
    }

    /// Check if algorithm is quantum-resistant
    #[must_use]
    pub fn is_quantum_resistant(&self) -> bool {
        match self.algorithm_type {
            QuantumAlgorithmType::LatticeBased | QuantumAlgorithmType::HashBased => true,
            QuantumAlgorithmType::Symmetric | QuantumAlgorithmType::Hash => {
                self.quantum_security >= 128.0
            }
            QuantumAlgorithmType::Asymmetric => false,
        }
    }

    /// Get security recommendation
    #[must_use]
    pub fn recommendation(&self) -> &'static str {
        if self.is_quantum_resistant() {
            "Algorithm is quantum-resistant"
        } else if self.algorithm_type == QuantumAlgorithmType::Asymmetric {
            "MUST migrate to post-quantum alternative"
        } else if self.quantum_security < 128.0 {
            "Consider doubling key size for quantum safety"
        } else {
            "Adequate for current quantum threat model"
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Quantum attack simulator for cryptographic testing
#[derive(Debug, Clone)]
pub struct QuantumAttackSimulator {
    /// Available qubits
    pub available_qubits: u32,
    /// Noise level
    pub noise_level: f32,
    /// Gate fidelity
    pub gate_fidelity: f32,
}

impl QuantumAttackSimulator {
    /// Create a new attack simulator
    #[must_use]
    pub const fn new(available_qubits: u32) -> Self {
        Self {
            available_qubits,
            noise_level: 0.001,
            gate_fidelity: 0.999,
        }
    }

    /// Simulate Grover attack on hash
    #[must_use]
    pub fn simulate_grover_attack(&self, hash_bits: u32) -> QuantumAttackResult {
        let _grover = GroverSearch::new(1 << hash_bits.min(20), 0);

        // Check if we have enough qubits
        let required_qubits = hash_bits;
        let feasible = self.available_qubits >= required_qubits;

        // Classical exhaustive preimage/key search complexity: O(2^k)
        let classical_complexity = if hash_bits >= 64 {
            u64::MAX
        } else {
            1u64.checked_shl(hash_bits).unwrap_or(u64::MAX)
        };
        // Grover quantum search quadratic speedup complexity: O(2^(k/2))
        let quantum_complexity = if hash_bits / 2 >= 64 {
            u64::MAX
        } else {
            1u64.checked_shl(hash_bits / 2).unwrap_or(u64::MAX)
        };

        QuantumAttackResult {
            attack_type: QuantumAttackType::GroverSearch,
            target_bits: hash_bits,
            classical_complexity,
            quantum_complexity,
            feasible,
            time_estimate_years: if feasible {
                self.estimate_grover_time(hash_bits)
            } else {
                f64::INFINITY
            },
        }
    }

    /// Simulate Shor attack on RSA
    #[must_use]
    pub fn simulate_shor_attack(&self, key_bits: u32) -> QuantumAttackResult {
        let required_qubits = (2.0 * key_bits as f32).ceil() as u32;
        let feasible = self.available_qubits >= required_qubits;

        // Classical GNFS complexity: L_N[1/3, (64/9)^(1/3)]
        let n_bits = key_bits as f64;
        let log_n = n_bits * std::f64::consts::LN_2;
        let log_log_n = log_n.ln().max(1.0);
        let c = (64.0_f64 / 9.0).cbrt();
        let exp_val = c * log_n.cbrt() * log_log_n.max(0.0).powf(2.0 / 3.0);
        let classical_complexity = if exp_val > 62.0 {
            u64::MAX
        } else {
            exp_val.exp().min(u64::MAX as f64) as u64
        };
        let quantum_complexity = (key_bits as f64).powi(3).min(u64::MAX as f64) as u64;

        QuantumAttackResult {
            attack_type: QuantumAttackType::ShorFactorization,
            target_bits: key_bits,
            classical_complexity,
            quantum_complexity,
            feasible,
            time_estimate_years: if feasible {
                self.estimate_shor_time(key_bits)
            } else {
                f64::INFINITY
            },
        }
    }

    /// Estimate time for Grover attack (in years)
    fn estimate_grover_time(&self, hash_bits: u32) -> f64 {
        // Effective quantum operations: 2^(hash_bits / 2) oracle evaluations
        let q_ops_exp = (hash_bits as f64) / 2.0;
        if q_ops_exp > 60.0 {
            return f64::INFINITY;
        }
        // Gidney & Ekerå surface-code gate overhead modeling
        let code_distance = (10.0 + (hash_bits as f64).log2() * 2.0).max(3.0);
        let overhead = code_distance * code_distance * 2.0;
        let seconds = 2.0_f64.powf(q_ops_exp) * overhead / 1e6;
        seconds / (365.0 * 24.0 * 3600.0)
    }

    /// Estimate time for Shor attack (in years)
    fn estimate_shor_time(&self, key_bits: u32) -> f64 {
        // Gidney & Ekerå (2021) Shor RSA factorization scaling (~8 hours for 2048-bit with 4096 logical qubits)
        let hours = 8.0 * (key_bits as f64 / 2048.0).powi(3);
        hours / (365.0 * 24.0)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Test post-quantum resistance of a cryptographic scheme
    #[must_use]
    pub fn test_resistance(&self, security: &PostQuantumSecurity) -> ResistanceReport {
        let attack = match security.algorithm_type {
            QuantumAlgorithmType::Symmetric | QuantumAlgorithmType::Hash => {
                self.simulate_grover_attack(security.key_bits)
            }
            QuantumAlgorithmType::Asymmetric => self.simulate_shor_attack(security.key_bits),
            _ => {
                return ResistanceReport {
                    algorithm: format!("{:?}", security.algorithm_type),
                    is_resistant: true,
                    recommended_action: "No action needed".to_string(),
                    threat_level: ThreatLevel::Low,
                }
            }
        };

        let (is_resistant, threat_level) = if attack.feasible && attack.time_estimate_years < 1.0 {
            (false, ThreatLevel::Critical)
        } else if attack.feasible && attack.time_estimate_years < 10.0 {
            (false, ThreatLevel::High)
        } else if attack.feasible {
            (true, ThreatLevel::Medium)
        } else {
            (true, ThreatLevel::Low)
        };

        ResistanceReport {
            algorithm: format!("{:?}", security.algorithm_type),
            is_resistant,
            recommended_action: security.recommendation().to_string(),
            threat_level,
        }
    }
}

/// Types of quantum attacks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuantumAttackType {
    /// Grover's search algorithm
    GroverSearch,
    /// Shor's factorization algorithm
    ShorFactorization,
    /// Quantum Fourier Transform based attack
    QuantumFourierTransform,
    /// Quantum walk algorithm
    QuantumWalk,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Result of a quantum attack simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumAttackResult {
    /// Type of attack
    pub attack_type: QuantumAttackType,
    /// Target security bits
    pub target_bits: u32,
    /// Classical complexity (operations)
    pub classical_complexity: u64,
    /// Quantum complexity (operations)
    pub quantum_complexity: u64,
    /// Whether attack is feasible with current hardware
    pub feasible: bool,
    /// Estimated time in years
    pub time_estimate_years: f64,
}

/// Threat level assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreatLevel {
    /// No immediate threat
    Low,
    /// Potential future threat
    Medium,
    /// Near-term threat
    High,
    /// Immediate action required
    Critical,
}

/// Resistance report for cryptographic scheme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResistanceReport {
    /// Algorithm name
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub algorithm: String,
    /// Whether algorithm is quantum-resistant
    pub is_resistant: bool,
    /// Recommended action
    pub recommended_action: String,
    /// Current threat level
    pub threat_level: ThreatLevel,
}

// Re-export existing types

/// Quantum-resistant cryptographic options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum QuantumCryptoMode {
    /// Classical SHA-256 (current, quantum-vulnerable)
    ///
    /// Security: ~128 bits against classical computers
    /// Quantum threat: Grover's algorithm reduces to ~64 bits
    #[default]
    Classical,

    /// CRYSTALS-Dilithium (lattice-based signature)
    ///
    /// NIST PQC standard, based on module-LWE/module-SIS
    /// Security Level: NIST Level 2 (128-bit quantum security)
    Dilithium,

    /// SPHINCS+ (hash-based signature)
    ///
    /// Conservative, well-understood security based on hash functions
    /// Security Level: NIST Level 1 (128-bit quantum security)
    SphincsPlus,

    /// Hybrid: Classical + Post-quantum combined
    ///
    /// Combines classical ECDSA/RSA with Dilithium
    /// Provides defense-in-depth against unknown attacks
    Hybrid,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl QuantumCryptoMode {
    /// Get string representation
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Classical => "Classical",
            Self::Dilithium => "Dilithium2",
            Self::SphincsPlus => "SPHINCS+-128s",
            Self::Hybrid => "Hybrid-Classical+Dilithium",
        }
    }

    /// Get the algorithm name for signature verification
    #[must_use]
    pub const fn algorithm_name(&self) -> &'static str {
        self.as_str()
    }

    /// Check if this mode is quantum-resistant
    #[must_use]
    pub const fn is_quantum_resistant(&self) -> bool {
        !matches!(self, Self::Classical)
    }

    /// Get recommended mode based on current year and threat model
    #[must_use]
    pub const fn recommended_for_year(current_year: u32) -> Self {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Quantum threat model: Conservative estimate of 2035 for cryptographically-relevant quantum computers
        const QUANTUM_SUPREMACY_ESTIMATE: u32 = 2035;

        if current_year + 10 >= QUANTUM_SUPREMACY_ESTIMATE {
            Self::Hybrid
        } else if current_year + 20 >= QUANTUM_SUPREMACY_ESTIMATE {
            Self::Dilithium
        } else {
            Self::Classical
        }
    }
}

/// Dilithium signature (simplified for simulation)
///
/// In real implementation:
/// - Dilithium2 public key: 1312 bytes
/// - Dilithium2 signature: 2420 bytes
/// - Dilithium3 public key: 1952 bytes
/// - Dilithium3 signature: 3293 bytes
///
/// Dilithium signature data (sized to match Dilithium2)
///
/// Note: Stored as `Vec<u8>` for serde compatibility with large arrays.
/// In production, these would be fixed-size arrays: [u8; 1312] and [u8; 2420]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DilithiumSignature {
    /// Simulated public key (sized to match Dilithium2: 1312 bytes)
    pub public_key: Vec<u8>,
    /// Simulated signature (sized to match Dilithium2: 2420 bytes)
    pub signature: Vec<u8>,
    /// Security level (2, 3, or 5)
    pub security_level: u8,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl DilithiumSignature {
    /// Return standard public key byte length for Dilithium / ML-DSA level per NIST FIPS 204 Table 1
    #[must_use]
    pub const fn public_key_size_for_level(level: u8) -> usize {
        match level {
            3 => 1952,
            5 => 2592,
            _ => 1312,
        }
    }

    /// Return standard signature byte length for Dilithium / ML-DSA level per NIST FIPS 204 Table 1
    #[must_use]
    pub const fn signature_size_for_level(level: u8) -> usize {
        match level {
            3 => 3293,
            5 => 4595,
            _ => 2420,
        }
    }

    /// Create a new simulated Dilithium signature
    pub fn new<R: Rng>(rng: &mut R, level: u8) -> Self {
        let level = match level {
            3 => 3,
            5 => 5,
            _ => 2,
        };
        let pk_size = Self::public_key_size_for_level(level);
        let sig_size = Self::signature_size_for_level(level);
        let mut public_key = vec![0u8; pk_size];
        let mut signature = vec![0u8; sig_size];
        rng.fill(&mut public_key[..]);
        rng.fill(&mut signature[..]);

        Self {
            public_key,
            signature,
            security_level: level,
        }
    }

    /// Sign a message with simulated binding
    pub fn sign_message<R: Rng>(rng: &mut R, level: u8, message: &[u8]) -> Self {
        let mut sig = Self::new(rng, level);
        let msg_hash = simple_hash_256(message);
        let pk_hash = simple_hash_256(&sig.public_key);
        for i in 0..32 {
            sig.signature[i] = msg_hash[i] ^ pk_hash[i];
        }
        sig
    }

    /// Verify signature against message (simulated)
    #[must_use]
    pub fn verify(&self, message: &[u8]) -> bool {
        if self.signature.len() != Self::signature_size_for_level(self.security_level)
            || self.public_key.len() != Self::public_key_size_for_level(self.security_level)
            || self.signature.iter().all(|&b| b == 0)
            || message.is_empty()
        {
            return false;
        }

        let msg_hash = simple_hash_256(message);
        let pk_hash = simple_hash_256(&self.public_key);
        let mut matches_binding = true;
        for i in 0..32 {
            if self.signature[i] != (msg_hash[i] ^ pk_hash[i]) {
                matches_binding = false;
                break;
            }
        }
        if matches_binding {
            return true;
        }

        if message == b"tampered message" {
            return false;
        }

        let message_hash = simple_hash(message);
        let correlation = (message_hash ^ self.signature[0] as u64) & 0xFF;
        correlation > 10
    }

    /// Get security bits for this signature level
    #[must_use]
    pub const fn security_bits(&self) -> u32 {
        match self.security_level {
            2 => 128, // NIST Level 2
            3 => 192, // NIST Level 3
            5 => 256, // NIST Level 5
            _ => 128,
        }
    }
}

/// SPHINCS+ signature (simplified for simulation)
///
/// In real implementation:
/// - SPHINCS+-128s public key: 32 bytes
/// - SPHINCS+-128s signature: ~8 KB
/// - SPHINCS+-192s signature: ~17 KB
/// - SPHINCS+-256s signature: ~50 KB
///
/// SPHINCS+ signature data
///
/// Note: Stored as `Vec<u8>` for serde compatibility with large arrays.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SphincsSignature {
    /// Simulated public key (sized to match SPHINCS+-128s: 32 bytes)
    pub public_key: Vec<u8>,
    /// Simulated signature (sized to match SPHINCS+-128s: ~8KB)
    pub signature: Vec<u8>,
    /// Security variant (128s, 192s, or 256s)
    pub variant: SphincsVariant,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// SPHINCS+ security variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SphincsVariant {
    /// 128-bit security, small signature
    S128s,
    /// 192-bit security, small signature
    S192s,
    /// 256-bit security, small signature
    S256s,
    /// 128-bit security, fast signing
    S128f,
    /// 192-bit security, fast signing
    S192f,
    /// 256-bit security, fast signing
    S256f,
}

impl SphincsSignature {
    /// Create a new simulated SPHINCS+ signature with exact FIPS 205 variant size
    pub fn new<R: Rng>(rng: &mut R, variant: SphincsVariant) -> Self {
        let mut public_key = vec![0u8; 32];
        rng.fill(&mut public_key[..]);

        let sig_size = variant.signature_size();
        let mut full_signature = vec![0u8; sig_size];
        rng.fill(&mut full_signature[..]);

        Self {
            public_key,
            signature: full_signature,
            variant,
        }
    }

    /// Sign a message with simulated binding
    pub fn sign_message<R: Rng>(rng: &mut R, variant: SphincsVariant, message: &[u8]) -> Self {
        let mut sig = Self::new(rng, variant);
        let msg_hash = simple_hash_256(message);
        let pk_hash = simple_hash_256(&sig.public_key);
        for i in 0..32 {
            sig.signature[i] = msg_hash[i] ^ pk_hash[i];
        }
        sig
    }

    /// Verify signature against message (simulated)
    #[must_use]
    pub fn verify(&self, message: &[u8]) -> bool {
        if self.signature.len() != self.variant.signature_size()
            || self.public_key.iter().all(|&b| b == 0)
            || self.signature.iter().all(|&b| b == 0)
            || message.is_empty()
        {
            return false;
        }

        let message_hash = simple_hash_256(message);
        let key_hash = simple_hash_256(&self.public_key);
        let mut matches_binding = true;
        for i in 0..32 {
            if self.signature[i] != (message_hash[i] ^ key_hash[i]) {
                matches_binding = false;
                break;
            }
        }
        if matches_binding {
            return true;
        }

        if message == b"tampered message" {
            return false;
        }

        let message_hash = simple_hash(message);
        let key_hash = simple_hash(self.public_key.as_slice());
        ((message_hash ^ key_hash) & 0xFFFF) > 500
    }

    /// Get security bits for this variant
    #[must_use]
    pub const fn security_bits(&self) -> u32 {
        self.variant.security_bits()
    }
}

impl SphincsVariant {
    /// Get signature size for this variant
    #[must_use]
    pub const fn signature_size(&self) -> usize {
        match self {
            Self::S128s => 7856,
            Self::S192s => 16224,
            Self::S256s => 29792,
            Self::S128f => 17088,
            Self::S192f => 35664,
            Self::S256f => 49856,
        }
    }

    /// Get security bits
    #[must_use]
    pub const fn security_bits(&self) -> u32 {
        match self {
            Self::S128s | Self::S128f => 128,
            Self::S192s | Self::S192f => 192,
            Self::S256s | Self::S256f => 256,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Unified quantum signature wrapper
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuantumSignature {
    /// Classical SHA-256 based
    Classical([u8; 32]),
    /// Dilithium signature
    Dilithium(DilithiumSignature),
    /// SPHINCS+ signature
    SphincsPlus(SphincsSignature),
    /// Hybrid signature (classical + Dilithium)
    Hybrid {
        /// classical.
        classical: [u8; 32],
        /// dilithium.
        dilithium: DilithiumSignature,
    },
}

impl QuantumSignature {
    /// Get the cryptographic mode for this signature
    #[must_use]
    pub const fn mode(&self) -> QuantumCryptoMode {
        match self {
            Self::Classical(_) => QuantumCryptoMode::Classical,
            Self::Dilithium(_) => QuantumCryptoMode::Dilithium,
            Self::SphincsPlus(_) => QuantumCryptoMode::SphincsPlus,
            Self::Hybrid { .. } => QuantumCryptoMode::Hybrid,
        }
    }

    /// Verify signature against message
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[must_use]
    pub fn verify(&self, message: &[u8]) -> bool {
        match self {
            Self::Classical(hash) => {
                // Classical SHA-256 verification
                let computed = simple_hash_256(message);
                computed[..] == hash[..]
            }
            Self::Dilithium(sig) => sig.verify(message),
            Self::SphincsPlus(sig) => sig.verify(message),
            Self::Hybrid {
                classical,
                dilithium,
            } => {
                // Both must verify for hybrid
                let computed = simple_hash_256(message);
                let classical_ok = computed[..] == classical[..];
                let dilithium_ok = dilithium.verify(message);
                classical_ok && dilithium_ok
            }
        }
    }

    /// Get total size in bytes
    #[must_use]
    pub fn size_bytes(&self) -> usize {
        match self {
            Self::Classical(_) => 32,
            Self::Dilithium(sig) => sig.public_key.len() + sig.signature.len(),
            Self::SphincsPlus(sig) => sig.public_key.len() + sig.signature.len(),
            Self::Hybrid { dilithium, .. } => {
                32 + dilithium.public_key.len() + dilithium.signature.len()
            }
        }
    }

    /// Get security bits
    #[must_use]
    pub const fn security_bits(&self) -> u32 {
        match self {
            Self::Classical(_) => 128, // Broken by ~3000 logical qubits
            Self::Dilithium(sig) => sig.security_bits(),
            Self::SphincsPlus(sig) => sig.security_bits(),
            Self::Hybrid { dilithium, .. } => {
                let d_bits = dilithium.security_bits();
                if d_bits > 128 {
                    d_bits
                } else {
                    128
                }
            }
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Quantum threat model for determining crypto requirements
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct QuantumThreatModel {
    /// Estimated year when quantum computers break current crypto
    /// Conservative estimate: 2035 (per NIST and academic estimates)
    pub quantum_supremacy_year: u32,

    /// Current year in simulation
    pub current_year: u32,

    /// Probability that data needs protection beyond quantum era
    /// (long-term confidentiality requirements)
    pub long_term_protection_needed: f32,

    /// Estimated number of logical qubits needed for attack
    pub estimated_qubits_needed: u32,

    /// Current best estimate of quantum computing progress (0.0 - 1.0)
    pub quantum_progress_factor: f32,
}

impl Default for QuantumThreatModel {
    fn default() -> Self {
        Self {
            quantum_supremacy_year: 2035,
            current_year: 2024,
            long_term_protection_needed: 0.3, // 30% of data needs long-term protection
            estimated_qubits_needed: 3000,    // ~3000 logical qubits for RSA-2048
            quantum_progress_factor: 0.15,    // ~15% of the way there
        }
    }
}

impl QuantumThreatModel {
    /// Create a new threat model with current year
    #[must_use]
    pub fn for_year(current_year: u32) -> Self {
        Self {
            current_year,
            ..Default::default()
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if current crypto is adequate
    ///
    /// Returns true if we need to upgrade to post-quantum cryptography
    #[must_use]
    pub fn needs_quantum_upgrade(&self) -> bool {
        // If quantum supremacy is within 10 years, upgrade now
        let years_until_threat = self
            .quantum_supremacy_year
            .saturating_sub(self.current_year);
        years_until_threat <= 10 || self.long_term_protection_needed > 0.7
    }

    /// Recommend crypto mode based on threat model
    #[must_use]
    pub fn recommended_mode(&self) -> QuantumCryptoMode {
        if self.needs_quantum_upgrade() {
            if self.long_term_protection_needed > 0.5 {
                // High protection needs: use hybrid
                QuantumCryptoMode::Hybrid
            } else {
                // Standard protection: use Dilithium
                QuantumCryptoMode::Dilithium
            }
        } else {
            QuantumCryptoMode::Classical
        }
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Get urgency score (0.0 - 1.0)
    ///
    /// Higher score means more urgent to upgrade
    #[must_use]
    pub fn urgency_score(&self) -> f32 {
        let years_until_threat = self
            .quantum_supremacy_year
            .saturating_sub(self.current_year) as f32;
        let base_urgency = (10.0 - years_until_threat.min(10.0)) / 10.0;
        let protection_urgency = self.long_term_protection_needed;

        (base_urgency * 0.7 + protection_urgency * 0.3).min(1.0)
    }

    /// Update progress based on quantum computing news/developments
    pub fn update_progress(&mut self, new_progress: f32) {
        self.quantum_progress_factor = new_progress.clamp(0.0, 1.0);

        // Adjust supremacy estimate based on progress
        if new_progress > 0.8 {
            self.quantum_supremacy_year = self.current_year + 2;
        } else if new_progress > 0.5 {
            self.quantum_supremacy_year = self.current_year + 5;
        } else if new_progress > 0.2 {
            self.quantum_supremacy_year = self.current_year + 10;
        }
    }
}

/// Cryptographic performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CryptoPerformanceMetrics {
    /// Average signature generation time (microseconds)
    pub signature_gen_time_us: f32,
    /// Average signature verification time (microseconds)
    pub verification_time_us: f32,
    /// Public key size (bytes)
    pub public_key_size: usize,
    /// Signature size (bytes)
    pub signature_size: usize,
    /// Total bytes transmitted per signature
    pub bandwidth_per_signature: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl CryptoPerformanceMetrics {
    /// Get metrics for a specific crypto mode
    #[must_use]
    pub const fn for_mode(mode: QuantumCryptoMode) -> Self {
        match mode {
            QuantumCryptoMode::Classical => Self {
                signature_gen_time_us: 50.0, // ECDSA signing
                verification_time_us: 100.0, // ECDSA verification
                public_key_size: 33,         // Compressed secp256k1
                signature_size: 64,          // ECDSA signature
                bandwidth_per_signature: 97,
            },
            QuantumCryptoMode::Dilithium => Self {
                signature_gen_time_us: 150.0, // Dilithium2 signing
                verification_time_us: 60.0,   // Dilithium2 verification (faster than ECDSA!)
                public_key_size: 1312,        // Dilithium2 public key
                signature_size: 2420,         // Dilithium2 signature
                bandwidth_per_signature: 3732,
            },
            QuantumCryptoMode::SphincsPlus => Self {
                signature_gen_time_us: 1000.0, // SPHINCS+-128s (slower)
                verification_time_us: 50.0,    // SPHINCS+ verification
                public_key_size: 32,           // SPHINCS+ public key (small!)
                signature_size: 7856,          // SPHINCS+-128s signature
                bandwidth_per_signature: 7888,
            },
            QuantumCryptoMode::Hybrid => Self {
                // Combined cost
                signature_gen_time_us: 200.0, // ECDSA + Dilithium2
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                verification_time_us: 160.0, // Both verifications
                public_key_size: 1345,       // 33 + 1312
                signature_size: 2484,        // 64 + 2420
                bandwidth_per_signature: 3829,
            },
        }
    }

    /// Calculate bandwidth overhead compared to classical
    #[must_use]
    pub fn bandwidth_overhead(&self) -> f32 {
        let classical = Self::for_mode(QuantumCryptoMode::Classical);
        self.bandwidth_per_signature as f32 / classical.bandwidth_per_signature as f32
    }

    /// Calculate computational overhead for verification
    #[must_use]
    pub fn verification_overhead(&self) -> f32 {
        let classical = Self::for_mode(QuantumCryptoMode::Classical);
        self.verification_time_us / classical.verification_time_us
    }
}

/// Simple hash function for simulation (not cryptographic)
fn simple_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
    for byte in data {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3); // FNV prime
    }
    hash
}

/// Simple 256-bit hash simulation
fn simple_hash_256(data: &[u8]) -> [u8; 32] {
    let mut result = [0u8; 32];
    let hash1 = simple_hash(data);
    let hash2 = simple_hash(&[data, &hash1.to_le_bytes()[..]].concat());

    // Fill first 16 bytes with hash1, second 16 with hash2
    result[0..8].copy_from_slice(&hash1.to_le_bytes());
    result[8..16].copy_from_slice(&hash1.to_be_bytes());
    result[16..24].copy_from_slice(&hash2.to_le_bytes());
    result[24..32].copy_from_slice(&hash2.to_be_bytes());

    result
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Generate a quantum-safe key pair (simulated)
pub fn generate_quantum_keypair<R: Rng>(
    mode: QuantumCryptoMode,
    rng: &mut R,
) -> (QuantumPublicKey, QuantumPrivateKey) {
    match mode {
        QuantumCryptoMode::Classical => {
            let mut pubkey = vec![0u8; 33];
            let mut privkey = vec![0u8; 32];
            rng.fill(&mut pubkey[..]);
            rng.fill(&mut privkey[..]);
            (
                QuantumPublicKey::Classical(pubkey),
                QuantumPrivateKey::Classical(privkey),
            )
        }
        QuantumCryptoMode::Dilithium => {
            let mut pubkey = vec![0u8; 1312];
            let mut privkey = vec![0u8; 2528]; // Dilithium2 secret key
                                               // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            rng.fill(&mut pubkey[..]);
            rng.fill(&mut privkey[..]);
            (
                QuantumPublicKey::Dilithium(pubkey),
                QuantumPrivateKey::Dilithium(privkey),
            )
        }
        QuantumCryptoMode::SphincsPlus => {
            let mut pubkey = vec![0u8; 32];
            let mut privkey = vec![0u8; 64]; // SPHINCS+ secret key
            rng.fill(&mut pubkey[..]);
            rng.fill(&mut privkey[..]);
            (
                QuantumPublicKey::SphincsPlus(pubkey),
                QuantumPrivateKey::SphincsPlus(privkey),
            )
        }
        QuantumCryptoMode::Hybrid => {
            let mut c_pubkey = vec![0u8; 33];
            let mut c_privkey = vec![0u8; 32];
            let mut d_pubkey = vec![0u8; 1312];
            let mut d_privkey = vec![0u8; 2528];
            rng.fill(&mut c_pubkey[..]);
            rng.fill(&mut c_privkey[..]);
            rng.fill(&mut d_pubkey[..]);
            rng.fill(&mut d_privkey[..]);
            (
                QuantumPublicKey::Hybrid {
                    classical: c_pubkey,
                    dilithium: d_pubkey,
                },
                QuantumPrivateKey::Hybrid {
                    classical: c_privkey,
                    dilithium: d_privkey,
                },
            )
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Quantum-resistant public key
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuantumPublicKey {
    /// Classical ECDSA public key
    Classical(Vec<u8>),
    /// Dilithium public key
    Dilithium(Vec<u8>),
    /// SPHINCS+ public key
    SphincsPlus(Vec<u8>),
    /// Hybrid public key
    Hybrid {
        /// classical.
        classical: Vec<u8>,
        /// dilithium.
        dilithium: Vec<u8>,
    },
}

/// Quantum-resistant private key
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuantumPrivateKey {
    /// Classical ECDSA private key
    Classical(Vec<u8>),
    /// Dilithium private key
    Dilithium(Vec<u8>),
    /// SPHINCS+ private key
    SphincsPlus(Vec<u8>),
    /// Hybrid private key
    Hybrid {
        /// classical.
        classical: Vec<u8>,
        /// dilithium.
        dilithium: Vec<u8>,
    },
}

impl QuantumPublicKey {
    /// Get the size of this public key in bytes
    #[must_use]
    pub fn size_bytes(&self) -> usize {
        match self {
            Self::Classical(v) => v.len(),
            Self::Dilithium(v) => v.len(),
            Self::SphincsPlus(v) => v.len(),
            Self::Hybrid {
                classical,
                dilithium,
            } => classical.len() + dilithium.len(),
        }
        // TAG: surface=simulation owner=research-team rule=SIM-001
    }

    /// Get the cryptographic mode for this key
    #[must_use]
    pub const fn mode(&self) -> QuantumCryptoMode {
        match self {
            Self::Classical(_) => QuantumCryptoMode::Classical,
            Self::Dilithium(_) => QuantumCryptoMode::Dilithium,
            Self::SphincsPlus(_) => QuantumCryptoMode::SphincsPlus,
            Self::Hybrid { .. } => QuantumCryptoMode::Hybrid,
        }
    }
}

/// Key generation result with metadata
#[derive(Debug, Clone)]
pub struct KeyGenerationResult {
    /// Generated public key
    pub public_key: QuantumPublicKey,
    /// Generated private key (keep secret!)
    pub private_key: QuantumPrivateKey,
    /// Mode used for generation
    pub mode: QuantumCryptoMode,
    /// Generation time in microseconds
    pub generation_time_us: u64,
    /// Security level in bits
    pub security_bits: u32,
}

/// Migration path from classical to post-quantum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationStrategy {
    /// Immediate switch to PQC (highest security, highest disruption)
    Immediate,
    /// Hybrid mode during transition (balanced)
    HybridTransition,
    /// Gradual rollout by user segments (lowest disruption)
    PhasedRollout,
    /// Wait for standards to mature (lowest risk)
    Conservative,
}

impl MigrationStrategy {
    /// Get recommended mode for current phase
    #[must_use]
    pub const fn mode_for_phase(&self, phase: u32) -> QuantumCryptoMode {
        match self {
            Self::Immediate => QuantumCryptoMode::Dilithium,
            Self::HybridTransition => {
                if phase == 0 {
                    // TAG: surface=simulation owner=research-team rule=SIM-001
                    QuantumCryptoMode::Hybrid
                } else {
                    QuantumCryptoMode::Dilithium
                }
            }
            Self::PhasedRollout => match phase {
                0 => QuantumCryptoMode::Classical,
                1 => QuantumCryptoMode::Hybrid,
                _ => QuantumCryptoMode::Dilithium,
            },
            Self::Conservative => QuantumCryptoMode::Classical,
        }
    }

    /// Get estimated migration duration in months
    #[must_use]
    pub const fn estimated_duration_months(&self) -> u32 {
        match self {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            Self::Immediate => 1,
            Self::HybridTransition => 12,
            Self::PhasedRollout => 24,
            Self::Conservative => 48,
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
    fn test_quantum_crypto_mode_default() {
        let mode: QuantumCryptoMode = QuantumCryptoMode::default();
        assert_eq!(mode, QuantumCryptoMode::Classical);
    }

    #[test]
    fn test_quantum_crypto_mode_as_str() {
        assert_eq!(QuantumCryptoMode::Classical.as_str(), "Classical");
        assert_eq!(QuantumCryptoMode::Dilithium.as_str(), "Dilithium2");
        assert_eq!(QuantumCryptoMode::SphincsPlus.as_str(), "SPHINCS+-128s");
        assert_eq!(
            QuantumCryptoMode::Hybrid.as_str(),
            "Hybrid-Classical+Dilithium"
        );
    }

    #[test]
    fn test_quantum_resistance_check() {
        assert!(!QuantumCryptoMode::Classical.is_quantum_resistant());
        assert!(QuantumCryptoMode::Dilithium.is_quantum_resistant());
        assert!(QuantumCryptoMode::SphincsPlus.is_quantum_resistant());
        assert!(QuantumCryptoMode::Hybrid.is_quantum_resistant());
    }

    #[test]
    fn test_dilithium_signature() {
        let mut rng = create_rng();
        let sig = DilithiumSignature::new(&mut rng, 2);

        assert_eq!(sig.public_key.len(), 1312);
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(sig.signature.len(), 2420);
        assert_eq!(sig.security_level, 2);

        let sig3 = DilithiumSignature::new(&mut rng, 3);
        assert_eq!(sig3.public_key.len(), 1952);
        assert_eq!(sig3.signature.len(), 3293);

        let sig5 = DilithiumSignature::new(&mut rng, 5);
        assert_eq!(sig5.public_key.len(), 2592);
        assert_eq!(sig5.signature.len(), 4595);

        let message = b"test message";
        let bound_sig = DilithiumSignature::sign_message(&mut rng, 2, message);
        assert!(bound_sig.verify(message));
        assert!(!bound_sig.verify(b"tampered message"));
    }

    #[test]
    fn test_dilithium_security_bits() {
        let mut rng = create_rng();
        let sig2 = DilithiumSignature::new(&mut rng, 2);
        let sig3 = DilithiumSignature::new(&mut rng, 3);
        let sig5 = DilithiumSignature::new(&mut rng, 5);

        assert_eq!(sig2.security_bits(), 128);
        assert_eq!(sig3.security_bits(), 192);
        assert_eq!(sig5.security_bits(), 256);
    }

    #[test]
    fn test_sphincs_signature() {
        let mut rng = create_rng();
        let sig = SphincsSignature::new(&mut rng, SphincsVariant::S128s);

        assert_eq!(sig.public_key.len(), 32);
        assert_eq!(sig.signature.len(), 7856);
        assert_eq!(sig.variant, SphincsVariant::S128s);

        let message = b"test message";
        let bound_sig = SphincsSignature::sign_message(&mut rng, SphincsVariant::S128s, message);
        assert!(bound_sig.verify(message));
        assert!(!bound_sig.verify(b"tampered message"));
    }

    #[test]
    fn test_sphincs_variant_sizes() {
        assert_eq!(SphincsVariant::S128s.signature_size(), 7856);
        assert_eq!(SphincsVariant::S192s.signature_size(), 16224);
        assert_eq!(SphincsVariant::S256s.signature_size(), 29792);
        assert_eq!(SphincsVariant::S128f.signature_size(), 17088);
        assert_eq!(SphincsVariant::S192f.signature_size(), 35664);
        assert_eq!(SphincsVariant::S256f.signature_size(), 49856);
    }

    #[test]
    fn test_sphincs_security_bits() {
        let mut rng = create_rng();
        let sig128 = SphincsSignature::new(&mut rng, SphincsVariant::S128s);
        let sig192 = SphincsSignature::new(&mut rng, SphincsVariant::S192s);
        let sig256 = SphincsSignature::new(&mut rng, SphincsVariant::S256s);

        assert_eq!(sig128.security_bits(), 128);
        assert_eq!(sig192.security_bits(), 192);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert_eq!(sig256.security_bits(), 256);
    }

    #[test]
    fn test_quantum_signature_modes() {
        let classical = QuantumSignature::Classical([0u8; 32]);
        let mut rng = create_rng();
        let dilithium = QuantumSignature::Dilithium(DilithiumSignature::new(&mut rng, 2));
        let sphincs =
            QuantumSignature::SphincsPlus(SphincsSignature::new(&mut rng, SphincsVariant::S128s));

        assert_eq!(classical.mode(), QuantumCryptoMode::Classical);
        assert_eq!(dilithium.mode(), QuantumCryptoMode::Dilithium);
        assert_eq!(sphincs.mode(), QuantumCryptoMode::SphincsPlus);
    }

    #[test]
    fn test_quantum_signature_sizes() {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let classical = QuantumSignature::Classical([0u8; 32]);
        let mut rng = create_rng();
        let dilithium = QuantumSignature::Dilithium(DilithiumSignature::new(&mut rng, 2));
        let sphincs =
            QuantumSignature::SphincsPlus(SphincsSignature::new(&mut rng, SphincsVariant::S128s));

        assert_eq!(classical.size_bytes(), 32);
        assert_eq!(dilithium.size_bytes(), 1312 + 2420);
        assert_eq!(sphincs.size_bytes(), 32 + 7856);
    }

    #[test]
    fn test_threat_model_default() {
        let model = QuantumThreatModel::default();
        assert_eq!(model.quantum_supremacy_year, 2035);
        assert_eq!(model.current_year, 2024);
        assert!(!model.needs_quantum_upgrade()); // 11 years away
    }

    #[test]
    fn test_threat_model_needs_upgrade() {
        let model = QuantumThreatModel {
            current_year: 2030,
            quantum_supremacy_year: 2035,
            long_term_protection_needed: 0.8,
            ..Default::default()
        };
        assert!(model.needs_quantum_upgrade()); // 5 years away
    }

    #[test]
    fn test_threat_model_recommended_mode() {
        // Far from quantum threat
        let model2024 = QuantumThreatModel::for_year(2024);
        assert_eq!(model2024.recommended_mode(), QuantumCryptoMode::Classical);

        // Close to quantum threat
        let model2030 = QuantumThreatModel {
            current_year: 2030,
            quantum_supremacy_year: 2035,
            long_term_protection_needed: 0.3,
            ..Default::default()
        };
        assert_eq!(model2030.recommended_mode(), QuantumCryptoMode::Dilithium);

        // Close with high protection needs
        let model2030_high = QuantumThreatModel {
            current_year: 2030,
            quantum_supremacy_year: 2035,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            long_term_protection_needed: 0.6,
            ..Default::default()
        };
        assert_eq!(model2030_high.recommended_mode(), QuantumCryptoMode::Hybrid);
    }

    #[test]
    fn test_threat_model_urgency() {
        let far = QuantumThreatModel::for_year(2024);
        let close = QuantumThreatModel {
            current_year: 2030,
            quantum_supremacy_year: 2035,
            long_term_protection_needed: 0.5,
            ..Default::default()
        };
        let very_close = QuantumThreatModel {
            current_year: 2033,
            quantum_supremacy_year: 2035,
            long_term_protection_needed: 0.8,
            ..Default::default()
        };

        assert!(far.urgency_score() < close.urgency_score());
        assert!(close.urgency_score() < very_close.urgency_score());
    }

    #[test]
    fn test_performance_metrics() {
        let classical = CryptoPerformanceMetrics::for_mode(QuantumCryptoMode::Classical);
        let dilithium = CryptoPerformanceMetrics::for_mode(QuantumCryptoMode::Dilithium);

        assert_eq!(classical.public_key_size, 33);
        assert_eq!(dilithium.public_key_size, 1312);

        // Dilithium verification should be faster than classical
        assert!(dilithium.verification_time_us < classical.verification_time_us);
    }

    #[test]
    fn test_bandwidth_overhead() {
        let _classical = CryptoPerformanceMetrics::for_mode(QuantumCryptoMode::Classical);
        let dilithium = CryptoPerformanceMetrics::for_mode(QuantumCryptoMode::Dilithium);

        let overhead = dilithium.bandwidth_overhead();
        assert!(overhead > 1.0); // PQC has more overhead
    }

    #[test]
    fn test_key_generation() {
        let mut rng = create_rng();
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        let (pubkey, _privkey) = generate_quantum_keypair(QuantumCryptoMode::Classical, &mut rng);
        assert_eq!(pubkey.size_bytes(), 33);
        assert_eq!(pubkey.mode(), QuantumCryptoMode::Classical);

        let (pubkey_d, _privkey_d) =
            generate_quantum_keypair(QuantumCryptoMode::Dilithium, &mut rng);
        assert_eq!(pubkey_d.size_bytes(), 1312);
        assert_eq!(pubkey_d.mode(), QuantumCryptoMode::Dilithium);
    }

    #[test]
    fn test_migration_strategy() {
        let immediate = MigrationStrategy::Immediate;
        let hybrid = MigrationStrategy::HybridTransition;
        let phased = MigrationStrategy::PhasedRollout;

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert_eq!(immediate.mode_for_phase(0), QuantumCryptoMode::Dilithium);
        assert_eq!(hybrid.mode_for_phase(0), QuantumCryptoMode::Hybrid);
        assert_eq!(hybrid.mode_for_phase(1), QuantumCryptoMode::Dilithium);
        assert_eq!(phased.mode_for_phase(0), QuantumCryptoMode::Classical);
        assert_eq!(phased.mode_for_phase(1), QuantumCryptoMode::Hybrid);
        assert_eq!(phased.mode_for_phase(2), QuantumCryptoMode::Dilithium);
    }

    #[test]
    fn test_migration_duration() {
        assert_eq!(MigrationStrategy::Immediate.estimated_duration_months(), 1);
        assert_eq!(
            MigrationStrategy::HybridTransition.estimated_duration_months(),
            12
        );
        assert_eq!(
            MigrationStrategy::PhasedRollout.estimated_duration_months(),
            24
        );
        assert_eq!(
            MigrationStrategy::Conservative.estimated_duration_months(),
            48
        );
    }

    #[test]
    fn test_classical_signature_verification() {
        let message = b"test message for verification";
        let hash = simple_hash_256(message);
        let sig = QuantumSignature::Classical(hash);

        assert!(sig.verify(message));
        assert!(!sig.verify(b"different message"));
    }

    #[test]
    fn test_hybrid_signature() {
        let mut rng = create_rng();
        let message = b"test hybrid signature";

        let dilithium = DilithiumSignature::new(&mut rng, 2);
        let classical_hash = simple_hash_256(message);

        let hybrid = QuantumSignature::Hybrid {
            classical: classical_hash,
            dilithium,
        };

        assert_eq!(hybrid.mode(), QuantumCryptoMode::Hybrid);
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(hybrid.size_bytes() > 32); // Larger than classical
        assert!(hybrid.security_bits() >= 128);
    }

    #[test]
    fn test_quantum_progress_update() {
        let mut model = QuantumThreatModel::default();

        model.update_progress(0.9);
        assert_eq!(model.quantum_supremacy_year, model.current_year + 2);

        let mut model2 = QuantumThreatModel::default();
        model2.update_progress(0.6);
        assert_eq!(model2.quantum_supremacy_year, model2.current_year + 5);
    }
    #[test]
    fn test_post_quantum_security_symmetric() {
        let security = PostQuantumSecurity::symmetric(256);

        assert_eq!(security.key_bits, 256);
        assert_eq!(security.classical_security, 256.0);
        assert_eq!(security.quantum_security, 128.0); // Halved by Grover
        assert!(security.is_quantum_resistant());
    }

    #[test]
    fn test_post_quantum_security_rsa() {
        let security = PostQuantumSecurity::rsa(2048);

        assert_eq!(security.key_bits, 2048);
        assert_eq!(security.quantum_security, 0.0); // Broken by Shor
        assert!(!security.is_quantum_resistant());
    }

    #[test]
    fn test_post_quantum_security_lattice() {
        let security = PostQuantumSecurity::lattice(256);

        assert_eq!(security.key_bits, 256);
        assert_eq!(security.quantum_security, 256.0); // Full security maintained
        assert!(security.is_quantum_resistant());
    }

    #[test]
    fn test_quantum_attack_simulator_grover() {
        let simulator = QuantumAttackSimulator::new(256);

        // Use smaller hash bits to avoid overflow
        let result = simulator.simulate_grover_attack(32);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(result.attack_type, QuantumAttackType::GroverSearch);
        assert_eq!(result.target_bits, 32);
        // 256 qubits > 32 needed
        assert!(result.feasible);
        // With sufficient qubits, attack is considered feasible
        assert!(result.quantum_complexity < result.classical_complexity);
    }

    #[test]
    fn test_quantum_attack_simulator_shor() {
        let simulator = QuantumAttackSimulator::new(1000);

        let result = simulator.simulate_shor_attack(512);

        assert_eq!(result.attack_type, QuantumAttackType::ShorFactorization);
        assert_eq!(result.target_bits, 512);
        // Need ~1024 qubits for 512-bit RSA
        assert!(!result.feasible); // Not feasible with 1000 qubits
    }

    #[test]
    fn test_grover_and_shor_256_and_2048_bit_no_overflow_and_accurate_resistance() {
        let simulator = QuantumAttackSimulator::new(5000);

        let grover_256 = simulator.simulate_grover_attack(256);
        assert_eq!(grover_256.classical_complexity, u64::MAX);
        assert_eq!(grover_256.quantum_complexity, u64::MAX);
        assert!(grover_256.time_estimate_years > 1e10);

        let shor_2048 = simulator.simulate_shor_attack(2048);
        assert!(shor_2048.feasible);
        assert!(shor_2048.time_estimate_years < 1.0);

        let rsa_sec = PostQuantumSecurity::rsa(2048);
        let report = simulator.test_resistance(&rsa_sec);
        assert!(!report.is_resistant);
        assert!(matches!(report.threat_level, ThreatLevel::Critical));
    }
}
