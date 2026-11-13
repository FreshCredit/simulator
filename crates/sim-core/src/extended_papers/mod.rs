//! Extended Research Papers (P2-RES-1)
//!
//! Additional research papers beyond the original 12 foundational papers.
//! This module provides implementations for cutting-edge research in social simulation.

pub mod collective_intelligence;
pub mod diffusion;
pub mod institutional_analysis;
pub mod market_simulation;
pub mod opinion_dynamics;
pub mod reputation_systems;
pub mod signaling;

pub use collective_intelligence::{CollectiveIntelligence, GroupPerformance, WisdomOfCrowdsConfig};
pub use diffusion::{DiffusionNetwork, Innovation};
pub use institutional_analysis::{Institution, Norm, SocialDilemma};
pub use market_simulation::{CascadeModel, MarketMaker, OpinionMarket};
pub use opinion_dynamics::{BoundedConfidence, OpinionDynamicsModel};
pub use reputation_systems::{Rating, Reputation, ReputationSystem};
pub use signaling::{Signal, SignalType, SignalingGame};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Registry for all extended paper implementations
#[derive(Debug, Clone, Default)]
pub struct ExtendedPaperRegistry {
    papers: Vec<ExtendedPaper>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Metadata for an extended paper
#[derive(Debug, Clone)]
pub struct ExtendedPaper {
    /// id.
    pub id: String,
    /// title.
    pub title: String,
    /// authors.
    pub authors: Vec<String>,
    /// year.
    pub year: u16,
    /// domain.
    pub domain: ResearchDomain,
    /// status.
    pub status: ImplementationStatus,
}

/// Research domains
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResearchDomain {
    /// network dynamics.
    NetworkDynamics,
    /// opinion formation.
    OpinionFormation,
    /// cooperation.
    Cooperation,
    /// social learning.
    SocialLearning,
    /// markets.
    Markets,
    /// norms.
    Norms,
    /// reputation.
    Reputation,
    /// signaling.
    Signaling,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Implementation status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImplementationStatus {
    /// complete.
    Complete,
    /// partial.
    Partial,
    /// planned.
    Planned,
}

impl ExtendedPaperRegistry {
    #[must_use]
    /// new.
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.register_default_papers();
        registry
    }

    fn register_default_papers(&mut self) {
        let papers = vec![
            ExtendedPaper {
                id: "collective_intelligence".to_string(),
                title: "Evidence for a Collective Intelligence Factor".to_string(),
                authors: vec!["Woolley et al.".to_string()],
                year: 2010,
                domain: ResearchDomain::SocialLearning,
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                status: ImplementationStatus::Complete,
            },
            ExtendedPaper {
                id: "diffusion".to_string(),
                title: "Complex Contagions and the Weakness of Long Ties".to_string(),
                authors: vec!["Centola, Macy".to_string()],
                year: 2007,
                domain: ResearchDomain::NetworkDynamics,
                status: ImplementationStatus::Complete,
            },
            ExtendedPaper {
                id: "institutional_analysis".to_string(),
                title: "Understanding Institutional Diversity".to_string(),
                authors: vec!["Ostrom".to_string()],
                year: 2005,
                domain: ResearchDomain::Norms,
                status: ImplementationStatus::Complete,
            },
            ExtendedPaper {
                id: "market_simulation".to_string(),
                title: "A Theory of Fads, Fashion, Custom".to_string(),
                authors: vec!["Banerjee".to_string()],
                year: 1992,
                domain: ResearchDomain::Markets,
                status: ImplementationStatus::Complete,
            },
            ExtendedPaper {
                id: "opinion_dynamics".to_string(),
                title: "Consensus and Contact in Opinion Dynamics".to_string(),
                // TAG: surface=simulation owner=research-team rule=SIM-001
                authors: vec!["Hegselmann, Krause".to_string()],
                year: 2002,
                domain: ResearchDomain::OpinionFormation,
                status: ImplementationStatus::Complete,
            },
            ExtendedPaper {
                id: "reputation_systems".to_string(),
                title: "The Reputation Society".to_string(),
                authors: vec!["Resnick et al.".to_string()],
                year: 2000,
                domain: ResearchDomain::Reputation,
                status: ImplementationStatus::Complete,
            },
            ExtendedPaper {
                id: "signaling".to_string(),
                title: "Mate Selection: A Selection for a Handicap".to_string(),
                authors: vec!["Zahavi".to_string()],
                year: 1975,
                domain: ResearchDomain::Signaling,
                status: ImplementationStatus::Complete,
            },
        ];

        self.papers = papers;
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get all papers
    #[must_use]
    pub fn all(&self) -> &[ExtendedPaper] {
        &self.papers
    }

    /// Get papers by domain
    #[must_use]
    pub fn by_domain(&self, domain: ResearchDomain) -> Vec<&ExtendedPaper> {
        self.papers.iter().filter(|p| p.domain == domain).collect()
    }

    /// Get paper by ID
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&ExtendedPaper> {
        self.papers.iter().find(|p| p.id == id)
    }

    /// Count papers
    #[must_use]
    pub fn count(&self) -> usize {
        self.papers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ExtendedPaperRegistry::new();
        assert_eq!(registry.count(), 7);
    }

    #[test]
    fn test_filter_by_domain() {
        let registry = ExtendedPaperRegistry::new();
        let networks = registry.by_domain(ResearchDomain::NetworkDynamics);
        assert!(!networks.is_empty());
    }

    #[test]
    fn test_get_paper() {
        let registry = ExtendedPaperRegistry::new();
        let paper = registry.get("diffusion");
        assert!(paper.is_some());
        assert_eq!(paper.unwrap().year, 2007);
    }
}
