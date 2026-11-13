// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Cross-Paper Meta-Analysis Module (P1-SIM-5)
//!
//! Aggregates results across all 12 research papers for comprehensive analysis.
//!
//! ## Features
//!
//! - **Paper Aggregation**: Collect results from all 12 papers
//! - **Cross-Cutting Themes**: Identify common patterns across papers
//! - **Synthesis Reports**: Generate overall research synthesis
//! - **Gap Analysis**: Identify missing research
//! - **Publication Readiness**: Assess which papers are ready for publication
//!
//! ## Example
//!
//! ```rust,ignore
//! use sim_service::meta_analysis::{MetaAggregator, ResearchPaper};
//!
//! let aggregator = MetaAggregator::new();
//! let analysis = aggregator.aggregate_all();
//!
//! for (paper, results) in &analysis.papers {
//!     println!("{}: {:.0}% coverage", paper.name(), results.coverage_percentage);
//! }
//! ```

pub mod aggregator;
pub mod types;

pub use aggregator::MetaAggregator;
pub use types::*;
