//! Statistical Functions for POM Validation
//!
//! Implements statistical computations required for Pattern-Oriented Modeling:
//! - Effect sizes (Cohen's d)
//! - Confidence intervals
//! - Entropy measures
//! - Correlation coefficients
//! - Normal distribution functions
//! - Data quality validation
//!
//! Per simulation-plan.md §5.3.3: Layer 2 statistics for POM validation

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Error function (erf) approximation
/// Uses Abramowitz-Stegun 7.1.26 rational polynomial approximation
/// Per simulation-plan.md §5.3.3 line 647-650
#[must_use]
pub fn erf(x: f32) -> f32 {
    // Constants from Abramowitz-Stegun
    const A1: f32 = 0.254_829_6;
    const A2: f32 = -0.284_496_74;
    const A3: f32 = 1.421_413_8;
    const A4: f32 = -1.453_152;
    const A5: f32 = 1.061_405_5;
    const P: f32 = 0.327_591_1;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    // t = 1 / (1 + px)
    let t = 1.0 / (1.0 + P * x);

    // Polynomial approximation using Horner's method
    let poly = A1 * t + A2 * t * t + A3 * t * t * t + A4 * t * t * t * t + A5 * t * t * t * t * t;

    sign * (1.0 - poly * (-x * x).exp())
}

/// Normal cumulative distribution function (CDF)
/// Φ(x) = 0.5 × (1 + erf(x/√2))
/// Per simulation-plan.md §5.3.3
#[must_use]
pub fn normal_cdf(x: f32) -> f32 {
    0.5 * (1.0 + erf(x / std::f32::consts::SQRT_2))
}

// =============================================================================
// DATA QUALITY VALIDATION
// =============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Validate sample size for statistical power
///
/// # Arguments
/// * `n` - Sample size
/// * `effect_size` - Expected effect size (Cohen's d or correlation r)
/// * `alpha` - Significance level (default 0.05)
/// * `power` - Desired power (default 0.80)
///
/// # Returns
/// true if sample size is adequate
#[must_use]
pub fn validate_sample_size(n: usize, effect_size: f32, alpha: f32, power: f32) -> bool {
    if n < 2 {
        return false;
    }

    // Simplified power calculation for two-sample t-test
    // n = 2 * ((z_(1-alpha/2) + z_power) / d)^2
    let z_alpha = match alpha {
        0.10 => 1.645,
        0.05 => 1.96,
        0.01 => 2.576,
        _ => 1.96,
    };

    let z_power = match power {
        0.80 => 0.84,
        0.90 => 1.28,
        0.95 => 1.645,
        _ => 0.84,
    };

    let required_n = (2.0 * ((z_alpha + z_power) / effect_size.max(0.01)).powi(2)) as usize;
    n >= required_n
}

/// Get minimum sample size recommendation
#[must_use]
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
pub fn required_sample_size(effect_size: f32, alpha: f32, power: f32) -> usize {
    let z_alpha = match alpha {
        0.10 => 1.645,
        0.05 => 1.96,
        0.01 => 2.576,
        _ => 1.96,
    };
    let z_power = match power {
        0.80 => 0.84,
        0.90 => 1.28,
        0.95 => 1.645,
        _ => 0.84,
    };
    (2.0 * ((z_alpha + z_power) / effect_size.max(0.01)).powi(2)) as usize
}

/// Check for NaN or Infinity in data
///
/// # Returns
/// Ok(()) if all values are valid
/// Err(`Vec<usize>`) with indices of invalid values
/// # Errors
///
/// Returns an error if the operation fails.
pub fn validate_numeric(values: &[f32]) -> Result<(), Vec<usize>> {
    let invalid_indices: Vec<usize> = values
        .iter()
        .enumerate()
        .filter(|(_, &v)| v.is_nan() || v.is_infinite())
        .map(|(i, _)| i)
        .collect();

    if invalid_indices.is_empty() {
        Ok(())
    } else {
        Err(invalid_indices)
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Clean data by removing NaN/Infinity values
#[must_use]
pub fn clean_numeric(values: &[f32]) -> Vec<f32> {
    values
        .iter()
        .copied()
        .filter(|&v| !v.is_nan() && !v.is_infinite())
        .collect()
}

/// Detect outliers using IQR method
///
/// Values outside [Q1 - 1.5×IQR, Q3 + 1.5×IQR] are outliers
///
/// # Returns
/// Indices of outlier values
#[must_use]
pub fn detect_outliers(values: &[f32]) -> Vec<usize> {
    if values.len() < 4 {
        return vec![];
    }

    let mut sorted: Vec<f32> = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let q1_idx = values.len() / 4;
    let q3_idx = 3 * values.len() / 4;
    let q1 = sorted[q1_idx];
    let q3 = sorted[q3_idx];
    let iqr = q3 - q1;

    let lower_bound = q1 - 1.5 * iqr;
    let upper_bound = q3 + 1.5 * iqr;

    values
        .iter()
        .enumerate()
        .filter(|(_, &v)| v < lower_bound || v > upper_bound)
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        .map(|(i, _)| i)
        .collect()
}

/// Detect outliers using Z-score method
///
/// Values with |Z| > threshold are outliers
#[must_use]
pub fn detect_outliers_zscore(values: &[f32], threshold: f32) -> Vec<usize> {
    if values.len() < 2 {
        return vec![];
    }

    let n = values.len() as f32;
    let mean = values.iter().sum::<f32>() / n;
    let variance = values.iter().map(|&v| (v - mean).powi(2)).sum::<f32>() / n;
    let std = variance.sqrt();

    let tol = 1e-5_f32 * mean.abs().max(1.0);
    if std <= tol {
        return vec![];
    }

    values
        .iter()
        .enumerate()
        .filter(|(_, &v)| ((v - mean) / std).abs() >= threshold)
        .map(|(i, _)| i)
        .collect()
}

/// Calculate Fisher-Pearson sample skewness G1 of a distribution (NIST e-Handbook §1.3.5.11)
fn calculate_skewness(values: &[f32]) -> f32 {
    if values.len() < 3 {
        return 0.0;
    }

    let n = values.len() as f32;
    let mean = values.iter().sum::<f32>() / n;
    let variance = values.iter().map(|&v| (v - mean).powi(2)).sum::<f32>() / (n - 1.0);
    let std = variance.sqrt();

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let tol = 1e-5_f32 * mean.abs().max(1.0);
    if std <= tol {
        return 0.0;
    }

    let raw_skewness = values
        .iter()
        .map(|&v| ((v - mean) / std).powi(3))
        .sum::<f32>()
        / n;

    // NIST sample size adjustment: G1 = sqrt(N*(N-1))/(N-2) * g1
    let adj = (n * (n - 1.0)).sqrt() / (n - 2.0);
    raw_skewness * adj
}

/// Calculate unbiased sample excess kurtosis G2 of a distribution (NIST e-Handbook §1.3.5.11)
fn calculate_kurtosis(values: &[f32]) -> f32 {
    if values.len() < 4 {
        return 0.0;
    }

    let n = values.len() as f32;
    let mean = values.iter().sum::<f32>() / n;
    let variance = values.iter().map(|&v| (v - mean).powi(2)).sum::<f32>() / (n - 1.0);
    let std = variance.sqrt();

    let tol = 1e-5_f32 * mean.abs().max(1.0);
    if std <= tol {
        return 0.0;
    }

    let m4 = values
        .iter()
        .map(|&v| ((v - mean) / std).powi(4))
        .sum::<f32>()
        / n;

    let g2 = m4 - 3.0;
    // NIST finite sample adjustment for excess kurtosis:
    // G2 = ((n - 1) / ((n - 2) * (n - 3))) * ((n + 1) * g2 + 6)
    let adj = (n - 1.0) / ((n - 2.0) * (n - 3.0));
    adj * ((n + 1.0) * g2 + 6.0)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Bonferroni correction for multiple comparisons
///
/// # Arguments
/// * `p_values` - Slice of p-values
/// * `alpha` - Desired family-wise error rate (default 0.05)
///
/// # Returns
/// Vector of bools indicating which tests are significant after correction
#[must_use]
pub fn bonferroni_correction(p_values: &[f32], alpha: f32) -> Vec<bool> {
    if p_values.is_empty() || !alpha.is_finite() || alpha <= 0.0 {
        return vec![false; p_values.len()];
    }

    let n = p_values.len();
    let corrected_alpha = alpha / n as f32;

    p_values
        .iter()
        .map(|&p| p.is_finite() && (0.0..=1.0).contains(&p) && p <= corrected_alpha)
        .collect()
}

/// Benjamini-Hochberg (1995) FDR step-up procedure
///
/// Controls false discovery rate at level q.
/// Finds the largest k* such that p(k*) <= (k*/n) * q, then rejects all i <= k*.
#[must_use]
pub fn fdr_correction(p_values: &[f32], q: f32) -> Vec<bool> {
    if p_values.is_empty() || !q.is_finite() || q <= 0.0 {
        return vec![false; p_values.len()];
    }

    let mut indexed: Vec<(usize, f32)> = p_values.iter().copied().enumerate().collect();

    // Sort by p-value (NaN and negative values placed at end as non-significant)
    indexed.sort_by(
        |a, b| match (a.1.is_nan() || a.1 < 0.0, b.1.is_nan() || b.1 < 0.0) {
            (true, true) => std::cmp::Ordering::Equal,
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            (false, false) => a.1.total_cmp(&b.1),
        },
    );

    let n = p_values.len();
    let mut significant = vec![false; n];

    // Find largest k such that p(k) <= (k/n) * q (step-up)
    let mut max_k = None;
    for (i, (_, p)) in indexed.iter().enumerate() {
        if p.is_finite() && *p >= 0.0 && *p <= 1.0 {
            let k = i + 1;
            let threshold = (k as f32 / n as f32) * q;
            if *p <= threshold {
                max_k = Some(i);
            }
        }
    }

    // Reject all hypotheses up to max_k
    if let Some(k_star) = max_k {
        for (idx, (original_idx, _)) in indexed.iter().enumerate() {
            if idx <= k_star {
                significant[*original_idx] = true;
            }
        }
    }

    significant
}

/// Holm-Bonferroni step-down correction
/// More powerful than standard Bonferroni
#[must_use]
pub fn holm_bonferroni_correction(p_values: &[f32], alpha: f32) -> Vec<bool> {
    if p_values.is_empty() || !alpha.is_finite() || alpha <= 0.0 {
        return vec![false; p_values.len()];
    }

    let mut indexed: Vec<(usize, f32)> = p_values.iter().copied().enumerate().collect();

    indexed.sort_by(
        |a, b| match (a.1.is_nan() || a.1 < 0.0, b.1.is_nan() || b.1 < 0.0) {
            (true, true) => std::cmp::Ordering::Equal,
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            (false, false) => a.1.total_cmp(&b.1),
        },
    );

    let n = p_values.len();
    let mut significant = vec![false; n];

    for (i, (original_idx, p)) in indexed.iter().enumerate() {
        if !p.is_finite() || *p < 0.0 || *p > 1.0 {
            break;
        }
        let corrected_alpha = alpha / (n - i) as f32;
        if *p <= corrected_alpha {
            significant[*original_idx] = true;
        } else {
            // Once we fail to reject, all subsequent are not significant
            break;
        }
    }

    significant
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Comprehensive data quality report
#[derive(Debug, Clone)]
pub struct DataQualityReport {
    /// sample size.
    pub sample_size: usize,
    /// sample size adequate.
    pub sample_size_adequate: bool,
    /// required sample size.
    pub required_sample_size: usize,
    /// nan values.
    pub nan_values: usize,
    /// infinite values.
    pub infinite_values: usize,
    /// outliers iqr.
    pub outliers_iqr: Vec<usize>,
    /// outliers zscore.
    pub outliers_zscore: Vec<usize>,
    /// distribution skewness.
    pub distribution_skewness: f32,
    /// distribution kurtosis.
    pub distribution_kurtosis: f32,
    /// is valid.
    pub is_valid: bool,
    /// warnings.
    pub warnings: Vec<String>,
}

impl DataQualityReport {
    #[must_use]
    /// from data.
    pub fn from_data(values: &[f32], expected_effect_size: f32) -> Self {
        let sample_size = values.len();
        let nan_count = values.iter().filter(|&&v| v.is_nan()).count();
        let inf_count = values.iter().filter(|&&v| v.is_infinite()).count();
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let clean_values = clean_numeric(values);
        let required_n = required_sample_size(expected_effect_size, 0.05, 0.80);
        let sample_size_adequate = clean_values.len() >= required_n;
        let outliers_iqr = detect_outliers(&clean_values);
        let outliers_zscore = detect_outliers_zscore(&clean_values, 3.0);
        let distribution_skewness = calculate_skewness(&clean_values);
        let distribution_kurtosis = calculate_kurtosis(&clean_values);

        let warnings = Self::build_warnings(
            sample_size_adequate,
            nan_count,
            inf_count,
            &outliers_iqr,
            distribution_skewness,
            clean_values.len(),
            required_n,
            expected_effect_size,
        );

        Self {
            sample_size,
            sample_size_adequate,
            required_sample_size: required_n,
            nan_values: nan_count,
            infinite_values: inf_count,
            outliers_iqr,
            outliers_zscore,
            distribution_skewness,
            distribution_kurtosis,
            is_valid: clean_values.len() >= 2,
            warnings,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn build_warnings(
        adequate: bool,
        nan_count: usize,
        inf_count: usize,
        outliers_iqr: &[usize],
        skewness: f32,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        clean_len: usize,
        required_n: usize,
        effect_size: f32,
    ) -> Vec<String> {
        let mut warnings = Vec::new();
        if !adequate {
            warnings.push(format!(
                "Sample size {clean_len} is below recommended {required_n} for effect size {effect_size} with 80% power"
            ));
        }
        if nan_count > 0 {
            warnings.push(format!("Found {nan_count} NaN values in data"));
        }
        if inf_count > 0 {
            warnings.push(format!("Found {inf_count} infinite values in data"));
        }
        if !outliers_iqr.is_empty() {
            warnings.push(format!(
                "Detected {} outliers using IQR method",
                outliers_iqr.len()
            ));
        }
        if skewness.abs() > 2.0 {
            warnings.push(format!(
                "High skewness detected ({skewness:.2}), consider transformation" // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            ));
        }
        warnings
    }

    /// Print a summary of the data quality report
    pub fn print_summary(&self) {
        tracing::info!("=== Data Quality Report ===");
        eprintln!(
            "Sample size: {} (required: {})",
            self.sample_size, self.required_sample_size
        );
        eprintln!("Sample size adequate: {}", self.sample_size_adequate);
        eprintln!("NaN values: {}", self.nan_values);
        eprintln!("Infinite values: {}", self.infinite_values);
        eprintln!("Outliers (IQR): {}", self.outliers_iqr.len());
        eprintln!("Outliers (Z-score): {}", self.outliers_zscore.len());
        eprintln!("Skewness: {:.3}", self.distribution_skewness);
        eprintln!("Kurtosis: {:.3}", self.distribution_kurtosis);
        eprintln!("Valid: {}", self.is_valid);

        if !self.warnings.is_empty() {
            eprintln!("\nWarnings:");
            for warning in &self.warnings {
                eprintln!("  - {warning}");
            }
        }
    }
}

// =============================================================================
// STATISTICAL FUNCTIONS WITH VALIDATION
// =============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Cohen's d effect size with confidence interval
/// Per simulation-plan.md §5.3.3 line 638
///
/// Returns: (d, `lower_ci`, `upper_ci`)
#[must_use]
pub fn cohens_d_with_ci(group1: &[f32], group2: &[f32]) -> (f32, f32, f32) {
    // Validate numeric inputs
    if let Err(invalid) = validate_numeric(group1) {
        eprintln!("Warning: NaN/Inf in group1 at indices: {invalid:?}");
    }
    if let Err(invalid) = validate_numeric(group2) {
        eprintln!("Warning: NaN/Inf in group2 at indices: {invalid:?}");
    }

    // Clean the data
    let g1_clean = clean_numeric(group1);
    let g2_clean = clean_numeric(group2);

    let n1 = g1_clean.len() as f32;
    let n2 = g2_clean.len() as f32;

    if n1 < 2.0 || n2 < 2.0 {
        return (0.0, 0.0, 0.0);
    }

    // Check sample size adequacy for medium effect size (d=0.5)
    if !validate_sample_size(g1_clean.len(), 0.5, 0.05, 0.80) {
        eprintln!(
            "Warning: Sample size {} may be underpowered for detecting medium effects",
            g1_clean.len()
        );
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }
    if !validate_sample_size(g2_clean.len(), 0.5, 0.05, 0.80) {
        eprintln!(
            "Warning: Sample size {} may be underpowered for detecting medium effects",
            g2_clean.len()
        );
    }

    // Means
    let mean1 = g1_clean.iter().sum::<f32>() / n1;
    let mean2 = g2_clean.iter().sum::<f32>() / n2;

    // Standard deviations
    let var1 = g1_clean.iter().map(|x| (x - mean1).powi(2)).sum::<f32>() / (n1 - 1.0);
    let var2 = g2_clean.iter().map(|x| (x - mean2).powi(2)).sum::<f32>() / (n2 - 1.0);
    let _sd1 = var1.sqrt();
    let _sd2 = var2.sqrt();

    // Pooled standard deviation
    // BANNED-001: "standard deviation" is a technical statistical term
    let pooled_var = (((n1 - 1.0) * var1 + (n2 - 1.0) * var2) / (n1 + n2 - 2.0)).max(0.0);
    let pooled_sd = pooled_var.sqrt();

    let tol = 1e-6_f32 * (mean1.abs().max(mean2.abs()).max(1.0));
    if pooled_sd <= tol {
        return (0.0, 0.0, 0.0);
    }

    // Cohen's d
    let d = (mean1 - mean2) / pooled_sd;

    // Standard error of d
    // BANNED-001: "standard error" is a technical statistical term
    // SE_d = sqrt((n1+n2)/(n1*n2) + d²/(2*(n1+n2)))
    let se_d = ((n1 + n2) / (n1 * n2) + d.powi(2) / (2.0 * (n1 + n2))).sqrt();

    // 95% CI using normal approximation
    let z = 1.96; // 95% confidence
    let lower_ci = d - z * se_d;
    let upper_ci = d + z * se_d;

    (d, lower_ci, upper_ci)
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Cohen's d (without CI) - simpler version
#[must_use]
pub fn cohens_d(group1: &[f32], group2: &[f32]) -> f32 {
    let (d, _, _) = cohens_d_with_ci(group1, group2);
    d
}

/// Pearson correlation coefficient
/// Per simulation-plan.md §5.3.3 line 637
#[must_use]
pub fn pearson_r(x: &[f32], y: &[f32]) -> Option<f32> {
    // Check sample size
    if !validate_sample_size(x.len(), 0.5, 0.05, 0.80) {
        eprintln!("Warning: Sample size {} underpowered for correlation (need n≥{} for r=0.5, power=0.80)",
                 x.len(), required_sample_size(0.5, 0.05, 0.80));
    }

    // Check for NaN/Inf
    if let Err(invalid) = validate_numeric(x) {
        eprintln!("Error: NaN/Inf in x at indices: {invalid:?}");
        return None;
    }
    if let Err(invalid) = validate_numeric(y) {
        eprintln!("Error: NaN/Inf in y at indices: {invalid:?}");
        return None;
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    if x.len() != y.len() || x.len() < 2 {
        return None;
    }

    // Check for outliers that might affect correlation
    let outliers_x = detect_outliers(x);
    let outliers_y = detect_outliers(y);
    if !outliers_x.is_empty() || !outliers_y.is_empty() {
        eprintln!("Warning: Detected outliers - x: {outliers_x:?}, y: {outliers_y:?}");
    }

    let n = x.len() as f32;
    let mean_x = x.iter().sum::<f32>() / n;
    let mean_y = y.iter().sum::<f32>() / n;

    let mut num = 0.0;
    let mut den_x = 0.0;
    let mut den_y = 0.0;

    for i in 0..x.len() {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        num += dx * dy;
        den_x += dx * dx;
        den_y += dy * dy;
    }

    let den = (den_x * den_y).sqrt();
    if den == 0.0 || !den.is_finite() {
        Some(0.0)
    } else {
        Some((num / den).clamp(-1.0, 1.0))
    }
}

pub mod metrics;
pub use metrics::*;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Mean and standard deviation
#[must_use]
pub fn mean_std(values: &[f32]) -> (f32, f32) {
    if values.is_empty() {
        return (0.0, 0.0);
    }

    // Clean data
    let clean_values = clean_numeric(values);

    if clean_values.is_empty() {
        return (0.0, 0.0);
    }

    let n = clean_values.len() as f32;
    let mean = clean_values.iter().sum::<f32>() / n;

    if clean_values.len() < 2 {
        return (mean, 0.0);
    }

    let variance = clean_values.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / (n - 1.0);
    (mean, variance.sqrt())
}

/// Confidence interval for mean (Student-t distribution approximation)
/// Uses Cornish-Fisher expansion for small samples (n <= 30) and z=1.96 for large samples
#[must_use]
pub fn mean_ci(values: &[f32]) -> (f32, f32, f32) {
    let (mean, std) = mean_std(values);
    let clean_values = clean_numeric(values);
    let n = clean_values.len() as f32;

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    if n < 2.0 {
        return (mean, mean, mean);
    }

    // Standard error
    let se = std / n.sqrt();

    // 95% CI Student-t critical value
    let z = if n > 30.0 {
        1.96
    } else {
        let df = (n - 1.0).max(1.0);
        match df.round() as usize {
            1 => 12.7062,
            2 => 4.3027,
            3 => 3.1824,
            4 => 2.7764,
            _ => {
                let z_norm = 1.959964_f32;
                z_norm + (9.4871_f32 / (4.0 * df)) + (270.36_f32 / (96.0 * df * df))
            }
        }
    };
    let margin = z * se;

    (mean, mean - margin, mean + margin)
}

/// Two-sample t-test (Welch's t-test for unequal variances)
/// Returns t-statistic and approximate p-value
#[must_use]
pub fn welch_t_test(group1: &[f32], group2: &[f32]) -> (f32, f32) {
    // Validate inputs
    if let Err(invalid) = validate_numeric(group1) {
        eprintln!("Warning: NaN/Inf in group1 at indices: {invalid:?}");
    }
    if let Err(invalid) = validate_numeric(group2) {
        eprintln!("Warning: NaN/Inf in group2 at indices: {invalid:?}");
    }

    let g1_clean = clean_numeric(group1);
    let g2_clean = clean_numeric(group2);

    let n1 = g1_clean.len() as f32;
    let n2 = g2_clean.len() as f32;

    if n1 < 2.0 || n2 < 2.0 {
        return (0.0, 1.0);
    }

    // Check sample size adequacy
    if !validate_sample_size(g1_clean.len(), 0.5, 0.05, 0.80) {
        eprintln!(
            "Warning: Group 1 sample size {} may be underpowered",
            g1_clean.len()
        );
    }
    if !validate_sample_size(g2_clean.len(), 0.5, 0.05, 0.80) {
        eprintln!(
            "Warning: Group 2 sample size {} may be underpowered",
            g2_clean.len()
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let (mean1, std1) = mean_std(&g1_clean);
    let (mean2, std2) = mean_std(&g2_clean);

    let n1_f64 = n1 as f64;
    let n2_f64 = n2 as f64;
    let v1 = (std1 as f64).powi(2) / n1_f64;
    let v2 = (std2 as f64).powi(2) / n2_f64;
    let se_f64 = (v1 + v2).sqrt();

    if se_f64 <= 1e-15 {
        let diff = (mean1 - mean2).abs() as f64;
        if diff <= 1e-6 * (mean1.abs().max(mean2.abs()).max(1.0) as f64) {
            return (0.0, 1.0);
        }
        let t = if mean1 > mean2 {
            f32::INFINITY
        } else {
            f32::NEG_INFINITY
        };
        return (t, 0.0);
    }

    let t = ((mean1 as f64 - mean2 as f64) / se_f64) as f32;
    let num_df = (v1 + v2).powi(2);
    let den_df = (v1.powi(2) / (n1_f64 - 1.0)) + (v2.powi(2) / (n2_f64 - 1.0));
    let df = if den_df > 1e-15 {
        (num_df / den_df).max(1.0)
    } else {
        (n1_f64 + n2_f64 - 2.0).max(1.0)
    };

    let t_abs = (t as f64).abs();
    let z_equiv = if df > 1000.0 {
        t_abs
    } else {
        let arg = (1.0 + (t_abs * t_abs) / df).ln();
        (df * arg).sqrt() * (1.0 - 0.25 / df)
    };
    let p = (2.0 * (1.0 - normal_cdf(z_equiv as f32))).clamp(0.0, 1.0);

    (t, p)
}

/// Bootstrap confidence interval
/// Uses percentile method with B resamples
#[must_use]
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
pub fn bootstrap_ci(values: &[f32], b: usize) -> (f32, f32, f32) {
    // Clean data first
    let clean_values = clean_numeric(values);

    if clean_values.len() < 2 || b == 0 {
        let mean = if clean_values.is_empty() {
            0.0
        } else {
            clean_values.iter().sum::<f32>() / clean_values.len() as f32
        };
        return (mean, mean, mean);
    }

    use rand::Rng;
    use rand::SeedableRng;

    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let n = clean_values.len();
    let mut bootstrap_means = Vec::with_capacity(b);

    for _ in 0..b {
        let mut sum = 0.0;
        for _ in 0..n {
            let idx = rng.gen_range(0..n);
            sum += clean_values[idx];
        }
        bootstrap_means.push(sum / n as f32);
    }

    bootstrap_means.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mean = clean_values.iter().sum::<f32>() / clean_values.len() as f32;
    let lower_idx = ((b - 1) as f32 * 0.025).round() as usize;
    let upper_idx = ((b - 1) as f32 * 0.975).round() as usize;

    (
        mean,
        bootstrap_means[lower_idx.min(b - 1)],
        bootstrap_means[upper_idx.min(b - 1)],
    )
}

// =============================================================================
// POM VALIDATION
// =============================================================================

pub mod pom;
pub use pom::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_erf() {
        // Test values from known erf approximations
        assert!((erf(0.0) - 0.0).abs() < 0.001);
        assert!((erf(1.0) - 0.8427).abs() < 0.01);
        assert!((erf(-1.0) + 0.8427).abs() < 0.01);
        assert!((erf(2.0) - 0.9953).abs() < 0.01);
    }

    #[test]
    fn test_normal_cdf() {
        assert!((normal_cdf(0.0) - 0.5).abs() < 0.001);
        assert!(normal_cdf(1.96) > 0.975);
        assert!(normal_cdf(-1.96) < 0.025);
    }

    #[test]
    fn test_validate_sample_size() {
        // Test that small samples fail
        assert!(!validate_sample_size(3, 0.5, 0.05, 0.80));
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert!(!validate_sample_size(10, 0.5, 0.05, 0.80));

        // Test that large enough samples pass
        assert!(validate_sample_size(64, 0.5, 0.05, 0.80));

        // Test with different effect sizes
        assert!(!validate_sample_size(64, 0.2, 0.05, 0.80)); // Small effect needs more
        assert!(validate_sample_size(400, 0.2, 0.05, 0.80));

        // Edge case
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert!(!validate_sample_size(1, 0.5, 0.05, 0.80));
        assert!(!validate_sample_size(0, 0.5, 0.05, 0.80));
    }

    #[test]
    fn test_required_sample_size() {
        // Medium effect size (d=0.5) requires ~64 per group for 80% power
        let n = required_sample_size(0.5, 0.05, 0.80);
        assert!((60..=70).contains(&n));

        // Small effect size (d=0.2) requires much more
        let n_small = required_sample_size(0.2, 0.05, 0.80);
        assert!(n_small > 300);

        // Large effect size (d=0.8) requires less
        let n_large = required_sample_size(0.8, 0.05, 0.80);
        assert!(n_large < 30);
    }

    #[test]
    fn test_validate_numeric() {
        // Valid data
        let valid = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!(validate_numeric(&valid).is_ok());

        // With NaN
        let with_nan = vec![1.0, f32::NAN, 3.0];
        let result = validate_numeric(&with_nan);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), vec![1]);

        // With Infinity
        let with_inf = vec![1.0, f32::INFINITY, 3.0];
        let result = validate_numeric(&with_inf);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), vec![1]);

        // With negative infinity
        let with_neg_inf = vec![1.0, f32::NEG_INFINITY, 3.0];
        let result = validate_numeric(&with_neg_inf);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), vec![1]);
    }

    #[test]
    fn test_clean_numeric() {
        let dirty = vec![1.0, f32::NAN, 3.0, f32::INFINITY, 5.0, f32::NEG_INFINITY];
        let clean = clean_numeric(&dirty);
        assert_eq!(clean, vec![1.0, 3.0, 5.0]);
        // TAG: surface=simulation owner=research-team rule=SIM-001

        // Already clean
        let already_clean = vec![1.0, 2.0, 3.0];
        assert_eq!(clean_numeric(&already_clean), vec![1.0, 2.0, 3.0]);

        // All dirty
        let all_dirty = vec![f32::NAN, f32::INFINITY];
        assert!(clean_numeric(&all_dirty).is_empty());
    }

    #[test]
    fn test_detect_outliers() {
        // No outliers in uniform data
        let uniform = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!(detect_outliers(&uniform).is_empty());

        // Clear outlier
        let with_outlier = vec![1.0, 2.0, 3.0, 4.0, 100.0];
        let outliers = detect_outliers(&with_outlier);
        assert_eq!(outliers, vec![4]);

        // Too small sample
        let small = vec![1.0, 2.0, 3.0];
        assert!(detect_outliers(&small).is_empty());
    }

    #[test]
    fn test_detect_outliers_zscore() {
        // No outliers within 3 SD
        let normal = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        assert!(detect_outliers_zscore(&normal, 3.0).is_empty());

        // Moderate outlier (z-score > 2.0 but < 3.0)
        let with_moderate_outlier = vec![1.0, 2.0, 3.0, 4.0, 5.0, 15.0];
        let outliers_3 = detect_outliers_zscore(&with_moderate_outlier, 3.0);
        assert!(outliers_3.is_empty()); // Not extreme enough for 3 SD

        // Lower threshold catches it
        let outliers_2 = detect_outliers_zscore(&with_moderate_outlier, 2.0);
        assert!(!outliers_2.is_empty());

        // More extreme outlier with larger sample to stabilize SD
        let with_extreme = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 100.0];
        let outliers_extreme = detect_outliers_zscore(&with_extreme, 3.0);
        assert!(!outliers_extreme.is_empty()); // Should detect index 10
    }

    #[test]
    fn test_multiple_comparison_corrections() {
        let p_values = vec![0.01, 0.03, 0.05, 0.1, 0.5];
        // TAG: surface=simulation owner=research-team rule=SIM-001

        // Bonferroni is very conservative
        let bonferroni = bonferroni_correction(&p_values, 0.05);
        assert_eq!(bonferroni.len(), 5);

        // FDR is less conservative
        let fdr = fdr_correction(&p_values, 0.05);
        assert_eq!(fdr.len(), 5);

        // Holm is intermediate
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let holm = holm_bonferroni_correction(&p_values, 0.05);
        assert_eq!(holm.len(), 5);

        // Test that ordering matters
        // With alpha=0.05 and 5 tests, Bonferroni requires p < 0.01
        assert!(bonferroni[0]); // p=0.01 is significant
        assert!(!bonferroni[1]); // p=0.03 is not

        // FDR should be more lenient
        // At least the first few should be significant
        assert!(fdr[0]);
    }
    #[test]
    fn test_fdr_benjamini_hochberg_step_up_monotonicity() {
        // m=3, q=0.05. Thresholds: k=1 -> 0.0167, k=2 -> 0.0333, k=3 -> 0.0500.
        // p = [0.020, 0.025, 0.030].
        // At k=3, 0.030 <= 0.0500, so k*=3!
        // Benjamini-Hochberg step-up requires rejecting all 3 hypotheses.
        let p_values = vec![0.020, 0.025, 0.030];
        let fdr = fdr_correction(&p_values, 0.05);
        assert_eq!(fdr, vec![true, true, true]);

        // Non-monotonic or invalid values
        let invalid = vec![f32::NAN, -0.05, 0.01];
        let fdr_invalid = fdr_correction(&invalid, 0.05);
        assert_eq!(fdr_invalid, vec![false, false, true]);
    }

    #[test]
    fn test_fisher_z_ci_boundary_and_perfect_correlation() {
        // Perfect correlation r = 1.0 and -1.0 must not return NaN
        let (lower_pos, upper_pos) = fisher_z_ci(1.0, 50);
        assert!(lower_pos.is_finite() && upper_pos.is_finite());
        assert!(lower_pos <= upper_pos);

        let (lower_neg, upper_neg) = fisher_z_ci(-1.0, 50);
        assert!(lower_neg.is_finite() && upper_neg.is_finite());
        assert!(lower_neg <= upper_neg);

        // NaN input
        let (low_nan, up_nan) = fisher_z_ci(f32::NAN, 50);
        assert!(low_nan.is_nan() && up_nan.is_nan());
    }

    #[test]
    fn test_data_quality_report() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 100.0]; // With outlier
        let report = DataQualityReport::from_data(&data, 0.5);

        assert_eq!(report.sample_size, 5);
        assert!(!report.sample_size_adequate); // n=5 is too small
        assert_eq!(report.nan_values, 0);
        assert_eq!(report.infinite_values, 0);
        assert!(!report.outliers_iqr.is_empty()); // Should detect 100.0 as outlier
        assert!(!report.warnings.is_empty());
    }

    #[test]
    fn test_data_quality_report_with_nan() {
        let data = vec![1.0, f32::NAN, 3.0, f32::INFINITY, 5.0];
        let report = DataQualityReport::from_data(&data, 0.5);

        assert_eq!(report.nan_values, 1);
        assert_eq!(report.infinite_values, 1);
        assert!(!report.warnings.is_empty());
    }

    #[test]
    fn test_cohens_d() {
        let group1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let group2 = vec![3.0, 4.0, 5.0, 6.0, 7.0];

        let (d, lower, upper) = cohens_d_with_ci(&group1, &group2);

        // Group2 is shifted up by 2, should have negative d
        assert!(d < 0.0);
        // Effect size should be moderate to large
        assert!(d.abs() > 0.5);
        // CI should be ordered
        assert!(lower < d && d < upper);
        // TAG: surface=simulation owner=research-team rule=SIM-001
    }

    #[test]
    fn test_cohens_d_with_nan() {
        let group1 = vec![1.0, 2.0, f32::NAN, 4.0, 5.0];
        let group2 = vec![3.0, 4.0, 5.0, 6.0, 7.0];

        // Should still work with NaN (cleaned internally)
        let (d, _, _) = cohens_d_with_ci(&group1, &group2);
        assert!(d < 0.0);
    }

    #[test]
    fn test_pearson_r() {
        // Perfect positive correlation
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let r = pearson_r(&x, &y);
        assert!(r.is_some());
        assert!((r.unwrap() - 1.0).abs() < 0.001);

        // Perfect negative correlation
        let y_neg = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        let r_neg = pearson_r(&x, &y_neg);
        assert!(r_neg.is_some());
        assert!((r_neg.unwrap() + 1.0).abs() < 0.001);

        // No correlation
        let y_const = vec![5.0, 5.0, 5.0, 5.0, 5.0];
        let r_zero = pearson_r(&x, &y_const);
        assert!(r_zero.is_some());
        assert!(r_zero.unwrap().abs() < 0.001);
    }

    #[test]
    fn test_pearson_correlation_f64_and_f32() {
        // Positive correlation
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let r64 = pearson_correlation_f64(&x, &y);
        assert!((r64 - 1.0).abs() < 1e-6);

        let x32 = vec![1.0f32, 2.0, 3.0, 4.0, 5.0];
        let y32 = vec![2.0f32, 4.0, 6.0, 8.0, 10.0];
        let r32 = pearson_correlation_f32(&x32, &y32);
        assert!((r32 - 1.0).abs() < 1e-6);

        // Negative correlation
        let y_neg = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        let r64_neg = pearson_correlation_f64(&x, &y_neg);
        assert!((r64_neg + 1.0).abs() < 1e-6);

        // Constant value (0 denominator) -> 0.0
        let y_const = vec![5.0, 5.0, 5.0, 5.0, 5.0];
        assert_eq!(pearson_correlation_f64(&x, &y_const), 0.0);

        // Edge cases: empty and length mismatch
        assert_eq!(pearson_correlation_f64(&[], &[]), 0.0);
        assert_eq!(pearson_correlation_f64(&[1.0], &[2.0]), 0.0);
        assert_eq!(pearson_correlation_f64(&[1.0, 2.0], &[1.0]), 0.0);
    }

    #[test]
    fn test_pearson_r_with_nan() {
        let x = vec![1.0, 2.0, f32::NAN, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let r = pearson_r(&x, &y);
        // Should return None due to NaN
        assert!(r.is_none());
    }

    #[test]
    fn test_pearson_r_with_validation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let (r, report_x, report_y) = pearson_r_with_validation(&x, &y);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        assert!(r.is_some());
        assert_eq!(report_x.sample_size, 5);
        assert_eq!(report_y.sample_size, 5);
    }

    #[test]
    fn test_fisher_z_ci() {
        let r = 0.5;
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let n = 100;
        let (lower, upper) = fisher_z_ci(r, n);

        // CI should contain r
        assert!(lower < r && r < upper);
        // CI should be reasonably tight for n=100
        assert!(upper - lower < 0.5);
    }

    #[test]
    fn test_shannon_entropy() {
        // Uniform distribution: maximum entropy
        let uniform = vec![25, 25, 25, 25];
        let h_uniform = shannon_entropy_mm(&uniform);
        assert!(h_uniform > 1.9); // Close to log2(4) = 2

        // Skewed distribution: lower entropy
        let skewed = vec![90, 10, 0, 0];
        let h_skewed = shannon_entropy_mm(&skewed);
        assert!(h_skewed < h_uniform);
    }

    #[test]
    fn test_gini_coefficient() {
        // Perfect equality
        let equal = vec![10.0, 10.0, 10.0, 10.0];
        let g_equal = gini_coefficient(&equal);
        assert!((g_equal - 0.0).abs() < 0.001);

        // Perfect inequality
        let unequal = vec![0.0, 0.0, 0.0, 40.0];
        let g_unequal = gini_coefficient(&unequal);
        assert!(g_unequal > 0.7);

        // Intermediate inequality
        let mid = vec![5.0, 10.0, 15.0, 20.0];
        let g_mid = gini_coefficient(&mid);
        assert!(g_mid > 0.2 && g_mid < 0.5);
    }

    #[test]
    fn test_gini_with_nan() {
        let with_nan = vec![10.0, 10.0, f32::NAN, 10.0];
        let g = gini_coefficient(&with_nan);
        // Should still calculate with cleaned data
        assert!((g - 0.0).abs() < 0.001);
    }

    #[test]
    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn test_mean_std() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, std) = mean_std(&values);

        assert!((mean - 3.0).abs() < 0.001);
        assert!((std - 1.581).abs() < 0.01); // sqrt(2.5)
    }

    #[test]
    fn test_mean_std_with_nan() {
        let values = vec![1.0, 2.0, f32::NAN, 4.0, 5.0];
        let (mean, _std) = mean_std(&values);

        // Should use cleaned data (4 values: 1,2,4,5)
        assert!((mean - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_pom_validation() {
        // Should pass
        let result = validate_pom_target("Wealth Inequality", "Gini", 0.45, 0.4, 0.6);
        assert!(result.passed);
        assert_eq!(result.pattern, "Wealth Inequality");

        // Should fail
        let result_fail = validate_pom_target("Criticality", "Branching Ratio", 0.5, 0.9, 1.1);
        assert!(!result_fail.passed);
    }

    #[test]
    fn test_pom_validation_with_nan() {
        let result = validate_pom_target("Test", "Metric", f32::NAN, 0.0, 1.0);
        assert!(!result.passed);
        assert!(!result.data_quality_warnings.is_empty());
    }

    #[test]
    fn test_welch_t_test() {
        let group1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let group2 = vec![1.0, 2.0, 3.0, 4.0, 5.0]; // Same

        let (t, p) = welch_t_test(&group1, &group2);
        assert!(t.abs() < 0.001);
        assert!(p > 0.9); // Not significant

        let group3 = vec![10.0, 11.0, 12.0, 13.0, 14.0]; // Very different
        let (t2, p2) = welch_t_test(&group1, &group3);
        assert!(t2.abs() > 5.0);
        assert!(p2 < 0.01); // Significant
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    #[test]
    fn test_welch_t_test_with_nan() {
        let group1 = vec![1.0, 2.0, f32::NAN, 4.0, 5.0];
        let group2 = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        // Should work with cleaned data
        let (t, p) = welch_t_test(&group1, &group2);
        assert!(t.abs() < 0.5); // Similar groups
        assert!(p > 0.5);
    }

    #[test]
    fn test_constant_f32_dispersion_invariance() {
        let constant_data = vec![100.0f32; 32];
        let outliers = detect_outliers_zscore(&constant_data, 3.0);
        assert!(
            outliers.is_empty(),
            "Constant data must produce no Z-score outliers"
        );

        let skewness = calculate_skewness(&constant_data);
        assert_eq!(skewness, 0.0, "Constant data must produce zero skewness");

        let kurtosis = calculate_kurtosis(&constant_data);
        assert_eq!(
            kurtosis, 0.0,
            "Constant data must produce zero excess kurtosis"
        );

        let large_const = vec![1_000_000.0f32; 32];
        assert!(detect_outliers_zscore(&large_const, 3.0).is_empty());
        assert_eq!(calculate_skewness(&large_const), 0.0);
        assert_eq!(calculate_kurtosis(&large_const), 0.0);
    }
    #[test]
    fn test_bonferroni_rejection_of_invalid_p_and_alpha() {
        let p_values = vec![-0.1, f32::NAN, 0.01, 1.5];
        let sig = bonferroni_correction(&p_values, 0.05);
        // Negative, NaN, and >1.0 p-values must never be marked significant
        assert_eq!(sig, vec![false, false, true, false]);

        // Invalid alpha returns all false
        let sig_invalid_alpha = bonferroni_correction(&[0.001], f32::NAN);
        assert_eq!(sig_invalid_alpha, vec![false]);
    }

    #[test]
    fn test_cohens_d_constant_float_near_zero_dispersion() {
        let g1 = vec![0.1_f32; 10];
        let g2 = vec![0.2_f32; 10];
        let (d, lower, upper) = cohens_d_with_ci(&g1, &g2);
        // With identical elements, pooled_sd is below numerical tolerance and correctly yields (0, 0, 0)
        assert_eq!(d, 0.0);
        assert_eq!(lower, 0.0);
        assert_eq!(upper, 0.0);
    }
}
