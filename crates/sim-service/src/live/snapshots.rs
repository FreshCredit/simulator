//! Live simulation snapshot and frame construction

use super::*;
use sim_core::Simulation;

pub mod grid_snapshot;
pub mod v4_snapshot;
pub use grid_snapshot::*;
pub use v4_snapshot::*;

/// Create condensed AI frame from full simulation frame
/// Returns whether agent is alive
#[inline]
#[allow(dead_code)]
fn is_agent_alive(a: &sim_core::agent::Agent) -> bool {
    a.alive
}

/// Compute mean agent energy
#[inline]
#[allow(dead_code)]
fn compute_mean_energy(total_energy: f32, count: usize) -> f32 {
    if count > 0 {
        total_energy / count as f32
    } else {
        0.0
    }
}

/// Compute mean agent DOCAS vector
#[inline]
#[allow(dead_code)]
fn compute_mean_docas(sums: [f32; 5], count: usize) -> [f32; 5] {
    if count > 0 {
        [
            sums[0] / count as f32,
            sums[1] / count as f32,
            sums[2] / count as f32,
            sums[3] / count as f32,
            sums[4] / count as f32,
        ]
    } else {
        [0.0; 5]
    }
}

/// Extract total payoff from game event
#[inline]
#[allow(dead_code)]
fn extract_game_event_payoff(a: f32, b: f32) -> f32 {
    a + b
}

/// Discretize cognitive bandwidth into 10 deciles
#[inline]
#[allow(dead_code)]
fn decile_from_bandwidth(bw: f32) -> usize {
    ((bw.clamp(0.0, 1.0) * 10.0).floor() as usize + 1).clamp(1, 10)
}

/// Normalize DOCAS simplex projection over active population
#[inline]
#[allow(dead_code)]
fn normalize_simplex_projection(sums: [f32; 5], count: f32) -> [f32; 5] {
    [
        sums[0] / count,
        sums[1] / count,
        sums[2] / count,
        sums[3] / count,
        sums[4] / count,
    ]
}

/// Resolve weather cognitive bandwidth modifier
#[inline]
#[allow(dead_code)]
fn resolve_weather_modifier(sim: &Simulation) -> f32 {
    if sim.config.environment.weather_enabled {
        sim.environment
            .weather
            .cognitive_bandwidth_weather_modifier()
    } else {
        1.0
    }
}

/// Returns whether agent has verified ID
#[inline]
#[allow(dead_code)]
fn is_agent_verified(agent: &sim_core::agent::Agent) -> bool {
    agent.has_verified_id
}

/// Compute average group edge weight
#[inline]
#[allow(dead_code)]
fn compute_group_weight(a: f32, b: f32) -> f32 {
    ((a + b) / 2.0).clamp(0.0, 1.0)
}

/// Compute average lineage edge weight
#[inline]
#[allow(dead_code)]
fn compute_lineage_weight(a: f32, b: f32) -> f32 {
    ((a + b) / 2.0).clamp(0.0, 1.0)
}

/// Creates a condensed AI simulation frame from a full simulation frame
pub fn create_ai_frame(full_frame: &SimulationFrame) -> AiSimulationFrame {
    let alive_count = full_frame.stats.alive_agents;

    // Calculate averages from agent data
    let (total_energy, docas_sums) = if alive_count > 0 {
        full_frame.agents.iter().filter(|a| a.alive).fold(
            (0.0f32, [0.0f32; 5]),
            |(energy, mut docas), agent| {
                let new_energy = energy + agent.energy;
                for (i, docas_val) in docas.iter_mut().enumerate().take(5) {
                    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                    *docas_val += agent.docas[i];
                }
                (new_energy, docas)
            },
        )
    } else {
        (0.0, [0.0; 5])
    };

    let avg_energy = if alive_count > 0 {
        total_energy / alive_count as f32
    } else {
        0.0
    };
    let avg_docas: [f32; 5] = if alive_count > 0 {
        [
            docas_sums[0] / alive_count as f32,
            docas_sums[1] / alive_count as f32,
            docas_sums[2] / alive_count as f32,
            docas_sums[3] / alive_count as f32,
            docas_sums[4] / alive_count as f32,
        ]
    } else {
        [0.0; 5]
    };

    // Extract notable events from game and encounter events
    let mut notable_events = Vec::new();

    // Add game events as notable events
    for event in &full_frame.game_events {
        notable_events.push(NotableEvent {
            tick: event.tick,
            event_type: format!("game_{}", event.game_type.to_lowercase()),
            description: format!(
                "Game between agents {} and {}: {} (payoffs: {:.1}, {:.1})",
                event.agent_a, event.agent_b, event.outcome, event.payoff_a, event.payoff_b
            ),
            agent_ids: vec![event.agent_a, event.agent_b],
            value: Some(event.payoff_a + event.payoff_b),
        });
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Add encounter events
    for event in &full_frame.encounter_events {
        notable_events.push(NotableEvent {
            tick: event.tick,
            event_type: format!("encounter_{}", event.encounter_type.to_lowercase()),
            description: format!(
                "Encounter between agents {} and {}: {}",
                event.agent_a, event.agent_b, event.encounter_type
            ),
            agent_ids: vec![event.agent_a, event.agent_b],
            value: None,
        });
    }

    AiSimulationFrame {
        tick: full_frame.tick,
        alive_count,
        avg_energy,
        avg_docas,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        notable_events,
        group_count: full_frame.stats.groups,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}

fn compute_population_kappas(sim: &Simulation, weather_mod: f32) -> Vec<f32> {
    let total_agents = sim.agents.len();
    sim.agents
        .iter()
        .filter(|a| a.alive)
        .map(|a| {
            let decile =
                ((a.cognitive_bandwidth.clamp(0.0, 1.0) * 10.0).floor() as usize + 1).clamp(1, 10);
            let is_perturbed =
                sim.config
                    .optogenetics
                    .is_agent_perturbed(a.id, decile, total_agents);
            let opto_mod = if is_perturbed {
                sim.config
                    .optogenetics
                    .modulate_cognitive_bandwidth(1.0, true)
            } else {
                1.0
            };
            a.effective_bandwidth(weather_mod, opto_mod)
        })
        .collect()
}

fn compute_docas_simplex(living_agents: &[&sim_core::agent::Agent]) -> [f32; 5] {
    let living_count = living_agents.len().max(1) as f32;
    let mut sum_docas = [0.0f32; 5];
    for a in living_agents {
        for (i, val) in a.docas.levels.iter().enumerate().take(5) {
            sum_docas[i] += val;
        }
    }
    [
        sum_docas[0] / living_count,
        sum_docas[1] / living_count,
        sum_docas[2] / living_count,
        sum_docas[3] / living_count,
        sum_docas[4] / living_count,
    ]
}

fn compute_kuramoto_coherence(sim: &Simulation, living_agents: &[&sim_core::agent::Agent]) -> f32 {
    let living_count = living_agents.len().max(1) as f32;
    let total_agents = sim.agents.len();
    let mut sum_cos = 0.0f32;
    let mut sum_sin = 0.0f32;

    for a in living_agents {
        let decile =
            ((a.cognitive_bandwidth.clamp(0.0, 1.0) * 10.0).floor() as usize + 1).clamp(1, 10);
        let is_perturbed = sim
            .config
            .optogenetics
            .is_agent_perturbed(a.id, decile, total_agents);
        let tau_centered = (a.docas.compute_tau() - 0.5) * 2.0;
        let phase = 2.0 * tau_centered.atan();
        let coupling_weight = if (sim.config.optogenetics.target_channels[sim_core::types::O]
            || sim.config.optogenetics.target_channels[sim_core::types::S])
            && sim.config.optogenetics.wavelength
                == sim_core::optogenetics::OpsinWavelength::ArchTYellow566nm
        {
            sim.config
                .optogenetics
                .modulate_kuramoto_coupling(1.0, is_perturbed)
        } else {
            1.0
        };
        sum_cos += coupling_weight * phase.cos();
        sum_sin += coupling_weight * phase.sin();
    }

    ((sum_cos * sum_cos + sum_sin * sum_sin).sqrt() / living_count).clamp(0.0, 1.0)
}

/// Start the background simulation loop
pub fn create_frame(sim: &Simulation) -> SimulationFrame {
    let snap = build_agent_snapshots(sim);
    let trust_edges = build_trust_edges(&snap.group_members, &snap.agent_tau, sim);

    let weather_mod = if sim.config.environment.weather_enabled {
        sim.environment
            .weather
            .cognitive_bandwidth_weather_modifier()
    } else {
        1.0
    };
    let kappas = compute_population_kappas(sim, weather_mod);
    let decile_grad = sim_core::visual_cloaking::DecileVulnerabilityGradient::evaluate_population(
        &sim.config.visual_cloaking,
        &kappas,
        sim.config.weather_shield_enabled,
    );

    let living_agents: Vec<&sim_core::agent::Agent> =
        sim.agents.iter().filter(|a| a.alive).collect();
    let simplex = compute_docas_simplex(&living_agents);
    let kuramoto_r = compute_kuramoto_coherence(sim, &living_agents);
    let v4 = build_v4_telemetry(sim, &decile_grad, simplex, kuramoto_r);

    SimulationFrame {
        tick: sim.tick,
        agents: snap.agents,
        stats: FrameStats {
            total_agents: sim.agents.len(),
            alive_agents: sim.living_agent_count(),
            verified_agents: snap.verified_count,
            groups: snap.groups.len(),
        },
        trust_edges,
        resources: build_resources(sim),
        weather: build_weather_grid(sim),
        terrain: build_terrain_grid(sim),
        game_events: sim
            .game_events_this_tick
            .iter()
            .map(|e| GameEvent {
                tick: e.tick,
                agent_a: e.agent_a,
                agent_b: e.agent_b,
                game_type: e.game_type.clone(),
                // TAG: surface=simulation owner=research-team rule=SIM-001
                outcome: e.outcome.clone(),
                payoff_a: e.payoff_a,
                payoff_b: e.payoff_b,
            })
            .collect(),
        encounter_events: sim
            .encounter_events_this_tick
            .iter()
            .map(|e| EncounterEvent {
                tick: e.tick,
                agent_a: e.agent_a,
                agent_b: e.agent_b,
                encounter_type: e.encounter_type.clone(),
            })
            .collect(),
        status: Some(SimulationStatus::Running),
        max_ticks: Some(sim.config.max_ticks),
        grid_dimensions: Some((sim.environment.width as u32, sim.environment.height as u32)),
        v4_telemetry: Some(v4),
    }
}

struct AgentSnapshotData {
    agents: Vec<AgentSnapshot>,
    verified_count: usize,
    groups: std::collections::HashSet<u64>,
    group_members: std::collections::HashMap<u64, Vec<(u64, f32)>>,
    agent_tau: std::collections::HashMap<u64, f32>,
}

fn build_agent_snapshots(sim: &Simulation) -> AgentSnapshotData {
    let mut agents = Vec::with_capacity(sim.agents.len());
    let mut verified_count = 0;
    let mut groups = std::collections::HashSet::new();
    let mut gen_min = u32::MAX;
    let mut gen_max = 0u32;
    let mut gen_sum = 0u64;
    let mut group_members: std::collections::HashMap<u64, Vec<(u64, f32)>> =
        std::collections::HashMap::new();
    let mut agent_tau: std::collections::HashMap<u64, f32> = std::collections::HashMap::new();

    for agent in &sim.agents {
        if !agent.alive {
            continue;
        }
        let tau = agent.compute_trust();
        agent_tau.insert(agent.id, tau);
        if let Some(gid) = agent.group_id {
            groups.insert(gid);
            group_members.entry(gid).or_default().push((agent.id, tau));
        }
        if agent.has_verified_id {
            verified_count += 1;
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let (generation, parent1_id, parent2_id) =
            sim.lineage.get_node(agent.id).map_or((0, None, None), |n| {
                (n.generation, n.parent1_id, n.parent2_id)
            });
        let (children_count, grandchildren_count) =
            sim.lineage.get_children_and_grandchildren_counts(agent.id);
        gen_min = gen_min.min(generation);
        gen_max = gen_max.max(generation);
        gen_sum += generation as u64;
        agents.push(build_agent_snapshot(
            agent,
            tau,
            generation,
            parent1_id,
            parent2_id,
            children_count,
            grandchildren_count,
            &sim.config.toggles.dials,
        ));
    }

    let alive_count = sim.living_agent_count();
    if alive_count > 0 && (sim.tick % 100 == 0 || gen_min > 10) {
        info!(
            "[SimLoop] [GenerationStats] tick={} alive={} gen_range=[{}..{}] gen_avg={:.1}",
            sim.tick,
            alive_count,
            gen_min,
            gen_max,
            // TAG: surface=simulation owner=platform-team rule=GENERAL-001
            gen_sum as f32 / alive_count as f32
        );
    }

    AgentSnapshotData {
        agents,
        verified_count,
        groups,
        group_members,
        agent_tau,
    }
}

fn build_agent_snapshot(
    agent: &sim_core::Agent,
    tau: f32,
    generation: u32,
    parent1_id: Option<u64>,
    parent2_id: Option<u64>,
    children_count: u32,
    grandchildren_count: u32,
    dials: &sim_core::toggles::DialConfig,
) -> AgentSnapshot {
    AgentSnapshot {
        agent_id: agent.id,
        x: agent.position.x,
        y: agent.position.y,
        z: agent.position.z,
        energy: agent.energy,
        docas: agent.docas.levels,
        genome_gains: agent.genome.docas_beta_with_dials(dials),
        alive: agent.alive,
        has_verified_id: agent.has_verified_id,
        has_identity: agent.has_verified_id,
        group_id: agent.group_id,
        age: agent.age,
        sex: agent.sex,
        tau,
        reputation: agent.interaction_reputation(),
        generation,
        parent1_id,
        parent2_id,
        children_count,
        grandchildren_count,
        last_dx: agent.last_dx,
        last_dy: agent.last_dy,
        lexicon: build_lexicon_summary(agent),
        biomarkers: Some(AgentBiomarkersSnapshot {
            hrv_rmssd_ms: agent.biomarkers.hrv_rmssd_ms,
            hrv_sdnn_ms: agent.biomarkers.hrv_sdnn_ms,
            resting_hr_bpm: agent.biomarkers.resting_hr_bpm,
            respiratory_rate_brpm: agent.biomarkers.respiratory_rate_brpm,
            sleep_efficiency_pct: agent.biomarkers.sleep_efficiency_pct,
            sleep_deep_pct: agent.biomarkers.sleep_deep_pct,
            sleep_rem_pct: agent.biomarkers.sleep_rem_pct,
            circadian_phase_rad: agent.biomarkers.circadian_phase_rad,
            eeg_frontal_theta_uv2: agent.biomarkers.eeg_frontal_theta_uv2,
            eeg_alpha_asymmetry: agent.biomarkers.eeg_alpha_asymmetry,
            eeg_beta_arousal_ratio: agent.biomarkers.eeg_beta_arousal_ratio,
            eeg_rewp_erp_uv: agent.biomarkers.eeg_rewp_erp_uv,
            glucose_mg_dl: agent.biomarkers.glucose_mg_dl,
            glucose_tir_pct: agent.biomarkers.glucose_tir_pct,
            glucose_cv_pct: agent.biomarkers.glucose_cv_pct,
            sleeping_wrist_temp_c: agent.biomarkers.sleeping_wrist_temp_c,
            fnirs_prefrontal_hbo2_umol: agent.biomarkers.fnirs_prefrontal_hbo2_umol,
            fnirs_prefrontal_hbr_umol: agent.biomarkers.fnirs_prefrontal_hbr_umol,
        }),
        financial: Some(PlaidFinancialSnapshot {
            market_trust_score: agent.economics.market_trust_score,
            cashflow_buffer_days: agent.economics.cashflow_buffer_days,
            spend_volatility: agent.economics.spend_volatility,
            nsf_overdraft_count: agent.economics.nsf_overdraft_count,
            on_time_payment_rate: agent.economics.on_time_payment_rate,
            credit_access: agent.economics.credit_access,
            debt_burden: agent.economics.debt_burden,
            wealth: agent.economics.wealth,
            bond_density: agent.economics.bond_density,
            concordance_mantel_r: agent.economics.concordance_mantel_r,
        }),
    }
}

/// Build trust network edges for visualization (P1-4)
/// Creates edges for: group membership, parent-child lineage
fn build_group_edges(
    group_members: &std::collections::HashMap<u64, Vec<(u64, f32)>>,
    agent_tau: &std::collections::HashMap<u64, f32>,
    edges: &mut Vec<TrustEdge>,
) {
    const MAX_GROUP_EDGES: usize = 500;

    for members in group_members.values() {
        if members.len() < 2 {
            continue;
        }

        let member_count = members.len();
        if member_count <= 6 {
            build_small_group_edges(members, edges, MAX_GROUP_EDGES);
        } else {
            build_large_group_edges(members, agent_tau, edges, MAX_GROUP_EDGES);
        }

        if edges.len() >= MAX_GROUP_EDGES {
            break;
        }
    }
}

fn build_small_group_edges(members: &[(u64, f32)], edges: &mut Vec<TrustEdge>, max_edges: usize) {
    let count = members.len();
    for i in 0..count {
        for j in (i + 1)..count {
            let (id_a, tau_a) = members[i];
            let (id_b, tau_b) = members[j];
            let weight = (tau_a + tau_b) / 2.0;

            edges.push(TrustEdge {
                from_id: id_a,
                to_id: id_b,
                weight: weight.clamp(0.0, 1.0),
                edge_type: EdgeType::GroupMember,
            });

            if edges.len() >= max_edges {
                return;
            }
        }
    }
}

fn build_large_group_edges(
    members: &[(u64, f32)],
    agent_tau: &std::collections::HashMap<u64, f32>,
    edges: &mut Vec<TrustEdge>,
    max_edges: usize,
) {
    let (leader_id, _) = members
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .copied()
        .unwrap_or((0, 0.0));

    for (member_id, member_tau) in members {
        if *member_id != leader_id {
            let leader_tau = agent_tau.get(&leader_id).copied().unwrap_or(0.5);
            let weight = (*member_tau + leader_tau) / 2.0;

            edges.push(TrustEdge {
                from_id: leader_id,
                to_id: *member_id,
                weight: weight.clamp(0.0, 1.0),
                edge_type: EdgeType::GroupMember,
            });

            if edges.len() >= max_edges {
                return;
            }
        }
    }
}

fn build_lineage_edges(
    sim: &Simulation,
    agent_tau: &std::collections::HashMap<u64, f32>,
    edges: &mut Vec<TrustEdge>,
) {
    const MAX_LINEAGE_EDGES: usize = 200;
    let mut lineage_edge_count = 0;

    for agent in &sim.agents {
        if !agent.alive {
            continue;
        }

        let Some(node) = sim.lineage.get_node(agent.id) else {
            continue;
        };

        if add_parent_edge(node.parent1_id, agent.id, agent_tau, edges) {
            lineage_edge_count += 1;
            if lineage_edge_count >= MAX_LINEAGE_EDGES {
                break;
            }
        }
        if add_parent_edge(node.parent2_id, agent.id, agent_tau, edges) {
            lineage_edge_count += 1;
            if lineage_edge_count >= MAX_LINEAGE_EDGES {
                break;
            }
        }
    }
}

fn add_parent_edge(
    parent_id: Option<u64>,
    child_id: u64,
    agent_tau: &std::collections::HashMap<u64, f32>,
    edges: &mut Vec<TrustEdge>,
) -> bool {
    let Some(pid) = parent_id else {
        return false;
    };
    if !agent_tau.contains_key(&pid) {
        return false;
    }
    let parent_tau = agent_tau.get(&pid).copied().unwrap_or(0.5);
    let child_tau = agent_tau.get(&child_id).copied().unwrap_or(0.5);

    edges.push(TrustEdge {
        from_id: pid,
        to_id: child_id,
        weight: ((parent_tau + child_tau) / 2.0).clamp(0.0, 1.0),
        edge_type: EdgeType::Lineage,
    });
    true
}

fn build_trust_edges(
    group_members: &std::collections::HashMap<u64, Vec<(u64, f32)>>,
    agent_tau: &std::collections::HashMap<u64, f32>,
    sim: &Simulation,
) -> Vec<TrustEdge> {
    let mut edges = Vec::new();
    build_group_edges(group_members, agent_tau, &mut edges);
    build_lineage_edges(sim, agent_tau, &mut edges);
    edges
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decile_binning_partitions_uniform_intervals() {
        let compute_decile =
            |bw: f32| -> usize { ((bw.clamp(0.0, 1.0) * 10.0).floor() as usize + 1).clamp(1, 10) };

        // Midpoints of 10 deciles
        assert_eq!(compute_decile(0.05), 1);
        assert_eq!(compute_decile(0.15), 2);
        assert_eq!(compute_decile(0.25), 3);
        assert_eq!(compute_decile(0.35), 4);
        assert_eq!(compute_decile(0.45), 5);
        assert_eq!(compute_decile(0.55), 6);
        assert_eq!(compute_decile(0.65), 7);
        assert_eq!(compute_decile(0.75), 8);
        assert_eq!(compute_decile(0.85), 9);
        assert_eq!(compute_decile(0.95), 10);

        // Boundary extremes
        assert_eq!(compute_decile(0.0), 1);
        assert_eq!(compute_decile(1.0), 10);
        assert_eq!(compute_decile(-0.5), 1);
        assert_eq!(compute_decile(1.5), 10);
    }

    #[test]
    fn test_create_ai_frame_empty_and_populated() {
        let empty_frame = SimulationFrame {
            tick: 1,
            agents: vec![],
            stats: FrameStats {
                total_agents: 0,
                alive_agents: 0,
                verified_agents: 0,
                groups: 0,
            },
            trust_edges: vec![],
            resources: None,
            weather: None,
            terrain: None,
            game_events: vec![],
            encounter_events: vec![],
            status: Some(SimulationStatus::Running),
            max_ticks: Some(100),
            grid_dimensions: Some((50, 50)),
            v4_telemetry: None,
        };

        let ai_frame = create_ai_frame(&empty_frame);
        assert_eq!(ai_frame.tick, 1);
        assert_eq!(ai_frame.alive_count, 0);
        assert_eq!(ai_frame.avg_energy, 0.0);
        assert_eq!(ai_frame.avg_docas, [0.0; 5]);
        assert!(ai_frame.notable_events.is_empty());
    }

    #[test]
    fn test_create_frame_from_simulation_instance() {
        let config = sim_core::simulation::SimulationConfig::default();
        let sim = sim_core::Simulation::new(config);
        let frame = create_frame(&sim);

        assert_eq!(frame.tick, 0);
        assert_eq!(frame.stats.total_agents, sim.agents.len());
        assert_eq!(frame.stats.alive_agents, sim.living_agent_count());
        if let Some(v4) = frame.v4_telemetry {
            assert!(v4.kuramoto_coherence_r >= 0.0 && v4.kuramoto_coherence_r <= 1.0);
            assert_eq!(v4.decile_uptake_rates.len(), 10);
        }
    }

    #[test]
    fn test_build_group_edges_caps() {
        let mut members = Vec::new();
        for i in 0..10 {
            members.push((i as u64, 0.5f32));
        }

        let mut edges = Vec::new();
        build_small_group_edges(&members[0..4], &mut edges, 50);
        assert_eq!(edges.len(), 6); // 4 choose 2 = 6

        let mut agent_tau = std::collections::HashMap::new();
        for i in 0..10 {
            agent_tau.insert(i as u64, 0.6f32);
        }
        let mut large_edges = Vec::new();
        build_large_group_edges(&members, &agent_tau, &mut large_edges, 50);
        assert_eq!(large_edges.len(), 9); // leader connects to 9 other members
    }
}
