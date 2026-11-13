// TAG: surface=simulation owner=research-team rule=SIM-001
//! ABM Financial Markets and Order Book simulation

use serde::{Deserialize, Serialize};

// P2-4: ABM FINANCIAL MARKETS
// Agent-based market simulation with order books
// ============================================================================

/// Order type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    /// Market order - execute immediately at best price
    Market,
    /// Limit order - execute at specified price or better
    Limit,
}

/// Order side
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    /// Buy order
    Buy,
    /// Sell order
    Sell,
}

/// Trading order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// Order ID
    pub id: u64,
    /// Agent who placed order
    pub agent_id: u64,
    /// Order type
    pub order_type: OrderType,
    /// Buy or sell
    // TAG: surface=simulation owner=research-team rule=SIM-001
    pub side: OrderSide,
    /// Price (for limit orders)
    pub price: f32,
    /// Quantity
    pub quantity: u32,
    /// Timestamp
    pub timestamp: u64,
}

/// Trade execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// Trade ID
    pub id: u64,
    /// Buyer agent ID
    pub buyer_id: u64,
    /// Seller agent ID
    pub seller_id: u64,
    /// Trade price
    pub price: f32,
    /// Trade quantity
    pub quantity: u32,
    /// Timestamp
    pub timestamp: u64,
    // TAG: surface=simulation owner=research-team rule=SIM-001
    // TAG: surface=simulation owner=research-team rule=SIM-001
}

/// Order book for a single asset
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrderBook {
    /// Buy orders (bids), sorted by price descending
    pub bids: Vec<Order>,
    /// Sell orders (asks), sorted by price ascending
    pub asks: Vec<Order>,
    /// Trade history
    pub trades: Vec<Trade>,
    /// Next order ID
    next_order_id: u64,
    /// Next trade ID
    next_trade_id: u64,
}

impl OrderBook {
    /// Create new empty order book
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get best bid price
    #[must_use]
    // TAG: surface=simulation owner=research-team rule=SIM-001
    pub fn best_bid(&self) -> Option<f32> {
        self.bids
            .iter()
            .map(|o| o.price)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Get best ask price
    #[must_use]
    pub fn best_ask(&self) -> Option<f32> {
        self.asks
            .iter()
            .map(|o| o.price)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Get mid price
    #[must_use]
    pub fn mid_price(&self) -> Option<f32> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid + ask) / 2.0),
            (Some(bid), None) => Some(bid),
            (None, Some(ask)) => Some(ask),
            (None, None) => None,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get spread
    #[must_use]
    pub fn spread(&self) -> Option<f32> {
        match (self.best_ask(), self.best_bid()) {
            (Some(ask), Some(bid)) => Some(ask - bid),
            _ => None,
        }
    }

    /// Place an order
    pub fn place_order(&mut self, mut order: Order, timestamp: u64) -> Vec<Trade> {
        order.id = self.next_order_id;
        order.timestamp = timestamp;
        self.next_order_id += 1;

        // P1 FIX: Pre-allocate trades vector with estimated capacity
        let mut trades = Vec::with_capacity(4);

        match order.order_type {
            OrderType::Market => {
                // Execute immediately
                trades.extend(self.execute_market_order(&order));
            }
            OrderType::Limit => {
                // Try to match, add to book if not fully filled
                let (remaining, limit_trades) = self.match_limit_order(&order);
                trades.extend(limit_trades);
                // TAG: surface=simulation owner=research-team rule=SIM-001
                if remaining.quantity > 0 {
                    self.add_to_book(remaining);
                }
            }
        }

        trades
    }

    /// Execute market order
    fn execute_market_order(&mut self, order: &Order) -> Vec<Trade> {
        let estimated_trades = self.asks.len().min(self.bids.len()).max(1);
        let mut trades = Vec::with_capacity(estimated_trades.min(16));
        let mut remaining_qty = order.quantity;

        match order.side {
            OrderSide::Buy => self.match_against_asks(order, &mut trades, &mut remaining_qty),
            OrderSide::Sell => self.match_against_bids(order, &mut trades, &mut remaining_qty),
        }

        self.trades.extend(trades.clone());
        trades
    }

    fn match_against_asks(
        &mut self,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        order: &Order,
        trades: &mut Vec<Trade>,
        remaining_qty: &mut u32,
    ) {
        while *remaining_qty > 0 {
            if let Some(ask_idx) = self.find_best_ask(order.agent_id) {
                let (agent_id, price, trade_qty, remove) = {
                    let ask = &mut self.asks[ask_idx];
                    let trade_qty = (*remaining_qty).min(ask.quantity);
                    let agent_id = ask.agent_id;
                    let price = ask.price;
                    *remaining_qty -= trade_qty;
                    ask.quantity -= trade_qty;
                    (agent_id, price, trade_qty, ask.quantity == 0)
                };
                let trade =
                    self.record_trade(order.agent_id, agent_id, price, trade_qty, order.timestamp);
                trades.push(trade);
                if remove {
                    self.asks.remove(ask_idx);
                }
            } else {
                break;
            }
        }
    }

    fn match_against_bids(
        &mut self,
        order: &Order,
        trades: &mut Vec<Trade>,
        remaining_qty: &mut u32,
        // TAG: surface=simulation owner=research-team rule=SIM-001
    ) {
        while *remaining_qty > 0 {
            if let Some(bid_idx) = self.find_best_bid(order.agent_id) {
                let (agent_id, price, trade_qty, remove) = {
                    let bid = &mut self.bids[bid_idx];
                    let trade_qty = (*remaining_qty).min(bid.quantity);
                    let agent_id = bid.agent_id;
                    let price = bid.price;
                    *remaining_qty -= trade_qty;
                    bid.quantity -= trade_qty;
                    (agent_id, price, trade_qty, bid.quantity == 0)
                };
                let trade =
                    self.record_trade(agent_id, order.agent_id, price, trade_qty, order.timestamp);
                trades.push(trade);
                if remove {
                    self.bids.remove(bid_idx);
                }
            } else {
                break;
            }
        }
    }

    fn record_trade(
        &mut self,
        buyer_id: u64,
        seller_id: u64,
        price: f32,
        quantity: u32,
        timestamp: u64,
    ) -> Trade {
        let trade = Trade {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            id: self.next_trade_id,
            buyer_id,
            seller_id,
            price,
            quantity,
            timestamp,
        };
        self.next_trade_id += 1;
        trade
    }

    /// Match limit order
    fn match_limit_order(&mut self, order: &Order) -> (Order, Vec<Trade>) {
        let mut remaining = order.clone();
        let mut trades = Vec::new();

        match order.side {
            OrderSide::Buy => self.match_limit_buy(&mut remaining, &mut trades),
            OrderSide::Sell => self.match_limit_sell(&mut remaining, &mut trades),
        }

        (remaining, trades)
    }

    fn match_limit_buy(&mut self, remaining: &mut Order, trades: &mut Vec<Trade>) {
        while remaining.quantity > 0 {
            if let Some(ask_idx) = self.find_ask_at_or_below(remaining.price, remaining.agent_id) {
                let (agent_id, price, trade_qty, remove) = {
                    let ask = &mut self.asks[ask_idx];
                    let trade_qty = remaining.quantity.min(ask.quantity);
                    let agent_id = ask.agent_id;
                    let price = ask.price;
                    remaining.quantity -= trade_qty;
                    // TAG: surface=simulation owner=research-team rule=SIM-001
                    ask.quantity -= trade_qty;
                    (agent_id, price, trade_qty, ask.quantity == 0)
                    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                };
                let trade = self.record_trade(
                    remaining.agent_id,
                    agent_id,
                    price,
                    trade_qty,
                    remaining.timestamp,
                );
                trades.push(trade.clone());
                self.trades.push(trade);
                if remove {
                    self.asks.remove(ask_idx);
                }
            } else {
                break;
            }
        }
    }

    fn match_limit_sell(&mut self, remaining: &mut Order, trades: &mut Vec<Trade>) {
        while remaining.quantity > 0 {
            if let Some(bid_idx) = self.find_bid_at_or_above(remaining.price, remaining.agent_id) {
                let (agent_id, price, trade_qty, remove) = {
                    let bid = &mut self.bids[bid_idx];
                    let trade_qty = remaining.quantity.min(bid.quantity);
                    let agent_id = bid.agent_id;
                    let price = bid.price;
                    remaining.quantity -= trade_qty;
                    bid.quantity -= trade_qty;
                    (agent_id, price, trade_qty, bid.quantity == 0)
                };
                let trade = self.record_trade(
                    agent_id,
                    // TAG: surface=simulation owner=research-team rule=SIM-001
                    remaining.agent_id,
                    price,
                    trade_qty,
                    remaining.timestamp,
                );
                trades.push(trade.clone());
                self.trades.push(trade);
                if remove {
                    self.bids.remove(bid_idx);
                }
            } else {
                break;
            }
        }
    }

    /// Add order to book
    fn add_to_book(&mut self, order: Order) {
        match order.side {
            OrderSide::Buy => {
                self.bids.push(order);
                self.bids.sort_by(|a, b| {
                    b.price
                        .partial_cmp(&a.price)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            OrderSide::Sell => {
                self.asks.push(order);
                self.asks.sort_by(|a, b| {
                    a.price
                        .partial_cmp(&b.price)
                        .unwrap_or(std::cmp::Ordering::Equal)
                    // TAG: surface=simulation owner=research-team rule=SIM-001
                });
            }
        }
    }

    fn find_best_ask(&self, agent_id: u64) -> Option<usize> {
        self.asks
            .iter()
            .enumerate()
            .filter(|(_, o)| o.agent_id != agent_id)
            .min_by(|(_, a), (_, b)| {
                a.price
                    .partial_cmp(&b.price)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
    }

    fn find_best_bid(&self, agent_id: u64) -> Option<usize> {
        self.bids
            .iter()
            .enumerate()
            .filter(|(_, o)| o.agent_id != agent_id)
            .max_by(|(_, a), (_, b)| {
                a.price
                    .partial_cmp(&b.price)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
        // TAG: surface=simulation owner=research-team rule=SIM-001
    }

    #[must_use]
    fn find_ask_at_or_below(&self, price: f32, agent_id: u64) -> Option<usize> {
        self.asks
            .iter()
            .enumerate()
            .filter(|(_, o)| o.price <= price && o.agent_id != agent_id)
            .min_by(|(_, a), (_, b)| {
                a.price
                    .partial_cmp(&b.price)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
    }

    #[must_use]
    fn find_bid_at_or_above(&self, price: f32, agent_id: u64) -> Option<usize> {
        self.bids
            .iter()
            .enumerate()
            .filter(|(_, o)| o.price >= price && o.agent_id != agent_id)
            .max_by(|(_, a), (_, b)| {
                a.price
                    .partial_cmp(&b.price)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    /// Calculate volatility from trade prices
    #[must_use]
    pub fn volatility(&self, window: usize) -> f32 {
        let recent: Vec<f32> = self
            .trades
            .iter()
            .rev()
            .take(window)
            .map(|t| t.price)
            .collect();
        if recent.len() < 2 {
            return 0.0;
        }

        let mean = recent.iter().sum::<f32>() / recent.len() as f32;
        let variance = recent.iter().map(|p| (p - mean).powi(2)).sum::<f32>() / recent.len() as f32;
        variance.sqrt()
    }
}

/// Simple agent-based market with multiple agents
#[derive(Debug, Clone)]
pub struct ABMMarket {
    /// Order book
    pub order_book: OrderBook,
    /// Current tick
    pub tick: u64,
    /// Fundamental value (for simulation)
    pub fundamental_value: f32,
}

impl Default for ABMMarket {
    fn default() -> Self {
        Self {
            order_book: OrderBook::new(),
            tick: 0,
            fundamental_value: 100.0,
        }
    }
}

impl ABMMarket {
    /// Create new market
    #[must_use]
    // TAG: surface=simulation owner=research-team rule=SIM-001
    pub fn new() -> Self {
        Self::default()
    }

    /// Market maker submits orders to provide liquidity
    pub fn market_maker_quote(&mut self, agent_id: u64, spread: f32, qty: u32) -> Vec<Trade> {
        let mid = self
            .order_book
            .mid_price()
            .unwrap_or(self.fundamental_value);
        let bid = mid - spread / 2.0;
        let ask = mid + spread / 2.0;

        // P1 FIX: Pre-allocate - market maker places 2 orders
        let mut trades = Vec::with_capacity(4);

        // Submit bid
        trades.extend(self.order_book.place_order(
            Order {
                id: 0,
                agent_id,
                order_type: OrderType::Limit,
                side: OrderSide::Buy,
                price: bid,
                quantity: qty,
                timestamp: self.tick,
                // TAG: surface=simulation owner=research-team rule=SIM-001
            },
            self.tick,
        ));

        // Submit ask
        trades.extend(self.order_book.place_order(
            Order {
                id: 0,
                agent_id,
                order_type: OrderType::Limit,
                side: OrderSide::Sell,
                price: ask,
                quantity: qty,
                timestamp: self.tick,
            },
            self.tick,
        ));

        trades
    }

    /// Step simulation forward
    pub fn step(&mut self) {
        self.tick += 1;
        // Deterministic pseudo-random drift derived from simulation tick seed
        let z = self.tick.wrapping_mul(0x9e3779b97f4a7c15);
        let rand_val = ((z >> 32) as u32 as f32) / (u32::MAX as f32);
        self.fundamental_value *= 1.0 + (rand_val - 0.5) * 0.001;
    }
    // TAG: surface=simulation owner=research-team rule=SIM-001
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
}

#[cfg(test)]
mod market_tests {
    use super::*;

    #[test]
    fn test_order_book_creation() {
        let book = OrderBook::new();
        assert!(book.bids.is_empty());
        assert!(book.asks.is_empty());
    }

    #[test]
    fn test_limit_order_placement() {
        let mut book = OrderBook::new();

        let order = Order {
            id: 0,
            agent_id: 1,
            order_type: OrderType::Limit,
            side: OrderSide::Buy,
            price: 100.0,
            quantity: 10,
            timestamp: 0,
        };

        book.place_order(order, 0);

        assert_eq!(book.bids.len(), 1);
        assert_eq!(book.bids[0].price, 100.0);
    }

    #[test]
    fn test_trade_execution() {
        let mut book = OrderBook::new();

        // Place sell limit order
        let sell = Order {
            id: 0,
            agent_id: 1,
            order_type: OrderType::Limit,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            side: OrderSide::Sell,
            price: 100.0,
            quantity: 10,
            timestamp: 0,
        };
        book.place_order(sell, 0);

        // Place buy market order
        let buy = Order {
            id: 0,
            agent_id: 2,
            order_type: OrderType::Market,
            side: OrderSide::Buy,
            price: 0.0, // Market order
            quantity: 5,
            timestamp: 1,
        };
        let trades = book.place_order(buy, 1);

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].price, 100.0);
        assert_eq!(trades[0].quantity, 5);
        assert_eq!(trades[0].buyer_id, 2);
        assert_eq!(trades[0].seller_id, 1);
    }

    #[test]
    fn test_mid_price() {
        let mut book = OrderBook::new();

        // No orders
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(book.mid_price().is_none());

        // Add bid
        book.place_order(
            Order {
                id: 0,
                agent_id: 1,
                order_type: OrderType::Limit,
                side: OrderSide::Buy,
                price: 99.0,
                quantity: 10,
                timestamp: 0,
            },
            0,
        );

        // Add ask
        book.place_order(
            Order {
                id: 0,
                agent_id: 2,
                order_type: OrderType::Limit,
                side: OrderSide::Sell,
                price: 101.0,
                quantity: 10,
                timestamp: 0,
            },
            0,
        );

        assert_eq!(book.mid_price(), Some(100.0));
        assert_eq!(book.spread(), Some(2.0));
        // TAG: surface=simulation owner=research-team rule=SIM-001
    }

    #[test]
    fn test_abm_market() {
        let mut market = ABMMarket::new();

        // Market maker provides liquidity
        let trades = market.market_maker_quote(1, 2.0, 100);

        // Should have placed two orders (not executed against each other)
        assert!(trades.is_empty() || trades.len() == 2);

        // Check orders in book
        assert!(!market.order_book.bids.is_empty() || !market.order_book.asks.is_empty());
    }

    #[test]
    fn test_limit_order_returns_executed_trades_and_prevents_self_trade() {
        let mut book = OrderBook::new();

        // 1. Agent 1 places sell limit order at 100.0, qty 10
        let sell = Order {
            id: 0,
            agent_id: 1,
            order_type: OrderType::Limit,
            side: OrderSide::Sell,
            price: 100.0,
            quantity: 10,
            timestamp: 0,
        };
        let sell_trades = book.place_order(sell, 0);
        assert!(sell_trades.is_empty());

        // 2. Self-trade prevention: Agent 1 places buy limit order at 105.0 crossing book
        let self_buy = Order {
            id: 0,
            agent_id: 1,
            order_type: OrderType::Limit,
            side: OrderSide::Buy,
            price: 105.0,
            quantity: 5,
            timestamp: 1,
        };
        let self_trades = book.place_order(self_buy, 1);
        assert!(self_trades.is_empty(), "Self-trades must be prevented");

        // 3. Agent 2 places buy limit order crossing book at 100.0, qty 5
        let buy = Order {
            id: 0,
            agent_id: 2,
            order_type: OrderType::Limit,
            side: OrderSide::Buy,
            price: 100.0,
            quantity: 5,
            timestamp: 2,
        };
        let trades = book.place_order(buy, 2);
        assert_eq!(
            trades.len(),
            1,
            "Limit order match must return executed trades"
        );
        assert_eq!(trades[0].buyer_id, 2);
        assert_eq!(trades[0].seller_id, 1);
        assert_eq!(trades[0].price, 100.0);
        assert_eq!(trades[0].quantity, 5);
    }
}
