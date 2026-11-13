// TAG: surface=simulation owner=research-team rule=SIM-001
//! Method implementations for EmbodiedCognition, BodySchema, and ExtendedMind.

use super::types::*;
use std::collections::HashMap;

impl EmbodiedCognition {
    /// Default body ownership level
    pub const DEFAULT_BODY_OWNERSHIP: f32 = 1.0;
    /// Threshold for extended cognition (transparency)
    pub const EXTENDED_COGNITION_THRESHOLD: f32 = 0.7;
    /// Rubber hand illusion distance threshold
    pub const RHI_DISTANCE_THRESHOLD: f32 = 0.3;
    /// Maximum sensorimotor contingencies to store
    pub const MAX_CONTINGENCIES: usize = 100;
    /// Maximum affordances to track
    pub const MAX_AFFORDANCES: usize = 50;

    /// Create new embodied cognition system
    #[must_use]
    pub fn new() -> Self {
        let mut body_parts = HashMap::new();
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        for part in [
            BodyPart::Head,
            BodyPart::Torso,
            BodyPart::LeftArm,
            BodyPart::RightArm,
            BodyPart::LeftHand,
            BodyPart::RightHand,
            BodyPart::LeftLeg,
            BodyPart::RightLeg,
            BodyPart::LeftFoot,
            BodyPart::RightFoot,
        ] {
            body_parts.insert(part, BodyPartState::default());
        }

        Self {
            body_schema: BodySchema {
                body_parts,
                proprioception: ProprioceptiveMap::default(),
                body_image: BodyImage::default(),
                action_capabilities: ActionCapabilities::default(),
            },
            sensorimotor_contingencies: Vec::new(),
            affordances: Vec::new(),
            body_ownership: Self::DEFAULT_BODY_OWNERSHIP,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            extended_mind: ExtendedMind::default(),
            perceptual_mode: PerceptualMode::Exploratory,
        }
    }

    /// Update body schema from sensory input
    pub fn update_body_schema(&mut self, sensory_input: &SensoryInput) {
        // Update limb positions from proprioceptive feedback
        for (part, position) in &sensory_input.hand_positions {
            if let Some(state) = self.body_schema.body_parts.get_mut(part) {
                state.position = *position;
            }
        }

        // Update body image (slow, conscious process)
        self.update_body_image();
    }

    /// Update body image based on capabilities
    fn update_body_image(&mut self) {
        let actual_strength = self.body_schema.action_capabilities.max_carry_weight;
        let ideal_strength = 50.0f32.max(1.0); // kg
        self.body_schema.body_image.perceived_strength =
            (actual_strength / ideal_strength).min(1.0);
        self.body_schema.body_image.body_satisfaction = (actual_strength / ideal_strength).min(1.0);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Detect affordances in environment
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn detect_affordances(&mut self, sensory_input: &SensoryInput) {
        self.affordances.clear();

        for object in &sensory_input.objects {
            match object.object_type {
                ObjectType::Chair => {
                    // Chair: affords sitting if at appropriate height
                    if object.height < 0.5 && object.height > 0.3 {
                        self.affordances.push(Affordance {
                            object_id: object.id,
                            action: ActionType::Sit,
                            affordance_strength: 0.9,
                            body_requirement: BodyRequirement {
                                min_reach: None,
                                min_strength: None,
                                hand_requirement: HandRequirement::None,
                            },
                            detection_saliency: 0.7,
                        });
                    }
                }
                ObjectType::Cup => {
                    // Cup: affords grasping
                    self.affordances.push(Affordance {
                        object_id: object.id,
                        action: ActionType::Grasp,
                        affordance_strength: 0.85,
                        body_requirement: BodyRequirement {
                            min_reach: Some(object.distance),
                            min_strength: Some(0.1),
                            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                            hand_requirement: HandRequirement::FineManipulation,
                        },
                        detection_saliency: 0.8,
                    });
                }
                ObjectType::Tool => {
                    // Tool: affords grasping and manipulation
                    self.affordances.push(Affordance {
                        object_id: object.id,
                        action: ActionType::Grasp,
                        affordance_strength: 0.75,
                        body_requirement: BodyRequirement {
                            min_reach: Some(object.distance),
                            min_strength: Some(0.5),
                            hand_requirement: HandRequirement::SingleHand,
                        },
                        detection_saliency: 0.6,
                    });
                }
                ObjectType::Surface => {
                    // Surface: affords placing/releasing
                    self.affordances.push(Affordance {
                        object_id: object.id,
                        action: ActionType::Release,
                        affordance_strength: 0.6,
                        body_requirement: BodyRequirement {
                            min_reach: Some(object.distance),
                            min_strength: None,
                            hand_requirement: HandRequirement::SingleHand,
                        },
                        detection_saliency: 0.5,
                    });
                }
                _ => {}
            }
        }

        // Sort by saliency and limit
        self.affordances.sort_by(|a, b| {
            b.detection_saliency
                .partial_cmp(&a.detection_saliency)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.affordances.truncate(Self::MAX_AFFORDANCES);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if agent can actualize an affordance
    #[must_use]
    pub fn can_actualize_affordance(&self, affordance: &Affordance) -> bool {
        // Check reach requirement
        if let Some(min_reach) = affordance.body_requirement.min_reach {
            if self.body_schema.action_capabilities.reach_radius < min_reach {
                return false;
            }
        }

        // Check strength requirement
        if let Some(min_strength) = affordance.body_requirement.min_strength {
            if self.body_schema.action_capabilities.max_carry_weight < min_strength {
                return false;
            }
        }

        true
    }

    /// Get actionable affordances (can actually be performed)
    #[must_use]
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn get_actionable_affordances(&self) -> Vec<&Affordance> {
        self.affordances
            .iter()
            .filter(|a| self.can_actualize_affordance(a))
            .collect()
    }

    /// Learn sensorimotor contingency through interaction
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn learn_contingency(
        &mut self,
        action: ActionType,
        sensory_modality: SensoryModality,
        observed_change: f32,
        direction: ChangeDirection,
    ) {
        // Check if similar contingency exists
        if let Some(existing) = self
            .sensorimotor_contingencies
            .iter_mut()
            .find(|c| c.action == action && c.sensory_modality == sensory_modality)
        {
            // Update with new observation (delta rule)
            let prediction_error = (observed_change - existing.expected_change_magnitude).abs();
            let learning_rate = 0.1;
            existing.expected_change_magnitude += learning_rate * observed_change;
            existing.learned_dependence = (existing.learned_dependence + 0.05).min(1.0);
            existing.prediction_confidence = (1.0 - prediction_error).clamp(0.0, 1.0);
        } else {
            // Create new contingency
            self.sensorimotor_contingencies
                .push(SensorimotorContingency {
                    action,
                    sensory_modality,
                    expected_change_magnitude: observed_change,
                    change_direction: direction,
                    learned_dependence: 0.1,
                    prediction_confidence: 0.5,
                });
        }

        // Limit contingencies
        if self.sensorimotor_contingencies.len() > Self::MAX_CONTINGENCIES {
            // Remove lowest confidence
            self.sensorimotor_contingencies.sort_by(|a, b| {
                b.prediction_confidence
                    .partial_cmp(&a.prediction_confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            self.sensorimotor_contingencies
                .truncate(Self::MAX_CONTINGENCIES);
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Predict sensory consequence of action
    #[must_use]
    pub fn predict_consequence(
        &self,
        action: ActionType,
        sensory_modality: SensoryModality,
    ) -> Option<(f32, f32)> {
        // Returns (expected_change, confidence)
        self.sensorimotor_contingencies
            .iter()
            .find(|c| c.action == action && c.sensory_modality == sensory_modality)
            .map(|c| (c.expected_change_magnitude, c.prediction_confidence))
    }

    /// Integrate cognitive scaffold
    pub fn integrate_scaffold(&mut self, scaffold: CognitiveScaffold) {
        let key = format!("{:?}", scaffold.scaffold_type);

        // Initial transparency based on reliability (use before move)
        let initial_transparency = scaffold.reliability * 0.3;

        self.extended_mind.cognitive_scaffolding.push(scaffold);

        self.extended_mind
            .transparency_gradients
            .insert(key, initial_transparency);
    }

    /// Use a scaffold (increases transparency with use)
    pub fn use_scaffold(&mut self, scaffold_type: ScaffoldType) {
        let key = format!("{scaffold_type:?}");

        // Find and update usage count
        for scaffold in &mut self.extended_mind.cognitive_scaffolding {
            if scaffold.scaffold_type == scaffold_type {
                scaffold.usage_count += 1;

                // Increase integration depth with use
                let use_bonus = (scaffold.usage_count as f32 * 0.02).min(0.3);
                scaffold.integration_depth =
                    (scaffold.integration_depth + 0.05 + use_bonus).min(1.0);

                // Update transparency
                let current = self
                    .extended_mind
                    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                    .transparency_gradients
                    .get(&key)
                    .copied()
                    .unwrap_or(0.0);
                let new_transparency = (current + 0.02 + use_bonus).min(1.0);
                self.extended_mind
                    .transparency_gradients
                    .insert(key, new_transparency);
                break;
            }
        }
    }

    /// Check if cognitive process is extended
    #[must_use]
    pub fn is_extended_cognition(&self, scaffold_type: ScaffoldType) -> bool {
        let key = format!("{scaffold_type:?}");
        self.extended_mind
            .transparency_gradients
            .get(&key)
            .is_some_and(|&t| t > Self::EXTENDED_COGNITION_THRESHOLD)
    }

    /// Get integration depth for scaffold
    #[must_use]
    pub fn get_integration_depth(&self, scaffold_type: ScaffoldType) -> f32 {
        self.extended_mind
            .cognitive_scaffolding
            .iter()
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            .find(|s| s.scaffold_type == scaffold_type)
            .map_or(0.0, |s| s.integration_depth)
    }

    /// Rubber hand illusion effect
    /// Visual-proprioceptive conflict reduces body ownership
    pub fn apply_rubber_hand_illusion(
        &mut self,
        visual_hand_pos: (f32, f32, f32),
        stimulated_body_part: BodyPart,
        tactile_stimulation: bool,
    ) {
        let actual_hand = self
            .body_schema
            .body_parts
            .get(&stimulated_body_part)
            .map_or((0.0, 0.0, 0.0), |h| h.position);

        // Distance between seen and felt hand
        let distance = ((visual_hand_pos.0 - actual_hand.0).powi(2)
            + (visual_hand_pos.1 - actual_hand.1).powi(2)
            + (visual_hand_pos.2 - actual_hand.2).powi(2))
        .sqrt();

        if distance < Self::RHI_DISTANCE_THRESHOLD && tactile_stimulation {
            // Illusion conditions met - transfer some ownership
            self.body_ownership = 0.6;
        } else {
            // Restore body ownership
            self.body_ownership = Self::DEFAULT_BODY_OWNERSHIP;
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Enactive perception: perception is action-dependent
    #[must_use]
    pub fn enact_perception(&self, action_history: &[ActionType]) -> PerceptualSalience {
        let action_pattern = self.analyze_action_pattern(action_history);

        PerceptualSalience {
            visual_salience: if action_pattern.exploratory { 0.8 } else { 0.5 },
            tactile_salience: if action_pattern.manipulative {
                0.9
            } else {
                0.4
            },
            proprioceptive_salience: 0.7, // Always present in embodiment
            auditory_salience: if action_pattern.locomotive { 0.6 } else { 0.4 },
        }
    }

    fn analyze_action_pattern(&self, actions: &[ActionType]) -> ActionPattern {
        let exploratory = actions.iter().any(|a| {
            matches!(
                a,
                ActionType::LookAt
                    | ActionType::FocusAttention
                    | ActionType::TurnLeft
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
                    | ActionType::TurnRight
            )
        });
        let manipulative = actions.iter().any(|a| {
            matches!(
                a,
                ActionType::Grasp | ActionType::Reach | ActionType::Release
            )
        });
        let locomotive = actions.iter().any(|a| {
            matches!(
                a,
                ActionType::MoveForward | ActionType::MoveBackward | ActionType::Stand
            )
        });

        ActionPattern {
            exploratory,
            manipulative,
            locomotive,
        }
    }

    /// Set perceptual mode
    pub fn set_perceptual_mode(&mut self, mode: PerceptualMode) {
        self.perceptual_mode = mode;
    }

    /// Get current perceptual mode
    #[must_use]
    pub const fn get_perceptual_mode(&self) -> PerceptualMode {
        self.perceptual_mode
    }

    /// Check if agent has pain
    #[must_use]
    pub fn has_pain(&self) -> bool {
        self.body_schema
            .body_parts
            .values()
            .any(|part| part.pain_level > 0.1)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get total pain level
    #[must_use]
    pub fn total_pain(&self) -> f32 {
        self.body_schema
            .body_parts
            .values()
            .map(|part| part.pain_level)
            .sum()
    }

    /// Update action capabilities
    pub fn update_capabilities(&mut self, capabilities: ActionCapabilities) {
        self.body_schema.action_capabilities = capabilities;
        self.update_body_image();
    }

    /// Add external representation
    pub fn add_external_representation(&mut self, representation: ExternalRepresentation) {
        self.extended_mind
            .external_representations
            .push(representation);
    }

    /// Create coupled system
    pub fn create_coupled_system(
        &mut self,
        components: Vec<SystemComponent>,
        pattern: CoordinationPattern,
    ) {
        let strength = match pattern {
            CoordinationPattern::Sequential => 0.6,
            CoordinationPattern::Parallel => 0.8,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            CoordinationPattern::Emergent => 0.9,
        };

        self.extended_mind.coupled_systems.push(CoupledSystem {
            components,
            coupling_strength: strength,
            coordination_pattern: pattern,
        });
    }

    /// Get body part position
    #[must_use]
    pub fn get_body_part_position(&self, part: BodyPart) -> Option<(f32, f32, f32)> {
        self.body_schema.body_parts.get(&part).map(|p| p.position)
    }

    /// Update body part state
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn update_body_part(&mut self, part: BodyPart, state: BodyPartState) {
        self.body_schema.body_parts.insert(part, state);
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    /// Get affordance by object and action
    #[must_use]
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn get_affordance(&self, object_id: ObjectId, action: ActionType) -> Option<&Affordance> {
        self.affordances
            .iter()
            .find(|a| a.object_id == object_id && a.action == action)
    }

    /// Get strongest affordance
    #[must_use]
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn get_strongest_affordance(&self) -> Option<&Affordance> {
        self.affordances.iter().max_by(|a, b| {
            a.affordance_strength
                .partial_cmp(&b.affordance_strength)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Clear all affordances
    pub fn clear_affordances(&mut self) {
        self.affordances.clear();
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get sensorimotor contingency count
    #[must_use]
    pub fn contingency_count(&self) -> usize {
        self.sensorimotor_contingencies.len()
    }

    /// Get scaffold count
    #[must_use]
    pub fn scaffold_count(&self) -> usize {
        self.extended_mind.cognitive_scaffolding.len()
    }

    /// Check if scaffold exists
    #[must_use]
    pub fn has_scaffold(&self, scaffold_type: ScaffoldType) -> bool {
        self.extended_mind
            .cognitive_scaffolding
            .iter()
            .any(|s| s.scaffold_type == scaffold_type)
    }
}

impl Default for EmbodiedCognition {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for BodyPartState {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0, 0.0),
            velocity: (0.0, 0.0, 0.0),
            posture: 0.5,
            temperature: 37.0,
            pain_level: 0.0,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        }
    }
}

impl Default for ActionCapabilities {
    fn default() -> Self {
        Self {
            reach_radius: 0.8,
            max_carry_weight: 20.0,
            locomotion_speed: 1.4,
            manipulation_precision: 0.5,
            balance_stability: 0.9,
        }
    }
}

impl Default for BodyImage {
    fn default() -> Self {
        Self {
            perceived_size: 1.0,
            perceived_strength: 0.5,
            body_satisfaction: 0.6,
            body_monitoring: 0.3,
        }
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

impl Default for BodyRequirement {
    fn default() -> Self {
        Self {
            min_reach: None,
            min_strength: None,
            hand_requirement: HandRequirement::None,
        }
    }
}

impl Default for SpatialLayout {
    fn default() -> Self {
        Self {
            dimensions: (1.0, 1.0),
            element_positions: Vec::new(),
        }
    }
}

impl Default for CognitiveScaffold {
    fn default() -> Self {
        Self {
            scaffold_type: ScaffoldType::Notebook,
            location: None,
            reliability: 0.8,
            accessibility: 0.8,
            integration_depth: 0.0,
            usage_count: 0,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for ExternalRepresentation {
    fn default() -> Self {
        Self {
            representation_type: RepresentationType::List,
            content: Vec::new(),
            spatial_layout: SpatialLayout::default(),
            coupling_strength: 0.5,
        }
    }
}

impl Default for SensorimotorContingency {
    fn default() -> Self {
        Self {
            action: ActionType::MoveForward,
            sensory_modality: SensoryModality::Visual,
            expected_change_magnitude: 0.0,
            change_direction: ChangeDirection::Transform,
            learned_dependence: 0.0,
            prediction_confidence: 0.0,
        }
    }
}

impl Default for Affordance {
    fn default() -> Self {
        Self {
            object_id: 0,
            action: ActionType::MoveForward,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            affordance_strength: 0.5,
            body_requirement: BodyRequirement::default(),
            detection_saliency: 0.5,
        }
    }
}

impl Default for CoupledSystem {
    fn default() -> Self {
        Self {
            components: Vec::new(),
            coupling_strength: 0.5,
            coordination_pattern: CoordinationPattern::Sequential,
        }
    }
}

impl Default for PerceptualSalience {
    fn default() -> Self {
        Self {
            visual_salience: 0.5,
            tactile_salience: 0.5,
            proprioceptive_salience: 0.7,
            auditory_salience: 0.4,
        }
    }
}
