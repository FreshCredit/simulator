//! Per-Tick Metrics (Layer 1)
//!
//! SIM-001: Isolation verified - no product dependencies
//! SIM-004: Deterministic - uses only primitive types
//!
//! Per §5.3.2 lines 606-616: Layer 1 statistical implementation
//! Per §5.3.5 lines 657-665: Implementation order

// TAG: surface=simulation owner=research-team rule=SIM-001
use crate::docas::DocasState;
use crate::types::{A, C, D, O, S};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute branching ratio σ for criticality tracking
/// Per §5.3.2 line 610: σ = `Σ(events_caused)` / `Σ(events_received)`
/// Per §6.3: Target σ ≈ 1.0 for edge-of-chaos dynamics
///
/// **Phase Required:** Phase 0
///
/// # Arguments
/// * `events_caused` - Total number of events caused by agents
/// * `events_received` - Total number of events received by agents
///
/// # Returns
/// Branching ratio σ. Returns 0.0 if `events_received` is 0.
///
/// Per Bak-Tang-Wiesenfeld (1987): σ = 1.0 at criticality
/// `events_caused`: births + deaths + encounters initiated
/// `events_received`: population affected (living + deaths)
///
/// # Example
/// ```
/// use sim_core::metrics::branching_ratio;
///
/// let sigma = branching_ratio(100, 100);
/// debug_assert_eq!(sigma, 1.0); // Critical state
/// ```
#[must_use]
pub fn branching_ratio(events_caused: u64, events_received: u64) -> f64 {
    if events_received == 0 {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        return 1.0; // Neutral default at criticality
    }
    events_caused as f64 / events_received as f64
}

/// Compute branching ratio from simulation metrics
/// Standardized implementation used across all modules
///
/// # Arguments
/// * `births` - Number of births this tick
/// * `deaths` - Number of deaths this tick
/// * `encounters` - Number of encounters this tick
/// * `population` - Current living population
///
/// # Returns
/// Branching ratio σ as f64
#[must_use]
pub fn compute_branching_ratio(
    births: u32,
    deaths: u32,
    encounters: u32,
    population: usize,
) -> f64 {
    let events_caused = births as u64 + deaths as u64 + encounters as u64;
    let events_received = population as u64 + deaths as u64;
    branching_ratio(events_caused, events_received)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute DOCAS magnitude ‖DOCAS‖ for physiological stream tracking
/// Per §5.3.2 line 611: ‖DOCAS‖ = √(D² + O² + C² + A² + S²)
/// Per §8.3.3 line 4787: Used in Ψbio calculation
///
/// **Phase Required:** Phase 1
///
/// # Arguments
/// * `docas` - DOCAS state vector
///
/// # Returns
/// Euclidean magnitude of DOCAS vector
///
/// # Example
/// ```
/// use sim_core::docas::DocasState;
/// use sim_core::metrics::docas_magnitude;
///
/// let docas = DocasState::new([0.6, 0.5, 0.4, 0.3, 0.7]);
/// let magnitude = docas_magnitude(&docas);
/// // √(0.36 + 0.25 + 0.16 + 0.09 + 0.49) = √1.35 ≈ 1.162
/// debug_assert!((magnitude - 1.162).abs() < 0.01);
/// ```
#[must_use]
pub fn docas_magnitude(docas: &DocasState) -> f32 {
    let d = docas.levels[D];
    let o = docas.levels[O];
    let c = docas.levels[C];
    let a = docas.levels[A];
    let s = docas.levels[S];

    (d * d + o * o + c * c + a * a + s * s).sqrt()
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Check if sigma is in critical regime
/// Per §6.3: σ ∈ [0.9, 1.1] is critical (edge-of-chaos)
/// Per SIM-007: Criticality maintenance target
#[must_use]
pub fn is_critical(sigma: f32) -> bool {
    (0.9..=1.1).contains(&sigma)
}

/// Compute criticality deviation from target (σ = 1.0)
/// Returns 0.0 if exactly at critical point, positive if deviating
#[must_use]
pub fn criticality_deviation(sigma: f32) -> f32 {
    (sigma - 1.0).abs()
}

/// Compute decision confidence based on criticality
/// Higher confidence when closer to critical point (σ = 1.0)
/// Per SIM-007: Confidence = 1.0 - |σ - 1.0|
#[must_use]
pub fn decision_confidence(sigma: f32) -> f32 {
    (1.0 - (sigma - 1.0).abs()).clamp(0.0, 1.0)
}

/// Criticality regulation signal
/// Returns adjustment needed to move sigma toward 1.0
/// Per SIM-007: Used to modulate agent behavior to maintain criticality
#[must_use]
pub fn criticality_regulation_signal(sigma: f32) -> f32 {
    1.0 - sigma // Positive if subcritical (σ < 1), negative if supercritical (σ > 1)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute population-level criticality metrics
/// Per §5.3.2: Aggregate σ across all agents
pub fn population_criticality_metrics(sigmas: &[f32]) -> CriticalityMetrics {
    if sigmas.is_empty() {
        return CriticalityMetrics::default();
    }

    let n = sigmas.len() as f32;
    let mean_sigma: f32 = sigmas.iter().sum::<f32>() / n;

    let variance: f32 = sigmas
        .iter()
        .map(|&s| (s - mean_sigma).powi(2))
        .sum::<f32>()
        / n;

    let critical_count = sigmas.iter().filter(|&&s| is_critical(s)).count() as f32;
    let critical_fraction = critical_count / n;

    CriticalityMetrics {
        mean_sigma,
        std_dev: variance.sqrt(),
        critical_fraction,
        min_sigma: sigmas.iter().copied().fold(f32::INFINITY, f32::min),
        max_sigma: sigmas.iter().copied().fold(f32::NEG_INFINITY, f32::max),
    }
}

/// Population-level criticality metrics
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
#[derive(Debug, Clone, Copy, Default)]
pub struct CriticalityMetrics {
    /// Mean σ across population
    pub mean_sigma: f32,
    /// Standard deviation of σ
    pub std_dev: f32,
    /// Fraction of agents in critical regime
    pub critical_fraction: f32,
    /// Minimum σ observed
    pub min_sigma: f32,
    /// Maximum σ observed
    pub max_sigma: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branching_ratio_critical() {
        // Critical state: σ = 1.0
        let sigma = branching_ratio(100, 100);
        assert_eq!(sigma, 1.0);
    }

    #[test]
    fn test_branching_ratio_subcritical() {
        // Subcritical: σ < 1.0
        let sigma = branching_ratio(50, 100);
        assert_eq!(sigma, 0.5);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_branching_ratio_supercritical() {
        // Supercritical: σ > 1.0
        let sigma = branching_ratio(150, 100);
        assert_eq!(sigma, 1.5);
    }

    #[test]
    fn test_branching_ratio_zero_received() {
        // Edge case: no events received
        // Returns 1.0 (critical state) as neutral default
        let sigma = branching_ratio(100, 0);
        assert_eq!(sigma, 1.0);
    }

    #[test]
    fn test_docas_magnitude_neutral() {
        // Neutral state: all 0.5
        let docas = DocasState::neutral();
        let magnitude = docas_magnitude(&docas);
        // √(5 × 0.5²) = √1.25 ≈ 1.118
        assert!((magnitude - 1.118).abs() < 0.01);
    }

    #[test]
    fn test_docas_magnitude_varied() {
        // Varied state
        let docas = DocasState::new([0.6, 0.5, 0.4, 0.3, 0.7]);
        let magnitude = docas_magnitude(&docas);
        // √(0.36 + 0.25 + 0.16 + 0.09 + 0.49) = √1.35 ≈ 1.162
        assert!((magnitude - 1.162).abs() < 0.01);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_docas_magnitude_zero() {
        // All zeros
        let docas = DocasState::new([0.0, 0.0, 0.0, 0.0, 0.0]);
        let magnitude = docas_magnitude(&docas);
        assert_eq!(magnitude, 0.0);
    }

    #[test]
    fn test_docas_magnitude_max() {
        // All ones
        let docas = DocasState::new([1.0, 1.0, 1.0, 1.0, 1.0]);
        let magnitude = docas_magnitude(&docas);
        // √5 ≈ 2.236
        assert!((magnitude - 2.236).abs() < 0.01);
    }

    // === SIM-007: Criticality Tracking Tests ===

    #[test]
    fn test_is_critical() {
        assert!(is_critical(1.0), "σ = 1.0 is critical");
        assert!(is_critical(0.9), "σ = 0.9 is critical (boundary)");
        assert!(is_critical(1.1), "σ = 1.1 is critical (boundary)");
        assert!(!is_critical(0.8), "σ = 0.8 is not critical");
        assert!(!is_critical(1.2), "σ = 1.2 is not critical");
    }

    #[test]
    fn test_criticality_deviation() {
        assert!((criticality_deviation(1.0) - 0.0).abs() < 0.001);
        assert!((criticality_deviation(0.9) - 0.1).abs() < 0.001);
        assert!((criticality_deviation(1.1) - 0.1).abs() < 0.001);
        assert!((criticality_deviation(0.8) - 0.2).abs() < 0.001);
        assert!((criticality_deviation(1.2) - 0.2).abs() < 0.001);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_decision_confidence() {
        assert!((decision_confidence(1.0) - 1.0).abs() < 0.001);
        assert!((decision_confidence(0.9) - 0.9).abs() < 0.001);
        assert!((decision_confidence(1.1) - 0.9).abs() < 0.001);
        assert!((decision_confidence(0.5) - 0.5).abs() < 0.001);
        assert!((decision_confidence(1.5) - 0.5).abs() < 0.001);
        assert!((decision_confidence(0.0) - 0.0).abs() < 0.001);
        assert!((decision_confidence(2.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_criticality_regulation_signal() {
        assert!((criticality_regulation_signal(1.0) - 0.0).abs() < 0.001);
        assert!((criticality_regulation_signal(0.8) - 0.2).abs() < 0.001); // Need more activity
        assert!((criticality_regulation_signal(1.2) - -0.2).abs() < 0.001); // Need less activity
    }

    #[test]
    fn test_population_criticality_metrics() {
        let sigmas = vec![0.9, 1.0, 1.1, 0.8, 1.2];
        let metrics = population_criticality_metrics(&sigmas);

        assert!((metrics.mean_sigma - 1.0).abs() < 0.01);
        assert!(metrics.critical_fraction > 0.5); // 3/5 are critical
        assert_eq!(metrics.min_sigma, 0.8);
        assert_eq!(metrics.max_sigma, 1.2);
    }
}
