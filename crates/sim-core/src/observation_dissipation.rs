// TAG: surface=simulation owner=research-team rule=SIM-001
//! Observation-Salience Dissipation & Critical Slowing (OSF v3.0 GTT §3.3 / TFP §1.2 & §4)
//!
//! Models mismatch dynamics as an Ornstein–Uhlenbeck process with observation salience m:
//! dε̃ = -(μ + ψ(m)) ε̃ dt + σ̃ dW
//!
//! Signatures:
//! - Stationary variance: Var_stat(ε̃) = σ̃² / (2(μ + ψ(m)))
//! - Autocorrelation time: τ_c = 1 / (μ + ψ(m)), with lag-1 correlation ρ(1) = exp(-(μ + ψ(m)))
//! - Critical slowing as ψ → -μ⁺ (surveillance chilling)

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Configuration for observation-salience dissipation and OU mismatch dynamics
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ObservationDissipationConfig {
    /// Corrective bandwidth mu > 0
    pub mu_bandwidth: f64,
    /// Salience shift psi(m) > -mu
    pub psi_salience: f64,
    /// Innovation noise amplitude sigma
    pub innovation_sigma: f64,
    /// Integration time step dt
    pub dt: f64,
}

impl Default for ObservationDissipationConfig {
    fn default() -> Self {
        Self {
            mu_bandwidth: 1.0,
            psi_salience: -0.05, // TFP user-owned residual salience
            innovation_sigma: 1.0,
            dt: 0.01,
        }
    }
}

impl ObservationDissipationConfig {
    /// Compute effective corrective dissipation rate mu_eff = max(mu + psi(m), 1e-4)
    #[must_use]
    pub fn effective_dissipation(&self) -> f64 {
        let mu = if self.mu_bandwidth.is_finite() {
            self.mu_bandwidth
        } else {
            1.0
        };
        let psi = if self.psi_salience.is_finite() {
            self.psi_salience
        } else {
            0.0
        };
        let mu_eff = mu + psi;
        mu_eff.max(1e-4)
    }

    /// Theoretical stationary variance: Var(eps) = sigma^2 / (2 mu_eff)
    #[must_use]
    pub fn theoretical_stationary_variance(&self) -> f64 {
        let mu_eff = self.effective_dissipation();
        let sigma = if self.innovation_sigma.is_finite() && self.innovation_sigma >= 0.0 {
            self.innovation_sigma
        } else {
            0.0
        };
        (sigma * sigma) / (2.0 * mu_eff)
    }

    /// Theoretical autocorrelation time tau_c = 1 / mu_eff
    #[must_use]
    pub fn theoretical_autocorrelation_time(&self) -> f64 {
        1.0 / self.effective_dissipation()
    }

    /// Theoretical lag-autocorrelation rho(lag) = exp(-mu_eff * |lag|)
    #[must_use]
    pub fn theoretical_lag_autocorrelation(&self, lag_time: f64) -> f64 {
        let lag = if lag_time.is_finite() {
            lag_time.abs()
        } else {
            0.0
        };
        (-self.effective_dissipation() * lag).exp()
    }

    /// Exact OU discretization step:
    /// x' = exp(-μ_eff dt) x + σ̃ sqrt((1 - exp(-2 μ_eff dt))/(2 μ_eff)) * ξ
    #[must_use]
    pub fn step_exact<R: Rng>(&self, current_epsilon: f64, rng: &mut R) -> f64 {
        if !current_epsilon.is_finite() || !self.dt.is_finite() || self.dt <= 0.0 {
            return 0.0;
        }
        let mu_eff = self.effective_dissipation();
        let dt = self.dt.max(0.0);
        let decay = (-mu_eff * dt).exp();
        let noise_var = (1.0 - (-2.0 * mu_eff * dt).exp()) / (2.0 * mu_eff);
        let sigma = if self.innovation_sigma.is_finite() && self.innovation_sigma >= 0.0 {
            self.innovation_sigma
        } else {
            0.0
        };
        let noise_std = (sigma * noise_var.max(0.0).sqrt()).max(0.0);
        let u1: f64 = rng.gen_range(1e-7..1.0);
        let u2: f64 = rng.gen_range(0.0..std::f64::consts::TAU);
        let standard_normal = (-2.0 * u1.ln()).sqrt() * u2.cos();
        decay * current_epsilon + noise_std * standard_normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_theoretical_predictions() {
        let config = ObservationDissipationConfig {
            mu_bandwidth: 1.0,
            psi_salience: 0.0,
            innovation_sigma: 1.0,
            dt: 0.01,
        };
        assert!((config.theoretical_stationary_variance() - 0.50).abs() < 1e-9);
        assert!((config.theoretical_autocorrelation_time() - 1.00).abs() < 1e-9);
        assert!((config.theoretical_lag_autocorrelation(1.0) - (-1.0_f64).exp()).abs() < 1e-9);
    }

    #[test]
    fn test_ou_simulation_convergence() {
        let config = ObservationDissipationConfig {
            mu_bandwidth: 1.0,
            psi_salience: 0.0,
            innovation_sigma: 1.0,
            dt: 0.05,
        };
        let mut rng = StdRng::seed_from_u64(42);
        let mut eps = 0.0;
        let mut sum = 0.0;
        let mut sum_sq = 0.0;
        let n = 20_000;
        for _ in 0..n {
            eps = config.step_exact(eps, &mut rng);
            sum += eps;
            sum_sq += eps * eps;
        }
        let mean = sum / n as f64;
        let var = (sum_sq / n as f64) - (mean * mean);
        assert!((var - 0.50).abs() < 0.05);
    }

    #[test]
    fn test_step_exact_guards_non_finite() {
        let config = ObservationDissipationConfig::default();
        let mut rng = StdRng::seed_from_u64(42);
        assert_eq!(config.step_exact(f64::NAN, &mut rng), 0.0);

        let config_neg_dt = ObservationDissipationConfig {
            dt: -0.01,
            ..Default::default()
        };
        assert_eq!(config_neg_dt.step_exact(1.0, &mut rng), 0.0);
    }

    #[test]
    fn test_theoretical_lag_autocorrelation_symmetry() {
        let config = ObservationDissipationConfig::default();
        let rho_pos = config.theoretical_lag_autocorrelation(2.5);
        let rho_neg = config.theoretical_lag_autocorrelation(-2.5);
        assert!((rho_pos - rho_neg).abs() < 1e-9);
        assert!(rho_pos > 0.0 && rho_pos <= 1.0);

        let rho_nan = config.theoretical_lag_autocorrelation(f64::NAN);
        assert_eq!(rho_nan, 1.0);
    }
}
