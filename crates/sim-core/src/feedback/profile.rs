use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{DimensionScore, Dot, FeedbackDimension};
use crate::types::{AgentId, Tick};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Baseball card: comprehensive agent capability profile
///
/// Inspired by Ray Dalio's "baseball cards" for employees.
/// Aggregates all feedback dimensions into a comprehensive
/// believability profile for an agent.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BaseballCard {
    /// Agent identifier
    pub agent_id: AgentId,
    /// Overall believability across all dimensions
    pub overall_believability: f32,
    /// Per-dimension scores
    pub dimension_scores: HashMap<FeedbackDimension, DimensionScore>,
    /// Top 3 strengths (highest scores with confidence)
    pub strengths: Vec<FeedbackDimension>,
    /// Top 3 weaknesses (lowest scores with confidence)
    pub weaknesses: Vec<FeedbackDimension>,
    /// When the card was last updated
    pub last_updated: Tick,
    /// Total dots received
    pub total_dots_received: usize,
    /// Total dots given
    pub total_dots_given: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl BaseballCard {
    /// Create a new baseball card for an agent
    #[must_use]
    pub fn new(agent_id: AgentId, tick: Tick) -> Self {
        let mut dimension_scores = HashMap::new();
        for dim in FeedbackDimension::all() {
            dimension_scores.insert(dim, DimensionScore::new(dim, tick));
        }

        Self {
            agent_id,
            overall_believability: 0.5,
            dimension_scores,
            strengths: Vec::new(),
            weaknesses: Vec::new(),
            last_updated: tick,
            total_dots_received: 0,
            total_dots_given: 0,
        }
    }

    /// Update the card with current dimension scores
    pub fn refresh(&mut self, collector: &DotCollector, tick: Tick) {
        self.overall_believability = collector.composite_believability();
        self.total_dots_received = collector.received.len();
        self.total_dots_given = collector.given.len();

        for dim in FeedbackDimension::all() {
            if let Some(score) = collector.believability.get(&dim) {
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                self.update_dimension_score(dim, *score, collector, tick);
            }
        }

        self.compute_strengths_and_weaknesses();
        self.last_updated = tick;
    }

    fn update_dimension_score(
        &mut self,
        dim: FeedbackDimension,
        score: f32,
        collector: &DotCollector,
        tick: Tick,
    ) {
        let dim_dots: Vec<&Dot> = collector
            .received
            .iter()
            .filter(|d| d.dimension == dim)
            .collect();
        let mut dim_score = DimensionScore::new(dim, tick);
        dim_score.update(&dim_dots, tick, 0.01);
        dim_score.score = score;
        if let Some(count) = collector.dot_counts.get(&dim) {
            dim_score.num_dots = *count;
        }
        self.dimension_scores.insert(dim, dim_score);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn compute_strengths_and_weaknesses(&mut self) {
        let mut sorted: Vec<_> = self
            .dimension_scores
            .iter()
            .filter(|(_, s)| s.confidence > 0.3 && s.num_dots >= 3)
            .collect();
        sorted.sort_by(|a, b| {
            b.1.score
                .partial_cmp(&a.1.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.strengths = sorted.iter().take(3).map(|(d, _)| **d).collect();
        sorted.sort_by(|a, b| {
            a.1.score
                .partial_cmp(&b.1.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.weaknesses = sorted.iter().take(3).map(|(d, _)| **d).collect();
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update the card from individual data fields (avoids borrow checker issues)
    #[allow(clippy::too_many_arguments)]
    pub fn update_from_data(
        &mut self,
        overall_believability: f32,
        total_received: usize,
        total_given: usize,
        believability: &HashMap<FeedbackDimension, f32>,
        dot_counts: &HashMap<FeedbackDimension, usize>,
        received_dots: &[Dot],
        tick: Tick,
        decay_rate: f32,
    ) {
        self.overall_believability = overall_believability;
        self.total_dots_received = total_received;
        self.total_dots_given = total_given;

        // Update dimension scores from data
        for dim in FeedbackDimension::all() {
            if let Some(score) = believability.get(&dim) {
                let dim_dots: Vec<&Dot> = received_dots
                    .iter()
                    .filter(|d| d.dimension == dim)
                    .collect();

                let mut dim_score = DimensionScore::new(dim, tick);
                dim_score.update(&dim_dots, tick, decay_rate);
                dim_score.score = *score;

                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                if let Some(count) = dot_counts.get(&dim) {
                    dim_score.num_dots = *count;
                }

                self.dimension_scores.insert(dim, dim_score);
            }
        }

        // Calculate strengths (top 3 dimensions with sufficient confidence)
        let mut sorted: Vec<_> = self
            .dimension_scores
            .iter()
            .filter(|(_, s)| s.confidence > 0.3 && s.num_dots >= 3)
            .collect();
        sorted.sort_by(|a, b| {
            b.1.score
                .partial_cmp(&a.1.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.strengths = sorted.iter().take(3).map(|(d, _)| **d).collect();

        // Calculate weaknesses (bottom 3 dimensions with sufficient confidence)
        sorted.sort_by(|a, b| {
            a.1.score
                .partial_cmp(&b.1.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.weaknesses = sorted.iter().take(3).map(|(d, _)| **d).collect();

        self.last_updated = tick;
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get score for a specific dimension
    #[must_use]
    pub fn get_dimension_score(&self, dim: FeedbackDimension) -> Option<f32> {
        self.dimension_scores.get(&dim).map(|s| s.score)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if agent has a particular strength
    #[must_use]
    pub fn has_strength(&self, dim: FeedbackDimension) -> bool {
        self.strengths.contains(&dim)
    }

    /// Check if agent has a particular weakness
    #[must_use]
    pub fn has_weakness(&self, dim: FeedbackDimension) -> bool {
        self.weaknesses.contains(&dim)
    }
}

/// Dot collector for an agent (all feedback received and given)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DotCollector {
    /// Agent identifier
    pub agent_id: AgentId,
    /// Dots received (feedback about this agent)
    pub received: Vec<Dot>,
    /// Dots given (feedback this agent provided)
    pub given: Vec<Dot>,
    /// Rolling window size for maintaining recent dots
    pub window_size: usize,
    /// Computed believability scores per dimension
    pub believability: HashMap<FeedbackDimension, f32>,
    /// Confidence-weighted counts per dimension
    pub dot_counts: HashMap<FeedbackDimension, usize>,
    /// Baseball card profile
    pub baseball_card: BaseballCard,
    /// Time decay rate for older dots (0.0 = no decay)
    pub decay_rate: f32,
}

impl Default for DotCollector {
    fn default() -> Self {
        Self {
            agent_id: 0,
            received: Vec::new(),
            given: Vec::new(),
            window_size: 100,
            believability: HashMap::new(),
            dot_counts: HashMap::new(),
            baseball_card: BaseballCard::default(),
            decay_rate: 0.01,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl DotCollector {
    /// Create a new dot collector for an agent
    #[must_use]
    pub fn new(agent_id: AgentId) -> Self {
        let mut collector = Self {
            agent_id,
            received: Vec::new(),
            given: Vec::new(),
            window_size: 100,
            believability: HashMap::new(),
            dot_counts: HashMap::new(),
            baseball_card: BaseballCard::new(agent_id, 0),
            decay_rate: 0.01,
        };

        // Initialize believability for all dimensions
        for dim in FeedbackDimension::all() {
            collector.believability.insert(dim, 0.5);
            collector.dot_counts.insert(dim, 0);
        }

        collector
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create with custom window size and decay rate
    #[must_use]
    pub fn with_config(agent_id: AgentId, window_size: usize, decay_rate: f32) -> Self {
        let mut collector = Self::new(agent_id);
        collector.window_size = window_size;
        collector.decay_rate = decay_rate.clamp(0.0, 1.0);
        collector
    }

    /// Add a received dot (feedback about this agent)
    pub fn add_received_dot(&mut self, dot: Dot) {
        // Verify the dot is for this agent
        if dot.to != self.agent_id {
            return;
        }

        self.received.push(dot);

        // Maintain window size (FIFO)
        if self.received.len() > self.window_size {
            self.received.remove(0);
        }

        self.recompute_believability();
    }

    /// Add a given dot (feedback this agent provided)
    pub fn add_given_dot(&mut self, dot: Dot) {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Verify the dot is from this agent
        if dot.from != self.agent_id {
            return;
        }

        self.given.push(dot);

        // Maintain window size
        if self.given.len() > self.window_size {
            self.given.remove(0);
        }
    }

    /// Compute believability scores from recent feedback
    fn recompute_believability(&mut self) {
        let current_tick = self.received.last().map_or(0, |d| d.tick);

        for dim in FeedbackDimension::all() {
            let dots: Vec<&Dot> = self
                .received
                .iter()
                .filter(|d| d.dimension == dim)
                .collect();
            if dots.is_empty() {
                continue;
            }
            if let Some(score) = Self::compute_weighted_score(&dots, current_tick, self.decay_rate)
            {
                self.believability.insert(dim, score);
                self.dot_counts.insert(dim, dots.len());
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let composite = self.composite_believability();
        let received_len = self.received.len();
        let given_len = self.given.len();
        self.baseball_card.update_from_data(
            composite,
            received_len,
            given_len,
            &self.believability,
            &self.dot_counts,
            &self.received,
            current_tick,
            self.decay_rate,
        );
    }

    fn compute_weighted_score(dots: &[&Dot], current_tick: Tick, decay_rate: f32) -> Option<f32> {
        let mut weighted_sum = 0.0;
        let mut weight_sum = 0.0;
        for dot in dots {
            let decay = dot.time_decay(current_tick, decay_rate);
            let weight = dot.confidence * decay;
            weighted_sum += dot.rating * weight;
            weight_sum += weight;
        }
        if weight_sum > 0.0 {
            Some((weighted_sum / weight_sum).clamp(0.0, 1.0))
        } else {
            None
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get composite believability (overall trustworthiness)
    #[must_use]
    pub fn composite_believability(&self) -> f32 {
        if self.believability.is_empty() {
            return 0.5;
        }

        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for (dim, score) in &self.believability {
            let count = self.dot_counts.get(dim).copied().unwrap_or(0);
            let confidence = (count as f32 / 10.0).min(1.0); // Max confidence at 10+ dots

            weighted_sum += score * confidence;
            total_weight += confidence;
        }

        if total_weight > 0.0 {
            (weighted_sum / total_weight).clamp(0.0, 1.0)
        } else {
            0.5
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get believability for specific dimension
    #[must_use]
    pub fn dimension_believability(&self, dim: FeedbackDimension) -> f32 {
        self.believability.get(&dim).copied().unwrap_or(0.5)
    }

    /// Generate baseball card profile
    #[must_use]
    pub fn generate_baseball_card(&self, tick: Tick) -> BaseballCard {
        let mut card = BaseballCard::new(self.agent_id, tick);
        card.refresh(self, tick);
        card
    }

    /// Get dots from specific time period
    #[must_use]
    pub fn get_dots_in_period(&self, start: Tick, end: Tick) -> Vec<&Dot> {
        self.received
            .iter()
            .filter(|d| d.tick >= start && d.tick <= end)
            .collect()
    }

    /// Get average rating from specific rater
    #[must_use]
    pub fn get_average_from_rater(&self, rater_id: AgentId) -> Option<f32> {
        let ratings: Vec<f32> = self
            .received
            .iter()
            .filter(|d| d.from == rater_id)
            .map(|d| d.rating)
            .collect();

        if ratings.is_empty() {
            None
        } else {
            Some(ratings.iter().sum::<f32>() / ratings.len() as f32)
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get all ratings from a specific rater
    #[must_use]
    pub fn get_ratings_from_rater(&self, rater_id: AgentId) -> Vec<&Dot> {
        self.received
            .iter()
            .filter(|d| d.from == rater_id)
            .collect()
    }

    /// Detect rating bias (does this agent rate differently than population?)
    #[must_use]
    pub fn detect_rating_bias(
        &self,
        population_average: &HashMap<FeedbackDimension, f32>,
    ) -> super::types::RatingBias {
        let mut bias = super::types::RatingBias::new(self.agent_id);

        if self.given.is_empty() {
            return bias;
        }

        // Group given ratings by dimension
        let mut by_dimension: HashMap<FeedbackDimension, Vec<f32>> = HashMap::new();
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let mut all_ratings: Vec<f32> = Vec::new();

        for dot in &self.given {
            by_dimension
                .entry(dot.dimension)
                .or_default()
                .push(dot.rating);
            all_ratings.push(dot.rating);
        }

        // Calculate overall bias
        let my_average = all_ratings.iter().sum::<f32>() / all_ratings.len() as f32;
        let pop_average: f32 =
            population_average.values().sum::<f32>() / population_average.len().max(1) as f32;
        bias.overall_bias = my_average - pop_average;
        bias.ratings_count = self.given.len();
        bias.confidence = (self.given.len() as f32 / 20.0).min(1.0); // Max at 20+ ratings

        // Calculate per-dimension bias
        for (dim, ratings) in by_dimension {
            if ratings.len() >= 3 {
                let my_dim_avg = ratings.iter().sum::<f32>() / ratings.len() as f32;
                let pop_dim_avg = population_average.get(&dim).copied().unwrap_or(0.5);
                bias.dimension_biases.insert(dim, my_dim_avg - pop_dim_avg);
            }
        }

        bias
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get rating distribution (histogram buckets)
    #[must_use]
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn get_rating_distribution(&self) -> HashMap<String, usize> {
        let mut distribution = HashMap::new();
        distribution.insert("0.0-0.2".to_string(), 0);
        distribution.insert("0.2-0.4".to_string(), 0);
        distribution.insert("0.4-0.6".to_string(), 0);
        distribution.insert("0.6-0.8".to_string(), 0);
        distribution.insert("0.8-1.0".to_string(), 0);

        for dot in &self.received {
            let bucket = match dot.rating {
                r if r < 0.2 => "0.0-0.2",
                r if r < 0.4 => "0.2-0.4",
                r if r < 0.6 => "0.4-0.6",
                r if r < 0.8 => "0.6-0.8",
                _ => "0.8-1.0",
            };
            if let Some(count) = distribution.get_mut(bucket) {
                *count += 1;
            }
        }

        distribution
    }

    /// Clear all dots (for testing or privacy)
    pub fn clear(&mut self) {
        self.received.clear();
        self.given.clear();
        for dim in FeedbackDimension::all() {
            self.believability.insert(dim, 0.5);
            self.dot_counts.insert(dim, 0);
        }
    }
}
