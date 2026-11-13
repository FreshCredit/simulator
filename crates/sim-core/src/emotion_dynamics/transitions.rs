//! Emotion State and Transitions (P1-AGENT-3)

use serde::{Deserialize, Serialize};

use super::appraisal::AppraisalResult;
use super::Emotion;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Continuous emotion state (valence-arousal space)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EmotionState {
    /// valence.
    pub valence: f32, // -1.0 (negative) to 1.0 (positive)
    /// arousal.
    pub arousal: f32, // 0.0 (calm) to 1.0 (excited)
    /// dominance.
    pub dominance: f32, // 0.0 (submissive) to 1.0 (dominant)
    /// timestamp.
    pub timestamp: u64,
}

impl EmotionState {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create new emotion state
    #[must_use]
    pub fn new(valence: f32, arousal: f32) -> Self {
        Self {
            valence: valence.clamp(-1.0, 1.0),
            arousal: arousal.clamp(0.0, 1.0),
            dominance: 0.5,
            timestamp: 0,
        }
    }

    /// Update state with change
    pub fn update(&mut self, change: EmotionTransition) {
        self.valence = (self.valence + change.delta_valence).clamp(-1.0, 1.0);
        self.arousal = (self.arousal + change.delta_arousal).clamp(0.0, 1.0);
        self.dominance = (self.dominance + change.delta_dominance).clamp(0.0, 1.0);
        self.timestamp += 1;
    }

    /// Decay toward baseline
    pub fn decay_toward_baseline(&mut self, baseline_v: f32, baseline_a: f32, rate: f32) {
        self.valence = lerp(self.valence, baseline_v, rate);
        self.arousal = lerp(self.arousal, baseline_a, rate);
        // Dominance decays toward 0.5
        self.dominance = lerp(self.dominance, 0.5, rate);
    }

    /// Classify discrete emotion from continuous state
    #[must_use]
    pub fn classify(&self) -> Emotion {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Use valence and arousal to determine emotion
        // Based on Russell's circumplex model
        if self.arousal < 0.2 {
            if self.valence < -0.3 {
                Emotion::Sadness
            } else {
                Emotion::Neutral
            }
        } else if self.valence > 0.5 {
            if self.arousal > 0.6 {
                Emotion::Joy
            } else {
                Emotion::Trust
            }
        } else if self.valence < -0.5 {
            if self.arousal > 0.7 {
                Emotion::Anger
            } else {
                Emotion::Disgust
            }
        } else if self.arousal > 0.7 {
            if self.valence < 0.0 {
                Emotion::Fear
            } else {
                Emotion::Surprise
            }
        } else {
            Emotion::Anticipation
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Distance from another state in valence-arousal space
    #[must_use]
    pub fn distance(&self, other: &Self) -> f32 {
        let dv = self.valence - other.valence;
        let da = self.arousal - other.arousal;
        dv.hypot(da)
    }

    /// Intensity of current emotion
    #[must_use]
    pub fn intensity(&self) -> f32 {
        (self.valence.abs() + self.arousal) / 2.0
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Emotion transition (change in state)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EmotionTransition {
    /// delta valence.
    pub delta_valence: f32,
    /// delta arousal.
    pub delta_arousal: f32,
    /// delta dominance.
    pub delta_dominance: f32,
    /// duration.
    pub duration: u32,
}

impl EmotionTransition {
    #[must_use]
    /// new.
    pub const fn new(delta_valence: f32, delta_arousal: f32) -> Self {
        Self {
            delta_valence,
            delta_arousal,
            delta_dominance: 0.0,
            duration: 1,
        }
    }

    /// Scale transition by factor
    pub fn scale(&mut self, factor: f32) {
        self.delta_valence *= factor;
        self.delta_arousal *= factor;
        self.delta_dominance *= factor;
    }
}

/// Transition model for emotion dynamics
#[derive(Debug, Clone)]
pub struct TransitionModel {
    /// inertia.
    pub inertia: f32,
    /// momentum.
    pub momentum: EmotionTransition,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl TransitionModel {
    #[must_use]
    /// new.
    pub const fn new() -> Self {
        Self {
            inertia: 0.8,
            momentum: EmotionTransition::new(0.0, 0.0),
        }
    }

    /// Calculate emotion change from appraisal
    #[must_use]
    pub fn calculate_change(
        &self,
        current: &EmotionState,
        appraisal: &AppraisalResult,
    ) -> EmotionTransition {
        // Valence change based on pleasantness
        let delta_v = appraisal.pleasantness * (1.0 - current.valence.abs() * 0.5);

        // Arousal change based on goal relevance and urgency
        let delta_a = (appraisal.goal_relevance + appraisal.urgency) / 2.0;

        // Dominance change based on coping potential
        let delta_d = appraisal.coping_potential - 0.5;

        let mut transition = EmotionTransition {
            delta_valence: delta_v,
            delta_arousal: delta_a,
            delta_dominance: delta_d,
            duration: 1,
        };

        // Apply momentum (inertia from previous change)
        transition.delta_valence += self.momentum.delta_valence * self.inertia;
        transition.delta_arousal += self.momentum.delta_arousal * self.inertia;

        transition
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update momentum
    pub fn update_momentum(&mut self, transition: &EmotionTransition) {
        self.momentum = *transition;
    }
}

impl Default for TransitionModel {
    fn default() -> Self {
        Self::new()
    }
}

/// Linear interpolation
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emotion_state() {
        let state = EmotionState::new(0.5, 0.5);
        assert!(state.valence >= -1.0 && state.valence <= 1.0);
        assert!(state.arousal >= 0.0 && state.arousal <= 1.0);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    #[test]
    fn test_state_update() {
        let mut state = EmotionState::new(0.0, 0.5);
        let change = EmotionTransition::new(0.3, -0.1);
        state.update(change);

        assert!(state.valence > 0.0);
        assert!(state.arousal < 0.5);
    }

    #[test]
    fn test_decay() {
        let mut state = EmotionState::new(0.8, 0.8);
        state.decay_toward_baseline(0.0, 0.0, 0.1);

        assert!(state.valence < 0.8);
        assert!(state.arousal < 0.8);
    }

    #[test]
    fn test_classify() {
        let joy = EmotionState::new(0.8, 0.7);
        assert_eq!(joy.classify(), Emotion::Joy);

        let fear = EmotionState::new(-0.3, 0.9);
        assert_eq!(fear.classify(), Emotion::Fear);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let neutral = EmotionState::new(0.0, 0.0);
        assert_eq!(neutral.classify(), Emotion::Neutral);
    }

    #[test]
    fn test_emotion_state_clamping() {
        let state = EmotionState::new(2.0, 2.0);
        assert_eq!(state.valence, 1.0);
        assert_eq!(state.arousal, 1.0);

        let state = EmotionState::new(-2.0, -1.0);
        assert_eq!(state.valence, -1.0);
        assert_eq!(state.arousal, 0.0);
    }

    #[test]
    fn test_state_update_clamping() {
        let mut state = EmotionState::new(0.9, 0.9);
        let change = EmotionTransition::new(0.5, 0.5);
        state.update(change);
        assert_eq!(state.valence, 1.0);
        assert_eq!(state.arousal, 1.0);
    }

    #[test]
    fn test_distance() {
        let a = EmotionState::new(0.0, 0.0);
        let b = EmotionState::new(1.0, 0.0);
        assert!((a.distance(&b) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_intensity() {
        let state = EmotionState::new(1.0, 1.0);
        assert_eq!(state.intensity(), 1.0);

        let state = EmotionState::new(0.0, 0.0);
        assert_eq!(state.intensity(), 0.0);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_transition_scale() {
        let mut transition = EmotionTransition::new(1.0, 0.5);
        transition.scale(2.0);
        assert_eq!(transition.delta_valence, 2.0);
        assert_eq!(transition.delta_arousal, 1.0);
    }

    #[test]
    fn test_transition_model_calculate_change() {
        let model = TransitionModel::new();
        let state = EmotionState::new(0.0, 0.5);
        let appraisal = crate::emotion_dynamics::appraisal::AppraisalResult {
            pleasantness: 0.8,
            goal_relevance: 0.7,
            coping_potential: 0.6,
            urgency: 0.5,
            novelty: 0.3,
        };
        let change = model.calculate_change(&state, &appraisal);
        assert!(change.delta_valence > 0.0);
        assert!(change.delta_arousal > 0.0);
    }

    #[test]
    fn test_transition_model_update_momentum() {
        let mut model = TransitionModel::new();
        let transition = EmotionTransition::new(0.5, 0.3);
        model.update_momentum(&transition);
        assert_eq!(model.momentum.delta_valence, 0.5);
        assert_eq!(model.momentum.delta_arousal, 0.3);
    }
}
