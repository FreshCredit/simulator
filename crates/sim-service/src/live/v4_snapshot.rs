// TAG: surface=simulation owner=research-team rule=SIM-001
//! V4 live telemetry and lexicon summaries for simulation frames

use super::*;
use sim_core::Simulation;

/// Returns the active V4 telemetry snapshot builder version.
#[inline]
#[must_use]
pub const fn v4_snapshot_version() -> &'static str {
    "0.1.0"
}

/// Default telemetry wavelength constant (590nm amber laser)
#[inline]
#[must_use]
pub const fn default_telemetry_wavelength() -> u32 {
    590
}

/// Default telemetry opto conductance
#[inline]
#[must_use]
pub const fn default_opto_conductance() -> f32 {
    1.0
}

/// Build V4 telemetry snapshot from simulation state and evaluated gradients
pub fn build_v4_telemetry(
    sim: &Simulation,
    decile_grad: &sim_core::visual_cloaking::DecileVulnerabilityGradient,
    simplex: [f32; 5],
    kuramoto_r: f32,
) -> V4TelemetrySnapshot {
    let avg_cortisol = simplex[sim_core::types::C];
    let is_unconscious =
        sim.config.baylor.mode == sim_core::baylor_gating::ConsciousnessMode::Unconscious;
    let cognitive_friction = sim.config.baylor.cognitive_friction(avg_cortisol);
    let executive_gate = sim.config.baylor.executive_threat_gate(avg_cortisol);
    let attractor_shift = sim
        .config
        .baylor
        .attractor_shift(simplex[sim_core::types::O], simplex[sim_core::types::S]);
    let active_branch = SIMULATION_PARAMS
        .read()
        .brain_it_branch
        .clone()
        .unwrap_or_else(|| "BranchAStructural".to_string());

    V4TelemetrySnapshot {
        decile_uptake_rates: decile_grad.decile_uptake_rates,
        odds_ratio_d1_vs_d10: decile_grad.odds_ratio_d1_vs_d10,
        kuramoto_coherence_r: kuramoto_r,
        simplex_projection: simplex,
        active_cloaking_ratio: sim.config.visual_cloaking.cloaking_ratio,
        active_brain_it_branch: active_branch,
        laser_wavelength: format!("{:?}", sim.config.optogenetics.wavelength),
        target_channels: sim.config.optogenetics.target_channels,
        weather_lux: sim.environment.weather.photic_lux_drive(),
        weather_thermal_stress: sim.environment.weather.thermal_stress_factor(),
        weather_shield_active: sim.config.weather_shield_enabled,
        opto_conductance: sim.config.optogenetics.conductance,
        baylor_unconscious_mode: is_unconscious,
        baylor_cognitive_friction: cognitive_friction,
        baylor_executive_gate: executive_gate,
        baylor_attractor_shift: attractor_shift,
    }
}

/// Build condensed lexicon summary and colexification graph from an agent
pub fn build_lexicon_summary(agent: &sim_core::Agent) -> Option<LexiconSummary> {
    if agent.lexicon.concepts.is_empty() {
        return None;
    }
    let top_cats: Vec<String> = agent
        .lexicon
        .concepts
        .values()
        .take(5)
        .map(|c| format!("{:?}", c.category))
        .collect();

    let [d, o, c, a, s] = agent.docas.levels;
    let tau = agent.compute_trust();
    let concept_names: std::collections::HashMap<u64, &str> = [
        (0, "Reliability"),
        (1, "Honesty"),
        (2, "Verification"),
        (3, "Bond"),
        (10, "Cooperation"),
        (11, "Defection"),
        (12, "Group"),
        (13, "Norm"),
        (20, "Food"),
        (21, "Energy"),
        (22, "Territory"),
        (30, "Joy"),
        (31, "Fear"),
        (32, "Anger"),
        (33, "Sadness"),
        (40, "Move"),
        (41, "Trade"),
        (42, "Share"),
        (43, "Attack"),
    ]
    .into_iter()
    .collect();

    let concepts = agent
        .lexicon
        .concepts
        .values()
        .map(|con| {
            let name = concept_names.get(&con.id).copied().unwrap_or("Concept");
            let freq = match con.id {
                0 | 1 | 2 | 3 => 1.0 + o * 1.5 + s * 1.0,
                10 | 12 | 13 | 42 => 1.0 + o * 2.0,
                11 | 43 => 1.0 + (1.0 - tau) * 2.0 + a * 1.5,
                30 => 1.0 + d * 2.0,
                31 => 1.0 + c * 2.5,
                32 => 1.0 + a * 2.0,
                33 => 1.0 + c * 1.5,
                20 | 21 | 22 | 41 => 1.0 + (100.0 - agent.energy).max(0.0) / 50.0,
                _ => 1.0,
            };
            LexiconConceptSnapshot {
                id: con.id,
                name: name.to_string(),
                category: format!("{:?}", con.category),
                embedding: con.embedding.values.to_vec(),
                frequency: freq,
                created_at: con.learned_tick,
            }
        })
        .collect();

    let colexifications = agent
        .lexicon
        .colexification_map
        .iter()
        .map(|&(c1, c2)| (c1, c2, 0.8))
        .collect();

    let colexification_map = agent
        .lexicon
        .colexification_map
        .iter()
        .map(|&(c1, c2)| (c1, c2))
        .collect();

    Some(LexiconSummary {
        concept_count: agent.lexicon.concepts.len(),
        drift_rate: agent.language_drift.aggregate_drift_rate,
        top_categories: top_cats,
        concepts,
        colexifications,
        colexification_map,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_lexicon_summary_empty() {
        let genome = sim_core::genome::Genome::default();
        let agent = sim_core::agent::Agent::new(
            1,
            sim_core::types::Position::new(0.0, 0.0, 0.0),
            genome,
            0,
            sim_core::types::Sex::Female,
        );
        assert!(build_lexicon_summary(&agent).is_none());
    }

    #[test]
    fn test_build_v4_telemetry_snapshot() {
        let sim = sim_core::Simulation::new(sim_core::simulation::SimulationConfig::default());
        let grad = sim_core::visual_cloaking::DecileVulnerabilityGradient::evaluate_population(
            &sim.config.visual_cloaking,
            &[0.5; 10],
            false,
        );
        let snap = build_v4_telemetry(&sim, &grad, [0.2; 5], 0.75);
        assert_eq!(snap.kuramoto_coherence_r, 0.75);
        assert_eq!(snap.simplex_projection, [0.2; 5]);
    }
}
