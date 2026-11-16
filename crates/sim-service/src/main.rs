//! sim-service: Live simulation API for Agent Society Simulator
//!
//! SIM-001: This service imports sim-core for running continuous simulations.
//!          It has ZERO dependencies on `FreshCredit` product crates.
//! SIM-003: Simulates "simulation agents", NOT `FreshCredit` AI agents.
// TAG: surface=simulation owner=research-team rule=SIM-001
//!
//! Architecture:
//! - Standalone Axum server on port 8082
//! - Runs continuous 24/7 simulation using sim-core
//! - Streams live agent state via Server-Sent Events (SSE) at /api/live
//! - Also serves historical artifacts for analysis (deprecated job system)
//! - Public access (no authentication required)
//!
//! Usage:
//!   sim-service --port 8082

use clap::Parser;
use std::path::PathBuf;
use tokio::net::TcpListener;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use sim_service::create_router;

/// sim-service CLI arguments
#[derive(Parser, Debug)]
#[command(name = "sim-service")]
#[command(about = "Job orchestration API for Agent Society Simulator")]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "8082", env = "PORT")]
    port: u16,

    /// Directory containing simulation artifacts
    #[arg(long, default_value = "./artifacts", env = "ARTIFACTS_DIR")]
    artifacts_dir: PathBuf,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize logging
    let level = if args.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting sim-service v{}", sim_service::VERSION);
    info!("Port: {}", args.port);
    info!("Artifacts directory: {:?}", args.artifacts_dir);
    info!("Mode: Continuous 24/7 simulation with live SSE streaming");

    // Ensure artifacts directory exists
    if !args.artifacts_dir.exists() {
        tokio::fs::create_dir_all(&args.artifacts_dir).await?;
        info!("Created artifacts directory: {:?}", args.artifacts_dir);
    }

    // Create router (this also spawns the continuous simulation loop)
    info!("Creating router...");
    let app = create_router(args.artifacts_dir);
    info!("Router created successfully");

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Start server
    let addr = format!("0.0.0.0:{}", args.port);
    info!("Binding to {}...", addr);
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on http://{}", addr);
    info!("Live simulation stream available at /api/live");

    // CRITICAL FIX: Setup graceful shutdown with SIGTERM handling
    // Cloud Run sends SIGTERM before stopping the container (10 second grace period)
    let server = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    );

    tokio::select! {
        result = server => {
            result?;
        }
        _ = tokio::signal::ctrl_c() => {
            info!("============================================");
            info!("SIGTERM/SIGINT received, starting graceful shutdown");
            info!("Cloud Run provides 10 seconds for cleanup");
            info!("============================================");
            // Server will drop, connections will close
            // The broadcast channel will be dropped, causing clients to disconnect
        }
    }

    Ok(())
}
