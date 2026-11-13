use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::identity::IdentityTheftStats;
use crate::types::{AgentId, Tick};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Memory usage statistics for the simulation
///
/// Tracks memory consumption by major components to help
/// diagnose memory issues and optimize resource usage.
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    /// Memory used by agents vector (bytes)
    pub agents_bytes: usize,
    /// Memory used by lineage tree (bytes)
    pub lineage_bytes: usize,
    /// Memory used by environment (bytes)
    pub environment_bytes: usize,
    /// Total memory estimate (bytes)
    pub total_bytes: usize,
    /// Number of alive agents
    pub alive_count: usize,
    /// Number of dead agents (not yet cleaned)
    pub dead_count: usize,
    /// Number of agents in lineage summaries
    pub summary_count: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Simulation metrics and statistics collected during run
///
/// Aggregates key performance indicators and system statistics
/// for analysis, validation, and monitoring of simulation health.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SimulationMetrics {
    /// Identity theft statistics (SIM-010)
    pub identity_theft_stats: Option<IdentityTheftStats>,
    /// Cooperation rate history
    pub cooperation_history: Vec<f32>,
    /// Mean trust (τ) history
    pub trust_history: Vec<f32>,
    /// Population size history
    pub population_history: Vec<usize>,
    /// Trade volume this tick (sum of debt amounts created)
    pub trade_volume_this_tick: f32,
    /// Disputes this tick (negative interactions from games)
    pub disputes_this_tick: u32,
    /// Encounters this tick (total agent interactions)
    pub encounters_this_tick: u32,
    /// Births this tick
    pub births_this_tick: u32,
    /// Deaths this tick
    pub deaths_this_tick: u32,
    /// Total events caused (for branching ratio calculation)
    pub events_caused: u32,
    /// Total events received (for branching ratio calculation)
    pub events_received: u32,
    /// Allee effect critical warning flag
    pub allee_critical_warning: bool,
    /// Current Allee fitness multiplier
    pub allee_fitness_multiplier: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Persistent event store for simulation analysis
///
/// Maintains a ring buffer of events with efficient querying.
/// Automatically prunes old events when capacity is exceeded.
#[derive(Debug, Clone)]
pub struct EventStore {
    /// All persisted game events (ring buffer, max 100K events)
    pub game_events: Vec<GameEvent>,
    /// All persisted encounter events (ring buffer, max 100K events)
    pub encounter_events: Vec<EncounterEvent>,
    /// Birth events (`agent_id`, tick, `parent_ids`) - `VecDeque` for O(1) ring buffer ops
    pub birth_events: VecDeque<(AgentId, Tick, Option<AgentId>, Option<AgentId>)>,
    /// Death events (`agent_id`, tick, cause) - `VecDeque` for O(1) ring buffer ops
    pub death_events: VecDeque<(AgentId, Tick, String)>,
    /// Group formation events (`group_id`, tick, `founder_ids`)
    pub group_formation_events: Vec<(u64, Tick, Vec<AgentId>)>,
    /// Maximum capacity before ring buffer wraps
    pub max_capacity: usize,
}

impl EventStore {
    /// Create new event store with default capacity
    #[must_use]
    pub fn new() -> Self {
        Self {
            game_events: Vec::with_capacity(10000),
            encounter_events: Vec::with_capacity(10000),
            birth_events: VecDeque::with_capacity(5000),
            death_events: VecDeque::with_capacity(5000),
            group_formation_events: Vec::with_capacity(1000),
            max_capacity: 100000,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Persist game events from current tick
    pub fn persist_game_events(&mut self, events: &[GameEvent]) {
        for event in events {
            if self.game_events.len() >= self.max_capacity {
                // Ring buffer: remove oldest 10% when full
                let remove_count = self.max_capacity / 10;
                self.game_events.drain(0..remove_count);
            }
            self.game_events.push(event.clone());
        }
    }

    /// Persist encounter events from current tick
    pub fn persist_encounter_events(&mut self, events: &[EncounterEvent]) {
        for event in events {
            if self.encounter_events.len() >= self.max_capacity {
                let remove_count = self.max_capacity / 10;
                self.encounter_events.drain(0..remove_count);
            }
            self.encounter_events.push(event.clone());
        }
    }

    /// Record a birth event
    pub fn record_birth(
        &mut self,
        agent_id: AgentId,
        tick: Tick,
        parent1: Option<AgentId>,
        parent2: Option<AgentId>,
    ) {
        if self.birth_events.len() >= self.max_capacity {
            self.birth_events.pop_front();
        }
        self.birth_events
            .push_back((agent_id, tick, parent1, parent2));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Record a death event
    pub fn record_death(&mut self, agent_id: AgentId, tick: Tick, cause: &str) {
        if self.death_events.len() >= self.max_capacity {
            self.death_events.pop_front();
        }
        self.death_events
            .push_back((agent_id, tick, cause.to_string()));
    }

    /// Record a group formation event
    pub fn record_group_formation(&mut self, group_id: u64, tick: Tick, founders: Vec<AgentId>) {
        if self.group_formation_events.len() >= 10000 {
            self.group_formation_events.remove(0);
        }
        self.group_formation_events.push((group_id, tick, founders));
    }

    /// Get events within a tick range
    #[must_use]
    pub fn get_game_events_in_range(&self, start_tick: Tick, end_tick: Tick) -> Vec<&GameEvent> {
        self.game_events
            .iter()
            .filter(|e| e.tick >= start_tick && e.tick <= end_tick)
            .collect()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get events for a specific agent
    #[must_use]
    pub fn get_agent_events(&self, agent_id: AgentId) -> Vec<&GameEvent> {
        self.game_events
            .iter()
            .filter(|e| e.agent_a == agent_id || e.agent_b == agent_id)
            .collect()
    }

    /// Get summary statistics
    #[must_use]
    pub fn get_summary(&self) -> EventSummary {
        EventSummary {
            total_game_events: self.game_events.len(),
            total_encounter_events: self.encounter_events.len(),
            total_births: self.birth_events.len(),
            total_deaths: self.death_events.len(),
            total_group_formations: self.group_formation_events.len(),
        }
    }

    /// Generate comprehensive report for AI analysis
    #[must_use]
    pub fn generate_report(&self, max_events: usize) -> SimulationReport {
        SimulationReport {
            summary: self.get_summary(),
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            recent_game_events: self
                .game_events
                .iter()
                .rev()
                .take(max_events)
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect(),
            recent_encounters: self
                .encounter_events
                .iter()
                .rev()
                .take(max_events)
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect(),
            births: self.birth_events.iter().copied().collect(),
            deaths: self.death_events.iter().cloned().collect(),
            group_formations: self.group_formation_events.clone(),
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for EventStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of events in the simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSummary {
    /// total game events.
    pub total_game_events: usize,
    /// total encounter events.
    pub total_encounter_events: usize,
    /// total births.
    pub total_births: usize,
    /// total deaths.
    pub total_deaths: usize,
    /// total group formations.
    pub total_group_formations: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Comprehensive simulation report for AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationReport {
    /// summary.
    pub summary: EventSummary,
    /// recent game events.
    pub recent_game_events: Vec<GameEvent>,
    /// recent encounters.
    pub recent_encounters: Vec<EncounterEvent>,
    /// births.
    pub births: Vec<(AgentId, Tick, Option<AgentId>, Option<AgentId>)>,
    /// deaths.
    pub deaths: Vec<(AgentId, Tick, String)>,
    /// group formations.
    pub group_formations: Vec<(u64, Tick, Vec<AgentId>)>,
}

/// Game event for activity log streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    /// tick.
    pub tick: Tick,
    /// agent a.
    pub agent_a: AgentId,
    /// agent b.
    pub agent_b: AgentId,
    /// game type.
    pub game_type: String,
    /// outcome.
    pub outcome: String,
    /// payoff a.
    pub payoff_a: f32,
    /// payoff b.
    pub payoff_b: f32,
}

/// Encounter event for activity log streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterEvent {
    /// tick.
    pub tick: Tick,
    /// agent a.
    pub agent_a: AgentId,
    /// agent b.
    pub agent_b: AgentId,
    /// encounter type.
    pub encounter_type: String,
}
