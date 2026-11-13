//! Blockchain anchoring integration for simulation artifacts
//!
//! Provides functionality to anchor simulation results to the `FreshCredit` testnet
//! via the agent-bridge service.

use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Configuration for blockchain anchoring
#[derive(Debug, Clone)]
pub struct BlockchainConfig {
    /// URL of the agent-bridge service
    pub bridge_url: String,
    /// Enable/disable anchoring
    pub enabled: bool,
}

impl Default for BlockchainConfig {
    fn default() -> Self {
        Self {
            bridge_url: std::env::var("AGENT_BRIDGE_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            enabled: true,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl BlockchainConfig {
    /// Load configuration from environment variables
    #[must_use]
    pub fn from_env() -> Self {
        let bridge_url = std::env::var("AGENT_BRIDGE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        let enabled = std::env::var("BLOCKCHAIN_ANCHORING_ENABLED")
            .map(|v| v.parse().unwrap_or(true))
            .unwrap_or(true);

        Self {
            bridge_url,
            enabled,
        }
    }
}

/// Request to anchor a hash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorRequest {
    /// hash
    pub hash: String,
    /// agent id
    pub agent_id: String,
    /// metadata
    pub metadata: Option<serde_json::Value>,
}

/// Response from anchoring operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorResponse {
    /// status
    pub status: String,
    /// block hash
    pub block_hash: Option<String>,
    /// extrinsic hash
    pub extrinsic_hash: Option<String>,
    /// block number
    pub block_number: Option<u32>,
    /// timestamp
    pub timestamp: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Receipt stored after successful anchoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorReceipt {
    /// block hash
    pub block_hash: Option<String>,
    /// extrinsic hash
    pub extrinsic_hash: Option<String>,
    /// block number
    pub block_number: Option<u32>,
    /// timestamp
    pub timestamp: String,
    /// metrics hash
    pub metrics_hash: String,
}

/// Error types for blockchain operations
#[derive(Debug, thiserror::Error)]
pub enum BlockchainError {
    #[error("HTTP error: {0}")]
    /// Http
    Http(#[from] reqwest::Error),

    #[error("Bridge error: {0}")]
    /// Bridge
    Bridge(String),

    #[error("IO error: {0}")]
    /// Io
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    /// Serialization
    Serialization(#[from] serde_json::Error),
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Blockchain anchor client
#[derive(Debug)]
pub struct BlockchainAnchor {
    config: BlockchainConfig,
    http_client: reqwest::Client,
}

impl BlockchainAnchor {
    /// Create a new blockchain anchor client
    #[must_use]
    pub fn new(config: BlockchainConfig) -> Self {
        Self {
            config,
            http_client: crate::http_client::shared_client().clone(),
        }
    }

    /// Create from environment configuration
    #[must_use]
    pub fn from_env() -> Self {
        Self::new(BlockchainConfig::from_env())
    }

    /// Check if anchoring is enabled
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate SHA-256 hash of file contents
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn hash_file(&self, path: &Path) -> Result<[u8; 32], BlockchainError> {
        // P0-PERF: Use tokio::fs for async file I/O
        let contents = tokio::fs::read_to_string(path).await?;
        let hash = sha2::Sha256::digest(contents.as_bytes());
        Ok(hash.into())
    }

    /// Generate SHA-256 hash of string contents
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[must_use]
    pub fn hash_string(&self, contents: &str) -> [u8; 32] {
        let hash = sha2::Sha256::digest(contents.as_bytes());
        hash.into()
    }

    /// Anchor simulation metrics to blockchain
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn anchor_simulation(
        &self,
        run_id: &str,
        stage: &str,
        agent_count: u32,
        metrics_hash: &[u8; 32],
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    ) -> Result<AnchorReceipt, BlockchainError> {
        if !self.config.enabled {
            return Err(BlockchainError::Bridge("Anchoring disabled".to_string()));
        }

        let request = AnchorRequest {
            hash: hex::encode(metrics_hash),
            agent_id: format!("sim-{stage}"),
            metadata: Some(serde_json::json!({
                "run_id": run_id,
                "stage": stage,
                "agent_count": agent_count,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "report_type": "simulation_metrics",
            })),
        };

        debug!("Anchoring simulation {} to blockchain", run_id);

        let response = self
            .http_client
            .post(format!("{}/anchor", self.config.bridge_url))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(BlockchainError::Bridge(error_text));
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let anchor_response: AnchorResponse = response.json().await?;

        info!(
            "Anchored simulation {} to testnet block {}",
            run_id,
            anchor_response.block_number.unwrap_or(0)
        );

        Ok(AnchorReceipt {
            block_hash: anchor_response.block_hash,
            extrinsic_hash: anchor_response.extrinsic_hash,
            block_number: anchor_response.block_number,
            timestamp: anchor_response.timestamp,
            metrics_hash: hex::encode(metrics_hash),
        })
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Store anchor receipt to disk
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn store_receipt(
        &self,
        artifacts_dir: &Path,
        run_id: &str,
        receipt: &AnchorReceipt,
    ) -> Result<(), BlockchainError> {
        let receipt_path = artifacts_dir.join(run_id).join("blockchain_receipt.json");

        // Ensure directory exists
        if let Some(parent) = receipt_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let receipt_json = serde_json::to_string_pretty(receipt)?;
        tokio::fs::write(&receipt_path, receipt_json).await?;

        debug!("Stored blockchain receipt to {}", receipt_path.display());

        Ok(())
    }

    /// Check if a receipt already exists for a run
    #[must_use]
    pub fn receipt_exists(&self, artifacts_dir: &Path, run_id: &str) -> bool {
        let receipt_path = artifacts_dir.join(run_id).join("blockchain_receipt.json");
        receipt_path.exists()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Verify a hash on the blockchain
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn verify_hash(&self, hash: &str) -> Result<serde_json::Value, BlockchainError> {
        let response = self
            .http_client
            .get(format!("{}/verify/{}", self.config.bridge_url, hash))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(BlockchainError::Bridge(error_text));
        }

        let result = response.json().await?;
        Ok(result)
    }

    /// Health check the agent bridge
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn health_check(&self) -> Result<serde_json::Value, BlockchainError> {
        let response = self
            .http_client
            .get(format!("{}/health", self.config.bridge_url))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(BlockchainError::Bridge(error_text));
        }

        let result = response.json().await?;
        Ok(result)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Periodic anchor task for live simulation
///
/// Anchors simulation state at regular intervals
pub async fn periodic_anchor_task(
    _anchor: Arc<BlockchainAnchor>,
    _artifacts_dir: std::path::PathBuf,
    interval_ticks: u64,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) {
    let mut tick_counter: u64 = 0;
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60)); // Anchor every 60 seconds

    // P10-R4: intentionally unbounded service loop — select! terminates on the shutdown watch; each arm is a bounded anchor attempt.
    loop {
        tokio::select! {
                    _ = interval.tick() => {
                        tick_counter += 1;

                        if tick_counter % interval_ticks == 0 {
                            // Generate a timestamp-based run_id for periodic anchors
                            let _run_id = format!("live-{}", chrono::Utc::now().timestamp());

                            // In a real implementation, we would:
                            // 1. Serialize current simulation state
                            // 2. Compute hash
                            // 3. Anchor to blockchain
                            // 4. Store receipt

                            debug!("Periodic anchor tick {} - would anchor here", tick_counter);
                        }
                    }
                    _ = shutdown.changed() => {
                        if *shutdown.borrow() {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                            info!("Shutting down periodic anchor task");
                            break;
                        }
                    }
                }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_config() -> BlockchainConfig {
        BlockchainConfig {
            bridge_url: "http://localhost:8082".to_string(),
            enabled: true,
        }
    }

    #[test]
    fn test_hash_string() {
        let anchor = BlockchainAnchor::new(test_config());
        let hash1 = anchor.hash_string("test data");
        let hash2 = anchor.hash_string("test data");
        let hash3 = anchor.hash_string("different data");

        // Same input should produce same hash
        assert_eq!(hash1, hash2);
        // Different input should produce different hash
        assert_ne!(hash1, hash3);
        // Hash should be 32 bytes
        assert_eq!(hash1.len(), 32);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_receipt_exists() {
        let dir = tempdir().unwrap();
        let anchor = BlockchainAnchor::new(test_config());

        // Should not exist initially
        assert!(!anchor.receipt_exists(dir.path(), "test-run"));

        // Create receipt file
        let run_dir = dir.path().join("test-run");
        std::fs::create_dir(&run_dir).unwrap();
        let receipt = AnchorReceipt {
            block_hash: Some("0x123".to_string()),
            extrinsic_hash: Some("0xabc".to_string()),
            block_number: Some(42),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metrics_hash: "0xdef".to_string(),
        };
        let receipt_json = serde_json::to_string_pretty(&receipt).unwrap();
        std::fs::write(run_dir.join("blockchain_receipt.json"), receipt_json).unwrap();

        // Should exist now
        assert!(anchor.receipt_exists(dir.path(), "test-run"));
    }
}
