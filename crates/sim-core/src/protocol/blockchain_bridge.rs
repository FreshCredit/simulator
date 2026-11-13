//! Blockchain Bridge for Real Blockchain Integration
//!
//! Per P0-3: Enable actual blockchain connection (not just simulation).
//! Per PQC-001: Supports post-quantum cryptography options.
//! This module provides the interface for connecting to external blockchains
//! while maintaining SIM-001 isolation.

use super::{ClaimType, VerificationHash};
use crate::quantum::QuantumCryptoMode;
use crate::types::{AgentId, Tick};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Bridge configuration
#[derive(Debug, Clone)]
pub struct BridgeConfig {
    /// Enable real blockchain connection
    pub enabled: bool,
    /// Bridge service URL
    pub bridge_url: String,
    /// Chain ID
    pub chain_id: u64,
    /// Quantum cryptography mode for post-quantum security
    /// Per PQC-001: Enables quantum-resistant signatures for blockchain anchoring
    pub quantum_mode: QuantumCryptoMode,
    /// Enable hybrid mode (classical + post-quantum) for transition period
    pub hybrid_mode: bool,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bridge_url: "http://localhost:9944".to_string(),
            chain_id: 42,
            quantum_mode: QuantumCryptoMode::Classical,
            hybrid_mode: false,
        }
    }
}

impl BridgeConfig {
    /// Create a new configuration with post-quantum security
    #[must_use]
    pub fn with_quantum_crypto(mode: QuantumCryptoMode) -> Self {
        Self {
            quantum_mode: mode,
            hybrid_mode: matches!(mode, QuantumCryptoMode::Hybrid),
            ..Default::default()
        }
    }

    /// Check if this configuration uses quantum-resistant signatures
    #[must_use]
    pub const fn is_quantum_resistant(&self) -> bool {
        self.quantum_mode.is_quantum_resistant()
    }

    /// Get estimated signature size for this configuration
    #[must_use]
    pub const fn estimated_signature_size(&self) -> usize {
        match self.quantum_mode {
            QuantumCryptoMode::Classical => 64,     // ECDSA signature
            QuantumCryptoMode::Dilithium => 2420,   // Dilithium2 signature
            QuantumCryptoMode::SphincsPlus => 7856, // SPHINCS+-128s signature
            QuantumCryptoMode::Hybrid => 2484,      // ECDSA + Dilithium2
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get recommended configuration for given security requirements
    ///
    /// # Arguments
    /// * `year` - Current year (for threat model assessment)
    /// * `long_term_protection` - Whether data needs protection beyond quantum era
    #[must_use]
    pub fn recommended(year: u32, long_term_protection: bool) -> Self {
        let quantum_mode = if year >= 2030 || long_term_protection {
            if long_term_protection {
                QuantumCryptoMode::Hybrid
            } else {
                QuantumCryptoMode::Dilithium
            }
        } else {
            QuantumCryptoMode::Classical
        };

        Self::with_quantum_crypto(quantum_mode)
    }
}

/// Bridge errors
#[derive(Debug, Clone)]
pub enum BridgeError {
    /// not connected.
    NotConnected,
    /// connection failed.
    ConnectionFailed(String),
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotConnected => write!(f, "Not connected"),
            Self::ConnectionFailed(msg) => write!(f, "Connection failed: {msg}"),
        }
    }
}

impl std::error::Error for BridgeError {}

/// Blockchain bridge trait
pub trait BlockchainBridge {
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn connect(&mut self) -> Result<(), BridgeError>;
    /// disconnect.
    fn disconnect(&mut self);
    /// is connected.
    fn is_connected(&self) -> bool;
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn anchor_claim(
        &mut self,
        agent_id: AgentId,
        claim_type: ClaimType,
        hash: VerificationHash,
        tick: Tick,
    ) -> Result<(), BridgeError>;
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Mock bridge for testing
#[derive(Debug)]
pub struct MockBlockchainBridge {
    config: BridgeConfig,
    connected: bool,
}

impl MockBlockchainBridge {
    #[must_use]
    /// new.
    pub const fn new(config: BridgeConfig) -> Self {
        Self {
            config,
            connected: false,
        }
    }
}

impl BlockchainBridge for MockBlockchainBridge {
    fn connect(&mut self) -> Result<(), BridgeError> {
        if self.config.enabled {
            self.connected = true;
            Ok(())
        } else {
            Err(BridgeError::NotConnected)
        }
    }

    fn disconnect(&mut self) {
        self.connected = false;
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn anchor_claim(
        &mut self,
        _agent_id: AgentId,
        _claim_type: ClaimType,
        _hash: VerificationHash,
        _tick: Tick,
    ) -> Result<(), BridgeError> {
        if !self.connected {
            return Err(BridgeError::NotConnected);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_bridge() {
        let config = BridgeConfig {
            enabled: true,
            ..Default::default()
        };
        let mut bridge = MockBlockchainBridge::new(config);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(!bridge.is_connected());
        bridge.connect().unwrap();
        assert!(bridge.is_connected());
    }

    #[test]
    fn test_bridge_config_quantum_modes() {
        // Classical configuration
        let classical = BridgeConfig::default();
        assert!(!classical.is_quantum_resistant());
        assert_eq!(classical.estimated_signature_size(), 64);

        // Dilithium configuration
        let dilithium = BridgeConfig::with_quantum_crypto(QuantumCryptoMode::Dilithium);
        assert!(dilithium.is_quantum_resistant());
        assert_eq!(dilithium.quantum_mode, QuantumCryptoMode::Dilithium);
        assert_eq!(dilithium.estimated_signature_size(), 2420);

        // SPHINCS+ configuration
        let sphincs = BridgeConfig::with_quantum_crypto(QuantumCryptoMode::SphincsPlus);
        assert!(sphincs.is_quantum_resistant());
        assert_eq!(sphincs.quantum_mode, QuantumCryptoMode::SphincsPlus);
        assert_eq!(sphincs.estimated_signature_size(), 7856);

        // Hybrid configuration
        let hybrid = BridgeConfig::with_quantum_crypto(QuantumCryptoMode::Hybrid);
        assert!(hybrid.is_quantum_resistant());
        assert!(hybrid.hybrid_mode);
        assert_eq!(hybrid.estimated_signature_size(), 2484);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_bridge_config_recommended() {
        // Year 2024 - classical should be fine
        let config_2024 = BridgeConfig::recommended(2024, false);
        assert_eq!(config_2024.quantum_mode, QuantumCryptoMode::Classical);

        // Year 2030 - approaching quantum threat
        let config_2030 = BridgeConfig::recommended(2030, false);
        assert!(config_2030.is_quantum_resistant());
        assert_eq!(config_2030.quantum_mode, QuantumCryptoMode::Dilithium);

        // Long-term protection needed - use hybrid
        let config_longterm = BridgeConfig::recommended(2024, true);
        assert!(config_longterm.is_quantum_resistant());
        assert_eq!(config_longterm.quantum_mode, QuantumCryptoMode::Hybrid);
    }

    #[test]
    fn test_quantum_mode_bandwidth_impact() {
        let classical = BridgeConfig::with_quantum_crypto(QuantumCryptoMode::Classical);
        let dilithium = BridgeConfig::with_quantum_crypto(QuantumCryptoMode::Dilithium);
        let sphincs = BridgeConfig::with_quantum_crypto(QuantumCryptoMode::SphincsPlus);

        // PQC signatures are significantly larger
        let dilithium_overhead = dilithium.estimated_signature_size() as f32
            / classical.estimated_signature_size() as f32;
        let sphincs_overhead =
            sphincs.estimated_signature_size() as f32 / classical.estimated_signature_size() as f32;

        assert!(dilithium_overhead > 10.0, "Dilithium should be ~38x larger");
        assert!(sphincs_overhead > 100.0, "SPHINCS+ should be ~123x larger");
    }
}
