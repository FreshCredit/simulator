//! Wet-Lab & Wearable IoT Biomarker Engine
//!
//! Grounded in:
//! - OSF Pre-registration `DOCAS-GATE-1.md` (Pharmacological & physiological readouts)
//! - `docas.md` §2.8 Table 1 (Five concurrent neuromodulatory timescales)
//! - `whitepaper.md` §3.1–3.4 (Dual-layer measurement & Sequestration Wall)
//! - `packages/native-core/src/capabilities/health.rs` (Apple HealthKit & Health Connect normalization)
//!
//! Models:
//! 1. Apple Watch / HealthKit: HRV (RMSSD, SDNN), Resting Heart Rate, Respiratory Rate, Sleep Architecture
//! 2. InteraXon Muse S / Muse 2 4-Channel EEG (AF7, AF8, TP9, TP10): Frontal Midline Theta, Frontal Alpha Asymmetry (FAA), Beta/Alpha Arousal Ratio, RewP ERP
//! 3. Dexcom Stelo Continuous Glucose Biosensor (CGM): 15-min interstitial glucose, Time-in-Range (TIR), Glycemic CV%, Postprandial & Cortisol Gluconeogenesis

use crate::circadian::CircadianSystem;
use crate::docas::DocasState;
use crate::types::{A, C, D, O, S};
use serde::{Deserialize, Serialize};

/// Comprehensive agent biometric and wearable sensor state
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AgentBiomarkers {
    // === Apple Watch / HealthKit Cardiovascular & Autonomic ===
    /// HRV Root Mean Square of Successive Differences in ms (vagal tone)
    pub hrv_rmssd_ms: f32,
    /// HRV Standard Deviation of NN intervals in ms (SDNN ≈ 0.8 * RMSSD)
    pub hrv_sdnn_ms: f32,
    /// Resting heart rate in beats per minute (bpm). Range: [48.0, 98.0]
    pub resting_hr_bpm: f32,
    /// Respiratory rate in breaths per minute (brpm). Range: [11.0, 22.0]
    pub respiratory_rate_brpm: f32,

    // === Apple Watch / HealthKit Sleep Architecture & Circadian ===
    /// Sleep efficiency percentage [50.0, 98.0]%. Healthy: >85%
    pub sleep_efficiency_pct: f32,
    /// Deep sleep (Slow-Wave Sleep / N3) percentage of night [10.0, 30.0]%
    pub sleep_deep_pct: f32,
    /// Rapid Eye Movement (REM) sleep percentage [15.0, 30.0]%
    pub sleep_rem_pct: f32,
    /// Core (Light / N1-N2) sleep percentage [40.0, 60.0]%
    pub sleep_core_pct: f32,
    /// Nighttime wakefulness / WASO percentage [2.0, 25.0]%
    pub sleep_awake_pct: f32,
    /// Current SCN circadian phase in radians [0, 2π). 0 = 6 AM, π = 6 PM
    pub circadian_phase_rad: f32,

    // === InteraXon Muse 4-Channel EEG (AF7, AF8, TP9, TP10) ===
    /// Frontal Midline Theta (4–8 Hz) power density in μV²/Hz (AF7/AF8)
    pub eeg_frontal_theta_uv2: f32,
    /// Frontal Alpha Asymmetry (FAA = ln(P_alpha_AF8) - ln(P_alpha_AF7))
    pub eeg_alpha_asymmetry: f32,
    /// Beta/Alpha arousal ratio (13–30 Hz / 8–13 Hz)
    pub eeg_beta_arousal_ratio: f32,
    /// Event-Related Potential Reward Positivity (RewP) in μV
    pub eeg_rewp_erp_uv: f32,

    // === Dexcom Stelo Continuous Glucose Biosensor (CGM) ===
    /// Interstitial continuous glucose level in mg/dL. Euglycemia: [70.0, 140.0]
    pub glucose_mg_dl: f32,
    /// Time-in-Range (70–140 mg/dL) percentage over rolling window
    pub glucose_tir_pct: f32,
    /// Glycemic Coefficient of Variation (CV = σ / μ). Stability: ≤36%
    pub glucose_cv_pct: f32,
    /// Filtered meal digestion absorption buffer
    pub glucose_absorption_buffer: f32,

    // === Apple Watch S8 Nocturnal Thermometry ===
    /// Nocturnal sleeping wrist temperature in °C. Baseline: 36.40°C, range: [35.5, 37.8]
    pub sleeping_wrist_temp_c: f32,

    // === Muse S Athena Prefrontal fNIRS (mPFC / BA 10/32) ===
    /// Prefrontal medial cortical oxygenated hemoglobin shift Δ[HbO2] in μmol/L. Baseline: 2.50
    pub fnirs_prefrontal_hbo2_umol: f32,
    /// Prefrontal medial cortical deoxygenated hemoglobin shift Δ[HbR] in μmol/L. Baseline: -0.50
    pub fnirs_prefrontal_hbr_umol: f32,
}

impl Default for AgentBiomarkers {
    fn default() -> Self {
        Self {
            hrv_rmssd_ms: 48.0,
            hrv_sdnn_ms: 52.0,
            resting_hr_bpm: 64.0,
            respiratory_rate_brpm: 14.5,
            sleep_efficiency_pct: 86.0,
            sleep_deep_pct: 20.0,
            sleep_rem_pct: 22.0,
            sleep_core_pct: 46.0,
            sleep_awake_pct: 12.0,
            circadian_phase_rad: 0.0,
            eeg_frontal_theta_uv2: 6.2,
            eeg_alpha_asymmetry: 0.05,
            eeg_beta_arousal_ratio: 0.85,
            eeg_rewp_erp_uv: 3.5,
            glucose_mg_dl: 92.0,
            glucose_tir_pct: 88.0,
            glucose_cv_pct: 19.0,
            glucose_absorption_buffer: 0.0,
            sleeping_wrist_temp_c: 36.40,
            fnirs_prefrontal_hbo2_umol: 2.50,
            fnirs_prefrontal_hbr_umol: -0.50,
        }
    }
}

impl AgentBiomarkers {
    /// Initialize individualized baseline biomarkers based on genome traits and sex
    #[must_use]
    pub fn new_individualized(genome_traits: &[f32; 14], sex: crate::types::Sex) -> Self {
        let mut b = Self::default();
        let stability_trait = genome_traits[4].clamp(0.0, 1.0);
        let stress_trait = genome_traits[2].clamp(0.0, 1.0);

        b.hrv_rmssd_ms = (32.0 + stability_trait * 35.0 - stress_trait * 15.0).clamp(20.0, 95.0);
        b.hrv_sdnn_ms = b.hrv_rmssd_ms * 1.1;

        b.resting_hr_bpm = (58.0 + stress_trait * 18.0 - stability_trait * 8.0
            + if sex == crate::types::Sex::Female {
                2.5
            } else {
                0.0
            })
        .clamp(50.0, 92.0);

        b.glucose_mg_dl = (86.0 + stress_trait * 12.0).clamp(78.0, 108.0);
        b.eeg_frontal_theta_uv2 = (4.5 + genome_traits[10] * 4.0).clamp(3.0, 10.0);
        b
    }

    /// Update biomarker dynamics each tick based on DOCAS state, circadian clock, and recent foraging
    pub fn update(
        &mut self,
        docas: &DocasState,
        circadian: &CircadianSystem,
        harvested_resource: f32,
        rpe_error: f32,
        cognitive_bandwidth: f32,
    ) {
        let d = docas.levels[D];
        let o = docas.levels[O];
        let c = docas.levels[C];
        let a = docas.levels[A];
        let s = docas.levels[S];

        self.circadian_phase_rad = circadian.phase;

        // 1. Apple Watch Autonomic Response (HRV & Resting HR)
        let autonomic_balance = (s * 0.45 + o * 0.35) - (c * 0.50 + a * 0.40);
        let target_rmssd = (48.0 + autonomic_balance * 32.0).clamp(16.0, 105.0);
        self.hrv_rmssd_ms += (target_rmssd - self.hrv_rmssd_ms) * 0.08;
        self.hrv_sdnn_ms = (self.hrv_rmssd_ms * 0.85).clamp(18.0, 115.0);

        let target_hr = (64.0 + (a * 18.0 + c * 10.0) - (s * 6.0 + o * 4.0)).clamp(48.0, 98.0);
        self.resting_hr_bpm += (target_hr - self.resting_hr_bpm) * 0.08;

        self.respiratory_rate_brpm = (13.5 + a * 5.0 + c * 2.0).clamp(11.0, 22.0);

        // 2. Sleep Architecture Modulated by Melatonin & Daytime Cortisol (Simplex Partition Conservation)
        let sleep_suppression = (c - 0.4).max(0.0) * 25.0;
        self.sleep_efficiency_pct = (88.0 - sleep_suppression + s * 5.0).clamp(52.0, 97.0);
        let raw_deep = (22.0 - sleep_suppression * 0.4).clamp(8.0, 28.0);
        let raw_rem = (22.0 + (1.0 - c) * 4.0).clamp(12.0, 26.0);
        let raw_awake = 100.0 - self.sleep_efficiency_pct;
        let raw_core = (100.0 - raw_deep - raw_rem - raw_awake).max(10.0);
        let sum = raw_deep + raw_rem + raw_core + raw_awake;
        let norm = 100.0 / sum;
        self.sleep_deep_pct = raw_deep * norm;
        self.sleep_rem_pct = raw_rem * norm;
        self.sleep_core_pct = raw_core * norm;
        self.sleep_awake_pct = raw_awake * norm;

        // 3. InteraXon Muse 4-Channel EEG Forward Modeling
        let target_theta =
            (4.0 + rpe_error.abs() * 7.0 + (1.0 - cognitive_bandwidth) * 3.5).clamp(2.5, 14.5);
        self.eeg_frontal_theta_uv2 += (target_theta - self.eeg_frontal_theta_uv2) * 0.12;

        let target_faa = ((d - c) * 0.55).clamp(-0.55, 0.55);
        self.eeg_alpha_asymmetry += (target_faa - self.eeg_alpha_asymmetry) * 0.10;

        let target_arousal = (0.60 + a * 1.30).clamp(0.35, 2.40);
        self.eeg_beta_arousal_ratio += (target_arousal - self.eeg_beta_arousal_ratio) * 0.10;

        let target_rewp = (rpe_error * 6.0).clamp(-4.0, 7.5);
        self.eeg_rewp_erp_uv += (target_rewp - self.eeg_rewp_erp_uv) * 0.25;

        // 4. Dexcom Stelo Interstitial Glucose Biosensor (CGM)
        if harvested_resource > 0.0 {
            self.glucose_absorption_buffer += harvested_resource * 6.0;
        }

        let absorption_rate = self.glucose_absorption_buffer * 0.15;
        self.glucose_absorption_buffer -= absorption_rate;

        let gluconeogenesis_bump = (c - 0.3).max(0.0) * 22.0;
        let target_glucose = 88.0 + absorption_rate * 2.5 + gluconeogenesis_bump;
        let delta_glucose = (target_glucose - self.glucose_mg_dl).abs();
        self.glucose_mg_dl += (target_glucose - self.glucose_mg_dl) * 0.06;
        self.glucose_mg_dl = self.glucose_mg_dl.clamp(65.0, 195.0);

        let in_range = self.glucose_mg_dl >= 70.0 && self.glucose_mg_dl <= 140.0;
        let tir_delta = if in_range { 1.0 } else { 0.0 };
        self.glucose_tir_pct =
            (self.glucose_tir_pct * 0.95 + tir_delta * 100.0 * 0.05).clamp(40.0, 100.0);

        // International Consensus on CGM (Danne et al. 2017): CV% = 100 * (std_dev / mean)
        let glycemic_swing = (delta_glucose / self.glucose_mg_dl.max(1.0)) * 100.0 * 3.5;
        self.glucose_cv_pct =
            (self.glucose_cv_pct * 0.95 + (12.0 + glycemic_swing) * 0.05).clamp(10.0, 42.0);

        // 5. Apple Watch S8 Nocturnal Thermometry (Allostatic Stress Drift & Circadian Rhythm)
        let circadian_thermal_dip = (self.circadian_phase_rad - std::f32::consts::PI).cos() * 0.15;
        let stress_thermal_drift = (c - 0.35).max(0.0) * 0.50;
        let target_temp = (36.40 + circadian_thermal_dip + stress_thermal_drift).clamp(35.5, 37.8);
        self.sleeping_wrist_temp_c += (target_temp - self.sleeping_wrist_temp_c) * 0.05;

        // 6. Muse S Athena Prefrontal fNIRS Forward Hemodynamics
        let task_activation = (d * 1.8 + a * 0.8 - c * 0.5 + rpe_error * 1.2).clamp(-1.0, 4.0);
        let target_hbo2 = (2.50 + task_activation).clamp(0.5, 6.5);
        let target_hbr = (-0.50 - task_activation * 0.35).clamp(-2.5, 0.5);
        self.fnirs_prefrontal_hbo2_umol += (target_hbo2 - self.fnirs_prefrontal_hbo2_umol) * 0.10;
        self.fnirs_prefrontal_hbr_umol += (target_hbr - self.fnirs_prefrontal_hbr_umol) * 0.10;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_biomarkers_default_and_docas_forward_dynamics() {
        let mut bio = AgentBiomarkers::default();
        assert_eq!(bio.sleeping_wrist_temp_c, 36.40);
        assert_eq!(bio.fnirs_prefrontal_hbo2_umol, 2.50);
        assert_eq!(bio.fnirs_prefrontal_hbr_umol, -0.50);

        let mut docas = DocasState::default();
        let circadian = CircadianSystem::default();

        // High stress state (elevated C-channel)
        docas.levels[C] = 0.90;
        docas.levels[D] = 0.10;
        bio.update(&docas, &circadian, 0.0, -0.5, 0.5);

        assert!(bio.sleeping_wrist_temp_c > 36.40);
        assert!(bio.hrv_rmssd_ms < 48.0);
    }
}
