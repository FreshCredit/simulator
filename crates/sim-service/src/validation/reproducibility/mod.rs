//! Reproducibility Validation Module (P1-VAL-5)
//!
//! Provides tools for verifying simulation results across different random seeds
//! to ensure statistical robustness and reproducibility.
//!
// TAG: surface=simulation owner=research-team rule=SIM-001
//! ## Features
//!
//! - **Seed Variation Testing**: Run simulations with systematic seed variations
//! - **Stability Metrics**: Coefficient of variation, confidence intervals, outlier detection
//! - **Convergence Detection**: Determine when results stabilize across seeds
//! - **Variance Analysis**: Between/within variance components, ICC calculation
//! - **Replication Recommendations**: Calculate required sample sizes for target precision
//!
//! ## Example
//!
//! ```rust,ignore
//! use sim_service::validation::reproducibility::{
//!     StabilityAnalyzer, ReproducibilityConfig, ReplicationResult
//! };
//! use std::collections::HashMap;
//!
//! let config = ReproducibilityConfig::default();
//! let analyzer = StabilityAnalyzer::new(config);
//!
//! let results = vec![
//!     ReplicationResult {
//!         index: 0,
//!         seed: 42,
//!         metrics: [("fitness".to_string(), 0.85)].into_iter().collect(),
//!         execution_time_secs: 1.0,
//!         success: true,
//!         error: None,
//!     },
//!     // ... more replications
//! ];
//!
//! let analysis = analyzer.analyze(&results);
//! println!("Stability score: {}", analysis.stability_score);
//! ```

// TAG: surface=simulation owner=research-team rule=SIM-001
pub mod stability;
pub mod types;
pub mod variance;

pub use stability::StabilityAnalyzer;
pub use types::{
    ConvergenceStatus, ReplicationResult, ReproducibilityConfig, ReproducibilityReport,
    ReproducibilityResult, StabilityLevel, VarianceAnalysisResponse, VarianceComponents,
};
pub use variance::VarianceEstimator;
