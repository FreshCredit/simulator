//! Emotional Dynamics Module (P1-AGENT-3)
//!
//! Continuous emotion state transitions for agents.

pub mod appraisal;
pub mod regulation;
pub mod transitions;

pub use appraisal::{AppraisalEngine, AppraisalResult, EventAppraisal};
pub use regulation::{EmotionRegulation, RegulationStrategy};
pub use transitions::{EmotionState, EmotionTransition, TransitionModel};

use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Emotional dynamics system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalDynamicsConfig {
    /// baseline valence.
    pub baseline_valence: f32,
    /// baseline arousal.
    pub baseline_arousal: f32,
    /// decay rate.
    pub decay_rate: f32,
    /// reactivity.
    pub reactivity: f32,
    /// regulation strength.
    pub regulation_strength: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for EmotionalDynamicsConfig {
    fn default() -> Self {
        Self {
            baseline_valence: 0.0,
            baseline_arousal: 0.0,
            decay_rate: 0.05,
            reactivity: 0.5,
            regulation_strength: 0.3,
        }
    }
}

/// Complete emotional dynamics system
#[derive(Debug, Clone)]
pub struct EmotionalDynamics {
    /// state.
    pub state: EmotionState,
    /// config.
    pub config: EmotionalDynamicsConfig,
    /// appraisal.
    pub appraisal: AppraisalEngine,
    /// regulation.
    pub regulation: EmotionRegulation,
    /// transition.
    pub transition: TransitionModel,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl EmotionalDynamics {
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    /// new.
    pub fn new(config: EmotionalDynamicsConfig) -> Self {
        Self {
            state: EmotionState::new(config.baseline_valence, config.baseline_arousal),
            config: config.clone(),
            appraisal: AppraisalEngine::new(),
            regulation: EmotionRegulation::new(config.regulation_strength),
            transition: TransitionModel::new(),
        }
    }

    /// Process an emotional event
    pub fn process_event(&mut self, event: &EventAppraisal) {
        // 1. Appraise the event
        let appraisal = self.appraisal.appraise(event);

        // 2. Calculate emotional change
        let change = self.transition.calculate_change(&self.state, &appraisal);

        // 3. Apply regulation
        let regulated = self.regulation.regulate(change);

        // 4. Update state
        self.state.update(regulated);

        // 5. Apply decay toward baseline
        self.state.decay_toward_baseline(
            self.config.baseline_valence,
            self.config.baseline_arousal,
            self.config.decay_rate,
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update for time step
    pub fn tick(&mut self) {
        self.state.decay_toward_baseline(
            self.config.baseline_valence,
            self.config.baseline_arousal,
            self.config.decay_rate,
        );
    }

    /// Get current emotion
    #[must_use]
    pub fn current_emotion(&self) -> Emotion {
        self.state.classify()
    }

    /// Get emotional influence on decision
    #[must_use]
    pub fn decision_influence(&self) -> f32 {
        self.state.arousal * (0.5 + self.state.valence.abs() * 0.5)
    }
}

impl Default for EmotionalDynamics {
    fn default() -> Self {
        Self::new(EmotionalDynamicsConfig::default())
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

/// Discrete emotion classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Emotion {
    /// joy.
    Joy,
    /// sadness.
    Sadness,
    /// anger.
    Anger,
    /// fear.
    Fear,
    /// surprise.
    Surprise,
    /// disgust.
    Disgust,
    /// trust.
    Trust,
    /// anticipation.
    Anticipation,
    /// neutral.
    Neutral,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Emotion {
    #[must_use]
    /// valence.
    pub const fn valence(&self) -> f32 {
        match self {
            Self::Joy => 1.0,
            Self::Trust => 0.8,
            Self::Anticipation => 0.5,
            Self::Surprise => 0.0,
            Self::Fear => -0.7,
            Self::Sadness => -0.8,
            Self::Anger => -0.6,
            Self::Disgust => -0.9,
            Self::Neutral => 0.0,
        }
    }

    #[must_use]
    /// arousal.
    pub const fn arousal(&self) -> f32 {
        match self {
            Self::Anger => 0.8,
            Self::Fear => 0.9,
            Self::Joy => 0.6,
            Self::Surprise => 0.8,
            Self::Anticipation => 0.5,
            Self::Disgust => 0.4,
            Self::Trust => 0.2,
            Self::Sadness => 0.1,
            Self::Neutral => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_emotion_classification() {
        let joy = Emotion::Joy;
        assert!(joy.valence() > 0.0);
        assert!(joy.arousal() > 0.0);

        let sadness = Emotion::Sadness;
        assert!(sadness.valence() < 0.0);
    }

    #[test]
    fn test_emotional_dynamics() {
        let mut dynamics = EmotionalDynamics::default();
        assert_eq!(dynamics.current_emotion(), Emotion::Neutral);

        dynamics.tick();
        // Should still be near neutral
        assert!(dynamics.state.valence.abs() < 0.1);
    }

    #[test]
    fn test_emotion_valence() {
        assert_eq!(Emotion::Joy.valence(), 1.0);
        assert_eq!(Emotion::Sadness.valence(), -0.8);
        assert_eq!(Emotion::Neutral.valence(), 0.0);
        assert_eq!(Emotion::Trust.valence(), 0.8);
        assert_eq!(Emotion::Disgust.valence(), -0.9);
    }

    #[test]
    fn test_emotion_arousal() {
        assert_eq!(Emotion::Fear.arousal(), 0.9);
        assert_eq!(Emotion::Anger.arousal(), 0.8);
        assert_eq!(Emotion::Joy.arousal(), 0.6);
        assert_eq!(Emotion::Neutral.arousal(), 0.0);
        assert_eq!(Emotion::Sadness.arousal(), 0.1);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_emotional_config_default() {
        let config = EmotionalDynamicsConfig::default();
        assert_eq!(config.baseline_valence, 0.0);
        assert_eq!(config.baseline_arousal, 0.0);
        assert_eq!(config.decay_rate, 0.05);
        assert_eq!(config.reactivity, 0.5);
        assert_eq!(config.regulation_strength, 0.3);
    }

    #[test]
    fn test_decision_influence() {
        let dynamics = EmotionalDynamics::default();
        let influence = dynamics.decision_influence();
        assert!(influence >= 0.0);
    }

    #[test]
    fn test_process_event() {
        let mut dynamics = EmotionalDynamics::default();
        let event = crate::emotion_dynamics::appraisal::EventAppraisal {
            event_type: crate::emotion_dynamics::appraisal::EventType::Reward,
            magnitude: 0.8,
            target: None,
            context: crate::emotion_dynamics::appraisal::EventContext::default(),
        };
        dynamics.process_event(&event);
        // After positive event, should have positive valence
        assert!(dynamics.state.valence > 0.0 || dynamics.current_emotion() == Emotion::Neutral);
    }
}
