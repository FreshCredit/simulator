//! Banking System with Diamond-Dybvig Bank Run Dynamics
//!
//! Implements financial contagion through bank runs based on:
//! - Diamond & Dybvig (1983): Multiple equilibrium model of bank runs
//! - Allen & Gale (2000): Interbank lending contagion
//! - Gorton (1988): Asymmetric information and deposit insurance
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-003: Terminology - SIMULATION banking, NOT financial advice

use crate::economics::util::{kahan_sum, KahanAccumulator};
use crate::types::{AgentId, Tick};
use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Unique identifier for banks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BankId(pub usize);

/// Bank health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BankHealth {
    /// Healthy bank - normal operations
    Healthy,
    /// Stressed bank - liquidity issues but solvent
    Stressed,
    /// Insolvent bank - negative equity
    Insolvent,
    /// Failed bank - closed, depositors being paid
    Failed,
}

/// Loan record for bank assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loan {
    /// Borrower agent ID
    pub borrower_id: AgentId,
    /// Principal amount
    pub principal: f32,
    /// Interest rate
    pub interest_rate: f32,
    /// Maturity tick
    pub maturity_tick: Tick,
    /// Collateral value
    pub collateral: f32,
    /// Probability of default
    pub default_probability: f32,
    /// Whether loan has defaulted
    pub defaulted: bool,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Loan {
    /// Create a new loan
    #[must_use]
    pub const fn new(
        borrower_id: AgentId,
        principal: f32,
        interest_rate: f32,
        maturity_tick: Tick,
        collateral: f32,
    ) -> Self {
        Self {
            borrower_id,
            principal,
            interest_rate,
            maturity_tick,
            collateral,
            default_probability: 0.0,
            defaulted: false,
        }
    }

    /// Calculate expected value of loan
    #[must_use]
    pub fn expected_value(&self) -> f32 {
        if self.defaulted {
            self.collateral
        } else {
            self.principal * (1.0 + self.interest_rate) * (1.0 - self.default_probability)
                + self.collateral * self.default_probability
        }
    }

    /// Mark loan as defaulted
    pub fn mark_defaulted(&mut self) {
        self.defaulted = true;
    }
}

/// Individual bank in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bank {
    /// Bank identifier
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub id: BankId,
    /// Bank name
    pub name: String,

    /// Deposit liabilities (`agent_id` -> deposit amount)
    pub deposits: HashMap<AgentId, f32>,
    /// Total deposits
    pub total_deposits: f32,

    /// Loan assets
    pub loans: Vec<Loan>,
    /// Total loans outstanding
    pub total_loans: f32,

    /// Liquid reserves (cash or central bank deposits)
    pub reserves: f32,

    /// Reserve requirement ratio (regulatory minimum)
    pub reserve_ratio: f32,

    /// Bank solvency ratio (assets / liabilities)
    pub solvency: f32,
    /// Bank liquidity ratio (liquid assets / deposits)
    pub liquidity: f32,
    /// Current health status
    pub health_status: BankHealth,

    /// Withdrawal queue during run (agents waiting to withdraw)
    pub withdrawal_queue: Vec<AgentId>,
    /// Agents who have already withdrawn (to prevent double-withdrawal)
    pub withdrawn_agents: Vec<AgentId>,

    /// Interbank lending exposure (`bank_id` -> amount lent)
    pub interbank_lending: HashMap<BankId, f32>,
    /// Interbank borrowing (`bank_id` -> amount borrowed)
    pub interbank_borrowing: HashMap<BankId, f32>,

    /// Total interbank assets
    pub interbank_assets: f32,
    /// Total interbank liabilities
    pub interbank_liabilities: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Bank {
    /// Create a new bank
    pub fn new(id: BankId, name: impl Into<String>, initial_reserves: f32) -> Self {
        Self {
            id,
            name: name.into(),
            deposits: HashMap::new(),
            total_deposits: 0.0,
            loans: Vec::new(),
            total_loans: 0.0,
            reserves: initial_reserves,
            reserve_ratio: 0.1, // 10% reserve requirement
            solvency: 1.0,
            liquidity: 1.0,
            health_status: BankHealth::Healthy,
            withdrawal_queue: Vec::new(),
            withdrawn_agents: Vec::new(),
            interbank_lending: HashMap::new(),
            interbank_borrowing: HashMap::new(),
            interbank_assets: 0.0,
            interbank_liabilities: 0.0,
        }
    }

    /// Accept a deposit from an agent
    pub fn accept_deposit(&mut self, agent_id: AgentId, amount: f32) {
        let current = self.deposits.get(&agent_id).copied().unwrap_or(0.0);
        self.deposits.insert(agent_id, current + amount);
        self.total_deposits += amount;
        self.reserves += amount;
        self.update_health_metrics();
    }

    /// Make a loan to an agent
    pub fn make_loan(&mut self, loan: Loan) -> bool {
        // Check if we have sufficient reserves after loan
        let required_reserves = self.total_deposits * self.reserve_ratio;
        let available_for_loan = self.reserves - required_reserves;

        if available_for_loan >= loan.principal {
            self.reserves -= loan.principal;
            self.total_loans += loan.principal;
            self.loans.push(loan);
            self.update_health_metrics();
            true
        } else {
            false // Insufficient reserves
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Process a withdrawal request
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn process_withdrawal(&mut self, agent_id: AgentId) -> f32 {
        // Check if already withdrawn
        if self.withdrawn_agents.contains(&agent_id) {
            return 0.0;
        }

        let deposit = self.deposits.get(&agent_id).copied().unwrap_or(0.0);

        if deposit <= 0.0 {
            return 0.0;
        }

        // Check if bank can pay
        if self.reserves >= deposit {
            self.reserves -= deposit;
            self.total_deposits -= deposit;
            self.deposits.remove(&agent_id);
            self.withdrawn_agents.push(agent_id);
            self.update_health_metrics();
            deposit
        } else {
            // Bank cannot pay full amount - suspend or pay partial
            let partial_payment = self.reserves;
            self.reserves = 0.0;
            self.total_deposits -= partial_payment;
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            if let Some(dep) = self.deposits.get_mut(&agent_id) {
                *dep -= partial_payment;
            }
            self.withdrawn_agents.push(agent_id);
            self.health_status = BankHealth::Stressed;
            self.update_health_metrics();
            partial_payment
        }
    }

    /// Queue an agent for withdrawal (during bank run)
    pub fn queue_withdrawal(&mut self, agent_id: AgentId) {
        if !self.withdrawal_queue.contains(&agent_id)
            && !self.withdrawn_agents.contains(&agent_id)
            && self.deposits.contains_key(&agent_id)
        {
            self.withdrawal_queue.push(agent_id);
        }
    }

    /// Add interbank lending exposure
    pub fn add_interbank_loan(&mut self, counterparty: BankId, amount: f32) {
        let current = self
            .interbank_lending
            .get(&counterparty)
            .copied()
            .unwrap_or(0.0);
        self.interbank_lending
            .insert(counterparty, current + amount);
        self.interbank_assets += amount;
        self.update_health_metrics();
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Add interbank borrowing
    pub fn add_interbank_borrowing(&mut self, counterparty: BankId, amount: f32) {
        let current = self
            .interbank_borrowing
            .get(&counterparty)
            .copied()
            .unwrap_or(0.0);
        self.interbank_borrowing
            .insert(counterparty, current + amount);
        self.interbank_liabilities += amount;
        self.update_health_metrics();
    }

    /// Update solvency and liquidity ratios
    /// Uses Kahan summation for precise loan value aggregation
    pub fn update_health_metrics(&mut self) {
        // Calculate total assets using Kahan summation for loan values
        let loan_values: Vec<f32> = self.loans.iter().map(Loan::expected_value).collect();
        let loan_value: f32 = kahan_sum(&loan_values);
        let total_assets = self.reserves + loan_value + self.interbank_assets;

        // Calculate total liabilities using Kahan summation
        let liability_values = vec![self.total_deposits, self.interbank_liabilities];
        let total_liabilities = kahan_sum(&liability_values);

        // Solvency ratio
        self.solvency = if total_liabilities > 0.0 {
            total_assets / total_liabilities
        } else {
            1.0
        };

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Liquidity ratio
        self.liquidity = if self.total_deposits > 0.0 {
            self.reserves / self.total_deposits
        } else {
            1.0
        };

        // Update health status based on metrics
        self.health_status = match () {
            () if self.solvency < 1.0 => BankHealth::Insolvent,
            () if self.liquidity < self.reserve_ratio => BankHealth::Stressed,
            () => BankHealth::Healthy,
        };
    }

    /// Check if bank is critically insolvent (below critical solvency threshold)
    fn is_critically_insolvent(&self) -> bool {
        self.solvency < 0.9
    }

    /// Check if bank has critical liquidity crisis with deposits
    fn has_liquidity_crisis(&self) -> bool {
        self.liquidity < 0.05 && self.total_deposits > 0.0
    }

    /// Check if bank has failed
    pub fn check_failure(&mut self) -> bool {
        // Bank fails if insolvent or cannot meet reserve requirements for too long
        if self.is_critically_insolvent() || self.has_liquidity_crisis() {
            self.health_status = BankHealth::Failed;
            true
        } else {
            false
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get total assets
    /// Uses Kahan summation for precise loan value aggregation
    #[must_use]
    pub fn total_assets(&self) -> f32 {
        let loan_values: Vec<f32> = self.loans.iter().map(Loan::expected_value).collect();
        let loan_value: f32 = kahan_sum(&loan_values);
        let asset_values = vec![self.reserves, loan_value, self.interbank_assets];
        kahan_sum(&asset_values)
    }

    /// Get total liabilities
    /// Uses Kahan summation for precise aggregation
    #[must_use]
    pub fn total_liabilities(&self) -> f32 {
        let liability_values = vec![self.total_deposits, self.interbank_liabilities];
        kahan_sum(&liability_values)
    }

    /// Get net worth (equity)
    #[must_use]
    pub fn net_worth(&self) -> f32 {
        self.total_assets() - self.total_liabilities()
    }
}

/// Systemic risk metrics for the banking system
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SystemicRiskMetrics {
    /// Capital adequacy ratio (total capital / risk-weighted assets)
    pub capital_adequacy: f32,
    /// Network density of interbank connections
    pub network_density: f32,
    /// Contagion potential (expected cascade size from random failure)
    pub contagion_potential: f32,
    /// Current run probability estimate
    pub run_probability: f32,
    /// Number of banks currently stressed
    pub stressed_banks: usize,
    /// Number of failed banks
    pub failed_banks: usize,
}

impl Default for SystemicRiskMetrics {
    fn default() -> Self {
        Self {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            capital_adequacy: 1.0,
            network_density: 0.0,
            contagion_potential: 0.0,
            run_probability: 0.0,
            stressed_banks: 0,
            failed_banks: 0,
        }
    }
}

/// Banking system with deposit-taking and lending
/// Implements Diamond-Dybvig model dynamics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankingSystem {
    /// Banks in the system
    pub banks: Vec<Bank>,

    /// Interbank lending network (`bank_id` -> [(counterparty, exposure)])
    pub interbank_network: HashMap<BankId, Vec<(BankId, f32)>>,

    /// Central bank liquidity available for lending
    pub central_bank_liquidity: f32,

    /// Deposit insurance coverage limit per depositor
    pub deposit_insurance_limit: f32,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Whether deposit insurance is active
    pub deposit_insurance_active: bool,

    /// Bank run state
    pub run_in_progress: bool,
    /// Tick when run started
    pub run_start_tick: Option<Tick>,
    /// Banks currently experiencing runs
    pub banks_in_run: Vec<BankId>,

    /// Failed banks
    pub failed_banks: Vec<BankId>,

    /// Systemic risk metrics
    pub risk_metrics: SystemicRiskMetrics,

    /// Total deposits in system
    pub total_deposits: f32,
    /// Total loans in system
    pub total_loans: f32,

    /// Sunspot variable (random trigger for self-fulfilling runs)
    pub sunspot_threshold: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl BankingSystem {
    /// Create a new banking system
    #[must_use]
    pub fn new(central_bank_liquidity: f32, deposit_insurance_limit: f32) -> Self {
        Self {
            banks: Vec::new(),
            interbank_network: HashMap::new(),
            central_bank_liquidity,
            deposit_insurance_limit,
            deposit_insurance_active: true,
            run_in_progress: false,
            run_start_tick: None,
            banks_in_run: Vec::new(),
            failed_banks: Vec::new(),
            risk_metrics: SystemicRiskMetrics::default(),
            total_deposits: 0.0,
            total_loans: 0.0,
            sunspot_threshold: 0.3,
        }
    }

    /// Add a bank to the system
    /// Uses Kahan summation for precise total aggregation
    pub fn add_bank(&mut self, bank: Bank) -> BankId {
        let id = BankId(self.banks.len());
        let mut bank = bank;
        bank.id = id;

        // Use Kahan accumulator for precise total updates
        let mut deposit_acc = KahanAccumulator::new();
        deposit_acc.add(self.total_deposits);
        deposit_acc.add(bank.total_deposits);
        self.total_deposits = deposit_acc.sum();

        let mut loan_acc = KahanAccumulator::new();
        loan_acc.add(self.total_loans);
        loan_acc.add(bank.total_loans);
        self.total_loans = loan_acc.sum();

        self.banks.push(bank);
        self.interbank_network.insert(id, Vec::new());
        id
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create interbank lending link
    pub fn add_interbank_link(&mut self, lender: BankId, borrower: BankId, amount: f32) {
        // Update network structure
        let exposures = self.interbank_network.entry(lender).or_default();
        if let Some(existing) = exposures.iter_mut().find(|(b, _)| *b == borrower) {
            existing.1 += amount;
        } else {
            exposures.push((borrower, amount));
        }

        // Update bank records
        if let Some(lender_bank) = self.banks.get_mut(lender.0) {
            lender_bank.add_interbank_loan(borrower, amount);
        }
        if let Some(borrower_bank) = self.banks.get_mut(borrower.0) {
            borrower_bank.add_interbank_borrowing(lender, amount);
        }
    }

    /// Agent decides whether to withdraw (Diamond-Dybvig decision model)
    ///
    /// # Arguments
    /// * `agent_id` - The agent making the decision
    /// * `bank_id` - The bank where deposits are held
    /// * `agent_trust` - Agent's trust in the banking system [0.0, 1.0]
    /// * `agent_impatience` - Agent's impatience parameter (Diamond-Dybvig type) [0.0, 1.0]
    /// * `fundamental_only` - If true, only consider fundamentals (no coordination effects)
    ///
    /// # Returns
    /// true if agent decides to withdraw
    #[must_use]
    pub fn agent_withdrawal_decision(
        &self,
        agent_id: AgentId,
        bank_id: BankId,
        agent_trust: f32,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        agent_impatience: f32,
        fundamental_only: bool,
    ) -> bool {
        let Some(bank) = self.banks.get(bank_id.0) else {
            return false;
        };

        // Check if agent has deposit
        let deposit = bank.deposits.get(&agent_id).copied().unwrap_or(0.0);
        if deposit <= 0.0 {
            return false;
        }

        // 1. Fundamental signal (bank solvency)
        let fundamental_signal = if bank.solvency < 1.0 {
            0.95 // Insolvent - definitely withdraw
        } else if bank.liquidity < bank.reserve_ratio * 0.5 {
            0.8 // Severely illiquid - very likely withdraw
        } else if bank.liquidity < bank.reserve_ratio {
            0.6 // Below reserve requirement - likely withdraw
        } else {
            0.1 // Healthy - unlikely to withdraw
        };

        // If fundamental-only mode (no sunspot/coordination effects)
        if fundamental_only {
            return fundamental_signal > 0.7;
        }

        // 2. Coordination signal (other depositors' behavior)
        let coordination_signal = if self.run_in_progress && self.banks_in_run.contains(&bank_id) {
            // Run already in progress at this bank - strong incentive to withdraw
            0.85
        } else if !bank.withdrawal_queue.is_empty() {
            // Some withdrawing - uncertainty and fear
            let queue_ratio = bank.withdrawal_queue.len() as f32 / bank.deposits.len() as f32;
            0.3 + queue_ratio * 0.5 // Up to 0.8 if many in queue
        } else {
            0.0 // No one withdrawing
        };

        // 3. Trust effect (inverse relationship)
        let trust_effect = (1.0 - agent_trust.clamp(0.0, 1.0)) * 0.4;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // 4. Impatience (Diamond-Dybvig type parameter)
        // High impatience = "impatient" agent who needs liquidity early
        let impatience_effect = agent_impatience * 0.3;

        // Combined withdrawal probability (weighted combination)
        let withdraw_prob = fundamental_signal * 0.35
            + coordination_signal * 0.35
            + trust_effect * 0.15
            + impatience_effect * 0.15;

        // Deposit insurance reduces withdrawal incentive (Gorton 1988)
        let insured_prob =
            if self.deposit_insurance_active && deposit <= self.deposit_insurance_limit {
                withdraw_prob * 0.2 // Strong reduction for insured deposits
            } else if self.deposit_insurance_active {
                // Partial insurance
                let insured_fraction = self.deposit_insurance_limit / deposit;
                withdraw_prob * (1.0 - 0.7 * insured_fraction)
            } else {
                withdraw_prob
            };

        // Decision threshold
        insured_prob > 0.5
    }

    /// Process withdrawal queue for all banks
    pub fn process_withdrawals(&mut self, tick: Tick, rng: &mut StdRng) {
        let mut run_triggered = false;
        let mut triggered_banks = Vec::new();
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let mut new_failures = Vec::new();

        for bank in &mut self.banks {
            if bank.health_status == BankHealth::Failed {
                continue;
            }

            // Capture initial queue length before drain
            let initial_queue_len = bank.withdrawal_queue.len();

            // Process withdrawal queue FIFO
            let queue = std::mem::take(&mut bank.withdrawal_queue);
            let mut withdrawals_total = 0.0;

            for agent_id in queue {
                let withdrawn = bank.process_withdrawal(agent_id);
                withdrawals_total += withdrawn;

                // Check if bank failed during processing (deduplicate failures)
                if bank.check_failure()
                    && !self.failed_banks.contains(&bank.id)
                    && !new_failures.contains(&bank.id)
                {
                    new_failures.push(bank.id);
                }
            }

            // Check if run threshold reached at this bank
            let withdrawal_rate = if bank.total_deposits + withdrawals_total > 0.0 {
                withdrawals_total / (bank.total_deposits + withdrawals_total)
            } else {
                0.0
            };

            // Run triggers if: withdrawal rate > 30%, or queue was long
            let queue_pressure = initial_queue_len as f32 / (bank.deposits.len() as f32 + 1.0);

            if (withdrawal_rate > 0.3 || queue_pressure > 0.25)
                && !self.banks_in_run.contains(&bank.id)
                && bank.health_status != BankHealth::Failed
            {
                run_triggered = true;
                triggered_banks.push(bank.id);
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Add new failures
        self.failed_banks.extend(new_failures);

        // Update totals using Kahan summation for precision
        let deposit_values: Vec<f32> = self.banks.iter().map(|b| b.total_deposits).collect();
        self.total_deposits = kahan_sum(&deposit_values);

        // Start run if triggered
        if run_triggered {
            for bank_id in triggered_banks {
                self.start_bank_run(bank_id, tick, rng);
            }
        }
    }

    /// Start bank run at a specific bank with contagion
    #[allow(clippy::collection_is_never_read)]
    fn start_bank_run(&mut self, trigger_bank: BankId, tick: Tick, rng: &mut StdRng) {
        if !self.run_in_progress {
            self.run_in_progress = true;
            self.run_start_tick = Some(tick);
        }

        if !self.banks_in_run.contains(&trigger_bank) {
            self.banks_in_run.push(trigger_bank);
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        }

        // Contagion through interbank network (Allen & Gale 2000)
        let mut affected = vec![trigger_bank];
        let mut to_check = vec![trigger_bank];
        let mut visited = std::collections::HashSet::new();
        visited.insert(trigger_bank);

        while let Some(bank_id) = to_check.pop() {
            // Find counterparties with exposure
            if let Some(exposures) = self.interbank_network.get(&bank_id) {
                for (counterparty, exposure) in exposures {
                    if !visited.contains(counterparty) && *exposure > 0.05 {
                        // Counterparty at risk due to exposure
                        visited.insert(*counterparty);
                        affected.push(*counterparty);
                        to_check.push(*counterparty);

                        // Trigger withdrawals at counterparty with probability based on exposure
                        let contagion_prob = (*exposure * 2.0).min(0.9);
                        let roll: f32 = rng.gen_range(0.0..1.0);

                        if roll < contagion_prob && !self.banks_in_run.contains(counterparty) {
                            self.banks_in_run.push(*counterparty);

                            // Queue all depositors at counterparty
                            if let Some(bank) = self.banks.get_mut(counterparty.0) {
                                // TAG: surface=simulation owner=research-team rule=SIM-001
                                for agent_id in bank.deposits.clone().keys() {
                                    bank.queue_withdrawal(*agent_id);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Sunspot-triggered bank run (self-fulfilling prophecy)
    /// Per Diamond-Dybvig: Random events can trigger runs in multiple equilibrium
    pub fn sunspot_trigger(&mut self, tick: Tick, rng: &mut StdRng, sunspot_strength: f32) -> bool {
        // Sunspot can only trigger if no run in progress and conditions are ripe
        if self.run_in_progress {
            return false;
        }

        // Check systemic vulnerability
        let avg_liquidity: f32 =
            self.banks.iter().map(|b| b.liquidity).sum::<f32>() / self.banks.len().max(1) as f32;
        let avg_trust: f32 = 0.5; // Would be passed from simulation

        // System is vulnerable if liquidity is low or trust is low
        let vulnerability = (1.0 - avg_liquidity) * 0.5 + (1.0 - avg_trust) * 0.5;

        // Sunspot triggers run with probability proportional to vulnerability
        let trigger_prob = vulnerability * sunspot_strength;
        let roll: f32 = rng.gen_range(0.0..1.0);

        if roll < trigger_prob {
            // Trigger run at weakest bank
            if let Some(weakest) = self
                .banks
                .iter()
                .filter(|b| b.health_status != BankHealth::Failed)
                .min_by(|a, b| {
                    a.liquidity
                        .partial_cmp(&b.liquidity)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|b| b.id)
            {
                // Queue all depositors at weakest bank
                if let Some(bank) = self.banks.get_mut(weakest.0) {
                    for agent_id in bank.deposits.clone().keys() {
                        bank.queue_withdrawal(*agent_id);
                    }
                }
                self.start_bank_run(weakest, tick, rng);
                return true;
            }
        }

        false
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Central bank intervention (lender of last resort)
    /// Per Bagehot: Lend freely to solvent but illiquid banks
    pub fn central_bank_intervention(&mut self) -> f32 {
        let mut total_provided = 0.0;

        for bank in &mut self.banks {
            // Only help banks that are solvent but illiquid
            if bank.health_status == BankHealth::Stressed && bank.solvency > 0.95 {
                let required_reserves = bank.total_deposits * bank.reserve_ratio;
                let liquidity_need = required_reserves - bank.reserves;

                if liquidity_need > 0.0 {
                    let provision = liquidity_need.min(self.central_bank_liquidity);

                    bank.reserves += provision;
                    self.central_bank_liquidity -= provision;
                    total_provided += provision;

                    // Check if intervention restored health
                    bank.update_health_metrics();
                    if bank.liquidity >= bank.reserve_ratio {
                        bank.health_status = BankHealth::Healthy;
                    }
                }
            }
        }

        // Check if run has stopped
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        self.check_run_status();

        total_provided
    }

    /// Deposit insurance payout for failed banks
    /// Uses Kahan summation for precise payout aggregation
    pub fn process_deposit_insurance(&mut self) -> f32 {
        if !self.deposit_insurance_active {
            return 0.0;
        }

        let mut payout_acc = KahanAccumulator::new();

        for bank_id in &self.failed_banks {
            if let Some(bank) = self.banks.get(bank_id.0) {
                for deposit in bank.deposits.values() {
                    let insured_amount = deposit.min(self.deposit_insurance_limit);
                    payout_acc.add(insured_amount);
                    // In a full simulation, this would transfer to the agent
                }
            }
        }

        payout_acc.sum()
    }

    /// Check and update bank run status
    fn check_run_status(&mut self) {
        // Run stops when all queues are empty and no bank is in critical state
        let queues_empty = self.banks.iter().all(|b| b.withdrawal_queue.is_empty());
        let no_critical = self
            .banks
            .iter()
            .all(|b| b.liquidity > 0.05 || b.health_status == BankHealth::Failed);

        if queues_empty && no_critical && self.run_in_progress {
            self.run_in_progress = false;
            self.banks_in_run.clear();
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute systemic risk metrics
    /// Uses Kahan summation for precise aggregation of bank assets and liabilities
    pub fn compute_systemic_risk_metrics(&mut self) {
        let asset_values: Vec<f32> = self.banks.iter().map(Bank::total_assets).collect();
        let total_assets: f32 = kahan_sum(&asset_values);

        let liability_values: Vec<f32> = self.banks.iter().map(Bank::total_liabilities).collect();
        let total_liabilities: f32 = kahan_sum(&liability_values);

        // Capital adequacy
        self.risk_metrics.capital_adequacy = if total_liabilities > 0.0 {
            total_assets / total_liabilities
        } else {
            1.0
        };

        // Network density
        let n = self.banks.len() as f32;
        let max_edges = n * (n - 1.0);
        let actual_edges: usize = self
            .interbank_network
            .values()
            .map(std::vec::Vec::len)
            .sum();
        self.risk_metrics.network_density = if max_edges > 0.0 {
            (actual_edges as f32) / max_edges
        } else {
            0.0
        };
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // Contagion potential
        self.risk_metrics.contagion_potential = self.compute_contagion_potential();

        // Run probability
        self.risk_metrics.run_probability = if self.run_in_progress { 1.0 } else { 0.0 };

        // Count stressed and failed banks
        self.risk_metrics.stressed_banks = self
            .banks
            .iter()
            .filter(|b| b.health_status == BankHealth::Stressed)
            .count();
        self.risk_metrics.failed_banks = self.failed_banks.len();
    }

    /// Compute contagion potential using network analysis
    /// Uses Kahan summation for precise exposure aggregation
    fn compute_contagion_potential(&self) -> f32 {
        // Simple measure: average exposure concentration
        let mut max_exposure: f32 = 0.0;
        let mut exposure_acc = crate::economics::util::KahanAccumulator::new();

        for exposures in self.interbank_network.values() {
            for (_, amount) in exposures {
                exposure_acc.add(*amount);
                max_exposure = max_exposure.max(*amount);
            }
        }

        let total_exposure = exposure_acc.sum();

        let avg_exposure = if self.interbank_network.is_empty() {
            0.0
        } else {
            total_exposure / self.interbank_network.len() as f32
        };

        // Contagion potential increases with concentration
        if avg_exposure > 0.0 {
            (max_exposure / avg_exposure).min(1.0) * self.risk_metrics.network_density
        } else {
            0.0
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get all depositors across all banks
    #[must_use]
    pub fn get_all_depositors(&self) -> Vec<AgentId> {
        let mut depositors = std::collections::HashSet::new();
        for bank in &self.banks {
            for agent_id in bank.deposits.keys() {
                depositors.insert(*agent_id);
            }
        }
        depositors.into_iter().collect()
    }

    /// Get total deposits for an agent across all banks
    /// Uses Kahan summation for precise aggregation
    #[must_use]
    pub fn get_agent_total_deposits(&self, agent_id: AgentId) -> f32 {
        let deposit_values: Vec<f32> = self
            .banks
            .iter()
            .map(|b| b.deposits.get(&agent_id).copied().unwrap_or(0.0))
            .collect();
        kahan_sum(&deposit_values)
    }

    /// Update all bank health metrics
    pub fn update_all_health_metrics(&mut self) {
        for bank in &mut self.banks {
            bank.update_health_metrics();
        }
        self.compute_systemic_risk_metrics();
    }

    /// Check if system is in crisis
    #[must_use]
    pub fn is_crisis(&self) -> bool {
        self.run_in_progress
            || self.risk_metrics.failed_banks > 0
            || self.risk_metrics.stressed_banks > self.banks.len() / 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn create_test_bank(id: usize, name: &str, reserves: f32) -> Bank {
        Bank::new(BankId(id), name, reserves)
    }

    #[test]
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    fn test_bank_creation() {
        let bank = create_test_bank(0, "Test Bank", 1000.0);
        assert_eq!(bank.reserves, 1000.0);
        assert_eq!(bank.health_status, BankHealth::Healthy);
        assert_eq!(bank.solvency, 1.0);
    }

    #[test]
    fn test_deposit_acceptance() {
        let mut bank = create_test_bank(0, "Test Bank", 1000.0);
        bank.accept_deposit(1, 500.0);

        assert_eq!(bank.total_deposits, 500.0);
        assert_eq!(bank.reserves, 1500.0);
        assert!(bank.deposits.contains_key(&1));
    }

    #[test]
    fn test_loan_creation() {
        let mut bank = create_test_bank(0, "Test Bank", 1000.0);
        bank.accept_deposit(1, 1000.0); // Now has 2000 reserves

        let loan = Loan::new(2, 500.0, 0.05, 100, 100.0);
        assert!(bank.make_loan(loan));

        assert_eq!(bank.total_loans, 500.0);
        assert_eq!(bank.reserves, 1500.0); // 2000 - 500
    }

    #[test]
    fn test_loan_insufficient_reserves() {
        let mut bank = create_test_bank(0, "Test Bank", 100.0);
        bank.accept_deposit(1, 100.0); // 200 total reserves
        bank.reserve_ratio = 0.2; // 20% reserve requirement = 40 required

        // Try to loan more than available (200 - 40 = 160 available)
        let loan = Loan::new(2, 200.0, 0.05, 100, 50.0);
        assert!(!bank.make_loan(loan)); // Should fail
    }

    #[test]
    fn test_withdrawal_processing() {
        let mut bank = create_test_bank(0, "Test Bank", 1000.0);
        bank.accept_deposit(1, 500.0);

        let withdrawn = bank.process_withdrawal(1);
        assert_eq!(withdrawn, 500.0);
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(bank.total_deposits, 0.0);
        assert_eq!(bank.reserves, 1000.0);
    }

    #[test]
    fn test_withdrawal_queue() {
        let mut bank = create_test_bank(0, "Test Bank", 1000.0);
        bank.accept_deposit(1, 500.0);
        bank.accept_deposit(2, 500.0);

        bank.queue_withdrawal(1);
        bank.queue_withdrawal(2);

        assert_eq!(bank.withdrawal_queue.len(), 2);
    }

    #[test]
    fn test_solvency_calculation() {
        let mut bank = create_test_bank(0, "Test Bank", 100.0);
        bank.accept_deposit(1, 1000.0);

        // Initially solvent
        assert!(bank.solvency >= 1.0);

        // Create bad loans that wipe out equity
        // Bank has 1100 in assets (100 reserves + 1000 deposit), 1000 liabilities
        // Net worth = 100. Need to lose more than 100 to become insolvent.
        let mut bad_loan = Loan::new(2, 900.0, 0.05, 100, 0.0);
        bad_loan.mark_defaulted();
        bank.loans.push(bad_loan);
        bank.total_loans += 900.0;
        // Also drain reserves to make sure we're insolvent
        bank.reserves = 50.0;

        bank.update_health_metrics();

        // Should now be insolvent
        assert!(bank.solvency < 1.0);
    }

    #[test]
    fn test_bank_failure() {
        let mut bank = create_test_bank(0, "Test Bank", 100.0);
        bank.accept_deposit(1, 1000.0);

        // Simulate massive losses
        bank.reserves = 0.0;
        bank.update_health_metrics();
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        assert!(bank.check_failure());
        assert_eq!(bank.health_status, BankHealth::Failed);
    }

    #[test]
    fn test_banking_system_creation() {
        let system = BankingSystem::new(10000.0, 250000.0);
        assert_eq!(system.central_bank_liquidity, 10000.0);
        assert_eq!(system.deposit_insurance_limit, 250000.0);
        assert!(!system.run_in_progress);
    }

    #[test]
    fn test_add_bank_to_system() {
        let mut system = BankingSystem::new(10000.0, 250000.0);
        let bank = create_test_bank(0, "Bank A", 1000.0);
        let id = system.add_bank(bank);

        assert_eq!(id.0, 0);
        assert_eq!(system.banks.len(), 1);
    }

    #[test]
    fn test_interbank_network() {
        let mut system = BankingSystem::new(10000.0, 250000.0);

        let bank_a = create_test_bank(0, "Bank A", 1000.0);
        let bank_b = create_test_bank(1, "Bank B", 1000.0);

        let id_a = system.add_bank(bank_a);
        let id_b = system.add_bank(bank_b);

        system.add_interbank_link(id_a, id_b, 500.0);

        assert!(system.interbank_network.contains_key(&id_a));
    }

    #[test]
    fn test_fundamental_withdrawal_decision() {
        let mut system = BankingSystem::new(10000.0, 250000.0);

        let mut bank = create_test_bank(0, "Bank A", 100.0);
        bank.accept_deposit(1, 1000.0);
        bank.solvency = 0.8; // Make insolvent

        system.add_bank(bank);
        // TAG: surface=simulation owner=research-team rule=SIM-001

        // Should withdraw due to insolvency
        let should_withdraw = system.agent_withdrawal_decision(1, BankId(0), 0.9, 0.1, true); // fundamental_only=true
        assert!(should_withdraw);
    }

    #[test]
    fn test_healthy_bank_no_withdrawal() {
        let mut system = BankingSystem::new(10000.0, 250000.0);

        let bank = create_test_bank(0, "Bank A", 10000.0); // Very healthy
        system.add_bank(bank);

        // Should not withdraw from healthy bank
        let should_withdraw = system.agent_withdrawal_decision(1, BankId(0), 0.9, 0.1, true);
        assert!(!should_withdraw);
    }

    #[test]
    fn test_deposit_insurance_effect() {
        let mut system = BankingSystem::new(10000.0, 250000.0);
        system.deposit_insurance_active = true;

        let mut bank = create_test_bank(0, "Bank A", 1000.0);
        bank.accept_deposit(1, 10000.0); // Below insurance limit

        // Make bank stressed but not failed
        bank.liquidity = 0.08;
        bank.solvency = 1.0;

        system.add_bank(bank);

        // With insurance, less likely to withdraw
        let should_withdraw = system.agent_withdrawal_decision(1, BankId(0), 0.5, 0.5, false);
        assert!(!should_withdraw); // Insured deposit should prevent run
    }

    #[test]
    fn test_run_contagion() {
        let mut system = BankingSystem::new(10000.0, 250000.0);
        let mut rng = StdRng::seed_from_u64(42);

        // Create two connected banks
        let bank_a = create_test_bank(0, "Bank A", 1000.0);
        let bank_b = create_test_bank(1, "Bank B", 1000.0);

        let id_a = system.add_bank(bank_a);
        let id_b = system.add_bank(bank_b);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // Create interbank link
        system.add_interbank_link(id_a, id_b, 300.0);

        // Add depositors
        if let Some(bank) = system.banks.get_mut(0) {
            bank.accept_deposit(1, 500.0);
        }
        if let Some(bank) = system.banks.get_mut(1) {
            bank.accept_deposit(2, 500.0);
        }

        // Start run at bank A
        system.start_bank_run(id_a, 1, &mut rng);

        // Bank B should now be at risk
        assert!(system.run_in_progress);
    }

    #[test]
    fn test_central_bank_intervention() {
        let mut system = BankingSystem::new(10000.0, 250000.0);

        let mut bank = create_test_bank(0, "Bank A", 10000.0); // High initial reserves
        bank.accept_deposit(1, 5000.0);
        // Bank now has 15000 reserves, 5000 deposits
        // Reserve requirement = 10% of 5000 = 500
        // We need liquidity < reserve_ratio (0.1) but solvency > 0.95
        // Set reserves to 300: liquidity = 300/5000 = 0.06 (stressed)
        // But assets = 300 + loans, liabilities = 5000
        // Add loans to ensure solvency > 0.95
        let loan = Loan::new(2, 10000.0, 0.05, 100, 5000.0);
        bank.loans.push(loan);
        bank.total_loans += 10000.0;
        bank.reserves = 300.0;
        bank.health_status = BankHealth::Stressed;
        bank.update_health_metrics();

        // Verify bank is stressed but solvent
        assert_eq!(bank.health_status, BankHealth::Stressed);
        assert!(
            bank.solvency > 0.95,
            "Bank should be solvent: {}",
            bank.solvency
        );

        system.add_bank(bank);
        // TAG: surface=simulation owner=research-team rule=SIM-001

        let provided = system.central_bank_intervention();

        assert!(provided > 0.0, "Should provide liquidity");
        assert!(system.central_bank_liquidity < 10000.0);
    }

    #[test]
    fn test_sunspot_trigger() {
        let mut system = BankingSystem::new(10000.0, 250000.0);
        let mut rng = StdRng::seed_from_u64(42);

        // Create vulnerable bank
        let mut bank = create_test_bank(0, "Bank A", 100.0);
        bank.accept_deposit(1, 1000.0);
        bank.liquidity = 0.05; // Low liquidity

        system.add_bank(bank);
        system.sunspot_threshold = 0.01; // Make very sensitive

        // Try to trigger with high sunspot strength
        let _triggered = system.sunspot_trigger(1, &mut rng, 1.0);

        // With vulnerable system and high sunspot, should trigger
        // Note: May not always trigger due to randomness
        // Test mainly checks function doesn't panic
    }

    #[test]
    fn test_systemic_risk_metrics() {
        let mut system = BankingSystem::new(10000.0, 250000.0);

        let bank_a = create_test_bank(0, "Bank A", 1000.0);
        let bank_b = create_test_bank(1, "Bank B", 1000.0);

        system.add_bank(bank_a);
        system.add_bank(bank_b);

        system.compute_systemic_risk_metrics();

        assert!(system.risk_metrics.capital_adequacy > 0.0);
        assert_eq!(system.risk_metrics.failed_banks, 0);
    }

    #[test]
    fn test_multiple_equilibria_existance() {
        // Test that the system can exhibit both good and bad equilibria
        // Per Diamond-Dybvig (1983): Multiple equilibria exist due to coordination
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // Good equilibrium: healthy bank, patient depositors wait
        {
            let mut system = BankingSystem::new(10000.0, 250000.0);
            let mut bank = create_test_bank(0, "Bank A", 10000.0);
            bank.accept_deposit(1, 5000.0);
            bank.update_health_metrics();
            system.add_bank(bank);

            // High trust patient agent in healthy bank - should wait
            let withdraw = system.agent_withdrawal_decision(1, BankId(0), 0.9, 0.1, true);
            assert!(!withdraw, "Good equilibrium: patient depositor should wait");
        }

        // Bad equilibrium: run in progress, impatient agents withdraw
        {
            let mut system = BankingSystem::new(10000.0, 250000.0);
            system.deposit_insurance_active = false;

            let mut bank = create_test_bank(0, "Bank A", 500.0); // Lower reserves
            bank.accept_deposit(1, 5000.0);
            // Liquidity = 5500/5000 = 1.1, still healthy
            // But set liquidity lower to increase fundamental concern slightly
            bank.reserves = 200.0; // liquidity = 0.24, still above reserve_ratio * 0.5
            bank.update_health_metrics();
            system.add_bank(bank);

            // Start a run
            system.run_in_progress = true;
            system.banks_in_run.push(BankId(0));

            // Low trust, high impatience during run - should withdraw
            // This demonstrates the bad equilibrium
            let withdraw = system.agent_withdrawal_decision(1, BankId(0), 0.1, 1.0, false);
            assert!(withdraw, "Bad equilibrium: coordination failure causes run");
        }
    }

    #[test]
    fn test_credit_freeze_during_run() {
        use crate::economics::EconomicState;

        let mut system = BankingSystem::new(10000.0, 250000.0);
        let mut econ_state = EconomicState {
            credit_access: 0.8,
            ..Default::default()
        };
        // TAG: surface=simulation owner=research-team rule=SIM-001

        // Normal credit access
        let normal_access = econ_state.credit_access;
        assert!(normal_access > 0.0);

        // Trigger run
        system.run_in_progress = true;
        econ_state.update_credit_with_banking_system(&system);

        // Credit should be reduced (multiplied by 0.2)
        assert!(econ_state.credit_access < normal_access);
        assert!((econ_state.credit_access - 0.16).abs() < 0.01); // 0.8 * 0.2 = 0.16
    }

    #[test]
    fn test_loan_expected_value() {
        let mut loan = Loan::new(1, 1000.0, 0.05, 100, 200.0);
        loan.default_probability = 0.1;

        let expected = loan.expected_value();
        // Expected = 1000 * 1.05 * 0.9 + 200 * 0.1 = 945 + 20 = 965
        assert!(expected > 900.0 && expected < 1000.0);

        // After default
        loan.mark_defaulted();
        assert_eq!(loan.expected_value(), 200.0); // Just collateral
    }

    #[test]
    fn test_network_density() {
        let mut system = BankingSystem::new(10000.0, 250000.0);

        // Create 3 banks
        for i in 0..3 {
            let bank = create_test_bank(i, &format!("Bank {i}"), 1000.0);
            system.add_bank(bank);
        }

        // Add some links
        system.add_interbank_link(BankId(0), BankId(1), 100.0);
        system.add_interbank_link(BankId(1), BankId(2), 100.0);

        system.compute_systemic_risk_metrics();

        // 2 links out of 6 possible (3 * 2)
        let density = system.risk_metrics.network_density;
        assert!(density > 0.0 && density <= 1.0);
    }

    #[test]
    fn test_bank_run_queue_pressure_and_unique_failed_bank_insurance() {
        let mut system = BankingSystem::new(10000.0, 250000.0);
        let mut rng = StdRng::seed_from_u64(12345);

        let mut bank = create_test_bank(0, "Bank Queue Test", 250.0);
        for i in 1..=4 {
            bank.accept_deposit(i, 100.0);
        }
        // Queue 2 depositors: initial_queue_len = 2, deposits = 4 => queue_pressure = 2 / (4 + 1) = 0.40 > 0.25!
        bank.queue_withdrawal(1);
        bank.queue_withdrawal(2);
        system.add_bank(bank);

        system.process_withdrawals(1, &mut rng);

        // Bank run must be triggered by queue pressure
        assert!(
            system.run_in_progress,
            "Bank run must be triggered by queue pressure"
        );

        // Now drain reserves to trigger failure and verify deduplicated failure recording and insurance payout
        system.banks[0].reserves = 0.0;
        system.banks[0].queue_withdrawal(3);
        system.banks[0].queue_withdrawal(4);
        system.process_withdrawals(2, &mut rng);

        // Failed banks list must contain bank 0 exactly once (deduplicated)
        assert_eq!(
            system.failed_banks,
            vec![BankId(0)],
            "Failed banks must contain bank 0 exactly once without duplicates"
        );

        let payout = system.process_deposit_insurance();
        assert!(payout > 0.0);
    }
}
