//! Costly Signaling Theory (P2-RES-1)
//!
//! Signals of quality and commitment (Zahavi).

use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Signaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalingConfig {
    /// signal costs.
    pub signal_costs: SignalCosts,
    /// receiver skepticism.
    pub receiver_skepticism: f32,
}

impl Default for SignalingConfig {
    fn default() -> Self {
        Self {
            signal_costs: SignalCosts::default(),
            receiver_skepticism: 0.3,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Costs for different signal types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalCosts {
    /// monetary.
    pub monetary: f32,
    /// time.
    pub time: f32,
    /// effort.
    pub effort: f32,
    /// risk.
    pub risk: f32,
}

impl Default for SignalCosts {
    fn default() -> Self {
        Self {
            monetary: 1.0,
            time: 0.5,
            effort: 0.8,
            risk: 1.2,
        }
    }
}

/// A costly signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    /// signal type.
    pub signal_type: SignalType,
    /// cost.
    pub cost: f32,
    /// sender quality.
    pub sender_quality: f32,
    /// honesty.
    pub honesty: f32, // 0-1, probability signal is honest
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Types of costly signals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalType {
    /// Gifts and generosity
    GiftGiving,
    /// Risk-taking displays
    RiskTaking,
    /// Time investment
    TimeInvestment,
    /// Elaborate displays
    Display,
    /// Public commitments
    Commitment,
}

/// Signaling game
#[derive(Debug, Clone)]
pub struct SignalingGame {
    /// config.
    pub config: SignalingConfig,
}

impl SignalingGame {
    #[must_use]
    /// new.
    pub const fn new(config: SignalingConfig) -> Self {
        Self { config }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate signal intensity based on sender quality
    #[must_use]
    pub fn signal_intensity(&self, quality: f32, signal_type: SignalType) -> f32 {
        let base_cost = match signal_type {
            SignalType::GiftGiving => self.config.signal_costs.monetary,
            SignalType::RiskTaking => self.config.signal_costs.risk,
            SignalType::TimeInvestment => self.config.signal_costs.time,
            SignalType::Display => self.config.signal_costs.effort,
            SignalType::Commitment => self.config.signal_costs.risk,
        };

        // Higher quality senders can afford stronger signals
        quality * base_cost * 2.0
    }

    /// Evaluate signal credibility from receiver's perspective
    #[must_use]
    pub fn evaluate_signal(&self, signal: &Signal, receiver_sophistication: f32) -> f32 {
        // Costly signals are more credible
        let cost_credibility = signal.cost.min(1.0);

        // Check if signal is affordable for claimed quality
        let affordable = if signal.sender_quality > 0.0 {
            signal.cost <= signal.sender_quality * 3.0
        } else {
            false
        };

        let honesty_factor = if affordable { signal.honesty } else { 0.0 };

        // Sophisticated receivers are better at detecting false signals
        let detection = receiver_sophistication * self.config.receiver_skepticism;

        cost_credibility * honesty_factor * (1.0 - detection * 0.3)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate handicap principle effect
    /// Only high quality individuals can afford costly signals
    #[must_use]
    pub fn handicap_principle(&self, signal_cost: f32, sender_quality: f32) -> bool {
        // High quality sender can afford costly signals
        // Cost is affordable if: sender_quality >= signal_cost * quality_threshold
        sender_quality >= signal_cost * 0.8
    }
}

impl Default for SignalingGame {
    fn default() -> Self {
        Self::new(SignalingConfig::default())
    }
}

/// Trust signal through costly action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustSignal {
    /// sender.
    pub sender: u64,
    /// recipient.
    pub recipient: u64,
    /// vulnerability.
    pub vulnerability: f32, // How much sender exposes themselves
    /// commitment value.
    pub commitment_value: f32,
}

// TAG: surface=simulation owner=platform-team rule=GENERAL-001
impl TrustSignal {
    /// Calculate trustworthiness based on signal cost
    #[must_use]
    pub fn trustworthiness(&self) -> f32 {
        // Higher vulnerability = stronger signal
        self.vulnerability * self.commitment_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_intensity() {
        let game = SignalingGame::default();
        let intensity = game.signal_intensity(0.8, SignalType::GiftGiving);
        assert!(intensity > 0.0);
    }

    #[test]
    fn test_evaluate_signal() {
        let game = SignalingGame::default();
        let signal = Signal {
            signal_type: SignalType::RiskTaking,
            cost: 0.8,
            sender_quality: 0.9,
            honesty: 0.95,
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let credibility = game.evaluate_signal(&signal, 0.7);
        assert!(credibility > 0.0);
    }

    #[test]
    fn test_handicap_principle() {
        let game = SignalingGame::default();

        // High quality sender (0.9) can afford moderate cost signal (0.5)
        // 0.9 >= 0.5 * 0.8 = 0.4 ✓
        assert!(game.handicap_principle(0.5, 0.9));

        // Low quality sender (0.2) cannot afford high cost signal (0.9)
        // 0.2 >= 0.9 * 0.8 = 0.72 ✗
        assert!(!game.handicap_principle(0.9, 0.2));
    }

    #[test]
    fn test_trust_signal() {
        let signal = TrustSignal {
            sender: 1,
            recipient: 2,
            vulnerability: 0.7,
            commitment_value: 0.8,
        };

        assert!(signal.trustworthiness() > 0.0);
    }
}
