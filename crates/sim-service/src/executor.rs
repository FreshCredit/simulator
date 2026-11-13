//! Job execution module
//!
//! Spawns sim-runner subprocess to execute simulation jobs.
//!
//! SIM-001: Does not import sim-runner directly. Uses subprocess spawning.

use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tracing::{debug, error, info};

use crate::blockchain::BlockchainAnchor;
use crate::jobs::{Job, JobStatus, JobStore};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Configuration for job executor
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Path to sim-runner binary
    pub sim_runner_path: PathBuf,
    /// Base directory for artifact output
    pub artifacts_dir: PathBuf,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            sim_runner_path: PathBuf::from("sim-runner"),
            artifacts_dir: PathBuf::from("./artifacts"),
        }
    }
}

#[allow(clippy::too_many_lines)]
/// Execute a job by spawning sim-runner subprocess
pub async fn execute_job(job: Job, store: Arc<JobStore>, config: ExecutorConfig) {
    let job_id = job.id.clone();
    info!("Starting execution of job {}", job_id);

    // Update status to Running
    store.update_status(&job_id, JobStatus::Running);

    // Build output directory path
    let output_dir = config.artifacts_dir.join(&job_id);

    // Build command arguments
    let mut cmd = Command::new(&config.sim_runner_path);
    cmd.arg("run")
        .arg("--agents")
        .arg(job.request.agent_count.to_string())
        .arg("--ticks")
        .arg(job.request.max_ticks.to_string())
        .arg("--stage")
        .arg(job.request.stage.to_string())
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        .arg("--seed")
        .arg(job.request.seed.to_string())
        .arg("--output")
        .arg(&output_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    info!(
        "Executing: {} run --agents {} --ticks {} --stage {} --seed {} --output {:?}",
        config.sim_runner_path.display(),
        job.request.agent_count,
        job.request.max_ticks,
        job.request.stage,
        job.request.seed,
        output_dir
    );

    // Spawn the process
    match cmd.spawn() {
        Ok(child) => {
            // Wait for completion
            match child.wait_with_output().await {
                Ok(output) => {
                    if output.status.success() {
                        info!("Job {} completed successfully", job_id);

                        // === BLOCKCHAIN ANCHORING ===
                        // Anchor simulation results to testnet
                        debug!("Initiating blockchain anchoring for job {}", job_id);

                        // TAG: surface=simulation owner=research-team rule=SIM-001
                        let blockchain = BlockchainAnchor::from_env();
                        if blockchain.is_enabled() {
                            // Read the metrics file to get hash
                            let metrics_path = output_dir.join("metrics.json");
                            match blockchain.hash_file(&metrics_path).await {
                                Ok(metrics_hash) => {
                                    match blockchain
                                        .anchor_simulation(
                                            &job_id,
                                            &job.request.stage.to_string(),
                                            job.request.agent_count,
                                            &metrics_hash,
                                        )
                                        .await
                                    {
                                        Ok(receipt) => {
                                            info!(
                                                "Job {} anchored to testnet block {} (extrinsic: {:?})",
                                                job_id,
                                                receipt.block_number.unwrap_or(0),
                                                receipt.extrinsic_hash
                                            );

                                            // Save receipt alongside metrics
                                            // P0-PERF: Wrap blocking I/O in spawn_blocking
                                            let receipt_path =
                                                output_dir.join("anchor_receipt.json");
                                            let receipt_json =
                                                serde_json::to_string_pretty(&receipt)
                                                    .unwrap_or_default();
                                            let path_clone = receipt_path.clone();
                                            if let Err(e) = tokio::task::spawn_blocking(move || {
                                                std::fs::write(&path_clone, receipt_json)
                                                // TAG: surface=simulation owner=research-team rule=SIM-001
                                            })
                                            .await
                                            {
                                                error!("Failed to save anchor receipt: {}", e);
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to anchor job {}: {}", job_id, e);
                                            // Don't fail the job - anchoring is best-effort
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to hash metrics for job {}: {}", job_id, e);
                                }
                            }
                        } else {
                            debug!("Blockchain anchoring disabled for job {}", job_id);
                        }

                        // The run_id is the job_id (we use job_id as output directory name)
                        store.complete(&job_id, job_id.clone());
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        error!("Job {} failed: {}", job_id, stderr);
                        store.fail(
                            &job_id,
                            format!("Exit code: {:?}, stderr: {}", output.status.code(), stderr),
                        );
                    }
                }
                Err(e) => {
                    error!("Job {} failed to wait: {}", job_id, e);
                    store.fail(&job_id, format!("Process wait error: {e}"));
                }
            }
        }
        Err(e) => {
            error!("Job {} failed to spawn: {}", job_id, e);
            store.fail(&job_id, format!("Failed to spawn sim-runner: {e}"));
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Spawn job execution in background
pub fn spawn_job_execution(job: Job, store: Arc<JobStore>, config: ExecutorConfig) {
    tokio::spawn(async move {
        execute_job(job, store, config).await;
    });
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Execute an experiment by spawning sim-runner subprocess
#[allow(clippy::too_many_arguments)]
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn execute_experiment(
    experiment_id: String,
    run_id: String,
    agents: usize,
    ticks: u64,
    replications: u32,
    seed: u64,
    _pde4b_variance: String,
    config: ExecutorConfig,
) -> Result<(), String> {
    info!(
        "Starting execution of experiment {} (run: {})",
        experiment_id, run_id
    );

    // Build output directory path
    let output_dir = config.artifacts_dir.join(&run_id);

    // Build command arguments for experiment
    let mut cmd = Command::new(&config.sim_runner_path);
    cmd.arg("experiment")
        .arg(&experiment_id)
        .arg("--agents")
        .arg(agents.to_string())
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        .arg("--ticks")
        .arg(ticks.to_string())
        .arg("--replications")
        .arg(replications.to_string())
        .arg("--seed")
        .arg(seed.to_string())
        .arg("--output")
        .arg(&output_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    info!(
        "Executing: {} experiment {} --agents {} --ticks {} --replications {} --seed {} --output {:?}",
        config.sim_runner_path.display(),
        experiment_id,
        agents,
        ticks,
        replications,
        seed,
        output_dir
    );

    // Spawn the process
    match cmd.spawn() {
        Ok(child) => {
            // Wait for completion
            match child.wait_with_output().await {
                Ok(output) => {
                    if output.status.success() {
                        info!("Experiment {} completed successfully", run_id);
                        Ok(())
                    // TAG: surface=simulation owner=research-team rule=SIM-001
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        error!("Experiment {} failed: {}", run_id, stderr);
                        Err(format!(
                            "Exit code: {:?}, stderr: {}",
                            output.status.code(),
                            stderr
                        ))
                    }
                }
                Err(e) => {
                    error!("Experiment {} failed to wait: {}", run_id, e);
                    Err(format!("Process wait error: {e}"))
                }
            }
        }
        Err(e) => {
            error!("Experiment {} failed to spawn: {}", run_id, e);
            Err(format!("Failed to spawn sim-runner: {e}"))
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Spawn experiment execution in background
#[allow(clippy::too_many_arguments)]
pub fn spawn_experiment_execution(
    experiment_id: String,
    run_id: String,
    agents: usize,
    ticks: u64,
    replications: u32,
    seed: u64,
    pde4b_variance: String,
    config: ExecutorConfig,
) {
    tokio::spawn(async move {
        let _ = execute_experiment(
            experiment_id,
            run_id,
            agents,
            ticks,
            replications,
            seed,
            pde4b_variance,
            config,
        )
        .await;
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_config_default() {
        let config = ExecutorConfig::default();
        assert_eq!(config.sim_runner_path, PathBuf::from("sim-runner"));
        assert_eq!(config.artifacts_dir, PathBuf::from("./artifacts"));
    }

    #[test]
    fn test_job_output_path() {
        let config = ExecutorConfig {
            sim_runner_path: PathBuf::from("/usr/bin/sim-runner"),
            artifacts_dir: PathBuf::from("/data/artifacts"),
        };
        let job_id = "test-job-123";
        let output_dir = config.artifacts_dir.join(job_id);
        assert_eq!(output_dir, PathBuf::from("/data/artifacts/test-job-123"));
    }
}
