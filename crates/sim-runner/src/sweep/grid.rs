// TAG: surface=simulation owner=research-team rule=SIM-001
//! Multi-parameter grid sweep execution, heatmap generation, and optimal region detection.

use super::monte_carlo::compute_std;
use crate::batch::{BatchError, BatchResult, BatchRunner};
use crate::config::RunConfig;
use rayon::prelude::*;
use std::collections::HashMap;

/// Configuration for grid sweep over multiple parameters.
#[derive(Debug, Clone)]
pub struct GridSweepConfig {
    /// Parameter names and their values to sweep over
    pub parameters: Vec<(String, Vec<f32>)>, // (param_name, values)
    /// Number of replications per grid point
    pub runs_per_config: usize,
    /// Base configuration template
    pub base_config: RunConfig,
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
impl GridSweepConfig {
    /// Create a new grid sweep configuration
    #[must_use]
    pub const fn new(base_config: RunConfig, runs_per_config: usize) -> Self {
        Self {
            parameters: Vec::new(),
            runs_per_config,
            base_config,
        }
    }

    /// Add a parameter to the grid sweep
    pub fn add_parameter(&mut self, name: &str, values: Vec<f32>) -> &mut Self {
        self.parameters.push((name.to_string(), values));
        self
    }

    /// Calculate total number of grid points (Cartesian product size)
    #[must_use]
    pub fn total_points(&self) -> usize {
        if self.parameters.is_empty() {
            return 0;
        }
        self.parameters
            .iter()
            .map(|(_, values)| values.len())
            .product()
    }

    /// Calculate total number of simulation runs
    #[must_use]
    pub fn total_runs(&self) -> usize {
        self.total_points() * self.runs_per_config
    }
}

/// A single point in the grid sweep parameter space
#[derive(Debug, Clone)]
pub struct GridPoint {
    /// Parameter name -> value mapping
    pub values: HashMap<String, f32>,
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Indices of runs corresponding to this grid point
    pub run_indices: Vec<usize>,
}

impl GridPoint {
    /// Create a new grid point from a parameter value mapping
    #[must_use]
    pub const fn new(values: HashMap<String, f32>) -> Self {
        Self {
            values,
            run_indices: Vec::new(),
        }
    }

    /// Get value for a specific parameter
    #[must_use]
    pub fn get(&self, param: &str) -> Option<f32> {
        self.values.get(param).copied()
    }
}

/// Results from a grid sweep analysis
#[derive(Debug, Clone)]
pub struct GridSweepResult {
    /// All grid points that were evaluated
    pub points: Vec<GridPoint>,
    /// Batch results for each grid point (one per point)
    pub results: Vec<BatchResult>,
    /// Parameter names in the order they were swept
    pub parameter_names: Vec<String>,
}

/// Statistics for a metric across grid points
#[derive(Debug, Clone)]
pub struct GridMetricStats {
    /// Minimum value observed
    pub min: f64,
    /// Maximum value observed
    pub max: f64,
    /// Mean value across grid points
    pub mean: f64,
    /// Standard deviation across grid points
    // TAG: surface=simulation owner=research-team rule=SIM-001
    pub std: f64,
}

/// Heatmap data for 2D visualization
#[derive(Debug, Clone)]
pub struct HeatmapData {
    /// Name of the parameter on the X axis
    pub x_param: String,
    /// Name of the parameter on the Y axis
    pub y_param: String,
    /// X axis parameter values
    pub x_values: Vec<f32>,
    /// Y axis parameter values
    pub y_values: Vec<f32>,
    /// 2D grid of metric values (None where missing)
    pub grid: Vec<Vec<Option<f64>>>,
}

impl HeatmapData {
    /// Export heatmap as CSV for external plotting
    #[must_use]
    pub fn to_csv(&self, metric_name: &str) -> String {
        let mut csv = format!("{},{},{}\n", self.x_param, self.y_param, metric_name);

        for (y_idx, y) in self.y_values.iter().enumerate() {
            for (x_idx, x) in self.x_values.iter().enumerate() {
                let value =
                    self.grid[y_idx][x_idx].map_or_else(|| "NA".to_string(), |v| format!("{v:.6}"));
                csv.push_str(&format!("{x},{y},{value}\n"));
            }
        }

        csv
    }
}

/// An identified optimal parameter region
#[derive(Debug, Clone)]
pub struct OptimalRegion {
    /// Index of the grid point in the sweep
    pub point_index: usize,
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Coordinates of the optimal point
    pub point: GridPoint,
    /// Metric value at the optimal point
    pub metric_value: f64,
}

/// Errors specific to grid sweep operations
#[derive(Debug, Clone)]
pub enum GridSweepError {
    /// Parameter name not recognized
    UnknownParameter(String),
    /// Data format or content is invalid
    InvalidData(String),
}

impl std::fmt::Display for GridSweepError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownParameter(p) => write!(f, "Unknown parameter: {p}"),
            Self::InvalidData(msg) => write!(f, "Invalid data: {msg}"),
        }
    }
}

impl std::error::Error for GridSweepError {}

impl GridSweepResult {
    /// Print summary of grid sweep results
    pub fn print_summary(&self) {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                      GRID SWEEP SUMMARY                      ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
        println!("  Grid Points:       {}", self.points.len());
        println!("  Parameters:        {:?}", self.parameter_names);
        println!();

        // Print header
        let param_headers: String = self
            .parameter_names
            .iter()
            .map(|p| format!("{p:>12}"))
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            .collect::<Vec<_>>()
            .join(" │ ");
        println!(
            "  {} │ {:>10} │ {:>10} │ {:>10}",
            param_headers, "Gini Mean", "Gini Std", "Final Agents"
        );

        let separator_len = if self.parameter_names.is_empty() {
            39
        } else {
            12 * self.parameter_names.len()
                + 3 * (self.parameter_names.len().saturating_sub(1))
                + 39
        };
        println!("  {}", "─".repeat(separator_len));

        // Print each point
        for (point, result) in self.points.iter().zip(self.results.iter()) {
            let param_values: String = self
                .parameter_names
                .iter()
                .map(|p| format!("{:>12.4}", point.get(p).unwrap_or(0.0)))
                .collect::<Vec<_>>()
                .join(" │ ");
            println!(
                "  {} │ {:>10.4} │ {:>10.4} │ {:>10.1}",
                param_values, result.gini_mean, result.gini_std, result.mean_final_agents
            );
        }
        println!();
    }

    /// Get statistics for a specific metric across all grid points
    pub fn metric_stats(&self, extractor: impl Fn(&BatchResult) -> f64) -> GridMetricStats {
        let values: Vec<f64> = self
            .results
            .iter()
            .map(extractor)
            .filter(|v| v.is_finite())
            .collect();
        if values.is_empty() {
            return GridMetricStats {
                min: 0.0,
                max: 0.0,
                mean: 0.0,
                std: 0.0,
            };
        }
        let mean = values.iter().sum::<f64>() / values.len() as f64;

        GridMetricStats {
            min: values.iter().copied().fold(f64::INFINITY, f64::min),
            max: values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            mean,
            std: compute_std(&values, mean),
        }
        // TAG: surface=simulation owner=research-team rule=SIM-001
    }

    /// Find the grid point with the best (maximum) value for a metric
    pub fn find_best(
        &self,
        extractor: impl Fn(&BatchResult) -> f64,
    ) -> Option<(usize, &GridPoint, f64)> {
        self.results
            .iter()
            .enumerate()
            .map(|(i, r)| (i, extractor(r)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, value)| (i, &self.points[i], value))
    }

    /// Find the grid point with the worst (minimum) value for a metric
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn find_worst(
        &self,
        extractor: impl Fn(&BatchResult) -> f64,
    ) -> Option<(usize, &GridPoint, f64)> {
        self.results
            .iter()
            .enumerate()
            .map(|(i, r)| (i, extractor(r)))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, value)| (i, &self.points[i], value))
    }

    /// Get heatmap data for two parameters (for 2D visualization)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn get_heatmap_data(
        &self,
        param_x: &str,
        param_y: &str,
        extractor: impl Fn(&BatchResult) -> f64,
    ) -> Result<HeatmapData, GridSweepError> {
        // Validate parameters exist
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        if !self.parameter_names.contains(&param_x.to_string()) {
            return Err(GridSweepError::UnknownParameter(param_x.to_string()));
        }
        if !self.parameter_names.contains(&param_y.to_string()) {
            return Err(GridSweepError::UnknownParameter(param_y.to_string()));
        }

        // Collect unique values for each parameter
        let mut x_values: Vec<f32> = self.points.iter().filter_map(|p| p.get(param_x)).collect();
        x_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        x_values.dedup_by(|a, b| (*a - *b).abs() < 1e-6);

        let mut y_values: Vec<f32> = self.points.iter().filter_map(|p| p.get(param_y)).collect();
        y_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        y_values.dedup_by(|a, b| (*a - *b).abs() < 1e-6);

        // Build 2D grid
        let mut grid: Vec<Vec<Option<f64>>> = vec![vec![None; x_values.len()]; y_values.len()];

        for (point, result) in self.points.iter().zip(self.results.iter()) {
            if let (Some(x), Some(y)) = (point.get(param_x), point.get(param_y)) {
                let x_idx = x_values
                    .iter()
                    .position(|v| (v - x).abs() < 1e-6)
                    .ok_or_else(|| GridSweepError::InvalidData(format!("X value {x} not found")))?;
                let y_idx = y_values
                    .iter()
                    .position(|v| (v - y).abs() < 1e-6)
                    .ok_or_else(|| GridSweepError::InvalidData(format!("Y value {y} not found")))?;
                grid[y_idx][x_idx] = Some(extractor(result));
            }
        }

        Ok(HeatmapData {
            x_param: param_x.to_string(),
            y_param: param_y.to_string(),
            x_values,
            y_values,
            grid,
        })
    }
    // TAG: surface=simulation owner=research-team rule=SIM-001

    /// Identify optimal parameter regions based on a metric
    pub fn identify_optimal_regions(
        &self,
        extractor: impl Fn(&BatchResult) -> f64,
        threshold_percentile: f64,
    ) -> Vec<OptimalRegion> {
        let values: Vec<f64> = self.results.iter().map(&extractor).collect();
        let threshold = percentile(&values, threshold_percentile);

        self.results
            .iter()
            .enumerate()
            .filter(|(_, r)| extractor(r) >= threshold)
            .map(|(i, r)| OptimalRegion {
                point_index: i,
                point: self.points[i].clone(),
                metric_value: extractor(r),
            })
            .collect()
    }
}

/// Compute percentile of a slice
/// # Errors
///
/// Returns an error if the operation fails.
pub(crate) fn percentile(values: &[f64], p: f64) -> f64 {
    let mut sorted: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if sorted.is_empty() {
        return 0.0;
    }
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let idx = ((p / 100.0) * (sorted.len().saturating_sub(1)) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

/// Grid sweep executor
#[derive(Debug)]
pub struct GridSweepRunner {
    config: GridSweepConfig,
    // TAG: surface=simulation owner=research-team rule=SIM-001
}

impl GridSweepRunner {
    /// Create a new grid sweep runner
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[must_use]
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub const fn new(config: GridSweepConfig) -> Self {
        Self { config }
    }

    /// Run the grid sweep
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn run(&self) -> Result<GridSweepResult, BatchError> {
        // Generate all combinations (Cartesian product)
        let combinations = self.generate_combinations();

        if combinations.is_empty() {
            return Ok(GridSweepResult {
                points: Vec::new(),
                results: Vec::new(),
                parameter_names: Vec::new(),
            });
        }

        let parameter_names: Vec<String> = self
            .config
            .parameters
            .iter()
            .map(|(name, _)| name.clone())
            .collect();

        // Run simulations for each combination
        let results: Vec<(GridPoint, BatchResult)> = combinations
            .par_iter()
            // TAG: surface=simulation owner=research-team rule=SIM-001
            .map(|values| {
                let config = self.create_run_config(values)?;
                let runner = BatchRunner::new(config);
                let result = runner.run(self.config.runs_per_config as u32)?;
                let point = GridPoint::new(values.clone());
                Ok((point, result))
            })
            .collect::<Result<Vec<_>, BatchError>>()?;

        let (points, batch_results): (Vec<GridPoint>, Vec<BatchResult>) =
            results.into_iter().unzip();

        Ok(GridSweepResult {
            points,
            results: batch_results,
            parameter_names,
        })
    }

    /// Generate all combinations of parameter values (Cartesian product)
    pub(crate) fn generate_combinations(&self) -> Vec<HashMap<String, f32>> {
        // If no parameters, return empty vector
        if self.config.parameters.is_empty() {
            return Vec::new();
        }

        // P1 FIX: Pre-allocate with estimated capacity to reduce reallocations
        // Estimate: product of all parameter value counts, or at least 4
        let estimated_capacity: usize = self
            .config
            .parameters
            .iter()
            .map(|(_, v)| v.len())
            .product::<usize>()
            .max(4);

        let mut combinations: Vec<HashMap<String, f32>> = Vec::with_capacity(estimated_capacity);
        combinations.push(HashMap::new());

        for (param_name, values) in &self.config.parameters {
            // P1 FIX: Pre-allocate new_combinations with exact capacity needed
            let new_capacity = combinations.len() * values.len();
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            let mut new_combinations = Vec::with_capacity(new_capacity);

            for combination in &combinations {
                for &value in values {
                    let mut new_combination = combination.clone();
                    new_combination.insert(param_name.clone(), value);
                    new_combinations.push(new_combination);
                }
            }

            combinations = new_combinations;
        }

        combinations
    }

    /// Create a `RunConfig` with the specified parameter values
    fn create_run_config(&self, values: &HashMap<String, f32>) -> Result<RunConfig, BatchError> {
        let mut config = self.config.base_config.clone();

        for (param, &value) in values {
            self.apply_parameter(&mut config, param, value)?;
        }

        Ok(config)
    }

    /// Apply a parameter value to the configuration
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn apply_parameter(
        &self,
        config: &mut RunConfig,
        param: &str,
        value: f32,
    ) -> Result<(), BatchError> {
        match param {
            // Simulation parameters
            "agent_count" => config.simulation.agent_count = value as u32,
            "max_ticks" => config.simulation.max_ticks = value as u64,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            "grid_width" => config.simulation.grid_width = value as usize,
            "grid_height" => config.simulation.grid_height = value as usize,
            "seed" => config.simulation.seed = value as u64,

            // DOCAS alpha parameters (genome ranges)
            "docas_alpha_d" => config.simulation.docas_alpha_d = Some(value),
            "docas_alpha_o" => config.simulation.docas_alpha_o = Some(value),
            "docas_alpha_c" => config.simulation.docas_alpha_c = Some(value),
            "docas_alpha_a" => config.simulation.docas_alpha_a = Some(value),
            "docas_alpha_s" => config.simulation.docas_alpha_s = Some(value),

            // DOCAS beta parameters (genome ranges)
            "docas_beta_d" => config.simulation.docas_beta_d = Some(value),
            "docas_beta_o" => config.simulation.docas_beta_o = Some(value),
            "docas_beta_c" => config.simulation.docas_beta_c = Some(value),
            "docas_beta_a" => config.simulation.docas_beta_a = Some(value),
            "docas_beta_s" => config.simulation.docas_beta_s = Some(value),

            // Planning parameters
            "planning_depth_k" => config.simulation.planning_depth_k = Some(value as u32),
            "planning_horizon_h" => config.simulation.planning_horizon_h = Some(value as u64),

            // Resource parameters
            "initial_resource" => config.simulation.initial_resource = Some(value),
            "resource_density" => config.simulation.resource_density = Some(value),

            // Protocol parameters
            "verification_cost" => config.simulation.verification_cost = Some(value),
            "attestation_stake" => config.simulation.attestation_stake = Some(value),

            _ => {
                return Err(BatchError::ConfigError(format!(
                    "Grid sweep: Unknown parameter: {param}"
                )))
            }
        }
        Ok(())
    }
}

/// Convenience function for running a grid sweep
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
///
/// # Example
///
/// ```
/// use sim_runner::{RunConfig, sweep::GridSweepConfig};
///
/// let base_config = RunConfig::default();
/// let grid_config = GridSweepConfig::new(base_config, 3)
///     .add_parameter("agent_count", vec![50.0, 100.0, 200.0])
///     .add_parameter("max_ticks", vec![500.0, 1000.0])
///     .clone();
///
/// // Run the grid sweep
/// // let result = sim_runner::sweep::run_grid_sweep(&grid_config)?;
/// ```
/// # Errors
///
/// Returns an error if the operation fails.
pub fn run_grid_sweep(config: &GridSweepConfig) -> Result<GridSweepResult, BatchError> {
    let runner = GridSweepRunner::new(config.clone());
    runner.run()
}
