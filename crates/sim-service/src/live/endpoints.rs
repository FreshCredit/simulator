//! Live simulation HTTP endpoints and controls

use super::*;
use axum::{
    extract::{Query, State},
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream::Stream;
use futures::StreamExt as FuturesStreamExt;
use std::convert::Infallible;
use tokio_stream::wrappers::BroadcastStream;
use tracing::{error, info, trace, warn};

/// SSE stream of live simulation frames
pub async fn live_stream(
    State(state): State<LiveSimulationState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let client_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    info!("[SSE] ============================================");
    info!("[SSE] NEW CLIENT CONNECTING to /api/live");
    info!("[SSE] Client ID: {}", client_id);
    info!(
        "[SSE] Current subscriber count: {}",
        state.sender.receiver_count()
    );
    info!("[SSE] Timestamp: {:?}", std::time::SystemTime::now());

    // Subscribe to broadcast channel
    let rx = state.sender.subscribe();
    info!("[SSE] Client {} subscribed to broadcast channel", client_id);

    // Send current frame first (for late joiners)
    let current = state.current_frame.read().clone();
    info!(
        "[SSE] Client {}: Current frame available: {}",
        client_id,
        current.is_some()
    );

    if let Some(ref frame) = current {
        info!(
            "[SSE] Client {}: Current frame - tick: {}, agents: {}, alive: {}",
            client_id,
            frame.tick,
            frame.agents.len(),
            frame.stats.alive_agents
        );
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    // Convert broadcast stream to SSE events using futures::StreamExt
    let client_id_for_logs = client_id;
    let broadcast_stream = BroadcastStream::new(rx);
    let stream = FuturesStreamExt::filter_map(broadcast_stream, move |result| async move {
        match result {
            Ok(frame) => {
                trace!(
                    "[SSE] Broadcasting frame tick: {} to client {}",
                    frame.tick,
                    client_id_for_logs
                );
                match Event::default().json_data(frame) {
                    Ok(event) => Some(Ok(event)),
                    Err(e) => {
                        error!(
                            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                            "[SSE] Failed to serialize frame for client {}: {}",
                            client_id_for_logs, e
                        );
                        None
                    }
                }
            }
            Err(e) => {
                warn!(
                    "[SSE] Client {}: Broadcast stream lagged, may have missed frames: {}",
                    client_id_for_logs, e
                );
                None
            }
        }
    });

    // Prepend current frame if available
    let stream: futures::stream::BoxStream<'static, Result<Event, Infallible>> =
        if let Some(frame) = current {
            info!(
                "[SSE] Client {}: Sending initial frame (tick={})",
                client_id, frame.tick
            );
            // Serialize frame to JSON - this should never fail for SimulationFrame
            let initial = futures::stream::once(async move {
                Ok(Event::default()
                    .json_data(frame)
                    .unwrap_or_else(|_| Event::default().data("{}")))
            });
            Box::pin(FuturesStreamExt::chain(initial, stream))
        } else {
            info!(
                "[SSE] Client {}: No current frame available, client will wait for next tick",
                client_id
            );
            Box::pin(stream)
        };

    info!(
        "[SSE] Client {}: Stream ready, returning SSE response",
        client_id
    );
    info!("[SSE] ============================================");

    Sse::new(stream).keep_alive(KeepAlive::default())
}

// TAG: surface=simulation owner=research-team rule=SIM-001
// TAG: surface=simulation owner=research-team rule=SIM-001
/// Speed control request
#[derive(Debug, Deserialize)]
pub struct SpeedRequest {
    /// Speed factor: 0.5 = half speed (200ms), 1.0 = normal (100ms), 2.0 = double speed (50ms)
    pub factor: f32,
}

/// Speed response
#[derive(Debug, Serialize)]
pub struct SpeedResponse {
    /// factor
    pub factor: f32,
    /// delay ms
    pub delay_ms: u64,
}

/// POST /api/live/speed - Set simulation speed
/// Phase 1: UI speed control without affecting simulation logic
pub async fn set_speed(axum::Json(request): axum::Json<SpeedRequest>) -> axum::Json<SpeedResponse> {
    // Clamp factor to valid range (0.5x to 5.0x per Master Directive Part 4)
    let factor = request.factor.clamp(0.5, 5.0);

    // Convert factor to delay: 100ms / factor
    // factor 0.5 -> 200ms (slower), factor 2.0 -> 50ms (faster)
    let delay_ms = (100.0 / factor) as u64;

    set_tick_delay(delay_ms);

    axum::Json(SpeedResponse { factor, delay_ms })
}

/// GET /api/live/speed - Get current simulation speed
pub async fn get_speed() -> axum::Json<SpeedResponse> {
    let delay_ms = get_tick_delay();
    let factor = 100.0 / delay_ms as f32;

    axum::Json(SpeedResponse { factor, delay_ms })
}

// ============================================================================
// Simulation Lifecycle Management (Pause/Resume/Stop/Export/Import)
// ============================================================================

/// POST /api/live/pause - Pause the simulation
pub async fn pause() -> axum::Json<serde_json::Value> {
    pause_simulation();
    axum::Json(serde_json::json!({
        "status": "paused",
        "message": "Simulation paused. Use /api/live/resume to continue."
    // TAG: surface=simulation owner=research-team rule=SIM-001
    }))
}

/// POST /api/live/resume - Resume the simulation
pub async fn resume() -> axum::Json<serde_json::Value> {
    resume_simulation();
    axum::Json(serde_json::json!({
        "status": "running",
        "message": "Simulation resumed"
    }))
}

/// POST /api/live/stop - Stop the simulation (terminates the loop)
pub async fn stop() -> axum::Json<serde_json::Value> {
    stop_simulation();
    axum::Json(serde_json::json!({
        "status": "stopped",
        "message": "Simulation stop requested. The loop will terminate after current tick."
    }))
}

/// POST /api/live/restart - Restart simulation with configurable parameters
/// This allows the UI to start a new simulation with custom settings
#[derive(Debug, Deserialize)]
pub struct RestartRequest {
    /// Resource density in [0, 1]
    #[serde(default = "default_resource_density")]
    pub resource_density: f32,
    /// Initial resource amount
    #[serde(default = "default_initial_resource")]
    pub initial_resource: f32,
    /// Number of agents to spawn
    #[serde(default = "default_agent_count")]
    pub agent_count: u32,
    /// Grid width
    #[serde(default = "default_grid_width")]
    pub grid_width: u32,
    /// Grid height
    #[serde(default = "default_grid_height")]
    pub grid_height: u32,
    /// Custom toggle configuration
    #[serde(default)]
    pub toggles: Option<sim_core::toggles::ToggleConfig>,
    /// Selected preset name
    #[serde(default)]
    pub preset: Option<String>,
    /// Max ticks
    #[serde(default)]
    pub max_ticks: Option<u64>,
    /// Layer 4: DOCAS alpha base
    #[serde(default)]
    pub docas_alpha_base: Option<f32>,
    /// Layer 4: Trust curvature window
    #[serde(default)]
    pub trust_curvature_window: Option<usize>,
    /// Layer 5: Planning horizon
    #[serde(default)]
    pub planning_horizon: Option<u64>,
    /// Layer 5: Emotion decay rate
    #[serde(default)]
    pub emotion_decay_rate: Option<f32>,
    /// Layer 6: Worldview drift rate
    #[serde(default)]
    pub worldview_drift_rate: Option<f32>,
    /// Layer 7: Trust decay rate
    #[serde(default)]
    pub trust_decay_rate: Option<f32>,
    /// Layer 7: Reputation threshold
    #[serde(default)]
    pub reputation_threshold: Option<f32>,
    /// Layer 8: Credit limit
    #[serde(default)]
    pub credit_limit: Option<f32>,
    /// Layer 8: Base interest rate
    #[serde(default)]
    pub interest_rate_base: Option<f32>,
    /// Layer 8: Transaction cost
    #[serde(default)]
    pub transaction_cost: Option<f32>,
    /// Layer 9: Block time in ticks
    #[serde(default)]
    pub block_time: Option<u64>,
    /// Layer 9: Consensus threshold
    #[serde(default)]
    pub consensus_threshold: Option<f32>,
    /// Movement speed multiplier
    #[serde(default)]
    pub movement_speed: Option<f32>,
    /// Social force influence factor
    #[serde(default)]
    pub social_force_influence: Option<f32>,
    /// Version 4.0: Optogenetic laser wavelength ("LaserOff", "ChR2Blue470nm", "ArchTYellow566nm")
    #[serde(default)]
    pub opto_wavelength: Option<String>,
    /// Version 4.0: Target optogenetic channels [D, O, C, A, S]
    #[serde(default)]
    pub opto_target_channels: Option<[bool; 5]>,
    /// Version 4.0: Cohort scope ("entire", "d1", "d10", "custom")
    #[serde(default)]
    pub opto_scope: Option<String>,
    /// Version 4.0: Custom subpopulation % in [0, 1]
    #[serde(default)]
    pub opto_subpop_pct: Option<f32>,
    /// Version 4.0: Visual cloaking ratio chi in [1.0, 5.0]
    #[serde(default)]
    pub visual_cloaking_ratio: Option<f32>,
    /// Version 4.0: Brain-IT Branch mode ("BranchAStructural" or "BranchBSemantic")
    #[serde(default)]
    pub brain_it_branch: Option<String>,
    /// Version 4.0: On-device Weather-Shield decision cooling-off buffer active
    #[serde(default)]
    pub weather_shield_enabled: Option<bool>,
    /// Version 4.0: Ambient temperature in Celsius (Sim D17)
    #[serde(default)]
    pub weather_temperature: Option<f32>,
    /// Version 4.0: Melanopsin 480nm circadian photic drive enabled
    #[serde(default)]
    pub melanopsin_enabled: Option<bool>,
    /// Version 4.0: Baylor unconscious narrative gating mode (Sim D18)
    #[serde(default)]
    pub baylor_unconscious_mode: Option<bool>,
    /// Kingman coalescence intensity kappa* (default: 1.62, Sim K)
    #[serde(default)]
    pub kingman_coalescent: Option<f32>,
    /// Lineage depth
    #[serde(default)]
    pub lineage_depth: Option<usize>,
    /// Music metric hierarchy ratio tau (Sim M)
    #[serde(default)]
    pub music_metric_hierarchy_tau: Option<f32>,
    /// Altitude effects enabled
    #[serde(default)]
    pub altitude_effects_enabled: Option<bool>,
    /// Weather enabled
    #[serde(default)]
    pub weather_enabled: Option<bool>,
    /// Energy consumption rate
    #[serde(default)]
    pub energy_consumption: Option<f32>,
    /// Movement cost
    #[serde(default)]
    pub movement_cost: Option<f32>,
    /// Aging beta
    #[serde(default)]
    pub aging_beta: Option<f32>,
    /// Reproduction threshold
    #[serde(default)]
    pub repro_threshold: Option<f32>,
    /// Mutation rate
    #[serde(default)]
    pub mutation_rate: Option<f32>,
    /// Language semantic drift rate factor
    #[serde(default)]
    pub language_drift_rate_factor: Option<f32>,
    /// Language coherence threshold
    #[serde(default)]
    pub language_coherence_threshold: Option<f32>,
    /// Civilization Challenge: Epigenetic methylation rate [0.0, 1.0]
    #[serde(default)]
    pub epigenetic_methylation_rate: Option<f32>,
    /// Civilization Challenge: Circadian sleep deficit [0.0, 1.0]
    #[serde(default)]
    pub circadian_sleep_deficit: Option<f32>,
    /// Civilization Challenge: Wealth redistribution tax rate [0.0, 0.50]
    #[serde(default)]
    pub tax_redistribution_rate: Option<f32>,
    /// Civilization Challenge: Bank run sunspot sensitivity [0.0, 2.0]
    #[serde(default)]
    pub bank_run_sensitivity: Option<f32>,
}

const fn default_resource_density() -> f32 {
    0.30
}
const fn default_initial_resource() -> f32 {
    50.0
}
const fn default_agent_count() -> u32 {
    20
}
const fn default_grid_width() -> u32 {
    100
}
const fn default_grid_height() -> u32 {
    100
}

/// restart
pub async fn restart(
    axum::Json(request): axum::Json<RestartRequest>,
) -> axum::Json<serde_json::Value> {
    let params = SimulationParams {
        resource_density: sanitize_f32(Some(request.resource_density), 0.30, 0.0, 1.0),
        initial_resource: sanitize_f32(Some(request.initial_resource), 50.0, 1.0, 100.0),
        agent_count: request.agent_count.clamp(5, 500),
        grid_width: request.grid_width.clamp(32, 512),
        grid_height: request.grid_height.clamp(32, 512),
        toggles: request.toggles,
        preset: request.preset,
        max_ticks: request.max_ticks,
        docas_alpha_base: request
            .docas_alpha_base
            .map(|v| sanitize_f32(Some(v), 0.1, 0.0, 1.0)),
        trust_curvature_window: request.trust_curvature_window,
        planning_horizon: request.planning_horizon,
        emotion_decay_rate: request
            .emotion_decay_rate
            .map(|v| sanitize_f32(Some(v), 0.05, 0.0, 1.0)),
        worldview_drift_rate: request
            .worldview_drift_rate
            .map(|v| sanitize_f32(Some(v), 0.05, 0.0, 1.0)),
        trust_decay_rate: request
            .trust_decay_rate
            .map(|v| sanitize_f32(Some(v), 0.05, 0.0, 1.0)),
        reputation_threshold: request
            .reputation_threshold
            .map(|v| sanitize_f32(Some(v), 0.5, 0.0, 1.0)),
        credit_limit: request
            .credit_limit
            .map(|v| sanitize_f32(Some(v), 100.0, 0.0, 10000.0)),
        interest_rate_base: request
            .interest_rate_base
            .map(|v| sanitize_f32(Some(v), 0.05, 0.0, 1.0)),
        transaction_cost: request
            .transaction_cost
            .map(|v| sanitize_f32(Some(v), 0.01, 0.0, 10.0)),
        block_time: request.block_time,
        consensus_threshold: request
            .consensus_threshold
            .map(|v| sanitize_f32(Some(v), 0.67, 0.5, 1.0)),
        movement_speed: request
            .movement_speed
            .map(|v| sanitize_f32(Some(v), 1.0, 0.1, 10.0)),
        social_force_influence: request
            .social_force_influence
            .map(|v| sanitize_f32(Some(v), 1.0, 0.0, 10.0)),
        opto_wavelength: request.opto_wavelength,
        opto_target_channels: request.opto_target_channels,
        opto_scope: request.opto_scope,
        opto_subpop_pct: request
            .opto_subpop_pct
            .map(|p| sanitize_f32(Some(p), 0.10, 0.01, 1.0)),
        visual_cloaking_ratio: request
            .visual_cloaking_ratio
            .map(|c| sanitize_f32(Some(c), 1.0, 1.0, 5.0)),
        brain_it_branch: request.brain_it_branch,
        weather_shield_enabled: request.weather_shield_enabled,
        weather_temperature: request
            .weather_temperature
            .map(|t| sanitize_f32(Some(t), 20.0, -30.0, 60.0)),
        melanopsin_enabled: request.melanopsin_enabled,
        baylor_unconscious_mode: request.baylor_unconscious_mode,
        kingman_coalescent: request
            .kingman_coalescent
            .map(|k| sanitize_f32(Some(k), 1.62, 0.01, 10.0)),
        lineage_depth: request.lineage_depth,
        music_metric_hierarchy_tau: request
            .music_metric_hierarchy_tau
            .map(|t| sanitize_f32(Some(t), 0.50, 0.0, 1.0)),
        altitude_effects_enabled: request.altitude_effects_enabled,
        weather_enabled: request.weather_enabled,
        energy_consumption: request
            .energy_consumption
            .map(|e| sanitize_f32(Some(e), 1.0, 0.1, 10.0)),
        movement_cost: request
            .movement_cost
            .map(|m| sanitize_f32(Some(m), 0.5, 0.01, 10.0)),
        aging_beta: request
            .aging_beta
            .map(|b| sanitize_f32(Some(b), 0.10, 0.01, 1.0)),
        repro_threshold: request
            .repro_threshold
            .map(|r| sanitize_f32(Some(r), 75.0, 10.0, 500.0)),
        mutation_rate: request
            .mutation_rate
            .map(|m| sanitize_f32(Some(m), 0.001, 0.00001, 0.5)),
        language_drift_rate_factor: request
            .language_drift_rate_factor
            .map(|d| sanitize_f32(Some(d), 1.0, 0.0, 10.0)),
        language_coherence_threshold: request
            .language_coherence_threshold
            .map(|c| sanitize_f32(Some(c), 0.70, 0.0, 1.0)),
        epigenetic_methylation_rate: request
            .epigenetic_methylation_rate
            .map(|e| sanitize_f32(Some(e), 0.05, 0.0, 1.0)),
        circadian_sleep_deficit: request
            .circadian_sleep_deficit
            .map(|c| sanitize_f32(Some(c), 0.0, 0.0, 1.0)),
        tax_redistribution_rate: request
            .tax_redistribution_rate
            .map(|t| sanitize_f32(Some(t), 0.0, 0.0, 0.50)),
        bank_run_sensitivity: request
            .bank_run_sensitivity
            .map(|b| sanitize_f32(Some(b), 1.0, 0.0, 2.0)),
    };

    restart_simulation_with_params(params.clone());

    axum::Json(serde_json::json!({
        "status": "restarting",
        "message": "Simulation restart requested with new parameters",
        "params": params
    }))
}

/// Real-time intervention request for live simulation updates
#[derive(Debug, Deserialize)]
pub struct InterventionRequest {
    /// Version 4.0: Optogenetic laser wavelength ("LaserOff", "ChR2Blue470nm", "ArchTYellow566nm")
    #[serde(default)]
    pub opto_wavelength: Option<String>,
    /// Version 4.0: Target optogenetic channels [D, O, C, A, S]
    #[serde(default)]
    pub opto_target_channels: Option<[bool; 5]>,
    /// Version 4.0: Cohort scope ("entire", "d1", "d10", "custom")
    #[serde(default)]
    pub opto_scope: Option<String>,
    /// Version 4.0: Custom subpopulation % in [0, 1]
    #[serde(default)]
    pub opto_subpop_pct: Option<f32>,
    /// Version 4.0: Visual cloaking ratio chi in [1.0, 5.0]
    #[serde(default)]
    pub visual_cloaking_ratio: Option<f32>,
    /// Version 4.0: Brain-IT Branch mode ("BranchAStructural" or "BranchBSemantic")
    #[serde(default)]
    pub brain_it_branch: Option<String>,
    /// Version 4.0: On-device Weather-Shield decision cooling-off buffer active
    #[serde(default)]
    pub weather_shield_enabled: Option<bool>,
    /// Version 4.0: Ambient temperature in Celsius (Sim D17)
    #[serde(default)]
    pub weather_temperature: Option<f32>,
    /// Version 4.0: Melanopsin 480nm circadian photic drive enabled
    #[serde(default)]
    pub melanopsin_enabled: Option<bool>,
    /// Version 4.0: Baylor unconscious narrative gating mode (Sim D18)
    #[serde(default)]
    pub baylor_unconscious_mode: Option<bool>,
    /// Live toggle configurations
    #[serde(default)]
    pub toggles: Option<sim_core::toggles::ToggleConfig>,
    /// Altitude topography effects enabled
    #[serde(default)]
    pub altitude_effects_enabled: Option<bool>,
    /// Environmental dynamic weather enabled
    #[serde(default)]
    pub weather_enabled: Option<bool>,
    /// Metabolic energy consumption rate
    #[serde(default)]
    pub energy_consumption: Option<f32>,
    /// Movement energy cost modifier
    #[serde(default)]
    pub movement_cost: Option<f32>,
    /// Gompertz mortality aging beta
    #[serde(default)]
    pub aging_beta: Option<f32>,
    /// Reproduction energy threshold
    #[serde(default)]
    pub repro_threshold: Option<f32>,
    /// Genetic mutation rate
    #[serde(default)]
    pub mutation_rate: Option<f32>,
    /// Wealth redistribution tax rate
    #[serde(default)]
    pub tax_redistribution_rate: Option<f32>,
    /// Bank run panic sensitivity
    #[serde(default)]
    pub bank_run_sensitivity: Option<f32>,
    /// DOCAS stimulus alpha base
    #[serde(default)]
    pub docas_alpha_base: Option<f32>,
    /// Trust curvature window size
    #[serde(default)]
    pub trust_curvature_window: Option<usize>,
    /// Trust decay rate
    #[serde(default)]
    pub trust_decay_rate: Option<f32>,
    /// Reputation credit access threshold
    #[serde(default)]
    pub reputation_threshold: Option<f32>,
    /// Maximum underwritten credit limit
    #[serde(default)]
    pub credit_limit: Option<f32>,
    /// Base interest rate
    #[serde(default)]
    pub interest_rate_base: Option<f32>,
    /// Layer 5: Planning horizon
    #[serde(default)]
    pub planning_horizon: Option<u64>,
    /// Layer 5: Emotion decay rate
    #[serde(default)]
    pub emotion_decay_rate: Option<f32>,
    /// Layer 6: Worldview drift rate
    #[serde(default)]
    pub worldview_drift_rate: Option<f32>,
    /// Layer 8: Transaction cost
    #[serde(default)]
    pub transaction_cost: Option<f32>,
    /// Layer 9: Block time
    #[serde(default)]
    pub block_time: Option<u64>,
    /// Layer 9: Consensus threshold
    #[serde(default)]
    pub consensus_threshold: Option<f32>,
    /// Epigenetic methylation rate
    #[serde(default)]
    pub epigenetic_methylation_rate: Option<f32>,
    /// Circadian sleep deficit
    #[serde(default)]
    pub circadian_sleep_deficit: Option<f32>,
    /// Kingman coalescent parameter
    #[serde(default)]
    pub kingman_coalescent: Option<f32>,
    /// Lineage depth
    #[serde(default)]
    pub lineage_depth: Option<usize>,
    /// Music metric hierarchy tau
    #[serde(default)]
    pub music_metric_hierarchy_tau: Option<f32>,
    /// Language drift rate factor
    #[serde(default)]
    pub language_drift_rate_factor: Option<f32>,
    /// Language coherence threshold
    #[serde(default)]
    pub language_coherence_threshold: Option<f32>,
}

fn build_params_from_intervention(req: InterventionRequest) -> SimulationParams {
    SimulationParams {
        opto_wavelength: req.opto_wavelength,
        opto_target_channels: req.opto_target_channels,
        opto_scope: req.opto_scope,
        opto_subpop_pct: req
            .opto_subpop_pct
            .map(|p| sanitize_f32(Some(p), 0.10, 0.01, 1.0)),
        visual_cloaking_ratio: req
            .visual_cloaking_ratio
            .map(|c| sanitize_f32(Some(c), 1.0, 1.0, 5.0)),
        brain_it_branch: req.brain_it_branch,
        weather_shield_enabled: req.weather_shield_enabled,
        weather_temperature: req
            .weather_temperature
            .map(|t| sanitize_f32(Some(t), 20.0, -30.0, 60.0)),
        melanopsin_enabled: req.melanopsin_enabled,
        baylor_unconscious_mode: req.baylor_unconscious_mode,
        toggles: req.toggles,
        altitude_effects_enabled: req.altitude_effects_enabled,
        weather_enabled: req.weather_enabled,
        energy_consumption: req.energy_consumption,
        movement_cost: req.movement_cost,
        aging_beta: req.aging_beta,
        repro_threshold: req.repro_threshold,
        mutation_rate: req.mutation_rate,
        tax_redistribution_rate: req.tax_redistribution_rate,
        bank_run_sensitivity: req.bank_run_sensitivity,
        docas_alpha_base: req.docas_alpha_base,
        trust_curvature_window: req.trust_curvature_window,
        trust_decay_rate: req.trust_decay_rate,
        reputation_threshold: req.reputation_threshold,
        credit_limit: req.credit_limit,
        interest_rate_base: req.interest_rate_base,
        planning_horizon: req.planning_horizon,
        emotion_decay_rate: req.emotion_decay_rate,
        worldview_drift_rate: req.worldview_drift_rate,
        transaction_cost: req.transaction_cost,
        block_time: req.block_time,
        consensus_threshold: req.consensus_threshold,
        epigenetic_methylation_rate: req.epigenetic_methylation_rate,
        circadian_sleep_deficit: req.circadian_sleep_deficit,
        kingman_coalescent: req.kingman_coalescent,
        lineage_depth: req.lineage_depth,
        music_metric_hierarchy_tau: req.music_metric_hierarchy_tau,
        language_drift_rate_factor: req.language_drift_rate_factor,
        language_coherence_threshold: req.language_coherence_threshold,
        ..Default::default()
    }
}

/// POST /api/live/intervention - Apply real-time intervention without restarting simulation
pub async fn live_intervention(
    axum::Json(request): axum::Json<InterventionRequest>,
) -> axum::Json<serde_json::Value> {
    let params = build_params_from_intervention(request);
    apply_live_intervention(params);
    axum::Json(serde_json::json!({
        "status": "ok",
        "message": "Live intervention applied"
    }))
}

/// GET /api/live/status - Get current simulation status
pub async fn get_status() -> axum::Json<serde_json::Value> {
    let paused = is_paused();
    let stopped = is_stopped();
    let started = HAS_STARTED.load(Ordering::Acquire);

    let status = if !started {
        "standby"
    } else if stopped {
        "stopped"
    } else if paused {
        "paused"
    } else {
        "running"
    };

    axum::Json(serde_json::json!({
        "status": status,
        "started": started,
        "paused": paused,
        "stopped": stopped,
        "tick_delay_ms": get_tick_delay()
    }))
}

/// Simulation state export format for save/load
// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationStateExport {
    /// tick
    pub tick: u64,
    /// config
    pub config: SimulationConfig,
    /// agents
    pub agents: Vec<AgentSnapshot>,
    /// stats
    pub stats: FrameStats,
    /// exported at
    pub exported_at: String,
    /// Historical time-series metrics
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub history: Vec<HistoricalMetricPoint>,
}

/// GET /api/live/state/export - Export current simulation state
pub async fn export_state(
    State(state): State<LiveSimulationState>,
) -> axum::Json<SimulationStateExport> {
    let frame = state.current_frame.read().clone();
    let config = get_live_simulation_config();
    let history = state.history.read().clone();

    if let Some(frame) = frame {
        let export = SimulationStateExport {
            tick: frame.tick,
            config,
            agents: frame.agents.clone(),
            stats: frame.stats,
            exported_at: chrono::Utc::now().to_rfc3339(),
            history,
        };
        axum::Json(export)
    } else {
        // Return empty state if no simulation running
        axum::Json(SimulationStateExport {
            tick: 0,
            config,
            agents: vec![],
            stats: FrameStats {
                total_agents: 0,
                alive_agents: 0,
                verified_agents: 0,
                groups: 0,
            },
            exported_at: chrono::Utc::now().to_rfc3339(),
            history,
        })
    }
}

/// Dual-format payload supporting either canonical SimulationStateExport or client-side UI envelope
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum StateImportPayload {
    /// Canonical backend export schema
    Direct(SimulationStateExport),
    /// Client-side UI frame wrapper
    UiEnvelope {
        /// Optional tick
        tick: Option<u64>,
        /// Snapshot frame
        frame: Option<SimulationFrame>,
        /// Optional timestamp
        timestamp: Option<String>,
    },
}

/// POST /api/live/state/import - Import simulation state with deep stateful hydration
pub async fn import_state(
    State(state): State<LiveSimulationState>,
    axum::Json(payload): axum::Json<StateImportPayload>,
) -> axum::Json<serde_json::Value> {
    let mut export = match payload {
        StateImportPayload::Direct(exp) => exp,
        StateImportPayload::UiEnvelope {
            tick,
            frame,
            timestamp: _,
        } => {
            let active_cfg = get_live_simulation_config();
            let (target_tick, agents, stats) = if let Some(f) = frame {
                (tick.unwrap_or(f.tick), f.agents, f.stats)
            } else {
                (
                    tick.unwrap_or(0),
                    vec![],
                    FrameStats {
                        total_agents: 0,
                        alive_agents: 0,
                        verified_agents: 0,
                        groups: 0,
                    },
                )
            };
            SimulationStateExport {
                tick: target_tick,
                config: active_cfg,
                agents,
                stats,
                exported_at: chrono::Utc::now().to_rfc3339(),
                history: vec![],
            }
        }
    };

    *state.history.write() = export.history.clone();

    // Sanitize agent count and float fields
    if export.agents.len() > 500 {
        export.agents.truncate(500);
    }
    let grid_w = export.config.environment.width as f32;
    let grid_h = export.config.environment.height as f32;
    for a in &mut export.agents {
        if !a.x.is_finite() {
            a.x = grid_w / 2.0;
        }
        if !a.y.is_finite() {
            a.y = grid_h / 2.0;
        }
        if !a.z.is_finite() {
            a.z = 0.0;
        }
        if !a.energy.is_finite() {
            a.energy = 50.0;
        }
        for val in &mut a.docas {
            if !val.is_finite() {
                *val = 0.5;
            }
            *val = val.clamp(0.0, 1.0);
        }
    }

    info!(
        "[SIM-CONTROL] Importing stateful checkpoint at tick {} with {} agents",
        export.tick,
        export.agents.len()
    );

    let params = SimulationParams {
        grid_width: export.config.environment.width as u32,
        grid_height: export.config.environment.height as u32,
        agent_count: (export.agents.len() as u32).clamp(5, 500),
        toggles: Some(export.config.toggles),
        max_ticks: Some(export.config.max_ticks),
        opto_wavelength: Some(format!("{:?}", export.config.optogenetics.wavelength)),
        opto_target_channels: Some(export.config.optogenetics.target_channels),
        visual_cloaking_ratio: Some(export.config.visual_cloaking.cloaking_ratio),
        weather_shield_enabled: Some(export.config.weather_shield_enabled),
        baylor_unconscious_mode: Some(
            export.config.baylor.mode == sim_core::baylor_gating::ConsciousnessMode::Unconscious,
        ),
        ..Default::default()
    };

    let total_agents = export.agents.len();
    let kappas: Vec<f32> = export
        .agents
        .iter()
        .filter(|a| a.alive)
        .map(|a| {
            let decile = ((a.tau * 10.0).floor() as usize).clamp(1, 10);
            let is_perturbed =
                export
                    .config
                    .optogenetics
                    .is_agent_perturbed(a.agent_id, decile, total_agents);
            let opto_mod = if is_perturbed {
                export
                    .config
                    .optogenetics
                    .modulate_cognitive_bandwidth(1.0, true)
            } else {
                1.0
            };
            a.tau * opto_mod
        })
        .collect();

    let decile_grad = sim_core::visual_cloaking::DecileVulnerabilityGradient::evaluate_population(
        &export.config.visual_cloaking,
        &kappas,
        export.config.weather_shield_enabled,
    );

    let living_count = export.agents.iter().filter(|a| a.alive).count().max(1) as f32;
    let mut sum_docas = [0.0f32; 5];
    for a in export.agents.iter().filter(|a| a.alive) {
        for (i, val) in a.docas.iter().enumerate().take(5) {
            sum_docas[i] += val;
        }
    }
    let simplex = [
        sum_docas[0] / living_count,
        sum_docas[1] / living_count,
        sum_docas[2] / living_count,
        sum_docas[3] / living_count,
        sum_docas[4] / living_count,
    ];

    let mut sum_cos = 0.0f32;
    let mut sum_sin = 0.0f32;
    for a in export.agents.iter().filter(|a| a.alive) {
        let decile = ((a.tau * 10.0).floor() as usize).clamp(1, 10);
        let is_perturbed =
            export
                .config
                .optogenetics
                .is_agent_perturbed(a.agent_id, decile, total_agents);
        let tau_centered = (a.tau - 0.5) * 2.0;
        let phase = 2.0 * tau_centered.atan();
        let coupling_weight = if (export.config.optogenetics.target_channels[sim_core::types::O]
            || export.config.optogenetics.target_channels[sim_core::types::S])
            && export.config.optogenetics.wavelength
                == sim_core::optogenetics::OpsinWavelength::ArchTYellow566nm
        {
            export
                .config
                .optogenetics
                .modulate_kuramoto_coupling(1.0, is_perturbed)
        } else {
            1.0
        };
        sum_cos += coupling_weight * phase.cos();
        sum_sin += coupling_weight * phase.sin();
    }
    let kuramoto_r =
        ((sum_cos * sum_cos + sum_sin * sum_sin).sqrt() / living_count).clamp(0.0, 1.0);

    let avg_cortisol = simplex[sim_core::types::C];
    let is_unconscious =
        export.config.baylor.mode == sim_core::baylor_gating::ConsciousnessMode::Unconscious;
    let cognitive_friction = export.config.baylor.cognitive_friction(avg_cortisol);
    let executive_gate = export.config.baylor.executive_threat_gate(avg_cortisol);
    let attractor_shift = export
        .config
        .baylor
        .attractor_shift(simplex[sim_core::types::O], simplex[sim_core::types::S]);

    let v4 = V4TelemetrySnapshot {
        decile_uptake_rates: decile_grad.decile_uptake_rates,
        odds_ratio_d1_vs_d10: decile_grad.odds_ratio_d1_vs_d10,
        kuramoto_coherence_r: kuramoto_r,
        simplex_projection: simplex,
        active_cloaking_ratio: export.config.visual_cloaking.cloaking_ratio,
        active_brain_it_branch: "BranchAStructural".to_string(),
        laser_wavelength: format!("{:?}", export.config.optogenetics.wavelength),
        target_channels: export.config.optogenetics.target_channels,
        weather_lux: 1000.0,
        weather_thermal_stress: 1.0,
        weather_shield_active: export.config.weather_shield_enabled,
        opto_conductance: export.config.optogenetics.conductance,
        baylor_unconscious_mode: is_unconscious,
        baylor_cognitive_friction: cognitive_friction,
        baylor_executive_gate: executive_gate,
        baylor_attractor_shift: attractor_shift,
    };

    // Immediate frame commit for atomic read-after-write consistency
    let immediate_frame = SimulationFrame {
        tick: export.tick,
        agents: export.agents.clone(),
        stats: export.stats,
        trust_edges: vec![],
        resources: None,
        weather: None,
        terrain: None,
        game_events: vec![],
        encounter_events: vec![],
        status: Some(SimulationStatus::Running),
        max_ticks: Some(export.config.max_ticks),
        grid_dimensions: Some((
            export.config.environment.width as u32,
            export.config.environment.height as u32,
        )),
        v4_telemetry: Some(v4),
    };
    *state.current_frame.write() = Some(immediate_frame);

    let export_for_sim = export.clone();
    set_pending_state_import(export_for_sim);
    restart_simulation_with_params(params);

    axum::Json(serde_json::json!({
        "status": "imported",
        "message": format!(
            "Simulation state successfully imported with true stateful hydration at tick {}",
            export.tick
        ),
        "tick": export.tick,
        "agents": export.agents.len()
    }))
}

/// GET /api/live/report - Get comprehensive simulation report
/// Returns event history and summary for AI analysis
pub async fn get_simulation_report(
    State(state): State<LiveSimulationState>,
    Query(params): Query<ReportParams>,
) -> axum::Json<serde_json::Value> {
    let max_events = params.max_events.unwrap_or(100);
    let frame_opt = state.current_frame.read().clone();

    if let Some(frame) = frame_opt {
        axum::Json(serde_json::json!({
            "status": "ok",
            "tick": frame.tick,
            "max_events": max_events,
            "summary": {
                "alive_agents": frame.stats.alive_agents,
                "total_agents": frame.stats.total_agents,
                "verified_agents": frame.stats.verified_agents,
                "groups": frame.stats.groups
            },
            "recent_agents": frame.agents.into_iter().take(max_events).collect::<Vec<_>>()
        }))
    } else {
        axum::Json(serde_json::json!({
            "status": "idle",
            "message": "Simulation not yet running or no frames generated",
            "summary": {
                "alive_agents": 0,
                "total_agents": 0,
                "verified_agents": 0,
                "groups": 0
            }
        }))
    }
}

/// Query parameters for report endpoint
#[derive(Debug, Deserialize)]
pub struct ReportParams {
    /// max events
    pub max_events: Option<usize>,
}
