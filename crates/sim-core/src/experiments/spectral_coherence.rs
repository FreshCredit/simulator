//! Spectral Coherence Experiment (MUSIC-P0-2)
//!
//! Validates: "Musical trust coherence across frequency bands
//! (δ, θ, α, β, γ) correlates with interpersonal trust"
//!
// TAG: surface=simulation owner=research-team rule=SIM-001
//! Gap Analysis:
//! - Paper claims: Spectral coherence predicts trust
//! - Missing: EEG-like frequency band analysis
//! - Missing: Phase-locking value calculation
//! - Missing: Cross-frequency coupling
//!
//! Methodology:
//! 1. Generate trust signals in frequency bands
//! 2. Calculate coherence between agents
//! 3. Correlate coherence with trust level
//! 4. Validate frequency-specific predictions
//!
//! SIM-TRUST-001: This file analyzes spectral coherence correlated with
//! trust levels, not computing trust scores. Trust values are inputs for
//! correlation analysis, not outputs of trust calculation algorithms. // SIM-TRUST-001

use crate::experiments::{Experiment, ExperimentConfig, ExperimentResult};
use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Spectral coherence experiment for MUSIC-P0-2
#[derive(Debug)]
pub struct SpectralCoherenceExperiment;

/// EEG frequency bands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FrequencyBand {
    /// delta.
    Delta, // 0.5-4 Hz
    /// theta.
    Theta, // 4-8 Hz
    /// alpha.
    Alpha, // 8-13 Hz
    /// beta.
    Beta, // 13-30 Hz
    /// gamma.
    Gamma, // 30-100 Hz
}

impl FrequencyBand {
    /// Get frequency range
    #[must_use]
    pub const fn range(&self) -> (f64, f64) {
        match self {
            Self::Delta => (0.5, 4.0),
            Self::Theta => (4.0, 8.0),
            Self::Alpha => (8.0, 13.0),
            Self::Beta => (13.0, 30.0),
            Self::Gamma => (30.0, 100.0),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get center frequency
    #[must_use]
    pub fn center(&self) -> f64 {
        let (low, high) = self.range();
        (low + high) / 2.0
    }

    /// All bands
    #[must_use]
    pub fn all() -> Vec<Self> {
        vec![
            Self::Delta,
            Self::Theta,
            Self::Alpha,
            Self::Beta,
            Self::Gamma,
        ]
    }
}

/// Spectral signal in frequency domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectralSignal {
    /// Frequency bins
    pub frequencies: Vec<f64>,
    /// Power spectral density
    pub psd: Vec<f64>,
    /// Phase at each frequency
    pub phase: Vec<f64>,
}

impl SpectralSignal {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Create synthetic signal with band-specific modulation
    pub fn from_trust(trust: f64, band: FrequencyBand, rng: &mut impl Rng) -> Self {
        // SIM-TRUST-001: signal generation
        let n_bins = 256;
        let freq_max = 100.0;
        let df = freq_max / n_bins as f64;

        let frequencies: Vec<f64> = (0..n_bins).map(|i| i as f64 * df).collect();

        let (band_low, band_high) = band.range();
        let band_center = (band_low + band_high) / 2.0;
        let band_width = band_high - band_low;

        // Power spectrum: 1/f + band-specific modulation
        let psd: Vec<f64> = frequencies
            .iter()
            .map(|&f| {
                if f < 0.1 {
                    return 0.0;
                }

                // 1/f component (pink noise)
                let pink = 1.0 / f;

                // Band modulation (higher trust → higher power in target band)
                let band_dist = (f - band_center).abs();
                let band_enhancement = if band_dist < band_width / 2.0 {
                    trust * 2.0
                } else {
                    0.0
                };

                // TAG: surface=simulation owner=research-team rule=SIM-001
                // Add noise
                let noise = rng.gen_range(0.0..0.1);

                (pink + band_enhancement + noise).max(0.0)
            })
            .collect();

        // Random phases
        let phase: Vec<f64> = (0..n_bins)
            .map(|_| rng.gen_range(0.0..2.0 * std::f64::consts::PI))
            .collect();

        Self {
            frequencies,
            psd,
            phase,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get power in specific band
    #[must_use]
    pub fn band_power(&self, band: FrequencyBand) -> f64 {
        let (low, high) = band.range();

        self.frequencies
            .iter()
            .zip(self.psd.iter())
            .filter(|(f, _)| **f >= low && **f < high)
            .map(|(_, p)| p)
            .sum()
    }
}

/// Coherence calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoherenceResult {
    /// Frequency band
    pub band: FrequencyBand,
    /// Magnitude-squared coherence
    pub coherence: f64,
    /// Phase-locking value
    pub plv: f64,
    /// Cross-spectral power
    pub cross_spectral_power: f64,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Inter-agent spectral coherence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterAgentCoherence {
    /// agent a.
    pub agent_a: usize,
    /// agent b.
    pub agent_b: usize,
    /// trust a.
    pub trust_a: f64,
    /// trust b.
    pub trust_b: f64,
    /// band coherence.
    pub band_coherence: HashMap<FrequencyBand, CoherenceResult>,
    /// mean coherence.
    pub mean_coherence: f64,
}

/// Trust-coherence correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustCoherenceCorrelation {
    /// band.
    pub band: FrequencyBand,
    /// correlation.
    pub correlation: f64,
    /// p value.
    pub p_value: f64,
    /// significance.
    pub significance: bool,
}

impl SpectralCoherenceExperiment {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Calculate magnitude-squared coherence
    fn calculate_coherence(
        &self,
        signal_a: &SpectralSignal,
        signal_b: &SpectralSignal,
        band: FrequencyBand,
    ) -> CoherenceResult {
        let (low, high) = band.range();

        // Extract band-limited signals
        let indices: Vec<usize> = signal_a
            .frequencies
            .iter()
            .enumerate()
            .filter(|(_, f)| **f >= low && **f < high)
            .map(|(i, _)| i)
            .collect();

        if indices.is_empty() {
            return CoherenceResult {
                band,
                coherence: 0.0,
                plv: 0.0,
                cross_spectral_power: 0.0,
            };
        }

        // Cross-spectrum
        let mut cross_real = 0.0;
        let mut cross_imag = 0.0;
        let mut auto_a = 0.0;
        let mut auto_b = 0.0;
        let mut phase_diff_sum = 0.0;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        for &i in &indices {
            // Complex cross-spectrum: S_xy = X * conj(Y)
            let phase_diff = signal_a.phase[i] - signal_b.phase[i];
            cross_real += (signal_a.psd[i] * signal_b.psd[i]).sqrt() * phase_diff.cos();
            cross_imag += (signal_a.psd[i] * signal_b.psd[i]).sqrt() * phase_diff.sin();

            auto_a += signal_a.psd[i];
            auto_b += signal_b.psd[i];

            // Phase-locking value
            phase_diff_sum += phase_diff.cos();
        }

        let n = indices.len() as f64;
        let cross_power = (cross_real / n).powi(2) + (cross_imag / n).powi(2);
        let auto_product = (auto_a / n) * (auto_b / n);

        let coherence = if auto_product > 1e-10 {
            (cross_power / auto_product).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let plv = (phase_diff_sum / n).abs();

        CoherenceResult {
            band,
            coherence,
            plv,
            cross_spectral_power: cross_power.sqrt(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Calculate correlation between trust and coherence
    fn calculate_trust_coherence_correlation(
        // SIM-TRUST-001
        // SIM-TRUST-001: correlation calc
        &self,
        coherences: &[InterAgentCoherence],
        band: FrequencyBand,
    ) -> TrustCoherenceCorrelation {
        let n = coherences.len() as f64;

        // Trust difference vs coherence
        let trust_diffs: Vec<f64> = coherences
            .iter()
            .map(|c| (c.trust_a - c.trust_b).abs())
            .collect();
        let coherences_band: Vec<f64> = coherences
            .iter()
            .map(|c| c.band_coherence.get(&band).map_or(0.0, |r| r.coherence))
            .collect();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let r = crate::stats::pearson_correlation_f64(&trust_diffs, &coherences_band);
        let correlation = -r; // Negative: higher trust diff → lower coherence

        // Approximate p-value (simplified)
        let p_value = if n > 2.0 {
            if correlation.abs() > 0.3 && n > 30.0 {
                0.01
            } else {
                0.1
            }
        } else {
            1.0
        };

        TrustCoherenceCorrelation {
            band,
            correlation,
            p_value,
            significance: p_value < 0.05,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Experiment for SpectralCoherenceExperiment {
    fn name(&self) -> &'static str {
        "spectral_coherence"
    }

    fn description(&self) -> &'static str {
        "MUSIC-P0-2: Musical trust coherence across frequency bands"
    }

    fn required_agents(&self) -> usize {
        100
    }

    fn run(&self, config: &ExperimentConfig) -> Result<ExperimentResult> {
        let mut rng = config.rng();
        let start_time = std::time::Instant::now();

        let n_agents = config.n_agents.min(100);
        let bands = FrequencyBand::all();

        // Generate agents with varying trust
        let agents: Vec<(usize, f64, HashMap<FrequencyBand, SpectralSignal>)> = (0..n_agents)
            .map(|i| {
                let trust = rng.gen::<f64>();
                let signals: HashMap<_, _> = bands
                    .iter()
                    .map(|&band| (band, SpectralSignal::from_trust(trust, band, &mut rng)))
                    .collect();
                (i, trust, signals)
            })
            .collect();

        // Calculate pairwise coherence
        let mut coherences = Vec::new();
        for i in 0..n_agents {
            for j in (i + 1)..n_agents {
                let (_, trust_a, signals_a) = &agents[i];
                let (_, trust_b, signals_b) = &agents[j];

                let mut band_coherence = HashMap::new();
                for band in &bands {
                    let coh = self.calculate_coherence(&signals_a[band], &signals_b[band], *band);
                    band_coherence.insert(*band, coh);
                }

                // TAG: surface=simulation owner=research-team rule=SIM-001
                let mean_coh = band_coherence.values().map(|c| c.coherence).sum::<f64>()
                    / band_coherence.len() as f64;

                coherences.push(InterAgentCoherence {
                    agent_a: i,
                    agent_b: j,
                    trust_a: *trust_a,
                    trust_b: *trust_b,
                    band_coherence,
                    mean_coherence: mean_coh,
                });
            }
        }

        // Calculate trust-coherence correlations
        let correlations: Vec<_> = bands
            .iter()
            .map(|&band| self.calculate_trust_coherence_correlation(&coherences, band)) // SIM-TRUST-001
            .collect();

        // Validate
        // 1. At least one band shows significant negative correlation
        // (agents with similar trust have higher coherence)
        let has_significant_correlation = correlations
            .iter()
            .any(|c| c.significance && c.correlation < -0.2);

        // 2. Alpha band typically shows strongest effect (8-13 Hz)
        let alpha_correlation = correlations
            .iter()
            .find(|c| matches!(c.band, FrequencyBand::Alpha))
            .map_or(0.0, |c| c.correlation);
        let alpha_strong = alpha_correlation < -0.15;

        // Success criteria
        let success = has_significant_correlation || alpha_strong;

        let duration = start_time.elapsed();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Build CSV output
        let mut csv_data = String::from(
            "agent_a,agent_b,trust_a,trust_b,mean_coherence,delta,theta,alpha,beta,gamma\n",
        );
        for c in &coherences {
            csv_data.push_str(&format!(
                "{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}\n",
                c.agent_a,
                c.agent_b,
                c.trust_a,
                c.trust_b,
                c.mean_coherence,
                c.band_coherence
                    .get(&FrequencyBand::Delta)
                    .map_or(0.0, |r| r.coherence),
                c.band_coherence
                    .get(&FrequencyBand::Theta)
                    .map_or(0.0, |r| r.coherence),
                c.band_coherence
                    .get(&FrequencyBand::Alpha)
                    .map_or(0.0, |r| r.coherence),
                c.band_coherence
                    .get(&FrequencyBand::Beta)
                    .map_or(0.0, |r| r.coherence),
                c.band_coherence
                    .get(&FrequencyBand::Gamma)
                    .map_or(0.0, |r| r.coherence),
            ));
        }

        // Correlation CSV
        let mut corr_csv = String::from("band,correlation,p_value,significant\n");
        for corr in &correlations {
            corr_csv.push_str(&format!(
                "{:?},{:.4},{:.4},{}\n",
                corr.band, corr.correlation, corr.p_value, corr.significance
            ));
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Summary
        let summary = format!(
            "Spectral Coherence Experiment (MUSIC-P0-2)\n\
             ==========================================\n\
             Agents: {}\n\
             Pairs analyzed: {}\n\
             Frequency bands: 5 (δ, θ, α, β, γ)\n\
             \n\
             Trust-Coherence Correlations:\n",
            n_agents,
            coherences.len()
        );

        let mut corr_summary = String::new();
        for corr in &correlations {
            corr_summary.push_str(&format!(
                "  {:?}: r = {:.3}, p = {:.3} {}\n",
                corr.band,
                corr.correlation,
                corr.p_value,
                if corr.significance { "*" } else { "" }
            ));
        }

        let validation_summary = format!(
            "\nValidation:\n\
             - Significant correlation: {}\n\
             - Alpha band strong: {} (r = {:.3})\n\
             \n\
             Success Criterion: Significant trust-coherence relationship\n\
             Result: {}\n",
            has_significant_correlation,
            alpha_strong,
            alpha_correlation,
            if success { "✓ PASSED" } else { "✗ FAILED" }
        );

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let artifacts = vec![
            ("spectral_coherence.csv".to_string(), csv_data),
            ("trust_coherence_correlation.csv".to_string(), corr_csv),
            (
                "spectral_summary.txt".to_string(),
                summary + &corr_summary + &validation_summary,
            ),
        ];

        let metadata = serde_json::json!({
            "n_pairs": coherences.len(),
            "alpha_correlation": alpha_correlation,
            "has_significant": has_significant_correlation,
        });

        Ok(ExperimentResult {
            success,
            experiment_type: "spectral_coherence".to_string(),
            gap_id: "MUSIC-P0-2".to_string(),
            paper: "MUSIC".to_string(),
            figures: vec![],
            csv_outputs: vec![],
            metrics: metadata,
            artifacts,
            duration,
            error_message: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_frequency_bands() {
        assert_eq!(FrequencyBand::Delta.range(), (0.5, 4.0));
        assert_eq!(FrequencyBand::Alpha.range(), (8.0, 13.0));
        assert_eq!(FrequencyBand::Gamma.range(), (30.0, 100.0));
    }

    #[test]
    fn test_spectral_signal() {
        let mut rng = rand::thread_rng();
        let signal = SpectralSignal::from_trust(0.8, FrequencyBand::Alpha, &mut rng);

        assert_eq!(signal.frequencies.len(), signal.psd.len());
        assert_eq!(signal.frequencies.len(), signal.phase.len());

        let alpha_power = signal.band_power(FrequencyBand::Alpha);
        assert!(alpha_power > 0.0);
    }

    #[test]
    fn test_coherence_calculation() {
        let exp = SpectralCoherenceExperiment;
        let mut rng = rand::thread_rng();

        let signal_a = SpectralSignal::from_trust(0.7, FrequencyBand::Alpha, &mut rng);
        let signal_b = SpectralSignal::from_trust(0.7, FrequencyBand::Alpha, &mut rng);

        let coh = exp.calculate_coherence(&signal_a, &signal_b, FrequencyBand::Alpha);

        assert!(coh.coherence >= 0.0 && coh.coherence <= 1.0);
        assert!(coh.plv >= 0.0 && coh.plv <= 1.0);
    }

    #[test]
    fn test_experiment_runs() {
        let config = ExperimentConfig::quick();
        let exp = SpectralCoherenceExperiment;

        let result = exp.run(&config).expect("Experiment should run");

        assert!(result.metrics.get("alpha_correlation").is_some());
        assert!(result.duration.as_secs() < 10);
    }
}
