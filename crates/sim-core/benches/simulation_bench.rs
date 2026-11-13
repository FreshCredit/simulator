#![allow(missing_docs)]
// Numeric cast precision is acceptable in these simulation benchmarks.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::float_cmp
)]

//! Performance Benchmarks (B-012)
//!
//! Criterion benchmarks for core simulation operations.
//! Tracks performance regressions and validates throughput requirements.

// TAG: surface=simulation owner=research-team rule=SIM-001
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rand::rngs::StdRng;
use rand::SeedableRng;
use sim_core::{
    agent::Agent,
    encounter::PERCEPTION_RADIUS,
    environment::SpatialHash,
    genome::Genome,
    merit::{MeritGovernanceType, MeritRegistry},
    simulation::{Simulation, SimulationConfig},
    types::Position,
    types::Sex,
};
use std::collections::HashMap;

/// Create benchmark simulation with n agents
fn create_sim_with_agents(n_agents: usize) -> Simulation {
    let config = SimulationConfig::default();
    let mut sim = Simulation::new(config);

    // Spawn agents at random positions
    for i in 0..n_agents {
        let x = (i % 64) as f32;
        let y = ((i / 64) % 64) as f32;
        sim.spawn_agent(Position::new_2d(x, y));
    }

    sim
}

/// Benchmark simulation initialization with agent spawning
fn bench_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("initialization");

    // TAG: surface=simulation owner=research-team rule=SIM-001
    for n_agents in &[100, 500, 1000] {
        group.throughput(Throughput::Elements(*n_agents as u64));
        group.bench_with_input(
            BenchmarkId::new("init_with_agents", n_agents),
            n_agents,
            |b, &n| {
                b.iter(|| {
                    let sim = create_sim_with_agents(n);
                    black_box(sim);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark single step execution
fn bench_step_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("step");

    for n_agents in &[100, 500, 1000] {
        let mut sim = create_sim_with_agents(*n_agents);

        group.throughput(Throughput::Elements(*n_agents as u64));
        group.bench_with_input(BenchmarkId::new("step", n_agents), n_agents, |b, _| {
            b.iter(|| {
                sim.step();
                black_box(&sim);
            });
        });
    }

    group.finish();
}

/// Benchmark full short simulation
fn bench_short_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("simulation");

    // TAG: surface=simulation owner=research-team rule=SIM-001
    for n_agents in &[100, 500] {
        group.throughput(Throughput::Elements((*n_agents * 100) as u64));
        group.bench_with_input(
            BenchmarkId::new("100_ticks", n_agents),
            n_agents,
            |b, &n| {
                b.iter(|| {
                    let mut sim = create_sim_with_agents(n);
                    sim.run(100);
                    black_box(sim.living_agent_count());
                });
            },
        );
    }

    group.finish();
}

/// Benchmark agent count queries
fn bench_agent_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("queries");

    let sim = create_sim_with_agents(1000);

    group.bench_function("living_agent_count", |b| {
        b.iter(|| {
            let count = sim.living_agent_count();
            black_box(count);
        });
    });

    group.bench_function("has_living_agents", |b| {
        b.iter(|| {
            let has = sim.has_living_agents();
            black_box(has);
        });
    });

    // TAG: surface=simulation owner=research-team rule=SIM-001
    group.finish();
}

/// Benchmark `SpatialHash` encounter detection
fn bench_spatial_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial_hash");

    for n_agents in &[100, 500, 1000, 5000] {
        // Create random agent positions
        let agents: Vec<(usize, Position)> = (0..*n_agents)
            .map(|i| {
                let x = (i % 64) as f32;
                let y = ((i / 64) % 64) as f32;
                (i, Position::new_2d(x, y))
            })
            .collect();

        group.throughput(Throughput::Elements(*n_agents as u64));
        group.bench_with_input(
            BenchmarkId::new("build_and_query", n_agents),
            n_agents,
            |b, _| {
                b.iter(|| {
                    let mut hash = SpatialHash::new(PERCEPTION_RADIUS);
                    hash.rebuild(&agents);
                    let pairs = hash.find_pairs_within_radius(PERCEPTION_RADIUS);
                    black_box(pairs);
                });
            },
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    group.finish();
}

/// Benchmark naive O(n²) encounter detection (for comparison)
fn bench_naive_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("naive_detection");

    for n_agents in &[100, 500, 1000] {
        let agents: Vec<(usize, Position)> = (0..*n_agents)
            .map(|i| {
                let x = (i % 64) as f32;
                let y = ((i / 64) % 64) as f32;
                (i, Position::new_2d(x, y))
            })
            .collect();

        group.throughput(Throughput::Elements(*n_agents as u64));
        group.bench_with_input(
            BenchmarkId::new("nested_loop", n_agents),
            n_agents,
            |b, _| {
                b.iter(|| {
                    let mut pairs = Vec::new();
                    for i in 0..agents.len() {
                        for j in (i + 1)..agents.len() {
                            let (idx1, pos1) = agents[i];
                            let (idx2, pos2) = agents[j];
                            if pos1.distance(&pos2) <= PERCEPTION_RADIUS {
                                pairs.push((idx1, idx2));
                            }
                        }
                    }
                    black_box(pairs);
                });
            },
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    group.finish();
}

/// Benchmark merit score computation
fn bench_merit_score_computation(c: &mut Criterion) {
    let mut group = c.benchmark_group("merit_score");

    let registry = MeritRegistry::new_default(0);
    let genome = Genome::random(&mut StdRng::seed_from_u64(42));

    for n_agents in &[100, 500, 1000] {
        let agents: Vec<Agent> = (0..*n_agents)
            .map(|i| {
                let mut agent = Agent::new(
                    i as u64,
                    Position::new_2d((i % 64) as f32, ((i / 64) % 64) as f32),
                    genome.clone(),
                    0,
                    if i % 2 == 0 { Sex::Male } else { Sex::Female },
                );
                agent.reputation_score = (i as f32 / *n_agents as f32).clamp(0.0, 1.0);
                agent.positive_interactions = (i * 10) as u32;
                agent.negative_interactions = ((n_agents - i) * 5) as u32;
                agent
            })
            .collect();

        group.throughput(Throughput::Elements(*n_agents as u64));
        group.bench_with_input(
            BenchmarkId::new("compute_all", n_agents),
            n_agents,
            |b, _| {
                b.iter(|| {
                    for agent in &agents {
                        let score = registry.compute_merit_score(agent, true);
                        black_box(score);
                    }
                });
            },
        );
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    group.finish();
}

/// Benchmark weighted voting
fn bench_weighted_voting(c: &mut Criterion) {
    let mut group = c.benchmark_group("merit_voting");

    let registry = MeritRegistry::new_default(0);
    let genome = Genome::random(&mut StdRng::seed_from_u64(42));

    for n_agents in &[100, 500, 1000, 5000] {
        let mut agents = HashMap::new();
        let votes: Vec<(u64, bool)> = (0..*n_agents)
            .map(|i| {
                let mut agent = Agent::new(
                    i as u64,
                    Position::new_2d((i % 64) as f32, ((i / 64) % 64) as f32),
                    genome.clone(),
                    0,
                    if i % 2 == 0 { Sex::Male } else { Sex::Female },
                );
                agent.reputation_score = (i as f32 / *n_agents as f32).clamp(0.0, 1.0);
                agents.insert(i as u64, agent);
                (i as u64, i % 2 == 0) // Half vote yes, half vote no
            })
            .collect();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        group.throughput(Throughput::Elements(*n_agents as u64));
        group.bench_with_input(
            BenchmarkId::new("compute_weighted_vote", n_agents),
            n_agents,
            |b, _| {
                b.iter(|| {
                    let result = registry.compute_weighted_vote(&votes, &agents, true);
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark top agent selection
fn bench_top_agents_selection(c: &mut Criterion) {
    let mut group = c.benchmark_group("merit_top_agents");

    let registry = MeritRegistry::new_default(0);
    let genome = Genome::random(&mut StdRng::seed_from_u64(42));

    for n_agents in &[100, 500, 1000] {
        let agents: Vec<Agent> = (0..*n_agents)
            .map(|i| {
                let mut agent = Agent::new(
                    i as u64,
                    Position::new_2d((i % 64) as f32, ((i / 64) % 64) as f32),
                    genome.clone(),
                    0,
                    if i % 2 == 0 { Sex::Male } else { Sex::Female },
                );
                agent.reputation_score = (i as f32 / *n_agents as f32).clamp(0.0, 1.0);
                agent
            })
            .collect();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        group.throughput(Throughput::Elements(*n_agents as u64));
        group.bench_with_input(
            BenchmarkId::new("select_top_10", n_agents),
            n_agents,
            |b, _| {
                b.iter(|| {
                    let top = registry.get_top_agents(&agents, true, 10);
                    black_box(top);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark weight evolution
fn bench_weight_evolution(c: &mut Criterion) {
    let mut group = c.benchmark_group("merit_evolution");

    for n_attrs in &[4, 8, 16] {
        let mut registry = MeritRegistry::new(MeritGovernanceType::DynamicVote);

        // Add attributes
        for i in 0..*n_attrs {
            registry
                .add_attribute(
                    &format!("attr_{i}"),
                    1.0 / *n_attrs as f32,
                    0,
                    "Test attribute",
                )
                .unwrap();
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let accuracy_data: Vec<(u32, f32)> = (0..*n_attrs)
            .map(|i| (i, (i as f32 / *n_attrs as f32).mul_add(0.5, 0.5)))
            .collect();

        group.bench_with_input(
            BenchmarkId::new("evolve_weights", n_attrs),
            n_attrs,
            |b, _| {
                b.iter(|| {
                    let mut reg = registry.clone();
                    reg.evolve_weights(&accuracy_data, 100).unwrap();
                    black_box(reg);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_initialization,
    bench_step_execution,
    bench_short_simulation,
    bench_agent_queries,
    bench_spatial_hash,
    bench_naive_detection,
    bench_merit_score_computation,
    bench_weighted_voting,
    bench_top_agents_selection,
    bench_weight_evolution,
);
criterion_main!(benches);
