//! Collective Intelligence (P2-RES-1)
//!
//! Wisdom of crowds and group decision-making (Woolley et al. 2010)

use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Collective intelligence configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WisdomOfCrowdsConfig {
    /// diversity weight.
    pub diversity_weight: f32,
    /// independence factor.
    pub independence_factor: f32,
    /// aggregation method.
    pub aggregation_method: AggregationMethod,
}

impl Default for WisdomOfCrowdsConfig {
    fn default() -> Self {
        Self {
            diversity_weight: 0.3,
            independence_factor: 0.5,
            aggregation_method: AggregationMethod::Median,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Aggregation methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregationMethod {
    /// mean.
    Mean,
    /// median.
    Median,
    /// trimmed mean.
    TrimmedMean,
    /// weighted by confidence.
    WeightedByConfidence,
}

/// Individual estimate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Estimate {
    /// agent id.
    pub agent_id: u64,
    /// value.
    pub value: f32,
    /// confidence.
    pub confidence: f32,
    /// expertise.
    pub expertise: f32,
}

/// Collective intelligence system
#[derive(Debug, Clone)]
pub struct CollectiveIntelligence {
    /// config.
    pub config: WisdomOfCrowdsConfig,
    /// estimates.
    pub estimates: Vec<Estimate>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl CollectiveIntelligence {
    #[must_use]
    /// new.
    pub const fn new(config: WisdomOfCrowdsConfig) -> Self {
        Self {
            config,
            estimates: Vec::new(),
        }
    }

    /// Add estimate
    pub fn add_estimate(&mut self, estimate: Estimate) {
        self.estimates.push(estimate);
    }

    /// Aggregate estimates using configured method
    #[must_use]
    pub fn aggregate(&self) -> Option<f32> {
        if self.estimates.is_empty() {
            return None;
        }

        match self.config.aggregation_method {
            AggregationMethod::Mean => Some(
                self.estimates.iter().map(|e| e.value).sum::<f32>() / self.estimates.len() as f32,
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            ),
            AggregationMethod::Median => {
                let mut values: Vec<f32> = self.estimates.iter().map(|e| e.value).collect();
                values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let mid = values.len() / 2;
                Some(if values.len() % 2 == 0 {
                    (values[mid - 1] + values[mid]) / 2.0
                } else {
                    values[mid]
                })
            }
            AggregationMethod::TrimmedMean => {
                let mut values: Vec<f32> = self.estimates.iter().map(|e| e.value).collect();
                values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let trim = (values.len() as f32 * 0.1) as usize;
                if trim * 2 >= values.len() || values.len() < 3 {
                    Some(values.iter().sum::<f32>() / values.len() as f32)
                } else {
                    let trimmed = &values[trim..values.len() - trim];
                    Some(trimmed.iter().sum::<f32>() / trimmed.len() as f32)
                }
            }
            AggregationMethod::WeightedByConfidence => {
                let weighted_sum: f32 = self.estimates.iter().map(|e| e.value * e.confidence).sum();
                let weight_sum: f32 = self.estimates.iter().map(|e| e.confidence).sum();
                Some(if weight_sum.abs() > 1e-12 {
                    weighted_sum / weight_sum
                } else {
                    self.estimates.iter().map(|e| e.value).sum::<f32>()
                        / self.estimates.len() as f32
                })
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate prediction accuracy
    #[must_use]
    pub fn accuracy(&self, true_value: f32) -> f32 {
        match self.aggregate() {
            Some(prediction) => {
                let denom = true_value.abs().max(1e-6);
                (1.0 - (prediction - true_value).abs() / denom).clamp(0.0, 1.0)
            }
            None => 0.0,
        }
    }

    /// Estimate diversity
    #[must_use]
    pub fn diversity(&self) -> f32 {
        if self.estimates.len() < 2 {
            return 0.0;
        }

        let mean =
            self.estimates.iter().map(|e| e.value).sum::<f32>() / self.estimates.len() as f32;
        let variance = self
            .estimates
            .iter()
            .map(|e| (e.value - mean).powi(2))
            .sum::<f32>()
            / self.estimates.len() as f32;

        variance.sqrt()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Collective accuracy score
    #[must_use]
    pub fn collective_accuracy(&self, true_value: f32) -> CollectiveAccuracy {
        if self.estimates.is_empty() {
            return CollectiveAccuracy {
                aggregate: 0.0,
                aggregate_error: 0.0,
                avg_individual_error: 0.0,
                improvement: 0.0,
                diversity: 0.0,
            };
        }
        let aggregate = self.aggregate().unwrap_or(0.0);
        let aggregate_error = (aggregate - true_value).abs();

        let individual_errors: Vec<f32> = self
            .estimates
            .iter()
            .map(|e| (e.value - true_value).abs())
            .collect();

        let avg_individual_error =
            individual_errors.iter().sum::<f32>() / individual_errors.len() as f32;

        CollectiveAccuracy {
            aggregate,
            aggregate_error,
            avg_individual_error,
            improvement: avg_individual_error - aggregate_error,
            diversity: self.diversity(),
        }
    }

    /// Clear estimates for new round
    pub fn clear(&mut self) {
        self.estimates.clear();
    }
}

impl Default for CollectiveIntelligence {
    fn default() -> Self {
        Self::new(WisdomOfCrowdsConfig::default())
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Collective accuracy metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveAccuracy {
    /// aggregate.
    pub aggregate: f32,
    /// aggregate error.
    pub aggregate_error: f32,
    /// avg individual error.
    pub avg_individual_error: f32,
    /// improvement.
    pub improvement: f32,
    /// diversity.
    pub diversity: f32,
}

/// Group performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GroupPerformance {
    /// group id.
    pub group_id: String,
    /// task count.
    pub task_count: u32,
    /// avg accuracy.
    pub avg_accuracy: f32,
    /// social sensitivity.
    pub social_sensitivity: f32,
    /// turn taking equity.
    pub turn_taking_equity: f32,
    /// proportion female.
    pub proportion_female: f32,
}

impl GroupPerformance {
    /// Calculate collective intelligence factor (c)
    /// Based on Woolley et al. 2010 findings
    #[must_use]
    pub fn collective_intelligence_factor(&self) -> f32 {
        // Social sensitivity and turn-taking are strong predictors
        let social_component = self.social_sensitivity * 0.5;
        let communication_component = self.turn_taking_equity * 0.3;
        let diversity_component = (1.0 - (self.proportion_female - 0.5).abs() * 2.0) * 0.2;

        social_component + communication_component + diversity_component
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Rank against other groups
    #[must_use]
    pub fn rank_among(&self, others: &[Self]) -> usize {
        let c_factor = self.collective_intelligence_factor();
        others
            .iter()
            .filter(|g| g.collective_intelligence_factor() > c_factor)
            .count()
            + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collective_accuracy() {
        let mut ci = CollectiveIntelligence::default();

        // Diverse estimates around true value
        ci.add_estimate(Estimate {
            agent_id: 1,
            value: 95.0,
            confidence: 0.8,
            expertise: 0.7,
        });
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        ci.add_estimate(Estimate {
            agent_id: 2,
            value: 105.0,
            confidence: 0.6,
            expertise: 0.5,
        });
        ci.add_estimate(Estimate {
            agent_id: 3,
            value: 98.0,
            confidence: 0.9,
            expertise: 0.9,
        });
        ci.add_estimate(Estimate {
            agent_id: 4,
            value: 102.0,
            confidence: 0.7,
            expertise: 0.6,
        });

        let true_value = 100.0;
        let accuracy = ci.collective_accuracy(true_value);

        assert!(accuracy.aggregate_error < accuracy.avg_individual_error);
        assert!(accuracy.improvement > 0.0);
    }

    #[test]
    fn test_diversity() {
        let mut ci = CollectiveIntelligence::default();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        ci.add_estimate(Estimate {
            agent_id: 1,
            value: 90.0,
            confidence: 0.5,
            expertise: 0.5,
        });
        ci.add_estimate(Estimate {
            agent_id: 2,
            value: 110.0,
            confidence: 0.5,
            expertise: 0.5,
        });

        let diversity = ci.diversity();
        assert!(diversity > 0.0);
    }

    #[test]
    fn test_group_c_factor() {
        let group = GroupPerformance {
            group_id: "test".to_string(),
            task_count: 10,
            avg_accuracy: 0.8,
            social_sensitivity: 0.7,
            turn_taking_equity: 0.6,
            proportion_female: 0.5,
        };

        let c = group.collective_intelligence_factor();
        assert!(c > 0.0 && c <= 1.0);
    }
}
