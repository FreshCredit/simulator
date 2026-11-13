use tracing::debug;

use crate::agent::Agent;
use crate::types::{AgentId, Position, Tick};

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Clone)]
/// simulation debug log.
pub struct SimulationDebugLog {
    /// tick.
    pub tick: Tick,
    /// event type.
    pub event_type: String,
    /// agent id.
    pub agent_id: Option<AgentId>,
    /// details.
    pub details: String,
    /// timestamp.
    pub timestamp: std::time::SystemTime,
}

/// Debug state for tracking simulation flow
#[derive(Debug, Clone)]
pub struct SimulationDebugState {
    /// last positions.
    pub last_positions: std::collections::HashMap<AgentId, Position>,
    /// agent movement log.
    pub agent_movement_log: std::collections::HashMap<AgentId, Vec<(Tick, Position, f32)>>, // (tick, pos, energy)
    /// recent logs.
    pub recent_logs: Vec<SimulationDebugLog>,
    /// max log entries.
    pub max_log_entries: usize,
    /// total reproductions.
    pub total_reproductions: u64,
    /// total deaths.
    pub total_deaths: u64,
    /// sampled agent id.
    pub sampled_agent_id: Option<AgentId>, // One agent per tick to log in detail
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for SimulationDebugState {
    fn default() -> Self {
        Self {
            last_positions: std::collections::HashMap::new(),
            agent_movement_log: std::collections::HashMap::new(),
            recent_logs: Vec::with_capacity(1000),
            max_log_entries: 1000,
            total_reproductions: 0,
            total_deaths: 0,
            sampled_agent_id: None,
        }
    }
}

impl SimulationDebugState {
    /// Log a debug event
    pub fn log(&mut self, tick: Tick, event_type: &str, agent_id: Option<AgentId>, details: &str) {
        let log_entry = SimulationDebugLog {
            tick,
            event_type: event_type.to_string(),
            agent_id,
            details: details.to_string(),
            timestamp: std::time::SystemTime::now(),
        };

        // Also print to stderr for real-time debugging
        if let Some(aid) = agent_id {
            debug!(" [{event_type}] tick={tick} agent={aid} | {details}");
        } else {
            debug!(" [{event_type}] tick={tick} | {details}");
        }

        self.recent_logs.push(log_entry);
        if self.recent_logs.len() > self.max_log_entries {
            self.recent_logs.remove(0);
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Record agent position for movement tracking
    pub fn record_position(
        &mut self,
        tick: Tick,
        agent_id: AgentId,
        position: Position,
        energy: f32,
    ) {
        self.agent_movement_log
            .entry(agent_id)
            .or_default()
            .push((tick, position, energy));

        // Keep only last 100 positions per agent
        if let Some(log) = self.agent_movement_log.get_mut(&agent_id) {
            if log.len() > 100 {
                log.remove(0);
            }
        }

        self.last_positions.insert(agent_id, position);

        if self.sampled_agent_id == Some(agent_id) {
            self.log(
                tick,
                "sampled_movement",
                Some(agent_id),
                "Recorded sampled agent trajectory step",
            );
        }
    }

    /// Get movement history for an agent
    #[must_use]
    pub fn get_agent_history(&self, agent_id: AgentId) -> Option<&Vec<(Tick, Position, f32)>> {
        self.agent_movement_log.get(&agent_id)
    }

    /// Select a random agent to sample for detailed logging
    pub fn select_sampled_agent(&mut self, agents: &[Agent]) {
        let alive_agents: Vec<AgentId> = agents.iter().filter(|a| a.alive).map(|a| a.id).collect();

        if !alive_agents.is_empty() {
            // Pick a consistent agent if possible, otherwise random
            self.sampled_agent_id = Some(alive_agents[alive_agents.len() / 2]);
        }
    }
}
