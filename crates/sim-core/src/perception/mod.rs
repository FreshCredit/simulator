//! Multi-modal Perception Module (P1-AGENT-1)
//!
//! Visual, auditory, and text input processing for agents.

pub mod auditory;
pub mod text;
pub mod visual;

pub use auditory::{AuditoryPerception, SoundEvent};
pub use text::{TextInput, TextPerception};
pub use visual::{VisualField, VisualPerception};

use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Combined multi-modal perception
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiModalPerception {
    /// visual.
    pub visual: Option<VisualPerception>,
    /// auditory.
    pub auditory: Option<AuditoryPerception>,
    /// text.
    pub text: Option<TextPerception>,
    /// timestamp.
    pub timestamp: u64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl MultiModalPerception {
    /// Create new empty perception
    #[must_use]
    pub const fn new(timestamp: u64) -> Self {
        Self {
            visual: None,
            auditory: None,
            text: None,
            timestamp,
        }
    }

    /// Check if any modality has input
    #[must_use]
    pub const fn has_input(&self) -> bool {
        self.visual.is_some() || self.auditory.is_some() || self.text.is_some()
    }

    /// Get attention weight for each modality
    #[must_use]
    pub fn get_modality_salience(&self) -> ModalityWeights {
        ModalityWeights {
            visual: self.visual.as_ref().map_or(0.0, |v| v.salience),
            auditory: self.auditory.as_ref().map_or(0.0, |a| a.salience),
            text: self.text.as_ref().map_or(0.0, |t| t.salience),
        }
    }
}

/// Weights for each modality
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ModalityWeights {
    /// visual.
    pub visual: f32,
    /// auditory.
    pub auditory: f32,
    /// text.
    pub text: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl ModalityWeights {
    /// Normalize weights to sum to 1.0
    pub fn normalize(&mut self) {
        let sum = self.visual + self.auditory + self.text;
        if sum > 0.0 {
            self.visual /= sum;
            self.auditory /= sum;
            self.text /= sum;
        }
    }

    /// Get dominant modality
    #[must_use]
    pub fn dominant(&self) -> Modality {
        if self.visual >= self.auditory && self.visual >= self.text {
            Modality::Visual
        } else if self.auditory >= self.text {
            Modality::Auditory
        } else {
            Modality::Text
        }
    }
}

/// Modality type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modality {
    /// visual.
    Visual,
    /// auditory.
    Auditory,
    /// text.
    Text,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Perception configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionConfig {
    /// visual range.
    pub visual_range: f32,
    /// auditory range.
    pub auditory_range: f32,
    /// field of view.
    pub field_of_view: f32, // degrees
    /// attention capacity.
    pub attention_capacity: usize,
    /// noise threshold.
    pub noise_threshold: f32,
}

impl Default for PerceptionConfig {
    fn default() -> Self {
        Self {
            visual_range: 50.0,
            auditory_range: 100.0,
            field_of_view: 120.0,
            attention_capacity: 7,
            noise_threshold: 0.1,
        }
    }
}

#[cfg(test)]
mod tests {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    use super::*;

    #[test]
    fn test_modality_weights() {
        let mut weights = ModalityWeights {
            visual: 0.5,
            auditory: 0.3,
            text: 0.2,
        };
        weights.normalize();

        assert!((weights.visual + weights.auditory + weights.text - 1.0).abs() < 0.001);
        assert_eq!(weights.dominant(), Modality::Visual);
    }

    #[test]
    fn test_multi_modal_perception() {
        let perception = MultiModalPerception::new(0);
        assert!(!perception.has_input());

        let mut with_visual = MultiModalPerception::new(1);
        with_visual.visual = Some(VisualPerception {
            objects: vec![],
            salience: 0.8,
            field_width: 100,
            field_height: 100,
        });
        assert!(with_visual.has_input());
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_modality_weights_normalize() {
        let mut weights = ModalityWeights {
            visual: 0.0,
            auditory: 0.0,
            text: 0.0,
        };
        weights.normalize();
        assert_eq!(weights.visual, 0.0);

        let mut weights = ModalityWeights {
            visual: 1.0,
            auditory: 1.0,
            text: 1.0,
        };
        weights.normalize();
        assert!((weights.visual - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_modality_weights_dominant() {
        let visual_dominant = ModalityWeights {
            visual: 0.8,
            auditory: 0.1,
            text: 0.1,
        };
        assert_eq!(visual_dominant.dominant(), Modality::Visual);

        let auditory_dominant = ModalityWeights {
            visual: 0.1,
            auditory: 0.8,
            text: 0.1,
        };
        assert_eq!(auditory_dominant.dominant(), Modality::Auditory);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let text_dominant = ModalityWeights {
            visual: 0.1,
            auditory: 0.1,
            text: 0.8,
        };
        assert_eq!(text_dominant.dominant(), Modality::Text);
    }

    #[test]
    fn test_perception_config_default() {
        let config = PerceptionConfig::default();
        assert_eq!(config.visual_range, 50.0);
        assert_eq!(config.auditory_range, 100.0);
        assert_eq!(config.field_of_view, 120.0);
        assert_eq!(config.attention_capacity, 7);
        assert_eq!(config.noise_threshold, 0.1);
    }

    #[test]
    fn test_multi_modal_salience() {
        let mut perception = MultiModalPerception::new(0);
        perception.visual = Some(VisualPerception {
            objects: vec![],
            salience: 0.9,
            field_width: 100,
            field_height: 100,
        });

        let salience = perception.get_modality_salience();
        assert!((salience.visual - 0.9).abs() < 0.001);
        assert_eq!(salience.auditory, 0.0);
    }
}
