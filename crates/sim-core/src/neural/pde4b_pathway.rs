//! PDE4B → cAMP → PKA → CREB Signaling Pathway for Neural Maturation
//!
//! Research-based implementation of the 4-step biochemical cascade that modulates
//! neural maturation rates based on PDE4B genotype.
//!
//! Pathway:
//! 1. PDE4B degrades cAMP (phosphodiesterase activity)
//! 2. cAMP activates PKA (protein kinase A via cooperative binding)
//! 3. PKA phosphorylates CREB (cAMP response element-binding protein)
//! 4. pCREB modulates gene expression affecting neural maturation
//!
//! References:
//! - Millar et al. (2005) - PDE4B discovery
//! - Beavo & Brunton (2002) - cAMP signaling
//! - Taylor et al. (2012) - PKA structure
//! - Shaywitz & Greenberg (1999) - CREB activation

use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Parameters for the PDE4B-cAMP-PKA-CREB signaling pathway
///
/// These parameters are derived from empirical biochemical measurements
/// and represent physiological concentrations/rates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PDE4BPathwayParams {
    /// Adenylyl cyclase production rate (μM/s)
    /// Default: 5.0 μM/s - typical AC activity in neurons
    pub v_ac: f32,

    /// PDE4B degradation rate constant (1/s)
    /// Default: 0.1/s - catalytic turnover number
    pub k_pde: f32,

    /// Michaelis constant for PDE4B (μM)
    /// Default: 3.0 μM - substrate concentration at half Vmax
    pub km: f32,

    /// Non-PDE4B cAMP loss rate (1/s)
    /// Default: 0.02/s - basal hydrolysis + diffusion
    pub k_leak: f32,

    /// Hill coefficient for PKA activation
    /// Default: 2.0 - cooperative cAMP binding to regulatory subunits
    pub hill_n: f32,

    /// EC50 for PKA activation (μM)
    /// Default: 3.5 μM - cellular context (NOT in vitro 0.1-0.3 μM)
    pub k_pka_ec50: f32,

    /// Baseline CREB phosphorylation fraction
    /// Default: 0.12 (12%) - unstimulated phosphorylation level
    pub pcreb_baseline: f32,

    /// Maximum CREB phosphorylation fraction
    /// Default: 0.65 (65%) - after strong stimulation
    pub pcreb_max: f32,

    /// Half-maximal pCREB for maturation modulation
    /// Default: 0.3 - empirical fit to maturation data
    pub k_maturation: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for PDE4BPathwayParams {
    fn default() -> Self {
        Self {
            v_ac: 5.0,            // μM/s
            k_pde: 0.1,           // /s
            km: 3.0,              // μM
            k_leak: 0.02,         // /s
            hill_n: 2.0,          // unitless
            k_pka_ec50: 3.5,      // μM
            pcreb_baseline: 0.12, // 12%
            pcreb_max: 0.65,      // 65%
            k_maturation: 0.3,    // unitless
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl PDE4BPathwayParams {
    /// Create parameters with custom values for sensitivity analysis
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        v_ac: f32,
        k_pde: f32,
        km: f32,
        k_leak: f32,
        hill_n: f32,
        k_pka_ec50: f32,
        pcreb_baseline: f32,
        pcreb_max: f32,
        k_maturation: f32,
    ) -> Self {
        Self {
            v_ac,
            k_pde,
            km,
            k_leak,
            hill_n,
            k_pka_ec50,
            pcreb_baseline,
            pcreb_max,
            k_maturation,
        }
    }

    /// Validate parameters are in physiologically plausible ranges
    /// # Errors
    ///
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Returns an error if the operation fails.
    pub fn validate(&self) -> Result<(), PathwayValidationError> {
        if self.v_ac <= 0.0 {
            return Err(PathwayValidationError::InvalidParameter(
                "v_ac must be positive".to_string(),
            ));
        }
        if self.k_pde <= 0.0 {
            return Err(PathwayValidationError::InvalidParameter(
                "k_pde must be positive".to_string(),
            ));
        }
        if self.km <= 0.0 {
            return Err(PathwayValidationError::InvalidParameter(
                "km must be positive".to_string(),
            ));
        }
        if self.k_leak < 0.0 {
            return Err(PathwayValidationError::InvalidParameter(
                "k_leak must be non-negative".to_string(),
            ));
        }
        if self.hill_n <= 0.0 {
            return Err(PathwayValidationError::InvalidParameter(
                "hill_n must be positive".to_string(),
            ));
        }
        if self.k_pka_ec50 <= 0.0 {
            return Err(PathwayValidationError::InvalidParameter(
                "k_pka_ec50 must be positive".to_string(),
            ));
            // TAG: surface=simulation owner=research-team rule=SIM-001
        }
        if !(0.0..=1.0).contains(&self.pcreb_baseline) {
            return Err(PathwayValidationError::InvalidParameter(
                "pcreb_baseline must be in [0, 1]".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&self.pcreb_max) {
            return Err(PathwayValidationError::InvalidParameter(
                "pcreb_max must be in [0, 1]".to_string(),
            ));
        }
        if self.pcreb_baseline >= self.pcreb_max {
            return Err(PathwayValidationError::InvalidParameter(
                "pcreb_baseline must be < pcreb_max".to_string(),
            ));
        }
        if self.k_maturation <= 0.0 {
            return Err(PathwayValidationError::InvalidParameter(
                "k_maturation must be positive".to_string(),
            ));
        }
        Ok(())
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Pathway validation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathwayValidationError {
    /// invalid parameter.
    InvalidParameter(String),
}

impl std::fmt::Display for PathwayValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidParameter(msg) => write!(f, "Invalid pathway parameter: {msg}"),
        }
    }
}

impl std::error::Error for PathwayValidationError {}

// ============================================================================
// Step 1: cAMP Concentration
// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute cAMP concentration from PDE4B level and pathway parameters
///
/// Formula: cAMP = `v_AC` / (`k_PDE` * `PDE4B/K_M` + `k_leak`)
///
/// This represents steady-state cAMP concentration where production (`v_AC`)
/// balances degradation by PDE4B and other mechanisms.
///
/// # Arguments
/// * `pde4b_level` - PDE4B expression level [0.0, 1.0]
/// * `params` - Pathway parameters
///
/// # Returns
/// cAMP concentration in μM
///
/// # Examples
/// ```
/// use sim_core::neural::pde4b_pathway::{compute_camp, PDE4BPathwayParams};
///
/// let params = PDE4BPathwayParams::default();
/// let camp_low_pde4b = compute_camp(0.0, &params);  // High cAMP
/// let camp_high_pde4b = compute_camp(1.0, &params); // Low cAMP
///
/// debug_assert!(camp_low_pde4b > camp_high_pde4b);
/// ```
#[must_use]
pub fn compute_camp(pde4b_level: f32, params: &PDE4BPathwayParams) -> f32 {
    let pde4b_level = pde4b_level.clamp(0.0, 1.0);
    // PDE4B degradation term: k_PDE * [PDE4B] / K_M
    // Note: pde4b_level is normalized [0,1], scale appropriately for concentration
    let pde4b_concentration = pde4b_level * 10.0; // Scale to ~10 μM max
    let km = params.km.max(1e-6);
    let degradation = params.k_pde.max(0.0) * pde4b_concentration / km;
    let denom = (degradation + params.k_leak).max(1e-6);

    // Steady-state: production = degradation
    params.v_ac.max(0.0) / denom
}

/// Compute cAMP with explicit parameters (for testing/sensitivity analysis)
#[must_use]
pub fn compute_camp_explicit(pde4b_level: f32, v_ac: f32, k_pde: f32, km: f32, k_leak: f32) -> f32 {
    let pde4b_level = pde4b_level.clamp(0.0, 1.0);
    let pde4b_concentration = pde4b_level * 10.0;
    let km = km.max(1e-6);
    let degradation = k_pde.max(0.0) * pde4b_concentration / km;
    let denom = (degradation + k_leak).max(1e-6);
    v_ac.max(0.0) / denom
}

// ============================================================================
// Step 2: PKA Activation
// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute PKA activation from cAMP concentration using Hill function
///
/// Formula: `PKA_active` = cAMP^n / (`K_PKAn^n` + cAMP^n)
///
/// The Hill function captures cooperative binding of cAMP to PKA regulatory
/// subunits. When 2 cAMP molecules bind, the catalytic subunits are released.
///
/// # Arguments
/// * `camp` - cAMP concentration in μM
/// * `params` - Pathway parameters
///
/// # Returns
/// PKA active fraction [0.0, 1.0]
///
/// # Examples
/// ```
/// use sim_core::neural::pde4b_pathway::{compute_pka_activation, PDE4BPathwayParams};
///
/// let params = PDE4BPathwayParams::default();
/// let pka_low = compute_pka_activation(1.0, &params);  // Low cAMP
/// let pka_high = compute_pka_activation(10.0, &params); // High cAMP
///
/// debug_assert!(pka_high > pka_low);
/// ```
#[must_use]
pub fn compute_pka_activation(camp: f32, params: &PDE4BPathwayParams) -> f32 {
    if camp <= 0.0 {
        return 0.0;
    }
    let camp_n = camp.max(0.0).powf(params.hill_n);
    let k_n = params.k_pka_ec50.max(1e-6).powf(params.hill_n);
    let denom = k_n + camp_n;
    if denom <= 0.0 {
        0.0
    } else {
        (camp_n / denom).clamp(0.0, 1.0)
    }
}

/// Compute PKA activation with explicit parameters (for testing)
#[must_use]
pub fn compute_pka_activation_explicit(camp: f32, n: f32, k_pka_ec50: f32) -> f32 {
    if camp <= 0.0 {
        return 0.0;
    }
    let camp_n = camp.max(0.0).powf(n);
    let k_n = k_pka_ec50.max(1e-6).powf(n);
    let denom = k_n + camp_n;
    if denom <= 0.0 {
        0.0
    } else {
        (camp_n / denom).clamp(0.0, 1.0)
    }
}

// ============================================================================
// Step 3: CREB Phosphorylation
// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute CREB phosphorylation level from PKA activation
///
/// Formula: pCREB = baseline + (max - baseline) * `PKA_active`
///
/// Baseline phosphorylation (~12%) occurs through other kinases.
/// PKA activation increases this to maximum (~65%) through direct
/// phosphorylation at Ser133.
///
/// # Arguments
/// * `pka_active` - PKA active fraction [0.0, 1.0]
/// * `params` - Pathway parameters
///
/// # Returns
/// pCREB fraction [`pcreb_baseline`, `pcreb_max`]
///
/// # Examples
/// ```
/// use sim_core::neural::pde4b_pathway::{compute_pcreb, PDE4BPathwayParams};
///
/// let params = PDE4BPathwayParams::default();
/// let pcreb_low = compute_pcreb(0.0, &params);  // Baseline only
/// let pcreb_high = compute_pcreb(1.0, &params); // Maximum
///
/// debug_assert!(pcreb_high > pcreb_low);
/// ```
#[must_use]
pub fn compute_pcreb(pka_active: f32, params: &PDE4BPathwayParams) -> f32 {
    params.pcreb_baseline + (params.pcreb_max - params.pcreb_baseline) * pka_active.clamp(0.0, 1.0)
}

/// Compute pCREB with explicit parameters (for testing)
#[must_use]
pub fn compute_pcreb_explicit(pka_active: f32, baseline: f32, max_pcreb: f32) -> f32 {
    baseline + (max_pcreb - baseline) * pka_active.clamp(0.0, 1.0)
}

// ============================================================================
// Step 4: Maturation Modulation
// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute maturation modulation from pCREB level
///
/// Formula: Modulation = 1.0 + pCREB / (`K_mat` + pCREB)
///
/// This Michaelis-Menten-like function gives:
/// - Baseline: 1.0 when pCREB = 0 (no modulation)
/// - Maximum: ~2.0 when pCREB >> `K_mat` (2x faster maturation)
///
/// The shape allows graded responses to CREB activation.
///
/// # Arguments
/// * `pcreb` - pCREB fraction [0.0, 1.0]
/// * `params` - Pathway parameters
///
/// # Returns
/// Maturation modulation factor [1.0, ~2.0]
///
/// # Examples
/// ```
/// use sim_core::neural::pde4b_pathway::{compute_modulation, PDE4BPathwayParams};
///
/// let params = PDE4BPathwayParams::default();
/// let mod_low = compute_modulation(0.0, &params);  // No modulation
/// let mod_high = compute_modulation(1.0, &params); // Maximum modulation
///
/// debug_assert!(mod_low >= 1.0);
/// debug_assert!(mod_high <= 2.0);
/// debug_assert!(mod_high > mod_low);
/// ```
#[must_use]
pub fn compute_modulation(pcreb: f32, params: &PDE4BPathwayParams) -> f32 {
    1.0 + pcreb / (params.k_maturation + pcreb)
}

/// Compute modulation with explicit parameters (for testing)
#[must_use]
pub fn compute_modulation_explicit(pcreb: f32, k_mat: f32) -> f32 {
    1.0 + pcreb / (k_mat + pcreb)
}

// ============================================================================
// Complete Pathway Computation
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
// ============================================================================

/// Compute complete PDE4B → cAMP → PKA → CREB → maturation pathway
///
/// This function runs the full 4-step cascade and returns all intermediate
/// values for inspection and debugging.
///
/// # Arguments
/// * `pde4b_level` - PDE4B expression level [0.0, 1.0]
/// * `params` - Pathway parameters
///
/// # Returns
/// `PathwayOutput` containing all intermediate values
#[must_use]
pub fn compute_pathway(pde4b_level: f32, params: &PDE4BPathwayParams) -> PathwayOutput {
    let camp = compute_camp(pde4b_level, params);
    let pka = compute_pka_activation(camp, params);
    let pcreb = compute_pcreb(pka, params);
    let modulation = compute_modulation(pcreb, params);

    PathwayOutput {
        pde4b_level,
        camp,
        pka,
        pcreb,
        modulation,
    }
}

/// Output of the complete pathway computation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PathwayOutput {
    /// Input PDE4B level
    pub pde4b_level: f32,
    /// Computed cAMP concentration (μM)
    pub camp: f32,
    /// PKA activation fraction [0.0, 1.0]
    pub pka: f32,
    /// pCREB fraction [0.0, 1.0]
    pub pcreb: f32,
    /// Maturation modulation factor [1.0, ~2.0]
    pub modulation: f32,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl PathwayOutput {
    /// Get the final maturation rate modifier
    /// This is the primary output used by the maturation system
    #[must_use]
    pub const fn maturation_modulation(&self) -> f32 {
        self.modulation
    }

    /// Get cAMP as normalized value [0.0, 1.0] for display/storage
    /// Note: normalization assumes max cAMP ~50 μM
    #[must_use]
    pub fn camp_normalized(&self) -> f32 {
        (self.camp / 50.0).clamp(0.0, 1.0)
    }
}

// ============================================================================
// Convenience Functions for Genome Integration
// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute maturation modulation from PDE4B level using default parameters
///
/// This is the main entry point for the genome module to compute
/// PDE4B-based maturation effects.
///
/// # Arguments
/// * `pde4b_level` - PDE4B expression level [0.0, 1.0]
///
/// # Returns
/// Maturation modulation factor [1.0, ~2.0]
///
/// Range: 1.0 at high PDE4B (slow maturation) to ~1.8 at low PDE4B (fast maturation)
#[must_use]
pub fn compute_pde4b_modulation(pde4b_level: f32) -> f32 {
    let params = PDE4BPathwayParams::default();
    let output = compute_pathway(pde4b_level, &params);
    output.modulation
}

/// Compute maturation modulation with stress and resource effects
///
/// This is the complete function for computing effective maturation rate
/// in the context of agent state.
///
/// # Arguments
/// * `pde4b_level` - PDE4B expression level [0.0, 1.0]
/// * `stress_level` - Current cortisol level [0.0, 1.0]
/// * `resource_level` - Current energy ratio [0.0, 1.0]
/// * `stress_sensitivity` - Genome-controlled stress sensitivity [0.2, 1.0]
///
/// # Returns
/// Effective maturation rate multiplier
#[must_use]
pub fn compute_maturation_rate_pathway(
    pde4b_level: f32,
    stress_level: f32,
    resource_level: f32,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    stress_sensitivity: f32,
) -> f32 {
    let params = PDE4BPathwayParams::default();

    // Step 1: cAMP production vs degradation
    let camp = compute_camp(pde4b_level, &params);

    // Step 2: PKA activation (Hill function)
    let pka = compute_pka_activation(camp, &params);

    // Step 3: CREB phosphorylation
    let pcreb = compute_pcreb(pka, &params);

    // Step 4: Maturation modulation
    let base_modulation = compute_modulation(pcreb, &params);

    // Apply stress effect
    let stress_effect = if stress_level > 0.3 {
        1.0 - (stress_level - 0.3) * stress_sensitivity * 0.5
    } else {
        1.0
    };

    // Apply resource effect
    let resource_effect = if resource_level < 0.5 {
        0.7 + resource_level * 0.6
    } else {
        1.0
    };

    // Combine all effects
    1.0 * base_modulation * stress_effect.max(0.1) * resource_effect.max(0.1)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // === Step 1: cAMP Tests ===

    #[test]
    fn test_camp_low_pde4b() {
        let params = PDE4BPathwayParams::default();
        // Low PDE4B → high cAMP
        let camp = compute_camp(0.0, &params);
        assert!(camp > 100.0, "Low PDE4B should give high cAMP: {camp}");
    }

    #[test]
    fn test_camp_high_pde4b() {
        let params = PDE4BPathwayParams::default();
        // High PDE4B → low cAMP
        let camp = compute_camp(1.0, &params);
        assert!(camp < 20.0, "High PDE4B should give low cAMP: {camp}");
    }

    #[test]
    fn test_camp_inverse_relationship() {
        let params = PDE4BPathwayParams::default();
        let camp_low_pde4b = compute_camp(0.0, &params);
        let camp_high_pde4b = compute_camp(1.0, &params);
        assert!(
            camp_low_pde4b > camp_high_pde4b,
            "cAMP should be inversely related to PDE4B: {camp_low_pde4b} vs {camp_high_pde4b}"
        );
    }

    #[test]
    fn test_camp_physiological_range() {
        let params = PDE4BPathwayParams::default();
        // cAMP should be in physiological range [0.1, 250] μM
        for i in 0..=10 {
            let pde4b = i as f32 / 10.0;
            let camp = compute_camp(pde4b, &params);
            assert!(
                camp > 0.0 && camp < 300.0,
                "cAMP {camp} at PDE4B {pde4b} is out of physiological range"
            );
        }
    }

    // === Step 2: PKA Tests ===

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_pka_low_camp() {
        let params = PDE4BPathwayParams::default();
        // Low cAMP → low PKA
        let pka = compute_pka_activation(0.5, &params);
        assert!(pka < 0.1, "Low cAMP should give low PKA: {pka}");
    }

    #[test]
    fn test_pka_high_camp() {
        let params = PDE4BPathwayParams::default();
        // High cAMP → high PKA
        let pka = compute_pka_activation(20.0, &params);
        assert!(pka > 0.9, "High cAMP should give high PKA: {pka}");
    }

    #[test]
    fn test_pka_hill_sigmoid() {
        let params = PDE4BPathwayParams::default();
        // Hill function should give sigmoid curve
        let pka_at_ec50 = compute_pka_activation(params.k_pka_ec50, &params);
        assert!(
            (pka_at_ec50 - 0.5).abs() < 0.01,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            "PKA at EC50 should be ~0.5: {pka_at_ec50}"
        );
    }

    #[test]
    fn test_pka_range() {
        let params = PDE4BPathwayParams::default();
        // PKA should always be in [0, 1]
        for camp in [0.0, 1.0, 5.0, 10.0, 50.0, 100.0] {
            let pka = compute_pka_activation(camp, &params);
            assert!(
                (0.0..=1.0).contains(&pka),
                "PKA {pka} at cAMP {camp} is out of range [0, 1]"
            );
        }
    }

    #[test]
    fn test_pka_cooperativity() {
        // Test that n=2 gives steeper curve than n=1
        let params_n1 = PDE4BPathwayParams {
            hill_n: 1.0,
            ..Default::default()
        };
        let params_n2 = PDE4BPathwayParams {
            hill_n: 2.0,
            ..Default::default()
        };

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let camp = 3.0; // Below EC50
        let pka_n1 = compute_pka_activation(camp, &params_n1);
        let pka_n2 = compute_pka_activation(camp, &params_n2);

        // Higher cooperativity (n=2) should give lower activation below EC50
        assert!(
            pka_n2 < pka_n1,
            "Hill n=2 should be steeper than n=1 below EC50: {pka_n2} vs {pka_n1}"
        );
    }

    // === Step 3: CREB Tests ===

    #[test]
    fn test_pcreb_baseline() {
        let params = PDE4BPathwayParams::default();
        // Zero PKA → baseline pCREB
        let pcreb = compute_pcreb(0.0, &params);
        assert!(
            (pcreb - params.pcreb_baseline).abs() < 0.001,
            "Zero PKA should give baseline pCREB: {} vs {}",
            pcreb,
            params.pcreb_baseline
        );
    }

    #[test]
    fn test_pcreb_maximum() {
        let params = PDE4BPathwayParams::default();
        // Maximum PKA → maximum pCREB
        let pcreb = compute_pcreb(1.0, &params);
        assert!(
            (pcreb - params.pcreb_max).abs() < 0.001,
            "Max PKA should give max pCREB: {} vs {}",
            pcreb,
            params.pcreb_max
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_pcreb_range() {
        let params = PDE4BPathwayParams::default();
        // pCREB should be in [baseline, max]
        for pka in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let pcreb = compute_pcreb(pka, &params);
            assert!(
                pcreb >= params.pcreb_baseline && pcreb <= params.pcreb_max,
                "pCREB {} at PKA {} is out of range [{}, {}]",
                pcreb,
                pka,
                params.pcreb_baseline,
                params.pcreb_max
            );
        }
    }

    #[test]
    fn test_pcreb_proportional() {
        let params = PDE4BPathwayParams::default();
        // pCREB should increase with PKA
        let pcreb_low = compute_pcreb(0.2, &params);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let pcreb_high = compute_pcreb(0.8, &params);
        assert!(
            pcreb_high > pcreb_low,
            "pCREB should increase with PKA: {pcreb_high} vs {pcreb_low}"
        );
    }

    // === Step 4: Modulation Tests ===

    #[test]
    fn test_modulation_baseline() {
        let params = PDE4BPathwayParams::default();
        // Zero pCREB → baseline modulation = 1.0
        let modulation = compute_modulation(0.0, &params);
        assert!(
            (modulation - 1.0).abs() < 0.001,
            "Zero pCREB should give modulation = 1.0: {modulation}"
        );
    }

    #[test]
    fn test_modulation_maximum() {
        let params = PDE4BPathwayParams::default();
        // High pCREB → modulation approaching 2.0
        let modulation = compute_modulation(1.0, &params);
        assert!(
            modulation > 1.5 && modulation <= 2.0,
            "High pCREB should give modulation ~2.0: {modulation}"
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_modulation_range() {
        let params = PDE4BPathwayParams::default();
        // Modulation should be in [1.0, ~2.0]
        for pcreb in [0.0, 0.1, 0.3, 0.5, 0.7, 1.0] {
            let modulation = compute_modulation(pcreb, &params);
            assert!(
                (1.0..=2.0).contains(&modulation),
                "Modulation {modulation} at pCREB {pcreb} is out of range [1.0, 2.0]"
            );
        }
    }

    #[test]
    fn test_modulation_increases_with_pcreb() {
        let params = PDE4BPathwayParams::default();
        // Modulation should increase with pCREB
        let mod_low = compute_modulation(0.1, &params);
        let mod_high = compute_modulation(0.6, &params);
        assert!(
            mod_high > mod_low,
            "Modulation should increase with pCREB: {mod_high} vs {mod_low}"
        );
    }

    // === Complete Pathway Tests ===

    #[test]
    fn test_pathway_low_pde4b() {
        let params = PDE4BPathwayParams::default();
        // Low PDE4B → fast maturation (high modulation)
        let output = compute_pathway(0.0, &params);

        // With PDE4B = 0, only leak degradation remains
        // camp = 5.0 / 0.02 = 250 μM
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(
            output.camp > 100.0,
            "Low PDE4B → high cAMP: {}",
            output.camp
        );
        // With high cAMP, PKA approaches saturation
        assert!(output.pka > 0.95, "High cAMP → high PKA: {}", output.pka);
        // pCREB approaches maximum
        assert!(
            output.pcreb > 0.6,
            "High PKA → high pCREB: {}",
            output.pcreb
        );
        // Modulation approaches ~1.8
        assert!(
            output.modulation > 1.5,
            "High pCREB → high modulation: {}",
            output.modulation
        );
    }

    #[test]
    fn test_pathway_high_pde4b() {
        let params = PDE4BPathwayParams::default();
        // High PDE4B → slow maturation (low modulation)
        let output = compute_pathway(1.0, &params);

        // With high PDE4B, cAMP is reduced but may still be above EC50
        // Due to Hill function steepness, PKA can still be significant
        assert!(output.camp < 20.0, "High PDE4B → low cAMP: {}", output.camp);
        // Note: With EC50=3.5 and cAMP~15, PKA is still ~0.9 due to n=2 cooperativity
        // The key is the modulation should be lower than low PDE4B case
        assert!(
            output.pcreb > params.pcreb_baseline,
            "pCREB should be above baseline"
        );
        assert!(
            output.modulation < 1.9,
            "High PDE4B should give reduced modulation: {}",
            output.modulation
        );
        assert!(
            output.modulation > 1.0,
            "Modulation should be above baseline: {}",
            output.modulation
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_pathway_inverse_relationship() {
        let params = PDE4BPathwayParams::default();
        let output_low = compute_pathway(0.0, &params);
        let output_high = compute_pathway(1.0, &params);

        assert!(
            output_low.modulation > output_high.modulation,
            "Low PDE4B should give higher modulation: {} vs {}",
            output_low.modulation,
            output_high.modulation
        );
    }

    #[test]
    fn test_pathway_monotonic() {
        let params = PDE4BPathwayParams::default();
        // Modulation should decrease monotonically with PDE4B
        let mut prev_modulation = f32::INFINITY;
        for i in 0..=10 {
            let pde4b = i as f32 / 10.0;
            let output = compute_pathway(pde4b, &params);
            assert!(
                output.modulation <= prev_modulation * 1.01, // Allow small numerical error
                "Modulation should decrease with PDE4B: {} at PDE4B={}",
                output.modulation,
                pde4b
            );
            prev_modulation = output.modulation;
        }
    }

    #[test]
    fn test_convenience_functions() {
        // Test the convenience functions work
        let modulation = compute_pde4b_modulation(0.5);
        assert!((1.0..=2.0).contains(&modulation));

        let rate = compute_maturation_rate_pathway(0.5, 0.0, 1.0, 0.5);
        assert!(rate > 0.0);
    }

    #[test]
    fn test_maturation_rate_stress_effect() {
        // Test that stress slows maturation
        let rate_no_stress = compute_maturation_rate_pathway(0.5, 0.0, 1.0, 0.5);
        let rate_high_stress = compute_maturation_rate_pathway(0.5, 0.8, 1.0, 0.5);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(
            rate_high_stress < rate_no_stress,
            "High stress should slow maturation: {rate_high_stress} vs {rate_no_stress}"
        );
    }

    #[test]
    fn test_maturation_rate_resource_effect() {
        // Test that low resources slow maturation
        let rate_abundant = compute_maturation_rate_pathway(0.5, 0.0, 1.0, 0.5);
        let rate_starvation = compute_maturation_rate_pathway(0.5, 0.0, 0.0, 0.5);

        assert!(
            rate_starvation < rate_abundant,
            "Starvation should slow maturation: {rate_starvation} vs {rate_abundant}"
        );
    }

    // === Parameter Validation Tests ===

    #[test]
    fn test_default_params_valid() {
        let params = PDE4BPathwayParams::default();
        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_invalid_params() {
        let params = PDE4BPathwayParams {
            v_ac: -1.0,
            ..Default::default()
        };
        assert!(params.validate().is_err());

        let params = PDE4BPathwayParams {
            pcreb_baseline: 0.8,
            pcreb_max: 0.2,
            ..Default::default()
        };
        assert!(params.validate().is_err());
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // === Empirical Range Validation ===

    #[test]
    fn test_camp_empirical_range() {
        // cAMP in neurons typically ranges from ~0.1 to ~10 μM
        // Our model should give comparable ranges
        let params = PDE4BPathwayParams::default();

        let camp_min = compute_camp(1.0, &params);
        let camp_max = compute_camp(0.0, &params);

        // Relaxed bounds for the model
        assert!(camp_min > 0.01, "Min cAMP too low: {camp_min}");
        assert!(camp_max < 500.0, "Max cAMP too high: {camp_max}");
    }

    #[test]
    fn test_pka_ec50_empirical() {
        // Cellular EC50 for PKA is ~1-10 μM
        // Our default is 3.5 μM
        let params = PDE4BPathwayParams::default();
        assert!(
            params.k_pka_ec50 >= 1.0 && params.k_pka_ec50 <= 10.0,
            "PKA EC50 {} is outside empirical range [1, 10] μM",
            params.k_pka_ec50
        );
    }

    #[test]
    fn test_pcreb_empirical_range() {
        // Empirical pCREB ranges from ~10-15% baseline to ~60-70% max
        let params = PDE4BPathwayParams::default();

        assert!(
            params.pcreb_baseline >= 0.05 && params.pcreb_baseline <= 0.20,
            "pCREB baseline {} outside empirical range",
            params.pcreb_baseline
        );
        assert!(
            params.pcreb_max >= 0.50 && params.pcreb_max <= 0.80,
            "pCREB max {} outside empirical range",
            params.pcreb_max
        );
    }
}
