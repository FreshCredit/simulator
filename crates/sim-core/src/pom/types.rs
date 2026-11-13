// TAG: surface=simulation owner=research-team rule=SIM-001
//! POM types, categories, and metrics structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// POM Validation Suite containing complete results across all categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomValidationSuite {
    /// Validation results by category
    pub categories: HashMap<PomCategory, CategoryResults>,
    /// Overall validation score (0-100)
    pub overall_score: f32,
    /// Number of targets passing
    pub passed_count: usize,
    /// Total number of targets
    pub total_count: usize,
}
// TAG: surface=simulation owner=platform-team rule=GENERAL-001

/// POM Target Categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PomCategory {
    /// Stage 0: Geography and fundamental survival
    Geography,
    /// Stage 1: Demographics and mortality
    Demographics,
    /// Stage 2: DOCAS neurochemical patterns
    Docas,
    /// Stage 3: Society and trust networks
    Society,
    /// Stage 4: Language and communication
    Language,
    /// Stage 5: Protocol and verification
    Protocol,
    /// Cross-cutting: Criticality and heavy tails
    Criticality,
    /// Cross-cutting: Economic patterns
    Economics,
    /// The Fresh Protocol patterns
    Tfp,
    /// General Theory of Trust patterns
    Gtt,
}

impl std::fmt::Display for PomCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Geography => write!(f, "Geography"),
            Self::Demographics => write!(f, "Demographics"),
            Self::Docas => write!(f, "DOCAS"),
            Self::Society => write!(f, "Society"),
            Self::Language => write!(f, "Language"),
            Self::Protocol => write!(f, "Protocol"),
            Self::Criticality => write!(f, "Criticality"),
            Self::Economics => write!(f, "Economics"),
            Self::Tfp => write!(f, "TFP"),
            Self::Gtt => write!(f, "GTT"),
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Results for a single category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryResults {
    /// Category name
    pub category: PomCategory,
    /// Individual target results
    pub targets: Vec<PomTargetResult>,
    /// Category pass rate (0-1)
    pub pass_rate: f32,
    /// Number passed
    pub passed: usize,
    /// Total targets
    pub total: usize,
}

/// Individual POM Target result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomTargetResult {
    /// Target identifier (e.g., "GEO-001")
    pub target_id: String,
    /// Human-readable name
    pub name: String,
    /// Pattern description
    pub description: String,
    /// Observed value
    pub observed: f32,
    /// Acceptance band (min, max)
    pub acceptance_band: (f32, f32),
    /// Whether validation passed
    pub passed: bool,
    /// Empirical source reference
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub source: String,
    /// Criticality level (P0=required, P1=important, P2=optional)
    pub priority: PomPriority,
}

/// Validation priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PomPriority {
    /// P0: Required for research validity
    P0,
    /// P1: Important but not blocking
    P1,
    /// P2: Optional/nice-to-have
    P2,
}

impl PomPriority {
    #[must_use]
    /// as str.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::P0 => "P0",
            Self::P1 => "P1",
            Self::P2 => "P2",
        }
    }
}

/// Metrics container for POM validation
/// Populated by compute functions from simulation state
#[derive(Debug, Clone, Default)]
pub struct PomMetrics {
    // Geography
    /// agent survival rate.
    pub agent_survival_rate: f32,
    /// energy cv.
    pub energy_cv: f32,
    // Demographics
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// mortality young.
    pub mortality_young: f32,
    /// mortality old.
    pub mortality_old: f32,
    // DOCAS
    /// dopamine baseline.
    pub dopamine_baseline: f32,
    /// oxytocin baseline.
    pub oxytocin_baseline: f32,
    /// cortisol response.
    pub cortisol_response: f32,
    /// trust heritability h squared.
    pub trust_heritability_h_squared: f32,
    /// pde4b learning correlation.
    pub pde4b_learning_correlation: f32,
    /// curvature kappa cooperation.
    pub curvature_kappa_cooperation: f32,
    /// viscosity variation psi.
    pub viscosity_variation_psi: f32,
    /// coupling phi sync.
    pub coupling_phi_sync: f32,
    /// component heritability.
    pub component_heritability: f32,
    // Society
    /// cooperation rate.
    pub cooperation_rate: f32,
    /// network clustering.
    pub network_clustering: f32,
    /// avg path length.
    pub avg_path_length: f32,
    // Language
    /// lexicon size.
    pub lexicon_size: f32,
    /// communication success.
    pub communication_success: f32,
    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Protocol
    /// verification rate.
    pub verification_rate: f32,
    /// identity theft rate.
    pub identity_theft_rate: f32,
    // Criticality
    /// branching ratio.
    pub branching_ratio: f32,
    /// avalanche size exp.
    pub avalanche_size_exp: f32,
    // Economics
    /// wealth gini.
    pub wealth_gini: f32,
    /// transaction velocity.
    pub transaction_velocity: f32,
    /// gini trust correlation.
    pub gini_trust_correlation: f32,
    /// growth trust correlation.
    pub growth_trust_correlation: f32,
    /// interest rate tau relation.
    pub interest_rate_tau_relation: f32,
    /// crisis tau threshold.
    pub crisis_tau_threshold: f32,
    /// money as highest tau.
    pub money_as_highest_tau: f32,
    /// credit as prepaid tau.
    pub credit_as_prepaid_tau: f32,
    // TFP
    /// multimodal sync auc.
    pub multimodal_sync_auc: f32,
    /// lambda cross domain variance.
    pub lambda_cross_domain_variance: f32,
    /// entropy reduction delta h.
    pub entropy_reduction_delta_h: f32,
    /// composite stability r squared.
    pub composite_stability_r_squared: f32,
    /// spectral coherence cf.
    pub spectral_coherence_cf: f32,
    // GTT
    /// lambda scale invariant variance.
    pub lambda_scale_invariant_variance: f32,
    /// lambda universal constant.
    pub lambda_universal_constant: f32,
    /// neural curvature kappa correlation.
    pub neural_curvature_kappa_correlation: f32,
    /// phase alignment score.
    pub phase_alignment_score: f32,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// tft ess emergence.
    pub tft_ess_emergence: f32,
    /// semantic coherence cosine.
    pub semantic_coherence_cosine: f32,
}
