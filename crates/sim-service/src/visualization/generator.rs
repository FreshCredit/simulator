//! Chart Generator for Experiment Results (P1-SIM-4)
//!
//! Generates SVG charts from simulation data.

use super::types::*;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Chart generator for creating visualizations
#[derive(Debug)]
pub struct ChartGenerator;

impl ChartGenerator {
    /// Create new chart generator
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Generate chart from dataset
    #[must_use]
    pub fn generate(&self, dataset: &Dataset, config: &ChartConfig) -> ChartResult {
        match config.chart_type {
            ChartType::Line => self.generate_line_chart(dataset, config),
            ChartType::Bar => self.generate_bar_chart(dataset, config),
            ChartType::Scatter => self.generate_scatter_plot(dataset, config),
            ChartType::Heatmap => self.generate_heatmap(dataset, config),
            ChartType::Histogram => self.generate_histogram(dataset, config),
            ChartType::BoxPlot => self.generate_box_plot(dataset, config),
            ChartType::Area => self.generate_area_chart(dataset, config),
            ChartType::Radar => self.generate_radar_chart(dataset, config),
        }
    }

    /// Generate line chart (SVG)
    #[allow(clippy::too_many_lines)]
    fn generate_line_chart(&self, dataset: &Dataset, config: &ChartConfig) -> ChartResult {
        let margin = 60;
        let width = config.width as i32;
        let height = config.height as i32;
        let chart_width = width - 2 * margin;
        let chart_height = height - 2 * margin;

        // Find data ranges
        let (x_min, x_max, y_min, y_max) = self.calculate_ranges(dataset);
        let x_range = (x_max - x_min).max(1e-12);
        let y_range = (y_max - y_min).max(1e-12);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
            <defs>
                <style>
                    .title {{ font: bold 16px sans-serif; }}
                    .axis-label {{ font: 12px sans-serif; }}
                    .tick-label {{ font: 10px sans-serif; }}
                    .legend {{ font: 11px sans-serif; }}
                    .line {{ fill: none; stroke-width: 2; }}
                    .grid {{ stroke: #e0e0e0; stroke-width: 1; }}
                    .axis {{ stroke: #333; stroke-width: 1; }}
                </style>
            </defs>
            <rect width="100%" height="100%" fill="white"/>
            <text x="{}" y="30" text-anchor="middle" class="title">{}</text>
        "#,
            width,
            height,
            width,
            height,
            width / 2,
            escape_xml(&config.title)
        );

        // Draw grid lines
        for i in 0..=5 {
            let y = margin + (chart_height * i) / 5;
            svg.push_str(&format!(
                r#"<line x1="{}" y1="{}" x2="{}" y2="{}" class="grid"/>"#,
                margin,
                y,
                margin + chart_width,
                y
            ));
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Draw axes
        svg.push_str(&format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" class="axis"/>
            <line x1="{}" y1="{}" x2="{}" y2="{}" class="axis"/>"#,
            margin,
            margin + chart_height,
            margin + chart_width,
            margin + chart_height,
            margin,
            margin,
            margin,
            margin + chart_height
        ));

        // Axis labels
        if let Some(ref y_label) = config.y_label {
            svg.push_str(&format!(
                r#"<text x="20" y="{}" text-anchor="middle" transform="rotate(-90 20 {})" class="axis-label">{}</text>"#,
                height / 2, height / 2, escape_xml(y_label)
            ));
        }
        if let Some(ref x_label) = config.x_label {
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" text-anchor="middle" class="axis-label">{}</text>"#,
                width / 2,
                height - 15,
                escape_xml(x_label)
            ));
        }

        // Draw data series
        let colors = self.get_colors(&config.color_scheme);
        for (i, series) in dataset.series.iter().enumerate() {
            let color = series
                .color
                .as_ref()
                .unwrap_or(&colors[i % colors.len()])
                .clone();

            // TAG: surface=simulation owner=research-team rule=SIM-001
            // Generate path
            let mut path = String::from("M");
            for (j, point) in series.data.iter().enumerate() {
                let x = margin + ((point.x - x_min) / x_range * chart_width as f64) as i32;
                let y = margin + chart_height
                    - ((point.y - y_min) / y_range * chart_height as f64) as i32;
                if j == 0 {
                    path.push_str(&format!("{x},{y} "));
                } else {
                    path.push_str(&format!("L{x},{y} "));
                }
            }

            svg.push_str(&format!(
                r#"<path d="{path}" class="line" stroke="{color}"/>"#
            ));

            // Draw points
            for point in &series.data {
                let x = margin + ((point.x - x_min) / x_range * chart_width as f64) as i32;
                let y = margin + chart_height
                    - ((point.y - y_min) / y_range * chart_height as f64) as i32;
                svg.push_str(&format!(
                    r#"<circle cx="{x}" cy="{y}" r="3" fill="{color}"/>"#
                ));
            }
        }

        // Legend
        if config.show_legend && !dataset.series.is_empty() {
            let legend_x = width - margin + 10;
            let mut legend_y = margin;
            for (i, series) in dataset.series.iter().enumerate() {
                // TAG: surface=simulation owner=research-team rule=SIM-001
                let color = series.color.as_ref().unwrap_or(&colors[i % colors.len()]);
                svg.push_str(&format!(
                    r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="2"/>
                    <text x="{}" y="{}" class="legend">{}</text>"#,
                    legend_x,
                    legend_y + 5,
                    legend_x + 20,
                    legend_y + 5,
                    color,
                    legend_x + 25,
                    legend_y + 5,
                    escape_xml(&series.name)
                ));
                legend_y += 20;
            }
        }

        svg.push_str("</svg>");

        ChartResult {
            format: ExportFormat::Svg,
            data: svg.into_bytes(),
            mime_type: "image/svg+xml".to_string(),
            width: config.width,
            height: config.height,
        }
    }

    /// Generate bar chart
    fn generate_bar_chart(&self, dataset: &Dataset, config: &ChartConfig) -> ChartResult {
        let margin = 60;
        let width = config.width as i32;
        let height = config.height as i32;
        let chart_width = width - 2 * margin;
        let chart_height = height - 2 * margin;

        let (_, _, y_min, y_max) = self.calculate_ranges(dataset);
        let y_range = (y_max - y_min).max(1e-12);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
            <defs>
                <style>
                    .title {{ font-family: sans-serif; font-size: 16px; font-weight: bold; }}
                    .axis-label {{ font-family: sans-serif; font-size: 12px; }}
                    .tick-label {{ font-family: sans-serif; font-size: 10px; }}
                    .bar {{ stroke: none; }}
                    .axis {{ stroke: #333; stroke-width: 1; }}
                </style>
            </defs>
            <rect width="100%" height="100%" fill="white"/>
            <text x="{}" y="30" text-anchor="middle" class="title">{}</text>
        "#,
            width,
            height,
            width,
            height,
            width / 2,
            escape_xml(&config.title)
        );

        // Draw axes
        svg.push_str(&format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" class="axis"/>
            <line x1="{}" y1="{}" x2="{}" y2="{}" class="axis"/>"#,
            margin,
            margin + chart_height,
            margin + chart_width,
            margin + chart_height,
            margin,
            margin,
            margin,
            margin + chart_height
        ));

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Draw bars
        let colors = self.get_colors(&config.color_scheme);
        let bar_count = dataset.series.iter().map(|s| s.data.len()).sum::<usize>();
        let bar_width = (chart_width / (bar_count as i32 + 1)).max(1);

        let mut x_pos = margin + bar_width / 2;
        for (i, series) in dataset.series.iter().enumerate() {
            let color = series
                .color
                .as_ref()
                .unwrap_or(&colors[i % colors.len()])
                .clone();

            for point in &series.data {
                let bar_height = ((point.y - y_min) / y_range * chart_height as f64) as i32;
                let y = margin + chart_height - bar_height;

                svg.push_str(&format!(
                    r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" class="bar"/>"#,
                    x_pos - bar_width / 4,
                    y,
                    bar_width / 2,
                    bar_height,
                    color
                ));

                x_pos += bar_width;
            }
        }

        svg.push_str("</svg>");

        ChartResult {
            format: ExportFormat::Svg,
            data: svg.into_bytes(),
            mime_type: "image/svg+xml".to_string(),
            width: config.width,
            height: config.height,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate scatter plot
    fn generate_scatter_plot(&self, dataset: &Dataset, config: &ChartConfig) -> ChartResult {
        let margin = 60;
        let width = config.width as i32;
        let height = config.height as i32;
        let chart_width = width - 2 * margin;
        let chart_height = height - 2 * margin;

        let (x_min, x_max, y_min, y_max) = self.calculate_ranges(dataset);
        let x_range = (x_max - x_min).max(1e-12);
        let y_range = (y_max - y_min).max(1e-12);

        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
            <defs>
                <style>
                    .title {{ font-family: sans-serif; font-size: 16px; font-weight: bold; }}
                    .axis-label {{ font-family: sans-serif; font-size: 12px; }}
                    .axis {{ stroke: #333; stroke-width: 1; }}
                </style>
            </defs>
            <rect width="100%" height="100%" fill="white"/>
            <text x="{}" y="30" text-anchor="middle" class="title">{}</text>
        "#,
            width,
            height,
            width,
            height,
            width / 2,
            escape_xml(&config.title)
        );

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Draw axes
        svg.push_str(&format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" class="axis"/>
            <line x1="{}" y1="{}" x2="{}" y2="{}" class="axis"/>"#,
            margin,
            margin + chart_height,
            margin + chart_width,
            margin + chart_height,
            margin,
            margin,
            margin,
            margin + chart_height
        ));

        // Draw scatter points
        let colors = self.get_colors(&config.color_scheme);
        for (i, series) in dataset.series.iter().enumerate() {
            let color = series
                .color
                .as_ref()
                .unwrap_or(&colors[i % colors.len()])
                .clone();

            for point in &series.data {
                let x = margin + ((point.x - x_min) / x_range * chart_width as f64) as i32;
                let y = margin + chart_height
                    - ((point.y - y_min) / y_range * chart_height as f64) as i32;
                svg.push_str(&format!(
                    r#"<circle cx="{x}" cy="{y}" r="4" fill="{color}" opacity="0.6"/>"#
                ));
            }
        }

        svg.push_str("</svg>");

        ChartResult {
            format: ExportFormat::Svg,
            data: svg.into_bytes(),
            mime_type: "image/svg+xml".to_string(),
            width: config.width,
            height: config.height,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate heatmap
    fn generate_heatmap(&self, _dataset: &Dataset, config: &ChartConfig) -> ChartResult {
        // Simplified heatmap - just a placeholder
        let svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\">\
                <rect width=\"100%\" height=\"100%\" fill=\"#f5f5f5\"/>\
                <text x=\"50%\" y=\"50%\" text-anchor=\"middle\">\
                    Heatmap: {}\
                </text>\
            </svg>",
            config.width,
            config.height,
            escape_xml(&config.title)
        );

        ChartResult {
            format: ExportFormat::Svg,
            data: svg.into_bytes(),
            mime_type: "image/svg+xml".to_string(),
            width: config.width,
            height: config.height,
        }
    }

    /// Generate histogram
    fn generate_histogram(&self, _dataset: &Dataset, config: &ChartConfig) -> ChartResult {
        let svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\">\
                <rect width=\"100%\" height=\"100%\" fill=\"white\"/>\
                <text x=\"50%\" y=\"50%\" text-anchor=\"middle\">\
                    Histogram: {}\
                </text>\
            </svg>",
            config.width,
            config.height,
            escape_xml(&config.title)
        );

        // TAG: surface=simulation owner=research-team rule=SIM-001
        ChartResult {
            format: ExportFormat::Svg,
            data: svg.into_bytes(),
            mime_type: "image/svg+xml".to_string(),
            width: config.width,
            height: config.height,
        }
    }

    /// Generate box plot
    fn generate_box_plot(&self, _dataset: &Dataset, config: &ChartConfig) -> ChartResult {
        let svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\">\
                <rect width=\"100%\" height=\"100%\" fill=\"white\"/>\
                <text x=\"50%\" y=\"50%\" text-anchor=\"middle\">\
                    Box Plot: {}\
                </text>\
            </svg>",
            config.width,
            config.height,
            escape_xml(&config.title)
        );

        ChartResult {
            format: ExportFormat::Svg,
            data: svg.into_bytes(),
            mime_type: "image/svg+xml".to_string(),
            width: config.width,
            height: config.height,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate area chart
    fn generate_area_chart(&self, dataset: &Dataset, config: &ChartConfig) -> ChartResult {
        // Similar to line chart but with filled area
        let margin = 60;
        let width = config.width as i32;
        let height = config.height as i32;
        let chart_width = width - 2 * margin;
        let chart_height = height - 2 * margin;

        let (x_min, x_max, y_min, y_max) = self.calculate_ranges(dataset);
        let x_range = (x_max - x_min).max(1e-12);
        let y_range = (y_max - y_min).max(1e-12);

        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
            <defs>
                <style>
                    .title {{ font-family: sans-serif; font-size: 16px; font-weight: bold; }}
                    .area {{ stroke: none; opacity: 0.3; }}
                    .line {{ fill: none; stroke-width: 2; }}
                    .axis {{ stroke: #333; stroke-width: 1; }}
                </style>
            </defs>
            <rect width="100%" height="100%" fill="white"/>
            <text x="{}" y="30" text-anchor="middle" class="title">{}</text>
        "#,
            width,
            height,
            width,
            height,
            width / 2,
            escape_xml(&config.title)
        );

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Draw axes
        svg.push_str(&format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" class="axis"/>
            <line x1="{}" y1="{}" x2="{}" y2="{}" class="axis"/>"#,
            margin,
            margin + chart_height,
            margin + chart_width,
            margin + chart_height,
            margin,
            margin,
            margin,
            margin + chart_height
        ));

        // Draw areas
        let colors = self.get_colors(&config.color_scheme);
        for (i, series) in dataset.series.iter().enumerate() {
            let color = series
                .color
                .as_ref()
                .unwrap_or(&colors[i % colors.len()])
                .clone();

            let mut path = String::from("M");
            for (j, point) in series.data.iter().enumerate() {
                let x = margin + ((point.x - x_min) / x_range * chart_width as f64) as i32;
                let y = margin + chart_height
                    - ((point.y - y_min) / y_range * chart_height as f64) as i32;
                if j == 0 {
                    path.push_str(&format!("{x},{y} "));
                } else {
                    path.push_str(&format!("L{x},{y} "));
                }
            }
            // Close the path for area fill
            let last_x = margin
                + series
                    .data
                    .last()
                    .map_or(0, |p| ((p.x - x_min) / x_range * chart_width as f64) as i32);
            let first_x = margin
                + series
                    .data
                    .first()
                    .map_or(0, |p| ((p.x - x_min) / x_range * chart_width as f64) as i32);
            path.push_str(&format!(
                "L{} {} L{} {} Z",
                last_x,
                margin + chart_height,
                first_x,
                margin + chart_height
            ));

            // TAG: surface=simulation owner=research-team rule=SIM-001
            svg.push_str(&format!(
                r#"<path d="{path}" class="area" fill="{color}"/>"#
            ));
        }

        svg.push_str("</svg>");

        ChartResult {
            format: ExportFormat::Svg,
            data: svg.into_bytes(),
            mime_type: "image/svg+xml".to_string(),
            width: config.width,
            height: config.height,
        }
    }

    /// Generate radar chart
    fn generate_radar_chart(&self, _dataset: &Dataset, config: &ChartConfig) -> ChartResult {
        let svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\">\
                <rect width=\"100%\" height=\"100%\" fill=\"white\"/>\
                <text x=\"50%\" y=\"50%\" text-anchor=\"middle\">\
                    Radar Chart: {}\
                </text>\
            </svg>",
            config.width,
            config.height,
            escape_xml(&config.title)
        );

        // TAG: surface=simulation owner=research-team rule=SIM-001
        ChartResult {
            format: ExportFormat::Svg,
            data: svg.into_bytes(),
            mime_type: "image/svg+xml".to_string(),
            width: config.width,
            height: config.height,
        }
    }

    /// Calculate data ranges
    fn calculate_ranges(&self, dataset: &Dataset) -> (f64, f64, f64, f64) {
        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;

        for series in &dataset.series {
            for point in &series.data {
                if point.x.is_finite() && point.y.is_finite() {
                    x_min = x_min.min(point.x);
                    x_max = x_max.max(point.x);
                    y_min = y_min.min(point.y);
                    y_max = y_max.max(point.y);
                }
            }
        }

        if !x_min.is_finite() || !x_max.is_finite() || x_max < x_min {
            x_min = 0.0;
            x_max = 1.0;
        }
        if !y_min.is_finite() || !y_max.is_finite() || y_max < y_min {
            y_min = 0.0;
            y_max = 1.0;
        }

        // If span is zero (constant series), expand by 1.0
        let span_x = (x_max - x_min).abs();
        let span_y = (y_max - y_min).abs();
        if span_x < 1e-12 {
            x_min -= 0.5;
            x_max += 0.5;
        }
        if span_y < 1e-12 {
            y_min -= 0.5;
            y_max += 0.5;
        }

        // Add padding
        let x_padding = (x_max - x_min) * 0.05;
        let y_padding = (y_max - y_min) * 0.05;

        (
            x_min - x_padding,
            x_max + x_padding,
            y_min - y_padding,
            y_max + y_padding,
        )
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get color palette
    fn get_colors(&self, scheme: &ColorScheme) -> Vec<String> {
        match scheme {
            ColorScheme::Viridis => vec![
                "#440154".to_string(),
                "#31688e".to_string(),
                "#35b779".to_string(),
                "#fde725".to_string(),
            ],
            ColorScheme::Plasma => vec![
                "#0d0887".to_string(),
                "#7e03a8".to_string(),
                "#cc4778".to_string(),
                "#f89441".to_string(),
            ],
            ColorScheme::Coolwarm => vec![
                "#d73027".to_string(),
                "#fc8d59".to_string(),
                "#91bfdb".to_string(),
                "#4575b4".to_string(),
            ],
            ColorScheme::Category10 => vec![
                "#1f77b4".to_string(),
                "#ff7f0e".to_string(),
                "#2ca02c".to_string(),
                "#d62728".to_string(),
                "#9467bd".to_string(),
                "#8c564b".to_string(),
                "#e377c2".to_string(),
                "#7f7f7f".to_string(),
                "#bcbd22".to_string(),
                "#17becf".to_string(),
            ],
            _ => vec![
                "#1f77b4".to_string(),
                "#ff7f0e".to_string(),
                "#2ca02c".to_string(),
                "#d62728".to_string(),
                "#9467bd".to_string(),
                "#8c564b".to_string(),
            ],
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for ChartGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Escape XML special characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_dataset() -> Dataset {
        Dataset {
            series: vec![DataSeries {
                name: "Series 1".to_string(),
                data: vec![
                    DataPoint {
                        x: 0.0,
                        y: 10.0,
                        label: None,
                        metadata: None,
                    },
                    DataPoint {
                        x: 1.0,
                        y: 20.0,
                        label: None,
                        metadata: None,
                    },
                    DataPoint {
                        // TAG: surface=simulation owner=research-team rule=SIM-001
                        x: 2.0,
                        y: 15.0,
                        label: None,
                        metadata: None,
                    },
                ],
                color: None,
            }],
            annotations: vec![],
        }
    }

    #[test]
    fn test_generate_line_chart() {
        let generator = ChartGenerator::new();
        let dataset = create_test_dataset();
        let config = ChartConfig::default();

        let result = generator.generate(&dataset, &config);
        assert_eq!(result.format, ExportFormat::Svg);
        assert!(!result.data.is_empty());
        assert_eq!(result.mime_type, "image/svg+xml");
    }

    #[test]
    fn test_generate_bar_chart() {
        let generator = ChartGenerator::new();
        let dataset = create_test_dataset();
        let config = ChartConfig {
            chart_type: ChartType::Bar,
            ..Default::default()
        };

        let result = generator.generate(&dataset, &config);
        assert_eq!(result.format, ExportFormat::Svg);
        assert!(!result.data.is_empty());
    }

    #[test]
    fn test_generate_charts_empty_and_constant_series() {
        let generator = ChartGenerator::new();
        let empty_dataset = Dataset {
            series: vec![],
            annotations: vec![],
        };
        let config = ChartConfig::default();

        let res_line = generator.generate(&empty_dataset, &config);
        assert!(!res_line.data.is_empty());

        let const_dataset = Dataset {
            series: vec![DataSeries {
                name: "Constant".to_string(),
                data: vec![
                    DataPoint {
                        x: 1.0,
                        y: 5.0,
                        label: None,
                        metadata: None,
                    },
                    DataPoint {
                        x: 1.0,
                        y: 5.0,
                        label: None,
                        metadata: None,
                    },
                ],
                color: None,
            }],
            annotations: vec![],
        };
        let res_const = generator.generate(&const_dataset, &config);
        let svg_str = String::from_utf8(res_const.data).unwrap();
        assert!(!svg_str.contains("NaN"));
        assert!(!svg_str.contains("Infinity"));
    }
}
