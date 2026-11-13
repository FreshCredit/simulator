// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Cross-Paper Meta-Analysis (P1-SIM-5)
//!
//! Aggregating results from all 12 research papers.

pub mod aggregator;
pub mod types;

pub use aggregator::MetaAggregator;
pub use types::*;
