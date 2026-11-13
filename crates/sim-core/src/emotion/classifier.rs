//! Emotion module: DOCAS→Emotion reverse projection for interpretability
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION emotion states, not medical/clinical
//!
// TAG: surface=simulation owner=research-team rule=SIM-001
//! Per §6.13 Emotion Empirical Predictions from the Unified Theory of Emotion:
//! - All emotion derives from trust deviation: Δτ = τ - τ₀
//! - Emotional intensity = |Δτ| (EMO-C1: R² ≥ 0.75)
//! - Emotional valence = sign(Δτ) (EMO-C2: r > 0.85)
//! - DOCAS phases map to emotion categories (EMO-C3: > 80% accuracy)
//!
//! ## Reverse Projection System
//!
//! Forward mappings exist: Music→DOCAS, Emotion→DOCAS, Language→DOCAS (§25.9)
//! This module implements the REVERSE mapping: DOCAS→Emotion for interpretability.

use serde::{Deserialize, Serialize};

use crate::docas::DocasState;
// DOCAS component indices used in classifier logic

/// Homeostatic trust baseline window size (ticks)
/// Per §6.13.2: Stabilization window for τ₀ computation
pub const TAU_ZERO_WINDOW: usize = 100;

/// Minimum emotion classification confidence threshold
/// Per §6.13.4: Expected classification accuracy > 80%
pub const MIN_CONFIDENCE_THRESHOLD: f32 = 0.70;

/// Emotion categories based on Unified Theory of Emotion
/// Per §6.13.4: Mapping table of DOCAS patterns to emotions
/// Emotion categories based on Unified Theory of Emotion
/// Simplified enum alias for compatibility with external interfaces
/// Per §6.13.4: Mapping table of DOCAS patterns to emotions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Emotion {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// High O, Low C, positive Δτ - Positive emotion: high reward, social bonding
    Joy,

    /// High S, stable, moderate τ - Positive emotion: stable, low arousal
    Contentment,

    /// High O, High S, high trust - Positive emotion: very high trust surge
    Awe,

    /// Balanced DOCAS, stable, high trust - Positive emotion: optimal challenge, flow state
    Flow,

    /// High C, High A - Negative emotion: threat anticipation
    Fear,

    /// High A, increasing - Negative emotion: threat + action mobilization
    Anger,

    /// High C, increasing, low trust - Negative emotion: loss of social bond
    Sadness,

    /// High C, Low D - Negative emotion: rapid trust collapse
    Disgust,

    /// Rapid change in any dimension - Neutral/ambivalent: high arousal, sudden change
    Surprise,

    /// Balanced, stable - Neutral baseline: near τ₀, minimal deviation
    Neutral,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Emotion {
    /// Convert from `EmotionCategory` to Emotion
    #[must_use]
    pub const fn from_category(category: EmotionCategory) -> Self {
        match category {
            EmotionCategory::Joy => Self::Joy,
            EmotionCategory::Contentment => Self::Contentment,
            EmotionCategory::Awe => Self::Awe,
            EmotionCategory::Flow => Self::Flow,
            EmotionCategory::Fear => Self::Fear,
            EmotionCategory::Anger => Self::Anger,
            EmotionCategory::Sadness => Self::Sadness,
            EmotionCategory::Disgust => Self::Disgust,
            EmotionCategory::Surprise => Self::Surprise,
            EmotionCategory::Neutral => Self::Neutral,
        }
    }

    /// Convert to `EmotionCategory`
    #[must_use]
    pub const fn to_category(&self) -> EmotionCategory {
        match self {
            Self::Joy => EmotionCategory::Joy,
            Self::Contentment => EmotionCategory::Contentment,
            Self::Awe => EmotionCategory::Awe,
            Self::Flow => EmotionCategory::Flow,
            Self::Fear => EmotionCategory::Fear,
            Self::Anger => EmotionCategory::Anger,
            Self::Sadness => EmotionCategory::Sadness,
            Self::Disgust => EmotionCategory::Disgust,
            Self::Surprise => EmotionCategory::Surprise,
            Self::Neutral => EmotionCategory::Neutral,
        }
    }

    /// Get human-readable label for emotion
    #[must_use]
    pub const fn label(&self) -> &'static str {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        match self {
            Self::Joy => "Joy",
            Self::Contentment => "Contentment",
            Self::Awe => "Awe",
            Self::Flow => "Flow",
            Self::Fear => "Fear",
            Self::Anger => "Anger",
            Self::Sadness => "Sadness",
            Self::Disgust => "Disgust",
            Self::Surprise => "Surprise",
            Self::Neutral => "Neutral",
        }
    }

    /// Get valence of emotion (-1.0 = negative, 0.0 = neutral, +1.0 = positive)
    #[must_use]
    pub const fn valence(&self) -> f32 {
        match self {
            Self::Joy | Self::Contentment | Self::Awe | Self::Flow => 1.0,
            Self::Fear | Self::Anger | Self::Sadness | Self::Disgust => -1.0,
            Self::Surprise | Self::Neutral => 0.0,
        }
    }

    /// Get arousal level (0.0 = low, 1.0 = high)
    #[must_use]
    pub const fn arousal(&self) -> f32 {
        match self {
            Self::Joy => 0.7,
            Self::Contentment => 0.2,
            Self::Awe => 0.9,
            Self::Flow => 0.5,
            Self::Fear => 0.8,
            Self::Anger => 0.9,
            Self::Sadness => 0.3,
            Self::Disgust => 0.6,
            Self::Surprise => 0.9,
            Self::Neutral => 0.1,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if emotion is negative
    #[must_use]
    pub fn is_negative(&self) -> bool {
        self.valence() < 0.0
    }

    /// Check if emotion is positive
    #[must_use]
    pub fn is_positive(&self) -> bool {
        self.valence() > 0.0
    }

    /// Check if emotion is neutral
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        self.valence() == 0.0
    }

    /// Get Russell's circumplex coordinates (valence, arousal) per P2-3
    /// Based on Russell 1980: A circumplex model of affect
    /// Maps emotion to 2D coordinate space: valence × arousal
    #[must_use]
    pub const fn circumplex_coordinates(&self) -> (f32, f32) {
        match self {
            Self::Joy => (0.80, 0.40),          // Q1: Pleasant, moderate arousal
            Self::Contentment => (0.75, -0.35), // Q4: Pleasant, low arousal
            Self::Awe => (0.70, 0.85),          // Q1: Pleasant, high arousal
            Self::Flow => (0.60, 0.10),         // Q1: Pleasant, low-moderate arousal
            Self::Fear => (-0.65, 0.75),        // Q2: Unpleasant, high arousal
            Self::Anger => (-0.60, 0.80),       // Q2: Unpleasant, high arousal
            Self::Sadness => (-0.75, -0.40),    // Q3: Unpleasant, low arousal
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            Self::Disgust => (-0.70, 0.30), // Q2: Unpleasant, moderate arousal
            Self::Surprise => (0.00, 0.90), // Top: Neutral valence, high arousal
            Self::Neutral => (0.00, 0.00),  // Center
        }
    }

    /// Get angle in degrees (0° = Pleasant/Right, 90° = High Arousal/Top)
    #[must_use]
    pub fn circumplex_angle(&self) -> f32 {
        let (v, a) = self.circumplex_coordinates();
        let angle = a.atan2(v).to_degrees();
        if angle < 0.0 {
            angle + 360.0
        } else {
            angle
        }
    }

    /// Get intensity (distance from center)
    #[must_use]
    pub fn circumplex_intensity(&self) -> f32 {
        let (v, a) = self.circumplex_coordinates();
        v.hypot(a)
    }
}

/// Russell's Circumplex Model coordinates for 2D emotion visualization (P2-3)
/// Per Russell 1980: Two orthogonal dimensions - valence × arousal
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CircumplexCoordinate {
    /// Valence: -1.0 (unpleasant) to +1.0 (pleasant)
    pub valence: f32,
    /// Arousal: -1.0 (low/deactivated) to +1.0 (high/activated)
    pub arousal: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl CircumplexCoordinate {
    /// Create new coordinate, clamped to valid range
    #[must_use]
    pub fn new(valence: f32, arousal: f32) -> Self {
        Self {
            valence: valence.clamp(-1.0, 1.0),
            arousal: arousal.clamp(-1.0, 1.0),
        }
    }

    /// Convert to polar coordinates (`angle_degrees`, radius)
    /// Angle: 0° = Pleasant (right), 90° = High Arousal (top)
    #[must_use]
    pub fn to_polar(&self) -> (f32, f32) {
        let angle = self.arousal.atan2(self.valence).to_degrees();
        let normalized = if angle < 0.0 { angle + 360.0 } else { angle };
        let radius = self.valence.hypot(self.arousal).min(1.0);
        (normalized, radius)
    }

    /// Get quadrant (1-4, or 0 for center)
    /// Q1: (+,+), Q2: (-,+), Q3: (-,-), Q4: (+,-)
    #[must_use]
    pub fn quadrant(&self) -> u8 {
        if self.valence.abs() < 0.1 && self.arousal.abs() < 0.1 {
            0 // Center/neutral
        } else if self.valence >= 0.0 && self.arousal >= 0.0 {
            1 // Q1: Pleasant, High Arousal
        } else if self.valence < 0.0 && self.arousal >= 0.0 {
            2 // Q2: Unpleasant, High Arousal
        } else if self.valence < 0.0 && self.arousal < 0.0 {
            3 // Q3: Unpleasant, Low Arousal
        } else {
            4 // Q4: Pleasant, Low Arousal
        }
    }

    /// Euclidean distance to another coordinate
    #[must_use]
    pub fn distance(&self, other: &Self) -> f32 {
        (self.valence - other.valence).hypot(self.arousal - other.arousal)
    }

    /// Get emotion label based on angular region (Russell's mapping)
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[must_use]
    pub fn emotion_label(&self) -> &'static str {
        let (angle, radius) = self.to_polar();
        if radius < 0.15 {
            return "Neutral";
        }
        match angle as u16 {
            0..=15 | 345..=360 => "Happy/Joy",
            16..=45 => "Excited/Elated",
            46..=75 => "Alert",
            76..=105 => "Surprised",
            106..=135 => "Tense",
            136..=165 => "Nervous/Stressed",
            166..=195 => "Sad/Depressed",
            196..=225 => "Lethargic",
            226..=255 => "Fatigued/Sleepy",
            256..=285 => "Calm",
            286..=315 => "Relaxed",
            316..=344 => "Contented",
            _ => "Mixed",
        }
    }

    /// Convert DOCAS state to circumplex coordinates
    /// Formula: valence ∝ (D + O + S - C) / 4, arousal ∝ (A + |Δτ| - S/2)
    #[must_use]
    pub fn from_docas(docas: &DocasState, delta_tau: f32, velocity: f32) -> Self {
        let d = docas.dopamine();
        let o = docas.oxytocin();
        let c = docas.cortisol();
        let a = docas.adrenaline();
        let s = docas.serotonin();

        // Valence: weighted combination of positive minus negative signals
        let valence = ((d - 0.5) * 0.25
            + (o - 0.5) * 0.30
            + (s - 0.5) * 0.15
            + -(c - 0.5) * 0.30
            + delta_tau.signum() * delta_tau.abs().min(0.5) * 0.5)
            .clamp(-1.0, 1.0);

        // Arousal: combines activation and change velocity
        let arousal =
            ((a - 0.5) * 0.35 + delta_tau.abs() * 0.35 + velocity.abs() * 0.20 + -(s - 0.5) * 0.10)
                .clamp(-1.0, 1.0);

        Self::new(valence, arousal)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Emotion categories based on Unified Theory of Emotion
/// Per §6.13.4: Mapping table of DOCAS patterns to emotions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash, Serialize, Deserialize)]
pub enum EmotionCategory {
    /// Positive emotion: high reward, social bonding
    /// DOCAS: D↑, O↑
    Joy,

    /// Positive emotion: stable, low arousal
    /// DOCAS: S dominant, low C
    Contentment,

    /// Positive emotion: very high trust surge
    /// DOCAS: D↑↑, O↑↑, rapid positive Δτ
    Awe,

    /// Positive emotion: optimal challenge, flow state
    /// DOCAS: S↑, C low, stable τ
    Flow,

    /// Negative emotion: threat anticipation
    /// DOCAS: C↑, A priming (not yet active)
    Fear,

    /// Negative emotion: threat + action mobilization
    /// DOCAS: C↑, A↑, agency signal
    Anger,

    /// Negative emotion: loss of social bond
    /// DOCAS: O↓, C↑ sustained
    Sadness,

    /// Negative emotion: rapid trust collapse
    /// DOCAS: C spike, withdrawal signal
    Disgust,

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Neutral/ambivalent: high arousal, sudden change
    /// DOCAS: A spike, regardless of direction
    Surprise,

    /// Neutral baseline: near τ₀, minimal deviation
    /// DOCAS: Balanced, stable
    Neutral,
}

impl EmotionCategory {
    /// Get all emotion categories for iteration
    #[must_use]
    pub const fn all() -> [Self; 10] {
        [
            Self::Joy,
            Self::Contentment,
            Self::Awe,
            Self::Flow,
            Self::Fear,
            Self::Anger,
            Self::Sadness,
            Self::Disgust,
            Self::Surprise,
            Self::Neutral,
        ]
    }

    /// Get human-readable label for emotion
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Joy => "Joy",
            Self::Contentment => "Contentment",
            Self::Awe => "Awe",
            Self::Flow => "Flow",
            Self::Fear => "Fear",
            Self::Anger => "Anger",
            Self::Sadness => "Sadness",
            Self::Disgust => "Disgust",
            Self::Surprise => "Surprise",
            Self::Neutral => "Neutral",
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get valence of emotion (-1.0 = negative, 0.0 = neutral, +1.0 = positive)
    #[must_use]
    pub const fn valence(&self) -> f32 {
        match self {
            Self::Joy => 1.0,
            Self::Contentment => 1.0,
            Self::Awe => 1.0,
            Self::Flow => 1.0,
            Self::Fear => -1.0,
            Self::Anger => -1.0,
            Self::Sadness => -1.0,
            Self::Disgust => -1.0,
            Self::Surprise => 0.0,
            Self::Neutral => 0.0,
        }
    }

    /// Get arousal level (0.0 = low, 1.0 = high)
    #[must_use]
    pub const fn arousal(&self) -> f32 {
        match self {
            Self::Joy => 0.7,
            Self::Contentment => 0.2,
            Self::Awe => 0.9,
            Self::Flow => 0.5,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            Self::Fear => 0.8,
            Self::Anger => 0.9,
            Self::Sadness => 0.3,
            Self::Disgust => 0.6,
            Self::Surprise => 0.9,
            Self::Neutral => 0.1,
        }
    }
}

/// Emotion classification result with confidence scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionClassification {
    /// Primary detected emotion
    pub primary: EmotionCategory,

    /// Confidence score for primary emotion [0.0, 1.0]
    pub confidence: f32,

    /// All emotion scores (for ambiguity detection)
    pub all_scores: [(EmotionCategory, f32); 10],

    /// Trust deviation that produced this classification (Δτ)
    pub delta_tau: f32,

    /// Trust deviation velocity (dΔτ/dt)
    pub delta_tau_velocity: f32,

    /// Homeostatic baseline used (τ₀)
    pub tau_zero: f32,

    /// Current trust level (τ)
    pub tau: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl EmotionClassification {
    /// Check if classification meets confidence threshold
    #[must_use]
    pub fn is_reliable(&self) -> bool {
        self.confidence >= MIN_CONFIDENCE_THRESHOLD
    }

    /// Get secondary emotion (second highest score)
    #[must_use]
    pub fn secondary_emotion(&self) -> Option<(EmotionCategory, f32)> {
        let mut sorted = self.all_scores.to_vec();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        sorted.get(1).copied()
    }

    /// Check if emotion is ambiguous (low confidence spread)
    #[must_use]
    pub fn is_ambiguous(&self) -> bool {
        if let Some((_, secondary_score)) = self.secondary_emotion() {
            let spread = self.confidence - secondary_score;
            spread < 0.15 // Less than 15% spread between top two
        } else {
            false
        }
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

/// Agent's emotional state tracker
/// Per §6.13: Emotion derives from trust dynamics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionState {
    /// Current emotional classification
    pub current: Option<EmotionClassification>,

    /// History of tau values for baseline computation (rolling window)
    pub tau_history: Vec<f32>,

    /// Current homeostatic baseline (τ₀)
    pub tau_zero: f32,

    /// Previous tau for velocity computation
    pub previous_tau: f32,

    /// Emotional trajectory history (for temporal analysis)
    pub emotion_history: Vec<(u64, EmotionCategory, f32)>, // (tick, emotion, intensity)

    /// Maximum history size
    pub max_history: usize,
}

impl Default for EmotionState {
    fn default() -> Self {
        Self {
            current: None,
            tau_history: Vec::with_capacity(TAU_ZERO_WINDOW),
            tau_zero: 0.75, // Default neutral baseline
            previous_tau: 0.75,
            emotion_history: Vec::new(),
            max_history: 1000,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl EmotionState {
    /// Create new emotion state with specified baseline
    #[must_use]
    pub fn new(tau_zero: f32) -> Self {
        Self {
            tau_zero,
            previous_tau: tau_zero,
            ..Default::default()
        }
    }

    /// Update τ₀ (homeostatic baseline) using exponential moving average
    /// Per §6.13.2: τ₀(t) = α × τ₀(t-1) + (1-α) × τ(t), α = 0.99
    pub fn update_baseline(&mut self, current_tau: f32) {
        self.previous_tau = self.tau_history.last().copied().unwrap_or(self.tau_zero);
        const ALPHA: f32 = 0.99;
        self.tau_zero = ALPHA * self.tau_zero + (1.0 - ALPHA) * current_tau;

        // Also maintain history window for alternative baseline computation
        self.tau_history.push(current_tau);
        if self.tau_history.len() > TAU_ZERO_WINDOW {
            self.tau_history.remove(0);
        }
    }

    /// Compute baseline as arithmetic mean over window
    #[must_use]
    pub fn window_baseline(&self) -> f32 {
        if self.tau_history.is_empty() {
            return self.tau_zero;
        }
        self.tau_history.iter().sum::<f32>() / self.tau_history.len() as f32
    }

    /// Get current trust deviation (Δτ)
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[must_use]
    pub fn delta_tau(&self, current_tau: f32) -> f32 {
        current_tau - self.tau_zero
    }

    /// Get trust deviation velocity (dΔτ/dt approximation)
    #[must_use]
    pub fn delta_tau_velocity(&self, current_tau: f32) -> f32 {
        current_tau - self.previous_tau
    }

    /// Record emotion to history with homeostatic baseline synchronization
    pub fn record_emotion(&mut self, tick: u64, emotion: EmotionCategory, intensity: f32) {
        self.emotion_history.push((tick, emotion, intensity));
        if self.emotion_history.len() > self.max_history {
            self.emotion_history.remove(0);
        }
        if let Some(ref mut cur) = self.current {
            cur.primary = emotion;
            cur.confidence = intensity.clamp(0.0, 1.0);
            cur.tau_zero = self.tau_zero;
        }
    }

    /// Get emotional trajectory (recent history)
    #[must_use]
    pub fn recent_trajectory(&self, n: usize) -> Vec<(u64, EmotionCategory, f32)> {
        self.emotion_history.iter().rev().take(n).copied().collect()
    }

    /// Compute emotional stability (variance of recent emotions)
    /// Per §6.13.5: Low variance indicates stable emotional state
    #[must_use]
    pub fn emotional_stability(&self, window: usize) -> f32 {
        let recent: Vec<f32> = self
            .emotion_history
            .iter()
            .rev()
            .take(window)
            .map(|(_, _, intensity)| *intensity)
            .collect();

        if recent.len() < 2 {
            return 1.0; // Maximum stability with no history
        }

        let mean = recent.iter().sum::<f32>() / recent.len() as f32;
        let variance = recent.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / recent.len() as f32;

        let vol = self.emotional_volatility(window);
        (1.0 - (variance + vol * 0.1).min(1.0)).clamp(0.0, 1.0)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get emotional volatility (standard deviation of recent Δτ)
    /// BANNED-001: "standard deviation" is a technical statistical term
    #[must_use]
    pub fn emotional_volatility(&self, window: usize) -> f32 {
        if self.tau_history.len() < 2 {
            return 0.0;
        }

        let recent: Vec<f32> = self
            .tau_history
            .iter()
            .rev()
            .take(window)
            .copied()
            .collect();

        let mean = recent.iter().sum::<f32>() / recent.len() as f32;
        let variance = recent.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / recent.len() as f32;

        variance.sqrt()
    }
}

/// DOCAS→Emotion classifier
/// Implements reverse projection from neuromodulator state to emotion labels
#[derive(Debug, Clone)]
pub struct EmotionClassifier {
    /// Optional custom classification thresholds
    pub joy_threshold: f32,
    /// fear threshold.
    pub fear_threshold: f32,
    /// awe threshold.
    pub awe_threshold: f32,
}

impl Default for EmotionClassifier {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    fn default() -> Self {
        Self {
            joy_threshold: 0.30,
            fear_threshold: -0.20,
            awe_threshold: 0.50,
        }
    }
}

impl EmotionClassifier {
    /// Create new classifier with default thresholds
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create classifier with custom thresholds
    #[must_use]
    pub const fn with_thresholds(joy: f32, fear: f32, awe: f32) -> Self {
        Self {
            joy_threshold: joy,
            fear_threshold: fear,
            awe_threshold: awe,
        }
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Classify DOCAS state into emotion
    /// Per §6.13.4: Algorithm using Δτ, dΔτ/dt, and DOCAS pattern
    ///
    /// Returns emotion classification with confidence scores
    pub fn classify(
        &self,
        docas: &DocasState,
        emotion_state: &mut EmotionState,
        current_tick: u64,
    ) -> EmotionClassification {
        // Compute current trust
        let tau = docas.compute_tau();

        // Update baseline
        emotion_state.update_baseline(tau);

        // Compute Δτ and velocity
        let delta_tau = emotion_state.delta_tau(tau);
        let velocity = emotion_state.delta_tau_velocity(tau);

        // Update previous tau
        emotion_state.previous_tau = tau;

        // Compute emotion scores based on DOCAS pattern
        let scores = self.compute_emotion_scores(docas, delta_tau, velocity);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Find primary emotion (highest score)
        let (primary, confidence) = scores
            .iter()
            .enumerate()
            .max_by(|(_, (_, a)), (_, (_, b))| {
                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map_or((EmotionCategory::Neutral, 0.0), |(idx, (_, score))| {
                (EmotionCategory::all()[idx], *score)
            });

        let classification = EmotionClassification {
            primary,
            confidence,
            all_scores: scores,
            delta_tau,
            delta_tau_velocity: velocity,
            tau_zero: emotion_state.tau_zero,
            tau,
        };

        // Record to history
        let intensity = delta_tau.abs();
        emotion_state.record_emotion(current_tick, primary, intensity);

        // Update current emotion state
        emotion_state.current = Some(classification.clone());

        classification
    }

    /// Compute emotion scores from DOCAS pattern
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Per §6.13.4: Pattern matching based on Δτ range, velocity, and DOCAS components
    fn compute_emotion_scores(
        &self,
        docas: &DocasState,
        delta_tau: f32,
        velocity: f32,
    ) -> [(EmotionCategory, f32); 10] {
        let d = docas.dopamine();
        let o = docas.oxytocin();
        let c = docas.cortisol();
        let a = docas.adrenaline();
        let s = docas.serotonin();

        let scores = [
            self.score_joy(delta_tau, velocity, d, o),
            self.score_contentment(delta_tau, s, c),
            self.score_awe(delta_tau, velocity, d, o),
            self.score_flow(delta_tau, velocity, s, c, a),
            self.score_fear(delta_tau, velocity, c, a),
            self.score_anger(delta_tau, velocity, c, a),
            self.score_sadness(delta_tau, velocity, o, c),
            self.score_disgust(delta_tau, velocity, c),
            self.score_surprise(delta_tau, velocity, a),
            self.score_neutral(delta_tau, velocity, (d, o, c, a, s)),
        ];

        let max_score = scores.iter().copied().fold(0.0f32, f32::max);

        let normalize = |s: f32| -> f32 {
            if max_score > 0.0 {
                (s / max_score).min(1.0)
            } else {
                0.0
            }
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        [
            (EmotionCategory::Joy, normalize(scores[0])),
            (EmotionCategory::Contentment, normalize(scores[1])),
            (EmotionCategory::Awe, normalize(scores[2])),
            (EmotionCategory::Flow, normalize(scores[3])),
            (EmotionCategory::Fear, normalize(scores[4])),
            (EmotionCategory::Anger, normalize(scores[5])),
            (EmotionCategory::Sadness, normalize(scores[6])),
            (EmotionCategory::Disgust, normalize(scores[7])),
            (EmotionCategory::Surprise, normalize(scores[8])),
            (EmotionCategory::Neutral, normalize(scores[9])),
        ]
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn score_joy(&self, delta_tau: f32, velocity: f32, d: f32, o: f32) -> f32 {
        let mut score = 0.0;
        if delta_tau > self.joy_threshold {
            score += (delta_tau - self.joy_threshold) * 2.0;
            score += d * 0.3;
            score += o * 0.3;
            if velocity > 0.0 {
                score += velocity * 0.5;
            }
        }
        score
    }

    fn score_contentment(&self, delta_tau: f32, s: f32, c: f32) -> f32 {
        let mut score = 0.0;
        if delta_tau.abs() < 0.10 {
            score += (0.10 - delta_tau.abs()) * 5.0;
            score += s * 0.4;
            score += (1.0 - c) * 0.2;
        }
        score
    }

    fn score_awe(&self, delta_tau: f32, velocity: f32, d: f32, o: f32) -> f32 {
        let mut score = 0.0;
        if delta_tau > self.awe_threshold {
            score += (delta_tau - self.awe_threshold) * 3.0;
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            score += d * 0.3;
            score += o * 0.3;
            if velocity > 0.1 {
                score += velocity * 2.0;
            }
        }
        score
    }

    fn score_flow(&self, delta_tau: f32, velocity: f32, s: f32, c: f32, a: f32) -> f32 {
        let mut score = 0.0;
        if delta_tau.abs() < 0.05 && velocity.abs() < 0.02 {
            score += 0.5;
            score += s * 0.4;
            score += (1.0 - c) * 0.3;
            score += (1.0 - a) * 0.1;
        }
        score
    }

    fn score_fear(&self, delta_tau: f32, velocity: f32, c: f32, a: f32) -> f32 {
        let mut score = 0.0;
        if delta_tau < self.fear_threshold {
            score += (self.fear_threshold - delta_tau) * 2.0;
            score += c * 0.4;
            if velocity < -0.05 {
                score += velocity.abs() * 1.5;
            }
            if a < 0.5 {
                score += 0.2;
            }
        }
        score
    }

    fn score_anger(&self, delta_tau: f32, velocity: f32, c: f32, a: f32) -> f32 {
        let mut score = 0.0;
        if delta_tau < self.fear_threshold {
            score += (self.fear_threshold - delta_tau) * 1.5;
            score += c * 0.3;
            score += a * 0.4;
            if velocity < 0.0 && velocity > -0.15 {
                score += 0.2;
            }
        }
        score
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn score_sadness(&self, delta_tau: f32, velocity: f32, o: f32, c: f32) -> f32 {
        let mut score = 0.0;
        if delta_tau < -0.15 {
            score += (-delta_tau) * 1.5;
            score += (1.0 - o) * 0.4;
            score += c * 0.2;
            if velocity.abs() < 0.05 {
                score += 0.2;
            }
        }
        score
    }

    fn score_disgust(&self, delta_tau: f32, velocity: f32, c: f32) -> f32 {
        let mut score = 0.0;
        if velocity < -0.10 {
            score += velocity.abs() * 2.0;
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            score += c * 0.4;
            if delta_tau < 0.0 {
                score += 0.3;
            }
        }
        score
    }

    fn score_surprise(&self, delta_tau: f32, velocity: f32, a: f32) -> f32 {
        let mut score = 0.0;
        if velocity.abs() > 0.15 && delta_tau.abs() > 0.10 {
            score += velocity.abs() * 2.0;
            score += delta_tau.abs() * 0.5;
            score += a * 0.3;
        }
        score
    }

    fn score_neutral(
        &self,
        delta_tau: f32,
        velocity: f32,
        docas: (f32, f32, f32, f32, f32),
    ) -> f32 {
        let (d, o, c, a, s) = docas;
        let mut score = 0.0;
        if delta_tau.abs() < 0.05 && velocity.abs() < 0.01 {
            score += 0.5;
            let balance = 1.0
                - ((d - 0.5).abs()
                    + (o - 0.5).abs()
                    + (c - 0.5).abs()
                    + (a - 0.5).abs()
                    + (s - 0.5).abs())
                    / 2.5;
            score += balance * 0.5;
        }
        score
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Quick classify without updating state (for batch analysis)
    #[must_use]
    pub fn classify_static(&self, docas: &DocasState, tau_zero: f32) -> (EmotionCategory, f32) {
        let tau = docas.compute_tau();
        let delta_tau = tau - tau_zero;
        let velocity = 0.0; // No velocity info in static classification

        let scores = self.compute_emotion_scores(docas, delta_tau, velocity);

        scores
            .iter()
            .enumerate()
            .max_by(|(_, (_, a)), (_, (_, b))| {
                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map_or((EmotionCategory::Neutral, 0.0), |(idx, (_, score))| {
                (EmotionCategory::all()[idx], *score)
            })
    }
}

/// Compute emotional intensity from DOCAS state
/// Per §6.13.1 EMO-C1: Emotional intensity = |Δτ|
#[must_use]
pub fn compute_intensity(docas: &DocasState, tau_zero: f32) -> f32 {
    let tau = docas.compute_tau();
    (tau - tau_zero).abs()
}

/// Compute emotional valence from DOCAS state
/// Per §6.13.1 EMO-C2: Emotional valence = sign(Δτ)
#[must_use]
pub fn compute_valence(docas: &DocasState, tau_zero: f32) -> f32 {
    let tau = docas.compute_tau();
    let delta = tau - tau_zero;
    if delta > 0.01 {
        1.0
    } else if delta < -0.01 {
        -1.0
    } else {
        0.0
    }
}

/// Batch classify multiple DOCAS states (for visualization/analysis)
#[must_use]
pub fn batch_classify(
    classifier: &EmotionClassifier,
    docas_states: &[DocasState],
    // TAG: surface=simulation owner=research-team rule=SIM-001
    tau_zeros: &[f32],
) -> Vec<(EmotionCategory, f32)> {
    docas_states
        .iter()
        .zip(tau_zeros.iter())
        .map(|(docas, &tz)| classifier.classify_static(docas, tz))
        .collect()
}

/// Compute population-level emotion distribution
#[must_use]
pub fn population_emotion_distribution(
    classifications: &[(EmotionCategory, f32)],
) -> [(EmotionCategory, f32); 10] {
    let mut counts = [0.0f32; 10];

    for (emotion, _) in classifications {
        let idx = *emotion as usize;
        counts[idx] += 1.0;
    }

    let total = classifications.len() as f32;
    let mut result = [(EmotionCategory::Neutral, 0.0f32); 10];

    for (i, count) in counts.iter().enumerate() {
        let emotion = EmotionCategory::all()[i];
        let proportion = if total > 0.0 { *count / total } else { 0.0 };
        result[i] = (emotion, proportion);
    }

    result
}

// =============================================================================
// NEW API: DOCAS→Emotion Reverse Projection Functions
// =============================================================================

/// Maps DOCAS vector to emotional state
///
/// Uses Δτ (trust delta), velocity (dDOCAS/dt), and absolute DOCAS values
/// Returns primary emotion and confidence score
///
/// Per §6.13: All emotion derives from trust deviation Δτ = τ - τ₀
///
/// # Arguments
/// * `docas` - [D, O, C, A, S] neuromodulator levels
/// * `docas_prev` - Previous tick for velocity computation (optional)
/// * `trust` - Current trust τ
/// * `trust_prev` - Previous trust τ
///
// TAG: surface=simulation owner=research-team rule=SIM-001
/// # Returns
/// * `(Emotion, f32)` - (emotion, confidence)
///
/// Context for emotion classification
struct EmotionContext {
    d: f32, // Dopamine
    o: f32, // Oxytocin
    c: f32, // Cortisol
    a: f32, // Adrenaline
    s: f32, // Serotonin
    velocity: f32,
    delta_tau: f32,
    trust: f32,
    docas_prev: Option<[f32; 5]>,
}

impl EmotionContext {
    const HIGH_THRESHOLD: f32 = 0.6;
    const LOW_THRESHOLD: f32 = 0.4;
    const RAPID_CHANGE_THRESHOLD: f32 = 0.15;

    fn new(docas: &[f32; 5], docas_prev: Option<&[f32; 5]>, trust: f32, trust_prev: f32) -> Self {
        let velocity = docas_prev.map_or(0.0, |prev| {
            let mut v = 0.0f32;
            for i in 0..5 {
                v += (docas[i] - prev[i]).powi(2);
                // TAG: surface=simulation owner=research-team rule=SIM-001
            }
            v.sqrt()
        });

        Self {
            d: docas[0],
            o: docas[1],
            c: docas[2],
            a: docas[3],
            s: docas[4],
            velocity,
            delta_tau: trust - trust_prev,
            trust,
            docas_prev: docas_prev.copied(),
        }
    }

    // Emotion detection predicates
    fn is_surprise(&self) -> bool {
        self.velocity > Self::RAPID_CHANGE_THRESHOLD && self.delta_tau.abs() > 0.10
    }

    fn is_joy(&self) -> bool {
        self.o > Self::HIGH_THRESHOLD && self.c < Self::LOW_THRESHOLD && self.delta_tau > 0.15
    }

    fn is_fear(&self) -> bool {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        self.c > Self::HIGH_THRESHOLD && self.a > 0.5 && self.delta_tau < -0.10
    }

    fn is_anger(&self) -> bool {
        if self.a <= Self::HIGH_THRESHOLD {
            return false;
        }
        let a_velocity = self.docas_prev.map_or(0.0, |prev| self.a - prev[3]);
        a_velocity > 0.05 || (self.delta_tau < -0.10 && self.delta_tau > -0.30)
    }

    fn is_contentment(&self) -> bool {
        self.s > Self::HIGH_THRESHOLD && self.velocity < 0.05 && self.delta_tau.abs() < 0.10
    }

    fn is_trust(&self) -> bool {
        self.d > Self::HIGH_THRESHOLD && self.s > Self::HIGH_THRESHOLD && self.delta_tau > 0.0
    }

    fn is_sadness(&self) -> bool {
        if self.c <= Self::HIGH_THRESHOLD || self.trust >= 0.6 {
            return false;
        }
        let c_velocity = self.docas_prev.map_or(0.0, |prev| self.c - prev[2]);
        c_velocity > 0.0 || self.delta_tau < -0.15
    }

    fn is_disgust(&self) -> bool {
        self.c > Self::HIGH_THRESHOLD && self.d < Self::LOW_THRESHOLD
    }

    fn is_flow(&self, balance: f32) -> bool {
        balance > 0.6 && self.velocity < 0.03 && self.trust > 0.8
    }

    fn is_neutral(&self, balance: f32) -> bool {
        balance > 0.5 && self.delta_tau.abs() < 0.05 && self.velocity < 0.02
    }
}

/// Maps DOCAS vector to emotional state
///
/// Uses Δτ (trust delta), velocity (dDOCAS/dt), and absolute DOCAS values
/// Returns primary emotion and confidence score
///
/// Per §6.13: All emotion derives from trust deviation Δτ = τ - τ₀
///
/// # Arguments
/// * `docas` - [D, O, C, A, S] neuromodulator levels
// TAG: surface=simulation owner=research-team rule=SIM-001
/// * `docas_prev` - Previous tick for velocity computation (optional)
/// * `trust` - Current trust τ
/// * `trust_prev` - Previous trust τ
///
/// # Returns
/// * `(Emotion, f32)` - (emotion, confidence)
#[must_use]
pub fn classify_emotion(
    docas: &[f32; 5],
    docas_prev: Option<&[f32; 5]>,
    trust: f32,
    trust_prev: f32,
) -> (Emotion, f32) {
    let ctx = EmotionContext::new(docas, docas_prev, trust, trust_prev);

    // Classification rules per §6.13.4

    // 1. Surprise: Rapid changes in any dimension take priority
    if ctx.is_surprise() {
        return (Emotion::Surprise, (ctx.velocity * 2.0).min(1.0));
    }

    // 2. Joy: High O + Low C + positive Δτ
    if ctx.is_joy() {
        let confidence = ((ctx.o - 0.5) + (0.5 - ctx.c) + ctx.delta_tau) / 1.5;
        return (Emotion::Joy, confidence.clamp(0.0, 1.0));
    }

    // 3. Fear: High C + High A + negative Δτ
    if ctx.is_fear() {
        let confidence = ((ctx.c - 0.5) + (ctx.a - 0.3) + ctx.delta_tau.abs()) / 1.5;
        return (Emotion::Fear, confidence.clamp(0.0, 1.0));
    }

    // 4. Anger: High A + increasing (positive velocity in A)
    if ctx.is_anger() {
        let a_velocity = ctx.docas_prev.map_or(0.0, |prev| ctx.a - prev[3]);
        let confidence = ((ctx.a - 0.5) + a_velocity.max(0.0) + ctx.delta_tau.abs()) / 1.5;
        return (Emotion::Anger, confidence.clamp(0.0, 1.0));
    }

    // 5. Contentment: High S + stable + low velocity
    if ctx.is_contentment() {
        let confidence = ((ctx.s - 0.5) + (0.05 - ctx.velocity) * 2.0) / 1.0;
        return (Emotion::Contentment, confidence.clamp(0.0, 1.0));
    }

    // 6. Trust: High D + High S + positive Δτ
    if ctx.is_trust() {
        let confidence = ((ctx.d - 0.5) + (ctx.s - 0.5) + ctx.delta_tau) / 1.5;
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        return (Emotion::Awe, confidence.clamp(0.0, 1.0)); // Awe represents high trust state
    }

    // 7. Sadness: High C + increasing C + low trust
    if ctx.is_sadness() {
        let c_velocity = ctx.docas_prev.map_or(0.0, |prev| ctx.c - prev[2]);
        let confidence = ((ctx.c - 0.5) + c_velocity.max(0.0) + (0.6 - ctx.trust)) / 1.5;
        return (Emotion::Sadness, confidence.clamp(0.0, 1.0));
    }

    // 8. Disgust: High C + Low D (withdrawal signal)
    if ctx.is_disgust() {
        let confidence = ((ctx.c - 0.5) + (0.4 - ctx.d)) / 1.0;
        return (Emotion::Disgust, confidence.clamp(0.0, 1.0));
    }

    // 9. Flow: Balanced DOCAS + stable + high trust
    let balance = 1.0
        - ((ctx.d - 0.5).abs()
            + (ctx.o - 0.5).abs()
            + (ctx.c - 0.5).abs()
            + (ctx.a - 0.5).abs()
            + (ctx.s - 0.5).abs())
            / 2.5;
    if ctx.is_flow(balance) {
        return (Emotion::Flow, balance);
    }
    // TAG: surface=simulation owner=research-team rule=SIM-001

    // 10. Neutral: Balanced, minimal deviation
    if ctx.is_neutral(balance) {
        return (Emotion::Neutral, balance);
    }

    // Default: Neutral with low confidence
    (Emotion::Neutral, 0.3)
}

/// Get emotional trajectory as time series
///
/// Extracts emotion history from `EmotionState` and returns as a vector of (tick, emotion) pairs
///
/// # Arguments
/// * `history` - History of (tick, emotion, intensity) tuples from `EmotionState`
///
/// # Returns
/// * `Vec<(Tick, Emotion)>` - Emotional trajectory
#[must_use]
pub fn emotion_trajectory(history: &[(u64, EmotionCategory, f32)]) -> Vec<(u64, Emotion)> {
    history
        .iter()
        .map(|(tick, emotion, _)| (*tick, Emotion::from_category(*emotion)))
        .collect()
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
/// Compute emotional stability (variance of emotion valences)
///
/// Returns stability score in [0, 1] where 1.0 = perfectly stable, 0.0 = highly unstable
///
/// # Arguments
/// * `history` - History of (tick, emotion, intensity) tuples
///
/// # Returns
/// * `f32` - Stability score in [0, 1]
#[must_use]
pub fn emotional_stability(history: &[(u64, EmotionCategory, f32)]) -> f32 {
    if history.len() < 2 {
        return 1.0; // Maximum stability with no variation
    }

    // Convert emotions to valences
    let valences: Vec<f32> = history
        .iter()
        .map(|(_, emotion, _)| emotion.valence())
        .collect();

    // Compute mean valence
    let mean = valences.iter().sum::<f32>() / valences.len() as f32;

    // Compute variance
    let variance = valences.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / valences.len() as f32;

    // Return stability as 1 - normalized variance
    // Max variance for valences in [-1, 1] is 1.0
    (1.0 - variance).clamp(0.0, 1.0)
}

/// Check if agent is in "crisis" state (sustained negative emotions)
///
/// Crisis is defined as having predominantly negative emotions over the specified window.
/// Per POM EMO-C2: Trust collapse preceded by Fear/Anger states
///
/// # Arguments
/// * `history` - History of (tick, emotion, intensity) tuples
/// * `window` - Number of recent ticks to consider
///
/// # Returns
/// * `bool` - True if agent is in crisis state
#[must_use]
pub fn is_in_crisis(history: &[(u64, EmotionCategory, f32)], window: usize) -> bool {
    if history.len() < window.max(1) {
        return false; // Not enough history to determine crisis
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Get recent history
    let recent: Vec<_> = history.iter().rev().take(window).collect();

    // Count negative emotions
    let negative_count = recent
        .iter()
        .filter(|(_, emotion, _)| emotion.valence() < 0.0)
        .count();

    // Crisis if more than 60% of recent emotions are negative
    let crisis_threshold = (window as f32 * 0.6) as usize;
    negative_count >= crisis_threshold
}

/// Compute emotional intensity time series
///
/// Returns the intensity values from emotion history
///
/// # Arguments
/// * `history` - History of (tick, emotion, intensity) tuples
///
/// # Returns
/// * `Vec<f32>` - Intensity time series
#[must_use]
pub fn emotion_intensity_series(history: &[(u64, EmotionCategory, f32)]) -> Vec<f32> {
    history.iter().map(|(_, _, intensity)| *intensity).collect()
}

/// Detect emotional transitions
///
/// Identifies points where emotion category changes
///
/// # Arguments
/// * `history` - History of (tick, emotion, intensity) tuples
///
/// # Returns
/// * `Vec<(u64, EmotionCategory, EmotionCategory)>` - (tick, `from_emotion`, `to_emotion`)
#[must_use]
pub fn detect_transitions(
    history: &[(u64, EmotionCategory, f32)],
) -> Vec<(u64, EmotionCategory, EmotionCategory)> {
    if history.len() < 2 {
        return Vec::new();
    }

    let mut transitions = Vec::new();
    for i in 1..history.len() {
        let (_, prev_emotion, _) = history[i - 1];
        let (curr_tick, curr_emotion, _) = history[i];

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        if prev_emotion != curr_emotion {
            transitions.push((curr_tick, prev_emotion, curr_emotion));
        }
    }

    transitions
}

/// Compute emotional valence trajectory
///
/// Returns valence time series for plotting
///
/// # Arguments
/// * `history` - History of (tick, emotion, intensity) tuples
///
/// # Returns
/// * `Vec<(u64, f32)>` - (tick, valence) pairs
#[must_use]
pub fn valence_trajectory(history: &[(u64, EmotionCategory, f32)]) -> Vec<(u64, f32)> {
    history
        .iter()
        .map(|(tick, emotion, _)| (*tick, emotion.valence()))
        .collect()
}

/// Compute emotional arousal trajectory
///
// TAG: surface=simulation owner=research-team rule=SIM-001
/// Returns arousal time series for plotting
///
/// # Arguments
/// * `history` - History of (tick, emotion, intensity) tuples
///
/// # Returns
/// * `Vec<(u64, f32)>` - (tick, arousal) pairs
#[must_use]
pub fn arousal_trajectory(history: &[(u64, EmotionCategory, f32)]) -> Vec<(u64, f32)> {
    history
        .iter()
        .map(|(tick, emotion, _)| (*tick, emotion.arousal()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::docas::DocasState;

    fn classifier() -> EmotionClassifier {
        EmotionClassifier::new()
    }

    #[test]
    fn test_compute_intensity_positive() {
        let docas = DocasState::new([0.9, 0.9, 0.1, 0.1, 0.9]);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let tau_zero = 0.5;
        let intensity = compute_intensity(&docas, tau_zero);
        assert!(intensity > 0.0);
    }

    #[test]
    fn test_compute_intensity_zero() {
        let docas = DocasState::new([0.5, 0.5, 0.5, 0.5, 0.5]);
        let tau_zero = docas.compute_tau();
        let intensity = compute_intensity(&docas, tau_zero);
        assert!(intensity < 0.01);
    }

    #[test]
    fn test_compute_valence_positive() {
        let docas = DocasState::new([0.9, 0.9, 0.1, 0.1, 0.9]);
        let tau_zero = 0.5;
        let valence = compute_valence(&docas, tau_zero);
        assert_eq!(valence, 1.0);
    }

    #[test]
    fn test_compute_valence_negative() {
        let docas = DocasState::new([0.1, 0.1, 0.9, 0.9, 0.1]);
        let tau_zero = 0.5;
        let valence = compute_valence(&docas, tau_zero);
        assert_eq!(valence, -1.0);
    }

    #[test]
    fn test_compute_valence_neutral() {
        let docas = DocasState::new([0.5, 0.5, 0.5, 0.5, 0.5]);
        let tau_zero = docas.compute_tau();
        let valence = compute_valence(&docas, tau_zero);
        assert_eq!(valence, 0.0);
    }

    #[test]
    fn test_batch_classify() {
        let classifier = classifier();
        let states = vec![
            DocasState::new([0.9, 0.9, 0.1, 0.1, 0.9]),
            DocasState::new([0.1, 0.1, 0.9, 0.9, 0.1]),
        ];
        let tau_zeros = vec![0.5, 0.5];
        let results = batch_classify(&classifier, &states, &tau_zeros);
        assert_eq!(results.len(), 2);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_population_emotion_distribution() {
        let classifications = vec![
            (EmotionCategory::Joy, 0.8),
            (EmotionCategory::Joy, 0.7),
            (EmotionCategory::Fear, 0.6),
            (EmotionCategory::Neutral, 0.5),
        ];
        let dist = population_emotion_distribution(&classifications);
        // Joy should have proportion 0.5 (2/4)
        assert!((dist[EmotionCategory::Joy as usize].1 - 0.5).abs() < 0.01);
        // Fear should have proportion 0.25 (1/4)
        assert!((dist[EmotionCategory::Fear as usize].1 - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_emotional_stability_constant() {
        let history = vec![
            (0, EmotionCategory::Joy, 0.5),
            (1, EmotionCategory::Joy, 0.5),
            (2, EmotionCategory::Joy, 0.5),
        ];
        let stability = emotional_stability(&history);
        // Constant emotions should have high stability
        assert!(stability > 0.9);
    }

    #[test]
    fn test_emotional_stability_variable() {
        let history = vec![
            (0, EmotionCategory::Joy, 0.9),
            (1, EmotionCategory::Fear, 0.1),
            (2, EmotionCategory::Anger, 0.2),
        ];
        let stability = emotional_stability(&history);
        // Variable emotions should have lower stability
        assert!(stability < 0.5);
    }

    #[test]
    fn test_is_in_crisis_true() {
        let history = vec![
            (0, EmotionCategory::Fear, 0.8),
            (1, EmotionCategory::Anger, 0.9),
            (2, EmotionCategory::Sadness, 0.85),
            (3, EmotionCategory::Fear, 0.87),
            (4, EmotionCategory::Anger, 0.82),
        ];
        assert!(is_in_crisis(&history, 5));
    }
    // TAG: surface=simulation owner=research-team rule=SIM-001

    #[test]
    fn test_is_in_crisis_false() {
        let history = vec![
            (0, EmotionCategory::Joy, 0.8),
            (1, EmotionCategory::Contentment, 0.7),
            (2, EmotionCategory::Flow, 0.6),
        ];
        assert!(!is_in_crisis(&history, 5));
    }

    #[test]
    fn test_emotion_intensity_series() {
        let history = vec![
            (0, EmotionCategory::Joy, 0.5),
            (1, EmotionCategory::Fear, 0.8),
            (2, EmotionCategory::Anger, 0.3),
        ];
        let series = emotion_intensity_series(&history);
        assert_eq!(series, vec![0.5, 0.8, 0.3]);
    }

    #[test]
    fn test_valence_trajectory() {
        let history = vec![
            (0, EmotionCategory::Joy, 0.5),
            (1, EmotionCategory::Fear, 0.8),
        ];
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let trajectory = valence_trajectory(&history);
        assert_eq!(trajectory.len(), 2);
        assert_eq!(trajectory[0].0, 0);
        assert_eq!(trajectory[1].0, 1);
        // Joy should have positive valence, fear negative
        assert!(trajectory[0].1 > 0.0);
        assert!(trajectory[1].1 < 0.0);
    }

    #[test]
    fn test_arousal_trajectory() {
        let history = vec![
            (0, EmotionCategory::Joy, 0.5),
            (1, EmotionCategory::Contentment, 0.3),
        ];
        let trajectory = arousal_trajectory(&history);
        assert_eq!(trajectory.len(), 2);
    }

    #[test]
    fn test_classify_emotion() {
        let docas = [0.9, 0.9, 0.1, 0.1, 0.9];
        let (emotion, confidence) = classify_emotion(&docas, None, 0.8, 0.5);
        assert!((0.0..=1.0).contains(&confidence));
        // High dopamine + oxytocin, low cortisol should yield Joy
        assert_eq!(emotion, Emotion::Joy);
    }

    #[test]
    fn test_emotion_state_velocity_updates_previous_tau() {
        let mut state = EmotionState::new(0.5);
        assert_eq!(state.previous_tau, 0.5);

        // Step 1: tau rises to 0.8
        state.update_baseline(0.8);
        assert_eq!(state.previous_tau, 0.5); // previous was initial 0.5
        let vel1 = state.delta_tau_velocity(0.8);
        assert!((vel1 - 0.3).abs() < 1e-5);

        // Step 2: tau rises further to 1.0
        state.update_baseline(1.0);
        assert_eq!(state.previous_tau, 0.8); // previous was step 1's 0.8!
        let vel2 = state.delta_tau_velocity(1.0);
        assert!((vel2 - 0.2).abs() < 1e-5);
    }
}
