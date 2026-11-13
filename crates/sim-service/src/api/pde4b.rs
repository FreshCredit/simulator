//! PDE4B Configuration API Module
//!
//! Provides REST endpoints for configuring PDE4B variance levels and retrieving
//! lambda distribution statistics.
//!
//! ## Routes
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET | `/api/config/pde4b` | Get current PDE4B configuration |
//! | POST | `/api/config/pde4b` | Update PDE4B configuration |
//! | GET | `/api/stats/lambda` | Get lambda distribution statistics |

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// PDE4B configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pde4bConfig {
    /// Variance level: "zero", "low", "medium", "high"
    pub variance: String,
    /// Mean PDE4B level (0.0 to 1.0)
    pub mean: f64,
    /// Standard deviation of PDE4B level
    pub std_dev: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for Pde4bConfig {
    fn default() -> Self {
        Self {
            variance: "medium".to_string(),
            mean: 0.5,
            std_dev: 0.10,
        }
    }
}

impl Pde4bConfig {
    /// Create a new `Pde4bConfig` from a variance level string
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn from_variance(variance: &str) -> Result<Self, String> {
        match variance {
            "zero" => Ok(Self {
                variance: "zero".to_string(),
                mean: 0.5,
                std_dev: 0.0,
            }),
            "low" => Ok(Self {
                variance: "low".to_string(),
                mean: 0.5,
                std_dev: 0.01,
            }),
            "medium" => Ok(Self {
                variance: "medium".to_string(),
                mean: 0.5,
                std_dev: 0.10,
            }),
            "high" => Ok(Self {
                variance: "high".to_string(),
                mean: 0.5,
                std_dev: 0.15,
            }),
            _ => Err(format!(
                "Invalid variance level: {variance}. Must be one of: zero, low, medium, high"
            )),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get the PDE4B level range for the current variance
    #[must_use]
    pub fn range(&self) -> (f64, f64) {
        match self.variance.as_str() {
            "zero" => (0.5, 0.5),
            "low" => (0.48, 0.52),
            "medium" => (0.30, 0.70),
            "high" => (0.20, 0.80),
            _ => (0.30, 0.70),
        }
    }

    /// Calculate lambda from PDE4B level using: λ = 0.61 × (1.5 - 0.5 × `pde4b_level`)
    #[must_use]
    pub fn calculate_lambda(&self, pde4b_level: f64) -> f64 {
        0.61 * (1.5 - 0.5 * pde4b_level)
    }

    /// Get lambda statistics for current configuration
    #[must_use]
    pub fn lambda_stats(&self) -> LambdaStats {
        let (min_pde4b, max_pde4b) = self.range();

        // For zero variance, lambda is fixed
        if self.variance == "zero" {
            let lambda = self.calculate_lambda(self.mean);
            return LambdaStats {
                mean: lambda,
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                std_dev: 0.0,
                min: lambda,
                max: lambda,
                range: (lambda, lambda),
            };
        }

        // Calculate lambda at boundaries
        let lambda_at_min_pde4b = self.calculate_lambda(min_pde4b);
        let lambda_at_max_pde4b = self.calculate_lambda(max_pde4b);

        // For normal distribution, lambda is linear transform of pde4b
        // E[λ] = 0.61 * (1.5 - 0.5 * E[pde4b]) = 0.61 * (1.5 - 0.5 * mean)
        let lambda_mean = self.calculate_lambda(self.mean);

        // Var[λ] = (0.61 * 0.5)^2 * Var[pde4b] = 0.305^2 * std_dev^2
        let lambda_std = 0.305 * self.std_dev;

        LambdaStats {
            mean: lambda_mean,
            std_dev: lambda_std,
            min: lambda_at_max_pde4b.min(lambda_at_min_pde4b),
            max: lambda_at_max_pde4b.max(lambda_at_min_pde4b),
            range: (lambda_at_max_pde4b, lambda_at_min_pde4b),
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Lambda distribution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LambdaStats {
    /// Mean lambda value
    pub mean: f64,
    /// Standard deviation of lambda
    pub std_dev: f64,
    /// Minimum lambda value (at max PDE4B)
    pub min: f64,
    /// Maximum lambda value (at min PDE4B)
    pub max: f64,
    /// Lambda range tuple (min, max)
    pub range: (f64, f64),
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Shared state for PDE4B configuration
#[derive(Debug, Clone)]
pub struct Pde4bState {
    /// config
    pub config: Arc<RwLock<Pde4bConfig>>,
}

impl Pde4bState {
    #[must_use]
    /// new
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(Pde4bConfig::default())),
        }
    }
}

impl Default for Pde4bState {
    fn default() -> Self {
        Self::new()
    }
}

/// Request body for updating PDE4B configuration
#[derive(Debug, Deserialize)]
pub struct UpdatePde4bRequest {
    /// variance
    pub variance: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Response body for PDE4B configuration
#[derive(Debug, Serialize)]
pub struct Pde4bResponse {
    /// config
    pub config: Pde4bConfig,
    /// lambda stats
    pub lambda_stats: LambdaStats,
}

/// GET /api/config/pde4b - Get current PDE4B configuration
pub async fn get_pde4b_config(State(state): State<Pde4bState>) -> impl IntoResponse {
    let config = state.config.read().await.clone();
    let lambda_stats = config.lambda_stats();

    info!(
        "Retrieved PDE4B configuration: variance={}",
        config.variance
    );

    Json(Pde4bResponse {
        config,
        lambda_stats,
    })
}

/// POST /api/config/pde4b - Update PDE4B configuration
pub async fn update_pde4b_config(
    State(state): State<Pde4bState>,
    Json(req): Json<UpdatePde4bRequest>,
) -> impl IntoResponse {
    // Validate variance level
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    let variance_lower = req.variance.to_lowercase();

    match Pde4bConfig::from_variance(&variance_lower) {
        Ok(new_config) => {
            let mut config = state.config.write().await;
            *config = new_config.clone();

            let lambda_stats = config.lambda_stats();

            info!(
                "Updated PDE4B configuration: variance={}",
                new_config.variance
            );

            Json(Pde4bResponse {
                config: new_config,
                lambda_stats,
            })
            .into_response()
        }
        Err(err) => {
            warn!("Invalid PDE4B variance level requested: {}", req.variance);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": err,
                    "valid_values": ["zero", "low", "medium", "high"]
                })),
            )
                .into_response()
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// GET /api/stats/lambda - Get lambda distribution statistics
pub async fn get_lambda_stats(State(state): State<Pde4bState>) -> impl IntoResponse {
    let config = state.config.read().await.clone();
    let lambda_stats = config.lambda_stats();

    info!(
        "Retrieved lambda statistics for variance={}",
        config.variance
    );

    Json(serde_json::json!({
        "variance": config.variance,
        "pde4b_mean": config.mean,
        "pde4b_std_dev": config.std_dev,
        "lambda": {
            "mean": lambda_stats.mean,
            "std_dev": lambda_stats.std_dev,
            "min": lambda_stats.min,
            "max": lambda_stats.max,
            "range": lambda_stats.range,
        },
        "formula": "λ = 0.61 × (1.5 - 0.5 × pde4b_level)",
    }))
}

// ============================================================================
// Tests
// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn test_pde4b_config_default() {
        let config = Pde4bConfig::default();
        assert_eq!(config.variance, "medium");
        assert_eq!(config.mean, 0.5);
        assert_eq!(config.std_dev, 0.10);
    }

    #[test]
    fn test_pde4b_config_from_variance() {
        let zero = Pde4bConfig::from_variance("zero").unwrap();
        assert_eq!(zero.variance, "zero");
        assert_eq!(zero.std_dev, 0.0);

        let low = Pde4bConfig::from_variance("low").unwrap();
        assert_eq!(low.variance, "low");
        assert_eq!(low.std_dev, 0.01);

        let medium = Pde4bConfig::from_variance("medium").unwrap();
        assert_eq!(medium.variance, "medium");
        assert_eq!(medium.std_dev, 0.10);

        let high = Pde4bConfig::from_variance("high").unwrap();
        assert_eq!(high.variance, "high");
        assert_eq!(high.std_dev, 0.15);
    }

    #[test]
    fn test_pde4b_config_invalid_variance() {
        let result = Pde4bConfig::from_variance("invalid");
        assert!(result.is_err());
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_calculate_lambda() {
        let config = Pde4bConfig::default();

        // λ = 0.61 × (1.5 - 0.5 × pde4b_level)
        // At pde4b = 0.5: λ = 0.61 × (1.5 - 0.25) = 0.61 × 1.25 = 0.7625
        let lambda = config.calculate_lambda(0.5);
        assert!((lambda - 0.7625).abs() < 0.0001);

        // At pde4b = 0.2: λ = 0.61 × (1.5 - 0.1) = 0.61 × 1.4 = 0.854
        let lambda = config.calculate_lambda(0.2);
        assert!((lambda - 0.854).abs() < 0.0001);

        // At pde4b = 0.8: λ = 0.61 × (1.5 - 0.4) = 0.61 × 1.1 = 0.671
        let lambda = config.calculate_lambda(0.8);
        assert!((lambda - 0.671).abs() < 0.0001);
    }

    #[test]
    fn test_lambda_stats_zero_variance() {
        let config = Pde4bConfig::from_variance("zero").unwrap();
        let stats = config.lambda_stats();

        // With zero variance, lambda is fixed at mean
        let expected_lambda = config.calculate_lambda(0.5);
        assert!((stats.mean - expected_lambda).abs() < 0.0001);
        assert_eq!(stats.std_dev, 0.0);
        assert!((stats.min - expected_lambda).abs() < 0.0001);
        assert!((stats.max - expected_lambda).abs() < 0.0001);
    }

    #[test]
    fn test_lambda_stats_medium_variance() {
        let config = Pde4bConfig::from_variance("medium").unwrap();
        let stats = config.lambda_stats();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Mean should be at pde4b = 0.5
        let expected_mean = config.calculate_lambda(0.5);
        assert!((stats.mean - expected_mean).abs() < 0.0001);

        // std_dev = 0.305 * 0.10 = 0.0305
        assert!((stats.std_dev - 0.0305).abs() < 0.0001);

        // Range should be from lambda at pde4b=0.7 to lambda at pde4b=0.3
        let lambda_at_30 = config.calculate_lambda(0.30);
        let lambda_at_70 = config.calculate_lambda(0.70);
        assert!((stats.min - lambda_at_70).abs() < 0.0001);
        assert!((stats.max - lambda_at_30).abs() < 0.0001);
    }

    #[test]
    fn test_range() {
        let zero = Pde4bConfig::from_variance("zero").unwrap();
        assert_eq!(zero.range(), (0.5, 0.5));

        let low = Pde4bConfig::from_variance("low").unwrap();
        assert_eq!(low.range(), (0.48, 0.52));

        let medium = Pde4bConfig::from_variance("medium").unwrap();
        assert_eq!(medium.range(), (0.30, 0.70));

        let high = Pde4bConfig::from_variance("high").unwrap();
        assert_eq!(high.range(), (0.20, 0.80));
    }
}
