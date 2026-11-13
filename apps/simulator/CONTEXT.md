# CONTEXT: `simulator` Agent Workspace

## YOU ARE HERE
**Path:** `freshcredit/apps/product/simulator/`  
**Type:** Agent Society Simulator (Visualization Interface)  
**Parent Isolation:** `freshcredit/apps/` — COMPLETELY ISOLATED from product crates

## YOUR SCOPE
Agent Society Simulator visualization interface with Rust backend and modern ES module frontend. This is a standalone visualization tool, completely separate from FreshCredit's product code.

**Critical Rule (SIM-001):** Simulator has ZERO dependencies on product crates (`apps/web`, `apps/api`, `apps/engine`, `apps/adapters`). It is a completely isolated subsystem.

**What you own:**
- Rust Axum backend (API for simulation data)
- Modern ES6 module frontend (91% test coverage)
- Agent behavior visualization
- Simulation playback controls
- Real-time data streaming via WebSocket
- Simulation artifact management

## YOUR BOUNDARIES

### Inbound (what can call you)
- **Browser Users:** Navigating to simulator UI
- **External Tools:** May call API for simulation data

### Outbound (what you can call)
- **`sim-service` API** — Simulation engine service (via HTTP API)
- **`sim-artifacts` crate** — Shared simulation schemas/types
- **No other product crates** — SIM-001 enforced

### Forbidden (what you must NOT do - CRITICAL)
- ❌ **NEVER import from `apps/web/`** — SIM-001 violation
- ❌ **NEVER import from `apps/product/api/`** — SIM-001 violation
- ❌ **NEVER import from `apps/product/engine/`** — SIM-001 violation
- ❌ **NEVER import from `apps/product/adapters/`** — SIM-001 violation
- ❌ **NEVER import from `crates/*` product crates** — Only `sim-artifacts` allowed
- ❌ **NEVER share code with product** — Duplicate code is acceptable for isolation

## LOCAL STRUCTURE
```
apps/product/simulator/
├── src/
│   ├── main.rs              # Axum server entry point
│   ├── lib.rs               # Library exports
│   ├── state.rs             # Application state
│   ├── error.rs             # Error handling
│   ├── config.rs            # Configuration
│   ├── handlers/            # HTTP handlers
│   │   ├── mod.rs
│   │   ├── simulation.rs    # Simulation endpoints
│   │   ├── agents.rs        # Agent data endpoints
│   │   └── websocket.rs     # WebSocket handler
│   ├── models/              # Internal models
│   └── services/            # Service layer
├── static/                  # Frontend assets (ES6 modules)
│   ├── js/                  # JavaScript modules
│   │   ├── main.js          # Entry point
│   │   ├── simulation.js    # Simulation controls
│   │   ├── agents.js        # Agent visualization
│   │   ├── charts.js        # Chart rendering
│   │   └── websocket.js     # WebSocket client
│   ├── css/                 # Stylesheets
│   └── index.html           # Main HTML page
├── tests/                   # Rust integration tests (0% coverage issue)
├── test/                    # JavaScript tests (91% coverage)
└── Cargo.toml
```

## EXTERNAL CONTRACTS
- **sim-service API:** HTTP/REST API for simulation engine
- **WebSocket:** Real-time streaming from sim-service
- **sim-artifacts crate:** Shared types (schemas, DTOs only - no business logic)

## ISOLATION RULES (SIM-001)
1. **Zero product dependencies** — Only `sim-*` crates allowed
2. **HTTP API only** — Communication with sim-service via HTTP, not direct imports
3. **Duplicate acceptable** — Better to duplicate code than create cross-dependencies
4. **Separate deployment** — Can be deployed independently of product
5. **Test coverage gap** — Rust code has 0% coverage (P1 issue to address)

## SIM-001 COMPLIANT DEPENDENCIES
```toml
[dependencies]
# ONLY simulator-specific dependencies allowed
sim-artifacts = { path = "../../crates/sim/sim-artifacts" }

# Web framework (Axum 0.8 - VERSION CONFLICT with workspace 0.7)
axum = "0.8"
tokio = { version = "1", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["fs", "cors"] }

# HTTP client for sim-service API
reqwest = { version = "0.12", features = ["json"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# WebSocket
tokio-tungstenite = "0.21"

# NO PRODUCT CRATES:
# ❌ engine = { path = "../engine" }
# ❌ freshcredit-types = { path = "../../crates/core/types" }
# ❌ adapters = { path = "../adapters" }
```

## VERSION CONFLICT NOTE
Simulator uses **Axum 0.8** while the rest of the workspace uses **Axum 0.7**. This is acceptable because:
1. SIM-001 enforces complete isolation
2. Simulator is not integrated with other Axum services
3. No shared middleware or extractors with product code

## KEY ENDPOINTS
```
GET /                      # Simulator UI (static files)
GET /api/simulations       # List simulations
GET /api/simulations/:id   # Get simulation details
GET /api/agents            # List agents
GET /api/agents/:id        # Get agent details
WS /ws                     # Real-time updates WebSocket
```

## CONFIGURATION
```bash
# Required
SIM_SERVICE_URL=http://localhost:8081
SIMULATOR_PORT=8082

# WebSocket
WS_HEARTBEAT_INTERVAL_SECS=30
```

## TEST COVERAGE ISSUE
```
JavaScript: 91% coverage (excellent)
Rust: 0% coverage (P1 issue)

Action needed: Add Rust integration tests in tests/ directory
```
