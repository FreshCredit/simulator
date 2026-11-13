//! Era tracking and transition event handling
//!
//! Tracks era transitions and maintains state across simulation phases.

use serde::{Deserialize, Serialize};

use crate::types::Tick;

use super::{SimulationEra, TemporalConfig};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Event raised when simulation era transitions
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EraTransitionEvent {
    /// Tick when transition occurred
    pub tick: Tick,

    /// Era we transitioned from
    pub from_era: SimulationEra,

    /// Era we transitioned to
    pub to_era: SimulationEra,

    /// Number of agents at time of transition
    pub population_size: usize,

    /// Protocol adoption rate at transition
    pub adoption_rate: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Tracks current and historical era state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EraTracker {
    /// Current era
    current_era: SimulationEra,

    /// Era at previous tick
    previous_era: Option<SimulationEra>,

    /// Tick when current era began
    era_start_tick: Tick,

    /// History of era transitions
    transition_history: Vec<EraTransitionEvent>,

    /// Whether this is the first tick of a new era
    is_era_start: bool,
}

impl EraTracker {
    /// Create new era tracker
    #[must_use]
    pub const fn new(start_era: SimulationEra) -> Self {
        Self {
            current_era: start_era,
            previous_era: None,
            era_start_tick: 0,
            transition_history: Vec::new(),
            is_era_start: true,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update tracker with current configuration and tick
    ///
    /// Returns Some(EraTransitionEvent) if a transition occurred
    pub fn update(
        &mut self,
        config: &TemporalConfig,
        current_tick: Tick,
        population_size: usize,
    ) -> Option<EraTransitionEvent> {
        let new_era = config.current_era(current_tick);

        self.previous_era = Some(self.current_era);

        if new_era == self.current_era {
            self.is_era_start = false;
            None
        } else {
            // Era transition occurred
            let event = EraTransitionEvent {
                tick: current_tick,
                from_era: self.current_era,
                to_era: new_era,
                population_size,
                adoption_rate: config.adoption_rate(current_tick),
            };

            self.transition_history.push(event);
            self.current_era = new_era;
            self.era_start_tick = current_tick;
            self.is_era_start = true;

            Some(event)
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        }
    }

    /// Get current era
    #[must_use]
    pub const fn current_era(&self) -> SimulationEra {
        self.current_era
    }

    /// Get previous era (if any)
    #[must_use]
    pub const fn previous_era(&self) -> Option<SimulationEra> {
        self.previous_era
    }

    /// Check if this is the first tick of current era
    #[must_use]
    pub const fn is_era_start(&self) -> bool {
        self.is_era_start
    }

    /// Get tick when current era began
    #[must_use]
    pub const fn era_start_tick(&self) -> Tick {
        self.era_start_tick
    }

    /// Get number of ticks in current era
    #[must_use]
    pub const fn ticks_in_era(&self, current_tick: Tick) -> Tick {
        current_tick - self.era_start_tick
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get transition history
    #[must_use]
    pub fn transition_history(&self) -> &[EraTransitionEvent] {
        &self.transition_history
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if a specific transition has occurred
    #[must_use]
    pub fn has_transitioned(&self, from: SimulationEra, to: SimulationEra) -> bool {
        self.transition_history
            .iter()
            .any(|e| e.from_era == from && e.to_era == to)
    }

    /// Get the most recent transition (if any)
    #[must_use]
    pub fn last_transition(&self) -> Option<&EraTransitionEvent> {
        self.transition_history.last()
    }

    /// Reset tracker to initial state
    pub fn reset(&mut self, start_era: SimulationEra) {
        self.current_era = start_era;
        self.previous_era = None;
        self.era_start_tick = 0;
        self.transition_history.clear();
        self.is_era_start = true;
    }
}

impl Default for EraTracker {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    fn default() -> Self {
        Self::new(SimulationEra::PreProtocol)
    }
}

/// Summary statistics for a completed era
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EraSummary {
    /// Era being summarized
    pub era: SimulationEra,

    /// Duration in ticks
    pub duration: Tick,

    /// Starting population
    pub start_population: usize,

    /// Ending population
    pub end_population: usize,

    /// Number of births during era
    pub births: usize,

    /// Number of deaths during era
    pub deaths: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Builder for era summaries
#[derive(Debug, Clone)]
pub struct EraSummaryBuilder {
    era: SimulationEra,
    start_tick: Tick,
    start_population: usize,
    births: usize,
    deaths: usize,
}

impl EraSummaryBuilder {
    /// Start building summary for an era
    #[must_use]
    pub const fn start(era: SimulationEra, tick: Tick, population: usize) -> Self {
        Self {
            era,
            start_tick: tick,
            start_population: population,
            births: 0,
            deaths: 0,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Record a birth
    pub fn record_birth(&mut self) {
        self.births += 1;
    }

    /// Record a death
    pub fn record_death(&mut self) {
        self.deaths += 1;
    }

    /// Build final summary
    #[must_use]
    pub const fn finish(self, end_tick: Tick, end_population: usize) -> EraSummary {
        EraSummary {
            era: self.era,
            duration: end_tick - self.start_tick,
            start_population: self.start_population,
            end_population,
            births: self.births,
            deaths: self.deaths,
        }
    }
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
#[cfg(test)]
mod tests {
    use super::*;
    use crate::temporal::TemporalConfig;

    #[test]
    fn test_era_tracker_basic() {
        let mut tracker = EraTracker::new(SimulationEra::PreProtocol);
        let config = TemporalConfig::dynamic_introduction(100, 100);

        // Initially in PreProtocol
        assert_eq!(tracker.current_era(), SimulationEra::PreProtocol);
        assert!(tracker.is_era_start());

        // Still in PreProtocol at tick 50
        let event = tracker.update(&config, 50, 100);
        assert!(event.is_none());
        assert_eq!(tracker.current_era(), SimulationEra::PreProtocol);
        assert!(!tracker.is_era_start());

        // Transition to Transition at tick 100
        let event = tracker.update(&config, 100, 100);
        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.from_era, SimulationEra::PreProtocol);
        assert_eq!(event.to_era, SimulationEra::Transition);
        assert_eq!(event.tick, 100);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(tracker.current_era(), SimulationEra::Transition);
        assert!(tracker.is_era_start());
        assert_eq!(tracker.ticks_in_era(150), 50);

        // Transition to PostProtocol at tick 200
        let event = tracker.update(&config, 200, 100);
        assert!(event.is_some());
        assert_eq!(event.unwrap().to_era, SimulationEra::PostProtocol);
    }

    #[test]
    fn test_transition_history() {
        let mut tracker = EraTracker::new(SimulationEra::PreProtocol);
        let config = TemporalConfig::dynamic_introduction(100, 100);

        // No transitions yet
        assert!(tracker.transition_history().is_empty());
        assert!(!tracker.has_transitioned(SimulationEra::PreProtocol, SimulationEra::Transition));

        // Two transitions
        tracker.update(&config, 100, 100);
        tracker.update(&config, 200, 100);

        assert_eq!(tracker.transition_history().len(), 2);
        assert!(tracker.has_transitioned(SimulationEra::PreProtocol, SimulationEra::Transition));
        assert!(tracker.has_transitioned(SimulationEra::Transition, SimulationEra::PostProtocol));

        let last = tracker.last_transition().unwrap();
        assert_eq!(last.to_era, SimulationEra::PostProtocol);
    }

    #[test]
    fn test_static_config_no_transitions() {
        let mut tracker = EraTracker::new(SimulationEra::PreProtocol);
        let config = TemporalConfig::static_protocol(false);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        tracker.update(&config, 0, 100);
        tracker.update(&config, 1000, 100);
        tracker.update(&config, 10000, 100);

        assert!(tracker.transition_history().is_empty());
        assert_eq!(tracker.current_era(), SimulationEra::PreProtocol);
    }

    #[test]
    fn test_summary_builder() {
        let mut builder = EraSummaryBuilder::start(SimulationEra::PreProtocol, 0, 100);

        // Simulate some events
        for _ in 0..10 {
            builder.record_birth();
        }
        for _ in 0..5 {
            builder.record_death();
        }

        let summary = builder.finish(500, 105);

        assert_eq!(summary.era, SimulationEra::PreProtocol);
        assert_eq!(summary.duration, 500);
        assert_eq!(summary.start_population, 100);
        assert_eq!(summary.end_population, 105);
        assert_eq!(summary.births, 10);
        assert_eq!(summary.deaths, 5);
    }
}
