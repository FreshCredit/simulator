//! Meta-analysis aggregator

use super::types::*;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Aggregates results across papers
#[derive(Debug, Clone)]
pub struct MetaAggregator {
    /// config.
    pub config: MetaAnalysisConfig,
    /// papers.
    pub papers: Vec<PaperResults>,
}

impl MetaAggregator {
    #[must_use]
    /// new.
    pub const fn new(config: MetaAnalysisConfig) -> Self {
        Self {
            config,
            papers: Vec::new(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Add paper results
    pub fn add_paper(&mut self, results: PaperResults) {
        self.papers.push(results);
    }

    /// Aggregate effect sizes across papers
    #[must_use]
    pub fn aggregate_effect_sizes(&self, metric: &str) -> Option<MetaEffectSize> {
        let study_data: Vec<(f64, usize)> = self
            .papers
            .iter()
            .filter_map(|p| {
                p.effect_sizes
                    .get(metric)
                    .copied()
                    .map(|e| (e, p.sample_size))
            })
            .filter(|(e, _)| e.is_finite())
            .collect();

        if study_data.is_empty() {
            return None;
        }

        let k = study_data.len() as f64;
        let effects: Vec<f64> = study_data.iter().map(|(e, _)| *e).collect();
        let sample_sizes: Vec<usize> = study_data.iter().map(|(_, n)| *n).collect();

        // Sample size-based inverse-variance precision weights w_i = n_i / 4.0
        let weights: Vec<f64> = sample_sizes
            .iter()
            .map(|&n| if n > 0 { (n as f64) / 4.0 } else { 1.0 })
            .collect();
        let total_weight: f64 = weights.iter().sum();

        let mean = if total_weight > 0.0 {
            study_data
                .iter()
                .zip(&weights)
                .map(|((e, _), w)| e * w)
                .sum::<f64>()
                / total_weight
        } else {
            effects.iter().sum::<f64>() / k
        };

        let std_error = if total_weight > 0.0 {
            (1.0 / total_weight).sqrt()
        } else if k > 1.0 {
            let variance = effects.iter().map(|e| (e - mean).powi(2)).sum::<f64>() / (k - 1.0);
            (variance / k).sqrt()
        } else {
            0.0
        };

        Some(MetaEffectSize {
            metric: metric.to_string(),
            mean_effect: mean,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            std_error,
            num_studies: effects.len(),
            heterogeneity: self.calculate_heterogeneity(&effects, &sample_sizes),
        })
    }

    /// Calculate heterogeneity (I²)
    fn calculate_heterogeneity(&self, effects: &[f64], sample_sizes: &[usize]) -> f64 {
        if effects.len() < 2 {
            return 0.0;
        }

        let k = effects.len() as f64;
        let df = k - 1.0;

        let weights: Vec<f64> = sample_sizes
            .iter()
            .map(|&n| if n > 0 { (n as f64) / 4.0 } else { 1.0 })
            .collect();
        let sum_w: f64 = weights.iter().sum();

        let q = if sum_w > 0.0 {
            let weighted_mean = effects
                .iter()
                .zip(&weights)
                .map(|(e, w)| e * w)
                .sum::<f64>()
                / sum_w;
            effects
                .iter()
                .zip(&weights)
                .map(|(e, w)| w * (e - weighted_mean).powi(2))
                .sum::<f64>()
        } else {
            let mean = effects.iter().sum::<f64>() / k;
            effects.iter().map(|e| (e - mean).powi(2)).sum::<f64>()
        };

        if q <= df || !q.is_finite() || q <= 0.0 {
            0.0
        } else {
            ((q - df) / q).clamp(0.0, 1.0)
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compare two papers
    #[must_use]
    pub fn compare_papers(&self, paper_a: &str, paper_b: &str) -> Option<CrossPaperComparison> {
        let a = self.papers.iter().find(|p| p.paper_id == paper_a)?;
        let b = self.papers.iter().find(|p| p.paper_id == paper_b)?;

        let mut common_metrics: Vec<String> = a
            .metrics
            .keys()
            .filter(|k| b.metrics.contains_key(*k))
            .cloned()
            .collect();

        if common_metrics.is_empty() {
            return None;
        }

        common_metrics.sort(); // SIM-004: deterministic order across platforms

        // Calculate correlation across common metrics
        let a_values: Vec<f64> = common_metrics
            .iter()
            .filter_map(|m| a.metrics.get(m).copied())
            .collect();
        let b_values: Vec<f64> = common_metrics
            .iter()
            .filter_map(|m| b.metrics.get(m).copied())
            .collect();
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        let correlation = Self::pearson_correlation(&a_values, &b_values).unwrap_or(0.0);

        let sign_of = |x: f64| -> i32 {
            if x.abs() <= 1e-12 {
                0
            } else if x > 0.0 {
                1
            } else {
                -1
            }
        };

        let common_effects: Vec<&String> = a
            .effect_sizes
            .keys()
            .filter(|k| b.effect_sizes.contains_key(*k))
            .collect();

        let agreements = common_effects
            .iter()
            .filter(|k| {
                let va = a.effect_sizes.get(**k).copied().unwrap_or(f64::NAN);
                let vb = b.effect_sizes.get(**k).copied().unwrap_or(f64::NAN);
                va.is_finite() && vb.is_finite() && sign_of(va) == sign_of(vb)
            })
            .count();
        let total_effects = common_effects.len();
        let agreement_score = if total_effects > 0 {
            agreements as f64 / total_effects as f64
        } else {
            0.0
        };

        Some(CrossPaperComparison {
            paper_a: paper_a.to_string(),
            paper_b: paper_b.to_string(),
            comparable_metrics: common_metrics,
            correlation,
            agreement_score,
        })
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Pearson correlation
    fn pearson_correlation(x: &[f64], y: &[f64]) -> Option<f64> {
        crate::stats::pearson_r_f64(x, y)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Identify cross-cutting themes
    #[must_use]
    pub fn identify_themes(&self) -> Vec<CrossCuttingTheme> {
        let mut themes = Vec::new();

        // Theme: Trust emergence
        let trust_papers: Vec<String> = self
            .papers
            .iter()
            .filter(|p| p.metrics.contains_key("trust_emergence"))
            .map(|p| p.paper_name.clone())
            .collect();

        if trust_papers.len() >= 2 {
            themes.push(CrossCuttingTheme {
                theme_name: "Trust Emergence".to_string(),
                description: "How trust emerges in social systems".to_string(),
                papers: trust_papers,
                strength: 0.8,
            });
        }

        // Theme: Cooperation dynamics
        let coop_papers: Vec<String> = self
            .papers
            .iter()
            .filter(|p| p.metrics.contains_key("cooperation_rate"))
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
            .map(|p| p.paper_name.clone())
            .collect();

        if coop_papers.len() >= 2 {
            themes.push(CrossCuttingTheme {
                theme_name: "Cooperation Dynamics".to_string(),
                description: "Patterns of cooperation and defection".to_string(),
                papers: coop_papers,
                strength: 0.75,
            });
        }

        // Theme: Network effects
        let network_papers: Vec<String> = self
            .papers
            .iter()
            .filter(|p| p.metrics.contains_key("network_clustering"))
            .map(|p| p.paper_name.clone())
            .collect();

        if network_papers.len() >= 2 {
            themes.push(CrossCuttingTheme {
                theme_name: "Network Structure Effects".to_string(),
                description: "Impact of network topology on outcomes".to_string(),
                papers: network_papers,
                strength: 0.7,
            });
        }

        themes
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Identify research gaps
    #[must_use]
    pub fn identify_gaps(&self) -> Vec<ResearchGap> {
        let mut gaps = Vec::new();

        // Check for missing replications
        let unreplicated: Vec<String> = self
            .papers
            .iter()
            .filter(|p| p.p_values.len() < 2)
            .map(|p| p.paper_id.clone())
            .collect();

        if !unreplicated.is_empty() {
            gaps.push(ResearchGap {
                gap_description: "Limited replication studies".to_string(),
                affected_papers: unreplicated,
                severity: GapSeverity::Medium,
                suggested_research: "Conduct systematic replication experiments".to_string(),
            });
        }

        // Check for small sample sizes
        let small_sample: Vec<String> = self
            .papers
            .iter()
            .filter(|p| p.sample_size < self.config.min_sample_size)
            .map(|p| p.paper_id.clone())
            .collect();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        if !small_sample.is_empty() {
            gaps.push(ResearchGap {
                gap_description: "Insufficient statistical power".to_string(),
                affected_papers: small_sample,
                severity: GapSeverity::High,
                suggested_research: "Increase sample sizes for more reliable estimates".to_string(),
            });
        }

        gaps
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Assess publication readiness
    #[must_use]
    pub fn assess_publication_readiness(&self, paper_id: &str) -> Option<PublicationReadiness> {
        let paper = self.papers.iter().find(|p| p.paper_id == paper_id)?;

        let replication_score = if paper.p_values.len() >= 2 { 0.9 } else { 0.5 };
        let novelty_score = if paper.effect_sizes.len() >= 3 {
            0.8
        } else {
            0.6
        };
        let rigor_score = if paper.sample_size >= self.config.min_sample_size {
            0.85
        } else {
            0.6
        };

        let overall = (replication_score + novelty_score + rigor_score) / 3.0;

        let mut recommendations = Vec::new();
        if replication_score < 0.8 {
            recommendations.push("Increase replication runs".to_string());
        }
        if rigor_score < 0.8 {
            recommendations.push("Increase sample size".to_string());
        }

        Some(PublicationReadiness {
            paper_id: paper_id.to_string(),
            replication_score,
            novelty_score,
            rigor_score,
            overall_readiness: overall,
            recommendations,
        })
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Generate comprehensive report
    #[must_use]
    pub fn generate_report(&self) -> MetaAnalysisReport {
        let themes = self.identify_themes();
        let gaps = self.identify_gaps();

        let readiness: Vec<PublicationReadiness> = self
            .papers
            .iter()
            .filter_map(|p| self.assess_publication_readiness(&p.paper_id))
            .collect();

        MetaAnalysisReport {
            num_papers: self.papers.len(),
            themes,
            gaps,
            publication_readiness: readiness,
            overall_quality: self.calculate_overall_quality(),
        }
    }

    fn calculate_overall_quality(&self) -> f64 {
        if self.papers.is_empty() {
            return 0.0;
        }

        let sample_scores: f64 = self
            .papers
            .iter()
            .map(|p| (p.sample_size as f64 / 100.0).min(1.0))
            .sum();

        sample_scores / self.papers.len() as f64
    }
}

impl Default for MetaAggregator {
    fn default() -> Self {
        Self::new(MetaAnalysisConfig::default())
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Meta-analyzed effect size
#[derive(Debug, Clone)]
pub struct MetaEffectSize {
    /// metric.
    pub metric: String,
    /// mean effect.
    pub mean_effect: f64,
    /// std error.
    pub std_error: f64,
    /// num studies.
    pub num_studies: usize,
    /// heterogeneity.
    pub heterogeneity: f64,
}

/// Comprehensive meta-analysis report
#[derive(Debug, Clone)]
pub struct MetaAnalysisReport {
    /// num papers.
    pub num_papers: usize,
    /// themes.
    pub themes: Vec<CrossCuttingTheme>,
    /// gaps.
    pub gaps: Vec<ResearchGap>,
    /// publication readiness.
    pub publication_readiness: Vec<PublicationReadiness>,
    /// overall quality.
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub overall_quality: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_paper(id: &str, sample_size: usize) -> PaperResults {
        let mut metrics = HashMap::new();
        metrics.insert("trust_emergence".to_string(), 0.75);

        let mut effect_sizes = HashMap::new();
        effect_sizes.insert("main_effect".to_string(), 0.5);

        PaperResults {
            paper_id: id.to_string(),
            paper_name: format!("Paper {id}"),
            metrics,
            effect_sizes,
            p_values: HashMap::new(),
            sample_size,
            confidence_level: 0.95,
        }
    }

    #[test]
    fn test_add_and_aggregate() {
        let mut aggregator = MetaAggregator::default();

        aggregator.add_paper(create_test_paper("A", 100));
        aggregator.add_paper(create_test_paper("B", 150));

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(aggregator.papers.len(), 2);
    }

    #[test]
    fn test_compare_papers() {
        let mut aggregator = MetaAggregator::default();

        aggregator.add_paper(create_test_paper("A", 100));
        aggregator.add_paper(create_test_paper("B", 100));

        let comparison = aggregator.compare_papers("A", "B");
        assert!(comparison.is_some());
    }

    #[test]
    fn test_identify_themes() {
        let mut aggregator = MetaAggregator::default();

        let mut paper = create_test_paper("A", 100);
        paper.metrics.insert("trust_emergence".to_string(), 0.8);
        aggregator.add_paper(paper);

        let mut paper2 = create_test_paper("B", 100);
        paper2.metrics.insert("trust_emergence".to_string(), 0.7);
        aggregator.add_paper(paper2);

        let themes = aggregator.identify_themes();
        assert!(!themes.is_empty());
    }

    #[test]
    fn test_identify_gaps() {
        let mut aggregator = MetaAggregator::default();
        aggregator.add_paper(create_test_paper("A", 20)); // Small sample

        let gaps = aggregator.identify_gaps();
        assert!(!gaps.is_empty());
    }
}
