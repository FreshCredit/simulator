---
title: "sim-core: Trust Emergence Simulation Engine"
date: "2026-02-16"
last_reviewed: "2026-03-05"
status: "active"
category: "sim core"
---
# sim-core: Trust Emergence Simulation Engine

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Docs](https://img.shields.io/badge/docs-WIP-blue.svg)](./docs)

A deterministic, research-grade agent-based model (ABM) for studying trust emergence across multiple developmental stages and domains.

## Overview

This simulation system implements a unified theoretical framework spanning 12 interconnected research papers on trust, from neurobiology to economics. The simulation is designed for:

- **Scientific reproducibility** - Deterministic execution with seeded RNG
- **Cross-domain validation** - Biological, financial, linguistic, and cultural trust mechanisms
- **Publication-ready outputs** - Statistical analysis, figures, and data export

## Quick Start

```bash
# Build the simulation engine
cargo build --release -p sim-core

# Run tests
cargo test -p sim-core

# Run a single experiment
cargo run --example lambda_universality
```

## Research Background

This simulation system supports research into the **Generalized Trust Theory (GTT)**, which posits that trust emerges from a common computational mechanism across diverse domains:

- **Neurobiology** - PDE4B-mediated plasticity and DOCAS state vectors
- **Emotion** - Trust-weighted affective modeling
- **Linguistics** - Semantic coherence and network centrality
- **Philosophy** - Historical oscillations in trust frameworks
- **Culture** - Bond density and cultural quadrant dynamics
- **Economics** - Trust-based GDP prediction and default modeling
- **Game Theory** - Trust-weighted expected utility
- **Meaning** - Semantic percolation and civilizational collapse
- **Music** - Aesthetic indicators of trust coherence

## Architecture

### Core Components

| Module | Purpose | Paper |
|--------|---------|-------|
| `unified_agent.rs` | Single agent type for all 12 papers | Foundation |
| `experiments/` | P0 gap validation experiments | All |
| `docas.rs` | Neuromodulator-inspired state | DOCAS |
| `genome.rs` | Heritable traits with diploid encoding | PDE4B |
| `culture.rs` | Cultural quadrant dynamics | Culture |
| `economics.rs` | Trust-based economic modeling | Economics |

### Key Design Principles

1. **Trust is NEVER hardcoded** - It emerges from agent behavior
2. **Deterministic execution** - Same seed produces same results
3. **Toggleable mechanisms** - 9 mechanisms for factorial experiments
4. **Zero product dependencies** - Pure research code

## Experiments

The simulation includes implementations for validating critical (P0) research claims:

| Experiment | Gap ID | Description |
|------------|--------|-------------|
| Lambda Universality | GTT-P0-1 | Validate λ ≈ 0.61 across domains |
| Twin Heritability | DOCAS-P0-2 | Validate h² ≈ 0.50 via MZ/DZ twins |
| Criticality Sigma | DOCAS-P0-3 | Demonstrate σ ∈ [0.9, 1.1] |
| ... | ... | 23 additional experiments |

## Project Structure

```
crates/sim-core/
├── src/
│   ├── lib.rs                 # Main library
│   ├── unified_agent.rs       # Core agent data structure
│   ├── experiments/           # P0 gap validation experiments
│   │   ├── mod.rs
│   │   ├── lambda_universality.rs
│   │   ├── twin_heritability.rs
│   │   └── criticality_sigma.rs
│   ├── agent.rs               # Agent behavior
│   ├── docas.rs               # DOCAS state vectors
│   ├── genome.rs              # Genetic architecture
│   ├── culture.rs             # Cultural dynamics
│   ├── economics.rs           # Economic modeling
│   └── ...                    # Additional modules
├── tests/                     # Integration tests
├── examples/                  # Example simulations
└── README.md                  # This file
```

## Usage

### Basic Simulation

```rust
use sim_core::{Simulation, SimulationConfig};

let config = SimulationConfig {
    n_agents: 1000,
    n_timesteps: 365,
    seed: 42,
};

let mut sim = Simulation::new(config);
sim.run();

// Access results
let agents = sim.agents();
```

### Running Experiments

```rust
use sim_core::experiments::{
    LambdaUniversalityExperiment,
    Experiment,
    ExperimentConfig
};

let exp = LambdaUniversalityExperiment::new();
let config = ExperimentConfig::default_with_type(
    ExperimentType::LambdaUniversality
);

let result = exp.run(&config)?;
assert!(result.success);  // Validates success criteria
```

## Determinism

The simulation is fully deterministic given a seed:

```rust
// Run 1
let sim1 = Simulation::new(SimulationConfig { seed: 42, .. });
sim1.run_to(1000);

// Run 2 (same seed)
let sim2 = Simulation::new(SimulationConfig { seed: 42, .. });
sim2.run_to(1000);

// Results are identical
assert_eq!(sim1.hash(), sim2.hash());
```

## Testing

```bash
# Run all tests
cargo test -p sim-core

# Run specific experiment tests
cargo test -p sim-core lambda

# Run with output
cargo test -p sim-core -- --nocapture
```

## OSF Pre-registration

This research has been pre-registered on the Open Science Framework:

🔗 **OSF Project:** [To be added upon pre-registration]

Pre-registration includes:
- Analysis plan for λ universality validation
- Hypothesis: λ ≈ 0.61 across biological, financial, and linguistic domains
- Statistical methods specified
- Expected sample sizes
- Primary outcome measures

## Citation

If you use this software in your research, please cite:

```bibtex
@software{freshcredit_sim_core_2026,
  title = {sim-core: Trust Emergence Simulation Engine},
  author = {FreshCredit Research Team},
  year = {2026},
  url = {https://github.com/freshcredit/sim-core}
}
```

Full citations for the 12-paper research series will be added upon publication.

## Documentation

- [Architecture Overview](../../docs/application/architecture/)
- [P0 Gap Implementation Plan](../../docs/simulation/audit/P0_IMPLEMENTATION_PLAN.md)
- [Experiment Specifications](../../docs/simulation/audit/simulation_verification.md)

## Contributing

This is a research codebase. For issues or questions about the simulation:

1. Check existing [documentation](../../docs/)
2. Review [P0 gap tracking](../../docs/simulation/audit/P0_GAP_TRACKING.md)
3. Open an issue with the "research" label

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

Additional terms apply for research components - see LICENSE for details.

## Acknowledgments

This research is part of the FreshCredit Research Program investigating trust as a fundamental computational mechanism across biological, social, and economic systems.

## Status

- **Simulation Core**: ✅ Implemented
- **UnifiedAgent**: ✅ Implemented
- **Experiments**: 🟡 3/26 templates complete
- **OSF Pre-registration**: ⬜ Pending
- **Publication**: ⬜ In preparation

---

**Note**: This is research software in active development. APIs may change as the 12-paper series progresses toward publication.
