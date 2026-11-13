// TAG: surface=blockchain owner=blockchain-team rule=BC-001
use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::error;

use crate::bridge::AgentBridge;
use crate::metrics::SimulationMetrics;
use crate::types::{
    AnalysisResponse, AnchorRequest, AnchorResponse, CollectMetricsRequest, CollectMetricsResponse,
    CompareAgentsRequest, CompareAgentsResponse, HealthStatus,
};

pub async fn health_handler(State(bridge): State<Arc<RwLock<AgentBridge>>>) -> Json<HealthStatus> {
    let bridge = bridge.read().await;
    Json(bridge.health().await)
}

pub async fn anchor_handler(
    State(bridge): State<Arc<RwLock<AgentBridge>>>,
    Json(request): Json<AnchorRequest>,
) -> Result<Json<AnchorResponse>, (StatusCode, String)> {
    let bridge = bridge.read().await;
    match bridge.anchor_hash(request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Anchor failed: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e))
        }
    }
}

pub async fn verify_handler(
    // TAG: surface=blockchain owner=blockchain-team rule=BC-001
    State(bridge): State<Arc<RwLock<AgentBridge>>>,
    axum::extract::Path(hash): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let bridge = bridge.read().await;
    match bridge.verify_hash(&hash).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => {
            error!("Verify failed: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e))
        }
    }
}

pub async fn collect_metrics_handler(
    State(bridge): State<Arc<RwLock<AgentBridge>>>,
    Json(request): Json<CollectMetricsRequest>,
) -> Json<CollectMetricsResponse> {
    let mut bridge = bridge.write().await;

    let metric = SimulationMetrics {
        simulation_id: request.simulation_id,
        agent_id: request.agent_id,
        timestamp: chrono::Utc::now(),
        metrics: request.metrics,
        fraud_score: request.fraud_score,
        block_number: request.block_number,
        extrinsic_hash: request.extrinsic_hash,
    };

    bridge.metrics_collector.add(metric);

    Json(CollectMetricsResponse {
        // TAG: surface=blockchain owner=blockchain-team rule=BC-001
        status: "success".to_string(),
        collected_count: bridge.metrics_collector.len(),
    })
}

pub async fn compare_agents_handler(
    State(bridge): State<Arc<RwLock<AgentBridge>>>,
    Json(request): Json<CompareAgentsRequest>,
) -> Json<CompareAgentsResponse> {
    let result = bridge
        .read()
        .await
        .metrics_collector
        .compare_agents(&request.agent_a, &request.agent_b);

    Json(CompareAgentsResponse {
        agent_a: request.agent_a,
        agent_b: request.agent_b,
        result,
    })
}

pub async fn get_analysis_handler(
    State(bridge): State<Arc<RwLock<AgentBridge>>>,
) -> Json<AnalysisResponse> {
    let bridge = bridge.read().await;

    // Get unique agent IDs
    let mut agents: Vec<String> = bridge
        .metrics_collector
        .metrics
        .iter()
        // TAG: surface=blockchain owner=blockchain-team rule=BC-001
        .map(|m| m.agent_id.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    agents.sort();

    Json(AnalysisResponse {
        total_metrics: bridge.metrics_collector.len(),
        agents,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::AgentBridge;
    use crate::types::{CollectMetricsRequest, CompareAgentsRequest};
    use std::collections::HashMap;

    fn test_state() -> State<Arc<RwLock<AgentBridge>>> {
        let bridge = AgentBridge::new("ws://invalid:0".to_string(), "//Alice".to_string());
        State(Arc::new(RwLock::new(bridge)))
    }

    #[tokio::test]
    async fn test_health_handler_returns_status() {
        let state = test_state();
        let Json(status) = health_handler(state).await;
        assert!(!status.status.is_empty());
    }

    // TAG: surface=blockchain owner=blockchain-team rule=BC-001
    #[tokio::test]
    async fn test_collect_metrics_handler_increments_count() {
        let state = test_state();
        let req = CollectMetricsRequest {
            simulation_id: "sim-1".to_string(),
            agent_id: "agent-1".to_string(),
            metrics: HashMap::new(),
            fraud_score: Some(0.5),
            block_number: Some(1),
            extrinsic_hash: None,
        };
        let Json(resp1) = collect_metrics_handler(state.clone(), Json(req.clone())).await;
        assert_eq!(resp1.collected_count, 1);

        let Json(resp2) = collect_metrics_handler(state, Json(req)).await;
        assert_eq!(resp2.collected_count, 2);
    }

    #[tokio::test]
    async fn test_compare_agents_handler_insufficient_data() {
        let state = test_state();
        let req = CompareAgentsRequest {
            agent_a: "agent-a".to_string(),
            agent_b: "agent-b".to_string(),
        };
        let Json(resp) = compare_agents_handler(state, Json(req)).await;
        assert!(resp.result.is_none());
    }

    #[tokio::test]
    async fn test_get_analysis_handler_empty() {
        // TAG: surface=blockchain owner=blockchain-team rule=BC-001
        let state = test_state();
        let Json(analysis) = get_analysis_handler(state).await;
        assert_eq!(analysis.total_metrics, 0);
        assert!(analysis.agents.is_empty());
    }

    #[tokio::test]
    async fn test_get_analysis_handler_with_metrics() {
        let state = test_state();
        let req1 = CollectMetricsRequest {
            simulation_id: "sim-1".to_string(),
            agent_id: "agent-beta".to_string(),
            metrics: HashMap::new(),
            fraud_score: Some(0.1),
            block_number: Some(1),
            extrinsic_hash: None,
        };
        let req2 = CollectMetricsRequest {
            simulation_id: "sim-2".to_string(),
            agent_id: "agent-alpha".to_string(),
            metrics: HashMap::new(),
            fraud_score: Some(0.2),
            block_number: Some(2),
            extrinsic_hash: None,
        };
        let _ = collect_metrics_handler(state.clone(), Json(req1)).await;
        let _ = collect_metrics_handler(state.clone(), Json(req2)).await;

        let Json(analysis) = get_analysis_handler(state).await;
        assert_eq!(analysis.total_metrics, 2);
        assert_eq!(analysis.agents, vec!["agent-alpha", "agent-beta"]);
    }
    // TAG: surface=blockchain owner=blockchain-team rule=BC-001
}
