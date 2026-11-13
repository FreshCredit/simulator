//! Order Book Implementation

use crate::types::AgentId;
use std::collections::BTreeMap;

/// Order identifier
pub type OrderId = u64;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Order side
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

impl From<Side> for super::Position {
    fn from(side: Side) -> Self {
        match side {
            Side::Buy => super::Position::Long,
            Side::Sell => super::Position::Short,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Limit order
#[derive(Debug, Clone)]
pub struct Order {
    pub id: OrderId,
    pub agent: AgentId,
    pub side: Side,
    pub price: f64,
    pub quantity: f64,
    pub filled: f64,
}

impl Order {
    /// Create new order
    pub fn new(id: OrderId, agent: AgentId, side: Side, price: f64, quantity: f64) -> Self {
        Self {
            id,
            agent,
            side,
            price,
            quantity,
            filled: 0.0,
        }
    }

    /// Check if order is completely filled
    pub fn is_filled(&self) -> bool {
        self.filled >= self.quantity
    }

    /// Get remaining quantity
    pub fn remaining(&self) -> f64 {
        self.quantity - self.filled
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Fill part of the order
    pub fn fill(&mut self, amount: f64) {
        self.filled += amount;
        self.filled = self.filled.min(self.quantity);
    }
}

/// Order book for limit orders
pub struct OrderBook {
    /// Buy orders (bids) - sorted by price descending
    bids: BTreeMap<u64, Vec<Order>>,
    /// Sell orders (asks) - sorted by price ascending
    asks: BTreeMap<u64, Vec<Order>>,
    /// Next order ID
    next_id: OrderId,
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderBook {
    /// Create new empty order book
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            next_id: 1,
        }
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Place a new order
    pub fn place_order(
        &mut self,
        agent: AgentId,
        side: Side,
        price: f64,
        quantity: f64,
    ) -> OrderId {
        if !price.is_finite() || price <= 0.0 || !quantity.is_finite() || quantity <= 0.0 {
            return 0;
        }

        let id = self.next_id;
        self.next_id += 1;

        let order = Order::new(id, agent, side, price, quantity);

        // Quantize price for BTreeMap key (multiply by 10000 for 4 decimal precision)
        let price_key = (price.max(0.0) * 10000.0).round() as u64;

        match side {
            Side::Buy => {
                self.bids.entry(price_key).or_default().push(order);
            }
            Side::Sell => {
                self.asks.entry(price_key).or_default().push(order);
            }
        }

        id
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Cancel an order
    #[allow(clippy::result_unit_err)]
    pub fn cancel_order(&mut self, order_id: OrderId) -> Result<(), ()> {
        let mut empty_bid_key = None;
        for (&price_key, orders) in self.bids.iter_mut() {
            if let Some(pos) = orders.iter().position(|o| o.id == order_id) {
                orders.remove(pos);
                if orders.is_empty() {
                    empty_bid_key = Some(price_key);
                }
                break;
            }
        }
        if let Some(k) = empty_bid_key {
            self.bids.remove(&k);
            return Ok(());
        }

        let mut empty_ask_key = None;
        for (&price_key, orders) in self.asks.iter_mut() {
            if let Some(pos) = orders.iter().position(|o| o.id == order_id) {
                orders.remove(pos);
                if orders.is_empty() {
                    empty_ask_key = Some(price_key);
                }
                break;
            }
        }
        if let Some(k) = empty_ask_key {
            self.asks.remove(&k);
            return Ok(());
        }

        Err(())
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Match orders
    pub fn match_orders(&mut self) -> Vec<Fill> {
        let mut fills = Vec::new();

        loop {
            // Get best bid and ask
            let best_bid = self.bids.keys().next_back().copied();
            let best_ask = self.asks.keys().next().copied();

            if let (Some(bid_key), Some(ask_key)) = (best_bid, best_ask) {
                let bid_price = bid_key as f64 / 10000.0;
                let ask_price = ask_key as f64 / 10000.0;

                // Check if orders cross
                if bid_price >= ask_price {
                    // Get mutable references to order lists
                    let (Some(bid_orders), Some(ask_orders)) =
                        (self.bids.get_mut(&bid_key), self.asks.get_mut(&ask_key))
                    else {
                        break;
                    };

                    // Match first orders
                    if let (Some(bid), Some(ask)) = (bid_orders.first_mut(), ask_orders.first_mut())
                    {
                        // Prevent self-trading
                        if bid.agent == ask.agent {
                            if bid.id > ask.id {
                                bid_orders.remove(0);
                            } else {
                                ask_orders.remove(0);
                            }
                            if bid_orders.is_empty() {
                                self.bids.remove(&bid_key);
                            }
                            if ask_orders.is_empty() {
                                self.asks.remove(&ask_key);
                            }
                            continue;
                        }

                        let fill_qty = bid.remaining().min(ask.remaining());
                        if fill_qty <= 1e-9 || !fill_qty.is_finite() {
                            if bid.is_filled() || bid.remaining() <= 1e-9 {
                                bid_orders.remove(0);
                            }
                            if ask.is_filled() || ask.remaining() <= 1e-9 {
                                ask_orders.remove(0);
                            }
                            if bid_orders.is_empty() {
                                self.bids.remove(&bid_key);
                            }
                            if ask_orders.is_empty() {
                                self.asks.remove(&ask_key);
                            }
                            continue;
                        }

                        let fill_price = (bid_price + ask_price) / 2.0;

                        bid.fill(fill_qty);
                        ask.fill(fill_qty);

                        fills.push(Fill {
                            bid_id: bid.id,
                            ask_id: ask.id,
                            bid_agent: bid.agent,
                            ask_agent: ask.agent,
                            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                            price: fill_price,
                            quantity: fill_qty,
                        });

                        // Remove filled orders
                        if bid.is_filled() {
                            bid_orders.remove(0);
                        }
                        if ask.is_filled() {
                            ask_orders.remove(0);
                        }

                        // Clean up empty price levels
                        if bid_orders.is_empty() {
                            self.bids.remove(&bid_key);
                        }
                        if ask_orders.is_empty() {
                            self.asks.remove(&ask_key);
                        }
                    }
                } else {
                    // No more matches possible
                    break;
                }
            } else {
                // No orders on one side
                break;
            }
        }

        fills
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get best bid price
    pub fn best_bid(&self) -> Option<f64> {
        self.bids.keys().next_back().map(|&k| k as f64 / 10000.0)
    }

    /// Get best ask price
    pub fn best_ask(&self) -> Option<f64> {
        self.asks.keys().next().map(|&k| k as f64 / 10000.0)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get mid price
    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid + ask) / 2.0),
            (Some(bid), None) => Some(bid),
            (None, Some(ask)) => Some(ask),
            (None, None) => None,
        }
    }

    /// Get market depth
    pub fn depth(&self) -> (f64, f64) {
        let bid_depth: f64 = self
            .bids
            .values()
            .flat_map(|orders| orders.iter().map(|o| o.remaining()))
            .sum();

        let ask_depth: f64 = self
            .asks
            .values()
            .flat_map(|orders| orders.iter().map(|o| o.remaining()))
            .sum();

        (bid_depth, ask_depth)
    }

    /// Get all orders
    pub fn all_orders(&self) -> Vec<&Order> {
        let mut orders = Vec::new();

        for order_list in self.bids.values() {
            orders.extend(order_list.iter());
        }
        for order_list in self.asks.values() {
            orders.extend(order_list.iter());
        }

        orders
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Fill record
#[derive(Debug, Clone)]
pub struct Fill {
    pub bid_id: OrderId,
    pub ask_id: OrderId,
    pub bid_agent: AgentId,
    pub ask_agent: AgentId,
    pub price: f64,
    pub quantity: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_place_order() {
        let mut book = OrderBook::new();
        let id = book.place_order(1, Side::Buy, 0.6, 100.0);

        assert_eq!(id, 1);
        assert_eq!(book.best_bid(), Some(0.6));
    }

    #[test]
    fn test_match_orders() {
        let mut book = OrderBook::new();

        // Place bid and ask that cross
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        book.place_order(1, Side::Buy, 0.7, 100.0);
        book.place_order(2, Side::Sell, 0.6, 50.0);

        let fills = book.match_orders();

        assert_eq!(fills.len(), 1);
        assert_eq!(fills[0].quantity, 50.0);
        assert!((fills[0].price - 0.65).abs() < 0.01);
    }

    #[test]
    fn test_no_match_when_orders_dont_cross() {
        let mut book = OrderBook::new();

        book.place_order(1, Side::Buy, 0.5, 100.0);
        book.place_order(2, Side::Sell, 0.7, 50.0);

        let fills = book.match_orders();

        assert_eq!(fills.len(), 0);
    }

    #[test]
    fn test_cancel_order() {
        let mut book = OrderBook::new();
        let id = book.place_order(1, Side::Buy, 0.6, 100.0);

        assert!(book.cancel_order(id).is_ok());
        assert!(book.best_bid().is_none());
    }

    #[test]
    fn test_self_trade_prevention_and_invalid_order_rejection() {
        let mut book = OrderBook::new();

        // 1. Invalid order rejected
        let invalid_id = book.place_order(1, Side::Buy, 0.6, 0.0);
        assert_eq!(invalid_id, 0);
        let nan_id = book.place_order(1, Side::Buy, f64::NAN, 10.0);
        assert_eq!(nan_id, 0);

        // 2. Self trade prevention: agent 1 placing crossing orders
        book.place_order(1, Side::Buy, 0.7, 50.0);
        book.place_order(1, Side::Sell, 0.6, 50.0);
        let fills = book.match_orders();
        assert!(fills.is_empty(), "Self-trades must be prevented");
    }
}
