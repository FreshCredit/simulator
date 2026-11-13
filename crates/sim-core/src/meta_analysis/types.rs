//! Meta-analysis types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Results from a single paper simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperResults {
    /// paper id.
    pub paper_id: String,
    /// paper name.
    pub paper_name: String,
    /// metrics.
    pub metrics: HashMap<String, f64>,
    /// effect sizes.
    pub effect_sizes: HashMap<String, f64>,
    /// p values.
    pub p_values: HashMap<String, f64>,
    /// sample size.
    pub sample_size: usize,
    /// confidence level.
    pub confidence_level: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Cross-paper comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPaperComparison {
    /// paper a.
    pub paper_a: String,
    /// paper b.
    pub paper_b: String,
    /// comparable metrics.
    pub comparable_metrics: Vec<String>,
    /// correlation.
    pub correlation: f64,
    /// agreement score.
    pub agreement_score: f64,
}

/// Publication readiness assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicationReadiness {
    /// paper id.
    pub paper_id: String,
    /// replication score.
    pub replication_score: f64,
    /// novelty score.
    pub novelty_score: f64,
    /// rigor score.
    pub rigor_score: f64,
    /// overall readiness.
    pub overall_readiness: f64,
    /// recommendations.
    pub recommendations: Vec<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Cross-cutting theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossCuttingTheme {
    /// theme name.
    pub theme_name: String,
    /// description.
    pub description: String,
    /// papers.
    pub papers: Vec<String>,
    /// strength.
    pub strength: f64,
}

/// Research gap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchGap {
    /// gap description.
    pub gap_description: String,
    /// affected papers.
    pub affected_papers: Vec<String>,
    /// severity.
    pub severity: GapSeverity,
    /// suggested research.
    pub suggested_research: String,
}

/// Gap severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GapSeverity {
    /// low.
    Low,
    /// medium.
    Medium,
    /// high.
    High,
    /// critical.
    Critical,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Reproducibility status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReproducibilityStatus {
    /// paper id.
    pub paper_id: String,
    /// reproducibility score.
    pub reproducibility_score: f64,
    /// available data.
    pub available_data: bool,
    /// available code.
    pub available_code: bool,
    /// documentation quality.
    pub documentation_quality: f64,
    /// third party replications.
    pub third_party_replications: usize,
}

/// Meta-analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaAnalysisConfig {
    /// confidence threshold.
    pub confidence_threshold: f64,
    /// effect size threshold.
    pub effect_size_threshold: f64,
    /// min sample size.
    pub min_sample_size: usize,
    /// require replication.
    pub require_replication: bool,
}

impl Default for MetaAnalysisConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.95,
            effect_size_threshold: 0.2,
            min_sample_size: 30,
            require_replication: true,
        }
    }
}
