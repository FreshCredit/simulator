---
title: "sim-artifacts"
date: "2026-03-05"
last_reviewed: "2026-03-05"
status: "stable"
category: "simulation"
---
<!-- DOC-002: Stable -->
<!-- DOC-007: Active -->
<!-- SIM-001: Compliant -->
<!-- Last Updated: 2026-03-05 -->

# sim-artifacts

Simulation artifact schemas: manifest, metrics, and replay formats for the Agent Society Simulator.

## Purpose

This crate defines the data structures and serialization formats for all simulation artifacts. It provides schema stability guarantees (SIM-005) and supports multiple output formats for research reproducibility and visualization.

## SIM-001 Compliance

✅ **No freshcredit-* dependencies** - Only serialization libraries  
✅ **No network dependencies** - No tokio, axum, reqwest  
✅ **No database dependencies** - No libsql, sqlx  
✅ **Deterministic serialization** - Stable sort orders, no timestamps in hashes  

## Research Context

This crate supports the Generalized Trust Theory (GTT) research program by providing standardized artifact formats for:

- **P0 Gap Validation**: Artifact verification for all critical experiments
- **OSF Pre-registration**: Reproducible data formats for open science
- **Cross-platform Analysis**: Standard formats for external researchers

## Public API

```rust
use sim_artifacts::{Manifest, TickMetrics, ReplayFrame, SCHEMA_VERSION};

// Create simulation manifest
let manifest = Manifest::new(
    42,                              // seed
    "abc123".to_string(),            // run_id
);

// Serialize to JSON
let json = serde_json::to_string_pretty(&manifest)?;

// Create metrics
let metrics = TickMetrics {
    tick: 100,
    agent_count: 1000,
    average_trust: 0.65,
    // ... additional fields
};

// Create replay frame
let frame = ReplayFrame {
    tick: 100,
    agent_states: vec![...],
};
```

## Key Types

| Name | Module | Description |
|------|--------|-------------|
| `Manifest` | `manifest` | Run metadata, config, seeds, file references |
| `RunConfig` | `manifest` | Simulation configuration snapshot |
| `SeedConfig` | `manifest` | RNG seed information |
| `DatasetVersions` | `manifest` | Input data version tracking |
| `TickMetrics` | `metrics` | Per-tick aggregate statistics |
| `MetricsWriter` | `metrics` | Parquet output writer |
| `MetricsReader` | `metrics` | Parquet input reader |
| `ReplayFrame` | `replay` | Per-tick agent state snapshot |
| `ReplayWriter` | `replay` | JSONL output writer |
| `ReplayReader` | `replay` | JSONL input reader |

## Artifact Types

| Artifact | Format | Purpose | Schema Version |
|----------|--------|---------|----------------|
| `manifest.json` | JSON | Run metadata, config, seeds | 1.0 |
| `metrics.parquet` | Parquet | Per-tick aggregate metrics | 1.0 |
| `replay/*.jsonl` | JSONL | Per-agent state for visualization | 1.0 |

## Schema Stability (SIM-005)

### Version Compatibility

```
Schema Version: 1.0
├─ Writers: Must use current version
└─ Readers: Must support N-1 versions
   └─ Current: 1.0
   └─ Backward: 0.x (if any)
```

### Breaking Changes Policy

| Change Type | Version Impact | Example |
|-------------|----------------|---------|
| Add optional field | Patch | New metadata field |
| Add enum variant | Minor | New agent type |
| Remove field | Major | Delete metric |
| Rename field | Major | Change field name |
| Change type | Major | i32 → f64 |

## Determinism Guarantees

| Aspect | Guarantee | Implementation |
|--------|-----------|----------------|
| JSON keys | Sorted | `serde_json` with sorted maps |
| Floats | Fixed precision | 6 decimal places |
| Timestamps | ISO 8601 UTC | `chrono` with explicit timezone |
| UUIDs | v4 random | Documented in manifest |
| Enums | String labels | Never numeric discriminants |

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    sim-artifacts                            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐   │
│  │   manifest    │  │    metrics    │  │    replay     │   │
│  │   ─────────   │  │   ─────────   │  │   ─────────   │   │
│  │   Manifest    │  │  TickMetrics  │  │  ReplayFrame  │   │
│  │   RunConfig   │  │  MetricsWriter│  │  ReplayWriter │   │
│  │  SeedConfig   │  │  MetricsReader│  │  ReplayReader │   │
│  └───────────────┘  └───────────────┘  └───────────────┘   │
│                                                             │
│  SCHEMA_VERSION = "1.0"                                     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Dependencies

### External (Allowed per SIM-001)
- `serde` - Serialization framework
- `serde_json` - JSON format
- `chrono` - DateTime handling with serde
- `uuid` - Unique identifiers
- `parquet` - Columnar storage format

### Internal
- None (SIM-001 compliant)

## Testing

```bash
# Run all tests
cargo test -p sim-artifacts

# Test schema compatibility
cargo test -p sim-artifacts schema

# Test serialization round-trips
cargo test -p sim-artifacts roundtrip
```

## Example Output

### manifest.json
```json
{
  "schema_version": "1.0",
  "run_id": "abc123",
  "seed": 42,
  "created_at": "2026-03-05T12:00:00Z",
  "config": {
    "n_agents": 1000,
    "max_ticks": 10000
  },
  "files": {
    "metrics": "metrics.parquet",
    "replay": "replay/"
  }
}
```

## Related Documentation

- [sim-core README](../sim-core/README.md)
- [sim-runner README](../sim-runner/README.md)
- [Simulation Architecture](../../../../docs/_analysis/architecture/CRATE_ARCHITECTURE_BLUEPRINT.md)
