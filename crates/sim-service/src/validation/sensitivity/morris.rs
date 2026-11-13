//! Morris screening method
//!
//! Elementary Effects method for parameter screening.
//! Computes mean (μ) and standard deviation (σ) of elementary effects
//! to identify important and interacting parameters.

use std::collections::HashMap;

use super::types::*;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Morris method analyzer
#[derive(Debug)]
pub struct MorrisAnalyzer {
    config: MorrisConfig,
}

impl MorrisAnalyzer {
    /// Create a new Morris analyzer
    #[must_use]
    pub const fn new(config: MorrisConfig) -> Self {
        Self { config }
    }

    /// Initialize empty elementary-effects storage.
    fn init_effects_storage(
        &self,
        outputs: &[String],
        parameters: &[SensitivityParameter],
    ) -> HashMap<String, HashMap<String, Vec<f64>>> {
        let mut storage = HashMap::new();
        for output in outputs {
            let mut param_effects = HashMap::new();
            for param in parameters {
                param_effects.insert(param.name.clone(), Vec::new());
            }
            storage.insert(output.clone(), param_effects);
        }
        storage
    }

    /// Generate a random base point within the parameter grid.
    fn generate_base_point<R: Rng>(
        &self,
        parameters: &[SensitivityParameter],
        rng: &mut R,
    ) -> HashMap<String, f64> {
        let num_levels = self.config.num_levels.max(2);
        let mut point = HashMap::new();
        for param in parameters {
            let lower = param.lower.min(param.upper);
            let upper = param.lower.max(param.upper);
            let level = rng.gen_range(0..num_levels);
            let value = lower + (upper - lower) * level as f64 / (num_levels - 1) as f64;
            point.insert(param.name.clone(), value);
        }
        point
    }

    /// Run a single trajectory, updating `elementary_effects` and `total_runs`.
    fn run_trajectory<F, R: Rng>(
        &self,
        parameters: &[SensitivityParameter],
        outputs: &[String],
        model: &F,
        elementary_effects: &mut HashMap<String, HashMap<String, Vec<f64>>>,
        total_runs: &mut usize,
        delta: f64,
        rng: &mut R,
    ) where
        F: Fn(&HashMap<String, f64>) -> HashMap<String, f64>,
    {
        let num_params = parameters.len();
        if num_params == 0 {
            return;
        }
        let num_levels = self.config.num_levels.max(2);
        let step_levels =
            ((delta * (num_levels - 1) as f64).round() as usize).clamp(1, num_levels - 1);
        let mut current_point = self.generate_base_point(parameters, rng);
        let mut param_order: Vec<usize> = (0..num_params).collect();
        param_order.shuffle(rng);

        let mut output1 = model(&current_point);
        *total_runs += 1;

        for param_idx in param_order {
            let param = &parameters[param_idx];
            let param_name = &param.name;

            // TAG: surface=simulation owner=research-team rule=SIM-001
            let lower = param.lower.min(param.upper);
            let upper = param.lower.max(param.upper);
            let curr_val = *current_point.get(param_name).unwrap_or(&lower);
            let current_level = self.value_to_level(curr_val, lower, upper);

            let can_increase = current_level + step_levels < num_levels;
            let can_decrease = current_level >= step_levels;

            let new_level = if can_increase && can_decrease {
                if rng.gen_bool(0.5) {
                    current_level + step_levels
                } else {
                    current_level - step_levels
                }
            } else if can_increase {
                current_level + step_levels
            } else if can_decrease {
                current_level - step_levels
            } else if current_level + 1 < num_levels {
                current_level + 1
            } else {
                current_level.saturating_sub(1)
            };

            let new_value = lower + (upper - lower) * new_level as f64 / (num_levels - 1) as f64;

            let mut perturbed_point = current_point.clone();
            perturbed_point.insert(param_name.clone(), new_value);

            let output2 = model(&perturbed_point);
            *total_runs += 1;

            let actual_delta = new_value - curr_val;

            if actual_delta.abs() > 1e-12 {
                for output in outputs {
                    let y1 = *output1.get(output).unwrap_or(&0.0);
                    let y2 = *output2.get(output).unwrap_or(&0.0);
                    let ee = (y2 - y1) / actual_delta * (upper - lower);

                    if ee.is_finite() {
                        if let Some(param_effects) = elementary_effects.get_mut(output) {
                            if let Some(effects) = param_effects.get_mut(param_name) {
                                effects.push(ee);
                            }
                        }
                    }
                }
            }

            current_point = perturbed_point;
            output1 = output2;
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Perform Morris screening analysis
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn analyze<F>(
        &self,
        parameters: &[SensitivityParameter],
        outputs: &[String],
        model: F,
        seed: Option<u64>,
    ) -> SensitivityResult
    where
        F: Fn(&HashMap<String, f64>) -> HashMap<String, f64>,
    {
        let start_time = std::time::Instant::now();
        let num_levels = self.config.num_levels.max(2);
        let delta = self
            .config
            .delta
            .unwrap_or_else(|| num_levels as f64 / (2.0 * (num_levels - 1) as f64));

        let mut rng = match seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_entropy(),
        };

        let mut elementary_effects = self.init_effects_storage(outputs, parameters);
        let mut total_runs = 0;

        for _ in 0..self.config.num_trajectories {
            self.run_trajectory(
                parameters,
                outputs,
                &model,
                &mut elementary_effects,
                &mut total_runs,
                delta,
                &mut rng,
            );
        }

        let output_results = self.compute_statistics(&elementary_effects, outputs, parameters);
        let parameter_ranking = self.compute_ranking(&output_results, parameters.len());

        SensitivityResult {
            method: SensitivityMethod::Morris,
            output_results,
            parameter_ranking,
            computation_time_secs: start_time.elapsed().as_secs_f64(),
            total_runs,
        }
    }

    /// Convert parameter value to grid level
    fn value_to_level(&self, value: f64, lower: f64, upper: f64) -> usize {
        let num_levels = self.config.num_levels.max(2);
        let low = lower.min(upper);
        let high = lower.max(upper);
        if high <= low || (high - low).abs() < 1e-12 {
            return 0;
        }
        let normalized = ((value - low) / (high - low)).clamp(0.0, 1.0);
        let level = (normalized * (num_levels - 1) as f64).round() as usize;
        level.min(num_levels - 1)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute Morris statistics (μ, σ) from elementary effects
    fn compute_statistics(
        &self,
        elementary_effects: &HashMap<String, HashMap<String, Vec<f64>>>,
        outputs: &[String],
        parameters: &[SensitivityParameter],
    ) -> HashMap<String, OutputSensitivity> {
        let mut output_results = HashMap::new();

        for output in outputs {
            let mut parameter_indices = HashMap::new();

            if let Some(param_effects) = elementary_effects.get(output) {
                for param in parameters {
                    if let Some(effects) = param_effects.get(&param.name) {
                        if !effects.is_empty() {
                            let k = effects.len();
                            // Mean of absolute elementary effects (μ*)
                            let mu_star = effects.iter().map(|e| e.abs()).sum::<f64>() / k as f64;

                            // Standard deviation (σ)
                            let mean = effects.iter().sum::<f64>() / k as f64;
                            let variance = if k > 1 {
                                effects.iter().map(|e| (e - mean).powi(2)).sum::<f64>()
                                    / (k - 1) as f64
                            } else {
                                0.0
                            };
                            let _sigma = variance.sqrt();

                            // Store μ* as primary index
                            let index = SensitivityIndex {
                                parameter: param.name.clone(),
                                value: mu_star,
                                index_type: IndexType::MorrisMu,
                                interpretation: SensitivityLevel::from_value(mu_star),
                            };
                            parameter_indices.insert(param.name.clone(), index);

                            // Store σ as MorrisSigma index
                            let sigma = variance.sqrt();
                            let sigma_index = SensitivityIndex {
                                parameter: format!("{}_sigma", param.name),
                                value: sigma,
                                index_type: IndexType::MorrisSigma,
                                interpretation: SensitivityLevel::from_value(sigma),
                            };
                            parameter_indices.insert(format!("{}_sigma", param.name), sigma_index);
                        }
                    }
                }
            }

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

        output_results
    }

    /// Compute parameter ranking based on Morris μ* (filtering out _sigma keys)
    fn compute_ranking(
        &self,
        output_results: &HashMap<String, OutputSensitivity>,
        _num_params: usize,
    ) -> Vec<ParameterRank> {
        rank_parameters_excluding(output_results, "_sigma")
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate Morris interpretation
    #[must_use]
    pub fn interpret(&self, mu: f64, sigma: f64) -> &'static str {
        if !mu.is_finite() || !sigma.is_finite() {
            return "Parameter has negligible effect";
        }
        let abs_mu = mu.abs();
        let abs_sigma = sigma.abs();
        // Morris classification based on |μ|-σ plot
        if abs_mu < 0.05 && abs_sigma < 0.05 {
            "Parameter has negligible effect"
        } else if abs_mu > 0.20 && abs_sigma < 0.10 {
            "Parameter has strong linear effect"
        } else if abs_mu > 0.20 && abs_sigma > 0.10 {
            "Parameter has strong non-linear or interaction effects"
        } else if abs_mu < 0.20 && abs_sigma > 0.10 {
            "Parameter has interaction effects but low individual influence"
        } else {
            "Parameter has moderate linear effect"
        }
    }

    /// Generate recommendations based on Morris results
    #[must_use]
    pub fn generate_recommendations(&self, result: &SensitivityResult) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Group parameters by Morris (μ*, σ) classification:
        // - negligible: μ* < 0.05
        // - linear: μ* >= 0.05 and σ < 0.10 (or σ / μ* < 0.35)
        // - interaction/nonlinear: μ* >= 0.05 and σ >= 0.10
        let mut linear_params = Vec::new();
        let mut interaction_params = Vec::new();
        let mut negligible_params = Vec::new();

        for rank in &result.parameter_ranking {
            let mu_star = rank.avg_sensitivity;
            let mut sorted_outputs: Vec<&String> = result.output_results.keys().collect();
            sorted_outputs.sort();
            let sigma_values: Vec<f64> = sorted_outputs
                .iter()
                .filter_map(|out_name| {
                    result.output_results.get(*out_name).and_then(|os| {
                        os.parameter_indices
                            .get(&format!("{}_sigma", rank.parameter))
                            .map(|idx| idx.value)
                    })
                })
                .collect();
            let sigma = if sigma_values.is_empty() {
                0.0
            } else {
                sigma_values.iter().sum::<f64>() / sigma_values.len() as f64
            };

            if mu_star < 0.05 {
                negligible_params.push(rank.parameter.clone());
            } else if sigma >= 0.10 && (mu_star <= 0.0 || sigma / mu_star >= 0.35) {
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                interaction_params.push(rank.parameter.clone());
            } else {
                linear_params.push(rank.parameter.clone());
            }
        }

        if !linear_params.is_empty() {
            recommendations.push(format!(
                "Parameters {} show strong linear effects. Prioritize for control.",
                linear_params.join(", ")
            ));
        }

        if !interaction_params.is_empty() {
            recommendations.push(format!(
                "Parameters {} may have interactions. Consider joint analysis.",
                interaction_params.join(", ")
            ));
        }

        if !negligible_params.is_empty() {
            recommendations.push(format!(
                "Parameters {} are negligible. Can be fixed at nominal values.",
                negligible_params.join(", ")
            ));
        }

        recommendations
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for MorrisAnalyzer {
    fn default() -> Self {
        Self::new(MorrisConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn linear_model(inputs: &HashMap<String, f64>) -> HashMap<String, f64> {
        let x1 = inputs.get("x1").copied().unwrap_or(0.0);
        let x2 = inputs.get("x2").copied().unwrap_or(0.0);
        let x3 = inputs.get("x3").copied().unwrap_or(0.0);

        let mut outputs = HashMap::new();
        // Linear model: y = 2*x1 + 0.5*x2 + 0.1*x3
        outputs.insert("y".to_string(), 2.0 * x1 + 0.5 * x2 + 0.1 * x3);
        outputs
    }

    fn nonlinear_model(inputs: &HashMap<String, f64>) -> HashMap<String, f64> {
        let x1 = inputs.get("x1").copied().unwrap_or(0.0);
        let x2 = inputs.get("x2").copied().unwrap_or(0.0);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let mut outputs = HashMap::new();
        // Nonlinear: y = x1^2 + x1*x2
        outputs.insert("y".to_string(), x1 * (x1 + x2));
        outputs
    }

    #[test]
    fn test_morris_linear() {
        let analyzer = MorrisAnalyzer::default();

        let params = vec![
            SensitivityParameter {
                name: "x1".to_string(),
                param_type: ParameterType::Continuous,
                baseline: 0.5,
                lower: 0.0,
                upper: 1.0,
                distribution: DistributionType::Uniform,
            },
            SensitivityParameter {
                name: "x2".to_string(),
                param_type: ParameterType::Continuous,
                baseline: 0.5,
                lower: 0.0,
                upper: 1.0,
                distribution: DistributionType::Uniform,
            },
            SensitivityParameter {
                name: "x3".to_string(),
                param_type: ParameterType::Continuous,
                baseline: 0.5,
                lower: 0.0,
                upper: 1.0,
                distribution: DistributionType::Uniform,
            },
        ];

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let result = analyzer.analyze(&params, &["y".to_string()], linear_model, Some(42));

        // x1 should be most important
        assert_eq!(result.parameter_ranking[0].parameter, "x1");
        // x3 should be least important
        assert_eq!(result.parameter_ranking[2].parameter, "x3");
    }

    #[test]
    fn test_morris_detects_interactions() {
        let analyzer = MorrisAnalyzer::default();

        let params = vec![
            SensitivityParameter {
                name: "x1".to_string(),
                param_type: ParameterType::Continuous,
                baseline: 0.5,
                lower: 0.0,
                upper: 1.0,
                distribution: DistributionType::Uniform,
            },
            SensitivityParameter {
                name: "x2".to_string(),
                param_type: ParameterType::Continuous,
                baseline: 0.5,
                lower: 0.0,
                upper: 1.0,
                distribution: DistributionType::Uniform,
            },
        ];

        let result = analyzer.analyze(&params, &["y".to_string()], nonlinear_model, Some(42));

        // Both parameters should be significant in nonlinear model
        assert!(result
            .parameter_ranking
            .iter()
            .all(|r| r.avg_sensitivity > 0.01));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_recommendations() {
        let analyzer = MorrisAnalyzer::default();

        let params = vec![
            SensitivityParameter {
                name: "important".to_string(),
                param_type: ParameterType::Continuous,
                baseline: 0.5,
                lower: 0.0,
                upper: 1.0,
                distribution: DistributionType::Uniform,
            },
            SensitivityParameter {
                name: "unimportant".to_string(),
                param_type: ParameterType::Continuous,
                baseline: 0.5,
                lower: 0.0,
                upper: 1.0,
                distribution: DistributionType::Uniform,
            },
        ];

        let result = analyzer.analyze(
            &params,
            &["y".to_string()],
            |inputs| {
                let imp = inputs.get("important").copied().unwrap_or(0.0);
                let _unimp = inputs.get("unimportant").copied().unwrap_or(0.0);
                let mut out = HashMap::new();
                // Only important affects output
                out.insert("y".to_string(), 10.0 * imp);
                out
            },
            Some(42),
        );

        let recs = analyzer.generate_recommendations(&result);
        assert!(!recs.is_empty());
    }
}
