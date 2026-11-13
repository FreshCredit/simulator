//! Neural Brain Visualization API
//!
//! Provides 3D brain region activation data for Three.js visualization.
//! Maps DOCAS levels to anatomical brain regions (MNI coordinates).

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::live::LiveSimulationState;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Brain region activation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainRegionActivation {
    /// Region name (e.g., "vta", "amygdala")
    pub region: String,
    /// MNI coordinates [x, y, z] in mm
    pub coordinates: [f32; 3],
    /// Activation level [0.0, 1.0]
    pub activation: f32,
    /// Associated neuromodulator
    pub neuromodulator: String,
    /// Display color (hex)
    pub color: String,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Complete neural brain state for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralBrainState {
    /// Agent ID
    pub agent_id: u64,
    /// DOCAS levels [D, O, C, A, S]
    pub docas: [f32; 5],
    /// Brain region activations
    pub regions: Vec<BrainRegionActivation>,
    /// Timestamp (simulation tick)
    pub tick: u64,
    /// Active contract stimulus ID (Sim D21 / TRIBE v2 battery)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stimulus_id: Option<String>,
    /// Stimulus category (FAIR, PREDATORY, TEASER, TRUST_SIGNAL, THREAT, CONTROL)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stimulus_category: Option<String>,
    /// Disclosure condition ("Transparent" vs "MultimodalCloaked")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cloaking_condition: Option<String>,
    /// Anterior Insula BOLD suppression rate
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ains_suppression_rate: Option<f32>,
    /// Auditory disclaimer speech rate in syllables/sec
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speech_rate_syl_sec: Option<f32>,
}

/// Brain region definitions with MNI coordinates
/// Based on standard anatomical atlases and Sim D21 Meta TRIBE v2 fMRI regions
const BRAIN_REGIONS: &[(&str, [f32; 3], &str, &str)] = &[
    // Dopamine system (reward)
    ("vta", [0.0, -20.0, -20.0], "dopamine", "#FFD700"), // Ventral Tegmental Area - gold
    ("nacc", [8.0, 8.0, -10.0], "dopamine", "#FFA500"),  // Nucleus Accumbens - orange
    ("sn", [-10.0, -20.0, -15.0], "dopamine", "#FF8C00"), // Substantia Nigra - dark orange
    // Oxytocin system (social)
    ("hypothalamus", [0.0, -4.0, -8.0], "oxytocin", "#FF1493"), // Hypothalamus - deep pink
    ("pvns", [0.0, -6.0, -6.0], "oxytocin", "#FF69B4"),         // PVN - hot pink
    // Cortisol system (stress)
    ("amygdala", [22.0, -6.0, -14.0], "cortisol", "#FF4500"), // Amygdala - red-orange
    ("pvn", [0.0, -6.0, -6.0], "cortisol", "#FF6347"),        // Paraventricular Nucleus - tomato
    ("hippocampus", [28.0, -22.0, -10.0], "cortisol", "#CD5C5C"), // Hippocampus - indian red
    // Adrenaline system (action/urgency)
    (
        "locus_coeruleus",
        [2.0, -36.0, -26.0],
        "adrenaline",
        "#FF8C00",
    ), // Locus Coeruleus - dark orange
    ("pituitary", [0.0, -10.0, -8.0], "adrenaline", "#FFA07A"), // Pituitary - light salmon
    // Serotonin system (stability)
    ("raphe_nuclei", [0.0, -30.0, -20.0], "serotonin", "#4169E1"), // Raphe Nuclei - royal blue
    ("drn", [0.0, -32.0, -22.0], "serotonin", "#6495ED"),          // Dorsal Raphe - cornflower blue
    // Integration & Cortical regions (Sim D21 TRIBE v2 Whole-Brain fMRI)
    ("prefrontal_cortex", [0.0, 50.0, 20.0], "mixed", "#9370DB"), // dlPFC (Valuation / Executive) - medium purple
    ("anterior_cingulate", [0.0, 12.0, 28.0], "mixed", "#8A2BE2"), // ACC (Conflict Monitoring) - blue violet
    ("insula", [38.0, 4.0, 4.0], "mixed", "#BA55D3"), // Anterior Insula (Norm Prediction Error) - medium orchid
    (
        "superior_temporal_gyrus",
        [54.0, -18.0, 6.0],
        "mixed",
        "#38BDF8",
    ), // STG (Auditory Phase-Locking) - sky blue
    (
        "default_mode_network",
        [0.0, -52.0, 26.0],
        "mixed",
        "#A855F7",
    ), // DMN (Counterfactual Deliberation) - purple
    ("visual_cortex", [12.0, -90.0, 0.0], "mixed", "#22C55E"), // Visual Cortex (V1/V4 Structural Margins) - green
];

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get neural brain state for a specific agent
///
/// Maps DOCAS levels and Sim D21 TRIBE v2 whole-brain fMRI encoding to cortical regions.
pub async fn get_agent_brain_state(
    State(state): State<LiveSimulationState>,
    Path(agent_id): Path<u64>,
) -> Json<NeuralBrainState> {
    let frame = state.current_frame.read();

    if let Some(ref current_frame) = *frame {
        if let Some(agent) = current_frame.agents.iter().find(|a| a.agent_id == agent_id) {
            let [d, o, c, a_level, s] = agent.docas;
            let live_cfg = crate::live::get_live_simulation_config();
            let cloaking_ratio = live_cfg.visual_cloaking.cloaking_ratio;
            let is_cloaked = cloaking_ratio > 1.5;

            let is_defaulted = !agent.alive && agent.energy <= 0.0;
            let is_predatory_threat = c > 0.35 || cloaking_ratio >= 2.5;

            // Sim D21 Meta TRIBE v2 BOLD values (battery27_multimodal_preds.npz)
            let (stim_id, stim_cat, ains_val, dlpfc_val, stg_val): (&str, &str, f32, f32, f32) =
                if is_defaulted {
                    ("PREDATORY_1", "PREDATORY", 0.5810 * 1.35, 0.0781, 0.1600)
                } else if is_predatory_threat {
                    if is_cloaked {
                        ("PREDATORY_1", "PREDATORY", 0.2324, 0.0781, 0.0800)
                    } else {
                        ("PREDATORY_1", "PREDATORY", 0.5810, 0.0781, 0.1603)
                    }
                } else if agent.tau > 0.8 {
                    ("FAIR_1", "FAIR", 0.0037, 0.0780, 0.1603)
                } else {
                    ("CONTROL_1", "CONTROL", 0.0018, 0.0714, 0.1602)
                };

            let condition_str = if is_defaulted {
                "ExPostDefaultShock"
            } else if is_cloaked {
                "MultimodalCloaked"
            } else {
                "Transparent"
            };

            let speech_rate: f32 = if is_cloaked { 11.8 } else { 4.5 };
            let suppression_rate: f32 = if is_cloaked { 0.59997 } else { 0.0 };

            let regions = BRAIN_REGIONS
                .iter()
                .map(|(name, coords, nm, color)| {
                    let mut activation = match *name {
                        "insula" => {
                            (ains_val * 0.70 + ((c + o) / 2.0) * 0.30).clamp(0.0f32, 1.0f32)
                        }
                        "prefrontal_cortex" => {
                            (dlpfc_val * 4.0 + (d + o + s) / 3.0 * 0.6).clamp(0.0f32, 1.0f32)
                        }
                        "superior_temporal_gyrus" => (stg_val * 4.5f32).clamp(0.0f32, 1.0f32),
                        "default_mode_network" => {
                            (0.40f32 + (c * 0.3) + (1.0 - agent.tau) * 0.3).clamp(0.0f32, 1.0f32)
                        }
                        "visual_cortex" => {
                            (0.35f32 + (cloaking_ratio / 5.0) * 0.45).clamp(0.0f32, 1.0f32)
                        }
                        _ => match *nm {
                            "dopamine" => (d * 0.8 + s * 0.2).clamp(0.0, 1.0),
                            "oxytocin" => (o * 0.9 + d * 0.1).clamp(0.0, 1.0),
                            "cortisol" => (c * 0.9 + a_level * 0.1).clamp(0.0, 1.0),
                            "adrenaline" => (a_level * 0.85 + c * 0.15).clamp(0.0, 1.0),
                            "serotonin" => (s * 0.9 + o * 0.1).clamp(0.0, 1.0),
                            "mixed" => match *name {
                                "anterior_cingulate" | "acc" => (c + a_level) / 2.0,
                                _ => 0.5,
                            }
                            .clamp(0.0, 1.0),
                            _ => 0.5,
                        },
                    };

                    if live_cfg.baylor.mode
                        == sim_core::baylor_gating::ConsciousnessMode::Unconscious
                        && c < live_cfg.baylor.cortisol_crit
                        && (*name == "prefrontal_cortex"
                            || *name == "anterior_cingulate"
                            || *name == "acc")
                    {
                        activation *= 0.08;
                    }

                    let display_color = if *name == "insula" && activation > 0.40 {
                        "#FF1E1E".to_string()
                    } else {
                        (*color).to_string()
                    };

                    BrainRegionActivation {
                        region: (*name).to_string(),
                        coordinates: *coords,
                        activation,
                        neuromodulator: (*nm).to_string(),
                        color: display_color,
                    }
                })
                .collect();

            return Json(NeuralBrainState {
                agent_id,
                docas: agent.docas,
                regions,
                tick: current_frame.tick,
                stimulus_id: Some(stim_id.to_string()),
                stimulus_category: Some(stim_cat.to_string()),
                cloaking_condition: Some(condition_str.to_string()),
                ains_suppression_rate: Some(suppression_rate),
                speech_rate_syl_sec: Some(speech_rate),
            });
        }
    }

    Json(NeuralBrainState {
        agent_id,
        docas: [0.0; 5],
        regions: vec![],
        tick: frame.as_ref().map_or(0, |f| f.tick),
        stimulus_id: None,
        stimulus_category: None,
        cloaking_condition: None,
        ains_suppression_rate: None,
        speech_rate_syl_sec: None,
    })
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Get population-level neural statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationNeuralStats {
    /// Mean DOCAS across population
    pub mean_docas: [f32; 5],
    /// Standard deviation of DOCAS
    pub std_docas: [f32; 5],
    /// Dominant neuromodulator (highest mean)
    pub dominant_nm: String,
    /// Brain region activations (population average)
    pub region_activations: Vec<BrainRegionActivation>,
    /// tick
    pub tick: u64,
}

/// get population neural stats
#[allow(clippy::too_many_lines)]
pub async fn get_population_neural_stats(
    State(state): State<LiveSimulationState>,
) -> Json<PopulationNeuralStats> {
    let frame = state.current_frame.read();

    if let Some(ref current_frame) = *frame {
        let alive_agents: Vec<_> = current_frame.agents.iter().filter(|a| a.alive).collect();

        if alive_agents.is_empty() {
            return Json(PopulationNeuralStats {
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                mean_docas: [0.0; 5],
                std_docas: [0.0; 5],
                dominant_nm: "none".to_string(),
                region_activations: vec![],
                tick: current_frame.tick,
            });
        }

        // Compute mean DOCAS
        let mut sum_docas = [0.0; 5];
        for agent in &alive_agents {
            for (i, item) in sum_docas.iter_mut().enumerate() {
                *item += agent.docas[i];
            }
        }
        let agent_count = alive_agents.len() as f32;
        let mean_docas: [f32; 5] = [
            sum_docas[0] / agent_count,
            sum_docas[1] / agent_count,
            sum_docas[2] / agent_count,
            sum_docas[3] / agent_count,
            sum_docas[4] / agent_count,
        ];

        // Compute std DOCAS
        let mut sum_sq_diff: [f32; 5] = [0.0; 5];
        for agent in &alive_agents {
            for i in 0..5 {
                let diff = agent.docas[i] - mean_docas[i];
                // TAG: surface=simulation owner=research-team rule=SIM-001
                sum_sq_diff[i] += diff * diff;
            }
        }
        let std_docas: [f32; 5] = [
            (sum_sq_diff[0] / agent_count).sqrt(),
            (sum_sq_diff[1] / agent_count).sqrt(),
            (sum_sq_diff[2] / agent_count).sqrt(),
            (sum_sq_diff[3] / agent_count).sqrt(),
            (sum_sq_diff[4] / agent_count).sqrt(),
        ];

        // Determine dominant neuromodulator
        let nm_names = [
            "dopamine",
            "oxytocin",
            "cortisol",
            "adrenaline",
            "serotonin",
        ];
        let dominant_idx = mean_docas
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map_or(0, |(i, _)| i);
        let dominant_nm = nm_names[dominant_idx].to_string();

        // Compute population-averaged region activations
        let dopamine = mean_docas[0];
        let oxytocin = mean_docas[1];
        let cortisol = mean_docas[2];
        let adrenaline = mean_docas[3];
        let serotonin = mean_docas[4];
        let region_activations = BRAIN_REGIONS
            .iter()
            .map(|(name, coords, nm, color)| {
                let mut activation = match *nm {
                    "dopamine" => (dopamine * 0.8 + serotonin * 0.2).clamp(0.0, 1.0),
                    "oxytocin" => (oxytocin * 0.9 + dopamine * 0.1).clamp(0.0, 1.0),
                    "cortisol" => (cortisol * 0.9 + adrenaline * 0.1).clamp(0.0, 1.0),
                    "adrenaline" => (adrenaline * 0.85 + cortisol * 0.15).clamp(0.0, 1.0),
                    "serotonin" => (serotonin * 0.9 + oxytocin * 0.1).clamp(0.0, 1.0),
                    "mixed" => match *name {
                        "prefrontal_cortex" => (dopamine + oxytocin + serotonin) / 3.0,
                        "anterior_cingulate" | "acc" => (cortisol + adrenaline) / 2.0,
                        "insula" => (oxytocin + cortisol) / 2.0,
                        _ => 0.5,
                    }
                    .clamp(0.0, 1.0),
                    _ => 0.5,
                };

                let live_cfg = crate::live::get_live_simulation_config();
                if live_cfg.baylor.mode == sim_core::baylor_gating::ConsciousnessMode::Unconscious
                    && cortisol < live_cfg.baylor.cortisol_crit
                    && (*name == "prefrontal_cortex"
                        || *name == "anterior_cingulate"
                        || *name == "acc")
                {
                    activation *= 0.08;
                }

                // TAG: surface=simulation owner=research-team rule=SIM-001
                BrainRegionActivation {
                    region: (*name).to_string(),
                    coordinates: *coords,
                    activation,
                    neuromodulator: (*nm).to_string(),
                    color: (*color).to_string(),
                }
            })
            .collect();

        return Json(PopulationNeuralStats {
            mean_docas,
            std_docas,
            dominant_nm,
            region_activations,
            tick: current_frame.tick,
        });
    }

    Json(PopulationNeuralStats {
        mean_docas: [0.0; 5],
        std_docas: [0.0; 5],
        dominant_nm: "none".to_string(),
        region_activations: vec![],
        tick: 0,
    })
}
