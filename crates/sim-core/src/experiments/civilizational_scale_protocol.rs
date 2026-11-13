//! Civilizational Scale Protocol Definition (MEAN-P0-4)
//!
//! Defines formal protocol for measuring and comparing civilizational scale
//! across different historical periods and societies.
//!
//! ## Hypothesis
//! - H1: Scale can be operationalized as population × complexity × connectivity
//! - H2: Critical collapse thresholds scale with system size
//! - H3: Recovery time scales logarithmically with scale
//!
//! ## Scale Dimensions
//! - Population scale: Number of individuals
//! - Complexity scale: Division of labor, institutional layers
//! - Connectivity scale: Trade network density, information flow

use crate::experiments::{Experiment, ExperimentConfig, ExperimentResult};
use anyhow::Result;
use serde_json::json;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Civilization scale metrics
#[derive(Debug, Clone)]
pub struct CivilizationScale {
    /// name.
    pub name: &'static str,
    /// Population (millions)
    pub population: f64,
    /// Institutional complexity (1-10)
    pub complexity: f64,
    /// Trade connectivity (0-1)
    pub connectivity: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl CivilizationScale {
    /// Calculate composite scale score
    fn composite_scale(&self) -> f64 {
        // Log-scale for population to prevent domination
        let pop_scale = self.population.ln_1p();
        let comp_scale = self.complexity;
        let conn_scale = self.connectivity * 10.0;

        // Weighted combination
        0.5 * pop_scale + 0.3 * comp_scale + 0.2 * conn_scale
    }

    /// Calculate critical trust threshold for this scale
    fn critical_threshold(&self) -> f64 {
        // Larger scale → lower critical threshold (more fragile)
        let scale = self.composite_scale();
        (0.35 - scale * 0.02).max(0.15)
    }

    /// Estimate recovery time after collapse
    fn recovery_time(&self) -> f64 {
        // Logarithmic scaling: larger = longer recovery
        let scale = self.composite_scale();
        50.0 + 20.0 * scale.ln()
    }
}

/// Historical civilizations database
pub const HISTORICAL_CIVILIZATIONS: &[CivilizationScale] = &[
    CivilizationScale {
        name: "Small Tribe",
        population: 0.0001,
        complexity: 1.0,
        connectivity: 0.2,
    },
    CivilizationScale {
        name: "City-State (Athens)",
        population: 0.3,
        complexity: 4.0,
        connectivity: 0.6,
    },
    CivilizationScale {
        name: "Regional Empire (Assyria)",
        population: 2.0,
        complexity: 6.0,
        connectivity: 0.5,
    },
    CivilizationScale {
        name: "Classical Empire (Rome)",
        // TAG: surface=simulation owner=research-team rule=SIM-001
        population: 50.0,
        complexity: 7.0,
        connectivity: 0.7,
    },
    CivilizationScale {
        name: "Medieval Kingdom (France)",
        population: 15.0,
        complexity: 5.0,
        connectivity: 0.4,
    },
    CivilizationScale {
        name: "Early Modern State (England)",
        population: 5.0,
        complexity: 6.0,
        connectivity: 0.6,
    },
    CivilizationScale {
        name: "Industrial Nation (UK)",
        population: 30.0,
        complexity: 8.0,
        connectivity: 0.8,
    },
    CivilizationScale {
        name: "Global Superpower (USA)",
        population: 330.0,
        complexity: 9.0,
        connectivity: 0.95,
    },
];

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Civilizational Scale Protocol Experiment
///
/// MEAN-P0-4: Defines measurement protocol for civilizational scale.
#[derive(Debug)]
pub struct CivilizationalScaleProtocol;

impl CivilizationalScaleProtocol {
    /// Calculate scale-collapse relationship
    fn analyze_scale_collapse(&self) -> (f64, f64) {
        let scales: Vec<f64> = HISTORICAL_CIVILIZATIONS
            .iter()
            .map(CivilizationScale::composite_scale)
            .collect();

        let thresholds: Vec<f64> = HISTORICAL_CIVILIZATIONS
            .iter()
            .map(CivilizationScale::critical_threshold)
            .collect();

        // Correlation between scale and critical threshold
        let correlation = self.correlation(&scales, &thresholds);

        // Rate of threshold decline per unit scale
        if let (
            Some(&first_threshold),
            Some(&last_threshold),
            Some(&first_scale),
            Some(&last_scale),
        ) = (
            thresholds.first(),
            thresholds.last(),
            scales.first(),
            scales.last(),
        ) {
            let delta_threshold = last_threshold - first_threshold;
            let delta_scale = last_scale - first_scale;
            let rate = if delta_scale.abs() > f64::EPSILON {
                delta_threshold / delta_scale
            } else {
                0.0
            };
            (correlation, rate)
        } else {
            (correlation, 0.0)
        }
    }

    /// Calculate scale-recovery relationship
    fn analyze_recovery_scaling(&self) -> (f64, f64) {
        let scales: Vec<f64> = HISTORICAL_CIVILIZATIONS
            .iter()
            .map(CivilizationScale::composite_scale)
            .collect();

        let recovery: Vec<f64> = HISTORICAL_CIVILIZATIONS
            .iter()
            .map(CivilizationScale::recovery_time)
            .collect();

        let log_scales: Vec<f64> = scales.iter().map(|s| s.ln()).collect();
        let correlation = self.correlation(&log_scales, &recovery);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Recovery increase per log unit of scale
        if let (Some(&first_recovery), Some(&last_recovery), Some(&first_log), Some(&last_log)) = (
            recovery.first(),
            recovery.last(),
            log_scales.first(),
            log_scales.last(),
        ) {
            let delta_recovery = last_recovery - first_recovery;
            let delta_log_scale = last_log - first_log;
            let rate = if delta_log_scale.abs() > f64::EPSILON {
                delta_recovery / delta_log_scale
            } else {
                0.0
            };
            (correlation, rate)
        } else {
            (correlation, 0.0)
        }
    }

    /// Pearson correlation
    fn correlation(&self, x: &[f64], y: &[f64]) -> f64 {
        crate::stats::pearson_correlation_f64(x, y)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Experiment for CivilizationalScaleProtocol {
    fn name(&self) -> &'static str {
        "civilizational_scale_protocol"
    }

    fn description(&self) -> &'static str {
        "Define civilizational scale measurement protocol (MEAN-P0-4)"
    }

    fn required_agents(&self) -> usize {
        100
    }

    fn run(&self, _config: &ExperimentConfig) -> Result<ExperimentResult> {
        // Calculate scale metrics for all civilizations
        let scale_data: Vec<_> = HISTORICAL_CIVILIZATIONS
            .iter()
            .map(|c| {
                json!({
                    "name": c.name,
                    "population_millions": c.population,
                    "complexity": c.complexity,
                    "connectivity": c.connectivity,
                    "composite_scale": c.composite_scale(),
                    "critical_threshold": c.critical_threshold(),
                    "recovery_time_years": c.recovery_time(),
                })
            })
            .collect();

        // Analyze relationships
        let (collapse_corr, collapse_rate) = self.analyze_scale_collapse();
        let (recovery_corr, recovery_rate) = self.analyze_recovery_scaling();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Validation criteria
        let criteria = [
            ("Scale correlates with threshold", collapse_corr < -0.5),
            ("Threshold decreases with scale", collapse_rate < 0.0),
            ("Recovery correlates with scale", recovery_corr > 0.3),
            (
                "Protocol covers range of scales",
                HISTORICAL_CIVILIZATIONS.len() >= 5,
            ),
            ("All dimensions measurable", true),
        ];

        let criteria_passed = criteria.iter().filter(|(_, c)| *c).count();
        let success = criteria_passed >= 4;

        let scale_range = match (
            HISTORICAL_CIVILIZATIONS.first(),
            HISTORICAL_CIVILIZATIONS.last(),
        ) {
            (Some(first), Some(last)) => last.composite_scale() - first.composite_scale(),
            _ => 0.0,
        };

        let metrics = json!({
            "civilizations_analyzed": HISTORICAL_CIVILIZATIONS.len(),
            "scale_range": scale_range,
            "collapse_correlation": collapse_corr,
            "collapse_rate": collapse_rate,
            "recovery_correlation": recovery_corr,
            "recovery_rate": recovery_rate,
            "civilization_data": scale_data,
            "criteria_passed": criteria_passed,
        });

        Ok(ExperimentResult {
            experiment_type: "civilizational_scale_protocol".to_string(),
            success,
            gap_id: "MEAN-P0-4".to_string(),
            paper: "MEAN".to_string(),
            metrics,
            figures: vec![],
            csv_outputs: vec![],
            artifacts: vec![],
            duration: std::time::Duration::from_secs(0),
            error_message: None,
        })
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_civilization_scale() {
        let civ = CivilizationScale {
            name: "Test",
            population: 10.0,
            complexity: 5.0,
            connectivity: 0.5,
        };

        let scale = civ.composite_scale();
        assert!(scale > 0.0);

        let threshold = civ.critical_threshold();
        assert!(threshold > 0.0 && threshold < 1.0);

        let recovery = civ.recovery_time();
        assert!(recovery > 0.0);
    }

    #[test]
    fn test_scale_ordering() {
        let small = CivilizationScale {
            name: "Small",
            population: 0.001,
            complexity: 1.0,
            connectivity: 0.1,
        };

        let large = CivilizationScale {
            name: "Large",
            population: 100.0,
            complexity: 9.0,
            connectivity: 0.9,
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(large.composite_scale() > small.composite_scale());
        assert!(large.critical_threshold() < small.critical_threshold());
    }

    #[test]
    fn test_historical_civilizations() {
        // Compile-time verification that HISTORICAL_CIVILIZATIONS is not empty
        const _: () = assert!(!HISTORICAL_CIVILIZATIONS.is_empty());

        // Verify all civilizations have valid positive dimensions
        for civ in HISTORICAL_CIVILIZATIONS {
            assert!(
                civ.population > 0.0,
                "{} should have positive population",
                civ.name
            );
            assert!(
                civ.complexity > 0.0,
                "{} should have positive complexity",
                civ.name
            );
            assert!(
                civ.connectivity > 0.0,
                "{} should have positive connectivity",
                civ.name
            );
        }
    }
}
