//! Automated Market Maker Implementation

use super::Side;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Automated Market Maker trait
pub trait AMM {
    /// Get current price
    fn current_price(&self) -> f64;

    /// Execute trade
    fn execute_trade(&mut self, side: Side, quantity: f64) -> (f64, f64);

    /// Get liquidity reserves
    fn reserves(&self) -> (f64, f64);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Constant Product Market Maker (CPMM)
///
/// Based on Uniswap's x * y = k formula
/// For binary prediction markets, we track YES and NO tokens
pub struct ConstantProductAMM {
    /// YES token reserve
    yes_reserve: f64,
    /// NO token reserve
    no_reserve: f64,
    /// Constant product (k = yes * no)
    k: f64,
    /// Fee percentage (0.001 = 0.1%)
    fee: f64,
}

impl ConstantProductAMM {
    /// Create new CPMM with initial liquidity
    pub fn new(initial_liquidity: f64) -> Self {
        let l = if initial_liquidity.is_finite() && initial_liquidity > 0.0 {
            initial_liquidity
        } else {
            1000.0
        };
        // Start with equal reserves (50/50 probability)
        let yes = l / 2.0;
        let no = l / 2.0;

        Self {
            yes_reserve: yes,
            no_reserve: no,
            k: yes * no,
            fee: 0.001,
        }
    }

    /// Create with custom initial price
    pub fn new_with_price(initial_liquidity: f64, initial_price: f64) -> Self {
        let p = initial_price.clamp(0.001, 0.999);
        let l = if initial_liquidity.is_finite() && initial_liquidity > 0.0 {
            initial_liquidity
        } else {
            1000.0
        };

        // In CPMM: price_yes = no_reserve / (yes_reserve + no_reserve)
        // With total reserve L = yes_reserve + no_reserve:
        let no = l * p;
        let yes = l * (1.0 - p);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        Self {
            yes_reserve: yes,
            no_reserve: no,
            k: yes * no,
            fee: 0.001,
        }
    }

    /// Get implied probability from reserves
    fn implied_probability(&self) -> f64 {
        let total = self.yes_reserve + self.no_reserve;
        if total <= 1e-12 {
            0.5
        } else {
            (self.no_reserve / total).clamp(0.001, 0.999)
        }
    }

    /// Calculate output amount for input
    fn get_output_amount(&self, input_amount: f64, input_reserve: f64, output_reserve: f64) -> f64 {
        if !input_amount.is_finite() || input_amount <= 0.0 || output_reserve <= 0.0 {
            return 0.0;
        }
        let input_with_fee = input_amount * (1.0 - self.fee);
        let numerator = input_with_fee * output_reserve;
        let denominator = input_reserve + input_with_fee;
        if denominator <= 1e-12 || !denominator.is_finite() {
            0.0
        } else {
            (numerator / denominator).min(output_reserve * 0.999)
        }
    }

    /// Buy YES tokens (bet on outcome)
    fn buy_yes(&mut self, amount: f64) -> f64 {
        let yes_out = self.get_output_amount(amount, self.no_reserve, self.yes_reserve);
        self.no_reserve += amount;
        self.yes_reserve -= yes_out;
        self.k = self.yes_reserve * self.no_reserve;
        yes_out
    }

    /// Buy NO tokens (bet against outcome)
    fn buy_no(&mut self, amount: f64) -> f64 {
        let no_out = self.get_output_amount(amount, self.yes_reserve, self.no_reserve);
        self.yes_reserve += amount;
        self.no_reserve -= no_out;
        self.k = self.yes_reserve * self.no_reserve;
        no_out
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl AMM for ConstantProductAMM {
    fn current_price(&self) -> f64 {
        self.implied_probability()
    }

    fn execute_trade(&mut self, side: Side, quantity: f64) -> (f64, f64) {
        if !quantity.is_finite() || quantity <= 0.0 {
            return (0.0, 0.0);
        }
        match side {
            Side::Buy => {
                // Buying YES (betting on outcome)
                let received = self.buy_yes(quantity);
                (received, quantity)
            }
            Side::Sell => {
                // Buying NO (betting against outcome)
                let received = self.buy_no(quantity);
                (received, quantity)
            }
        }
    }

    fn reserves(&self) -> (f64, f64) {
        (self.yes_reserve, self.no_reserve)
    }
}

/// Logarithmic Market Scoring Rule (LMSR) AMM
///
/// Alternative to CPMM with bounded loss
pub struct LMSR {
    /// Quantity of YES tokens outstanding
    q_yes: f64,
    /// Quantity of NO tokens outstanding
    q_no: f64,
    /// Liquidity parameter (b)
    b: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl LMSR {
    /// Create new LMSR
    pub fn new(liquidity: f64) -> Self {
        let b = if liquidity.is_finite() && liquidity > 0.0 {
            liquidity
        } else {
            100.0
        };
        Self {
            q_yes: 0.0,
            q_no: 0.0,
            b,
        }
    }

    /// Cost function C(q) = b * log(exp(q_yes/b) + exp(q_no/b)) using Log-Sum-Exp
    fn cost(&self) -> f64 {
        let b = self.b.max(1e-10);
        let m = self.q_yes.max(self.q_no) / b;
        let exp_yes = (self.q_yes / b - m).exp();
        let exp_no = (self.q_no / b - m).exp();
        b * (m + (exp_yes + exp_no).ln())
    }

    /// Price of outcome p_yes = 1 / (1 + exp((q_no - q_yes) / b))
    fn price(&self, yes: bool) -> f64 {
        let b = self.b.max(1e-10);
        let diff = (self.q_no - self.q_yes) / b;
        let p_yes = if diff >= 700.0 {
            0.0
        } else if diff <= -700.0 {
            1.0
        } else {
            1.0 / (1.0 + diff.exp())
        };

        if yes {
            p_yes
        } else {
            1.0 - p_yes
        }
    }

    /// Cost to buy shares
    fn cost_to_buy(&self, yes_shares: f64, no_shares: f64) -> f64 {
        let new_state = LMSR {
            q_yes: self.q_yes + yes_shares,
            q_no: self.q_no + no_shares,
            b: self.b,
        };

        new_state.cost() - self.cost()
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl AMM for LMSR {
    fn current_price(&self) -> f64 {
        self.price(true) // Price of YES
    }

    fn execute_trade(&mut self, side: Side, quantity: f64) -> (f64, f64) {
        if !quantity.is_finite() || quantity <= 0.0 {
            return (0.0, 0.0);
        }
        match side {
            Side::Buy => {
                // Buying YES shares
                let cost = self.cost_to_buy(quantity, 0.0);
                self.q_yes += quantity;
                (quantity, cost)
            }
            Side::Sell => {
                // Buying NO shares
                let cost = self.cost_to_buy(0.0, quantity);
                self.q_no += quantity;
                (quantity, cost)
            }
        }
    }

    fn reserves(&self) -> (f64, f64) {
        (self.q_yes, self.q_no)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpmm_initial_price() {
        let amm = ConstantProductAMM::new(1000.0);
        // Equal reserves should give 50% probability
        assert!((amm.current_price() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_cpmm_custom_price() {
        let amm = ConstantProductAMM::new_with_price(1000.0, 0.7);
        assert!((amm.current_price() - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_cpmm_trade_changes_price() {
        let mut amm = ConstantProductAMM::new(1000.0);
        let initial_price = amm.current_price();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Buy YES (bet on outcome)
        let initial_k = amm.k;
        amm.execute_trade(Side::Buy, 100.0);

        // Price should increase and k should increase with fee retention
        assert!(amm.current_price() > initial_price);
        assert!(amm.k >= initial_k);
    }

    #[test]
    fn test_lmsr_price() {
        let lmsr = LMSR::new(100.0);
        // No trades yet, should be 50/50
        assert!((lmsr.current_price() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_lmsr_trade_cost() {
        let mut lmsr = LMSR::new(100.0);

        let initial_cost = lmsr.cost();

        // Buy some YES shares
        let (shares, cost) = lmsr.execute_trade(Side::Buy, 10.0);

        assert_eq!(shares, 10.0);
        assert!(cost > 0.0);
        assert!(lmsr.cost() > initial_cost);
    }
}

#[test]
fn test_lmsr_large_shares_no_nan_overflow() {
    let mut lmsr = LMSR::new(10.0);
    // Extremely large share purchase (8000 / 10 = 800 > 710, where exp() overflows)
    let (shares, cost) = lmsr.execute_trade(Side::Buy, 8000.0);
    assert_eq!(shares, 8000.0);
    assert!(cost.is_finite());
    assert!(cost > 0.0);
    let price = lmsr.current_price();
    assert!(price.is_finite());
    assert!((price - 1.0).abs() < 1e-4);
}

#[test]
fn test_cpmm_custom_price_ratio_and_reserves() {
    let amm = ConstantProductAMM::new_with_price(1000.0, 0.7);
    let (yes, no) = amm.reserves();
    assert!((yes + no - 1000.0).abs() < 1e-6);
    assert!((amm.current_price() - 0.7).abs() < 1e-6);
}
