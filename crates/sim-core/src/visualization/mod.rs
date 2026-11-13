// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=integration
// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=api
//! Visualization System (P1-SIM-4)
//!
//! Chart generation for experiment results.

pub mod adapter;
pub mod generator;
pub mod types;

pub use adapter::DataAdapter;
pub use generator::ChartGenerator;
pub use types::*;
