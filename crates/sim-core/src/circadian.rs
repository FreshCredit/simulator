//! Circadian rhythm system (SCN-mediated)
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION neuromodulators, not medical advice
//!
//! The suprachiasmatic nucleus (SCN) drives ~24-hour cycles in physiology.
//! Circadian rhythms affect dopamine, cortisol, and other neuromodulators.
//!
//! Research basis:
//! - Castro-Diehl et al. (2021) - Circadian regulation of reward
//! - `McEwen` (1998) - Allostatic load varies by time of day
//! - Cortisol awakening response (CAR)

use std::f32::consts::PI;

use serde::{Deserialize, Serialize};

use crate::docas::DocasState;
use crate::types::{Tick, C, D, S};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Circadian rhythm system (SCN-mediated)
///
/// Models the ~24-hour biological clock that modulates neuromodulator levels
/// throughout the day. Phase 0 corresponds to dawn (6 AM), π to dusk (6 PM).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CircadianSystem {
    /// Phase of circadian clock [0, 2π]
    /// 0 = dawn (6 AM), π = dusk (6 PM)
    pub phase: f32,

    /// Period (typically 24 hours = 24*365 ticks, assuming 365 ticks/hour)
    pub period: Tick,

    /// Amplitude of oscillation (0.0 to 1.0)
    /// Controls the strength of circadian effects
    pub amplitude: f32,

    /// Zeitgeber strength (external time cue coupling)
    /// Higher values = stronger entrainment to external cues
    pub zeitgeber_strength: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for CircadianSystem {
    fn default() -> Self {
        Self {
            phase: 0.0,              // Start at dawn
            period: 24 * 365,        // 24 hours, 365 ticks per hour
            amplitude: 0.5,          // Moderate amplitude
            zeitgeber_strength: 0.3, // Moderate entrainment
        }
    }
}

impl CircadianSystem {
    /// Create a new circadian system with default parameters
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a circadian system with custom parameters
    ///
    /// # Arguments
    /// * `phase` - Initial phase [0, 2π], where 0 = dawn
    /// * `period` - Period in ticks (e.g., 24*365 for 24 hours)
    /// * `amplitude` - Oscillation amplitude [0.0, 1.0]
    /// * `zeitgeber_strength` - External cue coupling strength
    #[must_use]
    pub fn with_params(phase: f32, period: Tick, amplitude: f32, zeitgeber_strength: f32) -> Self {
        Self {
            phase: phase.rem_euclid(2.0 * PI),
            period,
            amplitude: amplitude.clamp(0.0, 1.0),
            zeitgeber_strength: zeitgeber_strength.clamp(0.0, 1.0),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update circadian phase based on elapsed tick
    ///
    /// Advances the phase by 2π/period each tick, wrapping around at 2π.
    pub fn update(&mut self, _tick: Tick) {
        if self.period == 0 {
            return;
        }
        // Advance phase: Δφ = 2π / T
        let phase_increment = 2.0 * PI / self.period as f32;
        self.phase += phase_increment;

        // Wrap to [0, 2π)
        if self.phase >= 2.0 * PI {
            self.phase -= 2.0 * PI;
        }
    }

    /// Get circadian modulation factor for dopamine
    ///
    /// Dopamine peaks in the morning (phase ~0, dawn) and has a trough at night.
    /// This reflects increased reward sensitivity and motivation in the morning.
    ///
    /// Returns a modulation factor around 1.0 (range: 1.0 ± 0.3*amplitude)
    #[must_use]
    pub fn dopamine_modulation(&self) -> f32 {
        // Peak at dawn (phase = 0), trough at dusk (phase = π)
        // cos(0) = 1 (peak), cos(π) = -1 (trough)
        let modulation = 1.0 + self.amplitude * 0.3 * self.phase.cos();
        modulation.max(0.1) // Prevent negative or near-zero values
    }

    /// Cortisol awakening response (CAR)
    ///
    /// Cortisol shows a sharp peak shortly after awakening (phase ~0.2, ~1 hour post-waking)
    /// followed by a gradual decline throughout the day.
    ///
    /// Returns a modulation factor with a Gaussian peak around awakening
    #[must_use]
    pub fn cortisol_modulation(&self) -> f32 {
        // CAR peak occurs shortly after phase 0 (awakening)
        // Peak at phase ~0.2 (about 1 hour after dawn at 365 ticks/hour)
        let car_phase_offset = 0.2_f32;
        // Compute shortest signed angular distance on S^1 to eliminate cliff at phase 0.2
        let mut diff = (self.phase - car_phase_offset).rem_euclid(2.0 * PI);
        if diff > PI {
            diff -= 2.0 * PI;
        }

        // Gaussian response centered at awakening
        // Width parameter 0.1 gives ~2-hour full width at half maximum
        let car_response = (-diff.powi(2) / 0.1).exp();

        // Baseline + CAR peak
        let modulation = 1.0 + self.amplitude * 0.5 * car_response;
        modulation.max(0.1)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Serotonin modulation
    ///
    /// Serotonin is relatively stable but shows slight variation with circadian phase.
    /// Generally more stable during the day, slightly reduced at night.
    ///
    /// Returns a modulation factor around 1.0 with small circadian variation
    #[must_use]
    pub fn serotonin_modulation(&self) -> f32 {
        // Relatively flat with slight dip at night (phase = π)
        // sin(π/2) at dawn, sin(3π/2) at dusk
        let modulation = 1.0 - self.amplitude * 0.1 * (self.phase - PI / 2.0).sin();
        modulation.max(0.1)
    }

    /// Melatonin level
    ///
    /// Melatonin is high at night (phase ~π) and low during the day.
    /// Acts as the "darkness hormone" and promotes sleep.
    ///
    /// Returns melatonin level in [0.0, 1.0]
    #[must_use]
    pub fn melatonin_level(&self) -> f32 {
        // Night is centered at phase = π
        let night_distance = (self.phase - PI).abs();

        // Melatonin high when close to π (within π/3 of night center)
        // π/3 ≈ 1.047 radians ≈ 4 hours on either side of midnight
        if night_distance < PI / 3.0 {
            // Linear falloff from center
            0.5 + 0.5 * (1.0 - night_distance / (PI / 3.0))
        } else {
            0.0 // Daytime - no melatonin
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get time of day as descriptive string
    ///
    /// Maps phase to human-readable time description
    #[must_use]
    pub fn time_of_day(&self) -> &'static str {
        let hour = (self.phase / (2.0 * PI) * 24.0) + 6.0; // 6 AM at phase 0
        match hour as u32 % 24 {
            5..=8 => "morning",
            9..=11 => "late_morning",
            12..=14 => "afternoon",
            15..=17 => "late_afternoon",
            18..=20 => "evening",
            21..=23 => "night",
            _ => "late_night", // 0-4
        }
    }

    /// Check if currently nighttime
    ///
    /// Night is defined as phase near π (within π/3)
    #[must_use]
    pub fn is_night(&self) -> bool {
        let night_distance = (self.phase - PI).abs();
        night_distance < PI / 3.0
    }

    /// Check if currently daytime
    #[must_use]
    pub fn is_day(&self) -> bool {
        !self.is_night()
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Get simulated hour (0-23, with 6 = phase 0)
    #[must_use]
    pub fn hour(&self) -> u32 {
        ((self.phase / (2.0 * PI) * 24.0) + 6.0) as u32 % 24
    }

    /// Apply circadian modulation to DOCAS state
    ///
    /// Modulates dopamine, cortisol, and serotonin based on circadian phase.
    /// This method applies multiplicative modulation to the current DOCAS levels.
    ///
    /// # Arguments
    /// * `docas` - Mutable reference to DOCAS state to modulate
    pub fn modulate_docas(&self, docas: &mut DocasState) {
        // Dopamine varies with circadian - peak in morning
        docas.levels[D] *= self.dopamine_modulation();

        // Cortisol has awakening response
        docas.levels[C] *= self.cortisol_modulation();

        // Serotonin relatively stable but slightly affected
        docas.levels[S] *= self.serotonin_modulation();

        // Melatonin indirectly affects serotonin
        let melatonin = self.melatonin_level();
        docas.levels[S] *= 1.0 - melatonin * 0.2;

        // Clamp to valid range [0, 1]
        for i in 0..docas.levels.len() {
            docas.levels[i] = docas.levels[i].clamp(0.0, 1.0);
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate sleep quality based on sleep duration
    ///
    /// Sleep quality follows a Gaussian distribution around the optimal
    /// sleep duration of ~8 hours. Too little or too much sleep reduces quality.
    ///
    /// # Arguments
    /// * `sleep_duration` - Hours of sleep
    ///
    /// # Returns
    /// Sleep quality in [0.0, 1.0], where 1.0 is optimal
    #[must_use]
    pub fn sleep_quality(&self, sleep_duration: f32) -> f32 {
        const OPTIMAL_SLEEP: f32 = 8.0;
        const SLEEP_WIDTH: f32 = 4.0; // Standard deviation

        let deviation = sleep_duration - OPTIMAL_SLEEP;
        (-(deviation * deviation) / (2.0 * SLEEP_WIDTH * SLEEP_WIDTH)).exp()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate next-day functioning based on sleep
    ///
    /// Combines sleep quality with circadian factors to predict
    /// cognitive performance on the following day.
    ///
    /// # Arguments
    /// * `sleep_duration` - Hours of sleep
    /// * `sleep_consistency` - Regularity of sleep schedule [0.0, 1.0]
    ///
    /// # Returns
    /// Predicted functioning level in [0.0, 1.0]
    #[must_use]
    pub fn next_day_functioning(&self, sleep_duration: f32, sleep_consistency: f32) -> f32 {
        let quality = self.sleep_quality(sleep_duration);
        let consistency_bonus = 0.2 * sleep_consistency;

        (quality * (1.0 + consistency_bonus)).min(1.0)
    }

    /// Entrain phase to external zeitgeber
    ///
    /// Simulates entrainment to external time cues (light, social schedules).
    /// Gradually shifts phase toward the zeitgeber time.
    ///
    /// # Arguments
    /// * `zeitgeber_phase` - Target phase from external cue
    /// * `strength` - Coupling strength for this entrainment event
    pub fn entrain(&mut self, zeitgeber_phase: f32, strength: f32) {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let target = zeitgeber_phase.rem_euclid(2.0 * PI);
        let current = self.phase;

        // Calculate shortest angular distance
        let mut diff = target - current;
        if diff > PI {
            diff -= 2.0 * PI;
        } else if diff < -PI {
            diff += 2.0 * PI;
        }

        // Gradual phase shift
        let effective_strength = strength * self.zeitgeber_strength;
        self.phase = (current + effective_strength * diff).rem_euclid(2.0 * PI);
    }

    /// Get all circadian modulation values at once
    ///
    /// Returns a struct with all neuromodulator modulations for efficient access
    #[must_use]
    pub fn get_modulations(&self) -> CircadianModulations {
        CircadianModulations {
            dopamine: self.dopamine_modulation(),
            cortisol: self.cortisol_modulation(),
            serotonin: self.serotonin_modulation(),
            melatonin: self.melatonin_level(),
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Collection of all circadian modulations
///
/// Returned by `CircadianSystem::get_modulations()` for efficient batch access
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CircadianModulations {
    /// Dopamine modulation factor
    pub dopamine: f32,
    /// Cortisol modulation factor
    pub cortisol: f32,
    /// Serotonin modulation factor
    pub serotonin: f32,
    /// Melatonin level [0.0, 1.0]
    pub melatonin: f32,
}

impl CircadianModulations {
    /// Create neutral modulations (all at baseline)
    #[must_use]
    pub const fn neutral() -> Self {
        Self {
            dopamine: 1.0,
            cortisol: 1.0,
            serotonin: 1.0,
            melatonin: 0.0,
        }
    }
}

/// Agent-specific circadian state
///
/// Tracks individual variation in circadian rhythms (chronotypes)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AgentCircadian {
    /// Base circadian system
    pub system: CircadianSystem,

    /// Chronotype offset (lark vs owl)
    /// Negative = morning lark, Positive = night owl
    pub chronotype: f32,

    /// Sleep debt accumulates with insufficient sleep
    pub sleep_debt: f32,

    /// Last sleep duration
    pub last_sleep_duration: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for AgentCircadian {
    fn default() -> Self {
        Self {
            system: CircadianSystem::default(),
            chronotype: 0.0,          // Neutral chronotype
            sleep_debt: 0.0,          // No sleep debt
            last_sleep_duration: 8.0, // Optimal sleep
        }
    }
}

impl AgentCircadian {
    /// Create a new agent circadian state with neutral chronotype
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with specific chronotype
    ///
    /// # Arguments
    /// * `chronotype` - Value in [-1.0, 1.0], negative = morning lark, positive = night owl
    #[must_use]
    pub fn with_chronotype(chronotype: f32) -> Self {
        let chronotype = chronotype.clamp(-1.0, 1.0);

        // Adjust phase based on chronotype
        // Larks peak earlier, owls peak later
        let phase_shift = chronotype * PI / 4.0; // Up to 3 hours shift

        Self {
            chronotype,
            system: CircadianSystem {
                phase: (PI + phase_shift).rem_euclid(2.0 * PI),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Record sleep event
    ///
    /// Updates sleep debt and last sleep duration
    pub fn record_sleep(&mut self, duration: f32) {
        self.last_sleep_duration = duration;

        // Update sleep debt
        const OPTIMAL_SLEEP: f32 = 8.0;
        let deficit = (OPTIMAL_SLEEP - duration).max(0.0);
        self.sleep_debt = (self.sleep_debt + deficit * 0.5).min(10.0); // Cap debt

        // Clear some sleep debt if slept well
        if duration >= OPTIMAL_SLEEP {
            self.sleep_debt = (self.sleep_debt - (duration - OPTIMAL_SLEEP) * 0.3).max(0.0);
        }
    }

    /// Get effective amplitude (reduced by sleep debt)
    #[must_use]
    pub fn effective_amplitude(&self) -> f32 {
        let base = self.system.amplitude;
        let debt_penalty = (self.sleep_debt / 10.0) * 0.3; // Up to 30% reduction
        (base - debt_penalty).max(0.1)
    }

    /// Check if agent is in optimal performance window
    ///
    /// Depends on chronotype - larks perform best in morning, owls in evening
    #[must_use]
    pub fn is_peak_performance_time(&self) -> bool {
        let hour = self.system.hour() as i32;
        let adjusted_hour = (hour as f32 - self.chronotype * 3.0) as i32;

        // Peak performance typically 2-4 hours after "natural" waking
        let peak_start = 9; // 9 AM adjusted
        let peak_end = 14; // 2 PM adjusted

        let normalized_hour = ((adjusted_hour % 24) + 24) % 24;
        normalized_hour >= peak_start && normalized_hour <= peak_end
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get cognitive performance modifier
    ///
    /// Combines circadian phase, chronotype, and sleep debt
    #[must_use]
    pub fn cognitive_performance(&self) -> f32 {
        let base = self.system.dopamine_modulation();
        let chronotype_bonus = if self.is_peak_performance_time() {
            0.1
        } else {
            0.0
        };
        let sleep_penalty = self.sleep_debt * 0.05;

        (base + chronotype_bonus - sleep_penalty).clamp(0.5, 1.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === CircadianSystem Basic Tests ===

    #[test]
    fn test_circadian_default() {
        let cs = CircadianSystem::default();
        assert_eq!(cs.phase, 0.0);
        assert_eq!(cs.period, 24 * 365);
        assert!((cs.amplitude - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_circadian_new() {
        let cs = CircadianSystem::new();
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert_eq!(cs.phase, 0.0);
    }

    #[test]
    fn test_circadian_custom_params() {
        let cs = CircadianSystem::with_params(PI, 1000, 0.8, 0.5);
        assert!((cs.phase - PI).abs() < 0.01);
        assert_eq!(cs.period, 1000);
        assert!((cs.amplitude - 0.8).abs() < 0.01);
        assert!((cs.zeitgeber_strength - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_circadian_phase_wraps() {
        // Phase should wrap to [0, 2π)
        let cs = CircadianSystem::with_params(3.0 * PI, 1000, 0.5, 0.3);
        assert!(cs.phase >= 0.0 && cs.phase < 2.0 * PI);
        assert!((cs.phase - PI).abs() < 0.01);
    }

    #[test]
    fn test_circadian_update_advances_phase() {
        let mut cs = CircadianSystem::with_params(0.0, 1000, 0.5, 0.3);
        let initial_phase = cs.phase;

        cs.update(1);

        // Phase should advance by 2π/1000
        let expected_increment = 2.0 * PI / 1000.0;
        assert!((cs.phase - initial_phase - expected_increment).abs() < 0.0001);
    }

    #[test]
    fn test_circadian_update_wraps() {
        let period = 1000;
        let phase_increment = 2.0 * PI / period as f32;
        let mut cs =
            CircadianSystem::with_params(2.0 * PI - phase_increment * 0.5, period, 0.5, 0.3);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        cs.update(1);

        // Should wrap around to near 0
        assert!(cs.phase < phase_increment);
    }

    // === Dopamine Modulation Tests ===

    #[test]
    fn test_dopamine_peak_at_dawn() {
        // At phase 0 (dawn), dopamine should be at peak
        let cs = CircadianSystem::with_params(0.0, 1000, 0.5, 0.3);
        let modulation = cs.dopamine_modulation();

        // Peak modulation: 1.0 + 0.5 * 0.3 * cos(0) = 1.0 + 0.15 = 1.15
        assert!((modulation - 1.15).abs() < 0.01);
    }

    #[test]
    fn test_dopamine_trough_at_dusk() {
        // At phase π (dusk), dopamine should be at trough
        let cs = CircadianSystem::with_params(PI, 1000, 0.5, 0.3);
        let modulation = cs.dopamine_modulation();

        // Trough modulation: 1.0 + 0.5 * 0.3 * cos(π) = 1.0 - 0.15 = 0.85
        assert!((modulation - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_dopamine_never_negative() {
        // Even with high amplitude, modulation should stay positive
        let cs = CircadianSystem::with_params(PI, 1000, 1.0, 0.3);
        let modulation = cs.dopamine_modulation();
        assert!(modulation > 0.0);
    }

    // === Cortisol Modulation Tests ===

    #[test]
    fn test_cortisol_peak_after_awakening() {
        // Cortisol should peak shortly after phase 0 (CAR)
        // CAR peak is at phase 0.2 with Gaussian response
        let cs = CircadianSystem::with_params(0.2, 1000, 0.5, 0.3);
        let modulation = cs.cortisol_modulation();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // At exact CAR peak, should be elevated
        // car_response = exp(-0/0.1) = 1.0, modulation = 1.0 + 0.5*0.5*1.0 = 1.25
        assert!(
            modulation > 1.2,
            "CAR modulation should be > 1.2, got {modulation}"
        );
    }

    #[test]
    fn test_cortisol_baseline_away_from_car() {
        // Far from awakening, cortisol should be near baseline
        let cs = CircadianSystem::with_params(PI, 1000, 0.5, 0.3);
        let modulation = cs.cortisol_modulation();

        // Should be close to 1.0 (baseline)
        assert!((modulation - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_cortisol_never_negative() {
        let cs = CircadianSystem::with_params(0.0, 1000, 1.0, 0.3);
        let modulation = cs.cortisol_modulation();
        assert!(modulation > 0.0);
    }
    #[test]
    fn test_cortisol_symmetry_around_car_peak() {
        // Test that phase 0.19 and phase 0.21 have equal cortisol modulation (symmetric around 0.20 peak)
        let cs_pre = CircadianSystem::with_params(0.19, 1000, 0.5, 0.3);
        let cs_post = CircadianSystem::with_params(0.21, 1000, 0.5, 0.3);
        let mod_pre = cs_pre.cortisol_modulation();
        let mod_post = cs_post.cortisol_modulation();

        assert!(
            (mod_pre - mod_post).abs() < 1e-4,
            "Cortisol awakening response must be symmetric around peak: pre={mod_pre}, post={mod_post}"
        );
        // Pre-peak at 0.19 must be elevated above baseline
        assert!(
            mod_pre > 1.24,
            "Pre-peak cortisol must be elevated, got {mod_pre}"
        );
    }

    // === Melatonin Tests ===

    #[test]
    fn test_melatonin_high_at_night() {
        // At phase π (midnight), melatonin should be high
        let cs = CircadianSystem::with_params(PI, 1000, 0.5, 0.3);
        let melatonin = cs.melatonin_level();

        assert!(melatonin > 0.8);
    }

    #[test]
    fn test_melatonin_zero_during_day() {
        // At phase 0 (dawn), melatonin should be zero
        let cs = CircadianSystem::with_params(0.0, 1000, 0.5, 0.3);
        let melatonin = cs.melatonin_level();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(melatonin, 0.0);
    }

    #[test]
    fn test_melatonin_transitions() {
        // Test melatonin transition at night boundaries
        let cs_transition = CircadianSystem::with_params(PI - PI / 3.0, 1000, 0.5, 0.3);
        let mel_transition = cs_transition.melatonin_level();

        // At the boundary, should be low but non-zero
        assert!((0.0..0.5).contains(&mel_transition));
    }

    // === Serotonin Tests ===

    #[test]
    fn test_serotonin_stable() {
        // Serotonin should be relatively stable
        let cs1 = CircadianSystem::with_params(0.0, 1000, 0.5, 0.3);
        let cs2 = CircadianSystem::with_params(PI, 1000, 0.5, 0.3);

        let mod1 = cs1.serotonin_modulation();
        let mod2 = cs2.serotonin_modulation();

        // Variation should be small (< 20%)
        assert!((mod1 - mod2).abs() < 0.2);
    }

    // === Time of Day Tests ===

    #[test]
    fn test_time_of_day_dawn() {
        let cs = CircadianSystem::with_params(0.0, 1000, 0.5, 0.3);
        assert_eq!(cs.time_of_day(), "morning");
        assert_eq!(cs.hour(), 6);
    }

    #[test]
    fn test_time_of_day_noon() {
        let cs = CircadianSystem::with_params(PI / 2.0, 1000, 0.5, 0.3);
        assert_eq!(cs.time_of_day(), "afternoon");
        assert_eq!(cs.hour(), 12);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_time_of_day_evening() {
        // Phase π corresponds to 6 PM (18:00), which is evening
        let cs = CircadianSystem::with_params(PI, 1000, 0.5, 0.3);
        assert_eq!(cs.time_of_day(), "evening");
        assert_eq!(cs.hour(), 18);
    }

    #[test]
    fn test_time_of_day_late_night() {
        // Late night (hours 0-4) corresponds to phase ~4.7 to ~5.5 radians
        // Phase = (hour - 6) / 24 * 2π
        // For hour 2: phase = (2-6)/24*2π = -4/24*2π = -π/3 ≈ 5.76 rad (wrapped)
        let cs = CircadianSystem::with_params(5.5, 1000, 0.5, 0.3);
        assert_eq!(cs.time_of_day(), "late_night");
    }

    #[test]
    fn test_is_night() {
        let cs_night = CircadianSystem::with_params(PI, 1000, 0.5, 0.3);
        assert!(cs_night.is_night());
        assert!(!cs_night.is_day());
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        let cs_day = CircadianSystem::with_params(0.0, 1000, 0.5, 0.3);
        assert!(!cs_day.is_night());
        assert!(cs_day.is_day());
    }

    // === DOCAS Modulation Tests ===

    #[test]
    fn test_modulate_docas_dawn() {
        // Test at phase 0.2 where CAR peaks
        let cs = CircadianSystem::with_params(0.2, 1000, 0.5, 0.3);
        let mut docas = DocasState::new([0.5, 0.5, 0.5, 0.5, 0.5]);

        cs.modulate_docas(&mut docas);

        // At CAR peak (phase 0.2): dopamine high, cortisol high, serotonin stable
        assert!(
            docas.levels[D] > 0.5,
            "Dopamine should be elevated at dawn, got {}",
            docas.levels[D]
        );
        assert!(
            docas.levels[C] > 0.5,
            "Cortisol should be elevated at CAR peak, got {}",
            docas.levels[C]
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_modulate_docas_clamped() {
        let cs = CircadianSystem::with_params(0.0, 1000, 1.0, 0.3);
        let mut docas = DocasState::new([0.9, 0.9, 0.9, 0.9, 0.9]);

        cs.modulate_docas(&mut docas);

        // All values should be clamped to [0, 1]
        for level in docas.levels {
            assert!((0.0..=1.0).contains(&level));
        }
    }

    // === Sleep Quality Tests ===

    #[test]
    fn test_sleep_quality_optimal() {
        let cs = CircadianSystem::default();
        let quality = cs.sleep_quality(8.0);

        // 8 hours is optimal, should be ~1.0
        assert!(quality > 0.95);
    }

    #[test]
    fn test_sleep_quality_decreases_with_deviation() {
        let cs = CircadianSystem::default();
        let optimal = cs.sleep_quality(8.0);
        let short = cs.sleep_quality(4.0);
        let long = cs.sleep_quality(12.0);

        assert!(short < optimal);
        assert!(long < optimal);
    }

    #[test]
    fn test_sleep_quality_extreme() {
        let cs = CircadianSystem::default();
        let no_sleep = cs.sleep_quality(0.0);
        let excessive = cs.sleep_quality(16.0);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Very poor sleep should still return positive value (exponential)
        assert!(no_sleep > 0.0);
        assert!(no_sleep < 0.5);
        assert!(excessive > 0.0);
        assert!(excessive < 0.5);
    }

    #[test]
    fn test_next_day_functioning() {
        let cs = CircadianSystem::default();

        let optimal = cs.next_day_functioning(8.0, 1.0);
        let poor = cs.next_day_functioning(4.0, 0.5);

        assert!(optimal > poor);
        assert!(optimal <= 1.0);
    }

    // === CircadianModulations Tests ===

    #[test]
    fn test_get_modulations() {
        // Test at CAR peak (phase 0.2) for best cortisol elevation
        let cs = CircadianSystem::with_params(0.2, 1000, 0.5, 0.3);
        let mods = cs.get_modulations();

        assert!(mods.dopamine > 1.0, "Dopamine should be elevated at dawn"); // Near peak at dawn
        assert!(
            mods.cortisol > 1.0,
            "Cortisol should be elevated at CAR peak (0.2), got {}",
            mods.cortisol
        ); // CAR peak
        assert!(mods.melatonin == 0.0, "Melatonin should be zero during day"); // Daytime
        assert!(
            mods.serotonin > 0.9 && mods.serotonin < 1.1,
            "Serotonin should be stable, got {}",
            mods.serotonin
        ); // Stable
    }

    #[test]
    fn test_modulations_neutral() {
        let mods = CircadianModulations::neutral();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(mods.dopamine, 1.0);
        assert_eq!(mods.cortisol, 1.0);
        assert_eq!(mods.serotonin, 1.0);
        assert_eq!(mods.melatonin, 0.0);
    }

    // === AgentCircadian Tests ===

    #[test]
    fn test_agent_circadian_default() {
        let ac = AgentCircadian::default();
        assert_eq!(ac.chronotype, 0.0);
        assert_eq!(ac.sleep_debt, 0.0);
        assert_eq!(ac.last_sleep_duration, 8.0);
    }

    #[test]
    fn test_agent_circadian_chronotype() {
        let lark = AgentCircadian::with_chronotype(-1.0);
        let owl = AgentCircadian::with_chronotype(1.0);

        // Lark (morning type) has negative chronotype, phase shifts backward
        // Owl (evening type) has positive chronotype, phase shifts forward
        // A lark at "neutral" time would be closer to their peak than an owl
        // Implementation formula: phase = (PI + chronotype * PI/4).rem_euclid(2*PI)
        // Lark (-1.0): PI + (-PI/4) = 3*PI/4 = 2.356
        // Owl (+1.0):  PI + (PI/4)  = 5*PI/4 = 3.927
        let expected_lark_phase = (PI + (-(PI / 4.0))).rem_euclid(2.0 * PI);
        let expected_owl_phase = (PI + (1.0_f32 * PI / 4.0)).rem_euclid(2.0 * PI);

        assert!(
            (lark.system.phase - expected_lark_phase).abs() < 0.01,
            "Lark phase {} should be near {}",
            lark.system.phase,
            expected_lark_phase
        );
        assert!(
            (owl.system.phase - expected_owl_phase).abs() < 0.01,
            "Owl phase {} should be near {}",
            owl.system.phase,
            expected_owl_phase
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_chronotype_clamped() {
        let extreme = AgentCircadian::with_chronotype(2.0);
        assert_eq!(extreme.chronotype, 1.0);

        let negative = AgentCircadian::with_chronotype(-2.0);
        assert_eq!(negative.chronotype, -1.0);
    }

    #[test]
    fn test_record_sleep_updates_debt() {
        let mut ac = AgentCircadian::default();
        ac.record_sleep(4.0); // 4 hours - insufficient

        assert!(ac.sleep_debt > 0.0);
        assert_eq!(ac.last_sleep_duration, 4.0);
    }

    #[test]
    fn test_record_sleep_reduces_debt() {
        let mut ac = AgentCircadian {
            sleep_debt: 5.0,
            ..AgentCircadian::default()
        };
        ac.record_sleep(10.0); // Extra sleep

        assert!(ac.sleep_debt < 5.0);
    }

    #[test]
    fn test_effective_amplitude_reduced_by_debt() {
        let mut ac = AgentCircadian::default();
        let base_amp = ac.effective_amplitude();

        ac.sleep_debt = 10.0; // Maximum debt
        let reduced_amp = ac.effective_amplitude();

        assert!(reduced_amp < base_amp);
    }

    #[test]
    fn test_is_peak_performance_time() {
        // Lark should peak in morning
        let mut lark = AgentCircadian::with_chronotype(-1.0);
        lark.system.phase = 0.0; // Dawn
        assert!(lark.is_peak_performance_time());

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Owl should not peak in morning
        let mut owl = AgentCircadian::with_chronotype(1.0);
        owl.system.phase = 0.0; // Dawn
        assert!(!owl.is_peak_performance_time());
    }

    #[test]
    fn test_cognitive_performance_factors() {
        let mut ac = AgentCircadian::default();
        let baseline = ac.cognitive_performance();

        // Add sleep debt
        ac.sleep_debt = 5.0;
        let impaired = ac.cognitive_performance();

        assert!(impaired < baseline);
    }

    // === Entrainment Tests ===

    #[test]
    fn test_entrain_shifts_phase() {
        let mut cs = CircadianSystem::with_params(0.0, 1000, 0.5, 0.5);

        cs.entrain(PI, 1.0); // Strong entrainment to opposite phase

        // Phase should have shifted toward PI
        assert!(cs.phase > 0.0);
        assert!(cs.phase < PI); // Partial shift due to coupling
    }

    #[test]
    fn test_entrain_shortest_path() {
        // Test that entrainment takes shortest angular path
        let mut cs = CircadianSystem::with_params(0.1, 1000, 0.5, 0.5);

        cs.entrain(2.0 * PI - 0.1, 1.0); // Target just before 2π

        // Should move backward (negative), not forward through almost 2π
        assert!(cs.phase < 0.2);
    }
}
