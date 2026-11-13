//! Bayesian Validation (P2-RES-5)
//!
//! Bayesian model validation and comparison.

use serde::{Deserialize, Serialize};

/// Lanczos approximation for log-gamma function ln(Γ(z)) (g=7, N=9) with Euler reflection for z < 0.5.
#[must_use]
pub fn ln_gamma(z: f64) -> f64 {
    if z <= 0.0 || !z.is_finite() {
        return 0.0;
    }
    // Euler reflection formula for z < 0.5: ln(Γ(z)) = ln(π) - ln(sin(πz)) - ln(Γ(1 - z))
    if z < 0.5 {
        let sin_pi_z = (std::f64::consts::PI * z).sin();
        if sin_pi_z <= 0.0 || !sin_pi_z.is_finite() {
            return 0.0;
        }
        return std::f64::consts::PI.ln() - sin_pi_z.ln() - ln_gamma(1.0 - z);
    }
    const P: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_827_201_4,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    let z = z - 1.0;
    let mut x = P[0];
    for (i, &p) in P.iter().enumerate().skip(1) {
        x += p / (z + i as f64);
    }
    let t = z + 7.5;
    0.5 * (2.0 * std::f64::consts::PI).ln() + (z + 0.5) * t.ln() - t + x.ln()
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Bayesian model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BayesianModel {
    /// name.
    pub name: String,
    /// parameters.
    pub parameters: Vec<Parameter>,
    /// likelihood.
    pub likelihood: LikelihoodFunction,
    /// prior.
    pub prior: PriorDistribution,
}

/// Model parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// name.
    pub name: String,
    /// prior mean.
    pub prior_mean: f64,
    /// prior std.
    pub prior_std: f64,
    /// posterior mean.
    pub posterior_mean: Option<f64>,
    /// posterior std.
    pub posterior_std: Option<f64>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Likelihood function type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LikelihoodFunction {
    /// gaussian.
    Gaussian,
    /// binomial.
    Binomial,
    /// poisson.
    Poisson,
    /// exponential.
    Exponential,
}

/// Prior distribution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PriorDistribution {
    /// gaussian.
    Gaussian,
    /// uniform.
    Uniform,
    /// beta.
    Beta,
    /// gamma.
    Gamma,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// MCMC configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McmcConfig {
    /// n samples.
    pub n_samples: usize,
    /// burn in.
    pub burn_in: usize,
    /// thinning.
    pub thinning: usize,
    /// step size.
    pub step_size: f64,
    /// random seed for reproducible MCMC sampling
    pub seed: Option<u64>,
}

impl Default for McmcConfig {
    fn default() -> Self {
        Self {
            n_samples: 1000,
            burn_in: 100,
            thinning: 1,
            step_size: 0.1,
            seed: Some(42),
        }
    }
}

/// Bayesian validator
#[derive(Debug, Clone)]
pub struct BayesianValidator {
    /// mcmc config.
    pub mcmc_config: McmcConfig,
}
// POT-005: Beta moments to shape parameters helper
fn beta_moments_to_params(mean: f64, std: f64) -> (f64, f64) {
    let mean = if mean.is_finite() {
        mean.clamp(1e-4, 1.0 - 1e-4)
    } else {
        0.5
    };
    let std = if std.is_finite() { std.max(1e-4) } else { 0.1 };
    let max_var = mean * (1.0 - mean);
    let var = std.powi(2).clamp(1e-6, 0.99 * max_var);
    let factor = (max_var / var) - 1.0;
    ((mean * factor).max(1e-4), ((1.0 - mean) * factor).max(1e-4))
}

// POT-005: Gamma moments to shape/rate parameters helper
fn gamma_moments_to_params(mean: f64, std: f64) -> (f64, f64) {
    let mean = if mean.is_finite() {
        mean.max(1e-4)
    } else {
        1.0
    };
    let std = if std.is_finite() { std.max(1e-3) } else { 1.0 };
    let var = std.powi(2);
    let shape = (mean * mean / var).max(1e-4);
    let rate = (mean / var).max(1e-4);
    (shape, rate)
}

// POT-005: Parameter MLE resolver helper
fn resolve_param_mle(
    likelihood: LikelihoodFunction,
    idx: usize,
    post: Option<f64>,
    sample_mean: f64,
    sample_var: f64,
    prior: f64,
) -> f64 {
    if let Some(p) = post {
        match (likelihood, idx) {
            (LikelihoodFunction::Binomial, 0) => p.clamp(1e-5, 1.0 - 1e-5),
            (LikelihoodFunction::Poisson, 0) => p.max(1e-5),
            (LikelihoodFunction::Exponential, 0) => p.max(1e-5),
            (LikelihoodFunction::Gaussian, 1) => p.max(1e-5),
            _ => p,
        }
    } else {
        match (likelihood, idx) {
            (LikelihoodFunction::Gaussian, 0) => sample_mean,
            (LikelihoodFunction::Gaussian, 1) => sample_var.sqrt().max(1e-5),
            (LikelihoodFunction::Binomial, 0) => sample_mean.clamp(1e-5, 1.0 - 1e-5),
            (LikelihoodFunction::Poisson, 0) => sample_mean.max(1e-5),
            (LikelihoodFunction::Exponential, 0) => {
                if sample_mean > 1e-5 {
                    1.0 / sample_mean
                } else {
                    1.0
                }
            }
            _ => prior,
        }
    }
}

fn gaussian_log_density_val(x: f64, mean: f64, std: f64) -> f64 {
    -0.5 * ((x - mean) / std).powi(2) - (std * (2.0 * std::f64::consts::PI).sqrt()).ln()
}

#[inline]
fn is_invalid_data(data: &[f64]) -> bool {
    data.is_empty() || data.iter().any(|x| !x.is_finite())
}

#[inline]
fn is_invalid_non_negative_data(data: &[f64]) -> bool {
    is_invalid_data(data) || data.iter().any(|x| *x < 0.0)
}

fn gaussian_log_likelihood(params: &[f64], data: &[f64]) -> f64 {
    if params.len() < 2 || is_invalid_data(data) {
        return f64::NEG_INFINITY;
    }
    let mean = params[0];
    let std = params[1];
    if !mean.is_finite() || !std.is_finite() || std <= 0.0 {
        return f64::NEG_INFINITY;
    }
    data.iter()
        .map(|&x| gaussian_log_density_val(x, mean, std))
        .sum()
}

fn binomial_log_likelihood(params: &[f64], data: &[f64]) -> f64 {
    if params.is_empty() || is_invalid_data(data) {
        return f64::NEG_INFINITY;
    }
    let p = params[0];
    if !p.is_finite() || p <= 0.0 || p >= 1.0 {
        return f64::NEG_INFINITY;
    }
    data.iter()
        .map(|&x| if x >= 0.5 { p.ln() } else { (1.0 - p).ln() })
        .sum()
}

fn poisson_log_likelihood(params: &[f64], data: &[f64]) -> f64 {
    if params.is_empty() || is_invalid_non_negative_data(data) {
        return f64::NEG_INFINITY;
    }
    let lambda = params[0];
    if !lambda.is_finite() || lambda <= 0.0 {
        return f64::NEG_INFINITY;
    }
    data.iter()
        .map(|&x| {
            let k = x.round();
            k * lambda.ln() - lambda - ln_gamma(k + 1.0)
        })
        .sum()
}

fn exponential_log_likelihood(params: &[f64], data: &[f64]) -> f64 {
    if params.is_empty() || is_invalid_non_negative_data(data) {
        return f64::NEG_INFINITY;
    }
    let rate = params[0];
    if !rate.is_finite() || rate <= 0.0 {
        return f64::NEG_INFINITY;
    }
    data.iter().map(|&x| rate.ln() - rate * x).sum()
}

impl BayesianValidator {
    #[must_use]
    /// new.
    pub const fn new(config: McmcConfig) -> Self {
        Self {
            mcmc_config: config,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Approximate posterior using MCMC (Metropolis-Hastings simplified)
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn approximate_posterior(
        &self,
        model: &mut BayesianModel,
        data: &[f64],
    ) -> PosteriorApproximation {
        use rand::Rng;
        use rand::SeedableRng;

        let mut rng = rand::rngs::StdRng::seed_from_u64(self.mcmc_config.seed.unwrap_or(42));
        let mut samples: Vec<Vec<f64>> = Vec::with_capacity(self.mcmc_config.n_samples);
        // Clamp initial parameters into valid distribution interior to prevent -inf trap
        let mut current_params: Vec<f64> = model
            .parameters
            .iter()
            .map(|p| match model.prior {
                PriorDistribution::Beta => p.prior_mean.clamp(1e-4, 1.0 - 1e-4),
                PriorDistribution::Gamma => p.prior_mean.max(1e-4),
                PriorDistribution::Gaussian | PriorDistribution::Uniform => p.prior_mean,
            })
            .collect();
        let mut current_log_prob = self.log_posterior(&current_params, data, model);

        let mut accepted = 0;
        let thinning = self.mcmc_config.thinning.max(1);
        let total_iterations = self.mcmc_config.burn_in + self.mcmc_config.n_samples * thinning;

        for step in 0..total_iterations {
            let proposed_params: Vec<f64> = current_params
                .iter()
                .map(|&p| p + (rng.gen::<f64>() - 0.5) * 2.0 * self.mcmc_config.step_size)
                .collect();

            let proposed_log_prob = self.log_posterior(&proposed_params, data, model);

            let log_diff = proposed_log_prob - current_log_prob;
            let acceptance_prob = if !proposed_log_prob.is_finite() {
                0.0
            } else if !current_log_prob.is_finite() || log_diff >= 0.0 {
                1.0
            } else {
                log_diff.exp()
            };

            if rng.gen::<f64>() < acceptance_prob {
                current_params = proposed_params;
                current_log_prob = proposed_log_prob;
                accepted += 1;
            }

            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            if step >= self.mcmc_config.burn_in
                && (step - self.mcmc_config.burn_in) % thinning == 0
                && samples.len() < self.mcmc_config.n_samples
            {
                samples.push(current_params.clone());
            }
        }

        if samples.is_empty() {
            samples.push(current_params.clone());
        }

        let acceptance_rate = if total_iterations > 0 {
            accepted as f64 / total_iterations as f64
        } else {
            0.0
        };

        let mut min_ess = samples.len() as f64;
        for (i, param) in model.parameters.iter_mut().enumerate() {
            let param_samples: Vec<f64> = samples.iter().map(|s| s[i]).collect();
            let k = param_samples.len();
            let mean = param_samples.iter().sum::<f64>() / k.max(1) as f64;
            param.posterior_mean = Some(mean);

            let variance = if k > 1 {
                param_samples
                    .iter()
                    .map(|&s| (s - mean).powi(2))
                    .sum::<f64>()
                    / (k - 1) as f64
            } else {
                0.0
            };
            param.posterior_std = Some(variance.sqrt());

            // Lag-1 autocorrelation for effective sample size (Geyer 1992, Vehtari et al. 2021)
            let param_ess = if k > 2 && variance > 1e-12 {
                let cov1: f64 = param_samples
                    .windows(2)
                    .map(|w| (w[0] - mean) * (w[1] - mean))
                    .sum::<f64>()
                    / (k - 1) as f64;
                let r1 = (cov1 / variance).clamp(-0.99, 0.99);
                if r1 < 1.0 {
                    (samples.len() as f64 * (1.0 - r1) / (1.0 + r1))
                        .clamp(1.0, samples.len() as f64)
                } else {
                    1.0
                }
            } else if k > 1 && variance <= 1e-12 {
                // Degenerate/frozen chain: all samples identical, only 1 unique independent state
                1.0
            } else {
                samples.len() as f64
            };
            if param_ess < min_ess {
                min_ess = param_ess;
            }
        }

        PosteriorApproximation {
            samples,
            acceptance_rate,
            effective_sample_size: min_ess.max(1.0),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Log posterior (log likelihood + log prior)
    fn log_posterior(&self, params: &[f64], data: &[f64], model: &BayesianModel) -> f64 {
        let log_likelihood = self.log_likelihood(params, data, model);
        let log_prior: f64 = model
            .parameters
            .iter()
            .zip(params.iter())
            .map(|(p, &v)| match model.prior {
                PriorDistribution::Gaussian => {
                    self.gaussian_log_density(v, p.prior_mean, p.prior_std.max(0.001))
                }
                PriorDistribution::Uniform => {
                    let half_width = p.prior_std.max(0.001);
                    if (v - p.prior_mean).abs() <= half_width {
                        -(2.0 * half_width).ln()
                    } else {
                        f64::NEG_INFINITY
                    }
                }
                PriorDistribution::Beta => {
                    let (a, b) = beta_moments_to_params(p.prior_mean, p.prior_std);
                    if v > 0.0 && v < 1.0 {
                        let norm = ln_gamma(a + b) - ln_gamma(a) - ln_gamma(b);
                        norm + (a - 1.0) * v.ln() + (b - 1.0) * (1.0 - v).ln()
                    } else {
                        f64::NEG_INFINITY
                    }
                }
                PriorDistribution::Gamma => {
                    let (shape, rate) = gamma_moments_to_params(p.prior_mean, p.prior_std);
                    if v > 0.0 {
                        shape * rate.ln() - ln_gamma(shape) + (shape - 1.0) * v.ln() - rate * v
                    } else {
                        f64::NEG_INFINITY
                    }
                }
            })
            .sum();

        log_likelihood + log_prior
    }

    /// Log likelihood
    fn log_likelihood(&self, params: &[f64], data: &[f64], model: &BayesianModel) -> f64 {
        match model.likelihood {
            LikelihoodFunction::Gaussian => gaussian_log_likelihood(params, data),
            LikelihoodFunction::Binomial => binomial_log_likelihood(params, data),
            LikelihoodFunction::Poisson => poisson_log_likelihood(params, data),
            LikelihoodFunction::Exponential => exponential_log_likelihood(params, data),
        }
    }

    fn gaussian_log_density(&self, x: f64, mean: f64, std: f64) -> f64 {
        gaussian_log_density_val(x, mean, std)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate Bayes factor for model comparison
    #[must_use]
    pub fn bayes_factor(
        &self,
        model1: &BayesianModel,
        model2: &BayesianModel,
        data: &[f64],
    ) -> f64 {
        let bic1 = self.bic(model1, data);
        let bic2 = self.bic(model2, data);

        if !bic1.is_finite() || !bic2.is_finite() {
            if bic1 == f64::INFINITY && bic2.is_finite() {
                return 0.0;
            }
            if bic1.is_finite() && bic2 == f64::INFINITY {
                return f64::MAX;
            }
            return 1.0;
        }

        let delta = (-0.5 * (bic1 - bic2)).clamp(-700.0, 700.0);
        delta.exp()
    }

    /// Bayesian Information Criterion
    fn bic(&self, model: &BayesianModel, data: &[f64]) -> f64 {
        if is_invalid_data(data) {
            return f64::INFINITY;
        }
        let k = model.parameters.len() as f64;
        let n = data.len() as f64;
        let sample_mean = data.iter().sum::<f64>() / n;
        let sample_var = data.iter().map(|&x| (x - sample_mean).powi(2)).sum::<f64>() / n;

        let mut mle_params: Vec<f64> = model
            .parameters
            .iter()
            .enumerate()
            .map(|(idx, p)| {
                resolve_param_mle(
                    model.likelihood,
                    idx,
                    None,
                    sample_mean,
                    sample_var,
                    p.prior_mean,
                )
            })
            .collect();

        if mle_params.is_empty() {
            mle_params = model.parameters.iter().map(|p| p.prior_mean).collect();
        }

        let log_lik = self.log_likelihood(&mle_params, data, model);

        -2.0 * log_lik + k * n.ln()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Model evidence (marginal likelihood) - Laplace approximation
    #[must_use]
    pub fn model_evidence(&self, model: &BayesianModel, data: &[f64]) -> f64 {
        let mode_params: Vec<f64> = model
            .parameters
            .iter()
            .enumerate()
            .map(|(idx, p)| {
                let raw = p.posterior_mean.unwrap_or(p.prior_mean);
                match (model.likelihood, idx) {
                    (LikelihoodFunction::Binomial, 0) => raw.clamp(1e-5, 1.0 - 1e-5),
                    (LikelihoodFunction::Poisson, 0) => raw.max(1e-5),
                    (LikelihoodFunction::Exponential, 0) => raw.max(1e-5),
                    (LikelihoodFunction::Gaussian, 1) => raw.max(1e-5),
                    _ => raw,
                }
            })
            .collect();
        // Tierney & Kadane (1986): Laplace approximation expands unnormalized posterior ln p(D, theta) = ln p(D | theta) + ln p(theta)
        let log_post_at_mode = self.log_posterior(&mode_params, data, model);

        let k = model.parameters.len() as f64;
        let log_det_post_var = model
            .parameters
            .iter()
            .map(|p| {
                let std = p.posterior_std.unwrap_or(p.prior_std).max(1e-4);
                (std * std).ln()
            })
            .sum::<f64>();

        log_post_at_mode + 0.5 * k * (2.0 * std::f64::consts::PI).ln() + 0.5 * log_det_post_var
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Posterior predictive check
    #[must_use]
    pub fn posterior_predictive_check(
        &self,
        model: &BayesianModel,
        observed_data: &[f64],
        n_replicates: usize,
    ) -> PpcResult {
        let clean_observed: Vec<f64> = observed_data
            .iter()
            .copied()
            .filter(|v| v.is_finite())
            .collect();

        if clean_observed.len() < 2 || n_replicates == 0 {
            let stat = self.test_statistic(observed_data);
            return PpcResult {
                observed_statistic: stat,
                replicated_mean: stat,
                p_value: 0.5,
                model_fit: "insufficient_data".to_string(),
            };
        }

        let mut test_statistics: Vec<f64> = Vec::with_capacity(n_replicates);
        use rand::SeedableRng;
        let mut rng = match self.mcmc_config.seed {
            Some(s) => rand::rngs::StdRng::seed_from_u64(s),
            None => rand::rngs::StdRng::from_entropy(),
        };

        for _ in 0..n_replicates {
            // Draw parameter vector theta^(m) ~ p(theta | y) integrating over parameter uncertainty (Gelman et al. 1996)
            let rep_params: Vec<f64> = model
                .parameters
                .iter()
                .enumerate()
                .map(|(idx, p)| {
                    let mean = p.posterior_mean.unwrap_or(p.prior_mean);
                    let std = p.posterior_std.unwrap_or(0.0).max(0.0);
                    let sampled = if std > 1e-6 {
                        use rand::Rng;
                        let u1: f64 = rng.gen_range(1e-7..1.0);
                        let u2: f64 = rng.gen_range(0.0..std::f64::consts::TAU);
                        let z = (-2.0 * u1.ln()).sqrt() * u2.cos();
                        mean + std * z
                    } else {
                        mean
                    };
                    match (model.likelihood, idx) {
                        (LikelihoodFunction::Binomial, 0) => sampled.clamp(1e-5, 1.0 - 1e-5),
                        (LikelihoodFunction::Poisson, 0) => sampled.max(1e-5),
                        (LikelihoodFunction::Exponential, 0) => sampled.max(1e-5),
                        (LikelihoodFunction::Gaussian, 1) => sampled.max(1e-5),
                        _ => sampled,
                    }
                })
                .collect();

            let replicated = self.simulate_data(&rep_params, clean_observed.len(), model, &mut rng);
            test_statistics.push(self.test_statistic(&replicated));
        }

        let observed_stat = self.test_statistic(&clean_observed);

        // Posterior predictive p-value: Pr(T(y_rep) >= T(y_obs) | y) (Gelman, Meng & Stern 1996)
        let count_extreme = test_statistics
            .iter()
            .filter(|&&s| s >= observed_stat)
            .count();

        let n_rep = n_replicates.max(1);
        let p_value = count_extreme as f64 / n_rep as f64;

        PpcResult {
            observed_statistic: observed_stat,
            replicated_mean: test_statistics.iter().sum::<f64>() / n_rep as f64,
            p_value,
            model_fit: if p_value > 0.05 && p_value < 0.95 {
                "good"
            } else {
                "poor"
            }
            .to_string(),
        }
    }

    fn simulate_data<R: rand::Rng>(
        &self,
        params: &[f64],
        n: usize,
        model: &BayesianModel,
        rng: &mut R,
    ) -> Vec<f64> {
        match model.likelihood {
            LikelihoodFunction::Gaussian => {
                let mean = params.first().copied().unwrap_or(0.0);
                let std = params.get(1).copied().unwrap_or(1.0).max(1e-4);

                // TAG: surface=simulation owner=research-team rule=SIM-001
                (0..n)
                    .map(|_| self.gaussian_sample(mean, std, rng))
                    .collect()
            }
            LikelihoodFunction::Binomial => {
                let p = params.first().copied().unwrap_or(0.5).clamp(0.0, 1.0);
                (0..n)
                    .map(|_| if rng.gen::<f64>() < p { 1.0 } else { 0.0 })
                    .collect()
            }
            LikelihoodFunction::Poisson => {
                let lambda = params.first().copied().unwrap_or(1.0).max(1e-4);
                if lambda > 60.0 {
                    // Gaussian approximation with Box-Muller and continuity correction for large lambda avoiding exp underflow
                    let std = lambda.sqrt();
                    (0..n)
                        .map(|_| {
                            let u1: f64 = rng.gen_range(1e-15..1.0);
                            let u2: f64 = rng.gen_range(0.0..std::f64::consts::TAU);
                            let z = (-2.0 * u1.ln()).sqrt() * u2.cos();
                            (lambda + std * z + 0.5).floor().max(0.0)
                        })
                        .collect()
                } else {
                    let l = (-lambda).exp();
                    (0..n)
                        .map(|_| {
                            let mut k = 0.0;
                            let mut p = 1.0;
                            loop {
                                k += 1.0;
                                p *= rng.gen::<f64>().clamp(1e-15, 1.0);
                                if p <= l {
                                    break;
                                }
                            }
                            (k - 1.0_f64).max(0.0)
                        })
                        .collect()
                }
            }
            LikelihoodFunction::Exponential => {
                let rate = params.first().copied().unwrap_or(1.0).max(1e-4);
                (0..n)
                    .map(|_| {
                        let u = rng.gen::<f64>().clamp(1e-15, 1.0 - 1e-15);
                        -u.ln() / rate
                    })
                    .collect()
            }
        }
    }

    fn gaussian_sample<R: rand::Rng>(&self, mean: f64, std: f64, rng: &mut R) -> f64 {
        let u1 = rng.gen::<f64>().clamp(1e-15, 1.0 - 1e-15);
        let u2 = rng.gen::<f64>().clamp(1e-15, 1.0 - 1e-15);
        let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        mean + std * z0
    }

    fn test_statistic(&self, data: &[f64]) -> f64 {
        let clean: Vec<f64> = data.iter().copied().filter(|v| v.is_finite()).collect();
        if clean.len() < 2 {
            return 0.0;
        }
        let mean = clean.iter().sum::<f64>() / clean.len() as f64;
        clean.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (clean.len() - 1) as f64
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Credible interval
    #[must_use]
    pub fn credible_interval(samples: &[f64], level: f64) -> (f64, f64) {
        if samples.is_empty() {
            return (0.0, 0.0);
        }
        let clean: Vec<f64> = samples.iter().copied().filter(|v| v.is_finite()).collect();
        if clean.is_empty() {
            return (0.0, 0.0);
        }
        let eff_level = level.clamp(0.01, 0.9999);
        let alpha = 1.0 - eff_level;
        let n = clean.len();
        let lower_idx = ((n as f64 * alpha / 2.0).floor() as usize).min(n - 1);
        let upper_idx = ((n as f64 * (1.0 - alpha / 2.0)).ceil() as usize).min(n - 1);

        let mut sorted = clean;
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        (sorted[lower_idx], sorted[upper_idx.max(lower_idx)])
    }

    /// Model comparison summary
    #[must_use]
    pub fn compare_models(&self, models: &[BayesianModel], data: &[f64]) -> ModelComparison {
        let mut results: Vec<ModelFit> = models
            .iter()
            .map(|m| ModelFit {
                model_name: m.name.clone(),
                bic: self.bic(m, data),
                evidence: self.model_evidence(m, data),
                n_parameters: m.parameters.len(),
            })
            .collect();

        results.sort_by(|a, b| match (a.bic.is_nan(), b.bic.is_nan()) {
            (true, true) => std::cmp::Ordering::Equal,
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            (false, false) => a
                .bic
                .partial_cmp(&b.bic)
                .unwrap_or(std::cmp::Ordering::Equal),
        });

        let best_model = results
            .first()
            .map(|r| r.model_name.clone())
            .unwrap_or_default();

        ModelComparison {
            fits: results,
            best_model,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for BayesianValidator {
    fn default() -> Self {
        Self::new(McmcConfig::default())
    }
}

/// Posterior approximation result
#[derive(Debug, Clone)]
pub struct PosteriorApproximation {
    /// samples.
    pub samples: Vec<Vec<f64>>,
    /// acceptance rate.
    pub acceptance_rate: f64,
    /// effective sample size.
    pub effective_sample_size: f64,
}

/// Posterior predictive check result
#[derive(Debug, Clone)]
pub struct PpcResult {
    /// observed statistic.
    pub observed_statistic: f64,
    /// replicated mean.
    pub replicated_mean: f64,
    /// p value.
    pub p_value: f64,
    /// model fit.
    pub model_fit: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Model fit metrics
#[derive(Debug, Clone)]
pub struct ModelFit {
    /// model name.
    pub model_name: String,
    /// bic.
    pub bic: f64,
    /// evidence.
    pub evidence: f64,
    /// n parameters.
    pub n_parameters: usize,
}

/// Model comparison
#[derive(Debug, Clone)]
pub struct ModelComparison {
    /// fits.
    pub fits: Vec<ModelFit>,
    /// best model.
    pub best_model: String,
}

#[cfg(test)]
#[path = "bayesian_tests.rs"]
mod tests;
