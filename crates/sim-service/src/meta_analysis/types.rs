//! Cross-Paper Meta-Analysis Types (P1-SIM-5)
//!
//! Types for aggregating results across all 12 research papers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// The 12 research papers in the Agent Society Simulator
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResearchPaper {
    /// Generalized Trust Theory (GTT)
    Gtt,
    /// DOCAS - Neurochemical model
    Docas,
    /// Trust Fusion Phenomenology (TFP)
    Tfp,
    /// Emotion dynamics
    Emotion,
    /// Cultural evolution
    Culture,
    /// Economic prediction
    Economics,
    /// Linguistics
    Linguistics,
    /// Philosophy
    Philosophy,
    /// Game theory experiments
    GameTheory,
    /// PDE4B genetics
    Pde4b,
    /// Meaning formation
    Meaning,
    /// Music perception
    Music,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl ResearchPaper {
    #[must_use]
    /// all
    pub fn all() -> Vec<Self> {
        vec![
            Self::Gtt,
            Self::Docas,
            Self::Tfp,
            Self::Emotion,
            Self::Culture,
            Self::Economics,
            Self::Linguistics,
            Self::Philosophy,
            Self::GameTheory,
            Self::Pde4b,
            Self::Meaning,
            Self::Music,
        ]
    }

    #[must_use]
    /// fn
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Gtt => "Generalized Trust Theory",
            Self::Docas => "DOCAS",
            Self::Tfp => "Trust Fusion Phenomenology",
            Self::Emotion => "Emotion Dynamics",
            Self::Culture => "Cultural Evolution",
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            Self::Economics => "Economic Prediction",
            Self::Linguistics => "Linguistics",
            Self::Philosophy => "Philosophy",
            Self::GameTheory => "Game Theory",
            Self::Pde4b => "PDE4B Genetics",
            Self::Meaning => "Meaning Formation",
            Self::Music => "Music Perception",
        }
    }

    #[must_use]
    /// fn
    pub const fn short_code(&self) -> &'static str {
        match self {
            Self::Gtt => "GTT",
            Self::Docas => "DOCAS",
            Self::Tfp => "TFP",
            Self::Emotion => "EMOT",
            Self::Culture => "CULT",
            Self::Economics => "ECON",
            Self::Linguistics => "LING",
            Self::Philosophy => "PHIL",
            Self::GameTheory => "GAME",
            Self::Pde4b => "PDE4B",
            Self::Meaning => "MEAN",
            Self::Music => "MUSIC",
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Aggregated results from all papers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPaperAnalysis {
    /// Results per research paper
    pub papers: HashMap<ResearchPaper, PaperResults>,
    /// Themes that cut across multiple papers
    pub cross_cutting_themes: Vec<CrossCuttingTheme>,
    /// Overall synthesis report
    pub overall_synthesis: SynthesisReport,
    /// Timestamp when the analysis was generated
    pub generated_at: String,
}

/// Results from a single paper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperResults {
    /// The research paper
    pub paper: ResearchPaper,
    /// Experiment summaries
    pub experiments: Vec<ExperimentSummary>,
    /// Key findings from the paper
    pub key_findings: Vec<KeyFinding>,
    /// Effect size metrics
    pub effect_sizes: Vec<EffectSizeMetric>,
    /// Validation status
    pub validation_status: ValidationStatus,
    /// Coverage percentage (0-100)
    pub coverage_percentage: f32,
}

/// Summary of an experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentSummary {
    /// Experiment identifier
    pub id: String,
    /// Experiment name
    pub name: String,
    /// Current status
    pub status: ExperimentStatus,
    /// Sample size
    pub sample_size: usize,
    /// Key metrics
    pub key_metrics: HashMap<String, f64>,
    /// Primary outcome value
    pub primary_outcome: f64,
    /// Confidence interval
    pub confidence_interval: (f64, f64),
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Experiment status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExperimentStatus {
    /// Experiment completed successfully
    Completed,
    /// Experiment is currently running
    Running,
    /// Experiment is pending execution
    Pending,
    /// Experiment failed
    Failed,
}

/// Key finding from a paper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyFinding {
    /// The claim being made
    pub claim: String,
    /// Strength of supporting evidence
    pub evidence_strength: EvidenceStrength,
    /// Replication status
    pub replication_status: ReplicationStatus,
    /// IDs of supporting experiments
    pub supporting_experiments: Vec<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Evidence strength
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStrength {
    /// Strong evidence
    Strong,
    /// Moderate evidence
    Moderate,
    /// Weak evidence
    Weak,
    /// Inconclusive evidence
    Inconclusive,
}

/// Replication status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplicationStatus {
    /// Successfully replicated
    Replicated,
    /// Partially replicated
    PartiallyReplicated,
    /// Not replicated
    NotReplicated,
    /// Replication pending
    PendingReplication,
}

/// Effect size metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectSizeMetric {
    /// Name of the metric
    pub metric_name: String,
    /// Effect size value
    pub value: f64,
    /// Confidence interval
    pub confidence_interval: (f64, f64),
    /// Human-readable interpretation
    pub interpretation: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Validation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStatus {
    /// Whether the study was pre-registered
    pub pre_registered: bool,
    /// Link to OSF repository
    pub osf_link: Option<String>,
    /// Number of replication attempts
    pub replication_attempts: usize,
    /// Number of successful replications
    pub successful_replications: usize,
}

/// Cross-cutting theme across papers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossCuttingTheme {
    /// Theme name
    pub theme: String,
    /// Theme description
    pub description: String,
    /// Papers affected by this theme
    pub affected_papers: Vec<ResearchPaper>,
    /// Consistency score (0-1)
    pub consistency_score: f32,
    /// Supporting evidence references
    pub supporting_evidence: Vec<String>,
}

/// Synthesis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisReport {
    /// Total number of experiments
    pub total_experiments: usize,
    /// Number of completed experiments
    pub completed_experiments: usize,
    /// Overall effect size
    pub overall_effect_size: f64,
    /// Heterogeneity metrics
    pub heterogeneity: HeterogeneityMetrics,
    /// Identified research gaps
    pub research_gaps: Vec<ResearchGap>,
    /// Recommendations for future work
    pub recommendations: Vec<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Heterogeneity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeterogeneityMetrics {
    /// I-squared statistic
    pub i_squared: f64,
    /// Tau-squared statistic
    pub tau_squared: f64,
    /// Q statistic
    pub q_statistic: f64,
    /// p-value
    pub p_value: f64,
}

/// Research gap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchGap {
    /// Research area
    pub area: String,
    /// Gap description
    pub description: String,
    /// Priority level
    pub priority: GapPriority,
    /// Affected papers
    pub affected_papers: Vec<ResearchPaper>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Gap priority
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GapPriority {
    /// Critical priority
    Critical,
    /// High priority
    High,
    /// Medium priority
    Medium,
    /// Low priority
    Low,
}

/// Meta-analysis request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaAnalysisRequest {
    /// papers
    pub papers: Option<Vec<ResearchPaper>>, // None = all papers
    /// include experiments
    pub include_experiments: bool,
    /// include themes
    pub include_themes: bool,
    /// include gaps
    pub include_gaps: bool,
}

/// Comparison request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperComparisonRequest {
    /// paper1
    pub paper1: ResearchPaper,
    /// paper2
    pub paper2: ResearchPaper,
    /// metrics
    pub metrics: Vec<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperComparison {
    /// paper1
    pub paper1: ResearchPaper,
    /// paper2
    pub paper2: ResearchPaper,
    /// correlations
    pub correlations: Vec<MetricCorrelation>,
    /// common themes
    pub common_themes: Vec<String>,
    /// divergent findings
    pub divergent_findings: Vec<String>,
}

/// Metric correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricCorrelation {
    /// metric
    pub metric: String,
    /// paper1 value
    pub paper1_value: f64,
    /// paper2 value
    pub paper2_value: f64,
    /// difference
    pub difference: f64,
    /// agreement
    pub agreement: bool,
}

/// Trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// metric
    pub metric: String,
    /// time periods
    pub time_periods: Vec<String>,
    /// values
    pub values: Vec<f64>,
    /// trend direction
    pub trend_direction: TrendDirection,
    /// slope
    pub slope: f64,
    /// r squared
    pub r_squared: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Trend direction
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendDirection {
    /// Increasing
    Increasing,
    /// Decreasing
    Decreasing,
    /// Stable
    Stable,
    /// Oscillating
    Oscillating,
}

/// Publication readiness report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicationReadiness {
    /// paper
    pub paper: ResearchPaper,
    /// overall score
    pub overall_score: f32, // 0-100
    /// criteria
    pub criteria: Vec<ReadinessCriterion>,
    /// blockers
    pub blockers: Vec<String>,
    /// recommendations
    pub recommendations: Vec<String>,
}

/// Readiness criterion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessCriterion {
    /// name
    pub name: String,
    /// satisfied
    pub satisfied: bool,
    /// score
    pub score: f32, // 0-100
    /// notes
    pub notes: String,
}
