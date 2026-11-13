// TAG: surface=simulation owner=research-team rule=SIM-001
//! Tests for embodied cognition system.

#[cfg(test)]
mod tests {
    use super::super::types::*;
    use crate::types::Position;

    // === EmbodiedCognition Creation Tests ===

    #[test]
    fn test_new_embodied_cognition() {
        let ec = EmbodiedCognition::new();

        assert_eq!(ec.body_schema.body_parts.len(), 10);
        assert!(ec.body_schema.body_parts.contains_key(&BodyPart::Head));
        assert!(ec.body_schema.body_parts.contains_key(&BodyPart::RightHand));
        assert_eq!(ec.body_ownership, 1.0);
        assert!(ec.affordances.is_empty());
        assert!(ec.sensorimotor_contingencies.is_empty());
    }

    #[test]
    fn test_default_values() {
        let ec = EmbodiedCognition::default();

        assert_eq!(ec.body_ownership, EmbodiedCognition::DEFAULT_BODY_OWNERSHIP);
        assert!(matches!(ec.perceptual_mode, PerceptualMode::Exploratory));
    }

    // === Body Schema Tests ===

    #[test]
    fn test_body_part_default() {
        let part = BodyPartState::default();

        assert_eq!(part.position, (0.0, 0.0, 0.0));
        assert_eq!(part.velocity, (0.0, 0.0, 0.0));
        assert_eq!(part.posture, 0.5);
        assert_eq!(part.temperature, 37.0);
        assert_eq!(part.pain_level, 0.0);
    }

    #[test]
    fn test_update_body_schema() {
        let mut ec = EmbodiedCognition::new();
        let mut sensory = SensoryInput::default();
        sensory
            .hand_positions
            .insert(BodyPart::RightHand, (1.0, 2.0, 3.0));

        ec.update_body_schema(&sensory);

        let hand = ec.body_schema.body_parts.get(&BodyPart::RightHand).unwrap();
        assert_eq!(hand.position, (1.0, 2.0, 3.0));
        // TAG: surface=simulation owner=research-team rule=SIM-001
    }

    #[test]
    fn test_body_part_operations() {
        let mut ec = EmbodiedCognition::new();

        // Update body part
        let new_state = BodyPartState {
            position: (5.0, 5.0, 5.0),
            velocity: (0.1, 0.1, 0.1),
            posture: 0.8,
            temperature: 36.5,
            pain_level: 0.2,
        };
        ec.update_body_part(BodyPart::LeftHand, new_state);

        let retrieved = ec.get_body_part_position(BodyPart::LeftHand);
        assert_eq!(retrieved, Some((5.0, 5.0, 5.0)));

        let stored = ec.body_schema.body_parts.get(&BodyPart::LeftHand).unwrap();
        assert_eq!(stored.pain_level, 0.2);
    }

    #[test]
    fn test_pain_detection() {
        let mut ec = EmbodiedCognition::new();

        assert!(!ec.has_pain());
        assert_eq!(ec.total_pain(), 0.0);

        // Add pain to left hand
        let painful_state = BodyPartState {
            pain_level: 0.5,
            ..Default::default()
        };
        ec.update_body_part(BodyPart::LeftHand, painful_state);

        assert!(ec.has_pain());
        assert_eq!(ec.total_pain(), 0.5);
    }

    // === Action Capabilities Tests ===

    #[test]
    fn test_action_capabilities_default() {
        let caps = ActionCapabilities::default();

        assert_eq!(caps.reach_radius, 0.8);
        assert_eq!(caps.max_carry_weight, 20.0);
        assert_eq!(caps.locomotion_speed, 1.4);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert_eq!(caps.manipulation_precision, 0.5);
        assert_eq!(caps.balance_stability, 0.9);
    }

    #[test]
    fn test_update_capabilities() {
        let mut ec = EmbodiedCognition::new();
        let new_caps = ActionCapabilities {
            reach_radius: 1.0,
            max_carry_weight: 30.0,
            locomotion_speed: 1.6,
            manipulation_precision: 0.7,
            balance_stability: 0.95,
        };

        ec.update_capabilities(new_caps);
        // TAG: surface=simulation owner=research-team rule=SIM-001
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        assert_eq!(ec.body_schema.action_capabilities.reach_radius, 1.0);
        assert_eq!(ec.body_schema.action_capabilities.max_carry_weight, 30.0);
        // Body image should also update
        assert!(ec.body_schema.body_image.perceived_strength > 0.5);
    }

    // === Affordance Tests ===

    #[test]
    fn test_detect_affordances_chair() {
        let mut ec = EmbodiedCognition::new();
        let mut sensory = SensoryInput::default();
        sensory.objects.push(DetectedObject {
            id: 1,
            object_type: ObjectType::Chair,
            position: Position::new(2.0, 0.0, 0.0),
            height: 0.45,
            distance: 2.0,
        });

        ec.detect_affordances(&sensory);

        assert!(!ec.affordances.is_empty());
        let chair_affordance = ec
            .affordances
            .iter()
            .find(|a| a.object_id == 1 && matches!(a.action, ActionType::Sit));
        assert!(chair_affordance.is_some());
        assert_eq!(chair_affordance.unwrap().affordance_strength, 0.9);
    }

    #[test]
    fn test_detect_affordances_cup() {
        let mut ec = EmbodiedCognition::new();
        let mut sensory = SensoryInput::default();
        sensory.objects.push(DetectedObject {
            id: 2,
            object_type: ObjectType::Cup,
            position: Position::new(1.0, 0.0, 0.0),
            height: 0.1,
            distance: 1.0,
        });

        ec.detect_affordances(&sensory);

        let cup_affordance = ec
            .affordances
            .iter()
            // TAG: surface=simulation owner=research-team rule=SIM-001
            .find(|a| a.object_id == 2 && matches!(a.action, ActionType::Grasp));
        assert!(cup_affordance.is_some());
        assert_eq!(
            cup_affordance.unwrap().body_requirement.hand_requirement,
            HandRequirement::FineManipulation
        );
    }

    #[test]
    fn test_can_actualize_affordance() {
        let ec = EmbodiedCognition::new();
        let affordance = Affordance {
            object_id: 1,
            action: ActionType::Grasp,
            affordance_strength: 0.8,
            body_requirement: BodyRequirement {
                min_reach: Some(0.5),
                min_strength: Some(5.0),
                hand_requirement: HandRequirement::SingleHand,
            },
            detection_saliency: 0.7,
        };

        assert!(ec.can_actualize_affordance(&affordance));

        // Create affordance beyond reach
        let far_affordance = Affordance {
            body_requirement: BodyRequirement {
                min_reach: Some(2.0), // Beyond default 0.8m reach
                ..Default::default()
            },
            ..Default::default()
        };

        assert!(!ec.can_actualize_affordance(&far_affordance));
    }

    #[test]
    fn test_get_strongest_affordance() {
        let mut ec = EmbodiedCognition::new();
        ec.affordances.push(Affordance {
            affordance_strength: 0.5,
            ..Default::default()
        });
        ec.affordances.push(Affordance {
            affordance_strength: 0.9,
            object_id: 2,
            ..Default::default()
        });
        ec.affordances.push(Affordance {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            affordance_strength: 0.3,
            object_id: 3,
            ..Default::default()
        });

        let strongest = ec.get_strongest_affordance().unwrap();
        assert_eq!(strongest.object_id, 2);
        assert_eq!(strongest.affordance_strength, 0.9);
    }

    #[test]
    fn test_get_actionable_affordances() {
        let mut ec = EmbodiedCognition::new();
        let mut sensory = SensoryInput::default();
        // Near cup (actionable)
        sensory.objects.push(DetectedObject {
            id: 1,
            object_type: ObjectType::Cup,
            position: Position::new(0.5, 0.0, 0.0),
            height: 0.1,
            distance: 0.5,
        });
        // Far cup (not actionable with default reach)
        sensory.objects.push(DetectedObject {
            id: 2,
            object_type: ObjectType::Cup,
            position: Position::new(10.0, 0.0, 0.0),
            height: 0.1,
            distance: 10.0,
        });

        ec.detect_affordances(&sensory);
        let actionable = ec.get_actionable_affordances();

        // Only the near cup should be actionable
        assert_eq!(actionable.len(), 1);
        assert_eq!(actionable[0].object_id, 1);
        // TAG: surface=simulation owner=research-team rule=SIM-001
    }

    // === Sensorimotor Contingency Tests ===

    #[test]
    fn test_learn_contingency() {
        let mut ec = EmbodiedCognition::new();

        ec.learn_contingency(
            ActionType::MoveForward,
            SensoryModality::Visual,
            0.5,
            ChangeDirection::Transform,
        );

        assert_eq!(ec.contingency_count(), 1);

        // Learn same contingency again - should update
        ec.learn_contingency(
            ActionType::MoveForward,
            SensoryModality::Visual,
            0.6,
            ChangeDirection::Transform,
        );

        assert_eq!(ec.contingency_count(), 1); // Still 1, not 2
        let learned = ec.predict_consequence(ActionType::MoveForward, SensoryModality::Visual);
        assert!(learned.is_some());
        let (magnitude, confidence) = learned.unwrap();
        assert!(magnitude > 0.0);
        assert!(confidence > 0.0);
    }

    #[test]
    fn test_predict_consequence() {
        let mut ec = EmbodiedCognition::new();

        // No contingency learned yet
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(ec
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
            .predict_consequence(ActionType::Grasp, SensoryModality::Tactile)
            .is_none());

        // Learn contingency
        ec.learn_contingency(
            ActionType::Grasp,
            SensoryModality::Tactile,
            0.8,
            ChangeDirection::Increase,
        );

        let prediction = ec.predict_consequence(ActionType::Grasp, SensoryModality::Tactile);
        assert!(prediction.is_some());
        let (magnitude, confidence) = prediction.unwrap();
        assert_eq!(magnitude, 0.8);
        assert!(confidence > 0.0);
    }

    #[test]
    fn test_contingency_confidence_increases() {
        let mut ec = EmbodiedCognition::new();

        ec.learn_contingency(
            ActionType::TurnLeft,
            SensoryModality::Visual,
            0.3,
            ChangeDirection::Transform,
        );
        let (_, conf1) = ec
            .predict_consequence(ActionType::TurnLeft, SensoryModality::Visual)
            .unwrap();

        // Learn again with similar value
        ec.learn_contingency(
            ActionType::TurnLeft,
            SensoryModality::Visual,
            0.32,
            ChangeDirection::Transform,
        );
        let (_, conf2) = ec
            .predict_consequence(ActionType::TurnLeft, SensoryModality::Visual)
            .unwrap();

        // Confidence should increase with consistent observations
        assert!(conf2 > conf1);
    }

    // === Extended Mind Tests ===

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_integrate_scaffold() {
        let mut ec = EmbodiedCognition::new();
        let scaffold = CognitiveScaffold {
            scaffold_type: ScaffoldType::Notebook,
            reliability: 0.9,
            accessibility: 0.8,
            ..Default::default()
        };

        ec.integrate_scaffold(scaffold);

        assert_eq!(ec.scaffold_count(), 1);
        assert!(ec.has_scaffold(ScaffoldType::Notebook));
    }

    #[test]
    fn test_use_scaffold_increases_transparency() {
        let mut ec = EmbodiedCognition::new();
        let scaffold = CognitiveScaffold {
            scaffold_type: ScaffoldType::Notebook,
            reliability: 0.9,
            accessibility: 0.8,
            ..Default::default()
        };

        ec.integrate_scaffold(scaffold);

        // Initially not extended
        assert!(!ec.is_extended_cognition(ScaffoldType::Notebook));
        let initial_depth = ec.get_integration_depth(ScaffoldType::Notebook);

        // Use scaffold multiple times
        for _ in 0..20 {
            ec.use_scaffold(ScaffoldType::Notebook);
        }

        let final_depth = ec.get_integration_depth(ScaffoldType::Notebook);
        assert!(final_depth > initial_depth);
    }

    #[test]
    fn test_extended_cognition_threshold() {
        let mut ec = EmbodiedCognition::new();
        let scaffold = CognitiveScaffold {
            scaffold_type: ScaffoldType::Calculator,
            reliability: 0.95,
            accessibility: 1.0,
            integration_depth: 0.75, // Above threshold
            ..Default::default()     // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        };

        ec.integrate_scaffold(scaffold);

        // Use scaffold many times to increase transparency above threshold
        for _ in 0..25 {
            ec.use_scaffold(ScaffoldType::Calculator);
        }

        assert!(ec.is_extended_cognition(ScaffoldType::Calculator));
    }

    #[test]
    fn test_add_external_representation() {
        let mut ec = EmbodiedCognition::new();
        let rep = ExternalRepresentation {
            representation_type: RepresentationType::Map,
            content: vec![1.0, 2.0, 3.0],
            coupling_strength: 0.8,
            ..Default::default()
        };

        ec.add_external_representation(rep);

        assert_eq!(ec.extended_mind.external_representations.len(), 1);
    }

    #[test]
    fn test_create_coupled_system() {
        let mut ec = EmbodiedCognition::new();

        ec.create_coupled_system(
            vec![
                SystemComponent::Brain,
                SystemComponent::Body,
                SystemComponent::Tool(1),
            ],
            CoordinationPattern::Parallel,
        );
        // TAG: surface=simulation owner=research-team rule=SIM-001

        assert_eq!(ec.extended_mind.coupled_systems.len(), 1);
        let system = &ec.extended_mind.coupled_systems[0];
        assert_eq!(system.components.len(), 3);
        assert!(matches!(
            system.coordination_pattern,
            CoordinationPattern::Parallel
        ));
    }

    // === Rubber Hand Illusion Tests ===

    #[test]
    fn test_rubber_hand_illusion() {
        let mut ec = EmbodiedCognition::new();

        // Set actual hand position
        let hand_state = BodyPartState {
            position: (0.0, 0.0, 0.0),
            ..Default::default()
        };
        ec.update_body_part(BodyPart::RightHand, hand_state);

        // Apply illusion with visual hand close and tactile stimulation
        ec.apply_rubber_hand_illusion((0.2, 0.0, 0.0), BodyPart::RightHand, true);

        assert!(ec.body_ownership < 1.0);
        assert_eq!(ec.body_ownership, 0.6);
    }

    #[test]
    fn test_rubber_hand_illusion_no_effect_when_far() {
        let mut ec = EmbodiedCognition::new();

        let hand_state = BodyPartState {
            position: (0.0, 0.0, 0.0),
            ..Default::default()
        };
        ec.update_body_part(BodyPart::RightHand, hand_state);
        // TAG: surface=simulation owner=research-team rule=SIM-001

        // Visual hand too far
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        ec.apply_rubber_hand_illusion((1.0, 0.0, 0.0), BodyPart::RightHand, true);

        assert_eq!(ec.body_ownership, 1.0); // No effect
    }

    #[test]
    fn test_rubber_hand_illusion_no_effect_without_tactile() {
        let mut ec = EmbodiedCognition::new();

        let hand_state = BodyPartState {
            position: (0.0, 0.0, 0.0),
            ..Default::default()
        };
        ec.update_body_part(BodyPart::RightHand, hand_state);

        // No tactile stimulation
        ec.apply_rubber_hand_illusion((0.1, 0.0, 0.0), BodyPart::RightHand, false);

        assert_eq!(ec.body_ownership, 1.0); // No effect
    }

    // === Enactive Perception Tests ===

    #[test]
    fn test_enact_perception_exploratory() {
        let ec = EmbodiedCognition::new();
        let actions = vec![ActionType::LookAt, ActionType::TurnLeft];

        let salience = ec.enact_perception(&actions);

        assert!(salience.visual_salience > 0.5); // Higher when exploring
        assert_eq!(salience.proprioceptive_salience, 0.7); // Always present
    }

    #[test]
    fn test_enact_perception_manipulative() {
        let ec = EmbodiedCognition::new();
        let actions = vec![ActionType::Grasp, ActionType::Reach];

        let salience = ec.enact_perception(&actions);

        assert!(salience.tactile_salience > 0.5); // Higher when manipulating
    }

    #[test]
    fn test_enact_perception_locomotive() {
        let ec = EmbodiedCognition::new();
        let actions = vec![ActionType::MoveForward, ActionType::Stand];

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let salience = ec.enact_perception(&actions);

        assert!(salience.auditory_salience > 0.4); // Higher when moving
    }

    #[test]
    fn test_perceptual_mode() {
        let mut ec = EmbodiedCognition::new();

        assert!(matches!(
            ec.get_perceptual_mode(),
            PerceptualMode::Exploratory
        ));

        ec.set_perceptual_mode(PerceptualMode::GoalDirected);
        assert!(matches!(
            ec.get_perceptual_mode(),
            PerceptualMode::GoalDirected
        ));
    }

    // === Body Image Tests ===

    #[test]
    fn test_body_image_default() {
        let image = BodyImage::default();

        assert_eq!(image.perceived_size, 1.0);
        assert_eq!(image.perceived_strength, 0.5);
        assert_eq!(image.body_satisfaction, 0.6);
        assert_eq!(image.body_monitoring, 0.3);
    }

    #[test]
    fn test_body_image_updates() {
        let mut ec = EmbodiedCognition::new();

        // Weak body
        ec.update_capabilities(ActionCapabilities {
            max_carry_weight: 10.0,
            ..Default::default()
        });
        let weak_satisfaction = ec.body_schema.body_image.body_satisfaction;

        // Strong body
        ec.update_capabilities(ActionCapabilities {
            max_carry_weight: 60.0,
            ..Default::default()
        });
        let strong_satisfaction = ec.body_schema.body_image.body_satisfaction;
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        assert!(strong_satisfaction > weak_satisfaction);
        assert_eq!(strong_satisfaction, 1.0); // Capped at 1.0
    }

    // === Utility Tests ===

    #[test]
    fn test_clear_affordances() {
        let mut ec = EmbodiedCognition::new();
        ec.affordances.push(Affordance::default());
        ec.affordances.push(Affordance::default());

        assert_eq!(ec.affordances.len(), 2);

        ec.clear_affordances();

        assert!(ec.affordances.is_empty());
    }

    #[test]
    fn test_get_affordance() {
        let mut ec = EmbodiedCognition::new();
        ec.affordances.push(Affordance {
            object_id: 42,
            action: ActionType::Grasp,
            affordance_strength: 0.8,
            ..Default::default()
        });

        let found = ec.get_affordance(42, ActionType::Grasp);
        assert!(found.is_some());
        assert_eq!(found.unwrap().affordance_strength, 0.8);

        let not_found = ec.get_affordance(42, ActionType::Sit);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_affordance_limit() {
        let mut ec = EmbodiedCognition::new();
        // TAG: surface=simulation owner=research-team rule=SIM-001

        // Add many affordances
        for i in 0..100 {
            let mut sensory = SensoryInput::default();
            sensory.objects.push(DetectedObject {
                id: i,
                object_type: ObjectType::Cup,
                position: Position::new(i as f32, 0.0, 0.0),
                height: 0.1,
                distance: i as f32,
            });
            ec.detect_affordances(&sensory);
        }

        assert!(ec.affordances.len() <= EmbodiedCognition::MAX_AFFORDANCES);
    }

    #[test]
    fn test_contingency_limit() {
        let mut ec = EmbodiedCognition::new();

        // Add many different contingencies
        for i in 0..150 {
            let action = if i % 2 == 0 {
                ActionType::MoveForward
            } else {
                ActionType::MoveBackward
            };
            let modality = match i % 5 {
                0 => SensoryModality::Visual,
                1 => SensoryModality::Auditory,
                2 => SensoryModality::Tactile,
                3 => SensoryModality::Proprioceptive,
                _ => SensoryModality::Vestibular,
            };
            ec.learn_contingency(action, modality, 0.5, ChangeDirection::Transform);
        }

        assert!(ec.contingency_count() <= EmbodiedCognition::MAX_CONTINGENCIES);
    }
}
