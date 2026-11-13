//! Temporal configuration for protocol introduction studies
//!
//! Per `RESEARCH_TEMPORAL_DYNAMICS.md` Section 1.4

use serde::{Deserialize, Serialize};

use crate::types::Tick;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Simulation era identifier
///
/// Represents temporal phase of society in relation to protocol introduction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SimulationEra {
    /// Pre-protocol: reputation-only society
    /// Agents rely on local reputation networks and kin-based trust
    #[default]
    PreProtocol,

    /// Transition: protocol being adopted
    /// Mixed society with S-curve adoption dynamics
    Transition,

    /// Post-protocol: mature protocol society
    /// Global verification infrastructure established
    PostProtocol,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl SimulationEra {
    /// Get display name for the era
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::PreProtocol => "Pre-Protocol",
            Self::Transition => "Transition",
            Self::PostProtocol => "Post-Protocol",
        }
    }

    /// Check if protocol is available in this era
    #[must_use]
    pub const fn protocol_available(&self) -> bool {
        matches!(self, Self::Transition | Self::PostProtocol)
    }

    /// Check if this is a stable era (not transition)
    #[must_use]
    pub const fn is_stable(&self) -> bool {
        matches!(self, Self::PreProtocol | Self::PostProtocol)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Temporal configuration for protocol introduction studies
///
/// Enables simulation of protocol introduction at arbitrary ticks with
/// S-curve adoption dynamics following the Bass diffusion model.
///
/// # Example
///
/// ```
/// use sim_core::temporal::{TemporalConfig, SimulationEra};
///
/// // Static configuration (protocol always off)
/// let static_off = TemporalConfig::static_protocol(false);
/// debug_assert_eq!(static_off.current_era(100), SimulationEra::PreProtocol);
///
/// // Dynamic introduction at tick 1000 with 500-tick transition
/// let dynamic = TemporalConfig::dynamic_introduction(1000, 500);
/// debug_assert_eq!(dynamic.current_era(500), SimulationEra::PreProtocol);
/// debug_assert_eq!(dynamic.current_era(1200), SimulationEra::Transition);
/// debug_assert_eq!(dynamic.current_era(1600), SimulationEra::PostProtocol);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalConfig {
    /// Tick when protocol is introduced
    /// If None, protocol is either always on or always off (static toggle)
    pub protocol_introduction_tick: Option<Tick>,

    /// Duration of transition period in ticks
    /// During this period, adoption follows S-curve
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub transition_duration: Tick,

    /// Bass diffusion innovation coefficient (p)
    /// Represents adoption driven by external influence (advertising, policy)
    /// Range: [0.01, 0.10], default: 0.03
    pub adoption_innovation_coeff: f32,

    /// Bass diffusion imitation coefficient (q)
    /// Represents adoption driven by internal influence (peer effects)
    /// Range: [0.20, 0.50], default: 0.38
    pub adoption_imitation_coeff: f32,

    /// Target protocol adoption rate at transition end
    /// Fraction of agents expected to adopt protocol
    /// Range: [0.50, 1.0], default: 0.90
    pub target_adoption_rate: f32,

    /// Whether to enable era-aware memory
    /// Agents will weight pre/post protocol experiences differently
    pub era_aware_memory: bool,

    /// Window size for volatility calculation (in ticks)
    pub volatility_window: usize,

    /// Whether to track damage event categorization
    pub track_damage_events: bool,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl TemporalConfig {
    /// Create static configuration (no temporal dynamics)
    ///
    /// # Arguments
    /// * `on` - If true, protocol is always on; if false, always off
    ///
    /// # Example
    /// ```
    /// use sim_core::temporal::TemporalConfig;
    ///
    /// let config = TemporalConfig::static_protocol(true);
    /// debug_assert_eq!(config.target_adoption_rate, 1.0);
    /// ```
    #[must_use]
    pub const fn static_protocol(on: bool) -> Self {
        Self {
            protocol_introduction_tick: None,
            transition_duration: 0,
            adoption_innovation_coeff: 0.0,
            adoption_imitation_coeff: 0.0,
            target_adoption_rate: if on { 1.0 } else { 0.0 },
            era_aware_memory: false,
            volatility_window: 100,
            track_damage_events: false,
        }
    }

    /// Create dynamic protocol introduction configuration
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    ///
    /// Uses Bass diffusion model (Bass, 1969) with default parameters
    /// from Rai & `McAndrews` (2015) solar PV adoption study.
    ///
    /// # Arguments
    /// * `intro_tick` - Tick when protocol introduction begins
    /// * `transition_ticks` - Duration of transition period
    ///
    /// # Example
    /// ```
    /// use sim_core::temporal::TemporalConfig;
    ///
    /// let config = TemporalConfig::dynamic_introduction(1000, 500);
    /// debug_assert_eq!(config.protocol_introduction_tick, Some(1000));
    /// debug_assert_eq!(config.transition_duration, 500);
    /// ```
    #[must_use]
    pub const fn dynamic_introduction(intro_tick: Tick, transition_ticks: Tick) -> Self {
        Self {
            protocol_introduction_tick: Some(intro_tick),
            transition_duration: transition_ticks,
            // Default parameters from Bass (1969) and Rai & McAndrews (2015)
            adoption_innovation_coeff: 0.03,
            adoption_imitation_coeff: 0.38,
            target_adoption_rate: 0.90,
            era_aware_memory: true,
            volatility_window: 100,
            track_damage_events: true,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Determine current era based on tick
    ///
    /// # Arguments
    /// * `current_tick` - Current simulation tick
    ///
    /// # Returns
    /// Current `SimulationEra` based on temporal configuration
    #[must_use]
    pub fn current_era(&self, current_tick: Tick) -> SimulationEra {
        match self.protocol_introduction_tick {
            None => {
                // Static configuration
                if self.target_adoption_rate > 0.5 {
                    SimulationEra::PostProtocol
                } else {
                    SimulationEra::PreProtocol
                }
            }
            Some(intro_tick) => {
                if current_tick < intro_tick {
                    SimulationEra::PreProtocol
                } else if current_tick < intro_tick + self.transition_duration {
                    SimulationEra::Transition
                } else {
                    SimulationEra::PostProtocol
                }
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate protocol adoption rate at given tick
    ///
    /// Returns value in [0.0, 1.0] representing expected fraction
    /// of agents using protocol. Follows Bass diffusion S-curve.
    ///
    /// # Arguments
    /// * `current_tick` - Current simulation tick
    ///
    /// # Returns
    /// Adoption rate in [0.0, 1.0]
    ///
    /// # Mathematical Model
    ///
    /// Bass diffusion model (Bass, 1969):
    /// ```text
    /// dA/dt = (p + q·A/m) · (m - A)
    /// ```
    ///
    /// Where:
    /// - A(t) = cumulative adopters
    /// - m = market potential (`target_adoption_rate`)
    /// - p = innovation coefficient
    /// - q = imitation coefficient
    ///
    /// # Example
    /// ```
    /// use sim_core::temporal::TemporalConfig;
    ///
    /// let config = TemporalConfig::dynamic_introduction(1000, 1000);
    ///
    /// // Before introduction: 0% adoption
    /// debug_assert_eq!(config.adoption_rate(500), 0.0);
    ///
    /// // At introduction start: near 0% (innovators only)
    /// debug_assert!(config.adoption_rate(1000) < 0.05);
    ///
    /// // Halfway through: significant adoption (imitation effects)
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// let mid_rate = config.adoption_rate(1500);
    /// debug_assert!(mid_rate > 0.2 && mid_rate < 0.8);
    ///
    /// // After transition: target adoption rate
    /// debug_assert!((config.adoption_rate(2000) - 0.9).abs() < 0.05);
    /// ```
    #[must_use]
    pub fn adoption_rate(&self, current_tick: Tick) -> f32 {
        match self.protocol_introduction_tick {
            None => self.target_adoption_rate,
            Some(intro_tick) => {
                if current_tick < intro_tick {
                    0.0
                } else if current_tick >= intro_tick + self.transition_duration {
                    self.target_adoption_rate
                } else {
                    // Time since introduction (normalized to 0-1)
                    let t = (current_tick - intro_tick) as f32;
                    let transition = self.transition_duration as f32;
                    let normalized_time = t / transition;

                    // Bass diffusion parameters - scaled so curve completes in [0,1]
                    // With default p=0.03, q=0.38, the curve needs ~50 time units
                    // to reach saturation. We scale up by TIME_SCALE to fit in normalized time.
                    const TIME_SCALE: f32 = 20.0;
                    let p = self.adoption_innovation_coeff * TIME_SCALE;
                    let q = self.adoption_imitation_coeff * TIME_SCALE;
                    let m = self.target_adoption_rate;

                    // Analytical solution to Bass diffusion equation
                    // A(t) = m * (1 - exp(-(p+q)t)) / (1 + (q/p) * exp(-(p+q)t))
                    let exp_term = (-(p + q) * normalized_time).exp();
                    let numerator = 1.0 - exp_term;
                    let denominator = 1.0 + (q / p) * exp_term;

                    let adoption = m * numerator / denominator;
                    adoption.clamp(0.0, m)
                }
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get expected number of new adopters in a tick
    ///
    /// Useful for determining how many agents should adopt protocol
    /// in the current simulation step.
    #[must_use]
    pub fn expected_new_adopters(&self, current_tick: Tick, population_size: usize) -> usize {
        if current_tick == 0 {
            return 0;
        }

        let current_rate = self.adoption_rate(current_tick);
        let prev_rate = self.adoption_rate(current_tick - 1);
        let delta_rate = current_rate - prev_rate;

        (delta_rate * population_size as f32) as usize
    }

    /// Check if we're at an era boundary
    ///
    /// Returns true if the tick represents the first tick of a new era
    #[must_use]
    pub const fn is_era_boundary(&self, current_tick: Tick) -> bool {
        match self.protocol_introduction_tick {
            None => false,
            Some(intro_tick) => {
                current_tick == intro_tick || current_tick == intro_tick + self.transition_duration
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get remaining ticks in current era
    ///
    /// Returns None if in stable era or static configuration
    #[must_use]
    pub fn remaining_in_era(&self, current_tick: Tick) -> Option<Tick> {
        match self.protocol_introduction_tick {
            None => None,
            Some(intro_tick) => {
                let era = self.current_era(current_tick);
                match era {
                    SimulationEra::PreProtocol => Some(intro_tick - current_tick),
                    SimulationEra::Transition => {
                        Some(intro_tick + self.transition_duration - current_tick)
                    }
                    SimulationEra::PostProtocol => None,
                }
            }
        }
    }
}

impl Default for TemporalConfig {
    fn default() -> Self {
        Self::static_protocol(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_protocol_always_on() {
        let config = TemporalConfig::static_protocol(true);

        assert_eq!(config.current_era(0), SimulationEra::PostProtocol);
        assert_eq!(config.current_era(1000), SimulationEra::PostProtocol);
        assert_eq!(config.adoption_rate(0), 1.0);
        assert_eq!(config.adoption_rate(1000), 1.0);
    }

    #[test]
    fn test_static_protocol_always_off() {
        let config = TemporalConfig::static_protocol(false);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(config.current_era(0), SimulationEra::PreProtocol);
        assert_eq!(config.current_era(1000), SimulationEra::PreProtocol);
        assert_eq!(config.adoption_rate(0), 0.0);
        assert_eq!(config.adoption_rate(1000), 0.0);
    }

    #[test]
    fn test_dynamic_introduction_eras() {
        let config = TemporalConfig::dynamic_introduction(1000, 500);

        // Pre-protocol
        assert_eq!(config.current_era(0), SimulationEra::PreProtocol);
        assert_eq!(config.current_era(999), SimulationEra::PreProtocol);

        // Transition starts
        assert_eq!(config.current_era(1000), SimulationEra::Transition);

        // Mid-transition
        assert_eq!(config.current_era(1250), SimulationEra::Transition);

        // End of transition
        assert_eq!(config.current_era(1499), SimulationEra::Transition);

        // Post-protocol
        assert_eq!(config.current_era(1500), SimulationEra::PostProtocol);
        assert_eq!(config.current_era(2000), SimulationEra::PostProtocol);
    }

    #[test]
    fn test_adoption_rate_s_curve() {
        let config = TemporalConfig::dynamic_introduction(1000, 1000);

        // Before: 0%
        assert_eq!(config.adoption_rate(999), 0.0);

        // At start: near 0 (innovators)
        let start_rate = config.adoption_rate(1000);
        assert!(
            (0.0..0.05).contains(&start_rate),
            "Start rate should be very low, got {start_rate}"
        );

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // At midpoint: accelerating (should be > 25% for typical S-curve)
        let mid_rate = config.adoption_rate(1500);
        assert!(
            mid_rate > 0.25 && mid_rate < 0.75,
            "Mid rate should be in middle range, got {mid_rate}"
        );

        // At end: target rate
        let end_rate = config.adoption_rate(2000);
        assert!(
            (end_rate - 0.9).abs() < 0.01,
            "End rate should be target (0.9), got {end_rate}"
        );

        // Verify monotonic increase
        let mut prev = 0.0;
        for tick in 1000..=2000 {
            let rate = config.adoption_rate(tick);
            assert!(rate >= prev, "Adoption rate should be monotonic");
            prev = rate;
        }
    }

    #[test]
    fn test_era_boundary_detection() {
        let config = TemporalConfig::dynamic_introduction(1000, 500);

        assert!(config.is_era_boundary(1000));
        assert!(config.is_era_boundary(1500));
        assert!(!config.is_era_boundary(1001));
        assert!(!config.is_era_boundary(999));
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_remaining_in_era() {
        let config = TemporalConfig::dynamic_introduction(1000, 500);

        assert_eq!(config.remaining_in_era(500), Some(500));
        assert_eq!(config.remaining_in_era(1250), Some(250));
        assert_eq!(config.remaining_in_era(1500), None);
    }

    #[test]
    fn test_simulation_era_helpers() {
        assert!(SimulationEra::PreProtocol.is_stable());
        assert!(!SimulationEra::Transition.is_stable());
        assert!(SimulationEra::PostProtocol.is_stable());

        assert!(!SimulationEra::PreProtocol.protocol_available());
        assert!(SimulationEra::Transition.protocol_available());
        assert!(SimulationEra::PostProtocol.protocol_available());
    }

    #[test]
    fn test_expected_new_adopters() {
        let config = TemporalConfig::dynamic_introduction(1000, 1000);

        // Before introduction
        let new_before = config.expected_new_adopters(500, 1000);
        assert_eq!(new_before, 0);

        // After transition
        let new_after = config.expected_new_adopters(2500, 1000);
        assert_eq!(new_after, 0);

        // During transition (some positive number)
        let new_during = config.expected_new_adopters(1500, 1000);
        assert!(new_during > 0, "Should have new adopters during transition");
    }
}
