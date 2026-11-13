---
title: "sim-report"
date: "2026-03-05"
last_reviewed: "2026-03-05"
status: "active"
category: "simulation"
---
<!-- DOC-002: Stable -->
<!-- DOC-007: Active -->
<!-- SIM-001: Compliant -->
<!-- B-002: Critical blocker for research output -->
<!-- Last Updated: 2026-03-05 -->

# sim-report

Report generation for FreshCredit Agent Society Simulator.

## Purpose

This crate provides comprehensive report generation for simulation results, including POM validation reports, GTT hypothesis testing reports, statistical summaries, determinism certificates, and reproducibility information. It is a critical component (B-002) for research output and OSF pre-registration.

## SIM-001 Compliance

✅ **No freshcredit-* dependencies** - Only sim-* crates allowed  
✅ **No network dependencies** - No external API calls  
✅ **No database dependencies** - File-based reporting  
✅ **Deterministic outputs** - Same inputs produce identical reports  

## Research Context

This crate directly supports the 12-paper research series by generating:

- **POM Validation Reports**: Structured validation of Physics of Medicine targets
- **GTT Hypothesis Reports**: Statistical validation of Generalized Trust Theory
- **Determinism Certificates**: Proof of reproducibility for OSF pre-registration
- **Reproducibility Packages**: Complete environment documentation

## Public API

```rust
use sim_report::{ReportGenerator, ReportConfig, GttHypothesisResult};

// Create generator
let config = ReportConfig::default();
let generator = ReportGenerator::new(config);

// Generate POM validation report
let pom_report = generator.generate_pom_report(&validation_suite);
let markdown = pom_report.to_markdown();

// Generate hypothesis report
let hypothesis_report = generator.generate_hypothesis_report(&gtt_result);
let json = hypothesis_report.to_json()?;

// Generate full simulation report
let full_report = generator.generate_full_report(
    Some(&pom_suite),
    Some(&hypothesis_result),
    metadata,
);
full_report.save_to_dir(Path::new("./output"))?;

// Generate determinism certificate
let cert = generator.generate_determinism_certificate(&seeds, &checksums);
```

## Key Types

| Name | Type | Description |
|------|------|-------------|
| `ReportGenerator` | struct | Main report generation engine |
| `ReportConfig` | struct | Report metadata and formatting options |
| `PomValidationReport` | struct | POM target validation results |
| `HypothesisReport` | struct | GTT hypothesis test report |
| `FullSimulationReport` | struct | Comprehensive simulation report |
| `DeterminismCertificate` | struct | SIM-004 compliance certificate |
| `GttHypothesisResult` | struct | Hypothesis test statistics |
| `ConditionResults` | struct | Experimental condition aggregates |

## Report Types

### POM Validation Report

```markdown
# POM Validation Report

## Summary
- Total Targets: 150
- Passed: 142 (94.7%)
- Failed: 8 (5.3%)
- P0 Pass Rate: 100% (25/25)

## Categories
- DOCAS: 12/12 passed
- Economics: 8/10 passed
- Culture: 15/15 passed
...
```

### GTT Hypothesis Report

| Statistic | Value |
|-----------|-------|
| Effect Size (d) | 0.65 |
| 95% CI | [0.42, 0.88] |
| t-statistic | 3.24 |
| p-value | 0.0021 |
| Supported | ✅ Yes |

### Determinism Certificate

```
┌─────────────────────────────────────────────────────────────┐
│              DETERMINISM CERTIFICATE                        │
├─────────────────────────────────────────────────────────────┤
│ Seeds tested: [42, 123, 456, 789, 999]                      │
│ Checksums:   [a1b2c3..., d4e5f6..., ...]                    │
│                                                             │
│ ✅ All checksums match                                      │
│ ✅ SIM-004 compliant                                        │
│                                                             │
│ Generated: 2026-03-05T12:00:00Z                             │
└─────────────────────────────────────────────────────────────┘
```

## Report Formats

| Format | Extension | Use Case |
|--------|-----------|----------|
| Markdown | `.md` | Human-readable research reports |
| JSON | `.json` | Machine-readable analysis |
| HTML | `.html` | Interactive web reports (planned) |
| PDF | `.pdf` | Publication submission (planned) |

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    sim-report                               │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────┐                                        │
│  │ ReportGenerator │                                        │
│  │  ───────────────│                                        │
│  │  generate_*()   │                                        │
│  └────────┬────────┘                                        │
│           │                                                 │
│     ┌─────┴─────┬─────────────┬─────────────────┐          │
│     ▼           ▼             ▼                 ▼          │
│  ┌──────┐  ┌─────────┐  ┌──────────┐  ┌──────────────┐    │
│  │templates│  │hypothesis│  │    pom     │  │determinism   │    │
│  │ mod.rs │  │_types.rs │  │_validation │  │_certificate │    │
│  └──────┘  └─────────┘  └──────────┘  └──────────────┘    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Dependencies

### External (Allowed per SIM-001)
- `serde` - Serialization
- `serde_json` - JSON output
- `toml` - Configuration parsing
- `chrono` - Timestamps
- `handlebars` - Template engine

### Internal (SIM-001 Compliant)
- `sim-core` - POM validation types
- `sim-artifacts` - Schema types

## Configuration

```rust
ReportConfig {
    title: "FreshCredit Agent Society Simulation Report".to_string(),
    author: "FreshCredit Research".to_string(),
    institution: "FreshCredit Labs".to_string(),
    version: "1.0.0".to_string(),
    include_raw_data: false,
    confidence_level: 0.95,
    date_format: "%Y-%m-%d %H:%M:%S UTC".to_string(),
}
```

## Testing

```bash
# Run all tests
cargo test -p sim-report

# Test report generation
cargo test -p sim-report report

# Test determinism certificates
cargo test -p sim-report determinism
```

## Related Documentation

- [sim-core README](../sim-core/README.md)
- [sim-runner README](../sim-runner/README.md)
- [P0 Implementation Plan](../../../../docs/simulation/audit/P0_IMPLEMENTATION_PLAN.md)
