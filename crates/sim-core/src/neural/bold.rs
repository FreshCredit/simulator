//! BOLD signal generation for fMRI validation
//!
//! Generates synthetic BOLD (Blood-Oxygen-Level Dependent) signals from
//! neural simulation results for comparison with empirical fMRI data.
//! Uses the Balloon-Windkessel hemodynamic model (Friston et al., 2000).

use super::backend::SimulationResult;
use super::NeuralValidationError;
use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// BOLD signal generator
#[derive(Debug)]
pub struct BoldGenerator {
    config: BoldConfig,
}

/// BOLD configuration
#[derive(Debug, Clone)]
pub struct BoldConfig {
    /// Echo time (TE) in seconds
    pub echo_time: f32,
    /// Repetition time (TR) in seconds
    pub repetition_time: f32,
    /// Hemodynamic response function parameters
    pub hrf: HrfModel,
    /// Signal-to-noise ratio
    pub snr: f32,
    /// Spatial smoothing kernel (mm FWHM)
    pub smoothing_fwhm: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for BoldConfig {
    fn default() -> Self {
        Self {
            echo_time: 0.03,      // 30ms
            repetition_time: 2.0, // 2s (typical TR)
            hrf: HrfModel::Canonical,
            snr: 10.0,
            smoothing_fwhm: 6.0,
        }
    }
}

/// Hemodynamic response function models
#[derive(Debug, Clone, Copy)]
pub enum HrfModel {
    /// Canonical double-gamma HRF
    Canonical,
    /// Balloon-Windkessel model
    BalloonWindkessel,
    /// Rigid Fourier basis
    /// n basis.
    FourierBasis {
        /// n basis.
        n_basis: usize,
    },
}

/// BOLD signal data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoldSignal {
    /// Timestamps (aligned to TR)
    pub timestamps: Vec<f32>,
    /// BOLD signal values (percent signal change)
    pub signal: Vec<f32>,
    /// Underlying neural activity (convolved)
    pub neural_convolved: Vec<f32>,
    /// Signal quality metrics
    pub quality: BoldQualityMetrics,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// BOLD quality metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BoldQualityMetrics {
    /// tSNR (temporal SNR)
    pub tsnr: f32,
    /// Mean percent signal change
    pub mean_psc: f32,
    /// Temporal autocorrelation
    pub autocorr: f32,
    /// Frequency domain peak (Hz)
    pub freq_peak: f32,
}

#[inline]
fn safe_dt(timestamps: &[f32]) -> f32 {
    if timestamps.len() < 2 {
        0.01
    } else {
        (timestamps[1] - timestamps[0]).abs().max(1e-4)
    }
}

impl BoldGenerator {
    /// Create new BOLD generator
    #[must_use]
    pub const fn new(config: BoldConfig) -> Self {
        Self { config }
    }

    /// Generate BOLD signal from neural simulation result
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn generate(&self, result: &SimulationResult) -> Result<BoldSignal, NeuralValidationError> {
        // Combine all neuromodulator signals into neural activity
        let neural_activity = self.combine_neuromodulators(result);

        // Convolve with HRF
        let convolved = match self.config.hrf {
            HrfModel::Canonical => {
                self.convolve_canonical_hrf(&neural_activity, &result.timestamps)
            }
            HrfModel::BalloonWindkessel => self.balloon_model(&neural_activity, &result.timestamps),
            HrfModel::FourierBasis { n_basis } => {
                self.fourier_basis_hrf(&neural_activity, &result.timestamps, n_basis)
            }
        };

        // Resample to TR
        let (timestamps_resampled, signal_resampled) =
            self.resample_to_tr(&convolved, &result.timestamps);

        // Add noise
        let signal_noisy = self.add_noise(&signal_resampled);

        // Compute quality metrics
        let quality = self.compute_quality_metrics(&signal_noisy, &signal_resampled);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        Ok(BoldSignal {
            timestamps: timestamps_resampled,
            signal: signal_noisy,
            neural_convolved: convolved,
            quality,
        })
    }

    /// Combine neuromodulator activities into single neural signal
    fn combine_neuromodulators(&self, result: &SimulationResult) -> Vec<f32> {
        let n = result.timestamps.len();
        let mut combined = vec![0.0; n];

        // Weighted combination based on neuromodulator contributions to BOLD
        // Dopamine and cortisol have stronger BOLD signals
        let weights = [0.30, 0.15, 0.35, 0.15, 0.05]; // [D, O, C, A, S]

        let activities = [
            &result.dopamine_activity,
            &result.oxytocin_activity,
            &result.cortisol_activity,
            &result.adrenaline_activity,
            &result.serotonin_activity,
        ];

        for i in 0..n {
            for (j, activity) in activities.iter().enumerate() {
                if i < activity.len() {
                    combined[i] += activity[i] * weights[j];
                }
            }
        }

        combined
    }

    /// Convolve with canonical double-gamma HRF
    fn convolve_canonical_hrf(&self, activity: &[f32], timestamps: &[f32]) -> Vec<f32> {
        if activity.is_empty() || timestamps.is_empty() {
            return Vec::new();
        }

        // Canonical HRF parameters
        let peak_delay = 6.0; // Peak at 6s
        let undershoot_delay = 16.0;
        let peak_dispersion = 1.0;
        let undershoot_dispersion = 1.0;
        let peak_undershoot_ratio = 6.0;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Bounded time step
        let dt = safe_dt(timestamps);

        // Generate HRF
        let hrf_duration = 32.0; // 32 seconds
        let n_hrf_points = (hrf_duration / dt) as usize;
        let mut hrf = vec![0.0; n_hrf_points];

        #[allow(clippy::needless_range_loop)]
        for i in 0..n_hrf_points {
            let t = i as f32 * dt;
            let gamma1 = self.gamma_pdf(t, peak_delay, peak_dispersion);
            let gamma2 = self.gamma_pdf(t, undershoot_delay, undershoot_dispersion);
            hrf[i] = gamma1 - gamma2 / peak_undershoot_ratio;
        }

        // Normalize
        let hrf_sum: f32 = hrf.iter().sum();
        if hrf_sum.abs() > 0.0 {
            for h in &mut hrf {
                *h /= hrf_sum;
            }
        }

        // Convolution
        let n = activity.len();
        let m = hrf.len();
        let mut result = vec![0.0; n];

        for i in 0..n {
            for j in 0..m {
                if i >= j && i - j < n {
                    result[i] += activity[i - j] * hrf[j];
                }
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        result
    }

    /// Gamma probability density function
    fn gamma_pdf(&self, x: f32, shape: f32, scale: f32) -> f32 {
        if x <= 0.0 {
            return 0.0;
        }
        let x_scaled = (x / scale).max(0.0);
        (x_scaled.powf(shape - 1.0) * (-x_scaled).exp()) / (scale * self.gamma(shape))
    }

    /// Approximate gamma function using Lanczos approximation (g=5, n=6)
    fn gamma(&self, x: f32) -> f32 {
        if x <= 0.0 {
            return 0.0;
        }
        if (x - 1.0).abs() < 1e-6 {
            return 1.0;
        }
        if (x - 2.0).abs() < 1e-6 {
            return 1.0;
        }
        if (x - 3.0).abs() < 1e-6 {
            return 2.0;
        }
        if (x - 0.5).abs() < 1e-6 {
            return std::f32::consts::PI.sqrt();
        }

        // Lanczos coefficients
        let p = [
            1.000_000_000_190_015,
            76.180_091_729_471_46,
            -86.505_320_329_416_77,
            24.014_098_240_830_91,
            -1.231_739_572_450_155,
            0.001_208_650_973_866_179,
            -0.000_005_395_239_384_953,
        ];
        let z = x as f64 - 1.0;
        let mut a = p[0];
        let t = z + 5.5;
        for (i, &coeff) in p.iter().enumerate().skip(1) {
            a += coeff / (z + i as f64);
        }
        let res = (2.0 * std::f64::consts::PI).sqrt() * t.max(0.0).powf(z + 0.5) * (-t).exp() * a;
        if res.is_finite() && res > 0.0 {
            res as f32
        } else {
            1.0
        }
    }

    /// Balloon-Windkessel hemodynamic model (Buxton et al., 1998; Friston et al., 2000)
    fn balloon_model(&self, activity: &[f32], timestamps: &[f32]) -> Vec<f32> {
        if activity.is_empty() || timestamps.is_empty() {
            return Vec::new();
        }
        let dt = safe_dt(timestamps);
        let n = activity.len();

        // Hemodynamic parameters (Friston et al., 2000)
        let kappa = 0.65; // Rate of signal decay
        let gamma = 0.41; // Rate of flow-dependent elimination
        let tau = 0.98; // Hemodynamic transit time
        let alpha = 0.32; // Grubb's exponent
        let rho = 0.34; // Resting oxygen extraction
        let v0 = 0.02; // Resting blood volume fraction

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // State variables
        let mut s: f32 = 0.0; // Vasodilatory signal
        let mut f: f32 = 1.0; // Inflow
        let mut v: f32 = 1.0; // Blood volume
        let mut q: f32 = 1.0; // Deoxyhemoglobin content

        let mut bold_signal = vec![0.0; n];

        for i in 0..n {
            let neural_input = activity[i].max(0.0);

            // Vasodilatory signal dynamics
            let ds = neural_input - kappa * s - gamma * (f - 1.0);
            s += ds * dt;

            // Inflow dynamics
            let df = s;
            f += df * dt;
            f = f.max(0.1); // Prevent negative flow

            // Blood volume dynamics
            let dv = (f - v.max(0.0).powf(1.0 / alpha)) / tau;
            v += dv * dt;
            v = v.max(0.01);

            // Deoxyhemoglobin dynamics (Friston et al., 2000 Eq. 4)
            // Oxygen extraction fraction: E(f, rho) = 1 - (1 - rho)^(1/f)
            let oxygen_extraction = if f > 1e-4 {
                1.0_f32 - (1.0_f32 - rho).clamp(0.0, 1.0).powf(1.0_f32 / f)
            } else {
                1.0_f32
            };
            let dq = (f * (oxygen_extraction / rho) - v.max(0.0).powf(1.0 / alpha) * (q / v)) / tau;
            q += dq * dt;
            q = q.max(0.01);

            // BOLD signal equation
            let k1 = 7.0 * rho;
            let k2 = 2.0;
            let k3 = 2.0 * rho - 0.2;

            // TAG: surface=simulation owner=research-team rule=SIM-001
            bold_signal[i] = v0 * (k1 * (1.0 - q) + k2 * (1.0 - q / v) + k3 * (1.0 - v));
        }

        bold_signal
    }

    /// Fourier basis HRF (damped harmonic basis expansion)
    fn fourier_basis_hrf(&self, activity: &[f32], timestamps: &[f32], n_basis: usize) -> Vec<f32> {
        if activity.is_empty() || timestamps.is_empty() {
            return Vec::new();
        }
        let dt = safe_dt(timestamps);
        let hrf_duration = 32.0f32;
        let n_hrf_points = (hrf_duration / dt).max(1.0) as usize;
        let n_components = n_basis.clamp(1, 8);

        let mut hrf = vec![0.0f32; n_hrf_points];
        for (i, hrf_val) in hrf.iter_mut().enumerate() {
            let t = i as f32 * dt;
            let decay = (-t / 8.0).exp();
            let mut basis_sum = 0.0;
            for k in 1..=n_components {
                let freq = (k as f32) * std::f32::consts::PI / hrf_duration;
                let sign = if k % 2 == 1 { 1.0 } else { -0.5 };
                basis_sum += sign * (freq * t).sin() / (k as f32);
            }
            *hrf_val = basis_sum * decay;
        }

        let hrf_sum: f32 = hrf.iter().map(|x| x.abs()).sum();
        if hrf_sum > 0.0 {
            for h in &mut hrf {
                *h /= hrf_sum;
            }
        }

        let n = activity.len();
        let m = hrf.len();
        let mut result = vec![0.0f32; n];
        for i in 0..n {
            for j in 0..m {
                if i >= j && i - j < n {
                    result[i] += activity[i - j] * hrf[j];
                }
            }
        }
        result
    }

    /// Resample signal to TR using linear interpolation in physical time
    fn resample_to_tr(&self, signal: &[f32], timestamps: &[f32]) -> (Vec<f32>, Vec<f32>) {
        let tr = self.config.repetition_time;
        let t_max = timestamps.last().copied().unwrap_or(0.0);
        let n_points = (t_max / tr).max(0.0) as usize + 1;

        let mut new_timestamps = Vec::with_capacity(n_points);
        let mut resampled = Vec::with_capacity(n_points);

        if signal.is_empty() || timestamps.is_empty() {
            return (new_timestamps, resampled);
        }

        let first_ts = timestamps.first().copied().unwrap_or(0.0);
        let last_ts = timestamps.last().copied().unwrap_or(0.0);
        let first_sig = signal.first().copied().unwrap_or(0.0);
        let last_sig = signal.last().copied().unwrap_or(0.0);

        for i in 0..n_points {
            let t = i as f32 * tr;
            new_timestamps.push(t);

            if t <= first_ts {
                resampled.push(first_sig);
            } else if t >= last_ts {
                resampled.push(last_sig);
            } else {
                let mut idx = 0;
                while idx + 1 < timestamps.len() && timestamps[idx + 1] < t {
                    idx += 1;
                }
                let t0 = timestamps[idx];
                let t1 = timestamps[idx + 1];
                let s0 = signal[idx];
                let s1 = signal[idx + 1];
                let frac = if (t1 - t0).abs() > 1e-6 {
                    (t - t0) / (t1 - t0)
                } else {
                    0.0
                };
                resampled.push(s0 + frac * (s1 - s0));
            }
        }

        (new_timestamps, resampled)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Add realistic fMRI Gaussian noise (scaled to signal RMS per Buxton / Welvaert & Rosseel 2013)
    fn add_noise(&self, signal: &[f32]) -> Vec<f32> {
        use rand::rngs::StdRng;
        use rand::Rng;
        use rand::SeedableRng;

        let n = signal.len();
        if n == 0 {
            return Vec::new();
        }

        let mut rng = StdRng::seed_from_u64(42);
        let rms = (signal.iter().map(|&x| x * x).sum::<f32>() / n as f32).sqrt();
        let std_dev = rms / self.config.snr.max(1e-4);

        if std_dev <= 0.0 || !std_dev.is_finite() {
            return signal.to_vec();
        }

        let mut noisy = Vec::with_capacity(n);
        let mut i = 0;
        while i < n {
            let u1: f32 = rng.gen_range(1e-7..1.0);
            let u2: f32 = rng.gen_range(0.0..std::f32::consts::TAU);
            let r = (-2.0 * u1.ln()).sqrt() * std_dev;
            noisy.push(signal[i] + r * u2.cos());
            i += 1;
            if i < n {
                noisy.push(signal[i] + r * u2.sin());
                i += 1;
            }
        }
        noisy
    }

    /// Compute quality metrics
    fn compute_quality_metrics(
        &self,
        noisy_signal: &[f32],
        clean_signal: &[f32],
    ) -> BoldQualityMetrics {
        let n = noisy_signal.len();
        if n == 0 {
            return BoldQualityMetrics::default();
        }

        // Temporal SNR
        let mean_signal = noisy_signal.iter().sum::<f32>() / n as f32;
        let noise: Vec<f32> = noisy_signal
            .iter()
            .zip(clean_signal.iter())
            .map(|(n, c)| n - c)
            .collect();
        let noise_std = (noise.iter().map(|&x| x * x).sum::<f32>() / n as f32).sqrt();
        let tsnr = if noise_std > 0.0 {
            mean_signal / noise_std
        } else {
            0.0
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Mean percent signal change
        let mean_psc = noisy_signal.iter().sum::<f32>() / n as f32 * 100.0;

        // Temporal autocorrelation (lag-1)
        let autocorr = if n > 1 {
            let num: f32 = noisy_signal
                .windows(2)
                .map(|w| (w[0] - mean_signal) * (w[1] - mean_signal))
                .sum();
            let den: f32 = noisy_signal
                .iter()
                .map(|&x| (x - mean_signal).powi(2))
                .sum();
            if den > 0.0 {
                num / den
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Frequency peak (simplified)
        let freq_peak = 0.1; // Default low-frequency peak

        BoldQualityMetrics {
            tsnr,
            mean_psc,
            autocorr,
            freq_peak,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compare generated BOLD to empirical fMRI data
    #[must_use]
    pub fn compare_to_empirical(
        &self,
        bold: &BoldSignal,
        empirical_reference: &BoldReference,
    ) -> BoldComparisonResult {
        let tsnr_match = (bold.quality.tsnr - empirical_reference.tsnr).abs() < 10.0;
        let psc_match = (bold.quality.mean_psc - empirical_reference.mean_psc).abs() < 0.5;
        let autocorr_match = (bold.quality.autocorr - empirical_reference.autocorr).abs() < 0.2;

        BoldComparisonResult {
            tsnr_match,
            psc_match,
            autocorr_match,
            overall_score: if tsnr_match && psc_match && autocorr_match {
                1.0
            } else {
                0.0
            },
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Empirical BOLD reference data
#[derive(Debug, Clone)]
pub struct BoldReference {
    /// tsnr.
    pub tsnr: f32,
    /// mean psc.
    pub mean_psc: f32,
    /// autocorr.
    pub autocorr: f32,
    /// source.
    pub source: String,
}

impl BoldReference {
    /// Create reference from typical fMRI data
    #[must_use]
    pub fn typical_fmri() -> Self {
        Self {
            tsnr: 50.0,
            mean_psc: 1.5,
            autocorr: 0.3,
            source: "Typical fMRI (3T, 2mm, TR=2s)".to_string(),
        }
    }

    /// Create reference from high-quality fMRI
    #[must_use]
    pub fn high_quality_fmri() -> Self {
        Self {
            tsnr: 100.0,
            mean_psc: 2.0,
            autocorr: 0.25,
            source: "High-quality fMRI (7T, 1mm, TR=1s)".to_string(),
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// BOLD comparison result
#[derive(Debug, Clone)]
pub struct BoldComparisonResult {
    /// tsnr match.
    pub tsnr_match: bool,
    /// psc match.
    pub psc_match: bool,
    /// autocorr match.
    pub autocorr_match: bool,
    /// overall score.
    pub overall_score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neural::DerivedDocasParams;
    use std::collections::HashMap;

    fn create_test_result() -> SimulationResult {
        SimulationResult {
            agent_id: 0,
            timestamps: (0..100).map(|i| i as f32 * 0.01).collect(),
            dopamine_activity: (0..100)
                .map(|i| (i as f32 * 0.1).sin() * 0.5 + 0.5)
                .collect(),
            oxytocin_activity: (0..100)
                .map(|i| (i as f32 * 0.05).cos() * 0.3 + 0.3)
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
                .collect(),
            cortisol_activity: (0..100).map(|i| if i > 50 { 0.8 } else { 0.2 }).collect(),
            adrenaline_activity: vec![0.4; 100],
            serotonin_activity: vec![0.5; 100],
            derived_docas: DerivedDocasParams::default(),
            bold_signal: None,
            metadata: HashMap::default(),
        }
    }

    #[test]
    fn test_bold_generation() {
        let generator = BoldGenerator::new(BoldConfig::default());
        let result = create_test_result();

        let bold = generator.generate(&result).unwrap();

        assert!(!bold.signal.is_empty());
        assert!(!bold.timestamps.is_empty());
        assert_eq!(bold.signal.len(), bold.timestamps.len());
    }

    #[test]
    fn test_balloon_model() {
        let generator = BoldGenerator::new(BoldConfig::default());
        let result = create_test_result();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let activity = generator.combine_neuromodulators(&result);
        let bold = generator.balloon_model(&activity, &result.timestamps);

        assert!(!bold.is_empty());

        // BOLD signal should be smooth
        for i in 1..bold.len() {
            let diff = (bold[i] - bold[i - 1]).abs();
            assert!(diff < 0.5, "BOLD signal changed too abruptly");
        }
    }

    #[test]
    fn test_bold_quality_metrics() {
        let generator = BoldGenerator::new(BoldConfig::default());
        let result = create_test_result();

        let bold = generator.generate(&result).unwrap();

        // Quality metrics should be reasonable
        assert!(
            bold.quality.tsnr >= 0.0,
            "tSNR should be non-negative, got {}",
            bold.quality.tsnr
        );
        assert!(
            bold.quality.mean_psc.abs() < 10.0,
            "PSC should be within reasonable range"
        );
        assert!(
            bold.quality.autocorr >= -1.0 && bold.quality.autocorr <= 1.0,
            "Autocorrelation should be in [-1, 1]"
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_bold_empirical_comparison() {
        let generator = BoldGenerator::new(BoldConfig::default());
        let result = create_test_result();
        let bold = generator.generate(&result).unwrap();

        let reference = BoldReference::typical_fmri();
        let comparison = generator.compare_to_empirical(&bold, &reference);

        // Should return valid comparison result
        assert!(comparison.overall_score >= 0.0 && comparison.overall_score <= 1.0);
    }

    #[test]
    fn test_bold_timestamp_degenerate_resilience() {
        let generator = BoldGenerator::new(BoldConfig::default());
        assert!(generator.convolve_canonical_hrf(&[], &[]).is_empty());
        assert!(generator.balloon_model(&[], &[]).is_empty());
        assert!(generator.fourier_basis_hrf(&[], &[], 4).is_empty());

        let act = vec![1.0];
        let ts = vec![0.5];
        let conv = generator.convolve_canonical_hrf(&act, &ts);
        assert_eq!(conv.len(), 1);
    }

    #[test]
    fn test_gamma_pdf() {
        let generator = BoldGenerator::new(BoldConfig::default());

        // Gamma PDF should be 0 at x=0
        assert_eq!(generator.gamma_pdf(0.0, 2.0, 1.0), 0.0);

        // Gamma PDF should be positive for x > 0
        assert!(generator.gamma_pdf(1.0, 2.0, 1.0) > 0.0);

        // Peak should be around (shape-1)*scale
        let peak_x = 6.0; // for shape=6, scale=1, peak at x=5
        let pdf_at_peak = generator.gamma_pdf(peak_x, 6.0, 1.0);
        let pdf_nearby = generator.gamma_pdf(peak_x + 2.0, 6.0, 1.0);
        assert!(
            pdf_at_peak > pdf_nearby,
            "PDF should decrease away from peak"
        );
    }

    #[test]
    fn test_hrf_convolution() {
        let generator = BoldGenerator::new(BoldConfig::default());
        let result = create_test_result();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let activity = generator.combine_neuromodulators(&result);
        let convolved = generator.convolve_canonical_hrf(&activity, &result.timestamps);

        // Convolved signal should be longer or same length
        assert!(convolved.len() >= activity.len());

        // Convolved signal should be smoother (less variance)
        let activity_var: f32 =
            activity.iter().map(|&x| x * x).sum::<f32>() / activity.len() as f32;
        let convolved_var: f32 =
            convolved.iter().map(|&x| x * x).sum::<f32>() / convolved.len() as f32;
        assert!(
            convolved_var < activity_var * 2.0,
            "Convolved should not amplify too much"
        );
    }

    #[test]
    fn test_noise_addition() {
        let generator = BoldGenerator::new(BoldConfig::default());
        let signal = vec![1.0; 100];

        let noisy = generator.add_noise(&signal);

        // Noisy signal should be different from original
        let has_variance = noisy.iter().any(|&x| x != 1.0);
        assert!(has_variance, "Noise should add variance");

        // Mean should be approximately preserved
        let mean: f32 = noisy.iter().sum::<f32>() / noisy.len() as f32;
        assert!(
            (mean - 1.0).abs() < 0.1,
            "Mean should be approximately preserved"
        );
    }

    #[test]
    fn test_resample_to_tr() {
        let config = BoldConfig {
            repetition_time: 2.0, // 2s TR
            ..Default::default()
        };
        let generator = BoldGenerator::new(config);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let signal: Vec<f32> = (0..1000).map(|i| i as f32 * 0.01).collect();
        let timestamps: Vec<f32> = (0..1000).map(|i| i as f32 * 0.01).collect();

        let (new_ts, resampled) = generator.resample_to_tr(&signal, &timestamps);

        // New timestamps should be at TR intervals
        for (i, &ts) in new_ts.iter().enumerate() {
            assert!(
                (ts - i as f32 * 2.0).abs() < 0.01,
                "Timestamps should be at TR intervals"
            );
        }

        assert!(!resampled.is_empty());
    }

    #[test]
    fn test_bold_reference_creation() {
        let typical = BoldReference::typical_fmri();
        assert!(typical.tsnr > 0.0);
        assert!(typical.mean_psc > 0.0);

        let high_quality = BoldReference::high_quality_fmri();
        assert!(
            high_quality.tsnr > typical.tsnr,
            "High-quality should have better tSNR"
        );
    }
}
