// TAG: surface=simulation owner=research-team rule=SIM-001
//! Agent Society Simulator - Standalone Service
//!
//! SIM-001: This service has ZERO dependencies on sim-core or product crates.
//! SIM-002: UI reads artifacts only - no simulation engine logic.
//!
//! Architecture:
//! - Standalone Axum server (like apps/blockchain)
//! - Serves static files (HTML, CSS, JS)
//! - Fetches artifacts from sim-service API or GCS
//! - PUBLIC ACCESS: No authentication required (research/academic tool)
//! - Deployed to separate Cloud Run service
//!
//! Note: Resource limits for job submission are enforced by sim-service backend,
//! not by this public UI service.

#![forbid(unsafe_code)]

use axum::{
    extract::DefaultBodyLimit,
    http::{header::HeaderName, HeaderValue},
    routing::{get, post},
    Router,
};
use clap::Parser;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tracing::info;

mod handlers;
mod types;

use types::{AppState, Args};
// TAG: surface=simulation owner=research-team rule=SIM-001

/// CORS layer configuration - environment-aware
/// P1-SECURITY: Restricted origins to prevent CSRF attacks
fn cors_layer() -> CorsLayer {
    use tower_http::cors::AllowOrigin;
    use tower_http::cors::AllowPrivateNetwork;

    // Allow specific origins based on environment
    // P1-SECURITY: Never use AllowOrigin::any() - even in development
    let is_prod = std::env::var("ENVIRONMENT")
        .map(|v| v == "production")
        .unwrap_or(false)
        || std::env::var("PRODUCTION")
            .map(|v| v == "1" || v == "true")
            .unwrap_or(false);

    let allowed_origins = if is_prod {
        AllowOrigin::list([
            HeaderValue::from_static("https://freshcredit.com"),
            HeaderValue::from_static("https://app.freshcredit.com"),
            HeaderValue::from_static("https://testnet.freshcredit.com"),
        ])
    } else {
        // Development: restrict to known local origins
        AllowOrigin::list([
            HeaderValue::from_static("http://localhost:3000"),
            HeaderValue::from_static("http://localhost:3001"),
            HeaderValue::from_static("http://localhost:5173"),
            HeaderValue::from_static("http://127.0.0.1:3000"),
            HeaderValue::from_static("http://127.0.0.1:3001"),
            HeaderValue::from_static("http://127.0.0.1:5173"),
        ])
    };

    CorsLayer::new()
            // TAG: surface=simulation owner=research-team rule=SIM-001
        .allow_origin(allowed_origins)
        .allow_methods([
            http::Method::GET,
            http::Method::POST,
            http::Method::PUT,
            http::Method::DELETE,
        ])
        .allow_headers([
            http::header::CONTENT_TYPE,
            http::header::AUTHORIZATION,
            http::header::ACCEPT,
        ])
        .allow_private_network(AllowPrivateNetwork::yes())
}

// ============================================================================
// Router Setup
// ============================================================================

fn create_router(state: Arc<AppState>) -> Router {
    // CORS layer for API endpoints
    let cors = cors_layer();

    let router = Router::new()
        // ARCH-001: Enforce body size limit globally
        .layer(DefaultBodyLimit::max(types::MAX_BODY_SIZE))
        // Main page
        .route("/", get(handlers::index))
        // Health check
        .route("/health", get(handlers::health))
        // API proxy routes (forward to sim-service)
        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Live simulation stream and lifecycle control routes (CRITICAL)
        .route("/api/live", get(handlers::live_stream))
        .route("/api/live/restart", post(handlers::restart_simulation))
        .route("/api/live/intervention", post(handlers::live_intervention))

        .route("/api/live/pause", post(handlers::pause_simulation))
        .route("/api/live/resume", post(handlers::resume_simulation))
        .route("/api/live/stop", post(handlers::stop_simulation))
        .route("/api/live/status", get(handlers::get_simulation_status))
        .route("/api/live/speed", post(handlers::set_speed))
        .route("/api/live/speed", get(handlers::get_speed))
        .route("/api/live/state/export", get(handlers::export_simulation_state))
        .route("/api/live/state/import", post(handlers::import_simulation_state))
        .route("/api/live/report", get(handlers::get_simulation_report))
        .route("/api/ai/stream", get(handlers::ai_stream))
        .route("/api/ai/chat", post(handlers::ai_chat))
        // Artifact routes
        .route("/api/runs", get(handlers::list_runs))
        .route("/api/runs/:run_id/manifest", get(handlers::get_manifest))
        .route("/api/runs/:run_id/replay/:tick", get(handlers::get_replay_frame))
        // Neural data routes
        .route("/api/neural/agent/:agent_id", get(handlers::get_neural_agent))
        .route("/api/neural/population", get(handlers::get_neural_population))
        // PDE4B configuration routes
        .route("/api/config/pde4b", get(handlers::get_pde4b_config).post(handlers::update_pde4b_config))
        .route("/api/stats/lambda", get(handlers::get_lambda_stats))
        // Experiment routes
        .route("/api/experiments", get(handlers::list_experiments))
        .route("/api/experiments/:exp_id", get(handlers::get_experiment))
        .route("/api/experiments/:exp_id/schema", get(handlers::get_experiment_schema))
        .route("/api/experiments/:exp_id/runs", post(handlers::launch_experiment))
        .route("/api/experiment-runs/:run_id", get(handlers::get_experiment_run_status))
        .route("/api/experiment-runs/:run_id/stream", get(handlers::stream_experiment_progress))
        // Validation routes
        .route("/api/validate", post(handlers::run_validation));

    let static_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static");
    let static_service = if static_dir.is_dir() {
        ServeDir::new(static_dir)
    } else {
        ServeDir::new("static")
    };

    router
        // Static files (CSS, JS)
        .nest_service("/static", static_service)
        .layer(cors)
        // P1-SECURITY: Allow this service to be embedded as a credentialless
        // cross-origin iframe from the staging/production web app.
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("cross-origin-resource-policy"),
            HeaderValue::from_static("cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("content-security-policy"),
            HeaderValue::from_static(
                "default-src 'self'; \
                 script-src 'self' 'wasm-unsafe-eval' https://cdn.jsdelivr.net; \
                 style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; \
                 font-src 'self' https://fonts.gstatic.com; \
                 img-src 'self' data: https: blob:; \
                 connect-src 'self' http: https: ws: wss:; \
                 frame-ancestors 'self' https://freshcredit.com https://app.freshcredit.com http://localhost:*;",
            ),
        ))
        .with_state(state)
}

// ============================================================================
// Main Entry Point
// TAG: surface=simulation owner=research-team rule=SIM-001
// ============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("simulator={log_level},tower_http=info"))
        .init();

    info!("Starting Agent Society Simulator");
    info!("Port: {}", args.port);
    info!("sim-service URL: {}", args.sim_service_url);

    // Create resilient HTTP client with bounded connect and request timeouts
    let http_client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    // Create application state
    let state = Arc::new(AppState {
        sim_service_url: args.sim_service_url,
        http_client,
    });

    // Create router with bounded 60s server-side request timeout
    let app = create_router(state).layer(tower_http::timeout::TimeoutLayer::new(
        std::time::Duration::from_secs(60),
    ));

    // Start server
    let addr = format!("0.0.0.0:{}", args.port);
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on http://{}", addr);

    axum::serve(listener, app).await?;
    // TAG: surface=simulation owner=research-team rule=SIM-001

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex, MutexGuard};

    /// Serializes tests that mutate the process-wide `PRODUCTION` env var
    /// (cargo test runs tests on multiple threads by default).
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn env_lock() -> MutexGuard<'static, ()> {
        ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    #[test]
    fn test_cors_layer_development() {
        let _guard = env_lock();
        std::env::remove_var("PRODUCTION");
        let _layer = cors_layer();
    }

    #[test]
    fn test_cors_layer_production() {
        let _guard = env_lock();
        std::env::set_var("PRODUCTION", "1");
        let _layer = cors_layer();
        std::env::remove_var("PRODUCTION");
    }

    #[test]
    fn test_create_router() {
        let state = Arc::new(AppState {
            sim_service_url: "http://localhost:8082".to_string(),
            http_client: reqwest::Client::new(),
        });
        let router = create_router(state);
        let _ = router;
    }
    // TAG: surface=simulation owner=research-team rule=SIM-001
}
