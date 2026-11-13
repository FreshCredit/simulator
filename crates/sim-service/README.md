---
title: "sim-service"
date: "2026-03-05"
last_reviewed: "2026-03-05"
status: "active"
category: "simulation"
---
<!-- DOC-002: Stable -->
<!-- DOC-007: Active -->
<!-- SIM-001: Compliant -->
<!-- Last Updated: 2026-03-05 -->

# sim-service

Job orchestration API for Agent Society Simulator.

## Purpose

This crate provides the HTTP API layer for the simulation system, serving as the bridge between the UI (`apps/simulator`) and the execution layer (`sim-runner`). It handles artifact serving, job submission, live simulation streaming, and blockchain anchoring.

## SIM-001 Compliance

✅ **Only sim-* dependencies** - sim-core, sim-artifacts  
⚠️ **Axum/tokio allowed** - HTTP server (network only for API)  
✅ **No freshcredit-* dependencies** - No product crate dependencies  
✅ **No database dependencies** - File-based artifact storage  

## Architecture (§5.2)

| Component | Responsibility |
|-----------|----------------|
| `apps/simulator` | UI - reads artifacts via this API |
| **`sim-service`** | **Job API, artifact serving, streaming** |
| `sim-runner` | Execution harness |
| `sim-core` | Deterministic kernel |

## Public API

```rust
use sim_service::{create_router, ExecutorConfig, AppState};

// Create router with state
let state = AppState::default();
let app = create_router().with_state(state);

// Or use pre-configured executor
let config = ExecutorConfig::default();
```

## HTTP Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |
| GET | `/api/live` | SSE stream of live simulation |
| GET | `/api/runs` | List available simulation runs |
| GET | `/api/runs/:id/manifest` | Get manifest.json for a run |
| GET | `/api/runs/:id/replay/:tick` | Get replay frame for a tick |
| POST | `/api/jobs` | Submit new simulation job |
| GET | `/api/jobs/:id` | Get job status |

## Key Modules

| Module | Purpose |
|--------|---------|
| `api` | HTTP route handlers |
| `artifacts` | Artifact file serving |
| `auth` | API authentication |
| `blockchain` | Hash anchoring to testnet |
| `events` | Event streaming (SSE/WebSocket) |
| `executor` | Job execution management |
| `health` | Health check endpoints |
| `jobs` | Job queue management |
| `live` | Live simulation streaming |
| `memory` | In-memory state management |
| `orchestrator` | Job orchestration |
| `rate_limit` | API rate limiting |
| `replay` | Replay frame serving |
| `router` | Axum router configuration |
| `streaming` | Real-time data streaming |
| `sweep` | Parameter sweep jobs |
| `validation` | Request validation |
| `visualization` | Visualization data API |

## Live Streaming

```
┌─────────────────────────────────────────────────────────────┐
│                    Live Simulation Stream                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Client                     Server                          │
│  ──────                     ──────                          │
│     │                          │                            │
│     │  GET /api/live          │                            │
│     │────────────────────────▶│                            │
│     │                          │                            │
│     │  SSE: tick 0 metrics    │                            │
│     │◀────────────────────────│                            │
│     │  SSE: tick 1 metrics    │                            │
│     │◀────────────────────────│                            │
│     │  SSE: tick N metrics    │                            │
│     │◀────────────────────────│                            │
│     │                          │                            │
│     │  Connection closed      │                            │
│     │◀────────────────────────│                            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Enterprise Compliance

The service initializes enterprise compliance features on startup:

| Feature | Rule | Status |
|---------|------|--------|
| Audit logging | ENT-001 | ✅ Initialized |
| RBAC permissions | ENT-002 | ✅ Initialized |
| Multi-tenant isolation | ENT-003 | ✅ Initialized |
| SSO support | ENT-004 | ✅ Initialized |

## Blockchain Integration

| Feature | Description |
|---------|-------------|
| Hash anchoring | Anchor simulation results to testnet |
| Verification | Verify anchored hashes on-chain |
| Report types | Identity, Transaction, Balance, etc. |

## Dependencies

### External (Allowed per SIM-001)
- `axum` - HTTP web framework
- `tokio` - Async runtime
- `tower-http` - HTTP middleware
- `serde_json` - JSON serialization
- `uuid` - Request identification
- `chrono` - Timestamp handling
- `parking_lot` - Sync primitives
- `tokio-stream` - Streaming utilities
- `tower_governor` - Rate limiting
- `sha2` - Hashing for blockchain

### Internal (SIM-001 Compliant)
- `sim-core` - Simulation types
- `sim-artifacts` - Artifact schemas

## Configuration

```rust
ExecutorConfig {
    max_concurrent_jobs: 4,
    artifact_storage_path: PathBuf::from("./artifacts"),
    enable_live_streaming: true,
    rate_limit_requests_per_second: 100,
}
```

## CLI Usage

```bash
# Start the service
cargo run -p sim-service

# With custom config
cargo run -p sim-service -- --port 8080 --artifacts ./data

# Production mode
cargo run -p sim-service --release
```

## Testing

```bash
# Run all tests
cargo test -p sim-service

# Test API endpoints
cargo test -p sim-service api

# Test streaming
cargo test -p sim-service streaming
```

## Related Documentation

- [sim-core README](../sim-core/README.md)
- [sim-runner README](../sim-runner/README.md)
- [Simulation Architecture](../../../../docs/_analysis/architecture/CRATE_ARCHITECTURE_BLUEPRINT.md)
