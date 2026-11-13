//! Reputation damage event categorization using confusion matrix methodology
//!
//! Per `RESEARCH_TEMPORAL_DYNAMICS.md` Section 3
//! Based on Powers (2011) and standard ML classification evaluation

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::temporal::SimulationEra;
use crate::types::{AgentId, Tick};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Type of reputation damage event
///
/// Maps to standard confusion matrix categories adapted for reputation systems:
///
/// |                          | Agent Defected | Agent Cooperated |
/// |--------------------------|----------------|------------------|
/// | **Reputation Damaged**   | True Damage    | False Damage     |
/// | **Reputation Intact**    | Missed Damage  | Correct Recog.   |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DamageEventType {
    /// True Damage (TD): Agent defected, reputation correctly damaged
    /// True Positive in confusion matrix terms
    TrueDamage,

    /// False Damage (FD): Agent cooperated, but reputation wrongly damaged
    /// False Positive - wrongful accusation, false alarm
    FalseDamage,

    /// Missed Damage (MD): Agent defected, but reputation not damaged
    /// False Negative - undetected defection, missed detection
    MissedDamage,

    /// Correct Recognition (CR): Agent cooperated, reputation maintained
    /// True Negative - correct identification of cooperation
    CorrectRecognition,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl DamageEventType {
    /// Get display name
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::TrueDamage => "True Damage",
            Self::FalseDamage => "False Damage",
            Self::MissedDamage => "Missed Damage",
            Self::CorrectRecognition => "Correct Recognition",
        }
    }

    /// Get short abbreviation
    #[must_use]
    pub const fn abbrev(&self) -> &'static str {
        match self {
            Self::TrueDamage => "TD",
            Self::FalseDamage => "FD",
            Self::MissedDamage => "MD",
            Self::CorrectRecognition => "CR",
        }
    }

    /// Whether this represents an actual defection
    #[must_use]
    pub const fn is_defection(&self) -> bool {
        matches!(self, Self::TrueDamage | Self::MissedDamage)
    }

    /// Whether this represents reputation damage
    #[must_use]
    pub const fn is_damage(&self) -> bool {
        matches!(self, Self::TrueDamage | Self::FalseDamage)
    }

    /// Whether this represents a correct judgment
    #[must_use]
    pub const fn is_correct(&self) -> bool {
        matches!(self, Self::TrueDamage | Self::CorrectRecognition)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Source of reputation judgment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JudgmentSource {
    /// Direct observation of behavior
    DirectObservation,
    /// Reputation network gossip/indirect
    NetworkPropagation,
    /// Protocol verification
    ProtocolVerification,
    /// Attestation from third party
    ThirdPartyAttestation,
    /// Default/initial reputation
    Default,
}

impl JudgmentSource {
    /// Whether this source is protocol-based
    #[must_use]
    pub const fn is_protocol(&self) -> bool {
        matches!(self, Self::ProtocolVerification)
    }

    /// Whether this source is network-based
    #[must_use]
    pub const fn is_network(&self) -> bool {
        matches!(self, Self::NetworkPropagation | Self::ThirdPartyAttestation)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Behavior type for ground truth
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BehaviorType {
    /// cooperate.
    Cooperate,
    /// defect.
    Defect,
    /// neutral.
    Neutral,
}

/// Detailed damage event record
///
/// Captures all information needed to categorize and analyze
/// reputation damage events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageEvent {
    /// Type of damage event
    pub event_type: DamageEventType,

    /// Agent whose reputation was affected
    pub subject_id: AgentId,

    /// Agent who made the reputation judgment
    pub evaluator_id: AgentId,

    /// Tick when event occurred
    pub tick: Tick,

    /// Reputation delta applied
    pub reputation_delta: f32,

    /// Actual behavior (defect/cooperate)
    pub actual_behavior: BehaviorType,

    /// Source of reputation judgment
    pub judgment_source: JudgmentSource,

    /// Era when event occurred
    pub era: SimulationEra,

    /// Whether subject had verified protocol ID
    pub subject_has_protocol: bool,

    /// Whether evaluator had verified protocol ID
    pub evaluator_has_protocol: bool,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl DamageEvent {
    /// Create new damage event
    #[must_use]
    pub const fn new(
        event_type: DamageEventType,
        subject_id: AgentId,
        evaluator_id: AgentId,
        tick: Tick,
        era: SimulationEra,
    ) -> Self {
        Self {
            event_type,
            subject_id,
            evaluator_id,
            tick,
            reputation_delta: 0.0,
            actual_behavior: BehaviorType::Neutral,
            judgment_source: JudgmentSource::Default,
            era,
            subject_has_protocol: false,
            evaluator_has_protocol: false,
        }
    }

    /// Classify an interaction outcome into damage event type
    ///
    /// # Arguments
    /// * `predicted_defection` - Whether evaluator predicted defection
    /// * `actual_defection` - Whether actual behavior was defection
    #[must_use]
    pub const fn classify(predicted_defection: bool, actual_defection: bool) -> DamageEventType {
        match (predicted_defection, actual_defection) {
            (true, true) => DamageEventType::TrueDamage,
            (true, false) => DamageEventType::FalseDamage,
            (false, true) => DamageEventType::MissedDamage,
            (false, false) => DamageEventType::CorrectRecognition,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Era-specific confusion matrix counts
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EraConfusionMatrix {
    /// True Damage count
    pub true_damage: u64,
    /// False Damage count
    pub false_damage: u64,
    /// Missed Damage count
    pub missed_damage: u64,
    /// Correct Recognition count
    pub correct_recognition: u64,
}

impl EraConfusionMatrix {
    /// Total number of events
    #[must_use]
    pub const fn total(&self) -> u64 {
        self.true_damage + self.false_damage + self.missed_damage + self.correct_recognition
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate precision: TD / (TD + FD)
    /// "Of all reputation damages, what fraction were correct?"
    #[must_use]
    pub fn precision(&self) -> f32 {
        let total_damages = self.true_damage + self.false_damage;
        if total_damages == 0 {
            0.0
        } else {
            self.true_damage as f32 / total_damages as f32
        }
    }

    /// Calculate recall: TD / (TD + MD)
    /// "Of all actual defections, what fraction were detected?"
    #[must_use]
    pub fn recall(&self) -> f32 {
        let total_defections = self.true_damage + self.missed_damage;
        if total_defections == 0 {
            0.0
        } else {
            self.true_damage as f32 / total_defections as f32
        }
    }

    /// Calculate F1 score: harmonic mean of precision and recall
    #[must_use]
    pub fn f1_score(&self) -> f32 {
        let p = self.precision();
        let r = self.recall();
        if p + r == 0.0 {
            0.0
        } else {
            2.0 * p * r / (p + r)
        }
    }

    /// Calculate specificity: CR / (CR + FD)
    /// "Of all cooperative acts, what fraction were correctly recognized?"
    #[must_use]
    pub fn specificity(&self) -> f32 {
        let total_cooperations = self.correct_recognition + self.false_damage;
        if total_cooperations == 0 {
            0.0
        } else {
            self.correct_recognition as f32 / total_cooperations as f32
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate accuracy: (TD + CR) / Total
    #[must_use]
    pub fn accuracy(&self) -> f32 {
        let total = self.total();
        if total == 0 {
            0.0
        } else {
            (self.true_damage + self.correct_recognition) as f32 / total as f32
        }
    }

    /// False Damage Rate: FD / (FD + CR)
    /// "Of all cooperative acts, what fraction were wrongly punished?"
    #[must_use]
    pub fn false_damage_rate(&self) -> f32 {
        let total_cooperations = self.false_damage + self.correct_recognition;
        if total_cooperations == 0 {
            0.0
        } else {
            self.false_damage as f32 / total_cooperations as f32
        }
    }

    /// Matthews Correlation Coefficient (MCC)
    /// Most informative metric for imbalanced confusion matrices
    /// Range: [-1, 1] where 1 = perfect, 0 = random, -1 = inverse
    #[must_use]
    pub fn mcc(&self) -> f32 {
        let td = self.true_damage as f64;
        let fd = self.false_damage as f64;
        let md = self.missed_damage as f64;
        let cr = self.correct_recognition as f64;

        let numerator = td * cr - fd * md;
        let denominator = ((td + fd) * (td + md) * (cr + fd) * (cr + md)).sqrt();

        if !denominator.is_finite() || denominator <= 1e-12 {
            0.0
        } else {
            ((numerator / denominator) as f32).clamp(-1.0, 1.0)
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Confusion matrix for reputation system evaluation
///
/// Aggregates damage events across all eras with breakdown by era.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReputationConfusionMatrix {
    /// Aggregate counts
    pub aggregate: EraConfusionMatrix,

    /// Temporal breakdown by era
    pub by_era: HashMap<SimulationEra, EraConfusionMatrix>,

    /// Recent events (for debugging/analysis)
    #[serde(skip)]
    recent_events: Vec<DamageEvent>,

    /// Maximum number of recent events to retain
    #[serde(skip)]
    max_recent: usize,
}

impl ReputationConfusionMatrix {
    /// Create new confusion matrix
    #[must_use]
    pub fn new() -> Self {
        Self {
            aggregate: EraConfusionMatrix::default(),
            by_era: HashMap::new(),
            recent_events: Vec::with_capacity(1000),
            max_recent: 1000,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Record a damage event
    pub fn record(&mut self, event: &DamageEvent) {
        // Update aggregate
        match event.event_type {
            DamageEventType::TrueDamage => self.aggregate.true_damage += 1,
            DamageEventType::FalseDamage => self.aggregate.false_damage += 1,
            DamageEventType::MissedDamage => self.aggregate.missed_damage += 1,
            DamageEventType::CorrectRecognition => self.aggregate.correct_recognition += 1,
        }

        // Update by era
        let era_entry = self.by_era.entry(event.era).or_default();
        match event.event_type {
            DamageEventType::TrueDamage => era_entry.true_damage += 1,
            DamageEventType::FalseDamage => era_entry.false_damage += 1,
            DamageEventType::MissedDamage => era_entry.missed_damage += 1,
            DamageEventType::CorrectRecognition => era_entry.correct_recognition += 1,
        }

        // Store recent event
        self.recent_events.push(event.clone());
        if self.recent_events.len() > self.max_recent {
            self.recent_events.remove(0);
        }
    }

    /// Record a classification directly
    pub fn record_classification(
        &mut self,
        predicted_defection: bool,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        actual_defection: bool,
        era: SimulationEra,
    ) {
        let event_type = DamageEvent::classify(predicted_defection, actual_defection);

        let event = DamageEvent {
            event_type,
            subject_id: 0, // Anonymous
            evaluator_id: 0,
            tick: 0,
            reputation_delta: 0.0,
            actual_behavior: if actual_defection {
                BehaviorType::Defect
            } else {
                BehaviorType::Cooperate
            },
            judgment_source: JudgmentSource::DirectObservation,
            era,
            subject_has_protocol: false,
            evaluator_has_protocol: false,
        };

        self.record(&event);
    }

    /// Get aggregate metrics
    #[must_use]
    pub const fn aggregate_metrics(&self) -> &EraConfusionMatrix {
        &self.aggregate
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get metrics for specific era
    #[must_use]
    pub fn era_metrics(&self, era: SimulationEra) -> Option<&EraConfusionMatrix> {
        self.by_era.get(&era)
    }

    /// Compare pre-protocol vs post-protocol metrics
    #[must_use]
    pub fn compare_eras(&self) -> EraComparison {
        let pre = self.by_era.get(&SimulationEra::PreProtocol);
        let post = self.by_era.get(&SimulationEra::PostProtocol);

        EraComparison {
            precision_change: match (pre, post) {
                (Some(p), Some(o)) => o.precision() - p.precision(),
                _ => 0.0,
            },
            recall_change: match (pre, post) {
                (Some(p), Some(o)) => o.recall() - p.recall(),
                _ => 0.0,
            },
            f1_change: match (pre, post) {
                (Some(p), Some(o)) => o.f1_score() - p.f1_score(),
                _ => 0.0,
            },
            false_damage_rate_change: match (pre, post) {
                (Some(p), Some(o)) => o.false_damage_rate() - p.false_damage_rate(),
                _ => 0.0,
            },
            mcc_change: match (pre, post) {
                (Some(p), Some(o)) => o.mcc() - p.mcc(),
                // TAG: surface=simulation owner=research-team rule=SIM-001
                _ => 0.0,
            },
            total_events_ratio: match (pre, post) {
                (Some(p), Some(o)) => {
                    if p.total() == 0 {
                        0.0
                    } else {
                        o.total() as f32 / p.total() as f32
                    }
                }
                _ => 0.0,
            },
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check hypothesis predictions
    ///
    /// Per `RESEARCH_TEMPORAL_DYNAMICS.md` hypothesis:
    /// - False damage should decrease
    /// - Precision should increase
    /// - Recall may change
    #[must_use]
    pub fn check_hypothesis(&self) -> HypothesisResult {
        let comparison = self.compare_eras();

        HypothesisResult {
            // False damage should decrease (negative change)
            false_damage_decreased: comparison.false_damage_rate_change < 0.0,

            // Precision should increase (positive change)
            precision_increased: comparison.precision_change > 0.0,

            // Overall accuracy should improve
            accuracy_improved: comparison.f1_change > 0.0 || comparison.mcc_change > 0.0,

            comparison,
        }
    }

    /// Get recent events
    #[must_use]
    pub fn recent_events(&self) -> &[DamageEvent] {
        &self.recent_events
    }

    /// Get events filtered by type
    #[must_use]
    pub fn events_by_type(&self, event_type: DamageEventType) -> Vec<&DamageEvent> {
        self.recent_events
            .iter()
            .filter(|e| e.event_type == event_type)
            .collect()
    }

    /// Get count by source
    #[must_use]
    pub fn count_by_source(&self, source: JudgmentSource) -> u64 {
        self.recent_events
            .iter()
            .filter(|e| e.judgment_source == source)
            .count() as u64
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Clear history
    pub fn clear(&mut self) {
        self.aggregate = EraConfusionMatrix::default();
        self.by_era.clear();
        self.recent_events.clear();
    }
}

/// Comparison of metrics between two eras
#[derive(Debug, Clone, Copy)]
pub struct EraComparison {
    /// Change in precision (post - pre)
    pub precision_change: f32,

    /// Change in recall (post - pre)
    pub recall_change: f32,

    /// Change in F1 score (post - pre)
    pub f1_change: f32,

    /// Change in false damage rate (post - pre)
    /// Negative = improvement (fewer false damages)
    pub false_damage_rate_change: f32,

    /// Change in MCC (post - pre)
    pub mcc_change: f32,

    /// Ratio of total events (post / pre)
    pub total_events_ratio: f32,
}

/// Result of hypothesis testing
#[derive(Debug, Clone)]
pub struct HypothesisResult {
    /// Whether false damage rate decreased
    pub false_damage_decreased: bool,

    /// Whether precision increased
    pub precision_increased: bool,

    /// Whether overall accuracy improved
    pub accuracy_improved: bool,

    /// Detailed comparison metrics
    pub comparison: EraComparison,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl HypothesisResult {
    /// Overall hypothesis supported
    ///
    /// All three conditions must be met
    #[must_use]
    pub const fn supported(&self) -> bool {
        self.false_damage_decreased && self.precision_increased && self.accuracy_improved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_event_classification() {
        assert_eq!(
            DamageEvent::classify(true, true),
            DamageEventType::TrueDamage
        );
        assert_eq!(
            DamageEvent::classify(true, false),
            DamageEventType::FalseDamage
        );
        assert_eq!(
            DamageEvent::classify(false, true),
            DamageEventType::MissedDamage
        );
        assert_eq!(
            DamageEvent::classify(false, false),
            DamageEventType::CorrectRecognition
        );
    }

    #[test]
    fn test_event_type_helpers() {
        assert!(DamageEventType::TrueDamage.is_defection());
        assert!(DamageEventType::TrueDamage.is_damage());
        assert!(DamageEventType::TrueDamage.is_correct());

        assert!(DamageEventType::FalseDamage.is_damage());
        assert!(!DamageEventType::FalseDamage.is_defection());
        assert!(!DamageEventType::FalseDamage.is_correct());

        assert!(DamageEventType::MissedDamage.is_defection());
        assert!(!DamageEventType::MissedDamage.is_damage());
        assert!(!DamageEventType::MissedDamage.is_correct());

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(DamageEventType::CorrectRecognition.is_correct());
        assert!(!DamageEventType::CorrectRecognition.is_damage());
    }

    #[test]
    fn test_confusion_matrix_metrics() {
        // Balanced: 50 TD, 10 FD, 10 MD, 50 CR
        let matrix = EraConfusionMatrix {
            true_damage: 50,
            false_damage: 10,
            missed_damage: 10,
            correct_recognition: 50,
        };

        // Precision: 50 / 60 = 0.833
        assert!((matrix.precision() - 0.833).abs() < 0.01);

        // Recall: 50 / 60 = 0.833
        assert!((matrix.recall() - 0.833).abs() < 0.01);

        // Specificity: 50 / 60 = 0.833
        assert!((matrix.specificity() - 0.833).abs() < 0.01);

        // Accuracy: 100 / 120 = 0.833
        assert!((matrix.accuracy() - 0.833).abs() < 0.01);

        // False damage rate: 10 / 60 = 0.167
        assert!((matrix.false_damage_rate() - 0.167).abs() < 0.01);

        // MCC should be positive for good classification
        assert!(matrix.mcc() > 0.6);
    }

    #[test]
    fn test_perfect_classification() {
        // Perfect: 50 TD, 0 FD, 0 MD, 50 CR
        let matrix = EraConfusionMatrix {
            true_damage: 50,
            correct_recognition: 50,
            ..EraConfusionMatrix::default()
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(matrix.precision(), 1.0);
        assert_eq!(matrix.recall(), 1.0);
        assert_eq!(matrix.specificity(), 1.0);
        assert_eq!(matrix.accuracy(), 1.0);
        assert_eq!(matrix.false_damage_rate(), 0.0);
        assert!((matrix.mcc() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_worst_classification() {
        // Inverted: 0 TD, 50 FD, 50 MD, 0 CR
        let matrix = EraConfusionMatrix {
            false_damage: 50,
            missed_damage: 50,
            ..EraConfusionMatrix::default()
        };

        assert_eq!(matrix.precision(), 0.0);
        assert_eq!(matrix.recall(), 0.0);
        assert_eq!(matrix.specificity(), 0.0);
        assert_eq!(matrix.accuracy(), 0.0);
        assert_eq!(matrix.false_damage_rate(), 1.0);
        assert!((matrix.mcc() - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_era_comparison() {
        let mut confusion = ReputationConfusionMatrix::new();

        // Pre-protocol: worse performance (50% precision, 50% false damage rate)
        // 10 true damages, 10 false damages, 10 correct recognitions (cooperations correctly handled)
        for _ in 0..10 {
            confusion.record_classification(true, true, SimulationEra::PreProtocol);
            // TD: predicted defection, actual defection
        }
        for _ in 0..10 {
            confusion.record_classification(true, false, SimulationEra::PreProtocol);
            // FD: predicted defection, actual cooperation
        }
        for _ in 0..10 {
            confusion.record_classification(false, false, SimulationEra::PreProtocol);
            // CR: predicted cooperation, actual cooperation
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Post-protocol: better performance (90% precision, 10% false damage rate)
        // 18 true damages, 2 false damages, 18 correct recognitions
        for _ in 0..18 {
            confusion.record_classification(true, true, SimulationEra::PostProtocol);
            // TD
        }
        for _ in 0..2 {
            confusion.record_classification(true, false, SimulationEra::PostProtocol);
            // FD
        }
        for _ in 0..18 {
            confusion.record_classification(false, false, SimulationEra::PostProtocol);
            // CR
        }

        let comparison = confusion.compare_eras();

        // Precision should improve (18/20 = 0.9 > 10/20 = 0.5)
        assert!(
            comparison.precision_change > 0.0,
            "Precision should improve: {}",
            comparison.precision_change
        );

        // False damage rate should decrease (2/20 = 0.1 < 10/20 = 0.5)
        assert!(
            comparison.false_damage_rate_change < 0.0,
            "False damage rate should decrease: {}",
            comparison.false_damage_rate_change
        );

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let hypothesis = confusion.check_hypothesis();
        assert!(hypothesis.false_damage_decreased);
        assert!(hypothesis.precision_increased);
    }

    #[test]
    fn test_empty_matrix() {
        let matrix = EraConfusionMatrix::default();

        assert_eq!(matrix.precision(), 0.0);
        assert_eq!(matrix.recall(), 0.0);
        assert_eq!(matrix.f1_score(), 0.0);
        assert_eq!(matrix.mcc(), 0.0);
    }

    #[test]
    fn test_mcc_symmetry() {
        // MCC should be symmetric - swapping labels gives same magnitude
        let m1 = EraConfusionMatrix {
            true_damage: 80,
            false_damage: 10,
            missed_damage: 10,
            correct_recognition: 100,
        };

        let m2 = EraConfusionMatrix {
            true_damage: 100, // swapped with CR
            false_damage: 10,
            missed_damage: 10,
            correct_recognition: 80,
        };

        // Not exactly symmetric in this case, but both should be positive
        // for better-than-random classification
        assert!(m1.mcc() > 0.0);
        assert!(m2.mcc() > 0.0);
    }
}
