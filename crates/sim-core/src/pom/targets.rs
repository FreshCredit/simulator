// TAG: surface=simulation owner=research-team rule=SIM-001
//! POM target definitions and empirical acceptance bands

use super::types::*;

/// POM Target definitions
#[derive(Debug)]
pub struct PomTarget {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// id.
    pub id: &'static str,
    /// name.
    pub name: &'static str,
    /// description.
    pub description: &'static str,
    /// category.
    pub category: PomCategory,
    /// min.
    pub min: f32,
    /// max.
    pub max: f32,
    /// source.
    pub source: &'static str,
    /// priority.
    pub priority: PomPriority,
}

impl PomTarget {
    /// Check whether a value falls within the target acceptance band.
    #[must_use]
    pub fn is_in_range(&self, val: f32) -> bool {
        val >= self.min && val <= self.max
    }

    /// Check whether this target is P0 priority.
    #[must_use]
    pub fn is_p0(&self) -> bool {
        matches!(self.priority, PomPriority::P0)
    }

    /// Target range midpoint.
    #[must_use]
    pub fn midpoint(&self) -> f32 {
        (self.min + self.max) / 2.0
    }
}

/// Complete POM Target Library - All 41 targets
pub const POM_TARGETS: &[PomTarget] = &[
    // === GEOGRAPHY (Stage 0) - 2 targets ===
    PomTarget {
        id: "GEO-001",
        name: "Agent Survival Rate",
        description: "Percentage of agents surviving to tick 1000",
        category: PomCategory::Geography,
        min: 0.60,
        max: 0.95,
        source: "simulation-plan.md 5.3.2",
        priority: PomPriority::P0,
    },
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    PomTarget {
        id: "GEO-002",
        name: "Energy Distribution CV",
        description: "Coefficient of variation of agent energy levels",
        category: PomCategory::Geography,
        min: 0.15,
        max: 0.45,
        source: "simulation-plan.md 5.3.2",
        priority: PomPriority::P1,
    },
    // === DEMOGRAPHICS (Stage 1) - 2 targets ===
    PomTarget {
        id: "DEM-001",
        name: "Mortality Rate Age 0-1",
        description: "Annual mortality rate for agents age 0-1",
        category: PomCategory::Demographics,
        min: 0.05,
        max: 0.20,
        source: "Human DSW2020",
        priority: PomPriority::P1,
    },
    PomTarget {
        id: "DEM-002",
        name: "Mortality Rate Age 50+",
        description: "Annual mortality rate for agents age 50+",
        category: PomCategory::Demographics,
        min: 0.10,
        max: 0.40,
        source: "Gompertz law",
        priority: PomPriority::P1,
    },
    // === DOCAS (Stage 2) - 9 targets ===
    PomTarget {
        id: "DOC-001",
        // TAG: surface=simulation owner=research-team rule=SIM-001
        name: "Dopamine Baseline",
        description: "Resting dopamine level (normalized)",
        category: PomCategory::Docas,
        min: 0.30,
        max: 0.70,
        source: "Schultz 2015",
        priority: PomPriority::P0,
    },
    PomTarget {
        id: "DOC-002",
        name: "Oxytocin Baseline",
        description: "Resting oxytocin level (normalized)",
        category: PomCategory::Docas,
        min: 0.20,
        max: 0.60,
        source: "Zak 2017",
        priority: PomPriority::P0,
    },
    PomTarget {
        id: "DOC-003",
        name: "Cortisol Stress Response",
        description: "Cortisol increase under stress",
        category: PomCategory::Docas,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        min: 0.20,
        max: 0.80,
        source: "Sapolsky 2004",
        priority: PomPriority::P1,
    },
    // DOCAS Framework Extensions (§6.6) - 6 targets
    PomTarget {
        id: "DOC-C4",
        name: "Trust Heritability h-Squared",
        description: "Heritability of trust behavior",
        category: PomCategory::Docas,
        min: 0.45,
        max: 0.55,
        source: "simulation-plan.md §6.6",
        priority: PomPriority::P1,
    },
    PomTarget {
        id: "DOC-C5",
        name: "PDE4B Learning Rate Correlation",
        description: "Correlation between PDE4B and learning rate",
        category: PomCategory::Docas,
        min: 0.30,
        max: 0.80,
        source: "simulation-plan.md §6.6",
        priority: PomPriority::P2,
    },
    PomTarget {
        id: "DOC-C6",
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        name: "Curvature Kappa vs Cooperation",
        description: "Neural curvature correlation with cooperation",
        category: PomCategory::Docas,
        min: 0.20,
        max: 0.70,
        source: "simulation-plan.md §6.6",
        priority: PomPriority::P1,
    },
    PomTarget {
        id: "DOC-C7",
        name: "Viscosity Variation Psi",
        description: "Trust network viscosity over time",
        category: PomCategory::Docas,
        min: 0.10,
        max: 0.50,
        source: "simulation-plan.md §6.6",
        priority: PomPriority::P2,
    },
    PomTarget {
        id: "DOC-C8",
        name: "Coupling Phi Sync",
        description: "Coupling strength leading to synchronization",
        category: PomCategory::Docas,
        min: 0.00,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        max: 1.00,
        source: "simulation-plan.md §6.6",
        priority: PomPriority::P1,
    },
    PomTarget {
        id: "DOC-C9",
        name: "Component Heritability",
        description: "Heritability of DOCAS components",
        category: PomCategory::Docas,
        min: 0.30,
        max: 0.70,
        source: "simulation-plan.md §6.6",
        priority: PomPriority::P2,
    },
    // === SOCIETY (Stage 3) - 3 targets ===
    PomTarget {
        id: "SOC-001",
        name: "Cooperation Rate",
        description: "Fraction of cooperative interactions",
        category: PomCategory::Society,
        min: 0.30,
        max: 0.80,
        source: "Henrich 2004",
        priority: PomPriority::P0,
    },
    PomTarget {
        id: "SOC-002",
        name: "Network Clustering",
        description: "Global clustering coefficient",
        category: PomCategory::Society,
        min: 0.20,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        max: 0.60,
        source: "Watts-Strogatz",
        priority: PomPriority::P1,
    },
    PomTarget {
        id: "SOC-003",
        name: "Average Path Length",
        description: "Mean shortest path in trust network",
        category: PomCategory::Society,
        min: 2.00,
        max: 6.00,
        source: "Milgram 1967",
        priority: PomPriority::P1,
    },
    // === LANGUAGE (Stage 4) - 2 targets ===
    PomTarget {
        id: "LAN-001",
        name: "Lexicon Size",
        description: "Average vocabulary size per agent",
        category: PomCategory::Language,
        min: 20.0,
        max: 200.0,
        source: "simulation-plan.md 6.16",
        priority: PomPriority::P1,
    },
    PomTarget {
        id: "LAN-002",
        name: "Communication Success Rate",
        description: "Fraction of successful message exchanges",
        category: PomCategory::Language,
        min: 0.50,
        max: 0.95,
        source: "simulation-plan.md 6.16",
        priority: PomPriority::P1,
    },
    // === PROTOCOL (Stage 5) - 2 targets ===
    PomTarget {
        id: "PRO-001",
        name: "Identity Verification Rate",
        description: "Fraction of agents with verified IDs",
        category: PomCategory::Protocol,
        min: 0.70,
        max: 1.00,
        source: "simulation-plan.md 6.23",
        priority: PomPriority::P0,
    },
    PomTarget {
        id: "PRO-002",
        name: "Identity Theft Rate",
        description: "Annual rate of successful identity theft",
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        category: PomCategory::Protocol,
        min: 0.00,
        max: 0.10,
        source: "BJS 2016: 10% (corrected from 7.3%)",
        priority: PomPriority::P0,
    },
    // === CRITICALITY - 2 targets ===
    PomTarget {
        id: "CRI-001",
        name: "Branching Ratio",
        description: "Avalanche branching ratio sigma",
        category: PomCategory::Criticality,
        min: 0.90,
        max: 1.10,
        source: "Bak-Tang-Wiesenfeld",
        priority: PomPriority::P0,
    },
    PomTarget {
        id: "CRI-002",
        name: "Avalanche Size Exponent",
        description: "Power-law exponent for avalanche sizes",
        category: PomCategory::Criticality,
        min: 1.20,
        max: 1.80,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        source: "simulation-plan.md 6.12",
        priority: PomPriority::P1,
    },
    // === ECONOMICS - 8 targets ===
    PomTarget {
        id: "ECO-001",
        name: "Wealth Gini Coefficient",
        description: "Gini coefficient for wealth distribution",
        category: PomCategory::Economics,
        min: 0.30,
        max: 0.70,
        source: "World Bank WDI",
        priority: PomPriority::P1,
    },
    PomTarget {
        id: "ECO-002",
        name: "Transaction Velocity",
        description: "Average transactions per tick per agent",
        category: PomCategory::Economics,
        min: 0.01,
        max: 0.50,
        source: "simulation-plan.md 6.20",
        priority: PomPriority::P2,
    },
    // Economics Extensions (§6.8) - 6 targets
    PomTarget {
        id: "ECO-C1",
        name: "Gini-Trust Correlation",
        description: "Gini ∝ 1 - bₜ correlation (r ≈ -0.89)",
        category: PomCategory::Economics,
        min: -0.95,
        max: -0.80,
        // TAG: surface=simulation owner=research-team rule=SIM-001
        source: "simulation-plan.md §6.8",
        priority: PomPriority::P1,
    },
    PomTarget {
        id: "ECO-C2",
        name: "Growth-Trust Correlation",
        description: "Growth ∝ bₜ × τ (R² > 0.93)",
        category: PomCategory::Economics,
        min: 0.93,
        max: 1.00,
        source: "simulation-plan.md §6.8",
        priority: PomPriority::P1,
    },
    PomTarget {
        id: "ECO-C3",
        name: "Interest Rate Tau Relation",
        description: "Interest rate r ≈ 1/τ - 1",
        category: PomCategory::Economics,
        min: 0.50,
        max: 2.00,
        source: "simulation-plan.md §6.8",
        priority: PomPriority::P2,
    },
    PomTarget {
        id: "ECO-C4",
        name: "Crisis Tau Threshold",
        description: "Crisis threshold at τ ≈ 0.62",
        category: PomCategory::Economics,
        min: 0.58,
        max: 0.66,
        source: "simulation-plan.md §6.8",
        priority: PomPriority::P1,
    },
    PomTarget {
        id: "ECO-C5",
        name: "Money as Highest-Tau Asset",
        description: "Money emerges as highest-τ asset",
        category: PomCategory::Economics,
        min: 0.50,
        max: 1.00,
        source: "simulation-plan.md §6.8",
        priority: PomPriority::P2,
    },
    PomTarget {
        id: "ECO-C6",
        name: "Credit as Prepaid Tau",
        description: "Credit as pre-paid τ mechanism",
        category: PomCategory::Economics,
        min: 0.50,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        max: 1.00,
        source: "simulation-plan.md §6.8",
        priority: PomPriority::P2,
    },
    // === TFP - The Fresh Protocol (§6.5) - 5 targets ===
    PomTarget {
        id: "TFP-C1",
        name: "Multimodal Sync AUC",
        description: "Area under curve for multimodal synchronization",
        category: PomCategory::Tfp,
        min: 0.88,
        max: 0.94,
        source: "simulation-plan.md §6.5",
        priority: PomPriority::P0,
    },
    PomTarget {
        id: "TFP-C2",
        name: "Lambda Cross-Domain Variance",
        description: "Variance of λ across domains",
        category: PomCategory::Tfp,
        min: 0.00,
        max: 0.20,
        source: "simulation-plan.md §6.5",
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        priority: PomPriority::P1,
    },
    PomTarget {
        id: "TFP-C3",
        name: "Entropy Reduction Delta-H",
        description: "Entropy reduction in bits",
        category: PomCategory::Tfp,
        min: -0.60,
        max: -0.48,
        source: "simulation-plan.md §6.5",
        priority: PomPriority::P1,
    },
    PomTarget {
        id: "TFP-C4",
        name: "Composite Stability R-Squared",
        description: "R² of composite stability metric",
        category: PomCategory::Tfp,
        min: 0.60,
        max: 1.00,
        source: "simulation-plan.md §6.5",
        priority: PomPriority::P1,
    },
    PomTarget {
        id: "TFP-C5",
        name: "Spectral Coherence Cf",
        description: "Spectral coherence approaching unity",
        category: PomCategory::Tfp,
        min: 0.95,
        max: 1.00,
        source: "simulation-plan.md §6.5",
        priority: PomPriority::P0,
    },
    // === GTT - General Theory of Trust (§6.7) - 6 targets ===
    PomTarget {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        id: "GTT-C1",
        name: "Lambda Scale-Invariant Variance",
        description: "Variance of λ across scales",
        category: PomCategory::Gtt,
        min: 0.00,
        max: 0.15,
        source: "simulation-plan.md §6.7",
        priority: PomPriority::P0,
    },
    PomTarget {
        id: "GTT-C2",
        name: "Lambda Universal Constant",
        description: "Universal λ constant ≈ 0.61",
        category: PomCategory::Gtt,
        min: 0.58,
        max: 0.64,
        source: "simulation-plan.md §6.7",
        priority: PomPriority::P1,
    },
    PomTarget {
        id: "GTT-C3",
        name: "Neural Curvature Kappa Correlation",
        description: "Correlation between neural curvature and trust",
        category: PomCategory::Gtt,
        min: 0.30,
        max: 0.80,
        source: "simulation-plan.md §6.7",
        priority: PomPriority::P1,
    },
    PomTarget {
        id: "GTT-C4",
        name: "Phase Alignment Score",
        description: "Phase alignment in trust network",
        category: PomCategory::Gtt,
        min: 0.90,
        max: 1.00,
        source: "simulation-plan.md §6.7",
        priority: PomPriority::P1,
    },
    PomTarget {
        id: "GTT-C5",
        name: "TFT ESS Emergence",
        description: "Tit-for-Tat evolutionarily stable strategy",
        category: PomCategory::Gtt,
        min: 0.50,
        max: 1.00,
        source: "simulation-plan.md §6.7",
        priority: PomPriority::P1,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    },
    PomTarget {
        id: "GTT-C6",
        name: "Semantic Coherence Cosine",
        description: "Semantic coherence cos θ",
        category: PomCategory::Gtt,
        min: 0.75,
        max: 1.00,
        source: "simulation-plan.md §6.7",
        priority: PomPriority::P2,
    },
];
