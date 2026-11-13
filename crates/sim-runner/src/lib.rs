// TAG: surface=simulation owner=research-team rule=SIM-001
//! sim-runner: Batch execution harness for Agent Society Simulator
//!
//! This crate provides:
//! - Config parsing and validation
//! - Batch execution orchestration
//! - Layer 2 statistics (aggregation, CIs, effect sizes)
//! - Parameter sweep execution
//!
//! # SIM-001 Compliance
//!
//! This crate only depends on `sim-core` and `sim-artifacts`.
//! It does NOT depend on any `FreshCredit` product crates (`freshcredit-*`).
//!
//! # Architecture (§5.3.1)
//!
//! | Layer | Component | Responsibility |
//! |-------|-----------|----------------|
//! | Layer 1 | `sim-core` | Per-tick metrics |
//! | **Layer 2** | **`sim-runner`** | **Aggregation, POM validation, CIs** |
//! | Layer 3 | `analysis/` | Post-hoc analysis |
//!
//! # Example
//!
//! ```no_run
//! use sim_runner::{RunConfig, BatchRunner};
//!
//! let config = RunConfig::default();
//! let runner = BatchRunner::new(config);
//! let results = runner.run(5)?; // Run 5 replications
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![allow(clippy::cast_precision_loss)]
#![allow(clippy::suboptimal_flops)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::unused_self)]
#![allow(clippy::format_push_string)]
#![allow(clippy::significant_drop_tightening)]
#![allow(clippy::match_same_arms)]
// TAG: surface=simulation owner=research-team rule=SIM-001
#![allow(clippy::option_if_let_else)]
#![allow(clippy::similar_names)]
#![allow(clippy::unreadable_literal)]
#![forbid(unsafe_code)]

pub mod batch;
pub mod comparison;
pub mod config;
pub mod experiment;
pub mod hypothesis;
pub mod stats;
pub mod sweep;
pub mod visualization;

pub use batch::{BatchResult, BatchRunner};
pub use comparison::{
    quick_comparison, ComparisonMetrics, ComparisonResult, ComparisonRunner, ModeResult,
};
pub use config::RunConfig;
pub use experiment::{
    run_experiment, AcceptanceCriterion, Comparison, ExperimentConfig, ExperimentError,
    ExperimentResult, ExperimentRunner, ExperimentSummary, ExperimentType, ReplicationResult,
    ValidationResult,
};
pub use hypothesis::{quick_gtt_test, ConditionResults, GttHypothesisResult, GttHypothesisRunner};
pub use stats::*;
pub use sweep::{
    run_grid_sweep, GridPoint, GridSweepConfig, GridSweepError, GridSweepResult, GridSweepRunner,
    HeatmapData, OptimalRegion,
};

/// Crate version for manifest compatibility
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_exists() {
        // Verify version string is set by comparing to expected value
        assert_eq!(VERSION, "0.1.0");
    }
    // TAG: surface=simulation owner=research-team rule=SIM-001
}
