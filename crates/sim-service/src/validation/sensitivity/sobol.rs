//! Sobol sensitivity analysis
//!
//! Variance-based decomposition for global sensitivity analysis.
//! Computes first-order (S1) and total-order (ST) indices.

use std::collections::HashMap;

use super::types::*;
use rand::rngs::StdRng;
use rand::SeedableRng;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Sobol analyzer
#[derive(Debug)]
pub struct SobolAnalyzer {
    config: SobolConfig,
}

impl SobolAnalyzer {
    /// Create a new Sobol analyzer
    #[must_use]
    pub const fn new(config: SobolConfig) -> Self {
        Self { config }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Perform Sobol sensitivity analysis
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
        let n = self.config.n;
        let _num_params = parameters.len();

        if parameters.is_empty() || n == 0 {
            return SensitivityResult {
                method: SensitivityMethod::Sobol,
                output_results: HashMap::new(),
                parameter_ranking: Vec::new(),
                computation_time_secs: 0.0,
                total_runs: 0,
            };
        }

        let mut rng = match seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_entropy(),
        };

        // Generate two independent sample matrices (N x k)
        let matrix_a = super::sobol_sampling::generate_sample_matrix(n, parameters, &mut rng);
        let matrix_b = super::sobol_sampling::generate_sample_matrix(n, parameters, &mut rng);

        // Evaluate model for matrices A and B
        let outputs_a = super::sobol_sampling::evaluate_matrix(&matrix_a, &model);
        let outputs_b = super::sobol_sampling::evaluate_matrix(&matrix_b, &model);
        let mut total_runs = 2 * n;

        // Generate and evaluate C_i = A_B^{(i)} once per parameter (Saltelli et al. 2010)
        let mut outputs_c_by_param = HashMap::with_capacity(parameters.len());
        for param in parameters {
            let matrix_c =
                super::sobol_sampling::generate_c_matrix(&matrix_a, &matrix_b, &param.name);
            let outputs_c = super::sobol_sampling::evaluate_matrix(&matrix_c, &model);
            total_runs += n;
            outputs_c_by_param.insert(param.name.clone(), outputs_c);
        }

        // Compute Sobol indices for each output
        let mut output_results = HashMap::new();

        for output in outputs {
            let values_a: Vec<f64> = outputs_a
                .iter()
                .map(|r| *r.get(output).unwrap_or(&0.0))
                .collect();
            let values_b: Vec<f64> = outputs_b
                .iter()
                .map(|r| *r.get(output).unwrap_or(&0.0))
                .collect();

            let mut s1_indices = HashMap::new();
            let mut st_indices = HashMap::new();
            let mut confidence_intervals = HashMap::new();

            for param in parameters {
                let Some(outputs_c) = outputs_c_by_param.get(&param.name) else {
                    continue;
                };
                let values_c: Vec<f64> = outputs_c
                    .iter()
                    .map(|r| *r.get(output).unwrap_or(&0.0))
                    .collect();

                // Compute first-order index S1 (Saltelli 2010: Y(B) × (Y(C_i) - Y(A)))
                let s1 = self.compute_s1(&values_b, &values_a, &values_c, n);
                let s1_index = SensitivityIndex {
                    parameter: param.name.clone(),
                    value: s1.clamp(0.0, 1.0),
                    index_type: IndexType::SobolFirstOrder,
                    interpretation: SensitivityLevel::from_value(s1),
                };
                s1_indices.insert(param.name.clone(), s1_index);

                // Compute total-order index ST (Jansen 1999 / Saltelli 2010 eq. f: (Y(A) - Y(C_i))^2 / (2N))
                let st = self.compute_st(&values_a, &values_b, &values_c, n);
                let st_index = SensitivityIndex {
                    parameter: param.name.clone(),
                    value: st.clamp(0.0, 1.0),
                    index_type: IndexType::SobolTotal,
                    interpretation: SensitivityLevel::from_value(st),
                };
                st_indices.insert(param.name.clone(), st_index);

                // Bootstrap confidence intervals
                if self.config.num_bootstrap > 0 {
                    let (s1_lower, s1_upper) = super::sobol_sampling::bootstrap_confidence_interval(
                        &values_b,
                        &values_a,
                        &values_c,
                        n,
                        self.config.num_bootstrap,
                        |b, a, c, n| self.compute_s1(b, a, c, n),
                        &mut rng,
                    );
                    confidence_intervals.insert(
                        format!("{}_S1", param.name),
                        (s1_lower.max(0.0), s1_upper.min(1.0)),
                    );

                    let (st_lower, st_upper) = super::sobol_sampling::bootstrap_confidence_interval(
                        &values_a,
                        &values_b,
                        &values_c,
                        n,
                        self.config.num_bootstrap,
                        |a, b, c, n| self.compute_st(a, b, c, n),
                        &mut rng,
                    );
                    confidence_intervals
                        .insert(param.name.clone(), (st_lower.max(0.0), st_upper.min(1.0)));
                }
            }

            // TAG: surface=simulation owner=research-team rule=SIM-001
            // Combine S1 and ST indices (sorted for SIM-004 determinism)
            let mut sorted_st_params: Vec<&String> = st_indices.keys().collect();
            sorted_st_params.sort();
            let mut combined_indices = HashMap::new();
            for param_name in &sorted_st_params {
                if let Some(st_index) = st_indices.get(*param_name) {
                    combined_indices.insert((*param_name).clone(), st_index.clone());
                }
            }
            let mut sorted_s1_params: Vec<&String> = s1_indices.keys().collect();
            sorted_s1_params.sort();
            for param_name in &sorted_s1_params {
                if let Some(s1_index) = s1_indices.get(*param_name) {
                    combined_indices.insert(format!("{param_name}_S1"), s1_index.clone());
                }
            }

            // Compute R² (sum of S1 should be <= 1, interaction = 1 - sum(S1))
            let sum_s1: f64 = sorted_s1_params
                .iter()
                .filter_map(|k| s1_indices.get(*k).map(|i| i.value))
                .sum();
            let r_squared = Some(sum_s1.min(1.0));

            output_results.insert(
                output.clone(),
                OutputSensitivity {
                    output_name: output.clone(),
                    parameter_indices: combined_indices,
                    r_squared,
                    confidence_intervals,
                },
            );
        }

        let parameter_ranking = self.compute_ranking(&output_results, parameters.len());

        SensitivityResult {
            method: SensitivityMethod::Sobol,
            output_results,
            parameter_ranking,
            computation_time_secs: start_time.elapsed().as_secs_f64(),
            total_runs,
        }
    }

    /// Generate sample matrix (N x k)
    #[must_use]
    pub fn generate_sample_matrix<R: rand::Rng>(
        &self,
        n: usize,
        parameters: &[SensitivityParameter],
        rng: &mut R,
    ) -> Vec<HashMap<String, f64>> {
        super::sobol_sampling::generate_sample_matrix(n, parameters, rng)
    }

    /// Compute first-order Sobol index (S1) using Saltelli (2002, 2010) & Owen (2013) centered estimator
    #[must_use]
    pub fn compute_s1(&self, b: &[f64], a: &[f64], c: &[f64], n: usize) -> f64 {
        super::sobol_sampling::compute_s1(b, a, c, n)
    }

    /// Compute total-order Sobol index (ST) using Jansen (1999) / Saltelli et al. (2010, eq. f) estimator
    #[must_use]
    pub fn compute_st(&self, a: &[f64], b: &[f64], c: &[f64], n: usize) -> f64 {
        super::sobol_sampling::compute_st(a, b, c, n)
    }

    /// Compute parameter ranking (ranking only original parameters, excluding _S1 suffixes)
    fn compute_ranking(
        &self,
        output_results: &HashMap<String, OutputSensitivity>,
        _num_params: usize,
    ) -> Vec<ParameterRank> {
        rank_parameters_excluding(output_results, "_S1")
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate Sobol-specific recommendations
    #[must_use]
    pub fn generate_recommendations(&self, result: &SensitivityResult) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check for parameters with ST > S1 (sorted for SIM-004 determinism)
        let mut interaction_detected = false;
        let mut sorted_outputs: Vec<&String> = result.output_results.keys().collect();
        sorted_outputs.sort();
        for out_key in sorted_outputs {
            if let Some(output_sens) = result.output_results.get(out_key) {
                let mut sorted_params: Vec<&String> =
                    output_sens.parameter_indices.keys().collect();
                sorted_params.sort();
                for param_name in sorted_params {
                    if let Some(index) = output_sens.parameter_indices.get(param_name) {
                        if index.index_type == IndexType::SobolTotal {
                            let s1_key = format!("{param_name}_S1");
                            if let Some(s1_index) = output_sens.parameter_indices.get(&s1_key) {
                                let interaction_gap = index.value - s1_index.value;
                                if interaction_gap > 0.15 {
                                    interaction_detected = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            if interaction_detected {
                break;
            }
        }

        if interaction_detected {
            recommendations.push(
                "Significant parameter interactions detected. Consider interaction terms in model."
                    .to_string(),
            );
        }

        // Check for dominant parameters
        if let Some(top) = result.parameter_ranking.first() {
            if top.avg_sensitivity > 0.50 {
                recommendations.push(format!(
                    "Parameter '{}' explains {}% of output variance.",
                    top.parameter,
                    (top.avg_sensitivity * 100.0) as i32
                ));
            }
        }

        recommendations
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for SobolAnalyzer {
    fn default() -> Self {
        Self::new(SobolConfig::default())
    }
}

#[cfg(test)]
#[allow(clippy::items_after_statements)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    fn g_function_model(inputs: &HashMap<String, f64>) -> HashMap<String, f64> {
        // Sobol G-function (standard test function)
        let a = [0.0, 0.5, 3.0, 9.0, 99.0]; // Coefficients
        let x: Vec<f64> = inputs.values().copied().collect();

        let mut y = 1.0;
        for (i, xi) in x.iter().enumerate() {
            let ai = a.get(i).copied().unwrap_or(1.0);
            y *= (xi.abs() * 2.0 - 1.0 + ai) / (1.0 + ai);
        }

        let mut outputs = HashMap::new();
        outputs.insert("y".to_string(), y);
        outputs
    }

    fn linear_additive_model(inputs: &HashMap<String, f64>) -> HashMap<String, f64> {
        // Additive model: y = x1 + 2*x2 + 3*x3
        let x1 = inputs.get("x1").copied().unwrap_or(0.0);
        let x2 = inputs.get("x2").copied().unwrap_or(0.0);
        let x3 = inputs.get("x3").copied().unwrap_or(0.0);

        let mut outputs = HashMap::new();
        outputs.insert("y".to_string(), x1 + 2.0 * x2 + 3.0 * x3);
        outputs
    }

    #[test]
    fn test_sobol_linear_model() {
        let config = SobolConfig {
            n: 8192, // Larger sample for stable variance estimates
            calc_second_order: false,
            num_bootstrap: 0,
        };
        let analyzer = SobolAnalyzer::new(config);

        // TAG: surface=simulation owner=research-team rule=SIM-001
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

        let result = analyzer.analyze(&params, &["y".to_string()], linear_additive_model, Some(42));

        // For an additive model y = x1 + 2*x2 + 3*x3 with uniform inputs,
        // all first-order indices should be in [0, 1] and sum to ~1.
        // x3 (coefficient 3.0) should have the largest sensitivity,
        // x1 (coefficient 1.0) should have the smallest.
        let output_result = result.output_results.get("y").unwrap();
        let st_x1 = output_result.parameter_indices.get("x1").unwrap().value;
        let st_x2 = output_result.parameter_indices.get("x2").unwrap().value;
        let st_x3 = output_result.parameter_indices.get("x3").unwrap().value;

        let s1_x1 = output_result.parameter_indices.get("x1_S1").unwrap().value;
        let s1_x2 = output_result.parameter_indices.get("x2_S1").unwrap().value;
        let s1_x3 = output_result.parameter_indices.get("x3_S1").unwrap().value;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!((0.0..=1.0).contains(&s1_x1), "S1 x1 in [0, 1]: {s1_x1}");
        assert!((0.0..=1.0).contains(&s1_x2), "S1 x2 in [0, 1]: {s1_x2}");
        assert!((0.0..=1.0).contains(&s1_x3), "S1 x3 in [0, 1]: {s1_x3}");

        // For additive model y = x1 + 2*x2 + 3*x3:
        // x3 (coeff 3.0, Var=9) > x2 (coeff 2.0, Var=4) > x1 (coeff 1.0, Var=1)
        assert!(
            s1_x3 > s1_x2,
            "S1: x3 should be > x2: x3={s1_x3}, x2={s1_x2}"
        );
        assert!(
            s1_x2 > s1_x1,
            "S1: x2 should be > x1: x2={s1_x2}, x1={s1_x1}"
        );
        assert!(
            st_x3 > st_x2,
            "ST: x3 should be > x2: x3={st_x3}, x2={st_x2}"
        );
        assert!(
            st_x2 > st_x1,
            "ST: x2 should be > x1: x2={st_x2}, x1={st_x1}"
        );

        // Parameter ranking should only rank the 3 true parameters, not _S1 duplicates
        assert_eq!(result.parameter_ranking.len(), 3);
        assert_eq!(result.parameter_ranking[0].parameter, "x3");

        // For an additive model with no interactions, sum of S1 ≈ 1.0
        let s1_sum = s1_x1 + s1_x2 + s1_x3;
        assert!(
            s1_sum > 0.8 && s1_sum <= 1.05,
            "Sum of first-order indices for additive model should be ≈ 1.0, got {s1_sum}"
        );
    }

    #[test]
    fn test_sobol_sample_matrix_generation() {
        let analyzer = SobolAnalyzer::default();
        let params = vec![SensitivityParameter {
            name: "x".to_string(),
            param_type: ParameterType::Continuous,
            baseline: 0.5,
            lower: 0.0,
            upper: 1.0,
            distribution: DistributionType::Uniform,
        }];

        // TAG: surface=simulation owner=research-team rule=SIM-001
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        let mut rng = StdRng::seed_from_u64(42);

        let matrix = analyzer.generate_sample_matrix(100, &params, &mut rng);

        assert_eq!(matrix.len(), 100);
        assert!(matrix.iter().all(|m| {
            let v = *m.get("x").unwrap();
            (0.0..=1.0).contains(&v)
        }));
    }

    #[test]
    fn test_sobol_indices_sum() {
        // For additive model, sum of S1 should be close to 1
        let config = SobolConfig {
            n: 2048,
            calc_second_order: false,
            num_bootstrap: 0,
        };
        let analyzer = SobolAnalyzer::new(config);

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

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let result = analyzer.analyze(
            &params,
            &["y".to_string()],
            |inputs| {
                let x1 = inputs.get("x1").copied().unwrap_or(0.0);
                let x2 = inputs.get("x2").copied().unwrap_or(0.0);
                let mut out = HashMap::new();
                out.insert("y".to_string(), x1 + x2); // Additive
                out
            },
            Some(42),
        );

        // For additive model, sum of S1 ≈ 1
        if let Some(output) = result.output_results.get("y") {
            let sum_s1: f64 = output
                .parameter_indices
                .values()
                .filter(|i| matches!(i.index_type, IndexType::SobolFirstOrder))
                .map(|i| i.value)
                .sum();
            // Allow some numerical error
            assert!(sum_s1 <= 1.1, "Sum of S1 should be <= 1.1, got {sum_s1}");
        }
    }

    #[test]
    fn test_sobol_shift_invariance() {
        // Metamorphic testing: S1(Y + 1_000_000.0) == S1(Y)
        let config = SobolConfig {
            n: 2048,
            calc_second_order: false,
            num_bootstrap: 0,
        };
        let analyzer = SobolAnalyzer::new(config);

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

        let base_res = analyzer.analyze(
            &params,
            &["y".to_string()],
            |inputs| {
                let x1 = inputs.get("x1").copied().unwrap_or(0.0);
                let x2 = inputs.get("x2").copied().unwrap_or(0.0);
                let mut out = HashMap::new();
                out.insert("y".to_string(), x1 + 2.0 * x2);
                out
            },
            Some(42),
        );

        let shifted_res = analyzer.analyze(
            &params,
            &["y".to_string()],
            |inputs| {
                let x1 = inputs.get("x1").copied().unwrap_or(0.0);
                let x2 = inputs.get("x2").copied().unwrap_or(0.0);
                let mut out = HashMap::new();
                out.insert("y".to_string(), x1 + 2.0 * x2 + 1_000_000.0);
                out
            },
            Some(42),
        );

        let s1_base = base_res
            .output_results
            .get("y")
            .unwrap()
            .parameter_indices
            .get("x1_S1")
            .unwrap()
            .value;
        let s1_shifted = shifted_res
            .output_results
            .get("y")
            .unwrap()
            .parameter_indices
            .get("x1_S1")
            .unwrap()
            .value;

        assert!(
            (s1_base - s1_shifted).abs() < 1e-10,
            "S1 must be strictly location-shift invariant: base={s1_base}, shifted={s1_shifted}"
        );
    }

    #[test]
    fn test_sobol_constant_zero_variance() {
        let config = SobolConfig {
            n: 256,
            calc_second_order: false,
            num_bootstrap: 0,
        };
        let analyzer = SobolAnalyzer::new(config);
        let params = vec![SensitivityParameter {
            name: "x1".to_string(),
            param_type: ParameterType::Continuous,
            baseline: 0.5,
            lower: 0.0,
            upper: 1.0,
            distribution: DistributionType::Uniform,
        }];

        let result = analyzer.analyze(
            &params,
            &["y".to_string()],
            |_| {
                let mut out = HashMap::new();
                out.insert("y".to_string(), 42.0); // Constant output, zero variance
                out
            },
            Some(42),
        );

        let output = result.output_results.get("y").unwrap();
        let s1 = output.parameter_indices.get("x1_S1").unwrap().value;
        let st = output.parameter_indices.get("x1").unwrap().value;

        assert_eq!(s1, 0.0, "Zero-variance output must yield S1 = 0.0");
        assert_eq!(st, 0.0, "Zero-variance output must yield ST = 0.0");
    }

    #[test]
    fn test_sobol_and_morris_resilient_to_non_finite_model_outputs() {
        let config = SobolConfig {
            n: 64,
            calc_second_order: false,
            num_bootstrap: 10,
        };
        let analyzer = SobolAnalyzer::new(config);
        let params = vec![SensitivityParameter {
            name: "x1".to_string(),
            param_type: ParameterType::Continuous,
            baseline: 0.5,
            lower: 0.0,
            upper: 1.0,
            distribution: DistributionType::Uniform,
        }];

        // Model returns NaN on high parameter inputs
        let result = analyzer.analyze(
            &params,
            &["y".to_string()],
            |inputs| {
                let x1 = inputs.get("x1").copied().unwrap_or(0.0);
                let mut out = HashMap::new();
                let y = if x1 > 0.8 { f64::NAN } else { x1 * 2.0 };
                out.insert("y".to_string(), y);
                out
            },
            Some(42),
        );

        let output = result.output_results.get("y").unwrap();
        let s1 = output.parameter_indices.get("x1_S1").unwrap().value;
        let st = output.parameter_indices.get("x1").unwrap().value;
        assert!(s1.is_finite() && (0.0..=1.0).contains(&s1));
        assert!(st.is_finite() && (0.0..=1.0).contains(&st));
    }
}
