//! Hypothesis Generation System (P2-RES-2)
//!
//! Automated hypothesis generation from simulation data.

use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Hypothesis priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HypothesisPriority {
    /// low.
    Low,
    /// medium.
    Medium,
    /// high.
    High,
    /// critical.
    Critical,
}

/// Hypothesis status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HypothesisStatus {
    /// generated.
    Generated,
    /// under test.
    UnderTest,
    /// supported.
    Supported,
    /// rejected.
    Rejected,
    /// inconclusive.
    Inconclusive,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// A research hypothesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    /// id.
    pub id: String,
    /// statement.
    pub statement: String,
    /// priority.
    pub priority: HypothesisPriority,
    /// status.
    pub status: HypothesisStatus,
    /// variables.
    pub variables: Vec<String>,
    /// expected relationship.
    pub expected_relationship: RelationshipType,
    /// supporting evidence.
    pub supporting_evidence: Vec<Evidence>,
    /// rejecting evidence.
    pub rejecting_evidence: Vec<Evidence>,
    /// confidence.
    pub confidence: f32,
    /// generated at.
    pub generated_at: u64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Relationship type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    /// positive correlation.
    PositiveCorrelation,
    /// negative correlation.
    NegativeCorrelation,
    /// causal.
    Causal,
    /// mediated.
    Mediated,
    /// moderated.
    Moderated,
}

/// Evidence for/against hypothesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    /// source.
    pub source: String,
    /// observation.
    pub observation: String,
    /// strength.
    pub strength: f32,
    /// timestamp.
    pub timestamp: u64,
}

/// Hypothesis generator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisConfig {
    /// correlation threshold.
    pub correlation_threshold: f32,
    /// min observations.
    pub min_observations: usize,
    /// max hypotheses.
    pub max_hypotheses: usize,
    /// auto promote.
    pub auto_promote: bool,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for HypothesisConfig {
    fn default() -> Self {
        Self {
            correlation_threshold: 0.3,
            min_observations: 5,
            max_hypotheses: 100,
            auto_promote: true,
        }
    }
}

/// Hypothesis generator engine
#[derive(Debug, Clone)]
pub struct HypothesisGenerator {
    /// config.
    pub config: HypothesisConfig,
    /// hypotheses.
    pub hypotheses: Vec<Hypothesis>,
    /// observation count.
    pub observation_count: u64,
}

impl HypothesisGenerator {
    #[must_use]
    /// new.
    pub const fn new(config: HypothesisConfig) -> Self {
        Self {
            config,
            hypotheses: Vec::new(),
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            observation_count: 0,
        }
    }

    /// Generate hypotheses from correlation matrix
    pub fn generate_from_correlations(
        &mut self,
        correlations: &[(String, String, f32)],
        tick: u64,
    ) -> Vec<Hypothesis> {
        let mut new_hypotheses = Vec::new();

        for (var_a, var_b, corr) in correlations {
            if !corr.is_finite() || corr.abs() < self.config.correlation_threshold {
                continue;
            }

            let relationship = if *corr > 0.0 {
                RelationshipType::PositiveCorrelation
            } else {
                RelationshipType::NegativeCorrelation
            };

            let hypothesis = Hypothesis {
                id: format!("H{}", self.observation_count),
                statement: format!(
                    "{} is {} correlated with {}",
                    var_a,
                    if *corr > 0.0 {
                        // TAG: surface=simulation owner=research-team rule=SIM-001
                        "positively"
                    } else {
                        "negatively"
                    },
                    var_b
                ),
                priority: self.priority_from_correlation(corr.abs()),
                status: HypothesisStatus::Generated,
                variables: vec![var_a.clone(), var_b.clone()],
                expected_relationship: relationship,
                supporting_evidence: vec![],
                rejecting_evidence: vec![],
                confidence: corr.abs(),
                generated_at: tick,
            };
            self.observation_count += 1;

            new_hypotheses.push(hypothesis);
        }

        self.hypotheses.extend(new_hypotheses.clone());

        if self.hypotheses.len() > self.config.max_hypotheses {
            self.hypotheses.sort_by(|a, b| {
                b.priority
                    .cmp(&a.priority)
                    .then_with(|| b.confidence.total_cmp(&a.confidence))
                    .then_with(|| a.id.cmp(&b.id))
            });
            self.hypotheses.truncate(self.config.max_hypotheses);
        }

        new_hypotheses
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate causal hypotheses from temporal patterns
    pub fn generate_causal_hypotheses(
        &mut self,
        temporal_patterns: &[(String, String, f32)],
        tick: u64,
    ) -> Vec<Hypothesis> {
        let mut new_hypotheses = Vec::new();

        for (cause, effect, strength) in temporal_patterns {
            if !strength.is_finite() || *strength < self.config.correlation_threshold {
                continue;
            }

            let hypothesis = Hypothesis {
                id: format!("H{}", self.observation_count),
                statement: format!("{cause} causally influences {effect}"),
                priority: self.priority_from_correlation(*strength),
                status: HypothesisStatus::Generated,
                variables: vec![cause.clone(), effect.clone()],
                expected_relationship: RelationshipType::Causal,
                supporting_evidence: vec![],
                rejecting_evidence: vec![],
                confidence: *strength,
                generated_at: tick,
            };
            self.observation_count += 1;

            new_hypotheses.push(hypothesis);
        }

        self.hypotheses.extend(new_hypotheses.clone());

        if self.hypotheses.len() > self.config.max_hypotheses {
            self.hypotheses.sort_by(|a, b| {
                b.priority
                    .cmp(&a.priority)
                    .then_with(|| b.confidence.total_cmp(&a.confidence))
                    .then_with(|| a.id.cmp(&b.id))
            });
            self.hypotheses.truncate(self.config.max_hypotheses);
        }

        new_hypotheses
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Add evidence to a hypothesis
    pub fn add_evidence(
        &mut self,
        hypothesis_id: &str,
        evidence: Evidence,
        supports: bool,
    ) -> Option<()> {
        if !evidence.strength.is_finite() {
            return None;
        }
        let min_obs = self.config.min_observations;
        let auto_promote = self.config.auto_promote;
        let hypothesis = self.hypotheses.iter_mut().find(|h| h.id == hypothesis_id)?;

        if supports {
            hypothesis.supporting_evidence.push(evidence);
        } else {
            hypothesis.rejecting_evidence.push(evidence);
        }

        let total = hypothesis.supporting_evidence.len() + hypothesis.rejecting_evidence.len();
        if total > 0 {
            let support_weight: f32 = hypothesis
                .supporting_evidence
                .iter()
                .map(|e| e.strength.max(0.0))
                .sum();
            let reject_weight: f32 = hypothesis
                .rejecting_evidence
                .iter()
                .map(|e| e.strength.max(0.0))
                .sum();
            let total_weight = support_weight + reject_weight;
            let support_score = if total_weight > 1e-6 {
                support_weight / total_weight
            } else {
                hypothesis.supporting_evidence.len() as f32 / total as f32
            };
            hypothesis.confidence = support_score;

            if auto_promote {
                hypothesis.status = if total < min_obs {
                    HypothesisStatus::UnderTest
                } else if hypothesis.confidence > 0.8 {
                    HypothesisStatus::Supported
                } else if hypothesis.confidence < 0.2 {
                    HypothesisStatus::Rejected
                } else {
                    HypothesisStatus::Inconclusive
                };
            }
        }

        Some(())
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get high priority hypotheses
    #[must_use]
    pub fn high_priority(&self) -> Vec<&Hypothesis> {
        self.hypotheses
            .iter()
            .filter(|h| {
                matches!(
                    h.priority,
                    HypothesisPriority::High | HypothesisPriority::Critical
                )
            })
            .collect()
    }

    /// Get supported hypotheses ready for publication
    #[must_use]
    pub fn publication_ready(&self) -> Vec<&Hypothesis> {
        self.hypotheses
            .iter()
            .filter(|h| h.status == HypothesisStatus::Supported && h.confidence > 0.85)
            .collect()
    }

    fn priority_from_correlation(&self, corr: f32) -> HypothesisPriority {
        if !corr.is_finite() {
            return HypothesisPriority::Low;
        }
        if corr > 0.8 {
            HypothesisPriority::Critical
        } else if corr > 0.6 {
            HypothesisPriority::High
        } else if corr > 0.4 {
            HypothesisPriority::Medium
        } else {
            HypothesisPriority::Low
        }
    }
}

impl Default for HypothesisGenerator {
    fn default() -> Self {
        Self::new(HypothesisConfig::default())
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Hypothesis test result
#[derive(Debug, Clone)]
pub struct HypothesisTestResult {
    /// hypothesis id.
    pub hypothesis_id: String,
    /// p value.
    pub p_value: f32,
    /// effect size.
    pub effect_size: f32,
    /// sample size.
    pub sample_size: usize,
    /// conclusion.
    pub conclusion: TestConclusion,
}

/// Test conclusion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestConclusion {
    /// support.
    Support,
    /// reject.
    Reject,
    /// inconclusive.
    Inconclusive,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    fn test_generate_from_correlations() {
        let mut generator = HypothesisGenerator::default();

        let correlations = vec![
            ("trust".to_string(), "cooperation".to_string(), 0.75),
            ("diversity".to_string(), "performance".to_string(), -0.4),
        ];

        let hypotheses = generator.generate_from_correlations(&correlations, 0);

        assert_eq!(hypotheses.len(), 2);
        assert!(hypotheses
            .iter()
            .any(|h| h.variables.contains(&"trust".to_string())));
    }

    #[test]
    fn test_nan_correlation_rejected() {
        let mut generator = HypothesisGenerator::default();
        let correlations = vec![
            ("var_nan".to_string(), "var_b".to_string(), f32::NAN),
            ("var_inf".to_string(), "var_b".to_string(), f32::INFINITY),
        ];
        let hypotheses = generator.generate_from_correlations(&correlations, 0);
        assert!(hypotheses.is_empty());
    }

    #[test]
    fn test_add_evidence() {
        let mut generator = HypothesisGenerator::default();

        let correlations = vec![("A".to_string(), "B".to_string(), 0.8)];
        generator.generate_from_correlations(&correlations, 0);

        generator.add_evidence(
            "H0",
            Evidence {
                source: "test".to_string(),
                observation: "positive correlation observed".to_string(),
                strength: 0.9,
                timestamp: 1,
            },
            true,
        );

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let h = &generator.hypotheses[0];
        assert_eq!(h.supporting_evidence.len(), 1);
    }

    #[test]
    fn test_publication_ready() {
        let mut generator = HypothesisGenerator::default();

        let correlations = vec![("A".to_string(), "B".to_string(), 0.9)];
        generator.generate_from_correlations(&correlations, 0);

        for i in 0..5 {
            generator.add_evidence(
                "H0",
                Evidence {
                    source: "exp".to_string(),
                    observation: format!("obs{i}"),
                    strength: 0.9,
                    timestamp: i as u64,
                },
                true,
            );
        }

        let ready = generator.publication_ready();
        assert!(!ready.is_empty());
    }
}
