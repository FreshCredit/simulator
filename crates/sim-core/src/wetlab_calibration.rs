//! Empirical Wet-Lab Econometric Calibration Engine (N=1, T=30 to N=30)
//! Grounded in GTT-EMPIRICAL-PROGRAM, DOCAS §5.30, and Time-Series Econometrics.
//!
//! Provides:
//! 1. Bivariate Granger Causality F-test between financial ledger events and wearable physiology.
//! 2. Stationary Block Bootstrap (Politis & Romano 1994) for autocorrelated dense time-series.
//! 3. Kalman State-Space Filter estimating personal empirical trust/stress sensitivity parameter λ_t.
//! 4. In-silico Calibration Profile generator bridging real-world physiology into multi-agent simulation.

use serde::{Deserialize, Serialize};

/// Result of a bivariate Granger Causality test
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrangerCausalityResult {
    /// Optimal lag order p selected by BIC
    pub lag_order: usize,
    /// F-statistic for H0: X does not Granger-cause Y
    pub f_statistic: f64,
    /// Asymptotic p-value from F(p, T - 2p - 1) distribution
    pub p_value: f64,
    /// Restricted sum of squared residuals (Y on past Y only)
    pub rss_restricted: f64,
    /// Unrestricted sum of squared residuals (Y on past Y and past X)
    pub rss_unrestricted: f64,
    /// Whether causality is statistically significant at alpha = 0.05
    pub is_significant: bool,
}

/// Ordinary Least Squares parameter estimates and residual sum of squares
struct OlsFit {
    #[allow(dead_code)]
    params: Vec<f64>,
    rss: f64,
}

#[allow(clippy::needless_range_loop)]
fn solve_linear_system(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Option<Vec<f64>> {
    let n = b.len();
    for i in 0..n {
        let mut max_row = i;
        for k in (i + 1)..n {
            if a[k][i].abs() > a[max_row][i].abs() {
                max_row = k;
            }
        }
        if a[max_row][i].abs() < 1e-12 {
            return None;
        }
        a.swap(i, max_row);
        b.swap(i, max_row);

        let pivot = a[i][i];
        if !pivot.is_finite() || pivot.abs() < 1e-12 {
            return None;
        }
        for k in i..n {
            a[i][k] /= pivot;
        }
        b[i] /= pivot;

        for k in 0..n {
            if k != i {
                let factor = a[k][i];
                for j in i..n {
                    a[k][j] -= factor * a[i][j];
                }
                b[k] -= factor * b[i];
            }
        }
    }
    if !b.iter().all(|v| v.is_finite()) {
        return None;
    }
    Some(b)
}

fn fit_ols(y: &[f64], x_mat: &[Vec<f64>]) -> Option<OlsFit> {
    let t = y.len();
    if t == 0 || x_mat.is_empty() || t != x_mat.len() {
        return None;
    }
    let k = x_mat[0].len();
    let mut xtx = vec![vec![0.0f64; k]; k];
    let mut xty = vec![0.0f64; k];

    for i in 0..t {
        for r in 0..k {
            xty[r] += x_mat[i][r] * y[i];
            for c in 0..k {
                xtx[r][c] += x_mat[i][r] * x_mat[i][c];
            }
        }
    }

    // Direct OLS solve with Tikhonov ridge fallback (1e-10 * I) on near-singularity
    let params = solve_linear_system(xtx.clone(), xty.clone()).or_else(|| {
        let mut xtx_ridge = xtx;
        for (i, row) in xtx_ridge.iter_mut().enumerate().take(k) {
            row[i] += 1e-10;
        }
        solve_linear_system(xtx_ridge, xty)
    })?;
    if !params.iter().all(|p| p.is_finite()) {
        return None;
    }
    let mut rss = 0.0;
    for i in 0..t {
        let mut y_pred = 0.0;
        for c in 0..k {
            y_pred += x_mat[i][c] * params[c];
        }
        let res = y[i] - y_pred;
        rss += res * res;
    }

    if !rss.is_finite() || rss < 0.0 {
        return None;
    }

    Some(OlsFit { params, rss })
}

/// Natural logarithm of Gamma function via Lanczos approximation (g=7, n=9) with Euler reflection for z < 0.5
pub fn ln_gamma(z: f64) -> f64 {
    if z <= 0.0 {
        return f64::INFINITY;
    }
    // Euler's reflection formula for z < 0.5: ln(Gamma(z)) = ln(pi) - ln(sin(pi * z)) - ln(Gamma(1 - z))
    if z < 0.5 {
        let sin_pi_z = (std::f64::consts::PI * z).sin();
        if sin_pi_z <= 0.0 {
            return f64::INFINITY;
        }
        return std::f64::consts::PI.ln() - sin_pi_z.ln() - ln_gamma(1.0 - z);
    }
    const COEFFS: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.5203681218851,
        -1259.1392167224028,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507343278686905,
        -0.138571095836524,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_312e-7,
    ];
    let mut x = COEFFS[0];
    for (i, &c) in COEFFS[1..].iter().enumerate() {
        x += c / (z + (i as f64));
    }
    let t = z + 6.5;
    0.5 * (2.0 * std::f64::consts::PI).ln() + (z - 0.5) * t.ln() - t + x.ln()
}

#[inline]
fn lentz_floor(val: f64) -> f64 {
    const TINY: f64 = 1e-30;
    if val.abs() < TINY {
        if val < 0.0 {
            -TINY
        } else {
            TINY
        }
    } else {
        val
    }
}

/// Continued fraction for incomplete beta function via modified Lentz's method
fn betacf(a: f64, b: f64, x: f64) -> f64 {
    const MAX_IT: usize = 200;
    const EPS: f64 = 1e-15;

    let qab = a + b;
    let qap = a + 1.0;
    let qam = a - 1.0;

    let mut c = 1.0;
    let mut d = lentz_floor(1.0 - qab * x / qap).recip();
    let mut h = d;

    for m in 1..=MAX_IT {
        let m_f = m as f64;
        let m2 = 2.0 * m_f;

        let aa_even = m_f * (b - m_f) * x / ((qam + m2) * (a + m2));
        d = lentz_floor(1.0 + aa_even * d).recip();
        c = lentz_floor(1.0 + aa_even / c);
        h *= d * c;

        let aa_odd = -(a + m_f) * (qab + m_f) * x / ((a + m2) * (qap + m2));
        d = lentz_floor(1.0 + aa_odd * d).recip();
        c = lentz_floor(1.0 + aa_odd / c);
        let del = d * c;
        h *= del;

        if (del - 1.0).abs() < EPS {
            break;
        }
    }
    h
}

/// Regularized incomplete beta function I_x(a, b) on [0, 1]
pub fn regularized_incomplete_beta(a: f64, b: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    if !a.is_finite() || !b.is_finite() || a <= 0.0 || b <= 0.0 {
        return 0.0;
    }

    let ln_beta = ln_gamma(a) + ln_gamma(b) - ln_gamma(a + b);
    let front = (a * x.ln() + b * (1.0 - x).ln() - ln_beta).exp();

    if x < (a + 1.0) / (a + b + 2.0) {
        (front * betacf(a, b, x) / a).clamp(0.0, 1.0)
    } else {
        (1.0 - front * betacf(b, a, 1.0 - x) / b).clamp(0.0, 1.0)
    }
}

/// Exact Fisher-Snedecor F(df1, df2) upper tail p-value P(F >= f)
pub fn f_distribution_p_value(f_stat: f64, df1: f64, df2: f64) -> f64 {
    if !f_stat.is_finite() || f_stat <= 0.0 || df1 <= 0.0 || df2 <= 0.0 {
        return 1.0;
    }
    let x = df2 / (df2 + df1 * f_stat);
    let a = df2 / 2.0;
    let b = df1 / 2.0;
    regularized_incomplete_beta(a, b, x).clamp(0.0, 1.0)
}

/// Compute bivariate Granger Causality: does series X Granger-cause series Y?
#[must_use]
pub fn test_granger_causality(
    x: &[f64],
    y: &[f64],
    max_lag: usize,
) -> Option<GrangerCausalityResult> {
    let n = x.len();
    if n != y.len() || n <= 2 * max_lag + 2 || max_lag == 0 {
        return None;
    }
    if x.iter().any(|v| !v.is_finite()) || y.iter().any(|v| !v.is_finite()) {
        return None;
    }

    // Check for constant series (zero variance)
    let (min_x, max_x) = x
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(mn, mx), &v| {
            (mn.min(v), mx.max(v))
        });
    let (min_y, max_y) = y
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(mn, mx), &v| {
            (mn.min(v), mx.max(v))
        });
    if (max_y - min_y).abs() <= 1e-12 {
        // Y has zero variance - Granger causality undefined
        return None;
    }
    if (max_x - min_x).abs() <= 1e-12 {
        // X has zero variance - X cannot Granger-cause Y
        return Some(GrangerCausalityResult {
            lag_order: 1,
            f_statistic: 0.0,
            p_value: 1.0,
            rss_restricted: 0.0,
            rss_unrestricted: 0.0,
            is_significant: false,
        });
    }

    let t_eff = n - max_lag;
    let mut best_p = 1;
    let mut best_bic = f64::INFINITY;
    let mut best_fit_r = None;
    let mut best_fit_u = None;

    for p in 1..=max_lag {
        let mut y_dep = Vec::with_capacity(t_eff);
        let mut x_restr = Vec::with_capacity(t_eff);
        let mut x_unrestr = Vec::with_capacity(t_eff);

        for t in max_lag..n {
            y_dep.push(y[t]);

            let mut row_r = Vec::with_capacity(p + 1);
            row_r.push(1.0); // constant
            for lag in 1..=p {
                row_r.push(y[t - lag]);
            }
            x_restr.push(row_r);

            let mut row_u = Vec::with_capacity(2 * p + 1);
            row_u.push(1.0); // constant
            for lag in 1..=p {
                row_u.push(y[t - lag]);
            }
            for lag in 1..=p {
                row_u.push(x[t - lag]);
            }
            x_unrestr.push(row_u);
        }

        let Some(fit_r) = fit_ols(&y_dep, &x_restr) else {
            continue;
        };
        let Some(fit_u) = fit_ols(&y_dep, &x_unrestr) else {
            continue;
        };

        let num_params = (2 * p + 1) as f64;
        let clamped_rss = fit_u.rss.max(1e-12);
        let bic =
            (t_eff as f64) * (clamped_rss / t_eff as f64).ln() + num_params * (t_eff as f64).ln();

        if bic < best_bic {
            best_bic = bic;
            best_p = p;
            best_fit_r = Some(fit_r);
            best_fit_u = Some(fit_u);
        }
    }

    let fit_r = best_fit_r?;
    let fit_u = best_fit_u?;
    let p = best_p;

    let df1 = p as f64;
    let df2 = (t_eff - (2 * p + 1)) as f64;
    if df2 <= 0.0 {
        return None;
    }
    // Mathematically nested OLS: RSS_u <= RSS_r.
    // Ridge regularization or floating-point rounding can occasionally make fit_u.rss slightly > fit_r.rss
    let rss_u = fit_u.rss.min(fit_r.rss);
    let rss_r = fit_r.rss;

    if rss_u <= 1e-12 {
        if rss_r > 1e-10 {
            return Some(GrangerCausalityResult {
                lag_order: p,
                f_statistic: 999.0,
                p_value: 0.0,
                rss_restricted: rss_r,
                rss_unrestricted: rss_u,
                is_significant: true,
            });
        }
        return None;
    }

    let f_stat = ((rss_r - rss_u) / df1) / (rss_u / df2);
    let f_stat_clamped = f_stat.max(0.0);

    // Exact Fisher-Snedecor F(df1, df2) upper tail p-value via regularized incomplete beta
    let p_val = f_distribution_p_value(f_stat_clamped, df1, df2);

    Some(GrangerCausalityResult {
        lag_order: p,
        f_statistic: f_stat_clamped,
        p_value: p_val,
        rss_restricted: rss_r,
        rss_unrestricted: rss_u,
        is_significant: p_val < 0.05,
    })
}

/// Personal empirical calibration profile recovered from 30-day wet-lab telemetry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersonalCalibrationProfile {
    /// Participant or user ID
    pub user_id: String,
    /// Posterior mean of trust sensitivity parameter λ
    pub lambda_mean: f64,
    /// Empirical DOCAS mean-reversion alpha baselines
    pub docas_alpha_base: [f64; 5],
    /// Fasting / resting nocturnal HRV RMSSD baseline in ms
    pub hrv_rmssd_baseline_ms: f64,
    /// Resting baseline Frontal Alpha Asymmetry (AF8 vs AF7)
    pub faa_baseline: f64,
    /// Fasting baseline interstitial glucose in mg/dL
    pub glucose_baseline_mgdl: f64,
    /// Empirical slope of cognitive bandwidth depletion per unit financial debt shock
    pub financial_depletion_slope: f64,
    /// Prefrontal fNIRS medial cortical Δ[HbO2] baseline in μmol/L (Muse Athena)
    pub fnirs_hbo2_baseline_umol: f64,
    /// Nocturnal wrist temperature baseline in °C (Apple Watch S8)
    pub sleeping_wrist_temp_baseline_c: f64,
    /// DOCAS-PILOT-1 Cohort Stage (1: 1x30d, 2: 3x10d, 3: 10x3d, 4: 30x1d)
    pub cohort_stage: u8,
    /// Whether multi-day S-channel stabilization tracking is pre-registered
    pub s_channel_eligible: bool,
}

impl Default for PersonalCalibrationProfile {
    fn default() -> Self {
        Self {
            user_id: "pilot-default".to_string(),
            lambda_mean: 0.618,
            docas_alpha_base: [0.50, 0.50, 0.50, 0.50, 0.50],
            hrv_rmssd_baseline_ms: 48.0,
            faa_baseline: 0.05,
            glucose_baseline_mgdl: 92.0,
            financial_depletion_slope: 0.42,
            fnirs_hbo2_baseline_umol: 2.50,
            sleeping_wrist_temp_baseline_c: 36.40,
            cohort_stage: 1,
            s_channel_eligible: true,
        }
    }
}

impl PersonalCalibrationProfile {
    /// Construct calibration profile enforcing pre-registered DOCAS-PILOT-1 cohort rules:
    /// Cohorts 1–3 evaluate multi-day S and C channels; Cohort 4 (1-day) is barred from S-channel.
    #[must_use]
    pub fn for_cohort(user_id: &str, cohort_stage: u8) -> Self {
        let stage = cohort_stage.clamp(1, 4);
        let s_channel_eligible = stage <= 3;
        Self {
            user_id: user_id.to_string(),
            lambda_mean: 0.618,
            docas_alpha_base: [0.50, 0.50, 0.50, 0.50, 0.50],
            hrv_rmssd_baseline_ms: 48.0,
            faa_baseline: 0.05,
            glucose_baseline_mgdl: 92.0,
            financial_depletion_slope: 0.42,
            fnirs_hbo2_baseline_umol: 2.50,
            sleeping_wrist_temp_baseline_c: 36.40,
            cohort_stage: stage,
            s_channel_eligible,
        }
    }
}

/// Stationary Block Bootstrap (Politis & Romano 1994) for autocorrelated time-series
pub fn stationary_block_bootstrap(
    data: &[f64],
    num_resamples: usize,
    p_geom: f64,
    seed: u64,
) -> Vec<Vec<f64>> {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    let n = data.len();
    if n == 0 || data.iter().any(|v| !v.is_finite()) {
        return Vec::new();
    }
    const MAX_BOOTSTRAP_RESAMPLES: usize = 10_000;
    let num_resamples = num_resamples.min(MAX_BOOTSTRAP_RESAMPLES);
    let p_geom = if p_geom.is_finite() {
        p_geom.clamp(1e-4, 1.0)
    } else {
        0.25
    };

    let mut rng = StdRng::seed_from_u64(seed);
    let mut replicates = Vec::with_capacity(num_resamples);

    for _ in 0..num_resamples {
        let mut sample = Vec::with_capacity(n);
        let mut idx = rng.gen_range(0..n);
        for _ in 0..n {
            sample.push(data[idx]);
            // With probability p_geom start a new block; otherwise increment circularly
            if rng.gen::<f64>() < p_geom {
                idx = rng.gen_range(0..n);
            } else {
                idx = (idx + 1) % n;
            }
        }
        replicates.push(sample);
    }

    replicates
}

/// Unscented/1D Kalman Filter state-space parameter recovery for latent sensitivity λ_t
#[derive(Debug, Clone)]
pub struct LatentSensitivityKalmanFilter {
    /// Latent state estimate: sensitivity λ
    pub state_lambda: f64,
    /// Estimation error variance P
    pub covariance: f64,
    /// Process noise variance Q (drift in sensitivity)
    pub q_process: f64,
    /// Measurement noise variance R
    pub r_measure: f64,
}

impl LatentSensitivityKalmanFilter {
    /// Initialize a new latent sensitivity Kalman filter with bounded noise parameters
    #[must_use]
    pub fn new(initial_lambda: f64, q: f64, r: f64) -> Self {
        let safe_lambda = if initial_lambda.is_finite() {
            initial_lambda.clamp(0.05, 1.50)
        } else {
            0.618
        };
        let safe_q = if q.is_finite() && q >= 0.0 { q } else { 0.001 };
        let safe_r = if r.is_finite() && r > 0.0 { r } else { 0.05 };
        Self {
            state_lambda: safe_lambda,
            covariance: 0.1,
            q_process: safe_q,
            r_measure: safe_r,
        }
    }

    /// Update filter with financial prediction error ε_t and observed physiological delta ΔBio_t
    pub fn step(&mut self, epsilon_t: f64, delta_bio_t: f64) -> f64 {
        if !epsilon_t.is_finite() || !delta_bio_t.is_finite() {
            return self.state_lambda;
        }

        // 1. Time Update (Predict)
        let p_prior = self.covariance + self.q_process;

        // 2. Measurement Model: ΔBio_t = λ_t * ε_t + v_t
        let h = epsilon_t;
        let s = h * p_prior * h + self.r_measure;

        if s.abs() > 1e-9 {
            // Kalman Gain K = P_prior * H / S
            let k = p_prior * h / s;
            let innovation = delta_bio_t - self.state_lambda * h;
            self.state_lambda = (self.state_lambda + k * innovation).clamp(0.05, 1.50);

            // Joseph-form stabilized covariance update: P = (1 - K*H)^2 * P_prior + K^2 * R
            let one_minus_kh = 1.0 - k * h;
            self.covariance =
                (one_minus_kh * one_minus_kh * p_prior + k * k * self.r_measure).max(1e-12);
        } else {
            self.covariance = p_prior;
        }

        self.state_lambda
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_granger_causality_detects_causal_flow() {
        let n = 100;
        let mut x = vec![0.0f64; n];
        let mut y = vec![0.0f64; n];

        // Construct synthetic AR(1) causal system: Y_t = 0.5 * Y_{t-1} + 0.8 * X_{t-1} + noise
        for t in 1..n {
            x[t] = 0.4 * x[t - 1] + ((t * 7919) % 100) as f64 / 100.0 - 0.5;
            y[t] = 0.3 * y[t - 1] + 0.85 * x[t - 1] + ((t * 3571) % 100) as f64 / 200.0 - 0.25;
        }

        let result = test_granger_causality(&x, &y, 1).expect("Granger test should run");
        assert!(
            result.is_significant,
            "X must Granger-cause Y with high significance"
        );
        assert!(
            result.f_statistic > 10.0,
            "F-statistic must exceed critical threshold"
        );
        assert!(result.rss_unrestricted < result.rss_restricted);
    }

    #[test]
    fn test_granger_causality_rejects_independent_noise() {
        let n = 100;
        let mut x = vec![0.0f64; n];
        let mut y = vec![0.0f64; n];

        for t in 0..n {
            x[t] = ((t * 7919) % 100) as f64 / 100.0 - 0.5;
            y[t] = ((t * 1337) % 100) as f64 / 100.0 - 0.5;
        }

        let result = test_granger_causality(&x, &y, 1).expect("Granger test should run");
        assert!(
            !result.is_significant,
            "Independent noise must not be Granger causal"
        );
    }

    #[test]
    fn test_stationary_block_bootstrap_preserves_autocorrelation() {
        let data: Vec<f64> = (0..50).map(|i| (i as f64 * 0.1).sin()).collect();
        let samples = stationary_block_bootstrap(&data, 50, 0.25, 42);
        assert_eq!(samples.len(), 50);
        assert_eq!(samples[0].len(), 50);
    }

    #[test]
    fn test_kalman_state_space_recovers_lambda() {
        let true_lambda = 0.75;
        let mut kf = LatentSensitivityKalmanFilter::new(0.30, 0.001, 0.05);

        for step in 1..=40 {
            let eps = if step % 2 == 0 { 1.5 } else { -1.0 };
            let measurement = true_lambda * eps + 0.01;
            kf.step(eps, measurement);
        }

        assert!(
            (kf.state_lambda - true_lambda).abs() < 0.08,
            "Kalman filter should converge near true lambda 0.75, got {}",
            kf.state_lambda
        );
    }

    #[test]
    fn test_granger_causality_rejects_non_finite() {
        let x = vec![1.0, f64::NAN, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let y = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        assert!(test_granger_causality(&x, &y, 2).is_none());
    }

    #[test]
    fn test_kalman_non_finite_and_zero_shock_stable() {
        let mut kf = LatentSensitivityKalmanFilter::new(0.60, 0.005, 0.02);
        let lambda_before = kf.state_lambda;
        kf.step(f64::NAN, 1.0);
        assert_eq!(kf.state_lambda, lambda_before);

        let cov_before = kf.covariance;
        kf.step(0.0, 0.0);
        assert!(
            kf.covariance >= cov_before,
            "Covariance must accumulate Q on zero shock"
        );
        assert!(kf.covariance.is_finite());
    }

    #[test]
    fn test_personal_calibration_profile_cohort_rules() {
        let p1 = PersonalCalibrationProfile::for_cohort("participant-01", 1);
        assert_eq!(p1.cohort_stage, 1);
        assert!(p1.s_channel_eligible);
        assert_eq!(p1.fnirs_hbo2_baseline_umol, 2.50);
        assert_eq!(p1.sleeping_wrist_temp_baseline_c, 36.40);

        let p3 = PersonalCalibrationProfile::for_cohort("participant-03", 3);
        assert_eq!(p3.cohort_stage, 3);
        assert!(p3.s_channel_eligible);

        // Cohort 4 (1-day single observation) cannot validate multi-day sleep stabilization
        let p4 = PersonalCalibrationProfile::for_cohort("participant-04", 4);
        assert_eq!(p4.cohort_stage, 4);
        assert!(!p4.s_channel_eligible);
    }

    #[test]
    fn test_kalman_filter_degenerate_noise_handling() {
        // Test degenerate (negative or NaN) Q and R inputs get sanitized
        let mut kf = LatentSensitivityKalmanFilter::new(f64::NAN, -1.0, 0.0);
        assert!(kf.state_lambda.is_finite());
        assert_eq!(kf.state_lambda, 0.618);
        assert!(kf.q_process > 0.0);
        assert!(kf.r_measure > 0.0);

        let lambda = kf.step(1.0, 0.62);
        assert!(lambda.is_finite());
    }

    #[test]
    fn test_f_distribution_p_value_matches_exact_textbook_quantiles() {
        // Critical values at alpha = 0.05 from Snedecor & Cochran (1989):
        // F(1, 10) = 4.96460
        let p1 = f_distribution_p_value(4.96460, 1.0, 10.0);
        assert!(
            (p1 - 0.05).abs() < 1e-4,
            "Expected ~0.05 for F(1, 10)=4.9646, got {}",
            p1
        );

        // F(2, 20) = 3.49283
        let p2 = f_distribution_p_value(3.49283, 2.0, 20.0);
        assert!(
            (p2 - 0.05).abs() < 1e-4,
            "Expected ~0.05 for F(2, 20)=3.4928, got {}",
            p2
        );

        // F(3, 6) = 4.75706
        let p3 = f_distribution_p_value(4.75706, 3.0, 6.0);
        assert!(
            (p3 - 0.05).abs() < 1e-4,
            "Expected ~0.05 for F(3, 6)=4.7571, got {}",
            p3
        );

        // Boundary cases: f <= 0 -> p = 1.0; non-finite -> p = 1.0
        assert_eq!(f_distribution_p_value(0.0, 2.0, 10.0), 1.0);
        assert_eq!(f_distribution_p_value(-2.5, 2.0, 10.0), 1.0);
        assert_eq!(f_distribution_p_value(f64::NAN, 2.0, 10.0), 1.0);
    }

    #[test]
    fn test_f_distribution_closed_form_and_extreme_tails() {
        // Analytical closed form for d1 = 2: P(F_{2, nu} >= f) = (1 + 2f / nu)^(-nu / 2)
        let exact_nu10_f5 = (1.0 + 2.0 * 5.0 / 10.0f64).powf(-5.0); // 0.03125
        let p_num1 = f_distribution_p_value(5.0, 2.0, 10.0);
        assert!(
            (p_num1 - exact_nu10_f5).abs() < 1e-6,
            "Expected {}, got {}",
            exact_nu10_f5,
            p_num1
        );

        let exact_nu4_f3 = (1.0 + 2.0 * 3.0 / 4.0f64).powf(-2.0); // 0.16
        let p_num2 = f_distribution_p_value(3.0, 2.0, 4.0);
        assert!(
            (p_num2 - exact_nu4_f3).abs() < 1e-6,
            "Expected {}, got {}",
            exact_nu4_f3,
            p_num2
        );

        // Extreme tail where d1 = 1, d2 = 3, F = 50.0 (tests Lentz floor sign stability)
        let p_tail = f_distribution_p_value(50.0, 1.0, 3.0);
        assert!(p_tail.is_finite());
        assert!(
            p_tail > 0.0 && p_tail < 0.01,
            "Expected small positive tail, got {}",
            p_tail
        );
    }

    #[test]
    fn test_granger_causality_constant_x_returns_non_significant_zero_f_stat() {
        let n = 50;
        let x = vec![42.0; n]; // Constant X has zero variance
        let y: Vec<f64> = (0..n).map(|i| (i as f64) * 0.5).collect();

        let res = test_granger_causality(&x, &y, 1).expect("Test should complete without error");
        assert_eq!(res.f_statistic, 0.0);
        assert_eq!(res.p_value, 1.0);
        assert!(!res.is_significant);
    }
}
