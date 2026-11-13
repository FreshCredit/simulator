pub mod classifier;
/// plutchik module.
pub mod plutchik;
/// validation module.
pub mod validation;

// TAG: surface=simulation owner=research-team rule=SIM-001
pub use classifier::*;
pub use plutchik::*;
pub use validation::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::docas::DocasState;

    #[test]
    fn test_emotion_category_valence() {
        assert_eq!(EmotionCategory::Joy.valence(), 1.0);
        assert_eq!(EmotionCategory::Fear.valence(), -1.0);
        assert_eq!(EmotionCategory::Surprise.valence(), 0.0);
        assert_eq!(EmotionCategory::Neutral.valence(), 0.0);
    }

    #[test]
    fn test_emotion_classifier_joy() {
        let classifier = EmotionClassifier::new();
        let mut state = EmotionState::new(0.75);

        // High trust state should produce joy
        let docas = DocasState::new([0.9, 0.8, 0.2, 0.1, 0.8]); // High D, O, S; low C, A
        let classification = classifier.classify(&docas, &mut state, 0);

        // Should detect positive emotion
        assert!(classification.delta_tau > 0.0);
        assert!(classification.confidence > 0.0);
    }

    #[test]
    fn test_emotion_classifier_fear() {
        let classifier = EmotionClassifier::new();
        let mut state = EmotionState::new(0.75);

        // Low trust, high cortisol should produce fear
        let docas = DocasState::new([0.3, 0.3, 0.8, 0.4, 0.4]); // Low D, O; high C
        let classification = classifier.classify(&docas, &mut state, 0);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Should detect negative emotion
        assert!(classification.delta_tau < 0.0);
        assert!(classification.confidence > 0.0);
    }

    #[test]
    fn test_compute_intensity() {
        let docas = DocasState::new([0.9, 0.9, 0.1, 0.1, 0.9]);
        let tau_zero = 0.75;
        let intensity = compute_intensity(&docas, tau_zero);

        // High trust should have high positive intensity
        assert!(intensity > 0.0);
    }

    #[test]
    fn test_compute_valence() {
        let docas_high = DocasState::new([0.9, 0.9, 0.1, 0.1, 0.9]);
        let docas_low = DocasState::new([0.2, 0.2, 0.8, 0.8, 0.3]);
        let tau_zero = 0.75;

        assert_eq!(compute_valence(&docas_high, tau_zero), 1.0);
        assert_eq!(compute_valence(&docas_low, tau_zero), -1.0);
    }

    #[test]
    fn test_emotion_state_baseline_update() {
        let mut state = EmotionState::new(0.75);

        // Update with higher tau
        state.update_baseline(0.9);
        assert!(state.tau_zero > 0.75); // Should increase

        // Update with lower tau
        state.update_baseline(0.5);
        assert!(state.tau_zero < 0.9); // Should decrease
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_emotion_state_trajectory() {
        let mut state = EmotionState::new(0.75);
        let classifier = EmotionClassifier::new();

        // Simulate emotional trajectory
        let docas_states = [
            DocasState::new([0.9, 0.8, 0.2, 0.1, 0.8]), // Joy
            DocasState::new([0.5, 0.5, 0.5, 0.5, 0.5]), // Neutral
            DocasState::new([0.3, 0.3, 0.8, 0.8, 0.3]), // Fear/Anger
        ];

        for (i, docas) in docas_states.iter().enumerate() {
            classifier.classify(docas, &mut state, i as u64);
        }

        assert_eq!(state.emotion_history.len(), 3);
    }

    #[test]
    fn test_validate_ekman_mapping() {
        let classifications = vec![
            (EmotionCategory::Joy, 0.8),
            (EmotionCategory::Sadness, 0.7),
            (EmotionCategory::Fear, 0.6),
            (EmotionCategory::Anger, 0.5),
            (EmotionCategory::Surprise, 0.9),
            (EmotionCategory::Disgust, 0.6),
        ];

        let score = validate_ekman_mapping(&classifications);
        assert!(score > 0.8); // Should cover most Ekman emotions
    }

    #[test]
    fn test_population_distribution() {
        let classifications = vec![
            (EmotionCategory::Joy, 0.8),
            (EmotionCategory::Joy, 0.7),
            (EmotionCategory::Fear, 0.6),
            (EmotionCategory::Anger, 0.5),
            (EmotionCategory::Neutral, 0.9),
        ];

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let dist = population_emotion_distribution(&classifications);
        let joy_count: f32 = dist
            .iter()
            .filter(|(e, _)| *e == EmotionCategory::Joy)
            .map(|(_, count)| *count)
            .sum();

        assert!((joy_count - 0.4).abs() < 0.01); // 2/5 = 0.4
    }

    #[test]
    fn test_emotion_is_ambiguous() {
        let classification = EmotionClassification {
            primary: EmotionCategory::Joy,
            confidence: 0.55,
            all_scores: [
                (EmotionCategory::Joy, 0.55),
                (EmotionCategory::Contentment, 0.50),
                (EmotionCategory::Awe, 0.3),
                (EmotionCategory::Flow, 0.2),
                (EmotionCategory::Fear, 0.1),
                (EmotionCategory::Anger, 0.1),
                (EmotionCategory::Sadness, 0.1),
                (EmotionCategory::Disgust, 0.1),
                (EmotionCategory::Surprise, 0.1),
                (EmotionCategory::Neutral, 0.1),
            ],
            delta_tau: 0.3,
            delta_tau_velocity: 0.1,
            tau_zero: 0.75,
            tau: 1.05,
        };

        assert!(classification.is_ambiguous());

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let clear_classification = EmotionClassification {
            primary: EmotionCategory::Joy,
            confidence: 0.9,
            all_scores: [
                (EmotionCategory::Joy, 0.9),
                (EmotionCategory::Contentment, 0.3),
                (EmotionCategory::Awe, 0.2),
                (EmotionCategory::Flow, 0.1),
                (EmotionCategory::Fear, 0.05),
                (EmotionCategory::Anger, 0.05),
                (EmotionCategory::Sadness, 0.05),
                (EmotionCategory::Disgust, 0.05),
                (EmotionCategory::Surprise, 0.05),
                (EmotionCategory::Neutral, 0.05),
            ],
            delta_tau: 0.5,
            delta_tau_velocity: 0.2,
            tau_zero: 0.75,
            tau: 1.25,
        };

        assert!(!clear_classification.is_ambiguous());
    }

    // === New DOCAS→Emotion API Tests ===

    #[test]
    fn test_classify_emotion_joy() {
        // High O, Low C, positive Δτ
        let docas = [0.8, 0.9, 0.2, 0.3, 0.7]; // High D, High O, Low C
        let trust = 1.2;
        let trust_prev = 0.8;

        let (emotion, confidence) = classify_emotion(&docas, None, trust, trust_prev);

        // Should detect positive emotion (Joy or Awe)
        assert!(
            emotion.is_positive(),
            "Expected positive emotion, got {emotion:?}"
        );
        assert!(confidence > 0.0, "Confidence should be positive");
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_classify_emotion_fear() {
        // High C, High A, negative Δτ
        let docas = [0.3, 0.3, 0.8, 0.8, 0.4]; // Low D, Low O, High C, High A
        let trust = 0.4;
        let trust_prev = 0.7;

        let (emotion, confidence) = classify_emotion(&docas, None, trust, trust_prev);

        // Should detect fear or anger
        assert!(
            emotion == Emotion::Fear || emotion == Emotion::Anger || emotion == Emotion::Sadness,
            "Expected negative emotion, got {emotion:?}"
        );
        assert!(confidence > 0.0);
    }

    #[test]
    fn test_classify_emotion_contentment() {
        // High S, stable, neutral trust
        let docas = [0.6, 0.6, 0.3, 0.2, 0.9]; // High S, Low A
        let prev_docas = [0.59, 0.59, 0.31, 0.21, 0.89]; // Similar = low velocity
        let trust = 0.85;
        let trust_prev = 0.84;

        let (emotion, _confidence) = classify_emotion(&docas, Some(&prev_docas), trust, trust_prev);

        // Should detect contentment, flow, or neutral
        assert!(
            emotion == Emotion::Contentment
                || emotion == Emotion::Flow
                || emotion == Emotion::Neutral
                || emotion.is_positive(),
            "Expected stable positive/neutral emotion, got {emotion:?}"
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_classify_emotion_surprise() {
        // Rapid change should trigger surprise
        let docas = [0.9, 0.2, 0.8, 0.1, 0.5];
        let prev_docas = [0.5, 0.5, 0.5, 0.5, 0.5]; // Large difference
        let trust = 0.8;
        let trust_prev = 0.5; // Large Δτ

        let (emotion, confidence) = classify_emotion(&docas, Some(&prev_docas), trust, trust_prev);

        // Should detect surprise due to rapid change
        assert_eq!(
            emotion,
            Emotion::Surprise,
            "Expected Surprise for rapid change, got {emotion:?}"
        );
        assert!(confidence > 0.0);
    }

    #[test]
    fn test_classify_emotion_disgust() {
        // High C, Low D
        let docas = [0.2, 0.4, 0.9, 0.3, 0.5]; // Low D, High C
        let trust = 0.5;
        let trust_prev = 0.55;

        let (emotion, _confidence) = classify_emotion(&docas, None, trust, trust_prev);

        // Should detect disgust or other negative emotion
        assert!(
            emotion.is_negative() || emotion == Emotion::Neutral,
            "Expected negative or neutral emotion for High C + Low D, got {emotion:?}"
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_emotion_enum_valence() {
        assert_eq!(Emotion::Joy.valence(), 1.0);
        assert_eq!(Emotion::Contentment.valence(), 1.0);
        assert_eq!(Emotion::Awe.valence(), 1.0);
        assert_eq!(Emotion::Flow.valence(), 1.0);
        assert_eq!(Emotion::Fear.valence(), -1.0);
        assert_eq!(Emotion::Anger.valence(), -1.0);
        assert_eq!(Emotion::Sadness.valence(), -1.0);
        assert_eq!(Emotion::Disgust.valence(), -1.0);
        assert_eq!(Emotion::Surprise.valence(), 0.0);
        assert_eq!(Emotion::Neutral.valence(), 0.0);
    }

    #[test]
    fn test_emotion_enum_arousal() {
        // High arousal emotions
        assert!(Emotion::Awe.arousal() > 0.8);
        assert!(Emotion::Anger.arousal() > 0.8);
        assert!(Emotion::Surprise.arousal() > 0.8);

        // Low arousal emotions
        assert!(Emotion::Contentment.arousal() < 0.3);
        assert!(Emotion::Neutral.arousal() < 0.2);
    }

    #[test]
    fn test_emotion_conversion() {
        // Test conversion between Emotion and EmotionCategory
        for cat in EmotionCategory::all() {
            let emotion = Emotion::from_category(cat);
            let back_to_cat = emotion.to_category();
            assert_eq!(cat, back_to_cat, "Conversion should be reversible");
        }
    }

    #[test]
    fn test_emotion_trajectory() {
        let history = vec![
            (1u64, EmotionCategory::Joy, 0.5),
            (2u64, EmotionCategory::Joy, 0.6),
            (3u64, EmotionCategory::Fear, 0.4),
            (4u64, EmotionCategory::Anger, 0.5),
        ];

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let trajectory = emotion_trajectory(&history);

        assert_eq!(trajectory.len(), 4);
        assert_eq!(trajectory[0].1, Emotion::Joy);
        assert_eq!(trajectory[2].1, Emotion::Fear);
        assert_eq!(trajectory[3].1, Emotion::Anger);
    }

    #[test]
    fn test_emotional_stability() {
        // Stable: all same emotion
        let stable_history = vec![
            (1u64, EmotionCategory::Contentment, 0.3),
            (2u64, EmotionCategory::Contentment, 0.3),
            (3u64, EmotionCategory::Contentment, 0.3),
        ];
        let stability = emotional_stability(&stable_history);
        assert_eq!(
            stability, 1.0,
            "Constant emotions should have perfect stability"
        );

        // Unstable: alternating positive and negative
        let unstable_history = vec![
            (1u64, EmotionCategory::Joy, 0.5),
            (2u64, EmotionCategory::Fear, 0.5),
            (3u64, EmotionCategory::Joy, 0.5),
            (4u64, EmotionCategory::Fear, 0.5),
        ];
        let instability = emotional_stability(&unstable_history);
        assert!(
            instability < 1.0,
            "Alternating emotions should have lower stability"
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_is_in_crisis() {
        // Not in crisis: mostly positive emotions
        let positive_history = vec![
            (1u64, EmotionCategory::Joy, 0.5),
            (2u64, EmotionCategory::Contentment, 0.4),
            (3u64, EmotionCategory::Joy, 0.6),
            (4u64, EmotionCategory::Neutral, 0.2),
            (5u64, EmotionCategory::Joy, 0.5),
        ];
        assert!(
            !is_in_crisis(&positive_history, 5),
            "Mostly positive should not be crisis"
        );

        // In crisis: mostly negative emotions
        let negative_history = vec![
            (1u64, EmotionCategory::Fear, 0.5),
            (2u64, EmotionCategory::Anger, 0.6),
            (3u64, EmotionCategory::Fear, 0.5),
            (4u64, EmotionCategory::Sadness, 0.4),
            (5u64, EmotionCategory::Anger, 0.6),
        ];
        assert!(
            is_in_crisis(&negative_history, 5),
            "Mostly negative should be crisis"
        );

        // Not enough history
        assert!(
            !is_in_crisis(&negative_history, 10),
            "Should not be crisis with insufficient history"
        );
    }

    #[test]
    fn test_detect_transitions() {
        let history = vec![
            (1u64, EmotionCategory::Joy, 0.5),
            (2u64, EmotionCategory::Joy, 0.6),
            (3u64, EmotionCategory::Fear, 0.7), // Transition Joy -> Fear
            (4u64, EmotionCategory::Fear, 0.6),
            (5u64, EmotionCategory::Anger, 0.8), // Transition Fear -> Anger
        ];

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let transitions = detect_transitions(&history);

        assert_eq!(transitions.len(), 2);
        assert_eq!(transitions[0].1, EmotionCategory::Joy);
        assert_eq!(transitions[0].2, EmotionCategory::Fear);
        assert_eq!(transitions[1].1, EmotionCategory::Fear);
        assert_eq!(transitions[1].2, EmotionCategory::Anger);
    }

    #[test]
    fn test_valence_trajectory() {
        let history = vec![
            (1u64, EmotionCategory::Joy, 0.5),
            (2u64, EmotionCategory::Fear, 0.6),
            (3u64, EmotionCategory::Neutral, 0.3),
        ];

        let trajectory = valence_trajectory(&history);

        assert_eq!(trajectory.len(), 3);
        assert_eq!(trajectory[0].1, 1.0); // Joy
        assert_eq!(trajectory[1].1, -1.0); // Fear
        assert_eq!(trajectory[2].1, 0.0); // Neutral
    }

    #[test]
    fn test_arousal_trajectory() {
        let history = vec![
            (1u64, EmotionCategory::Contentment, 0.5), // Low arousal
            (2u64, EmotionCategory::Anger, 0.6),       // High arousal
        ];

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let trajectory = arousal_trajectory(&history);

        assert_eq!(trajectory.len(), 2);
        assert!(trajectory[0].1 < trajectory[1].1); // Contentment < Anger
    }

    #[test]
    fn test_validate_emo_c1() {
        let emotions = vec![
            (1u64, EmotionCategory::Joy, 0.5),
            (2u64, EmotionCategory::Joy, 0.6),
            (3u64, EmotionCategory::Contentment, 0.4),
        ];
        let outcomes = vec![1.0, 0.9, 0.8]; // Positive outcomes

        let correlation = validate_emo_c1(&emotions, &outcomes);
        // Positive emotions with positive outcomes should correlate
        assert!((-1.0..=1.0).contains(&correlation));
    }

    #[test]
    fn test_validate_emo_c2() {
        let history = vec![
            (5u64, EmotionCategory::Joy, 0.5),
            (6u64, EmotionCategory::Fear, 0.6),
            (7u64, EmotionCategory::Anger, 0.7),
            (10u64, EmotionCategory::Sadness, 0.8), // Collapse at tick 10
        ];

        // Should detect Fear/Anger before collapse at tick 10 (ticks 5-9)
        assert!(validate_emo_c2(&history, 10, 5));

        // Should not detect for collapse at tick 6 (no Fear/Anger in ticks 1-5)
        assert!(!validate_emo_c2(&history, 6, 5));
    }

    #[test]
    fn test_validate_emo_c3() {
        let history = vec![
            (5u64, EmotionCategory::Fear, 0.5),
            (6u64, EmotionCategory::Joy, 0.6),
            (7u64, EmotionCategory::Contentment, 0.7),
            (10u64, EmotionCategory::Joy, 0.5), // Cooperation at tick 10
        ];

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Should detect Joy/Contentment before cooperation at tick 10 (ticks 5-9)
        assert!(validate_emo_c3(&history, 10, 5));

        // Should not detect for cooperation at tick 6 (no Joy/Contentment in ticks 1-5)
        assert!(!validate_emo_c3(&history, 6, 5));
    }

    #[test]
    fn test_emotion_intensity_series() {
        let history = vec![
            (1u64, EmotionCategory::Joy, 0.5),
            (2u64, EmotionCategory::Fear, 0.7),
            (3u64, EmotionCategory::Anger, 0.3),
        ];

        let intensities = emotion_intensity_series(&history);

        assert_eq!(intensities, vec![0.5, 0.7, 0.3]);
    }

    #[test]
    fn test_predict_crisis_probability() {
        // High crisis probability: recent fear/anger
        let crisis_history = vec![
            (1u64, EmotionCategory::Fear, 0.8),
            (2u64, EmotionCategory::Anger, 0.9),
            (3u64, EmotionCategory::Fear, 0.8),
        ];
        let prob = predict_crisis_probability(&crisis_history, 3);
        assert!(prob > 0.3, "High fear/anger should predict crisis");

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Low crisis probability: positive emotions
        let good_history = vec![
            (1u64, EmotionCategory::Joy, 0.8),
            (2u64, EmotionCategory::Contentment, 0.7),
            (3u64, EmotionCategory::Joy, 0.8),
        ];
        let low_prob = predict_crisis_probability(&good_history, 3);
        assert!(
            low_prob < 0.3,
            "Positive emotions should not predict crisis"
        );
    }

    // P2-3: Russell's Circumplex Model Tests
    #[test]
    fn test_emotion_circumplex_coordinates() {
        // Test that emotions have expected coordinates
        let joy_coords = Emotion::Joy.circumplex_coordinates();
        assert!(joy_coords.0 > 0.0, "Joy should have positive valence");
        assert!(joy_coords.1 > 0.0, "Joy should have positive arousal");

        let sad_coords = Emotion::Sadness.circumplex_coordinates();
        assert!(sad_coords.0 < 0.0, "Sadness should have negative valence");
        assert!(sad_coords.1 < 0.0, "Sadness should have negative arousal");

        let fear_coords = Emotion::Fear.circumplex_coordinates();
        assert!(fear_coords.0 < 0.0, "Fear should have negative valence");
        assert!(fear_coords.1 > 0.0, "Fear should have positive arousal");

        let content_coords = Emotion::Contentment.circumplex_coordinates();
        assert!(
            content_coords.0 > 0.0,
            "Contentment should have positive valence"
        );
        assert!(
            content_coords.1 < 0.0,
            "Contentment should have negative arousal"
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_circumplex_quadrants() {
        // Q1: Joy (positive valence, positive arousal)
        let joy = CircumplexCoordinate::new(0.8, 0.4);
        assert_eq!(joy.quadrant(), 1);

        // Q2: Fear (negative valence, positive arousal)
        let fear = CircumplexCoordinate::new(-0.6, 0.8);
        assert_eq!(fear.quadrant(), 2);

        // Q3: Sadness (negative valence, negative arousal)
        let sad = CircumplexCoordinate::new(-0.7, -0.4);
        assert_eq!(sad.quadrant(), 3);

        // Q4: Contentment (positive valence, negative arousal)
        let content = CircumplexCoordinate::new(0.7, -0.3);
        assert_eq!(content.quadrant(), 4);

        // Center: Neutral
        let neutral = CircumplexCoordinate::new(0.0, 0.0);
        assert_eq!(neutral.quadrant(), 0);
    }

    #[test]
    fn test_circumplex_polar_conversion() {
        let coord = CircumplexCoordinate::new(0.707, 0.707); // 45 degrees
        let (angle, radius) = coord.to_polar();
        assert!(
            (angle - 45.0).abs() < 5.0,
            "Angle should be ~45°, got {angle}"
        );
        assert!(
            (radius - 1.0).abs() < 0.1,
            "Radius should be ~1.0, got {radius}"
        );

        // Pure positive valence (0 degrees)
        let right = CircumplexCoordinate::new(1.0, 0.0);
        let (angle, _) = right.to_polar();
        assert!(
            !(5.0..=355.0).contains(&angle),
            "Right should be ~0°, got {angle}"
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_docas_to_circumplex() {
        let docas = DocasState::new([0.9, 0.9, 0.1, 0.1, 0.9]); // High D, O, S; Low C, A
        let coord = CircumplexCoordinate::from_docas(&docas, 0.3, 0.1);

        // High D, O should produce positive valence
        assert!(
            coord.valence > 0.0,
            "High D+O should yield positive valence"
        );

        // Low A should produce moderate arousal
        assert!(
            coord.arousal > -0.5 && coord.arousal < 0.5,
            "Low A should yield moderate arousal, got {}",
            coord.arousal
        );
    }

    #[test]
    fn test_circumplex_emotion_labels() {
        // Test that emotion_label returns valid labels (just verify no panic)
        let happy = CircumplexCoordinate::new(0.9, 0.3);
        let happy_label = happy.emotion_label();
        // Just verify it returns a non-empty string
        assert!(!happy_label.is_empty(), "Happy label should not be empty");

        let calm = CircumplexCoordinate::new(0.5, -0.7);
        let calm_label = calm.emotion_label();
        assert!(!calm_label.is_empty(), "Calm label should not be empty");

        let stressed = CircumplexCoordinate::new(-0.6, 0.8);
        let stressed_label = stressed.emotion_label();
        assert!(
            !stressed_label.is_empty(),
            "Stressed label should not be empty"
        );

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Test neutral at center
        let neutral = CircumplexCoordinate::new(0.0, 0.0);
        assert_eq!(neutral.emotion_label(), "Neutral");
    }

    // P2-2: Plutchik Wheel Tests
    #[test]
    fn test_plutchik_opposites() {
        assert_eq!(PlutchikEmotion::Joy.opposite(), PlutchikEmotion::Sadness);
        assert_eq!(PlutchikEmotion::Sadness.opposite(), PlutchikEmotion::Joy);
        assert_eq!(PlutchikEmotion::Trust.opposite(), PlutchikEmotion::Disgust);
        assert_eq!(PlutchikEmotion::Fear.opposite(), PlutchikEmotion::Anger);
        assert_eq!(
            PlutchikEmotion::Anticipation.opposite(),
            PlutchikEmotion::Surprise
        );
    }

    #[test]
    fn test_plutchik_intensity_levels() {
        let joy_levels = PlutchikEmotion::Joy.intensity_levels();
        assert!(!joy_levels.is_empty());

        // Low intensity should give mild label
        let mild = PlutchikEmotion::Joy.label_at_intensity(0.1);
        assert_eq!(mild, "Serenity");

        // High intensity should give intense label
        let intense = PlutchikEmotion::Joy.label_at_intensity(0.9);
        assert_eq!(intense, "Ecstasy");
    }

    #[test]
    fn test_plutchik_wheel_angle() {
        // Joy at top (0°)
        assert_eq!(PlutchikEmotion::Joy.wheel_angle(), 0.0);
        // Trust at 45°
        assert_eq!(PlutchikEmotion::Trust.wheel_angle(), 45.0);
        // Anger at 270°
        assert_eq!(PlutchikEmotion::Anger.wheel_angle(), 270.0);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_plutchik_similarity() {
        // Same emotion = 1.0
        assert_eq!(PlutchikEmotion::Joy.similarity(&PlutchikEmotion::Joy), 1.0);

        // Opposite = 0.0
        assert_eq!(
            PlutchikEmotion::Joy.similarity(&PlutchikEmotion::Sadness),
            0.0
        );

        // Adjacent = 0.75 (45° apart)
        let sim = PlutchikEmotion::Joy.similarity(&PlutchikEmotion::Trust);
        assert!(
            sim > 0.7 && sim < 0.8,
            "Adjacent similarity should be ~0.75, got {sim}"
        );
    }

    #[test]
    fn test_plutchik_dyad_creation() {
        // Joy + Trust = Love
        let love = PlutchikDyad::from_emotions(PlutchikEmotion::Joy, PlutchikEmotion::Trust);
        assert_eq!(love, Some(PlutchikDyad::Love));

        // Order doesn't matter
        let love2 = PlutchikDyad::from_emotions(PlutchikEmotion::Trust, PlutchikEmotion::Joy);
        assert_eq!(love2, Some(PlutchikDyad::Love));

        // Opposites don't form dyad
        let none = PlutchikDyad::from_emotions(PlutchikEmotion::Joy, PlutchikEmotion::Sadness);
        assert_eq!(none, None);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_plutchik_dyad_components() {
        let (e1, e2) = PlutchikDyad::Love.components();
        assert_eq!(e1, PlutchikEmotion::Joy);
        assert_eq!(e2, PlutchikEmotion::Trust);

        let (e3, e4) = PlutchikDyad::Aggressiveness.components();
        assert_eq!(e3, PlutchikEmotion::Anger);
        assert_eq!(e4, PlutchikEmotion::Anticipation);
    }

    #[test]
    fn test_plutchik_state() {
        let mut state = PlutchikState::default();

        // Set joy intensity
        state.set(PlutchikEmotion::Joy, 0.8);
        assert!((state.get(PlutchikEmotion::Joy) - 0.8).abs() < 0.01);

        // Dominant emotion should be Joy
        let dominant = state.dominant_emotion();
        assert!(dominant.is_some());
        assert_eq!(dominant.unwrap().0, PlutchikEmotion::Joy);
    }

    #[test]
    fn test_plutchik_detect_dyad() {
        let mut state = PlutchikState::default();

        // Set two adjacent emotions high
        state.set(PlutchikEmotion::Joy, 0.8);
        state.set(PlutchikEmotion::Trust, 0.7);

        // Should detect Love dyad
        let dyad = state.detect_dyad();
        assert_eq!(dyad, Some(PlutchikDyad::Love));
    }
}
