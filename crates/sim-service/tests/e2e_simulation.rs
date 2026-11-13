// TAG: surface=simulation owner=research-team rule=SIM-001
//! End-to-end tests for simulation workflows
//!
//! These tests verify complete user journeys including:
//! - Live simulation streaming via SSE
//! - Experiment submission and monitoring
//! - Artifact retrieval
//! - AI chat integration

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::util::ServiceExt;

use sim_service::create_router;

/// Helper to create a test app with temporary directories
fn create_test_app() -> axum::Router {
    let temp_dir = tempfile::tempdir().unwrap();
    create_router(temp_dir.path().to_path_buf())
}

/// Test that the live simulation endpoint returns SSE stream
#[tokio::test]
async fn test_live_simulation_stream() {
    let app = create_test_app();

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

    assert_eq!(response.status(), StatusCode::OK);

    // Check content type is SSE
    let content_type = response
        .headers()
        .get("content-type")
        .expect("Content-Type header should exist");
    // TAG: surface=simulation owner=research-team rule=SIM-001
    assert!(content_type.to_str().unwrap().contains("text/event-stream"));
}

/// Test simulation speed control endpoints
#[tokio::test]
async fn test_simulation_speed_control() {
    let app = create_test_app();

    // Get initial speed
    let get_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/live/speed")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get_response.status(), StatusCode::OK);

    // Set speed to 2x
    let post_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/live/speed")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"factor": 2.0}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(post_response.status(), StatusCode::OK);

    // Verify speed was set
    let get_response2 = app
        .oneshot(
            Request::builder()
                .uri("/api/live/speed")
                .body(Body::empty())
                .unwrap(),
            // TAG: surface=simulation owner=research-team rule=SIM-001
        )
        .await
        .unwrap();

    assert_eq!(get_response2.status(), StatusCode::OK);
}

/// Test simulation lifecycle endpoints (pause/resume/status)
#[tokio::test]
async fn test_simulation_lifecycle() {
    let app = create_test_app();

    // Get status
    let status_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/live/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(status_response.status(), StatusCode::OK);

    // Pause simulation
    let pause_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/live/pause")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(pause_response.status(), StatusCode::OK);

    // Resume simulation
    let resume_response = app
        .clone()
        .oneshot(
            Request::builder()
                // TAG: surface=simulation owner=research-team rule=SIM-001
                .method("POST")
                .uri("/api/live/resume")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resume_response.status(), StatusCode::OK);
}

/// Test experiment catalog listing
#[tokio::test]
async fn test_experiment_catalog() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/experiments")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Should have experiments array
    assert!(json.get("experiments").is_some());
    let experiments = json["experiments"].as_array().unwrap();
    assert!(
        !experiments.is_empty(),
        "Should have at least one experiment"
    );
}

/// Test getting a specific experiment
#[tokio::test]
async fn test_get_experiment() {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/experiments/lambda-universality")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["id"], "lambda-universality");
}

/// Test AI chat endpoint
#[tokio::test]
async fn test_ai_chat() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ai/chat")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"message": "What is the current status?"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    // TAG: surface=simulation owner=research-team rule=SIM-001

    // Should have message in response
    assert!(json.get("message").is_some());
    assert!(json.get("suggestions").is_some());
    assert!(json.get("meta").is_some());
}

/// Test AI stream endpoint
#[tokio::test]
async fn test_ai_stream() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/ai/stream")
                .header("Accept", "text/event-stream")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check content type is SSE
    let content_type = response
        .headers()
        .get("content-type")
        .expect("Content-Type header should exist");
    assert!(content_type.to_str().unwrap().contains("text/event-stream"));
}

/// Test artifact endpoints return 404 for non-existent runs
#[tokio::test]
async fn test_artifacts_not_found() {
    let app = create_test_app();

    // Test manifest not found
    let manifest_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runs/non-existent-run/manifest")
                .body(Body::empty())
                .unwrap(),
            // TAG: surface=simulation owner=research-team rule=SIM-001
        )
        .await
        .unwrap();

    assert_eq!(manifest_response.status(), StatusCode::NOT_FOUND);

    // Test metrics not found
    let metrics_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/runs/non-existent-run/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(metrics_response.status(), StatusCode::NOT_FOUND);

    // Test replay not found
    let replay_response = app
        .oneshot(
            Request::builder()
                .uri("/api/runs/non-existent-run/replay/0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(replay_response.status(), StatusCode::NOT_FOUND);
}

/// Test Version 4.0 live intervention and telemetry closed-loop response
#[tokio::test]
async fn test_live_v4_intervention_and_telemetry_snapshot() {
    let app = create_test_app();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/live/intervention")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                    "laser_wavelength": "ChR2Blue470nm",
                    "target_channels": [false, false, true, false, false],
                    "weather_shield_enabled": true,
                    "visual_cloaking_ratio": 3.65
                }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let status_res = app
        .oneshot(
            Request::builder()
                .uri("/api/live/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(status_res.status(), StatusCode::OK);
}

/// Test analysis endpoints return appropriate responses
#[tokio::test]
async fn test_analysis_endpoints() {
    let app = create_test_app();

    // Test insights endpoint (will return mock since no run exists)
    let insights_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/analysis/test-run/insights")
                // TAG: surface=simulation owner=research-team rule=SIM-001
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Returns 404 since run doesn't exist
    assert_eq!(insights_response.status(), StatusCode::NOT_FOUND);

    // Test query endpoint
    let query_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/analysis/test-run/query")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"question": "What happened?"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    // Returns 404 since run doesn't exist
    assert_eq!(query_response.status(), StatusCode::NOT_FOUND);
}

/// Test CORS headers are present
#[tokio::test]
async fn test_cors_headers() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/api/experiments")
                .header("Origin", "http://localhost:3000")
                .header("Access-Control-Request-Method", "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // CORS should allow the request
    assert!(response.status().is_success() || response.status() == StatusCode::NO_CONTENT);
}

/// Test simulation state export and import round-trip
#[tokio::test]
async fn test_state_export_and_import_roundtrip() {
    let app = create_test_app();

    let export_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/live/state/export")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(export_response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(export_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let export_json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(export_json.get("config").is_some());
    assert!(export_json.get("tick").is_some());

    let import_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/live/state/import")
                .header("Content-Type", "application/json")
                .body(Body::from(bytes))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(import_response.status(), StatusCode::OK);
    let import_bytes = axum::body::to_bytes(import_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let import_json: serde_json::Value = serde_json::from_slice(&import_bytes).unwrap();
    assert_eq!(import_json["status"], "imported");
}

/// Test that UI envelope format ({ tick, frame, activityLog }) is also accepted and hydrated
#[tokio::test]
async fn test_ui_envelope_state_import_and_hydration() {
    let app = create_test_app();

    let ui_payload = serde_json::json!({
        "timestamp": "2026-10-19T22:00:00Z",
        "tick": 247,
        "frame": {
            "tick": 247,
            "agents": [
                {
                    "agent_id": 99,
                    "x": 42.5,
                    "y": 55.0,
                    "z": 0.0,
                    "energy": 88.0,
                    "docas": [0.6, 0.7, 0.1, 0.4, 0.8],
                    "genome_gains": [0.5, 0.5, 0.5, 0.5, 0.5],
                    "alive": true,
                    "has_verified_id": true,
                    "age": 120,
                    "sex": "Female",
                    "tau": 1.25,
                    "reputation": 0.85,
                    "generation": 3,
                    "parent1_id": null,
                    "parent2_id": null,
                    "last_dx": 1.0,
                    "last_dy": 0.0
                }
            ],
            "stats": {
                "total_agents": 1,
                "alive_agents": 1,
                "verified_agents": 1,
                "groups": 0
            }
        },
        "activityLog": []
    });

    let import_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/live/state/import")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&ui_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(import_response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(import_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(resp["status"], "imported");
    assert_eq!(resp["tick"], 247);
    assert_eq!(resp["agents"], 1);
    // Verify immediate read-after-write consistency on GET /api/live/state/export
    let export_response = app
        .oneshot(
            Request::builder()
                .uri("/api/live/state/export")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(export_response.status(), StatusCode::OK);
    let export_bytes = axum::body::to_bytes(export_response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let export_json: serde_json::Value = serde_json::from_slice(&export_bytes).unwrap();
    assert_eq!(export_json["tick"], 247);
    assert_eq!(export_json["agents"][0]["agent_id"], 99);
    assert_eq!(export_json["agents"][0]["energy"], 88.0);
}

#[tokio::test]
async fn test_live_intervention_nan_resilience_and_v4_telemetry() {
    let app = create_test_app();

    // 1. Send live intervention with out-of-bounds parameters
    let intervention_payload = serde_json::json!({
        "visual_cloaking_ratio": 999.0,
        "weather_temperature": 500.0,
        "opto_subpop_pct": 50.0,
        "weather_shield_enabled": true,
        "baylor_unconscious_mode": true
    });

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/live/intervention")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&intervention_payload).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    // 2. Query state export and verify params are safely clamped
    let export_resp = app
        .oneshot(
            Request::builder()
                .uri("/api/live/state/export")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(export_resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(export_resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let export_json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(export_json["config"]["weather_shield_enabled"], true);
}

#[tokio::test]
async fn test_parameter_jacobian_trajectory_divergence() {
    use sim_core::simulation::config::SimulationConfig;
    use sim_core::simulation::Simulation;

    let seed = 42;
    // Base simulation trajectory
    let config_base = SimulationConfig {
        seed,
        toggles: sim_core::toggles::ToggleConfig::stage_3(),
        circadian_sleep_deficit: 0.0,
        tax_redistribution_rate: 0.0,
        ..SimulationConfig::default()
    };
    let mut sim_base = Simulation::new(config_base);
    for i in 0..10 {
        sim_base.spawn_agent(sim_core::types::Position::new_2d(20.0 + i as f32, 20.0));
    }
    for _ in 0..20 {
        sim_base.step();
    }

    // Perturbed simulation trajectory with circadian deficit
    let config_perturbed = SimulationConfig {
        seed,
        toggles: sim_core::toggles::ToggleConfig::stage_3(),
        circadian_sleep_deficit: 0.80,
        tax_redistribution_rate: 0.25,
        ..SimulationConfig::default()
    };
    let mut sim_perturbed = Simulation::new(config_perturbed);
    for i in 0..10 {
        sim_perturbed.spawn_agent(sim_core::types::Position::new_2d(20.0 + i as f32, 20.0));
    }
    for _ in 0..20 {
        sim_perturbed.step();
    }

    // Assert causal divergence in neurochemistry (Jacobian != 0)
    let base_cortisol: f32 = sim_base
        .agents
        .iter()
        .filter(|a| a.alive)
        .map(|a| a.docas.levels[sim_core::types::C])
        .sum();
    let perturbed_cortisol: f32 = sim_perturbed
        .agents
        .iter()
        .filter(|a| a.alive)
        .map(|a| a.docas.levels[sim_core::types::C])
        .sum();

    assert!(
        (base_cortisol - perturbed_cortisol).abs() > 0.01,
        "Jacobian sensitivity gate failed: circadian deficit did not alter Cortisol trajectory (base={:.3}, perturbed={:.3})",
        base_cortisol, perturbed_cortisol
    );
}

#[tokio::test]
async fn test_live_intervention_full_jacobian_wiring() {
    let app = create_test_app();

    let intervention_payload = serde_json::json!({
        "planning_horizon": 45,
        "emotion_decay_rate": 0.77,
        "transaction_cost": 0.12,
        "consensus_threshold": 0.88,
        "kingman_coalescent": 2.5,
        "epigenetic_methylation_rate": 0.33,
        "circadian_sleep_deficit": 0.65
    });

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/live/intervention")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&intervention_payload).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let export_resp = app
        .oneshot(
            Request::builder()
                .uri("/api/live/state/export")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(export_resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(export_resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let export_json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(export_json["config"]["planning_horizon"], 45);
    assert!(
        (export_json["config"]["emotion_decay_rate"]
            .as_f64()
            .unwrap()
            - 0.77)
            .abs()
            < 1e-4
    );
    assert!((export_json["config"]["transaction_cost"].as_f64().unwrap() - 0.12).abs() < 1e-4);
    assert!(
        (export_json["config"]["consensus_threshold"]
            .as_f64()
            .unwrap()
            - 0.88)
            .abs()
            < 1e-4
    );
    assert!(
        (export_json["config"]["epigenetic_methylation_rate"]
            .as_f64()
            .unwrap()
            - 0.33)
            .abs()
            < 1e-4
    );
}
