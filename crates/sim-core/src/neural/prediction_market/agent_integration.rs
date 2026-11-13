//! Agent Theory of Mind and Market Integration

use super::{DocasMetric, Market, Order, Side};
use crate::types::{AgentId, Tick};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Theory of Mind model for predicting other agents' DOCAS states
pub struct TheoryOfMind {
    /// Target agent being modeled
    target_agent: AgentId,

    /// Belief distribution over DOCAS metrics
    /// Key: (metric, quantile) -> belief probability
    beliefs: HashMap<(DocasMetric, u8), f64>,

    /// Model confidence (0.0 to 1.0)
    confidence: f64,

    /// Prediction accuracy history (exponential moving average)
    accuracy_ema: f64,

    /// Social cues observed
    cues: Vec<SocialCue>,

    /// Last update tick
    last_update: Tick,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl TheoryOfMind {
    /// Create new Theory of Mind model
    pub fn new(target_agent: AgentId) -> Self {
        Self {
            target_agent,
            beliefs: HashMap::new(),
            confidence: 0.5,
            accuracy_ema: 0.5,
            cues: Vec::new(),
            last_update: 0,
        }
    }

    /// Get target agent
    pub fn target_agent(&self) -> AgentId {
        self.target_agent
    }

    /// Get confidence
    pub fn confidence(&self) -> f64 {
        self.confidence
    }

    /// Get predicted value for metric
    pub fn predict(&self, metric: DocasMetric) -> f64 {
        // Return mean of belief distribution
        let mut sum = 0.0;
        let mut count = 0;

        for quantile in 0..=100u8 {
            if let Some(&belief) = self.beliefs.get(&(metric, quantile)) {
                sum += quantile as f64 * belief;
                count += 1;
            }
        }

        if count > 0 {
            sum / (count as f64 * 100.0)
        } else {
            0.5 // Default to neutral
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get prediction variance
    pub fn prediction_variance(&self, metric: DocasMetric) -> f64 {
        let mean = self.predict(metric);
        let mut variance = 0.0;
        let mut total_prob = 0.0;

        for quantile in 0..=100u8 {
            if let Some(&prob) = self.beliefs.get(&(metric, quantile)) {
                let value = quantile as f64 / 100.0;
                variance += prob * (value - mean).powi(2);
                total_prob += prob;
            }
        }

        if total_prob > 0.0 {
            variance / total_prob
        } else {
            0.25 // Maximum variance (uniform)
        }
    }

    /// Update beliefs based on social cue
    pub fn observe_cue(&mut self, cue: SocialCue, current_tick: Tick) {
        self.cues.push(cue.clone());
        self.last_update = current_tick;

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Bayesian update of beliefs based on cue
        match cue {
            SocialCue::Encounter {
                outcome,
                docas_delta,
            } => {
                // Update beliefs based on social encounter outcome
                self.update_from_encounter(outcome, docas_delta);
            }
            SocialCue::ReputationChange { new_reputation } => {
                // Reputation changes affect oxytocin/cortisol
                self.update_from_reputation(new_reputation);
            }
            SocialCue::Transaction { amount, success } => {
                // Economic transactions affect dopamine
                self.update_from_transaction(amount, success);
            }
            SocialCue::Gossip {
                about_agent,
                sentiment,
            } => {
                // Gossip provides information about social dynamics
                if about_agent == self.target_agent {
                    self.update_from_direct_gossip(sentiment);
                }
            }
        }

        // Update confidence based on cue reliability
        self.confidence = (self.confidence + cue.reliability()) / 2.0;
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update beliefs from social encounter
    fn update_from_encounter(&mut self, outcome: EncounterOutcome, _docas_delta: [f64; 5]) {
        // Positive encounters increase oxytocin
        // Negative encounters increase cortisol/adrenaline
        match outcome {
            EncounterOutcome::Cooperate => {
                // Increase oxytocin belief
                self.shift_belief(DocasMetric::Oxytocin, 0.1);
            }
            EncounterOutcome::Defect => {
                // Increase cortisol belief
                self.shift_belief(DocasMetric::Cortisol, 0.1);
                self.shift_belief(DocasMetric::Adrenaline, 0.05);
            }
            EncounterOutcome::Reciprocate => {
                // Increase dopamine (reward)
                self.shift_belief(DocasMetric::Dopamine, 0.1);
            }
        }
    }

    /// Update beliefs from reputation change
    fn update_from_reputation(&mut self, reputation: f64) {
        // Higher reputation -> more oxytocin
        // Lower reputation -> more cortisol
        if reputation > 0.5 {
            self.shift_belief(DocasMetric::Oxytocin, (reputation - 0.5) * 0.2);
        } else {
            self.shift_belief(DocasMetric::Cortisol, (0.5 - reputation) * 0.2);
        }
    }

    /// Update beliefs from transaction
    fn update_from_transaction(&mut self, amount: f64, success: bool) {
        if success {
            // Successful transactions increase dopamine
            self.shift_belief(DocasMetric::Dopamine, amount.min(0.1));
        } else {
            // Failed transactions increase cortisol
            self.shift_belief(DocasMetric::Cortisol, 0.05);
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update from gossip about target
    fn update_from_direct_gossip(&mut self, sentiment: f64) {
        // Positive sentiment -> more oxytocin expected
        // Negative sentiment -> more cortisol expected
        self.shift_belief(DocasMetric::Oxytocin, sentiment * 0.1);
        self.shift_belief(DocasMetric::Cortisol, -sentiment * 0.1);
    }

    /// Shift belief distribution
    fn shift_belief(&mut self, metric: DocasMetric, delta: f64) {
        let has_metric = self.beliefs.keys().any(|(m, _)| *m == metric);
        if !has_metric {
            self.beliefs.insert((metric, 50), 1.0);
        }

        // Simple shift: move probability mass in direction of delta
        let new_beliefs: HashMap<(DocasMetric, u8), f64> = self
            .beliefs
            .iter()
            .filter(|((m, _), _)| *m == metric)
            .map(|((m, q), prob)| {
                let shift = (delta * 100.0) as i32;
                let new_q = (*q as i32 + shift).clamp(0, 100) as u8;
                ((*m, new_q), *prob)
            })
            .collect();

        self.beliefs.retain(|(m, _), _| *m != metric);
        self.beliefs.extend(new_beliefs);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update accuracy from actual resolution
    pub fn update_accuracy(&mut self, metric: DocasMetric, actual: f64) {
        let predicted = self.predict(metric);
        let error = (predicted - actual).abs();

        // Exponential moving average of accuracy (1 - error)
        let alpha = 0.3; // Update rate
        let new_accuracy = 1.0 - error;
        self.accuracy_ema = alpha * new_accuracy + (1.0 - alpha) * self.accuracy_ema;
    }

    /// Get expected profit from participating in market
    pub fn expected_profit(&self, market: &Market) -> f64 {
        if market.status() != super::MarketStatus::Active {
            return 0.0;
        }

        let predicted = self.predict(market.metric());
        let market_price = market.current_price();

        // Expected value of contract
        let expected_value = predicted;

        // Profit if buying
        let buy_profit = (expected_value - market_price).max(0.0);

        // Profit if selling
        let sell_profit = (market_price - expected_value).max(0.0);

        // Return maximum expected profit
        buy_profit.max(sell_profit) * self.confidence * self.accuracy_ema
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Social cue types
#[derive(Debug, Clone)]
pub enum SocialCue {
    /// Social encounter outcome
    Encounter {
        outcome: EncounterOutcome,
        docas_delta: [f64; 5],
    },
    /// Reputation change
    ReputationChange { new_reputation: f64 },
    /// Economic transaction
    Transaction { amount: f64, success: bool },
    /// Gossip about another agent
    Gossip {
        about_agent: AgentId,
        sentiment: f64, // -1.0 to 1.0
    },
}

impl SocialCue {
    /// Get reliability of this cue
    pub fn reliability(&self) -> f64 {
        match self {
            // Direct observations are most reliable
            SocialCue::Encounter { .. } => 0.9,
            SocialCue::ReputationChange { .. } => 0.8,
            SocialCue::Transaction { .. } => 0.85,
            // Gossip is less reliable
            SocialCue::Gossip { .. } => 0.5,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Encounter outcomes
#[derive(Debug, Clone, Copy)]
pub enum EncounterOutcome {
    Cooperate,
    Defect,
    Reciprocate,
}

/// Market participant trait for agents
pub trait MarketParticipant {
    /// Generate bid based on Theory of Mind
    fn generate_bid(&self, market: &Market, tom: &TheoryOfMind) -> Option<MarketBid>;

    /// Decide whether to participate in market
    fn should_participate(&self, market: &Market, tom: &TheoryOfMind) -> bool;

    /// Get portfolio of current positions
    fn get_portfolio(&self) -> &super::Portfolio;
}

/// Market bid
#[derive(Debug, Clone)]
pub struct MarketBid {
    /// Side (buy/sell)
    pub side: Side,
    /// Price limit
    pub price: f64,
    /// Quantity
    pub quantity: f64,
    /// Confidence in prediction
    pub confidence: f64,
}

impl MarketBid {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create new bid
    pub fn new(side: Side, price: f64, quantity: f64, confidence: f64) -> Self {
        Self {
            side,
            price,
            quantity,
            confidence,
        }
    }

    /// Convert to order
    pub fn to_order(&self, id: u64, agent_id: AgentId) -> Order {
        Order::new(id, agent_id, self.side, self.price, self.quantity)
    }
}

/// Simple market participant implementation
pub struct SimpleMarketParticipant {
    agent_id: AgentId,
    portfolio: super::Portfolio,
    risk_tolerance: f64,
    min_confidence: f64,
}

impl SimpleMarketParticipant {
    /// Create new participant
    pub fn new(agent_id: AgentId, risk_tolerance: f64) -> Self {
        Self {
            agent_id,
            portfolio: super::Portfolio::new(agent_id),
            risk_tolerance,
            min_confidence: 0.6,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Set minimum confidence threshold
    pub fn set_min_confidence(&mut self, threshold: f64) {
        self.min_confidence = threshold;
    }
}

impl MarketParticipant for SimpleMarketParticipant {
    fn generate_bid(&self, market: &Market, tom: &TheoryOfMind) -> Option<MarketBid> {
        if tom.confidence() < self.min_confidence {
            return None;
        }

        let predicted = tom.predict(market.metric());
        let market_price = market.current_price();

        // Determine side and price
        let (side, price) = if predicted > market_price + 0.05 {
            // Buy if prediction > market price + margin
            (Side::Buy, market_price)
        } else if predicted < market_price - 0.05 {
            // Sell if prediction < market price - margin
            (Side::Sell, market_price)
        } else {
            return None; // No edge
        };

        // Quantity based on confidence and risk tolerance
        let quantity = tom.confidence() * self.risk_tolerance * 100.0;

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        Some(MarketBid::new(side, price, quantity, tom.confidence()))
    }

    fn should_participate(&self, market: &Market, tom: &TheoryOfMind) -> bool {
        let expected_profit = tom.expected_profit(market);
        expected_profit > 0.1 // Threshold for participation
    }

    fn get_portfolio(&self) -> &super::Portfolio {
        &self.portfolio
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theory_of_mind_creation() {
        let tom = TheoryOfMind::new(42);
        assert_eq!(tom.target_agent(), 42);
        assert_eq!(tom.confidence(), 0.5);
    }

    #[test]
    fn test_theory_of_mind_prediction() {
        let mut tom = TheoryOfMind::new(42);

        // Initially returns neutral
        assert_eq!(tom.predict(DocasMetric::Dopamine), 0.5);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Add some beliefs
        tom.observe_cue(
            SocialCue::Encounter {
                outcome: EncounterOutcome::Cooperate,
                docas_delta: [0.1, 0.2, 0.0, 0.0, 0.0],
            },
            100,
        );

        // Prediction should shift
        assert!(tom.predict(DocasMetric::Oxytocin) > 0.5);
    }

    #[test]
    fn test_social_cue_reliability() {
        let encounter = SocialCue::Encounter {
            outcome: EncounterOutcome::Cooperate,
            docas_delta: [0.0; 5],
        };
        assert_eq!(encounter.reliability(), 0.9);

        let gossip = SocialCue::Gossip {
            about_agent: 1,
            sentiment: 0.5,
        };
        assert_eq!(gossip.reliability(), 0.5);
    }

    #[test]
    fn test_market_bid_creation() {
        let bid = MarketBid::new(Side::Buy, 0.6, 100.0, 0.8);

        assert_eq!(bid.side, Side::Buy);
        assert_eq!(bid.price, 0.6);
        assert_eq!(bid.quantity, 100.0);
        assert_eq!(bid.confidence, 0.8);
    }
}
