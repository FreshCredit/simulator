//! Chart generator

use super::types::*;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Chart generator
#[derive(Debug)]
pub struct ChartGenerator;

impl ChartGenerator {
    #[must_use]
    /// new.
    pub const fn new() -> Self {
        Self
    }

    /// Generate line chart
    #[must_use]
    pub fn generate_line_chart(
        &self,
        config: &ChartConfig,
        series: &[ChartSeries],
    ) -> GeneratedChart {
        let svg = self.render_svg(config, series, "line");

        GeneratedChart {
            config: config.clone(),
            svg,
            series: series.to_vec(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate bar chart
    #[must_use]
    pub fn generate_bar_chart(
        &self,
        config: &ChartConfig,
        series: &[ChartSeries],
    ) -> GeneratedChart {
        let svg = self.render_svg(config, series, "bar");

        GeneratedChart {
            config: config.clone(),
            svg,
            series: series.to_vec(),
        }
    }

    /// Generate scatter plot
    #[must_use]
    pub fn generate_scatter(&self, config: &ChartConfig, series: &[ChartSeries]) -> GeneratedChart {
        let svg = self.render_svg(config, series, "scatter");

        GeneratedChart {
            config: config.clone(),
            svg,
            series: series.to_vec(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate histogram
    #[must_use]
    pub fn generate_histogram(
        &self,
        config: &ChartConfig,
        data: &[f64],
        bins: usize,
    ) -> GeneratedChart {
        let series = self.bin_data(data, bins);
        let svg = self.render_svg(config, &series, "histogram");

        GeneratedChart {
            config: config.clone(),
            svg,
            series,
        }
    }

    /// Generate heatmap
    #[must_use]
    pub fn generate_heatmap(&self, config: &ChartConfig, matrix: &[Vec<f64>]) -> GeneratedChart {
        let series = vec![];
        let svg = self.render_heatmap_svg(config, matrix);

        GeneratedChart {
            config: config.clone(),
            svg,
            series,
        }
    }

    /// Generate timeseries chart
    #[must_use]
    pub fn generate_timeseries(
        &self,
        config: &ChartConfig,
        series: &[ChartSeries],
    ) -> GeneratedChart {
        let mut ts_config = config.clone();
        ts_config.chart_type = ChartType::Timeseries;

        self.generate_line_chart(&ts_config, series)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate box plot
    #[must_use]
    pub fn generate_boxplot(
        &self,
        config: &ChartConfig,
        data: &[(String, Vec<f64>)],
    ) -> GeneratedChart {
        let series = data
            .iter()
            .map(|(name, values)| {
                let points = self.compute_boxplot_points(values);
                ChartSeries {
                    name: name.clone(),
                    data: points,
                    color: None,
                }
            })
            .collect::<Vec<_>>();

        let svg = self.render_svg(config, &series, "boxplot");

        GeneratedChart {
            config: config.clone(),
            svg,
            series,
        }
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    /// Export to SVG string
    #[must_use]
    pub fn export_svg(&self, chart: &GeneratedChart) -> String {
        chart.svg.clone()
    }

    fn render_svg(
        &self,
        config: &ChartConfig,
        _series: &[ChartSeries],
        _chart_type: &str,
    ) -> String {
        let dim = &config.dimensions;
        let w = dim.width;
        let h = dim.height;

        format!(
            r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
                <title>{}</title>
                <rect width="100%" height="100%" fill="white"/>
                <text x="{}" y="20" text-anchor="middle" font-size="16">{}</text>
                <text x="{}" y="{}" text-anchor="middle" font-size="12">{}</text>
                <text x="20" y="{}" text-anchor="middle" font-size="12" transform="rotate(-90 20 {})">{}</text>
            </svg>"#,
            w,
            h,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            config.title,
            w / 2,
            config.title,
            w / 2,
            h - 10,
            config.x_label,
            h / 2,
            h / 2,
            config.y_label
        )
    }

    fn render_heatmap_svg(&self, config: &ChartConfig, _matrix: &[Vec<f64>]) -> String {
        let dim = &config.dimensions;
        format!(
            r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
                <title>{}</title>
                <rect width="100%" height="100%" fill="white"/>
                <text x="{}" y="20" text-anchor="middle" font-size="16">{}</text>
            </svg>"#,
            dim.width,
            dim.height,
            config.title,
            dim.width / 2,
            config.title
        )
    }

    fn bin_data(&self, data: &[f64], bins: usize) -> Vec<ChartSeries> {
        if data.is_empty() || bins == 0 {
            return vec![];
        }

        let min = data.iter().copied().fold(f64::INFINITY, f64::min);
        let max = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        // Guard against division by zero when all values are equal (max == min)
        let range = max - min;
        // TAG: surface=simulation owner=research-team rule=SIM-001
        if range == 0.0 {
            // All values are identical - put them all in the first bin
            let mut counts = vec![0; bins];
            counts[0] = data.len();
            let points: Vec<ChartPoint> = counts
                .iter()
                .enumerate()
                .map(|(i, &count)| ChartPoint {
                    x: min + (i as f64 + 0.5),
                    y: count as f64,
                    label: None,
                })
                .collect();
            return vec![ChartSeries {
                name: "Distribution".to_string(),
                data: points,
                color: Some("#4A90D9".to_string()),
            }];
        }

        let bin_width = range / bins as f64;

        let mut counts = vec![0; bins];
        for &value in data {
            let bin = ((value - min) / bin_width).min(bins as f64 - 1.0) as usize;
            counts[bin] += 1;
        }

        let points: Vec<ChartPoint> = counts
            .iter()
            .enumerate()
            .map(|(i, &count)| ChartPoint {
                x: min + bin_width * (i as f64 + 0.5),
                y: count as f64,
                label: None,
            })
            .collect();

        vec![ChartSeries {
            name: "Frequency".to_string(),
            data: points,
            color: None,
        }]
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn compute_boxplot_points(&self, values: &[f64]) -> Vec<ChartPoint> {
        if values.is_empty() {
            return vec![];
        }

        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let n = sorted.len();
        let q1_idx = n / 4;
        let q2_idx = n / 2;
        let q3_idx = 3 * n / 4;

        vec![
            ChartPoint {
                x: 0.0,
                y: sorted[0],
                label: Some("min".to_string()),
            },
            ChartPoint {
                x: 1.0,
                y: sorted[q1_idx],
                label: Some("q1".to_string()),
            },
            ChartPoint {
                x: 2.0,
                y: sorted[q2_idx],
                label: Some("median".to_string()),
            },
            ChartPoint {
                x: 3.0,
                y: sorted[q3_idx],
                label: Some("q3".to_string()),
            },
            ChartPoint {
                x: 4.0,
                y: sorted[n - 1],
                label: Some("max".to_string()),
            },
        ]
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for ChartGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_chart_generation() {
        let generator = ChartGenerator;
        let config = ChartConfig::default();
        let series = vec![ChartSeries {
            name: "Test".to_string(),
            data: vec![
                ChartPoint {
                    x: 0.0,
                    y: 1.0,
                    label: None,
                },
                ChartPoint {
                    x: 1.0,
                    y: 2.0,
                    label: None,
                },
            ],
            color: None,
        }];

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let chart = generator.generate_line_chart(&config, &series);
        assert!(!chart.svg.is_empty());
        assert!(chart.svg.contains("<svg"));
    }

    #[test]
    fn test_histogram_generation() {
        let generator = ChartGenerator;
        let config = ChartConfig::default();
        let data = vec![1.0, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0];

        let chart = generator.generate_histogram(&config, &data, 4);
        assert!(!chart.svg.is_empty());
        assert!(!chart.series.is_empty());
    }

    #[test]
    fn test_bar_chart_generation() {
        let generator = ChartGenerator;
        let config = ChartConfig::default();
        let series = vec![ChartSeries {
            name: "Test".to_string(),
            data: vec![
                ChartPoint {
                    x: 0.0,
                    y: 5.0,
                    label: Some("A".to_string()),
                },
                ChartPoint {
                    x: 1.0,
                    y: 10.0,
                    label: Some("B".to_string()),
                },
            ],
            color: None,
        }];

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let chart = generator.generate_bar_chart(&config, &series);
        assert!(!chart.svg.is_empty());
        assert!(chart.svg.contains("<svg"));
    }

    #[test]
    fn test_scatter_chart_generation() {
        let generator = ChartGenerator;
        let config = ChartConfig::default();
        let series = vec![ChartSeries {
            name: "Test".to_string(),
            data: vec![
                ChartPoint {
                    x: 0.0,
                    y: 1.0,
                    label: None,
                },
                ChartPoint {
                    x: 1.0,
                    y: 2.0,
                    label: None,
                },
            ],
            color: None,
        }];

        let chart = generator.generate_scatter(&config, &series);
        assert!(!chart.svg.is_empty());
    }

    #[test]
    fn test_timeseries_generation() {
        let generator = ChartGenerator;
        let config = ChartConfig::default();
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let series = vec![ChartSeries {
            name: "Test".to_string(),
            data: vec![
                ChartPoint {
                    x: 0.0,
                    y: 1.0,
                    label: None,
                },
                ChartPoint {
                    x: 1.0,
                    y: 2.0,
                    label: None,
                },
            ],
            color: None,
        }];

        let chart = generator.generate_timeseries(&config, &series);
        assert!(!chart.svg.is_empty());
    }

    #[test]
    fn test_boxplot_generation() {
        let generator = ChartGenerator;
        let config = ChartConfig::default();
        let data = vec![("Group1".to_string(), vec![1.0, 2.0, 3.0, 4.0, 5.0])];

        let chart = generator.generate_boxplot(&config, &data);
        assert!(!chart.svg.is_empty());
    }

    #[test]
    fn test_heatmap_generation() {
        let generator = ChartGenerator;
        let config = ChartConfig::default();
        let matrix = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];

        let chart = generator.generate_heatmap(&config, &matrix);
        assert!(!chart.svg.is_empty());
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_export_svg() {
        let generator = ChartGenerator;
        let config = ChartConfig::default();
        let series = vec![];
        let chart = generator.generate_line_chart(&config, &series);

        let svg = generator.export_svg(&chart);
        assert!(!svg.is_empty());
        assert_eq!(svg, chart.svg);
    }

    #[test]
    fn test_histogram_empty_data() {
        let generator = ChartGenerator;
        let config = ChartConfig::default();
        let chart = generator.generate_histogram(&config, &[], 4);
        assert!(chart.series.is_empty());
    }

    #[test]
    fn test_histogram_uniform_data() {
        let generator = ChartGenerator;
        let config = ChartConfig::default();
        let data = vec![5.0, 5.0, 5.0, 5.0];

        let chart = generator.generate_histogram(&config, &data, 4);
        assert!(!chart.series.is_empty());
        assert_eq!(chart.series[0].data[0].y, 4.0);
    }
}
