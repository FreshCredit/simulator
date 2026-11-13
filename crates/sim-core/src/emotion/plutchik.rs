use serde::{Deserialize, Serialize};

// ============================================================================
// P2-2: PLUTCHIK WHEEL EMOTIONAL DYNAMICS
// Per Plutchik (1980): Psychoevolutionary theory with 8 primary emotions
// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// The 8 primary emotions in Plutchik's wheel
/// Arranged as 4 opposing pairs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlutchikEmotion {
    /// Joy vs Sadness
    Joy,
    /// sadness.
    Sadness,
    /// Trust vs Disgust
    Trust,
    /// disgust.
    Disgust,
    /// Fear vs Anger
    Fear,
    /// anger.
    Anger,
    /// Anticipation vs Surprise
    Anticipation,
    /// surprise.
    Surprise,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl PlutchikEmotion {
    /// Get the opposite emotion
    #[must_use]
    pub const fn opposite(&self) -> Self {
        match self {
            Self::Joy => Self::Sadness,
            Self::Sadness => Self::Joy,
            Self::Trust => Self::Disgust,
            Self::Disgust => Self::Trust,
            Self::Fear => Self::Anger,
            Self::Anger => Self::Fear,
            Self::Anticipation => Self::Surprise,
            Self::Surprise => Self::Anticipation,
        }
    }

    /// Get emotion intensity levels (mild, moderate, strong, intense)
    #[must_use]
    pub fn intensity_levels(&self) -> Vec<(f32, &'static str)> {
        match self {
            Self::Joy => vec![(0.25, "Serenity"), (0.50, "Joy"), (0.75, "Ecstasy")],
            Self::Trust => {
                vec![(0.25, "Acceptance"), (0.50, "Trust"), (0.75, "Admiration")]
            }
            Self::Fear => vec![(0.25, "Apprehension"), (0.50, "Fear"), (0.75, "Terror")],
            Self::Surprise => vec![
                (0.25, "Distraction"),
                (0.50, "Surprise"),
                (0.75, "Amazement"),
            ],
            Self::Sadness => {
                vec![(0.25, "Pensiveness"), (0.50, "Sadness"), (0.75, "Grief")]
            }
            Self::Disgust => {
                vec![(0.25, "Boredom"), (0.50, "Disgust"), (0.75, "Loathing")]
            }
            Self::Anger => vec![(0.25, "Annoyance"), (0.50, "Anger"), (0.75, "Rage")],
            Self::Anticipation => vec![
                (0.25, "Interest"),
                (0.50, "Anticipation"),
                (0.75, "Vigilance"),
            ],
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get label for specific intensity [0, 1]
    #[must_use]
    pub fn label_at_intensity(&self, intensity: f32) -> &'static str {
        let levels = self.intensity_levels();
        for (threshold, label) in &levels {
            if intensity <= *threshold {
                return label;
            }
        }
        levels.last().map_or("Intense", |(_, l)| *l)
    }

    /// Basic emotion color (for visualization)
    #[must_use]
    pub const fn color(&self) -> (u8, u8, u8) {
        match self {
            Self::Joy => (255, 255, 0),            // Yellow
            Self::Trust => (0, 255, 0),            // Green
            Self::Fear => (0, 128, 0),             // Dark Green
            Self::Surprise => (0, 0, 255),         // Blue
            Self::Sadness => (128, 0, 128),        // Purple
            Self::Disgust => (255, 165, 0),        // Orange
            Self::Anger => (255, 0, 0),            // Red
            Self::Anticipation => (255, 192, 203), // Pink
        }
    }

    /// Get angle on Plutchik wheel (0-360 degrees)
    /// Joy at top (0°), proceeding clockwise
    #[must_use]
    pub const fn wheel_angle(&self) -> f32 {
        match self {
            Self::Joy => 0.0,
            Self::Trust => 45.0,
            Self::Fear => 90.0,
            Self::Surprise => 135.0,
            Self::Sadness => 180.0,
            Self::Disgust => 225.0,
            Self::Anger => 270.0,
            Self::Anticipation => 315.0,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Similarity between two emotions (0 = opposite, 1 = same)
    #[must_use]
    pub fn similarity(&self, other: &Self) -> f32 {
        if self == other {
            return 1.0;
        }
        if *other == self.opposite() {
            return 0.0;
        }

        // Adjacent emotions on wheel have similarity 0.5
        let angle_diff = (self.wheel_angle() - other.wheel_angle()).abs();
        let normalized_diff = angle_diff.min(360.0 - angle_diff);

        // 45° apart = adjacent = 0.5 similarity
        // 90° apart = 0.25 similarity
        // 135° apart = 0.125 similarity
        (1.0 - normalized_diff / 180.0).max(0.0)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Dyads (combinations of adjacent primary emotions)
/// Per Plutchik: Adjacent emotions combine to create complex emotions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlutchikDyad {
    /// Joy + Trust = Love
    Love,
    /// Trust + Fear = Submission
    Submission,
    /// Fear + Surprise = Awe
    Awe,
    /// Surprise + Sadness = Disapproval
    Disapproval,
    /// Sadness + Disgust = Remorse
    Remorse,
    /// Disgust + Anger = Contempt
    Contempt,
    /// Anger + Anticipation = Aggressiveness
    Aggressiveness,
    /// Anticipation + Joy = Optimism
    Optimism,
}

impl PlutchikDyad {
    /// Get the component emotions
    #[must_use]
    pub const fn components(&self) -> (PlutchikEmotion, PlutchikEmotion) {
        match self {
            Self::Love => (PlutchikEmotion::Joy, PlutchikEmotion::Trust),
            Self::Submission => (PlutchikEmotion::Trust, PlutchikEmotion::Fear),
            Self::Awe => (PlutchikEmotion::Fear, PlutchikEmotion::Surprise),
            Self::Disapproval => (PlutchikEmotion::Surprise, PlutchikEmotion::Sadness),
            Self::Remorse => (PlutchikEmotion::Sadness, PlutchikEmotion::Disgust),
            Self::Contempt => (PlutchikEmotion::Disgust, PlutchikEmotion::Anger),
            Self::Aggressiveness => (PlutchikEmotion::Anger, PlutchikEmotion::Anticipation),
            Self::Optimism => (PlutchikEmotion::Anticipation, PlutchikEmotion::Joy),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create dyad from two emotions (if they are adjacent)
    #[must_use]
    pub fn from_emotions(e1: PlutchikEmotion, e2: PlutchikEmotion) -> Option<Self> {
        let all_dyads = [
            Self::Love,
            Self::Submission,
            Self::Awe,
            Self::Disapproval,
            Self::Remorse,
            Self::Contempt,
            Self::Aggressiveness,
            Self::Optimism,
        ];

        for dyad in &all_dyads {
            let (c1, c2) = dyad.components();
            if (c1 == e1 && c2 == e2) || (c1 == e2 && c2 == e1) {
                return Some(*dyad);
            }
        }
        None
    }
}

/// Plutchik wheel state with intensities for all 8 emotions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlutchikState {
    /// Intensity of each emotion [0, 1]
    pub intensities: [f32; 8],
}

impl Default for PlutchikState {
    fn default() -> Self {
        Self {
            intensities: [0.0; 8],
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl PlutchikState {
    /// Get intensity for specific emotion
    #[must_use]
    pub fn get(&self, emotion: PlutchikEmotion) -> f32 {
        let idx = emotion.wheel_angle() as usize / 45;
        self.intensities[idx.min(7)]
    }

    /// Set intensity for specific emotion
    pub fn set(&mut self, emotion: PlutchikEmotion, intensity: f32) {
        let idx = emotion.wheel_angle() as usize / 45;
        self.intensities[idx.min(7)] = intensity.clamp(0.0, 1.0);
    }

    /// Get dominant emotion (highest intensity)
    #[must_use]
    pub fn dominant_emotion(&self) -> Option<(PlutchikEmotion, f32)> {
        self.intensities
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .filter(|(_, &v)| v > 0.01)
            .map(|(idx, &v)| {
                let emotion = match idx {
                    0 => PlutchikEmotion::Joy,
                    1 => PlutchikEmotion::Trust,
                    2 => PlutchikEmotion::Fear,
                    3 => PlutchikEmotion::Surprise,
                    4 => PlutchikEmotion::Sadness,
                    5 => PlutchikEmotion::Disgust,
                    6 => PlutchikEmotion::Anger,
                    7 => PlutchikEmotion::Anticipation,
                    _ => PlutchikEmotion::Joy,
                };
                (emotion, v)
            })
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if state represents a dyad
    #[must_use]
    pub fn detect_dyad(&self) -> Option<PlutchikDyad> {
        // Find two emotions with similar high intensities
        let threshold = 0.5;
        let active: Vec<_> = self
            .intensities
            .iter()
            .enumerate()
            .filter(|(_, &v)| v >= threshold)
            .map(|(i, _)| i)
            .collect();

        if active.len() == 2 {
            let e1 = match active[0] {
                0 => PlutchikEmotion::Joy,
                1 => PlutchikEmotion::Trust,
                2 => PlutchikEmotion::Fear,
                3 => PlutchikEmotion::Surprise,
                4 => PlutchikEmotion::Sadness,
                5 => PlutchikEmotion::Disgust,
                6 => PlutchikEmotion::Anger,
                7 => PlutchikEmotion::Anticipation,
                _ => PlutchikEmotion::Joy,
            };
            let e2 = match active[1] {
                0 => PlutchikEmotion::Joy,
                1 => PlutchikEmotion::Trust,
                2 => PlutchikEmotion::Fear,
                3 => PlutchikEmotion::Surprise,
                4 => PlutchikEmotion::Sadness,
                5 => PlutchikEmotion::Disgust,
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                6 => PlutchikEmotion::Anger,
                7 => PlutchikEmotion::Anticipation,
                _ => PlutchikEmotion::Joy,
            };
            PlutchikDyad::from_emotions(e1, e2)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opposite_emotions() {
        assert_eq!(PlutchikEmotion::Joy.opposite(), PlutchikEmotion::Sadness);
        assert_eq!(PlutchikEmotion::Sadness.opposite(), PlutchikEmotion::Joy);
        assert_eq!(PlutchikEmotion::Trust.opposite(), PlutchikEmotion::Disgust);
        assert_eq!(PlutchikEmotion::Disgust.opposite(), PlutchikEmotion::Trust);
        assert_eq!(PlutchikEmotion::Fear.opposite(), PlutchikEmotion::Anger);
        assert_eq!(PlutchikEmotion::Anger.opposite(), PlutchikEmotion::Fear);
        assert_eq!(
            PlutchikEmotion::Anticipation.opposite(),
            PlutchikEmotion::Surprise
        );
        assert_eq!(
            PlutchikEmotion::Surprise.opposite(),
            PlutchikEmotion::Anticipation
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_double_opposite_is_identity() {
        for emotion in [
            PlutchikEmotion::Joy,
            PlutchikEmotion::Trust,
            PlutchikEmotion::Fear,
            PlutchikEmotion::Surprise,
            PlutchikEmotion::Sadness,
            PlutchikEmotion::Disgust,
            PlutchikEmotion::Anger,
            PlutchikEmotion::Anticipation,
        ] {
            assert_eq!(emotion.opposite().opposite(), emotion);
        }
    }

    #[test]
    fn test_intensity_levels() {
        let joy_levels = PlutchikEmotion::Joy.intensity_levels();
        assert_eq!(joy_levels.len(), 3);
        assert_eq!(joy_levels[1].1, "Joy");

        let anger_levels = PlutchikEmotion::Anger.intensity_levels();
        assert_eq!(anger_levels[2].1, "Rage");
    }

    #[test]
    fn test_label_at_intensity() {
        assert_eq!(PlutchikEmotion::Joy.label_at_intensity(0.1), "Serenity");
        assert_eq!(PlutchikEmotion::Joy.label_at_intensity(0.5), "Joy");
        assert_eq!(PlutchikEmotion::Joy.label_at_intensity(0.8), "Ecstasy");
        assert_eq!(PlutchikEmotion::Joy.label_at_intensity(1.0), "Ecstasy");
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_color() {
        assert_eq!(PlutchikEmotion::Joy.color(), (255, 255, 0));
        assert_eq!(PlutchikEmotion::Anger.color(), (255, 0, 0));
        assert_eq!(PlutchikEmotion::Trust.color(), (0, 255, 0));
    }

    #[test]
    fn test_wheel_angle() {
        assert_eq!(PlutchikEmotion::Joy.wheel_angle(), 0.0);
        assert_eq!(PlutchikEmotion::Trust.wheel_angle(), 45.0);
        assert_eq!(PlutchikEmotion::Sadness.wheel_angle(), 180.0);
    }

    #[test]
    fn test_similarity_same() {
        assert_eq!(PlutchikEmotion::Joy.similarity(&PlutchikEmotion::Joy), 1.0);
    }

    #[test]
    fn test_similarity_opposite() {
        assert_eq!(
            PlutchikEmotion::Joy.similarity(&PlutchikEmotion::Sadness),
            0.0
        );
    }

    #[test]
    fn test_similarity_adjacent() {
        // Joy and Trust are adjacent (45° apart)
        let sim = PlutchikEmotion::Joy.similarity(&PlutchikEmotion::Trust);
        assert!((sim - 0.75).abs() < 0.01, "Expected ~0.75, got {sim}");
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_similarity_90_degrees() {
        // Joy and Fear are 90° apart
        let sim = PlutchikEmotion::Joy.similarity(&PlutchikEmotion::Fear);
        assert!((sim - 0.5).abs() < 0.01, "Expected ~0.5, got {sim}");
    }

    #[test]
    fn test_dyad_components() {
        let (e1, e2) = PlutchikDyad::Love.components();
        assert_eq!(e1, PlutchikEmotion::Joy);
        assert_eq!(e2, PlutchikEmotion::Trust);
    }

    #[test]
    fn test_dyad_from_emotions() {
        assert_eq!(
            PlutchikDyad::from_emotions(PlutchikEmotion::Joy, PlutchikEmotion::Trust),
            Some(PlutchikDyad::Love)
        );
        assert_eq!(
            PlutchikDyad::from_emotions(PlutchikEmotion::Trust, PlutchikEmotion::Joy),
            Some(PlutchikDyad::Love)
        );
        // Non-adjacent emotions
        assert_eq!(
            PlutchikDyad::from_emotions(PlutchikEmotion::Joy, PlutchikEmotion::Anger),
            None
        );
    }

    #[test]
    fn test_plutchik_state_default() {
        let state = PlutchikState::default();
        for i in 0..8 {
            assert_eq!(state.intensities[i], 0.0);
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_plutchik_state_get_set() {
        let mut state = PlutchikState::default();
        state.set(PlutchikEmotion::Joy, 0.8);
        assert_eq!(state.get(PlutchikEmotion::Joy), 0.8);

        // Clamping
        state.set(PlutchikEmotion::Joy, 1.5);
        assert_eq!(state.get(PlutchikEmotion::Joy), 1.0);
        state.set(PlutchikEmotion::Joy, -0.5);
        assert_eq!(state.get(PlutchikEmotion::Joy), 0.0);
    }

    #[test]
    fn test_dominant_emotion() {
        let mut state = PlutchikState::default();
        assert!(state.dominant_emotion().is_none());

        state.set(PlutchikEmotion::Joy, 0.8);
        state.set(PlutchikEmotion::Trust, 0.3);
        let (dominant, intensity) = state.dominant_emotion().unwrap();
        assert_eq!(dominant, PlutchikEmotion::Joy);
        assert_eq!(intensity, 0.8);
    }

    #[test]
    fn test_detect_dyad() {
        let mut state = PlutchikState::default();
        assert!(state.detect_dyad().is_none());

        state.set(PlutchikEmotion::Joy, 0.6);
        state.set(PlutchikEmotion::Trust, 0.6);
        assert_eq!(state.detect_dyad(), Some(PlutchikDyad::Love));

        state.set(PlutchikEmotion::Fear, 0.6);
        assert!(state.detect_dyad().is_none()); // Three emotions active
    }
}
