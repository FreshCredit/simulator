//! Belief Inference Engine (P1-AGENT-2)

use serde::{Deserialize, Serialize};

use super::belief::{Belief, BeliefState};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Inference engine for belief reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceEngine {
    /// rules.
    pub rules: Vec<InferenceRule>,
}

/// Rule for belief inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRule {
    /// name.
    pub name: String,
    /// condition.
    pub condition: InferenceCondition,
    /// conclusion type.
    pub conclusion_type: ConclusionType,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Type of conclusion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConclusionType {
    /// trait belief.
    TraitBelief {
        /// agent id.
        agent_id: u64,
        /// trait name.
        trait_name: String,
        /// value.
        value: f32,
    },
    /// confidence update.
    ConfidenceUpdate {
        /// belief id.
        belief_id: u64,
        /// new confidence.
        new_confidence: f32,
    },
}

/// Condition for inference
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InferenceCondition {
    /// Field.
    /// has belief.
    HasBelief {
        /// content pattern.
        content_pattern: String,
    },
    /// Field.
    /// confidence above.
    ConfidenceAbove {
        /// threshold.
        threshold: f32,
    },
    /// and.
    And(Box<InferenceCondition>, Box<InferenceCondition>),
    /// or.
    Or(Box<InferenceCondition>, Box<InferenceCondition>),
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Belief inference methods
pub trait BeliefInference {
    /// Infer beliefs from observations
    fn infer_from_observation(&self, observation: &Observation) -> Vec<Belief>;

    /// Update beliefs based on communication
    fn update_from_communication(&mut self, message: &Message, source: u64);

    /// Apply theory of mind reasoning
    fn tom_inference(&self, target_agent: u64) -> BeliefState;
}

/// Observation of behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// agent id.
    pub agent_id: u64,
    /// action.
    pub action: String,
    /// context.
    pub context: String,
    /// timestamp.
    pub timestamp: u64,
}

/// Communicated message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// content.
    pub content: String,
    /// truth value.
    pub truth_value: Option<bool>,
    /// confidence.
    pub confidence: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl InferenceEngine {
    /// Create new inference engine
    #[must_use]
    pub const fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add inference rule
    pub fn add_rule(&mut self, rule: InferenceRule) {
        self.rules.push(rule);
    }

    /// Check if condition is satisfied
    #[allow(clippy::only_used_in_recursion)]
    #[must_use]
    pub fn check_condition(&self, condition: &InferenceCondition, state: &BeliefState) -> bool {
        match condition {
            InferenceCondition::HasBelief { content_pattern } => state
                .beliefs
                .iter()
                .any(|b| format!("{:?}", b.content).contains(content_pattern)),
            InferenceCondition::ConfidenceAbove { threshold } => {
                state.beliefs.iter().any(|b| b.confidence >= *threshold)
            }
            InferenceCondition::And(c1, c2) => {
                self.check_condition(c1, state) && self.check_condition(c2, state)
            }
            InferenceCondition::Or(c1, c2) => {
                self.check_condition(c1, state) || self.check_condition(c2, state)
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Infer beliefs using rules
    #[must_use]
    pub fn infer(&self, state: &BeliefState, subject: u64) -> Vec<Belief> {
        let mut new_beliefs = Vec::new();

        for rule in &self.rules {
            if self.check_condition(&rule.condition, state) {
                let belief = self.apply_conclusion(&rule.conclusion_type, subject);
                if let Some(b) = belief {
                    new_beliefs.push(b);
                }
            }
        }

        new_beliefs
    }

    /// Apply conclusion type to generate belief
    fn apply_conclusion(&self, conclusion: &ConclusionType, subject: u64) -> Option<Belief> {
        match conclusion {
            ConclusionType::TraitBelief {
                agent_id,
                trait_name,
                value,
            } => Some(Belief {
                subject,
                content: super::belief::BeliefContent::Trait {
                    agent_id: *agent_id,
                    trait_name: trait_name.clone(),
                    value: *value,
                },
                confidence: 0.7,
                source: super::belief::BeliefSource::Inference,
                timestamp: 0,
            }),
            ConclusionType::ConfidenceUpdate { .. } => None,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Bayes update of belief confidence, strictly bounded to [0.0, 1.0] probability interval
    #[must_use]
    pub fn bayes_update(prior: f32, likelihood: f32, marginal: f32) -> f32 {
        if marginal <= 1e-12
            || !marginal.is_finite()
            || !prior.is_finite()
            || !likelihood.is_finite()
        {
            return prior.clamp(0.0, 1.0);
        }
        ((likelihood * prior) / marginal).clamp(0.0, 1.0)
    }

    /// Abductive inference (best explanation)
    #[must_use]
    pub fn abduce(
        &self,
        _observation: &Observation,
        possible_explanations: &[String],
    ) -> Option<String> {
        // Simplified - return most likely explanation
        possible_explanations.first().cloned()
    }
}

impl Default for InferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bayes_update() {
        let prior = 0.5;
        let likelihood = 0.8;
        let marginal = 0.6;

        let posterior = InferenceEngine::bayes_update(prior, likelihood, marginal);
        assert!(posterior > 0.0 && posterior <= 1.0);
    }

    #[test]
    fn test_bayes_update_boundary_clamping_and_subnormal_protection() {
        // Exceeding 1.0 is clamped
        let over = InferenceEngine::bayes_update(0.8, 0.9, 0.1);
        assert!((over - 1.0).abs() < f32::EPSILON);

        // Zero or negative marginal fails soft to clamped prior
        let zero_marginal = InferenceEngine::bayes_update(0.4, 0.9, 0.0);
        assert!((zero_marginal - 0.4).abs() < f32::EPSILON);

        let neg_marginal = InferenceEngine::bayes_update(0.4, 0.9, -0.5);
        assert!((neg_marginal - 0.4).abs() < f32::EPSILON);

        // NaN fails soft
        let nan_update = InferenceEngine::bayes_update(0.4, f32::NAN, 0.5);
        assert!((nan_update - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn test_inference_condition() {
        let engine = InferenceEngine::new();
        let state = BeliefState::new(1);

        let condition = InferenceCondition::ConfidenceAbove { threshold: 0.5 };
        assert!(!engine.check_condition(&condition, &state));
    }
}
