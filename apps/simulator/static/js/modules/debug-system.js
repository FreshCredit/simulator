/**
 * Debug System Module
 *
 * Comprehensive debugging and diagnostics for the Agent Society Simulator.
 * Provides frame tracking, agent position history, connection monitoring,
 * and diagnostic utilities.
 *
 * Can be tree-shaken in production builds when DEBUG flag is false.
 *
 * @module DebugSystem
 * @author FreshCredit Engineering
 * @version 1.0.0
 */

/**
 * Configuration for debug system
 */
const DEBUG_CONFIG = {
    enabled: typeof window !== 'undefined' && window.location.search.includes('debug=true'),
    maxFrameHistory: 50,
    maxAgentHistory: 100,
    frozenAgentThreshold: 20,
    logToConsole: true,
    logToEventLog: true,
};

/**
 * DebugSystem - Comprehensive debugging utilities
 *
 * Tracks:
 * - Frame receipt history
 * - Agent position history
 * - Connection events
 * - Movement statistics
 * - Frame flow health
 *
 * @example
 * const debug = new DebugSystem();
 * debug.recordFrameReceipt(frame);
 * debug.recordAgentPosition(agent, frame.tick);
 * console.log(debug.getStats());
 */
export class DebugSystem extends EventTarget {
    /**
     * Create a new DebugSystem
     * @param {Object} options - Configuration options
     * @param {boolean} [options.enabled=true] - Whether debugging is enabled
     * @param {number} [options.maxFrameHistory=50] - Max frames to keep in history
     * @param {number} [options.maxAgentHistory=100] - Max positions per agent
     * @param {number} [options.frozenAgentThreshold=20] - Ticks before warning
     */
    constructor(options = {}) {
        super();
        this.config = { ...DEBUG_CONFIG, ...options };

        // Frame tracking
        this.frameHistory = [];
        this.frameStats = {
            framesReceived: 0,
            framesRendered: 0,
            lastFrameTime: 0,
            avgFrameInterval: 0,
            frameIntervals: [],
        };

        // Agent tracking
        this.agentPositionHistory = {};
        this.movementStats = {
            totalMovement: 0,
            samples: 0,
            frozenAgentTicks: {},
        };

        // Connection tracking
        this.connectionEvents = [];

        // Health monitoring
        this.healthCheckInterval = null;
    }

    /**
     * Check if debugging is enabled
     * @returns {boolean}
     */
    isEnabled() {
        return this.config.enabled;
    }

    /**
     * Enable/disable debugging at runtime
     * @param {boolean} enabled
     */
    setEnabled(enabled) {
        this.config.enabled = enabled;
        this.dispatchEvent(new CustomEvent('enabledChanged', { detail: { enabled } }));
    }

    /**
     * Log a debug message
     * @param {string} category - Log category (e.g., 'CONN', 'FRAME', 'AGENT')
     * @param {string} message - Log message
     * @param {*} [data] - Optional data to log
     */
    log(category, message, data = null) {
        if (!this.isEnabled()) return;

        const timestamp = new Date().toISOString();
        const prefix = `[Debug][${category}]`;

        if (this.config.logToConsole) {
            if (data !== null) {
                console.log(`${prefix} ${message}`, data);
            } else {
                console.log(`${prefix} ${message}`);
            }
        }

        // Log critical events to event log
        if (this.config.logToEventLog && (category === 'ERROR' || category === 'FROZEN' || category === 'CONN')) {
            this.dispatchEvent(
                new CustomEvent('criticalEvent', {
                    detail: { category, message, data, timestamp },
                })
            );
        }
    }

    /**
     * Record frame receipt for analysis
     * @param {Object} frame - Frame data
     */
    recordFrameReceipt(frame) {
        if (!this.isEnabled() || !frame) return;

        const now = performance.now();
        const receipt = {
            tick: frame.tick,
            timestamp: now,
            agentCount: frame.agents?.length || 0,
            aliveCount: frame.stats?.alive_agents || 0,
        };

        this.frameHistory.push(receipt);
        if (this.frameHistory.length > this.config.maxFrameHistory) {
            this.frameHistory.shift();
        }

        // Calculate frame interval
        if (this.frameStats.lastFrameTime > 0) {
            const interval = now - this.frameStats.lastFrameTime;
            this.frameStats.frameIntervals.push(interval);
            if (this.frameStats.frameIntervals.length > 10) {
                this.frameStats.frameIntervals.shift();
            }
            this.frameStats.avgFrameInterval =
                this.frameStats.frameIntervals.reduce((a, b) => a + b, 0) /
                this.frameStats.frameIntervals.length;
        }

        this.frameStats.lastFrameTime = now;
        this.frameStats.framesReceived++;

        // Record agent positions
        if (frame.agents) {
            frame.agents.forEach((agent) => this.recordAgentPosition(agent, frame.tick));
        }

        // Throttled logging
        if (frame.tick <= 5 || frame.tick % 50 === 0) {
            this.log('Frame', `Received tick=${frame.tick}`, {
                agents: receipt.agentCount,
                alive: receipt.aliveCount,
                avgInterval: this.frameStats.avgFrameInterval.toFixed(1) + 'ms',
            });
        }
    }

    /**
     * Record agent position for movement tracking
     * @param {Object} agent - Agent data
     * @param {number} tick - Current tick
     */
    recordAgentPosition(agent, tick) {
        if (!this.isEnabled()) return;

        const id = agent.agent_id;
        if (!this.agentPositionHistory[id]) {
            this.agentPositionHistory[id] = [];
        }

        const history = this.agentPositionHistory[id];
        const prevPos = history.length > 0 ? history[history.length - 1] : null;

        history.push({
            tick,
            x: agent.x,
            y: agent.y,
            energy: agent.energy,
            alive: agent.alive,
            timestamp: performance.now(),
        });

        // Keep only recent history
        if (history.length > this.config.maxAgentHistory) {
            history.shift();
        }

        // Check for frozen agents
        if (prevPos && agent.alive) {
            const dx = agent.x - prevPos.x;
            const dy = agent.y - prevPos.y;
            const moved = Math.abs(dx) > 0.001 || Math.abs(dy) > 0.001;

            if (!moved) {
                this.movementStats.frozenAgentTicks[id] = (this.movementStats.frozenAgentTicks[id] || 0) + 1;

                // Warn if agent hasn't moved for threshold ticks
                if (this.movementStats.frozenAgentTicks[id] === this.config.frozenAgentThreshold) {
                    this.log('FROZEN', `Agent ${id} hasn't moved for ${this.config.frozenAgentThreshold} ticks`, {
                        pos: `(${agent.x.toFixed(2)}, ${agent.y.toFixed(2)})`,
                        energy: agent.energy,
                        age: agent.age,
                    });
                }
            } else {
                this.movementStats.frozenAgentTicks[id] = 0;
                this.movementStats.totalMovement += Math.sqrt(dx * dx + dy * dy);
                this.movementStats.samples++;
            }
        }
    }

    /**
     * Record connection event
     * @param {string} event - Event type
     * @param {*} [details] - Event details
     */
    recordConnectionEvent(event, details = null) {
        if (!this.isEnabled()) return;

        const entry = {
            timestamp: performance.now(),
            event,
            details,
        };

        this.connectionEvents.push(entry);
        if (this.connectionEvents.length > 20) {
            this.connectionEvents.shift();
        }

        this.log('CONN', event, details);
    }

    /**
     * Record that a frame was rendered
     */
    recordFrameRendered() {
        this.frameStats.framesRendered++;
    }

    /**
     * Get current debug statistics
     * @returns {Object} Debug statistics
     */
    getStats() {
        return {
            frames: {
                received: this.frameStats.framesReceived,
                rendered: this.frameStats.framesRendered,
                avgInterval: this.frameStats.avgFrameInterval,
                history: this.frameHistory,
            },
            movement: {
                avgMovement:
                    this.movementStats.samples > 0
                        ? this.movementStats.totalMovement / this.movementStats.samples
                        : 0,
                frozenAgents: Object.entries(this.movementStats.frozenAgentTicks)
                    .filter(([_, ticks]) => ticks > 10)
                    .map(([id, ticks]) => ({ agentId: parseInt(id), frozenTicks: ticks })),
            },
            agents: {
                tracked: Object.keys(this.agentPositionHistory).length,
            },
        };
    }

    /**
     * Get movement history for a specific agent
     * @param {number} agentId - Agent ID
     * @returns {Object|null} Movement history
     */
    getAgentHistory(agentId) {
        const history = this.agentPositionHistory[agentId];
        if (!history) {
            this.log('Debug', `No history found for agent ${agentId}`);
            return null;
        }

        // Calculate movement statistics
        let totalDistance = 0;
        const movementPoints = [];

        for (let i = 1; i < history.length; i++) {
            const dx = history[i].x - history[i - 1].x;
            const dy = history[i].y - history[i - 1].y;
            const dist = Math.sqrt(dx * dx + dy * dy);
            totalDistance += dist;

            movementPoints.push({
                tick: history[i].tick,
                from: `(${history[i - 1].x.toFixed(2)}, ${history[i - 1].y.toFixed(2)})`,
                to: `(${history[i].x.toFixed(2)}, ${history[i].y.toFixed(2)})`,
                distance: dist.toFixed(4),
                energy: history[i].energy,
            });
        }

        const result = {
            agentId,
            totalPositions: history.length,
            totalDistance: totalDistance.toFixed(4),
            currentPosition: history[history.length - 1],
            movementHistory: movementPoints.slice(-20), // Last 20 movements
        };

        return result;
    }

    /**
     * Check frame flow health
     * @returns {Object} Health status
     */
    checkFrameFlow() {
        const now = performance.now();
        const lastFrame = this.frameHistory[this.frameHistory.length - 1];

        if (!lastFrame) {
            return { healthy: false, issue: 'No frames received yet' };
        }

        const timeSinceLastFrame = now - lastFrame.timestamp;
        const healthy = timeSinceLastFrame < 1000; // Consider stale if > 1 second

        return {
            healthy,
            lastTick: lastFrame.tick,
            timeSinceLastFrame: Math.round(timeSinceLastFrame) + 'ms',
            avgFrameInterval: Math.round(this.frameStats.avgFrameInterval) + 'ms',
            framesInHistory: this.frameHistory.length,
            issue: healthy ? null : `No frame received for ${Math.round(timeSinceLastFrame)}ms`,
        };
    }

    /**
     * Run full diagnostic check
     * @returns {Object} Diagnostic results
     */
    runDiagnostics() {
        const results = {
            timestamp: new Date().toISOString(),
            frameFlow: this.checkFrameFlow(),
            stats: this.getStats(),
            sampleAgent: null,
        };

        // Get history for first available agent
        const agentIds = Object.keys(this.agentPositionHistory);
        if (agentIds.length > 0) {
            results.sampleAgent = this.getAgentHistory(parseInt(agentIds[0]));
        }

        this.log('Diagnostics', 'Full diagnostic results', results);
        return results;
    }

    /**
     * Cleanup dead agents from history
     * @param {Array} currentAgents - Current alive agents
     * @param {number} currentTick - Current tick
     * @param {number} [threshold=100] - Ticks before cleanup
     */
    cleanupDeadAgents(currentAgents, currentTick, threshold = 100) {
        const aliveIds = new Set(currentAgents.map((a) => a.agent_id));
        const allIds = Object.keys(this.agentPositionHistory);

        for (const id of allIds) {
            const agentId = parseInt(id);
            if (!aliveIds.has(agentId)) {
                const history = this.agentPositionHistory[id];
                const lastEntry = history[history.length - 1];
                if (lastEntry && currentTick - lastEntry.tick > threshold) {
                    delete this.agentPositionHistory[id];
                    delete this.movementStats.frozenAgentTicks[agentId];
                }
            }
        }
    }

    /**
     * Clear all debug data
     */
    clear() {
        this.frameHistory = [];
        this.frameStats = {
            framesReceived: 0,
            framesRendered: 0,
            lastFrameTime: 0,
            avgFrameInterval: 0,
            frameIntervals: [],
        };
        this.agentPositionHistory = {};
        this.movementStats = {
            totalMovement: 0,
            samples: 0,
            frozenAgentTicks: {},
        };
        this.connectionEvents = [];

        this.log('Debug', 'Debug data cleared');
    }

    /**
     * Start periodic health checks
     * @param {number} [intervalMs=5000] - Check interval
     */
    startHealthChecks(intervalMs = 5000) {
        this.stopHealthChecks();

        this.healthCheckInterval = setInterval(() => {
            const health = this.checkFrameFlow();
            if (!health.healthy) {
                this.log('ERROR', `Frame flow issue: ${health.issue}`);
                this.dispatchEvent(new CustomEvent('frameFlowIssue', { detail: health }));
            }
        }, intervalMs);
    }

    /**
     * Stop periodic health checks
     */
    stopHealthChecks() {
        if (this.healthCheckInterval) {
            clearInterval(this.healthCheckInterval);
            this.healthCheckInterval = null;
        }
    }

    /**
     * Destroy the debug system and cleanup
     */
    destroy() {
        this.stopHealthChecks();
        this.clear();
    }
}

/**
 * Singleton instance for application-wide debugging
 */
export const debug = new DebugSystem();

export default DebugSystem;
