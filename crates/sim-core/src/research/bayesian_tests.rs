use super::*;

fn create_test_model(name: &str) -> BayesianModel {
    BayesianModel {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        name: name.to_string(),
        parameters: vec![
            Parameter {
                name: "mu".to_string(),
                prior_mean: 0.0,
                prior_std: 1.0,
                posterior_mean: None,
                posterior_std: None,
            },
            Parameter {
                name: "sigma".to_string(),
                prior_mean: 1.0,
                prior_std: 0.5,
                posterior_mean: None,
                posterior_std: None,
            },
        ],
        likelihood: LikelihoodFunction::Gaussian,
        prior: PriorDistribution::Gaussian,
    }
}

#[test]
fn test_log_likelihood_rejects_out_of_support_parameters() {
    let validator = BayesianValidator::default();
    let mut model = create_test_model("gaussian");
    let data = vec![1.0, 2.0, 3.0];

    // Non-positive std in Gaussian
    assert_eq!(
        validator.log_likelihood(&[1.0, 0.0], &data, &model),
        f64::NEG_INFINITY
    );
    assert_eq!(
        validator.log_likelihood(&[1.0, -1.0], &data, &model),
        f64::NEG_INFINITY
    );

    // Binomial out of (0, 1)
    model.likelihood = LikelihoodFunction::Binomial;
    assert_eq!(
        validator.log_likelihood(&[0.0], &data, &model),
        f64::NEG_INFINITY
    );
    assert_eq!(
        validator.log_likelihood(&[1.0], &data, &model),
        f64::NEG_INFINITY
    );
    assert_eq!(
        validator.log_likelihood(&[-0.5], &data, &model),
        f64::NEG_INFINITY
    );
    assert_eq!(
        validator.log_likelihood(&[1.5], &data, &model),
        f64::NEG_INFINITY
    );

    // Poisson / Exponential non-positive rate
    model.likelihood = LikelihoodFunction::Poisson;
    assert_eq!(
        validator.log_likelihood(&[0.0], &data, &model),
        f64::NEG_INFINITY
    );
    assert_eq!(
        validator.log_likelihood(&[-2.0], &data, &model),
        f64::NEG_INFINITY
    );

    model.likelihood = LikelihoodFunction::Exponential;
    assert_eq!(
        validator.log_likelihood(&[0.0], &data, &model),
        f64::NEG_INFINITY
    );
    assert_eq!(
        validator.log_likelihood(&[-1.0], &data, &model),
        f64::NEG_INFINITY
    );
}

#[test]
fn test_approximate_posterior() {
    let validator = BayesianValidator::default();
    let mut model = create_test_model("test");

    use rand::Rng;
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let data: Vec<f64> = (0..100).map(|_| 5.0 + (rng.gen::<f64>() - 0.5)).collect();

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let posterior = validator.approximate_posterior(&mut model, &data);

    assert!(posterior.acceptance_rate > 0.0);
    assert_eq!(posterior.samples.len(), 1000);
    assert!(model.parameters[0].posterior_mean.unwrap().is_finite());
    assert!(model.parameters[1].posterior_mean.unwrap().is_finite());
}

#[test]
fn test_bic() {
    let validator = BayesianValidator::default();
    let model = create_test_model("test");
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];

    let bic = validator.bic(&model, &data);
    assert!(bic.is_finite());
}

#[test]
fn test_model_comparison() {
    let validator = BayesianValidator::default();
    let models = vec![create_test_model("model1"), create_test_model("model2")];
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];

    let comparison = validator.compare_models(&models, &data);
    assert_eq!(comparison.fits.len(), 2);
}

#[test]
fn test_credible_interval() {
    let samples: Vec<f64> = (0..1000).map(|i| i as f64 / 100.0).collect();
    let (lower, upper) = BayesianValidator::credible_interval(&samples, 0.95);

    assert!(lower < upper);
    assert!(lower >= 0.0);
}

#[test]
fn test_bayes_factor_boundary_and_overflow_safety() {
    let validator = BayesianValidator::default();
    let m1 = create_test_model("m1");
    let m2 = create_test_model("m2");
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];

    let bf = validator.bayes_factor(&m1, &m2, &data);
    assert!(!bf.is_nan());
    assert!(bf > 0.0);

    // Identical models yield BF ~ 1.0
    assert!((bf - 1.0).abs() < 1e-6);
}

#[test]
fn test_ln_gamma_euler_reflection_exactness() {
    // Γ(0.5) = sqrt(π) => ln(Γ(0.5)) = 0.5 * ln(π)
    let half_exact = 0.5 * std::f64::consts::PI.ln();
    assert!((ln_gamma(0.5) - half_exact).abs() < 1e-12);

    // Euler reflection identity: ln(Γ(z)) + ln(Γ(1-z)) = ln(π / sin(πz))
    let z1 = 0.25;
    let expected_sum1 = (std::f64::consts::PI / (std::f64::consts::PI * z1).sin()).ln();
    assert!(((ln_gamma(z1) + ln_gamma(1.0 - z1)) - expected_sum1).abs() < 1e-12);

    let z2 = 0.1;
    let expected_sum2 = (std::f64::consts::PI / (std::f64::consts::PI * z2).sin()).ln();
    assert!(((ln_gamma(z2) + ln_gamma(1.0 - z2)) - expected_sum2).abs() < 1e-12);
}

#[test]
fn test_approximate_posterior_clamps_boundary_parameters() {
    let validator = BayesianValidator::new(McmcConfig {
        n_samples: 100,
        burn_in: 20,
        thinning: 1,
        step_size: 0.05,
        seed: Some(42),
    });

    let mut beta_model = BayesianModel {
        name: "beta_test".to_string(),
        parameters: vec![Parameter {
            name: "p".to_string(),
            prior_mean: 0.0,
            prior_std: 0.5,
            posterior_mean: None,
            posterior_std: None,
        }],
        likelihood: LikelihoodFunction::Binomial,
        prior: PriorDistribution::Beta,
    };
    let data = vec![1.0, 1.0, 0.0, 1.0];
    let post = validator.approximate_posterior(&mut beta_model, &data);
    assert!(post.samples.len() == 100);
    let p_mean = beta_model.parameters[0].posterior_mean.unwrap();
    assert!(p_mean.is_finite() && p_mean > 0.0 && p_mean < 1.0);
}

#[test]
fn test_frozen_chain_ess_is_one() {
    let validator = BayesianValidator::new(McmcConfig {
        n_samples: 50,
        burn_in: 0,
        thinning: 1,
        step_size: 0.0, // step size 0 => 100% frozen/stuck chain
        seed: Some(42),
    });

    let mut model = create_test_model("frozen");
    let data = vec![1.0, 2.0, 3.0];
    let post = validator.approximate_posterior(&mut model, &data);
    assert_eq!(post.effective_sample_size, 1.0);
}

#[test]
fn test_posterior_predictive_check_deterministic_with_seed() {
    let validator = BayesianValidator::new(McmcConfig {
        seed: Some(1337),
        ..Default::default()
    });
    let model = create_test_model("gaussian");
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];

    let ppc1 = validator.posterior_predictive_check(&model, &data, 50);
    let ppc2 = validator.posterior_predictive_check(&model, &data, 50);

    assert_eq!(ppc1.p_value, ppc2.p_value);
    assert!((ppc1.replicated_mean - ppc2.replicated_mean).abs() < 1e-12);
}

#[test]
fn test_log_likelihood_rejects_nan_and_negative_data() {
    let validator = BayesianValidator::default();
    let mut model = create_test_model("gaussian");
    let nan_data = vec![1.0, f64::NAN, 3.0];
    let empty_data: Vec<f64> = vec![];

    assert_eq!(
        validator.log_likelihood(&[1.0, 1.0], &nan_data, &model),
        f64::NEG_INFINITY
    );
    assert_eq!(
        validator.log_likelihood(&[1.0, 1.0], &empty_data, &model),
        f64::NEG_INFINITY
    );

    model.likelihood = LikelihoodFunction::Binomial;
    assert_eq!(
        validator.log_likelihood(&[0.5], &nan_data, &model),
        f64::NEG_INFINITY
    );

    model.likelihood = LikelihoodFunction::Poisson;
    let neg_data = vec![1.0, -2.0, 3.0];
    assert_eq!(
        validator.log_likelihood(&[2.0], &nan_data, &model),
        f64::NEG_INFINITY
    );
    assert_eq!(
        validator.log_likelihood(&[2.0], &neg_data, &model),
        f64::NEG_INFINITY
    );

    model.likelihood = LikelihoodFunction::Exponential;
    assert_eq!(
        validator.log_likelihood(&[1.5], &nan_data, &model),
        f64::NEG_INFINITY
    );
    assert_eq!(
        validator.log_likelihood(&[1.5], &neg_data, &model),
        f64::NEG_INFINITY
    );
}

#[test]
fn test_moment_matching_prior_mean_preservation() {
    for &target_mean in &[0.1, 0.3, 0.5, 0.7, 0.9] {
        for &std in &[0.001, 0.05, 0.2, 0.9] {
            let (alpha, beta) = beta_moments_to_params(target_mean, std);
            let recovered_mean = alpha / (alpha + beta);
            assert!(
                (recovered_mean - target_mean).abs() < 1e-6,
                "Beta mean mismatch: target {target_mean}, got {recovered_mean} with std {std}"
            );
        }
    }

    for &target_mean in &[0.5, 2.0, 5.0, 25.0] {
        for &std in &[0.0001, 0.01, 0.5, 2.0, 10.0] {
            let (shape, rate) = gamma_moments_to_params(target_mean, std);
            let recovered_mean = shape / rate;
            assert!(
                (recovered_mean - target_mean).abs() < 1e-6,
                "Gamma mean mismatch: target {target_mean}, got {recovered_mean} with std {std}"
            );
        }
    }
}
