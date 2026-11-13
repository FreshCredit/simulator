//! Economic Model Comparison (ECON-P0-4)
//!
//! Compares trust-based economic prediction against alternative models:
//! - Solow growth model
//! - Mankiw-Romer-Weil (MRW) model
//! - Lucas human capital model
//! - Institutional quality models

use crate::experiments::{Experiment, ExperimentConfig, ExperimentResult};
use anyhow::Result;
use serde_json::json;

// TAG: surface=simulation owner=research-team rule=SIM-001

/// Country economic data
#[derive(Debug, Clone)]
pub struct CountryData {
    /// name
    pub name: &'static str,
    /// gdp per capita
    pub gdp_per_capita: f64,
    /// savings rate
    pub savings_rate: f64,
    /// population growth
    pub population_growth: f64,
    /// human capital
    pub human_capital: f64,
    /// institutional quality
    pub institutional_quality: f64,
    /// trust level
    pub trust_level: f64,
}

/// Sample countries (World Bank data approximations)
pub const COUNTRIES: &[CountryData] = &[
    CountryData {
        name: "Norway",
        gdp_per_capita: 75000.0,
        savings_rate: 0.30,
        population_growth: 0.008,
        human_capital: 0.95,
        institutional_quality: 0.95,
        trust_level: 0.74,
    },
    CountryData {
        name: "USA",
        gdp_per_capita: 63000.0,
        savings_rate: 0.18,
        population_growth: 0.007,
        human_capital: 0.90,
        institutional_quality: 0.85,
        trust_level: 0.40,
    },
    CountryData {
        name: "Germany",
        gdp_per_capita: 46000.0,
        savings_rate: 0.28,
        population_growth: 0.002,
        human_capital: 0.92,
        institutional_quality: 0.88,
        trust_level: 0.45,
    },
    CountryData {
        name: "Japan",
        gdp_per_capita: 40000.0,
        savings_rate: 0.28,
        population_growth: -0.002,
        human_capital: 0.94,
        institutional_quality: 0.87,
        trust_level: 0.35,
    },
    CountryData {
        name: "UK",
        gdp_per_capita: 42000.0,
        savings_rate: 0.15,
        population_growth: 0.005,
        human_capital: 0.88,
        institutional_quality: 0.85,
        trust_level: 0.32,
    },
    CountryData {
        name: "France",
        gdp_per_capita: 39000.0,
        savings_rate: 0.23,
        population_growth: 0.003,
        human_capital: 0.87,
        institutional_quality: 0.82,
        trust_level: 0.28,
    },
    CountryData {
        name: "Italy",
        gdp_per_capita: 32000.0,
        savings_rate: 0.20,
        population_growth: 0.001,
        human_capital: 0.82,
        institutional_quality: 0.70,
        trust_level: 0.26,
    },
    CountryData {
        name: "South Korea",
        gdp_per_capita: 31000.0,
        savings_rate: 0.35,
        population_growth: 0.002,
        human_capital: 0.90,
        institutional_quality: 0.78,
        trust_level: 0.30,
    },
    CountryData {
        name: "Spain",
        gdp_per_capita: 27000.0,
        savings_rate: 0.23,
        population_growth: 0.002,
        human_capital: 0.80,
        institutional_quality: 0.75,
        trust_level: 0.22,
    },
    CountryData {
        name: "Chile",
        gdp_per_capita: 16000.0,
        savings_rate: 0.20,
        population_growth: 0.009,
        human_capital: 0.72,
        institutional_quality: 0.68,
        trust_level: 0.15,
    },
    CountryData {
        name: "Mexico",
        gdp_per_capita: 10000.0,
        savings_rate: 0.24,
        population_growth: 0.011,
        human_capital: 0.65,
        institutional_quality: 0.55,
        trust_level: 0.12,
    },
    CountryData {
        name: "Brazil",
        gdp_per_capita: 7500.0,
        savings_rate: 0.16,
        population_growth: 0.007,
        human_capital: 0.60,
        institutional_quality: 0.50,
        trust_level: 0.08,
    },
    CountryData {
        name: "China",
        gdp_per_capita: 10500.0,
        savings_rate: 0.45,
        population_growth: 0.005,
        human_capital: 0.70,
        institutional_quality: 0.60,
        trust_level: 0.42,
    },
    CountryData {
        name: "India",
        gdp_per_capita: 2100.0,
        savings_rate: 0.30,
        population_growth: 0.010,
        human_capital: 0.50,
        institutional_quality: 0.52,
        trust_level: 0.32,
    },
    CountryData {
        name: "Nigeria",
        gdp_per_capita: 2000.0,
        savings_rate: 0.15,
        population_growth: 0.025,
        human_capital: 0.35,
        institutional_quality: 0.30,
        trust_level: 0.10,
    },
];

/// Model predictions
#[derive(Debug, Clone)]
pub struct ModelPrediction {
    /// model name
    pub model_name: &'static str,
    /// r squared
    pub r_squared: f64,
    /// aic
    pub aic: f64,
    /// predictions
    pub predictions: Vec<f64>,
}

/// Economic Model Comparison Experiment
///
/// ECON-P0-4: Compares trust-based model against alternatives.
#[derive(Debug, Default)]
pub struct EconomicModelComparison;

impl EconomicModelComparison {
    /// new
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Solow model: ln(y) = ln(A) + α/(1-α)ln(s) - α/(1-α)ln(n+g+δ)
    fn solow_model(&self, data: &[CountryData]) -> ModelPrediction {
        let alpha = 0.33; // Capital share

        let predictions: Vec<f64> = data
            .iter()
            .map(|c| {
                let tech = 10.2; // Technology level (log)
                let capital_term = (alpha / (1.0 - alpha)) * c.savings_rate.max(1e-6).ln();
                let labor_term =
                    -(alpha / (1.0 - alpha)) * (c.population_growth + 0.03).max(1e-6).ln();
                (tech + capital_term + labor_term).exp()
            })
            .collect();

        let r2 = self.r_squared(
            &data.iter().map(|c| c.gdp_per_capita).collect::<Vec<_>>(),
            &predictions,
        );
        let aic = self.aic(
            &data.iter().map(|c| c.gdp_per_capita).collect::<Vec<_>>(),
            &predictions,
            1,
        );

        ModelPrediction {
            model_name: "Solow",
            r_squared: r2,
            aic,
            predictions,
        }
    }

    /// MRW model: adds human capital
    fn mrw_model(&self, data: &[CountryData]) -> ModelPrediction {
        let alpha = 0.33;
        let beta = 0.33; // Human capital share

        let predictions: Vec<f64> = data
            .iter()
            .map(|c| {
                let tech = 8.0;
                let capital_term = (alpha / (1.0 - alpha - beta)) * c.savings_rate.max(1e-6).ln();
                let labor_term =
                    -(alpha / (1.0 - alpha - beta)) * (c.population_growth + 0.03).max(1e-6).ln();
                let hc_term = (beta / (1.0 - alpha - beta)) * c.human_capital.max(1e-6).ln();
                (tech + capital_term + labor_term + hc_term).exp()
            })
            .collect();

        let r2 = self.r_squared(
            &data.iter().map(|c| c.gdp_per_capita).collect::<Vec<_>>(),
            &predictions,
        );
        let aic = self.aic(
            &data.iter().map(|c| c.gdp_per_capita).collect::<Vec<_>>(),
            &predictions,
            2,
        );

        ModelPrediction {
            model_name: "MRW",
            r_squared: r2,
            aic,
            predictions,
        }
    }

    /// Trust-augmented model
    fn trust_model(&self, data: &[CountryData]) -> ModelPrediction {
        let alpha = 0.30;
        let beta = 0.30;
        let gamma = 0.15; // Trust coefficient

        let predictions: Vec<f64> = data
            .iter()
            .map(|c| {
                let tech = 7.5;
                let capital_term =
                    (alpha / (1.0 - alpha - beta - gamma)) * c.savings_rate.max(1e-6).ln();
                let labor_term = -(alpha / (1.0 - alpha - beta - gamma))
                    * (c.population_growth + 0.03).max(1e-6).ln();
                let hc_term =
                    (beta / (1.0 - alpha - beta - gamma)) * c.human_capital.max(1e-6).ln();
                let trust_term =
                    (gamma / (1.0 - alpha - beta - gamma)) * (c.trust_level + 0.1).max(1e-6).ln();
                (tech + capital_term + labor_term + hc_term + trust_term).exp()
            })
            .collect();

        let r2 = self.r_squared(
            &data.iter().map(|c| c.gdp_per_capita).collect::<Vec<_>>(),
            &predictions,
        );
        let aic = self.aic(
            &data.iter().map(|c| c.gdp_per_capita).collect::<Vec<_>>(),
            &predictions,
            3,
        );

        ModelPrediction {
            model_name: "Trust-Augmented",
            r_squared: r2,
            aic,
            predictions,
        }
    }

    /// Calculate R-squared
    fn r_squared(&self, actual: &[f64], predicted: &[f64]) -> f64 {
        if actual.len() != predicted.len() || actual.is_empty() {
            return 0.0;
        }

        let clean: Vec<(f64, f64)> = actual
            .iter()
            .zip(predicted.iter())
            .filter(|&(&a, &p)| a.is_finite() && p.is_finite())
            .map(|(&a, &p)| (a, p))
            .collect();

        if clean.len() < 2 {
            return 0.0;
        }

        let n = clean.len() as f64;
        let mean_actual = clean.iter().map(|(a, _)| a).sum::<f64>() / n;
        let ss_total: f64 = clean.iter().map(|(a, _)| (a - mean_actual).powi(2)).sum();
        let ss_residual: f64 = clean.iter().map(|(a, p)| (a - p).powi(2)).sum();

        if ss_total <= 1e-12 || !ss_total.is_finite() || !ss_residual.is_finite() {
            return 0.0;
        }

        (1.0 - (ss_residual / ss_total)).clamp(0.0, 1.0)
    }

    /// Calculate AIC (Akaike Information Criterion)
    fn aic(&self, actual: &[f64], predicted: &[f64], k: usize) -> f64 {
        let clean: Vec<(f64, f64)> = actual
            .iter()
            .zip(predicted.iter())
            .filter(|&(&a, &p)| a.is_finite() && p.is_finite())
            .map(|(&a, &p)| (a, p))
            .collect();

        if clean.is_empty() {
            return 0.0;
        }

        let n = clean.len() as f64;
        let rss: f64 = clean.iter().map(|(a, p)| (a - p).powi(2)).sum();
        let variance_est = (rss / n).max(1e-12);
        n * variance_est.ln() + 2.0 * k as f64
    }
}

impl Experiment for EconomicModelComparison {
    fn name(&self) -> &str {
        "economic_model_comparison"
    }

    fn description(&self) -> &str {
        "Compare trust-based model against economic alternatives (ECON-P0-4)"
    }

    fn required_agents(&self) -> usize {
        100
    }

    fn run(&self, _config: &ExperimentConfig) -> Result<ExperimentResult> {
        let data = COUNTRIES;

        // Fit all models
        let solow = self.solow_model(data);
        let mrw = self.mrw_model(data);
        let trust = self.trust_model(data);

        let models = [solow, mrw.clone(), trust.clone()];

        // Find best model by AIC
        let best_aic = models.iter().map(|m| m.aic).fold(f64::INFINITY, f64::min);
        let trust_delta_aic = trust.aic - best_aic;

        // Trust model should be best or close
        let trust_best = (trust.aic - best_aic).abs() < f64::EPSILON;

        // Trust model should improve on MRW
        let improvement_over_mrw = mrw.aic - trust.aic;

        // Calculate trust × institutions interaction
        let inst_trust_corr = {
            let inst: Vec<f64> = data.iter().map(|c| c.institutional_quality).collect();
            let trust: Vec<f64> = data.iter().map(|c| c.trust_level).collect();

            let n = inst.len() as f64;
            let mean_i = inst.iter().sum::<f64>() / n;
            let mean_t = trust.iter().sum::<f64>() / n;

            let num: f64 = inst
                .iter()
                .zip(trust.iter())
                .map(|(i, t)| (i - mean_i) * (t - mean_t))
                .sum();
            let den_i: f64 = inst.iter().map(|i| (i - mean_i).powi(2)).sum();
            let den_t: f64 = trust.iter().map(|t| (t - mean_t).powi(2)).sum();

            num / (den_i * den_t).sqrt()
        };

        // Validation criteria
        let criteria = [
            ("Trust model competitive AIC", trust_delta_aic < 10.0),
            ("Improvement over baseline", improvement_over_mrw > 2.0),
            ("Trust × Institutions correlation", inst_trust_corr > 0.3),
            ("R² > 0.85", trust.r_squared > 0.85),
        ];

        let criteria_passed = criteria.iter().filter(|(_, c)| *c).count();
        let success = criteria_passed >= 3;

        let model_comparison: Vec<_> = models
            .iter()
            .map(|m| {
                json!({
                    "model": m.model_name,
                    "r_squared": m.r_squared,
                    "aic": m.aic,
                    "delta_aic": m.aic - best_aic,
                })
            })
            .collect();

        let metrics = json!({
            "solow_r2": data.iter().map(|c| c.gdp_per_capita).collect::<Vec<_>>(),
            "trust_r2": trust.r_squared,
            "trust_aic": trust.aic,
            "improvement_over_mrw": improvement_over_mrw,
            "trust_best": trust_best,
            "inst_trust_correlation": inst_trust_corr,
            "model_comparison": model_comparison,
            "criteria_passed": criteria_passed,
        });

        Ok(ExperimentResult {
            experiment_type: "economic_model_comparison".to_string(),
            success,
            gap_id: "ECON-P0-4".to_string(),
            paper: "ECONOMICS".to_string(),
            metrics,
            figures: vec![],
            csv_outputs: vec![],
            artifacts: vec![],
            duration: std::time::Duration::from_secs(0),
            error_message: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_country_data() {
        assert!(COUNTRIES.len() >= 5);
        assert!(COUNTRIES.iter().all(|c| c.gdp_per_capita > 0.0));
    }

    #[test]
    fn test_solow_model() {
        let exp = EconomicModelComparison::new();
        let pred = exp.solow_model(COUNTRIES);
        assert!(pred.aic.is_finite());
        assert_eq!(pred.predictions.len(), COUNTRIES.len());
    }

    #[test]
    fn test_model_comparison() {
        let exp = EconomicModelComparison::new();
        let solow = exp.solow_model(COUNTRIES);
        let trust = exp.trust_model(COUNTRIES);

        // Both should produce predictions
        assert!(solow.predictions.iter().all(|&p| p > 0.0));
        assert!(trust.predictions.iter().all(|&p| p > 0.0));
    }
}
