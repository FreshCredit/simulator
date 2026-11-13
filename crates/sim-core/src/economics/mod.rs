//! Economics module: Credit-Innovation feedback loop
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-ECON-001: Economic state centralization (all metrics in `EconomicState` struct)
//! SIM-003: Terminology - SIMULATION economics, NOT financial advice
//!
//! Per docs/simulation/economics.md: Unified Theory of Economics
//! Core theorem: Economic phenomena are downstream of τ (trust) and bₜ (bond density)
//!
//! Economic Feedback Loop (per simulation-plan.md):
//! Credit Access → Debt Burden → Crime Rate → Community Engagement →
//! Education → Innovation (energy efficiency) → Credit Repayment → System Rebalance

// TAG: surface=simulation owner=research-team rule=SIM-001
pub mod banking;
pub mod goods;
pub mod market;
pub mod prospect_theory;
pub mod time_preference;
pub mod util;

pub use goods::*;
pub use market::*;
pub use prospect_theory::*;
use serde::{Deserialize, Serialize};

pub use time_preference::*;

use util::kahan_sum;

/// Agent-level economic state
/// SIM-ECON-001: All economic metrics centralized here
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EconomicState {
    /// Credit access level [0.0, 1.0]
    /// 0.0 = no credit access, 1.0 = full credit access
    /// Reputation-gated: requires τ above threshold
    pub credit_access: f32,

    /// Current debt burden [0.0, ∞)
    /// Ratio of debt to income/resources
    /// High debt → stress (cortisol) → lower τ → reduced credit
    pub debt_burden: f32,

    /// Innovation contribution [0.0, 1.0]
    /// Reduces energy consumption cost (efficiency gain)
    /// Per economics.md: Innovation emerges from education + community
    pub innovation_contribution: f32,

    /// Wealth accumulation (abstract units)
    /// Used for Gini calculation and growth metrics
    pub wealth: f32,

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Credit score proxy (bₜ metric per agent)
    /// Bond density: fraction of commitments honored
    pub bond_density: f32,

    // === ECON-P0-1: GDP Model Extensions ===
    /// Per-tick income from economic activity
    /// GDP per agent component
    pub income: f32,

    /// Per-tick consumption expenditure
    pub consumption: f32,

    /// Probability of default on debt [0.0, 1.0]
    /// Computed from debt burden and trust
    pub default_probability: f32,

    /// Whether agent actually defaulted (ground truth for AUC)
    pub actual_default: bool,

    /// Historical GDP contribution (smoothed)
    pub gdp_contribution: f32,

    // === Plaid Financial Timeseries & Dual-Estimator Sequestration Wall ===
    /// Market-observed trust score S_mkt ∈ [0, 1] (Report -> Score -> Offer pipeline)
    /// Driven exclusively by financial validation events (on-time settlements, cash-flow stability)
    pub market_trust_score: f32,

    /// Plaid cash-flow liquidity buffer in days of baseline consumption
    pub cashflow_buffer_days: f32,

    /// Plaid spend volatility: rolling coefficient of variation of expenditure
    pub spend_volatility: f32,

    /// Plaid non-sufficient funds / overdraft penalty event counter
    pub nsf_overdraft_count: u32,

    /// Ratio of on-time debt settlements [0.0, 1.0]
    pub on_time_payment_rate: f32,

    /// Dual-layer Concordance Statistic: correlation (Mantel r) between
    /// physiological biomarker trajectory and financial market score trajectory
    pub concordance_mantel_r: f32,
}

impl Default for EconomicState {
    fn default() -> Self {
        Self {
            credit_access: 0.0,           // Start with no credit (must earn via reputation)
            debt_burden: 0.0,             // No initial debt
            innovation_contribution: 0.0, // No initial innovation
            wealth: 10.0,                 // Base starting wealth
            bond_density: 0.5,            // Neutral bond density (no history)
            // ECON-P0-1 extensions
            income: 1.0,      // Base income
            consumption: 0.8, // Base consumption (80% of income)
            default_probability: 0.0,
            actual_default: false,
            gdp_contribution: 0.0,
            market_trust_score: 0.50,
            cashflow_buffer_days: 12.5,
            spend_volatility: 0.18,
            nsf_overdraft_count: 0,
            on_time_payment_rate: 1.0,
            concordance_mantel_r: 0.88,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl EconomicState {
    /// Update credit access based on banking system health.
    pub fn update_credit_with_banking_system(&mut self, banking: &banking::BankingSystem) {
        if banking.run_in_progress {
            // Credit freezes during runs
            self.credit_access *= 0.2;
            self.innovation_contribution *= 0.5;
        } else {
            // Healthy banks enable credit
            let health_factor = banking.risk_metrics.capital_adequacy.min(1.0);
            self.credit_access = self.credit_access.min(health_factor);
        }
    }

    /// Effective credit access considering bank runs.
    pub fn effective_credit_access(&self, banking: &banking::BankingSystem) -> f32 {
        if banking.run_in_progress {
            self.credit_access * 0.2
        } else {
            self.credit_access
                .min(banking.risk_metrics.capital_adequacy)
        }
    }

    /// Create new economic state with specified values
    #[must_use]
    pub fn new(
        credit_access: f32,
        debt_burden: f32,
        innovation_contribution: f32,
        wealth: f32,
        bond_density: f32,
    ) -> Self {
        Self {
            credit_access: credit_access.clamp(0.0, 1.0),
            debt_burden: debt_burden.max(0.0),
            innovation_contribution: innovation_contribution.clamp(0.0, 1.0),
            wealth: wealth.max(0.0),
            bond_density: bond_density.clamp(0.0, 1.0),
            // New fields with defaults
            income: 1.0,
            consumption: 0.8,
            default_probability: 0.0,
            actual_default: false,
            gdp_contribution: 0.0,
            market_trust_score: 0.50,
            cashflow_buffer_days: 12.5,
            spend_volatility: 0.18,
            nsf_overdraft_count: 0,
            on_time_payment_rate: 1.0,
            concordance_mantel_r: 0.88,
        }
    }

    /// Compute effective energy cost modifier from innovation
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Higher innovation → lower energy consumption per tick
    /// Per economics.md: Innovation = energy efficiency
    #[must_use]
    pub fn energy_cost_modifier(&self) -> f32 {
        // Range: [0.7, 1.0] - max 30% energy savings from innovation
        1.0 - (self.innovation_contribution * 0.3)
    }

    /// Compute stress contribution from debt burden
    /// High debt → higher cortisol → lower trust
    /// Per economics.md: Debt burden → stress → crime
    #[must_use]
    pub fn debt_stress(&self) -> f32 {
        let cushion = (1.0 + (self.wealth.max(0.0) / 100.0).ln_1p()).max(1.0);
        let effective_burden = self.debt_burden / cushion;
        // Sigmoid function: stress increases slowly at first, then rapidly
        1.0 / (1.0 + (-effective_burden + 1.5).exp())
    }

    /// Update bond density based on commitment outcome
    /// Per economics.md: bₜ = fraction of agents whose word is their bond
    pub fn record_commitment(&mut self, honored: bool) {
        // Exponential moving average with α = 0.1
        let outcome = if honored { 1.0 } else { 0.0 };
        self.bond_density = 0.9 * self.bond_density + 0.1 * outcome;
        if honored {
            self.credit_access = (self.credit_access + 0.01).min(1.0);
        }
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Compute implied interest rate from trust
    /// Per economics.md: r ≈ 1/τ - 1
    #[must_use]
    pub fn implied_interest_rate(tau: f32) -> f32 {
        if tau <= 0.01 {
            return 100.0; // Cap at 10000% for near-zero trust
        }
        (1.0 / tau - 1.0).clamp(0.0, 100.0)
    }

    /// Compute default probability from debt burden and trust
    /// Per ECON-P0-2: Crisis threshold detection
    pub fn compute_default_probability(&mut self, tau: f32) {
        // Higher debt burden → higher default probability
        // Lower trust → higher default probability
        let debt_factor = (self.debt_burden / (1.0 + self.debt_burden)).min(1.0);
        let trust_factor = (1.0 - tau).clamp(0.0, 1.0);

        self.default_probability = (0.6 * debt_factor + 0.4 * trust_factor).clamp(0.0, 1.0);
    }

    /// Update income based on trust and bond density
    /// Per ECON-P0-1: GDP = f(b × τ)
    pub fn update_income(&mut self, tau: f32) {
        // Base income scaled by economic activity
        // Higher trust and bond density → higher income
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let economic_factor = (self.bond_density * tau).sqrt();
        self.income = (1.0 + 2.0 * economic_factor).clamp(0.5, 5.0);

        // Update GDP contribution (smoothed)
        self.gdp_contribution = 0.9 * self.gdp_contribution + 0.1 * self.income;
    }

    /// Update consumption (typically 70-90% of income)
    pub fn update_consumption(&mut self) {
        // Consumption decreases with high debt burden (precautionary savings)
        let savings_rate = 0.1 + 0.3 * (self.debt_burden / (1.0 + self.debt_burden));
        self.consumption = (self.income * (1.0 - savings_rate)).clamp(0.0, self.income);
    }

    /// Check for actual default and update state
    /// Returns true if default occurred
    pub fn check_default(&mut self, rng: &mut impl rand::Rng) -> bool {
        let roll: f32 = rng.gen();
        self.actual_default = roll < self.default_probability;

        if self.actual_default {
            // Default consequences: lose credit access, increase debt burden
            self.credit_access = 0.0;
            self.debt_burden *= 1.5; // Penalty
        }

        self.actual_default
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute savings rate
    #[must_use]
    pub fn savings_rate(&self) -> f32 {
        if self.income <= 0.0 {
            0.0
        } else {
            (self.income - self.consumption) / self.income
        }
    }
}

/// Group-level economic metrics
/// Tracks aggregate economic health of a group/community
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupEconomics {
    /// Group identifier
    pub group_id: u64,

    /// Mean trust (τ) of group members
    pub mean_tau: f32,

    /// Bond density (bₜ) of group
    pub bond_density: f32,

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Gini coefficient of wealth inequality [0, 1]
    /// Per economics.md: Gini ∝ 1 - bₜ
    pub gini: f32,

    /// Aggregate wealth of group
    pub total_wealth: f32,

    /// Community engagement level [0, 1]
    /// Derived from social interactions and group cohesion
    pub community_engagement: f32,

    /// Education/skill level [0, 1]
    /// Affects innovation contribution
    pub education_level: f32,

    /// Crime rate [0, 1]
    /// Inverse of trust and bond density
    /// Per economics.md: Low bₜ → high crime
    pub crime_rate: f32,
}

impl Default for GroupEconomics {
    fn default() -> Self {
        Self {
            group_id: 0,
            mean_tau: 0.5,
            bond_density: 0.5,
            gini: 0.3,
            total_wealth: 0.0,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            community_engagement: 0.5,
            education_level: 0.5,
            crime_rate: 0.1,
        }
    }
}

impl GroupEconomics {
    /// Create new group economics with specified group ID
    #[must_use]
    pub fn new(group_id: u64) -> Self {
        Self {
            group_id,
            ..Default::default()
        }
    }

    /// Update group economics from member states
    /// SIM-CREDIT-001: Credit requires reputation threshold
    pub fn update_from_members(&mut self, members: &[(f32, f32, f32)]) {
        // members: Vec of (tau, wealth, bond_density) tuples
        if members.is_empty() {
            return;
        }

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let n = members.len() as f32;

        // Mean tau - use Kahan summation for precision
        let tau_values: Vec<f32> = members.iter().map(|(tau, _, _)| *tau).collect();
        self.mean_tau = kahan_sum(&tau_values) / n;

        // Mean bond density - use Kahan summation for precision
        let bd_values: Vec<f32> = members.iter().map(|(_, _, bd)| *bd).collect();
        self.bond_density = kahan_sum(&bd_values) / n;

        // Total wealth - use Kahan summation for precision
        let wealth_values: Vec<f32> = members.iter().map(|(_, w, _)| *w).collect();
        self.total_wealth = kahan_sum(&wealth_values);

        // Gini coefficient (simplified calculation)
        // Per economics.md: Gini ∝ 1 - bₜ
        // Also compute from actual wealth distribution
        self.gini = compute_gini(&wealth_values);

        // Crime rate: inverse of bond density and trust
        // Per economics.md: Low bₜ → high extractive rent → crime
        self.crime_rate = ((1.0 - self.bond_density) * (1.5 - self.mean_tau)).clamp(0.0, 1.0);

        // Community engagement: derived from trust and interaction patterns
        self.community_engagement = (self.mean_tau * self.bond_density).clamp(0.0, 1.0);

        // Education: grows with community engagement, decays with crime
        // This creates the feedback loop: engagement → education → innovation
        self.education_level =
            (0.5 + 0.3 * self.community_engagement - 0.2 * self.crime_rate).clamp(0.0, 1.0);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute growth rate prediction
    /// Per economics.md: d(GDP)/dt ∝ bₜ × τ
    #[must_use]
    pub fn predicted_growth_rate(&self) -> f32 {
        self.bond_density * self.mean_tau
    }

    /// Check for crisis conditions
    /// Per economics.md: Crisis when τ drops below threshold (~0.62)
    #[must_use]
    pub fn is_crisis(&self) -> bool {
        self.mean_tau < 0.62 && self.gini > 0.5
    }
}

/// Compute Gini coefficient from wealth distribution
/// Standard formula: G = (2 * Σ(i * wᵢ)) / (n * Σwᵢ) - (n + 1) / n
/// Uses Kahan summation for precision in economic aggregations
#[must_use]
pub fn compute_gini(wealth: &[f32]) -> f32 {
    if wealth.is_empty() {
        return 0.0;
    }

    let n = wealth.len() as f32;
    if n == 0.0 {
        return 0.0;
    }

    let mut sorted = wealth.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // Use Kahan summation for total wealth
    let total: f32 = kahan_sum(&sorted);
    if total <= 0.0 || !total.is_finite() {
        return 0.0;
    }

    // Use Kahan summation for weighted sum
    let weighted_values: Vec<f32> = sorted
        .iter()
        .enumerate()
        .map(|(i, w)| (i as f32 + 1.0) * w)
        .collect();
    let weighted_sum: f32 = kahan_sum(&weighted_values);
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    // Guard against division by zero
    let denominator = n * total;
    if denominator == 0.0 || !denominator.is_finite() {
        return 0.0;
    }

    let gini = (2.0 * weighted_sum) / denominator - (n + 1.0) / n;
    gini.clamp(0.0, 1.0)
}

/// SIM-CREDIT-001: Credit access requires reputation threshold
/// Returns credit access level based on trust and bond density
#[must_use]
pub fn compute_credit_access(tau: f32, bond_density: f32, reputation: f32) -> f32 {
    // Minimum thresholds for credit access
    const TAU_THRESHOLD: f32 = 0.5; // Minimum trust for any credit
    const BD_THRESHOLD: f32 = 0.4; // Minimum bond density
    const REP_THRESHOLD: f32 = 0.3; // Minimum reputation

    if tau < TAU_THRESHOLD || bond_density < BD_THRESHOLD || reputation < REP_THRESHOLD {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        return 0.0;
    }

    // Credit access scales with composite score
    let composite = 0.4 * tau + 0.3 * bond_density + 0.3 * reputation;

    // Map composite [0.4, 1.0] to credit access [0.0, 1.0]
    ((composite - 0.4) / 0.6).clamp(0.0, 1.0)
}

/// Update agent credit access based on current state
/// SIM-CREDIT-001: Reputation-gated credit
pub fn update_credit_access(economic_state: &mut EconomicState, tau: f32, reputation: f32) {
    economic_state.credit_access =
        compute_credit_access(tau, economic_state.bond_density, reputation);
}

/// Apply innovation contribution based on group education level
/// Per economics.md: Education → Innovation → Energy efficiency
#[must_use]
pub fn compute_innovation(education: f32, community_engagement: f32) -> f32 {
    // Innovation = f(education, community)
    // Both factors contribute, but education is primary
    (0.6 * education + 0.4 * community_engagement).clamp(0.0, 1.0)
}

// ============================================================================
// ECON-P0-1: GDP Model
// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Population-level GDP metrics
/// Per ECON-P0-1: GDP = f(b × τ) with R² > 0.85
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PopulationGdp {
    /// Total GDP (sum of all agent contributions)
    pub total_gdp: f32,
    /// GDP per capita
    pub gdp_per_capita: f32,
    /// GDP growth rate
    pub growth_rate: f32,
    /// Aggregate bond density × mean trust (the b × τ model)
    pub bond_trust_product: f32,
    /// Model R² (target: > 0.85)
    pub model_r_squared: f32,
    /// Investment (aggregate savings)
    pub total_investment: f32,
    /// Aggregate consumption
    pub total_consumption: f32,
    /// Default rate (fraction of agents who defaulted)
    pub default_rate: f32,
}
// TAG: surface=simulation owner=platform-team rule=GENERAL-001

impl PopulationGdp {
    /// Compute GDP from population economic states
    /// Per ECON-P0-1: GDP is predicted by b × τ
    /// Uses Kahan summation for precision in economic aggregations
    #[must_use]
    pub fn from_agents(agents: &[(EconomicState, f32)]) -> Self
where {
        if agents.is_empty() {
            return Self::default();
        }

        let n = agents.len() as f32;

        // Aggregate metrics using Kahan summation for precision
        let gdp_values: Vec<f32> = agents.iter().map(|(e, _)| e.gdp_contribution).collect();
        let total_gdp: f32 = kahan_sum(&gdp_values);

        let consumption_values: Vec<f32> = agents.iter().map(|(e, _)| e.consumption).collect();
        let total_consumption: f32 = kahan_sum(&consumption_values);

        let investment_values: Vec<f32> = agents
            .iter()
            .map(|(e, _)| e.income - e.consumption)
            .collect();
        let total_investment: f32 = kahan_sum(&investment_values);

        let total_defaults: f32 = agents.iter().filter(|(e, _)| e.actual_default).count() as f32;

        // Mean bond density and trust using Kahan summation
        let bond_density_values: Vec<f32> = agents.iter().map(|(e, _)| e.bond_density).collect();
        let mean_bond_density: f32 = kahan_sum(&bond_density_values) / n;
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        let tau_values: Vec<f32> = agents.iter().map(|(_, tau)| *tau).collect();
        let mean_tau: f32 = kahan_sum(&tau_values) / n;
        let bond_trust_product = mean_bond_density * mean_tau;

        // Predicted GDP from model
        let predicted_gdp = bond_trust_product * n * 2.0; // Scaling factor

        // Compute R² (simplified - actual would use historical data)
        let ss_res = (total_gdp - predicted_gdp).powi(2);
        let ss_tot = total_gdp.powi(2); // Simplified
        let r_squared = if ss_tot > 0.0 {
            1.0 - (ss_res / ss_tot)
        } else {
            0.85 // Default target
        };

        Self {
            total_gdp,
            gdp_per_capita: total_gdp / n,
            growth_rate: 0.0, // Would compute from history
            bond_trust_product,
            model_r_squared: r_squared.clamp(0.0, 1.0),
            total_investment: total_investment.max(0.0),
            total_consumption,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            default_rate: total_defaults / n,
        }
    }

    /// Validate ECON-P0-1: R² > 0.85
    #[must_use]
    pub fn validate_model(&self) -> bool {
        self.model_r_squared > 0.85
    }
}

/// Crisis detection per ECON-P0-2
/// Returns true if economic crisis conditions met
#[must_use]
pub fn detect_crisis(mean_tau: f32, default_rate: f32, gini: f32) -> bool {
    // Crisis when: τ < 0.68 AND (high defaults OR high inequality)
    let low_trust = mean_tau < 0.68;
    let high_defaults = default_rate > 0.1; // > 10% default rate
    let high_inequality = gini > 0.5;

    low_trust && (high_defaults || high_inequality)
}

/// Compute crisis severity [0.0, 1.0]
#[must_use]
pub fn crisis_severity(mean_tau: f32, default_rate: f32) -> f32 {
    let trust_component = (0.68 - mean_tau.clamp(0.0, 0.68)) / 0.68;
    let default_component = (default_rate / 0.2).min(1.0); // Cap at 20% default rate

    (0.5 * trust_component + 0.5 * default_component).clamp(0.0, 1.0)
}

// =============================================================================
// GAP-003: Endowment Effect Implementation
