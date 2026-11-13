// TAG: surface=simulation owner=research-team rule=SIM-001
//! Simulator route handlers — proxy to sim-service

use askama::Template;
use axum::{
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json, Response},
};
use futures::stream::StreamExt;
use std::sync::Arc;
use tokio::time::Duration;
use tracing::{info, warn};

use crate::types::{AppState, IndexTemplate, MAX_BODY_SIZE, PROXY_TIMEOUT};

// ============================================================================
// Route Handlers
// ============================================================================

/// Main simulator page
pub async fn index(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    // Use empty string for same-origin proxy (most reliable for deployments)
    // This ensures the frontend uses relative URLs that go through this UI service
    // SEC-001: HTML escape the value to prevent XSS
    let template = IndexTemplate {
        sim_service_url: html_escape::encode_safe("").to_string(),
    };
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            warn!("Template render error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Health check endpoint
pub async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Check sim-service connectivity
    let sim_service_connected = state
        .http_client
        .get(format!("{}/health", state.sim_service_url))
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);
    // TAG: surface=simulation owner=research-team rule=SIM-001

    Json(serde_json::json!({
        "status": "ok",
        "service": "simulator",
        "sim_service_connected": sim_service_connected
    }))
}

// ============================================================================
// API Proxy Routes (proxy to sim-service)
// ============================================================================

/// List available simulation runs
pub async fn list_runs(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    proxy_get(state, "/api/runs").await
}

/// Get manifest for a specific run
pub async fn get_manifest(
    State(state): State<Arc<AppState>>,
    Path(run_id): Path<String>,
) -> impl IntoResponse {
    proxy_get(state, &format!("/api/runs/{run_id}/manifest")).await
}

/// Get replay frame for a specific tick
pub async fn get_replay_frame(
    State(state): State<Arc<AppState>>,
    Path((run_id, tick)): Path<(String, u64)>,
) -> impl IntoResponse {
    proxy_get(state, &format!("/api/runs/{run_id}/replay/{tick}")).await
}

// ============================================================================
// PDE4B API Proxy Routes
// ============================================================================

/// Get PDE4B configuration
pub async fn get_pde4b_config(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    proxy_get(state, "/api/config/pde4b").await
}

/// Update PDE4B configuration
pub async fn update_pde4b_config(
    State(state): State<Arc<AppState>>,
    body: Body,
) -> impl IntoResponse {
    proxy_post_body(state, "/api/config/pde4b", body).await
}
// TAG: surface=simulation owner=research-team rule=SIM-001

/// Get lambda statistics
pub async fn get_lambda_stats(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    proxy_get(state, "/api/stats/lambda").await
}

// ============================================================================
// Experiment API Proxy Routes
// ============================================================================

/// List all experiments
pub async fn list_experiments(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    proxy_get(state, "/api/experiments").await
}

/// Get experiment details
pub async fn get_experiment(
    State(state): State<Arc<AppState>>,
    Path(exp_id): Path<String>,
) -> impl IntoResponse {
    proxy_get(state, &format!("/api/experiments/{exp_id}")).await
}

/// Get experiment schema
pub async fn get_experiment_schema(
    State(state): State<Arc<AppState>>,
    Path(exp_id): Path<String>,
) -> impl IntoResponse {
    proxy_get(state, &format!("/api/experiments/{exp_id}/schema")).await
}

/// Launch experiment
pub async fn launch_experiment(
    State(state): State<Arc<AppState>>,
    Path(exp_id): Path<String>,
    body: Body,
) -> impl IntoResponse {
    proxy_post_body(state, &format!("/api/experiments/{exp_id}/runs"), body).await
}

/// Get experiment run status
pub async fn get_experiment_run_status(
    State(state): State<Arc<AppState>>,
    Path(run_id): Path<String>,
) -> impl IntoResponse {
    proxy_get(state, &format!("/api/experiment-runs/{run_id}")).await
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Stream experiment progress (SSE)
pub async fn stream_experiment_progress(
    State(state): State<Arc<AppState>>,
    Path(run_id): Path<String>,
) -> impl IntoResponse {
    // PERF-001: Add request timeout (shorter for SSE)
    match state
        .http_client
        .get(format!(
            "{}/api/experiment-runs/{}/stream",
            state.sim_service_url, run_id
        ))
        .timeout(Duration::from_secs(60))
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            (
                StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                body,
            )
                .into_response()
        }
        Err(e) => {
            warn!("sim-service error: {e}");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                format!(r#"{{"error": "sim-service unavailable: {e}"}}"#),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn setup_state() -> (Arc<AppState>, MockServer) {
        let mock_server = MockServer::start().await;
        let state = Arc::new(AppState {
            sim_service_url: mock_server.uri(),
            http_client: reqwest::Client::new(),
            // TAG: surface=simulation owner=research-team rule=SIM-001
        });
        (state, mock_server)
    }

    #[tokio::test]
    async fn test_index_handler() {
        let state = Arc::new(AppState {
            sim_service_url: "http://localhost:8082".to_string(),
            http_client: reqwest::Client::new(),
        });
        let response = index(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), MAX_BODY_SIZE)
            .await
            .unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("Agent Society Simulator"));
    }

    #[tokio::test]
    async fn test_health_handler_success() {
        let (state, mock_server) = setup_state().await;
        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"status":"ok"})),
            )
            .mount(&mock_server)
            .await;

        let response = health(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), MAX_BODY_SIZE)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["service"], "simulator");
        assert_eq!(json["sim_service_connected"], true);
    }

    #[tokio::test]
    async fn test_health_handler_failure() {
        let (state, _mock_server) = setup_state().await;
        let response = health(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let body = axum::body::to_bytes(response.into_body(), MAX_BODY_SIZE)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["sim_service_connected"], false);
    }

    #[tokio::test]
    async fn test_list_runs_handler() {
        let (state, mock_server) = setup_state().await;
        Mock::given(method("GET"))
            .and(path("/api/runs"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!([{"id":"run1"}])),
            )
            .mount(&mock_server)
            .await;

        let response = list_runs(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_manifest_handler() {
        let (state, mock_server) = setup_state().await;
        Mock::given(method("GET"))
            .and(path("/api/runs/run-123/manifest"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"run_id":"run-123"})),
            )
            .mount(&mock_server)
            .await;

        let response = get_manifest(State(state), Path("run-123".to_string()))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_replay_frame_handler() {
        let (state, mock_server) = setup_state().await;
        Mock::given(method("GET"))
            .and(path("/api/runs/run-123/replay/42"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"tick":42})))
            .mount(&mock_server)
            .await;

        let response = get_replay_frame(State(state), Path(("run-123".to_string(), 42)))
            // TAG: surface=simulation owner=research-team rule=SIM-001
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_pde4b_config_handler() {
        let (state, mock_server) = setup_state().await;
        Mock::given(method("GET"))
            .and(path("/api/config/pde4b"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"config":{}})),
            )
            .mount(&mock_server)
            .await;

        let response = get_pde4b_config(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_pde4b_config_handler_success() {
        let (state, mock_server) = setup_state().await;
        Mock::given(method("POST"))
            .and(path("/api/config/pde4b"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"updated":true})),
            )
            .mount(&mock_server)
            .await;

        let body = Body::from(r#"{"param":"value"}"#);
        let response = update_pde4b_config(State(state), body)
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_pde4b_config_handler_empty_body() {
        let (state, mock_server) = setup_state().await;
        Mock::given(method("POST"))
            .and(path("/api/config/pde4b"))
            .respond_with(ResponseTemplate::new(200).set_body_string(""))
            .mount(&mock_server)
            .await;

        let body = Body::from("");
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let response = update_pde4b_config(State(state), body)
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_lambda_stats_handler() {
        let (state, mock_server) = setup_state().await;
        Mock::given(method("GET"))
            .and(path("/api/stats/lambda"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"lambda":0.5})),
            )
            .mount(&mock_server)
            .await;

        let response = get_lambda_stats(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_experiments_handler() {
        let (state, mock_server) = setup_state().await;
        Mock::given(method("GET"))
            .and(path("/api/experiments"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!([{"id":"exp1"}])),
            )
            .mount(&mock_server)
            .await;

        let response = list_experiments(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_experiment_handler() {
        let (state, mock_server) = setup_state().await;
        Mock::given(method("GET"))
            .and(path("/api/experiments/exp-123"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":"exp-123"})),
            )
            .mount(&mock_server)
            .await;

        let response = get_experiment(State(state), Path("exp-123".to_string()))
            .await
            // TAG: surface=simulation owner=research-team rule=SIM-001
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_experiment_schema_handler() {
        let (state, mock_server) = setup_state().await;
        Mock::given(method("GET"))
            .and(path("/api/experiments/exp-123/schema"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"schema":{}})),
            )
            .mount(&mock_server)
            .await;

        let response = get_experiment_schema(State(state), Path("exp-123".to_string()))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_launch_experiment_handler() {
        let (state, mock_server) = setup_state().await;
        Mock::given(method("POST"))
            .and(path("/api/experiments/exp-123/runs"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"run_id":"run-1"})),
            )
            .mount(&mock_server)
            .await;

        let body = Body::from(r#"{"params":{}}"#);
        let response = launch_experiment(State(state), Path("exp-123".to_string()), body)
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_experiment_run_status_handler() {
        let (state, mock_server) = setup_state().await;
        Mock::given(method("GET"))
            .and(path("/api/experiment-runs/run-123"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"status":"running"})),
            )
            .mount(&mock_server)
            .await;
        // TAG: surface=simulation owner=research-team rule=SIM-001

        let response = get_experiment_run_status(State(state), Path("run-123".to_string()))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_stream_experiment_progress_success() {
        let (state, mock_server) = setup_state().await;
        Mock::given(method("GET"))
            .and(path("/api/experiment-runs/run-123/stream"))
            .respond_with(ResponseTemplate::new(200).set_body_string("data: progress\n\n"))
            .mount(&mock_server)
            .await;

        let response = stream_experiment_progress(State(state), Path("run-123".to_string()))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_stream_experiment_progress_failure() {
        let state = Arc::new(AppState {
            sim_service_url: "http://127.0.0.1:1".to_string(),
            http_client: reqwest::Client::new(),
        });
        let response = stream_experiment_progress(State(state), Path("run-123".to_string()))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_get_neural_agent_handler() {
        let (state, mock_server) = setup_state().await;
        Mock::given(method("GET"))
            .and(path("/api/neural/agent/42"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"agent_id":42})),
            )
            .mount(&mock_server)
            .await;

        let response = get_neural_agent(State(state), Path(42))
            .await
            .into_response();
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_neural_population_handler() {
        let (state, mock_server) = setup_state().await;
        Mock::given(method("GET"))
            .and(path("/api/neural/population"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"count":100})),
            )
            .mount(&mock_server)
            .await;

        let response = get_neural_population(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_live_stream_success() {
        let (state, mock_server) = setup_state().await;
        Mock::given(method("GET"))
            .and(path("/api/live"))
            .respond_with(ResponseTemplate::new(200).set_body_string("data: live\n\n"))
            .mount(&mock_server)
            .await;

        let response = live_stream(State(state)).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get(axum::http::header::CONTENT_TYPE)
                .unwrap(),
            "text/event-stream"
        );
        assert_eq!(
            response
                .headers()
                .get(axum::http::header::CACHE_CONTROL)
                .unwrap(),
            "no-cache, no-store, must-revalidate"
        );
        assert_eq!(response.headers().get("X-Accel-Buffering").unwrap(), "no");
    }

    #[tokio::test]
    async fn test_live_stream_failure() {
        let state = Arc::new(AppState {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            sim_service_url: "http://127.0.0.1:1".to_string(),
            http_client: reqwest::Client::new(),
        });
        let response = live_stream(State(state)).await;
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_live_stream_error_status() {
        let (state, mock_server) = setup_state().await;
        Mock::given(method("GET"))
            .and(path("/api/live"))
            .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
            .mount(&mock_server)
            .await;

        let response = live_stream(State(state)).await;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_set_speed_handler() {
        let (state, mock_server) = setup_state().await;
        Mock::given(method("POST"))
            .and(path("/api/live/speed"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"speed":1.0})),
            )
            .mount(&mock_server)
            .await;

        let body = Body::from(r#"{"speed":2.0}"#);
        let response = set_speed(State(state), body).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_speed_handler() {
        let (state, mock_server) = setup_state().await;
        Mock::given(method("GET"))
            .and(path("/api/live/speed"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"speed":1.0})),
            )
            .mount(&mock_server)
            .await;

        let response = get_speed(State(state)).await.into_response();
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_run_validation_handler() {
        let (state, mock_server) = setup_state().await;
        Mock::given(method("POST"))
            .and(path("/api/validate"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"valid":true})),
            )
            .mount(&mock_server)
            .await;

        let response = run_validation(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    // Error path tests for proxy functions
    #[tokio::test]
    async fn test_proxy_get_error_status() {
        let (state, mock_server) = setup_state().await;
        Mock::given(method("GET"))
            .and(path("/api/runs"))
            .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
            .mount(&mock_server)
            .await;

        let response = list_runs(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_proxy_post_error_status() {
        let (state, mock_server) = setup_state().await;
        Mock::given(method("POST"))
            .and(path("/api/validate"))
            .respond_with(ResponseTemplate::new(500).set_body_string("error"))
            .mount(&mock_server)
            .await;

        let response = run_validation(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_proxy_post_body_error_status() {
        let (state, mock_server) = setup_state().await;
        Mock::given(method("POST"))
            // TAG: surface=simulation owner=research-team rule=SIM-001
            .and(path("/api/config/pde4b"))
            .respond_with(ResponseTemplate::new(422).set_body_string("invalid"))
            .mount(&mock_server)
            .await;

        let body = Body::from(r#"{"bad":"data"}"#);
        let response = update_pde4b_config(State(state), body)
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}

// ============================================================================
// Neural Data Proxy Routes
// ============================================================================

/// Get agent brain state for neural visualization
pub async fn get_neural_agent(
    State(state): State<Arc<AppState>>,
    Path(agent_id): Path<u64>,
) -> impl IntoResponse {
    proxy_get(state, &format!("/api/neural/agent/{agent_id}")).await
}

/// Get population neural statistics
pub async fn get_neural_population(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    proxy_get(state, "/api/neural/population").await
}

// ============================================================================
fn sse_client(fallback: &reqwest::Client) -> reqwest::Client {
    static SSE_CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    let client = SSE_CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(3600))
                .connect_timeout(std::time::Duration::from_secs(10))
                .pool_idle_timeout(std::time::Duration::from_secs(120))
                .pool_max_idle_per_host(10)
                .tcp_keepalive(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| fallback.clone())
        })
        .clone();
    debug_assert!(!format!("{client:?}").is_empty(), "sse client initialized");
    client
}

// Live Simulation Stream Proxy Routes (CRITICAL - SSE streaming)
// ============================================================================

/// Live simulation stream (SSE) - proxied from sim-service
/// CRITICAL FIX: Uses streaming to avoid buffering infinite SSE stream
fn build_sse_response(resp: reqwest::Response, status: reqwest::StatusCode) -> Response {
    let stream = resp
        .bytes_stream()
        .map(|result| result.map_err(std::io::Error::other));

    let axum_status =
        StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    Response::builder()
        .status(axum_status)
        .header(axum::http::header::CONTENT_TYPE, "text/event-stream")
        .header(
            axum::http::header::CACHE_CONTROL,
            "no-cache, no-store, must-revalidate",
        )
        .header("X-Accel-Buffering", "no")
        .header(axum::http::header::CONNECTION, "keep-alive")
        .body(Body::from_stream(stream))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

/// CRITICAL FIX: Added proper timeouts and keep-alive handling
pub async fn live_stream(State(state): State<Arc<AppState>>) -> Response {
    info!("[PROXY] Connecting to sim-service live stream");
    info!("[PROXY] Target URL: {}/api/live", state.sim_service_url);

    let sse_client = sse_client(&state.http_client);

    info!("[PROXY] Sending request to sim-service...");
    let resp = match sse_client
        .get(format!("{}/api/live", state.sim_service_url))
        .send()
        .await
    {
        Ok(resp) => {
            info!("[PROXY] Got response from sim-service: {}", resp.status());
            resp
        }
        Err(e) => {
            warn!("[PROXY] Failed to connect to sim-service: {e}");
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                format!(r#"{{"error": "sim-service unavailable: {e}"}}"#),
            )
                .into_response();
        }
    };

    let status = resp.status();
    info!("[PROXY] sim-service response status: {status}");
    info!("[PROXY] Response headers: {:?}", resp.headers());

    if !status.is_success() {
        let error_body = resp.text().await.unwrap_or_default();
        warn!(
            "[PROXY] sim-service returned error: {} - {}",
            status, error_body
        );
        return (
            StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            format!(r#"{{"error": "sim-service error: {status}"}}"#),
        )
            .into_response();
    }

    build_sse_response(resp, status)
}

/// Set simulation speed
pub async fn set_speed(State(state): State<Arc<AppState>>, body: Body) -> impl IntoResponse {
    proxy_post_body(state, "/api/live/speed", body).await
}

/// Get current simulation speed
pub async fn get_speed(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    proxy_get(state, "/api/live/speed").await
}
/// Restart live simulation with custom parameters
pub async fn restart_simulation(
    State(state): State<Arc<AppState>>,
    body: Body,
) -> impl IntoResponse {
    proxy_post_body(state, "/api/live/restart", body).await
}
/// Apply live intervention parameters in real-time without restart
pub async fn live_intervention(
    State(state): State<Arc<AppState>>,
    body: Body,
) -> impl IntoResponse {
    proxy_post_body(state, "/api/live/intervention", body).await
}

/// Pause live simulation
pub async fn pause_simulation(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    proxy_post_empty(state, "/api/live/pause").await
}

/// Resume live simulation
pub async fn resume_simulation(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    proxy_post_empty(state, "/api/live/resume").await
}

/// Stop live simulation
pub async fn stop_simulation(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    proxy_post_empty(state, "/api/live/stop").await
}

/// Get live simulation status
pub async fn get_simulation_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    proxy_get(state, "/api/live/status").await
}

/// Export simulation state checkpoint
pub async fn export_simulation_state(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    proxy_get(state, "/api/live/state/export").await
}

/// Import simulation state checkpoint
pub async fn import_simulation_state(
    State(state): State<Arc<AppState>>,
    body: Body,
) -> impl IntoResponse {
    proxy_post_body(state, "/api/live/state/import", body).await
}

/// Get end-of-run simulation report
pub async fn get_simulation_report(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    proxy_get(state, "/api/live/report").await
}

// ============================================================================
// AI Chat Proxy Routes
// ============================================================================

/// AI chat context stream (SSE) - proxied from sim-service
pub async fn ai_stream(State(state): State<Arc<AppState>>) -> Response {
    info!("[PROXY] Connecting to sim-service AI stream");

    let sse_client = sse_client(&state.http_client);

    let resp = match sse_client
        .get(format!("{}/api/ai/stream", state.sim_service_url))
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            warn!("[PROXY] Failed to connect to sim-service AI stream: {e}");
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                format!(r#"{{"error": "sim-service unavailable: {e}"}}"#),
            )
                .into_response();
        }
    };

    let status = resp.status();
    if !status.is_success() {
        let error_body = resp.text().await.unwrap_or_default();
        warn!("[PROXY] sim-service AI stream error: {status} - {error_body}");
        return (
            StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            format!(r#"{{"error": "sim-service error: {status}"}}"#),
        )
            .into_response();
    }

    let stream = resp
        .bytes_stream()
        .map(|result| result.map_err(std::io::Error::other));
    let axum_status =
        StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    Response::builder()
        .status(axum_status)
        .header(axum::http::header::CONTENT_TYPE, "text/event-stream")
        .header(
            axum::http::header::CACHE_CONTROL,
            "no-cache, no-store, must-revalidate",
        )
        .header("X-Accel-Buffering", "no")
        .header(axum::http::header::CONNECTION, "keep-alive")
        .body(Body::from_stream(stream))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

/// AI chat message endpoint - proxied from sim-service
pub async fn ai_chat(State(state): State<Arc<AppState>>, body: Body) -> impl IntoResponse {
    proxy_post_body(state, "/api/ai/chat", body).await
}

// ============================================================================
// Validation API Proxy Routes
// ============================================================================

/// Run validation suite
pub async fn run_validation(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    proxy_post(state, "/api/validate").await
}

// ============================================================================
// Shared Proxy Helpers
// ============================================================================
// TAG: surface=simulation owner=research-team rule=SIM-001

async fn proxy_get(state: Arc<AppState>, path: &str) -> impl IntoResponse {
    match state
        .http_client
        .get(format!("{}{}", state.sim_service_url, path))
        .timeout(PROXY_TIMEOUT)
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            (
                StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                body,
            )
                .into_response()
        }
        Err(e) => {
            warn!("sim-service error: {e}");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                format!(r#"{{"error": "sim-service unavailable: {e}"}}"#),
            )
                .into_response()
        }
    }
}

async fn proxy_post(state: Arc<AppState>, path: &str) -> impl IntoResponse {
    match state
        .http_client
        .post(format!("{}{}", state.sim_service_url, path))
        .timeout(PROXY_TIMEOUT)
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            (
                StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                body,
            )
                .into_response()
        }
        Err(e) => {
            warn!("sim-service error: {e}");
            (
                // TAG: surface=simulation owner=research-team rule=SIM-001
                StatusCode::SERVICE_UNAVAILABLE,
                format!(r#"{{"error": "sim-service unavailable: {e}"}}"#),
            )
                .into_response()
        }
    }
}

async fn proxy_post_body(state: Arc<AppState>, path: &str, body: Body) -> impl IntoResponse {
    let body_bytes = match axum::body::to_bytes(body, MAX_BODY_SIZE).await {
        Ok(bytes) => bytes,
        Err(e) => {
            warn!("Failed to read request body: {e}");
            return (
                StatusCode::BAD_REQUEST,
                format!(r#"{{"error": "Invalid request body: {e}"}}"#),
            )
                .into_response();
        }
    };

    match state
        .http_client
        .post(format!("{}{}", state.sim_service_url, path))
        .header("Content-Type", "application/json")
        .body(body_bytes)
        .timeout(PROXY_TIMEOUT)
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            (
                StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                body,
            )
                .into_response()
        }
        Err(e) => {
            warn!("sim-service error: {e}");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                format!(r#"{{"error": "sim-service unavailable: {e}"}}"#),
            )
                .into_response()
        }
    }
    // TAG: surface=simulation owner=research-team rule=SIM-001
}

async fn proxy_post_empty(state: Arc<AppState>, path: &str) -> impl IntoResponse {
    match state
        .http_client
        .post(format!("{}{}", state.sim_service_url, path))
        .timeout(PROXY_TIMEOUT)
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            (
                StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                body,
            )
                .into_response()
        }
        Err(e) => {
            warn!("sim-service error: {e}");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                format!(r#"{{"error": "sim-service unavailable: {e}"}}"#),
            )
                .into_response()
        }
    }
}
