---
title: "PDE4B Experiment Configurations"
date: "2026-02-16"
last_reviewed: "2026-03-05"
status: "stable"
category: "pde4b"
---
<!-- DOC-002: Stable -->
<!-- DOC-007: Active -->
<!-- Last Updated: 2026-02-16 -->

# PDE4B Experiment Configurations

TOML configuration files for PDE4B variance experiments testing trust emergence under different genetic variance conditions.

## Purpose

This directory contains configuration files for running PDE4B variance experiments, which test how trust emergence varies under different genetic variance conditions in the Agent Society Simulator.

## Directory Structure

```
crates/sim-runner/configs/experiments/pde4b/
├── README.md              # This file
├── zero_variance.toml     # Baseline control (no genetic diversity)
├── low_variance.toml      # Strong selection, narrow genetic range
├── medium_variance.toml   # "Goldilocks zone" optimal diversity
└── high_variance.toml     # High diversity, potential coordination challenges
```

## Related Documentation

📁 [Full Experiment Documentation](../../../../../docs/simulation/experiments/pde4b_variance/README.md)

📁 [Simulation Documentation](../../../../../docs/simulation/)

📁 [Simulation Documentation](../../../../../docs/simulation/)

## Quick Start

```bash
cargo run --bin sim-runner -- --config configs/experiments/pde4b/medium_variance.toml
```
