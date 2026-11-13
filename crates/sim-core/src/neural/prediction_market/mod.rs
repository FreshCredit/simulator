//! Neural Prediction Markets Module
//!
//! Implements prediction markets for DOCAS (Dopamine, Oxytocin, Cortisol,
//! Adrenaline, Serotonin) neuromodulator state forecasting.
//!
#![allow(missing_docs, dead_code)]
// TAG: surface=simulation owner=research-team rule=SIM-001
//! # Architecture
//!
//! - **Market**: Core prediction market structure with order book and AMM
//! - **OrderBook**: Limit order matching for price discovery
//! - **AMM**: Automated market maker for continuous liquidity
//! - **Oracle**: Brian2-based resolution mechanism
//! - **Contract**: Neural prediction contracts (binary, scalar, index)
//! - **Agent Integration**: Theory of Mind model integration
//!
//! # Example
//!
//! ```rust,no_run
//! use sim_core::neural::prediction_market::{Market, MarketId, DocasMetric};
//!
//! // Create market for dopamine prediction
//! let market = Market::new(
//!     MarketId(1),
//!     42,
//!     DocasMetric::Dopamine,
//!     11000,
//!     1000.0,
//! );
//! ```

pub mod agent_integration;
pub mod amm;
pub mod contract;
pub mod market;
pub mod oracle;
pub mod order_book;
pub mod scoring;

pub use agent_integration::{MarketParticipant, TheoryOfMind};
pub use amm::{ConstantProductAMM, AMM};
pub use contract::{Contract, ContractType, Portfolio, Position};
pub use market::{Market, MarketStatus};
pub use oracle::{NeuralOracle, OracleError, Resolution};
pub use order_book::{Order, OrderBook, OrderId, Side};
pub use scoring::{scoring_rules, LogarithmicScoringRule};

use crate::types::{AgentId, Tick};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Configuration for neural prediction markets
#[derive(Debug, Clone)]
pub struct PredictionMarketConfig {
    /// Initial liquidity for new markets
    pub initial_liquidity: f64,
    /// Minimum bet size
    pub min_bet: f64,
    /// Maximum bet size
    pub max_bet: f64,
    /// Market maker fee (percentage)
    pub maker_fee: f64,
    /// Market taker fee (percentage)
    pub taker_fee: f64,
    /// Oracle resolution delay (ticks after market close)
    pub resolution_delay: Tick,
    /// Maximum market duration
    pub max_duration: Tick,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for PredictionMarketConfig {
    fn default() -> Self {
        Self {
            initial_liquidity: 1000.0,
            min_bet: 1.0,
            max_bet: 10000.0,
            maker_fee: 0.001,
            taker_fee: 0.002,
            resolution_delay: 100,
            max_duration: 10000,
        }
    }
}

/// Metric for DOCAS prediction
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DocasMetric {
    /// Dopamine level
    Dopamine,
    /// Oxytocin level
    Oxytocin,
    /// Cortisol level
    Cortisol,
    /// Adrenaline level
    Adrenaline,
    /// Serotonin level
    Serotonin,
    /// Composite index (weighted combination)
    CompositeIndex,
}

impl DocasMetric {
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            DocasMetric::Dopamine => "Dopamine",
            DocasMetric::Oxytocin => "Oxytocin",
            DocasMetric::Cortisol => "Cortisol",
            DocasMetric::Adrenaline => "Adrenaline",
            DocasMetric::Serotonin => "Serotonin",
            DocasMetric::CompositeIndex => "DOCAS Index",
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get typical range for this metric
    pub fn typical_range(&self) -> (f64, f64) {
        match self {
            DocasMetric::Dopamine => (0.0, 1.0),
            DocasMetric::Oxytocin => (0.0, 1.0),
            DocasMetric::Cortisol => (0.0, 1.0),
            DocasMetric::Adrenaline => (0.0, 1.0),
            DocasMetric::Serotonin => (0.0, 1.0),
            DocasMetric::CompositeIndex => (0.0, 1.0),
        }
    }
}

/// Prediction market manager
pub struct PredictionMarketManager {
    markets: Vec<Market>,
    config: PredictionMarketConfig,
    next_market_id: u64,
}

impl PredictionMarketManager {
    /// Create new market manager
    pub fn new(config: PredictionMarketConfig) -> Self {
        Self {
            markets: Vec::new(),
            config,
            next_market_id: 1,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create a new prediction market
    pub fn create_market(
        &mut self,
        target_agent: AgentId,
        metric: DocasMetric,
        prediction_tick: Tick,
    ) -> MarketId {
        let market_id = MarketId(self.next_market_id);
        self.next_market_id += 1;

        let market = Market::new(
            market_id,
            target_agent,
            metric,
            prediction_tick,
            self.config.initial_liquidity,
        );

        self.markets.push(market);
        market_id
    }

    /// Get market by ID
    pub fn get_market(&self, id: MarketId) -> Option<&Market> {
        self.markets.iter().find(|m| m.id() == id)
    }

    /// Get mutable market by ID
    pub fn get_market_mut(&mut self, id: MarketId) -> Option<&mut Market> {
        self.markets.iter_mut().find(|m| m.id() == id)
    }

    /// Get all active markets
    pub fn active_markets(&self) -> Vec<&Market> {
        self.markets
            .iter()
            .filter(|m| m.status() == MarketStatus::Active)
            .collect()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update markets (call each tick)
    pub fn update(&mut self, current_tick: Tick) {
        for market in &mut self.markets {
            market.update(current_tick);
        }
    }

    /// Resolve expired markets
    pub fn resolve_markets(
        &mut self,
        oracle: &mut NeuralOracle,
        current_tick: Tick,
    ) -> Vec<Resolution> {
        let mut resolutions = Vec::new();

        for market in &mut self.markets {
            if market.can_resolve(current_tick) {
                match oracle.resolve(market) {
                    Ok(resolution) => {
                        market.settle(&resolution);
                        resolutions.push(resolution);
                    }
                    Err(e) => {
                        eprintln!("Failed to resolve market {}: {}", market.id().0, e);
                    }
                }
            }
        }

        resolutions
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Market identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MarketId(pub u64);

/// Errors in prediction market operations
#[derive(Debug)]
pub enum MarketError {
    /// Market not found
    NotFound(MarketId),
    /// Invalid order
    InvalidOrder(String),
    /// Insufficient funds
    InsufficientFunds { required: f64, available: f64 },
    /// Market closed
    MarketClosed(MarketId),
    /// Oracle error
    OracleError(String),
    /// Resolution failed
    ResolutionFailed(String),
}

impl std::fmt::Display for MarketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketError::NotFound(id) => write!(f, "Market {} not found", id.0),
            MarketError::InvalidOrder(msg) => write!(f, "Invalid order: {}", msg),
            MarketError::InsufficientFunds {
                required,
                available,
            } => {
                write!(
                    f,
                    "Insufficient funds: required {}, available {}",
                    required, available
                )
            }
            MarketError::MarketClosed(id) => write!(f, "Market {} is closed", id.0),
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            MarketError::OracleError(msg) => write!(f, "Oracle error: {}", msg),
            MarketError::ResolutionFailed(msg) => write!(f, "Resolution failed: {}", msg),
        }
    }
}

impl std::error::Error for MarketError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_creation() {
        let config = PredictionMarketConfig::default();
        let mut manager = PredictionMarketManager::new(config);

        let market_id = manager.create_market(1, DocasMetric::Dopamine, 1000);

        let market = manager.get_market(market_id).unwrap();
        assert_eq!(market.target_agent(), 1);
        assert_eq!(market.metric(), DocasMetric::Dopamine);
    }

    #[test]
    fn test_docas_metric_name() {
        assert_eq!(DocasMetric::Dopamine.name(), "Dopamine");
        assert_eq!(DocasMetric::Oxytocin.name(), "Oxytocin");
    }
}
