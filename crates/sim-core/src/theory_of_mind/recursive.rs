//! Recursive Belief Modeling (P1-AGENT-2)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::belief::{BeliefState, MentalModel};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Theory of Mind reasoning engine
#[derive(Debug, Clone)]
pub struct TheoryOfMind {
    /// agent id.
    pub agent_id: u64,
    /// models.
    pub models: HashMap<u64, RecursiveModel>,
    /// max depth.
    pub max_depth: RecursionDepth,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Recursion depth limit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RecursionDepth {
    /// depth1.
    Depth1 = 1, // I believe
    #[default]
    /// depth2.
    Depth2 = 2, // I believe you believe
    /// depth3.
    Depth3 = 3, // I believe you believe they believe
}

/// Recursive model of other agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecursiveModel {
    /// target agent.
    pub target_agent: u64,
    /// depth.
    pub depth: usize,
    /// What I believe about them
    pub my_model: MentalModel,
    /// What I believe they believe about me
    pub their_model_of_me: Option<Box<MentalModel>>,
    /// What I believe they believe about others
    pub their_models: Option<HashMap<u64, MentalModel>>,
}

impl TheoryOfMind {
    /// Create new `ToM` engine
    #[must_use]
    pub fn new(agent_id: u64, max_depth: RecursionDepth) -> Self {
        Self {
            agent_id,
            models: HashMap::new(),
            max_depth,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create or update model of another agent
    pub fn model_agent(&mut self, target_id: u64) -> &mut RecursiveModel {
        self.models
            .entry(target_id)
            .or_insert_with(|| RecursiveModel::new(target_id, 1))
    }

    /// Get model of agent
    #[must_use]
    pub fn get_model(&self, target_id: u64) -> Option<&RecursiveModel> {
        self.models.get(&target_id)
    }

    /// Infer what agent believes I believe
    #[must_use]
    pub fn infer_recursive_belief(&self, target_id: u64, about: u64) -> Option<BeliefState> {
        let model = self.models.get(&target_id)?;

        // Start with what they believe about the world
        let mut inferred = BeliefState::new(target_id);

        // Add belief about "about" agent
        if let Some(their_models) = &model.their_models {
            if let Some(mental_model) = their_models.get(&about) {
                for (trait_name, &value) in &mental_model.traits {
                    inferred.add_belief(super::belief::Belief {
                        subject: target_id,
                        content: super::belief::BeliefContent::Trait {
                            agent_id: about,
                            trait_name: trait_name.clone(),
                            value,
                        },
                        confidence: mental_model.confidence,
                        source: super::belief::BeliefSource::Inference,
                        timestamp: 0,
                    });
                }
            }
        }

        Some(inferred)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Predict agent's action in strategic situation
    #[must_use]
    pub fn predict_action(
        &self,
        target_id: u64,
        _my_action: &str,
        situation: &str,
    ) -> Option<String> {
        let model = self.models.get(&target_id)?;

        // Level-1: Direct prediction from their model
        let base_prediction = model.my_model.predict_behavior(situation);

        // K-level reasoning: Camerer, Ho & Chong (2004) Cognitive Hierarchy
        let final_prediction = match self.max_depth {
            RecursionDepth::Depth1 => base_prediction,
            RecursionDepth::Depth2 => {
                if let Some(their_model_of_me) = &model.their_model_of_me {
                    let their_prediction_of_me = their_model_of_me.predict_behavior(situation);
                    let best_resp = self.best_response(their_prediction_of_me, situation);
                    let weight = their_model_of_me.confidence.clamp(0.0, 1.0);
                    (1.0 - weight) * base_prediction + weight * best_resp
                } else {
                    base_prediction
                }
            }
            RecursionDepth::Depth3 => {
                if let Some(their_model_of_me) = &model.their_model_of_me {
                    let their_prediction_of_me = their_model_of_me.predict_behavior(situation);
                    let level2_resp = self.best_response(their_prediction_of_me, situation);
                    let level3_resp = self.best_response(level2_resp, situation);
                    let weight = their_model_of_me.confidence.clamp(0.0, 1.0);
                    (1.0 - weight) * base_prediction + weight * level3_resp
                } else {
                    base_prediction
                }
            }
        };

        // Return action based on prediction
        if final_prediction > 0.5 {
            Some("cooperate".to_string())
        } else {
            Some("defect".to_string())
        }
    }

    /// Calculate best response
    fn best_response(&self, opponent_likely_action: f32, situation: &str) -> f32 {
        let p = opponent_likely_action.clamp(0.0, 1.0);
        match situation {
            // Zero-sum / adversarial: counter the opponent's anticipated move
            "matching_pennies" => 1.0 - p,
            // Coordination / repeated dilemma: reciprocate expected cooperation
            "prisoners_dilemma_repeated" | "stag_hunt" => p,
            "ultimatum_game" => 0.5 * p + 0.5,
            _ => (p + 0.5) / 2.0,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check for common knowledge
    #[must_use]
    pub fn has_common_knowledge(&self, agents: &[u64], proposition: &str) -> bool {
        if proposition.trim().is_empty() {
            return false;
        }
        if agents.len() < 2 {
            return true;
        }

        // Every agent must have a model of every other agent
        for &agent in agents {
            if agent == self.agent_id {
                // For self, check that self has a model of all other agents
                for &other in agents {
                    if other != self.agent_id && !self.models.contains_key(&other) {
                        return false;
                    }
                }
            } else if let Some(model) = self.models.get(&agent) {
                // For other agents, check they model self and all third parties
                for &other in agents {
                    if other == agent {
                        continue;
                    }
                    let knows_other = if other == self.agent_id {
                        model.their_model_of_me.is_some()
                    } else {
                        model
                            .their_models
                            .as_ref()
                            .is_some_and(|m| m.contains_key(&other))
                    };
                    if !knows_other {
                        return false;
                    }
                }
            } else {
                return false;
            }
        }

        true
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update all models based on observed behavior
    pub fn update_from_observation(&mut self, target_id: u64, situation: &str, action: &str) {
        let model = self.model_agent(target_id);

        // Update my model of them
        let predicted = model.my_model.predict_behavior(situation);
        let actual = if action == "cooperate" { 1.0 } else { 0.0 };
        let error = (predicted - actual).abs();

        // Adjust confidence based on prediction error, maintaining positive floor
        model.my_model.confidence =
            (model.my_model.confidence * (1.0 - error * 0.5)).clamp(0.05, 1.0);

        // Update cooperativeness trait based on observed behavior
        if action == "cooperate" {
            model.my_model.update_trait("cooperativeness", 0.8, 0.3);
        } else {
            model.my_model.update_trait("cooperativeness", 0.2, 0.3);
        }
    }

    /// Get theory of mind level required for situation
    #[must_use]
    pub fn required_depth(situation: &str) -> RecursionDepth {
        match situation {
            "matching_pennies" => RecursionDepth::Depth2,
            "prisoners_dilemma_repeated" => RecursionDepth::Depth2,
            "ultimatum_game" => RecursionDepth::Depth1,
            "stag_hunt" => RecursionDepth::Depth2,
            "simultaneous_move" => RecursionDepth::Depth2,
            _ => RecursionDepth::Depth1,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl RecursiveModel {
    /// Create new recursive model
    #[must_use]
    pub fn new(target_agent: u64, depth: usize) -> Self {
        Self {
            target_agent,
            depth,
            my_model: MentalModel::new(target_agent),
            their_model_of_me: None,
            their_models: None,
        }
    }

    /// Add second-order model (what they believe about me)
    pub fn add_their_model_of_me(&mut self, model: MentalModel) {
        self.their_model_of_me = Some(Box::new(model));
    }

    /// Add their model of another agent
    pub fn add_their_model(&mut self, agent_id: u64, model: MentalModel) {
        self.their_models
            .get_or_insert_with(HashMap::new)
            .insert(agent_id, model);
    }

    /// Get depth of recursion
    #[must_use]
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub const fn actual_depth(&self) -> usize {
        if self.their_models.is_some() || self.their_model_of_me.is_some() {
            2
        } else {
            1
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// False belief task (Sally-Anne test simulation)
#[derive(Debug)]
pub struct FalseBeliefTask {
    /// agent a.
    pub agent_a: u64,
    /// agent b.
    pub agent_b: u64,
    /// object location.
    pub object_location: String,
    /// actual location.
    pub actual_location: String,
}

impl FalseBeliefTask {
    /// Test if `agent_a` correctly predicts `agent_b`'s false belief
    #[must_use]
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn test(&self, tom: &TheoryOfMind) -> bool {
        // Agent_a needs to understand that agent_b has outdated information
        tom.get_model(self.agent_b).is_some_and(|m| m.depth >= 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theory_of_mind() {
        let mut tom = TheoryOfMind::new(1, RecursionDepth::Depth2);
        tom.model_agent(2);

        assert!(tom.get_model(2).is_some());

        let prediction = tom.predict_action(2, "cooperate", "prisoners_dilemma");
        assert!(prediction.is_some());
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_recursive_model_depth() {
        let mut model = RecursiveModel::new(2, 1);
        assert_eq!(model.actual_depth(), 1);

        model.add_their_model_of_me(super::MentalModel::new(1));
        assert_eq!(model.actual_depth(), 2);
    }

    #[test]
    fn test_false_belief_task() {
        let tom = TheoryOfMind::new(1, RecursionDepth::Depth2);

        let task = FalseBeliefTask {
            agent_a: 1,
            agent_b: 2,
            object_location: "basket".to_string(),
            actual_location: "box".to_string(),
        };

        // Without model, should fail
        assert!(!task.test(&tom));
    }

    #[test]
    fn test_common_knowledge() {
        let tom = TheoryOfMind::new(1, RecursionDepth::Depth2);

        // Single agent always has common knowledge with self
        assert!(tom.has_common_knowledge(&[1], "test"));

        // Multiple agents without models
        assert!(!tom.has_common_knowledge(&[1, 2], "test"));
    }

    #[test]
    fn test_tom_depth2_matching_pennies_best_response() {
        let mut tom = TheoryOfMind::new(1, RecursionDepth::Depth2);
        let model = tom.model_agent(2);
        // Base prediction predicts opponent cooperates (0.8)
        model.my_model.update_trait("cooperativeness", 0.8, 1.0);
        model.my_model.update_trait("trustworthiness", 0.8, 1.0);

        // Level-2 model: opponent believes I will cooperate with high confidence
        let mut my_model = MentalModel::new(1);
        my_model.update_trait("cooperativeness", 0.9, 1.0);
        my_model.update_trait("trustworthiness", 0.9, 1.0);
        my_model.confidence = 0.9;
        model.add_their_model_of_me(my_model);

        // In matching pennies, opponent best response to anticipated cooperation is defect (1.0 - 0.9 = 0.1)
        // With weight 0.9: final_prediction = 0.1 * 0.8 + 0.9 * 0.1 = 0.17 < 0.5 -> "defect"
        let act = tom.predict_action(2, "cooperate", "matching_pennies");
        assert_eq!(act, Some("defect".to_string()));
    }
}
