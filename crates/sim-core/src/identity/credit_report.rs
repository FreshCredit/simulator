// TAG: surface=simulation owner=research-team rule=SIM-001
//! Credit report, errors, and risk profile modeling

use serde::{Deserialize, Serialize};

/// Credit report error severity levels
/// Per `empirical_parameters`: 44% error rate, 14.5% serious errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CreditErrorSeverity {
    /// No errors on credit report
    #[default]
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    None,
    /// Minor errors (< 5% impact on reputation)
    Minor,
    /// Moderate errors (5-15% impact)
    Moderate,
    /// Serious errors (> 15% impact, 14.5% of total population)
    Serious,
}

impl CreditErrorSeverity {
    /// Get reputation damage multiplier for this severity
    #[must_use]
    pub const fn reputation_damage(&self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Minor => 0.05,
            Self::Moderate => 0.12,
            Self::Serious => 0.25,
        }
    }

    /// Get opportunity cost multiplier for this severity
    #[must_use]
    pub const fn opportunity_cost_multiplier(&self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Minor => 0.10,
            Self::Moderate => 0.20,
            Self::Serious => 0.40,
        }
    }
}

/// Types of credit report errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreditErrorType {
    /// Incorrect identity information
    IdentityMismatch,
    /// Errors in account status or balance
    AccountError,
    /// Fraudulent accounts opened in agent's name
    FraudulentAccount,
    /// Outdated or stale information
    OutdatedInfo,
    /// Incorrect payment history
    PaymentHistoryError,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Credit report status for an agent
/// Per GAP-002: Credit report error cascade implementation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreditReport {
    /// Whether the credit report has any errors
    pub has_error: bool,
    /// Severity of the error(s)
    pub severity: CreditErrorSeverity,
    /// Types of errors present
    pub error_types: Vec<CreditErrorType>,
    /// When the error was first reported (tick)
    pub error_reported_tick: Option<u64>,
    /// Whether the error has been disputed/resolved
    pub is_resolved: bool,
}

impl CreditReport {
    /// Create a clean credit report
    #[must_use]
    pub const fn clean() -> Self {
        Self {
            has_error: false,
            severity: CreditErrorSeverity::None,
            error_types: Vec::new(),
            error_reported_tick: None,
            is_resolved: true,
        }
    }

    /// Create a credit report with specified error
    #[must_use]
    pub const fn with_error(
        severity: CreditErrorSeverity,
        error_types: Vec<CreditErrorType>,
        tick: u64,
    ) -> Self {
        Self {
            has_error: true,
            severity,
            error_types,
            error_reported_tick: Some(tick),
            is_resolved: false,
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    /// Calculate reputation damage from this credit report
    #[must_use]
    pub const fn reputation_damage(&self) -> f32 {
        if !self.has_error || self.is_resolved {
            0.0
        } else {
            self.severity.reputation_damage()
        }
    }

    /// Calculate opportunity cost from this credit report
    #[must_use]
    pub fn opportunity_cost(&self, base_opportunity: f32) -> f32 {
        if !self.has_error || self.is_resolved {
            0.0
        } else {
            base_opportunity * self.severity.opportunity_cost_multiplier()
        }
    }

    /// Mark error as resolved
    pub fn resolve(&mut self) {
        self.is_resolved = true;
        self.has_error = false;
        self.severity = CreditErrorSeverity::None;
    }
}

/// Statistics for credit report errors across the simulation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreditReportStats {
    /// Total number of agents with credit report errors
    pub total_with_errors: u32,
    /// Number of agents with serious errors
    pub serious_errors: u32,
    /// Number of agents with moderate errors
    pub moderate_errors: u32,
    /// Number of agents with minor errors
    pub minor_errors: u32,
    /// Total reputation damage from errors
    pub total_reputation_damage: f32,
    /// Total opportunity cost from errors
    pub total_opportunity_cost: f32,
    /// Current error rate (% of agents affected)
    pub current_error_rate: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Risk profile for identity theft victimization
/// Per BJS 2016: Victims cluster by demographics and prior victimization
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RiskProfile {
    /// Income level proxy (0.0-1.0, higher = more risk)
    /// BJS: $75K+ households at higher risk
    pub income_proxy: f32,

    /// Age risk factor (0.0-1.0)
    /// BJS: Ages 35-64 at highest risk
    pub age_risk: f32,

    /// Number of prior victimizations
    /// BJS: 72% of victims are revictimized
    pub prior_victimizations: u32,

    /// Network exposure factor (0.0-1.0)
    /// Based on engagement network centrality
    pub network_exposure: f32,

    /// Protocol protection level (0.0-1.0)
    pub protocol_protection: f32,
}

impl Default for RiskProfile {
    fn default() -> Self {
        Self {
            income_proxy: 0.5,
            age_risk: 0.5,
            prior_victimizations: 0,
            network_exposure: 0.0,
            protocol_protection: 0.0,
        }
    }
}

impl RiskProfile {
    /// Compute composite risk score (0.0-1.0)
    /// Weights: Income 30%, Age 20%, Prior 40%, Network 10%
    #[must_use]
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub fn compute_risk_score(&self) -> f32 {
        let base_risk =
            0.30 * self.income_proxy + 0.20 * self.age_risk + 0.10 * self.network_exposure;

        // Prior victimization multiplier (3x risk after first theft)
        let victimization_multiplier = 1.0 + (self.prior_victimizations as f32 * 2.0);

        let risk = base_risk * victimization_multiplier;

        // Protocol protection reduces risk
        let protection_factor = 1.0 - (self.protocol_protection * 0.5);

        (risk * protection_factor).min(1.0)
    }

    /// Record a new victimization
    pub fn record_victimization(&mut self) {
        self.prior_victimizations += 1;
    }
}
