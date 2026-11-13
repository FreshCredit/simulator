#![allow(clippy::cast_precision_loss)]
#![allow(clippy::suboptimal_flops)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::manual_range_patterns)]
#![allow(clippy::unused_enumerate_index)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::excessive_precision)]
#![allow(clippy::manual_clamp)]
// TAG: surface=simulation owner=research-team rule=SIM-001
#![allow(clippy::cast_lossless)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::unused_self)]
#![allow(clippy::unused_async)]
#![allow(clippy::format_push_string)]
#![allow(clippy::significant_drop_tightening)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::similar_names)]
#![allow(clippy::unreadable_literal)]
//! # sim-service
//!
//! Job orchestration API for Agent Society Simulator.
//!
//! ## SIM-001 Compliance
//! This crate depends on `sim-artifacts` for schema types and `sim-core` for simulation.
//! It does NOT depend on any `FreshCredit` product crates (`freshcredit-*`).
//!
//! ## Architecture (§5.2)
//!
//! | Component | Responsibility |
//! |-----------|----------------|
//! | `apps/simulator` | UI - reads artifacts via this API |
//! | **`sim-service`** | **Job API, artifact serving** |
//! | `sim-runner` | Execution harness |
//! | `sim-core` | Deterministic kernel |
//!
//! ## Endpoints
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET | `/health` | Health check |
//! | GET | `/api/live` | SSE stream of live simulation |
//! | GET | `/api/runs` | List available simulation runs |
//! | GET | `/api/runs/:id/manifest` | Get manifest.json for a run |
//! | GET | `/api/runs/:id/replay/:tick` | Get replay frame for a tick |
//! | POST | `/api/jobs` | Submit new simulation job (deprecated) |
//! | GET | `/api/jobs/:id` | Get job status (deprecated) |

// TAG: surface=simulation owner=research-team rule=SIM-001
#![forbid(unsafe_code)]

pub mod ai;
pub mod api;
pub mod artifacts;
pub mod auth;
pub mod blockchain;
pub mod events;
pub mod executor;
pub mod health;
pub mod http_client;
pub mod jobs;
pub mod live;
pub mod memory;
pub mod meta_analysis;
pub mod orchestrator;
pub mod patterns;
pub mod persistence;
pub mod rate_limit;
pub mod replay;
pub mod router;
pub mod streaming;
pub mod sweep;
pub mod validation;
pub mod visualization;

pub use executor::ExecutorConfig;
pub use live::{spawn_simulation_loop, LiveSimulationState};
pub use router::create_router;
pub use sweep::{SweepRequest, SweepResponse, SweepState};

/// Crate version for API responses
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Application state shared across handlers
#[derive(Debug, Default)]
pub struct AppState {
    // ROADMAP: Add database connection, connection manager, etc.
    // For now, this is a placeholder for route handlers
}

/// Initialize enterprise compliance features
///
/// This function is called during service startup to ensure
/// all enterprise compliance requirements are satisfied:
/// - ENT-001: Audit logging
/// - ENT-002: RBAC permissions
/// - ENT-003: Multi-tenant isolation
/// - ENT-004: SSO support
///
/// # Panics
/// Panics if compliance initialization fails (this should never happen)
pub fn init_compliance() {
    // Initialize enterprise compliance features
    api::init_enterprise_compliance();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_exists() {
        // Verify version string is set by comparing to expected value
        assert_eq!(VERSION, "0.1.0");
    }
}
