//! Market-Based Social Influence (P2-RES-1)
//!
//! Information cascades and opinion markets.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Market configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpinionMarketConfig {
    /// initial liquidity.
    pub initial_liquidity: f32,
    /// trading fee.
    pub trading_fee: f32,
    /// price sensitivity.
    pub price_sensitivity: f32,
}

impl Default for OpinionMarketConfig {
    fn default() -> Self {
        Self {
            initial_liquidity: 1000.0,
            trading_fee: 0.01,
            price_sensitivity: 0.1,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Opinion market for predicting outcomes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpinionMarket {
    /// market id.
    pub market_id: String,
    /// topic.
    pub topic: String,
    /// yes price.
    pub yes_price: f32, // Probability of "yes" outcome
    /// no price.
    pub no_price: f32, // Probability of "no" outcome
    /// yes volume.
    pub yes_volume: f32,
    /// no volume.
    pub no_volume: f32,
    /// trades.
    pub trades: Vec<Trade>,
}

/// Individual trade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// trader.
    pub trader: u64,
    /// direction.
    pub direction: TradeDirection,
    /// amount.
    pub amount: f32,
    /// price.
    pub price: f32,
    /// timestamp.
    pub timestamp: u64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// trade direction.
pub enum TradeDirection {
    /// yes.
    Yes,
    /// no.
    No,
}

/// Market maker for opinion markets
#[derive(Debug, Clone)]
pub struct MarketMaker {
    /// config.
    pub config: OpinionMarketConfig,
    /// markets.
    pub markets: HashMap<String, OpinionMarket>,
}

impl MarketMaker {
    #[must_use]
    /// new.
    pub fn new(config: OpinionMarketConfig) -> Self {
        Self {
            config,
            markets: HashMap::new(),
        }
    }

    /// Create new market
    pub fn create_market(&mut self, topic: String) -> String {
        let id = format!("market_{}", self.markets.len());
        let market = OpinionMarket {
            market_id: id.clone(),
            topic,
            yes_price: 0.5,
            no_price: 0.5,
            yes_volume: self.config.initial_liquidity,
            no_volume: self.config.initial_liquidity,
            trades: Vec::new(),
        };
        self.markets.insert(id.clone(), market);
        id
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Execute trade
    pub fn trade(&mut self, market_id: &str, trade: Trade) -> Option<f32> {
        let market = self.markets.get_mut(market_id)?;

        // Update volumes and calculate new prices using numerically stable logistic difference
        let (yes_price, no_price) = match trade.direction {
            TradeDirection::Yes => {
                let new_yes_volume = market.yes_volume + trade.amount;
                let diff = (new_yes_volume - market.no_volume) * self.config.price_sensitivity;
                // Logistic sigmoid: 1 / (1 + exp(-diff))
                let yes_p = 1.0 / (1.0 + (-diff).exp());
                market.yes_volume = new_yes_volume;
                (yes_p, 1.0 - yes_p)
            }
            TradeDirection::No => {
                let new_no_volume = market.no_volume + trade.amount;
                let diff = (new_no_volume - market.yes_volume) * self.config.price_sensitivity;
                let no_p = 1.0 / (1.0 + (-diff).exp());
                market.no_volume = new_no_volume;
                (1.0 - no_p, no_p)
            }
        };

        market.yes_price = yes_price;
        market.no_price = no_price;
        market.trades.push(trade);
        Some(yes_price)
    }

    /// Get market price as prediction
    #[must_use]
    pub fn get_prediction(&self, market_id: &str) -> Option<f32> {
        self.markets.get(market_id).map(|m| m.yes_price)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Detect information cascade
    #[must_use]
    pub fn detect_cascade(&self, market_id: &str, window: usize) -> Option<CascadeDetection> {
        let market = self.markets.get(market_id)?;
        if market.trades.len() < window {
            return None;
        }

        let recent: Vec<&Trade> = market.trades.iter().rev().take(window).collect();

        // Count consecutive same-direction trades
        let yes_count = recent
            .iter()
            .filter(|t| matches!(t.direction, TradeDirection::Yes))
            .count();
        let no_count = recent
            .iter()
            .filter(|t| matches!(t.direction, TradeDirection::No))
            .count();

        let cascade_strength = (yes_count.max(no_count) as f32) / (window as f32);
        let cascade_direction = if yes_count > no_count {
            TradeDirection::Yes
        } else {
            TradeDirection::No
        };

        Some(CascadeDetection {
            is_cascade: cascade_strength > 0.7,
            strength: cascade_strength,
            direction: cascade_direction,
            trade_count: window,
        })
    }
}

impl Default for MarketMaker {
    fn default() -> Self {
        Self::new(OpinionMarketConfig::default())
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Cascade detection result
#[derive(Debug, Clone)]
pub struct CascadeDetection {
    /// is cascade.
    pub is_cascade: bool,
    /// strength.
    pub strength: f32,
    /// direction.
    pub direction: TradeDirection,
    /// trade count.
    pub trade_count: usize,
}

/// Information cascade model (Banerjee, Bikhchandani et al.)
#[derive(Debug, Clone)]
pub struct CascadeModel {
    /// prior probability.
    pub prior_probability: f32,
    /// signal accuracy.
    pub signal_accuracy: f32,
}

impl CascadeModel {
    #[must_use]
    /// new.
    pub const fn new(prior: f32, accuracy: f32) -> Self {
        Self {
            prior_probability: prior,
            signal_accuracy: accuracy,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Predict cascade formation based on signal accuracy
    #[must_use]
    pub fn cascade_probability(&self, n_agents: usize) -> f32 {
        // As signal accuracy increases, cascades form more readily
        // but are also more accurate
        let base = self.signal_accuracy * self.signal_accuracy;
        let network_effect = 1.0 - (-0.1 * n_agents as f32).exp();
        base * network_effect
    }

    /// Expected accuracy of cascaded decisions
    #[must_use]
    pub fn cascade_accuracy(&self) -> f32 {
        let p = self.prior_probability.clamp(0.001, 0.999);
        let q = self.signal_accuracy.clamp(0.001, 0.999);
        // Under Bikhchandani, Hirshleifer & Welch (1992), informative cascades match true state with signal precision q
        let p_high_signal = p * q + (1.0 - p) * (1.0 - q);
        let p_low_signal = 1.0 - p_high_signal;
        let p_correct_given_high = (p * q) / p_high_signal;
        let p_correct_given_low = ((1.0 - p) * q) / p_low_signal;
        (p_high_signal * p_correct_given_high + p_low_signal * p_correct_given_low).clamp(0.0, 1.0)
    }
}

impl Default for CascadeModel {
    fn default() -> Self {
        Self::new(0.5, 0.6)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_creation() {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let mut maker = MarketMaker::default();
        let id = maker.create_market("Will it rain?".to_string());

        assert!(maker.markets.contains_key(&id));
        assert_eq!(maker.get_prediction(&id), Some(0.5));
    }

    #[test]
    fn test_trade_updates_price() {
        let mut maker = MarketMaker::new(OpinionMarketConfig {
            initial_liquidity: 100.0, // Smaller liquidity for visible price movement
            trading_fee: 0.01,
            price_sensitivity: 0.1,
        });
        let id = maker.create_market("Test".to_string());

        maker.trade(
            &id,
            Trade {
                trader: 1,
                direction: TradeDirection::Yes,
                amount: 50.0, // Significant relative to liquidity
                price: 0.5,
                timestamp: 0,
            },
        );

        let price = maker.get_prediction(&id).unwrap();
        // Price should move up after Yes trade
        assert!(price > 0.5, "Expected price > 0.5, got {price}");
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_cascade_detection() {
        let mut maker = MarketMaker::default();
        let id = maker.create_market("Test".to_string());

        // Add 10 consecutive Yes trades
        for i in 0..10 {
            maker.trade(
                &id,
                Trade {
                    trader: i as u64,
                    direction: TradeDirection::Yes,
                    amount: 50.0,
                    price: 0.5,
                    timestamp: i as u64,
                },
            );
        }

        let detection = maker.detect_cascade(&id, 10).unwrap();
        assert!(detection.is_cascade);
        assert!(matches!(detection.direction, TradeDirection::Yes));
    }

    #[test]
    fn test_cascade_model() {
        let model = CascadeModel::new(0.5, 0.7);

        let prob = model.cascade_probability(20);
        assert!(prob > 0.0 && prob <= 1.0);

        let acc = model.cascade_accuracy();
        // With prior 0.5 and accuracy 0.7: 0.5 * 0.7 + 0.5 * 0.3 = 0.5
        // Should be at least 0.5
        assert!(acc >= 0.5);
    }
}
