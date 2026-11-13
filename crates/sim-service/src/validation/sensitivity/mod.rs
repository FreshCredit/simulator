// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Sensitivity Analysis (P1-VAL-2)
//!
//! Tests parameter robustness and identifies which inputs most affect outputs.
//! Supports multiple methods: local sensitivity, Morris screening, Sobol indices.

pub mod local;
pub mod morris;
pub mod sobol;
pub mod sobol_sampling;
pub mod tornado;
pub mod types;

pub use local::LocalSensitivityAnalyzer;
pub use morris::MorrisAnalyzer;
pub use sobol::SobolAnalyzer;
pub use tornado::TornadoDiagram;
pub use types::*;
