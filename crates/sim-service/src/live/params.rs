//! Live simulation parameter overrides and dials

use serde::{Deserialize, Serialize};

/// Configurable simulation parameters
/// These can be updated via the restart API to start a new simulation with custom settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationParams {
    /// resource density
    pub resource_density: f32,
    /// initial resource
    pub initial_resource: f32,
    /// agent count
    pub agent_count: u32,
    /// grid width
    pub grid_width: u32,
    /// grid height
    pub grid_height: u32,
    /// Custom toggle configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toggles: Option<sim_core::toggles::ToggleConfig>,
    /// Selected simulation preset ("physics", "biology", "protocol", "reconciled", "stage_0".."stage_5")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
    /// Maximum ticks to run
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_ticks: Option<u64>,
    /// Layer 4: DOCAS stimulus response rate
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docas_alpha_base: Option<f32>,
    /// Layer 4: Trust curvature window size
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trust_curvature_window: Option<usize>,
    /// Layer 5: Planning horizon
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planning_horizon: Option<u64>,
    /// Layer 5: Emotion decay rate
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emotion_decay_rate: Option<f32>,
    /// Layer 6: Worldview drift rate
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worldview_drift_rate: Option<f32>,
    /// Layer 7: Trust decay rate
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trust_decay_rate: Option<f32>,
    /// Layer 7: Reputation threshold
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reputation_threshold: Option<f32>,
    /// Layer 8: Credit limit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credit_limit: Option<f32>,
    /// Layer 8: Base interest rate
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interest_rate_base: Option<f32>,
    /// Layer 8: Transaction cost
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_cost: Option<f32>,
    /// Layer 9: Block time in ticks
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_time: Option<u64>,
    /// Layer 9: Consensus threshold
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consensus_threshold: Option<f32>,
    /// Movement speed multiplier
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub movement_speed: Option<f32>,
    /// Social force influence
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub social_force_influence: Option<f32>,
    /// Version 4.0: Optogenetic laser wavelength
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opto_wavelength: Option<String>,
    /// Version 4.0: Target optogenetic channels [D, O, C, A, S]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opto_target_channels: Option<[bool; 5]>,
    /// Version 4.0: Cohort scope
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opto_scope: Option<String>,
    /// Version 4.0: Custom subpopulation %
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opto_subpop_pct: Option<f32>,
    /// Version 4.0: Visual cloaking ratio chi
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visual_cloaking_ratio: Option<f32>,
    /// Version 4.0: Brain-IT Branch mode
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brain_it_branch: Option<String>,
    /// Version 4.0: Weather-Shield enabled
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weather_shield_enabled: Option<bool>,
    /// Version 4.0: Ambient temperature in Celsius
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weather_temperature: Option<f32>,
    /// Version 4.0: Melanopsin 480nm enabled
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub melanopsin_enabled: Option<bool>,
    /// Version 4.0: Baylor unconscious mode
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baylor_unconscious_mode: Option<bool>,
    /// Kingman coalescence intensity kappa*
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kingman_coalescent: Option<f32>,
    /// Lineage depth
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lineage_depth: Option<usize>,
    /// Music metric hierarchy ratio tau
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub music_metric_hierarchy_tau: Option<f32>,
    /// Altitude effects enabled
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub altitude_effects_enabled: Option<bool>,
    /// Weather enabled
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weather_enabled: Option<bool>,
    /// Energy consumption rate
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy_consumption: Option<f32>,
    /// Movement cost
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub movement_cost: Option<f32>,
    /// Aging beta
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aging_beta: Option<f32>,
    /// Reproduction threshold
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repro_threshold: Option<f32>,
    /// Mutation rate
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mutation_rate: Option<f32>,
    /// Language semantic drift rate factor
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_drift_rate_factor: Option<f32>,
    /// Language coherence threshold
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_coherence_threshold: Option<f32>,
    /// Civilization Challenge: Epigenetic methylation rate [0.0, 1.0]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub epigenetic_methylation_rate: Option<f32>,
    /// Civilization Challenge: Circadian sleep deficit [0.0, 1.0]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circadian_sleep_deficit: Option<f32>,
    /// Civilization Challenge: Wealth redistribution tax rate [0.0, 0.50]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tax_redistribution_rate: Option<f32>,
    /// Civilization Challenge: Bank run sunspot sensitivity [0.0, 2.0]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bank_run_sensitivity: Option<f32>,
}

impl Default for SimulationParams {
    fn default() -> Self {
        Self {
            resource_density: 0.30,
            initial_resource: 50.0,
            agent_count: 20,
            grid_width: 100,
            grid_height: 100,
            toggles: None,
            preset: None,
            max_ticks: None,
            docas_alpha_base: None,
            trust_curvature_window: None,
            planning_horizon: None,
            emotion_decay_rate: None,
            worldview_drift_rate: None,
            trust_decay_rate: None,
            opto_wavelength: None,
            opto_target_channels: None,
            opto_scope: None,
            opto_subpop_pct: None,
            visual_cloaking_ratio: None,
            brain_it_branch: None,
            weather_shield_enabled: None,
            weather_temperature: None,
            melanopsin_enabled: None,
            baylor_unconscious_mode: None,
            kingman_coalescent: None,
            lineage_depth: None,
            music_metric_hierarchy_tau: None,
            altitude_effects_enabled: None,
            weather_enabled: None,
            energy_consumption: None,
            movement_cost: None,
            aging_beta: None,
            repro_threshold: None,
            mutation_rate: None,
            language_drift_rate_factor: None,
            language_coherence_threshold: None,
            epigenetic_methylation_rate: None,
            circadian_sleep_deficit: None,
            tax_redistribution_rate: None,
            bank_run_sensitivity: None,

            reputation_threshold: None,
            credit_limit: None,
            interest_rate_base: None,
            transaction_cost: None,
            block_time: None,
            consensus_threshold: None,
            movement_speed: None,
            social_force_influence: None,
        }
    }
}

use sim_core::simulation::SimulationConfig;

/// Assign value from Option if Some
#[inline]
pub fn set_if_some<T>(dst: &mut T, src: Option<T>) {
    if let Some(v) = src {
        *dst = v;
    }
}

/// Apply simulation parameter overrides to SimulationConfig
pub fn apply_config_overrides(cfg: &mut SimulationConfig, params: &SimulationParams) {
    set_if_some(&mut cfg.toggles, params.toggles);
    set_if_some(
        &mut cfg.weather_shield_enabled,
        params.weather_shield_enabled,
    );
    if let Some(chi) = params.visual_cloaking_ratio {
        cfg.visual_cloaking =
            sim_core::visual_cloaking::VisualAgreementStimulus::with_cloaking_ratio(chi);
    }
    if let Some(unconscious) = params.baylor_unconscious_mode {
        cfg.baylor.mode = if unconscious {
            sim_core::baylor_gating::ConsciousnessMode::Unconscious
        } else {
            sim_core::baylor_gating::ConsciousnessMode::Conscious
        };
    }
    set_if_some(&mut cfg.energy_consumption_rate, params.energy_consumption);
    set_if_some(&mut cfg.movement_cost, params.movement_cost);
    set_if_some(&mut cfg.mortality_aging_beta, params.aging_beta);
    set_if_some(&mut cfg.reproduction_threshold, params.repro_threshold);
    set_if_some(&mut cfg.mutation_rate, params.mutation_rate);
    set_if_some(
        &mut cfg.tax_redistribution_rate,
        params.tax_redistribution_rate,
    );
    set_if_some(&mut cfg.bank_run_sensitivity, params.bank_run_sensitivity);
    set_if_some(&mut cfg.docas_alpha_base, params.docas_alpha_base);
    set_if_some(
        &mut cfg.trust_curvature_window,
        params.trust_curvature_window,
    );
    set_if_some(&mut cfg.trust_decay_rate, params.trust_decay_rate);
    set_if_some(&mut cfg.reputation_threshold, params.reputation_threshold);
    set_if_some(&mut cfg.credit_limit, params.credit_limit);
    set_if_some(&mut cfg.interest_rate_base, params.interest_rate_base);
    set_if_some(
        &mut cfg.environment.altitude_effects_enabled,
        params.altitude_effects_enabled,
    );
    set_if_some(&mut cfg.environment.weather_enabled, params.weather_enabled);
    set_if_some(&mut cfg.planning_horizon, params.planning_horizon);
    set_if_some(&mut cfg.emotion_decay_rate, params.emotion_decay_rate);
    set_if_some(&mut cfg.worldview_drift_rate, params.worldview_drift_rate);
    set_if_some(&mut cfg.transaction_cost, params.transaction_cost);
    set_if_some(&mut cfg.block_time, params.block_time);
    set_if_some(&mut cfg.consensus_threshold, params.consensus_threshold);
    set_if_some(
        &mut cfg.epigenetic_methylation_rate,
        params.epigenetic_methylation_rate,
    );
    set_if_some(
        &mut cfg.circadian_sleep_deficit,
        params.circadian_sleep_deficit,
    );
    set_if_some(&mut cfg.movement_speed, params.movement_speed);
    set_if_some(
        &mut cfg.social_force_influence,
        params.social_force_influence,
    );
    set_if_some(
        &mut cfg.toggles.dials.kingman_coalescent,
        params.kingman_coalescent,
    );
    set_if_some(&mut cfg.toggles.dials.lineage_depth, params.lineage_depth);
    set_if_some(
        &mut cfg.toggles.dials.music_metric_hierarchy_tau,
        params.music_metric_hierarchy_tau,
    );
    set_if_some(
        &mut cfg.toggles.dials.language_drift_rate_factor,
        params.language_drift_rate_factor,
    );
    set_if_some(
        &mut cfg.toggles.dials.language_coherence_threshold,
        params.language_coherence_threshold,
    );
}

/// Update Option destination from source if Some
#[inline]
pub fn update_opt<T>(dst: &mut Option<T>, src: Option<T>) {
    if let Some(v) = src {
        *dst = Some(v);
    }
}

/// Update simulation parameter fields from incoming parameter delta
pub fn update_simulation_params(dst: &mut SimulationParams, src: &SimulationParams) {
    update_opt(&mut dst.planning_horizon, src.planning_horizon);
    update_opt(&mut dst.emotion_decay_rate, src.emotion_decay_rate);
    update_opt(&mut dst.worldview_drift_rate, src.worldview_drift_rate);
    update_opt(&mut dst.transaction_cost, src.transaction_cost);
    update_opt(&mut dst.block_time, src.block_time);
    update_opt(&mut dst.consensus_threshold, src.consensus_threshold);
    update_opt(
        &mut dst.epigenetic_methylation_rate,
        src.epigenetic_methylation_rate,
    );
    update_opt(
        &mut dst.circadian_sleep_deficit,
        src.circadian_sleep_deficit,
    );
    update_opt(&mut dst.kingman_coalescent, src.kingman_coalescent);
    update_opt(&mut dst.lineage_depth, src.lineage_depth);
    update_opt(
        &mut dst.music_metric_hierarchy_tau,
        src.music_metric_hierarchy_tau,
    );
    update_opt(
        &mut dst.language_drift_rate_factor,
        src.language_drift_rate_factor,
    );
    update_opt(
        &mut dst.language_coherence_threshold,
        src.language_coherence_threshold,
    );
    update_opt(&mut dst.opto_wavelength, src.opto_wavelength.clone());
    update_opt(&mut dst.opto_target_channels, src.opto_target_channels);
    update_opt(&mut dst.opto_scope, src.opto_scope.clone());
    update_opt(&mut dst.opto_subpop_pct, src.opto_subpop_pct);
    update_opt(&mut dst.visual_cloaking_ratio, src.visual_cloaking_ratio);
    update_opt(&mut dst.brain_it_branch, src.brain_it_branch.clone());
    update_opt(&mut dst.weather_shield_enabled, src.weather_shield_enabled);
    update_opt(&mut dst.weather_temperature, src.weather_temperature);
    update_opt(&mut dst.melanopsin_enabled, src.melanopsin_enabled);
    update_opt(
        &mut dst.baylor_unconscious_mode,
        src.baylor_unconscious_mode,
    );
    update_opt(&mut dst.toggles, src.toggles);
    update_opt(&mut dst.energy_consumption, src.energy_consumption);
    update_opt(&mut dst.movement_cost, src.movement_cost);
    update_opt(&mut dst.aging_beta, src.aging_beta);
    update_opt(&mut dst.repro_threshold, src.repro_threshold);
    update_opt(&mut dst.mutation_rate, src.mutation_rate);
    update_opt(
        &mut dst.tax_redistribution_rate,
        src.tax_redistribution_rate,
    );
    update_opt(&mut dst.bank_run_sensitivity, src.bank_run_sensitivity);
    update_opt(&mut dst.docas_alpha_base, src.docas_alpha_base);
    update_opt(&mut dst.trust_curvature_window, src.trust_curvature_window);
    update_opt(&mut dst.trust_decay_rate, src.trust_decay_rate);
    update_opt(&mut dst.reputation_threshold, src.reputation_threshold);
    update_opt(&mut dst.credit_limit, src.credit_limit);
    update_opt(&mut dst.interest_rate_base, src.interest_rate_base);
    update_opt(
        &mut dst.altitude_effects_enabled,
        src.altitude_effects_enabled,
    );
    update_opt(&mut dst.weather_enabled, src.weather_enabled);
}
