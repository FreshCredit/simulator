# Simulator Modules

Modular ES module architecture for the Agent Society Simulator.

## Quick Start

```javascript
import { SimulatorViewer } from '../viewer-modern.js';

const viewer = new SimulatorViewer('simulation-canvas');
viewer.init();
viewer.connect();
```

## Modules

### Core Modules

#### ConnectionManager
SSE connection lifecycle management with automatic reconnection.

```javascript
import { ConnectionManager } from './connection-manager.js';

const conn = new ConnectionManager('http://localhost:8082');
conn.addEventListener('frame', (e) => handleFrame(e.detail));
conn.connect();
```

**Events:** `connecting`, `connected`, `disconnected`, `frame`, `reconnecting`, `stalled`, `failed`, `error`

#### StateStore
Centralized reactive state management.

```javascript
import { store } from './state-store.js';

store.set('layers.agents', false);
store.subscribe('layers.agents', (value) => render());
```

#### APIClient
Type-safe HTTP API client with automatic retries.

```javascript
import { APIClient } from './api-client.js';

const api = new APIClient('http://localhost:8082');
await api.pause();
await api.setSpeed(2.0);
```

### Utility Modules

#### DebugSystem
Comprehensive debugging and diagnostics.

```javascript
import { DebugSystem } from './debug-system.js';

const debug = new DebugSystem();
debug.recordFrameReceipt(frame);
const stats = debug.getStats();
```

#### EventLog
Batched activity logging with smart auto-scroll.

```javascript
import { EventLog } from './event-log.js';

const log = new EventLog('log-container');
log.log('Event message', 'success', agentId, 'reproduction');
```

#### GridCache
Offscreen canvas caching for performance.

```javascript
import { GridCache } from './grid-cache.js';

const cache = new GridCache();
cache.build(128, 128, 10, config);
cache.draw(ctx, offsetX, offsetY);
```

#### PerformanceMonitor
Real-time performance tracking.

```javascript
import { PerformanceMonitor } from './perf-monitor.js';

const perf = new PerformanceMonitor();
perf.start();
perf.recordFrame();
console.log(perf.getFPS());
```

## Architecture

```
┌─────────────────────────────────────┐
│         SimulatorViewer             │
│  (Orchestrator - viewer-modern.js)  │
└──────────────┬──────────────────────┘
               │
    ┌──────────┼──────────┐
    │          │          │
    ▼          ▼          ▼
┌────────┐ ┌────────┐ ┌────────┐
│ConnectionManager │ │StateStore │ │APIClient │
└────────┘ └────────┘ └────────┘
    │          │          │
    ▼          ▼          ▼
┌────────┐ ┌────────┐ ┌────────┐
│DebugSystem │ │EventLog │ │GridCache │
└────────┘ └────────┘ └────────┘
```

## Testing

```bash
npm test
```

## Browser Support

- Chrome 80+
- Firefox 75+
- Safari 13.1+
- Edge 80+

Requires ES module support.
