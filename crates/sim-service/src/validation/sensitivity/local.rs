//! Local sensitivity analysis (One-at-a-time)
//!
//! Measures how small changes in each parameter affect the output,
//! holding all other parameters constant at their baseline values.

use std::collections::HashMap;

use super::types::*;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Local sensitivity analyzer
#[derive(Debug)]
pub struct LocalSensitivityAnalyzer {
    config: LocalSensitivityConfig,
}

impl LocalSensitivityAnalyzer {
    /// Create a new local sensitivity analyzer
    #[must_use]
    pub const fn new(config: LocalSensitivityConfig) -> Self {
        Self { config }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Analyze local sensitivity for given parameters
    pub fn analyze<F>(
        &self,
        parameters: &[SensitivityParameter],
        outputs: &[String],
        model: F,
    ) -> SensitivityResult
    where
        F: Fn(&HashMap<String, f64>) -> HashMap<String, f64>,
    {
        let start_time = std::time::Instant::now();
        let mut output_results: HashMap<String, OutputSensitivity> = HashMap::new();
        let mut total_runs = 0;

        // Initialize output results
        for output in outputs {
            output_results.insert(
                output.clone(),
                OutputSensitivity {
                    output_name: output.clone(),
                    parameter_indices: HashMap::new(),
                    r_squared: None,
                    confidence_intervals: HashMap::new(),
                },
            );
        }

        // Run baseline once
        let baseline_values: HashMap<String, f64> = parameters
            .iter()
            .map(|p| (p.name.clone(), p.baseline))
            .collect();
        let baseline_outputs = model(&baseline_values);
        total_runs += 1;

        // For each parameter, compute local sensitivity
        for param in parameters {
            let lower = param.lower.min(param.upper);
            let upper = param.lower.max(param.upper);
            let range = (upper - lower).max(1e-12);
            let delta = range * self.config.perturbation.clamp(1e-4, 0.5);

            let low_val = (param.baseline - delta).max(lower);
            let high_val = (param.baseline + delta).min(upper);

            // Evaluate at perturbed points
            let mut low_values = baseline_values.clone();
            let mut high_values = baseline_values.clone();

            low_values.insert(param.name.clone(), low_val);
            high_values.insert(param.name.clone(), high_val);

            let low_outputs = model(&low_values);
            let high_outputs = model(&high_values);
            total_runs += 2;

            let actual_step = (high_val - low_val).abs();
            let fwd_step = (high_val - param.baseline).abs();
            let bwd_step = (param.baseline - low_val).abs();

            // TAG: surface=simulation owner=research-team rule=SIM-001
            // Compute sensitivity coefficients
            for output in outputs {
                let baseline = *baseline_outputs.get(output).unwrap_or(&0.0);
                let low = *low_outputs.get(output).unwrap_or(&0.0);
                let high = *high_outputs.get(output).unwrap_or(&0.0);

                // Central difference approximation with boundary fallbacks
                let sensitivity = if self.config.central_differences {
                    if actual_step > 1e-12 {
                        (high - low) / actual_step
                    } else {
                        0.0
                    }
                } else if fwd_step > 1e-12 {
                    (high - baseline) / fwd_step
                } else if bwd_step > 1e-12 {
                    (baseline - low) / bwd_step
                } else {
                    0.0
                };

                // Normalized sensitivity (elasticity with zero-baseline scale preservation)
                let param_scale = if param.baseline.abs() > 1e-6 {
                    param.baseline.abs()
                } else {
                    range.max(1.0)
                };
                let output_scale = if baseline.abs() > 1e-6 {
                    baseline.abs()
                } else {
                    1.0
                };
                let normalized_sensitivity =
                    (sensitivity.abs() * param_scale / output_scale).min(100.0);

                let index = SensitivityIndex {
                    parameter: param.name.clone(),
                    value: normalized_sensitivity,
                    index_type: IndexType::LocalCoefficient,
                    interpretation: SensitivityLevel::from_value(normalized_sensitivity),
                };

                if let Some(output_sens) = output_results.get_mut(output) {
                    output_sens
                        .parameter_indices
                        .insert(param.name.clone(), index);
                }
            }
        }

        // Compute parameter ranking
        let parameter_ranking = self.compute_ranking(&output_results, parameters.len());

        SensitivityResult {
            method: SensitivityMethod::Local,
            output_results,
            parameter_ranking,
            computation_time_secs: start_time.elapsed().as_secs_f64(),
            total_runs,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Analyze with finite difference grid (more accurate)
    pub fn analyze_with_grid<F>(
        &self,
        parameters: &[SensitivityParameter],
        outputs: &[String],
        model: F,
    ) -> SensitivityResult
    where
        F: Fn(&HashMap<String, f64>) -> HashMap<String, f64>,
    {
        let start_time = std::time::Instant::now();
        let num_points = self.config.num_points.max(2);
        let mut output_results: HashMap<String, OutputSensitivity> = HashMap::new();
        let mut total_runs = 0;

        // Initialize
        for output in outputs {
            output_results.insert(
                output.clone(),
                OutputSensitivity {
                    output_name: output.clone(),
                    parameter_indices: HashMap::new(),
                    r_squared: None,
                    confidence_intervals: HashMap::new(),
                },
            );
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Baseline
        let baseline_values: HashMap<String, f64> = parameters
            .iter()
            .map(|p| (p.name.clone(), p.baseline))
            .collect();
        let baseline_outputs = model(&baseline_values);
        total_runs += 1;

        for param in parameters {
            let lower = param.lower.min(param.upper);
            let upper = param.lower.max(param.upper);
            let range = (upper - lower).max(1e-12);

            // Generate evaluation points
            let mut points = Vec::new();
            let mut outputs_at_points: HashMap<String, Vec<f64>> = HashMap::new();

            for i in 0..num_points {
                let fraction = i as f64 / (num_points - 1) as f64;
                let value = lower + range * fraction;

                let mut test_values = baseline_values.clone();
                test_values.insert(param.name.clone(), value);

                let result = model(&test_values);
                total_runs += 1;

                points.push(value);
                for output in outputs {
                    outputs_at_points
                        .entry(output.clone())
                        .or_default()
                        .push(*result.get(output).unwrap_or(&0.0));
                }
            }

            // Compute regression-based sensitivity
            for output in outputs {
                if let Some(y_values) = outputs_at_points.get(output) {
                    let slope = self.compute_linear_regression(&points, y_values);

                    // TAG: surface=simulation owner=research-team rule=SIM-001
                    let baseline = *baseline_values.get(&param.name).unwrap_or(&0.0);
                    let output_baseline = *baseline_outputs.get(output).unwrap_or(&0.0);

                    let param_scale = if baseline.abs() > 1e-6 {
                        baseline.abs()
                    } else {
                        range.max(1.0)
                    };
                    let output_scale = if output_baseline.abs() > 1e-6 {
                        output_baseline.abs()
                    } else {
                        1.0
                    };
                    let normalized = (slope.abs() * param_scale / output_scale).min(100.0);

                    let index = SensitivityIndex {
                        parameter: param.name.clone(),
                        value: normalized,
                        index_type: IndexType::StandardizedRegression,
                        interpretation: SensitivityLevel::from_value(normalized),
                    };

                    if let Some(output_sens) = output_results.get_mut(output) {
                        output_sens
                            .parameter_indices
                            .insert(param.name.clone(), index);
                    }
                }
            }
        }

        let parameter_ranking = self.compute_ranking(&output_results, parameters.len());

        SensitivityResult {
            method: SensitivityMethod::Local,
            output_results,
            parameter_ranking,
            computation_time_secs: start_time.elapsed().as_secs_f64(),
            total_runs,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute linear regression slope using centered deviations to eliminate numerical cancellation
    fn compute_linear_regression(&self, x: &[f64], y: &[f64]) -> f64 {
        let pairs: Vec<(f64, f64)> = x
            .iter()
            .zip(y.iter())
            .filter(|(&xi, &yi)| xi.is_finite() && yi.is_finite())
            .map(|(&xi, &yi)| (xi, yi))
            .collect();

        if pairs.len() < 2 {
            return 0.0;
        }

        let n = pairs.len() as f64;
        let mean_x = pairs.iter().map(|p| p.0).sum::<f64>() / n;
        let mean_y = pairs.iter().map(|p| p.1).sum::<f64>() / n;

        let mut ss_xx = 0.0;
        let mut ss_xy = 0.0;

        for (xi, yi) in pairs {
            let dx = xi - mean_x;
            let dy = yi - mean_y;
            ss_xx += dx * dx;
            ss_xy += dx * dy;
        }

        if ss_xx < 1e-12 {
            0.0
        } else {
            ss_xy / ss_xx
        }
    }

    /// Compute parameter ranking across all outputs
    fn compute_ranking(
        &self,
        output_results: &HashMap<String, OutputSensitivity>,
        _num_params: usize,
    ) -> Vec<ParameterRank> {
        rank_parameters(output_results)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate recommendations based on results
    #[must_use]
    pub fn generate_recommendations(&self, result: &SensitivityResult) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check for dominant parameters
        if let Some(top) = result.parameter_ranking.first() {
            if top.avg_sensitivity > 0.5 {
                recommendations.push(format!(
                    "Parameter '{}' dominates output variation ({}%). Consider tight control.",
                    top.parameter,
                    (top.avg_sensitivity * 100.0) as i32
                ));
            }
        }

        // Check for negligible parameters
        let negligible: Vec<_> = result
            .parameter_ranking
            .iter()
            .filter(|r| r.avg_sensitivity < 0.05)
            .map(|r| r.parameter.clone())
            .collect();

        if !negligible.is_empty() && negligible.len() < result.parameter_ranking.len() {
            recommendations.push(format!(
                "Parameters {} have negligible effect. Consider fixing at baseline values.",
                negligible.join(", ")
            ));
        }

        // Check parameter count efficiency
        let significant_count = result
            .parameter_ranking
            .iter()
            .filter(|r| r.significant_outputs > 0)
            .count();

        if significant_count < result.parameter_ranking.len() / 2 {
            recommendations.push(
                "Many parameters show low sensitivity. Consider reducing parameter space."
                    .to_string(),
            );
        }

        recommendations
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for LocalSensitivityAnalyzer {
    fn default() -> Self {
        Self::new(LocalSensitivityConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_model(inputs: &HashMap<String, f64>) -> HashMap<String, f64> {
        let x1 = inputs.get("x1").copied().unwrap_or(1.0);
        let x2 = inputs.get("x2").copied().unwrap_or(1.0);
        let x3 = inputs.get("x3").copied().unwrap_or(1.0);

        let mut outputs = HashMap::new();
        // y = 2*x1 + 0.5*x2 + 0.1*x3
        outputs.insert("y".to_string(), 2.0 * x1 + 0.5 * x2 + 0.1 * x3);
        outputs
    }

    #[test]
    fn test_local_sensitivity_ranking() {
        let analyzer = LocalSensitivityAnalyzer::default();

        let params = vec![
            SensitivityParameter {
                name: "x1".to_string(),
                // TAG: surface=simulation owner=research-team rule=SIM-001
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
            SensitivityParameter {
                name: "x3".to_string(),
                param_type: ParameterType::Continuous,
                baseline: 1.0,
                lower: 0.0,
                upper: 2.0,
                distribution: DistributionType::Uniform,
            },
        ];

        let result = analyzer.analyze(&params, &["y".to_string()], test_model);

        // x1 should be most sensitive (coefficient 2.0)
        assert_eq!(result.parameter_ranking[0].parameter, "x1");
        // x3 should be least sensitive (coefficient 0.1)
        assert_eq!(result.parameter_ranking[2].parameter, "x3");
    }

    #[test]
    fn test_recommendations() {
        let analyzer = LocalSensitivityAnalyzer::default();

        let params = vec![
            // TAG: surface=simulation owner=research-team rule=SIM-001
            SensitivityParameter {
                name: "dominant".to_string(),
                param_type: ParameterType::Continuous,
                baseline: 1.0,
                lower: 0.0,
                upper: 2.0,
                distribution: DistributionType::Uniform,
            },
            SensitivityParameter {
                name: "weak".to_string(),
                param_type: ParameterType::Continuous,
                baseline: 1.0,
                lower: 0.0,
                upper: 2.0,
                distribution: DistributionType::Uniform,
            },
        ];

        let result = analyzer.analyze(&params, &["y".to_string()], |inputs| {
            let dom = inputs.get("dominant").copied().unwrap_or(1.0);
            let weak = inputs.get("weak").copied().unwrap_or(1.0);
            let mut out = HashMap::new();
            // Dominant parameter has 10x coefficient
            out.insert("y".to_string(), 10.0 * dom + 0.1 * weak);
            out
        });

        let recs = analyzer.generate_recommendations(&result);
        assert!(!recs.is_empty());
        assert!(recs[0].contains("dominant"));
    }

    #[test]
    fn test_zero_baseline_and_boundary_forward_difference() {
        let mut config = LocalSensitivityConfig::default();
        config.central_differences = false; // test forward differences at boundary
        let analyzer = LocalSensitivityAnalyzer::new(config);

        let params = vec![
            SensitivityParameter {
                name: "zero_base".to_string(),
                param_type: ParameterType::Continuous,
                baseline: 0.0, // baseline at 0 must not collapse elasticity to 0
                lower: -1.0,
                upper: 1.0,
                distribution: DistributionType::Uniform,
            },
            SensitivityParameter {
                name: "at_upper_bound".to_string(),
                param_type: ParameterType::Continuous,
                baseline: 2.0, // baseline equals upper bound, forward step is 0
                lower: 0.0,
                upper: 2.0,
                distribution: DistributionType::Uniform,
            },
        ];

        let result = analyzer.analyze(&params, &["y".to_string()], |inputs| {
            let z = inputs.get("zero_base").copied().unwrap_or(0.0);
            let u = inputs.get("at_upper_bound").copied().unwrap_or(2.0);
            let mut out = HashMap::new();
            out.insert("y".to_string(), 5.0 * z + 3.0 * u);
            out
        });

        let y_res = &result.output_results["y"];
        assert!(y_res.parameter_indices["zero_base"].value > 0.0);
        assert!(y_res.parameter_indices["at_upper_bound"].value > 0.0);
    }
}
