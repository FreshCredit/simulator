//! Visualization module for generating PNG charts from simulation metrics
//!
//! Provides chart generation for time series, scatter plots, and histograms
//! using a colorblind-accessible palette.

use plotters::prelude::*;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

// Colorblind-accessible palette (Tol colors)

/// Blue (Tol color)
pub const COLOR_BLUE: RGBColor = RGBColor(0x00, 0x77, 0xBB);
/// Orange (Tol color)
pub const COLOR_ORANGE: RGBColor = RGBColor(0xEE, 0x77, 0x33);
/// Green (Tol color)
pub const COLOR_GREEN: RGBColor = RGBColor(0x00, 0x99, 0x88);
/// Red (Tol color)
pub const COLOR_RED: RGBColor = RGBColor(0xCC, 0x33, 0x11);
/// Gray (Tol color)
pub const COLOR_GRAY: RGBColor = RGBColor(0x88, 0x88, 0x88);
/// Light gray (Tol color)
pub const COLOR_LIGHT_GRAY: RGBColor = RGBColor(0xDD, 0xDD, 0xDD);

/// Chart configuration for output sizing and styling
#[derive(Debug, Clone)]
pub struct ChartConfig {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Chart title
    pub title: String,
    /// Output file path
    pub output_path: PathBuf,
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            title: "Chart".to_string(),
            output_path: PathBuf::from("chart.png"),
        }
    }
    // TAG: surface=simulation owner=research-team rule=SIM-001
}

impl ChartConfig {
    /// Create a new chart config with specified dimensions and output path
    pub fn new(
        width: u32,
        height: u32,
        title: impl Into<String>,
        output_path: impl AsRef<Path>,
    ) -> Self {
        Self {
            width,
            height,
            title: title.into(),
            output_path: output_path.as_ref().to_path_buf(),
        }
    }
}

/// Time series data for line charts
#[derive(Debug, Clone)]
pub struct TimeSeriesData {
    /// X-axis values (typically tick numbers)
    pub x_values: Vec<f64>,
    /// Y-axis values
    pub y_values: Vec<f64>,
    /// Data series label
    pub label: String,
    /// Color for the line
    pub color: RGBColor,
}

impl TimeSeriesData {
    /// Create new time series data
    pub fn new(
        x_values: Vec<f64>,
        y_values: Vec<f64>,
        label: impl Into<String>,
        color: RGBColor,
    ) -> Self {
        Self {
            x_values,
            y_values,
            label: label.into(),
            color,
        }
    }

    /// Calculate min/max for x values
    pub fn x_range(&self) -> (f64, f64) {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let clean: Vec<f64> = self
            .x_values
            .iter()
            .copied()
            .filter(|v| v.is_finite())
            .collect();
        if clean.is_empty() {
            return (0.0, 1.0);
        }
        let min = clean.iter().copied().fold(f64::INFINITY, f64::min);
        let max = clean.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        if (max - min).abs() < f64::EPSILON {
            return (min - 0.5, max + 0.5);
        }
        // Add small padding
        let padding = (max - min) * 0.05;
        (min - padding, max + padding)
    }

    /// Calculate min/max for y values
    pub fn y_range(&self) -> (f64, f64) {
        let clean: Vec<f64> = self
            .y_values
            .iter()
            .copied()
            .filter(|v| v.is_finite())
            .collect();
        if clean.is_empty() {
            return (0.0, 1.0);
        }
        let min = clean.iter().copied().fold(f64::INFINITY, f64::min);
        let max = clean.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        // Add small padding
        let padding = (max - min) * 0.05;
        if (max - min).abs() < f64::EPSILON {
            // Handle constant values
            return (min - 0.5, max + 0.5);
        }
        (min - padding, max + padding)
    }
}

/// Scatter plot data with optional regression
#[derive(Debug, Clone)]
pub struct ScatterData {
    /// Data points (x, y)
    pub points: Vec<(f64, f64)>,
    /// X-axis label
    // TAG: surface=simulation owner=research-team rule=SIM-001
    pub x_label: String,
    /// Y-axis label
    pub y_label: String,
    /// Plot title
    pub title: String,
}

impl ScatterData {
    /// Create new scatter data
    pub fn new(
        points: Vec<(f64, f64)>,
        x_label: impl Into<String>,
        y_label: impl Into<String>,
        title: impl Into<String>,
    ) -> Self {
        Self {
            points,
            x_label: x_label.into(),
            y_label: y_label.into(),
            title: title.into(),
        }
    }

    /// Calculate linear regression (slope, intercept, `r_squared`)
    #[must_use]
    pub fn linear_regression(&self) -> Option<(f64, f64, f64)> {
        let clean: Vec<(f64, f64)> = self
            .points
            .iter()
            .copied()
            .filter(|(x, y)| x.is_finite() && y.is_finite())
            .collect();
        let n = clean.len() as f64;
        if n < 2.0 {
            return None;
        }

        let sum_x: f64 = clean.iter().map(|p| p.0).sum();
        let sum_y: f64 = clean.iter().map(|p| p.1).sum();
        let sum_xy: f64 = clean.iter().map(|p| p.0 * p.1).sum();
        let sum_x2: f64 = clean.iter().map(|p| p.0 * p.0).sum();

        let sum_x_squared = sum_x * sum_x;
        let denominator = n * sum_x2 - sum_x_squared;
        if denominator.abs() < 1e-10 {
            return None;
        }

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let slope = (n * sum_xy - sum_x * sum_y) / denominator;
        let intercept = (sum_y - slope * sum_x) / n;

        // Calculate R-squared
        let mean_y = sum_y / n;
        let ss_tot: f64 = clean.iter().map(|p| (p.1 - mean_y).powi(2)).sum();
        let ss_res: f64 = clean
            .iter()
            .map(|p| {
                let y_pred = slope * p.0 + intercept;
                (p.1 - y_pred).powi(2)
            })
            .sum();

        let r_squared = if ss_tot.abs() < 1e-10 {
            0.0
        } else {
            (1.0 - (ss_res / ss_tot)).clamp(0.0, 1.0)
        };

        Some((slope, intercept, r_squared))
    }

    /// Get x range with padding
    pub fn x_range(&self) -> (f64, f64) {
        if self.points.is_empty() {
            return (0.0, 1.0);
        }
        let min = self
            .points
            .iter()
            .map(|p| p.0)
            .fold(f64::INFINITY, f64::min);
        let max = self
            .points
            .iter()
            .map(|p| p.0)
            .fold(f64::NEG_INFINITY, f64::max);
        let padding = (max - min) * 0.05;
        if (max - min).abs() < f64::EPSILON {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            return (min - 0.5, max + 0.5);
        }
        (min - padding, max + padding)
    }

    /// Get y range with padding
    pub fn y_range(&self) -> (f64, f64) {
        if self.points.is_empty() {
            return (0.0, 1.0);
        }
        let min = self
            .points
            .iter()
            .map(|p| p.1)
            .fold(f64::INFINITY, f64::min);
        let max = self
            .points
            .iter()
            .map(|p| p.1)
            .fold(f64::NEG_INFINITY, f64::max);
        let padding = (max - min) * 0.05;
        if (max - min).abs() < f64::EPSILON {
            return (min - 0.5, max + 0.5);
        }
        (min - padding, max + padding)
    }
}

/// Histogram data for distribution charts
#[derive(Debug, Clone)]
pub struct HistogramData {
    /// Bin edges (n+1 values for n bins)
    pub bins: Vec<f64>,
    /// Counts per bin
    pub counts: Vec<usize>,
    /// Chart title
    pub title: String,
}

impl HistogramData {
    /// Create histogram from data values with specified number of bins
    pub fn from_data(data: &[f64], num_bins: usize, title: impl Into<String>) -> Self {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let clean: Vec<f64> = data.iter().copied().filter(|v| v.is_finite()).collect();
        if clean.is_empty() || num_bins == 0 {
            return Self {
                bins: vec![0.0, 1.0],
                counts: vec![0],
                title: title.into(),
            };
        }

        let min = clean.iter().copied().fold(f64::INFINITY, f64::min);
        let max = clean.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        if (max - min).abs() < f64::EPSILON {
            // All values are the same
            return Self {
                bins: vec![min - 0.5, max + 0.5],
                counts: vec![clean.len()],
                title: title.into(),
            };
        }

        let bin_width = (max - min) / num_bins as f64;
        let mut bins = vec![min];
        for i in 1..=num_bins {
            bins.push(min + bin_width * i as f64);
        }

        let mut counts = vec![0usize; num_bins];
        for &value in &clean {
            let bin_idx = (((value - min) / bin_width).floor() as isize)
                .clamp(0, num_bins as isize - 1) as usize;
            counts[bin_idx] += 1;
        }

        Self {
            bins,
            counts,
            title: title.into(),
        }
    }

    /// Calculate mean of the underlying data (if known)
    #[must_use]
    // TAG: surface=simulation owner=research-team rule=SIM-001
    pub fn mean(&self) -> Option<f64> {
        if self.counts.is_empty() {
            return None;
        }

        let total_count: usize = self.counts.iter().sum();
        if total_count == 0 {
            return None;
        }

        let sum: f64 = self
            .counts
            .iter()
            .enumerate()
            .map(|(i, &count)| {
                let bin_center = (self.bins[i] + self.bins[i + 1]) / 2.0;
                bin_center * count as f64
            })
            .sum();

        Some(sum / total_count as f64)
    }

    /// Calculate standard deviation of the underlying data (if known)
    #[must_use]
    pub fn std_dev(&self) -> Option<f64> {
        let mean = self.mean()?;
        let total_count: usize = self.counts.iter().sum();
        if total_count == 0 {
            return None;
        }

        let variance_sum: f64 = self
            .counts
            .iter()
            .enumerate()
            .map(|(i, &count)| {
                let bin_center = (self.bins[i] + self.bins[i + 1]) / 2.0;
                (bin_center - mean).powi(2) * count as f64
            })
            .sum();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let variance = variance_sum / total_count as f64;
        Some(variance.sqrt())
    }
}

/// Generate a time series line chart
///
/// # Arguments
/// * `data` - The time series data to plot
/// * `config` - Chart configuration (dimensions, title, output path)
///
/// # Returns
/// * `Ok(())` if the chart was successfully generated
/// * `Err(Box<dyn Error>)` if there was an error
/// # Errors
///
/// Returns an error if the operation fails.
pub fn generate_time_series_chart(
    data: &TimeSeriesData,
    config: &ChartConfig,
) -> Result<(), Box<dyn Error>> {
    // Ensure output directory exists
    if let Some(parent) = config.output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let root =
        BitMapBackend::new(&config.output_path, (config.width, config.height)).into_drawing_area();
    root.fill(&WHITE)?;

    let (x_min, x_max) = data.x_range();
    let (y_min, y_max) = data.y_range();

    let mut chart = ChartBuilder::on(&root)
        .caption(&config.title, ("sans-serif", 20).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

    chart
        // TAG: surface=simulation owner=research-team rule=SIM-001
        .configure_mesh()
        .x_desc("Tick")
        .y_desc(&data.label)
        .axis_style(BLACK)
        .label_style(("sans-serif", 12).into_font())
        .light_line_style(COLOR_LIGHT_GRAY)
        .draw()?;

    let data_points: Vec<(f64, f64)> = data
        .x_values
        .iter()
        .zip(data.y_values.iter())
        .map(|(&x, &y)| (x, y))
        .collect();

    chart.draw_series(LineSeries::new(data_points, data.color.stroke_width(2)))?;

    root.present()?;
    Ok(())
}

/// Generate a scatter plot with optional regression line
///
/// # Arguments
/// * `data` - The scatter data points
/// * `config` - Chart configuration
///
/// # Returns
/// * `Ok(())` if the chart was successfully generated
/// # Errors
///
/// Returns an error if the operation fails.
pub fn generate_scatter_plot(
    data: &ScatterData,
    config: &ChartConfig,
) -> Result<(), Box<dyn Error>> {
    // Ensure output directory exists
    if let Some(parent) = config.output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let root =
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        BitMapBackend::new(&config.output_path, (config.width, config.height)).into_drawing_area();
    root.fill(&WHITE)?;

    let (x_min, x_max) = data.x_range();
    let (y_min, y_max) = data.y_range();

    let mut chart = ChartBuilder::on(&root)
        .caption(&data.title, ("sans-serif", 20).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

    chart
        .configure_mesh()
        .x_desc(&data.x_label)
        .y_desc(&data.y_label)
        .axis_style(BLACK)
        .label_style(("sans-serif", 12).into_font())
        .light_line_style(COLOR_LIGHT_GRAY)
        .draw()?;

    // Draw scatter points
    chart.draw_series(
        data.points
            .iter()
            .map(|&(x, y)| Circle::new((x, y), 3, COLOR_BLUE.filled())),
    )?;

    // Draw regression line if calculable
    if let Some((slope, intercept, r_squared)) = data.linear_regression() {
        let line_start = (x_min, slope * x_min + intercept);
        let line_end = (x_max, slope * x_max + intercept);
        chart.draw_series(LineSeries::new(
            vec![line_start, line_end],
            COLOR_RED.stroke_width(2),
        ))?;

        // Add R² annotation
        let r2_text = format!("R² = {r_squared:.4}");
        chart.draw_series(std::iter::once(Text::new(
            // TAG: surface=simulation owner=research-team rule=SIM-001
            r2_text,
            (
                x_max - (x_max - x_min) * 0.25,
                y_max - (y_max - y_min) * 0.05,
            ),
            ("sans-serif", 14).into_font().color(&COLOR_RED),
        )))?;
    }

    root.present()?;
    Ok(())
}

/// Generate a histogram chart
///
/// # Arguments
/// * `data` - The histogram data (bins and counts)
/// * `config` - Chart configuration
///
/// # Returns
/// * `Ok(())` if the chart was successfully generated
/// # Errors
///
/// Returns an error if the operation fails.
pub fn generate_histogram(
    data: &HistogramData,
    config: &ChartConfig,
) -> Result<(), Box<dyn Error>> {
    // Ensure output directory exists
    if let Some(parent) = config.output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let root =
        BitMapBackend::new(&config.output_path, (config.width, config.height)).into_drawing_area();
    root.fill(&WHITE)?;

    let x_min = data.bins.first().copied().unwrap_or(0.0);
    let x_max = data.bins.last().copied().unwrap_or(1.0);
    let y_max = data.counts.iter().copied().max().unwrap_or(0) as f64 * 1.1;

    let mut chart = ChartBuilder::on(&root)
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        .caption(&data.title, ("sans-serif", 20).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(x_min..x_max, 0.0..y_max)?;

    chart
        .configure_mesh()
        .x_desc("Value")
        .y_desc("Count")
        .axis_style(BLACK)
        .label_style(("sans-serif", 12).into_font())
        .light_line_style(COLOR_LIGHT_GRAY)
        .draw()?;

    // Draw bars
    let bar_width = (x_max - x_min) / data.bins.len() as f64 * 0.9;
    for (i, &count) in data.counts.iter().enumerate() {
        let bin_start = data.bins[i];
        let bin_end = data.bins[i + 1];
        let bin_center = (bin_start + bin_end) / 2.0;

        chart.draw_series(std::iter::once(Rectangle::new(
            [
                (bin_center - bar_width / 2.0, 0.0),
                (bin_center + bar_width / 2.0, count as f64),
            ],
            COLOR_BLUE.filled(),
        )))?;
    }

    // Add mean and std dev annotation
    if let (Some(mean), Some(std)) = (data.mean(), data.std_dev()) {
        let stats_text = format!("μ = {mean:.2}, σ = {std:.2}");
        chart.draw_series(std::iter::once(Text::new(
            stats_text,
            (x_min + (x_max - x_min) * 0.05, y_max * 0.95),
            ("sans-serif", 14).into_font().color(&COLOR_GRAY),
        )))?;
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    root.present()?;
    Ok(())
}

/// Generate a multi-metric overlay chart with dual Y-axes
///
/// # Arguments
/// * `data_series` - Array of time series data to plot (up to 2 series supported)
/// * `config` - Chart configuration
/// * `left_label` - Label for left Y-axis
/// * `right_label` - Label for right Y-axis
///
/// # Returns
/// * `Ok(())` if the chart was successfully generated
/// # Errors
///
/// Returns an error if the operation fails.
/// # Panics
///
/// Panics if preconditions are not met.
pub fn generate_multi_metric_chart(
    data_series: &[TimeSeriesData],
    config: &ChartConfig,
    left_label: &str,
    right_label: &str,
) -> Result<(), Box<dyn Error>> {
    if data_series.len() < 2 {
        return Err("At least 2 data series required for multi-metric chart".into());
    }

    // Ensure output directory exists
    if let Some(parent) = config.output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let root =
        BitMapBackend::new(&config.output_path, (config.width, config.height)).into_drawing_area();
    root.fill(&WHITE)?;

    let data1 = &data_series[0];
    let data2 = &data_series[1];

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let (x_min, x_max) = data1.x_range();
    let (y1_min, y1_max) = data1.y_range();
    let (y2_min, y2_max) = data2.y_range();

    // Build chart with dual Y-axes
    let mut chart = ChartBuilder::on(&root)
        .caption(&config.title, ("sans-serif", 20).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .right_y_label_area_size(60)
        .build_cartesian_2d(x_min..x_max, y1_min..y1_max)?
        .set_secondary_coord(x_min..x_max, y2_min..y2_max);

    // Configure primary Y-axis (left)
    chart
        .configure_mesh()
        .x_desc("Tick")
        .y_desc(left_label)
        .axis_style(BLACK)
        .label_style(("sans-serif", 12).into_font())
        .light_line_style(COLOR_LIGHT_GRAY)
        .draw()?;

    // Configure secondary Y-axis (right)
    chart
        .configure_secondary_axes()
        .y_desc(right_label)
        .axis_style(COLOR_ORANGE)
        .label_style(("sans-serif", 12).into_font().color(&COLOR_ORANGE))
        .draw()?;

    // Draw first series on primary axis
    let points1: Vec<(f64, f64)> = data1
        .x_values
        .iter()
        .zip(data1.y_values.iter())
        .map(|(&x, &y)| (x, y))
        .collect();

    // TAG: surface=simulation owner=research-team rule=SIM-001
    chart
        .draw_series(LineSeries::new(points1, data1.color.stroke_width(2)))?
        .label(&data1.label)
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], data1.color.stroke_width(2)));

    // Draw second series on secondary axis
    let points2: Vec<(f64, f64)> = data2
        .x_values
        .iter()
        .zip(data2.y_values.iter())
        .map(|(&x, &y)| (x, y))
        .collect();

    chart
        .draw_secondary_series(LineSeries::new(points2, data2.color.stroke_width(2)))?
        .label(&data2.label)
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], data2.color.stroke_width(2)));

    // Add legend
    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperRight)
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    root.present()?;
    Ok(())
}

/// Metrics record from metrics.jsonl file
#[derive(Debug, Deserialize)]
struct MetricsRecord {
    tick: u64,
    agent_count: u32,
    cooperation_rate: f64,
    gini: f64,

    trade_volume: u32,

    dispute_rate: f64,
    #[allow(dead_code)]
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    verification_rate: f64,
    avg_energy: f64,
    #[allow(dead_code)]
    avg_docas_magnitude: f64,

    branching_ratio: f64,
    #[allow(dead_code)]
    active_norms: u32,
    #[allow(dead_code)]
    institution_count: u32,
    #[allow(dead_code)]
    total_anchors: u32,
    #[allow(dead_code)]
    total_verifications: u32,
}

/// Manifest structure for parsing manifest.json
#[derive(Debug, Deserialize)]
struct Manifest {
    #[serde(rename = "metrics_file")]
    metrics_file: String,
}

/// Generate all PNG charts for an experiment
///
/// Reads the manifest.json and metrics.jsonl files from the experiment run
/// and generates 4 time series charts:
/// 1. `agent_count` over time
/// 2. `cooperation_rate` over time
/// 3. gini coefficient over time
/// 4. `avg_energy` over time
///
/// # Arguments
/// * `manifest_path` - Path to the manifest.json file
/// * `output_dir` - Directory to save the PNG files (created if doesn't exist)
///
/// # Returns
/// * `Ok(Vec<PathBuf>)` - List of paths to generated PNG files
/// * `Err(Box<dyn Error>)` - Error if files are missing or malformed
/// # Errors
///
// TAG: surface=simulation owner=research-team rule=SIM-001
/// Returns an error if the operation fails.
pub fn generate_experiment_pngs(
    manifest_path: &Path,
    output_dir: &Path,
) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    if !manifest_path.exists() {
        return Err(format!("Manifest file not found: {}", manifest_path.display()).into());
    }

    let manifest_content = std::fs::read_to_string(manifest_path)?;
    let manifest: Manifest = serde_json::from_str(&manifest_content)
        .map_err(|e| format!("Failed to parse manifest.json: {e}"))?;

    let manifest_dir = manifest_path
        .parent()
        .ok_or("Manifest path has no parent directory")?;
    let metrics_path = manifest_dir.join(&manifest.metrics_file);

    if !metrics_path.exists() {
        return Err(format!("Metrics file not found: {}", metrics_path.display()).into());
    }

    std::fs::create_dir_all(output_dir)?;

    let records = load_metrics_records(&metrics_path)?;
    if records.is_empty() {
        return Err("No metrics records found in metrics.jsonl".into());
    }

    let ticks: Vec<f64> = records.iter().map(|r| r.tick as f64).collect();
    let agent_counts: Vec<f64> = records.iter().map(|r| r.agent_count as f64).collect();
    let cooperation_rates: Vec<f64> = records.iter().map(|r| r.cooperation_rate).collect();
    let gini_values: Vec<f64> = records.iter().map(|r| r.gini).collect();
    let avg_energies: Vec<f64> = records.iter().map(|r| r.avg_energy).collect();

    let mut generated_files = Vec::new();

    generate_standard_charts(
        &ticks,
        agent_counts,
        cooperation_rates.clone(),
        gini_values.clone(),
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        avg_energies,
        output_dir,
        &mut generated_files,
    )?;

    let trade_volumes: Vec<f64> = records.iter().map(|r| r.trade_volume as f64).collect();
    let branching_ratios: Vec<f64> = records.iter().map(|r| r.branching_ratio).collect();
    let dispute_rates: Vec<f64> = records.iter().map(|r| r.dispute_rate).collect();

    generate_optional_charts(
        &ticks,
        OptionalChartData {
            trade_volumes,
            branching_ratios,
            dispute_rates,
            cooperation_rates,
            gini_values,
        },
        output_dir,
        &mut generated_files,
    )?;

    Ok(generated_files)
}

fn load_metrics_records(path: &Path) -> Result<Vec<MetricsRecord>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let record: MetricsRecord = serde_json::from_str(&line)
            .map_err(|e| format!("Failed to parse metrics.jsonl line {}: {}", line_num + 1, e))?;
        records.push(record);
    }
    Ok(records)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
fn generate_standard_charts(
    ticks: &[f64],
    agent_counts: Vec<f64>,
    cooperation_rates: Vec<f64>,
    gini_values: Vec<f64>,
    avg_energies: Vec<f64>,
    output_dir: &Path,
    generated_files: &mut Vec<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    let charts = [
        (
            agent_counts,
            "Agent Count",
            COLOR_BLUE,
            "agent_count.png",
            "Agent Count Over Time",
        ),
        (
            cooperation_rates,
            "Cooperation Rate",
            COLOR_ORANGE,
            "cooperation_rate.png",
            "Cooperation Rate Over Time",
        ),
        (
            gini_values,
            "Gini Coefficient",
            COLOR_GREEN,
            "gini.png",
            "Gini Coefficient Over Time",
        ),
        (
            avg_energies,
            "Average Energy",
            COLOR_RED,
            "avg_energy.png",
            "Average Energy Over Time",
        ),
    ];

    for (data, label, color, filename, title) in charts {
        let ts_data = TimeSeriesData::new(ticks.to_vec(), data, label, color);
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let config = ChartConfig::new(800, 600, title, output_dir.join(filename));
        generate_time_series_chart(&ts_data, &config)?;
        generated_files.push(config.output_path);
    }

    Ok(())
}

struct OptionalChartData {
    trade_volumes: Vec<f64>,
    branching_ratios: Vec<f64>,
    dispute_rates: Vec<f64>,
    cooperation_rates: Vec<f64>,
    gini_values: Vec<f64>,
}

fn generate_optional_charts(
    ticks: &[f64],
    data: OptionalChartData,
    output_dir: &Path,
    generated_files: &mut Vec<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    if data.trade_volumes.iter().any(|&v| v > 0.0) {
        let ts_data = TimeSeriesData::new(
            ticks.to_vec(),
            data.trade_volumes,
            "Trade Volume",
            COLOR_BLUE,
        );
        let config = ChartConfig::new(
            800,
            600,
            "Trade Volume Over Time",
            output_dir.join("trade_volume.png"),
        );
        generate_time_series_chart(&ts_data, &config)?;
        generated_files.push(config.output_path);
    }

    let branching_data = TimeSeriesData::new(
        ticks.to_vec(),
        // TAG: surface=simulation owner=research-team rule=SIM-001
        data.branching_ratios,
        "Branching Ratio",
        COLOR_ORANGE,
    );
    let branching_config = ChartConfig::new(
        800,
        600,
        "Branching Ratio Over Time (Instability Measure)",
        output_dir.join("branching_ratio.png"),
    );
    generate_time_series_chart(&branching_data, &branching_config)?;
    generated_files.push(branching_config.output_path);

    let multi_config = ChartConfig::new(
        800,
        600,
        "Cooperation Rate vs Gini Coefficient",
        output_dir.join("multi_metric.png"),
    );
    let coop_data = TimeSeriesData::new(
        ticks.to_vec(),
        data.cooperation_rates.clone(),
        "Cooperation Rate",
        COLOR_BLUE,
    );
    let gini_data = TimeSeriesData::new(
        ticks.to_vec(),
        data.gini_values.clone(),
        "Gini",
        COLOR_ORANGE,
    );
    generate_multi_metric_chart(
        &[coop_data, gini_data],
        &multi_config,
        "Cooperation Rate",
        "Gini Coefficient",
    )?;
    generated_files.push(multi_config.output_path);

    let scatter_points: Vec<(f64, f64)> = data
        .cooperation_rates
        .iter()
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        .zip(data.gini_values.iter())
        .map(|(&c, &g)| (c, g))
        .collect();
    if scatter_points.len() >= 2 {
        let scatter_data = ScatterData::new(
            scatter_points,
            "Cooperation Rate",
            "Gini Coefficient",
            "Cooperation vs Inequality",
        );
        let scatter_config = ChartConfig::new(
            800,
            600,
            "Cooperation vs Inequality",
            output_dir.join("cooperation_vs_gini.png"),
        );
        generate_scatter_plot(&scatter_data, &scatter_config)?;
        generated_files.push(scatter_config.output_path);
    }

    if data.dispute_rates.iter().any(|&v| v > 0.0) {
        let ts_data = TimeSeriesData::new(
            ticks.to_vec(),
            data.dispute_rates,
            "Dispute Rate",
            COLOR_RED,
        );
        let config = ChartConfig::new(
            800,
            600,
            "Dispute Rate Over Time",
            output_dir.join("dispute_rate.png"),
        );
        generate_time_series_chart(&ts_data, &config)?;
        generated_files.push(config.output_path);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    use super::*;

    #[test]
    fn test_time_series_data_range() {
        let data = TimeSeriesData::new(
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
            vec![10.0, 20.0, 30.0, 20.0, 10.0],
            "Test",
            COLOR_BLUE,
        );

        let (x_min, x_max) = data.x_range();
        assert!(x_min < 1.0);
        assert!(x_max > 5.0);

        let (y_min, y_max) = data.y_range();
        assert!(y_min < 10.0);
        assert!(y_max > 30.0);
    }

    #[test]
    fn test_scatter_regression() {
        let points: Vec<(f64, f64)> = vec![(1.0, 2.0), (2.0, 4.0), (3.0, 6.0), (4.0, 8.0)];
        let data = ScatterData::new(points, "X", "Y", "Test");

        let (slope, intercept, r_squared) = data.linear_regression().unwrap();
        assert!((slope - 2.0).abs() < 0.001);
        assert!(intercept.abs() < 0.001);
        assert!((r_squared - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_histogram_from_data() {
        let data = vec![1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 4.0, 4.0, 5.0];
        let hist = HistogramData::from_data(&data, 4, "Test");

        assert_eq!(hist.bins.len(), 5);
        assert_eq!(hist.counts.len(), 4);
        assert!(hist.mean().is_some());
        assert!(hist.std_dev().is_some());
    }

    #[test]
    fn test_histogram_nan_resilience() {
        let data = vec![1.0, f64::NAN, 2.0, f64::INFINITY, 3.0];
        let hist = HistogramData::from_data(&data, 2, "NaN Test");
        assert_eq!(hist.counts.iter().sum::<usize>(), 3);
        assert!(hist.mean().unwrap().is_finite());
    }
}
