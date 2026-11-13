---
title: "Inbreeding Depression Module"
date: "2026-02-16"
last_reviewed: "2026-03-05"
status: "stable"
category: "genetics"
---
<!-- DOC-002: Stable -->
<!-- DOC-007: Active -->
<!-- Last Updated: 2026-02-16 -->

# Inbreeding Depression Module

Implementation of inbreeding depression for the Agent Society Simulator based on pedigree-based inbreeding coefficient (F) and deleterious recessive allele exposure.

## Purpose

This module implements inbreeding depression mechanics for the Agent Society Simulator. Inbreeding depression occurs when related individuals mate, increasing homozygosity and exposing deleterious recessive alleles that are normally masked in heterozygotes.

## Directory Structure

```
crates/sim-core/src/genetics/
├── README.md           # This file
├── mod.rs              # Module exports
└── inbreeding.rs       # Inbreeding depression implementation
```

## Key Concepts

### Inbreeding Coefficient (F)

| Relationship | F Value |
|-------------|---------|
| Unrelated | 0.0 |
| Second cousins | 0.0625 |
| First cousins | 0.125 |
| Half-siblings | 0.25 |
| Full siblings / Parent-offspring | 0.25 |
| Self-fertilization | 0.5 |

### Fitness Reduction

Fitness follows the formula: **W = exp(-B × F)** where B ~ 1.5

At F = 0.5, fitness reduction is typically 30-50%.

## Quick Start

### Basic Usage

```rust
use sim_core::genetics::{InbreedingTracker, can_reproduce_inbreeding};
use sim_core::lineage::LineageTree;

// Calculate F coefficient from pedigree
let f = InbreedingTracker::from_pedigree(parent1_id, parent2_id, &lineage);

// Check if reproduction should be allowed
if can_reproduce_inbreeding(agent1_id, agent2_id, &lineage) {
    // Proceed with reproduction
}
```

### Agent Integration

```rust
use sim_core::Agent;

// When creating offspring
let mut child = Agent::new(new_id, position, genome, tick, sex);
child.initialize_inbreeding(parent1_id, Some(parent2_id), &lineage);

// Check inbreeding status
if child.has_inbreeding_depression() {
    let f = child.inbreeding_coefficient();
    let fitness = child.inbreeding_fitness();
    println!("Inbreeding depression detected: F={}, fitness={}", f, fitness);
}
```

### Population Statistics

```rust
use sim_core::genetics::compute_population_inbreeding_stats;

let trackers: Vec<InbreedingTracker> = agents.iter()
    .map(|a| a.inbreeding)
    .collect();

let stats = compute_population_inbreeding_stats(&trackers);
println!("Mean F: {}", stats.mean_f);
println!("Inbred fraction: {}", stats.inbred_fraction);
```

## API Reference

### InbreedingTracker

- `new()` - Create tracker with F = 0 (outbred)
- `with_f_coefficient(f)` - Create tracker with specific F
- `from_pedigree(p1, p2, lineage)` - Calculate F from ancestry
- `compute_fitness(genome)` - Compute fitness from deleterious loci
- `apply_energy_penalty(base_energy)` - Apply energy reduction
- `mortality_risk_increase()` - Get additional mortality risk

### Utility Functions

- `can_reproduce_inbreeding(a1, a2, lineage)` - Check mating compatibility
- `calculate_offspring_f(p1, p2, lineage)` - Predict offspring F
- `predict_offspring_fitness(...)` - Predict offspring fitness
- `describe_relationship(f)` - Get human-readable relationship
- `is_inbred(f)` / `is_severe_inbreeding(f)` - Classification helpers

## Related Documentation

📁 [Simulation Documentation](../../../docs/simulation/)

📁 [Layer 2: Genealogy Spec](../../../docs/simulation/layer-specs/layer-1-genealogy-spec.md)

📁 [Simulation Documentation](../../../docs/simulation/)

## Constants

- `INBREEDING_AVOIDANCE_THRESHOLD` = 0.125 (first cousin level)
- `MIN_REPRODUCTION_FITNESS` = 0.7
- `DEFAULT_B_PARAMETER` = 1.5 (fitness decay parameter)

## References

- Charlesworth & Willis (2009): The genetics of inbreeding depression
- Keller & Waller (2002): Inbreeding effects in wild populations
- Wright (1922): Coefficients of inbreeding and relationship
