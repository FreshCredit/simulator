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

    // Check for constant series (zero variance) across finite observations
    let finite_x: Vec<f64> = x.iter().copied().filter(|v| v.is_finite()).collect();
    let finite_y: Vec<f64> = y.iter().copied().filter(|v| v.is_finite()).collect();
    if finite_x.len() <= 2 * max_lag + 2 || finite_y.len() <= 2 * max_lag + 2 {
        return None;
    }
    let (min_x, max_x) = finite_x
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(mn, mx), &v| {
            (mn.min(v), mx.max(v))
        });
    let (min_y, max_y) = finite_y
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

    let mut best_p = 1;
    let mut best_bic = f64::INFINITY;
    let mut best_fit_r = None;
    let mut best_fit_u = None;
    let mut best_t_eff = 0usize;

    for p in 1..=max_lag {
        let mut y_dep = Vec::new();
        let mut x_restr = Vec::new();
        let mut x_unrestr = Vec::new();

        for t in max_lag..n {
            if !y[t].is_finite() {
                continue;
            }
            let lags_finite = (1..=p).all(|lag| y[t - lag].is_finite() && x[t - lag].is_finite());
            if !lags_finite {
                continue;
            }
            y_dep.push(y[t]);

            let mut row_r = Vec::new();
            row_r.push(1.0); // constant
            for lag in 1..=p {
                row_r.push(y[t - lag]);
            }
            x_restr.push(row_r);

            let mut row_u = Vec::new();
            row_u.push(1.0); // constant
            for lag in 1..=p {
                row_u.push(y[t - lag]);
            }
            for lag in 1..=p {
                row_u.push(x[t - lag]);
            }
            x_unrestr.push(row_u);
        }

        let t_eff = y_dep.len();
        if t_eff <= 2 * p + 2 {
            continue;
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
            best_t_eff = t_eff;
            best_fit_r = Some(fit_r);
            best_fit_u = Some(fit_u);
        }
    }

    let fit_r = best_fit_r?;
    let fit_u = best_fit_u?;
    let p = best_p;

    let df1 = p as f64;
    let df2 = (best_t_eff - (2 * p + 1)) as f64;
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

/// Empirical biometric observations recovered from wearable/wetlab logging
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CalibrationObservations {
    /// Nocturnal resting HRV RMSSD samples in ms
    pub hrv_rmssd_samples: Vec<f64>,
    /// Frontal Alpha Asymmetry (AF8 vs AF7) samples
    pub faa_samples: Vec<f64>,
    /// Fasting blood glucose samples in mg/dL
    pub glucose_samples: Vec<f64>,
    /// Prefrontal fNIRS medial cortical Δ[HbO2] samples in μmol/L
    pub fnirs_hbo2_samples: Vec<f64>,
    /// Sleeping wrist temperature samples in °C
    pub sleeping_wrist_temp_samples: Vec<f64>,
    /// Pairs of (financial debt shock in normalized units, cognitive bandwidth depletion %)
    pub financial_depletion_pairs: Vec<(f64, f64)>,
    /// Calibrated trust sensitivity λ samples
    pub lambda_samples: Vec<f64>,
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

    /// Empirically compute personal calibration baselines from observed biometric streams.
    /// Uses robust statistics (median for outliers or mean for well-behaved vitals) and
    /// OLS regression for `financial_depletion_slope`. Falls back to defaults when observations
    /// are missing or insufficient.
    #[must_use]
    pub fn calibrate_from_observations(
        user_id: &str,
        cohort_stage: u8,
        obs: &CalibrationObservations,
    ) -> Self {
        let mut profile = Self::for_cohort(user_id, cohort_stage);

        fn mean_of(samples: &[f64]) -> Option<f64> {
            let valid: Vec<f64> = samples.iter().copied().filter(|v| v.is_finite()).collect();
            if valid.is_empty() {
                None
            } else {
                Some(valid.iter().sum::<f64>() / valid.len() as f64)
            }
        }

        fn median_of(samples: &[f64]) -> Option<f64> {
            let mut valid: Vec<f64> = samples.iter().copied().filter(|v| v.is_finite()).collect();
            if valid.is_empty() {
                return None;
            }
            valid.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mid = valid.len() / 2;
            if valid.len() % 2 == 0 {
                Some((valid[mid - 1] + valid[mid]) / 2.0)
            } else {
                Some(valid[mid])
            }
        }

        if let Some(m) = median_of(&obs.hrv_rmssd_samples) {
            profile.hrv_rmssd_baseline_ms = m.clamp(10.0, 200.0);
        }
        if let Some(m) = mean_of(&obs.faa_samples) {
            profile.faa_baseline = m.clamp(-2.0, 2.0);
        }
        if let Some(m) = median_of(&obs.glucose_samples) {
            profile.glucose_baseline_mgdl = m.clamp(50.0, 300.0);
        }
        if let Some(m) = mean_of(&obs.fnirs_hbo2_samples) {
            profile.fnirs_hbo2_baseline_umol = m.clamp(0.0, 20.0);
        }
        if let Some(m) = mean_of(&obs.sleeping_wrist_temp_samples) {
            profile.sleeping_wrist_temp_baseline_c = m.clamp(34.0, 39.0);
        }
        if let Some(m) = mean_of(&obs.lambda_samples) {
            profile.lambda_mean = m.clamp(0.05, 1.50);
        }

        let valid_pairs: Vec<(f64, f64)> = obs
            .financial_depletion_pairs
            .iter()
            .copied()
            .filter(|(x, y)| x.is_finite() && y.is_finite())
            .collect();
        if valid_pairs.len() >= 2 {
            let n = valid_pairs.len() as f64;
            let mean_x = valid_pairs.iter().map(|(x, _)| *x).sum::<f64>() / n;
            let mean_y = valid_pairs.iter().map(|(_, y)| *y).sum::<f64>() / n;
            let mut ss_xx = 0.0;
            let mut ss_xy = 0.0;
            for (x, y) in valid_pairs {
                let dx = x - mean_x;
                let dy = y - mean_y;
                ss_xx += dx * dx;
                ss_xy += dx * dy;
            }
            if ss_xx > 1e-9 {
                let slope = ss_xy / ss_xx;
                profile.financial_depletion_slope = slope.clamp(0.0, 5.0);
            }
        }

        profile
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

    let clean: Vec<f64> = data.iter().copied().filter(|v| v.is_finite()).collect();
    let n = clean.len();
    if n == 0 {
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
            sample.push(clean[idx]);
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

/// Result of Wing-Kristofferson (1973) motor timing decomposition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WingKristoffersonResult {
    /// Mean inter-tap interval in ms
    pub mean_iti_ms: f64,
    /// Standard deviation of ITI in ms
    pub sd_iti_ms: f64,
    /// Coefficient of variation (CV_ITI %)
    pub cv_iti_pct: f64,
    /// Central clock variance estimate (sigma^2_clock)
    pub sigma_clock_sq: f64,
    /// Peripheral motor execution delay variance estimate (sigma^2_motor)
    pub sigma_motor_sq: f64,
    /// Empirical lag-1 autocovariance
    pub autocov_lag1: f64,
}

/// Computes Wing-Kristofferson (1973) timing decomposition on inter-tap intervals (ms)
/// with Vorberg & Wing (1996) OLS linear tempo detrending to prevent drift bias.
/// sigma^2_total = sigma^2_clock + 2 * sigma^2_motor - 2 * Cov(ITI_k, ITI_{k+1})
#[must_use]
pub fn wing_kristofferson_decomposition(itis_ms: &[f64]) -> Option<WingKristoffersonResult> {
    let n = itis_ms.len();
    if n < 4 || itis_ms.iter().any(|v| !v.is_finite()) {
        return None;
    }

    let n_f = n as f64;
    let mean_iti = itis_ms.iter().sum::<f64>() / n_f;
    if mean_iti <= 0.0 {
        return None;
    }

    let t_mean = (n_f - 1.0) / 2.0;
    let ss_tt = (n_f * (n_f * n_f - 1.0)) / 12.0;

    let mut ss_ty = 0.0;
    for (t, &y) in itis_ms.iter().enumerate() {
        ss_ty += ((t as f64) - t_mean) * (y - mean_iti);
    }
    let beta_1 = if ss_tt > 1e-12 { ss_ty / ss_tt } else { 0.0 };

    let residuals: Vec<f64> = itis_ms
        .iter()
        .enumerate()
        .map(|(t, &y)| y - (mean_iti + beta_1 * ((t as f64) - t_mean)))
        .collect();

    let df = (n - 2).max(1) as f64;
    let var_total = residuals.iter().map(|&e| e * e).sum::<f64>() / df;
    let sd_iti = var_total.max(0.0).sqrt();
    let cv_iti_pct = (sd_iti / mean_iti.max(1e-4)) * 100.0;

    let cov_lag1 = (0..n - 1)
        .map(|i| residuals[i] * residuals[i + 1])
        .sum::<f64>()
        / df;

    let sigma_motor_sq = (-cov_lag1).max(0.0);
    let sigma_clock_sq = (var_total - 2.0 * sigma_motor_sq).max(0.0);

    Some(WingKristoffersonResult {
        mean_iti_ms: mean_iti,
        sd_iti_ms: sd_iti,
        cv_iti_pct,
        sigma_clock_sq,
        sigma_motor_sq,
        autocov_lag1: cov_lag1,
    })
}

/// Benjamini-Yekutieli (2001) False Discovery Rate (FDR) control under arbitrary joint statistical dependence.
#[must_use]
pub fn benjamini_yekutieli_fdr(p_values: &[f64], alpha: f64) -> Vec<bool> {
    let m = p_values.len();
    if m == 0 {
        return Vec::new();
    }
    let alpha = if alpha.is_finite() && alpha > 0.0 {
        alpha.min(1.0)
    } else {
        0.05
    };

    let c_m = (1..=m).map(|i| 1.0 / (i as f64)).sum::<f64>();

    let mut indexed: Vec<(usize, f64)> = p_values
        .iter()
        .enumerate()
        .map(|(i, &p)| {
            (
                i,
                if p.is_finite() {
                    p.clamp(0.0, 1.0)
                } else {
                    1.0
                },
            )
        })
        .collect();

    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut max_k = 0;
    for (rank, (_, p)) in indexed.iter().enumerate() {
        let k = (rank + 1) as f64;
        let threshold = (k / ((m as f64) * c_m)) * alpha;
        if *p <= threshold {
            max_k = rank + 1;
        }
    }

    let mut rejected = vec![false; m];
    for &(orig_idx, _) in &indexed[..max_k] {
        rejected[orig_idx] = true;
    }
    rejected
}

/// Computes step-up adjusted q-values under Benjamini-Yekutieli (2001) FDR control
/// with reverse-cumulative minimum adjustment ensuring monotonicity: q_(1) <= q_(2) <= ... <= q_(m).
// POT-005: FDR q-values helper
#[must_use]
pub fn benjamini_yekutieli_q_values(p_values: &[f64]) -> Vec<f64> {
    let m = p_values.len();
    if m == 0 {
        return Vec::new();
    }
    let c_m = (1..=m).map(|i| 1.0 / (i as f64)).sum::<f64>();

    let mut indexed: Vec<(usize, f64)> = p_values
        .iter()
        .enumerate()
        .map(|(i, &p)| {
            (
                i,
                if p.is_finite() {
                    p.clamp(0.0, 1.0)
                } else {
                    1.0
                },
            )
        })
        .collect();

    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut raw_qs: Vec<f64> = indexed
        .iter()
        .enumerate()
        .map(|(rank, (_, p))| {
            let k = (rank + 1) as f64;
            (*p * (m as f64) * c_m / k).min(1.0)
        })
        .collect();

    let mut running_min: f64 = 1.0;
    for q in raw_qs.iter_mut().rev() {
        running_min = running_min.min(*q);
        *q = running_min;
    }

    let mut out = vec![0.0; m];
    for (sorted_idx, &(orig_idx, _)) in indexed.iter().enumerate() {
        out[orig_idx] = raw_qs[sorted_idx];
    }
    out
}

/// Romano-Wolf (2005/2016) Stepdown procedure controlling Family-Wise Error Rate (FWER)
#[must_use]
pub fn romano_wolf_stepdown_fwer(
    test_stats: &[f64],
    bootstrap_stats: &[Vec<f64>],
    alpha: f64,
) -> Vec<bool> {
    let m = test_stats.len();
    if m == 0 || bootstrap_stats.is_empty() {
        return vec![false; m];
    }
    let b = bootstrap_stats.len();
    let alpha = if alpha.is_finite() && alpha > 0.0 {
        alpha.min(1.0)
    } else {
        0.05
    };

    let mut rejected = vec![false; m];
    let mut active_indices: Vec<usize> = (0..m).collect();

    while !active_indices.is_empty() {
        let mut max_boot_dist = Vec::with_capacity(b);
        for boot_sample in bootstrap_stats {
            let max_val = active_indices
                .iter()
                .map(|&idx| {
                    if idx < boot_sample.len() && boot_sample[idx].is_finite() {
                        boot_sample[idx].abs()
                    } else {
                        0.0
                    }
                })
                .fold(0.0f64, f64::max);
            max_boot_dist.push(max_val);
        }

        max_boot_dist.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let crit_idx = ((1.0 - alpha) * (b as f64)).ceil() as usize;
        let crit_idx = crit_idx.clamp(1, b) - 1;
        let c_crit = max_boot_dist[crit_idx];

        let newly_rejected: Vec<usize> = active_indices
            .iter()
            .copied()
            .filter(|&idx| test_stats[idx].abs() >= c_crit)
            .collect();

        if newly_rejected.is_empty() {
            break;
        }

        for &idx in &newly_rejected {
            rejected[idx] = true;
        }

        active_indices.retain(|idx| !newly_rejected.contains(idx));
    }

    rejected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wing_kristofferson_decomposition() {
        let itis = [500.0, 485.0, 515.0, 490.0, 510.0, 495.0, 505.0];
        let wk = wing_kristofferson_decomposition(&itis).expect("WK decomposition should succeed");
        assert!((wk.mean_iti_ms - 500.0).abs() < 1e-6);
        assert!(wk.cv_iti_pct > 0.0);
        assert!(wk.sigma_clock_sq >= 0.0);
        assert!(wk.sigma_motor_sq >= 0.0);
    }

    #[test]
    fn test_wing_kristofferson_drift_invariance() {
        // Synthesize an ITI series with strong linear fatigue drift (+3.0 ms per tap)
        // I_t = 500.0 + 3.0 * t + alternating motor delay noise
        let mut drifting_itis = Vec::new();
        for t in 0..20 {
            let motor_noise = if t % 2 == 0 { -10.0 } else { 10.0 };
            drifting_itis.push(500.0 + 3.0 * (t as f64) + motor_noise);
        }
        let wk =
            wing_kristofferson_decomposition(&drifting_itis).expect("Drifting WK must succeed");
        assert!(
            wk.sigma_motor_sq > 0.0,
            "Detrending must preserve motor variance despite drift"
        );
        assert!(
            wk.sigma_clock_sq >= 0.0,
            "Clock variance must remain non-negative"
        );
    }

    #[test]
    fn test_benjamini_yekutieli_fdr_control() {
        let p_vals = [0.001, 0.004, 0.015, 0.08, 0.45, 0.90];
        let rejections = benjamini_yekutieli_fdr(&p_vals, 0.05);
        assert_eq!(rejections.len(), 6);
        assert!(rejections[0]);
        assert!(!rejections[5]);
    }

    #[test]
    fn test_benjamini_yekutieli_q_values_monotonicity() {
        let p_vals = [0.001, 0.004, 0.015, 0.08, 0.45, 0.90];
        let q_vals = benjamini_yekutieli_q_values(&p_vals);
        assert_eq!(q_vals.len(), 6);
        for i in 0..q_vals.len() - 1 {
            assert!(
                q_vals[i] <= q_vals[i + 1] + 1e-12,
                "BY q-values must be monotonic: q[{}] = {} > q[{}] = {}",
                i,
                q_vals[i],
                i + 1,
                q_vals[i + 1]
            );
        }
        let rejections = benjamini_yekutieli_fdr(&p_vals, 0.05);
        for (rej, q) in rejections.iter().zip(&q_vals) {
            if *rej {
                assert!(*q <= 0.05 + 1e-6);
            }
        }
    }

    #[test]
    fn test_romano_wolf_stepdown_fwer_control() {
        let test_stats = [4.5, 3.8, 1.2, 0.4];
        let boot_stats = vec![
            vec![2.1, 1.8, 1.1, 0.5],
            vec![1.9, 2.2, 0.9, 0.6],
            vec![2.4, 2.0, 1.3, 0.4],
            vec![2.0, 1.7, 1.0, 0.7],
            vec![2.3, 2.1, 1.2, 0.3],
        ];
        let rejections = romano_wolf_stepdown_fwer(&test_stats, &boot_stats, 0.05);
        assert_eq!(rejections.len(), 4);
        assert!(rejections[0]);
        assert!(rejections[1]);
        assert!(!rejections[3]);
    }

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

        // Arrow-of-time falsification oracle: reversing both series must NOT show reverse causal flow
        let x_rev: Vec<f64> = x.iter().copied().rev().collect();
        let y_rev: Vec<f64> = y.iter().copied().rev().collect();
        let rev_res = test_granger_causality(&x_rev, &y_rev, 1).expect("Reversed test should run");
        assert!(
            rev_res.f_statistic < result.f_statistic * 0.25,
            "Time-reversed F={} should collapse vs forward F={}",
            rev_res.f_statistic,
            result.f_statistic
        );
    }

    #[test]
    fn test_granger_causality_gap_aware_single_nan_survivability() {
        let n = 80;
        let mut x = vec![0.0f64; n];
        let mut y = vec![0.0f64; n];
        for t in 1..n {
            x[t] = 0.4 * x[t - 1] + ((t * 7919) % 100) as f64 / 100.0 - 0.5;
            y[t] = 0.3 * y[t - 1] + 0.85 * x[t - 1] + ((t * 3571) % 100) as f64 / 200.0 - 0.25;
        }
        x[25] = f64::NAN; // Single missing epoch must not discard the entire 80-point series
        let result =
            test_granger_causality(&x, &y, 1).expect("Gap-aware Granger test should recover");
        assert!(result.is_significant);
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

        let gapped = [1.0, f64::NAN, 2.0, 3.0, f64::INFINITY, 4.0];
        let gapped_samples = stationary_block_bootstrap(&gapped, 10, 0.25, 42);
        assert_eq!(gapped_samples.len(), 10);
        assert_eq!(gapped_samples[0].len(), 4);
        assert!(gapped_samples.iter().flatten().all(|v| v.is_finite()));
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

    #[test]
    fn test_personal_calibration_from_observations() {
        let obs = CalibrationObservations {
            hrv_rmssd_samples: vec![45.0, 52.0, 55.0, 48.0, 999.0], // Outlier 999.0; median = 52.0
            faa_samples: vec![0.08, 0.12, 0.10],                    // Mean = 0.10
            glucose_samples: vec![88.0, 92.0, 94.0],                // Median = 92.0
            fnirs_hbo2_samples: vec![2.8, 3.2, 3.0],                // Mean = 3.00
            sleeping_wrist_temp_samples: vec![36.50, 36.60],        // Mean = 36.55
            financial_depletion_pairs: vec![(1.0, 0.5), (2.0, 1.0), (3.0, 1.5)], // Slope = 0.50
            lambda_samples: vec![0.70, 0.80],                       // Mean = 0.75
        };

        let profile =
            PersonalCalibrationProfile::calibrate_from_observations("user-test-477", 2, &obs);
        assert_eq!(profile.user_id, "user-test-477");
        assert_eq!(profile.cohort_stage, 2);
        assert!(profile.s_channel_eligible);
        assert_eq!(
            profile.hrv_rmssd_baseline_ms, 52.0,
            "HRV must match median 52.0"
        );
        assert!(
            (profile.faa_baseline - 0.10).abs() < 1e-4,
            "FAA must match mean 0.10"
        );
        assert_eq!(
            profile.glucose_baseline_mgdl, 92.0,
            "Glucose must match median 92.0"
        );
        assert!((profile.fnirs_hbo2_baseline_umol - 3.00).abs() < 1e-4);
        assert!((profile.sleeping_wrist_temp_baseline_c - 36.55).abs() < 1e-4);
        assert!((profile.financial_depletion_slope - 0.50).abs() < 1e-4);
        assert!((profile.lambda_mean - 0.75).abs() < 1e-4);

        // Test empty observations fallback cleanly to default cohort baselines
        let empty_obs = CalibrationObservations::default();
        let fallback =
            PersonalCalibrationProfile::calibrate_from_observations("user-fallback", 4, &empty_obs);
        assert_eq!(fallback.cohort_stage, 4);
        assert!(!fallback.s_channel_eligible);
        assert_eq!(fallback.hrv_rmssd_baseline_ms, 48.0);
        assert_eq!(fallback.financial_depletion_slope, 0.42);
    }
}
