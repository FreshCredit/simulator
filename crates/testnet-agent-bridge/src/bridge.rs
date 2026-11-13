// TAG: surface=blockchain owner=blockchain-team rule=BC-001
#[cfg(test)]
mod tests {
    use super::*;
    use freshcredit_runtime::runtime_types::pallet_freshcredit::ReportType as RuntimeReportType;

    fn test_bridge() -> AgentBridge {
        AgentBridge::new("ws://localhost:9944".to_string(), "//Alice".to_string())
    }

    #[test]
    fn test_generate_report_id_deterministic() {
        let bridge = test_bridge();
        let id1 = bridge.generate_report_id("agent-1", "abc123");
        let id2 = bridge.generate_report_id("agent-1", "abc123");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_generate_report_id_unique_per_agent() {
        let bridge = test_bridge();
        let id1 = bridge.generate_report_id("agent-1", "abc123");
        let id2 = bridge.generate_report_id("agent-2", "abc123");
        assert_ne!(id1, id2);
    }

    // TAG: surface=blockchain owner=blockchain-team rule=BC-001
    #[test]
    fn test_generate_report_id_unique_per_hash() {
        let bridge = test_bridge();
        let id1 = bridge.generate_report_id("agent-1", "abc123");
        let id2 = bridge.generate_report_id("agent-1", "def456");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_report_id_32_bytes() {
        let bridge = test_bridge();
        let id = bridge.generate_report_id("agent-1", "abc123");
        assert_eq!(id.len(), 32);
    }

    #[test]
    fn test_parse_report_type_identity_verification() {
        let bridge = test_bridge();
        let result = bridge.parse_report_type("identity_verification");
        assert_eq!(
            std::mem::discriminant(&result),
            std::mem::discriminant(&RuntimeReportType::IdentityVerification)
        );
    }

    #[test]
    // TAG: surface=blockchain owner=blockchain-team rule=BC-001
    fn test_parse_report_type_transaction_history() {
        let bridge = test_bridge();
        let result = bridge.parse_report_type("transaction_history");
        assert_eq!(
            std::mem::discriminant(&result),
            std::mem::discriminant(&RuntimeReportType::TransactionHistory)
        );
    }

    #[test]
    fn test_parse_report_type_balance_snapshot() {
        let bridge = test_bridge();
        let result = bridge.parse_report_type("balance_snapshot");
        assert_eq!(
            std::mem::discriminant(&result),
            std::mem::discriminant(&RuntimeReportType::BalanceSnapshot)
        );
    }

    #[test]
    fn test_parse_report_type_consumer_preferences() {
        let bridge = test_bridge();
        let result = bridge.parse_report_type("consumer_preferences");
        assert_eq!(
            std::mem::discriminant(&result),
            std::mem::discriminant(&RuntimeReportType::ConsumerPreferences) // TAG: surface=blockchain owner=blockchain-team rule=BC-001
                                                                            // TAG: surface=api
                                                                            // TAG: surface=api
        );
    }

    #[test]
    fn test_parse_report_type_provider_requirements() {
        let bridge = test_bridge();
        let result = bridge.parse_report_type("provider_requirements");
        assert_eq!(
            std::mem::discriminant(&result),
            std::mem::discriminant(&RuntimeReportType::ProviderRequirements)
        );
    }

    #[test]
    fn test_parse_report_type_case_insensitive() {
        let bridge = test_bridge();
        let result_lower = bridge.parse_report_type("identity_verification");
        let result_mixed = bridge.parse_report_type("Identity_Verification");
        let result_upper = bridge.parse_report_type("IDENTITYVERIFICATION");
        assert_eq!(
            std::mem::discriminant(&result_lower),
            std::mem::discriminant(&result_mixed)
        );
        assert_eq!(
            // TAG: surface=blockchain
            std::mem::discriminant(&result_lower),
            std::mem::discriminant(&result_upper) // TAG: surface=blockchain owner=blockchain-team rule=BC-001
        );
    }

    #[test]
    fn test_parse_report_type_unknown_default() {
        let bridge = test_bridge();
        let result_unknown = bridge.parse_report_type("unknown");
        let result_default = bridge.parse_report_type("simulation_metrics");
        assert_eq!(
            std::mem::discriminant(&result_unknown),
            std::mem::discriminant(&RuntimeReportType::PlaidFinancial)
        );
        assert_eq!(
            std::mem::discriminant(&result_default),
            std::mem::discriminant(&RuntimeReportType::PlaidFinancial)
        );
    }

    #[test]
    fn test_load_signer_valid_alice() {
        let bridge = AgentBridge::new("ws://localhost:9944".to_string(), "//Alice".to_string());
        let signer = bridge.load_signer();
        assert!(signer.is_ok());
    }

    #[test]
    // TAG: surface=blockchain owner=blockchain-team rule=BC-001
    fn test_load_signer_invalid_suri() {
        let bridge = AgentBridge::new(
            "ws://localhost:9944".to_string(),
            "not-a-valid-suri!!!".to_string(),
        );
        let signer = bridge.load_signer();
        assert!(signer.is_err());
    }
}
use std::sync::Arc;

use subxt::{OnlineClient, SubstrateConfig};
use subxt_rpcs::LegacyRpcMethods;
use subxt_rpcs::RpcClient;
use subxt_signer::sr25519::Keypair;
use tracing::info;

use crate::metrics::MetricsCollector;
use crate::types::{AnchorRequest, AnchorResponse, HealthStatus};

// Generate runtime interface from metadata
// This matches the runtime used by blockchain-client
#[subxt::subxt(runtime_metadata_path = "freshcredit_metadata_v1.scale")]

// TAG: surface=blockchain owner=blockchain-team rule=BC-001
pub mod freshcredit_runtime {}

/// Bridge between simulation agents and the `FreshCredit` testnet
///
/// Manages the connection to the Substrate-based testnet and provides
/// methods for anchoring simulation data and querying blockchain state.
pub struct AgentBridge {
    /// WebSocket URL of the testnet node
    testnet_ws_url: String,
    /// Service account seed for transaction signing
    service_seed: String,
    /// Collector for simulation metrics
    pub(crate) metrics_collector: MetricsCollector,
}

impl AgentBridge {
    pub fn new(testnet_ws_url: String, service_seed: String) -> Self {
        Self {
            testnet_ws_url,
            service_seed,
            metrics_collector: MetricsCollector::new(10000),
        }
    }

    /// Get or create the subxt client
    async fn get_client(&self) -> Result<Arc<OnlineClient<SubstrateConfig>>, String> {
        // TAG: surface=blockchain owner=blockchain-team rule=BC-001
        // Create new client (no caching for simplicity - subxt handles connection pooling)
        info!("Creating subxt client for {}", self.testnet_ws_url);

        let rpc_client = if self.testnet_ws_url.starts_with("wss://") {
            RpcClient::from_url(&self.testnet_ws_url)
                .await
                .map_err(|e| format!("Failed to connect to secure RPC: {e}"))?
        } else {
            RpcClient::from_insecure_url(&self.testnet_ws_url)
                .await
                .map_err(|e| format!("Failed to connect to RPC: {e}"))?
        };

        let client = OnlineClient::<SubstrateConfig>::from_rpc_client(rpc_client)
            .await
            .map_err(|e| format!("Failed to create online client: {e}"))?;

        Ok(Arc::new(client))
    }

    /// Load the service account signer
    fn load_signer(&self) -> Result<Keypair, String> {
        // Parse the seed/SURI
        let suri = self
            .service_seed
            .parse()
            .map_err(|e| format!("Invalid SURI format: {e:?}"))?;

        Keypair::from_uri(&suri).map_err(|e| format!("Failed to create keypair: {e:?}"))
    }

    pub async fn anchor_hash(&self, request: AnchorRequest) -> Result<AnchorResponse, String> {
        // TAG: surface=blockchain owner=blockchain-team rule=BC-001
        info!("Anchoring hash for agent {}", request.agent_id);

        // Validate hash (32 bytes)
        let hash_bytes: [u8; 32] = match hex::decode(&request.hash) {
            Ok(bytes) if bytes.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                arr
            }
            _ => return Err("Hash must be 32 bytes hex".to_string()),
        };

        // Get subxt client
        let client = self.get_client().await?;

        // Load signer
        let signer = self.load_signer()?;

        // Generate report_id from hash and agent_id
        let report_id = self.generate_report_id(&request.agent_id, &request.hash);

        // Determine report type from metadata or default to PlaidFinancial
        let report_type = request
            .metadata
            .as_ref()
            .and_then(|m| m.get("report_type").and_then(|v| v.as_str()))
            // TAG: surface=blockchain owner=blockchain-team rule=BC-001
            .unwrap_or("simulation_metrics");

        let runtime_report_type = self.parse_report_type(report_type);

        // Build the extrinsic - anchor_report_hash(report_id, hash, report_type)
        let tx = freshcredit_runtime::tx().fresh_credit().anchor_report_hash(
            report_id,
            hash_bytes,
            runtime_report_type,
        );

        // Submit and wait for finalization
        info!("Submitting extrinsic to testnet...");
        let tx_progress = client
            .tx()
            .await
            .map_err(|e| format!("Failed to get tx client: {e}"))?
            .sign_and_submit_then_watch_default(&tx, &signer)
            .await
            .map_err(|e| format!("Failed to submit transaction: {e}"))?;

        let tx_in_block = tokio::time::timeout(
            std::time::Duration::from_secs(60),
            tx_progress.wait_for_finalized(),
        )
        .await
        .map_err(|_| "Transaction timed out awaiting block finalization (60s)".to_string())?
        .map_err(|e| format!("Transaction finalization failed: {e}"))?;

        let block_hash = tx_in_block.block_hash();
        let extrinsic_hash = tx_in_block.extrinsic_hash();

        let _events = tx_in_block
            .wait_for_success()
            .await
            .map_err(|e| format!("Transaction execution failed: {e}"))?;

        // TAG: surface=blockchain owner=blockchain-team rule=BC-001
        // Query the finalized block header directly using the true block hash
        let rpc_client = if self.testnet_ws_url.starts_with("wss://") {
            RpcClient::from_url(&self.testnet_ws_url)
                .await
                .map_err(|e| format!("Failed to connect for block query: {e}"))?
        } else {
            RpcClient::from_insecure_url(&self.testnet_ws_url)
                .await
                .map_err(|e| format!("Failed to connect for block query: {e}"))?
        };
        let rpc = LegacyRpcMethods::<subxt::config::RpcConfigFor<SubstrateConfig>>::new(rpc_client);

        let header = rpc
            .chain_get_header(Some(block_hash))
            .await
            .map_err(|e| format!("Failed to get header for finalized block: {e}"))?;
        let block_number = header.map_or(0, |h| h.number);

        info!(
            "Successfully anchored hash in finalized block {} (0x{}) (extrinsic: 0x{})",
            block_number,
            hex::encode(block_hash.0),
            hex::encode(extrinsic_hash.0)
        );

        Ok(AnchorResponse {
            status: "success".to_string(),
            block_hash: Some(format!("0x{}", hex::encode(block_hash.0))),
            extrinsic_hash: Some(format!("0x{}", hex::encode(extrinsic_hash.0))),
            // TAG: surface=blockchain owner=blockchain-team rule=BC-001
            block_number: Some(u32::try_from(block_number).unwrap_or(0)),
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Generate `report_id` from `agent_id` and hash
    // Allow: kept as a method so anchoring call sites read as instance
    // operations (`self.generate_report_id(...)`); it is stateless by design.
    #[allow(clippy::unused_self)]
    fn generate_report_id(&self, agent_id: &str, hash: &str) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(b"freshcredit-simulation:");
        hasher.update(agent_id.as_bytes());
        hasher.update(b":");
        hasher.update(hash.as_bytes());
        hasher.finalize().into()
    }

    /// Parse report type string to runtime enum
    // Allow: kept as a method alongside the other report-mapping helpers for a
    // uniform anchoring API surface; it is stateless by design.
    #[allow(clippy::unused_self)]
    fn parse_report_type(
        &self,
        report_type: &str,
    ) -> freshcredit_runtime::runtime_types::pallet_freshcredit::ReportType {
        use freshcredit_runtime::runtime_types::pallet_freshcredit::ReportType as RuntimeReportType;
        match report_type.to_lowercase().as_str() {
            "identity_verification" | "identityverification" => {
                RuntimeReportType::IdentityVerification
            }
            // TAG: surface=blockchain owner=blockchain-team rule=BC-001
            "transaction_history" | "transactionhistory" => RuntimeReportType::TransactionHistory,
            "balance_snapshot" | "balancesnapshot" => RuntimeReportType::BalanceSnapshot,
            "consumer_preferences" | "consumerpreferences" => {
                RuntimeReportType::ConsumerPreferences
            }
            "provider_requirements" | "providerrequirements" => {
                RuntimeReportType::ProviderRequirements
            }
            _ => RuntimeReportType::PlaidFinancial, // Default
        }
    }

    // Allow: the healthy/degraded branch structure is kept explicit for
    // anchoring health-check auditability rather than folded into closures.
    #[allow(clippy::branches_sharing_code, clippy::option_if_let_else)]
    pub async fn health(&self) -> HealthStatus {
        let start = std::time::Instant::now();

        // Try to connect and get latest block via RPC
        match RpcClient::from_insecure_url(&self.testnet_ws_url).await {
            Ok(rpc_client) => {
                let rpc = LegacyRpcMethods::<subxt::config::RpcConfigFor<SubstrateConfig>>::new(
                    rpc_client,
                );
                if let Ok(header) = rpc.chain_get_header(None).await {
                    // Latency can never realistically approach u64::MAX millis.
                    let latency = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
                    HealthStatus {
                        status: "healthy".to_string(),
                        testnet_connection: "connected".to_string(),
                        paseo_connection: Some("connected".to_string()),
                        // TAG: surface=blockchain owner=blockchain-team rule=BC-001
                        latest_block: header.map(|h| u32::try_from(h.number).unwrap_or(0)),
                        latency_ms: Some(latency),
                        queued_transactions: 0,
                    }
                } else {
                    let latency = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
                    HealthStatus {
                        status: "degraded".to_string(),
                        testnet_connection: "error".to_string(),
                        paseo_connection: Some("error".to_string()),
                        latest_block: None,
                        latency_ms: Some(latency),
                        queued_transactions: 0,
                    }
                }
            }
            Err(_) => HealthStatus {
                status: "unhealthy".to_string(),
                testnet_connection: "disconnected".to_string(),
                paseo_connection: Some("disconnected".to_string()),
                latest_block: None,
                latency_ms: None,
                queued_transactions: 0,
            },
        }
    }

    /// Verify a hash exists on the blockchain
    // TAG: surface=blockchain owner=blockchain-team rule=BC-001
    pub async fn verify_hash(&self, hash: &str) -> Result<serde_json::Value, String> {
        // Parse hash
        let hash_bytes: [u8; 32] = match hex::decode(hash) {
            Ok(bytes) if bytes.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                arr
            }
            _ => return Err("Invalid hash format".to_string()),
        };

        let client = self.get_client().await?;
        let signer = self.load_signer()?;
        let signer_account_id = subxt::utils::AccountId32::from(signer.public_key().0);

        // Query storage for all report hashes under signer's account
        let at = client
            .at_current_block()
            .await
            .map_err(|e| format!("Failed to get latest block: {e}"))?;

        let address = freshcredit_runtime::storage()
            .fresh_credit()
            .report_hashes();

        // TAG: surface=blockchain owner=blockchain-team rule=BC-001
        let mut iter = at
            .storage()
            .iter(&address, (signer_account_id,))
            .await
            .map_err(|e| format!("Failed to iterate storage: {e}"))?;

        while let Some(Ok(kv)) = iter.next().await {
            let stored_hash = kv
                .value()
                .decode()
                .map_err(|e| format!("Decode error: {e}"))?;
            if stored_hash == hash_bytes {
                // Found matching hash
                return Ok(serde_json::json!({
                    "hash": hash,
                    "verified": true,
                    "account": format!("0x{}", hex::encode(signer_account_id.0)),
                }));
            }
        }

        Ok(serde_json::json!({
            "hash": hash,
            "verified": false,
        }))
    }
    // TAG: surface=blockchain owner=blockchain-team rule=BC-001
}
