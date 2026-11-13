// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Quantum-Resistant Cryptography Module
//!
//! Implements post-quantum cryptographic options for post-quantum security.
//! Current SHA-256 is vulnerable to quantum attacks (Grover's algorithm).
//! This module provides lattice-based and hash-based signatures for quantum resistance.
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

/// post quantum module.
pub mod post_quantum;
/// quantum algo module.
pub mod quantum_algo;
pub mod quantum_sim;

pub use post_quantum::*;
pub use quantum_algo::*;
pub use quantum_sim::*;
