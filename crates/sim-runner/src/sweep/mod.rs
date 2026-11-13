//! Parameter sweep execution for sim-runner
//!
//! Supports various sweep types:
//! - Linear: evenly spaced values
//! - Logarithmic: log-spaced values
//! - Grid: multi-parameter grid search
//! - Monte Carlo: random sampling
//!
//! # Morris Screening (§4.4.1)
//!
//! The Morris method efficiently identifies inactive parameters by computing
//! elementary effects across parameter trajectories.

pub mod grid;
pub mod monte_carlo;
#[cfg(test)]
pub mod tests;

pub use grid::*;
pub use monte_carlo::*;

use crate::batch::{BatchError, BatchResult, BatchRunner};
use crate::config::RunConfig;
use rayon::prelude::*;
use std::collections::HashMap;

/// Parameter sweep executor
#[derive(Debug)]
pub struct SweepRunner {
    base_config: RunConfig,
}

impl SweepRunner {
    /// Create a new sweep runner with base configuration
    #[must_use]
    pub const fn new(base_config: RunConfig) -> Self {
        Self { base_config }
    }

    /// Execute parameter sweep
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn run_sweep(
        &self,
        parameter: &str,
        values: &[f64],
        replications: u32,
    ) -> Result<SweepResult, BatchError> {
        let sweep_results: Vec<(f64, BatchResult)> = values
            .par_iter()
            .map(|&value| {
                let config = self.modify_config(parameter, value)?;
                let runner = BatchRunner::new(config);
                // TAG: surface=simulation owner=research-team rule=SIM-001
                let result = runner.run(replications)?;
                Ok((value, result))
            })
            .collect::<Result<Vec<_>, BatchError>>()?;

        Ok(SweepResult {
            parameter: parameter.to_string(),
            values: values.to_vec(),
            results: sweep_results,
        })
    }

    /// Generate linear sweep values
    #[must_use]
    pub fn linear_values(start: f64, end: f64, steps: usize) -> Vec<f64> {
        if steps <= 1 {
            return vec![start];
        }
        let step_size = (end - start) / (steps - 1) as f64;
        (0..steps).map(|i| start + i as f64 * step_size).collect()
    }

    /// Generate logarithmic sweep values
    #[must_use]
    pub fn log_values(start: f64, end: f64, steps: usize) -> Vec<f64> {
        if steps <= 1 {
            return vec![start];
        }
        let safe_start = start.max(1e-12);
        let safe_end = end.max(safe_start + 1e-12);
        let log_start = safe_start.ln();
        let log_end = safe_end.ln();
        let log_step = (log_end - log_start) / (steps - 1) as f64;
        (0..steps)
            .map(|i| (log_start + i as f64 * log_step).exp())
            .collect()
    }

    /// Modify configuration for a specific parameter value
    fn modify_config(&self, parameter: &str, value: f64) -> Result<RunConfig, BatchError> {
        let mut config = self.base_config.clone();

        match parameter {
            "agent_count" => config.simulation.agent_count = value as u32,
            "max_ticks" => config.simulation.max_ticks = value as u64,
            "grid_width" => config.simulation.grid_width = value as usize,
            "grid_height" => config.simulation.grid_height = value as usize,
            "seed" => config.simulation.seed = value as u64,
            _ => {
                return Err(BatchError::ConfigError(format!(
                    "Unknown parameter: {parameter}"
                )));
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            }
        }

        Ok(config)
    }
}

/// Result from parameter sweep
#[derive(Debug, Clone)]
pub struct SweepResult {
    /// Name of the parameter being swept
    pub parameter: String,
    /// Parameter values tested
    pub values: Vec<f64>,
    /// Batch results for each parameter value
    pub results: Vec<(f64, BatchResult)>,
}

impl SweepResult {
    /// Print sweep summary
    pub fn print_summary(&self) {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                     PARAMETER SWEEP SUMMARY                  ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
        println!("  Parameter: {}", self.parameter);
        println!("  Values:    {} points", self.values.len());
        println!();
        println!(
            "  {:>10} │ {:>10} │ {:>10} │ {:>10}",
            "Value", "Gini", "BR", "Agents"
        );
        println!("  ───────────┼────────────┼────────────┼────────────");
        for (value, result) in &self.results {
            println!(
                "  {:>10.2} │ {:>10.4} │ {:>10.4} │ {:>10.1}",
                value, result.gini_mean, result.branching_ratio_mean, result.mean_final_agents
            );
        }
        println!();
    }
    // TAG: surface=simulation owner=research-team rule=SIM-001

    /// Get sensitivity metrics
    #[must_use]
    pub fn sensitivity(&self) -> SensitivityMetrics {
        let ginis: Vec<f64> = self
            .results
            .iter()
            .map(|(_, r)| r.gini_mean)
            .filter(|v| v.is_finite())
            .collect();
        let brs: Vec<f64> = self
            .results
            .iter()
            .map(|(_, r)| r.branching_ratio_mean)
            .filter(|v| v.is_finite())
            .collect();
        let clean_vals: Vec<f64> = self
            .values
            .iter()
            .copied()
            .filter(|v| v.is_finite())
            .collect();

        SensitivityMetrics {
            parameter: self.parameter.clone(),
            gini_range: if ginis.is_empty() {
                0.0
            } else {
                max_f64(&ginis) - min_f64(&ginis)
            },
            branching_ratio_range: if brs.is_empty() {
                0.0
            } else {
                max_f64(&brs) - min_f64(&brs)
            },
            value_range: if clean_vals.is_empty() {
                0.0
            } else {
                max_f64(&clean_vals) - min_f64(&clean_vals)
            },
        }
    }
}

/// Sensitivity metrics for a parameter
#[derive(Debug, Clone)]
pub struct SensitivityMetrics {
    /// Name of the parameter
    pub parameter: String,
    /// Range of Gini values observed
    pub gini_range: f64,
    /// Range of branching ratios observed
    pub branching_ratio_range: f64,
    /// Range of parameter values tested
    pub value_range: f64,
}

fn max_f64(values: &[f64]) -> f64 {
    values.iter().copied().fold(f64::NEG_INFINITY, f64::max)
}

fn min_f64(values: &[f64]) -> f64 {
    values.iter().copied().fold(f64::INFINITY, f64::min)
}

/// Morris screening executor (§4.4.1)
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
///
/// Identifies inactive parameters through elementary effects computation.
#[derive(Debug)]
pub struct MorrisScreener {
    base_config: RunConfig,
    parameters: Vec<String>,
    parameter_ranges: HashMap<String, (f64, f64)>,
}

impl MorrisScreener {
    /// Create a new Morris screener
    #[must_use]
    pub fn new(base_config: RunConfig) -> Self {
        let mut parameter_ranges = HashMap::new();
        parameter_ranges.insert("agent_count".to_string(), (50.0, 1000.0));
        parameter_ranges.insert("max_ticks".to_string(), (100.0, 5000.0));
        parameter_ranges.insert("grid_width".to_string(), (32.0, 128.0));
        parameter_ranges.insert("grid_height".to_string(), (32.0, 128.0));

        let mut parameters: Vec<String> = parameter_ranges.keys().cloned().collect();
        parameters.sort(); // SIM-004: deterministic order across processes

        Self {
            base_config,
            parameters,
            parameter_ranges,
        }
    }

    /// Set parameter ranges
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn set_range(&mut self, parameter: &str, min: f64, max: f64) {
        let (lower, upper) = if min <= max { (min, max) } else { (max, min) };
        self.parameter_ranges
            .insert(parameter.to_string(), (lower, upper));
        if !self.parameters.contains(&parameter.to_string()) {
            self.parameters.push(parameter.to_string());
            self.parameters.sort(); // SIM-004: keep parameters deterministically sorted
        }
    }

    /// Run Morris screening with r trajectories and p levels
    /// # Errors
    ///
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Returns an error if the operation fails.
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn screen(
        &self,
        trajectories: usize,
        levels: usize,
        replications: u32,
    ) -> Result<MorrisResult, BatchError> {
        let k = self.parameters.len();
        if k == 0 || trajectories == 0 {
            return Ok(MorrisResult {
                trajectories: 0,
                levels: levels.max(2),
                parameters: Vec::new(),
            });
        }
        let levels = levels.max(2);
        let delta = levels as f64 / (2.0 * (levels - 1) as f64);

        let mut elementary_effects: HashMap<String, Vec<f64>> = HashMap::new();
        for param in &self.parameters {
            elementary_effects.insert(param.clone(), Vec::new());
        }

        // Generate and evaluate trajectories per Morris (1991)
        use rand::Rng;
        use rand::SeedableRng;

        for t in 0..trajectories {
            let traj_seed = self
                .base_config
                .simulation
                .seed
                .wrapping_add(t as u64 * 104729)
                .wrapping_add(101);
            let mut traj_rng = rand::rngs::StdRng::seed_from_u64(traj_seed);

            // In Morris (1991), base point coordinates are drawn from grid levels [0, 1 - delta]
            let max_base_level = ((1.0 - delta) * (levels - 1) as f64).floor().max(0.0) as usize;
            let mut current_point: Vec<f64> = (0..k)
                .map(|_| {
                    let level_idx = if max_base_level > 0 {
                        traj_rng.gen_range(0..=max_base_level)
                    } else {
                        0
                    };
                    level_idx as f64 / (levels - 1) as f64
                })
                .collect();

            let mut current_result = self.evaluate_point(&current_point, replications)?;

            // Random permutation of parameters for this trajectory (Morris trajectory)
            let mut order: Vec<usize> = (0..k).collect();
            for i in (1..k).rev() {
                let j = traj_rng.gen_range(0..=i);
                order.swap(i, j);
            }

            for &param_idx in &order {
                let param = &self.parameters[param_idx];
                let mut perturbed = current_point.clone();
                let step = if current_point[param_idx] + delta <= 1.0 {
                    perturbed[param_idx] = current_point[param_idx] + delta;
                    delta
                } else {
                    perturbed[param_idx] = (current_point[param_idx] - delta).max(0.0);
                    -delta
                };

                let perturbed_result = self.evaluate_point(&perturbed, replications)?;

                // Compute elementary effect on Gini
                let ee = (perturbed_result.gini_mean - current_result.gini_mean) / step;
                if let Some(list) = elementary_effects.get_mut(param) {
                    list.push(ee);
                }

                current_point = perturbed;
                current_result = perturbed_result;
            }
        }

        // Compute μ* and σ for each parameter
        // P1 FIX: Pre-allocate with known capacity
        let mut results: Vec<MorrisParameterResult> = Vec::with_capacity(self.parameters.len());
        let default_ees = Vec::new();
        for param in &self.parameters {
            let ees = elementary_effects.get(param).unwrap_or(&default_ees);
            let n = ees.len().max(1) as f64;
            let mu_star = ees.iter().map(|e| e.abs()).sum::<f64>() / n;
            let mu = ees.iter().sum::<f64>() / n;
            let variance = ees.iter().map(|e| (e - mu).powi(2)).sum::<f64>()
                / (ees.len().saturating_sub(1)).max(1) as f64;
            let sigma = variance.sqrt();

            results.push(MorrisParameterResult {
                parameter: param.clone(),
                mu_star,
                sigma,
                is_inactive: mu_star < 0.01, // Threshold for inactivity
            });
        }

        // Sort by μ* descending
        results.sort_by(|a, b| {
            b.mu_star
                .partial_cmp(&a.mu_star)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(MorrisResult {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            trajectories,
            levels,
            parameters: results,
        })
    }

    /// Evaluate a single point in parameter space
    fn evaluate_point(
        &self,
        normalized_values: &[f64],
        replications: u32,
    ) -> Result<BatchResult, BatchError> {
        let mut config = self.base_config.clone();

        for (i, param) in self.parameters.iter().enumerate() {
            let Some(&(min, max)) = self.parameter_ranges.get(param) else {
                continue;
            };
            let value = min + normalized_values[i] * (max - min);

            match param.as_str() {
                "agent_count" => config.simulation.agent_count = value as u32,
                "max_ticks" => config.simulation.max_ticks = value as u64,
                "grid_width" => config.simulation.grid_width = value as usize,
                "grid_height" => config.simulation.grid_height = value as usize,
                _ => {}
            }
        }

        let runner = BatchRunner::new(config);
        runner.run(replications)
    }
}

/// Result from Morris screening
#[derive(Debug, Clone)]
pub struct MorrisResult {
    /// Number of trajectories used in screening
    pub trajectories: usize,
    /// Number of discrete levels per parameter
    pub levels: usize,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Results for each parameter
    pub parameters: Vec<MorrisParameterResult>,
}

impl MorrisResult {
    /// Print Morris screening summary
    pub fn print_summary(&self) {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                     MORRIS SCREENING RESULTS                 ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
        println!("  Trajectories: {}", self.trajectories);
        println!("  Levels:       {}", self.levels);
        println!();
        println!(
            "  {:>15} │ {:>10} │ {:>10} │ {:>10}",
            "Parameter", "μ*", "σ", "Status"
        );
        println!("  ────────────────┼────────────┼────────────┼────────────");
        for p in &self.parameters {
            let status = if p.is_inactive { "INACTIVE" } else { "ACTIVE" };
            println!(
                "  {:>15} │ {:>10.4} │ {:>10.4} │ {:>10}",
                p.parameter, p.mu_star, p.sigma, status
            );
        }
        println!();
    }

    /// Get list of inactive parameters
    #[must_use]
    pub fn inactive_parameters(&self) -> Vec<String> {
        self.parameters
            .iter()
            .filter(|p| p.is_inactive)
            .map(|p| p.parameter.clone())
            .collect()
    }
}

/// Morris result for a single parameter
// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Clone)]
pub struct MorrisParameterResult {
    /// Name of the parameter
    pub parameter: String,
    /// Mean of absolute elementary effects
    pub mu_star: f64,
    /// Standard deviation of elementary effects
    pub sigma: f64,
    /// Whether the parameter is considered inactive
    pub is_inactive: bool,
}
