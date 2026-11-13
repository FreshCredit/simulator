//! Prediction Contract Implementation

use super::{MarketId, Resolution};
use crate::types::AgentId;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global contract ID counter
static NEXT_CONTRACT_ID: AtomicU64 = AtomicU64::new(1);

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Prediction contract
#[derive(Debug, Clone)]
pub struct Contract {
    /// Contract ID
    id: ContractId,
    /// Parent market
    market_id: MarketId,
    /// Contract holder
    holder: AgentId,
    /// Contract type
    contract_type: ContractType,
    /// Position (long/short)
    position: Position,
    /// Quantity
    quantity: f64,
    /// Entry price
    entry_price: f64,
    /// Settlement status
    settlement: SettlementStatus,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Contract {
    /// Create new contract
    pub fn new(
        market_id: MarketId,
        holder: AgentId,
        contract_type: ContractType,
        position: Position,
        quantity: f64,
        entry_price: f64,
    ) -> Self {
        let id = ContractId(NEXT_CONTRACT_ID.fetch_add(1, Ordering::SeqCst));

        Self {
            id,
            market_id,
            holder,
            contract_type,
            position,
            quantity,
            entry_price,
            settlement: SettlementStatus::Open,
        }
    }

    /// Get contract ID
    pub fn id(&self) -> ContractId {
        self.id
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get holder
    pub fn holder(&self) -> AgentId {
        self.holder
    }

    /// Get position
    pub fn position(&self) -> Position {
        self.position
    }

    /// Get quantity
    pub fn quantity(&self) -> f64 {
        self.quantity
    }

    /// Get entry price
    pub fn entry_price(&self) -> f64 {
        self.entry_price
    }

    /// Check if settled
    pub fn is_settled(&self) -> bool {
        matches!(self.settlement, SettlementStatus::Settled(_))
    }

    /// Settle contract with resolution
    pub fn settle(&mut self, resolution: &Resolution) {
        if self.is_settled() {
            return;
        }

        let payout = match &self.contract_type {
            ContractType::Binary => {
                let predicted_yes = matches!(self.position, Position::Long);
                resolution.binary_payout(predicted_yes) * self.quantity
            }
            ContractType::Scalar { max_range } => {
                let unit_long = resolution.scalar_payout(self.entry_price, *max_range);
                let unit_payout = match self.position {
                    Position::Long => unit_long,
                    Position::Short => (1.0 - unit_long).max(0.0),
                };
                unit_payout * self.quantity
            }
            ContractType::Index { weights } => {
                // Weighted payout based on index weights
                let weight_factor = if weights.is_empty() {
                    1.0
                } else {
                    let sum: f64 = weights.iter().sum();
                    if sum > 0.0 {
                        sum / weights.len() as f64
                    } else {
                        1.0
                    }
                };
                let raw_val = resolution.actual_value.clamp(0.0, 1.0);
                let unit_payout = match self.position {
                    Position::Long => raw_val,
                    Position::Short => (1.0 - raw_val).max(0.0),
                };
                unit_payout * weight_factor * self.quantity
            }
        };

        self.settlement = SettlementStatus::Settled(payout);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get settlement amount
    pub fn settlement_amount(&self) -> Option<f64> {
        match self.settlement {
            SettlementStatus::Settled(amount) => Some(amount),
            _ => None,
        }
    }

    /// Calculate unrealized P&L at current price
    pub fn unrealized_pnl(&self, current_price: f64) -> f64 {
        match self.position {
            Position::Long => (current_price - self.entry_price) * self.quantity,
            Position::Short => (self.entry_price - current_price) * self.quantity,
        }
    }

    /// Get market value at current price
    pub fn market_value(&self, current_price: f64) -> f64 {
        match self.position {
            Position::Long => current_price * self.quantity,
            Position::Short => (1.0 - current_price) * self.quantity,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Contract identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContractId(pub u64);

/// Contract type
#[derive(Debug, Clone)]
pub enum ContractType {
    /// Binary yes/no contract
    Binary,
    /// Scalar prediction (range-based)
    Scalar { max_range: f64 },
    /// Index contract (weighted combination)
    Index { weights: Vec<f64> },
}

impl ContractType {
    /// Create binary contract
    pub fn binary() -> Self {
        Self::Binary
    }

    /// Create scalar contract with range
    pub fn scalar(max_range: f64) -> Self {
        Self::Scalar { max_range }
    }

    /// Create index contract with weights
    pub fn index(weights: Vec<f64>) -> Self {
        Self::Index { weights }
    }

    /// Get human-readable description
    pub fn description(&self) -> String {
        match self {
            ContractType::Binary => "Binary".to_string(),
            ContractType::Scalar { max_range } => {
                format!("Scalar (range: {})", max_range)
            }
            ContractType::Index { weights } => {
                format!("Index ({} components)", weights.len())
            }
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Position type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    /// Long position (bet on outcome)
    Long,
    /// Short position (bet against outcome)
    Short,
}

impl Position {
    /// Get opposite position
    pub fn opposite(&self) -> Self {
        match self {
            Position::Long => Position::Short,
            Position::Short => Position::Long,
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Position::Long => "Long",
            Position::Short => "Short",
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Settlement status
#[derive(Debug, Clone, Copy)]
pub enum SettlementStatus {
    /// Contract is open
    Open,
    /// Contract is settled
    Settled(f64),
}

/// Portfolio of contracts for an agent
pub struct Portfolio {
    /// Agent ID
    agent_id: AgentId,
    /// Contracts held
    contracts: Vec<Contract>,
    /// Realized P&L
    realized_pnl: f64,
}

impl Portfolio {
    /// Create new portfolio
    pub fn new(agent_id: AgentId) -> Self {
        Self {
            agent_id,
            contracts: Vec::new(),
            realized_pnl: 0.0,
        }
    }

    /// Add contract
    pub fn add_contract(&mut self, contract: Contract) {
        self.contracts.push(contract);
    }

    /// Get all contracts
    pub fn contracts(&self) -> &[Contract] {
        &self.contracts
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get open contracts
    pub fn open_contracts(&self) -> Vec<&Contract> {
        self.contracts.iter().filter(|c| !c.is_settled()).collect()
    }

    /// Get settled contracts
    pub fn settled_contracts(&self) -> Vec<&Contract> {
        self.contracts.iter().filter(|c| c.is_settled()).collect()
    }

    /// Get total exposure (sum of open contract values)
    pub fn total_exposure(&self, current_prices: &std::collections::HashMap<MarketId, f64>) -> f64 {
        self.open_contracts()
            .iter()
            .map(|c| {
                let price = current_prices.get(&c.market_id).copied().unwrap_or(0.5);
                c.market_value(price)
            })
            .sum()
    }

    /// Get unrealized P&L
    pub fn unrealized_pnl(&self, current_prices: &std::collections::HashMap<MarketId, f64>) -> f64 {
        self.open_contracts()
            .iter()
            .map(|c| {
                let price = current_prices.get(&c.market_id).copied().unwrap_or(0.5);
                c.unrealized_pnl(price)
            })
            .sum()
    }

    /// Get realized P&L
    pub fn realized_pnl(&self) -> f64 {
        self.realized_pnl
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update realized P&L from settled contracts
    pub fn update_realized_pnl(&mut self) {
        for contract in &self.contracts {
            if let Some(amount) = contract.settlement_amount() {
                // Calculate profit/loss
                let cost = contract.entry_price() * contract.quantity();
                self.realized_pnl += amount - cost;
            }
        }
    }

    /// Get positions by market
    pub fn positions_by_market(&self) -> std::collections::HashMap<MarketId, Vec<&Contract>> {
        let mut map: std::collections::HashMap<MarketId, Vec<&Contract>> =
            std::collections::HashMap::new();

        for contract in &self.contracts {
            map.entry(contract.market_id).or_default().push(contract);
        }

        map
    }
}

#[cfg(test)]
mod tests {
    use super::super::DocasMetric;
    use super::*;

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    #[test]
    fn test_zero_sum_complement_settlement_conservation() {
        use super::super::DocasMetric;

        let res_binary = Resolution {
            market_id: MarketId(1),
            agent_id: 42,
            metric: DocasMetric::Dopamine,
            actual_value: 0.8, // > 0.5 is Yes
            timestamp: 100,
        };

        // 1. Binary Contract
        let mut bin_long = Contract::new(
            MarketId(1),
            42,
            ContractType::Binary,
            Position::Long,
            50.0,
            0.6,
        );
        let mut bin_short = Contract::new(
            MarketId(1),
            43,
            ContractType::Binary,
            Position::Short,
            50.0,
            0.6,
        );
        bin_long.settle(&res_binary);
        bin_short.settle(&res_binary);
        let bin_payout_sum =
            bin_long.settlement_amount().unwrap() + bin_short.settlement_amount().unwrap();
        assert!((bin_payout_sum - 50.0).abs() < 1e-12);

        // 2. Scalar Contract
        let res_scalar = Resolution {
            market_id: MarketId(2),
            agent_id: 42,
            metric: DocasMetric::Cortisol,
            actual_value: 0.75,
            timestamp: 100,
        };
        let mut sca_long = Contract::new(
            MarketId(2),
            42,
            ContractType::Scalar { max_range: 1.0 },
            Position::Long,
            100.0,
            0.5,
        );
        let mut sca_short = Contract::new(
            MarketId(2),
            43,
            ContractType::Scalar { max_range: 1.0 },
            Position::Short,
            100.0,
            0.5,
        );
        sca_long.settle(&res_scalar);
        sca_short.settle(&res_scalar);
        let sca_payout_sum =
            sca_long.settlement_amount().unwrap() + sca_short.settlement_amount().unwrap();
        assert!((sca_payout_sum - 100.0).abs() < 1e-12);

        // 3. Index Contract
        let res_index = Resolution {
            market_id: MarketId(3),
            agent_id: 42,
            metric: DocasMetric::CompositeIndex,
            actual_value: 0.4,
            timestamp: 100,
        };
        let mut idx_long = Contract::new(
            MarketId(3),
            42,
            ContractType::Index { weights: vec![1.0] },
            Position::Long,
            80.0,
            0.5,
        );
        let mut idx_short = Contract::new(
            MarketId(3),
            43,
            ContractType::Index { weights: vec![1.0] },
            Position::Short,
            80.0,
            0.5,
        );
        idx_long.settle(&res_index);
        idx_short.settle(&res_index);
        let idx_payout_sum =
            idx_long.settlement_amount().unwrap() + idx_short.settlement_amount().unwrap();
        assert!((idx_payout_sum - 80.0).abs() < 1e-12);
    }

    #[test]
    fn test_contract_creation() {
        let contract = Contract::new(
            MarketId(1),
            42,
            ContractType::Binary,
            Position::Long,
            100.0,
            0.6,
        );

        assert_eq!(contract.holder(), 42);
        assert_eq!(contract.position(), Position::Long);
        assert_eq!(contract.quantity(), 100.0);
        assert_eq!(contract.entry_price(), 0.6);
        assert!(!contract.is_settled());
    }

    #[test]
    fn test_contract_settlement() {
        let mut contract = Contract::new(
            MarketId(1),
            42,
            ContractType::Binary,
            Position::Long,
            100.0,
            0.6,
        );

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let resolution = Resolution {
            market_id: MarketId(1),
            agent_id: 1,
            metric: DocasMetric::Dopamine,
            actual_value: 0.7, // Above threshold
            timestamp: 1000,
        };

        contract.settle(&resolution);

        assert!(contract.is_settled());
        assert_eq!(contract.settlement_amount(), Some(100.0)); // Won full amount
    }

    #[test]
    fn test_unrealized_pnl() {
        let contract = Contract::new(
            MarketId(1),
            42,
            ContractType::Binary,
            Position::Long,
            100.0,
            0.6,
        );

        // Price went up
        let pnl = contract.unrealized_pnl(0.7);
        assert!((pnl - 10.0).abs() < 0.001);

        // Price went down
        let pnl = contract.unrealized_pnl(0.5);
        assert!((pnl - (-10.0)).abs() < 0.001);
    }

    #[test]
    fn test_short_position() {
        let contract = Contract::new(
            MarketId(1),
            42,
            ContractType::Binary,
            Position::Short,
            100.0,
            0.6,
        );

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Price went up - short loses
        let pnl = contract.unrealized_pnl(0.7);
        assert!((pnl - (-10.0)).abs() < 0.001);

        // Price went down - short wins
        let pnl = contract.unrealized_pnl(0.5);
        assert!((pnl - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_portfolio() {
        let mut portfolio = Portfolio::new(42);

        let contract1 = Contract::new(
            MarketId(1),
            42,
            ContractType::Binary,
            Position::Long,
            100.0,
            0.6,
        );
        let contract2 = Contract::new(
            MarketId(2),
            42,
            ContractType::Binary,
            Position::Short,
            50.0,
            0.4,
        );

        portfolio.add_contract(contract1);
        portfolio.add_contract(contract2);

        assert_eq!(portfolio.contracts().len(), 2);
        assert_eq!(portfolio.open_contracts().len(), 2);
    }
}
