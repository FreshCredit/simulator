use serde::{Deserialize, Serialize};

use crate::types::{AgentId, Tick};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Dimensions on which agents can be rated
/// Per Dalio: "baseball card" dimensions of competence
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeedbackDimension {
    /// Ability to achieve goals
    Competence,
    /// Consistency in behavior
    Reliability,
    /// Honesty, alignment of words/actions
    Integrity,
    /// Willingness to work with others
    Collaboration,
    /// Creativity and novel solutions
    Innovation,
    /// Ability to help others learn
    Teaching,
    /// Quality of decisions under uncertainty
    DecisionMaking,
    /// Performance under pressure
    StressHandling,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl FeedbackDimension {
    /// All available feedback dimensions
    #[must_use]
    pub fn all() -> Vec<Self> {
        vec![
            Self::Competence,
            Self::Reliability,
            Self::Integrity,
            Self::Collaboration,
            Self::Innovation,
            Self::Teaching,
            Self::DecisionMaking,
            Self::StressHandling,
        ]
    }

    /// Human-readable description of the dimension
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Competence => "Ability to achieve goals",
            Self::Reliability => "Consistency in behavior",
            Self::Integrity => "Honesty and alignment of words/actions",
            Self::Collaboration => "Willingness to work with others",
            Self::Innovation => "Creativity and novel solutions",
            Self::Teaching => "Ability to help others learn",
            Self::DecisionMaking => "Quality of decisions under uncertainty",
            Self::StressHandling => "Performance under pressure",
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Type of interaction that generated the feedback
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InteractionType {
    /// Economic exchange
    Trade,
    /// Joint work on shared goal
    Collaboration,
    /// Resolution of disagreement
    ConflictResolution,
    /// Knowledge transfer
    Teaching,
    /// Choice under uncertainty
    DecisionMaking,
    /// Response to emergency
    CrisisResponse,
    /// General interaction
    General,
}

impl InteractionType {
    /// All interaction types
    #[must_use]
    pub fn all() -> Vec<Self> {
        vec![
            Self::Trade,
            Self::Collaboration,
            Self::ConflictResolution,
            Self::Teaching,
            Self::DecisionMaking,
            Self::CrisisResponse,
            Self::General,
        ]
    }

    /// Get dimensions most relevant to this interaction type
    #[must_use]
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub fn relevant_dimensions(&self) -> Vec<FeedbackDimension> {
        match self {
            Self::Trade => vec![
                FeedbackDimension::Integrity,
                FeedbackDimension::Reliability,
                FeedbackDimension::Competence,
            ],
            Self::Collaboration => vec![
                FeedbackDimension::Collaboration,
                FeedbackDimension::Reliability,
                FeedbackDimension::Competence,
            ],
            Self::ConflictResolution => vec![
                FeedbackDimension::Integrity,
                FeedbackDimension::DecisionMaking,
                FeedbackDimension::StressHandling,
            ],
            Self::Teaching => vec![
                FeedbackDimension::Teaching,
                FeedbackDimension::Collaboration,
                FeedbackDimension::Competence,
            ],
            Self::DecisionMaking => vec![
                FeedbackDimension::DecisionMaking,
                FeedbackDimension::Competence,
                FeedbackDimension::Innovation,
            ],
            Self::CrisisResponse => vec![
                FeedbackDimension::StressHandling,
                FeedbackDimension::DecisionMaking,
                FeedbackDimension::Reliability,
            ],
            Self::General => FeedbackDimension::all(),
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// A single feedback "dot" (rating)
///
/// Represents one rating given by an agent to another agent
/// on a specific dimension. Includes confidence and context.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dot {
    /// Agent who gave the rating
    pub from: AgentId,
    /// Agent being rated
    pub to: AgentId,
    /// Dimension being rated
    pub dimension: FeedbackDimension,
    /// Rating value (0.0 to 1.0)
    pub rating: f32,
    /// Rater's confidence in this rating (0.0 to 1.0)
    pub confidence: f32,
    /// When the rating was given
    pub tick: Tick,
    /// Context description (what was being evaluated)
    pub context: String,
    /// Type of interaction
    pub interaction_type: InteractionType,
}

impl Dot {
    /// Create a new dot with validation
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        from: AgentId,
        to: AgentId,
        dimension: FeedbackDimension,
        rating: f32,
        confidence: f32,
        tick: Tick,
        context: impl Into<String>,
        interaction_type: InteractionType,
    ) -> Self {
        Self {
            from,
            to,
            dimension,
            rating: rating.clamp(0.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
            tick,
            context: context.into(),
            interaction_type,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get weighted rating (rating * confidence)
    #[must_use]
    pub fn weighted_rating(&self) -> f32 {
        self.rating * self.confidence
    }

    /// Calculate time decay factor (older = less weight)
    /// `decay_rate`: how quickly weight decreases (0.0 = no decay, 1.0 = instant decay)
    #[must_use]
    pub fn time_decay(&self, current_tick: Tick, decay_rate: f32) -> f32 {
        let age = current_tick.saturating_sub(self.tick);
        let decay = (-decay_rate * age as f32).exp();
        decay.clamp(0.0, 1.0)
    }
}

/// Score for a single dimension with metadata
///
/// Aggregates multiple dots for a specific feedback dimension,
/// computing weighted scores and tracking trends over time.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DimensionScore {
    /// The dimension being scored
    pub dimension: FeedbackDimension,
    /// Current score (0.0 to 1.0)
    pub score: f32,
    /// Confidence in the score (0.0 to 1.0)
    pub confidence: f32,
    /// Number of dots contributing to this score
    pub num_dots: usize,
    /// Trend direction (-1.0 = declining, 1.0 = improving)
    pub trend: f32,
    /// When this score was last updated
    pub last_updated: Tick,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl DimensionScore {
    /// Create a new dimension score
    #[must_use]
    pub const fn new(dimension: FeedbackDimension, tick: Tick) -> Self {
        Self {
            dimension,
            score: 0.5, // Neutral starting point
            confidence: 0.0,
            num_dots: 0,
            trend: 0.0,
            last_updated: tick,
        }
    }

    /// Update score with new dots
    pub fn update(&mut self, dots: &[&Dot], current_tick: Tick, decay_rate: f32) {
        if dots.is_empty() {
            return;
        }

        let mut weighted_sum = 0.0;
        let mut weight_sum = 0.0;
        let mut old_weighted_sum = 0.0;
        let mut old_weight_sum = 0.0;

        for dot in dots {
            let decay = dot.time_decay(current_tick, decay_rate);
            let weight = dot.confidence * decay;
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001

            weighted_sum += dot.rating * weight;
            weight_sum += weight;

            // For trend: separate recent from older ratings
            if decay > 0.7 {
                // Recent ratings (high decay = recent)
                old_weighted_sum += dot.rating * weight;
                old_weight_sum += weight;
            }
        }

        if weight_sum > 0.0 {
            let new_score = weighted_sum / weight_sum;

            // Calculate trend
            if old_weight_sum > 0.0 && self.num_dots > 0 {
                let _recent_avg = old_weighted_sum / old_weight_sum;
                self.trend = (new_score - self.score).clamp(-1.0, 1.0);
            }

            self.score = new_score;
            self.confidence = (weight_sum / dots.len() as f32).clamp(0.0, 1.0);
            self.num_dots = dots.len();
        }

        self.last_updated = current_tick;
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Bias in an agent's ratings compared to population
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RatingBias {
    /// Agent being analyzed
    pub agent_id: AgentId,
    /// Overall bias (positive = rates higher than average)
    pub overall_bias: f32,
    /// Per-dimension biases
    pub dimension_biases: std::collections::HashMap<FeedbackDimension, f32>,
    /// Confidence in the bias calculation
    pub confidence: f32,
    /// Number of ratings analyzed
    pub ratings_count: usize,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl RatingBias {
    /// Create a new rating bias analysis
    #[must_use]
    pub fn new(agent_id: AgentId) -> Self {
        Self {
            agent_id,
            overall_bias: 0.0,
            dimension_biases: std::collections::HashMap::new(),
            confidence: 0.0,
            ratings_count: 0,
        }
    }

    /// Check if agent is a "harsh" rater (negative bias)
    #[must_use]
    pub fn is_harsh(&self) -> bool {
        self.overall_bias < -0.1 && self.confidence > 0.5
    }

    /// Check if agent is a "lenient" rater (positive bias)
    #[must_use]
    pub fn is_lenient(&self) -> bool {
        self.overall_bias > 0.1 && self.confidence > 0.5
    }

    /// Check if agent has balanced ratings (low bias)
    #[must_use]
    pub fn is_balanced(&self) -> bool {
        self.overall_bias.abs() < 0.05 && self.confidence > 0.5
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Result of a believability-weighted vote
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BelievabilityVoteResult {
    /// Weighted votes for "yes"
    pub weighted_yes: f32,
    /// Weighted votes for "no"
    pub weighted_no: f32,
    /// Total believability of participating agents
    pub total_believability_participating: f32,
    /// Whether the vote passed (> 0.5 weighted threshold)
    pub passed: bool,
    /// Margin of victory (positive = yes won)
    pub margin: f32,
    /// Participation rate (0.0 to 1.0)
    pub participation_rate: f32,
    /// Total eligible voters
    pub total_eligible: usize,
    /// Number of voters who participated
    pub num_voted: usize,
}

impl BelievabilityVoteResult {
    /// Get the winning side as a string
    #[must_use]
    pub const fn winner(&self) -> &'static str {
        if self.passed {
            "yes"
        } else {
            "no"
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get consensus level (1.0 = unanimous, 0.5 = split)
    #[must_use]
    pub fn consensus_level(&self) -> f32 {
        let total = self.weighted_yes + self.weighted_no;
        if total == 0.0 {
            return 0.0;
        }
        let max_side = self.weighted_yes.max(self.weighted_no);
        max_side / total
    }
}

/// Population statistics for a dimension
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PopulationStats {
    /// The dimension
    pub dimension: FeedbackDimension,
    /// Mean score
    pub mean: f32,
    /// Standard deviation
    pub std_dev: f32,
    /// Minimum score
    pub min: f32,
    /// Maximum score
    pub max: f32,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Number of agents
    pub count: usize,
}

impl PopulationStats {
    /// Get percentile rank for a score
    #[must_use]
    pub fn percentile_rank(&self, score: f32) -> f32 {
        if self.std_dev == 0.0 {
            return 50.0;
        }
        // Approximate percentile using normal distribution
        let z_score = (score - self.mean) / self.std_dev;
        // Simplified: map z-score to percentile
        (50.0 + z_score * 34.0).clamp(0.0, 100.0)
    }

    /// Check if score is above average
    #[must_use]
    pub fn is_above_average(&self, score: f32) -> bool {
        score > self.mean
    }

    /// Check if score is an outlier (2+ std dev from mean)
    #[must_use]
    pub fn is_outlier(&self, score: f32) -> bool {
        (score - self.mean).abs() > 2.0 * self.std_dev
    }
}
