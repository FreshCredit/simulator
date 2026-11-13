//! Emotion Regulation (P1-AGENT-3)

use serde::{Deserialize, Serialize};

use super::transitions::EmotionTransition;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Emotion regulation system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionRegulation {
    /// strategies.
    pub strategies: Vec<RegulationStrategy>,
    /// base strength.
    pub base_strength: f32,
}

impl EmotionRegulation {
    /// Create new regulation system
    #[must_use]
    pub fn new(base_strength: f32) -> Self {
        Self {
            strategies: vec![
                RegulationStrategy::Reappraisal,
                RegulationStrategy::Suppression,
            ],
            base_strength,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Regulate an emotion transition
    #[must_use]
    pub fn regulate(&self, transition: EmotionTransition) -> EmotionTransition {
        let mut regulated = transition;

        for strategy in &self.strategies {
            regulated = strategy.apply(regulated, self.base_strength);
        }

        regulated
    }

    /// Add regulation strategy
    pub fn add_strategy(&mut self, strategy: RegulationStrategy) {
        self.strategies.push(strategy);
    }

    /// Remove extreme emotional responses
    #[must_use]
    pub fn dampen_extremes(&self, transition: EmotionTransition) -> EmotionTransition {
        let dampening = 1.0 - self.base_strength * 0.3;

        EmotionTransition {
            delta_valence: transition.delta_valence * dampening,
            delta_arousal: transition.delta_arousal * dampening,
            delta_dominance: transition.delta_dominance * dampening,
            duration: transition.duration,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Regulation strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegulationStrategy {
    /// Reinterpret the situation
    Reappraisal,
    /// Inhibit emotional expression
    Suppression,
    /// Redirect attention
    AttentionDeployment,
    /// Change the situation
    SituationSelection,
    /// Modify the situation
    SituationModification,
}

impl RegulationStrategy {
    /// Apply strategy to transition
    #[must_use]
    pub fn apply(&self, transition: EmotionTransition, strength: f32) -> EmotionTransition {
        match self {
            Self::Reappraisal => {
                // Reduces intensity while preserving direction
                let factor = 1.0 - strength * 0.3;
                EmotionTransition {
                    delta_valence: transition.delta_valence * factor,
                    delta_arousal: transition.delta_arousal * factor,
                    delta_dominance: transition.delta_dominance,
                    duration: transition.duration + 1,
                }
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            }
            Self::Suppression => {
                // Reduces arousal more than valence
                EmotionTransition {
                    delta_valence: transition.delta_valence * (1.0 - strength * 0.2),
                    delta_arousal: transition.delta_arousal * (1.0 - strength * 0.5),
                    delta_dominance: transition.delta_dominance + strength * 0.1,
                    duration: transition.duration,
                }
            }
            Self::AttentionDeployment => {
                // Delays and reduces intensity
                EmotionTransition {
                    delta_valence: transition.delta_valence * (1.0 - strength * 0.4),
                    delta_arousal: transition.delta_arousal * (1.0 - strength * 0.4),
                    delta_dominance: transition.delta_dominance,
                    duration: transition.duration + (strength * 2.0) as u32,
                }
            }
            Self::SituationSelection => {
                // Strong reduction, takes time
                let factor = 1.0 - strength * 0.5;
                EmotionTransition {
                    delta_valence: transition.delta_valence * factor,
                    delta_arousal: transition.delta_arousal * factor,
                    delta_dominance: transition.delta_dominance + strength * 0.2,
                    duration: transition.duration + (strength * 3.0) as u32,
                }
            }
            Self::SituationModification => {
                // TAG: surface=simulation owner=research-team rule=SIM-001
                // Moderate reduction
                let factor = 1.0 - strength * 0.35;
                EmotionTransition {
                    delta_valence: transition.delta_valence * factor,
                    delta_arousal: transition.delta_arousal * factor,
                    delta_dominance: transition.delta_dominance + strength * 0.15,
                    duration: transition.duration + 1,
                }
            }
        }
    }

    /// Energy cost of applying strategy
    #[must_use]
    pub const fn energy_cost(&self) -> f32 {
        match self {
            Self::Reappraisal => 0.3,
            Self::Suppression => 0.4,
            Self::AttentionDeployment => 0.2,
            Self::SituationSelection => 0.5,
            Self::SituationModification => 0.4,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Emotion regulation success metrics
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RegulationMetrics {
    /// success rate.
    pub success_rate: f32,
    /// average latency.
    pub average_latency: f32,
    /// energy expenditure.
    pub energy_expenditure: f32,
    /// goal achievement.
    pub goal_achievement: f32,
}

impl Default for RegulationMetrics {
    fn default() -> Self {
        Self {
            success_rate: 0.7,
            average_latency: 2.0,
            energy_expenditure: 0.3,
            goal_achievement: 0.6,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::transitions::EmotionTransition;
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    use super::*;

    #[test]
    fn test_reappraisal() {
        let strategy = RegulationStrategy::Reappraisal;
        let transition = EmotionTransition::new(0.8, 0.6);
        let regulated = strategy.apply(transition, 0.5);

        assert!(regulated.delta_valence < transition.delta_valence);
        assert!(regulated.duration > transition.duration);
    }

    #[test]
    fn test_suppression() {
        let strategy = RegulationStrategy::Suppression;
        let transition = EmotionTransition::new(0.8, 0.6);
        let regulated = strategy.apply(transition, 0.5);

        // Suppression should reduce arousal more than valence
        let arousal_reduction = 1.0 - regulated.delta_arousal / transition.delta_arousal;
        let valence_reduction = 1.0 - regulated.delta_valence / transition.delta_valence;
        assert!(arousal_reduction > valence_reduction);
    }

    #[test]
    fn test_regulation_system() {
        let regulation = EmotionRegulation::new(0.5);
        let transition = EmotionTransition::new(1.0, 0.8);
        let regulated = regulation.regulate(transition);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(regulated.delta_valence <= transition.delta_valence);
    }

    #[test]
    fn test_attention_deployment() {
        let strategy = RegulationStrategy::AttentionDeployment;
        let transition = EmotionTransition::new(0.8, 0.6);
        let regulated = strategy.apply(transition, 0.5);

        assert!(regulated.delta_valence < transition.delta_valence);
        assert!(regulated.duration > transition.duration);
    }

    #[test]
    fn test_situation_selection() {
        let strategy = RegulationStrategy::SituationSelection;
        let transition = EmotionTransition::new(0.8, 0.6);
        let regulated = strategy.apply(transition, 0.5);

        assert!(regulated.delta_valence < transition.delta_valence);
        assert!(regulated.delta_dominance > transition.delta_dominance);
    }

    #[test]
    fn test_situation_modification() {
        let strategy = RegulationStrategy::SituationModification;
        let transition = EmotionTransition::new(0.8, 0.6);
        let regulated = strategy.apply(transition, 0.5);

        assert!(regulated.delta_valence < transition.delta_valence);
    }

    #[test]
    fn test_energy_costs() {
        assert_eq!(RegulationStrategy::Reappraisal.energy_cost(), 0.3);
        assert_eq!(RegulationStrategy::Suppression.energy_cost(), 0.4);
        assert_eq!(RegulationStrategy::AttentionDeployment.energy_cost(), 0.2);
        assert_eq!(RegulationStrategy::SituationSelection.energy_cost(), 0.5);
        assert_eq!(RegulationStrategy::SituationModification.energy_cost(), 0.4);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_dampen_extremes() {
        let regulation = EmotionRegulation::new(0.5);
        let transition = EmotionTransition::new(1.0, 0.8);
        let dampened = regulation.dampen_extremes(transition);

        assert!(dampened.delta_valence.abs() <= transition.delta_valence.abs());
        assert!(dampened.delta_arousal.abs() <= transition.delta_arousal.abs());
    }

    #[test]
    fn test_add_strategy() {
        let mut regulation = EmotionRegulation::new(0.5);
        assert_eq!(regulation.strategies.len(), 2);
        regulation.add_strategy(RegulationStrategy::AttentionDeployment);
        assert_eq!(regulation.strategies.len(), 3);
    }

    #[test]
    fn test_regulation_metrics_default() {
        let metrics = RegulationMetrics::default();
        assert_eq!(metrics.success_rate, 0.7);
        assert_eq!(metrics.average_latency, 2.0);
        assert_eq!(metrics.energy_expenditure, 0.3);
        assert_eq!(metrics.goal_achievement, 0.6);
    }
}
