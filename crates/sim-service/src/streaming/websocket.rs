//! WebSocket streaming for real-time experiment monitoring (P1-API-2)
//!
//! Provides WebSocket endpoints for live experiment progress updates.
//!
//! ## Routes
//!
//! | Path | Description |
//! |------|-------------|
//! | `/ws/experiments/:id` | Subscribe to experiment progress |
//! | `/ws/batch/:batch_id` | Subscribe to batch experiment updates |

use axum::{
    extract::{ConnectInfo, Path, State, WebSocketUpgrade},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::rate_limit::connection::ConnectionLimiter;

const MAX_WS_MESSAGE_BYTES: usize = 64 * 1024;
const MAX_WS_FRAME_BYTES: usize = 16 * 1024;
const MAX_WS_WRITE_BUFFER_BYTES: usize = 256 * 1024;

fn extract_ws_client_ip(headers: &HeaderMap, peer: Option<ConnectInfo<SocketAddr>>) -> IpAddr {
    let peer_ip = peer.map(|ci| ci.0.ip());
    let behind_proxy = peer_ip.is_none_or(|ip| ip.is_loopback())
        || std::env::var("TRUST_PROXY_HEADERS")
            .is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
    if behind_proxy {
        if let Some(xff_ip) = headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| {
                s.split(',')
                    .rev()
                    .map(str::trim)
                    .find_map(|entry| entry.parse::<IpAddr>().ok())
            })
        {
            return xff_ip;
        }
    }
    peer_ip.unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    /// Progress update
    Progress {
        /// Run identifier
        run_id: String,
        /// Experiment identifier
        experiment_id: String,
        /// Progress percentage
        progress_pct: f64,
        /// Current step
        current_step: String,
        /// Metrics
        metrics: Option<serde_json::Value>,
        /// Timestamp
        timestamp: String,
    },
    /// Status change
    StatusChange {
        /// Run identifier
        run_id: String,
        /// Experiment identifier
        experiment_id: String,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        /// Old status
        old_status: String,
        /// New status
        new_status: String,
        /// Timestamp
        timestamp: String,
    },
    /// Metric update
    MetricUpdate {
        /// Run identifier
        run_id: String,
        /// Experiment identifier
        experiment_id: String,
        /// Metric name
        metric_name: String,
        /// Metric value
        metric_value: f64,
        /// Timestamp
        timestamp: String,
    },
    /// Completion notification
    Complete {
        /// Run identifier
        run_id: String,
        /// Experiment identifier
        experiment_id: String,
        /// Whether the run succeeded
        success: bool,
        /// Final metrics
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        final_metrics: serde_json::Value,
        /// Duration in seconds
        duration_secs: f64,
        /// Timestamp
        timestamp: String,
    },
    /// Error notification
    Error {
        /// Run identifier
        run_id: String,
        /// Experiment identifier
        experiment_id: String,
        /// Error message
        error_message: String,
        /// Timestamp
        timestamp: String,
    },
    /// Ping/keepalive
    Ping {
        /// Timestamp
        timestamp: String,
    },
    /// Pong response
    Pong {
        /// Timestamp
        timestamp: String,
    },
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// WebSocket connection state
#[derive(Debug, Clone)]
pub struct WsConnection {
    /// connection id
    pub connection_id: String,
    /// subscribed experiments
    pub subscribed_experiments: Vec<String>,
    /// subscribed batches
    pub subscribed_batches: Vec<String>,
    /// connected at
    pub connected_at: String,
}

/// GET /ws/experiments/:id - WebSocket endpoint for experiment progress
pub async fn experiment_websocket(
    State(limiter): State<Arc<ConnectionLimiter>>,
    headers: HeaderMap,
    peer: Option<ConnectInfo<SocketAddr>>,
    Path(experiment_id): Path<String>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let ip = extract_ws_client_ip(&headers, peer);
    if !limiter.check(ip).await {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    info!(experiment_id = %experiment_id, "WebSocket connection requested for experiment");

    ws.max_message_size(MAX_WS_MESSAGE_BYTES)
        .max_frame_size(MAX_WS_FRAME_BYTES)
        .max_write_buffer_size(MAX_WS_WRITE_BUFFER_BYTES)
        .on_upgrade(move |socket| handle_experiment_socket(socket, experiment_id))
}

/// Handle experiment WebSocket connection
async fn handle_experiment_socket(mut socket: axum::extract::ws::WebSocket, experiment_id: String) {
    let connection_id = uuid::Uuid::new_v4().to_string();
    info!(
            connection_id = %connection_id,
            experiment_id = %experiment_id,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            "WebSocket connection established"
        );

    // Send initial connection confirmation
    let confirm_msg = WsMessage::Progress {
        run_id: connection_id.clone(),
        experiment_id: experiment_id.clone(),
        progress_pct: 0.0,
        current_step: "connected".to_string(),
        metrics: None,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    if let Ok(json) = serde_json::to_string(&confirm_msg) {
        if socket
            .send(axum::extract::ws::Message::Text(json))
            .await
            .is_err()
        {
            error!(connection_id = %connection_id, "Failed to send initial message");
            return;
        }
    }

    // Main message loop
    // P10-R4: bounded by connection lifetime — every select! arm breaks on socket close or error.
    loop {
        tokio::select! {
                    // Handle incoming messages from client
                    msg = socket.recv() => {
                        match msg {
                            Some(Ok(axum::extract::ws::Message::Text(text))) => {
        // TAG: surface=simulation owner=research-team rule=SIM-001
                                // Parse client message
                                match serde_json::from_str::<WsMessage>(&text) {
                                    Ok(WsMessage::Ping { .. }) => {
                                        // Respond with pong
                                        let pong = WsMessage::Pong {
                                            timestamp: chrono::Utc::now().to_rfc3339(),
                                        };
                                        if let Ok(json) = serde_json::to_string(&pong) {
                                            let _ = socket.send(axum::extract::ws::Message::Text(json)).await;
                                        }
                                    }
                                    Ok(_) => {
                                        // Other client messages - log for now
                                        info!(connection_id = %connection_id, "Received client message");
                                    }
                                    Err(e) => {
                                        warn!(connection_id = %connection_id, error = %e, "Invalid client message");
                                    }
                                }
                            }
                            Some(Ok(axum::extract::ws::Message::Close(_))) => {
                                info!(connection_id = %connection_id, "Client closed connection");
                                break;
                            }
                            Some(Err(e)) => {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                                error!(connection_id = %connection_id, error = %e, "WebSocket error");
                                break;
                            }
                            None => {
                                info!(connection_id = %connection_id, "Connection closed");
                                break;
                            }
                            _ => {}
                        }
                    }

                    // ROADMAP: Subscribe to experiment progress updates from executor
                    // For now, just send periodic heartbeats
                    () = tokio::time::sleep(tokio::time::Duration::from_secs(30)) => {
                        let ping = WsMessage::Ping {
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        };
                        if let Ok(json) = serde_json::to_string(&ping) {
                            if socket.send(axum::extract::ws::Message::Text(json)).await.is_err() {
                                break;
                            }
                        }
                    }
                }
    }

    info!(connection_id = %connection_id, "WebSocket connection closed");
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /`ws/batch/:batch_id` - WebSocket endpoint for batch updates
pub async fn batch_websocket(
    State(limiter): State<Arc<ConnectionLimiter>>,
    headers: HeaderMap,
    peer: Option<ConnectInfo<SocketAddr>>,
    Path(batch_id): Path<String>,
    ws: WebSocketUpgrade,
    // TAG: surface=simulation owner=research-team rule=SIM-001
) -> impl IntoResponse {
    let ip = extract_ws_client_ip(&headers, peer);
    if !limiter.check(ip).await {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    info!(batch_id = %batch_id, "WebSocket connection requested for batch");

    ws.max_message_size(MAX_WS_MESSAGE_BYTES)
        .max_frame_size(MAX_WS_FRAME_BYTES)
        .max_write_buffer_size(MAX_WS_WRITE_BUFFER_BYTES)
        .on_upgrade(move |socket| handle_batch_socket(socket, batch_id))
}

/// Handle batch WebSocket connection
async fn handle_batch_socket(mut socket: axum::extract::ws::WebSocket, batch_id: String) {
    let connection_id = uuid::Uuid::new_v4().to_string();
    info!(
        connection_id = %connection_id,
        batch_id = %batch_id,
        "Batch WebSocket connection established"
    );

    // Send initial connection confirmation
    let confirm_msg = WsMessage::Progress {
        run_id: connection_id.clone(),
        experiment_id: batch_id.clone(),
        progress_pct: 0.0,
        current_step: "batch_connected".to_string(),
        metrics: None,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    if let Ok(json) = serde_json::to_string(&confirm_msg) {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let _ = socket.send(axum::extract::ws::Message::Text(json)).await;
    }

    // ROADMAP: Subscribe to batch progress updates
    // For now, just keep connection alive
    // P10-R4: bounded by connection lifetime — breaks on socket close or heartbeat send failure.
    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(Ok(axum::extract::ws::Message::Close(_))) | None => {
                        break;
                    }
                    _ => {}
                }
            }
            () = tokio::time::sleep(tokio::time::Duration::from_secs(60)) => {
                // Send periodic heartbeat
                let ping = WsMessage::Ping {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                if let Ok(json) = serde_json::to_string(&ping) {
                    if socket.send(axum::extract::ws::Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
        }
    }

    info!(connection_id = %connection_id, "Batch WebSocket connection closed");
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Broadcast message to all subscribers of an experiment
pub async fn broadcast_to_experiment(_experiment_id: &str, _message: WsMessage) {
    // ROADMAP: Implement broadcast to all connected clients
    // This will require a connection manager (e.g., using DashMap)
}

/// Connection manager for tracking active WebSocket connections
#[derive(Debug)]
pub struct ConnectionManager {
    // ROADMAP: Use DashMap<String, Vec<WsConnection>> for experiment -> connections mapping
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionManager {
    #[must_use]
    /// fn
    pub const fn new() -> Self {
        Self {}
    }

    /// add connection
    pub fn add_connection(&self, _connection: WsConnection) {
        // ROADMAP: Implement connection registration
    }

    /// fn
    pub const fn remove_connection(&self, _connection_id: &str) {
        // ROADMAP: Implement connection removal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_message_serialization() {
        let msg = WsMessage::Progress {
            run_id: "run_1".to_string(),
            experiment_id: "exp_1".to_string(),
            progress_pct: 75.5,
            current_step: "evaluation".to_string(),
            metrics: Some(serde_json::json!({"accuracy": 0.94})),
            timestamp: "2026-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("progress_pct"));
        assert!(json.contains("75.5"));

        let ping = WsMessage::Ping {
            timestamp: "2026-01-01T00:00:00Z".to_string(),
        };
        let ping_json = serde_json::to_string(&ping).unwrap();
        assert!(ping_json.contains("ping"));

        let mgr = ConnectionManager::default();
        mgr.remove_connection("conn_1");
    }
}
