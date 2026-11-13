//! Experiment Runner Framework
//!
//! Per §7.6.3 Blocker 3: Required for ALL 26 experiments to produce compliant output

use sim_core::Simulation;

mod export;
mod extractors;
pub(crate) mod stats;
pub mod types;

pub use types::*;

/// Runner responsible for orchestrating experiment execution
pub struct ExperimentRunner;

impl ExperimentRunner {
    /// Returns an error if the operation fails.
    pub fn run(config: &ExperimentConfig) -> Result<ExperimentResult, ExperimentError> {
        let start_time = std::time::Instant::now();

        println!("Running experiment: {}", config.name);
        println!("  Type: {:?}", config.experiment_type);
        // TAG: surface=simulation owner=research-team rule=SIM-001
        println!(
            "  Agents: {}, Ticks: {}, Replications: {}",
            config.agent_count, config.tick_count, config.replications
        );

        let replications = Self::run_all_replications(config)?;
        let summary = Self::compute_summary(&replications);
        let validation = Self::validate(&summary, &config.acceptance_criteria);

        let mut result = ExperimentResult {
            config: config.clone(),
            replications,
            summary,
            validation: validation.clone(),
            passed: validation.passed,
            csv_path: None,
            manifest_path: None,
            png_paths: Vec::new(),
        };

        Self::generate_all_outputs(config, &mut result)?;

        let elapsed = start_time.elapsed();
        println!("Experiment completed in {elapsed:.2?}");
        println!(
            "Result: {}",
            if result.passed {
                "✅ PASSED"
            } else {
                "❌ FAILED"
            }
        );

        Ok(result)
    }

    #[allow(clippy::unnecessary_wraps)]
    fn run_all_replications(
        config: &ExperimentConfig,
    ) -> Result<Vec<ReplicationResult>, ExperimentError> {
        let mut replications = Vec::with_capacity(config.replications);
        // TAG: surface=simulation owner=research-team rule=SIM-001
        for rep_id in 0..config.replications {
            let seed = config.seed_base + rep_id as u64;
            let rep_result = Self::run_replication(config, rep_id, seed);
            replications.push(rep_result);
            if (rep_id + 1) % 100 == 0 || rep_id == config.replications - 1 {
                println!(
                    "  Completed {}/{} replications",
                    rep_id + 1,
                    config.replications
                );
            }
        }
        Ok(replications)
    }

    fn generate_all_outputs(
        config: &ExperimentConfig,
        result: &mut ExperimentResult,
    ) -> Result<(), ExperimentError> {
        if config.output_csv {
            let csv_path = config.output_dir.join(format!("{}.csv", config.name));
            Self::export_csv(result, &csv_path)?;
            result.csv_path = Some(csv_path);
        }
        if config.output_json {
            let manifest_path = config
                .output_dir
                .join(format!("{}_manifest.json", config.name));
            Self::export_manifest(result, &manifest_path)?;
            result.manifest_path = Some(manifest_path);
        }
        if config.output_png {
            result.png_paths = Self::generate_png_charts(result, &config.output_dir)?;
        }
        Ok(())
    }

    /// Run a single replication
    fn run_replication(
        config: &ExperimentConfig,
        rep_id: usize,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        seed: u64,
    ) -> ReplicationResult {
        let start_time = std::time::Instant::now();

        // Create and run simulation
        let mut sim_config = config.sim_config.clone();
        sim_config.seed = seed;
        sim_config.max_ticks = config.tick_count;
        sim_config.toggles = config.toggles;
        let mut sim = Simulation::new(sim_config);

        // Spawn agents distributed across the environment grid to avoid O(N^2) singularity
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};
        let mut pos_rng = StdRng::seed_from_u64(seed.wrapping_add(100));
        let max_x = (sim.config.environment.width as f32 - 4.0).max(10.0);
        let max_y = (sim.config.environment.height as f32 - 4.0).max(10.0);
        for _ in 0..config.agent_count {
            let pos = sim_core::types::Position {
                x: pos_rng.gen_range(2.0..max_x),
                y: pos_rng.gen_range(2.0..max_y),
                z: 0.0,
            };
            sim.spawn_agent(pos);
        }

        // Run simulation
        sim.run(config.tick_count);

        // Extract metrics based on experiment type
        let metrics = Self::extract_metrics(&sim, &config.experiment_type);

        let duration_ms = start_time.elapsed().as_millis() as u64;

        ReplicationResult {
            rep_id,
            seed,
            metrics,
            duration_ms,
            success: true,
            error: None,
        }
    }
}

/// Convenience function to execute an experiment by type
pub fn run_experiment(exp_type: ExperimentType) -> Result<ExperimentResult, ExperimentError> {
    let config = ExperimentConfig::for_experiment(exp_type);
    ExperimentRunner::run(&config)
}
