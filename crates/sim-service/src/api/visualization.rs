//! Visualization API Module (P1-SIM-4)
//!
//! REST endpoints for generating charts and visualizations from experiment results.

use axum::{
    extract::Query,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::collections::HashMap;

use crate::visualization::{
    ChartConfig, ChartGenerator, ChartType, ColorScheme, ComparisonData, DataAdapter, DataPoint,
    DataSeries, Dataset, ExportFormat, RadarData,
};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Query parameters for chart generation
#[derive(Debug, Deserialize)]
pub struct ChartQuery {
    /// experiment id
    pub experiment_id: String,
    /// metric
    pub metric: String,
    /// chart type
    pub chart_type: Option<String>,
    /// width
    pub width: Option<u32>,
    /// height
    pub height: Option<u32>,
    /// format
    pub format: Option<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Generate a chart from experiment data
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn generate_chart(
    Query(query): Query<ChartQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let generator = ChartGenerator::new();

    // Parse chart type
    let chart_type = query
        .chart_type
        .as_deref()
        .map_or(ChartType::Line, parse_chart_type);

    // Parse format
    let format = query
        .format
        .as_deref()
        .map_or(ExportFormat::Svg, parse_format);

    // Generate synthetic data for demo
    let dataset = generate_demo_dataset(&query.metric);

    let config = ChartConfig {
        chart_type,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        width: query.width.unwrap_or(800),
        height: query.height.unwrap_or(600),
        title: format!("{} - {}", query.experiment_id, query.metric),
        x_label: Some("Time".to_string()),
        y_label: Some(query.metric),
        show_legend: true,
        color_scheme: ColorScheme::Category10,
        format,
    };

    let result = generator.generate(&dataset, &config);

    let mime_type = match result.format {
        ExportFormat::Svg => "image/svg+xml",
        ExportFormat::Png => "image/png",
        ExportFormat::Json => "application/json",
        ExportFormat::Html => "text/html",
    };

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, mime_type)],
        result.data,
    ))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Generate a comparison chart
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn generate_comparison_chart(
    Json(request): Json<ComparisonRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let generator = ChartGenerator::new();
    let adapter = DataAdapter::new();

    let dataset = adapter.comparison_to_dataset(&ComparisonData {
        categories: request.categories,
        values: request.values,
        errors: request.errors,
    });

    let config = ChartConfig {
        chart_type: ChartType::Bar,
        width: request.width.unwrap_or(800),
        height: request.height.unwrap_or(600),
        title: request.title,
        x_label: Some("Category".to_string()),
        y_label: Some("Value".to_string()),
        show_legend: true,
        color_scheme: ColorScheme::Category10,
        format: ExportFormat::Svg,
    };

    let result = generator.generate(&dataset, &config);

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/svg+xml")],
        result.data,
    ))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Generate a scatter plot for correlation
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn generate_scatter_plot(
    Json(request): Json<ScatterRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let generator = ChartGenerator::new();
    let adapter = DataAdapter::new();

    let correlation = adapter.create_correlation(
        &request.x_values,
        &request.y_values,
        &request.x_metric,
        &request.y_metric,
    );

    let dataset = adapter.correlation_to_dataset(&correlation);

    let config = ChartConfig {
        chart_type: ChartType::Scatter,
        width: request.width.unwrap_or(800),
        height: request.height.unwrap_or(600),
        title: format!(
            "{} vs {} (r={:.3})",
            request.x_metric, request.y_metric, correlation.correlation
        ),
        x_label: Some(request.x_metric),
        y_label: Some(request.y_metric),
        show_legend: true,
        color_scheme: ColorScheme::Viridis,
        format: ExportFormat::Svg,
    };

    let result = generator.generate(&dataset, &config);

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/svg+xml")],
        result.data,
    ))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Generate a histogram
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn generate_histogram(
    Json(request): Json<HistogramRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let generator = ChartGenerator::new();
    let adapter = DataAdapter::new();

    let distribution = adapter.create_histogram(&request.values, request.bins.unwrap_or(20));
    let dataset = adapter.distribution_to_dataset(&distribution);

    let config = ChartConfig {
        chart_type: ChartType::Histogram,
        width: request.width.unwrap_or(800),
        height: request.height.unwrap_or(600),
        title: format!(
            "Distribution of {} (μ={:.2}, σ={:.2})",
            request.metric, distribution.mean, distribution.std_dev
        ),
        x_label: Some(request.metric),
        y_label: Some("Density".to_string()),
        show_legend: true,
        color_scheme: ColorScheme::Plasma,
        format: ExportFormat::Svg,
    };

    let result = generator.generate(&dataset, &config);

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/svg+xml")],
        result.data,
    ))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Generate a heatmap
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn generate_heatmap(
    Json(request): Json<HeatmapRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let generator = ChartGenerator::new();

    // Create dataset from matrix
    let dataset = create_heatmap_dataset(&request.matrix, &request.x_labels, &request.y_labels);

    let config = ChartConfig {
        chart_type: ChartType::Heatmap,
        width: request.width.unwrap_or(800),
        height: request.height.unwrap_or(600),
        title: request.title,
        x_label: None,
        y_label: None,
        show_legend: true,
        color_scheme: ColorScheme::Coolwarm,
        format: ExportFormat::Svg,
    };

    let result = generator.generate(&dataset, &config);

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/svg+xml")],
        result.data,
    ))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Generate a radar chart
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn generate_radar_chart(
    Json(request): Json<RadarRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let generator = ChartGenerator::new();
    let adapter = DataAdapter::new();

    let values_len = request.values.len();
    let radar_data = RadarData {
        dimensions: request.dimensions,
        values: request.values,
        max_values: request.max_values.unwrap_or_else(|| vec![1.0; values_len]),
    };

    let dataset = adapter.radar_to_dataset(&radar_data);

    let config = ChartConfig {
        chart_type: ChartType::Radar,
        width: request.width.unwrap_or(600),
        height: request.height.unwrap_or(600),
        title: request.title,
        x_label: None,
        y_label: None,
        show_legend: true,
        color_scheme: ColorScheme::Category10,
        format: ExportFormat::Svg,
    };

    let result = generator.generate(&dataset, &config);

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/svg+xml")],
        result.data,
    ))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get available chart types
pub async fn list_chart_types() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "chart_types": [
            {"id": "line", "name": "Line Chart", "description": "Time series visualization"},
            {"id": "bar", "name": "Bar Chart", "description": "Categorical comparison"},
            {"id": "scatter", "name": "Scatter Plot", "description": "Correlation visualization"},
            {"id": "heatmap", "name": "Heatmap", "description": "Matrix data visualization"},
            {"id": "histogram", "name": "Histogram", "description": "Distribution visualization"},
            {"id": "box_plot", "name": "Box Plot", "description": "Statistical summary"},
            {"id": "area", "name": "Area Chart", "description": "Cumulative data"},
            {"id": "radar", "name": "Radar Chart", "description": "Multi-dimensional data"},
        ]
    }))
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get available color schemes
pub async fn list_color_schemes() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "color_schemes": [
            {"id": "default", "name": "Default", "colors": ["#1f77b4", "#ff7f0e", "#2ca02c"]},
            {"id": "viridis", "name": "Viridis", "colors": ["#440154", "#31688e", "#35b779", "#fde725"]},
            {"id": "plasma", "name": "Plasma", "colors": ["#0d0887", "#7e03a8", "#cc4778", "#f89441"]},
            {"id": "coolwarm", "name": "Coolwarm", "colors": ["#d73027", "#fc8d59", "#91bfdb", "#4575b4"]},
            {"id": "category10", "name": "Category 10", "colors": ["#1f77b4", "#ff7f0e", "#2ca02c", "#d62728", "#9467bd"]},
            {"id": "category20", "name": "Category 20", "colors": ["#1f77b4", "#aec7e8", "#ff7f0e", "#ffbb78", "#2ca02c"]},
        ]
    }))
}

/// Generate a complete dashboard
pub async fn generate_dashboard(Json(request): Json<DashboardRequest>) -> Json<serde_json::Value> {
    // Generate multiple charts for dashboard
    let mut charts = Vec::new();

    for chart_config in request.charts {
        let chart_id = format!("chart_{}", charts.len());
        charts.push(serde_json::json!({
            "id": chart_id,
            "type": format!("{:?}", chart_config.chart_type).to_lowercase(),
            "title": chart_config.title,
            "config": chart_config,
        }));
    }

    Json(serde_json::json!({
            "dashboard": {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                "title": request.title,
                "layout": request.layout,
                "charts": charts,
            }
        }))
}

// Helper functions

fn parse_chart_type(s: &str) -> ChartType {
    match s.to_lowercase().as_str() {
        "line" => ChartType::Line,
        "bar" => ChartType::Bar,
        "scatter" => ChartType::Scatter,
        "heatmap" => ChartType::Heatmap,
        "histogram" => ChartType::Histogram,
        "box_plot" => ChartType::BoxPlot,
        "area" => ChartType::Area,
        "radar" => ChartType::Radar,
        _ => ChartType::Line,
    }
}

fn parse_format(s: &str) -> ExportFormat {
    match s.to_lowercase().as_str() {
        "svg" => ExportFormat::Svg,
        "png" => ExportFormat::Png,
        "json" => ExportFormat::Json,
        "html" => ExportFormat::Html,
        _ => ExportFormat::Svg,
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
fn generate_demo_dataset(metric: &str) -> Dataset {
    // Generate synthetic time series data
    let mut data = Vec::new();
    for i in 0..50 {
        let t = i as f64;
        let value = match metric {
            "fitness" => 0.5 + 0.3 * (t / 10.0).sin() + t * 0.01,
            "diversity" => 0.8 - t * 0.01 + 0.1 * (t / 5.0).cos(),
            "trust" => 0.6 + 0.2 * (t / 20.0).sin(),
            "cooperation" => 0.4 + 0.4 * (1.0 - (-t * 0.05).exp()),
            _ => 0.5 + 0.2 * (t / 10.0).sin(),
        };
        data.push(DataPoint {
            x: t,
            y: value.clamp(0.0, 1.0),
            label: None,
            metadata: None,
        });
    }

    Dataset {
        series: vec![DataSeries {
            name: metric.to_string(),
            data,
            color: None,
        }],
        annotations: vec![],
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
fn create_heatmap_dataset(
    matrix: &[Vec<f64>],
    x_labels: &[String],
    y_labels: &[String],
) -> Dataset {
    // Flatten matrix to points
    let mut points = Vec::new();
    for (y, row) in matrix.iter().enumerate() {
        for (x, &value) in row.iter().enumerate() {
            points.push(DataPoint {
                x: x as f64,
                y: y as f64,
                label: Some(format!(
                    "{} x {}",
                    x_labels.get(x).map_or("?", std::string::String::as_str),
                    y_labels.get(y).map_or("?", std::string::String::as_str)
                )),
                metadata: Some({
                    let mut m = HashMap::new();
                    m.insert("value".to_string(), value.to_string());
                    m
                }),
            });
        }
    }

    Dataset {
        series: vec![DataSeries {
            name: "Heatmap".to_string(),
            data: points,
            color: None,
        }],
        annotations: vec![],
    }
}

// Request/Response types

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Deserialize)]
/// `ComparisonRequest` data
pub struct ComparisonRequest {
    /// categories
    pub categories: Vec<String>,
    /// values
    pub values: Vec<f64>,
    /// errors
    pub errors: Option<Vec<f64>>,
    /// title
    pub title: String,
    /// width
    pub width: Option<u32>,
    /// height
    pub height: Option<u32>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Deserialize)]
/// `ScatterRequest` data
pub struct ScatterRequest {
    /// x metric
    pub x_metric: String,
    /// y metric
    pub y_metric: String,
    /// x values
    pub x_values: Vec<f64>,
    /// y values
    pub y_values: Vec<f64>,
    /// width
    pub width: Option<u32>,
    /// height
    pub height: Option<u32>,
}

#[derive(Debug, Deserialize)]
/// `HistogramRequest` data
pub struct HistogramRequest {
    /// metric
    pub metric: String,
    /// values
    pub values: Vec<f64>,
    /// bins
    pub bins: Option<usize>,
    /// width
    pub width: Option<u32>,
    /// height
    pub height: Option<u32>,
}

#[derive(Debug, Deserialize)]
/// `HeatmapRequest` data
pub struct HeatmapRequest {
    /// title
    pub title: String,
    /// matrix
    pub matrix: Vec<Vec<f64>>,
    /// x labels
    pub x_labels: Vec<String>,
    /// y labels
    pub y_labels: Vec<String>,
    /// width
    pub width: Option<u32>,
    /// height
    pub height: Option<u32>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Deserialize)]
/// `RadarRequest` data
pub struct RadarRequest {
    /// title
    pub title: String,
    /// dimensions
    pub dimensions: Vec<String>,
    /// values
    pub values: Vec<f64>,
    /// max values
    pub max_values: Option<Vec<f64>>,
    /// width
    pub width: Option<u32>,
    /// height
    pub height: Option<u32>,
}

#[derive(Debug, Deserialize)]
/// `DashboardRequest` data
pub struct DashboardRequest {
    /// title
    pub title: String,
    /// layout
    pub layout: String,
    /// charts
    pub charts: Vec<ChartConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_chart_types() {
        let response = list_chart_types().await;
        let chart_types = response.0["chart_types"].as_array().unwrap();
        assert_eq!(chart_types.len(), 8);
        assert_eq!(chart_types[0]["id"], "line");
    }

    #[tokio::test]
    async fn test_list_color_schemes() {
        let response = list_color_schemes().await;
        let schemes = response.0["color_schemes"].as_array().unwrap();
        assert_eq!(schemes.len(), 6);
        assert_eq!(schemes[0]["id"], "default");
    }
}
