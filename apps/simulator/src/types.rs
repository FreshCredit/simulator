// TAG: surface=simulation owner=research-team rule=SIM-001
//! Simulator types and configuration

use askama::Template;
use clap::Parser;
use tokio::time::Duration;

// ARCH-001: Maximum request body size (10MB)
pub const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;
// PERF-001: Request timeout for proxy operations
pub const PROXY_TIMEOUT: Duration = Duration::from_secs(30);

// ============================================================================
// CLI Arguments
// ============================================================================

#[derive(Parser, Debug)]
#[command(name = "simulator")]
#[command(about = "Agent Society Simulator - Visualization Service")]
pub struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "8081", env = "PORT")]
    pub port: u16,

    /// sim-service API URL for artifact retrieval
    #[arg(long, default_value = "http://localhost:8082", env = "SIM_SERVICE_URL")]
    pub sim_service_url: String,

    /// Enable verbose logging
    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[arg(short, long)]
    pub verbose: bool,
}

// ============================================================================
// Application State
// ============================================================================

#[derive(Clone)]
pub struct AppState {
    pub sim_service_url: String,
    pub http_client: reqwest::Client,
}

// ============================================================================
// Templates
// ============================================================================

#[derive(Template)]
#[template(path = "simulator/index.html")]
pub struct IndexTemplate {
    pub sim_service_url: String,
}

#[derive(Template)]
#[template(path = "simulator/health.html")]
#[allow(dead_code)]
pub struct HealthTemplate {
    pub status: String,
    pub sim_service_connected: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_args_default_values() {
        let args = Args::parse_from(["simulator"]);
        assert_eq!(args.port, 8081);
        assert_eq!(args.sim_service_url, "http://localhost:8082");
        assert!(!args.verbose);
    }

    #[test]
    fn test_args_custom_port() {
        let args = Args::parse_from(["simulator", "--port", "9090"]);
        assert_eq!(args.port, 9090);
    }

    #[test]
    fn test_args_custom_sim_service_url() {
        let args = Args::parse_from(["simulator", "--sim-service-url", "http://sim:8080"]);
        assert_eq!(args.sim_service_url, "http://sim:8080");
    }

    #[test]
    fn test_args_verbose_short_flag() {
        let args = Args::parse_from(["simulator", "-v"]);
        assert!(args.verbose);
    }

    #[test]
    fn test_args_verbose_long_flag() {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let args = Args::parse_from(["simulator", "--verbose"]);
        assert!(args.verbose);
    }

    #[test]
    fn test_args_combined_short_options() {
        let args = Args::parse_from(["simulator", "-p", "3000", "-v"]);
        assert_eq!(args.port, 3000);
        assert!(args.verbose);
    }

    #[test]
    fn test_max_body_size_constant() {
        assert_eq!(MAX_BODY_SIZE, 10 * 1024 * 1024);
    }

    #[test]
    fn test_proxy_timeout_constant() {
        assert_eq!(PROXY_TIMEOUT, Duration::from_secs(30));
    }

    #[test]
    fn test_app_state_creation() {
        let state = AppState {
            sim_service_url: "http://test:8080".to_string(),
            http_client: reqwest::Client::new(),
        };
        assert_eq!(state.sim_service_url, "http://test:8080");
    }
    // TAG: surface=simulation owner=research-team rule=SIM-001
}
