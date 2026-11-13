//! Tornado diagram generator
//!
//! Visualizes parameter sensitivity by showing the range of output
//! variation when each parameter varies from its min to max.

use std::collections::HashMap;

use super::types::*;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Tornado diagram generator
#[derive(Debug)]
pub struct TornadoDiagram;

impl TornadoDiagram {
    /// Create a new tornado diagram generator
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Generate tornado diagram data for all outputs
    pub fn generate<F>(
        &self,
        parameters: &[SensitivityParameter],
        outputs: &[String],
        model: F,
    ) -> HashMap<String, TornadoData>
    where
        F: Fn(&HashMap<String, f64>) -> HashMap<String, f64>,
    {
        let mut results = HashMap::new();

        // Get baseline values
        let baseline_values: HashMap<String, f64> = parameters
            .iter()
            .map(|p| (p.name.clone(), p.baseline))
            .collect();

        // Evaluate baseline
        let baseline_outputs = model(&baseline_values);

        for output in outputs {
            let baseline_value = *baseline_outputs.get(output).unwrap_or(&0.0);
            let mut bars = Vec::new();

            // TAG: surface=simulation owner=research-team rule=SIM-001
            for param in parameters {
                let lower = param.lower.min(param.upper);
                let upper = param.lower.max(param.upper);

                // Evaluate at parameter bounds
                let mut low_values = baseline_values.clone();
                let mut high_values = baseline_values.clone();

                low_values.insert(param.name.clone(), lower);
                high_values.insert(param.name.clone(), upper);

                let low_result = model(&low_values);
                let high_result = model(&high_values);

                let low_val = *low_result.get(output).unwrap_or(&0.0);
                let high_val = *high_result.get(output).unwrap_or(&0.0);

                let swing = (high_val - low_val).abs();
                let relative_swing = if baseline_value == 0.0 {
                    swing
                } else {
                    swing / baseline_value.abs()
                };

                bars.push(TornadoBar {
                    parameter: param.name.clone(),
                    low_value: low_val,
                    high_value: high_val,
                    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                    swing,
                    relative_swing,
                });
            }

            // Sort bars by swing (descending) for tornado effect with deterministic tie-breaking
            bars.sort_by(|a, b| {
                b.swing
                    .partial_cmp(&a.swing)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.parameter.cmp(&b.parameter))
            });

            results.insert(
                output.clone(),
                TornadoData {
                    output_name: output.clone(),
                    bars,
                    baseline_value,
                    units: None,
                },
            );
        }

        results
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate tornado data with correlation coefficients
    pub fn generate_with_correlation<F>(
        &self,
        parameters: &[SensitivityParameter],
        outputs: &[String],
        model: F,
        num_samples: usize,
    ) -> (
        HashMap<String, TornadoData>,
        HashMap<String, OutputSensitivity>,
    )
    where
        F: Fn(&HashMap<String, f64>) -> HashMap<String, f64>,
    {
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};

        let mut tornado_data = HashMap::new();
        let mut output_results: HashMap<String, OutputSensitivity> = HashMap::new();

        if num_samples == 0 {
            return (tornado_data, output_results);
        }

        // Sample-based approach for correlation
        let mut rng = StdRng::seed_from_u64(42);

        for output in outputs {
            let mut param_samples: HashMap<String, Vec<f64>> = HashMap::new();
            let mut output_samples: Vec<f64> = Vec::new();

            // TAG: surface=simulation owner=research-team rule=SIM-001
            // Generate samples
            for _ in 0..num_samples {
                let mut sample_values = HashMap::new();
                for param in parameters {
                    let lower = param.lower.min(param.upper);
                    let upper = param.lower.max(param.upper);
                    let value = if (upper - lower).abs() < 1e-12 {
                        lower
                    } else {
                        rng.gen_range(lower..=upper)
                    };
                    sample_values.insert(param.name.clone(), value);
                    param_samples
                        .entry(param.name.clone())
                        .or_default()
                        .push(value);
                }

                let result = model(&sample_values);
                output_samples.push(*result.get(output).unwrap_or(&0.0));
            }

            // Compute correlations
            let baseline_value = output_samples.iter().sum::<f64>() / output_samples.len() as f64;
            let mut parameter_indices = HashMap::new();
            let mut bars = Vec::new();

            for param in parameters {
                if let Some(samples) = param_samples.get(&param.name) {
                    let correlation = self.pearson_correlation(samples, &output_samples);

                    // Tornado bar based on correlation and conditional partition averages
                    let low_partition: Vec<f64> = output_samples
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| samples[*i] <= param.baseline)
                        .map(|(_, &v)| v)
                        .collect();
                    let high_partition: Vec<f64> = output_samples
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| samples[*i] >= param.baseline)
                        .map(|(_, &v)| v)
                        .collect();

                    let low_val = if low_partition.is_empty() {
                        baseline_value
                    } else {
                        low_partition.iter().sum::<f64>() / low_partition.len() as f64
                    };
                    let high_val = if high_partition.is_empty() {
                        baseline_value
                    } else {
                        high_partition.iter().sum::<f64>() / high_partition.len() as f64
                    };

                    // TAG: surface=simulation owner=research-team rule=SIM-001
                    let swing = (high_val - low_val).abs();
                    let relative_swing = if baseline_value.abs() > 1e-12 {
                        swing / baseline_value.abs()
                    } else {
                        swing
                    };

                    bars.push(TornadoBar {
                        parameter: param.name.clone(),
                        low_value: low_val,
                        high_value: high_val,
                        swing,
                        relative_swing,
                    });

                    let index = SensitivityIndex {
                        parameter: param.name.clone(),
                        value: correlation.abs(),
                        index_type: IndexType::TornadoCorrelation,
                        interpretation: SensitivityLevel::from_value(correlation.abs()),
                    };
                    parameter_indices.insert(param.name.clone(), index);
                }
            }

            // Sort bars for tornado with deterministic tie-breaking
            bars.sort_by(|a, b| {
                b.swing
                    .partial_cmp(&a.swing)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.parameter.cmp(&b.parameter))
            });

            tornado_data.insert(
                output.clone(),
                TornadoData {
                    output_name: output.clone(),
                    bars,
                    baseline_value,
                    units: None,
                },
            );

            // TAG: surface=simulation owner=research-team rule=SIM-001
            output_results.insert(
                output.clone(),
                OutputSensitivity {
                    output_name: output.clone(),
                    parameter_indices,
                    r_squared: None,
                    confidence_intervals: HashMap::new(),
                },
            );
        }

        (tornado_data, output_results)
    }

    /// Compute Pearson correlation coefficient
    fn pearson_correlation(&self, x: &[f64], y: &[f64]) -> f64 {
        sim_core::stats::pearson_correlation_f64(x, y)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate spider plot data (for visualization)
    pub fn generate_spider_data<F>(
        &self,
        parameters: &[SensitivityParameter],
        outputs: &[String],
        model: F,
        num_points: usize,
    ) -> HashMap<String, HashMap<String, Vec<(f64, f64)>>>
    where
        F: Fn(&HashMap<String, f64>) -> HashMap<String, f64>,
    {
        let mut results: HashMap<String, HashMap<String, Vec<(f64, f64)>>> = HashMap::new();

        let baseline_values: HashMap<String, f64> = parameters
            .iter()
            .map(|p| (p.name.clone(), p.baseline))
            .collect();

        let num_pts = num_points.max(2);

        for output in outputs {
            let mut param_curves = HashMap::new();

            for param in parameters {
                let lower = param.lower.min(param.upper);
                let upper = param.lower.max(param.upper);
                let range = upper - lower;
                let mut curve = Vec::new();

                for i in 0..num_pts {
                    let fraction = i as f64 / (num_pts - 1) as f64;
                    let value = lower + range * fraction;

                    // TAG: surface=simulation owner=research-team rule=SIM-001
                    let mut test_values = baseline_values.clone();
                    test_values.insert(param.name.clone(), value);

                    let result = model(&test_values);
                    let output_value = *result.get(output).unwrap_or(&0.0);

                    curve.push((value, output_value));
                }

                param_curves.insert(param.name.clone(), curve);
            }

            results.insert(output.clone(), param_curves);
        }

        results
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute sensitivity result from tornado data
    #[must_use]
    pub fn to_sensitivity_result(
        &self,
        tornado_data: HashMap<String, TornadoData>,
        computation_time_secs: f64,
        total_runs: usize,
    ) -> SensitivityResult {
        let mut output_results: HashMap<String, OutputSensitivity> = HashMap::new();
        let mut param_totals: HashMap<String, Vec<f64>> = HashMap::new();

        let mut sorted_outputs: Vec<_> = tornado_data.into_iter().collect();
        sorted_outputs.sort_by(|a, b| a.0.cmp(&b.0));

        for (output_name, data) in sorted_outputs {
            let mut parameter_indices = HashMap::new();
            let mut sorted_bars = data.bars;
            sorted_bars.sort_by(|a, b| {
                b.swing
                    .total_cmp(&a.swing)
                    .then_with(|| a.parameter.cmp(&b.parameter))
            });

            for bar in &sorted_bars {
                let index = SensitivityIndex {
                    parameter: bar.parameter.clone(),
                    value: bar.relative_swing,
                    index_type: IndexType::TornadoCorrelation,
                    interpretation: SensitivityLevel::from_value(bar.relative_swing),
                };

                parameter_indices.insert(bar.parameter.clone(), index);

                param_totals
                    .entry(bar.parameter.clone())
                    .or_default()
                    .push(bar.relative_swing);
            }
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001

            output_results.insert(
                output_name.clone(),
                OutputSensitivity {
                    output_name,
                    parameter_indices,
                    r_squared: None,
                    confidence_intervals: HashMap::new(),
                },
            );
        }

        // Compute ranking
        let mut parameter_ranking: Vec<ParameterRank> = param_totals
            .into_iter()
            .map(|(param, values)| {
                let avg = values.iter().sum::<f64>() / values.len() as f64;
                ParameterRank {
                    parameter: param,
                    rank: 0,
                    avg_sensitivity: avg,
                    significant_outputs: values
                        .iter()
                        .filter(|&&v| {
                            SensitivityLevel::from_value(v) != SensitivityLevel::Negligible
                        })
                        .count(),
                }
            })
            .collect();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        parameter_ranking.sort_by(|a, b| {
            b.avg_sensitivity
                .total_cmp(&a.avg_sensitivity)
                .then_with(|| a.parameter.cmp(&b.parameter))
        });
        for (i, rank) in parameter_ranking.iter_mut().enumerate() {
            rank.rank = i + 1;
        }

        SensitivityResult {
            method: SensitivityMethod::Tornado,
            output_results,
            parameter_ranking,
            computation_time_secs,
            total_runs,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for TornadoDiagram {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::items_after_statements)]
mod tests {
    use super::*;

    fn test_model(inputs: &HashMap<String, f64>) -> HashMap<String, f64> {
        let x1 = inputs.get("x1").copied().unwrap_or(1.0);
        let x2 = inputs.get("x2").copied().unwrap_or(1.0);

        let mut outputs = HashMap::new();
        outputs.insert("y".to_string(), 3.0 * x1 + 1.0 * x2);
        outputs
    }

    #[test]
    fn test_tornado_generation() {
        let tornado = TornadoDiagram::new();

        let params = vec![
            SensitivityParameter {
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                name: "x1".to_string(),
                param_type: ParameterType::Continuous,
                baseline: 1.0,
                lower: 0.0,
                upper: 2.0,
                distribution: DistributionType::Uniform,
            },
            SensitivityParameter {
                name: "x2".to_string(),
                param_type: ParameterType::Continuous,
                baseline: 1.0,
                lower: 0.0,
                upper: 2.0,
                distribution: DistributionType::Uniform,
            },
        ];

        let result = tornado.generate(&params, &["y".to_string()], test_model);

        let data = result.get("y").expect("Should have y output");
        assert_eq!(data.bars.len(), 2);

        // x1 should have larger swing (coefficient 3.0 vs 1.0)
        assert!(data.bars[0].swing > data.bars[1].swing);
    }

    #[test]
    fn test_spider_data() {
        let tornado = TornadoDiagram::new();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let params = vec![SensitivityParameter {
            name: "x1".to_string(),
            param_type: ParameterType::Continuous,
            baseline: 1.0,
            lower: 0.0,
            upper: 2.0,
            distribution: DistributionType::Uniform,
        }];

        let result = tornado.generate_spider_data(&params, &["y".to_string()], test_model, 10);

        let output_data = result.get("y").expect("Should have y");
        let x1_curve = output_data.get("x1").expect("Should have x1 curve");

        assert_eq!(x1_curve.len(), 10);
        // Check monotonicity (positive coefficient)
        for i in 1..x1_curve.len() {
            assert!(x1_curve[i].1 >= x1_curve[i - 1].1);
        }
    }

    #[test]
    fn test_correlation_calculation() {
        let tornado = TornadoDiagram::new();

        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let r = tornado.pearson_correlation(&x, &y);
        assert!(r > 0.99); // Perfect positive correlation

        let y_neg = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        let r_neg = tornado.pearson_correlation(&x, &y_neg);
        assert!(r_neg < -0.99); // Perfect negative correlation
    }
}
