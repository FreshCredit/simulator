// TAG: surface=blockchain owner=blockchain-team rule=BC-001
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::metrics::StatisticalTestResult;

/// Request to anchor a hash on the blockchain
///
/// Used to permanently record simulation data hashes on the `FreshCredit` testnet
/// for auditability and verification purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorRequest {
    /// 32-byte hash (hex-encoded) to anchor on-chain
    pub hash: String,
    /// Optional metadata about the anchored data
    pub metadata: Option<serde_json::Value>,
    /// Identifier of the agent submitting the hash
    pub agent_id: String,
}

/// Response from a hash anchoring operation
///
/// Contains the blockchain transaction details confirming the anchor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorResponse {
    /// Status of the anchoring operation ("success" or "error")
    pub status: String,
    /// Block hash where the extrinsic was included
    pub block_hash: Option<String>,
    /// Hash of the submitted extrinsic
    pub extrinsic_hash: Option<String>,
    /// Block number where the extrinsic was included
    pub block_number: Option<u32>,
    /// ISO 8601 timestamp of the response
    // TAG: surface=blockchain owner=blockchain-team rule=BC-001
    pub timestamp: String,
}

/// Health status of the agent bridge service
///
/// Provides connectivity information about the testnet connection
/// and service operational status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall service status ("healthy", "degraded", or "unhealthy")
    pub status: String,
    /// Testnet connection status ("connected", "error", or "disconnected")
    pub testnet_connection: String,
    /// Paseo testnet connection status alias for E2E contract compatibility
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paseo_connection: Option<String>,
    /// Latest block number known to the bridge
    pub latest_block: Option<u32>,
    /// Latency to the testnet node in milliseconds
    pub latency_ms: Option<u64>,
    /// Number of transactions queued for submission
    pub queued_transactions: usize,
}

/// Request to collect simulation metrics
///
/// Stores agent metrics for statistical analysis and comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectMetricsRequest {
    /// Unique identifier for the simulation run
    pub simulation_id: String,
    /// Identifier of the agent being measured
    pub agent_id: String,
    /// Key-value pairs of metric names and values
    pub metrics: HashMap<String, f64>,
    /// Optional fraud detection score
    pub fraud_score: Option<f64>,
    // TAG: surface=blockchain owner=blockchain-team rule=BC-001
    /// Block number associated with this measurement
    pub block_number: Option<u32>,
    /// Hash of the anchored extrinsic, if any
    pub extrinsic_hash: Option<String>,
}

/// Response from metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectMetricsResponse {
    /// Status of the collection operation
    pub status: String,
    /// Total number of metrics now stored
    pub collected_count: usize,
}

/// Request to compare two agents statistically
///
/// Performs a Mann-Whitney U test to compare fraud scores
/// between two agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareAgentsRequest {
    /// First agent identifier
    pub agent_a: String,
    /// Second agent identifier
    pub agent_b: String,
}

/// Response from agent comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareAgentsResponse {
    /// First agent identifier
    pub agent_a: String,
    /// Second agent identifier
    // TAG: surface=blockchain owner=blockchain-team rule=BC-001
    pub agent_b: String,
    /// Statistical test result, if comparison was possible
    pub result: Option<StatisticalTestResult>,
}

/// Response with metrics analysis summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResponse {
    /// Total number of metrics collected
    pub total_metrics: usize,
    /// List of unique agent IDs with metrics
    pub agents: Vec<String>,
    /// ISO 8601 timestamp of the analysis
    pub timestamp: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_anchor_request_serde_roundtrip() {
        let original = AnchorRequest {
            hash: "0xdeadbeef".to_string(),
            metadata: Some(serde_json::json!({"key": "value"})),
            agent_id: "agent-1".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: AnchorRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(original.hash, deserialized.hash);
        assert_eq!(original.agent_id, deserialized.agent_id);
    }

    #[test]
    fn test_anchor_response_serde_roundtrip() {
        // TAG: surface=blockchain owner=blockchain-team rule=BC-001
        let original = AnchorResponse {
            status: "success".to_string(),
            block_hash: Some("0xabc".to_string()),
            extrinsic_hash: Some("0xdef".to_string()),
            block_number: Some(42),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: AnchorResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(original.status, deserialized.status);
        assert_eq!(original.block_number, deserialized.block_number);
    }

    #[test]
    fn test_health_status_serde_roundtrip() {
        let original = HealthStatus {
            status: "healthy".to_string(),
            testnet_connection: "connected".to_string(),
            paseo_connection: Some("connected".to_string()),
            latest_block: Some(100),
            latency_ms: Some(50),
            queued_transactions: 0,
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: HealthStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(original.status, deserialized.status);
        assert_eq!(original.latency_ms, deserialized.latency_ms);
    }

    #[test]
    fn test_collect_metrics_request_serde_roundtrip() {
        let mut metrics = HashMap::new();
        metrics.insert("score".to_string(), 0.95);
        let original = CollectMetricsRequest {
            // TAG: surface=blockchain owner=blockchain-team rule=BC-001
            simulation_id: "sim-1".to_string(),
            agent_id: "agent-1".to_string(),
            metrics,
            fraud_score: Some(0.1),
            block_number: Some(10),
            extrinsic_hash: Some("0xhash".to_string()),
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: CollectMetricsRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(original.simulation_id, deserialized.simulation_id);
        assert_eq!(original.fraud_score, deserialized.fraud_score);
    }

    #[test]
    fn test_compare_agents_response_serde_roundtrip() {
        let original = CompareAgentsResponse {
            agent_a: "agent-a".to_string(),
            agent_b: "agent-b".to_string(),
            result: Some(crate::metrics::StatisticalTestResult {
                test_name: "Mann-Whitney U".to_string(),
                statistic: 10.0,
                p_value: 0.01,
                significant: true,
                interpretation: "Significant".to_string(),
            }),
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: CompareAgentsResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(original.agent_a, deserialized.agent_a);
        assert_eq!(original.agent_b, deserialized.agent_b);
        assert!(deserialized.result.is_some());
        let result = deserialized.result.unwrap();
        assert!(result.significant);
    }
    // TAG: surface=blockchain owner=blockchain-team rule=BC-001
}
