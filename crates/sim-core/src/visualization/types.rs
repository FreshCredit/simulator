//! Visualization types

use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Chart types supported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChartType {
    /// line.
    Line,
    /// bar.
    Bar,
    /// scatter.
    Scatter,
    /// histogram.
    Histogram,
    /// heatmap.
    Heatmap,
    /// network.
    Network,
    /// timeseries.
    Timeseries,
    /// box plot.
    BoxPlot,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Chart series data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSeries {
    /// name.
    pub name: String,
    /// data.
    pub data: Vec<ChartPoint>,
    /// color.
    pub color: Option<String>,
}

/// Individual chart point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartPoint {
    /// x.
    pub x: f64,
    /// y.
    pub y: f64,
    /// label.
    pub label: Option<String>,
}

/// Color schemes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorScheme {
    /// viridis.
    Viridis,
    /// plasma.
    Plasma,
    /// inferno.
    Inferno,
    /// magma.
    Magma,
    /// coolwarm.
    Coolwarm,
    /// category10.
    Category10,
    /// category20.
    Category20,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Chart dimensions
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ChartDimensions {
    /// width.
    pub width: u32,
    /// height.
    pub height: u32,
    /// margin top.
    pub margin_top: u32,
    /// margin right.
    pub margin_right: u32,
    /// margin bottom.
    pub margin_bottom: u32,
    /// margin left.
    pub margin_left: u32,
}

impl Default for ChartDimensions {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            margin_top: 20,
            margin_right: 30,
            margin_bottom: 50,
            margin_left: 60,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Chart configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    /// title.
    pub title: String,
    /// x label.
    pub x_label: String,
    /// y label.
    pub y_label: String,
    /// chart type.
    pub chart_type: ChartType,
    /// dimensions.
    pub dimensions: ChartDimensions,
    /// color scheme.
    pub color_scheme: ColorScheme,
    /// show legend.
    pub show_legend: bool,
    /// show grid.
    pub show_grid: bool,
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            title: "Chart".to_string(),
            x_label: "X".to_string(),
            y_label: "Y".to_string(),
            chart_type: ChartType::Line,
            dimensions: ChartDimensions::default(),
            color_scheme: ColorScheme::Category10,
            show_legend: true,
            show_grid: true,
        }
    }
}

/// Generated chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedChart {
    /// config.
    pub config: ChartConfig,
    /// svg.
    pub svg: String,
    /// series.
    pub series: Vec<ChartSeries>,
}
