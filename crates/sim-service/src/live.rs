//! Live simulation streaming module
//!
//! SIM-001: Uses sim-core directly (allowed for sim-* crates)
//! SIM-003: Simulation agents, NOT `FreshCredit` AI agents
//! SIM-004: Deterministic kernel with seeded RNG
//!
//! Runs a continuous 24/7 simulation and streams agent states via SSE.

// TAG: surface=simulation owner=research-team rule=SIM-001
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sim_core::environment::EnvironmentConfig;
use sim_core::{simulation::SimulationConfig, Position, Simulation};
use std::{
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Duration,
};
use tracing::{debug, info, trace, warn};

/// Configurable tick delay for simulation speed control
/// Phase 1: Allows UI to control simulation speed without affecting logic
/// Default: 100ms = 10 FPS, Range: 50-2000ms (0.5x to 20x speed factor)
static TICK_DELAY_MS: AtomicU64 = AtomicU64::new(100);

/// Pause state - when true, simulation stops ticking
/// Initialized to true so the simulation stays in standby until the user clicks Start/Configure
static PAUSED: AtomicBool = AtomicBool::new(true);

/// Track if user has explicitly launched a session
static HAS_STARTED: AtomicBool = AtomicBool::new(false);

/// Stop state - when true, simulation loop exits
static STOPPED: AtomicBool = AtomicBool::new(false);

/// Maximum background execution ticks when no clients are connected before auto-pausing.
/// 3,000 ticks = 5 minutes of continuous background execution at 100ms/tick.
const MAX_BACKGROUND_TICKS: u32 = 3000;

pub mod params;
pub use params::*;

/// Shared simulation parameters - can be updated to restart with new config
static SIMULATION_PARAMS: RwLock<SimulationParams> = RwLock::new(SimulationParams {
    resource_density: 0.30,
    initial_resource: 50.0,
    agent_count: 20,
    grid_width: 100,
    grid_height: 100,
    toggles: None,
    preset: None,
    max_ticks: None,
    docas_alpha_base: None,
    trust_curvature_window: None,
    planning_horizon: None,
    emotion_decay_rate: None,
    worldview_drift_rate: None,
    trust_decay_rate: None,
    reputation_threshold: None,
    credit_limit: None,
    interest_rate_base: None,
    transaction_cost: None,
    block_time: None,
    consensus_threshold: None,
    movement_speed: None,
    social_force_influence: None,
    opto_wavelength: None,
    opto_target_channels: None,
    opto_scope: None,
    opto_subpop_pct: None,
    visual_cloaking_ratio: None,
    brain_it_branch: None,
    weather_shield_enabled: None,
    weather_temperature: None,
    melanopsin_enabled: None,
    baylor_unconscious_mode: None,
    kingman_coalescent: None,
    lineage_depth: None,
    music_metric_hierarchy_tau: None,
    altitude_effects_enabled: None,
    weather_enabled: None,
    energy_consumption: None,
    movement_cost: None,
    aging_beta: None,
    repro_threshold: None,
    mutation_rate: None,
    language_drift_rate_factor: None,
    language_coherence_threshold: None,
    epigenetic_methylation_rate: None,
    circadian_sleep_deficit: None,
    tax_redistribution_rate: None,
    bank_run_sensitivity: None,
});

/// Shared active simulation configuration for state export
static LIVE_SIM_CONFIG: RwLock<Option<SimulationConfig>> = RwLock::new(None);

/// Get active live simulation configuration for state export
pub fn get_live_simulation_config() -> SimulationConfig {
    LIVE_SIM_CONFIG.read().clone().unwrap_or_default()
}
/// Pending imported simulation checkpoint for stateful hydration
static PENDING_STATE_IMPORT: RwLock<Option<endpoints::SimulationStateExport>> = RwLock::new(None);

/// Set pending state import to be hydrated into the next simulation instance
pub fn set_pending_state_import(export: endpoints::SimulationStateExport) {
    let mut pending = PENDING_STATE_IMPORT.write();
    *pending = Some(export);
}

/// Flag to trigger a restart with new parameters
static RESTART_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Update simulation parameters and request a restart
pub fn restart_simulation_with_params(params: SimulationParams) {
    let mut sim_params = SIMULATION_PARAMS.write();
    *sim_params = params;
    HAS_STARTED.store(true, Ordering::Release);
    PAUSED.store(false, Ordering::Release);
    RESTART_REQUESTED.store(true, Ordering::Release);
    stop_simulation();
    info!("[SIM-CONTROL] Restart requested with new parameters");
}
/// Monotonic intervention sequence counter to trigger event-driven standby frame broadcasts
static INTERVENTION_SEQ: AtomicU64 = AtomicU64::new(1);

/// Update simulation intervention parameters in real-time without stopping the simulation
pub fn apply_live_intervention(params: SimulationParams) {
    let mut cfg_lock = LIVE_SIM_CONFIG.write();
    let cfg = cfg_lock.get_or_insert_with(SimulationConfig::default);
    apply_config_overrides(cfg, &params);

    let mut sim_params = SIMULATION_PARAMS.write();
    update_simulation_params(&mut sim_params, &params);

    INTERVENTION_SEQ.fetch_add(1, Ordering::SeqCst);
    info!("[SIM-CONTROL] Live intervention parameters applied in real-time");
}

/// Check if a restart was requested
pub fn is_restart_requested() -> bool {
    RESTART_REQUESTED.load(Ordering::Acquire)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Clear the restart request flag
pub fn clear_restart_request() {
    RESTART_REQUESTED.store(false, Ordering::Release);
}

/// Set the simulation tick delay
/// `delay_ms`: milliseconds between ticks (50-2000)
pub fn set_tick_delay(delay_ms: u64) {
    let clamped = delay_ms.clamp(20, 2000);
    TICK_DELAY_MS.store(clamped, Ordering::Release);
    info!("Simulation tick delay set to {}ms", clamped);
}

/// Get current simulation tick delay
pub fn get_tick_delay() -> u64 {
    TICK_DELAY_MS.load(Ordering::Acquire)
}

/// Pause the simulation
pub fn pause_simulation() {
    PAUSED.store(true, Ordering::Release);
    info!("[SIM-CONTROL] Simulation paused");
}

/// Resume the simulation
pub fn resume_simulation() {
    PAUSED.store(false, Ordering::Release);
    info!("[SIM-CONTROL] Simulation resumed");
}
// TAG: surface=simulation owner=platform-team rule=GENERAL-001

/// Check if simulation is paused
pub fn is_paused() -> bool {
    PAUSED.load(Ordering::Acquire)
}

/// Stop the simulation (terminates the loop)
pub fn stop_simulation() {
    STOPPED.store(true, Ordering::Release);
    info!("[SIM-CONTROL] Simulation stop requested");
}

/// Reset stop flag (for starting a new simulation)
pub fn reset_stop_flag() {
    STOPPED.store(false, Ordering::Release);
}

/// Check if simulation is stopped
pub fn is_stopped() -> bool {
    STOPPED.load(Ordering::Acquire)
}

#[path = "live/types.rs"]
pub mod types;
pub use types::*;

#[path = "live/snapshots.rs"]
pub mod snapshots;
pub use snapshots::*;

#[path = "live/endpoints.rs"]
pub mod endpoints;
pub use endpoints::*;

#[path = "live/ai.rs"]
pub mod ai;
pub use ai::*;

/// RESET FIX: Automatically restarts when last client disconnects
fn process_tick(
    state: &LiveSimulationState,
    sim: &mut Simulation,
    tick_count: &mut u64,
    last_log: &mut std::time::Instant,
    last_positions: &mut std::collections::HashMap<u64, (f32, f32)>,
    empty_client_count: &mut u32,
    stalled_tick_count: &mut u64,
) -> bool {
    let live_params = SIMULATION_PARAMS.read().clone();
    sync_live_params(&live_params, sim);

    sim.step();
    *tick_count += 1;

    if sim.tick <= 5 || sim.tick % 100 == 0 {
        info!(
            "[SimLoop] [Tick] tick={} alive={} births_this_tick={} deaths_this_tick={}",
            sim.tick,
            sim.living_agent_count(),
            sim.metrics.births_this_tick,
            sim.metrics.deaths_this_tick
        );
    }

    let frame = create_frame(sim);
    *state.current_frame.write() = Some(frame.clone());

    record_historical_metrics(state, sim);

    let receiver_count = state.sender.receiver_count();
    match state.sender.send(frame.clone()) {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        Ok(sent) => {
            if receiver_count > 0 {
                trace!(
                    "[SimLoop] Broadcast frame tick={} to {} receivers",
                    frame.tick,
                    sent
                );
            }
            *stalled_tick_count = 0;
        }
        Err(_e) => {
            if *tick_count % 100 == 0 {
                debug!("[SimLoop] No receivers for frame tick={}", frame.tick);
            }
            if receiver_count > 0 {
                *stalled_tick_count += 1;
                if *stalled_tick_count > 50 {
                    warn!(
                        "[SimLoop] WARNING: Failed to broadcast for {} consecutive ticks with {} receivers",
                        stalled_tick_count, receiver_count
                    );
                }
            }
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        }
    }

    if sim.tick <= 5 || sim.tick % 50 == 0 {
        log_frame_movement(&frame, last_positions, receiver_count);
    }

    if last_log.elapsed().as_secs() >= 10 {
        let fps = *tick_count as f64 / last_log.elapsed().as_secs_f64();
        let agent_positions: Vec<(u64, f32, f32)> = frame
            .agents
            .iter()
            .take(3)
            .map(|a| (a.agent_id, a.x, a.y))
            .collect();
        info!(
            "[SimLoop] [Status] tick={}, alive_agents={}, subscribers={}, fps={:.1}",
            frame.tick, frame.stats.alive_agents, receiver_count, fps
        );
        info!("[SimLoop] [AgentPositions] {:?}", agent_positions);
        *last_log = std::time::Instant::now();
        *tick_count = 0;
    }

    if sim.living_agent_count() == 0 {
        info!("[SimLoop] ============================================");
        info!("[SimLoop] COMPLETED: all agents dead at tick {}", sim.tick);
        info!(
            "[SimLoop] Total reproductions: {}, Total deaths: {}",
            sim.debug_state.total_reproductions, sim.debug_state.total_deaths
        );
        info!("[SimLoop] ============================================");
        send_final_frame(
            state,
            sim,
            sim.config.max_ticks,
            SimulationStatus::Completed,
        );
        return true;
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let receiver_count = state.sender.receiver_count();
    if receiver_count == 0 {
        *empty_client_count += 1;
        if *empty_client_count >= MAX_BACKGROUND_TICKS && !is_paused() {
            pause_simulation();
            info!(
                "[SimLoop] Max background budget reached ({} ticks) - auto-paused in place at tick {} (preserving state)",
                MAX_BACKGROUND_TICKS, sim.tick
            );
        }
    } else {
        *empty_client_count = 0;
    }

    false
}

fn record_historical_metrics(state: &LiveSimulationState, sim: &Simulation) {
    let living_agents: Vec<_> = sim.agents.iter().filter(|a| a.alive).collect();
    let living_count = living_agents.len();
    let avg_trust = if living_count > 0 {
        living_agents.iter().map(|a| a.compute_trust()).sum::<f32>() / living_count as f32
    } else {
        0.0
    };
    let energies: Vec<f32> = living_agents.iter().map(|a| a.energy).collect();
    let gini = sim_core::stats::gini_coefficient(&energies);
    let defaults_count = sim.metrics.disputes_this_tick as u64;
    let point = HistoricalMetricPoint {
        tick: sim.tick,
        population: sim.agents.len(),
        alive_agents: living_count,
        avg_trust,
        defaults: defaults_count,
        default_rate: if sim.agents.is_empty() {
            0.0
        } else {
            (defaults_count as f32) / (sim.agents.len() as f32)
        },
        gini_index: gini,
    };
    let mut hist = state.history.write();
    hist.push(point);
    if hist.len() > 500 {
        hist.remove(0);
    }
}

/// Sanitize an optional float with finite checks and bounds clamping
pub fn sanitize_f32(val: Option<f32>, default: f32, min: f32, max: f32) -> f32 {
    match val {
        Some(v) if v.is_finite() => v.clamp(min, max),
        _ => default,
    }
}

fn sync_optogenetics_params(params: &SimulationParams, sim: &mut Simulation) {
    if let Some(ref w) = params.opto_wavelength {
        sim.config.optogenetics.wavelength = match w.as_str() {
            "ChR2Blue470nm" => sim_core::optogenetics::OpsinWavelength::ChR2Blue470nm,
            "ArchTYellow566nm" => sim_core::optogenetics::OpsinWavelength::ArchTYellow566nm,
            _ => sim_core::optogenetics::OpsinWavelength::LaserOff,
        };
    }
    if let Some(ch) = params.opto_target_channels {
        sim.config.optogenetics.target_channels = ch;
    }
    if let Some(ref scope) = params.opto_scope {
        let subpop = sanitize_f32(params.opto_subpop_pct, 0.10, 0.01, 1.0);
        sim.config.optogenetics.scope = match scope.as_str() {
            "d1" => sim_core::optogenetics::OptogeneticScope::LowestDecileD1,
            "d10" => sim_core::optogenetics::OptogeneticScope::TopDecileD10,
            "custom" => sim_core::optogenetics::OptogeneticScope::CustomSubpopulation(subpop),
            _ => sim_core::optogenetics::OptogeneticScope::EntirePopulation,
        };
    }
}

fn sync_environmental_params(params: &SimulationParams, sim: &mut Simulation) {
    if let Some(chi) = params.visual_cloaking_ratio {
        let safe_chi = sanitize_f32(Some(chi), 1.0, 1.0, 5.0);
        sim.config.visual_cloaking =
            sim_core::visual_cloaking::VisualAgreementStimulus::with_cloaking_ratio(safe_chi);
    }
    if let Some(shield) = params.weather_shield_enabled {
        sim.config.weather_shield_enabled = shield;
    }
    if let Some(temp) = params.weather_temperature {
        let safe_temp = sanitize_f32(Some(temp), 20.0, -30.0, 60.0);
        sim.environment.weather.temperature = safe_temp;
        sim.environment.weather.temperature_override = Some(safe_temp);
    }
    if let Some(melanopsin) = params.melanopsin_enabled {
        sim.environment.weather.enabled = melanopsin;
        sim.config.environment.weather_enabled = melanopsin;
    }
    if let Some(unconscious) = params.baylor_unconscious_mode {
        sim.config.baylor.mode = if unconscious {
            sim_core::baylor_gating::ConsciousnessMode::Unconscious
        } else {
            sim_core::baylor_gating::ConsciousnessMode::Conscious
        };
    }
}

fn sync_live_params(params: &SimulationParams, sim: &mut Simulation) {
    apply_config_overrides(&mut sim.config, params);
    set_if_some(
        &mut sim.environment.altitude_effects_enabled,
        params.altitude_effects_enabled,
    );
    set_if_some(&mut sim.environment.weather_enabled, params.weather_enabled);
    sync_optogenetics_params(params, sim);
    sync_environmental_params(params, sim);
}

async fn run_simulation_session(state: &LiveSimulationState, config: &SimulationConfig) {
    let mut sim = Simulation::new(config.clone());
    let params = SIMULATION_PARAMS.read().clone();
    sync_live_params(&params, &mut sim);
    *LIVE_SIM_CONFIG.write() = Some(sim.config.clone());
    let agent_count = params.agent_count;

    let pending_import = PENDING_STATE_IMPORT.write().take();
    if let Some(import) = pending_import {
        info!(
            "[SimLoop] Hydrating simulation from imported checkpoint at tick {} ({} agents)",
            import.tick,
            import.agents.len()
        );
        sim.tick = import.tick;
        sim.agents.clear();

        let mut max_id = 0u64;
        for snap in import.agents {
            max_id = max_id.max(snap.agent_id);
            let pos = Position {
                x: snap.x,
                y: snap.y,
                z: snap.z,
            };
            let mut agent = sim_core::agent::Agent::new(
                snap.agent_id,
                pos,
                sim_core::genome::Genome::default(),
                sim.tick.saturating_sub(snap.age),
                snap.sex,
            );
            // Restore genome gains to preserve exact agent color per SIM-GENOME-001
            for (k, &g) in snap.genome_gains.iter().enumerate() {
                let trait_val = ((g - 0.05) / 0.15).clamp(0.0, 1.0);
                if (k + 5) * 2 + 1 < agent.genome.alleles.len() {
                    agent.genome.alleles[(k + 5) * 2] = [trait_val, trait_val];
                    agent.genome.alleles[(k + 5) * 2 + 1] = [trait_val, trait_val];
                }
            }
            agent.energy = snap.energy;
            agent.docas.levels = snap.docas;
            agent.alive = snap.alive;
            agent.has_verified_id = snap.has_verified_id;
            agent.age = snap.age;
            agent.reputation_score = snap.reputation;
            agent.group_id = snap.group_id;
            agent.last_dx = snap.last_dx;
            agent.last_dy = snap.last_dy;
            sim.hydrate_agent_subsystems(&mut agent);

            let trait_snapshot =
                sim_core::lineage::TraitSnapshot::from_genome(&agent.genome, sim.tick);
            let lineage_node = sim_core::lineage::LineageNode {
                agent_id: snap.agent_id,
                birth_tick: sim.tick.saturating_sub(snap.age),
                death_tick: if snap.alive { None } else { Some(sim.tick) },
                generation: snap.generation,
                parent1_id: snap.parent1_id,
                parent2_id: snap.parent2_id,
                birth_traits: trait_snapshot,
                children: vec![],
            };
            sim.lineage.add_initial(lineage_node);
            sim.agents.push(agent);
        }
        sim.set_next_agent_id(max_id + 1);
        info!(
            "[SimLoop] State hydration complete: restored {} agents at tick {}",
            sim.agents.len(),
            sim.tick
        );
    } else {
        info!(
            "[SimLoop] Spawning {} initial agents centered in map...",
            agent_count
        );
        let center_x = params.grid_width as f32 / 2.0;
        let center_y = params.grid_height as f32 / 2.0;
        let max_radius = 0.22 * (params.grid_width.min(params.grid_height) as f32);
        const GOLDEN_ANGLE: f32 = 2.3999632; // pi * (3 - sqrt(5))

        for i in 0..agent_count {
            let frac = (i as f32 + 0.5) / agent_count.max(1) as f32;
            let r = max_radius * frac.sqrt();
            let theta = (i as f32) * GOLDEN_ANGLE;
            let jitter_x = ((i * 17 + 31) % 100) as f32 / 100.0 - 0.5;
            let jitter_y = ((i * 37 + 73) % 100) as f32 / 100.0 - 0.5;

            let x = (center_x + r * theta.cos() + jitter_x * 2.0)
                .clamp(2.0, (params.grid_width as f32) - 2.0);
            let y = (center_y + r * theta.sin() + jitter_y * 2.0)
                .clamp(2.0, (params.grid_height as f32) - 2.0);

            let agent_id = sim.spawn_agent(Position::new_2d(x, y));
            info!(
                "[SimLoop] [AgentSpawn] id={} pos=({:.1},{:.1}) [CenterSpawning]",
                agent_id, x, y
            );
        }
    }

    info!("[SimLoop] Spawned {} initial agents", sim.agents.len());
    let initial_frame = create_frame(&sim);
    *state.current_frame.write() = Some(initial_frame.clone());
    let _ = state.sender.send(initial_frame);

    info!("[SimLoop] --- Generation verification for first 10 agents ---");
    for (i, agent) in sim.agents.iter().take(10).enumerate() {
        if let Some(node) = sim.lineage.get_node(agent.id) {
            info!(
                "[SimLoop] Agent {}: id={}, generation={}",
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                i,
                agent.id,
                node.generation
            );
        }
    }
    info!(
        "[SimLoop] Lineage tree stats: total_births={}, max_generation={}",
        sim.lineage.total_births(),
        sim.lineage.max_generation()
    );
    info!("[SimLoop] Max ticks: {} (0 = infinite)", config.max_ticks);
    info!("[SimLoop] Simulation will start ticking every 100ms");
    info!("[SimLoop] ============================================");

    let mut last_log = std::time::Instant::now();
    let mut tick_count = 0u64;
    let mut last_positions: std::collections::HashMap<u64, (f32, f32)> =
        std::collections::HashMap::new();
    let mut empty_client_count: u32 = 0;
    let mut stalled_tick_count = 0u64;
    let mut last_intervention_seq = 0u64;

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    loop {
        if is_stopped() {
            send_final_frame(state, &sim, config.max_ticks, SimulationStatus::Stopped);
            break;
        }

        if is_paused() {
            let current_seq = INTERVENTION_SEQ.load(Ordering::Acquire);
            if current_seq != last_intervention_seq {
                let live_params = SIMULATION_PARAMS.read().clone();
                sync_live_params(&live_params, &mut sim);
                let standby_frame = create_frame(&sim);
                *state.current_frame.write() = Some(standby_frame.clone());
                if state.sender.receiver_count() > 0 {
                    let _ = state.sender.send(standby_frame);
                }
                last_intervention_seq = current_seq;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
            continue;
        }

        if config.max_ticks > 0 && sim.tick >= config.max_ticks {
            send_final_frame(state, &sim, config.max_ticks, SimulationStatus::Completed);
            break;
        }

        let should_break = process_tick(
            state,
            &mut sim,
            &mut tick_count,
            &mut last_log,
            &mut last_positions,
            &mut empty_client_count,
            &mut stalled_tick_count,
        );

        if should_break {
            break;
        }

        let delay_ms = TICK_DELAY_MS.load(Ordering::Acquire);
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }
}

fn send_final_frame(
    state: &LiveSimulationState,
    sim: &Simulation,
    max_ticks: u64,
    status: SimulationStatus,
) {
    let mut frame = create_frame(sim);
    frame.status = Some(status);
    frame.max_ticks = Some(max_ticks);
    frame.grid_dimensions = Some((sim.environment.width as u32, sim.environment.height as u32));
    let _ = state.sender.send(frame);
}

fn log_frame_movement(
    frame: &SimulationFrame,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    last_positions: &mut std::collections::HashMap<u64, (f32, f32)>,
    receiver_count: usize,
) {
    let mut total_movement = 0.0;
    let mut movement_count = 0;
    for agent in &frame.agents {
        if let Some((old_x, old_y)) = last_positions.get(&agent.agent_id) {
            let dx = agent.x - old_x;
            let dy = agent.y - old_y;
            total_movement += dx.hypot(dy);
            movement_count += 1;
        }
    }
    let avg_movement = if movement_count > 0 {
        total_movement / movement_count as f32
    } else {
        0.0
    };

    info!(
        "[SimLoop] [FrameBroadcast] tick={} agents={} alive={} receivers={} avg_movement={:.3}",
        frame.tick,
        frame.agents.len(),
        frame.stats.alive_agents,
        receiver_count,
        avg_movement
    );

    for agent in &frame.agents {
        last_positions.insert(agent.agent_id, (agent.x, agent.y));
    }
}

/// spawn simulation loop
pub fn spawn_simulation_loop(state: LiveSimulationState) {
    tokio::spawn(async move {
        loop {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            info!("[SimLoop] ============================================");
            info!("[SimLoop] STARTING simulation loop");
            info!("[SimLoop] Broadcast channel capacity: 256 frames");
            info!("[SimLoop] Timestamp: {:?}", std::time::SystemTime::now());

            reset_stop_flag();
            PAUSED.store(!HAS_STARTED.load(Ordering::Acquire), Ordering::Release);
            clear_restart_request();

            let params = SIMULATION_PARAMS.read().clone();
            info!("[SimLoop] Using parameters: {:?}", params);

            let toggles = if let Some(t) = params.toggles {
                t
            } else if let Some(ref p) = params.preset {
                match p.to_lowercase().as_str() {
                    "stage_0" | "physics" => sim_core::toggles::ToggleConfig::stage_0(),
                    "stage_1" | "biology" | "sim-d2" => sim_core::toggles::ToggleConfig::stage_1(),
                    "stage_2" | "cognition" | "sim-g9" | "sim-w1" | "sim-d1" | "sim-gen2"
                    | "sim-gen3" | "sim-bio1" | "sim-at3" | "sim-thermo1" | "sim-patch1"
                    | "sim-mem" => sim_core::toggles::ToggleConfig::stage_2(),
                    "stage_3" | "sim-g7" | "sim-g8" => sim_core::toggles::ToggleConfig::stage_3(),
                    "stage_4" | "culture" => sim_core::toggles::ToggleConfig::stage_4(),
                    "stage_5" | "reconciled" | "full" | "sim-p1" | "sim-p3" | "sim-p5"
                    | "sim-p8" | "sim-w2" | "sim-w3" | "sim-w4" | "sim-w6" | "sim-vot1"
                    | "sim-p8c" | "sim-p8d" | "sim-w8" | "sim-w9" | "sim-at2" | "sim-amp" => {
                        sim_core::toggles::ToggleConfig::stage_5()
                    }
                    _ => sim_core::toggles::ToggleConfig::stage_5(),
                }
            } else {
                sim_core::toggles::ToggleConfig::stage_5()
            };

            let mut config = SimulationConfig {
                toggles,
                environment: EnvironmentConfig {
                    width: params.grid_width as usize,
                    height: params.grid_height as usize,
                    resource_density: params.resource_density,
                    initial_resource: params.initial_resource,
                    altitude_effects_enabled: true,
                    weather_enabled: true,
                    ..EnvironmentConfig::default()
                },
                weather_shield_enabled: true,
                ..SimulationConfig::default()
            };

            apply_config_overrides(&mut config, &params);
            if let Some(mt) = params.max_ticks {
                config.max_ticks = mt;
            }

            run_simulation_session(&state, &config).await;

            info!("[SimLoop] Simulation loop terminated - will restart when clients connect");
            info!("[SimLoop] Waiting for new client connections or restart request...");
            loop {
                tokio::time::sleep(Duration::from_millis(100)).await;
                if is_restart_requested() || state.sender.receiver_count() > 0 {
                    info!(
                        "[SimLoop] Restart requested or new client connected - starting simulation"
                    );
                    break;
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stop_flag_operations() {
        reset_stop_flag();
        assert!(!is_stopped());
        STOPPED.store(true, Ordering::Release);
        assert!(is_stopped());
        reset_stop_flag();
        assert!(!is_stopped());
    }
}
