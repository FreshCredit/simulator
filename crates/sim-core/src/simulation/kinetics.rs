// TAG: surface=simulation owner=research-team rule=SIM-001
//! Agent kinetics, master law presets, environmental stimulus, and epigenetics modulation

use super::*;
use crate::agent::Agent;

impl Simulation {
    #[inline]
    pub(crate) fn compute_raw_threat(energy: f32) -> f32 {
        if energy < 30.0 {
            (30.0 - energy) / 30.0
        } else {
            0.0
        }
    }

    #[inline]
    pub(crate) fn compute_thermal_cortisol_bump(
        config: &SimulationConfig,
        environment: &Environment,
    ) -> f32 {
        if config.environment.weather_enabled && !config.weather_shield_enabled {
            let temp = environment.weather.temperature();
            if temp > 24.0 {
                ((temp - 24.0) * 0.015).min(0.35)
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    #[inline]
    fn compute_weather_photic_drive(phi_lux: f32, energy: f32) -> f32 {
        let energy_bonus = if energy > 60.0 { 0.3 } else { 0.0 };
        (0.25 + 0.45 * phi_lux + energy_bonus).clamp(0.1, 1.0)
    }

    #[inline]
    pub(crate) fn compute_raw_stability(
        config: &SimulationConfig,
        environment: &Environment,
        energy: f32,
    ) -> f32 {
        if config.environment.weather_enabled {
            let phi_lux = environment.weather.photic_lux_drive();
            Self::compute_weather_photic_drive(phi_lux, energy)
        } else if energy > 60.0 {
            0.7
        } else {
            0.3
        }
    }

    #[inline]
    fn compute_optogenetic_threat(raw_threat: f32, photo_current: f32) -> f32 {
        (raw_threat + 1.5 * photo_current).min(1.0)
    }

    #[inline]
    fn compute_effective_threat_signal(raw_threat: f32, thermal_bump: f32) -> f32 {
        (raw_threat + thermal_bump).clamp(0.0, 1.0)
    }

    #[inline]
    pub(crate) fn apply_sleep_deficit(
        config: &SimulationConfig,
        threat: f32,
        stability: f32,
    ) -> (f32, f32) {
        if config.circadian_sleep_deficit > 0.0 {
            let stab = (stability * (1.0 - config.circadian_sleep_deficit * 0.5)).clamp(0.05, 1.0);
            let thr = (threat + config.circadian_sleep_deficit * 0.25).min(1.0);
            (thr, stab)
        } else {
            (threat, stability)
        }
    }

    pub(crate) fn compute_threat_and_stability(
        agent: &Agent,
        config: &SimulationConfig,
        environment: &Environment,
        is_perturbed: bool,
        _resource_at_pos: f32,
    ) -> (f32, f32) {
        let raw_threat = Self::compute_raw_threat(agent.energy);
        let threat_signal = if is_perturbed
            && config.optogenetics.target_channels[crate::types::C]
            && config.optogenetics.photo_current() > 0.0
        {
            Self::compute_optogenetic_threat(raw_threat, config.optogenetics.photo_current())
        } else {
            raw_threat
        };

        let thermal_cortisol_bump = Self::compute_thermal_cortisol_bump(config, environment);
        let threat_signal =
            Self::compute_effective_threat_signal(threat_signal, thermal_cortisol_bump);
        let stability_signal = Self::compute_raw_stability(config, environment, agent.energy);

        Self::apply_sleep_deficit(config, threat_signal, stability_signal)
    }

    #[inline]
    pub(crate) fn select_master_law_preset(
        config: &SimulationConfig,
    ) -> crate::master_law::MasterLawPreset {
        if config.toggles.protocol_enabled {
            crate::master_law::MasterLawPreset::ProtocolAsymmetric
        } else if !config.toggles.genome_enabled {
            crate::master_law::MasterLawPreset::PhysicsLeakFree
        } else {
            crate::master_law::MasterLawPreset::BiologyHomeostatic
        }
    }

    #[inline]
    pub(crate) fn apply_baylor_unconscious_shift(
        agent: &Agent,
        config: &SimulationConfig,
        master_law_config: &mut crate::master_law::MasterLawConfig,
    ) {
        if config.baylor.mode == crate::baylor_gating::ConsciousnessMode::Unconscious {
            let cortisol_val = agent.docas.levels[crate::types::C];
            if cortisol_val < config.baylor.cortisol_crit {
                master_law_config.lambda_minus = 0.0;
            }
            let shift = config.baylor.attractor_shift(
                agent.docas.levels[crate::types::O],
                agent.docas.levels[crate::types::S],
            );
            master_law_config.s_base += shift as f64;
        }
    }

    #[inline]
    pub(crate) fn apply_optogenetic_conductance_inhibition(
        config: &SimulationConfig,
        is_perturbed: bool,
        master_law_config: &mut crate::master_law::MasterLawConfig,
    ) {
        if is_perturbed
            && config.optogenetics.target_channels[crate::types::D]
            && config.optogenetics.photo_current() < 0.0
        {
            let g = config.optogenetics.conductance;
            master_law_config.lambda_plus *= (1.0 - g).max(0.0) as f64;
        }
    }

    pub(crate) fn compute_agent_master_law(
        agent: &Agent,
        config: &SimulationConfig,
        environment: &Environment,
        is_perturbed: bool,
    ) -> crate::master_law::MasterLawConfig {
        let preset = Self::select_master_law_preset(config);
        let mut master_law_config = crate::master_law::MasterLawConfig::for_preset(preset);

        if config.docas_alpha_base > 0.0 {
            let gain_scale = (config.docas_alpha_base as f64 * 2.0).clamp(0.1, 4.0);
            master_law_config.lambda_plus *= gain_scale;
            master_law_config.lambda_minus *= gain_scale;
        }
        if config.trust_decay_rate > 0.0 {
            master_law_config.leak_delta = (config.trust_decay_rate as f64).clamp(0.0, 0.99);
        }

        if config.environment.weather_enabled {
            master_law_config.s_base = environment
                .weather
                .environmental_baseline_drift(master_law_config.s_base as f32)
                as f64;
        }

        Self::apply_baylor_unconscious_shift(agent, config, &mut master_law_config);
        Self::apply_optogenetic_conductance_inhibition(
            config,
            is_perturbed,
            &mut master_law_config,
        );

        master_law_config
    }

    pub(crate) fn update_agent_affect_and_epigenetics(
        agent: &mut Agent,
        config: &SimulationConfig,
        tick: u64,
        rng: &mut StdRng,
    ) {
        agent.update_psychology();
        if config.emotion_decay_rate > 0.0 {
            let retention = config.emotion_decay_rate.clamp(0.01, 0.99);
            agent.affect_valence *= retention;
            agent.affect_arousal *= retention;
        }

        if config.toggles.philosophy_enabled {
            agent.update_philosophy(tick);
            if config.worldview_drift_rate > 0.0 {
                if let Some(ref mut phil) = agent.philosophical_state {
                    let rate = config.worldview_drift_rate.clamp(0.0001, 1.0);
                    phil.tau_phi = (phil.tau_phi * (1.0 - rate * 0.1)).clamp(-1.0, 1.0);
                    phil.worldview.trust_axis =
                        (phil.worldview.trust_axis * (1.0 - rate * 0.1)).clamp(-1.0, 1.0);
                }
            }
        } else if let Some(ref mut phil) = agent.philosophical_state {
            phil.tau_phi = 0.0;
            phil.worldview.trust_axis = 0.0;
        }

        agent.update_epigenetics(tick);
        agent.update_environmental_epigenetics(tick, rng);
        if config.epigenetic_methylation_rate > 0.0 {
            let extra_cycles = (config.epigenetic_methylation_rate * 4.0) as usize;
            for _ in 0..extra_cycles {
                agent.update_environmental_epigenetics(tick, rng);
            }
        }
    }
}
