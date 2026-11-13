//! Basic Simulation Runner Example
//!
//! This example demonstrates how to run the sim-core simulation locally on macOS.
//! It creates a simulation with configurable stages and outputs metrics to stdout.
//! Optionally outputs artifacts (manifest.json, metrics.jsonl, replay/*.jsonl) to a directory.
// TAG: surface=simulation owner=research-team rule=SIM-001
//!
//! # Usage
//! ```bash
//! cd crates/sim-core
//! cargo run --example basic_run
//! cargo run --example basic_run -- --stage 3 --agents 100 --ticks 500
//! cargo run --example basic_run -- --stage 5 --output ./run_output --agents 50 --ticks 200
//! ```
//!
//! # SIM-001: Isolation verified - no product dependencies (sim-artifacts is allowed)
//! # SIM-003: Terminology - SIMULATION agents, not `FreshCredit` AI agents
//! # SIM-004: Deterministic - same seed produces identical output
//! # SIM-005: Schema versioned artifacts

// Numeric cast precision is acceptable in this example simulation runner.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::float_cmp
)]

use sim_artifacts::{Manifest, MetricsWriter, ReplayFrame, ReplayWriter, TickMetrics};
use sim_core::{
    metrics::{branching_ratio, docas_magnitude},
    simulation::{Simulation, SimulationConfig},
    toggles::ToggleConfig,
    Position,
};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config = parse_args(&args);
    let output_dir = get_output_dir_from_args(&args);

    print_banner();
    print_config(&config, output_dir.as_ref());

    let mut sim = Simulation::new(config.clone());
    let agent_count = get_agent_count_from_args(&args);
    println!("Spawning {agent_count} agents...");
    spawn_agents(&mut sim, agent_count);
    println!();

    let (mut manifest, mut metrics_writer, replay_dir) =
        initialize_artifacts(output_dir.as_ref(), &config, agent_count);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let (events_caused, events_received) =
        run_simulation_loop(&mut sim, &config, &mut metrics_writer, replay_dir.as_ref());

    finalize_artifacts(output_dir.as_ref(), &mut manifest, metrics_writer);
    print_summary(&sim, events_caused, events_received);
}

fn print_banner() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           sim-core: Agent Society Simulator                  ║");
    println!("║           Local Testing Runner (Phase A MVP)                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
}

fn print_config(config: &SimulationConfig, output_dir: Option<&PathBuf>) {
    println!("Configuration:");
    println!("  Seed:       {}", config.seed);
    println!("  Stage:      {}", get_stage_name(&config.toggles));
    println!("  Max Ticks:  {}", config.max_ticks);
    println!(
        "  Grid Size:  {}x{}",
        config.environment.width, config.environment.height
    );
    if let Some(dir) = output_dir {
        println!("  Output:     {}", dir.display());
    }
    println!();
}

fn initialize_artifacts(
    output_dir: Option<&PathBuf>,
    config: &SimulationConfig,
    agent_count: usize,
) -> (Option<Manifest>, Option<MetricsWriter>, Option<PathBuf>) {
    let Some(dir) = output_dir else {
        return (None, None, None);
    };

    // TAG: surface=simulation owner=research-team rule=SIM-001
    std::fs::create_dir_all(dir).expect("Failed to create output directory");
    let replay_path = dir.join("replay");
    std::fs::create_dir_all(&replay_path).expect("Failed to create replay directory");

    let git_commit = std::env::var("GIT_COMMIT").unwrap_or_else(|_| "unknown".to_string());
    let mut manifest = Manifest::new(config.seed, git_commit);
    fill_manifest_config(&mut manifest, config, agent_count);

    let metrics_path = dir.join("metrics.jsonl");
    let metrics_writer =
        MetricsWriter::new(&metrics_path).expect("Failed to create metrics writer");

    println!("Artifact output enabled:");
    println!("  manifest.json:  {}", dir.join("manifest.json").display());
    println!("  metrics.jsonl:  {}", metrics_path.display());
    println!("  replay/:        {}", replay_path.display());
    println!();

    (Some(manifest), Some(metrics_writer), Some(replay_path))
}

fn fill_manifest_config(manifest: &mut Manifest, config: &SimulationConfig, agent_count: usize) {
    manifest.config.agent_count = agent_count as u32;
    manifest.config.max_ticks = config.max_ticks;
    manifest.config.grid_width = config.environment.width as u32;
    manifest.config.grid_height = config.environment.height as u32;
    manifest.config.stage = get_stage_name(&config.toggles).to_string();
    manifest.config.toggles.genome_enabled = config.toggles.genome_enabled;
    manifest.config.toggles.docas_enabled = config.toggles.docas_enabled;
    manifest.config.toggles.dmn_enabled = config.toggles.dmn_enabled;
    manifest.config.toggles.criticality_enabled = config.toggles.criticality_enabled;
    manifest.config.toggles.choice_architecture_enabled =
        config.toggles.choice_architecture_enabled;
    manifest.config.toggles.institutions_enabled = config.toggles.institutions_enabled;
    manifest.config.toggles.norms_enabled = config.toggles.norms_enabled;
    manifest.config.toggles.language_enabled = config.toggles.language_enabled;
    manifest.config.toggles.protocol_enabled = config.toggles.protocol_enabled;
    manifest.metrics_file = "metrics.jsonl".to_string();
}

// TAG: surface=simulation owner=research-team rule=SIM-001
fn run_simulation_loop(
    sim: &mut Simulation,
    config: &SimulationConfig,
    metrics_writer: &mut Option<MetricsWriter>,
    replay_dir: Option<&PathBuf>,
) -> (u64, u64) {
    println!("Running simulation...");
    println!("─────────────────────────────────────────────────────────────────");
    println!(
        "{:>6} │ {:>6} │ {:>8} │ {:>8} │ {:>10}",
        "Tick", "Alive", "Avg E", "Avg DOCAS", "Branching"
    );
    println!("─────────────────────────────────────────────────────────────────");

    let report_interval = (config.max_ticks / 10).max(1);
    let replay_interval = (config.max_ticks / 20).max(1);
    let mut events_caused = 0u64;
    let mut events_received = 0u64;

    while sim.tick < config.max_ticks {
        let alive_before = sim.agents.iter().filter(|a| a.alive).count();
        sim.step();
        let alive_after = sim.agents.iter().filter(|a| a.alive).count();

        if alive_after > alive_before {
            events_caused += (alive_after - alive_before) as u64;
        }
        events_received += alive_after as u64;

        let (alive, avg_energy, avg_docas, sigma) =
            compute_tick_metrics(sim, config, events_caused, events_received);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        if let Some(ref mut writer) = metrics_writer {
            write_tick_metrics(writer, sim, config, alive, avg_energy, avg_docas, sigma);
        }

        if let Some(replay_path) = replay_dir {
            if sim.tick % replay_interval == 0 {
                write_replay_frame(replay_path, sim);
            }
        }

        if sim.tick % report_interval == 0 || sim.tick == config.max_ticks {
            println!(
                "{:>6} │ {:>6} │ {:>8.2} │ {:>8.3} │ {:>10.4}",
                sim.tick, alive, avg_energy, avg_docas, sigma
            );
        }

        if sim.agents.iter().all(|a| !a.alive) {
            println!("\n⚠ All agents died at tick {}", sim.tick);
            break;
        }
    }

    println!("─────────────────────────────────────────────────────────────────");
    println!();

    (events_caused, events_received)
}

fn compute_tick_metrics(
    sim: &Simulation,
    config: &SimulationConfig,
    events_caused: u64,
    events_received: u64,
    // TAG: surface=simulation owner=research-team rule=SIM-001
) -> (usize, f32, f32, f64) {
    let alive = sim.agents.iter().filter(|a| a.alive).count();
    let avg_energy = if alive > 0 {
        sim.agents
            .iter()
            .filter(|a| a.alive)
            .map(|a| a.energy)
            .sum::<f32>()
            / alive as f32
    } else {
        0.0
    };
    let avg_docas = if alive > 0 && config.toggles.docas_enabled {
        sim.agents
            .iter()
            .filter(|a| a.alive)
            .map(|a| docas_magnitude(&a.docas))
            .sum::<f32>()
            / alive as f32
    } else {
        0.0
    };
    let sigma = branching_ratio(events_caused, events_received);
    (alive, avg_energy, avg_docas, sigma)
}

fn write_tick_metrics(
    writer: &mut MetricsWriter,
    sim: &Simulation,
    config: &SimulationConfig,
    alive: usize,
    avg_energy: f32,
    avg_docas: f32,
    sigma: f64,
) {
    let tick_metrics = TickMetrics {
        tick: sim.tick,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        agent_count: alive as u64,
        cooperation_rate: 0.0,
        gini: 0.0,
        trade_volume: 0,
        dispute_rate: 0.0,
        verification_rate: if config.toggles.protocol_enabled {
            f64::from(sim.protocol.verification_rate())
        } else {
            0.0
        },
        avg_energy: f64::from(avg_energy),
        avg_docas_magnitude: f64::from(avg_docas),
        branching_ratio: sigma,
        active_norms: sim.active_norms.len() as u64,
        institution_count: sim.institutions.institutions.len() as u64,
        total_anchors: sim.protocol.total_anchors,
        total_verifications: sim.protocol.total_verifications,
    };
    writer
        .write(&tick_metrics)
        .expect("Failed to write metrics");
}

fn write_replay_frame(replay_path: &PathBuf, sim: &Simulation) {
    let mut replay_writer =
        ReplayWriter::new(replay_path, sim.tick).expect("Failed to create replay writer");
    for (idx, agent) in sim.agents.iter().enumerate() {
        let frame = ReplayFrame {
            tick: sim.tick,
            agent_id: idx as u64,
            x: agent.position.x,
            y: agent.position.y,
            energy: agent.energy,
            docas: agent.docas.to_array(),
            alive: agent.alive,
            has_verified_id: agent.has_verified_id,
            group_id: agent.group_id,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            age: agent.age,
        };
        replay_writer
            .write(&frame)
            .expect("Failed to write replay frame");
    }
    replay_writer
        .finish()
        .expect("Failed to finish replay writer");
}

fn finalize_artifacts(
    output_dir: Option<&PathBuf>,
    manifest: &mut Option<Manifest>,
    metrics_writer: Option<MetricsWriter>,
) {
    let Some(dir) = output_dir else { return };

    if let Some(writer) = metrics_writer {
        writer.finish().expect("Failed to finish metrics writer");
    }

    if let Some(ref mut m) = manifest {
        m.complete();
        let manifest_path = dir.join("manifest.json");
        let manifest_json = m.to_json().expect("Failed to serialize manifest");
        std::fs::write(&manifest_path, manifest_json).expect("Failed to write manifest");
        println!("✓ Artifacts written to {}", dir.display());
    }
}

fn parse_args(args: &[String]) -> SimulationConfig {
    let mut config = SimulationConfig::default();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            "--seed" | "-s" => {
                if i + 1 < args.len() {
                    config.seed = args[i + 1].parse().unwrap_or(42);
                    i += 1;
                }
            }
            "--stage" => {
                if i + 1 < args.len() {
                    config.toggles = match args[i + 1].as_str() {
                        "1" => ToggleConfig::stage_1(),
                        "2" => ToggleConfig::stage_2(),
                        "3" => ToggleConfig::stage_3(),
                        "4" => ToggleConfig::stage_4(),
                        "5" => ToggleConfig::stage_5(),
                        _ => ToggleConfig::stage_0(),
                    };
                    i += 1;
                }
            }
            "--ticks" | "-t" => {
                if i + 1 < args.len() {
                    config.max_ticks = args[i + 1].parse().unwrap_or(1000);
                    i += 1;
                }
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }

    config
}

// TAG: surface=simulation owner=research-team rule=SIM-001
fn get_agent_count_from_args(args: &[String]) -> usize {
    let mut i = 1;
    while i < args.len() {
        if (args[i] == "--agents" || args[i] == "-a") && i + 1 < args.len() {
            return args[i + 1].parse().unwrap_or(10);
        }
        i += 1;
    }
    10 // Default
}

fn get_output_dir_from_args(args: &[String]) -> Option<PathBuf> {
    let mut i = 1;
    while i < args.len() {
        if (args[i] == "--output" || args[i] == "-o") && i + 1 < args.len() {
            return Some(PathBuf::from(&args[i + 1]));
        }
        i += 1;
    }
    None
}

fn spawn_agents(sim: &mut Simulation, count: usize) {
    let grid_size = sim.config.environment.width as f32;
    for i in 0..count {
        // Distribute agents across the grid
        let x = (i as f32 * 7.0) % grid_size;
        let y = (i as f32 * 11.0) % grid_size;
        sim.spawn_agent(Position::new_2d(x, y));
    }
}

const fn get_stage_name(toggles: &ToggleConfig) -> &'static str {
    if toggles.protocol_enabled {
        "Stage 5 (Protocol)"
    } else if toggles.language_enabled {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        "Stage 4 (Language)"
    } else if toggles.norms_enabled || toggles.institutions_enabled {
        "Stage 3 (Society)"
    } else if toggles.dmn_enabled || toggles.docas_enabled {
        "Stage 2 (Cognition)"
    } else if toggles.genome_enabled {
        "Stage 1 (Biology)"
    } else {
        "Stage 0 (Geography)"
    }
}

fn print_summary(sim: &Simulation, events_caused: u64, events_received: u64) {
    let alive = sim.agents.iter().filter(|a| a.alive).count();
    let total = sim.agents.len();
    let sigma = branching_ratio(events_caused, events_received);

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                     SIMULATION SUMMARY                       ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    let tick = sim.tick;
    let alive_pct = (alive as f32 / total as f32) * 100.0;
    println!();
    println!("  Final Tick:        {tick}");
    println!("  Agents Alive:      {alive} / {total} ({alive_pct:.1}%)");
    println!("  Branching Ratio:   {sigma:.4} (target: ~1.0 for criticality)");

    if sim.config.toggles.docas_enabled && alive > 0 {
        let avg_docas: f32 = sim
            .agents
            .iter()
            .filter(|a| a.alive)
            .map(|a| docas_magnitude(&a.docas))
            .sum::<f32>()
            / alive as f32;
        println!("  Avg DOCAS Mag:     {avg_docas:.3}");
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    if sim.config.toggles.norms_enabled {
        let norms_count = sim.active_norms.len();
        println!("  Active Norms:      {norms_count}");
    }

    if sim.config.toggles.institutions_enabled {
        let inst_count = sim.institutions.institutions.len();
        println!("  Institutions:      {inst_count}");
    }

    if sim.config.toggles.protocol_enabled {
        let verified_count = sim.agents.iter().filter(|a| a.has_verified_id).count();
        let verified_pct = if total > 0 {
            (verified_count as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        let total_anchors = sim.protocol.total_anchors;
        let total_verifications = sim.protocol.total_verifications;
        let verification_rate = sim.protocol.verification_rate();
        println!("  Verified Agents:   {verified_count} / {total} ({verified_pct:.1}%)");
        println!("  Total Anchors:     {total_anchors}");
        println!("  Verifications:     {total_verifications} (rate: {verification_rate:.2})");
        if sim.protocol.is_stable() {
            println!("  Protocol Status:   ✓ STABLE (rate > 0.1)");
        } else if sim.protocol.total_verifications > 0 {
            println!("  Protocol Status:   ⚠ UNSTABLE (rate < 0.1)");
        }
    }

    println!();
    println!(
        "✓ Simulation complete. SIM-004: Deterministic (seed={})",
        sim.config.seed
    );
}

// TAG: surface=simulation owner=research-team rule=SIM-001
fn print_help() {
    println!("sim-core: Agent Society Simulator - Local Testing Runner");
    println!();
    println!("USAGE:");
    println!("    cargo run --example basic_run [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -s, --seed <N>      Random seed for deterministic execution (default: 42)");
    println!("    -a, --agents <N>    Number of agents to spawn (default: 10)");
    println!("    -t, --ticks <N>     Maximum ticks to run (default: 1000)");
    println!("    --stage <0-5>       Stage preset (0=Geography, ..., 5=Protocol)");
    println!("    -o, --output <DIR>  Output directory for artifacts (optional)");
    println!("    -h, --help          Print this help message");
    println!();
    println!("EXAMPLES:");
    println!("    cargo run --example basic_run");
    println!("    cargo run --example basic_run -- --stage 5 --agents 100 --ticks 500");
    println!("    cargo run --example basic_run -- -s 12345 -a 50 -t 2000");
    println!("    cargo run --example basic_run -- --output ./run_output --stage 5 -a 50");
    println!();
    println!("ARTIFACTS (when --output is specified):");
    println!("    manifest.json  - Run metadata, config, seeds, status");
    println!("    metrics.jsonl  - Per-tick aggregate metrics");
    println!("    replay/*.jsonl - Per-agent state for visualization replay");
    println!();
    println!("STAGES:");
    println!("    0 - Geography: Grid world, resources, movement");
    println!("    1 - Biology:   + Genome, reproduction, Gompertz mortality");
    println!("    2 - Cognition: + DOCAS neuromodulators, DMN planning");
    println!("    3 - Society:   + Norms, institutions, Beinhocker dynamics");
    println!("    4 - Language:  + Lexicons, colexification, semantic drift");
    println!("    5 - Protocol:  + Blockchain verification simulation");
}
