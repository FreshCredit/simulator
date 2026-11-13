//! Research Tools (P2-RES)
//!
//! Advanced research capabilities for the simulation framework.

pub mod bayesian;
pub mod causal;
pub mod hypothesis;
pub mod literature;

// TAG: surface=simulation owner=research-team rule=SIM-001
pub use bayesian::{
    BayesianModel, BayesianValidator, LikelihoodFunction, McmcConfig, ModelComparison, ModelFit,
    Parameter, PosteriorApproximation, PpcResult, PriorDistribution,
};
pub use causal::{
    CausalConfig, CausalEdge, CausalEffect, CausalGraph, CausalInference, CausalRelationship,
    EdgeType, GrangerResult,
};
pub use hypothesis::{
    Evidence, Hypothesis, HypothesisConfig, HypothesisGenerator, HypothesisPriority,
    HypothesisStatus, HypothesisTestResult, RelationshipType, TestConclusion,
};
pub use literature::{
    CitationNetwork, LiteratureConfig, LiteratureEntry, LiteratureReview, Synthesis, Theme,
};

use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Research module configuration
///
/// Aggregates configuration for all research subsystems:
/// hypothesis generation, literature review, causal inference,
/// and Bayesian validation.
#[derive(Debug, Clone, Default)]
pub struct ResearchConfig {
    /// hypothesis.
    pub hypothesis: HypothesisConfig,
    /// literature.
    pub literature: LiteratureConfig,
    /// causal.
    pub causal: CausalConfig,
    /// mcmc.
    pub mcmc: McmcConfig,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Unified research engine
///
/// Orchestrates all research capabilities including hypothesis
/// generation, literature review, causal inference, and Bayesian
/// model validation.
#[derive(Debug, Clone)]
pub struct ResearchEngine {
    /// config.
    pub config: ResearchConfig,
    /// hypothesis gen.
    pub hypothesis_gen: HypothesisGenerator,
    /// literature.
    pub literature: LiteratureReview,
    /// causal.
    pub causal: CausalInference,
    /// bayesian.
    pub bayesian: BayesianValidator,
}

impl ResearchEngine {
    #[must_use]
    /// new.
    pub fn new(config: ResearchConfig) -> Self {
        Self {
            hypothesis_gen: HypothesisGenerator::new(config.hypothesis.clone()),
            literature: LiteratureReview::new(config.literature.clone()),
            causal: CausalInference::new(config.causal.clone()),
            bayesian: BayesianValidator::new(config.mcmc.clone()),
            config,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate research report
    #[must_use]
    pub fn generate_report(&self) -> ResearchReport {
        ResearchReport {
            active_hypotheses: self.hypothesis_gen.hypotheses.len(),
            supported_hypotheses: self.hypothesis_gen.publication_ready().len(),
            literature_entries: self.literature.entries.len(),
            causal_relationships: self.causal.graph.edges.len(),
        }
    }
}

impl Default for ResearchEngine {
    fn default() -> Self {
        Self::new(ResearchConfig::default())
    }
}

/// Research report summary
///
/// Provides aggregate statistics about ongoing research activities
/// including active hypotheses, supported findings, and discovered
/// causal relationships.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchReport {
    /// active hypotheses.
    pub active_hypotheses: usize,
    /// supported hypotheses.
    pub supported_hypotheses: usize,
    /// literature entries.
    pub literature_entries: usize,
    /// causal relationships.
    pub causal_relationships: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_research_engine_creation() {
        let engine = ResearchEngine::default();
        let report = engine.generate_report();
        assert_eq!(report.active_hypotheses, 0);
    }
}
