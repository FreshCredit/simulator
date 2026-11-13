// TAG: surface=simulation owner=research-team rule=SIM-001
//! Distribution math for meta-analysis (Normal, Student's t, Chi-square, Log-gamma)

/// Rational approximation for standard normal quantile
#[must_use]
pub fn normal_quantile(p: f64) -> f64 {
    if !p.is_finite() {
        return 0.0;
    }
    let p_clamped = p.clamp(1e-10, 1.0 - 1e-10);
    let q = if p_clamped < 0.5 {
        p_clamped
    } else {
        1.0 - p_clamped
    };
    let t = (-2.0 * q.ln()).sqrt();
    let c0 = 2.515517;
    let c1 = 0.802853;
    let c2 = 0.010328;
    let d1 = 1.432788;
    let d2 = 0.189269;
    let d3 = 0.001308;
    let z = t - ((c2 * t + c1) * t + c0) / (((d3 * t + d2) * t + d1) * t + 1.0);
    if p_clamped < 0.5 {
        -z
    } else {
        z
    }
}

/// Standard normal CDF
#[must_use]
pub fn normal_cdf(z: f64) -> f64 {
    if !z.is_finite() {
        return if z.is_sign_negative() { 0.0 } else { 1.0 };
    }
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if z < 0.0 { -1.0 } else { 1.0 };
    let x = z.abs() / std::f64::consts::SQRT_2;

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    0.5 * (1.0 + sign * y)
}

/// Critical z-value using rational approximation of inverse normal CDF
#[must_use]
pub fn z_critical(confidence: f64) -> f64 {
    if !confidence.is_finite() {
        return 1.95996;
    }
    let alpha = (1.0 - confidence).clamp(1e-10, 0.999);
    normal_quantile(1.0 - alpha / 2.0)
}

fn chi_square_cdf_even(x: f64, nu: usize, df: f64) -> f64 {
    let m = nu / 2;
    let z = x / 2.0;
    if z > 500.0 {
        let wh_z = ((x / df).cbrt() - (1.0 - 2.0 / (9.0 * df))) / (2.0 / (9.0 * df)).sqrt();
        return normal_cdf(wh_z).clamp(0.0, 1.0);
    }
    let mut sum = 1.0;
    let mut term = 1.0;
    for j in 1..m {
        term *= z / j as f64;
        sum += term;
    }
    (1.0 - (-z).exp() * sum).clamp(0.0, 1.0)
}

fn chi_square_cdf_gamma(x: f64, df: f64) -> f64 {
    let a = df / 2.0;
    let z = x / 2.0;
    if z < a + 1.0 {
        let ln_gamma_a = log_gamma(a);
        let mut term = 1.0 / a;
        let mut sum = term;
        for n in 1..100 {
            term *= z / (a + n as f64);
            sum += term;
            if term.abs() < sum.abs() * 1e-15 {
                break;
            }
        }
        let factor = a * z.ln() - z - ln_gamma_a;
        (factor.exp() * sum).clamp(0.0, 1.0)
    } else {
        let wh_z = ((x / df).cbrt() - (1.0 - 2.0 / (9.0 * df))) / (2.0 / (9.0 * df)).sqrt();
        normal_cdf(wh_z).clamp(0.0, 1.0)
    }
}

/// Chi-square cumulative distribution function
#[must_use]
pub fn chi_square_cdf(x: f64, df: f64) -> f64 {
    if !x.is_finite() || !df.is_finite() || x <= 0.0 || df <= 0.0 {
        return 0.0;
    }
    let nu = (df.round() as usize).max(1);

    if nu == 1 {
        return (2.0 * normal_cdf(x.sqrt()) - 1.0).clamp(0.0, 1.0);
    }
    if nu == 2 {
        return (1.0 - (-x / 2.0).exp()).clamp(0.0, 1.0);
    }
    if nu % 2 == 0 {
        return chi_square_cdf_even(x, nu, df);
    }
    chi_square_cdf_gamma(x, df)
}

fn student_t_cdf_odd(theta: f64, sin_th: f64, cos_th: f64, nu: usize) -> f64 {
    let mut s = 0.0;
    let mut term = 1.0;
    let cos2 = cos_th * cos_th;
    let m = (nu - 1) / 2;
    for j in 1..=m {
        let coef = (2.0 * j as f64 - 2.0) / (2.0 * j as f64 - 1.0);
        term = if j == 1 { 1.0 } else { term * coef * cos2 };
        s += term;
    }
    (0.5 + (theta + sin_th * cos_th * s) / std::f64::consts::PI).clamp(0.0, 1.0)
}

fn student_t_cdf_even(sin_th: f64, cos_th: f64, nu: usize) -> f64 {
    let mut s = 1.0;
    let mut term = 1.0;
    let cos2 = cos_th * cos_th;
    let m = nu / 2;
    for j in 1..m {
        let coef = (2.0 * j as f64 - 1.0) / (2.0 * j as f64);
        term *= coef * cos2;
        s += term;
    }
    (0.5 + 0.5 * sin_th * s).clamp(0.0, 1.0)
}

/// Student's t cumulative distribution function
#[must_use]
pub fn student_t_cdf(t: f64, df: f64) -> f64 {
    if !t.is_finite() {
        return if t.is_sign_negative() { 0.0 } else { 1.0 };
    }
    if !df.is_finite() || df <= 0.0 {
        return 0.5;
    }
    if df >= 30.0 {
        return normal_cdf(t);
    }
    let nu = (df.round() as usize).max(1);
    let theta = (t / (nu as f64).sqrt()).atan();
    let cos_th = theta.cos();
    let sin_th = theta.sin();

    if nu % 2 == 1 {
        student_t_cdf_odd(theta, sin_th, cos_th, nu)
    } else {
        student_t_cdf_even(sin_th, cos_th, nu)
    }
}

/// Student's t quantile function
#[must_use]
pub fn student_t_quantile(p: f64, df: f64) -> f64 {
    if !p.is_finite() || !df.is_finite() || df <= 0.0 {
        return 0.0;
    }
    let p_clamped = p.clamp(1e-9, 1.0 - 1e-9);
    if df >= 30.0 {
        return normal_quantile(p_clamped);
    }
    let nu = (df.round() as usize).max(1);

    if nu == 1 {
        return (std::f64::consts::PI * (p_clamped - 0.5)).tan();
    }
    if nu == 2 {
        let num = 2.0 * p_clamped - 1.0;
        let den = (2.0 * p_clamped * (1.0 - p_clamped)).sqrt().max(1e-12);
        return num / den;
    }

    let nu_f = nu as f64;
    let ln_c_nu = log_gamma((nu_f + 1.0) / 2.0)
        - log_gamma(nu_f / 2.0)
        - 0.5 * (nu_f * std::f64::consts::PI).ln();
    let c_nu = ln_c_nu.exp();

    let mut x = normal_quantile(p_clamped);
    for _ in 0..8 {
        let cdf = student_t_cdf(x, nu_f);
        let pdf = c_nu * (1.0 + x * x / nu_f).powf(-(nu_f + 1.0) / 2.0);
        if pdf < 1e-15 {
            break;
        }
        let diff = (cdf - p_clamped) / pdf;
        x -= diff;
        if diff.abs() < 1e-10 {
            break;
        }
    }
    x
}

/// Lanczos log-gamma approximation
#[must_use]
pub fn log_gamma(x: f64) -> f64 {
    if !x.is_finite() || x <= 0.0 {
        return f64::INFINITY;
    }
    let p = [
        0.99999999999980993,
        676.5203681218851,
        -1259.1392167224028,
        771.32342877765313,
        -176.61502916214059,
        12.507343278686905,
        -0.138571095836526,
        9.9843695780195716e-6,
        1.5056327351493116e-7,
    ];
    if x < 0.5 {
        let sin_pix = (std::f64::consts::PI * x).sin();
        if sin_pix <= 0.0 {
            return f64::INFINITY;
        }
        std::f64::consts::PI.ln() - sin_pix.ln() - log_gamma(1.0 - x)
    } else {
        let z = x - 1.0;
        let mut a = p[0];
        for (i, &val) in p.iter().enumerate().skip(1) {
            a += val / (z + i as f64);
        }
        let t = z + 7.5;
        0.5 * (2.0 * std::f64::consts::PI).ln() + (z + 0.5) * t.ln() - t + a.ln()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dist_normal_cdf_and_quantile() {
        assert!((normal_cdf(0.0) - 0.5).abs() < 1e-6);
        assert!((normal_quantile(0.5) - 0.0).abs() < 1e-6);
        assert!((z_critical(0.95) - 1.96).abs() < 0.01);
    }
}
