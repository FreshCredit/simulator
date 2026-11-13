---
title: "sim-runner"
date: "2026-03-05"
last_reviewed: "2026-03-05"
status: "active"
category: "simulation"
---
<!-- DOC-002: Stable -->
<!-- DOC-007: Active -->
<!-- SIM-001: Compliant -->
<!-- Last Updated: 2026-03-05 -->

# sim-runner

Batch execution harness for Agent Society Simulator.

## Purpose

This crate provides the execution layer for simulation experiments, including configuration parsing, batch orchestration, parameter sweeps, and Layer 2 statistics (aggregation, confidence intervals, effect sizes). It serves as the bridge between `sim-core` (deterministic kernel) and `sim-service` (API layer).

## SIM-001 Compliance

✅ **Only sim-* dependencies** - sim-core, sim-artifacts, sim-report  
✅ **No freshcredit-* dependencies** - Zero product crate dependencies  
✅ **Deterministic execution** - Reproducible batch runs  
✅ **No external I/O during runs** - Configurable output only  

## Research Context

Implements the experimental methodology for the 12-paper GTT research series:

- **Replication Management**: Multiple runs per condition for statistical power
- **Parameter Sweeps**: Factorial experiments across mechanism toggles
- **Statistical Analysis**: Built-in effect size calculation and hypothesis testing
- **Visualization**: Quick comparison plots for experimental results

## Architecture (§5.3.1)

| Layer | Component | Responsibility |
|-------|-----------|----------------|
| Layer 1 | `sim-core` | Per-tick metrics, agent behavior |
| **Layer 2** | **`sim-runner`** | **Aggregation, POM validation, CIs** |
| Layer 3 | `analysis/` | Post-hoc analysis (external) |

## Public API

```rust
use sim_runner::{BatchRunner, RunConfig, GridSweepRunner};

// Simple batch execution
let config = RunConfig::default();
let runner = BatchRunner::new(config);
let results = runner.run(5)?; // 5 replications

// Parameter sweep
let sweep_config = GridSweepConfig {
    parameters: vec!["n_agents".to_string(), "trust_decay".to_string()],
    ranges: vec![
        vec![100.0, 500.0, 1000.0],
        vec![0.01, 0.05, 0.1],
    ],
};
let sweep_runner = GridSweepRunner::new(sweep_config);
let sweep_results = sweep_runner.run_all()?;

// Quick GTT hypothesis test
let gtt_result = sim_runner::quick_gtt_test(20, 42)?;
println!("Hypothesis supported: {}", gtt_result.hypothesis_supported);
```

## Key Types

| Name | Module | Description |
|------|--------|-------------|
| `BatchRunner` | `batch` | Execute multiple replications |
| `BatchResult` | `batch` | Aggregated replication results |
| `RunConfig` | `config` | Simulation configuration |
| `ExperimentRunner` | `experiment` | Full experiment orchestration |
| `ExperimentResult` | `experiment` | Complete experiment output |
| `GttHypothesisRunner` | `hypothesis` | GTT hypothesis testing |
| `GttHypothesisResult` | `hypothesis` | Statistical test results |
| `GridSweepRunner` | `sweep` | Parameter grid execution |
| `GridSweepResult` | `sweep` | Sweep analysis results |
| `ComparisonRunner` | `comparison` | Quick experiment comparison |

## Batch Execution

```
┌─────────────────────────────────────────────────────────────┐
│                    Batch Execution                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Input: RunConfig { seed: 42, n_agents: 1000, ... }         │
│                          │                                  │
│                          ▼                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │           BatchRunner::run(n=5)                     │   │
│  └─────────────────────────────────────────────────────┘   │
│                          │                                  │
│         ┌────────────────┼────────────────┐                │
│         ▼                ▼                ▼                │
│  ┌──────────┐     ┌──────────┐     ┌──────────┐           │
│  │  Run 1   │     │  Run 2   │     │  Run N   │           │
│  │ seed=42  │     │ seed=43  │     │ seed=46  │           │
│  │ (sim-core)│    │ (sim-core)│    │ (sim-core)│          │
│  └────┬─────┘     └────┬─────┘     └────┬─────┘           │
│       └────────────────┼────────────────┘                 │
│                        ▼                                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │         Aggregate Statistics                        │   │
│  │  - Mean, Std, CI                                    │   │
│  │  - Effect sizes                                     │   │
│  │  - POM validation                                   │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Parameter Sweeps

| Sweep Type | Description | Use Case |
|------------|-------------|----------|
| `GridSweep` | Full factorial grid | Mechanism interaction testing |
| `RandomSweep` | Latin hypercube sampling | Exploration of parameter space |
| `AdaptiveSweep` | Bayesian optimization | Efficient frontier finding |

## Statistics (Layer 2)

| Statistic | Calculation | Output |
|-----------|-------------|--------|
| Mean | `Σx / n` | Central tendency |
| Std Dev | `√(Σ(x-μ)² / (n-1))` | Variability |
| 95% CI | `μ ± t(0.975, n-1) × SE` | Confidence bounds |
| Cohen's d | `(μ₁ - μ₂) / σ_pooled` | Effect size |
| t-statistic | `(μ₁ - μ₂) / SE_diff` | Hypothesis test |

## Determinism Guarantees

| Aspect | Guarantee |
|--------|-----------|
| Same config + seed | Identical results |
| Sequential seeds | Independent streams |
| Parallel execution | Thread-safe, reproducible |
| Checkpoint/resume | Deterministic continuation |

## Dependencies

### External (Allowed per SIM-001)
- `clap` - CLI argument parsing
- `rayon` - Parallel execution
- `plotters` - Visualization
- `rand` - Seeding (not in sim)
- `uuid` - Run identification

### Internal (SIM-001 Compliant)
- `sim-core` - Deterministic kernel
- `sim-artifacts` - Output schemas
- `sim-report` - Report generation

## CLI Usage

```bash
# Run batch of 10 replications
cargo run -p sim-runner -- --config experiment.toml --runs 10

# Run parameter sweep
cargo run -p sim-runner -- sweep --config sweep.toml

# Quick comparison
cargo run -p sim-runner -- compare --with-protocol --without-protocol
```

## Testing

```bash
# Run all tests
cargo test -p sim-runner

# Test batch execution
cargo test -p sim-runner batch

# Test sweeps
cargo test -p sim-runner sweep

# Test statistics
cargo test -p sim-runner stats
```

## Related Documentation

- [sim-core README](../sim-core/README.md)
- [sim-service README](../sim-service/README.md)
- [Simulation Architecture](../../../../docs/_analysis/architecture/CRATE_ARCHITECTURE_BLUEPRINT.md)
