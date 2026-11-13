use super::ValidationError;
use crate::types::Tick;
use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Empirical parameters derived from peer-reviewed research
/// These parameters are loaded from `empirical_parameters.toml`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmpiricalParameters {
    // Identity & Verification (7 parameters)
    /// credit report error rate.
    pub credit_report_error_rate: f32,
    /// serious error rate.
    pub serious_error_rate: f32,
    /// identity theft victim rate.
    pub identity_theft_victim_rate: f32,
    /// synthetic id detection rate.
    pub synthetic_id_detection_rate: f32,
    /// breach exposure rate.
    pub breach_exposure_rate: f32,
    /// reporting rate.
    pub reporting_rate: f32,
    /// attack vector infrastructure.
    pub attack_vector_infrastructure: f32,
    /// verification steps.
    pub verification_steps: u32,
    /// qualifier recovery rate.
    pub qualifier_recovery_rate: f32,

    // Cost & Economics (4 parameters)
    /// tri merge cost range.
    pub tri_merge_cost_range: (f32, f32),
    /// single bureau cost.
    pub single_bureau_cost: f32,
    /// internal verification cost.
    pub internal_verification_cost: f32,
    /// fraud cost multiplier.
    pub fraud_cost_multiplier: f32,

    // Friction & Abandonment (3 parameters)
    /// verification abandonment rate.
    pub verification_abandonment_rate: f32,
    /// step abandonment rate.
    pub step_abandonment_rate: f32,
    /// friction reduction lift.
    pub friction_reduction_lift: f32,

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Health & Social (3 parameters)
    /// financial shame mortality multiplier.
    pub financial_shame_mortality_multiplier: f32,
    /// poverty threshold daily.
    pub poverty_threshold_daily: f32,
    /// informal asset value.
    pub informal_asset_value: f64,

    // DOCAS Parameters (6 parameters)
    /// docas alpha range.
    pub docas_alpha_range: (f32, f32),
    /// docas beta range.
    pub docas_beta_range: (f32, f32),
    /// criticality sigma target.
    pub criticality_sigma_target: f32,
    /// cortisol half life ticks.
    pub cortisol_half_life_ticks: f32,
    /// dopamine learning base.
    pub dopamine_learning_base: f32,
    /// docas homeostatic baseline.
    pub docas_homeostatic_baseline: f32,

    // Genome & Heritability (6 parameters)
    /// heritability h2.
    pub heritability_h2: f32,
    /// pde4b variance optimal.
    pub pde4b_variance_optimal: (f32, f32),
    /// mutation rate.
    pub mutation_rate: f32,
    /// recombination rate.
    pub recombination_rate: f32,
    /// parent offspring correlation min.
    pub parent_offspring_correlation_min: f32,
    /// learning rate lambda.
    pub learning_rate_lambda: f32,

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Game Theory (3 parameters)
    /// cooperation threshold pd.
    pub cooperation_threshold_pd: f32,
    /// cooperation rate target.
    pub cooperation_rate_target: (f32, f32),
    /// system2 proportion.
    pub system2_proportion: (f32, f32),

    // Economics (4 parameters)
    /// gini coefficient target.
    pub gini_coefficient_target: (f32, f32),
    /// credit access threshold.
    pub credit_access_threshold: f32,
    /// bond density threshold.
    pub bond_density_threshold: f32,
    /// wealth trust correlation.
    pub wealth_trust_correlation: f32,

    // Recovery (5 parameters)
    /// identity theft recovery min.
    pub identity_theft_recovery_min: Tick,
    /// identity theft recovery max.
    pub identity_theft_recovery_max: Tick,
    /// stress recovery half life.
    pub stress_recovery_half_life: f32,
    /// reputation recovery half life.
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub reputation_recovery_half_life: f32,
    /// protocol protection effectiveness.
    pub protocol_protection_effectiveness: (f32, f32),

    // POM Targets (10 parameters)
    /// docas c1 heritability.
    pub docas_c1_heritability: f32,
    /// docas c2 homeostasis variance.
    pub docas_c2_homeostasis_variance: f32,
    /// tfp c3 cross domain coherence.
    pub tfp_c3_cross_domain_coherence: (f32, f32),
    /// gtt c2 cooperation threshold.
    pub gtt_c2_cooperation_threshold: f32,
    /// econ c1 gini trust correlation.
    pub econ_c1_gini_trust_correlation: (f32, f32),
    /// sim group 001 clustering.
    pub sim_group_001_clustering: (f32, f32),
    /// sim credit 001 access threshold.
    pub sim_credit_001_access_threshold: f32,
    /// id theft 001 theft rate.
    pub id_theft_001_theft_rate: (f32, f32),
    /// id theft 003 protection reduction.
    pub id_theft_003_protection_reduction: (f32, f32),
    /// cri 001 branching ratio.
    pub cri_001_branching_ratio: (f32, f32),

    // v4.1 Empirical Research Calibration Constants (Sim K, Sim D16-D21, Sim G25, Sim P8b, Sim ECOL)
    /// Kingman coalescence intensity kappa* (Sim K)
    #[serde(default = "default_calibrated_kingman_kappa")]
    pub calibrated_kingman_kappa: f32,
    /// Perky reality threshold T_reality (Sim D19)
    #[serde(default = "default_reality_threshold_t_star")]
    pub reality_threshold_t_star: f32,
    /// Auditory speech ceiling r_max in syl/s (Sim D20)
    #[serde(default = "default_auditory_speech_ceiling_r_max")]
    pub auditory_speech_ceiling_r_max: f32,
    /// Visual agreement cloaking ratio chi (Sim D16)
    #[serde(default = "default_visual_cloaking_ratio_chi")]
    pub visual_cloaking_ratio_chi: f32,
    /// Allostatic yield pain threshold A_crit (Sim G25)
    #[serde(default = "default_allostatic_yield_a_crit")]
    pub allostatic_yield_a_crit: f32,
    /// Asymmetric loss penalty ratio lambda^- / lambda^+ (Sim P8b, Sim C)
    #[serde(default = "default_asymmetric_penalty_ratio")]
    pub asymmetric_penalty_ratio: f32,
    /// Baseline ecological carrying capacity K_0 (Sim ECOL)
    #[serde(default = "default_carrying_capacity_k0")]
    pub carrying_capacity_k0: f32,
    /// Allee effect extinction threshold N_crit (Sim ECOL)
    #[serde(default = "default_allee_threshold_n_crit")]
    pub allee_threshold_n_crit: usize,
}

const fn default_calibrated_kingman_kappa() -> f32 {
    1.62
}
const fn default_reality_threshold_t_star() -> f32 {
    2.50
}
const fn default_auditory_speech_ceiling_r_max() -> f32 {
    9.80
}
const fn default_visual_cloaking_ratio_chi() -> f32 {
    3.65
}
const fn default_allostatic_yield_a_crit() -> f32 {
    50.0
}
const fn default_asymmetric_penalty_ratio() -> f32 {
    4.0
}
const fn default_carrying_capacity_k0() -> f32 {
    500.0
}
const fn default_allee_threshold_n_crit() -> usize {
    25
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for EmpiricalParameters {
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn default() -> Self {
        Self {
            // Identity & Verification
            credit_report_error_rate: 0.44,
            serious_error_rate: 0.145,
            identity_theft_victim_rate: 0.10, // BJS 2016: 10% annual (corrected from 7.3%)
            synthetic_id_detection_rate: 0.05,
            breach_exposure_rate: 5.12,
            reporting_rate: 0.046,
            attack_vector_infrastructure: 0.68,
            verification_steps: 3,
            qualifier_recovery_rate: 0.13,

            // Cost & Economics
            tri_merge_cost_range: (50.0, 110.0),
            single_bureau_cost: 1.95,
            internal_verification_cost: 0.16,
            fraud_cost_multiplier: 5.75,

            // TAG: surface=simulation owner=research-team rule=SIM-001
            // Friction & Abandonment
            verification_abandonment_rate: 0.70,
            step_abandonment_rate: 0.18,
            friction_reduction_lift: 0.18,

            // Health & Social
            financial_shame_mortality_multiplier: 1.26,
            poverty_threshold_daily: 30.0,
            informal_asset_value: 540_000_000_000_000.0,

            // DOCAS
            docas_alpha_range: (0.90, 0.99),
            docas_beta_range: (0.05, 0.20),
            criticality_sigma_target: 1.0,
            cortisol_half_life_ticks: 100.0,
            dopamine_learning_base: 0.61,
            docas_homeostatic_baseline: 0.70,

            // Genome
            heritability_h2: 0.50,
            pde4b_variance_optimal: (0.08, 0.12),
            mutation_rate: 0.001,
            recombination_rate: 0.5,
            parent_offspring_correlation_min: 0.5,
            learning_rate_lambda: 0.61,

            // Game Theory
            cooperation_threshold_pd: 0.65,
            cooperation_rate_target: (0.35, 0.65),
            system2_proportion: (0.01, 0.05),

            // Economics
            gini_coefficient_target: (0.40, 0.60),
            credit_access_threshold: 0.3,
            bond_density_threshold: 0.4,
            wealth_trust_correlation: -0.89,

            // Recovery
            identity_theft_recovery_min: 100,
            identity_theft_recovery_max: 500,
            stress_recovery_half_life: 30.0,
            reputation_recovery_half_life: 700.0,
            protocol_protection_effectiveness: (0.20, 0.40),

            // TAG: surface=simulation owner=research-team rule=SIM-001
            // POM Targets
            docas_c1_heritability: 0.50,
            docas_c2_homeostasis_variance: 0.02,
            tfp_c3_cross_domain_coherence: (0.45, 0.55),
            gtt_c2_cooperation_threshold: 0.65,
            econ_c1_gini_trust_correlation: (-0.95, -0.75),
            sim_group_001_clustering: (0.30, 0.60),
            sim_credit_001_access_threshold: 0.3,
            id_theft_001_theft_rate: (0.05, 0.10),
            id_theft_003_protection_reduction: (0.20, 0.40),
            cri_001_branching_ratio: (0.90, 1.10),

            // v4.1 Empirical Research Calibration Constants
            calibrated_kingman_kappa: 1.62,
            reality_threshold_t_star: 2.50,
            auditory_speech_ceiling_r_max: 9.80,
            visual_cloaking_ratio_chi: 3.65,
            allostatic_yield_a_crit: 50.0,
            asymmetric_penalty_ratio: 4.0,
            carrying_capacity_k0: 500.0,
            allee_threshold_n_crit: 25,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl EmpiricalParameters {
    /// Validate all empirical parameters are within scientific bounds
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.validate_identity()?;
        self.validate_costs()?;
        self.validate_friction()?;
        self.validate_health()?;
        self.validate_docas()?;
        self.validate_genome()?;
        self.validate_game_theory()?;
        self.validate_economics()?;
        self.validate_recovery()?;
        self.validate_pom()?;
        Ok(())
    }

    fn validate_identity(&self) -> Result<(), ValidationError> {
        self.validate_rate("credit_report_error_rate", self.credit_report_error_rate)?;
        self.validate_rate("serious_error_rate", self.serious_error_rate)?;
        self.validate_rate(
            "identity_theft_victim_rate",
            self.identity_theft_victim_rate,
        )?;
        self.validate_rate(
            "synthetic_id_detection_rate",
            self.synthetic_id_detection_rate,
        )?;
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        self.validate_rate("reporting_rate", self.reporting_rate)?;
        self.validate_rate(
            "attack_vector_infrastructure",
            self.attack_vector_infrastructure,
        )
    }

    fn validate_costs(&self) -> Result<(), ValidationError> {
        if self.tri_merge_cost_range.0 >= self.tri_merge_cost_range.1 {
            return Err(ValidationError {
                field: "tri_merge_cost_range".into(),
                message: format!("min must be < max, got {:?}", self.tri_merge_cost_range),
            });
        }
        if self.single_bureau_cost <= 0.0 {
            return Err(ValidationError {
                field: "single_bureau_cost".into(),
                message: format!("must be > 0, got {}", self.single_bureau_cost),
            });
        }
        Ok(())
    }

    fn validate_friction(&self) -> Result<(), ValidationError> {
        self.validate_rate(
            "verification_abandonment_rate",
            self.verification_abandonment_rate,
        )?;
        self.validate_rate("step_abandonment_rate", self.step_abandonment_rate)?;
        self.validate_rate("friction_reduction_lift", self.friction_reduction_lift)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn validate_health(&self) -> Result<(), ValidationError> {
        if self.financial_shame_mortality_multiplier < 1.0 {
            return Err(ValidationError {
                field: "financial_shame_mortality_multiplier".into(),
                message: format!(
                    "must be >= 1.0, got {}",
                    self.financial_shame_mortality_multiplier
                ),
            });
        }
        if self.poverty_threshold_daily <= 0.0 {
            return Err(ValidationError {
                field: "poverty_threshold_daily".into(),
                message: format!("must be > 0, got {}", self.poverty_threshold_daily),
            });
        }
        Ok(())
    }

    fn validate_docas(&self) -> Result<(), ValidationError> {
        self.validate_range("docas_alpha_range", self.docas_alpha_range, 0.8, 1.0)?;
        self.validate_range("docas_beta_range", self.docas_beta_range, 0.0, 0.5)
    }

    fn validate_genome(&self) -> Result<(), ValidationError> {
        if !(0.3..=0.7).contains(&self.heritability_h2) {
            return Err(ValidationError {
                field: "heritability_h2".into(),
                message: format!("must be in [0.3, 0.7], got {}", self.heritability_h2),
            });
        }
        self.validate_range(
            "pde4b_variance_optimal",
            self.pde4b_variance_optimal,
            0.0,
            0.5,
        )
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn validate_game_theory(&self) -> Result<(), ValidationError> {
        self.validate_rate("cooperation_threshold_pd", self.cooperation_threshold_pd)?;
        self.validate_range(
            "cooperation_rate_target",
            self.cooperation_rate_target,
            0.0,
            1.0,
        )?;
        self.validate_range("system2_proportion", self.system2_proportion, 0.0, 1.0)
    }

    fn validate_economics(&self) -> Result<(), ValidationError> {
        self.validate_range(
            "gini_coefficient_target",
            self.gini_coefficient_target,
            0.0,
            1.0,
        )?;
        self.validate_rate("credit_access_threshold", self.credit_access_threshold)?;
        self.validate_rate("bond_density_threshold", self.bond_density_threshold)
    }

    fn validate_recovery(&self) -> Result<(), ValidationError> {
        if self.identity_theft_recovery_min >= self.identity_theft_recovery_max {
            return Err(ValidationError {
                field: "identity_theft_recovery".into(),
                message: format!(
                    "min must be < max, got {} and {}",
                    self.identity_theft_recovery_min, self.identity_theft_recovery_max
                ),
            });
        }
        Ok(())
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn validate_pom(&self) -> Result<(), ValidationError> {
        self.validate_rate("docas_c1_heritability", self.docas_c1_heritability)?;
        self.validate_range(
            "tfp_c3_cross_domain_coherence",
            self.tfp_c3_cross_domain_coherence,
            0.0,
            1.0,
        )?;
        self.validate_range(
            "econ_c1_gini_trust_correlation",
            self.econ_c1_gini_trust_correlation,
            -1.0,
            0.0,
        )?;
        self.validate_range(
            "id_theft_001_theft_rate",
            self.id_theft_001_theft_rate,
            0.0,
            1.0,
        )?;
        self.validate_range(
            "id_theft_003_protection_reduction",
            self.id_theft_003_protection_reduction,
            0.0,
            1.0,
        )
    }

    fn validate_rate(&self, field: &str, value: f32) -> Result<(), ValidationError> {
        if !(0.0..=1.0).contains(&value) {
            return Err(ValidationError {
                field: field.into(),
                message: format!("rate must be in [0, 1], got {value}"),
            });
        }
        Ok(())
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    fn validate_range(
        &self,
        field: &str,
        range: (f32, f32),
        min: f32,
        max: f32,
    ) -> Result<(), ValidationError> {
        if range.0 < min || range.1 > max || range.0 >= range.1 {
            return Err(ValidationError {
                field: field.into(),
                message: format!("must have {min} ≤ min < max ≤ {max}, got {range:?}"),
            });
        }
        Ok(())
    }

    /// Get identity theft recovery duration (random within range)
    pub fn random_recovery_duration(&self, rng: &mut impl rand::Rng) -> Tick {
        rng.gen_range(self.identity_theft_recovery_min..=self.identity_theft_recovery_max)
    }

    /// Check if theft rate is within POM target
    #[must_use]
    pub fn is_theft_rate_valid(&self, rate: f32) -> bool {
        rate >= self.id_theft_001_theft_rate.0 && rate <= self.id_theft_001_theft_rate.1
    }

    /// Check if protocol protection is within target
    #[must_use]
    pub fn is_protection_effectiveness_valid(&self, effectiveness: f32) -> bool {
        effectiveness >= self.id_theft_003_protection_reduction.0
            && effectiveness <= self.id_theft_003_protection_reduction.1
    }
}
