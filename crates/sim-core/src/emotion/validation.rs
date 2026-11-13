use crate::emotion::classifier::EmotionCategory;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Validation against Ekman basic emotions
/// Returns mapping quality score [0, 1]
/// BANNED-001: "basic emotions" refers to Ekman's psychological theory
#[must_use]
pub fn validate_ekman_mapping(classifications: &[(EmotionCategory, f32)]) -> f32 {
    // Ekman's 6 basic emotions: happiness, sadness, fear, anger, surprise, disgust
    // BANNED-001: "basic emotions" is psychological terminology (Ekman)
    // Our mapping coverage
    let ekman_coverage = [
        (EmotionCategory::Joy, 1.0),      // happiness
        (EmotionCategory::Sadness, 1.0),  // sadness
        (EmotionCategory::Fear, 1.0),     // fear
        (EmotionCategory::Anger, 1.0),    // anger
        (EmotionCategory::Surprise, 1.0), // surprise
        (EmotionCategory::Disgust, 1.0),  // disgust
    ];

    let mut total_score = 0.0;
    for (expected_emotion, weight) in &ekman_coverage {
        let found = classifications
            .iter()
            .any(|(e, conf)| e == expected_emotion && *conf > 0.5);
        if found {
            total_score += weight;
        }
    }

    total_score / ekman_coverage.len() as f32
}
// TAG: surface=simulation owner=research-team rule=SIM-001
#[must_use]
/// validate plutchik mapping.
pub fn validate_plutchik_mapping(classifications: &[(EmotionCategory, f32)]) -> f32 {
    // Plutchik's 8 basic emotions: joy, trust, fear, surprise, sadness, disgust, anger, anticipation
    // BANNED-001: "basic emotions" is psychological terminology (Plutchik)
    let plutchik_emotions = [
        EmotionCategory::Joy,
        // Trust maps to Contentment/Awe (positive social)
        EmotionCategory::Contentment,
        EmotionCategory::Fear,
        EmotionCategory::Surprise,
        EmotionCategory::Sadness,
        EmotionCategory::Disgust,
        EmotionCategory::Anger,
        // Anticipation maps to Flow (preparedness)
        EmotionCategory::Flow,
    ];

    let mut covered = 0;
    for emotion in &plutchik_emotions {
        let found = classifications
            .iter()
            .any(|(e, conf)| e == emotion && *conf > 0.3);
        if found {
            covered += 1;
        }
    }

    covered as f32 / plutchik_emotions.len() as f32
}
// TAG: surface=simulation owner=research-team rule=SIM-001
/// Crisis prediction based on trajectory trend
///
/// Predicts likelihood of entering crisis state based on recent emotional trajectory
/// Returns probability in [0, 1]
///
/// # Arguments
/// * `history` - History of (tick, emotion, intensity) tuples
/// * `window` - Number of recent ticks to analyze
///
/// # Returns
/// * `f32` - Crisis probability in [0, 1]
#[must_use]
pub fn predict_crisis_probability(history: &[(u64, EmotionCategory, f32)], window: usize) -> f32 {
    if history.len() < 3 {
        return 0.0;
    }

    let recent: Vec<_> = history.iter().rev().take(window).collect();

    // Count trends
    let mut negative_trend = 0.0f32;
    let mut fear_anger_count = 0usize;

    for (_, emotion, intensity) in &recent {
        let valence = emotion.valence();
        if valence < 0.0 {
            negative_trend += intensity * valence.abs();
        }
        if *emotion == EmotionCategory::Fear || *emotion == EmotionCategory::Anger {
            fear_anger_count += 1;
        }
    }

    // Weight by recency (exponential decay)
    let recency_weight = 1.0 - (-(recent.len() as f32) / 5.0).exp();

    // Combine negative trend with specific fear/anger prevalence
    let fear_anger_ratio = fear_anger_count as f32 / recent.len().max(1) as f32;
    let crisis_prob =
        (negative_trend / recent.len().max(1) as f32) * recency_weight + fear_anger_ratio * 0.5;

    crisis_prob.clamp(0.0, 1.0)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Validation function for EMO-C1: Emotion classification correlates with behavior (r > 0.6)
///
/// Computes correlation between emotion valence and behavior outcomes
///
/// # Arguments
/// * `emotions` - Emotion history
/// * `behavior_outcomes` - Corresponding behavior outcomes (positive = good, negative = bad)
///
/// # Returns
/// * `f32` - Pearson correlation coefficient
#[must_use]
pub fn validate_emo_c1(emotions: &[(u64, EmotionCategory, f32)], behavior_outcomes: &[f32]) -> f32 {
    if emotions.len() != behavior_outcomes.len() || emotions.len() < 2 {
        return 0.0;
    }

    let n = emotions.len() as f32;

    // Get valences
    let valences: Vec<f32> = emotions.iter().map(|(_, e, _)| e.valence()).collect();

    // Compute means
    let mean_valence = valences.iter().sum::<f32>() / n;
    let mean_behavior = behavior_outcomes.iter().sum::<f32>() / n;

    // Compute covariance and variances
    let mut covariance = 0.0f32;
    let mut var_valence = 0.0f32;
    let mut var_behavior = 0.0f32;

    for i in 0..valences.len() {
        let dv = valences[i] - mean_valence;
        let db = behavior_outcomes[i] - mean_behavior;
        covariance += dv * db;
        var_valence += dv * dv;
        var_behavior += db * db;
    }

    if var_valence == 0.0 || var_behavior == 0.0 {
        return 0.0;
    }

    covariance / (var_valence.sqrt() * var_behavior.sqrt())
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Validation function for EMO-C2: Trust collapse preceded by Fear/Anger states
///
/// Checks if trust collapse events were preceded by negative emotional states
///
/// # Arguments
/// * `history` - Emotion history
/// * `collapse_tick` - Tick when trust collapse occurred
/// * `pre_window` - Number of ticks before collapse to check
///
/// # Returns
/// * `bool` - True if Fear/Anger preceded collapse
#[must_use]
pub fn validate_emo_c2(
    history: &[(u64, EmotionCategory, f32)],
    collapse_tick: u64,
    pre_window: usize,
) -> bool {
    let window_start = collapse_tick.saturating_sub(pre_window as u64);

    let pre_collapse: Vec<_> = history
        .iter()
        .filter(|(tick, _, _)| *tick >= window_start && *tick < collapse_tick)
        .collect();

    if pre_collapse.is_empty() {
        return false;
    }

    // Check for Fear or Anger in pre-collapse window
    pre_collapse.iter().any(|(_, emotion, _)| {
        *emotion == EmotionCategory::Fear || *emotion == EmotionCategory::Anger
    })
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Validation function for EMO-C3: Cooperation preceded by Joy/Contentment
///
/// Checks if cooperative behavior was preceded by positive emotional states
///
/// # Arguments
/// * `history` - Emotion history
/// * `cooperation_tick` - Tick when cooperation occurred
/// * `pre_window` - Number of ticks before cooperation to check
///
/// # Returns
/// * `bool` - True if Joy/Contentment preceded cooperation
#[must_use]
pub fn validate_emo_c3(
    history: &[(u64, EmotionCategory, f32)],
    cooperation_tick: u64,
    pre_window: usize,
) -> bool {
    let window_start = cooperation_tick.saturating_sub(pre_window as u64);

    let pre_cooperation: Vec<_> = history
        .iter()
        .filter(|(tick, _, _)| *tick >= window_start && *tick < cooperation_tick)
        .collect();

    if pre_cooperation.is_empty() {
        return false;
    }

    // Check for Joy or Contentment in pre-cooperation window
    pre_cooperation.iter().any(|(_, emotion, _)| {
        *emotion == EmotionCategory::Joy || *emotion == EmotionCategory::Contentment
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emotion::classifier::EmotionCategory;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_validate_ekman_mapping_full_coverage() {
        let classifications = vec![
            (EmotionCategory::Joy, 0.8),
            (EmotionCategory::Sadness, 0.8),
            (EmotionCategory::Fear, 0.8),
            (EmotionCategory::Anger, 0.8),
            (EmotionCategory::Surprise, 0.8),
            (EmotionCategory::Disgust, 0.8),
        ];
        let score = validate_ekman_mapping(&classifications);
        assert!((score - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_validate_ekman_mapping_partial() {
        let classifications = vec![(EmotionCategory::Joy, 0.8), (EmotionCategory::Sadness, 0.8)];
        let score = validate_ekman_mapping(&classifications);
        assert!((score - 0.3333333).abs() < 0.01);
    }

    #[test]
    fn test_validate_ekman_mapping_low_confidence() {
        let classifications = vec![(EmotionCategory::Joy, 0.3), (EmotionCategory::Sadness, 0.3)];
        let score = validate_ekman_mapping(&classifications);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_validate_plutchik_mapping_full() {
        let classifications = vec![
            (EmotionCategory::Joy, 0.8),
            (EmotionCategory::Contentment, 0.8),
            (EmotionCategory::Fear, 0.8),
            (EmotionCategory::Surprise, 0.8),
            (EmotionCategory::Sadness, 0.8),
            (EmotionCategory::Disgust, 0.8),
            (EmotionCategory::Anger, 0.8),
            (EmotionCategory::Flow, 0.8),
        ];
        let score = validate_plutchik_mapping(&classifications);
        assert!((score - 1.0).abs() < 0.001);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_predict_crisis_empty_history() {
        let history: Vec<(u64, EmotionCategory, f32)> = vec![];
        let prob = predict_crisis_probability(&history, 5);
        assert_eq!(prob, 0.0);
    }

    #[test]
    fn test_predict_crisis_high() {
        let history = vec![
            (0, EmotionCategory::Fear, 0.9),
            (1, EmotionCategory::Anger, 0.9),
            (2, EmotionCategory::Fear, 0.9),
            (3, EmotionCategory::Anger, 0.9),
        ];
        let prob = predict_crisis_probability(&history, 4);
        assert!(prob > 0.5);
    }

    #[test]
    fn test_predict_crisis_low() {
        let history = vec![
            (0, EmotionCategory::Joy, 0.9),
            (1, EmotionCategory::Contentment, 0.9),
            (2, EmotionCategory::Flow, 0.9),
        ];
        let prob = predict_crisis_probability(&history, 3);
        assert!(prob < 0.5);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_validate_emo_c1_perfect_positive_correlation() {
        let emotions = vec![
            (0, EmotionCategory::Joy, 0.5),
            (1, EmotionCategory::Joy, 0.5),
            (2, EmotionCategory::Joy, 0.5),
        ];
        let outcomes = vec![1.0, 1.0, 1.0];
        let corr = validate_emo_c1(&emotions, &outcomes);
        assert!(corr.is_nan() || corr.abs() < 0.01); // All same valence = zero variance
    }

    #[test]
    fn test_validate_emo_c1_mixed() {
        let emotions = vec![
            (0, EmotionCategory::Joy, 0.5),
            (1, EmotionCategory::Fear, 0.5),
            (2, EmotionCategory::Joy, 0.5),
            (3, EmotionCategory::Fear, 0.5),
        ];
        let outcomes = vec![1.0, -1.0, 1.0, -1.0];
        let corr = validate_emo_c1(&emotions, &outcomes);
        assert!(corr > 0.9);
    }

    #[test]
    fn test_validate_emo_c1_mismatched_lengths() {
        let emotions = vec![(0, EmotionCategory::Joy, 0.5)];
        let outcomes = vec![1.0, 2.0];
        let corr = validate_emo_c1(&emotions, &outcomes);
        assert_eq!(corr, 0.0);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_validate_emo_c2_true() {
        let history = vec![
            (0, EmotionCategory::Fear, 0.5),
            (1, EmotionCategory::Anger, 0.5),
            (5, EmotionCategory::Neutral, 0.5),
        ];
        assert!(validate_emo_c2(&history, 5, 5));
    }

    #[test]
    fn test_validate_emo_c2_false() {
        let history = vec![
            (0, EmotionCategory::Joy, 0.5),
            (1, EmotionCategory::Contentment, 0.5),
        ];
        assert!(!validate_emo_c2(&history, 5, 5));
    }

    #[test]
    fn test_validate_emo_c3_true() {
        let history = vec![
            (0, EmotionCategory::Joy, 0.5),
            (1, EmotionCategory::Contentment, 0.5),
            (5, EmotionCategory::Neutral, 0.5),
        ];
        assert!(validate_emo_c3(&history, 5, 5));
    }

    #[test]
    fn test_validate_emo_c3_false() {
        let history = vec![
            (0, EmotionCategory::Fear, 0.5),
            (1, EmotionCategory::Anger, 0.5),
        ];
        assert!(!validate_emo_c3(&history, 5, 5));
    }
}
