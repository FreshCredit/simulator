//! Causal Inference (P2-RES-4)
//!
//! Causal discovery and inference from observational data.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Causal relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalRelationship {
    /// cause.
    pub cause: String,
    /// effect.
    pub effect: String,
    /// strength.
    pub strength: f32,
    /// confidence.
    pub confidence: f32,
    /// mechanism.
    pub mechanism: Option<String>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Causal graph
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CausalGraph {
    /// nodes.
    pub nodes: Vec<String>,
    /// edges.
    pub edges: Vec<CausalEdge>,
}

/// Causal edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEdge {
    /// from.
    pub from: String,
    /// to.
    pub to: String,
    /// weight.
    pub weight: f32,
    /// edge type.
    pub edge_type: EdgeType,
}

/// Edge type in causal graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    /// direct.
    Direct,
    /// mediated.
    Mediated,
    /// confounded.
    Confounded,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Causal inference configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalConfig {
    /// significance threshold.
    pub significance_threshold: f32,
    /// min effect size.
    pub min_effect_size: f32,
    /// max lag.
    pub max_lag: usize,
}

impl Default for CausalConfig {
    fn default() -> Self {
        Self {
            significance_threshold: 0.05,
            min_effect_size: 0.1,
            max_lag: 5,
        }
    }
}

/// Causal inference engine
#[derive(Debug, Clone)]
pub struct CausalInference {
    /// config.
    pub config: CausalConfig,
    /// graph.
    pub graph: CausalGraph,
}

impl CausalInference {
    #[must_use]
    /// new.
    pub fn new(config: CausalConfig) -> Self {
        Self {
            config,
            graph: CausalGraph::default(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Granger causality test using multivariate OLS and Schwarz BIC lag selection
    #[must_use]
    pub fn granger_causality(&self, x: &[f32], y: &[f32], max_lag: usize) -> GrangerResult {
        let clean_pairs: Vec<(f64, f64)> = x
            .iter()
            .zip(y.iter())
            .filter(|(&a, &b)| a.is_finite() && b.is_finite())
            .map(|(&a, &b)| (f64::from(a), f64::from(b)))
            .collect();

        let max_lag = max_lag.max(1);
        if clean_pairs.len() <= 2 * max_lag + 2 || clean_pairs.len() < 4 {
            return GrangerResult {
                f_statistic: 0.0,
                p_value: 1.0,
                causal: false,
                optimal_lag: 0,
            };
        }

        let x_f64: Vec<f64> = clean_pairs.iter().map(|p| p.0).collect();
        let y_f64: Vec<f64> = clean_pairs.iter().map(|p| p.1).collect();

        if let Some(res) =
            crate::wetlab_calibration::test_granger_causality(&x_f64, &y_f64, max_lag)
        {
            GrangerResult {
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                f_statistic: (res.f_statistic as f32).max(0.0),
                p_value: (res.p_value as f32).clamp(0.0, 1.0),
                causal: (res.p_value as f32) < self.config.significance_threshold,
                optimal_lag: res.lag_order,
            }
        } else {
            GrangerResult {
                f_statistic: 0.0,
                p_value: 1.0,
                causal: false,
                optimal_lag: 0,
            }
        }
    }

    /// Pearson correlation
    #[must_use]
    pub fn correlation(x: &[f32], y: &[f32]) -> f32 {
        crate::stats::pearson_correlation_f32(x, y)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// PC algorithm skeleton with first-order conditional independence pruning (Spirtes et al. 2000)
    pub fn pc_skeleton(
        &mut self,
        variables: &[String],
        correlations: &HashMap<(String, String), f32>,
    ) -> CausalGraph {
        let mut graph = CausalGraph {
            nodes: variables.to_vec(),
            edges: Vec::new(),
        };

        let get_corr = |u: &str, v: &str| -> f32 {
            if u == v {
                return 1.0;
            }
            let raw = correlations
                .get(&(u.to_string(), v.to_string()))
                .or_else(|| correlations.get(&(v.to_string(), u.to_string())))
                .copied()
                .unwrap_or(0.0);
            if raw.is_finite() {
                raw.clamp(-1.0, 1.0)
            } else {
                0.0
            }
        };

        let mut candidate_pairs = Vec::new();
        for (i, var_i) in variables.iter().enumerate() {
            for var_j in variables.iter().skip(i + 1) {
                if var_i == var_j {
                    continue;
                }
                let r_ij = get_corr(var_i, var_j);
                if r_ij.abs() > self.config.min_effect_size {
                    candidate_pairs.push((var_i.clone(), var_j.clone(), r_ij.abs()));
                }
            }
        }

        for (var_i, var_j, weight) in candidate_pairs {
            let r_ij = get_corr(&var_i, &var_j);
            let mut conditionally_independent = false;

            for var_z in variables {
                if var_z == &var_i || var_z == &var_j {
                    continue;
                }
                let r_iz = get_corr(&var_i, var_z);
                let r_jz = get_corr(&var_j, var_z);

                if r_iz.abs() > self.config.min_effect_size
                    && r_jz.abs() > self.config.min_effect_size
                {
                    let denom =
                        ((1.0 - r_iz * r_iz).max(0.0) * (1.0 - r_jz * r_jz).max(0.0)).sqrt();
                    let num = r_ij - r_iz * r_jz;
                    if denom > 1e-4 {
                        let partial_r = num / denom;
                        if partial_r.abs() <= self.config.min_effect_size {
                            conditionally_independent = true;
                            break;
                        }
                    } else if num.abs() <= self.config.min_effect_size {
                        conditionally_independent = true;
                        break;
                    }
                }
            }

            if !conditionally_independent {
                graph.edges.push(CausalEdge {
                    from: var_i,
                    to: var_j,
                    weight,
                    edge_type: EdgeType::Direct,
                });
            }
        }

        self.graph = graph.clone();
        graph
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Discover causal relationships from temporal data
    #[must_use]
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn discover_causal_relationships(
        &self,
        time_series: &HashMap<String, Vec<f32>>,
    ) -> Vec<CausalRelationship> {
        let mut relationships = Vec::new();

        let mut variables: Vec<String> = time_series.keys().cloned().collect();
        variables.sort();

        for (i, cause) in variables.iter().enumerate() {
            for effect in variables.iter().skip(i + 1) {
                let (Some(x), Some(y)) = (time_series.get(cause), time_series.get(effect)) else {
                    continue;
                };

                let result_xy = self.granger_causality(x, y, self.config.max_lag);
                let result_yx = self.granger_causality(y, x, self.config.max_lag);

                let is_bidirectional = result_xy.causal && result_yx.causal;
                let mechanism = if is_bidirectional {
                    Some("bidirectional_feedback".to_string())
                } else {
                    Some("unidirectional_granger".to_string())
                };

                if result_xy.causal {
                    relationships.push(CausalRelationship {
                        cause: cause.clone(),
                        effect: effect.clone(),
                        strength: result_xy.f_statistic.min(10.0) / 10.0,
                        confidence: 1.0 - result_xy.p_value,
                        mechanism: mechanism.clone(),
                    });
                }

                if result_yx.causal {
                    relationships.push(CausalRelationship {
                        cause: effect.clone(),
                        effect: cause.clone(),
                        strength: result_yx.f_statistic.min(10.0) / 10.0,
                        confidence: 1.0 - result_yx.p_value,
                        mechanism,
                    });
                }
            }
        }

        relationships
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Estimate causal effect using do-calculus approximation
    #[must_use]
    pub fn estimate_causal_effect(
        &self,
        cause_data: &[f32],
        effect_data: &[f32],
        intervention_value: f32,
    ) -> CausalEffect {
        if !intervention_value.is_finite() {
            return CausalEffect {
                intervention_value: 0.0,
                estimated_outcome: 0.0,
                confidence_interval: (0.0, 0.0),
                method: "invalid_intervention".to_string(),
            };
        }

        let clean_pairs: Vec<(f32, f32)> = cause_data
            .iter()
            .zip(effect_data.iter())
            .filter(|(&x, &y)| x.is_finite() && y.is_finite())
            .map(|(&x, &y)| (x, y))
            .collect();

        let n = clean_pairs.len();
        if n < 2 {
            return CausalEffect {
                intervention_value,
                estimated_outcome: 0.0,
                confidence_interval: (0.0, 0.0),
                method: "insufficient_data".to_string(),
            };
        }

        let mean_cause = clean_pairs.iter().map(|p| p.0).sum::<f32>() / n as f32;
        let mean_effect = clean_pairs.iter().map(|p| p.1).sum::<f32>() / n as f32;

        let mut cov = 0.0f32;
        let mut var_cause = 0.0f32;
        for &(x, y) in &clean_pairs {
            let dx = x - mean_cause;
            let dy = y - mean_effect;
            cov += dx * dy;
            var_cause += dx * dx;
        }

        // Structural OLS slope beta_1 = Cov(X, Y) / Var(X)
        let slope = if var_cause.abs() > 1e-12 {
            cov / var_cause
        } else {
            0.0
        };

        let estimated_effect = mean_effect + slope * (intervention_value - mean_cause);

        // Compute OLS residual standard error s_e
        let mut sse = 0.0f32;
        for &(x, y) in &clean_pairs {
            let pred = mean_effect + slope * (x - mean_cause);
            let err = y - pred;
            sse += err * err;
        }
        let df = (n as f32 - 2.0).max(1.0);
        let s_e = (sse / df).sqrt();

        // Standard error of the mean response at intervention_value
        let dx_interv = intervention_value - mean_cause;
        let se_pred = if var_cause > 1e-12 {
            s_e * (1.0 / n as f32 + (dx_interv * dx_interv) / var_cause).sqrt()
        } else {
            s_e / (n as f32).sqrt()
        };
        let margin = (1.96 * se_pred).max(1e-4);

        CausalEffect {
            intervention_value,
            estimated_outcome: estimated_effect,
            confidence_interval: (estimated_effect - margin, estimated_effect + margin),
            method: "linear_approximation".to_string(),
        }
    }

    /// Identify confounders (common causes)
    #[must_use]
    pub fn identify_confounders(
        &self,
        x: &str,
        y: &str,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        candidates: &[String],
        correlations: &HashMap<(String, String), f32>,
    ) -> Vec<String> {
        let mut confounders = Vec::new();

        for candidate in candidates {
            if candidate == x || candidate == y {
                continue;
            }

            let corr_cx = correlations
                .get(&(candidate.clone(), x.to_string()))
                .or_else(|| correlations.get(&(x.to_string(), candidate.clone())))
                .copied()
                .unwrap_or(0.0);

            let corr_cy = correlations
                .get(&(candidate.clone(), y.to_string()))
                .or_else(|| correlations.get(&(y.to_string(), candidate.clone())))
                .copied()
                .unwrap_or(0.0);

            let threshold = self.config.min_effect_size.max(0.3);
            if corr_cx.abs() > threshold && corr_cy.abs() > threshold {
                confounders.push(candidate.clone());
            }
        }

        confounders
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for CausalInference {
    fn default() -> Self {
        Self::new(CausalConfig::default())
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Granger causality result
#[derive(Debug, Clone, Copy)]
pub struct GrangerResult {
    /// f statistic.
    pub f_statistic: f32,
    /// p value.
    pub p_value: f32,
    /// causal.
    pub causal: bool,
    /// optimal lag.
    pub optimal_lag: usize,
}

/// Causal effect estimate
#[derive(Debug, Clone)]
pub struct CausalEffect {
    /// intervention value.
    pub intervention_value: f32,
    /// estimated outcome.
    pub estimated_outcome: f32,
    /// confidence interval.
    pub confidence_interval: (f32, f32),
    /// method.
    pub method: String,
}

#[cfg(test)]
/// Standard complementary error function erfc(x) for x >= 0 (Abramowitz & Stegun 7.1.26, max error < 1.5e-7)
#[allow(clippy::excessive_precision)]
fn erfc_approx(x: f32) -> f32 {
    if x <= 0.0 {
        return 1.0;
    }
    if x > 10.0 {
        return 0.0;
    }
    let p = 0.3275911_f32;
    let a1 = 0.254829592_f32;
    let a2 = -0.284496736_f32;
    let a3 = 1.421413741_f32;
    let a4 = -1.453152027_f32;
    let a5 = 1.061405429_f32;

    let t = 1.0 / (1.0 + p * x);
    let poly = ((((a5 * t + a4) * t + a3) * t + a2) * t + a1) * t;
    poly * (-x * x).exp()
}

#[cfg(test)]
/// Two-tailed p-value for F(1, df2) via Wallace (1959) t-to-Z transformation:
/// z = sqrt(df2 * ln(1 + F/df2)) * (1 - 1/(4*df2))
fn f_1_df2_survival(f_stat: f32, df2: f32) -> f32 {
    if !f_stat.is_finite() || f_stat <= 0.0 || !df2.is_finite() || df2 < 1.0 {
        return 1.0;
    }
    let term = (1.0 + f_stat / df2).ln();
    let z = (df2 * term).sqrt() * (1.0 - 0.25 / df2);
    let x = z / std::f32::consts::SQRT_2;
    erfc_approx(x).clamp(0.0, 1.0)
}

#[cfg(test)]
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
mod tests {
    use super::*;

    #[test]
    fn test_f_1_df2_calibration() {
        // Critical value for Z = 1.96: Z^2 = 3.84146 -> p should be exactly 0.05
        let p = f_1_df2_survival(3.84146, 1000.0);
        assert!((p - 0.05).abs() < 0.005, "p was {p}, expected ~0.05");
    }

    #[test]
    fn test_granger_causality() {
        let inference = CausalInference::default();

        // X has rich dynamic variance
        let x: Vec<f32> = (0..120).map(|i| (i as f32 * 0.2).sin()).collect();
        // Y is causally driven by lagged X plus small innovation
        let mut y = vec![0.0f32; 120];
        for i in 1..120 {
            y[i] = 0.8 * x[i - 1] + 0.1 * ((i as f32 * 0.7).cos());
        }

        let result = inference.granger_causality(&x, &y, 3);
        assert!(result.causal, "Lagged driver must be identified as causal");
        assert!(result.f_statistic > 0.0);
    }

    #[test]
    fn test_granger_causality_rejects_independent_deterministic_ramps() {
        let inference = CausalInference::default();
        let x: Vec<f32> = (0..100).map(|i| i as f32).collect();
        let y: Vec<f32> = x.iter().map(|&xi| xi + 1.0).collect();

        let result = inference.granger_causality(&x, &y, 3);
        assert!(
            !result.causal,
            "Independent collinear ramps must not report Granger causality"
        );
        assert_eq!(result.f_statistic, 0.0);
    }

    #[test]
    fn test_pc_skeleton_prunes_mediated_chain() {
        let mut inference = CausalInference::new(CausalConfig {
            min_effect_size: 0.15,
            significance_threshold: 0.05,
            max_lag: 3,
        });

        // Causal chain: X -> Z -> Y
        // r(X, Z) = 0.8, r(Z, Y) = 0.8, r(X, Y) = 0.64
        // Partial correlation r(X, Y | Z) = (0.64 - 0.8 * 0.8) / sqrt((1 - 0.64)*(1 - 0.64)) = 0.0!
        let mut correlations = HashMap::new();
        correlations.insert(("X".to_string(), "Z".to_string()), 0.8f32);
        correlations.insert(("Z".to_string(), "Y".to_string()), 0.8f32);
        correlations.insert(("X".to_string(), "Y".to_string()), 0.64f32);

        let variables = vec!["X".to_string(), "Z".to_string(), "Y".to_string()];
        let graph = inference.pc_skeleton(&variables, &correlations);

        assert_eq!(
            graph.edges.len(),
            2,
            "Must keep (X, Z) and (Z, Y), pruning (X, Y)"
        );
        let has_xy = graph
            .edges
            .iter()
            .any(|e| (e.from == "X" && e.to == "Y") || (e.from == "Y" && e.to == "X"));
        assert!(
            !has_xy,
            "Direct edge (X, Y) must be pruned by conditional independence on Z"
        );
    }

    #[test]
    fn test_causal_discovery_collinear_mediator_and_no_self_loops() {
        let mut inference = CausalInference::new(CausalConfig {
            min_effect_size: 0.15,
            significance_threshold: 0.05,
            max_lag: 3,
        });

        // Duplicate variables must produce no self loops
        let variables_with_dups = vec!["X".to_string(), "X".to_string(), "Y".to_string()];
        let mut correlations = HashMap::new();
        correlations.insert(("X".to_string(), "Y".to_string()), 0.5f32);
        let graph = inference.pc_skeleton(&variables_with_dups, &correlations);
        assert!(
            !graph.edges.iter().any(|e| e.from == e.to),
            "No self-loops allowed"
        );

        // Perfect collinear chain: r(X, Z) = 1.0, r(Z, Y) = 1.0, r(X, Y) = 1.0
        correlations.insert(("X".to_string(), "Z".to_string()), 1.0f32);
        correlations.insert(("Z".to_string(), "Y".to_string()), 1.0f32);
        correlations.insert(("X".to_string(), "Y".to_string()), 1.0f32);
        let vars = vec!["X".to_string(), "Z".to_string(), "Y".to_string()];
        let graph_collinear = inference.pc_skeleton(&vars, &correlations);
        let has_direct_xy = graph_collinear
            .edges
            .iter()
            .any(|e| (e.from == "X" && e.to == "Y") || (e.from == "Y" && e.to == "X"));
        assert!(!has_direct_xy, "Collinear mediated link X-Y must be pruned");
    }

    #[test]
    fn test_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let corr = CausalInference::correlation(&x, &y);
        assert!((corr - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_discover_relationships() {
        let inference = CausalInference::default();

        let mut time_series = HashMap::new();
        let x: Vec<f32> = (0..60).map(|i| (i as f32 * 0.25).sin()).collect();
        let mut y = vec![0.0f32; 60];
        for i in 1..60 {
            y[i] = 0.8 * x[i - 1] + 0.1 * ((i as f32 * 0.5).cos());
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        time_series.insert("X".to_string(), x);
        time_series.insert("Y".to_string(), y);

        let relationships = inference.discover_causal_relationships(&time_series);
        assert!(!relationships.is_empty());
    }

    #[test]
    fn test_estimate_causal_effect() {
        let inference = CausalInference::default();

        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let effect = inference.estimate_causal_effect(&x, &y, 6.0);
        // mean_effect=6, mean_cause=3, slope=2, intervention=6
        // expected = 6 + 2 * (6 - 3) = 12
        assert!((effect.estimated_outcome - 12.0).abs() < 0.01);
    }

    #[test]
    fn test_identify_confounders() {
        let inference = CausalInference::default();

        let mut correlations = HashMap::new();
        correlations.insert(("Z".to_string(), "X".to_string()), 0.8);
        correlations.insert(("Z".to_string(), "Y".to_string()), 0.7);
        correlations.insert(("X".to_string(), "Y".to_string()), 0.5);

        let confounders =
            inference.identify_confounders("X", "Y", &["Z".to_string()], &correlations);
        assert!(confounders.contains(&"Z".to_string()));
    }
}
