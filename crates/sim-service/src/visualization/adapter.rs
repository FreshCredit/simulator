//! Data Adapter for Experiment Results (P1-SIM-4)
//!
//! Converts simulation results into visualization datasets.

use std::collections::HashMap;

use super::types::*;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Adapter for converting experiment results to visualization data
#[derive(Debug)]
pub struct DataAdapter;

impl DataAdapter {
    /// Create new data adapter
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Convert time series metrics to dataset
    #[must_use]
    pub fn time_series_to_dataset(&self, series: &[TimeSeriesData]) -> Dataset {
        let data_series: Vec<DataSeries> = series
            .iter()
            .map(|ts| DataSeries {
                name: ts.metric_name.clone(),
                data: ts
                    .timestamps
                    .iter()
                    .zip(ts.values.iter())
                    .map(|(&t, &v)| DataPoint {
                        x: t as f64,
                        y: v,
                        label: None,
                        metadata: None,
                    })
                    .collect(),
                color: None,
            })
            .collect();

        Dataset {
            series: data_series,
            annotations: vec![],
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    /// Convert comparison data to dataset
    #[must_use]
    pub fn comparison_to_dataset(&self, data: &ComparisonData) -> Dataset {
        let data_points: Vec<DataPoint> = data
            .categories
            .iter()
            .enumerate()
            .map(|(i, cat)| DataPoint {
                x: i as f64,
                y: data.values.get(i).copied().unwrap_or(0.0),
                label: Some(cat.clone()),
                metadata: None,
            })
            .collect();

        Dataset {
            series: vec![DataSeries {
                name: "Values".to_string(),
                data: data_points,
                color: None,
            }],
            annotations: vec![],
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Convert correlation data to dataset
    #[must_use]
    pub fn correlation_to_dataset(&self, data: &CorrelationData) -> Dataset {
        let data_points: Vec<DataPoint> = data
            .points
            .iter()
            .map(|(x, y)| DataPoint {
                x: *x,
                y: *y,
                label: None,
                metadata: None,
            })
            .collect();

        Dataset {
            series: vec![DataSeries {
                name: format!(
                    "{} vs {} (r={:.3})",
                    data.x_metric, data.y_metric, data.correlation
                ),
                data: data_points,
                color: None,
            }],
            annotations: vec![],
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Convert distribution data to dataset
    #[must_use]
    pub fn distribution_to_dataset(&self, data: &DistributionData) -> Dataset {
        let data_points: Vec<DataPoint> = data
            .bins
            .iter()
            .map(|bin| DataPoint {
                x: (bin.start + bin.end) / 2.0,
                y: bin.density,
                label: Some(format!("{:.2}-{:.2}", bin.start, bin.end)),
                metadata: None,
            })
            .collect();

        Dataset {
            series: vec![DataSeries {
                name: format!("Distribution (μ={:.2}, σ={:.2})", data.mean, data.std_dev),
                data: data_points,
                color: None,
            }],
            annotations: vec![],
        }
    }

    /// Convert radar data to dataset
    #[must_use]
    pub fn radar_to_dataset(&self, data: &RadarData) -> Dataset {
        let data_points: Vec<DataPoint> = data
            .dimensions
            .iter()
            .enumerate()
            .map(|(i, dim)| DataPoint {
                x: i as f64,
                y: data.values.get(i).copied().unwrap_or(0.0),
                label: Some(dim.clone()),
                metadata: None,
            })
            .collect();

        Dataset {
            series: vec![DataSeries {
                name: "Radar".to_string(),
                data: data_points,
                color: None,
            }],
            annotations: vec![],
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Convert raw metrics `HashMap` to time series
    #[must_use]
    pub fn metrics_to_time_series(
        &self,
        metrics: &HashMap<String, Vec<(u64, f64)>>,
    ) -> Vec<TimeSeriesData> {
        metrics
            .iter()
            .map(|(name, points)| {
                let values: Vec<f64> = points.iter().map(|(_, v)| *v).collect();
                let timestamps: Vec<u64> = points.iter().map(|(t, _)| *t).collect();

                TimeSeriesData {
                    metric_name: name.clone(),
                    timestamps,
                    values: values.clone(),
                    statistics: self.calculate_stats(&values),
                }
            })
            .collect()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create comparison data from metric averages
    #[must_use]
    pub fn create_comparison(
        &self,
        categories: &[String],
        values: &[f64],
        errors: Option<&[f64]>,
    ) -> ComparisonData {
        ComparisonData {
            categories: categories.to_vec(),
            values: values.to_vec(),
            errors: errors.map(<[f64]>::to_vec),
        }
    }

    /// Create correlation data from two series
    #[must_use]
    pub fn create_correlation(
        &self,
        x: &[f64],
        y: &[f64],
        x_name: &str,
        y_name: &str,
    ) -> CorrelationData {
        let points: Vec<(f64, f64)> = x.iter().zip(y.iter()).map(|(&a, &b)| (a, b)).collect();
        let correlation = self.calculate_correlation(x, y);
        let p_value = self.estimate_p_value(correlation, x.len());

        CorrelationData {
            x_metric: x_name.to_string(),
            y_metric: y_name.to_string(),
            points,
            correlation,
            p_value,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create histogram bins from data
    #[must_use]
    pub fn create_histogram(&self, data: &[f64], num_bins: usize) -> DistributionData {
        if data.is_empty() || num_bins == 0 {
            return DistributionData {
                bins: vec![],
                mean: 0.0,
                median: 0.0,
                std_dev: 0.0,
            };
        }

        let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let span = max - min;

        let (bin_width, num_active_bins) = if span <= 1e-12 || !span.is_finite() {
            (1.0, 1)
        } else {
            (span / num_bins as f64, num_bins)
        };

        let mut bins: Vec<Bin> = (0..num_active_bins)
            .map(|i| Bin {
                start: min + i as f64 * bin_width,
                end: min + (i + 1) as f64 * bin_width,
                count: 0,
                density: 0.0,
            })
            .collect();

        // Count values in bins
        for &value in data {
            let bin_index = if span <= 1e-12 || !span.is_finite() {
                0
            } else {
                let idx = ((value - min) / bin_width).floor() as usize;
                idx.min(num_active_bins - 1)
            };
            bins[bin_index].count += 1;
        }

        // Calculate density
        let total = data.len() as f64;
        for bin in &mut bins {
            bin.density = bin.count as f64 / (total * bin_width);
        }

        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = if data.len() > 1 {
            data.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / (data.len() - 1) as f64
        } else {
            0.0
        };
        let std_dev = variance.sqrt();

        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = if sorted.len() % 2 == 0 {
            (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };

        DistributionData {
            bins,
            mean,
            median,
            std_dev,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate statistics for time series
    fn calculate_stats(&self, values: &[f64]) -> TimeSeriesStats {
        if values.is_empty() {
            return TimeSeriesStats {
                mean: 0.0,
                std_dev: 0.0,
                min: 0.0,
                max: 0.0,
                trend: TrendDirection::Stable,
            };
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance =
            values.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        // Simple trend detection
        let trend = if values.len() < 2 {
            TrendDirection::Stable
        } else {
            let first_half: f64 =
                values[..values.len() / 2].iter().sum::<f64>() / (values.len() / 2) as f64;
            let second_half: f64 = values[values.len() / 2..].iter().sum::<f64>()
                / (values.len() - values.len() / 2) as f64;
            let denom = first_half.abs().max(1e-12);
            let change = (second_half - first_half) / denom;

            // TAG: surface=simulation owner=research-team rule=SIM-001
            if change > 0.1 {
                TrendDirection::Increasing
            } else if change < -0.1 {
                TrendDirection::Decreasing
            } else {
                TrendDirection::Stable
            }
        };

        TimeSeriesStats {
            mean,
            std_dev,
            min,
            max,
            trend,
        }
    }

    /// Calculate Pearson correlation coefficient
    fn calculate_correlation(&self, x: &[f64], y: &[f64]) -> f64 {
        if x.len() != y.len() || x.is_empty() {
            return 0.0;
        }

        let n = x.len() as f64;
        let mean_x = x.iter().sum::<f64>() / n;
        let mean_y = y.iter().sum::<f64>() / n;

        let mut numerator = 0.0;
        let mut denom_x = 0.0;
        let mut denom_y = 0.0;

        for i in 0..x.len() {
            let dx = x[i] - mean_x;
            let dy = y[i] - mean_y;
            numerator += dx * dy;
            denom_x += dx * dx;
            denom_y += dy * dy;
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let denominator = (denom_x * denom_y).sqrt();
        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }

    /// Estimate two-tailed p-value from Pearson correlation r and sample size n
    fn estimate_p_value(&self, r: f64, n: usize) -> f64 {
        if n < 3 || !r.is_finite() {
            return 1.0;
        }

        let r_clamped = r.clamp(-0.999999999999999, 0.999999999999999);
        let denom = (1.0 - r_clamped * r_clamped).max(1e-15);
        let t = r_clamped * ((n as f64 - 2.0) / denom).sqrt();

        let abs_t = t.abs();
        if abs_t == 0.0 {
            1.0
        } else {
            let p_two_tailed = 2.0 * (1.0 - crate::api::meta::normal_cdf_approx(abs_t));
            p_two_tailed.clamp(0.0, 1.0)
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for DataAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_to_time_series() {
        let adapter = DataAdapter::new();
        let mut metrics = HashMap::new();
        metrics.insert("fitness".to_string(), vec![(0, 0.5), (1, 0.6), (2, 0.7)]);

        let series = adapter.metrics_to_time_series(&metrics);
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].metric_name, "fitness");
        assert_eq!(series[0].values.len(), 3);
    }

    #[test]
    fn test_create_histogram() {
        let adapter = DataAdapter::new();
        let data = vec![1.0, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0];

        let histogram = adapter.create_histogram(&data, 4);
        assert_eq!(histogram.bins.len(), 4);
        assert!(histogram.mean > 0.0);
        assert!(histogram.std_dev > 0.0);
    }

    #[test]
    fn test_create_correlation() {
        let adapter = DataAdapter::new();
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let corr = adapter.create_correlation(&x, &y, "x", "y");
        assert!(corr.correlation > 0.99); // Perfect positive correlation
        assert_eq!(corr.points.len(), 5);
    }
    #[test]
    fn test_create_histogram_constant_data_and_zero_bins() {
        let adapter = DataAdapter::new();
        let const_data = vec![42.0, 42.0, 42.0];
        let hist = adapter.create_histogram(&const_data, 5);
        assert_eq!(hist.bins.len(), 1);
        assert_eq!(hist.bins[0].count, 3);
        assert!((hist.bins[0].density - 1.0).abs() < 1e-6);

        let zero_bins = adapter.create_histogram(&const_data, 0);
        assert!(zero_bins.bins.is_empty());
    }

    #[test]
    fn test_estimate_p_value_perfect_correlation_finite() {
        let adapter = DataAdapter::new();
        let p_perfect = adapter.estimate_p_value(1.0, 100);
        assert!(p_perfect.is_finite());
        assert!(p_perfect < 1e-6);

        let p_zero = adapter.estimate_p_value(0.0, 100);
        assert!((p_zero - 1.0).abs() < 1e-4);
    }
}
