//! Data adapter for visualization

use super::types::*;
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Data adapter for converting simulation data to chart format
#[derive(Debug)]
pub struct DataAdapter;

impl DataAdapter {
    #[must_use]
    /// new.
    pub const fn new() -> Self {
        Self
    }

    /// Adapt time series data
    #[must_use]
    pub fn adapt_timeseries(&self, data: &[(u64, f64)], series_name: &str) -> ChartSeries {
        let points: Vec<ChartPoint> = data
            .iter()
            .map(|(t, v)| ChartPoint {
                x: *t as f64,
                y: *v,
                label: None,
            })
            .collect();

        ChartSeries {
            name: series_name.to_string(),
            data: points,
            color: None,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Adapt multiple series
    #[must_use]
    pub fn adapt_multiple_series(
        &self,
        data: &HashMap<String, Vec<(u64, f64)>>,
    ) -> Vec<ChartSeries> {
        data.iter()
            .map(|(name, series_data)| self.adapt_timeseries(series_data, name))
            .collect()
    }

    /// Adapt distribution data
    pub fn adapt_distribution(&self, values: &[f64], bins: usize) -> Vec<ChartSeries> {
        if values.is_empty() || bins == 0 {
            return vec![];
        }

        let min = values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        if min == max {
            return vec![ChartSeries {
                name: "Distribution".to_string(),
                data: vec![ChartPoint {
                    x: min,
                    y: values.len() as f64,
                    label: None,
                }],
                color: None,
            }];
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        }

        let bin_width = (max - min) / bins as f64;
        let mut counts = vec![0; bins];

        for &value in values {
            let bin = ((value - min) / bin_width).floor().min(bins as f64 - 1.0) as usize;
            counts[bin] += 1;
        }

        let points: Vec<ChartPoint> = counts
            .iter()
            .enumerate()
            .map(|(i, &count)| ChartPoint {
                x: min + bin_width * (i as f64 + 0.5),
                y: count as f64,
                label: Some(format!(
                    "{:.2}-{:.2}",
                    min + bin_width * i as f64,
                    min + bin_width * (i as f64 + 1.0)
                )),
            })
            .collect();

        vec![ChartSeries {
            name: "Frequency".to_string(),
            data: points,
            color: None,
        }]
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Adapt correlation data for scatter plot
    #[must_use]
    pub fn adapt_scatter(
        &self,
        x_values: &[f64],
        y_values: &[f64],
        labels: Option<&[String]>,
        series_name: &str,
    ) -> ChartSeries {
        let points: Vec<ChartPoint> = x_values
            .iter()
            .zip(y_values.iter())
            .enumerate()
            .map(|(i, (x, y))| ChartPoint {
                x: *x,
                y: *y,
                label: labels.and_then(|l| l.get(i).cloned()),
            })
            .collect();

        ChartSeries {
            name: series_name.to_string(),
            data: points,
            color: None,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Adapt comparison data for bar chart
    #[must_use]
    pub fn adapt_comparison(
        &self,
        categories: &[String],
        values: &[f64],
        series_name: &str,
    ) -> ChartSeries {
        let points: Vec<ChartPoint> = categories
            .iter()
            .zip(values.iter())
            .enumerate()
            .map(|(i, (cat, val))| ChartPoint {
                x: i as f64,
                y: *val,
                label: Some(cat.clone()),
            })
            .collect();

        ChartSeries {
            name: series_name.to_string(),
            data: points,
            color: None,
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
    fn test_adapt_timeseries() {
        let adapter = DataAdapter;
        let data = vec![(0, 1.0), (1, 2.0), (2, 1.5)];

        let series = adapter.adapt_timeseries(&data, "Test");
        assert_eq!(series.data.len(), 3);
        assert_eq!(series.name, "Test");
    }

    #[test]
    fn test_adapt_distribution() {
        let adapter = DataAdapter;
        let values = vec![1.0, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0];
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        let series = adapter.adapt_distribution(&values, 4);
        assert!(!series.is_empty());
        assert_eq!(series[0].data.len(), 4);
    }

    #[test]
    fn test_adapt_distribution_empty() {
        let adapter = DataAdapter;
        let series = adapter.adapt_distribution(&[], 4);
        assert!(series.is_empty());
    }

    #[test]
    fn test_adapt_distribution_single_value() {
        let adapter = DataAdapter;
        let series = adapter.adapt_distribution(&[5.0], 4);
        assert!(!series.is_empty());
        assert_eq!(series[0].data.len(), 1);
    }

    #[test]
    fn test_adapt_scatter() {
        let adapter = DataAdapter;
        let x = vec![1.0, 2.0, 3.0];
        let y = vec![10.0, 20.0, 30.0];

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let series = adapter.adapt_scatter(&x, &y, None, "Scatter");
        assert_eq!(series.data.len(), 3);
        assert_eq!(series.name, "Scatter");
    }

    #[test]
    fn test_adapt_comparison() {
        let adapter = DataAdapter;
        let categories = vec!["A".to_string(), "B".to_string()];
        let values = vec![10.0, 20.0];

        let series = adapter.adapt_comparison(&categories, &values, "Comparison");
        assert_eq!(series.data.len(), 2);
        assert_eq!(series.data[0].label, Some("A".to_string()));
    }

    #[test]
    fn test_adapt_multiple_series() {
        let adapter = DataAdapter;
        let mut data = std::collections::HashMap::new();
        data.insert("Series1".to_string(), vec![(0, 1.0), (1, 2.0)]);
        data.insert("Series2".to_string(), vec![(0, 3.0), (1, 4.0)]);

        let series = adapter.adapt_multiple_series(&data);
        assert_eq!(series.len(), 2);
    }
}
