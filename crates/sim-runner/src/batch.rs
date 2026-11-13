//! Batch execution orchestration for sim-runner
//!
//! Runs multiple simulation replications and aggregates results.
//!
//! # Architecture
//!
//! - Parallel execution using rayon
//! - Per-run artifact generation
//! - Aggregate statistics with confidence intervals
//!
//! # Example
//!
//! ```no_run
//! use sim_runner::{RunConfig, BatchRunner};
//!
//! let config = RunConfig::default();
//! let runner = BatchRunner::new(config);
//! let results = runner.run(5)?; // Run 5 replications
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::config::RunConfig;
use crate::stats::{gini_coefficient, mean, mean_with_ci, std_dev};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use sim_artifacts::{Manifest, MetricsWriter, ReplayFrame, ReplayWriter, TickMetrics};
use sim_core::{
    environment::EnvironmentConfig,
    metrics::docas_magnitude,
    simulation::{Simulation, SimulationConfig},
    toggles::ToggleConfig as SimToggleConfig,
    Position,
};
use std::path::PathBuf;

/// Batch execution runner
#[derive(Debug)]
pub struct BatchRunner {
    config: RunConfig,
}

impl BatchRunner {
    /// Create a new batch runner with the given configuration
    #[must_use]
    pub const fn new(config: RunConfig) -> Self {
        Self { config }
    }

    /// Run batch execution with the specified number of replications
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn run(&self, replications: u32) -> Result<BatchResult, BatchError> {
        self.config
            .validate()
            .map_err(|e| BatchError::ConfigError(e.to_string()))?;

        let seeds: Vec<u64> = (0..replications)
            .map(|i| self.config.simulation.seed.wrapping_add(i as u64 * 1000003))
            .collect();

        // Parallel execution
        let run_results: Vec<RunResult> = if self.config.batch.parallel {
            seeds
                .par_iter()
                .enumerate()
                .map(|(idx, &seed)| self.run_single(seed, idx))
                .collect::<Result<Vec<_>, _>>()?
        } else {
            seeds
                .iter()
                .enumerate()
                .map(|(idx, &seed)| self.run_single(seed, idx))
                .collect::<Result<Vec<_>, _>>()?
        };

        // Aggregate results
        Ok(self.aggregate_results(&run_results))
    }

    /// Run a single simulation with the given seed
    fn run_single(&self, seed: u64, run_index: usize) -> Result<RunResult, BatchError> {
        let mut sim = self.create_simulation(seed);
        let output_dir = self.setup_output_dir(run_index)?;
        let mut metrics_writer = self.create_metrics_writer(&output_dir)?;
        let mut tick_metrics_vec: Vec<TickMetrics> = Vec::new();

        // P10-R4: bounded — exits when sim.tick reaches max_ticks or the agent population empties.
        while sim.tick < self.config.simulation.max_ticks && !sim.agents.is_empty() {
            sim.step();
            let tick_metrics = self.compute_tick_metrics(&sim);
            tick_metrics_vec.push(tick_metrics.clone());

            if let Some(ref mut writer) = metrics_writer {
                writer
                    .write(&tick_metrics)
                    .map_err(|e| BatchError::IoError(e.to_string()))?;
            }

            self.write_replay_if_needed(&sim, &output_dir)?;
            // TAG: surface=simulation owner=research-team rule=SIM-001
        }

        if let Some(writer) = metrics_writer {
            writer
                .finish()
                .map_err(|e| BatchError::IoError(e.to_string()))?;
        }

        self.write_manifest(&output_dir, seed)?;
        self.build_run_result(&sim, run_index, &tick_metrics_vec)
    }

    fn create_simulation(&self, seed: u64) -> Simulation {
        let toggles = self.create_toggle_config();
        let mut sim_config = SimulationConfig {
            seed,
            max_ticks: self.config.simulation.max_ticks,
            environment: EnvironmentConfig {
                width: self.config.simulation.grid_width,
                height: self.config.simulation.grid_height,
                depth: 1,
                resource_density: self.config.simulation.resource_density.unwrap_or(0.3),
                initial_resource: self.config.simulation.initial_resource.unwrap_or(100.0),
                altitude_effects_enabled: false,
                weather_enabled: false,
            },
            toggles,
            ..Default::default()
        };

        if let Some(v) = self.config.simulation.docas_alpha_d {
            sim_config.docas_alpha_base = v;
        }
        if let Some(v) = self.config.simulation.planning_horizon_h {
            sim_config.planning_horizon = v;
        }
        if let Some(v) = self.config.simulation.verification_cost {
            sim_config.transaction_cost = v;
        }

        let mut sim = Simulation::new(sim_config);
        let mut rng = StdRng::seed_from_u64(seed);
        for _ in 0..self.config.simulation.agent_count {
            let x: f32 = rng.gen_range(0..self.config.simulation.grid_width) as f32;
            let y: f32 = rng.gen_range(0..self.config.simulation.grid_height) as f32;
            sim.spawn_agent(Position::new_2d(x, y));
        }
        sim
    }

    fn setup_output_dir(&self, run_index: usize) -> Result<Option<PathBuf>, BatchError> {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        if self.config.output.dir.is_empty() {
            return Ok(None);
        }
        let dir = PathBuf::from(&self.config.output.dir).join(format!("run_{run_index:04}"));
        std::fs::create_dir_all(&dir).map_err(|e| BatchError::IoError(e.to_string()))?;
        Ok(Some(dir))
    }

    #[allow(clippy::ref_option)]
    fn create_metrics_writer(
        &self,
        output_dir: &Option<PathBuf>,
    ) -> Result<Option<MetricsWriter>, BatchError> {
        let Some(ref dir) = output_dir else {
            return Ok(None);
        };
        let metrics_path = dir.join("metrics.jsonl");
        MetricsWriter::new(&metrics_path)
            .map(Some)
            .map_err(|e| BatchError::IoError(e.to_string()))
    }

    fn compute_tick_metrics(&self, sim: &Simulation) -> TickMetrics {
        let alive_count = sim.agents.iter().filter(|a| a.alive).count();
        let energies: Vec<f64> = sim
            .agents
            .iter()
            .filter(|a| a.alive)
            .map(|a| a.energy as f64)
            .collect();
        let avg_energy = mean(&energies);
        let gini = gini_coefficient(&energies);

        let docas_mags: Vec<f64> = sim
            .agents
            .iter()
            .filter(|a| a.alive)
            .map(|a| docas_magnitude(&a.docas) as f64)
            .collect();
        let avg_docas = mean(&docas_mags);

        let sigma = sim_core::metrics::compute_branching_ratio(
            sim.metrics.births_this_tick,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            sim.metrics.deaths_this_tick,
            sim.metrics.encounters_this_tick,
            alive_count,
        );

        let (total_pos, total_neg): (u32, u32) = sim
            .agents
            .iter()
            .filter(|a| a.alive)
            .map(|a| (a.positive_interactions, a.negative_interactions))
            .fold((0, 0), |(p, n), (ap, an)| (p + ap, n + an));
        let cooperation_rate = if total_pos + total_neg > 0 {
            total_pos as f64 / (total_pos + total_neg) as f64
        } else {
            0.5
        };

        let dispute_rate = if sim.metrics.encounters_this_tick > 0 {
            sim.metrics.disputes_this_tick as f64 / sim.metrics.encounters_this_tick as f64
        } else {
            0.0
        };

        TickMetrics {
            tick: sim.tick,
            agent_count: alive_count as u64,
            cooperation_rate,
            gini,
            trade_volume: sim.metrics.trade_volume_this_tick as u64,
            dispute_rate,
            verification_rate: sim.protocol.verification_rate() as f64,
            avg_energy,
            avg_docas_magnitude: avg_docas,
            branching_ratio: sigma,
            active_norms: sim.active_norms.len() as u64,
            institution_count: sim.institutions.institutions.len() as u64,
            total_anchors: sim.protocol.total_anchors,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            total_verifications: sim.protocol.total_verifications,
        }
    }

    #[allow(clippy::ref_option)]
    fn write_replay_if_needed(
        &self,
        sim: &Simulation,
        output_dir: &Option<PathBuf>,
    ) -> Result<(), BatchError> {
        if sim.tick % self.config.output.replay_interval != 0 {
            return Ok(());
        }
        let Some(ref dir) = output_dir else {
            return Ok(());
        };
        let replay_dir = dir.join("replay");
        std::fs::create_dir_all(&replay_dir).ok(); // AUDIT-OK(fire-and-forget): failure surfaces immediately below in ReplayWriter::new, which returns a logged error
        let mut replay_writer = ReplayWriter::new(&replay_dir, sim.tick)
            .map_err(|e| BatchError::IoError(e.to_string()))?;

        for (idx, agent) in sim.agents.iter().enumerate() {
            let has_verified = sim.protocol.get_agent_identity_hash(idx as u64).is_some();
            let frame = ReplayFrame {
                tick: sim.tick,
                agent_id: idx as u64,
                x: agent.position.x,
                y: agent.position.y,
                energy: agent.energy,
                docas: agent.docas.to_array(),
                alive: agent.alive,
                has_verified_id: has_verified,
                group_id: agent.group_id,
                age: agent.age,
            };
            replay_writer
                .write(&frame)
                .map_err(|e| BatchError::IoError(e.to_string()))?;
        }
        // TAG: surface=simulation owner=research-team rule=SIM-001
        replay_writer
            .finish()
            .map_err(|e| BatchError::IoError(e.to_string()))
    }

    #[allow(clippy::ref_option)]
    fn write_manifest(&self, output_dir: &Option<PathBuf>, seed: u64) -> Result<(), BatchError> {
        let Some(ref dir) = output_dir else {
            return Ok(());
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        };
        let git_commit = std::env::var("GIT_COMMIT").unwrap_or_else(|_| "unknown".to_string());
        let mut manifest = Manifest::new(seed, git_commit);
        manifest.config.agent_count = self.config.simulation.agent_count;
        manifest.config.max_ticks = self.config.simulation.max_ticks;
        manifest.config.grid_width = self.config.simulation.grid_width as u32;
        manifest.config.grid_height = self.config.simulation.grid_height as u32;
        manifest.config.stage = format!("{}", self.config.simulation.stage);
        manifest.metrics_file = "metrics.jsonl".to_string();
        manifest.complete();
        let json = manifest
            .to_json()
            .map_err(|e| BatchError::IoError(e.to_string()))?;
        std::fs::write(dir.join("manifest.json"), json)
            .map_err(|e| BatchError::IoError(e.to_string()))
    }

    #[allow(clippy::unnecessary_wraps)]
    fn build_run_result(
        &self,
        sim: &Simulation,
        run_index: usize,
        tick_metrics_vec: &[TickMetrics],
    ) -> Result<RunResult, BatchError> {
        let final_metrics = tick_metrics_vec.last().cloned();
        Ok(RunResult {
            seed: sim.config.seed,
            run_index,
            final_tick: sim.tick,
            final_agent_count: sim.agents.iter().filter(|a| a.alive).count() as u64,
            final_gini: final_metrics.as_ref().map_or(0.0, |m| m.gini),
            final_branching_ratio: final_metrics.as_ref().map_or(0.0, |m| m.branching_ratio),
            avg_docas_magnitude: mean(
                &tick_metrics_vec
                    .iter()
                    .map(|m| m.avg_docas_magnitude)
                    .collect::<Vec<_>>(),
            ),
            verification_rate: final_metrics.as_ref().map_or(0.0, |m| m.verification_rate),
        })
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create toggle config from `RunConfig`
    fn create_toggle_config(&self) -> SimToggleConfig {
        let stage = self.config.simulation.stage;
        let mut toggles = match stage {
            0 => SimToggleConfig::stage_0(),
            1 => SimToggleConfig::stage_1(),
            2 => SimToggleConfig::stage_2(),
            3 => SimToggleConfig::stage_3(),
            4 => SimToggleConfig::stage_4(),
            5 => SimToggleConfig::stage_5(),
            _ => SimToggleConfig::default(),
        };

        // Override with explicit toggle settings if provided
        if self.config.toggles.genome_enabled {
            toggles.genome_enabled = true;
        }
        if self.config.toggles.docas_enabled {
            toggles.docas_enabled = true;
        }
        if self.config.toggles.dmn_enabled {
            toggles.dmn_enabled = true;
        }
        if self.config.toggles.criticality_enabled {
            toggles.criticality_enabled = true;
        }
        if self.config.toggles.choice_architecture_enabled {
            toggles.choice_architecture_enabled = true;
        }
        if self.config.toggles.institutions_enabled {
            toggles.institutions_enabled = true;
        }
        if self.config.toggles.norms_enabled {
            toggles.norms_enabled = true;
        }
        if self.config.toggles.language_enabled {
            toggles.language_enabled = true;
        }
        if self.config.toggles.protocol_enabled {
            toggles.protocol_enabled = true;
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        toggles
    }

    /// Aggregate results from multiple runs
    fn aggregate_results(&self, results: &[RunResult]) -> BatchResult {
        let ginis: Vec<f64> = results.iter().map(|r| r.final_gini).collect();
        let branching_ratios: Vec<f64> = results.iter().map(|r| r.final_branching_ratio).collect();
        let docas_mags: Vec<f64> = results.iter().map(|r| r.avg_docas_magnitude).collect();
        let agent_counts: Vec<f64> = results.iter().map(|r| r.final_agent_count as f64).collect();

        let (gini_mean, gini_ci_low, gini_ci_high) = mean_with_ci(&ginis);
        let (br_mean, br_ci_low, br_ci_high) = mean_with_ci(&branching_ratios);
        let (docas_mean, docas_ci_low, docas_ci_high) = mean_with_ci(&docas_mags);

        BatchResult {
            run_count: results.len() as u32,
            seeds: results.iter().map(|r| r.seed).collect(),
            gini_mean,
            gini_ci: (gini_ci_low, gini_ci_high),
            gini_std: std_dev(&ginis),
            branching_ratio_mean: br_mean,
            branching_ratio_ci: (br_ci_low, br_ci_high),
            docas_magnitude_mean: docas_mean,
            docas_magnitude_ci: (docas_ci_low, docas_ci_high),
            mean_final_agents: mean(&agent_counts),
            std_final_agents: std_dev(&agent_counts),
            individual_results: results.to_vec(),
        }
    }
}

/// Result from a single simulation run
#[derive(Debug, Clone)]
pub struct RunResult {
    /// Random seed used for this run
    pub seed: u64,
    /// Index of this run within the batch
    pub run_index: usize,
    /// Final tick count when simulation ended
    pub final_tick: u64,
    /// Number of agents remaining at end of simulation
    // TAG: surface=simulation owner=research-team rule=SIM-001
    pub final_agent_count: u64,
    /// Final Gini coefficient (inequality measure)
    pub final_gini: f64,
    /// Final branching ratio (viral spread metric)
    pub final_branching_ratio: f64,
    /// Average DOCAS magnitude across the run
    pub avg_docas_magnitude: f64,
    /// Fraction of claims that passed verification
    pub verification_rate: f64,
}

/// Aggregated result from batch execution
#[derive(Debug, Clone)]
pub struct BatchResult {
    /// Number of runs in the batch
    pub run_count: u32,
    /// Seeds used for each run
    pub seeds: Vec<u64>,
    /// Mean Gini coefficient across runs
    pub gini_mean: f64,
    /// 95% confidence interval for Gini coefficient
    pub gini_ci: (f64, f64),
    /// Standard deviation of Gini coefficient
    pub gini_std: f64,
    /// Mean branching ratio across runs
    pub branching_ratio_mean: f64,
    /// 95% confidence interval for branching ratio
    pub branching_ratio_ci: (f64, f64),
    /// Mean DOCAS magnitude across runs
    pub docas_magnitude_mean: f64,
    /// 95% confidence interval for DOCAS magnitude
    pub docas_magnitude_ci: (f64, f64),
    /// Mean final agent count across runs
    pub mean_final_agents: f64,
    /// Standard deviation of final agent count
    pub std_final_agents: f64,
    /// Individual results for each run
    pub individual_results: Vec<RunResult>,
}

impl BatchResult {
    /// Print summary to stdout
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub fn print_summary(&self) {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                     BATCH EXECUTION SUMMARY                  ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
        println!("  Runs:              {}", self.run_count);
        println!(
            "  Final Agents:      {:.1} ± {:.1}",
            self.mean_final_agents, self.std_final_agents
        );
        println!(
            "  Gini:              {:.4} [{:.4}, {:.4}]",
            self.gini_mean, self.gini_ci.0, self.gini_ci.1
        );
        println!(
            "  Branching Ratio:   {:.4} [{:.4}, {:.4}]",
            self.branching_ratio_mean, self.branching_ratio_ci.0, self.branching_ratio_ci.1
        );
        println!(
            "  DOCAS Magnitude:   {:.4} [{:.4}, {:.4}]",
            self.docas_magnitude_mean, self.docas_magnitude_ci.0, self.docas_magnitude_ci.1
        );
        println!();
    }
}

/// Batch execution errors
#[derive(Debug, Clone)]
pub enum BatchError {
    /// Invalid configuration provided
    ConfigError(String),
    /// I/O operation failed
    IoError(String),
    /// Simulation execution error
    SimulationError(String),
}

impl std::fmt::Display for BatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigError(msg) => write!(f, "Config error: {msg}"),
            Self::IoError(msg) => write!(f, "IO error: {msg}"),
            // TAG: surface=simulation owner=research-team rule=SIM-001
            Self::SimulationError(msg) => write!(f, "Simulation error: {msg}"),
        }
    }
}

impl std::error::Error for BatchError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_runner_single() {
        let mut config = RunConfig::default();
        config.simulation.agent_count = 10;
        config.simulation.max_ticks = 50;
        config.output.dir = String::new(); // No output

        let runner = BatchRunner::new(config);
        // ERR-001: Test code - unwrap() exempt for test assertions
        let result = runner.run(1).unwrap();

        assert_eq!(result.run_count, 1);
        assert!(result.mean_final_agents > 0.0);
    }

    #[test]
    fn test_batch_runner_multiple() {
        let mut config = RunConfig::default();
        config.simulation.agent_count = 10;
        config.simulation.max_ticks = 50;
        config.output.dir = String::new();

        let runner = BatchRunner::new(config);
        // ERR-001: Test code - unwrap() exempt for test assertions
        let result = runner.run(3).unwrap();

        assert_eq!(result.run_count, 3);
        assert_eq!(result.seeds.len(), 3);
        assert_eq!(result.individual_results.len(), 3);
    }
}
