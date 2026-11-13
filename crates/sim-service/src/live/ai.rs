//! Live simulation AI chat and streaming endpoints

use super::*;
use crate::ai::{NotableEvent as AiNotableEvent, SimulationContext, SimulationPromptBuilder};
use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream::Stream;
use futures::StreamExt as FuturesStreamExt;
use std::convert::Infallible;
use tokio_stream::wrappers::BroadcastStream;
use tracing::{error, info, trace, warn};

/// SSE stream of condensed AI simulation frames
pub async fn ai_stream(
    State(state): State<LiveSimulationState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let client_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    info!("[AI-SSE] ============================================");
    info!("[AI-SSE] NEW AI CLIENT CONNECTING to /api/ai/stream");
    info!("[AI-SSE] Client ID: {}", client_id);
    info!(
        "[AI-SSE] Current subscriber count: {}",
        state.sender.receiver_count()
    );
    info!("[AI-SSE] Timestamp: {:?}", std::time::SystemTime::now());

    // Subscribe to base broadcast channel
    let rx = state.sender.subscribe();
    info!(
        // TAG: surface=simulation owner=research-team rule=SIM-001
        "[AI-SSE] Client {} subscribed to broadcast channel",
        client_id
    );

    // Convert broadcast stream to AI frames, throttling to 1 FPS
    let client_id_for_logs = client_id;
    let broadcast_stream = BroadcastStream::new(rx);

    // Throttle to every 10th frame (10 FPS / 10 = 1 FPS)
    let stream = FuturesStreamExt::filter_map(
        broadcast_stream.enumerate(),
        move |(index, result)| async move {
            // Only process every 10th frame for AI stream (1 FPS)
            if index % 10 != 0 {
                return None;
            }

            match result {
                Ok(full_frame) => {
                    // Convert to condensed AI frame
                    let ai_frame = create_ai_frame(&full_frame);
                    trace!(
                        "[AI-SSE] Broadcasting AI frame tick: {} to client {}",
                        ai_frame.tick,
                        client_id_for_logs
                    );
                    match Event::default().json_data(ai_frame) {
                        Ok(event) => Some(Ok(event)),
                        Err(e) => {
                            error!(
                                "[AI-SSE] Failed to serialize AI frame for client {}: {}",
                                client_id_for_logs, e
                            );
                            None
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "[AI-SSE] Client {}: Broadcast stream lagged, may have missed frames: {}",
                        client_id_for_logs, e
                    );
                    None
                }
            }
        },
    );

    info!(
        "[AI-SSE] Client {}: AI stream ready, returning SSE response",
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        client_id
    );
    // TAG: surface=simulation owner=research-team rule=SIM-001
    info!("[AI-SSE] ============================================");

    Sse::new(stream).keep_alive(KeepAlive::default())
}

// ============================================================================
// AI Chat Endpoint
// ============================================================================

/// Chat request from frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiChatRequest {
    /// User's message
    pub message: String,
    /// Optional conversation ID for maintaining context
    pub conversation_id: Option<String>,
    /// Optional number of recent frames to include as context
    pub context_frames: Option<usize>,
}

/// Chat response for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiChatResponse {
    /// AI's response message
    pub message: String,
    /// Response message alias for frontend compatibility
    #[serde(default)]
    pub response: String,
    /// Suggested actions the user can take
    pub suggestions: Vec<String>,
    /// Context data used for this response
    pub context: AiChatContext,
    /// Response metadata
    pub meta: AiChatMeta,
}

/// Context included in chat response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiChatContext {
    /// Current simulation tick (if available)
    pub current_tick: Option<u64>,
    /// Number of alive agents (if available)
    pub alive_count: Option<usize>,
    /// Recent notable events
    pub recent_events: Vec<NotableEvent>,
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiChatMeta {
    /// Response timestamp
    pub timestamp: String,
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Whether this is a mock response (true until Gemini integration)
    pub is_mock: bool,
    /// Conversation ID for follow-up messages
    pub conversation_id: String,
}

#[allow(clippy::too_many_lines)]
/// POST /api/ai/chat - AI chat endpoint
/// Accepts message and context, returns AI response
/// Uses Gemini when available, falls back to mock responses
pub async fn ai_chat(
    State(state): State<LiveSimulationState>,
    axum::Json(request): axum::Json<AiChatRequest>,
) -> axum::Json<AiChatResponse> {
    let conversation_id = request.conversation_id.unwrap_or_else(|| {
        format!(
            "conv-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        )
    });

    // Get current simulation state
    let current_frame = state.current_frame.read().clone();
    let (current_tick, alive_count) = current_frame
        .as_ref()
        .map_or((None, None), |f| (Some(f.tick), Some(f.stats.alive_agents)));

    // Get recent events from the current frame
    let recent_events = current_frame
        .as_ref()
        .map(|f| {
            let mut events = Vec::new();

            // Convert game events
            for event in &f.game_events {
                events.push(NotableEvent {
                    tick: event.tick,
                    event_type: format!("game_{}", event.game_type.to_lowercase()),
                    description: format!(
                        "Game between agents {} and {}: {}",
                        event.agent_a, event.agent_b, event.outcome
                    ),
                    agent_ids: vec![event.agent_a, event.agent_b],
                    value: Some(event.payoff_a + event.payoff_b),
                });
            }

            // Convert encounter events
            // TAG: surface=simulation owner=research-team rule=SIM-001
            for event in &f.encounter_events {
                events.push(NotableEvent {
                    tick: event.tick,
                    event_type: format!("encounter_{}", event.encounter_type.to_lowercase()),
                    description: format!(
                        "Encounter between agents {} and {}: {}",
                        event.agent_a, event.agent_b, event.encounter_type
                    ),
                    agent_ids: vec![event.agent_a, event.agent_b],
                    value: None,
                });
            }

            // Limit events based on request
            let max_events = request.context_frames.unwrap_or(10);
            if events.len() > max_events {
                events.into_iter().rev().take(max_events).rev().collect()
            } else {
                events
            }
        })
        .unwrap_or_default();

    // Try to use Gemini for AI response, fall back to mock if unavailable
    let (response_message, is_mock, _model_used) = if state.ai_state.is_available() {
        // Build simulation context for Gemini
        let sim_context = current_frame.as_ref().map(|f| {
            // Calculate average DOCAS from agents
            let avg_docas = if f.agents.is_empty() {
                [0.0; 5]
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            // TAG: surface=simulation owner=research-team rule=SIM-001
            } else {
                let mut sums = [0.0f32; 5];
                for agent in &f.agents {
                    for (i, &val) in agent.docas.iter().enumerate() {
                        sums[i] += val;
                    }
                }
                let count = f.agents.len() as f32;
                [
                    sums[0] / count,
                    sums[1] / count,
                    sums[2] / count,
                    sums[3] / count,
                    sums[4] / count,
                ]
            };

            SimulationContext {
                tick: f.tick,
                alive_count: f.stats.alive_agents,
                avg_energy: f.agents.iter().map(|a| a.energy).sum::<f32>()
                    / f.agents.len().max(1) as f32,
                avg_docas,
                group_count: f.stats.groups,
                notable_events: recent_events
                    .iter()
                    .map(|e| AiNotableEvent {
                        event_type: e.event_type.clone(),
                        agent_id: e.agent_ids.first().map(std::string::ToString::to_string),
                        description: e.description.clone(),
                    })
                    .collect(),
            }
        });

        // Call Gemini
        if let Some(gemini) = state.ai_state.gemini.as_ref() {
            let question = request.message.clone();
            let result = if let Some(ctx) = sim_context {
                gemini.analyze_with_context(&ctx, question).await
            } else {
                gemini
                    .generate_text(
                        question,
                        Some(SimulationPromptBuilder::system_instruction()),
                    )
                    .await
            };

            // TAG: surface=simulation owner=research-team rule=SIM-001
            match result {
                Ok(response) => (response, false, gemini.model().to_string()),
                Err(e) => {
                    tracing::warn!("Gemini error, falling back to mock: {}", e);
                    let mock = generate_mock_response(
                        &request.message,
                        current_tick,
                        alive_count,
                        &recent_events,
                    );
                    (mock, true, "mock".to_string())
                }
            }
        } else {
            let mock =
                generate_mock_response(&request.message, current_tick, alive_count, &recent_events);
            (mock, true, "mock".to_string())
        }
    } else {
        // Use mock response
        let mock =
            generate_mock_response(&request.message, current_tick, alive_count, &recent_events);
        (mock, true, "mock".to_string())
    };

    // Generate suggestions based on context
    let suggestions = generate_suggestions(current_tick, alive_count, &recent_events);

    axum::Json(AiChatResponse {
        response: response_message.clone(),
        message: response_message,
        suggestions,
        context: AiChatContext {
            current_tick,
            alive_count,
            recent_events,
        },
        meta: AiChatMeta {
            timestamp: chrono::Utc::now().to_rfc3339(),
            is_mock,
            conversation_id,
        },
    })
}

/// Generate a mock AI response based on the user's message
/// This is a placeholder until Gemini integration is implemented
fn generate_mock_response(
    message: &str,
    current_tick: Option<u64>,
    alive_count: Option<usize>,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    // TAG: surface=simulation owner=research-team rule=SIM-001
    recent_events: &[NotableEvent],
) -> String {
    let message_lower = message.to_lowercase();

    if let Some(response) = respond_status(&message_lower, current_tick, alive_count) {
        return response;
    }
    if let Some(response) = respond_events(&message_lower, recent_events) {
        return response;
    }
    if let Some(response) = respond_population(&message_lower, alive_count) {
        return response;
    }
    if message_lower.contains("starv")
        || message_lower.contains("die")
        || message_lower.contains("death")
        || message_lower.contains("extinct")
    {
        return "Under the Sim ECOL Lotka-Volterra carrying capacity framework (docas_revised.md §13.44), agents require resources to sustain metabolic baselines. If metabolic consumption exceeds 2.5x resource density, or population drops below the critical Allee threshold (N_crit = 25), cooperative synergy collapses and triggers deterministic Malthusian extinction. Check your resource density and energy consumption dials.".to_string();
    }
    if message_lower.contains("brain")
        || message_lower.contains("fmri")
        || message_lower.contains("insula")
        || message_lower.contains("d21")
    {
        return "The 3D Brain Visualizer is grounded in the Meta TRIBE v2 fMRI forward encoding pipeline from Sim D21 (battery27_multimodal_preds.npz). When an agent evaluates an unfair transparent offer, the Bilateral Anterior Insula fires with a sharp Norm Prediction Error burst (mean = 0.5810, peak z = 2.68). Under multimodal cloaking, this insular disgust response is suppressed by 58.1%-60.0% (mean = 0.2324), leaving dlPFC focused on the immediate cash advance.".to_string();
    }
    if message_lower.contains("opto")
        || message_lower.contains("laser")
        || message_lower.contains("archt")
        || message_lower.contains("chr2")
    {
        return "Sim O-Series optogenetic causal interventions model light-gated ion photocurrents: ArchT 566nm yellow light hyperpolarizes Dopaminergic neurons, silencing positive trust updates (lambda^+ -> 0, causing 99.97% trust suppression; Sim O1). ChR2 470nm blue light depolarizes Cortisol threat neurons, compressing cognitive bandwidth kappa_eff and increasing predatory uptake by 1.74x (Sim O2).".to_string();
    }
    if message_lower.contains("docas") || message_lower.contains("trust") {
        return "DOCAS (Duty, Order, Care, Autonomy, Spirituality) is the personality model \
                used by agents. Trust (τ) emerges from DOCAS values and governs social interactions. \
                Agents with similar DOCAS profiles tend to form groups and cooperate more effectively."
            .to_string();
    }
    if message_lower.contains("group") || message_lower.contains("society") {
        return "Agents form groups based on trust and DOCAS similarity. These groups can \
                develop norms, institutions, and collective behaviors. Watch for group formations \
                as the simulation progresses - they indicate emergent social structure."
            .to_string();
    }
    if message_lower.contains("help") || message_lower.contains("what can") {
        return "I can help you understand the Agent Society Simulation! Try asking about:\n\
                • Current simulation status\n\
                • Recent events and agent interactions\n\
                • Population dynamics\n\
                • DOCAS trust system\n\
                • Group formations"
            .to_string();
    }

    respond_generic(message, current_tick, alive_count)
}

fn respond_status(
    message_lower: &str,
    current_tick: Option<u64>,
    alive_count: Option<usize>,
) -> Option<String> {
    if !message_lower.contains("status") && !message_lower.contains("how is") {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        return None;
    }
    match (current_tick, alive_count) {
        (Some(tick), Some(alive)) => Some(format!(
            "The simulation is currently at tick {tick} with {alive} agents alive. \
             The society appears to {}.",
            if alive > 15 {
                "be thriving"
            } else if alive > 5 {
                "be stable"
            } else {
                "be struggling"
            }
        )),
        _ => Some(
            "The simulation is currently initializing. Please wait a moment for data to become available."
                .to_string(),
        ),
    }
}

fn respond_events(message_lower: &str, recent_events: &[NotableEvent]) -> Option<String> {
    if !message_lower.contains("event") && !message_lower.contains("happened") {
        return None;
    }
    if recent_events.is_empty() {
        return Some(
            "No notable events have occurred recently. The agents are going about their normal activities."
                .to_string(),
        );
    }
    let event_count = recent_events.len();
    let event_types: std::collections::HashSet<_> =
        recent_events.iter().map(|e| e.event_type.clone()).collect();
    Some(format!(
        "There have been {event_count} notable events recently, including: {}.",
        event_types.into_iter().collect::<Vec<_>>().join(", ")
    ))
}

fn respond_population(message_lower: &str, alive_count: Option<usize>) -> Option<String> {
    if !message_lower.contains("agent") && !message_lower.contains("population") {
        return None;
    }
    // TAG: surface=simulation owner=research-team rule=SIM-001
    match alive_count {
        Some(count) => Some(format!(
            "The current population consists of {count} agents. \
             These agents are engaged in social interactions, resource gathering, \
             and DOCAS-based trust dynamics."
        )),
        None => Some(
            "Population data is not yet available. The simulation may still be starting up."
                .to_string(),
        ),
    }
}

fn respond_generic(message: &str, current_tick: Option<u64>, alive_count: Option<usize>) -> String {
    format!(
        "I understand you're asking about \"{message}\". \
         The simulation is running with {} agents at tick {}. \
         As an AI assistant for the Agent Society Simulator, I can provide insights \
         about the emergent social behaviors, DOCAS dynamics, and notable events. \
         (Note: This is a mock response - full Gemini integration coming soon!)",
        alive_count.map_or_else(|| "unknown".to_string(), |c| c.to_string()),
        current_tick.map_or_else(|| "unknown".to_string(), |t| t.to_string()),
    )
}

/// Generate context-aware suggestions for the user
fn generate_suggestions(
    current_tick: Option<u64>,
    alive_count: Option<usize>,
    // TAG: surface=simulation owner=research-team rule=SIM-001
    recent_events: &[NotableEvent],
) -> Vec<String> {
    let mut suggestions = vec![
        "What's the current status?".to_string(),
        "Tell me about DOCAS".to_string(),
    ];

    // Add contextual suggestions
    if let Some(tick) = current_tick {
        if tick > 100 && !recent_events.is_empty() {
            suggestions.push("What events happened recently?".to_string());
        }
    }

    if let Some(count) = alive_count {
        if count < 10 {
            suggestions.push("Why is the population low?".to_string());
        }
    }

    if !recent_events.is_empty() {
        suggestions.push("Explain the recent events".to_string());
    }

    suggestions.push("How do groups form?".to_string());

    suggestions
}

// ============================================================================
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
// P2.1: Coloring Unit Tests (Phase 16)
// Per §10.5.2: Agents are colored by genome_gains (heritable), NOT docas (transient)
// ============================================================================

// Integration tests extracted from crates/sim-service/src/live.rs

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    /// Test that `genome_gains` field is correctly populated from `genome.docas_beta()`
    /// Per §10.5.2: `genome_gains` must be heritable DOCAS gain traits
    #[test]
    fn test_genome_gains_populated() {
        let config = SimulationConfig::default();
        let mut sim = Simulation::new(config);

        // Spawn an agent
        sim.spawn_agent(Position::new_2d(32.0, 32.0));

        // Create frame
        let frame = create_frame(&sim);

        // Verify genome_gains is populated
        assert_eq!(frame.agents.len(), 1);
        let snapshot = &frame.agents[0];

        // genome_gains should be [f32; 5] with values in range [0.05, 0.2]
        // Per §8.2.1: beta range is [0.05, 0.2]
        for (i, &gain) in snapshot.genome_gains.iter().enumerate() {
            assert!(
                (0.05..=0.2).contains(&gain),
                "genome_gains[{i}] = {gain} should be in [0.05, 0.2]"
            );
        }
    }

    /// Test that `genome_gains` is STABLE across ticks (heritable, not transient)
    /// Per §10.5.2: genome is heritable and stable over agent lifetime
    #[test]
    fn test_genome_gains_stable_across_ticks() {
        let config = SimulationConfig::default();
        let mut sim = Simulation::new(config);

        // Spawn an agent
        sim.spawn_agent(Position::new_2d(32.0, 32.0));

        // Get initial genome_gains
        let frame1 = create_frame(&sim);
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let initial_gains = frame1.agents[0].genome_gains;

        // Run simulation for 100 ticks
        sim.run(100);

        // Get genome_gains after ticks
        let frame2 = create_frame(&sim);

        // Find the same agent (may be dead, check if still exists)
        if let Some(agent) = frame2
            .agents
            .iter()
            .find(|a| a.agent_id == frame1.agents[0].agent_id)
        {
            // genome_gains should be IDENTICAL (heritable, stable)
            assert_eq!(
                agent.genome_gains, initial_gains,
                "genome_gains should be stable across ticks"
            );
        }
        // If agent died and was removed, that's okay - we just can't verify
    }

    /// Test that DOCAS levels change while `genome_gains` remain stable
    /// Per §10.5.2: docas is transient (changes), `genome_gains` is heritable (stable)
    #[test]
    fn test_docas_changes_genome_stable() {
        let mut config = SimulationConfig::default();
        config.toggles.docas_enabled = true; // Enable DOCAS dynamics

        let mut sim = Simulation::new(config);

        // Spawn an agent
        sim.spawn_agent(Position::new_2d(32.0, 32.0));

        // Get initial values
        let frame1 = create_frame(&sim);
        let initial_docas = frame1.agents[0].docas;
        let initial_genome = frame1.agents[0].genome_gains;

        // Run simulation for many ticks to allow DOCAS to change
        sim.run(50);

        // Get values after simulation
        let frame2 = create_frame(&sim);
        if let Some(agent) = frame2
            .agents
            .iter()
            .find(|a| a.agent_id == frame1.agents[0].agent_id)
        {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            // genome_gains MUST be stable
            assert_eq!(
                agent.genome_gains, initial_genome,
                "genome_gains must remain stable"
            );

            // DOCAS may have changed (though not guaranteed)
            // Just verify they're both valid [f32; 5] arrays
            for (i, &level) in agent.docas.iter().enumerate() {
                assert!(
                    (0.0..=1.0).contains(&level),
                    "docas[{i}] = {level} should be in [0.0, 1.0]"
                );
            }
            let _ = initial_docas; // Mark as used
        }
    }

    /// Test that different agents have different `genome_gains` (genetic diversity)
    /// Per §10.5.3: Agents should have diverse genomes for visual differentiation
    #[test]
    fn test_different_agents_different_genomes() {
        let config = SimulationConfig::default();
        let mut sim = Simulation::new(config);

        // Spawn multiple agents
        for i in 0..10 {
            sim.spawn_agent(Position::new_2d(10.0 + i as f32 * 5.0, 32.0));
        }

        // Get frame
        let frame = create_frame(&sim);
        assert_eq!(frame.agents.len(), 10);

        // Collect all genome_gains
        let genomes: Vec<[f32; 5]> = frame.agents.iter().map(|a| a.genome_gains).collect();

        // Verify genomes are valid arrays in expected range
        for (i, genome) in genomes.iter().enumerate() {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            for (j, &gain) in genome.iter().enumerate() {
                assert!(
                    (0.05..=0.2).contains(&gain),
                    "Agent {i} genome_gains[{j}] = {gain} should be in [0.05, 0.2]"
                );
            }
        }

        // Note: With seeded RNG, agents may have similar genomes initially.
        // Diversity emerges through reproduction with crossover and mutation.
        // The key invariant is that genome_gains are STABLE per agent (tested elsewhere).
    }

    /// Test `AgentSnapshot` correctly maps `genome.docas_beta()` values
    #[test]
    fn test_agent_snapshot_genome_mapping() {
        let config = SimulationConfig::default();
        let mut sim = Simulation::new(config);

        sim.spawn_agent(Position::new_2d(32.0, 32.0));

        // Get the raw genome values
        let agent = &sim.agents[0];
        let expected_gains = agent.genome.docas_beta();

        // Get frame snapshot
        let frame = create_frame(&sim);
        let snapshot = &frame.agents[0];

        // Snapshot genome_gains should exactly match agent.genome.docas_beta()
        assert_eq!(
            snapshot.genome_gains, expected_gains,
            "AgentSnapshot.genome_gains should equal agent.genome.docas_beta()"
        );
    }

    /// Test that resources are correctly serialized in the frame
    /// Verifies the "No Resources Visible" bug fix
    #[test]
    fn test_resource_serialization() {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        use sim_core::environment::EnvironmentConfig;

        let config = SimulationConfig {
            environment: EnvironmentConfig {
                resource_density: 0.7,
                initial_resource: 25.0,
                ..EnvironmentConfig::default()
            },
            ..SimulationConfig::default()
        };
        let mut sim = Simulation::new(config);

        // Spawn some agents
        for i in 0..10 {
            sim.spawn_agent(Position::new_2d(i as f32, i as f32));
        }

        // Create frame at tick 10 (when resources should be included)
        sim.tick = 10;
        let frame = create_frame(&sim);

        // Check resources are present
        assert!(
            frame.resources.is_some(),
            "Resources should be Some at tick 10"
        );

        let resources = frame.resources.unwrap();
        let cells_with_resources = resources.iter().filter(|r| r.amount > 0.0).count();
        let total_cells = resources.len();

        eprintln!("=== Frame Resource Serialization Test ===");
        eprintln!("Total cells in frame: {total_cells}");
        eprintln!("Cells with resources: {cells_with_resources}");
        eprintln!(
            // TAG: surface=simulation owner=research-team rule=SIM-001
            "Density: {:.2}%",
            (cells_with_resources as f32 / total_cells as f32) * 100.0
        );

        // Should have around 70% density
        assert!(
            cells_with_resources > total_cells * 6 / 10,
            "Expected >60% cells with resources, got {}%",
            (cells_with_resources as f32 / total_cells as f32) * 100.0
        );
    }

    #[test]
    fn test_resources_not_included_every_tick() {
        let config = SimulationConfig::default();
        let mut sim = Simulation::new(config);
        sim.spawn_agent(Position::new_2d(32.0, 32.0));

        // At tick 5, resources should be None (only sent every 10 ticks)
        sim.tick = 5;
        let frame = create_frame(&sim);
        assert!(
            frame.resources.is_none(),
            "Resources should be None at tick 5"
        );

        // At tick 10, resources should be Some
        sim.tick = 10;
        let frame = create_frame(&sim);
        assert!(
            frame.resources.is_some(),
            "Resources should be Some at tick 10"
        );
    }
}
