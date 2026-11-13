// TAG: surface=simulation owner=research-team rule=SIM-001
#![allow(clippy::cast_possible_wrap)]
//! # sim-artifacts
//!
//! Simulation artifact schemas for manifest, metrics, and replay formats.
//!
//! ## SIM-001 Compliance
//! This crate has ZERO dependencies on `FreshCredit` product crates (`freshcredit-*`).
//! Only serialization and file format libraries are allowed.
//!
//! ## SIM-005 Schema Stability
//! All schemas are versioned. Breaking changes require major version increment.
//! Readers must support schema version N-1 for backward compatibility.
//!
//! ## Artifact Types
//!
//! | Artifact | Format | Purpose |
//! |----------|--------|---------|
//! | `manifest.json` | JSON | Run metadata, config, seeds, file references |
//! | `metrics.parquet` | Parquet | Per-tick aggregate metrics (columnar) |
//! | `replay/*.jsonl` | JSONL | Per-agent state for visualization replay |
//!
//! ## Usage
//!
//! ```rust
//! use sim_artifacts::{Manifest, TickMetrics, ReplayFrame};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a manifest
//! let manifest = Manifest::new(42, "abc123".to_string());
//!
//! // Serialize to JSON
//! let json = serde_json::to_string_pretty(&manifest)?;
//! # Ok(())
//! # }
//! ```

// TAG: surface=simulation owner=research-team rule=SIM-001

#![forbid(unsafe_code)]

pub mod manifest;
pub mod metrics;
pub mod replay;

// Re-export main types
pub use manifest::{DatasetVersions, Manifest, RunConfig, SeedConfig};
pub use metrics::{MetricsReader, MetricsWriter, TickMetrics};
pub use replay::{ReplayFrame, ReplayReader, ReplayWriter};

/// Schema version for all artifacts
/// SIM-005: Increment major version for breaking changes
pub const SCHEMA_VERSION: &str = "1.0";

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_version() {
        assert_eq!(SCHEMA_VERSION, "1.0");
    }

    #[test]
    fn test_version_exists() {
        // Verify version string is set by comparing to expected value
        assert_eq!(VERSION, "0.1.0");
    }
    // TAG: surface=simulation owner=research-team rule=SIM-001
}
