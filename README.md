# Agent Society Simulator

Open-source deterministic multi-agent simulation kernel and scientific research testbed.
Maintained by **The FreshCredit Org** under the Apache-2.0 / MIT Dual License.

## Architecture

This workspace contains the scientific and computational simulation engine for the Trust Stack research collection:

- **`crates/sim-core`**: Pure, deterministic simulation kernel (GTT, DOCAS neuromodulatory dynamics, prospect theory, evolutionary genetics, wetlab calibration). Zero network or database dependencies (SIM-001).
- **`crates/sim-artifacts`**: Artifact schema definitions, Apache Parquet writers, and tick replay frames.
- **`crates/sim-report`**: Formal hypothesis testing reports and POM validation outputs.
- **`crates/sim-runner`**: Batch parameter sweeps, Monte Carlo distributions, and statistical estimation.
- **`crates/sim-service`**: Job orchestration and telemetry streaming API for live visualization.
- **`crates/testnet-agent-bridge`**: Sidecar for cryptographic proof anchoring to Substrate blockchains.
- **`apps/simulator`**: Visualization viewer UI.

## Reproducibility & Research Alignment

All simulations correspond to the 6 peer-reviewed manuscripts in the research collection:
1. *The Value of Trust*
2. *Can Trust Be Measured?*
3. *The General Theory of Trust (GTT)*
4. *The DOCAS Framework*
5. *The Fresh Protocol (TFP)*
6. *FreshCredit Sovereign Financial Infrastructure*

Artifacts, pre-registered experimental logs, and Lean 4 formal proofs are mirrored at OSF and in `devonshigaki/trust-stack-research-collection`.

## Building and Testing

```bash
cargo build --workspace
cargo test --workspace
```
