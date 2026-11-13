---
title: "testnet-agent-bridge"
date: "2026-03-05"
last_reviewed: "2026-03-05"
status: "active"
category: "service"
---
<!-- DOC-002: Stable -->
<!-- DOC-007: Active -->
<!-- Last Updated: 2026-03-05 -->

# testnet-agent-bridge

Testnet agent bridge for anchoring simulation results to FreshCredit blockchain testnet.

## Purpose

This crate translates simulation agent actions into Substrate extrinsics for the FreshCredit testnet. It provides an HTTP API for anchoring simulation hashes, verifying on-chain data, collecting simulation metrics, and comparing agent performance with statistical analysis.

## Responsibilities

- **Hash Anchoring**: Submit simulation result hashes to the blockchain
- **Hash Verification**: Verify anchored hashes exist on-chain
- **Metrics Collection**: Collect and store simulation metrics
- **Agent Comparison**: Statistical comparison between agents
- **Health Monitoring**: Testnet connection status and latency
- **Report Type Mapping**: Map simulation data to on-chain report types

## Public API (HTTP)

### Health Check
```bash
GET /health
```

### Anchor Hash
```bash
POST /anchor
Content-Type: application/json

{
  "hash": "a1b2c3...",  // 32-byte hex string
  "agent_id": "agent-001",
  "metadata": {
    "report_type": "simulation_metrics"
  }
}
```

### Verify Hash
```bash
GET /verify/{hash}
```

### Collect Metrics
```bash
POST /metrics/collect
Content-Type: application/json

{
  "simulation_id": "sim-001",
  "agent_id": "agent-001",
  "metrics": {
    "trust_score": 0.85,
    "transaction_count": 150
  },
  "fraud_score": 0.02
}
```

### Compare Agents
```bash
POST /metrics/compare
Content-Type: application/json

{
  "agent_a": "agent-001",
  "agent_b": "agent-002"
}
```

## Key Types

| Name | Type | Description |
|------|------|-------------|
| `AgentBridge` | struct | Main bridge component |
| `AnchorRequest` | struct | Hash anchoring request |
| `AnchorResponse` | struct | Anchoring result with block info |
| `HealthStatus` | struct | Connection health metrics |
| `SimulationMetrics` | struct | Agent performance metrics |
| `StatisticalTestResult` | struct | Agent comparison statistics |
| `MetricsCollector` | struct | In-memory metrics storage |

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                 Testnet Agent Bridge                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐                                            │
│  │   Axum      │                                            │
│  │   Router    │                                            │
│  │  ─────────  │                                            │
│  │  /health    │                                            │
│  │  /anchor    │                                            │
│  │  /verify    │                                            │
│  │  /metrics/* │                                            │
│  └──────┬──────┘                                            │
│         │                                                   │
│         ▼                                                   │
│  ┌──────────────────┐                                       │
│  │   AgentBridge    │                                       │
│  │  ─────────────── │                                       │
│  │  - subxt client  │                                       │
│  │  - signer mgmt   │                                       │
│  │  - hash gen      │                                       │
│  │  - report types  │                                       │
│  └────────┬─────────┘                                       │
│           │                                                 │
│     ┌─────┴─────┐                                           │
│     ▼           ▼                                           │
│  ┌──────┐  ┌─────────────────┐                              │
│  │Metrics │  │   subxt Client   │                              │
│  │Coll.  │  │  ─────────────── │                              │
│  │       │  │  - OnlineClient  │                              │
│  │       │  │  - Signer (sr25519)                              │
│  │       │  │  - Extrinsics    │                              │
│  └───┬───┘  └────────┬────────┘                              │
│      │               │                                       │
│      │               ▼                                       │
│      │        ┌─────────────────┐                            │
│      │        │  Testnet Node   │                            │
│      │        │  (Substrate)    │                            │
│      │        │  ─────────────  │                            │
│      │        │  fresh_credit   │                            │
│      │        │  pallet         │                            │
│      │        └─────────────────┘                            │
│      │                                                       │
│      └──────────────────────────────────────▶ (analysis)    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Report Types

| Type String | On-chain Type | Description |
|-------------|---------------|-------------|
| `simulation_metrics` | `PlaidFinancial` | Default simulation output |
| `identity_verification` | `IdentityVerification` | KYC/identity data |
| `transaction_history` | `TransactionHistory` | Agent transactions |
| `balance_snapshot` | `BalanceSnapshot` | Account balances |
| `consumer_preferences` | `ConsumerPreferences` | User preferences |
| `provider_requirements` | `ProviderRequirements` | Provider settings |

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `TESTNET_WS_URL` | No | `ws://localhost:9944` | WebSocket RPC endpoint |
| `SERVICE_ACCOUNT_SEED` | No | `//Alice` | Substrate account SURI |
| `BIND_ADDR` | No | `0.0.0.0:8080` | HTTP server bind address |

## Statistical Analysis

The metrics collector provides statistical comparison between agents:

| Statistic | Calculation |
|-----------|-------------|
| Mean | `Σx / n` |
| Std Dev | `√(Σ(x-μ)² / n)` |
| T-statistic | Two-sample t-test |
| P-value | Statistical significance |
| Significant difference | p < 0.05 |

## Dependencies

### External
- `axum` - HTTP web framework
- `tokio` - Async runtime
- `tower-http` - HTTP middleware (CORS, tracing)
- `subxt` - Substrate client
- `subxt-signer` - Transaction signing
- `serde` - Serialization
- `chrono` - Timestamp handling
- `hex` - Hex encoding
- `sha2` - Hash generation
- `statrs` - Statistical functions
- `sp-crypto-hashing` - Substrate hashing

### Internal
- None (standalone service)

## Testing

```bash
# Build the service
cargo build -p testnet-agent-bridge

# Run with local testnet
cargo run -p testnet-agent-bridge

# Run with custom endpoint
TESTNET_WS_URL=wss://testnet.freshcredit.com \
  cargo run -p testnet-agent-bridge
```

## Related Documentation

- [FreshCredit Testnet Documentation](../../../../docs/testnet/)
- [Subxt Documentation](https://docs.rs/subxt)
- [Architecture Overview](../../../../docs/_analysis/architecture/CRATE_ARCHITECTURE_BLUEPRINT.md)
