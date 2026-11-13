// TAG: surface=simulation owner=research-team rule=SIM-001
//! Agent psychology, DOCAS neuromodulator updates, and wearable biomarker dynamics

/// Psychology module status check
#[inline]
#[must_use]
#[allow(dead_code)]
pub const fn is_psychology_active() -> bool {
    true
}

use super::*;
use crate::agent::Agent;

/// Returns the active agent psychology engine version.
#[inline]
#[must_use]
#[allow(dead_code)]
pub const fn psychology_engine_version() -> &'static str {
    "0.1.0"
}

impl Simulation {
    #[inline]
    pub(crate) fn calculate_social_validation(agent: &Agent) -> f32 {
        let total_interactions = agent.positive_interactions + agent.negative_interactions;
        if total_interactions == 0 {
            (agent.reputation_score - 0.5) * 1.5
        } else {
            let ratio = agent.positive_interactions as f32 / total_interactions as f32;
            (ratio - 0.5) * 2.0
        }
    }

    #[inline]
    pub(crate) fn calculate_foraging_reward_error(resource_at_pos: f32, agent_energy: f32) -> f32 {
        ((resource_at_pos / 4.0 - 0.25) + (agent_energy - 40.0) / 80.0).clamp(-1.0, 1.0)
    }

    #[inline]
    pub(crate) fn apply_weather_cognitive_modifier(
        agent: &mut Agent,
        config: &SimulationConfig,
        env: &Environment,
    ) {
        if config.environment.weather_enabled {
            let weather_bandwidth_mod = if config.weather_shield_enabled {
                1.0
            } else {
                env.weather.cognitive_bandwidth_weather_modifier()
            };
            agent.cognitive_bandwidth =
                (agent.cognitive_bandwidth * weather_bandwidth_mod).clamp(0.05, 1.0);
        }
    }

    #[inline]
    pub(crate) fn update_agent_wearable_biomarkers(
        agent: &mut Agent,
        circadian: &crate::circadian::CircadianSystem,
        resource_at_pos: f32,
        reward_error: f32,
    ) {
        agent.biomarkers.update(
            &agent.docas,
            circadian,
            resource_at_pos,
            reward_error,
            agent.cognitive_bandwidth,
        );
    }

    pub(crate) fn update_docas_and_psychology(&mut self) {
        let mut rng = self
            .rng
            .take()
            .unwrap_or_else(|| StdRng::seed_from_u64(self.config.seed));

        let total_agents = self.agents.len();
        for agent in &mut self.agents {
            if !agent.alive {
                continue;
            }

            agent.update_geometric_properties();

            let decile = ((agent.cognitive_bandwidth.clamp(0.0, 1.0) * 10.0).floor() as usize + 1)
                .clamp(1, 10);
            let is_perturbed =
                self.config
                    .optogenetics
                    .is_agent_perturbed(agent.id, decile, total_agents);

            let resource_at_pos = self.environment.resource_at(&agent.position).unwrap_or(0.0);
            let raw_reward_error =
                Self::calculate_foraging_reward_error(resource_at_pos, agent.energy);
            let reward_error = self
                .config
                .optogenetics
                .modulate_dopamine_update(raw_reward_error, is_perturbed);

            let social_validation = Self::calculate_social_validation(agent);

            let (threat_signal, raw_stability) = Self::compute_threat_and_stability(
                agent,
                &self.config,
                &self.environment,
                is_perturbed,
                resource_at_pos,
            );

            // Center stimuli around homeostatic 0 so leaky integrator never pins at extremes
            let centered_threat = (threat_signal - 0.25) * 2.0;
            let centered_stability = (raw_stability - 0.5) * 2.0;
            let action_urgency = ((50.0 - agent.energy) / 50.0).clamp(-1.0, 1.0);

            agent.docas.update_with_dials(
                &agent.genome,
                &self.config.toggles.dials,
                reward_error,
                social_validation,
                centered_threat,
                action_urgency,
                centered_stability,
            );

            // Update wet-lab & IoT wearable biomarkers (Apple Watch HRV/Sleep, Muse EEG, Stelo CGM)
            if self.config.toggles.biomarkers_enabled {
                Self::update_agent_wearable_biomarkers(
                    agent,
                    &self.circadian,
                    resource_at_pos,
                    reward_error,
                );
            }

            // Sim D17: Weather degradation of cognitive verification bandwidth
            Self::apply_weather_cognitive_modifier(agent, &self.config, &self.environment);

            if self.config.trust_curvature_window > 0 {
                agent.trust_curvature.window_size = self.config.trust_curvature_window.clamp(3, 50);
            }

            let master_law_config = Self::compute_agent_master_law(
                agent,
                &self.config,
                &self.environment,
                is_perturbed,
            );

            agent.update_trust_master_law(social_validation, &master_law_config);

            Self::update_agent_affect_and_epigenetics(agent, &self.config, self.tick, &mut rng);
        }

        self.rng = Some(rng);
    }
}
