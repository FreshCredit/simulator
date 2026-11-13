//! Temporal metrics tracking: Volatility, damage events, and hypothesis validation
//!
//! Tracks metrics required for testing temporal dynamics hypotheses:
//! - Reputation volatility (standard deviation of reputation changes)
//! - False damage events (Type I/II errors in reputation damage)
//! - Network propagation patterns

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use crate::types::{AgentId, Tick};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Tracks reputation volatility over time
///
/// Volatility is calculated as the standard deviation of reputation changes
/// within a sliding window. This enables testing the hypothesis that
/// reputation volatility declines after protocol introduction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityTracker {
    /// Window size for calculation (in ticks)
    window_size: usize,

    /// Historical reputation samples per agent
    /// Stores (tick, `reputation_value`) pairs
    reputation_history: HashMap<AgentId, VecDeque<(Tick, f32)>>,

    /// Running volatility per agent
    current_volatility: HashMap<AgentId, f32>,

    /// Population-average volatility history
    population_volatility_history: Vec<(Tick, f32)>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl VolatilityTracker {
    /// Create new volatility tracker
    ///
    /// # Arguments
    /// * `window_size` - Number of ticks to include in volatility calculation
    #[must_use]
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            reputation_history: HashMap::new(),
            current_volatility: HashMap::new(),
            population_volatility_history: Vec::new(),
        }
    }

    /// Update tracker with current agent reputations
    ///
    /// Call this every tick to maintain reputation history
    pub fn update(&mut self, tick: Tick, agents: &[(AgentId, f32)]) {
        // Update individual agent histories
        for (agent_id, reputation) in agents {
            let history = self
                .reputation_history
                .entry(*agent_id)
                .or_insert_with(|| VecDeque::with_capacity(self.window_size + 1));

            history.push_back((tick, *reputation));
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001

            // Maintain window size
            while history.len() > self.window_size {
                history.pop_front();
            }

            // Recalculate volatility for this agent
            if history.len() >= 2 {
                let volatility = Self::calculate_volatility_from_history(history);
                self.current_volatility.insert(*agent_id, volatility);
            }
        }

        // Update population average
        let avg_volatility = self.population_volatility();
        self.population_volatility_history
            .push((tick, avg_volatility));
    }

    /// Calculate volatility from reputation history
    ///
    /// Volatility = standard deviation of reputation changes
    fn calculate_volatility_from_history(history: &VecDeque<(Tick, f32)>) -> f32 {
        if history.len() < 2 {
            return 0.0;
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Calculate changes between consecutive samples
        // VecDeque doesn't have windows(), so iterate manually
        let mut changes = Vec::with_capacity(history.len() - 1);
        for i in 1..history.len() {
            let prev = history.get(i - 1).map_or(&0.0, |(_, r)| r);
            let curr = history.get(i).map_or(&0.0, |(_, r)| r);
            changes.push(curr - prev);
        }

        if changes.is_empty() {
            return 0.0;
        }

        // Calculate standard deviation
        let mean = changes.iter().sum::<f32>() / changes.len() as f32;
        let variance =
            changes.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / changes.len() as f32;

        variance.sqrt()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get current volatility for a specific agent
    #[must_use]
    pub fn agent_volatility(&self, agent_id: AgentId) -> f32 {
        self.current_volatility
            .get(&agent_id)
            .copied()
            .unwrap_or(0.0)
    }

    /// Get population-average volatility
    #[must_use]
    pub fn population_volatility(&self) -> f32 {
        if self.current_volatility.is_empty() {
            return 0.0;
        }

        let sum: f32 = self.current_volatility.values().sum();
        sum / self.current_volatility.len() as f32
    }

    /// Get volatility history for population
    #[must_use]
    pub fn volatility_history(&self) -> &[(Tick, f32)] {
        &self.population_volatility_history
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate volatility change between two time periods
    ///
    /// Returns (`pre_volatility`, `post_volatility`, `percent_change`)
    #[must_use]
    pub fn compare_periods(&self, pre_ticks: &[Tick], post_ticks: &[Tick]) -> (f32, f32, f32) {
        let pre_vol = self.average_volatility_over_ticks(pre_ticks);
        let post_vol = self.average_volatility_over_ticks(post_ticks);

        let percent_change = if pre_vol > 0.0 {
            ((post_vol - pre_vol) / pre_vol) * 100.0
        } else {
            0.0
        };

        (pre_vol, post_vol, percent_change)
    }

    /// Average volatility over specific ticks
    fn average_volatility_over_ticks(&self, ticks: &[Tick]) -> f32 {
        let volatilities: Vec<f32> = ticks
            .iter()
            .filter_map(|tick| {
                self.population_volatility_history
                    .iter()
                    .find(|(t, _)| t == tick)
                    .map(|(_, v)| *v)
            })
            .collect();

        if volatilities.is_empty() {
            0.0
        } else {
            volatilities.iter().sum::<f32>() / volatilities.len() as f32
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get agents with highest volatility
    #[must_use]
    pub fn highest_volatility_agents(&self, n: usize) -> Vec<(AgentId, f32)> {
        let mut agents: Vec<(AgentId, f32)> = self
            .current_volatility
            .iter()
            .map(|(id, vol)| (*id, *vol))
            .collect();

        agents.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        agents.truncate(n);
        agents
    }

    /// Clear all history (for era transitions)
    pub fn clear(&mut self) {
        self.reputation_history.clear();
        self.current_volatility.clear();
        // Don't clear population history - we want cross-era comparisons
    }
}

impl Default for VolatilityTracker {
    fn default() -> Self {
        Self::new(100) // Default 100-tick window
    }
}

/// Tracks damage events for false positive/negative analysis
///
/// Categorizes reputation damage as:
/// - True Positive: Correctly identified defection
/// - False Positive: Reputation damaged without actual defection
/// - True Negative: Correctly maintained reputation
/// - False Negative: No damage despite actual defection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DamageEventType {
    /// Reputation damage occurred after actual defection
    TruePositive,
    /// Reputation damage occurred without defection
    FalsePositive,
    /// No reputation damage, no defection
    TrueNegative,
    /// No reputation damage despite defection
    FalseNegative,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl DamageEventType {
    /// Is this a positive prediction (damage occurred)?
    #[must_use]
    pub const fn is_positive(&self) -> bool {
        matches!(self, Self::TruePositive | Self::FalsePositive)
    }

    /// Is this a true prediction (correct)?
    #[must_use]
    pub const fn is_true(&self) -> bool {
        matches!(self, Self::TruePositive | Self::TrueNegative)
    }
}

/// A single damage event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageEvent {
    /// Tick when event occurred
    pub tick: Tick,

    /// Agent who was damaged (or not)
    pub agent_id: AgentId,

    /// Type of damage event
    pub event_type: DamageEventType,

    /// Reputation change amount
    pub reputation_change: f32,

    /// Whether defection actually occurred
    pub actual_defection: bool,

    /// Context (game type, encounter details)
    pub context: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Tracks damage events and calculates false positive/negative rates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageTracker {
    /// All damage events
    events: Vec<DamageEvent>,

    /// Count by event type
    counts: HashMap<DamageEventType, usize>,

    /// Events by era (for temporal analysis)
    events_by_era: HashMap<String, Vec<DamageEvent>>,

    /// Window size for rolling rates
    window_size: usize,

    /// Current window events (for rolling calculation)
    current_window: VecDeque<DamageEvent>,
}

impl DamageTracker {
    /// Create new damage tracker
    #[must_use]
    pub fn new(window_size: usize) -> Self {
        let mut counts = HashMap::new();
        counts.insert(DamageEventType::TruePositive, 0);
        counts.insert(DamageEventType::FalsePositive, 0);
        counts.insert(DamageEventType::TrueNegative, 0);
        counts.insert(DamageEventType::FalseNegative, 0);

        Self {
            events: Vec::new(),
            counts,
            events_by_era: HashMap::new(),
            window_size,
            current_window: VecDeque::new(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Record a damage event
    pub fn record_event(
        &mut self,
        tick: Tick,
        agent_id: AgentId,
        event_type: DamageEventType,
        reputation_change: f32,
        actual_defection: bool,
        context: impl Into<String>,
    ) {
        let event = DamageEvent {
            tick,
            agent_id,
            event_type,
            reputation_change,
            actual_defection,
            context: context.into(),
        };

        // Add to main list
        self.events.push(event.clone());

        // Update counts
        *self.counts.entry(event_type).or_insert(0) += 1;

        // Add to rolling window
        self.current_window.push_back(event);
        while self.current_window.len() > self.window_size {
            self.current_window.pop_front();
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Record an event with era context
    #[allow(clippy::too_many_arguments)]
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn record_event_in_era(
        &mut self,
        tick: Tick,
        agent_id: AgentId,
        event_type: DamageEventType,
        reputation_change: f32,
        actual_defection: bool,
        context: impl Into<String>,
        era: &str,
    ) {
        self.record_event(
            tick,
            agent_id,
            event_type,
            reputation_change,
            actual_defection,
            context,
        );

        // Also add to era-specific list
        if let Some(last_event) = self.events.last().cloned() {
            self.events_by_era
                .entry(era.to_string())
                .or_default()
                .push(last_event);
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get total count by event type
    #[must_use]
    pub fn count(&self, event_type: DamageEventType) -> usize {
        self.counts.get(&event_type).copied().unwrap_or(0)
    }

    /// Get false positive rate
    ///
    /// FPR = False Positives / (False Positives + True Negatives)
    #[must_use]
    pub fn false_positive_rate(&self) -> f32 {
        let fp = self.count(DamageEventType::FalsePositive) as f32;
        let tn = self.count(DamageEventType::TrueNegative) as f32;

        if fp + tn > 0.0 {
            fp / (fp + tn)
        } else {
            0.0
        }
    }

    /// Get false negative rate
    ///
    /// FNR = False Negatives / (False Negatives + True Positives)
    #[must_use]
    pub fn false_negative_rate(&self) -> f32 {
        let fn_count = self.count(DamageEventType::FalseNegative) as f32;
        let tp = self.count(DamageEventType::TruePositive) as f32;

        if fn_count + tp > 0.0 {
            fn_count / (fn_count + tp)
        } else {
            0.0
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get rolling false positive rate (over current window)
    #[must_use]
    pub fn rolling_false_positive_rate(&self) -> f32 {
        let fp = self
            .current_window
            .iter()
            .filter(|e| e.event_type == DamageEventType::FalsePositive)
            .count() as f32;
        let tn = self
            .current_window
            .iter()
            .filter(|e| e.event_type == DamageEventType::TrueNegative)
            .count() as f32;

        if fp + tn > 0.0 {
            fp / (fp + tn)
        } else {
            0.0
        }
    }

    /// Get all events
    #[must_use]
    pub fn events(&self) -> &[DamageEvent] {
        &self.events
    }

    /// Get events for a specific era
    #[must_use]
    pub fn era_events(&self, era: &str) -> &[DamageEvent] {
        self.events_by_era
            .get(era)
            .map_or(&[], std::vec::Vec::as_slice)
    }

    /// Get total damage events (positives)
    #[must_use]
    pub fn total_damage_events(&self) -> usize {
        self.count(DamageEventType::TruePositive) + self.count(DamageEventType::FalsePositive)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get damage rate per tick
    #[must_use]
    pub fn damage_rate(&self, total_ticks: Tick) -> f32 {
        if total_ticks == 0 {
            return 0.0;
        }
        self.total_damage_events() as f32 / total_ticks as f32
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
        self.counts.clear();
        self.counts.insert(DamageEventType::TruePositive, 0);
        self.counts.insert(DamageEventType::FalsePositive, 0);
        self.counts.insert(DamageEventType::TrueNegative, 0);
        self.counts.insert(DamageEventType::FalseNegative, 0);
        self.current_window.clear();
    }

    /// Get summary statistics
    #[must_use]
    pub fn summary(&self) -> DamageSummary {
        let total = self.events.len();
        let tp = self.count(DamageEventType::TruePositive);
        let fp = self.count(DamageEventType::FalsePositive);
        let tn = self.count(DamageEventType::TrueNegative);
        let fn_count = self.count(DamageEventType::FalseNegative);

        DamageSummary {
            total_events: total,
            true_positives: tp,
            false_positives: fp,
            true_negatives: tn,
            false_negatives: fn_count,
            false_positive_rate: self.false_positive_rate(),
            false_negative_rate: self.false_negative_rate(),
            accuracy: if total > 0 {
                (tp + tn) as f32 / total as f32
            } else {
                0.0
            },
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for DamageTracker {
    fn default() -> Self {
        Self::new(1000)
    }
}

/// Summary statistics for damage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageSummary {
    /// total events.
    pub total_events: usize,
    /// true positives.
    pub true_positives: usize,
    /// false positives.
    pub false_positives: usize,
    /// true negatives.
    pub true_negatives: usize,
    /// false negatives.
    pub false_negatives: usize,
    /// false positive rate.
    pub false_positive_rate: f32,
    /// false negative rate.
    pub false_negative_rate: f32,
    /// accuracy.
    pub accuracy: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Combined temporal metrics collector
///
/// Integrates volatility and damage tracking for hypothesis testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalMetrics {
    /// volatility.
    pub volatility: VolatilityTracker,
    /// damage.
    pub damage: DamageTracker,
}

impl TemporalMetrics {
    /// Create new temporal metrics collector
    #[must_use]
    pub fn new(volatility_window: usize, damage_window: usize) -> Self {
        Self {
            volatility: VolatilityTracker::new(volatility_window),
            damage: DamageTracker::new(damage_window),
        }
    }

    /// Update metrics for current tick
    pub fn update(&mut self, tick: Tick, agents: &[(AgentId, f32)]) {
        self.volatility.update(tick, agents);
    }

    /// Get summary for hypothesis validation
    #[must_use]
    pub fn hypothesis_summary(&self) -> HypothesisSummary {
        HypothesisSummary {
            current_volatility: self.volatility.population_volatility(),
            total_damage_events: self.damage.total_damage_events(),
            false_positive_rate: self.damage.false_positive_rate(),
            false_negative_rate: self.damage.false_negative_rate(),
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for TemporalMetrics {
    fn default() -> Self {
        Self::new(100, 1000)
    }
}

/// Summary for hypothesis validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisSummary {
    /// current volatility.
    pub current_volatility: f32,
    /// total damage events.
    pub total_damage_events: usize,
    /// false positive rate.
    pub false_positive_rate: f32,
    /// false negative rate.
    pub false_negative_rate: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volatility_tracker_basic() {
        let mut tracker = VolatilityTracker::new(10);

        // Simulate reputation changes for one agent
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let agent_id: AgentId = 1;
        for tick in 0..20 {
            let reputation = if tick % 2 == 0 { 0.5 } else { 0.6 };
            tracker.update(tick, &[(agent_id, reputation)]);
        }

        // Should have volatility from alternating values
        let vol = tracker.agent_volatility(agent_id);
        assert!(vol > 0.0, "Should have non-zero volatility");

        // Population volatility should be same as single agent
        assert_eq!(tracker.population_volatility(), vol);
    }

    #[test]
    fn test_damage_tracker_rates() {
        let mut tracker = DamageTracker::new(100);

        // Add some events
        for i in 0..10 {
            tracker.record_event(
                i as Tick,
                i as AgentId,
                DamageEventType::TruePositive,
                -0.1,
                true,
                "defection",
            );
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        for i in 10..15 {
            tracker.record_event(
                i as Tick,
                i as AgentId,
                DamageEventType::FalsePositive,
                -0.1,
                false,
                "error",
            );
        }

        // Check counts
        assert_eq!(tracker.count(DamageEventType::TruePositive), 10);
        assert_eq!(tracker.count(DamageEventType::FalsePositive), 5);

        // Check rates
        let summary = tracker.summary();
        assert!(summary.false_positive_rate > 0.0);
        assert!(summary.accuracy > 0.0);
    }

    #[test]
    fn test_compare_periods() {
        let mut tracker = VolatilityTracker::new(5);

        // First period: high volatility
        for tick in 0..10 {
            let reputation = if tick % 2 == 0 { 0.3 } else { 0.7 };
            tracker.update(tick, &[(1, reputation)]);
        }

        // Second period: low volatility
        for tick in 10..20 {
            tracker.update(tick, &[(1, 0.5)]);
        }

        let (pre, post, change) =
            tracker.compare_periods(&(0..10).collect::<Vec<_>>(), &(10..20).collect::<Vec<_>>());

        assert!(pre > post, "Pre-volatility should be higher");
        assert!(change < 0.0, "Should show negative change (reduction)");
    }
}
