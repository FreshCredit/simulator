// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Macro Experiment Patterns
//!
//! High-level experiment patterns that compile down to concrete simulation batches:
//! - Parameter Sweeps: Grid/random search over parameter space
//! - A/B Tests: Control vs experiment with statistical comparison
//! - Factorial Designs: Full/fractional factorial experiments
//! - Scenario Tests: Compare scenarios against baselines
//! - Baseline Comparisons: Multi-variant baseline comparisons

pub mod executor;
pub mod templates;
pub mod types;

pub use executor::PatternExecutor;
pub use types::*;
