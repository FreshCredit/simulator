// TAG: surface=simulation owner=research-team rule=SIM-001
//! Stereographic Kuramoto Phase Map & Two-Channel Observation Leakage (OSF v3.0 GTT §3.4 / TFP §3.6)
//!
//! Corrected phase map: θ_k = 2 arctan(ε̃_k) for ε̃ ~ Cauchy(0,1) maps to uniform phases on [-π, π],
//! resolving the atan(ε̃) artifact that previously inflated K=0 coherence to R ≈ 0.635.
//! Isolated baseline is R(K=0) = 0.040 ± 0.020 ≈ 1/√N.
//!
//! Two-channel leakage model:
//! 1. Coupling attenuation: K_eff = K(1 - ℓ)
//! 2. Phase observation noise: θ̂_j = θ_j + η_j, where η_j ~ N(0, (π/2 · ℓ)²)

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Configuration for Kuramoto coupled oscillator dynamics and observation leakage
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct KuramotoLeakageConfig {
    /// Global coupling strength K
    pub coupling_k: f64,
    /// Observation leakage fraction ell in [0, 1]
    pub leakage_ell: f64,
    /// Integration time step dt
    pub dt: f64,
}

impl Default for KuramotoLeakageConfig {
    fn default() -> Self {
        Self {
            coupling_k: 4.5,
            leakage_ell: 0.01,
            dt: 0.05,
        }
    }
}

/// Population of coupled phase oscillators under observation leakage
pub struct KuramotoPopulation {
    /// Number of oscillator agents
    pub n_agents: usize,
    /// Current oscillator phases theta_i in [-pi, pi]
    pub phases: Vec<f64>,
    /// Natural frequencies omega_i ~ Cauchy(0, 1)
    pub natural_frequencies: Vec<f64>,
    /// Active configuration
    pub config: KuramotoLeakageConfig,
}

/// Wrap an arbitrary angle theta into canonical circle group interval [-π, π)
#[inline]
#[must_use]
pub fn wrap_phase_pi(theta: f64) -> f64 {
    (theta + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}

impl KuramotoPopulation {
    /// Initialize a new population of oscillators with stereographic phase initialization
    #[must_use]
    pub fn new<R: Rng>(n_agents: usize, config: KuramotoLeakageConfig, rng: &mut R) -> Self {
        let mut natural_frequencies = Vec::with_capacity(n_agents);
        let mut phases = Vec::with_capacity(n_agents);

        for _ in 0..n_agents {
            // Standard Cauchy(0, 1) generator: tan(π * (u - 0.5))
            let u: f64 = rng.gen_range(0.001..0.999);
            let cauchy_omega = (std::f64::consts::PI * (u - 0.5)).tan();
            natural_frequencies.push(cauchy_omega);

            // Repaired stereographic phase map: θ = 2 * atan(cauchy_eps)
            let u_eps: f64 = rng.gen_range(0.001..0.999);
            let cauchy_eps = (std::f64::consts::PI * (u_eps - 0.5)).tan();
            let theta = wrap_phase_pi(2.0 * cauchy_eps.atan());
            phases.push(theta);
        }

        Self {
            n_agents,
            phases,
            natural_frequencies,
            config,
        }
    }

    /// Calculate complex Kuramoto order parameter (R, psi) where R in [0, 1] is coherence
    #[must_use]
    pub fn order_parameter(&self) -> (f64, f64) {
        if self.phases.is_empty() {
            return (0.0, 0.0);
        }
        let mut sum_cos = 0.0;
        let mut sum_sin = 0.0;
        for &th in &self.phases {
            sum_cos += th.cos();
            sum_sin += th.sin();
        }
        let n = self.phases.len() as f64;
        let r = (sum_cos * sum_cos + sum_sin * sum_sin).sqrt() / n;
        let psi = sum_sin.atan2(sum_cos);
        (r, psi)
    }

    fn compute_noisy_order_parameter<R: Rng>(&self, obs_noise_std: f64, rng: &mut R) -> (f64, f64) {
        let mut sum_obs_cos = 0.0;
        let mut sum_obs_sin = 0.0;
        for &th in &self.phases {
            let u1: f64 = rng.gen_range(1e-7..1.0);
            let u2: f64 = rng.gen_range(0.0..std::f64::consts::TAU);
            let noise = if obs_noise_std > 1e-6 {
                (-2.0 * u1.ln()).sqrt() * u2.cos() * obs_noise_std
            } else {
                0.0
            };
            let th_obs = th + noise;
            sum_obs_cos += th_obs.cos();
            sum_obs_sin += th_obs.sin();
        }

        let n = self.n_agents.max(1) as f64;
        let r_obs = (sum_obs_cos * sum_obs_cos + sum_obs_sin * sum_obs_sin).sqrt() / n;
        let psi_obs = sum_obs_sin.atan2(sum_obs_cos);
        (r_obs, psi_obs)
    }

    /// Advance oscillator population by one time step dt with leakage attenuation and observation noise
    pub fn step<R: Rng>(&mut self, rng: &mut R) {
        let k_eff = self.config.coupling_k * (1.0 - self.config.leakage_ell).max(0.0);
        let obs_noise_std = (std::f64::consts::PI / 2.0) * self.config.leakage_ell;
        let (r_obs, psi_obs) = self.compute_noisy_order_parameter(obs_noise_std, rng);

        for i in 0..self.n_agents {
            let d_theta =
                self.natural_frequencies[i] + k_eff * r_obs * (psi_obs - self.phases[i]).sin();
            self.phases[i] = wrap_phase_pi(self.phases[i] + d_theta * self.config.dt);
        }
    }

    /// Sim O3: Advance oscillator population with per-agent optogenetic coupling modulation
    pub fn step_with_optogenetics<R: Rng>(
        &mut self,
        opto: &crate::optogenetics::OptogeneticConfig,
        rng: &mut R,
    ) {
        let base_k_eff = self.config.coupling_k * (1.0 - self.config.leakage_ell).max(0.0);
        let obs_noise_std = (std::f64::consts::PI / 2.0) * self.config.leakage_ell;
        let (r_obs, psi_obs) = self.compute_noisy_order_parameter(obs_noise_std, rng);

        for i in 0..self.n_agents {
            let is_perturbed = opto.is_agent_perturbed(i as u64, 5, self.n_agents);
            let k_i = opto.modulate_kuramoto_coupling(base_k_eff as f32, is_perturbed) as f64;
            let d_theta =
                self.natural_frequencies[i] + k_i * r_obs * (psi_obs - self.phases[i]).sin();
            self.phases[i] = wrap_phase_pi(self.phases[i] + d_theta * self.config.dt);
        }
    }

    /// Sim P9 / TDF: Advance oscillator population with dynamic metric hierarchy and TDF music vector coupling
    pub fn step_with_music_tdf<R: Rng>(&mut self, tdf: &crate::music::TdfMusicVector, rng: &mut R) {
        let dynamic_boost = 1.2 * (f64::from(tdf.metric_hierarchy_tau));
        let phase_align = 1.0 + 0.5 * (f64::from(tdf.phase_dynamics)).cos();
        let base_k_eff = (self.config.coupling_k + dynamic_boost)
            * phase_align
            * (1.0 - self.config.leakage_ell).max(0.0);
        let obs_noise_std = (std::f64::consts::PI / 2.0) * self.config.leakage_ell;
        let (r_obs, psi_obs) = self.compute_noisy_order_parameter(obs_noise_std, rng);

        for i in 0..self.n_agents {
            let d_theta =
                self.natural_frequencies[i] + base_k_eff * r_obs * (psi_obs - self.phases[i]).sin();
            self.phases[i] = wrap_phase_pi(self.phases[i] + d_theta * self.config.dt);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_stereographic_map_uncoupled_order_parameter() {
        let mut rng = StdRng::seed_from_u64(42);
        let config = KuramotoLeakageConfig {
            coupling_k: 0.0,
            leakage_ell: 0.0,
            dt: 0.05,
        };
        let pop = KuramotoPopulation::new(500, config, &mut rng);
        let (r, _) = pop.order_parameter();
        // Finite size floor for N=500 is 1/√500 ≈ 0.0447. R should be near 0.04 ± 0.02, NOT 0.635!
        assert!(r < 0.12, "R was {r}, expected near finite-size floor ~0.04");
    }

    #[test]
    fn test_sim_o3_kuramoto_mutant_resilience() {
        let mut rng_ctrl = StdRng::seed_from_u64(42);
        let mut rng_mut = StdRng::seed_from_u64(42);
        let config = KuramotoLeakageConfig {
            coupling_k: 4.5,
            leakage_ell: 0.01,
            dt: 0.05,
        };
        let mut pop_ctrl = KuramotoPopulation::new(200, config, &mut rng_ctrl);
        let mut pop_mut = KuramotoPopulation::new(200, config, &mut rng_mut);

        let opto_off = crate::optogenetics::OptogeneticConfig::default();
        let mut opto_mut = crate::optogenetics::OptogeneticConfig::kuramoto_os_mutant_subpop(0.10);
        opto_mut.step_kinetics(true);

        for _ in 0..50 {
            pop_ctrl.step_with_optogenetics(&opto_off, &mut rng_ctrl);
            pop_mut.step_with_optogenetics(&opto_mut, &mut rng_mut);
        }

        let (r_ctrl, _) = pop_ctrl.order_parameter();
        let (r_mut, _) = pop_mut.order_parameter();

        assert!(r_ctrl > 0.40, "Coupled network must achieve coherence");
        assert!(
            r_mut > 0.30,
            "10% mutant subpopulation must maintain macroscopic resilience"
        );
        let drop_pct = (r_ctrl - r_mut) / r_ctrl;
        assert!(
            drop_pct < 0.40,
            "Coherence collapse ({drop_pct:.2}) must be < 40% threshold (Hypothesis O3 confirms resilience)"
        );
    }

    #[test]
    fn test_coupled_coherence_above_critical() {
        let mut rng = StdRng::seed_from_u64(42);
        let config = KuramotoLeakageConfig {
            coupling_k: 6.0,
            leakage_ell: 0.0,
            dt: 0.05,
        };
        let mut pop = KuramotoPopulation::new(200, config, &mut rng);
        for _ in 0..400 {
            pop.step(&mut rng);
        }
        let (r, _) = pop.order_parameter();
        assert!(r > 0.50, "R was {r}, expected strong coherence at K=6.0");
    }
}
