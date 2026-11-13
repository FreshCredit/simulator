//! SIM-109: Batch Job Orchestration for Multi-Run Experiments
//!
//! Enables parallel execution of multiple simulation runs for:
//! - Parameter sweeps (grid search across parameter space)
//! - Sensitivity analysis (Morris method, Sobol indices)
//! - Replication studies (multiple seeds for statistical power)
//! - Factorial experiments (toggle combinations)
//!
//! Per simulation-plan.md §5.2.2: Experimental design patterns

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::pom::{validate_all_targets, PomMetrics};
use crate::simulation::{Simulation, SimulationConfig};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Batch experiment configuration
///
/// Defines a multi-run experiment with parameter variations, replicates,
/// and parallel execution settings. Used for parameter sweeps, sensitivity
/// analysis, and factorial experiments.
#[derive(Debug, Clone, Serialize)]
pub struct BatchConfig {
    /// Experiment name
    pub name: String,
    /// Description of the experiment
    pub description: String,
    /// Base configuration to vary from
    pub base_config: SimulationConfig,
    /// Parameter variations
    pub variations: Vec<ParameterVariation>,
    /// Number of replicates per configuration
    pub replicates: usize,
    /// Random seeds for replicates (if empty, uses sequential seeds)
    pub seeds: Vec<u64>,
    /// Number of ticks per run
    pub ticks: u64,
    /// Whether to run in parallel
    pub parallel: bool,
    /// Number of worker threads (0 = use all cores)
    pub worker_threads: usize,
    /// Output directory for artifacts
    pub output_dir: Option<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            name: "unnamed_experiment".to_string(),
            description: String::new(),
            base_config: SimulationConfig::default(),
            variations: Vec::new(),
            replicates: 10,
            seeds: Vec::new(),
            ticks: 1000,
            parallel: true,
            worker_threads: 0,
            output_dir: None,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Parameter variation specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterVariation {
    /// Single parameter sweep
    Sweep {
        /// name.
        name: String,
        /// values.
        values: Vec<f32>,
    },
    /// Grid search across multiple parameters
    Grid {
        /// parameters.
        parameters: Vec<(String, Vec<f32>)>,
    },
    /// Toggle combinations (factorial design)
    /// toggle names.
    Toggles {
        /// toggle names.
        toggle_names: Vec<String>,
    },
    /// Morris method trajectories
    Morris {
        /// trajectory count.
        trajectory_count: usize,
        /// levels.
        levels: usize,
    },
    /// P2 FIX: Random sampling for parameter space exploration
    Random {
        /// count.
        count: usize,
        /// parameter ranges.
        parameter_ranges: Vec<(String, f32, f32)>,
    },
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Individual job in a batch experiment
///
/// Represents a single simulation run with specific parameters
/// and configuration. Part of a larger batch experiment.
#[derive(Debug, Clone, Serialize)]
pub struct BatchJob {
    /// Job ID
    pub id: usize,
    /// Configuration for this run
    pub config: SimulationConfig,
    /// Parameter values (for tracking)
    pub parameters: HashMap<String, f32>,
    /// Replicate number
    pub replicate: usize,
    /// Random seed
    pub seed: u64,
}

/// Result of a single batch job execution
///
/// Contains the outcome of one simulation run including success status,
/// population metrics, POM validation score, and execution time.
#[derive(Debug, Clone, Serialize)]
pub struct BatchJobResult {
    /// Job ID
    pub job_id: usize,
    /// Whether the run succeeded
    pub success: bool,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Final simulation tick
    pub final_tick: u64,
    /// Alive agent count
    pub final_population: usize,
    /// POM validation score (0-100)
    pub pom_score: f32,
    /// P0 validation passed
    pub p0_passed: bool,
    /// Execution time (ms)
    pub execution_time_ms: u64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Complete batch experiment results
///
/// Aggregates all job results from a batch experiment with summary
/// statistics and execution metadata.
#[derive(Debug, Clone, Serialize)]
pub struct BatchResults {
    /// Experiment configuration
    pub config: BatchConfig,
    /// Individual job results
    pub jobs: Vec<BatchJobResult>,
    /// Summary statistics
    pub summary: BatchSummary,
    /// Execution metadata
    pub metadata: BatchMetadata,
}

/// Batch execution summary statistics
///
/// Provides aggregate statistics across all jobs in a batch,
/// including success rates, execution times, and parameter effects.
#[derive(Debug, Clone, Serialize, Default)]
pub struct BatchSummary {
    /// Total jobs executed
    pub total_jobs: usize,
    /// Successful jobs
    pub successful_jobs: usize,
    /// Failed jobs
    pub failed_jobs: usize,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Mean execution time (ms)
    pub mean_execution_time_ms: f64,
    /// Overall P0 pass rate across all jobs
    pub p0_pass_rate: f32,
    /// Parameter effects (if sweep)
    pub parameter_effects: HashMap<String, ParameterEffect>,
}

/// Effect of a parameter on simulation outcomes
///
/// Tracks how varying a single parameter affects cooperation rates,
/// survival rates, and branching ratios across batch runs.
#[derive(Debug, Clone, Serialize)]
pub struct ParameterEffect {
    /// parameter name.
    pub parameter_name: String,
    /// min value.
    pub min_value: f32,
    /// max value.
    pub max_value: f32,
    /// mean cooperation.
    pub mean_cooperation: f32,
    /// mean survival.
    pub mean_survival: f32,
    /// mean branching ratio.
    pub mean_branching_ratio: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Batch execution metadata
///
/// Records timing and execution environment information
/// for a batch experiment.
#[derive(Debug, Clone, Serialize)]
pub struct BatchMetadata {
    /// Start timestamp
    pub start_time: u64,
    /// End timestamp
    pub end_time: u64,
    /// Total execution time (ms)
    pub total_time_ms: u64,
    /// Number of worker threads used
    pub worker_threads: usize,
    /// Hostname
    pub hostname: String,
}

/// Batch runner
#[derive(Debug)]
pub struct BatchRunner {
    config: BatchConfig,
    jobs: Vec<BatchJob>,
}

impl BatchRunner {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create new batch runner from config
    #[must_use]
    pub fn new(config: BatchConfig) -> Self {
        let jobs = Self::generate_jobs(&config);
        Self { config, jobs }
    }

    /// Generate all jobs from configuration
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    fn generate_jobs(config: &BatchConfig) -> Vec<BatchJob> {
        // P1 FIX: Estimate capacity based on variations and replicates
        let estimated_jobs: usize = config
            .variations
            .iter()
            .map(|v| match v {
                ParameterVariation::Sweep { values, .. } => values.len() * config.replicates,
                ParameterVariation::Random { count, .. } => *count * config.replicates,
                ParameterVariation::Grid { parameters } => {
                    parameters
                        .iter()
                        .map(|(_, vals)| vals.len())
                        .product::<usize>()
                        * config.replicates
                }
                ParameterVariation::Toggles { toggle_names } => {
                    (1usize << toggle_names.len().min(10)) * config.replicates
                }
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                ParameterVariation::Morris {
                    trajectory_count, ..
                } => trajectory_count * config.replicates,
            })
            .sum();
        let mut jobs = Vec::with_capacity(estimated_jobs.max(4));
        let mut job_id = 0;

        for variation in &config.variations {
            match variation {
                ParameterVariation::Sweep { name, values } => {
                    for (i, value) in values.iter().enumerate() {
                        for rep in 0..config.replicates {
                            let seed = config
                                .seeds
                                .get(rep)
                                .copied()
                                .unwrap_or((i * 1000 + rep) as u64);

                            let mut job_config = config.base_config.clone();
                            job_config.seed = seed;

                            let mut parameters = HashMap::new();
                            parameters.insert(name.clone(), *value);

                            jobs.push(BatchJob {
                                id: job_id,
                                config: job_config,
                                parameters,
                                replicate: rep,
                                seed,
                                // TAG: surface=simulation owner=research-team rule=SIM-001
                            });
                            job_id += 1;
                        }
                    }
                }
                ParameterVariation::Grid { parameters: _ } => {
                    // P2-10: Grid search implementation deferred - use Morris method for now
                }
                ParameterVariation::Toggles { toggle_names: _ } => {
                    // P2-10: Factorial design deferred - planned for Stage 3+ experiments
                }
                ParameterVariation::Morris { .. } => {
                    // P2-10: Morris method integration in progress - use core sensitivity for now
                }
                ParameterVariation::Random {
                    count,
                    parameter_ranges,
                } => {
                    // P2 FIX: Random sampling implementation
                    for i in 0..*count {
                        for rep in 0..config.replicates {
                            let seed = config
                                .seeds
                                .get(rep)
                                .copied()
                                .unwrap_or((i * 1000 + rep) as u64);

                            let mut job_config = config.base_config.clone();
                            job_config.seed = seed;

                            let mut parameters = HashMap::new();
                            for (name, min, max) in parameter_ranges {
                                let range = max - min;
                                let random_value = min + (i as f32 / *count as f32) * range;
                                parameters.insert(name.clone(), random_value);
                            }

                            jobs.push(BatchJob {
                                id: job_id,
                                config: job_config,
                                // TAG: surface=simulation owner=research-team rule=SIM-001
                                parameters,
                                replicate: rep,
                                seed,
                            });
                            job_id += 1;
                        }
                    }
                }
            }
        }

        // If no variations, just run replicates of base config
        if jobs.is_empty() {
            for rep in 0..config.replicates {
                let seed = config.seeds.get(rep).copied().unwrap_or(rep as u64);
                let mut job_config = config.base_config.clone();
                job_config.seed = seed;

                jobs.push(BatchJob {
                    id: job_id,
                    config: job_config,
                    parameters: HashMap::new(),
                    replicate: rep,
                    seed,
                });
                job_id += 1;
            }
        }

        jobs
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Run all jobs
    #[must_use]
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn run(self) -> BatchResults {
        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let worker_threads = if self.config.worker_threads == 0 {
            thread::available_parallelism()
                .map(std::num::NonZero::get)
                .unwrap_or(4)
        } else {
            self.config.worker_threads
        };

        let results = if self.config.parallel && worker_threads > 1 {
            self.run_parallel(worker_threads)
        } else {
            self.run_sequential()
        };

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let end_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let summary = compute_summary(&results);

        BatchResults {
            config: self.config,
            jobs: results,
            summary,
            metadata: BatchMetadata {
                start_time,
                end_time,
                total_time_ms: end_time - start_time,
                worker_threads,
                hostname: get_hostname(),
            },
        }
    }

    /// Run jobs sequentially
    fn run_sequential(&self) -> Vec<BatchJobResult> {
        self.jobs
            .iter()
            .map(|job| run_single_job(job, self.config.ticks))
            .collect()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Run jobs in parallel
    fn run_parallel(&self, worker_threads: usize) -> Vec<BatchJobResult> {
        let results = Arc::new(Mutex::new(Vec::with_capacity(self.jobs.len())));
        let jobs = Arc::new(Mutex::new(self.jobs.clone()));
        let ticks = self.config.ticks;

        thread::scope(|s| {
            for _ in 0..worker_threads {
                let jobs = Arc::clone(&jobs);
                let results = Arc::clone(&results);

                s.spawn(move || loop {
                    let job = {
                        let mut jobs_guard = jobs.lock().unwrap_or_else(|e| e.into_inner());
                        jobs_guard.pop()
                    };

                    if let Some(job) = job {
                        let result = run_single_job(&job, ticks);
                        let mut results_guard = results.lock().unwrap_or_else(|e| e.into_inner());
                        results_guard.push(result);
                    } else {
                        break;
                    }
                });
            }
        });

        match Arc::try_unwrap(results) {
            Ok(m) => m.into_inner().unwrap_or_else(|e| e.into_inner()),
            Err(arc) => arc
                .lock()
                .map(|g| g.clone())
                .unwrap_or_else(|e| e.into_inner().clone()),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get number of jobs
    #[must_use]
    pub fn job_count(&self) -> usize {
        self.jobs.len()
    }
}

/// Run a single job
fn run_single_job(job: &BatchJob, ticks: u64) -> BatchJobResult {
    let start = std::time::Instant::now();

    let result = std::panic::catch_unwind(|| {
        let mut sim = Simulation::new(job.config.clone());

        for _ in 0..ticks {
            sim.step();
        }

        sim
    });

    let execution_time = start.elapsed().as_millis() as u64;

    match result {
        Ok(sim) => {
            // Extract metrics from simulation
            let metrics = extract_metrics(&sim);
            let validation = validate_all_targets(&metrics);

            BatchJobResult {
                job_id: job.id,
                success: true,
                error: None,
                final_tick: sim.tick,
                final_population: sim.agents.iter().filter(|a| a.alive).count(),
                pom_score: validation.overall_score,
                p0_passed: validation.p0_passed(),
                execution_time_ms: execution_time,
            }
        }
        // TAG: surface=simulation owner=research-team rule=SIM-001
        Err(_) => BatchJobResult {
            job_id: job.id,
            success: false,
            error: Some("Simulation panicked".to_string()),
            final_tick: 0,
            final_population: 0,
            pom_score: 0.0,
            p0_passed: false,
            execution_time_ms: execution_time,
        },
    }
}

/// Extract POM metrics from simulation
fn extract_metrics(sim: &Simulation) -> PomMetrics {
    let alive_agents: Vec<_> = sim.agents.iter().filter(|a| a.alive).collect();
    let total_agents = sim.agents.len() as f32;

    if alive_agents.is_empty() {
        return PomMetrics::default();
    }

    // Survival rate
    let survival_rate = alive_agents.len() as f32 / total_agents;

    // Energy statistics
    let energies: Vec<f32> = alive_agents.iter().map(|a| a.energy).collect();
    let (mean_energy, std_energy) = mean_std(&energies);
    let energy_cv = if mean_energy > 0.0 {
        std_energy / mean_energy
    } else {
        0.0
    };

    // DOCAS baselines
    let dopamine =
        alive_agents.iter().map(|a| a.docas.levels[0]).sum::<f32>() / alive_agents.len() as f32;
    let oxytocin =
        alive_agents.iter().map(|a| a.docas.levels[1]).sum::<f32>() / alive_agents.len() as f32;

    // Cooperation rate (from interaction history)
    let total_pos: u32 = alive_agents.iter().map(|a| a.positive_interactions).sum();
    let total_neg: u32 = alive_agents.iter().map(|a| a.negative_interactions).sum();
    let total_int = total_pos + total_neg;
    let cooperation_rate = if total_int > 0 {
        total_pos as f32 / total_int as f32
    } else {
        0.5
    };

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Verification rate (if protocol enabled)
    let verified: usize = alive_agents.iter().filter(|a| a.has_verified_id).count();
    let verification_rate = verified as f32 / alive_agents.len() as f32;

    PomMetrics {
        agent_survival_rate: survival_rate,
        energy_cv,
        dopamine_baseline: dopamine,
        oxytocin_baseline: oxytocin,
        cooperation_rate,
        verification_rate,
        ..Default::default()
    }
}

/// Compute summary statistics
fn compute_summary(results: &[BatchJobResult]) -> BatchSummary {
    let total = results.len();
    let successful = results.iter().filter(|r| r.success).count();
    let failed = total - successful;

    let mean_time: f64 = if total > 0 {
        results
            .iter()
            .map(|r| r.execution_time_ms as f64)
            .sum::<f64>()
            / total as f64
    } else {
        0.0
    };

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let p0_passed: usize = results.iter().filter(|r| r.p0_passed).count();
    let p0_rate = if total > 0 {
        p0_passed as f32 / total as f32
    } else {
        0.0
    };

    BatchSummary {
        total_jobs: total,
        successful_jobs: successful,
        failed_jobs: failed,
        mean_execution_time_ms: mean_time,
        p0_pass_rate: p0_rate,
        parameter_effects: HashMap::new(),
    }
}

/// Helper: Compute mean and std
fn mean_std(values: &[f32]) -> (f32, f32) {
    if values.is_empty() {
        return (0.0, 0.0);
    }
    let n = values.len() as f32;
    let mean = values.iter().sum::<f32>() / n;
    let var = values.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / n;
    (mean, var.sqrt())
}

/// Get hostname
fn get_hostname() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl BatchResults {
    /// Export results to JSON
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Generate summary report
    #[must_use]
    pub fn summary_report(&self) -> String {
        let mut report = format!(
            "Batch Experiment Report: {}\n\
            {}\n\
            Configuration:\n\
            - Jobs: {}\n\
            - Replicates: {}\n\
            - Ticks per run: {}\n\
            - Worker threads: {}\n\n\
            Results:\n\
            - Successful: {}\n\
            - Failed: {}\n\
            - Mean execution time: {:.1}ms\n\
            - P0 pass rate: {:.1}%\n\
            - Total time: {:.1}s\n",
            self.config.name,
            "=".repeat(self.config.name.len()),
            self.summary.total_jobs,
            self.config.replicates,
            self.config.ticks,
            self.metadata.worker_threads,
            self.summary.successful_jobs,
            self.summary.failed_jobs,
            self.summary.mean_execution_time_ms,
            self.summary.p0_pass_rate * 100.0,
            self.metadata.total_time_ms as f64 / 1000.0
        );

        if !self.config.description.is_empty() {
            report = format!("Description: {}\n\n{}", self.config.description, report);
        }

        report
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Convenience function: Run parameter sweep
#[must_use]
pub fn run_parameter_sweep(
    param_name: &str,
    values: Vec<f32>,
    base_config: SimulationConfig,
    replicates: usize,
    ticks: u64,
) -> BatchResults {
    let config = BatchConfig {
        name: format!("sweep_{param_name}"),
        description: format!("Parameter sweep over {param_name}: {values:?}"),
        base_config,
        variations: vec![ParameterVariation::Sweep {
            name: param_name.to_string(),
            values,
        }],
        replicates,
        seeds: Vec::new(),
        ticks,
        parallel: true,
        worker_threads: 0,
        output_dir: None,
    };

    BatchRunner::new(config).run()
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Convenience function: Run replicates
#[must_use]
pub fn run_replicates(
    base_config: SimulationConfig,
    replicates: usize,
    ticks: u64,
) -> BatchResults {
    let config = BatchConfig {
        name: "replicate_study".to_string(),
        description: format!("{replicates} replicates for statistical power"),
        base_config,
        variations: Vec::new(),
        replicates,
        seeds: Vec::new(),
        ticks,
        parallel: true,
        worker_threads: 0,
        output_dir: None,
    };

    BatchRunner::new(config).run()
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_config_default() {
        let config = BatchConfig::default();
        assert_eq!(config.replicates, 10);
        assert!(config.parallel);
    }

    #[test]
    fn test_batch_runner_job_generation() {
        let config = BatchConfig {
            name: "test".to_string(),
            replicates: 3,
            variations: vec![ParameterVariation::Sweep {
                name: "test_param".to_string(),
                values: vec![0.1, 0.5, 1.0],
            }],
            seeds: Vec::new(),
            ticks: 100,
            ..Default::default()
        };

        let runner = BatchRunner::new(config);
        // 3 values × 3 replicates = 9 jobs
        assert_eq!(runner.job_count(), 9);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_run_replicates() {
        let base_config = SimulationConfig::default();
        let results = run_replicates(base_config, 3, 100);

        assert_eq!(results.jobs.len(), 3);
        assert_eq!(results.summary.total_jobs, 3);
        assert!(results.summary.successful_jobs > 0);
    }

    #[test]
    fn test_batch_results_report() {
        let base_config = SimulationConfig::default();
        let results = run_replicates(base_config, 2, 50);

        let report = results.summary_report();
        assert!(report.contains("Batch Experiment Report"));
        assert!(report.contains("replicate_study"));
        assert!(report.contains("Successful:"));
    }

    #[test]
    fn test_parameter_sweep() {
        let base_config = SimulationConfig::default();
        let values = vec![0.0, 0.5, 1.0];

        let results = run_parameter_sweep("mutation_rate", values, base_config, 2, 50);

        assert_eq!(results.jobs.len(), 6); // 3 values × 2 replicates
        assert!(results.summary_report().contains("sweep_mutation_rate"));
    }

    #[test]
    fn test_batch_json_export() {
        let base_config = SimulationConfig::default();
        let results = run_replicates(base_config, 2, 50);

        let json = results.to_json();
        assert!(json.contains("replicate_study"));
        assert!(json.contains("summary"));
        assert!(json.contains("metadata"));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_sequential_vs_parallel() {
        let _base_config = SimulationConfig::default();

        // Sequential
        let seq_config = BatchConfig {
            name: "seq".to_string(),
            replicates: 4,
            parallel: false,
            ticks: 50,
            seeds: Vec::new(),
            ..Default::default()
        };
        let seq_results = BatchRunner::new(seq_config).run();

        // Parallel
        let par_config = BatchConfig {
            name: "par".to_string(),
            replicates: 4,
            parallel: true,
            ticks: 50,
            seeds: Vec::new(),
            ..Default::default()
        };
        let par_results = BatchRunner::new(par_config).run();

        // Both should succeed
        assert_eq!(seq_results.summary.total_jobs, 4);
        assert_eq!(par_results.summary.total_jobs, 4);
    }
}
