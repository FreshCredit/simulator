use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::profile::{BaseballCard, DotCollector};
use super::types::{BelievabilityVoteResult, FeedbackDimension, PopulationStats, RatingBias};
use crate::types::AgentId;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// `DotCollectorRegistry`: manages all agents' dot collectors
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DotCollectorRegistry {
    /// Collectors per agent
    pub collectors: HashMap<AgentId, DotCollector>,
    /// Population averages per dimension
    pub population_averages: HashMap<FeedbackDimension, f32>,
    /// Default window size for new collectors
    pub default_window_size: usize,
    /// Default decay rate for new collectors
    pub default_decay_rate: f32,
}

impl DotCollectorRegistry {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create a new registry
    #[must_use]
    pub fn new() -> Self {
        let mut registry = Self {
            collectors: HashMap::new(),
            population_averages: HashMap::new(),
            default_window_size: 100,
            default_decay_rate: 0.01,
        };

        // Initialize population averages
        for dim in FeedbackDimension::all() {
            registry.population_averages.insert(dim, 0.5);
        }

        registry
    }

    /// Create with custom configuration
    #[must_use]
    pub fn with_config(window_size: usize, decay_rate: f32) -> Self {
        let mut registry = Self::new();
        registry.default_window_size = window_size;
        registry.default_decay_rate = decay_rate.clamp(0.0, 1.0);
        registry
    }

    /// Get or create collector for agent
    pub fn get_collector(&mut self, agent_id: AgentId) -> &mut DotCollector {
        self.collectors.entry(agent_id).or_insert_with(|| {
            DotCollector::with_config(agent_id, self.default_window_size, self.default_decay_rate)
        })
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get collector (immutable)
    #[must_use]
    pub fn get_collector_ref(&self, agent_id: AgentId) -> Option<&DotCollector> {
        self.collectors.get(&agent_id)
    }

    /// Record a dot between two agents
    pub fn record_dot(&mut self, dot: super::types::Dot) {
        let from_id = dot.from;
        let to_id = dot.to;

        // Add to receiver's received
        let receiver = self.get_collector(to_id);
        receiver.add_received_dot(dot.clone());

        // Add to giver's given
        let giver = self.get_collector(from_id);
        giver.add_given_dot(dot);

        // Update population averages
        self.update_population_averages();
    }

    /// Record multiple dots at once
    pub fn record_dots(&mut self, dots: Vec<super::types::Dot>) {
        for dot in dots {
            self.record_dot(dot);
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        }
    }

    /// Get top agents by dimension
    #[must_use]
    pub fn get_top_by_dimension(&self, dim: FeedbackDimension, n: usize) -> Vec<(AgentId, f32)> {
        let mut agents: Vec<(AgentId, f32)> = self
            .collectors
            .iter()
            .filter_map(|(id, collector)| {
                collector
                    .believability
                    .get(&dim)
                    .filter(|&&_| {
                        // Only include if has sufficient data
                        let count = collector.dot_counts.get(&dim).copied().unwrap_or(0);
                        count >= 3
                    })
                    .map(|&score| (*id, score))
            })
            .collect();

        agents.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        agents.truncate(n);
        agents
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get bottom agents by dimension (for identifying those needing improvement)
    #[must_use]
    pub fn get_bottom_by_dimension(&self, dim: FeedbackDimension, n: usize) -> Vec<(AgentId, f32)> {
        let mut agents: Vec<(AgentId, f32)> = self
            .collectors
            .iter()
            .filter_map(|(id, collector)| {
                collector
                    .believability
                    .get(&dim)
                    .filter(|&&_| {
                        let count = collector.dot_counts.get(&dim).copied().unwrap_or(0);
                        count >= 3
                    })
                    .map(|&score| (*id, score))
            })
            .collect();

        agents.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        agents.truncate(n);
        agents
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get believability-weighted vote
    #[must_use]
    pub fn compute_believability_weighted_vote(
        &self,
        votes: &[(AgentId, bool)],
        eligible_voters: Option<&[AgentId]>,
    ) -> BelievabilityVoteResult {
        let mut weighted_yes = 0.0;
        let mut weighted_no = 0.0;
        let mut total_believability = 0.0;
        let mut num_voted = 0;

        for (agent_id, vote) in votes {
            if let Some(collector) = self.collectors.get(agent_id) {
                let believability = collector.composite_believability();
                total_believability += believability;
                num_voted += 1;

                if *vote {
                    weighted_yes += believability;
                } else {
                    weighted_no += believability;
                }
            }
        }

        let total_weighted = weighted_yes + weighted_no;
        let passed = if total_weighted > 0.0 {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            weighted_yes / total_weighted > 0.5
        } else {
            false
        };

        let margin = if total_weighted > 0.0 {
            (weighted_yes - weighted_no) / total_weighted
        } else {
            0.0
        };

        let total_eligible = eligible_voters.map_or(votes.len(), <[u64]>::len);
        let participation_rate = if total_eligible > 0 {
            num_voted as f32 / total_eligible as f32
        } else {
            0.0
        };

        BelievabilityVoteResult {
            weighted_yes,
            weighted_no,
            total_believability_participating: total_believability,
            passed,
            margin,
            participation_rate,
            total_eligible,
            num_voted,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update population averages
    fn update_population_averages(&mut self) {
        for dim in FeedbackDimension::all() {
            let scores: Vec<f32> = self
                .collectors
                .values()
                .filter_map(|c| c.believability.get(&dim).copied())
                .collect();

            if !scores.is_empty() {
                let avg = scores.iter().sum::<f32>() / scores.len() as f32;
                self.population_averages.insert(dim, avg);
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get population-wide statistics
    #[must_use]
    pub fn get_population_stats(&self) -> HashMap<FeedbackDimension, PopulationStats> {
        let mut stats = HashMap::new();

        for dim in FeedbackDimension::all() {
            // Only include agents who have actually received dots for this dimension
            let scores: Vec<f32> = self
                .collectors
                .values()
                .filter_map(|c| {
                    let has_dots = c.dot_counts.get(&dim).copied().unwrap_or(0) > 0;
                    if has_dots {
                        c.believability.get(&dim).copied()
                    } else {
                        None
                    }
                })
                .collect();

            if !scores.is_empty() {
                let mean = scores.iter().sum::<f32>() / scores.len() as f32;
                let variance =
                    scores.iter().map(|&s| (s - mean).powi(2)).sum::<f32>() / scores.len() as f32;
                let std_dev = variance.sqrt();
                let min = scores.iter().fold(f32::INFINITY, |a, &b| a.min(b));
                let max = scores.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));

                stats.insert(
                    dim,
                    PopulationStats {
                        dimension: dim,
                        mean,
                        std_dev,
                        min,
                        max,
                        count: scores.len(),
                    },
                );
            }
        }

        stats
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get agents with rating bias analysis
    #[must_use]
    pub fn get_rating_biases(&self) -> Vec<RatingBias> {
        self.collectors
            .values()
            .map(|c| c.detect_rating_bias(&self.population_averages))
            .collect()
    }

    /// Get all baseball cards
    #[must_use]
    pub fn get_all_baseball_cards(&self) -> Vec<&BaseballCard> {
        self.collectors.values().map(|c| &c.baseball_card).collect()
    }

    /// Get baseball card for specific agent
    #[must_use]
    pub fn get_baseball_card(&self, agent_id: AgentId) -> Option<&BaseballCard> {
        self.collectors.get(&agent_id).map(|c| &c.baseball_card)
    }

    /// Remove an agent from the registry
    pub fn remove_agent(&mut self, agent_id: AgentId) {
        self.collectors.remove(&agent_id);
        self.update_population_averages();
    }

    /// Get total number of dots recorded
    #[must_use]
    pub fn total_dots_recorded(&self) -> usize {
        self.collectors.values().map(|c| c.received.len()).sum()
    }

    /// Clear all data (for testing)
    pub fn clear(&mut self) {
        self.collectors.clear();
        for dim in FeedbackDimension::all() {
            self.population_averages.insert(dim, 0.5);
        }
    }
}
