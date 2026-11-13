// TAG: surface=simulation owner=research-team rule=SIM-001
//! Integration tests for `sim-service::api`
//!
//! Extracted from src/api.rs to reduce per-file CCN.
//! SIM-Service API Integration Tests
//!
//! CODE-009: EXCEPTION - Integration test for sim-service crate
//! B-005: API test coverage for production deployment
//!
//! Tests all REST API endpoints for the sim-service:
//! - Health check
//! - Artifact endpoints (runs, manifest, metrics, replay)
//! - Job management (submit, status, list)
//! - Parameter sweep
//! - Live simulation streaming

use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use std::path::PathBuf;
use tower::util::ServiceExt;

use sim_service::router::create_router;

// ============================================================================
// Health Check Tests
// ============================================================================

/// Test health check endpoint returns OK
#[tokio::test]
async fn test_health_check() {
    let app = create_router(PathBuf::from("./test_artifacts"));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "ok");
    assert_eq!(json["service"], "sim-service");
    assert!(!json["version"].as_str().unwrap().is_empty());
}

/// Test health check includes version
#[tokio::test]
async fn test_health_check_version_format() {
    let app = create_router(PathBuf::from("./test_artifacts"));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Version should be in semver format (e.g., "0.1.0")
    let version = json["version"].as_str().unwrap();
    assert!(version.contains('.'));
}

// ============================================================================
// Artifact API Tests
// ============================================================================

/// Test listing runs endpoint
#[tokio::test]
async fn test_list_runs_empty() {
    let temp_dir = tempfile::tempdir().unwrap();
    let app = create_router(temp_dir.path().to_path_buf());

    let response = app
        .oneshot(
            // TAG: surface=simulation owner=research-team rule=SIM-001
            Request::builder()
                .uri("/api/runs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return OK even if empty
    assert_eq!(response.status(), StatusCode::OK);
}

/// Test get manifest with invalid `run_id` returns 404
#[tokio::test]
async fn test_get_manifest_not_found() {
    let temp_dir = tempfile::tempdir().unwrap();
    let app = create_router(temp_dir.path().to_path_buf());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/runs/nonexistent/manifest")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// Test get metrics with invalid `run_id` returns 404
#[tokio::test]
async fn test_get_metrics_not_found() {
    let temp_dir = tempfile::tempdir().unwrap();
    let app = create_router(temp_dir.path().to_path_buf());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/runs/nonexistent/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // TAG: surface=simulation owner=research-team rule=SIM-001

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// Test get replay frame with invalid `run_id` returns 404
#[tokio::test]
async fn test_get_replay_frame_not_found() {
    let temp_dir = tempfile::tempdir().unwrap();
    let app = create_router(temp_dir.path().to_path_buf());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/runs/nonexistent/replay/100")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// Test artifact endpoints have CORS headers
#[tokio::test]
async fn test_cors_headers_present() {
    let temp_dir = tempfile::tempdir().unwrap();
    let app = create_router(temp_dir.path().to_path_buf());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/runs")
                .header("Origin", "https://app.freshcredit.io")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // CORS middleware should add access-control-allow-origin header for allowed origins
    assert!(response
        .headers()
        .contains_key("access-control-allow-origin"));
}

// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
// Job API Tests
// ============================================================================

/// Test listing jobs endpoint
#[tokio::test]
async fn test_list_jobs() {
    let temp_dir = tempfile::tempdir().unwrap();
    let app = create_router(temp_dir.path().to_path_buf());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/jobs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Should return an object with "jobs" array and "count"
    assert!(json.as_object().is_some());
    assert!(json.get("jobs").is_some());
    assert!(json.get("count").is_some());
}

/// Test submitting a job with invalid config returns error
#[tokio::test]
async fn test_submit_job_invalid_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let app = create_router(temp_dir.path().to_path_buf());

    let invalid_config = serde_json::json!({
        "agent_count": 0,  // Invalid: must be > 0
        "max_ticks": 100,
        "stage": 1,
        "seed": 42
    });

    let response = app
        .oneshot(
            // TAG: surface=simulation owner=research-team rule=SIM-001
            Request::builder()
                .method("POST")
                .uri("/api/jobs")
                .header("content-type", "application/json")
                .body(Body::from(invalid_config.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return bad request for invalid config (agent_count == 0)
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// Test getting job status for non-existent job returns 404
#[tokio::test]
async fn test_get_job_status_not_found() {
    let temp_dir = tempfile::tempdir().unwrap();
    let app = create_router(temp_dir.path().to_path_buf());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/jobs/nonexistent-job-id")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Parameter Presets Tests
// ============================================================================

/// Test listing parameter presets
#[tokio::test]
async fn test_list_presets() {
    let temp_dir = tempfile::tempdir().unwrap();
    let app = create_router(temp_dir.path().to_path_buf());

    let response = app
        .oneshot(
            Request::builder()
                // TAG: surface=simulation owner=research-team rule=SIM-001
                .uri("/api/presets")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Should return an object with "presets" array and "count"
    assert!(json.as_object().is_some());
    assert!(json.get("presets").is_some());
    assert!(json.get("count").is_some());
}

// ============================================================================
// Parameter Sweep Tests
// ============================================================================

/// Test submitting a parameter sweep with invalid parameters (no seeds)
#[tokio::test]
async fn test_submit_sweep_invalid() {
    let temp_dir = tempfile::tempdir().unwrap();
    let app = create_router(temp_dir.path().to_path_buf());

    // Valid structure but empty seeds (triggers validation error)
    let invalid_sweep = serde_json::json!({
        "base_config": {
            "agent_count": 100,
            "max_ticks": 1000,
            "stage": 1
        },
        "sweep_params": {
            "seeds": []  // Empty seeds - should fail validation
        }
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/sweep")
                // TAG: surface=simulation owner=research-team rule=SIM-001
                .header("content-type", "application/json")
                .body(Body::from(invalid_sweep.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return bad request for invalid sweep config (no runs generated)
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Live Simulation Tests
// ============================================================================

/// Test live simulation endpoint exists
#[tokio::test]
async fn test_live_stream_endpoint() {
    let temp_dir = tempfile::tempdir().unwrap();
    let app = create_router(temp_dir.path().to_path_buf());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/live")
                .header("Accept", "text/event-stream")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should accept the connection (returns OK or streaming response)
    assert!(response.status().is_success() || response.status() == StatusCode::OK);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

/// Test 404 for unknown endpoints
#[tokio::test]
async fn test_unknown_endpoint_404() {
    let temp_dir = tempfile::tempdir().unwrap();
    let app = create_router(temp_dir.path().to_path_buf());

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/unknown-endpoint")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// Test method not allowed
#[tokio::test]
async fn test_method_not_allowed() {
    let temp_dir = tempfile::tempdir().unwrap();
    let app = create_router(temp_dir.path().to_path_buf());

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

// ============================================================================
// Content Type Tests
// ============================================================================

/// Test API returns JSON content type
#[tokio::test]
async fn test_json_content_type() {
    let temp_dir = tempfile::tempdir().unwrap();
    let app = create_router(temp_dir.path().to_path_buf());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                // TAG: surface=simulation owner=research-team rule=SIM-001
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    assert!(content_type.contains("application/json"));
}

// ============================================================================
// Smoke Test
// ============================================================================

/// Quick smoke test to verify all major endpoints are accessible
#[tokio::test]
async fn test_api_smoke() {
    let temp_dir = tempfile::tempdir().unwrap();
    let app = create_router(temp_dir.path().to_path_buf());

    // Test all major GET endpoints
    let endpoints = vec!["/health", "/api/runs", "/api/jobs", "/api/presets"];

    for endpoint in endpoints {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(endpoint)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(
            response.status().is_success(),
            "Endpoint {} failed with status {:?}",
            endpoint,
            response.status()
        );
    }
}
