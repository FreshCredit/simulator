// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Visualization Module (P1-SIM-4)
//!
//! Auto-generates charts and graphs from simulation results.
//!
//! ## Features
//!
//! - **Chart Types**: Line, bar, scatter, heatmap, histogram, box plot, area, radar
//! - **Export Formats**: SVG, PNG, JSON, HTML
//! - **Color Schemes**: Viridis, Plasma, Coolwarm, Category10, Category20
//! - **Data Adapters**: Convert experiment results to visualization datasets
//!
//! ## Example
//!
//! ```rust,ignore
//! use sim_service::visualization::{
//!     ChartGenerator, DataAdapter, ChartConfig, ChartType, Dataset, DataSeries
//! };
//!
//! // Create dataset from results
//! let adapter = DataAdapter::new();
//! let dataset = adapter.time_series_to_dataset(&time_series_data);
//!
//! // Generate chart
//! let generator = ChartGenerator::new();
//! let config = ChartConfig {
//!     chart_type: ChartType::Line,
//!     title: "Fitness Over Time".to_string(),
//!     ..Default::default()
//! };
//! let chart = generator.generate(&dataset, &config);
//! ```

pub mod adapter;
pub mod generator;
pub mod types;

pub use adapter::DataAdapter;
pub use generator::ChartGenerator;
pub use types::*;
