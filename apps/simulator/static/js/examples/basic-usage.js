/**
 * Basic Usage Examples
 * Simple examples demonstrating how to use the modern simulator modules.
 */

// ============================================================================
// Example 1: Basic Viewer Setup
// ============================================================================

import { SimulatorViewer } from '../viewer-modern.js';

const viewer = new SimulatorViewer('simulation-canvas');
viewer.init();
viewer.connect();
viewer.setLayer('agents', true);
viewer.setFilter('trait', 'd');

// ============================================================================
// Example 2: Using State Store Directly
// ============================================================================

import { store } from '../modules/state-store.js';

const unsubscribe = store.subscribe('layers.agents', (visible) => {
    console.log('Agents layer:', visible ? 'visible' : 'hidden');
});

store.set('layers.agents', false);

store.batch(() => {
    store.set('layers.terrain', true);
    store.set('simulation.speed', 1.5);
});

// ============================================================================
// Example 3: Event Handling
// ============================================================================

import { ConnectionManager } from '../modules/connection-manager.js';

const conn = new ConnectionManager('http://localhost:8082');

conn.addEventListener('frame', (e) => {
    const frame = e.detail;
    console.log(`Tick ${frame.tick}: ${frame.agents.length} agents`);
});

conn.addEventListener('connected', () => console.log('Connected'));
conn.addEventListener('disconnected', () => console.log('Disconnected'));

conn.connect();

// ============================================================================
// Example 4: API Calls
// ============================================================================

import { APIClient } from '../modules/api-client.js';

const api = new APIClient('http://localhost:8082');

await api.pause();
await api.setSpeed(2.0);
await api.resume();

const status = await api.getStatus();
console.log('Current tick:', status.tick);

// ============================================================================
// Example 5: Performance Monitoring
// ============================================================================

import { PerformanceMonitor } from '../modules/perf-monitor.js';

const perf = new PerformanceMonitor();
perf.start();

function render() {
    // ... render code ...
    perf.recordFrame();
    requestAnimationFrame(render);
}

setInterval(() => {
    const stats = perf.getStats();
    console.log(`FPS: ${stats.fps.toFixed(1)} (${stats.rating})`);
}, 1000);
