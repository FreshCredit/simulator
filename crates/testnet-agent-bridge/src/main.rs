// TAG: surface=blockchain owner=blockchain-team rule=BC-001
//! Testnet Agent Bridge
//!
//! Translates simulation agent actions into Substrate extrinsics
//! for the `FreshCredit` testnet.

#![forbid(unsafe_code)]

mod bridge;
mod cors;
mod handlers;
mod metrics;
mod types;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::bridge::AgentBridge;
use crate::cors::cors_layer;
use crate::handlers::{
    anchor_handler, collect_metrics_handler, compare_agents_handler, get_analysis_handler,
    health_handler, verify_handler,
};

/// Get service seed from environment
/// SECURITY: Returns error in production if not set (CRITICAL-001)
fn get_service_seed() -> anyhow::Result<String> {
    if let Ok(seed) = std::env::var("SERVICE_ACCOUNT_SEED") {
        Ok(seed)
    } else {
        // Check if we're in production
        // TAG: surface=blockchain owner=blockchain-team rule=BC-001
        if let Ok(env) = std::env::var("ENVIRONMENT") {
            if env == "production" {
                return Err(anyhow::anyhow!(
                    "SERVICE_ACCOUNT_SEED must be set in production environment"
                ));
            }
        }

        // Development fallback with warning
        tracing::warn!("SECURITY: Using default //Alice seed - NEVER use in production!");
        tracing::warn!("Set SERVICE_ACCOUNT_SEED environment variable");
        Ok("//Alice".to_string())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Get configuration from environment
    let testnet_ws_url =
        std::env::var("TESTNET_WS_URL").unwrap_or_else(|_| "ws://localhost:9944".to_string());
    let service_seed = get_service_seed()?;
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());

    info!("Starting Testnet Agent Bridge");
    info!("Connecting to testnet at {}", testnet_ws_url);

    // Create bridge
    let bridge = Arc::new(RwLock::new(AgentBridge::new(testnet_ws_url, service_seed)));

    // Build router
    let app = Router::new()
        .route("/health", get(health_handler))
        // TAG: surface=blockchain owner=blockchain-team rule=BC-001
        .route("/anchor", post(anchor_handler))
        .route("/verify/:hash", get(verify_handler))
        .route("/metrics/collect", post(collect_metrics_handler))
        .route("/metrics/compare", post(compare_agents_handler))
        .route("/metrics/analysis", get(get_analysis_handler))
        .layer(cors_layer())
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(bridge);

    // Start server
    let addr: SocketAddr = bind_addr.parse()?;
    info!("Agent bridge listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Environment variables are process-global; serialize env-mutating tests.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_get_service_seed_with_env() {
        let _guard = ENV_LOCK.lock().unwrap();
        // Save current state
        let previous = std::env::var("SERVICE_ACCOUNT_SEED").ok();
        std::env::set_var("SERVICE_ACCOUNT_SEED", "//Bob");
        // Remove ENVIRONMENT if set to avoid production branch
        let prev_env = std::env::var("ENVIRONMENT").ok();
        std::env::remove_var("ENVIRONMENT");

        let seed = get_service_seed().unwrap();
        assert_eq!(seed, "//Bob");
        // TAG: surface=blockchain owner=blockchain-team rule=BC-001

        // Restore
        match previous {
            Some(v) => std::env::set_var("SERVICE_ACCOUNT_SEED", v),
            None => std::env::remove_var("SERVICE_ACCOUNT_SEED"),
        }
        match prev_env {
            Some(v) => std::env::set_var("ENVIRONMENT", v),
            None => std::env::remove_var("ENVIRONMENT"),
        }
    }

    #[test]
    fn test_get_service_seed_fallback() {
        let _guard = ENV_LOCK.lock().unwrap();
        // Save current state
        let previous = std::env::var("SERVICE_ACCOUNT_SEED").ok();
        let prev_env = std::env::var("ENVIRONMENT").ok();

        std::env::remove_var("SERVICE_ACCOUNT_SEED");
        std::env::remove_var("ENVIRONMENT");

        let seed = get_service_seed().unwrap();
        assert_eq!(seed, "//Alice");

        // Restore
        match previous {
            Some(v) => std::env::set_var("SERVICE_ACCOUNT_SEED", v),
            None => std::env::remove_var("SERVICE_ACCOUNT_SEED"),
        }
        match prev_env {
            Some(v) => std::env::set_var("ENVIRONMENT", v),
            None => std::env::remove_var("ENVIRONMENT"),
        }
    }
    // TAG: surface=blockchain owner=blockchain-team rule=BC-001
}
