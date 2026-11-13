//! Scoring Rules for Prediction Markets

use super::Resolution;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Scoring rule trait
pub trait ScoringRule {
    /// Calculate score for a prediction
    fn score(&self, prediction: f64, outcome: f64) -> f64;

    /// Calculate expected score
    fn expected_score(&self, predicted_prob: f64, true_prob: f64) -> f64;

    /// Check if scoring rule is proper (incentivizes honest reporting)
    fn is_proper(&self) -> bool;
}

/// Logarithmic scoring rule
///
/// Score = log(prediction) if outcome occurs
///       = log(1 - prediction) if outcome doesn't occur
///
/// This is a strictly proper scoring rule.
pub struct LogarithmicScoringRule;

// TAG: surface=simulation owner=research-team rule=SIM-001
impl LogarithmicScoringRule {
    /// Create new logarithmic scoring rule
    pub fn new() -> Self {
        Self
    }
}

impl Default for LogarithmicScoringRule {
    fn default() -> Self {
        Self::new()
    }
}

impl ScoringRule for LogarithmicScoringRule {
    fn score(&self, prediction: f64, outcome: f64) -> f64 {
        let prediction = prediction.clamp(1e-10, 1.0 - 1e-10);

        if outcome >= 0.5 {
            prediction.ln()
        } else {
            (1.0 - prediction).ln()
        }
    }

    fn expected_score(&self, predicted_prob: f64, true_prob: f64) -> f64 {
        let p = predicted_prob.clamp(1e-10, 1.0 - 1e-10);
        let q = true_prob;

        q * p.ln() + (1.0 - q) * (1.0 - p).ln()
    }

    fn is_proper(&self) -> bool {
        true // Logarithmic is strictly proper
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Quadratic (Brier) scoring rule
///
/// Score = 2*p - p^2 - 1 if outcome = 1
///       = 1 - p^2 if outcome = 0
///
/// Or equivalently: Score = 1 - (outcome - prediction)^2
pub struct QuadraticScoringRule;

impl QuadraticScoringRule {
    /// Create new quadratic scoring rule
    pub fn new() -> Self {
        Self
    }
}

impl Default for QuadraticScoringRule {
    fn default() -> Self {
        Self::new()
    }
}

impl ScoringRule for QuadraticScoringRule {
    fn score(&self, prediction: f64, outcome: f64) -> f64 {
        let p = prediction.clamp(0.0, 1.0);
        let outcome_bin = if outcome >= 0.5 { 1.0 } else { 0.0 };
        1.0 - (outcome_bin - p).powi(2)
    }

    fn expected_score(&self, predicted_prob: f64, true_prob: f64) -> f64 {
        let p = predicted_prob.clamp(0.0, 1.0);
        let q = true_prob.clamp(0.0, 1.0);

        // E[score] = q * (1 - (1-p)^2) + (1-q) * (1 - p^2)
        q * (1.0 - (1.0 - p).powi(2)) + (1.0 - q) * (1.0 - p.powi(2))
    }

    fn is_proper(&self) -> bool {
        true // Quadratic is strictly proper
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Spherical scoring rule
///
/// Score = prediction / sqrt(prediction^2 + (1-prediction)^2) if outcome = 1
///       = (1-prediction) / sqrt(prediction^2 + (1-prediction)^2) if outcome = 0
pub struct SphericalScoringRule;

impl SphericalScoringRule {
    /// Create new spherical scoring rule
    pub fn new() -> Self {
        Self
    }
}

impl Default for SphericalScoringRule {
    fn default() -> Self {
        Self::new()
    }
}

impl ScoringRule for SphericalScoringRule {
    fn score(&self, prediction: f64, outcome: f64) -> f64 {
        let p = prediction.clamp(1e-10, 1.0 - 1e-10);
        let outcome_bin = if outcome >= 0.5 { 1.0 } else { 0.0 };
        let norm = (p.powi(2) + (1.0 - p).powi(2)).sqrt().max(1e-10);

        if outcome_bin == 1.0 {
            p / norm
        } else {
            (1.0 - p) / norm
        }
    }

    fn expected_score(&self, predicted_prob: f64, true_prob: f64) -> f64 {
        let p = predicted_prob.clamp(1e-10, 1.0 - 1e-10);
        let q = true_prob.clamp(0.0, 1.0);
        let norm = (p.powi(2) + (1.0 - p).powi(2)).sqrt().max(1e-10);

        (q * p + (1.0 - q) * (1.0 - p)) / norm
    }

    fn is_proper(&self) -> bool {
        true // Spherical is strictly proper
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Exponential scoring rule (survival analysis style)
pub struct ExponentialScoringRule {
    /// Temperature parameter
    beta: f64,
}

impl ExponentialScoringRule {
    /// Create new exponential scoring rule
    pub fn new(beta: f64) -> Self {
        Self { beta }
    }
}

impl ScoringRule for ExponentialScoringRule {
    fn score(&self, prediction: f64, outcome: f64) -> f64 {
        (-self.beta * (outcome - prediction).abs()).exp()
    }

    fn expected_score(&self, predicted_value: f64, true_mean: f64) -> f64 {
        // Simplified expectation
        (-self.beta * (predicted_value - true_mean).abs()).exp()
    }

    fn is_proper(&self) -> bool {
        false // Not proper, but useful for some applications
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Market scoring rule wrapper
///
/// Tracks cumulative scores for market participants
pub struct MarketScorer {
    /// Scoring rule to use
    rule: Box<dyn ScoringRule>,
    /// Cumulative scores by agent (deterministic BTreeMap order)
    scores: std::collections::BTreeMap<u64, f64>,
}

impl MarketScorer {
    /// Create new market scorer
    pub fn new(rule: Box<dyn ScoringRule>) -> Self {
        Self {
            rule,
            scores: std::collections::BTreeMap::new(),
        }
    }

    /// Record a prediction
    pub fn record_prediction(&mut self, agent_id: u64, prediction: f64, outcome: f64) {
        let score = self.rule.score(prediction, outcome);
        *self.scores.entry(agent_id).or_insert(0.0) += score;
    }

    /// Get cumulative score for agent
    pub fn get_score(&self, agent_id: u64) -> f64 {
        self.scores.get(&agent_id).copied().unwrap_or(0.0)
    }

    /// Get all scores
    pub fn all_scores(&self) -> &std::collections::BTreeMap<u64, f64> {
        &self.scores
    }

    /// Rank agents by score with deterministic tie-breaking (SIM-004)
    pub fn rankings(&self) -> Vec<(u64, f64)> {
        let mut rankings: Vec<_> = self
            .scores
            .iter()
            .map(|(&id, &score)| (id, score))
            .collect();

        rankings.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        rankings
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate market accuracy
    pub fn market_accuracy(&self, resolutions: &[Resolution]) -> f64 {
        if resolutions.is_empty() {
            return 0.0;
        }

        if self.scores.is_empty() {
            // Default uninformative prior baseline
            let total_error: f64 = resolutions
                .iter()
                .map(|r| (0.5 - r.actual_value).abs())
                .sum();
            return 1.0 - (total_error / resolutions.len() as f64);
        }

        // Aggregate calibrated logistic score across active forecasters
        let avg_score: f64 = self.scores.values().sum::<f64>() / self.scores.len() as f64;
        let accuracy = 1.0 / (1.0 + (-avg_score).exp());
        accuracy.clamp(0.0, 1.0)
    }
}

/// Utility functions for scoring
pub mod scoring_rules {
    use super::*;

    /// Brier score (mean squared error for binary outcomes)
    pub fn brier_score(predictions: &[f64], outcomes: &[f64]) -> f64 {
        if predictions.is_empty() || outcomes.is_empty() {
            return 0.0;
        }
        let n = predictions.len().min(outcomes.len()) as f64;
        predictions
            .iter()
            .zip(outcomes.iter())
            .map(|(&p, &o)| (p - o).powi(2))
            .sum::<f64>()
            / n
    }

    /// Logarithmic score
    pub fn log_score(predictions: &[f64], outcomes: &[f64]) -> f64 {
        if predictions.is_empty() || outcomes.is_empty() {
            return 0.0;
        }
        let n = predictions.len().min(outcomes.len()) as f64;
        let rule = LogarithmicScoringRule::new();
        predictions
            .iter()
            .zip(outcomes.iter())
            .map(|(&p, &o)| rule.score(p, o))
            .sum::<f64>()
            / n
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calibration metric
    pub fn calibration_error(predictions: &[f64], outcomes: &[f64], n_bins: usize) -> f64 {
        if predictions.is_empty() || outcomes.is_empty() || n_bins == 0 {
            return 0.0;
        }

        let mut bin_sums = vec![0.0; n_bins];
        let mut bin_counts = vec![0usize; n_bins];
        let mut bin_accuracies = vec![0.0; n_bins];

        for (&p, &o) in predictions.iter().zip(outcomes.iter()) {
            if !p.is_finite() || !o.is_finite() {
                continue;
            }
            let p_clamped = p.clamp(0.0, 0.999999);
            let bin = ((p_clamped * n_bins as f64) as usize).min(n_bins - 1);
            bin_sums[bin] += p.clamp(0.0, 1.0);
            bin_counts[bin] += 1;
            if o >= 0.5 {
                bin_accuracies[bin] += 1.0;
            }
        }

        let mut calibration_error = 0.0;
        let mut total_weight = 0;

        for i in 0..n_bins {
            if bin_counts[i] > 0 {
                let avg_prediction = bin_sums[i] / bin_counts[i] as f64;
                let avg_outcome = bin_accuracies[i] / bin_counts[i] as f64;
                calibration_error += bin_counts[i] as f64 * (avg_prediction - avg_outcome).abs();
                total_weight += bin_counts[i];
            }
        }

        if total_weight > 0 {
            calibration_error / total_weight as f64
        } else {
            0.0
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Resolution metric (discrimination)
    pub fn resolution(predictions: &[f64], outcomes: &[f64], n_bins: usize) -> f64 {
        if predictions.is_empty() || outcomes.is_empty() || n_bins == 0 {
            return 0.0;
        }

        let overall_rate =
            outcomes.iter().filter(|&&o| o >= 0.5).count() as f64 / outcomes.len() as f64;

        let mut bin_accuracies = vec![0.0; n_bins];
        let mut bin_counts = vec![0usize; n_bins];

        for (&p, &o) in predictions.iter().zip(outcomes.iter()) {
            let p_clamped = p.clamp(0.0, 0.999999);
            let bin = ((p_clamped * n_bins as f64) as usize).min(n_bins - 1);
            bin_counts[bin] += 1;
            if o >= 0.5 {
                bin_accuracies[bin] += 1.0;
            }
        }

        let mut resolution = 0.0;
        let mut total_weight = 0;

        for i in 0..n_bins {
            if bin_counts[i] > 0 {
                let bin_rate = bin_accuracies[i] / bin_counts[i] as f64;
                resolution += bin_counts[i] as f64 * (bin_rate - overall_rate).powi(2);
                total_weight += bin_counts[i];
            }
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        }

        if total_weight > 0 {
            resolution / total_weight as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logarithmic_scoring() {
        let rule = LogarithmicScoringRule::new();

        // Perfect prediction
        assert!(rule.score(1.0, 1.0) > rule.score(0.9, 1.0));

        // Wrong prediction
        assert!(rule.score(0.1, 1.0) < 0.0);

        // Check properness
        assert!(rule.is_proper());
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_quadratic_scoring() {
        let rule = QuadraticScoringRule::new();

        // Perfect prediction
        assert!((rule.score(1.0, 1.0) - 1.0).abs() < 0.001);

        // Complete miss
        assert!((rule.score(0.0, 1.0) - 0.0).abs() < 0.001);

        // 50/50 guess
        assert!((rule.score(0.5, 1.0) - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_brier_score() {
        let predictions = vec![0.9, 0.8, 0.3, 0.2];
        let outcomes = vec![1.0, 1.0, 0.0, 0.0];

        let score = scoring_rules::brier_score(&predictions, &outcomes);

        // Average of (0.1)^2 + (0.2)^2 + (0.3)^2 + (0.2)^2 = 0.18 / 4 = 0.045
        assert!((score - 0.045).abs() < 0.001);
    }

    #[test]
    fn test_market_scorer() {
        let mut scorer = MarketScorer::new(Box::new(QuadraticScoringRule::new()));

        scorer.record_prediction(1, 0.8, 1.0);
        scorer.record_prediction(1, 0.3, 0.0);
        scorer.record_prediction(2, 0.7, 1.0);

        assert!(scorer.get_score(1) > scorer.get_score(2));

        let rankings = scorer.rankings();
        assert_eq!(rankings[0].0, 1);
    }
}
