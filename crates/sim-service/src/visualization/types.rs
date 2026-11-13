//! Visualization Types for Experiment Results (P1-SIM-4)
//!
//! Core type definitions for chart generation and data visualization.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Types of charts supported
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChartType {
    /// Line chart for time series
    Line,
    /// Bar chart for categorical comparison
    Bar,
    /// Scatter plot for correlations
    Scatter,
    /// Heatmap for matrix data
    Heatmap,
    /// Histogram for distributions
    Histogram,
    /// Box plot for statistical summaries
    BoxPlot,
    /// Area chart for cumulative data
    Area,
    /// Radar/spider chart for multi-dimensional data
    Radar,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Chart configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    /// chart type
    pub chart_type: ChartType,
    /// width
    pub width: u32,
    /// height
    pub height: u32,
    /// title
    pub title: String,
    /// x label
    pub x_label: Option<String>,
    /// y label
    pub y_label: Option<String>,
    /// show legend
    pub show_legend: bool,
    /// color scheme
    pub color_scheme: ColorScheme,
    /// format
    pub format: ExportFormat,
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            chart_type: ChartType::Line,
            width: 800,
            height: 600,
            title: "Chart".to_string(),
            x_label: None,
            y_label: None,
            show_legend: true,
            color_scheme: ColorScheme::Default,
            format: ExportFormat::Svg,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Color schemes for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorScheme {
    /// Default
    Default,
    /// Viridis
    Viridis,
    /// Plasma
    Plasma,
    /// Coolwarm
    Coolwarm,
    /// Category10
    Category10,
    /// Category20
    Category20,
}

/// Export formats
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    /// Svg
    Svg,
    /// Png
    Png,
    /// Json
    Json,
    /// Html
    Html,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Data series for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSeries {
    /// name
    pub name: String,
    /// data
    pub data: Vec<DataPoint>,
    /// color
    pub color: Option<String>,
}

/// Individual data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// x
    pub x: f64,
    /// y
    pub y: f64,
    /// label
    pub label: Option<String>,
    /// metadata
    pub metadata: Option<HashMap<String, String>>,
}

/// Dataset for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    /// series
    pub series: Vec<DataSeries>,
    /// annotations
    pub annotations: Vec<Annotation>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Chart annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    /// x
    pub x: f64,
    /// y
    pub y: f64,
    /// text
    pub text: String,
    /// style
    pub style: AnnotationStyle,
}

/// Annotation styling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationStyle {
    /// color
    pub color: String,
    /// font size
    pub font_size: u32,
    /// arrow
    pub arrow: bool,
}

impl Default for AnnotationStyle {
    fn default() -> Self {
        Self {
            color: "#ff0000".to_string(),
            font_size: 12,
            arrow: true,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Generated chart result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartResult {
    /// format
    pub format: ExportFormat,
    /// data
    pub data: Vec<u8>,
    /// mime type
    pub mime_type: String,
    /// width
    pub width: u32,
    /// height
    pub height: u32,
}

/// Time series data from experiment results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesData {
    /// metric name
    pub metric_name: String,
    /// timestamps
    pub timestamps: Vec<u64>,
    /// values
    pub values: Vec<f64>,
    /// statistics
    pub statistics: TimeSeriesStats,
}

/// Statistics for time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesStats {
    /// mean
    pub mean: f64,
    /// std dev
    pub std_dev: f64,
    /// min
    pub min: f64,
    /// max
    pub max: f64,
    /// trend
    pub trend: TrendDirection,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Trend direction
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendDirection {
    /// Increasing
    Increasing,
    /// Decreasing
    Decreasing,
    /// Stable
    Stable,
    /// Oscillating
    Oscillating,
}

/// Comparison data for bar charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonData {
    /// categories
    pub categories: Vec<String>,
    /// values
    pub values: Vec<f64>,
    /// errors
    pub errors: Option<Vec<f64>>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Correlation data for scatter plots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationData {
    /// x metric
    pub x_metric: String,
    /// y metric
    pub y_metric: String,
    /// points
    pub points: Vec<(f64, f64)>,
    /// correlation
    pub correlation: f64,
    /// p value
    pub p_value: f64,
}

/// Heatmap data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapData {
    /// x labels
    pub x_labels: Vec<String>,
    /// y labels
    pub y_labels: Vec<String>,
    /// values
    pub values: Vec<Vec<f64>>,
    /// min value
    pub min_value: f64,
    /// max value
    pub max_value: f64,
}

/// Distribution data for histograms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionData {
    /// bins
    pub bins: Vec<Bin>,
    /// mean
    pub mean: f64,
    /// median
    pub median: f64,
    /// std dev
    pub std_dev: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Histogram bin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bin {
    /// start
    pub start: f64,
    /// end
    pub end: f64,
    /// count
    pub count: u32,
    /// density
    pub density: f64,
}

/// Multi-dimensional data for radar charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadarData {
    /// dimensions
    pub dimensions: Vec<String>,
    /// values
    pub values: Vec<f64>,
    /// max values
    pub max_values: Vec<f64>,
}

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// title
    pub title: String,
    /// charts
    pub charts: Vec<ChartConfig>,
    /// layout
    pub layout: DashboardLayout,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Dashboard layout options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DashboardLayout {
    /// Single
    Single,
    /// `TwoColumn`
    TwoColumn,
    /// `ThreeColumn`
    ThreeColumn,
    /// Grid
    Grid {
        /// Number of rows
        rows: u32,
        /// Number of columns
        cols: u32,
    },
}

/// Visualization request from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationRequest {
    /// experiment id
    pub experiment_id: String,
    /// metrics
    pub metrics: Vec<String>,
    /// chart type
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub chart_type: ChartType,
    /// config
    pub config: ChartConfig,
    /// filters
    pub filters: Option<HashMap<String, String>>,
}

/// Visualization response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationResponse {
    /// chart
    pub chart: ChartResult,
    /// metadata
    pub metadata: VisualizationMetadata,
}

/// Visualization metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationMetadata {
    /// experiment id
    pub experiment_id: String,
    /// metrics visualized
    pub metrics_visualized: Vec<String>,
    /// data points
    pub data_points: usize,
    /// generated at
    pub generated_at: String,
}
