/**
 * Viewer Adapter Layer
 *
 * Provides backwards compatibility for legacy code while using the new modular architecture.
 * This adapter wraps the modern viewer and exposes the legacy API.
 *
 * This allows gradual migration:
 * 1. Deploy adapter alongside legacy viewer
 * 2. Test with feature flag
 * 3. Switch over when confident
 * 4. Remove legacy viewer.js
 *
 * @module ViewerAdapter
 * @author FreshCredit Engineering
 * @version 2.0.0
 */

import { SimulatorViewer } from './viewer-modern.js';
import { store } from './modules/state-store.js';

/**
 * Configuration for adapter behavior
 */
const ADAPTER_CONFIG = {
    // Feature flag - set to true to use modern implementation
    useModern: typeof window !== 'undefined' && window.location.search.includes('modern=true'),
    // Log warnings when using deprecated APIs
    warnOnDeprecated: true,
    // Enable debug logging
    debug: false,
};

/**
 * ViewerAdapter - Backwards compatibility wrapper
 *
 * Wraps the modern viewer and provides the legacy SimulatorViewer API.
 * All methods delegate to the modern implementation.
 */
class ViewerAdapter {
    constructor() {
        this.modern = null;
        this.legacy = null;
        this.isModern = ADAPTER_CONFIG.useModern;

        // Initialize based on feature flag
        if (this.isModern) {
            this.initModern();
        }

        this.log('Adapter initialized, mode:', this.isModern ? 'modern' : 'legacy');
    }

    /**
     * Log adapter messages
     * @private
     */
    log(...args) {
        if (ADAPTER_CONFIG.debug) {
            console.log('[ViewerAdapter]', ...args);
        }
    }

    /**
     * Warn about deprecated API usage
     * @private
     */
    warnDeprecated(oldApi, newApi) {
        if (ADAPTER_CONFIG.warnOnDeprecated && this.isModern) {
            console.warn(
                `[ViewerAdapter] Deprecated API: "${oldApi}" is deprecated. ` +
                    `Use "${newApi}" instead. This will be removed in v3.0.`
            );
        }
    }

    /**
     * Initialize modern viewer
     * @private
     */
    initModern() {
        this.modern = new SimulatorViewer('simulation-canvas');
        this.modern.init();
        this.log('Modern viewer initialized');
    }

    /**
     * Switch to modern implementation
     */
    enableModern() {
        if (this.isModern) return;

        this.log('Switching to modern implementation');
        this.isModern = true;
        this.initModern();

        // Copy state from legacy if available
        if (this.legacy && window.SimulatorViewer) {
            // Migration logic here if needed
        }
    }

    /**
     * Switch to legacy implementation
     */
    enableLegacy() {
        if (!this.isModern) return;

        this.log('Switching to legacy implementation');
        this.isModern = false;

        if (this.modern) {
            this.modern.destroy();
            this.modern = null;
        }
    }

    // ============================================================================
    // Core Lifecycle (Legacy API)
    // ============================================================================

    /**
     * Initialize the viewer
     * @param {string} canvasId - Canvas element ID
     * @returns {boolean} Success
     */
    init(canvasId) {
        this.log('init() called');

        if (this.isModern) {
            return this.modern.init();
        }

        // Delegate to legacy
        return window.SimulatorViewerLegacy?.init(canvasId) ?? false;
    }

    /**
     * Connect to live stream
     */
    connect() {
        this.log('connect() called');

        if (this.isModern) {
            return this.modern.connect();
        }

        return window.SimulatorViewerLegacy?.connect();
    }

    /**
     * Disconnect from stream
     */
    disconnect() {
        this.log('disconnect() called');

        if (this.isModern) {
            return this.modern.disconnect();
        }

        return window.SimulatorViewerLegacy?.disconnect();
    }

    /**
     * Check if connected
     * @returns {boolean}
     */
    isConnected() {
        if (this.isModern) {
            return this.modern.isConnected();
        }

        return window.SimulatorViewerLegacy?.isConnected?.() ?? false;
    }

    // ============================================================================
    // Layer Control (Legacy API)
    // ============================================================================

    /**
     * Set layer visibility
     * @param {string} layerName - Layer name
     * @param {boolean} visible - Visibility
     */
    setLayer(layerName, visible) {
        this.warnDeprecated('setLayer(layer, visible)', 'store.set("layers.{layer}", visible)');

        if (this.isModern) {
            return this.modern.setLayer(layerName, visible);
        }

        return window.SimulatorViewerLegacy?.setLayer(layerName, visible);
    }

    /**
     * Set layer opacity
     * @param {string} layerName - Layer name
     * @param {number} opacity - Opacity (0-1)
     */
    setLayerOpacity(layerName, opacity) {
        this.warnDeprecated('setLayerOpacity(layer, opacity)', 'store.set("layerOpacity.{layer}", opacity)');

        if (this.isModern) {
            store.set(`layerOpacity.${layerName}`, opacity);
            return;
        }

        return window.SimulatorViewerLegacy?.setLayerOpacity(layerName, opacity);
    }

    /**
     * Set filter value
     * @param {string} filterPath - Filter path
     * @param {*} value - Filter value
     */
    setFilter(filterPath, value) {
        this.warnDeprecated('setFilter(path, value)', 'store.set("filters.{path}", value)');

        if (this.isModern) {
            return this.modern.setFilter(filterPath, value);
        }

        return window.SimulatorViewerLegacy?.setFilter(filterPath, value);
    }

    /**
     * Reset all filters
     */
    resetFilters() {
        if (this.isModern) {
            return this.modern.resetFilters();
        }

        return window.SimulatorViewerLegacy?.resetFilters?.();
    }

    /**
     * Get current filters
     * @returns {Object}
     */
    getFilters() {
        if (this.isModern) {
            return store.get('filters');
        }

        return window.SimulatorViewerLegacy?.getFilters?.();
    }

    // ============================================================================
    // Simulation Control (Legacy API)
    // ============================================================================

    /**
     * Set simulation speed
     * @param {number} factor - Speed factor
     */
    async setSpeed(factor) {
        if (this.isModern) {
            return this.modern.setSpeed(factor);
        }

        return window.SimulatorViewerLegacy?.setSpeed?.(factor);
    }

    /**
     * Pause simulation
     */
    async pause() {
        if (this.isModern) {
            return this.modern.pause();
        }

        return window.SimulatorViewerLegacy?.pause?.();
    }

    /**
     * Resume simulation
     */
    async resume() {
        if (this.isModern) {
            return this.modern.resume();
        }

        return window.SimulatorViewerLegacy?.resume?.();
    }

    /**
     * Stop simulation
     */
    async stop() {
        if (this.isModern) {
            return this.modern.stop();
        }

        return window.SimulatorViewerLegacy?.stop?.();
    }

    /**
     * Get simulation status
     * @returns {Promise<Object>}
     */
    async getStatus() {
        if (this.isModern) {
            return this.modern.api.getStatus();
        }

        return window.SimulatorViewerLegacy?.getStatus?.();
    }

    // ============================================================================
    // State Queries (Legacy API)
    // ============================================================================

    /**
     * Get current tick
     * @returns {number}
     */
    getCurrentTick() {
        if (this.isModern) {
            return this.modern.getCurrentTick();
        }

        return window.SimulatorViewerLegacy?.getCurrentTick?.() ?? 0;
    }

    /**
     * Get statistics
     * @returns {Object}
     */
    getStats() {
        if (this.isModern) {
            return this.modern.getStats();
        }

        return window.SimulatorViewerLegacy?.getStats?.();
    }

    // ============================================================================
    // Agent Selection (Legacy API)
    // ============================================================================

    /**
     * Open agent inspector modal
     * @param {number} agentId - Agent ID
     */
    openAgentModal(agentId) {
        this.warnDeprecated('openAgentModal(id)', 'viewer.selectAgent(id)');

        if (this.isModern) {
            return this.modern.selectAgent(agentId);
        }

        return window.SimulatorViewerLegacy?.openAgentModal?.(agentId);
    }

    /**
     * Close agent inspector modal
     */
    closeAgentModal() {
        this.warnDeprecated('closeAgentModal()', 'viewer.deselectAgent()');

        if (this.isModern) {
            return this.modern.deselectAgent();
        }

        return window.SimulatorViewerLegacy?.closeAgentModal?.();
    }

    // ============================================================================
    // Rendering (Legacy API)
    // ============================================================================

    /**
     * Render current frame
     */
    render() {
        if (this.isModern) {
            return this.modern.render();
        }

        return window.SimulatorViewerLegacy?.render?.();
    }

    /**
     * Set render config
     * @param {Object} config - Render configuration
     */
    setRenderConfig(config) {
        this.warnDeprecated('setRenderConfig(config)', 'Direct store mutation');

        if (this.isModern) {
            Object.entries(config).forEach(([key, value]) => {
                store.set(`renderConfig.${key}`, value);
            });
            return;
        }

        return window.SimulatorViewerLegacy?.setRenderConfig?.(config);
    }

    /**
     * Get render config
     * @returns {Object}
     */
    getRenderConfig() {
        if (this.isModern) {
            return store.get('renderConfig');
        }

        return window.SimulatorViewerLegacy?.getRenderConfig?.();
    }

    // ============================================================================
    // Event Log (Legacy API)
    // ============================================================================

    /**
     * Log an event
     * @param {string} message - Event message
     * @param {string} type - Event type
     * @param {number} [agentId] - Agent ID
     */
    logEvent(message, type = 'info', agentId = null) {
        if (this.isModern) {
            return this.modern.logEvent(message, type, agentId);
        }

        return window.SimulatorViewerLegacy?.logEvent?.(message, type, agentId);
    }

    /**
     * Get activity log
     * @returns {Array}
     */
    getActivityLog() {
        if (this.isModern) {
            return this.modern.eventLog.getEvents();
        }

        return window.SimulatorViewerLegacy?.getActivityLog?.();
    }

    /**
     * Clear activity log
     */
    clearActivityLog() {
        if (this.isModern) {
            return this.modern.eventLog.clear();
        }

        return window.SimulatorViewerLegacy?.clearActivityLog?.();
    }

    // ============================================================================
    // Export/Import (Legacy API)
    // ============================================================================

    /**
     * Export to PNG
     */
    exportToPNG() {
        this.warnDeprecated('exportToPNG()', 'viewer.exportState()');

        if (this.isModern) {
            // Modern implementation would use canvas.toDataURL()
            console.warn('exportToPNG not yet implemented in modern viewer');
            return;
        }

        return window.SimulatorViewerLegacy?.exportToPNG?.();
    }

    /**
     * Export simulation state
     * @returns {Object}
     */
    exportState() {
        if (this.isModern) {
            return this.modern.exportState();
        }

        return window.SimulatorViewerLegacy?.exportState?.();
    }

    /**
     * Import simulation state
     * @param {Object} state - State to import
     */
    importState(state) {
        if (this.isModern) {
            return this.modern.importState(state);
        }

        return window.SimulatorViewerLegacy?.importState?.(state);
    }

    // ============================================================================
    // Debug (Legacy API)
    // ============================================================================

    /**
     * Get debug state
     * @returns {Object}
     */
    getDebugState() {
        if (this.isModern) {
            return this.modern.getDebugState();
        }

        return window.SimulatorViewerLegacy?.getDebugState?.();
    }

    /**
     * Get frame history
     * @returns {Array}
     */
    getFrameHistory() {
        if (this.isModern) {
            return this.modern.debug?.frameHistory ?? [];
        }

        return window.SimulatorViewerLegacy?.getFrameHistory?.();
    }

    /**
     * Get agent position history
     * @returns {Object}
     */
    getAgentPositionHistory() {
        if (this.isModern) {
            return this.modern.debug?.agentPositionHistory ?? {};
        }

        return window.SimulatorViewerLegacy?.getAgentPositionHistory?.();
    }

    /**
     * Get connection events
     * @returns {Array}
     */
    getConnectionEvents() {
        if (this.isModern) {
            return this.modern.debug?.connectionEvents ?? [];
        }

        return window.SimulatorViewerLegacy?.getConnectionEvents?.();
    }

    /**
     * Clear debug state
     */
    clearDebugState() {
        if (this.isModern) {
            return this.modern.clearDebug();
        }

        return window.SimulatorViewerLegacy?.clearDebugState?.();
    }

    // ============================================================================
    // Configuration (Legacy API)
    // ============================================================================

    /**
     * Set grid configuration
     * @param {Object} config - Grid config
     */
    setGridConfig(config) {
        if (this.isModern) {
            store.set('simulation.gridSize', config);
            return;
        }

        return window.SimulatorViewerLegacy?.setGridConfig?.(config);
    }

    /**
     * Set zoom level
     * @param {number} zoom - Zoom level
     */
    setZoom(zoom) {
        if (this.isModern) {
            store.set('renderConfig.zoom', zoom);
            return;
        }

        return window.SimulatorViewerLegacy?.setZoom?.(zoom);
    }

    /**
     * Set pan offset
     * @param {number} x - X offset
     * @param {number} y - Y offset
     */
    setPan(x, y) {
        if (this.isModern) {
            store.batch(() => {
                store.set('renderConfig.panX', x);
                store.set('renderConfig.panY', y);
            });
            return;
        }

        return window.SimulatorViewerLegacy?.setPan?.(x, y);
    }

    // ============================================================================
    // Game Over / UI (Legacy API)
    // ============================================================================

    /**
     * Show export button
     */
    showExportButton() {
        if (this.isModern) {
            return this.modern.showSection('export-section');
        }

        return window.SimulatorViewerLegacy?.showExportButton?.();
    }

    /**
     * Hide export button
     */
    hideExportButton() {
        if (this.isModern) {
            return this.modern.hideSection('export-section');
        }

        return window.SimulatorViewerLegacy?.hideExportButton?.();
    }

    /**
     * Hide game over
     */
    hideGameOver() {
        if (this.isModern) {
            return this.modern.hideGameOver();
        }

        return window.SimulatorViewerLegacy?.hideGameOver?.();
    }

    /**
     * Start default simulation
     */
    startDefaultSimulation() {
        this.warnDeprecated('startDefaultSimulation()', 'Direct API call');

        if (this.isModern) {
            // Modern implementation
            console.warn('startDefaultSimulation not yet implemented');
            return;
        }

        return window.SimulatorViewerLegacy?.startDefaultSimulation?.();
    }

    // ============================================================================
    // AI Chat (Legacy API)
    // ============================================================================

    /**
     * Open AI chat
     */
    openAIChat() {
        if (window.FreshCreditAgent) {
            window.FreshCreditAgent.open?.();
        }
    }

    /**
     * Close AI chat
     */
    closeAIChat() {
        if (window.FreshCreditAgent) {
            window.FreshCreditAgent.close?.();
        }
    }

    /**
     * Toggle AI chat
     */
    toggleAIChat() {
        if (window.FreshCreditAgent) {
            window.FreshCreditAgent.toggle?.();
        }
    }

    /**
     * Check if AI chat is open
     * @returns {boolean}
     */
    isAIChatOpen() {
        return window.FreshCreditAgent?.isOpen?.() ?? false;
    }

    // ============================================================================
    // Frame Buffer (Legacy API)
    // ============================================================================

    /**
     * Get frame buffer
     * @returns {Array}
     */
    getFrameBuffer() {
        this.warnDeprecated('getFrameBuffer()', 'Internal implementation detail');

        if (this.isModern) {
            return []; // Modern viewer doesn't expose buffer directly
        }

        return window.SimulatorViewerLegacy?.getFrameBuffer?.();
    }

    /**
     * Get buffered frame
     * @param {number} tick - Tick number
     * @returns {Object|null}
     */
    getBufferedFrame(tick) {
        this.warnDeprecated('getBufferedFrame(tick)', 'Internal implementation detail');

        if (this.isModern) {
            return null;
        }

        return window.SimulatorViewerLegacy?.getBufferedFrame?.(tick);
    }
}

// Create singleton instance
const adapter = new ViewerAdapter();

// Export for ES modules
export { ViewerAdapter, adapter };
export default adapter;

// Expose to window for legacy compatibility
if (typeof window !== 'undefined') {
    window.SimulatorViewer = adapter;

    // Expose adapter controls
    window.ViewerAdapter = {
        enableModern: () => adapter.enableModern(),
        enableLegacy: () => adapter.enableLegacy(),
        isModern: () => adapter.isModern,
        getModernInstance: () => adapter.modern,
    };
}
