//! Oracle Implementation using Brian2

use super::{DocasMetric, Market, MarketId};
use crate::neural::brian2_backend::Brian2Backend;
use crate::types::{AgentId, Tick};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Neural oracle for market resolution
pub struct NeuralOracle {
    /// Brian2 backend for ground truth
    brian2: Brian2Backend,
    /// Resolution cache (avoid recomputing) keyed by (AgentId, Tick, Metric)
    cache: std::collections::BTreeMap<(AgentId, Tick, DocasMetric), f64>,
}

impl NeuralOracle {
    /// Create new oracle
    pub fn new(brian2: Brian2Backend) -> Self {
        Self {
            brian2,
            cache: std::collections::BTreeMap::new(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Resolve a market
    pub fn resolve(&mut self, market: &Market) -> Result<Resolution, OracleError> {
        let target = market.target_agent();
        let tick = market.prediction_tick();
        let metric = market.metric();

        // Check cache with distinct (target, tick, metric) key
        if let Some(&value) = self.cache.get(&(target, tick, metric)) {
            return Ok(Resolution {
                market_id: market.id(),
                agent_id: target,
                metric,
                actual_value: value,
                timestamp: tick,
            });
        }

        // Run Brian2 simulation or deterministic metric resolution
        let actual_value = self.simulate_docas(target, tick, metric)?;

        // Cache result
        self.cache.insert((target, tick, metric), actual_value);

        Ok(Resolution {
            market_id: market.id(),
            agent_id: target,
            metric,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            actual_value,
            timestamp: tick,
        })
    }

    /// Simulate agent DOCAS using Brian2
    fn simulate_docas(
        &mut self,
        _agent_id: AgentId,
        _tick: Tick,
        metric: DocasMetric,
    ) -> Result<f64, OracleError> {
        // In real implementation, this would:
        // 1. Get agent state at tick
        // 2. Run Brian2 simulation
        // 3. Extract metric value

        // Mock implementation
        let mock_value = match metric {
            DocasMetric::Dopamine => 0.5,
            DocasMetric::Oxytocin => 0.4,
            DocasMetric::Cortisol => 0.3,
            DocasMetric::Adrenaline => 0.6,
            DocasMetric::Serotonin => 0.7,
            DocasMetric::CompositeIndex => 0.5,
        };

        Ok(mock_value)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Batch resolve multiple markets
    pub fn batch_resolve(&mut self, markets: &[&Market]) -> Vec<Result<Resolution, OracleError>> {
        let mut results = Vec::new();

        for market in markets {
            results.push(self.resolve(market));
        }

        results
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache stats
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.cache.len(), self.cache.len())
    }
}

/// Market resolution result
#[derive(Debug, Clone)]
pub struct Resolution {
    /// Market ID
    pub market_id: MarketId,
    /// Target agent
    pub agent_id: AgentId,
    /// Resolved metric
    pub metric: DocasMetric,
    /// Actual value
    pub actual_value: f64,
    /// Resolution timestamp
    pub timestamp: Tick,
}

impl Resolution {
    /// Calculate payout for binary contract
    pub fn binary_payout(&self, predicted_yes: bool) -> f64 {
        // Threshold for binary resolution
        let threshold = 0.5;
        let outcome_yes = self.actual_value >= threshold;

        if predicted_yes == outcome_yes {
            1.0 // Winner
        } else {
            0.0 // Loser
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate payout for scalar contract (Brier score based)
    pub fn scalar_payout(&self, predicted_value: f64, max_range: f64) -> f64 {
        if !max_range.is_finite()
            || max_range <= 0.0
            || !predicted_value.is_finite()
            || !self.actual_value.is_finite()
        {
            return 0.0;
        }
        let error = (self.actual_value - predicted_value).abs();
        let normalized_error = (error / max_range).clamp(0.0, 1.0);

        // Higher accuracy = higher payout
        1.0 - normalized_error
    }

    /// Get human-readable summary
    pub fn summary(&self) -> String {
        format!(
            "Resolution for Market {}: Agent {}'s {} = {:.3}",
            self.market_id.0,
            self.agent_id,
            self.metric.name(),
            self.actual_value
        )
    }
}

/// Oracle errors
#[derive(Debug)]
pub enum OracleError {
    /// Simulation failed
    SimulationFailed(String),
    /// Agent not found
    AgentNotFound(AgentId),
    /// Invalid tick
    InvalidTick(Tick),
    /// Brian2 backend error
    BackendError(String),
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl std::fmt::Display for OracleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OracleError::SimulationFailed(msg) => write!(f, "Simulation failed: {}", msg),
            OracleError::AgentNotFound(id) => write!(f, "Agent {} not found", id),
            OracleError::InvalidTick(tick) => write!(f, "Invalid tick: {}", tick),
            OracleError::BackendError(msg) => write!(f, "Backend error: {}", msg),
        }
    }
}

impl std::error::Error for OracleError {}

/// Multi-oracle consensus mechanism
pub struct MultiOracleConsensus {
    /// List of oracles
    oracles: Vec<NeuralOracle>,
    /// Required agreement threshold
    threshold: f64,
}

impl MultiOracleConsensus {
    /// Create new consensus mechanism
    pub fn new(threshold: f64) -> Self {
        Self {
            oracles: Vec::new(),
            threshold,
        }
    }

    /// Add oracle
    pub fn add_oracle(&mut self, oracle: NeuralOracle) {
        self.oracles.push(oracle);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Resolve with consensus
    pub fn resolve_consensus(&mut self, market: &Market) -> Result<Resolution, OracleError> {
        // Get resolutions from all oracles
        let mut resolutions = Vec::new();

        for oracle in &mut self.oracles {
            match oracle.resolve(market) {
                Ok(res) => resolutions.push(res),
                Err(e) => eprintln!("Oracle failed: {}", e),
            }
        }

        if resolutions.is_empty() {
            return Err(OracleError::SimulationFailed(
                "All oracles failed".to_string(),
            ));
        }

        // Check agreement
        let values: Vec<f64> = resolutions.iter().map(|r| r.actual_value).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;

        if variance > self.threshold {
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            return Err(OracleError::SimulationFailed(format!(
                "Oracle disagreement too high: variance={}",
                variance
            )));
        }

        // Return consensus resolution
        Ok(Resolution {
            market_id: market.id(),
            agent_id: market.target_agent(),
            metric: resolutions[0].metric,
            actual_value: mean,
            timestamp: resolutions[0].timestamp,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution_binary_payout() {
        let resolution = Resolution {
            market_id: MarketId(1),
            agent_id: 42,
            metric: DocasMetric::Dopamine,
            actual_value: 0.7,
            timestamp: 1000,
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Actual = 0.7 >= 0.5, so YES wins
        assert_eq!(resolution.binary_payout(true), 1.0);
        assert_eq!(resolution.binary_payout(false), 0.0);

        let resolution_no = Resolution {
            market_id: MarketId(2),
            agent_id: 43,
            metric: DocasMetric::Cortisol,
            actual_value: 0.3,
            timestamp: 1000,
        };

        // Actual = 0.3 < 0.5, so NO wins
        assert_eq!(resolution_no.binary_payout(true), 0.0);
        assert_eq!(resolution_no.binary_payout(false), 1.0);
    }

    #[test]
    fn test_resolution_scalar_payout() {
        let resolution = Resolution {
            market_id: MarketId(1),
            agent_id: 42,
            metric: DocasMetric::Dopamine,
            actual_value: 0.7,
            timestamp: 1000,
        };

        // Perfect prediction
        assert!((resolution.scalar_payout(0.7, 1.0) - 1.0).abs() < 0.001);

        // Off by 0.2
        assert!((resolution.scalar_payout(0.5, 1.0) - 0.8).abs() < 0.001);

        // Off by 0.5
        assert!((resolution.scalar_payout(0.2, 1.0) - 0.5).abs() < 0.001);
    }
}
