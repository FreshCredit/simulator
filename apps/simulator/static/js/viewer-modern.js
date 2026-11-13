/**
 * Agent Society Simulator - Modern Viewer Module
 *
 * SIM-002: UI reads live stream from sim-service - no sim-core imports
 * SIM-003: Simulation agents, NOT FreshCredit AI agents
 *
 * This is the modernized, modular implementation of the viewer.
 * It uses ES modules for better maintainability, testability, and performance.
 *
 * Architecture:
 * - ConnectionManager: SSE connection lifecycle
 * - StateStore: Centralized state management
 * - APIClient: HTTP/SSE API communication
 * - DebugSystem: Debugging and diagnostics
 * - EventLog: Activity logging
 *
 * @module SimulatorViewer
 * @author FreshCredit Engineering
 * @version 2.0.0
 */

import { ConnectionManager } from './modules/connection-manager.js';
import { StateStore, DEFAULT_STATE } from './modules/state-store.js';
import { APIClient } from './modules/api-client.js';
import { DebugSystem } from './modules/debug-system.js';
import { EventLog } from './modules/event-log.js';

/**
 * Configuration constants
 */
const CONFIG = {
    frameDuration: 100, // ms, matches server tick rate
    gridSize: { width: 128, height: 128 },
    apiUrl: window.SIM_SERVICE_URL || '',
    canvasId: 'simulation-canvas',
};

/**
 * SimulatorViewer - Main application class
 *
 * Orchestrates all modules and provides the public API.
 */
export class SimulatorViewer {
    /**
     * Create a new SimulatorViewer
     * @param {string} canvasId - ID of the canvas element
     * @param {Object} options - Configuration options
     */
    constructor(canvasId = CONFIG.canvasId, options = {}) {
        this.canvasId = canvasId;
        this.canvas = document.getElementById(canvasId);
        this.ctx = this.canvas?.getContext('2d');

        // Initialize modules
        this.api = new APIClient(options.apiUrl || CONFIG.apiUrl);
        this.connection = new ConnectionManager(options.apiUrl || CONFIG.apiUrl);
        this.store = options.store || new StateStore();
        this.debug = options.debug !== false ? new DebugSystem() : null;
        this.eventLog = new EventLog('activity-log-content');

        // Rendering state
        this.renderPending = false;
        this.lastRenderTime = 0;

        // Bind methods
        this.handleFrame = this.handleFrame.bind(this);
        this.render = this.render.bind(this);
        this.handleCanvasClick = this.handleCanvasClick.bind(this);

        // Setup event listeners
        this.setupEventListeners();
    }

    /**
     * Setup all event listeners
     * @private
     */
    setupEventListeners() {
        // Connection events
        this.connection.addEventListener('frame', (e) => this.handleFrame(e.detail));
        this.connection.addEventListener('connected', () => {
            this.store.set('simulation.status', 'running');
            this.store.set('ui.connectionStatus', 'connected');
            this.eventLog.log('Connected to simulation', 'success');
            this.hideOverlay();
        });
        this.connection.addEventListener('disconnected', () => {
            this.store.set('ui.connectionStatus', 'disconnected');
            this.eventLog.log('Disconnected from simulation', 'warning');
        });
        this.connection.addEventListener('reconnecting', (e) => {
            this.store.set('ui.connectionStatus', 'reconnecting');
            this.eventLog.log(`Reconnecting (attempt ${e.detail.attempt})...`, 'warning');
        });
        this.connection.addEventListener('failed', () => {
            this.store.set('simulation.status', 'failed');
            this.store.set('ui.connectionStatus', 'failed');
            this.eventLog.log('Connection failed after max retries', 'danger');
        });

        // Canvas click
        if (this.canvas) {
            this.canvas.addEventListener('click', this.handleCanvasClick);
        }

        // Window resize
        window.addEventListener('resize', () => this.resizeCanvas());

        // Simulation start event from controls
        window.addEventListener('simulationStarted', (e) => this.handleSimulationStart(e.detail));
    }

    /**
     * Initialize the viewer
     * @returns {boolean} Success
     */
    init() {
        if (!this.canvas || !this.ctx) {
            console.error('[Viewer] Canvas not found');
            return false;
        }

        this.resizeCanvas();

        // Subscribe to state changes that require re-render
        this.store.subscribe('layers', () => this.scheduleRender());
        this.store.subscribe('filters', () => this.scheduleRender());
        this.store.subscribe('frame.current', () => this.scheduleRender());

        console.log('[Viewer] Initialized successfully');
        return true;
    }

    /**
     * Handle simulation start event
     * @param {Object} detail - Event detail
     * @private
     */
    handleSimulationStart(detail) {
        console.log('[Viewer] Simulation started:', detail);

        // Update grid size
        this.store.set('simulation.gridSize', CONFIG.gridSize);

        // Show UI sections
        this.showSection('mechanism-layers-section');
        this.showSection('visualization-layers-section');
        this.showSection('stats-section');
        this.showSection('activity-log-panel');

        // Reset state
        this.store.set('simulation.status', 'running');
        this.store.set('simulation.currentTick', 0);
        this.store.set('agents.all', []);

        // Clear event log
        this.eventLog.clear();
        this.eventLog.log('Simulation started', 'success');

        // Hide export button
        this.hideSection('export-section');
        this.hideGameOver();

        // Connect to stream
        this.connect();
    }

    /**
     * Handle incoming frame
     * @param {Object} frame - Frame data
     * @private
     */
    handleFrame(frame) {
        if (!frame) return;

        // Update debug system
        if (this.debug) {
            this.debug.recordFrameReceipt(frame);
        }

        // Handle simulation completion
        if (frame.status === 'Completed' || frame.status === 'Stopped') {
            this.handleSimulationEnd(frame.status);
            return;
        }

        // Update state
        const previousFrame = this.store.get('frame.current');
        this.store.batch(() => {
            this.store.set('frame.previous', previousFrame);
            this.store.set('frame.current', frame);
            this.store.set('simulation.currentTick', frame.tick);
            this.store.set('agents.all', frame.agents || []);
            this.store.set('agents.count', frame.agents?.length || 0);
            this.store.set('agents.aliveCount', frame.stats?.alive_agents || 0);
            this.store.set('stats', {
                totalAgents: frame.stats?.total_agents || 0,
                aliveAgents: frame.stats?.alive_agents || 0,
                verifiedAgents: frame.stats?.verified_agents || 0,
                groupCount: frame.stats?.group_count || 0,
            });
        });

        // Build agent map for O(1) lookups
        if (frame.agents) {
            frame.agentMap = new Map(frame.agents.map((a) => [a.agent_id, a]));
        }

        // Cleanup debug history
        if (this.debug && frame.agents) {
            this.debug.cleanupDeadAgents(frame.agents, frame.tick);
        }

        // Update tick display
        const tickDisplay = document.getElementById('tick-display');
        if (tickDisplay) {
            tickDisplay.textContent = frame.tick;
        }

        // Detect events
        this.detectEvents(frame);

        // Dispatch event for other modules
        window.dispatchEvent(new CustomEvent('simulatorFrame', { detail: frame }));
    }

    /**
     * Handle simulation end
     * @param {string} status - End status
     * @private
     */
    handleSimulationEnd(status) {
        this.store.set('simulation.status', status.toLowerCase());
        this.eventLog.log(`Simulation ${status.toLowerCase()}`, status === 'Completed' ? 'success' : 'info');

        this.showGameOver(status);
        this.showSection('export-section');
        this.disconnect();

        window.dispatchEvent(new CustomEvent('simulationEnded', { detail: { status } }));
    }

    /**
     * Detect and log significant events
     * @param {Object} frame - Frame data
     * @private
     */
    detectEvents(frame) {
        if (!frame.agents) return;

        const aliveAgents = frame.agents.filter((a) => a.alive);
        const previousAlive = this.store.get('agents.aliveCount') || 0;

        // Process frame events
        if (frame.game_events) {
            frame.game_events.forEach((event) => {
                this.eventLog.log(
                    `Game: ${event.game_type} between Agent #${event.agent_a} and #${event.agent_b} - ${event.outcome}`,
                    'info'
                );
            });
        }

        if (frame.encounter_events) {
            frame.encounter_events.forEach((event) => {
                this.eventLog.log(
                    `Encounter: Agent #${event.agent_a} met #${event.agent_b} (${event.encounter_type})`,
                    'info'
                );
            });
        }

        // Detect reproduction (new agents with parents)
        if (frame.tick > 0 && aliveAgents.length > previousAlive) {
            const newAgents = aliveAgents.filter(
                (a) => a.age === 0 && a.parent1_id !== null && a.parent1_id !== undefined
            );
            if (newAgents.length > 0) {
                this.eventLog.log(`${newAgents.length} agents reproduced`, 'success', null, 'reproduction');
            }
        }

        // Low population warning
        if (aliveAgents.length < 5 && previousAlive >= 5) {
            this.eventLog.log('WARNING: Population critically low!', 'danger');
        }

        // High population milestone
        if (aliveAgents.length >= 100 && previousAlive < 100) {
            this.eventLog.log('Milestone: 100 agents alive', 'success');
        }

        // Tick milestones
        if (frame.tick % 100 === 0 && frame.tick > 0) {
            this.eventLog.log(`Tick ${frame.tick}: ${aliveAgents.length} agents alive`, 'info');
        }
    }

    /**
     * Schedule a render on next animation frame
     * @private
     */
    scheduleRender() {
        if (this.renderPending) return;

        this.renderPending = true;
        requestAnimationFrame(() => {
            this.renderPending = false;
            this.render();
        });
    }

    /**
     * Render the current frame
     */
    render() {
        if (!this.ctx || !this.canvas) return;

        const startTime = performance.now();

        const frame = this.store.get('frame.current');
        const layers = this.store.get('layers');
        const filters = this.store.get('filters');

        // Setup canvas
        const dpr = window.devicePixelRatio || 1;
        this.ctx.setTransform(1, 0, 0, 1, 0, 0);
        this.ctx.scale(dpr, dpr);

        // Clear
        const width = this.canvas.clientWidth;
        const height = this.canvas.clientHeight;
        this.ctx.fillStyle = '#0a0a0c';
        this.ctx.fillRect(0, 0, width, height);

        // Render layers
        // Note: This is a simplified render - the full implementation would
        // use a dedicated Renderer module
        if (frame && frame.agents) {
            this.renderAgents(frame.agents, layers, filters);
        }

        // Record render time
        const renderTime = performance.now() - startTime;
        if (this.debug) {
            this.debug.recordFrameRendered();
        }

        this.lastRenderTime = renderTime;
    }

    /**
     * Render agents
     * @param {Array} agents - Agent array
     * @param {Object} layers - Layer visibility
     * @param {Object} filters - Active filters
     * @private
     */
    renderAgents(agents, layers, filters) {
        if (!layers.agents) return;

        // ROADMAP: Implement full agent rendering or delegate to Renderer module
        // For now, just draw simple circles
        const centerX = this.canvas.clientWidth / 2;
        const centerY = this.canvas.clientHeight / 2;
        const scale = Math.min(this.canvas.clientWidth, this.canvas.clientHeight) / 150;

        this.ctx.fillStyle = '#4ade80';

        agents.forEach((agent) => {
            // Apply filters
            if (filters.status === 'alive' && !agent.alive) return;

            const x = centerX + (agent.x - 64) * scale;
            const y = centerY + (agent.y - 64) * scale;

            this.ctx.beginPath();
            this.ctx.arc(x, y, 3, 0, Math.PI * 2);
            this.ctx.fill();
        });
    }

    /**
     * Handle canvas click for agent selection
     * @param {MouseEvent} event - Click event
     * @private
     */
    handleCanvasClick(event) {
        const frame = this.store.get('frame.current');
        if (!frame || !frame.agents) return;

        const rect = this.canvas.getBoundingClientRect();
        const clickX = event.clientX - rect.left;
        const clickY = event.clientY - rect.top;

        // Convert to grid coordinates
        const centerX = this.canvas.clientWidth / 2;
        const centerY = this.canvas.clientHeight / 2;
        const scale = Math.min(this.canvas.clientWidth, this.canvas.clientHeight) / 150;

        const gridX = (clickX - centerX) / scale + 64;
        const gridY = (clickY - centerY) / scale + 64;

        // Find closest agent using agentMap for O(1) lookups
        const clickRadius = 2;
        let closestAgent = null;
        let closestDistance = Infinity;

        // Check nearby cells
        const cellX = Math.floor(gridX);
        const cellY = Math.floor(gridY);

        for (let dx = -1; dx <= 1; dx++) {
            for (let dy = -1; dy <= 1; dy++) {
                // This would use spatial hash in full implementation
                // For now, scan all agents
                frame.agents.forEach((agent) => {
                    const dist = Math.sqrt(
                        Math.pow(agent.x - gridX, 2) + Math.pow(agent.y - gridY, 2)
                    );
                    if (dist <= clickRadius && dist < closestDistance) {
                        closestDistance = dist;
                        closestAgent = agent;
                    }
                });
            }
        }

        if (closestAgent) {
            this.selectAgent(closestAgent.agent_id);
        } else {
            this.deselectAgent();
        }
    }

    /**
     * Select an agent
     * @param {number} agentId - Agent ID
     */
    selectAgent(agentId) {
        this.store.set('agents.selected', agentId);
        this.eventLog.log(`Selected agent #${agentId}`, 'info', agentId);
        // ROADMAP: Open agent inspector modal
    }

    /**
     * Deselect current agent
     */
    deselectAgent() {
        this.store.set('agents.selected', null);
    }

    /**
     * Resize canvas to fill container
     */
    resizeCanvas() {
        if (!this.canvas) return;

        const container = this.canvas.parentElement;
        if (!container) return;

        const dpr = window.devicePixelRatio || 1;
        const displayWidth = container.clientWidth;
        const displayHeight = container.clientHeight;

        if (displayWidth === 0 || displayHeight === 0) return;

        this.canvas.style.width = displayWidth + 'px';
        this.canvas.style.height = displayHeight + 'px';
        this.canvas.width = displayWidth * dpr;
        this.canvas.height = displayHeight * dpr;

        this.scheduleRender();
    }

    /**
     * Connect to live simulation stream
     */
    connect() {
        this.connection.connect();
    }

    /**
     * Disconnect from simulation stream
     */
    disconnect() {
        this.connection.disconnect();
    }

    /**
     * Check if connected
     * @returns {boolean}
     */
    isConnected() {
        return this.connection.isConnected();
    }

    /**
     * Set layer visibility
     * @param {string} layerName - Layer name
     * @param {boolean} visible - Visibility
     */
    setLayer(layerName, visible) {
        this.store.set(`layers.${layerName}`, visible);
    }

    /**
     * Set filter value
     * @param {string} filterPath - Filter path
     * @param {*} value - Filter value
     */
    setFilter(filterPath, value) {
        this.store.set(`filters.${filterPath}`, value);
    }

    /**
     * Reset all filters
     */
    resetFilters() {
        this.store.batch(() => {
            this.store.set('filters.trait', 'all');
            this.store.set('filters.status', 'alive');
            this.store.set('filters.relativeDistanceThreshold', 10);
        });
    }

    /**
     * Set simulation speed
     * @param {number} factor - Speed factor
     */
    async setSpeed(factor) {
        try {
            await this.api.setSpeed(factor);
            this.store.set('simulation.speed', factor);
            this.eventLog.log(`Speed set to ${factor}x`, 'info');
        } catch (err) {
            this.eventLog.log(`Failed to set speed: ${err.message}`, 'danger');
        }
    }

    /**
     * Pause simulation
     */
    async pause() {
        try {
            await this.api.pause();
            this.store.set('simulation.status', 'paused');
            this.eventLog.log('Simulation paused', 'warning');
        } catch (err) {
            this.eventLog.log(`Failed to pause: ${err.message}`, 'danger');
        }
    }

    /**
     * Resume simulation
     */
    async resume() {
        try {
            await this.api.resume();
            this.store.set('simulation.status', 'running');
            this.eventLog.log('Simulation resumed', 'success');
        } catch (err) {
            this.eventLog.log(`Failed to resume: ${err.message}`, 'danger');
        }
    }

    /**
     * Stop simulation
     */
    async stop() {
        try {
            await this.api.stop();
            this.store.set('simulation.status', 'stopped');
            this.eventLog.log('Simulation stopped', 'info');
        } catch (err) {
            this.eventLog.log(`Failed to stop: ${err.message}`, 'danger');
        }
    }

    /**
     * Get current tick
     * @returns {number}
     */
    getCurrentTick() {
        return this.store.get('simulation.currentTick');
    }

    /**
     * Get statistics
     * @returns {Object}
     */
    getStats() {
        return this.store.get('stats');
    }

    /**
     * Show a UI section
     * @param {string} sectionId - Section ID
     * @private
     */
    showSection(sectionId) {
        const section = document.getElementById(sectionId);
        if (section) {
            section.classList.remove('hidden');
        }
    }

    /**
     * Hide a UI section
     * @param {string} sectionId - Section ID
     * @private
     */
    hideSection(sectionId) {
        const section = document.getElementById(sectionId);
        if (section) {
            section.classList.add('hidden');
        }
    }

    /**
     * Hide canvas overlay
     * @private
     */
    hideOverlay() {
        const overlay = document.getElementById('canvas-overlay');
        if (overlay) {
            overlay.classList.add('hidden');
        }
    }

    /**
     * Show game over screen
     * @param {string} status - Game status
     * @private
     */
    showGameOver(status) {
        const overlay = document.getElementById('game-over-overlay');
        if (overlay) {
            overlay.classList.remove('hidden');
        }

        if (window.PixelatedLoader) {
            window.PixelatedLoader.showGameOver();
        }
    }

    /**
     * Hide game over screen
     * @private
     */
    hideGameOver() {
        const overlay = document.getElementById('game-over-overlay');
        if (overlay) {
            overlay.classList.add('hidden');
        }

        if (window.PixelatedLoader) {
            window.PixelatedLoader.stop();
        }
    }

    /**
     * Run diagnostics
     * @returns {Object} Diagnostic results
     */
    runDiagnostics() {
        if (this.debug) {
            return this.debug.runDiagnostics();
        }
        return { error: 'Debug system not enabled' };
    }

    /**
     * Get debug state
     * @returns {Object|null}
     */
    getDebugState() {
        return this.debug?.getStats() || null;
    }

    /**
     * Clear debug data
     */
    clearDebug() {
        this.debug?.clear();
    }

    /**
     * Log an event
     * @param {string} message - Message
     * @param {string} type - Event type
     * @param {number} [agentId] - Agent ID
     */
    logEvent(message, type = 'info', agentId = null) {
        this.eventLog.log(message, type, agentId);
    }

    /**
     * Export simulation state
     * @returns {Object}
     */
    exportState() {
        return {
            store: this.store.get(),
            events: this.eventLog.getEvents(),
            timestamp: new Date().toISOString(),
        };
    }

    /**
     * Import simulation state
     * @param {Object} state - State to import
     */
    importState(state) {
        if (state.store) {
            // Merge state carefully
            Object.keys(state.store).forEach((key) => {
                this.store.set(key, state.store[key]);
            });
        }

        if (state.events) {
            // Restore event log
            state.events.forEach((event) => {
                this.eventLog.log(event.message, event.type, event.agentId);
            });
        }
    }

    /**
     * Destroy the viewer and cleanup
     */
    destroy() {
        this.disconnect();
        this.connection.destroy();
        this.debug?.destroy();
        this.eventLog.destroy();

        if (this.canvas) {
            this.canvas.removeEventListener('click', this.handleCanvasClick);
        }

        console.log('[Viewer] Destroyed');
    }
}

// Create singleton instance
let viewerInstance = null;

/**
 * Get or create the singleton viewer instance
 * @param {string} canvasId - Canvas ID
 * @returns {SimulatorViewer}
 */
export function getViewer(canvasId = CONFIG.canvasId) {
    if (!viewerInstance) {
        viewerInstance = new SimulatorViewer(canvasId);
    }
    return viewerInstance;
}

/**
 * Initialize the viewer (for legacy compatibility)
 * @param {string} canvasId - Canvas ID
 * @returns {boolean}
 */
export function init(canvasId = CONFIG.canvasId) {
    const viewer = getViewer(canvasId);
    return viewer.init();
}

// Expose to window for legacy compatibility
if (typeof window !== 'undefined') {
    window.SimulatorViewerModern = {
        getViewer,
        init,
        SimulatorViewer,
    };
}

export default SimulatorViewer;
