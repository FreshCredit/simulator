// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Effect Size Meta-Analysis (P1-VAL-4)
//!
//! Compares simulated vs empirical effect sizes across multiple studies.
//! Implements fixed-effect and random-effects meta-analysis models.

pub mod analysis;
pub mod bias;
pub mod comparison;
pub mod dist;
pub mod forest;
pub mod funnel;
pub mod heterogeneity;
pub mod types;

pub use analysis::MetaAnalyzer;
pub use forest::ForestPlot;
pub use funnel::FunnelPlot;
pub use types::*;
