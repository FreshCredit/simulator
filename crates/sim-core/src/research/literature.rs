//! Literature Review Automation (P2-RES-3)
//!
//! Automated literature review and synthesis.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Literature entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteratureEntry {
    /// id.
    pub id: String,
    /// title.
    pub title: String,
    /// authors.
    pub authors: Vec<String>,
    /// year.
    pub year: u16,
    /// journal.
    pub journal: Option<String>,
    /// doi.
    pub doi: Option<String>,
    /// abstract text.
    pub abstract_text: String,
    /// keywords.
    pub keywords: Vec<String>,
    /// citations.
    pub citations: Vec<String>,
    /// cited by.
    pub cited_by: Vec<String>,
    /// relevance score.
    pub relevance_score: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Literature review configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteratureConfig {
    /// relevance threshold.
    pub relevance_threshold: f32,
    /// max papers.
    pub max_papers: usize,
    /// min citations.
    pub min_citations: usize,
    /// recency weight.
    pub recency_weight: f32,
}

impl Default for LiteratureConfig {
    fn default() -> Self {
        Self {
            relevance_threshold: 0.5,
            max_papers: 100,
            min_citations: 5,
            recency_weight: 0.3,
        }
    }
}

/// Literature review engine
#[derive(Debug, Clone)]
pub struct LiteratureReview {
    /// config.
    pub config: LiteratureConfig,
    /// entries.
    pub entries: Vec<LiteratureEntry>,
    /// synthesis.
    pub synthesis: Option<Synthesis>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl LiteratureReview {
    #[must_use]
    /// new.
    pub const fn new(config: LiteratureConfig) -> Self {
        Self {
            config,
            entries: Vec::new(),
            synthesis: None,
        }
    }

    /// Add literature entry
    pub fn add_entry(&mut self, entry: LiteratureEntry) {
        self.entries.push(entry);
    }

    /// Search by keyword
    #[must_use]
    pub fn search(&self, query: &str) -> Vec<&LiteratureEntry> {
        let query_lower = query.to_lowercase();
        self.entries
            .iter()
            .filter(|e| {
                e.title.to_lowercase().contains(&query_lower)
                    || e.keywords
                        .iter()
                        .any(|k| k.to_lowercase().contains(&query_lower))
                    || e.abstract_text.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Rank by relevance to research topic
    pub fn rank_by_relevance(&mut self, topic_keywords: &[String]) {
        let topic_set: HashSet<String> = topic_keywords.iter().map(|k| k.to_lowercase()).collect();

        // Calculate recency scores first (doesn't need mutable borrow)
        let recency_scores: Vec<f32> = self
            .entries
            .iter()
            .map(|e| Self::calculate_recency(e.year))
            .collect();

        for (i, entry) in self.entries.iter_mut().enumerate() {
            let keyword_matches = entry
                .keywords
                .iter()
                .filter(|k| topic_set.contains(&k.to_lowercase()))
                .count();

            let abstract_matches = topic_set
                .iter()
                .filter(|t| entry.abstract_text.to_lowercase().contains(*t))
                .count();

            let citation_score = (entry.cited_by.len() as f32 / 100.0).min(1.0);

            entry.relevance_score = (keyword_matches as f32 * 0.4
                + abstract_matches as f32 * 0.3
                + citation_score * 0.2
                + recency_scores[i] * self.config.recency_weight)
                .min(1.0);
        }

        self.entries.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    fn calculate_recency(year: u16) -> f32 {
        let current_year = 2024;
        let age = (current_year - year) as f32;
        (-age / 10.0).exp()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Find seminal papers (highly cited, foundational)
    #[must_use]
    pub fn find_seminal_papers(&self, n: usize) -> Vec<&LiteratureEntry> {
        let mut candidates: Vec<&LiteratureEntry> = self
            .entries
            .iter()
            .filter(|e| e.cited_by.len() >= self.config.min_citations)
            .collect();

        candidates.sort_by(|a, b| b.cited_by.len().cmp(&a.cited_by.len()));
        candidates.into_iter().take(n).collect()
    }

    /// Generate synthesis
    pub fn generate_synthesis(&mut self, topic: &str) -> Synthesis {
        let relevant: Vec<&LiteratureEntry> = self
            .entries
            .iter()
            .filter(|e| e.relevance_score >= self.config.relevance_threshold)
            .collect();

        let mut themes: HashMap<String, Vec<String>> = HashMap::new();
        let mut method_counts: HashMap<String, usize> = HashMap::new();

        for entry in &relevant {
            for keyword in &entry.keywords {
                themes
                    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                    .entry(keyword.clone())
                    .or_default()
                    .push(entry.id.clone());
            }

            if entry.abstract_text.contains("experiment") {
                *method_counts.entry("experimental".to_string()).or_default() += 1;
            }
            if entry.abstract_text.contains("simulation") {
                *method_counts.entry("simulation".to_string()).or_default() += 1;
            }
            if entry.abstract_text.contains("theoretical") {
                *method_counts.entry("theoretical".to_string()).or_default() += 1;
            }
        }

        let mut major_themes: Vec<Theme> = themes
            .into_iter()
            .filter(|(_, papers)| papers.len() >= 3)
            .map(|(name, papers)| Theme {
                name,
                paper_count: papers.len(),
                representative_papers: papers.into_iter().take(5).collect(),
            })
            .collect();

        major_themes.sort_by(|a, b| b.paper_count.cmp(&a.paper_count));

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let synthesis = Synthesis {
            topic: topic.to_string(),
            papers_reviewed: relevant.len(),
            major_themes: major_themes.into_iter().take(10).collect(),
            dominant_methods: method_counts.into_iter().collect(),
            gaps_identified: self.identify_gaps(&relevant),
            consensus_areas: vec![],
        };

        self.synthesis = Some(synthesis.clone());
        synthesis
    }

    fn identify_gaps(&self, relevant: &[&LiteratureEntry]) -> Vec<String> {
        let mut gaps = Vec::new();

        let recent_count = relevant.iter().filter(|e| e.year >= 2020).count();
        if recent_count < relevant.len() / 4 {
            gaps.push("Limited recent research".to_string());
        }

        let has_experimental = relevant
            .iter()
            .any(|e| e.abstract_text.contains("experiment"));
        let has_simulation = relevant
            .iter()
            .any(|e| e.abstract_text.contains("simulation"));

        if has_experimental && !has_simulation {
            gaps.push("Limited simulation studies".to_string());
        }
        if has_simulation && !has_experimental {
            gaps.push("Limited experimental validation".to_string());
        }

        gaps
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Citation network analysis
    #[must_use]
    pub fn citation_network(&self) -> CitationNetwork {
        let mut nodes: HashSet<String> = HashSet::new();
        let mut edges: Vec<(String, String)> = Vec::new();

        for entry in &self.entries {
            nodes.insert(entry.id.clone());
            for cited in &entry.citations {
                edges.push((entry.id.clone(), cited.clone()));
                nodes.insert(cited.clone());
            }
        }

        CitationNetwork {
            nodes: nodes.into_iter().collect(),
            edges,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Find influential papers (high betweenness in citation network)
    #[must_use]
    pub fn influential_papers(&self, n: usize) -> Vec<&LiteratureEntry> {
        let mut influence_scores: HashMap<String, f32> = HashMap::new();

        for entry in &self.entries {
            let in_degree = entry.cited_by.len() as f32;
            let out_degree = entry.citations.len() as f32;
            let score = in_degree + out_degree * 0.5;
            influence_scores.insert(entry.id.clone(), score);
        }

        let mut entries: Vec<&LiteratureEntry> = self.entries.iter().collect();
        entries.sort_by(|a, b| {
            let score_a = influence_scores.get(&a.id).unwrap_or(&0.0);
            let score_b = influence_scores.get(&b.id).unwrap_or(&0.0);
            score_b
                .partial_cmp(score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        entries.into_iter().take(n).collect()
    }
}

impl Default for LiteratureReview {
    fn default() -> Self {
        Self::new(LiteratureConfig::default())
    }
}

/// Research theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// name.
    pub name: String,
    /// paper count.
    pub paper_count: usize,
    /// representative papers.
    pub representative_papers: Vec<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Literature synthesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synthesis {
    /// topic.
    pub topic: String,
    /// papers reviewed.
    pub papers_reviewed: usize,
    /// major themes.
    pub major_themes: Vec<Theme>,
    /// dominant methods.
    pub dominant_methods: Vec<(String, usize)>,
    /// gaps identified.
    pub gaps_identified: Vec<String>,
    /// consensus areas.
    pub consensus_areas: Vec<(String, Vec<f32>)>,
}

/// Citation network
#[derive(Debug, Clone)]
pub struct CitationNetwork {
    /// nodes.
    pub nodes: Vec<String>,
    /// edges.
    pub edges: Vec<(String, String)>,
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entry(id: &str, year: u16, citations: usize) -> LiteratureEntry {
        LiteratureEntry {
            id: id.to_string(),
            title: format!("Paper {id}"),
            authors: vec!["Author".to_string()],
            year,
            journal: Some("Journal".to_string()),
            doi: None,
            abstract_text: "Test abstract about simulation and experiments".to_string(),
            keywords: vec!["trust".to_string(), "cooperation".to_string()],
            citations: vec![],
            cited_by: (0..citations).map(|i| format!("C{i}")).collect(),
            relevance_score: 0.0,
        }
    }

    #[test]
    fn test_add_and_search() {
        let mut review = LiteratureReview::default();
        review.add_entry(create_test_entry("A", 2020, 10));

        let results = review.search("trust");
        assert_eq!(results.len(), 1);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_rank_by_relevance() {
        let mut review = LiteratureReview::default();
        review.add_entry(create_test_entry("A", 2020, 50));
        review.add_entry(create_test_entry("B", 2010, 10));

        review.rank_by_relevance(&["trust".to_string()]);

        assert!(review.entries[0].relevance_score > review.entries[1].relevance_score);
    }

    #[test]
    fn test_find_seminal() {
        let mut review = LiteratureReview::default();
        review.add_entry(create_test_entry("A", 2020, 100));
        review.add_entry(create_test_entry("B", 2020, 5));

        let seminal = review.find_seminal_papers(1);
        assert_eq!(seminal.len(), 1);
        assert_eq!(seminal[0].id, "A");
    }

    #[test]
    fn test_generate_synthesis() {
        let mut review = LiteratureReview::default();

        for i in 0..5 {
            let mut entry = create_test_entry(&format!("P{i}"), 2020, 10);
            entry.relevance_score = 0.8;
            review.add_entry(entry);
        }

        let synthesis = review.generate_synthesis("Trust");
        assert!(synthesis.papers_reviewed > 0);
    }
}
