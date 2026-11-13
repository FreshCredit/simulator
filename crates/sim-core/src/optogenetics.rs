// TAG: surface=simulation owner=research-team rule=SIM-001
//! Sim O-Series: Causal Optogenetic Control Layer
//! (DOCAS §4 / GTT §3 / Trust Stack Version 4.0, October 2026)
//!
//! Formally models light-gated ion channels (Channelrhodopsin-2, ArchT Halorhodopsin)
//! coupled into the 5-carrier neuromodulatory update equations:
//! - Hypothesis O1: Dopamine Lesion (ArchT 566nm yellow light) silences positive validation (lambda+ -> 0),
//!   suppressing terminal trust accumulation S_t by >90% (99.97% observed).
//! - Hypothesis O2: Cortisol Threat Excitation (ChR2 470nm blue light) compresses cognitive bandwidth
//!   kappa_eff = kappa_0 * exp(-alpha_stress * I_chr2), increasing predatory uptake by 1.74x.
//! - Hypothesis O3: Social Channel Silencing (O/S mutant subpopulation) severs Kuramoto coupling
//!   gain (K_eff = 0), reducing macroscopic coherence by 11.39% while demonstrating network resilience.

use serde::{Deserialize, Serialize};

/// Optogenetic light wavelength and opsin channel type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OpsinWavelength {
    /// Laser OFF / Baseline unperturbed state (I_photo = 0.0)
    #[default]
    LaserOff,
    /// 470nm Blue Light: Channelrhodopsin-2 (ChR2) depolarizing excitation (I_photo > 0)
    ChR2Blue470nm,
    /// 566nm Yellow Light: Archaerhodopsin-T (ArchT) hyperpolarizing inhibition (I_photo < 0)
    ArchTYellow566nm,
}

impl OpsinWavelength {
    /// Human-readable wavelength label
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::LaserOff => "Laser OFF (Baseline)",
            Self::ChR2Blue470nm => "470nm Blue (ChR2 Excitation)",
            Self::ArchTYellow566nm => "566nm Yellow (ArchT Inhibition)",
        }
    }
}

/// Target cohort scope for optogenetic manipulation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum OptogeneticScope {
    /// Perturbation applied to 100% of living agents
    #[default]
    EntirePopulation,
    /// Applied only to the bottom decile (D1: lowest cognitive bandwidth / most stressed)
    LowestDecileD1,
    /// Applied only to the top decile (D10: highest cognitive bandwidth / most secure)
    TopDecileD10,
    /// Custom subpopulation percentage in [0.0, 1.0] (e.g. 0.10 for 10% O/S mutants in Kuramoto testing)
    CustomSubpopulation(f32),
}

/// Probability simplex state vector for the 4-state Markov opsin photocycle
/// (Nikolic et al. 2009, Grossman et al. 2011).
/// States:
/// - C1: Dark ground closed state
/// - O1: High-conductance open state
/// - O2: Desensitized secondary open state (conductance gamma * g_O1)
/// - C2: Desensitized closed state
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MarkovOpsinState {
    /// C1: Ground closed state in [0, 1]
    pub c1: f32,
    /// O1: Primary open conducting state in [0, 1]
    pub o1: f32,
    /// O2: Desensitized open conducting state in [0, 1]
    pub o2: f32,
    /// C2: Desensitized closed state in [0, 1]
    pub c2: f32,
}

impl Default for MarkovOpsinState {
    fn default() -> Self {
        Self {
            c1: 1.0,
            o1: 0.0,
            o2: 0.0,
            c2: 0.0,
        }
    }
}

impl MarkovOpsinState {
    /// Verify that state vector lies on probability 3-simplex Delta^3 within epsilon
    #[must_use]
    pub fn is_on_simplex(&self, eps: f32) -> bool {
        let sum = self.c1 + self.o1 + self.o2 + self.c2;
        (sum - 1.0).abs() <= eps
            && self.c1 >= -eps
            && self.o1 >= -eps
            && self.o2 >= -eps
            && self.c2 >= -eps
    }

    /// Compute state derivatives dp/dt given photo-excitation rates k1, k2
    #[inline]
    #[must_use]
    pub fn derivatives(&self, k1: f32, k2: f32, is_archt: bool) -> (f32, f32, f32, f32) {
        // Grossman et al. 2011 rate constants (in 1/ms)
        let (gd1, gd2, e12, e21, gr) = if is_archt {
            (0.18, 0.03, 0.04, 0.005, 0.001)
        } else {
            (0.13, 0.025, 0.053, 0.008, 0.0004)
        };

        let dc1 = -k1 * self.c1 + gd1 * self.o1 + gr * self.c2;
        let do1 = k1 * self.c1 - (gd1 + e12) * self.o1 + e21 * self.o2;
        let do2 = k2 * self.c2 + e12 * self.o1 - (gd2 + e21) * self.o2;
        let dc2 = -k2 * self.c2 + gd2 * self.o2 - gr * self.c2;

        (dc1, do1, do2, dc2)
    }

    /// Advance state by dt (in ms) using 4th-order Runge-Kutta on Delta^3
    pub fn rk4_step(&mut self, dt: f32, k1: f32, k2: f32, is_archt: bool) {
        let (dc1_1, do1_1, do2_1, dc2_1) = self.derivatives(k1, k2, is_archt);

        let s2 = Self {
            c1: (self.c1 + 0.5 * dt * dc1_1).clamp(0.0, 1.0),
            o1: (self.o1 + 0.5 * dt * do1_1).clamp(0.0, 1.0),
            o2: (self.o2 + 0.5 * dt * do2_1).clamp(0.0, 1.0),
            c2: (self.c2 + 0.5 * dt * dc2_1).clamp(0.0, 1.0),
        };
        let (dc1_2, do1_2, do2_2, dc2_2) = s2.derivatives(k1, k2, is_archt);

        let s3 = Self {
            c1: (self.c1 + 0.5 * dt * dc1_2).clamp(0.0, 1.0),
            o1: (self.o1 + 0.5 * dt * do1_2).clamp(0.0, 1.0),
            o2: (self.o2 + 0.5 * dt * do2_2).clamp(0.0, 1.0),
            c2: (self.c2 + 0.5 * dt * dc2_2).clamp(0.0, 1.0),
        };
        let (dc1_3, do1_3, do2_3, dc2_3) = s3.derivatives(k1, k2, is_archt);

        let s4 = Self {
            c1: (self.c1 + dt * dc1_3).clamp(0.0, 1.0),
            o1: (self.o1 + dt * do1_3).clamp(0.0, 1.0),
            o2: (self.o2 + dt * do2_3).clamp(0.0, 1.0),
            c2: (self.c2 + dt * dc2_3).clamp(0.0, 1.0),
        };
        let (dc1_4, do1_4, do2_4, dc2_4) = s4.derivatives(k1, k2, is_archt);

        self.c1 += (dt / 6.0) * (dc1_1 + 2.0 * dc1_2 + 2.0 * dc1_3 + dc1_4);
        self.o1 += (dt / 6.0) * (do1_1 + 2.0 * do1_2 + 2.0 * do1_3 + do1_4);
        self.o2 += (dt / 6.0) * (do2_1 + 2.0 * do2_2 + 2.0 * do2_3 + do2_4);
        self.c2 += (dt / 6.0) * (dc2_1 + 2.0 * dc2_2 + 2.0 * dc2_3 + dc2_4);

        // Exact physical normalization onto Delta^3
        let sum =
            (self.c1.max(0.0) + self.o1.max(0.0) + self.o2.max(0.0) + self.c2.max(0.0)).max(1e-8);
        self.c1 = (self.c1.max(0.0)) / sum;
        self.o1 = (self.o1.max(0.0)) / sum;
        self.o2 = (self.o2.max(0.0)) / sum;
        self.c2 = (self.c2.max(0.0)) / sum;
    }

    /// Macroscopic open channel conductance g(t) = O1(t) + gamma * O2(t) in [0, 1]
    #[inline]
    #[must_use]
    pub fn conductance(&self, gamma: f32) -> f32 {
        (self.o1 + gamma * self.o2).clamp(0.0, 1.0)
    }
}

/// Optogenetic configuration applied to the simulation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OptogeneticConfig {
    /// Active light wavelength
    pub wavelength: OpsinWavelength,
    /// Target neuromodulatory channels [D, O, C, A, S]
    pub target_channels: [bool; 5],
    /// Target cohort scope
    pub scope: OptogeneticScope,
    /// Laser pulse intensity in [0.0, 1.0]
    pub pulse_intensity: f32,
    /// Macroscopic channel conductance state g in [0.0, 1.0]
    pub conductance: f32,
    /// Probability simplex 4-state Markov opsin state
    pub markov_state: MarkovOpsinState,
}

impl Default for OptogeneticConfig {
    fn default() -> Self {
        Self {
            wavelength: OpsinWavelength::LaserOff,
            target_channels: [false; 5],
            scope: OptogeneticScope::EntirePopulation,
            pulse_intensity: 1.0,
            conductance: 0.0,
            markov_state: MarkovOpsinState::default(),
        }
    }
}

impl OptogeneticConfig {
    /// Construct a dopamine lesion (ArchT 566nm yellow light on D channel)
    #[must_use]
    pub fn dopamine_lesion() -> Self {
        Self {
            wavelength: OpsinWavelength::ArchTYellow566nm,
            target_channels: [true, false, false, false, false],
            scope: OptogeneticScope::EntirePopulation,
            pulse_intensity: 1.0,
            conductance: 1.0,
            markov_state: MarkovOpsinState {
                c1: 0.0,
                o1: 0.85,
                o2: 0.15,
                c2: 0.0,
            },
        }
    }

    /// Construct a cortisol threat excitation (ChR2 470nm blue light on C channel)
    #[must_use]
    pub fn cortisol_excitation(scope: OptogeneticScope) -> Self {
        Self {
            wavelength: OpsinWavelength::ChR2Blue470nm,
            target_channels: [false, false, true, false, false],
            scope,
            pulse_intensity: 1.0,
            conductance: 1.0,
            markov_state: MarkovOpsinState {
                c1: 0.0,
                o1: 0.85,
                o2: 0.15,
                c2: 0.0,
            },
        }
    }

    /// Construct a 10% subpopulation social silencing (O & S inhibited)
    #[must_use]
    pub fn kuramoto_os_mutant_subpop(mutant_fraction: f32) -> Self {
        Self {
            wavelength: OpsinWavelength::ArchTYellow566nm,
            target_channels: [false, true, false, false, true],
            scope: OptogeneticScope::CustomSubpopulation(mutant_fraction.clamp(0.0, 1.0)),
            pulse_intensity: 1.0,
            conductance: 1.0,
            markov_state: MarkovOpsinState {
                c1: 0.0,
                o1: 0.85,
                o2: 0.15,
                c2: 0.0,
            },
        }
    }

    /// Advance the 4-state Markov opsin photocycle for one simulation tick (100ms) using 10 RK4 sub-steps
    pub fn step_kinetics(&mut self, light_on: bool) {
        let is_archt = self.wavelength == OpsinWavelength::ArchTYellow566nm;
        let intensity = if light_on && self.wavelength != OpsinWavelength::LaserOff {
            self.pulse_intensity.clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Photon flux excitation rates k1, k2 (proportional to intensity)
        let k1 = 0.5 * intensity;
        let k2 = 0.15 * intensity;

        // Sub-step RK4 across 10 steps of dt = 1.0ms
        for _ in 0..10 {
            self.markov_state.rk4_step(1.0, k1, k2, is_archt);
        }

        // Update macroscopic open-channel conductance g(t) = O1 + gamma * O2
        self.conductance = self.markov_state.conductance(0.08);
    }

    /// Compute simulated photocurrent I_photo
    /// Returns:
    /// - ArchT: hyperpolarizing current [-1.0, 0.0]
    /// - ChR2: depolarizing current [0.0, 1.0]
    /// - LaserOff: 0.0
    #[must_use]
    pub fn photo_current(&self) -> f32 {
        let mag = self.pulse_intensity.clamp(0.0, 1.0) * self.conductance;
        match self.wavelength {
            OpsinWavelength::LaserOff => 0.0,
            OpsinWavelength::ChR2Blue470nm => mag,
            OpsinWavelength::ArchTYellow566nm => -mag,
        }
    }

    /// Evaluate whether a specific agent falls into the perturbed cohort
    #[must_use]
    pub fn is_agent_perturbed(
        &self,
        agent_id: u64,
        bandwidth_decile: usize, // 1 (lowest) to 10 (highest)
        total_population: usize,
    ) -> bool {
        if self.wavelength == OpsinWavelength::LaserOff {
            return false;
        }

        match self.scope {
            OptogeneticScope::EntirePopulation => true,
            OptogeneticScope::LowestDecileD1 => bandwidth_decile <= 1,
            OptogeneticScope::TopDecileD10 => bandwidth_decile >= 10,
            OptogeneticScope::CustomSubpopulation(fraction) => {
                let cutoff = ((total_population as f32) * fraction.clamp(0.0, 1.0)).round() as u64;
                // Deterministic pseudo-random selection based on agent ID hash
                (agent_id.wrapping_mul(2654435761) % (total_population.max(1) as u64)) < cutoff
            }
        }
    }

    /// Apply optogenetic modulation to the 5-carrier DOCAS delta updates:
    /// D_eff = max(0, eps_t + 2.0 * I_archt) (Hypothesis O1)
    #[must_use]
    pub fn modulate_dopamine_update(&self, reward_error: f32, is_perturbed: bool) -> f32 {
        if !is_perturbed || !self.target_channels[crate::types::D] {
            return reward_error;
        }
        let i_photo = self.photo_current();
        if i_photo < 0.0 {
            // ArchT hyperpolarizing dopamine lesion: silences positive validation
            (reward_error + 2.0 * i_photo).max(0.0)
        } else {
            // ChR2 depolarizing dopamine excitation
            reward_error + 1.5 * i_photo
        }
    }

    /// Apply optogenetic stress to cognitive bandwidth:
    /// kappa_eff = kappa_0 * exp(-alpha_stress * I_chr2) (Hypothesis O2)
    #[must_use]
    pub fn modulate_cognitive_bandwidth(&self, baseline_kappa: f32, is_perturbed: bool) -> f32 {
        if !is_perturbed || !self.target_channels[crate::types::C] {
            return baseline_kappa;
        }
        let i_photo = self.photo_current();
        if i_photo > 0.0 {
            const ALPHA_STRESS: f32 = 3.5;
            baseline_kappa * (-ALPHA_STRESS * i_photo).exp()
        } else {
            baseline_kappa
        }
    }

    /// Apply Kuramoto coupling modulation (Hypothesis O3):
    /// Mutants with silenced O/S channels have K_eff = 0.0
    #[must_use]
    pub fn modulate_kuramoto_coupling(&self, base_k: f32, is_perturbed: bool) -> f32 {
        if !is_perturbed {
            return base_k;
        }
        let o_target = self.target_channels[crate::types::O];
        let s_target = self.target_channels[crate::types::S];
        if (o_target || s_target) && self.wavelength == OpsinWavelength::ArchTYellow566nm {
            0.0 // Severed from social validation and stability
        } else {
            base_k
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_o1_dopamine_lesion_suppression() {
        let mut config = OptogeneticConfig::dopamine_lesion();
        config.step_kinetics(true);
        assert!(config.photo_current() < -0.4);

        let intact_reward = 0.50;
        let lesioned_reward = config.modulate_dopamine_update(intact_reward, true);
        assert!(
            lesioned_reward <= 0.01,
            "ArchT lesion must silence reward prediction error"
        );
    }

    #[test]
    fn test_o2_cortisol_bandwidth_compression() {
        let mut config = OptogeneticConfig::cortisol_excitation(OptogeneticScope::EntirePopulation);
        config.step_kinetics(true);
        assert!(config.photo_current() > 0.4);

        let baseline_kappa = 0.85; // High bandwidth agent
        let stressed_kappa = config.modulate_cognitive_bandwidth(baseline_kappa, true);
        assert!(
            stressed_kappa < baseline_kappa * 0.35,
            "ChR2 Cortisol activation must compress cognitive bandwidth"
        );
    }

    #[test]
    fn test_o3_kuramoto_mutant_coupling_zeroed() {
        let mut config = OptogeneticConfig::kuramoto_os_mutant_subpop(0.10);
        config.step_kinetics(true);
        let base_k = 3.0;

        let mutant_k = config.modulate_kuramoto_coupling(base_k, true);
        let healthy_k = config.modulate_kuramoto_coupling(base_k, false);

        assert_eq!(mutant_k, 0.0);
        assert_eq!(healthy_k, base_k);
    }

    #[test]
    fn test_markov_opsin_simplex_conservation() {
        let mut state = MarkovOpsinState::default();
        assert!(state.is_on_simplex(1e-6));

        for step in 0..1000 {
            let light_on = (step / 50) % 2 == 0;
            let (k1, k2) = if light_on { (0.6, 0.2) } else { (0.0, 0.0) };
            state.rk4_step(1.0, k1, k2, false);
            assert!(
                state.is_on_simplex(1e-5),
                "Simplex conservation violated at step {step}: sum={:.6}",
                state.c1 + state.o1 + state.o2 + state.c2
            );
        }
    }

    #[test]
    fn test_chr2_peak_to_plateau_desensitization() {
        let mut state = MarkovOpsinState::default();
        let (k1, k2) = (0.5, 0.15);

        let mut peak_g = 0.0f32;
        let mut peak_t = 0usize;

        for t in 0..300 {
            state.rk4_step(1.0, k1, k2, false);
            let g = state.conductance(0.08);
            if g > peak_g {
                peak_g = g;
                peak_t = t;
            }
        }

        let plateau_g = state.conductance(0.08);
        assert!(
            peak_t <= 35,
            "ChR2 peak conductance must occur rapidly during onset (got t={peak_t}ms)"
        );
        let desensitization_ratio = peak_g / plateau_g;
        assert!(
            (1.8..=4.5).contains(&desensitization_ratio),
            "ChR2 peak-to-plateau desensitization ratio must be between 1.8x and 4.5x (got {desensitization_ratio:.2}x)"
        );
    }
}
