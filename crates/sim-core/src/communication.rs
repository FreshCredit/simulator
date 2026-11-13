//! Multi-modal communication system
//!
//! Implements gesture, facial expression, and prosody (tone of voice) channels
//! along with verbal communication for holistic agent interaction.
//! CODE-006: Large module - core simulation component
//! Research foundations:
//! - `McNeill` (1992): Gesture and speech are integrated
//! - Ekman (1992): Universal facial expressions
//! - Pell et al. (2009): Prosody conveys emotion

use serde::{Deserialize, Serialize};

use crate::docas::DocasState;
use crate::language::Lexicon;
use crate::types::{AgentId, CommunicationIntent, ConceptId};

/// Spatial reference for deictic gestures (3D coordinates)
pub type SpatialReference = (f32, f32, f32);

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Semantic frame for verbal content meaning representation
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticFrame {
    /// Frame type (e.g., "request", "statement", "question")
    pub frame_type: String,
    /// Core entities mentioned
    pub entities: Vec<String>,
    /// Predicates/actions
    pub predicates: Vec<String>,
}

/// Multi-modal communication system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultimodalCommunication {
    /// Verbal channel (speech/text content)
    pub verbal: VerbalChannel,
    /// Gesture channel (body movement)
    pub gesture: GestureChannel,
    /// Facial expression channel
    pub facial: FacialChannel,
    /// Prosody channel (tone of voice)
    pub prosody: ProsodyChannel,
    /// Integration analysis across modalities
    pub integration: ModalityIntegration,
}

/// Verbal content channel
/// Phase 2.5: Now includes concept-based semantic content for language integration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct VerbalChannel {
    /// Concept IDs for semantic content (NEW - for language integration)
    pub concept_content: Vec<ConceptId>,
    /// Raw text content for display/debugging
    pub content: String,
    /// Semantic meaning representation
    pub semantic_meaning: SemanticFrame,
    /// Words indicating certainty (e.g., "definitely", "certainly")
    pub certainty_words: Vec<String>,
    /// Words indicating uncertainty (e.g., "maybe", "perhaps", "sort of")
    pub hedge_words: Vec<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl VerbalChannel {
    /// Create a verbal channel from concept IDs
    #[must_use]
    pub fn from_concepts(concepts: &[ConceptId], text: &str) -> Self {
        Self {
            concept_content: concepts.to_vec(),
            content: text.to_string(),
            semantic_meaning: SemanticFrame::default(),
            certainty_words: Vec::new(),
            hedge_words: Vec::new(),
        }
    }
}

/// Gesture types following `McNeill`'s classification
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GestureType {
    /// No gesture
    #[default]
    None,
    /// Depicts shape/motion (e.g., drawing a circle to describe roundness)
    Iconic,
    /// Represents abstract concept (e.g., "rising" hierarchy)
    Metaphoric,
    /// Points to location/person
    Deictic,
    /// Rhythm marker, beats time with speech
    Beat,
    /// Culture-specific emblem (e.g., OK sign, thumbs up)
    Emblem,
}

/// Gesture channel
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GestureChannel {
    /// Type of gesture
    pub gesture_type: GestureType,
    /// How well image matches content (0.0 to 1.0)
    pub iconicity: f32,
    /// Whether this is a rhythm marker
    pub beat: bool,
    /// Whether gesture represents abstract concept
    pub metaphoric: bool,
    /// Target location for deictic gestures
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub deictic_target: Option<SpatialReference>,
    /// Gesture intensity (0.0 to 1.0)
    pub intensity: f32,
}

impl Default for GestureChannel {
    fn default() -> Self {
        Self {
            gesture_type: GestureType::None,
            iconicity: 0.0,
            beat: false,
            metaphoric: false,
            deictic_target: None,
            intensity: 0.5,
        }
    }
}

/// Facial expressions based on Ekman's universal emotions
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FacialExpression {
    /// Neutral expression
    #[default]
    Neutral,
    /// Happiness/joy
    Happy,
    /// Sadness
    Sad,
    /// Anger
    Angry,
    /// Fear
    Fearful,
    /// Surprise
    Surprised,
    /// Disgust
    Disgusted,
    /// Contempt
    Contemptuous,
    /// Confusion/uncertainty
    Confused,
    /// Concentration/focus
    Concentrating,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Facial expression channel
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FacialChannel {
    /// Current expression
    pub expression: FacialExpression,
    /// Expression intensity (0.0 to 1.0)
    pub intensity: f32,
    /// Left/right asymmetry (0.0 = symmetric, 1.0 = highly asymmetric)
    /// Higher asymmetry may indicate fake expressions
    pub asymmetry: f32,
    /// Duration in milliseconds
    pub duration_ms: u32,
    /// Whether a micro-expression was detected (brief involuntary leak)
    pub micro_expression: bool,
}

impl Default for FacialChannel {
    fn default() -> Self {
        Self {
            expression: FacialExpression::Neutral,
            intensity: 0.0,
            asymmetry: 0.0,
            duration_ms: 0,
            micro_expression: false,
        }
    }
}

/// Pitch contour patterns
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PitchContour {
    /// Rising pitch - questions, uncertainty
    Rising,
    /// Falling pitch - statements, certainty
    Falling,
    /// Level/monotone - boredom, neutrality
    #[default]
    Level,
    /// Rising then falling - surprise, emphasis
    RisingFalling,
    /// Falling then rising - reservation, continuation
    FallingRising,
}

/// Voice quality types
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VoiceQuality {
    /// Normal voice
    #[default]
    Modal,
    /// Breathy voice - intimacy, tiredness
    Breathy,
    /// Creaky/vocal fry - authority, finality
    Creaky,
    /// Tense voice - stress, anger
    Tense,
    /// Whispery voice - secrecy, closeness
    Whispery,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Prosody (tone of voice) channel
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProsodyChannel {
    /// Mean pitch in Hz
    pub pitch_mean: f32,
    /// Pitch range/variability in Hz
    pub pitch_range: f32,
    /// Pitch contour pattern
    pub pitch_contour: PitchContour,
    /// Speech rate in syllables per second
    pub speech_rate: f32,
    /// Mean intensity in dB
    pub intensity_mean: f32,
    /// Intensity range in dB
    pub intensity_range: f32,
    /// Voice quality
    pub voice_quality: VoiceQuality,
}

impl Default for ProsodyChannel {
    fn default() -> Self {
        Self {
            pitch_mean: 120.0,
            pitch_range: 50.0,
            pitch_contour: PitchContour::Level,
            speech_rate: 4.0,
            intensity_mean: 65.0,
            intensity_range: 10.0,
            voice_quality: VoiceQuality::Modal,
        }
    }
}

/// Which modality dominates in case of conflict
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ModalityDominance {
    /// Trust words over other cues
    #[default]
    VerbalDominant,
    /// Trust body language
    GesturalDominant,
    /// Trust facial expressions
    FacialDominant,
    /// Trust tone of voice
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    ProsodicDominant,
    /// All modalities balanced
    Balanced,
}

/// Integration analysis across modalities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModalityIntegration {
    /// How well modalities align (0.0 = contradictory, 1.0 = perfectly aligned)
    pub alignment_score: f32,
    /// Which modality dominates interpretation
    pub dominance: ModalityDominance,
    /// Timing coordination (0.0 = out of sync, 1.0 = perfectly synchronized)
    pub temporal_sync: f32,
    /// Semantic congruence across modalities (0.0 to 1.0)
    pub semantic_congruence: f32,
}

impl Default for ModalityIntegration {
    fn default() -> Self {
        Self {
            alignment_score: 1.0,
            dominance: ModalityDominance::VerbalDominant,
            temporal_sync: 1.0,
            semantic_congruence: 1.0,
        }
    }
}

/// Deception indicators detected in communication
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DeceptionSignals {
    /// Verbal hedging or hiding
    pub verbal_hiding: bool,
    /// Facial asymmetry suggesting fake expression
    pub facial_leakage: bool,
    /// Micro-expression detected (involuntary leak)
    pub micro_expression_detected: bool,
    /// Vocal stress indicators
    pub vocal_stress: bool,
    /// Reduced gestures compared to baseline
    pub gesture_suppression: bool,
    /// Modalities contradict each other
    pub modality_mismatch: bool,
    /// Overall deception risk score (0.0 to 1.0)
    pub overall_deception_risk: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Illocutionary force of speech act (from speech act theory)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IllocutionaryForce {
    /// Statement/assertion ("The sky is blue")
    Assertive,
    /// Request/command ("Close the door")
    Directive,
    /// Promise/commitment ("I'll do it tomorrow")
    Commissive,
    /// Expressing emotion ("Thank you", "I'm sorry")
    Expressive,
    /// Declaration that changes state ("You're fired")
    Declarative,
}

/// Interpretation of received communication
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Interpretation {
    /// Literal semantic meaning
    pub literal_meaning: String,
    /// Implicatures (what is implied beyond literal meaning)
    pub implicatures: Vec<String>,
    /// Illocutionary force of the speech act
    pub illocutionary_force: IllocutionaryForce,
    /// Sincerity assessment (0.0 = clearly insincere, 1.0 = fully sincere)
    pub sincerity_assessment: f32,
}
// TAG: surface=simulation owner=platform-team rule=GENERAL-001

impl Default for Interpretation {
    fn default() -> Self {
        Self {
            literal_meaning: String::new(),
            implicatures: Vec::new(),
            illocutionary_force: IllocutionaryForce::Assertive,
            sincerity_assessment: 1.0,
        }
    }
}

/// Emotional impact of received communication
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmotionalImpact {
    /// Change in emotional valence (-1.0 to 1.0)
    pub valence_change: f32,
    /// Change in arousal (-1.0 to 1.0)
    pub arousal_change: f32,
    /// Change in trust level (-1.0 to 1.0)
    pub trust_change: f32,
}

impl Default for EmotionalImpact {
    fn default() -> Self {
        Self {
            valence_change: 0.0,
            arousal_change: 0.0,
            trust_change: 0.0,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Communication context
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommunicationContext {
    /// Social distance between sender and receiver (0.0 = intimate, 1.0 = distant)
    pub social_distance: f32,
    /// Power difference (positive = sender has more power)
    pub power_difference: f32,
    /// Urgency of communication (0.0 to 1.0)
    pub urgency: f32,
    /// Shared cultural background (0.0 to 1.0)
    pub cultural_common_ground: f32,
}

impl Default for CommunicationContext {
    fn default() -> Self {
        Self {
            social_distance: 0.5,
            power_difference: 0.0,
            urgency: 0.5,
            cultural_common_ground: 0.5,
        }
    }
}

/// Complete communicative act
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommunicativeAct {
    /// Sender agent ID
    pub sender: AgentId,
    /// Receiver agent ID (None = broadcast)
    pub receiver: Option<AgentId>,
    /// Multi-modal communication content
    pub modalities: MultimodalCommunication,
    /// Timestamp (simulation tick)
    pub timestamp: u64,
    /// Communication context
    pub context: CommunicationContext,
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
/// Communication reception and interpretation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommunicationReception {
    /// The received communicative act
    pub received_act: CommunicativeAct,
    /// Interpretation of the act
    pub interpretation: Interpretation,
    /// Emotional impact on receiver
    pub emotional_impact: EmotionalImpact,
}

impl MultimodalCommunication {
    /// Create new multi-modal communication with verbal content
    #[must_use]
    pub fn new(verbal_content: &str) -> Self {
        Self {
            verbal: VerbalChannel {
                concept_content: Vec::new(), // Phase 2.5: Empty by default, populated by generate_from_docas_with_lexicon
                content: verbal_content.to_string(),
                semantic_meaning: SemanticFrame::default(),
                certainty_words: Vec::new(),
                hedge_words: Vec::new(),
            },
            gesture: GestureChannel::default(),
            facial: FacialChannel::default(),
            prosody: ProsodyChannel::default(),
            integration: ModalityIntegration::default(),
        }
    }

    /// Set gesture based on content with fluent API
    #[must_use]
    pub fn with_gesture(mut self, gesture_type: GestureType) -> Self {
        self.gesture.gesture_type = gesture_type;
        self.gesture.intensity = match gesture_type {
            GestureType::Iconic => 0.7,
            GestureType::Beat => 0.5,
            GestureType::Deictic => 0.8,
            GestureType::Emblem => 0.6,
            GestureType::Metaphoric => 0.6,
            GestureType::None => 0.0,
        };
        self.gesture.beat = gesture_type == GestureType::Beat;
        self.gesture.metaphoric = gesture_type == GestureType::Metaphoric;
        self
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Set facial expression with fluent API
    #[must_use]
    pub fn with_expression(mut self, expression: FacialExpression, intensity: f32) -> Self {
        self.facial.expression = expression;
        self.facial.intensity = intensity.clamp(0.0, 1.0);
        self
    }

    /// Set prosody with fluent API
    #[must_use]
    pub const fn with_prosody(mut self, contour: PitchContour, quality: VoiceQuality) -> Self {
        self.prosody.pitch_contour = contour;
        self.prosody.voice_quality = quality;
        self
    }

    /// Set custom gesture intensity
    #[must_use]
    pub fn with_gesture_intensity(mut self, intensity: f32) -> Self {
        self.gesture.intensity = intensity.clamp(0.0, 1.0);
        self
    }

    /// Set deictic target for pointing gestures
    #[must_use]
    pub const fn with_deictic_target(mut self, target: SpatialReference) -> Self {
        self.gesture.deictic_target = Some(target);
        self.gesture.gesture_type = GestureType::Deictic;
        self
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Set micro-expression detection
    #[must_use]
    pub const fn with_micro_expression(mut self, detected: bool) -> Self {
        self.facial.micro_expression = detected;
        self
    }

    /// Set facial asymmetry (indicator of genuine vs fake expressions)
    #[must_use]
    pub fn with_asymmetry(mut self, asymmetry: f32) -> Self {
        self.facial.asymmetry = asymmetry.clamp(0.0, 1.0);
        self
    }

    /// Set pitch parameters
    #[must_use]
    pub fn with_pitch(mut self, mean: f32, range: f32) -> Self {
        self.prosody.pitch_mean = mean.max(50.0);
        self.prosody.pitch_range = range.max(0.0);
        self
    }

    /// Set speech rate
    #[must_use]
    pub fn with_speech_rate(mut self, syllables_per_second: f32) -> Self {
        self.prosody.speech_rate = syllables_per_second.clamp(1.0, 10.0);
        self
    }

    /// Set verbal certainty and hedge words
    #[must_use]
    pub fn with_verbal_markers(
        mut self,
        certainty_words: Vec<String>,
        hedge_words: Vec<String>,
    ) -> Self {
        self.verbal.certainty_words = certainty_words;
        self.verbal.hedge_words = hedge_words;
        self
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute overall communicative valence (-1.0 to 1.0)
    /// Positive = pleasant/positive, Negative = unpleasant/negative
    #[must_use]
    pub fn valence(&self) -> f32 {
        let verbal_valence = self.verbal_valence();
        let gesture_valence = self.gesture_valence();
        let facial_valence = self.facial_valence();
        let prosody_valence = self.prosody_valence();

        // Weighted combination based on research on multi-modal perception
        // Facial expressions most influential for emotion detection
        // Verbal content most influential for semantic meaning
        let combined = 0.4 * verbal_valence
            + 0.2 * gesture_valence
            + 0.25 * facial_valence
            + 0.15 * prosody_valence;

        combined.clamp(-1.0, 1.0)
    }

    /// Compute verbal channel valence
    fn verbal_valence(&self) -> f32 {
        let hedge_penalty = self.verbal.hedge_words.len() as f32 * -0.1;
        let certainty_bonus = self.verbal.certainty_words.len() as f32 * 0.1;
        (hedge_penalty + certainty_bonus).clamp(-1.0, 1.0)
    }

    /// Compute gesture channel valence
    fn gesture_valence(&self) -> f32 {
        use GestureType::{Beat, Deictic, Emblem, Iconic, Metaphoric, None};
        let base = match self.gesture.gesture_type {
            Iconic => 0.2,
            Beat => 0.0,
            Deictic => 0.1,
            Emblem => 0.0,
            Metaphoric => 0.1,
            None => 0.0,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        };
        base * self.gesture.intensity
    }

    /// Compute facial channel valence
    fn facial_valence(&self) -> f32 {
        use FacialExpression::{
            Angry, Concentrating, Confused, Contemptuous, Disgusted, Fearful, Happy, Neutral, Sad,
            Surprised,
        };
        let base = match self.facial.expression {
            Happy => 0.8,
            Sad => -0.6,
            Angry => -0.8,
            Fearful => -0.7,
            Disgusted => -0.5,
            Contemptuous => -0.4,
            Surprised => 0.1,
            Confused => -0.2,
            Concentrating => 0.0,
            Neutral => 0.0,
        };
        base * self.facial.intensity
    }

    /// Compute prosody channel valence
    fn prosody_valence(&self) -> f32 {
        use PitchContour::{Falling, FallingRising, Level, Rising, RisingFalling};
        use VoiceQuality::{Breathy, Creaky, Modal, Tense, Whispery};

        let contour_valence = match self.prosody.pitch_contour {
            Rising => 0.1,         // Engagement, questioning
            Falling => 0.0,        // Finality, certainty
            Level => -0.1,         // Boredom, disengagement
            RisingFalling => 0.2,  // Enthusiasm, surprise
            FallingRising => -0.1, // Reservation, doubt
        };

        let quality_valence = match self.prosody.voice_quality {
            Modal => 0.0,
            Breathy => 0.1,  // Intimacy
            Creaky => -0.1,  // Authority can be perceived as cold
            Tense => -0.3,   // Stress
            Whispery => 0.0, // Neutral for valence
        };

        f32::clamp(contour_valence + quality_valence, -1.0, 1.0)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute arousal level (0.0 = calm, 1.0 = highly aroused)
    #[must_use]
    pub fn arousal(&self) -> f32 {
        let facial_arousal = match self.facial.expression {
            FacialExpression::Angry => 0.8,
            FacialExpression::Fearful => 0.9,
            FacialExpression::Surprised => 0.9,
            FacialExpression::Happy => 0.5,
            FacialExpression::Sad => 0.2,
            _ => 0.3,
        } * self.facial.intensity;

        let prosody_arousal = (self.prosody.pitch_range / 100.0).min(1.0);
        let gesture_arousal = self.gesture.intensity * 0.5;

        (0.4 * facial_arousal + 0.3 * prosody_arousal + 0.3 * gesture_arousal).clamp(0.0, 1.0)
    }

    /// Detect deception indicators across all modalities
    #[must_use]
    pub fn deception_indicators(&self) -> DeceptionSignals {
        let mut signals = DeceptionSignals::default();

        // Verbal: excessive hedging suggests hiding information
        if self.verbal.hedge_words.len() > 2 {
            signals.verbal_hiding = true;
        }

        // Facial: asymmetry suggests fake/deliberate expression
        if self.facial.asymmetry > 0.3 {
            signals.facial_leakage = true;
        }

        // Facial: micro-expressions are involuntary truth leaks
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        if self.facial.micro_expression {
            signals.micro_expression_detected = true;
        }

        // Prosody: high pitch range indicates stress
        if self.prosody.pitch_range > 80.0 {
            signals.vocal_stress = true;
        }

        // Gesture: suppression (no gestures with long content) suggests control
        if self.gesture.gesture_type == GestureType::None && self.verbal.content.len() > 50 {
            signals.gesture_suppression = true;
        }

        // Alignment: modalities contradict each other
        let verbal_positive = self.verbal_valence() > 0.3;
        let facial_negative = self.facial_valence() < -0.3;
        if verbal_positive && facial_negative {
            signals.modality_mismatch = true;
        }

        signals.overall_deception_risk = self.compute_deception_risk(&signals);
        signals
    }

    /// Compute overall deception risk from individual signals
    fn compute_deception_risk(&self, signals: &DeceptionSignals) -> f32 {
        let mut risk = 0.0;
        if signals.verbal_hiding {
            risk += 0.15;
        }
        if signals.facial_leakage {
            risk += 0.25;
        }
        if signals.micro_expression_detected {
            risk += 0.30;
        }
        if signals.vocal_stress {
            risk += 0.15;
        }
        if signals.gesture_suppression {
            risk += 0.10;
        }
        if signals.modality_mismatch {
            risk += 0.25;
        }
        f32::min(risk, 1.0)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update integration analysis based on current modality states
    pub fn update_integration(&mut self) {
        // Compute semantic congruence across modalities
        self.integration.semantic_congruence = self.compute_semantic_congruence();

        // Compute alignment score
        self.integration.alignment_score = self.compute_alignment();

        // Determine dominance based on which modality is most intense
        self.integration.dominance = self.compute_dominance();
    }

    /// Compute semantic congruence (how well modalities agree)
    fn compute_semantic_congruence(&self) -> f32 {
        let verbal_val = self.verbal_valence();
        let facial_val = self.facial_valence();
        let gesture_val = self.gesture_valence();
        let prosody_val = self.prosody_valence();

        // Variance of valences - lower variance = higher congruence
        let mean = (verbal_val + facial_val + gesture_val + prosody_val) / 4.0;
        let variance = ((verbal_val - mean).powi(2)
            + (facial_val - mean).powi(2)
            + (gesture_val - mean).powi(2)
            + (prosody_val - mean).powi(2))
            / 4.0;

        // Convert variance to congruence (1.0 = perfectly congruent)
        (1.0 - variance.sqrt()).clamp(0.0, 1.0)
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    /// Compute alignment score
    fn compute_alignment(&self) -> f32 {
        // Check for contradictions between modalities
        let verbal_positive = self.verbal_valence() > 0.2;
        let verbal_negative = self.verbal_valence() < -0.2;
        let facial_positive = self.facial_valence() > 0.2;
        let facial_negative = self.facial_valence() < -0.2;

        // If verbal and facial contradict, lower alignment
        let contradiction =
            (verbal_positive && facial_negative) || (verbal_negative && facial_positive);

        if contradiction {
            0.3 * self.integration.semantic_congruence
        } else {
            0.7 + 0.3 * self.integration.semantic_congruence
        }
    }

    /// Determine which modality dominates
    fn compute_dominance(&self) -> ModalityDominance {
        let intensities = [
            (
                ModalityDominance::VerbalDominant,
                self.verbal.content.len() as f32 / 100.0,
            ),
            (ModalityDominance::GesturalDominant, self.gesture.intensity),
            (ModalityDominance::FacialDominant, self.facial.intensity),
            (
                ModalityDominance::ProsodicDominant,
                (self.prosody.pitch_range / 100.0).min(1.0),
            ),
        ];

        let max = intensities
            // TAG: surface=simulation owner=research-team rule=SIM-001
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        match max {
            Some((dominance, intensity)) if *intensity > 0.5 => *dominance,
            _ => ModalityDominance::Balanced,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Estimate sincerity based on modality alignment
    #[must_use]
    pub fn estimated_sincerity(&self) -> f32 {
        let deception = self.deception_indicators();
        1.0 - deception.overall_deception_risk
    }

    /// Create a communicative act from this communication
    #[must_use]
    pub const fn to_communicative_act(
        self,
        sender: AgentId,
        receiver: Option<AgentId>,
        timestamp: u64,
        context: CommunicationContext,
    ) -> CommunicativeAct {
        CommunicativeAct {
            sender,
            receiver,
            modalities: self,
            timestamp,
            context,
        }
    }
}

/// Generate communication from DOCAS state
/// Maps neuromodulator state to multi-modal expression
#[must_use]
pub fn generate_from_docas(docas: &DocasState, content: &str) -> MultimodalCommunication {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    let tau = docas.compute_tau();
    let mut comm = MultimodalCommunication::new(content);
    comm = apply_trust_expression(comm, tau, docas);
    comm = apply_hormone_effects(comm, docas);
    comm
}

fn apply_trust_expression(
    mut comm: MultimodalCommunication,
    tau: f32,
    docas: &DocasState,
) -> MultimodalCommunication {
    if tau > 1.0 {
        comm = comm
            .with_expression(FacialExpression::Happy, 0.6)
            .with_prosody(PitchContour::Falling, VoiceQuality::Modal)
            .with_gesture(GestureType::Iconic);
        comm.gesture.iconicity = 0.8;
    } else if tau < 0.5 {
        comm = comm
            .with_expression(FacialExpression::Neutral, 0.3)
            .with_prosody(PitchContour::Rising, VoiceQuality::Tense);
        if docas.cortisol() > 0.7 {
            comm = comm.with_expression(FacialExpression::Fearful, docas.cortisol() * 0.7);
        }
    } else {
        comm = comm
            .with_expression(FacialExpression::Neutral, 0.4)
            .with_prosody(PitchContour::Level, VoiceQuality::Modal);
    }
    comm
}

fn apply_hormone_effects(
    mut comm: MultimodalCommunication,
    docas: &DocasState,
) -> MultimodalCommunication {
    if docas.adrenaline() > 0.6 {
        comm.prosody.speech_rate = 5.0 + docas.adrenaline() * 2.0;
        comm.gesture.intensity = 0.7 + docas.adrenaline() * 0.2;
    }
    if docas.oxytocin() > 0.6 {
        comm.prosody.voice_quality = VoiceQuality::Breathy;
        comm = comm.with_expression(FacialExpression::Happy, docas.oxytocin() * 0.5);
    }
    comm
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Phase 2.5: Generate communication with lexicon-based semantic content
/// Returns both the multi-modal communication AND the concept IDs for language processing
pub fn generate_from_docas_with_lexicon(
    docas: &DocasState,
    lexicon: &Lexicon,
    intent: CommunicationIntent,
    content_text: &str,
    rng: &mut impl rand::Rng,
) -> (MultimodalCommunication, Vec<ConceptId>) {
    use crate::language::ConceptCategory;

    // Select concepts from lexicon based on intent
    let target_categories: Vec<ConceptCategory> = match intent {
        CommunicationIntent::Greeting => vec![ConceptCategory::Social, ConceptCategory::Emotion],
        CommunicationIntent::Threat => vec![ConceptCategory::Emotion, ConceptCategory::Action],
        CommunicationIntent::Request => vec![ConceptCategory::Social, ConceptCategory::Action],
        CommunicationIntent::ShareTrust => vec![ConceptCategory::Trust, ConceptCategory::Social],
        CommunicationIntent::Negotiate => vec![
            ConceptCategory::Trust,
            ConceptCategory::Social,
            ConceptCategory::Action,
        ],
    };

    let mut selected_concepts = Vec::new();
    for category in target_categories {
        let category_concepts: Vec<_> = lexicon
            .concepts
            .values()
            .filter(|c| c.category == category)
            .collect();

        if !category_concepts.is_empty() {
            let idx = rng.gen_range(0..category_concepts.len());
            selected_concepts.push(category_concepts[idx].id);
        }
    }

    // Generate multi-modal expression using existing logic
    let mut comm = generate_from_docas(docas, content_text);

    // Update verbal channel with concept content
    comm.verbal.concept_content.clone_from(&selected_concepts);

    (comm, selected_concepts)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Interpret a received communicative act
#[must_use]
pub fn interpret_communication(act: &CommunicativeAct) -> Interpretation {
    // Literal meaning from verbal content
    // Infer illocutionary force from prosody and context
    // Detect implicatures based on modality mismatches
    // Assess sincerity
    Interpretation {
        literal_meaning: act.modalities.verbal.content.clone(),
        illocutionary_force: infer_illocutionary_force(act),
        implicatures: detect_implicatures(act),
        sincerity_assessment: act.modalities.estimated_sincerity(),
    }
}

/// Infer illocutionary force from multi-modal cues
fn infer_illocutionary_force(act: &CommunicativeAct) -> IllocutionaryForce {
    use IllocutionaryForce::{Assertive, Commissive, Directive, Expressive};
    use PitchContour::{Falling, Rising};

    match act.modalities.prosody.pitch_contour {
        Rising => {
            // Questions are typically rising
            if act.modalities.verbal.content.ends_with('?') {
                return Assertive; // Questions are a type of assertive (request for info)
            }
            Directive // Rising without question mark might be request
        }
        Falling => {
            if act.modalities.verbal.content.contains("promise")
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
                || act.modalities.verbal.content.contains("will")
            {
                Commissive
            } else {
                Assertive
            }
        }
        _ => {
            // Check for expressive markers
            let content_lower = act.modalities.verbal.content.to_lowercase();
            if content_lower.contains("thank")
                || content_lower.contains("sorry")
                || content_lower.contains("congratulations")
            {
                Expressive
            } else {
                Assertive
            }
        }
    }
}

/// Detect conversational implicatures
fn detect_implicatures(act: &CommunicativeAct) -> Vec<String> {
    let mut implicatures = Vec::new();
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    let deception = act.modalities.deception_indicators();

    // If deception signals detected, add implicature
    if deception.overall_deception_risk > 0.5 {
        implicatures.push("Speaker may be withholding information".to_string());
    }

    // Modality mismatch implicatures
    if deception.modality_mismatch {
        implicatures.push("Speaker's words don't match emotional expression".to_string());
    }

    // Prosody-based implicatures
    match act.modalities.prosody.pitch_contour {
        PitchContour::FallingRising => {
            implicatures.push("Speaker has reservations".to_string());
        }
        PitchContour::RisingFalling => {
            implicatures.push("Speaker is emphasizing a point".to_string());
        }
        _ => {}
    }

    implicatures
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute emotional impact of receiving a communicative act
#[must_use]
pub fn compute_emotional_impact(
    act: &CommunicativeAct,
    receiver_docas: &DocasState,
) -> EmotionalImpact {
    let valence = act.modalities.valence();
    let arousal = act.modalities.arousal();

    // Trust change depends on sincerity and valence
    let sincerity = act.modalities.estimated_sincerity();
    let trust_change = if valence > 0.0 {
        valence * sincerity * 0.3
    } else {
        valence * 0.5 // Negative valence hurts trust more
    };

    // Modulate by receiver's current state
    let current_trust = receiver_docas.compute_tau();
    let trust_sensitivity = 1.0 - (current_trust / 2.0); // Lower trust = more sensitive to changes

    EmotionalImpact {
        valence_change: valence * 0.3,
        arousal_change: (arousal - 0.5) * 0.4,
        trust_change: trust_change * trust_sensitivity,
    }
}

/// Create a communication reception from a received act
#[must_use]
pub fn receive_communication(
    act: CommunicativeAct,
    receiver_docas: &DocasState,
) -> CommunicationReception {
    let interpretation = interpret_communication(&act);
    let emotional_impact = compute_emotional_impact(&act, receiver_docas);

    CommunicationReception {
        received_act: act,
        interpretation,
        emotional_impact,
    }
}

/// Phase 2.5: Receive communication with language system integration
/// This version calls `process_message_reception` to update lexicons via semantic alignment
pub fn receive_communication_with_language(
    act: CommunicativeAct,
    receiver_docas: &DocasState,
    // TAG: surface=simulation owner=research-team rule=SIM-001
    receiver_lexicon: &mut Lexicon,
    sender_lexicon: &Lexicon,
    alignment_factor: f32,
    tick: u64,
) -> CommunicationReception {
    // Standard multi-modal interpretation
    let interpretation = interpret_communication(&act);
    let emotional_impact = compute_emotional_impact(&act, receiver_docas);

    // NEW: Semantic alignment via language system
    if !act.modalities.verbal.concept_content.is_empty() {
        use crate::language::Message;

        let message = Message::new(
            act.sender,
            act.modalities.verbal.concept_content.clone(),
            tick,
        );

        crate::language::process_message_reception(
            receiver_lexicon,
            sender_lexicon,
            &message,
            alignment_factor,
        );
    }

    CommunicationReception {
        received_act: act,
        interpretation,
        emotional_impact,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Basic Construction Tests ===

    #[test]
    fn test_multimodal_communication_new() {
        let comm = MultimodalCommunication::new("Hello world");
        assert_eq!(comm.verbal.content, "Hello world");
        assert_eq!(comm.gesture.gesture_type, GestureType::None);
        assert_eq!(comm.facial.expression, FacialExpression::Neutral);
        assert_eq!(comm.prosody.pitch_mean, 120.0);
    }

    #[test]
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    fn test_with_gesture() {
        let comm = MultimodalCommunication::new("Look at this").with_gesture(GestureType::Iconic);
        assert_eq!(comm.gesture.gesture_type, GestureType::Iconic);
        assert!(comm.gesture.intensity > 0.0);
    }

    #[test]
    fn test_with_expression() {
        let comm = MultimodalCommunication::new("Great news!")
            .with_expression(FacialExpression::Happy, 0.8);
        assert_eq!(comm.facial.expression, FacialExpression::Happy);
        assert_eq!(comm.facial.intensity, 0.8);
    }

    #[test]
    fn test_with_expression_clamping() {
        let comm =
            MultimodalCommunication::new("Test").with_expression(FacialExpression::Happy, 1.5);
        assert_eq!(comm.facial.intensity, 1.0); // Clamped
    }

    #[test]
    fn test_with_prosody() {
        let comm = MultimodalCommunication::new("Really?")
            .with_prosody(PitchContour::Rising, VoiceQuality::Breathy);
        assert_eq!(comm.prosody.pitch_contour, PitchContour::Rising);
        assert_eq!(comm.prosody.voice_quality, VoiceQuality::Breathy);
    }

    // === Valence Computation Tests ===

    #[test]
    fn test_verbal_valence_with_hedges() {
        let comm = MultimodalCommunication::new("Maybe sort of").with_verbal_markers(
            vec![],
            vec!["maybe".to_string(), "sort".to_string(), "of".to_string()],
        );
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let valence = comm.verbal_valence();
        assert!(valence < 0.0); // Hedges reduce valence
    }

    #[test]
    fn test_verbal_valence_with_certainty() {
        let comm = MultimodalCommunication::new("Definitely certainly").with_verbal_markers(
            vec!["definitely".to_string(), "certainly".to_string()],
            vec![],
        );
        let valence = comm.verbal_valence();
        assert!(valence > 0.0); // Certainty increases valence
    }

    #[test]
    fn test_facial_valence_happy() {
        let comm =
            MultimodalCommunication::new("Hello").with_expression(FacialExpression::Happy, 1.0);
        let valence = comm.facial_valence();
        assert!(valence > 0.5);
    }

    #[test]
    fn test_facial_valence_angry() {
        let comm =
            MultimodalCommunication::new("Hello").with_expression(FacialExpression::Angry, 1.0);
        let valence = comm.facial_valence();
        assert!(valence < -0.5);
    }

    #[test]
    fn test_facial_valence_scales_with_intensity() {
        let comm1 =
            MultimodalCommunication::new("Hello").with_expression(FacialExpression::Happy, 0.5);
        let comm2 =
            MultimodalCommunication::new("Hello").with_expression(FacialExpression::Happy, 1.0);
        assert!(comm2.facial_valence() > comm1.facial_valence());
    }

    #[test]
    fn test_overall_valence_combination() {
        let comm = MultimodalCommunication::new("Great!")
            .with_expression(FacialExpression::Happy, 0.8)
            .with_gesture(GestureType::Iconic);
        let valence = comm.valence();
        assert!(valence > 0.0);
    }

    // === Deception Detection Tests ===
    // TAG: surface=simulation owner=research-team rule=SIM-001

    #[test]
    fn test_deception_verbal_hiding() {
        let comm = MultimodalCommunication::new("Maybe perhaps possibly").with_verbal_markers(
            vec![],
            vec![
                "maybe".to_string(),
                "perhaps".to_string(),
                "possibly".to_string(),
            ],
        );
        let signals = comm.deception_indicators();
        assert!(signals.verbal_hiding);
    }

    #[test]
    fn test_deception_facial_leakage() {
        let comm = MultimodalCommunication::new("I'm fine")
            .with_expression(FacialExpression::Happy, 0.8)
            .with_asymmetry(0.5);
        let signals = comm.deception_indicators();
        assert!(signals.facial_leakage);
    }

    #[test]
    fn test_deception_micro_expression() {
        let comm = MultimodalCommunication::new("I'm fine").with_micro_expression(true);
        let signals = comm.deception_indicators();
        assert!(signals.micro_expression_detected);
    }

    #[test]
    fn test_deception_vocal_stress() {
        let mut comm = MultimodalCommunication::new("I'm fine");
        comm.prosody.pitch_range = 100.0; // High pitch range
        let signals = comm.deception_indicators();
        assert!(signals.vocal_stress);
    }

    #[test]
    fn test_deception_gesture_suppression() {
        let comm = MultimodalCommunication::new(
            "This is a very long explanation that goes on and on about something important",
        );
        let signals = comm.deception_indicators();
        assert!(signals.gesture_suppression);
    }

    #[test]
    fn test_deception_modality_mismatch() {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Verbal: strongly positive with certainty words (> 0.3 threshold)
        // Need 4+ certainty words to exceed 0.3 threshold (0.1 per word)
        // Facial: strongly negative (Sad at high intensity < -0.3 threshold)
        // This creates a clear modality mismatch
        let mut comm =
            MultimodalCommunication::new("Everything is wonderful! Great! Excellent! Absolutely!");
        comm.verbal.certainty_words = vec![
            "wonderful".to_string(),
            "great".to_string(),
            "excellent".to_string(),
            "absolutely".to_string(),
        ];
        comm.facial.expression = FacialExpression::Sad;
        comm.facial.intensity = 0.9;

        // Verify verbal valence is strongly positive (> 0.3 for mismatch detection)
        let verbal_val = comm.verbal_valence();
        assert!(
            verbal_val > 0.3,
            "Verbal valence should be > 0.3, got {verbal_val}"
        );

        // Verify facial valence is strongly negative (< -0.3 for mismatch detection)
        let facial_val = comm.facial_valence();
        assert!(
            facial_val < -0.3,
            "Facial valence should be < -0.3, got {facial_val}"
        );

        let signals = comm.deception_indicators();
        assert!(signals.modality_mismatch, "Should detect modality mismatch");
    }

    #[test]
    fn test_deception_risk_calculation() {
        let comm = MultimodalCommunication::new("Maybe...")
            .with_expression(FacialExpression::Happy, 0.8)
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
            .with_asymmetry(0.5)
            .with_micro_expression(true);
        let signals = comm.deception_indicators();
        assert!(signals.overall_deception_risk > 0.0);
        assert!(signals.overall_deception_risk <= 1.0);
    }

    // === DOCAS Integration Tests ===

    #[test]
    fn test_generate_from_docas_high_trust() {
        let docas = DocasState::new([0.9, 0.8, 0.2, 0.1, 0.8]); // High D, O, S
        let comm = generate_from_docas(&docas, "Hello friend");
        assert_eq!(comm.facial.expression, FacialExpression::Happy);
        assert_eq!(comm.prosody.pitch_contour, PitchContour::Falling);
    }

    #[test]
    fn test_generate_from_docas_low_trust() {
        // Low trust state (high cortisol and adrenaline, low dopamine/oxytocin)
        let docas = DocasState::new([0.2, 0.2, 0.6, 0.5, 0.3]);
        let comm = generate_from_docas(&docas, "I don't know");
        // Low trust gives neutral expression with rising tone (uncertainty)
        // Only high cortisol (>0.7) triggers fear
        assert_eq!(comm.facial.expression, FacialExpression::Neutral);
        assert_eq!(comm.prosody.pitch_contour, PitchContour::Rising);
    }

    #[test]
    fn test_generate_from_docas_high_cortisol() {
        // High cortisol with low trust (tau < 0.5) should show fear
        let docas = DocasState::new([0.2, 0.2, 0.8, 0.1, 0.3]); // High cortisol, low trust
        let comm = generate_from_docas(&docas, "Watch out!");
        assert_eq!(comm.facial.expression, FacialExpression::Fearful);
    }

    #[test]
    fn test_generate_from_docas_high_adrenaline() {
        let docas = DocasState::new([0.5, 0.5, 0.2, 0.9, 0.5]); // High adrenaline
        let comm = generate_from_docas(&docas, "Quick!");
        assert!(comm.prosody.speech_rate > 5.0);
        assert!(comm.gesture.intensity > 0.7);
    }

    #[test]
    fn test_generate_from_docas_high_oxytocin() {
        let docas = DocasState::new([0.6, 0.9, 0.2, 0.1, 0.7]); // High oxytocin
        let comm = generate_from_docas(&docas, "My dear");
        assert_eq!(comm.prosody.voice_quality, VoiceQuality::Breathy);
        // TAG: surface=simulation owner=research-team rule=SIM-001
    }

    // === Integration Tests ===

    #[test]
    fn test_semantic_congruence() {
        let comm = MultimodalCommunication::new("Great!")
            .with_expression(FacialExpression::Happy, 0.8)
            .with_gesture(GestureType::Iconic);
        let mut comm = comm;
        comm.update_integration();
        assert!(comm.integration.semantic_congruence > 0.5);
    }

    #[test]
    fn test_alignment_with_contradiction() {
        let comm = MultimodalCommunication::new("I'm so happy! Great! Wonderful!")
            .with_verbal_markers(
                vec![
                    "happy".to_string(),
                    "great".to_string(),
                    "wonderful".to_string(),
                ],
                vec![],
            )
            .with_expression(FacialExpression::Sad, 0.9);
        let mut comm = comm;
        comm.update_integration();
        assert!(comm.integration.alignment_score < 0.5);
    }

    #[test]
    fn test_dominance_computation() {
        let comm = MultimodalCommunication::new("Important message here")
            .with_expression(FacialExpression::Happy, 0.9);
        let mut comm = comm;
        comm.update_integration();
        // With high facial intensity, facial should dominate
        assert_eq!(
            comm.integration.dominance,
            ModalityDominance::FacialDominant
        );
    }

    // === Communication Act Tests ===

    #[test]
    fn test_communicative_act_creation() {
        let comm = MultimodalCommunication::new("Hello");
        let act = comm.to_communicative_act(1, Some(2), 100, CommunicationContext::default());
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(act.sender, 1);
        assert_eq!(act.receiver, Some(2));
        assert_eq!(act.timestamp, 100);
    }

    #[test]
    fn test_interpret_communication() {
        let comm = MultimodalCommunication::new("Are you coming?")
            .with_prosody(PitchContour::Rising, VoiceQuality::Modal);
        let act = comm.to_communicative_act(1, Some(2), 100, CommunicationContext::default());
        let interpretation = interpret_communication(&act);
        assert!(!interpretation.literal_meaning.is_empty());
    }

    #[test]
    fn test_compute_emotional_impact() {
        let comm = MultimodalCommunication::new("Congratulations!")
            .with_expression(FacialExpression::Happy, 0.9);
        let act = comm.to_communicative_act(1, Some(2), 100, CommunicationContext::default());
        let receiver_docas = DocasState::neutral();
        let impact = compute_emotional_impact(&act, &receiver_docas);
        assert!(impact.valence_change > 0.0);
    }

    #[test]
    fn test_receive_communication() {
        let comm =
            MultimodalCommunication::new("Thank you").with_expression(FacialExpression::Happy, 0.7);
        let act = comm.to_communicative_act(1, Some(2), 100, CommunicationContext::default());
        let receiver_docas = DocasState::neutral();
        let reception = receive_communication(act, &receiver_docas);
        assert!(reception.interpretation.sincerity_assessment > 0.0);
        assert!(reception.emotional_impact.valence_change >= 0.0);
    }

    // === Enum Tests ===

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[test]
    fn test_gesture_type_variants() {
        let types = [
            GestureType::None,
            GestureType::Iconic,
            GestureType::Metaphoric,
            GestureType::Deictic,
            GestureType::Beat,
            GestureType::Emblem,
        ];
        assert_eq!(types.len(), 6);
    }

    #[test]
    fn test_facial_expression_variants() {
        let expressions = [
            FacialExpression::Neutral,
            FacialExpression::Happy,
            FacialExpression::Sad,
            FacialExpression::Angry,
            FacialExpression::Fearful,
            FacialExpression::Surprised,
            FacialExpression::Disgusted,
            FacialExpression::Contemptuous,
            FacialExpression::Confused,
            FacialExpression::Concentrating,
        ];
        assert_eq!(expressions.len(), 10);
    }

    #[test]
    fn test_pitch_contour_variants() {
        let contours = [
            PitchContour::Rising,
            PitchContour::Falling,
            PitchContour::Level,
            PitchContour::RisingFalling,
            PitchContour::FallingRising,
        ];
        assert_eq!(contours.len(), 5);
    }

    #[test]
    fn test_voice_quality_variants() {
        let qualities = [
            VoiceQuality::Modal,
            VoiceQuality::Breathy,
            VoiceQuality::Creaky,
            VoiceQuality::Tense,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            VoiceQuality::Whispery,
        ];
        assert_eq!(qualities.len(), 5);
    }

    #[test]
    fn test_modality_dominance_variants() {
        let dominances = [
            ModalityDominance::VerbalDominant,
            ModalityDominance::GesturalDominant,
            ModalityDominance::FacialDominant,
            ModalityDominance::ProsodicDominant,
            ModalityDominance::Balanced,
        ];
        assert_eq!(dominances.len(), 5);
    }

    #[test]
    fn test_illocutionary_force_variants() {
        let forces = [
            IllocutionaryForce::Assertive,
            IllocutionaryForce::Directive,
            IllocutionaryForce::Commissive,
            IllocutionaryForce::Expressive,
            IllocutionaryForce::Declarative,
        ];
        assert_eq!(forces.len(), 5);
    }

    // === Fluent API Tests ===

    #[test]
    fn test_chained_builder_pattern() {
        let comm = MultimodalCommunication::new("Hello!")
            .with_gesture(GestureType::Iconic)
            .with_gesture_intensity(0.8)
            .with_expression(FacialExpression::Happy, 0.9)
            .with_prosody(PitchContour::RisingFalling, VoiceQuality::Modal)
            .with_pitch(150.0, 60.0)
            .with_speech_rate(5.0);

        assert_eq!(comm.gesture.gesture_type, GestureType::Iconic);
        assert_eq!(comm.gesture.intensity, 0.8);
        assert_eq!(comm.facial.expression, FacialExpression::Happy);
        assert_eq!(comm.prosody.pitch_mean, 150.0);
        assert_eq!(comm.prosody.speech_rate, 5.0);
    }

    #[test]
    fn test_deictic_target() {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let target = (1.0, 2.0, 3.0);
        let comm = MultimodalCommunication::new("Look there").with_deictic_target(target);
        assert_eq!(comm.gesture.deictic_target, Some(target));
        assert_eq!(comm.gesture.gesture_type, GestureType::Deictic);
    }

    #[test]
    fn test_arousal_computation() {
        let comm = MultimodalCommunication::new("Exciting!")
            .with_expression(FacialExpression::Surprised, 0.9);
        let arousal = comm.arousal();
        assert!(arousal > 0.5);
    }

    #[test]
    fn test_speech_rate_clamping() {
        let comm = MultimodalCommunication::new("Fast!").with_speech_rate(15.0);
        assert_eq!(comm.prosody.speech_rate, 10.0); // Clamped to max
    }

    #[test]
    fn test_pitch_clamping() {
        let comm = MultimodalCommunication::new("Low").with_pitch(-50.0, 50.0);
        assert_eq!(comm.prosody.pitch_mean, 50.0); // Min pitch is 50
    }

    #[test]
    fn test_implicature_detection() {
        let comm = MultimodalCommunication::new("Well...")
            .with_prosody(PitchContour::FallingRising, VoiceQuality::Modal);
        let act = comm.to_communicative_act(1, Some(2), 100, CommunicationContext::default());
        let interpretation = interpret_communication(&act);
        // Should detect reservation from falling-rising contour
        assert!(interpretation
            .implicatures
            .iter()
            .any(|i| i.contains("reservations")));
    }
}
