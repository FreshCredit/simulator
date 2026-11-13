//! Belief Representation (P1-AGENT-2)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// A belief held by an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Belief {
    /// subject.
    pub subject: u64, // Who holds this belief
    /// content.
    pub content: BeliefContent,
    /// confidence.
    pub confidence: f32, // 0.0 to 1.0
    /// source.
    pub source: BeliefSource,
    /// timestamp.
    pub timestamp: u64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Content of a belief
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BeliefContent {
    /// Belief about world state
    WorldState {
        /// proposition.
        proposition: String,
        /// value.
        value: bool,
    },
    /// Belief about another agent's trait
    Trait {
        /// agent id.
        agent_id: u64,
        /// trait name.
        trait_name: String,
        /// value.
        value: f32,
    },
    /// Belief about another agent's intention
    Intention {
        /// agent id.
        agent_id: u64,
        /// action.
        action: String,
        /// likelihood.
        likelihood: f32,
    },
    /// Belief about another's belief (higher-order)
    MetaBelief {
        /// about.
        about: u64,
        /// inner.
        inner: Box<BeliefContent>,
    },
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Source of belief
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BeliefSource {
    /// observation.
    Observation,
    /// inference.
    Inference,
    /// Field.
    /// communication.
    Communication {
        /// from.
        from: u64,
    },
    /// default.
    Default,
}

/// Complete belief state for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeliefState {
    /// agent id.
    pub agent_id: u64,
    /// beliefs.
    pub beliefs: Vec<Belief>,
    /// uncertainty.
    pub uncertainty: f32,
}

impl BeliefState {
    #[must_use]
    /// new.
    pub const fn new(agent_id: u64) -> Self {
        Self {
            agent_id,
            beliefs: Vec::new(),
            uncertainty: 0.5,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Add a belief
    pub fn add_belief(&mut self, belief: Belief) {
        self.beliefs.push(belief);
    }

    /// Get beliefs about a specific agent
    #[must_use]
    pub fn beliefs_about(&self, agent_id: u64) -> Vec<&Belief> {
        self.beliefs
            .iter()
            .filter(|b| match &b.content {
                BeliefContent::Trait { agent_id: id, .. } => *id == agent_id,
                BeliefContent::Intention { agent_id: id, .. } => *id == agent_id,
                BeliefContent::MetaBelief { about, .. } => *about == agent_id,
                _ => false,
            })
            .collect()
    }

    /// Get confidence-weighted belief about trait
    #[must_use]
    pub fn trait_belief(&self, agent_id: u64, trait_name: &str) -> Option<f32> {
        let matches: Vec<_> = self
            .beliefs
            .iter()
            .filter(|b| {
                matches!(&b.content,
                BeliefContent::Trait { agent_id: id, trait_name: t, .. }
                if *id == agent_id && t == trait_name)
            })
            .collect();

        if matches.is_empty() {
            return None;
        }

        let mut weighted_sum = 0.0f32;
        let mut total_confidence = 0.0f32;
        let mut unweighted_sum = 0.0f32;
        let count = matches.len() as f32;

        for b in &matches {
            if let BeliefContent::Trait { value, .. } = b.content {
                let conf = if b.confidence.is_finite() && b.confidence >= 0.0 {
                    b.confidence
                } else {
                    0.0
                };
                let safe_val = if value.is_finite() { value } else { 0.5 };
                weighted_sum += safe_val * conf;
                total_confidence += conf;
                unweighted_sum += safe_val;
            }
        }

        if total_confidence > 1e-12 {
            Some((weighted_sum / total_confidence).clamp(0.0, 1.0))
        } else if count > 0.0 {
            Some((unweighted_sum / count).clamp(0.0, 1.0))
        } else {
            None
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update uncertainty based on belief consistency
    pub fn update_uncertainty(&mut self) {
        let n = self.beliefs.len() as f32;
        if n < 2.0 {
            self.uncertainty = 0.5;
            return;
        }

        // Simple uncertainty based on confidence variance
        let mean_conf: f32 = self.beliefs.iter().map(|b| b.confidence).sum::<f32>() / n;
        let variance: f32 = self
            .beliefs
            .iter()
            .map(|b| (b.confidence - mean_conf).powi(2))
            .sum::<f32>()
            / n;

        self.uncertainty = (1.0 - mean_conf) + variance.sqrt();
        self.uncertainty = self.uncertainty.clamp(0.0, 1.0);
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Mental model of another agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentalModel {
    /// target agent.
    pub target_agent: u64,
    /// Believed traits
    pub traits: HashMap<String, f32>,
    /// Believed preferences
    pub preferences: HashMap<String, f32>,
    /// Model confidence
    pub confidence: f32,
    /// Interaction history
    pub interaction_count: u32,
}

impl MentalModel {
    #[must_use]
    /// new.
    pub fn new(target_agent: u64) -> Self {
        Self {
            target_agent,
            traits: HashMap::new(),
            preferences: HashMap::new(),
            confidence: 0.5,
            interaction_count: 0,
        }
    }

    /// Update trait belief
    pub fn update_trait(&mut self, trait_name: &str, value: f32, weight: f32) {
        let current = self.traits.get(trait_name).copied().unwrap_or(0.5);
        let new_value = current * (1.0 - weight) + value * weight;
        self.traits.insert(trait_name.to_string(), new_value);
        self.interaction_count += 1;
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Predict behavior based on traits
    #[must_use]
    pub fn predict_behavior(&self, situation: &str) -> f32 {
        // Simple prediction using trait activation
        let trust = self.traits.get("trustworthiness").copied().unwrap_or(0.5);
        let cooperation = self.traits.get("cooperativeness").copied().unwrap_or(0.5);

        match situation {
            "prisoners_dilemma" => cooperation * trust,
            "ultimatum" => cooperation * 0.8 + trust * 0.2,
            _ => (trust + cooperation) / 2.0,
        }
    }

    /// Calculate model accuracy against actual behavior
    #[must_use]
    pub fn accuracy(&self, predicted: f32, actual: f32) -> f32 {
        1.0 - (predicted - actual).abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_belief_state() {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let mut state = BeliefState::new(1);
        state.add_belief(Belief {
            subject: 1,
            content: BeliefContent::Trait {
                agent_id: 2,
                trait_name: "trustworthy".to_string(),
                value: 0.8,
            },
            confidence: 0.9,
            source: BeliefSource::Observation,
            timestamp: 0,
        });

        let belief = state.trait_belief(2, "trustworthy");
        assert!(belief.is_some());
        assert!((belief.unwrap() - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_mental_model() {
        let mut model = MentalModel::new(2);
        model.update_trait("trustworthiness", 0.8, 0.5);

        let prediction = model.predict_behavior("prisoners_dilemma");
        assert!((0.0..=1.0).contains(&prediction));
    }

    #[test]
    fn test_trait_belief_zero_confidence_no_nan() {
        let mut state = BeliefState::new(1);
        state.add_belief(Belief {
            subject: 1,
            content: BeliefContent::Trait {
                agent_id: 2,
                trait_name: "cooperation".to_string(),
                value: 0.75,
            },
            confidence: 0.0,
            source: BeliefSource::Default,
            timestamp: 0,
        });
        let val = state.trait_belief(2, "cooperation");
        assert!(val.is_some());
        let res = val.unwrap();
        assert!(!res.is_nan());
        assert!((res - 0.75).abs() < 1e-5);
    }
}
