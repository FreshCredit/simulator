// TAG: surface=simulation owner=research-team rule=SIM-001
//! Monte Carlo parameter sweep execution, sampling distributions, and sensitivity analysis.

use crate::batch::{BatchError, BatchResult, BatchRunner};
use crate::config::RunConfig;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Distribution type for Monte Carlo sampling
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DistributionType {
    /// Uniform distribution across parameter bounds
    #[default]
    Uniform,
    /// Normal (Gaussian) distribution
    Normal {
        /// Mean of the normal distribution
        mean: f32,
        /// Standard deviation of the normal distribution
        std_dev: f32,
    },
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Log-uniform distribution (for parameters spanning orders of magnitude)
    LogUniform,
    /// Latin Hypercube Sampling for efficient space coverage
    LatinHypercube,
}

/// Configuration for Monte Carlo parameter sweep
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonteCarloConfig {
    /// Number of random samples to generate
    pub n_samples: usize,
    /// Parameter bounds: (name, min, max)
    pub parameter_bounds: Vec<(String, f32, f32)>,
    /// Distribution type for sampling
    #[serde(default)]
    pub distribution: DistributionType,
    /// Base configuration template
    pub base_config: RunConfig,
    /// Random seed for reproducibility
    pub seed: u64,
}

impl MonteCarloConfig {
    /// Create a new Monte Carlo configuration
    #[must_use]
    pub const fn new(n_samples: usize, base_config: RunConfig, seed: u64) -> Self {
        Self {
            n_samples,
            parameter_bounds: Vec::new(),
            distribution: DistributionType::Uniform,
            base_config,
            seed,
        }
    }

    /// Add a parameter with bounds
    pub fn add_parameter(&mut self, name: &str, min: f32, max: f32) {
        self.parameter_bounds.push((name.to_string(), min, max));
    }

    /// Set the distribution type
    #[must_use]
    // TAG: surface=simulation owner=research-team rule=SIM-001
    pub const fn with_distribution(mut self, distribution: DistributionType) -> Self {
        self.distribution = distribution;
        self
    }
}

/// A single Monte Carlo sample with its parameters and result
#[derive(Debug, Clone)]
pub struct MonteCarloSample {
    /// Sample index
    pub index: usize,
    /// Sampled parameter values
    pub parameters: HashMap<String, f32>,
    /// Simulation result for this sample
    pub result: BatchResult,
}

/// Complete results from a Monte Carlo sweep
#[derive(Debug, Clone)]
pub struct MonteCarloResult {
    /// Number of samples
    pub n_samples: usize,
    /// Distribution type used
    pub distribution: DistributionType,
    /// Individual sample results
    pub samples: Vec<MonteCarloSample>,
    /// Aggregate statistics
    pub statistics: MonteCarloStatistics,
}

/// Statistical summary of Monte Carlo results
#[derive(Debug, Clone)]
pub struct MonteCarloStatistics {
    /// Mean Gini coefficient across samples
    pub gini_mean: f64,
    /// Standard deviation of Gini coefficient
    pub gini_std: f64,
    /// Mean branching ratio across samples
    pub branching_ratio_mean: f64,
    /// Standard deviation of branching ratio
    pub branching_ratio_std: f64,
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Mean final agent count across samples
    pub final_agents_mean: f64,
    /// Standard deviation of final agent count
    pub final_agents_std: f64,
    /// Mean DOCAS magnitude across samples
    pub docas_magnitude_mean: f64,
    /// Standard deviation of DOCAS magnitude
    pub docas_magnitude_std: f64,
    /// Parameter sensitivities (correlation with outputs)
    pub parameter_sensitivities: HashMap<String, ParameterSensitivity>,
}

/// Sensitivity metrics for a single parameter
#[derive(Debug, Clone)]
pub struct ParameterSensitivity {
    /// Parameter name
    pub parameter: String,
    /// Pearson correlation with Gini coefficient
    pub gini_correlation: f64,
    /// Pearson correlation with branching ratio
    pub branching_ratio_correlation: f64,
    /// Pearson correlation with final agent count
    pub final_agents_correlation: f64,
    /// Rank correlation (Spearman) with Gini
    pub gini_rank_correlation: f64,
}

/// Sample generator for Monte Carlo sampling
#[derive(Debug)]
pub struct SampleGenerator {
    /// Random number generator
    rng: StdRng,
    /// Distribution type
    distribution: DistributionType,
    /// Latin Hypercube intervals (pre-computed)
    lhs_intervals: Option<Vec<Vec<usize>>>,
    /// Current LHS sample index
    lhs_index: usize,
    /// Number of samples configured
    n_samples: usize,
}

impl SampleGenerator {
    /// Create a new sample generator with the given seed and distribution
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[must_use]
    pub fn new(seed: u64, distribution: DistributionType, n_samples: usize) -> Self {
        let rng = StdRng::seed_from_u64(seed);
        let safe_n = n_samples.max(1);

        Self {
            rng,
            distribution,
            lhs_intervals: None,
            lhs_index: 0,
            n_samples: safe_n,
        }
    }

    /// Generate Latin Hypercube permutation matrix
    /// Returns a matrix where each row is a parameter and each column is a sample
    /// with shuffled interval indices
    #[allow(dead_code)]
    const fn generate_lhs_permutation(_n_samples: usize) -> Vec<Vec<usize>> {
        Vec::new()
    }

    /// Generate a single sample from the given parameter bounds
    pub fn sample(&mut self, bounds: &[(String, f32, f32)]) -> HashMap<String, f32> {
        match self.distribution {
            DistributionType::LatinHypercube => {
                self.sample_lhs(bounds, self.lhs_index, self.lhs_intervals.is_some())
            }
            _ => self.sample_standard(bounds),
        }
    }

    /// Standard random sampling (Uniform, Normal, `LogUniform`)
    fn sample_standard(&mut self, bounds: &[(String, f32, f32)]) -> HashMap<String, f32> {
        let mut sample = HashMap::new();

        for (name, min_val, max_val) in bounds {
            let (min, max) = if min_val <= max_val {
                (*min_val, *max_val)
            } else {
                (*max_val, *min_val)
            };
            if (max - min).abs() < 1e-6 {
                sample.insert(name.clone(), min);
                continue;
            }
            // TAG: surface=simulation owner=research-team rule=SIM-001
            let value = match self.distribution {
                DistributionType::Uniform => self.rng.gen_range(min..max),
                DistributionType::Normal { mean, std_dev } => {
                    // Sample from normal, then clamp to bounds
                    let normal_sample: f32 = self.sample_normal(mean, std_dev);
                    normal_sample.clamp(min, max)
                }
                DistributionType::LogUniform => {
                    // Sample uniformly in log space
                    let safe_min = min.max(1e-6);
                    let safe_max = max.max(safe_min + 1e-6);
                    let log_min = safe_min.ln();
                    let log_max = safe_max.ln();
                    let log_sample: f32 = self.rng.gen_range(log_min..log_max);
                    log_sample.exp()
                }
                DistributionType::LatinHypercube => unreachable!(),
            };
            sample.insert(name.clone(), value);
        }

        sample
    }

    /// Sample from normal distribution (Box-Muller transform)
    fn sample_normal(&mut self, mean: f32, std_dev: f32) -> f32 {
        let u1: f32 = self.rng.gen_range(1e-7_f32..1.0);
        let u2: f32 = self.rng.gen_range(0.0_f32..1.0);

        // Box-Muller transform
        let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();

        mean + std_dev * z0
    }

    /// Latin Hypercube Sampling
    /// Divides each parameter range into n equal intervals and samples
    /// exactly once from each interval across all samples
    fn sample_lhs(
        &mut self,
        bounds: &[(String, f32, f32)],
        sample_idx: usize,
        _initialized: bool,
    ) -> HashMap<String, f32> {
        let n = self.n_samples;
        if self.lhs_intervals.is_none()
            || self.lhs_intervals.as_ref().map_or(0, |v| v.len()) != bounds.len()
        {
            let mut intervals = Vec::with_capacity(bounds.len());
            for param_idx in 0..bounds.len() {
                let mut param_rng = StdRng::seed_from_u64(
                    self.rng
                        .gen::<u64>()
                        .wrapping_add((param_idx as u64).wrapping_mul(1_000_003)),
                );
                let mut permutation: Vec<usize> = (0..n).collect();
                for i in (1..permutation.len()).rev() {
                    let j = param_rng.gen_range(0..=i);
                    permutation.swap(i, j);
                }
                intervals.push(permutation);
            }
            self.lhs_intervals = Some(intervals);
        }

        let mut sample = HashMap::new();

        if let Some(ref intervals) = self.lhs_intervals {
            for (param_idx, (name, min_val, max_val)) in bounds.iter().enumerate() {
                let (min, max) = if min_val <= max_val {
                    (*min_val, *max_val)
                } else {
                    (*max_val, *min_val)
                };
                if (max - min).abs() < 1e-6 {
                    sample.insert(name.clone(), min);
                    continue;
                }
                let interval_idx = intervals[param_idx][sample_idx % n];
                let interval_width = (max - min) / n as f32;
                let interval_start = min + interval_idx as f32 * interval_width;
                let interval_end = (interval_start + interval_width).max(interval_start + 1e-6);

                let value: f32 = self.rng.gen_range(interval_start..interval_end);
                sample.insert(name.clone(), value.clamp(min, max));
            }
        }

        self.lhs_index += 1;
        sample
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Reset the generator state
    pub fn reset(&mut self, seed: u64) {
        self.rng = StdRng::seed_from_u64(seed);
        self.lhs_intervals = None;
        self.lhs_index = 0;
    }
}

/// Generate random samples within parameter bounds using the specified distribution
/// # Errors
///
/// Returns an error if the operation fails.
pub fn sample_parameters(
    bounds: &[(String, f32, f32)],
    distribution: DistributionType,
    rng: &mut StdRng,
) -> HashMap<String, f32> {
    let mut sample = HashMap::new();

    for (name, min_val, max_val) in bounds {
        let (min, max) = if min_val <= max_val {
            (*min_val, *max_val)
        } else {
            (*max_val, *min_val)
        };
        if (max - min).abs() < 1e-6 {
            sample.insert(name.clone(), min);
            continue;
        }

        let value = match distribution {
            DistributionType::Uniform => rng.gen_range(min..max),
            DistributionType::Normal { mean, std_dev } => {
                let u1: f32 = rng.gen_range(1e-7_f32..1.0);
                let u2: f32 = rng.gen_range(0.0_f32..1.0);
                let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
                let normal_sample = mean + std_dev * z0;
                normal_sample.clamp(min, max)
            }
            DistributionType::LogUniform => {
                let safe_min = min.max(1e-6);
                let safe_max = max.max(safe_min + 1e-6);
                let log_min = safe_min.ln();
                let log_max = safe_max.ln();
                let log_sample: f32 = rng.gen_range(log_min..log_max);
                log_sample.exp()
            }
            DistributionType::LatinHypercube => rng.gen_range(min..max),
        };
        sample.insert(name.clone(), value);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    sample
}

/// Monte Carlo sweep runner
#[derive(Debug)]
pub struct MonteCarloRunner {
    config: MonteCarloConfig,
}

impl MonteCarloRunner {
    /// Create a new Monte Carlo runner
    #[must_use]
    pub const fn new(config: MonteCarloConfig) -> Self {
        Self { config }
    }

    /// Run Monte Carlo sweep with the configured number of samples
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn run(&self, replications: u32) -> Result<MonteCarloResult, BatchError> {
        self.run_monte_carlo_sweep(replications)
    }

    /// Execute Monte Carlo parameter sweep
    ///
    /// Generates `n_samples` random parameter sets, runs a simulation for each,
    /// and collects results with sampled parameters attached.
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn run_monte_carlo_sweep(&self, replications: u32) -> Result<MonteCarloResult, BatchError> {
        // Validate configuration
        if self.config.parameter_bounds.is_empty() {
            return Err(BatchError::ConfigError(
                "Monte Carlo requires at least one parameter bound".to_string(),
            ));
        }
        if self.config.n_samples == 0 {
            return Err(BatchError::ConfigError(
                "n_samples must be greater than 0".to_string(),
                // TAG: surface=simulation owner=research-team rule=SIM-001
            ));
        }

        let n_samples = self.config.n_samples;
        let bounds = &self.config.parameter_bounds;
        let distribution = self.config.distribution;
        let base_config = &self.config.base_config;

        // Generate samples and run simulations in parallel
        let samples: Vec<MonteCarloSample> = (0..n_samples)
            .into_par_iter()
            .map(|sample_idx| {
                // Create a seeded RNG for this sample
                let sample_seed = self.config.seed.wrapping_add(sample_idx as u64 * 1000003);
                let mut rng = StdRng::seed_from_u64(sample_seed);

                // Sample parameters
                let parameters = if distribution == DistributionType::LatinHypercube {
                    // For LHS, we need special handling
                    sample_lhs_parameters(bounds, n_samples, sample_idx, self.config.seed, &mut rng)
                } else {
                    sample_parameters(bounds, distribution, &mut rng)
                };

                // Create RunConfig with sampled parameters
                let config = self.create_config_from_sample(base_config, &parameters);

                // Run simulation
                let runner = BatchRunner::new(config);
                let result = runner.run(replications)?;

                Ok(MonteCarloSample {
                    index: sample_idx,
                    parameters,
                    result,
                })
            })
            .collect::<Result<Vec<_>, BatchError>>()?;

        // Compute aggregate statistics
        let statistics = compute_monte_carlo_statistics(&samples, bounds);

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        Ok(MonteCarloResult {
            n_samples,
            distribution,
            samples,
            statistics,
        })
    }

    /// Create a `RunConfig` from sampled parameters
    fn create_config_from_sample(
        &self,
        base_config: &RunConfig,
        parameters: &HashMap<String, f32>,
    ) -> RunConfig {
        let mut config = base_config.clone();

        for (name, value) in parameters {
            match name.as_str() {
                "agent_count" => config.simulation.agent_count = *value as u32,
                "max_ticks" => config.simulation.max_ticks = *value as u64,
                "grid_width" => config.simulation.grid_width = *value as usize,
                "grid_height" => config.simulation.grid_height = *value as usize,
                "seed" => config.simulation.seed = *value as u64,
                "docas_alpha_d" => config.simulation.docas_alpha_d = Some(*value),
                "docas_alpha_o" => config.simulation.docas_alpha_o = Some(*value),
                "docas_alpha_c" => config.simulation.docas_alpha_c = Some(*value),
                "docas_alpha_a" => config.simulation.docas_alpha_a = Some(*value),
                "docas_alpha_s" => config.simulation.docas_alpha_s = Some(*value),
                "docas_beta_d" => config.simulation.docas_beta_d = Some(*value),
                "docas_beta_o" => config.simulation.docas_beta_o = Some(*value),
                "docas_beta_c" => config.simulation.docas_beta_c = Some(*value),
                "docas_beta_a" => config.simulation.docas_beta_a = Some(*value),
                "docas_beta_s" => config.simulation.docas_beta_s = Some(*value),
                "planning_depth_k" => config.simulation.planning_depth_k = Some(*value as u32),
                "planning_horizon_h" => config.simulation.planning_horizon_h = Some(*value as u64),
                "initial_resource" => config.simulation.initial_resource = Some(*value),
                "resource_density" => config.simulation.resource_density = Some(*value),
                "verification_cost" => config.simulation.verification_cost = Some(*value),
                "attestation_stake" => config.simulation.attestation_stake = Some(*value),
                _ => {}
            }
        }

        config
    }
}

/// Latin Hypercube Sampling implementation
///
/// Divides each parameter range into `n_samples` equal intervals.
/// For each sample, selects one interval per parameter such that
/// each interval is sampled exactly once across all samples.
pub(crate) fn sample_lhs_parameters(
    bounds: &[(String, f32, f32)],
    // TAG: surface=simulation owner=research-team rule=SIM-001
    n_samples: usize,
    sample_idx: usize,
    sweep_seed: u64,
    rng: &mut StdRng,
) -> HashMap<String, f32> {
    let mut sample = HashMap::new();
    let n = n_samples.max(1);

    for (param_idx, (name, min_val, max_val)) in bounds.iter().enumerate() {
        let (min, max) = if min_val <= max_val {
            (*min_val, *max_val)
        } else {
            (*max_val, *min_val)
        };
        if (max - min).abs() < 1e-6 {
            sample.insert(name.clone(), min);
            continue;
        }

        // Shared deterministic permutation per parameter across all n_samples
        let param_seed = sweep_seed
            .wrapping_add((param_idx as u64).wrapping_mul(1_000_003))
            .wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut param_rng = StdRng::seed_from_u64(param_seed);

        // Generate the single shared permutation of interval indices for this parameter
        let mut permutation: Vec<usize> = (0..n).collect();
        for i in (1..permutation.len()).rev() {
            let j = param_rng.gen_range(0..=i);
            permutation.swap(i, j);
        }

        // Exact 1-to-1 Latin Hypercube stratum selection
        let interval_idx = permutation[sample_idx % n];
        let interval_width = (max - min) / n as f32;
        let interval_start = min + interval_idx as f32 * interval_width;
        let interval_end = (interval_start + interval_width).max(interval_start + 1e-6);

        // Sample uniformly within the uniquely assigned interval
        let value: f32 = rng.gen_range(interval_start..interval_end);
        sample.insert(name.clone(), value.clamp(min, max));
    }

    sample
}

/// Compute statistics across Monte Carlo samples
fn compute_monte_carlo_statistics(
    samples: &[MonteCarloSample],
    bounds: &[(String, f32, f32)],
) -> MonteCarloStatistics {
    if samples.is_empty() {
        return MonteCarloStatistics {
            gini_mean: 0.0,
            gini_std: 0.0,
            branching_ratio_mean: 0.0,
            branching_ratio_std: 0.0,
            final_agents_mean: 0.0,
            final_agents_std: 0.0,
            docas_magnitude_mean: 0.0,
            docas_magnitude_std: 0.0,
            parameter_sensitivities: HashMap::new(),
        };
    }

    let n = samples.len() as f64;

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    // Extract metric values
    let ginis: Vec<f64> = samples.iter().map(|s| s.result.gini_mean).collect();
    let brs: Vec<f64> = samples
        .iter()
        .map(|s| s.result.branching_ratio_mean)
        .collect();
    let agents: Vec<f64> = samples.iter().map(|s| s.result.mean_final_agents).collect();
    let docas: Vec<f64> = samples
        .iter()
        .map(|s| s.result.docas_magnitude_mean)
        .collect();

    // Compute means
    let gini_mean = ginis.iter().sum::<f64>() / n;
    let branching_ratio_mean = brs.iter().sum::<f64>() / n;
    let final_agents_mean = agents.iter().sum::<f64>() / n;
    let docas_magnitude_mean = docas.iter().sum::<f64>() / n;

    // Compute standard deviations
    let gini_std = compute_std(&ginis, gini_mean);
    let branching_ratio_std = compute_std(&brs, branching_ratio_mean);
    let final_agents_std = compute_std(&agents, final_agents_mean);
    let docas_magnitude_std = compute_std(&docas, docas_magnitude_mean);

    // Compute parameter sensitivities
    let mut parameter_sensitivities = HashMap::new();

    for (name, _, _) in bounds {
        let param_values: Vec<f64> = samples
            .iter()
            .map(|s| *s.parameters.get(name).unwrap_or(&0.0) as f64)
            .collect();

        let sensitivity = ParameterSensitivity {
            parameter: name.clone(),
            gini_correlation: pearson_correlation(&param_values, &ginis),
            branching_ratio_correlation: pearson_correlation(&param_values, &brs),
            final_agents_correlation: pearson_correlation(&param_values, &agents),
            gini_rank_correlation: spearman_correlation(&param_values, &ginis),
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        parameter_sensitivities.insert(name.clone(), sensitivity);
    }

    MonteCarloStatistics {
        gini_mean,
        gini_std,
        branching_ratio_mean,
        branching_ratio_std,
        final_agents_mean,
        final_agents_std,
        docas_magnitude_mean,
        docas_magnitude_std,
        parameter_sensitivities,
    }
}

/// Compute standard deviation
pub(crate) fn compute_std(values: &[f64], mean: f64) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let variance =
        values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    variance.sqrt()
}

/// Pearson correlation coefficient
pub(crate) fn pearson_correlation(x: &[f64], y: &[f64]) -> f64 {
    sim_core::stats::pearson_correlation_f64(x, y)
}

/// Spearman rank correlation coefficient
pub(crate) fn spearman_correlation(x: &[f64], y: &[f64]) -> f64 {
    sim_core::stats::spearman_correlation_f64(x, y)
}

/// Convert values to ranks (handling ties by averaging)
#[allow(dead_code)]
pub(crate) fn rank_data(data: &[f64]) -> Vec<f64> {
    sim_core::stats::rank_data_f64(data)
}

impl MonteCarloResult {
    /// Print Monte Carlo sweep summary
    pub fn print_summary(&self) {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                  MONTE CARLO SWEEP RESULTS                   ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
        println!("  Samples:      {}", self.n_samples);
        println!("  Distribution: {:?}", self.distribution);
        println!();

        println!("  STATISTICS:");
        println!("  {:<20} {:>12} ± {:<10}", "Metric:", "Mean", "Std Dev");
        println!(
            "  {:<20} {:>12.4} ± {:.4}",
            "Gini Coefficient:", self.statistics.gini_mean, self.statistics.gini_std
        );
        println!(
            "  {:<20} {:>12.4} ± {:.4}",
            "Branching Ratio:",
            self.statistics.branching_ratio_mean,
            self.statistics.branching_ratio_std
        );
        println!(
            "  {:<20} {:>12.1} ± {:.1}",
            "Final Agents:", self.statistics.final_agents_mean, self.statistics.final_agents_std
        );
        println!(
            "  {:<20} {:>12.4} ± {:.4}",
            "DOCAS Magnitude:",
            self.statistics.docas_magnitude_mean,
            self.statistics.docas_magnitude_std
        );
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        println!();

        if !self.statistics.parameter_sensitivities.is_empty() {
            println!("  PARAMETER SENSITIVITIES (Pearson correlation with Gini):");
            let mut sensitivities: Vec<_> =
                self.statistics.parameter_sensitivities.values().collect();
            sensitivities.sort_by(|a, b| {
                b.gini_correlation
                    .abs()
                    .partial_cmp(&a.gini_correlation.abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            for sens in sensitivities {
                println!(
                    "  {:<20} {:>10.4} (BR: {:>7.4}, Agents: {:>7.4})",
                    sens.parameter,
                    sens.gini_correlation,
                    sens.branching_ratio_correlation,
                    sens.final_agents_correlation
                );
            }
            println!();
        }
    }

    /// Get samples sorted by a specific metric
    #[must_use]
    pub fn samples_sorted_by_gini(&self) -> Vec<&MonteCarloSample> {
        let mut sorted: Vec<_> = Vec::with_capacity(self.n_samples);
        sorted.extend(self.samples.iter());
        sorted.sort_by(|a, b| {
            a.result
                .gini_mean
                .partial_cmp(&b.result.gini_mean)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted
    }

    /// Get the sample with maximum Gini coefficient
    #[must_use]
    // TAG: surface=simulation owner=research-team rule=SIM-001
    pub fn max_gini_sample(&self) -> Option<&MonteCarloSample> {
        self.samples.iter().max_by(|a, b| {
            a.result
                .gini_mean
                .partial_cmp(&b.result.gini_mean)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Get the sample with minimum Gini coefficient
    #[must_use]
    pub fn min_gini_sample(&self) -> Option<&MonteCarloSample> {
        self.samples.iter().min_by(|a, b| {
            a.result
                .gini_mean
                .partial_cmp(&b.result.gini_mean)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Get confidence intervals for metrics
    #[must_use]
    pub fn confidence_intervals(&self, confidence: f64) -> MetricConfidenceIntervals {
        let conf = confidence.clamp(0.50, 0.9999);
        let z_score = crate::stats::normal_cdf_inv(1.0 - (1.0 - conf) / 2.0);
        let n = (self.n_samples as f64).max(1.0);
        let sqrt_n = n.sqrt();

        let gini_se = self.statistics.gini_std / sqrt_n;
        let br_se = self.statistics.branching_ratio_std / sqrt_n;
        let fa_se = self.statistics.final_agents_std / sqrt_n;

        MetricConfidenceIntervals {
            gini: (
                self.statistics.gini_mean - z_score * gini_se,
                self.statistics.gini_mean + z_score * gini_se,
            ),
            branching_ratio: (
                self.statistics.branching_ratio_mean - z_score * br_se,
                self.statistics.branching_ratio_mean + z_score * br_se,
            ),
            final_agents: (
                self.statistics.final_agents_mean - z_score * fa_se,
                self.statistics.final_agents_mean + z_score * fa_se,
            ),
        }
    }
}

/// Confidence intervals for metrics
#[derive(Debug, Clone)]
pub struct MetricConfidenceIntervals {
    /// 95% confidence interval for Gini coefficient
    pub gini: (f64, f64),
    /// 95% confidence interval for branching ratio
    pub branching_ratio: (f64, f64),
    /// 95% confidence interval for final agent count
    pub final_agents: (f64, f64),
}
