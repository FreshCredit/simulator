//! Prediction Market Core Implementation

use super::{
    ConstantProductAMM, Contract, ContractType, DocasMetric, MarketError, MarketId, OrderBook,
    Resolution, Side, AMM,
};
use crate::types::{AgentId, Tick};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Prediction market for DOCAS forecasting
pub struct Market {
    id: MarketId,
    target_agent: AgentId,
    metric: DocasMetric,
    prediction_tick: Tick,
    creation_tick: Tick,

    /// Order book for limit orders
    order_book: OrderBook,

    /// Automated market maker for liquidity
    amm: ConstantProductAMM,

    /// Market status
    status: MarketStatus,

    /// Total trading volume
    volume: f64,

    /// Outstanding contracts
    contracts: Vec<Contract>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Market {
    /// Create new prediction market
    pub fn new(
        id: MarketId,
        target_agent: AgentId,
        metric: DocasMetric,
        prediction_tick: Tick,
        initial_liquidity: f64,
    ) -> Self {
        Self {
            id,
            target_agent,
            metric,
            prediction_tick,
            creation_tick: 0, // Set by manager
            order_book: OrderBook::new(),
            amm: ConstantProductAMM::new(initial_liquidity),
            status: MarketStatus::Active,
            volume: 0.0,
            contracts: Vec::new(),
        }
    }

    /// Get market ID
    pub fn id(&self) -> MarketId {
        self.id
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get target agent
    pub fn target_agent(&self) -> AgentId {
        self.target_agent
    }

    /// Get predicted metric
    pub fn metric(&self) -> DocasMetric {
        self.metric
    }

    /// Get prediction tick
    pub fn prediction_tick(&self) -> Tick {
        self.prediction_tick
    }

    /// Get market status
    pub fn status(&self) -> MarketStatus {
        self.status
    }

    /// Get current order book
    pub fn order_book(&self) -> &OrderBook {
        &self.order_book
    }

    /// Get current AMM price
    pub fn current_price(&self) -> f64 {
        self.amm.current_price()
    }

    /// Get total volume
    pub fn volume(&self) -> f64 {
        self.volume
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    /// Place a limit order
    pub fn place_limit_order(
        &mut self,
        agent: AgentId,
        side: Side,
        price: f64,
        quantity: f64,
    ) -> Result<u64, MarketError> {
        if self.status != MarketStatus::Active {
            return Err(MarketError::MarketClosed(self.id));
        }

        if !price.is_finite() || price <= 0.0 || price > 1.0 {
            return Err(MarketError::InvalidOrder(
                "Price must be finite and in (0, 1]".to_string(),
            ));
        }

        if !quantity.is_finite() || quantity <= 0.0 {
            return Err(MarketError::InvalidOrder(
                "Quantity must be finite and positive".to_string(),
            ));
        }

        let order_id = self.order_book.place_order(agent, side, price, quantity);

        // Try to match orders, accumulate volume, and mint matched contract positions
        let fills = self.order_book.match_orders();
        for fill in fills {
            self.volume += fill.quantity * fill.price;
            let buyer_contract = Contract::new(
                self.id,
                fill.bid_agent,
                ContractType::Binary,
                super::Position::Long,
                fill.quantity,
                fill.price,
            );
            let seller_contract = Contract::new(
                self.id,
                fill.ask_agent,
                ContractType::Binary,
                super::Position::Short,
                fill.quantity,
                fill.price,
            );
            self.contracts.push(buyer_contract);
            self.contracts.push(seller_contract);
        }

        Ok(order_id)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Place a market order (executes against AMM)
    pub fn place_market_order(
        &mut self,
        agent: AgentId,
        side: Side,
        quantity: f64,
    ) -> Result<Contract, MarketError> {
        if self.status != MarketStatus::Active {
            return Err(MarketError::MarketClosed(self.id));
        }

        if !quantity.is_finite() || quantity <= 0.0 {
            return Err(MarketError::InvalidOrder(
                "Quantity must be finite and positive".to_string(),
            ));
        }

        // Execute against AMM: returns (executed_shares, total_cost)
        let (executed_qty, total_cost) = self.amm.execute_trade(side, quantity);

        if executed_qty <= 0.0 || !total_cost.is_finite() || total_cost <= 0.0 {
            return Err(MarketError::InvalidOrder(
                "Insufficient liquidity".to_string(),
            ));
        }

        self.volume += total_cost;
        let avg_price = (total_cost / executed_qty).clamp(0.001, 0.999);

        // Create contract
        let contract = Contract::new(
            self.id,
            agent,
            ContractType::Binary,
            side.into(),
            executed_qty,
            avg_price,
        );

        self.contracts.push(contract.clone());

        Ok(contract)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Cancel an order
    pub fn cancel_order(&mut self, order_id: u64) -> Result<(), MarketError> {
        self.order_book
            .cancel_order(order_id)
            .map_err(|_| MarketError::InvalidOrder("Order not found".to_string()))
    }

    /// Update market state (called each tick)
    pub fn update(&mut self, current_tick: Tick) {
        // Check if market should close
        if current_tick >= self.prediction_tick && self.status == MarketStatus::Active {
            self.status = MarketStatus::Closed;
        }
    }

    /// Check if market can be resolved
    pub fn can_resolve(&self, current_tick: Tick) -> bool {
        self.status == MarketStatus::Closed && current_tick >= self.prediction_tick + 100
        // Resolution delay
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Settle market with resolution
    pub fn settle(&mut self, resolution: &Resolution) {
        if self.status != MarketStatus::Closed {
            return;
        }

        // Pay out contracts
        for contract in &mut self.contracts {
            contract.settle(resolution);
        }

        self.status = MarketStatus::Resolved;
    }

    /// Get market depth (bid/ask volumes)
    pub fn market_depth(&self) -> (f64, f64) {
        self.order_book.depth()
    }

    /// Get open interest
    pub fn open_interest(&self) -> f64 {
        self.contracts.iter().map(|c| c.quantity()).sum()
    }
}

/// Market status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketStatus {
    /// Market is open for trading
    Active,
    /// Market is closed, awaiting resolution
    Closed,
    /// Market is resolved and settled
    Resolved,
    /// Market was cancelled
    Cancelled,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl std::fmt::Display for MarketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketStatus::Active => write!(f, "Active"),
            MarketStatus::Closed => write!(f, "Closed"),
            MarketStatus::Resolved => write!(f, "Resolved"),
            MarketStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::DocasMetric;
    use super::*;

    #[test]
    fn test_market_creation() {
        let market = Market::new(
            MarketId(1),
            42,
            DocasMetric::Dopamine,
            1000,
            1000.0,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        );

        assert_eq!(market.id().0, 1);
        assert_eq!(market.target_agent(), 42);
        assert_eq!(market.metric(), DocasMetric::Dopamine);
        assert_eq!(market.status(), MarketStatus::Active);
    }

    #[test]
    fn test_market_price_bounds() {
        let mut market = Market::new(MarketId(1), 42, DocasMetric::Dopamine, 1000, 1000.0);

        // Invalid price - too high
        let result = market.place_limit_order(1, Side::Buy, 1.5, 10.0);
        assert!(result.is_err());

        // Invalid price - zero
        let result = market.place_limit_order(1, Side::Buy, 0.0, 10.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_limit_order_fill_mints_contracts_and_settles() {
        let mut market = Market::new(MarketId(1), 42, DocasMetric::Dopamine, 100, 1000.0);

        let buy_res = market.place_limit_order(1, Side::Buy, 0.60, 10.0);
        assert!(buy_res.is_ok());

        let sell_res = market.place_limit_order(2, Side::Sell, 0.60, 10.0);
        assert!(sell_res.is_ok());

        assert_eq!(market.open_interest(), 20.0);
        assert_eq!(market.contracts.len(), 2);

        market.update(100);
        assert_eq!(market.status(), MarketStatus::Closed);

        let resolution = Resolution {
            market_id: MarketId(1),
            agent_id: 42,
            metric: DocasMetric::Dopamine,
            actual_value: 0.8,
            timestamp: 100,
        };
        market.settle(&resolution);
        assert_eq!(market.status(), MarketStatus::Resolved);

        assert_eq!(market.contracts[0].settlement_amount(), Some(10.0));
        assert_eq!(market.contracts[1].settlement_amount(), Some(0.0));
    }
}
