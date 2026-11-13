/**
 * Event Log Module
 *
 * Activity logging system for the Agent Society Simulator.
 * Provides batched event logging, scroll management, and retro terminal-style rendering.
 *
 * @module EventLog
 * @author FreshCredit Engineering
 * @version 1.0.0
 */

/**
 * Configuration for event logging
 */
const CONFIG = {
    maxEvents: 100,
    batchInterval: 3000, // 3 seconds
    defaultType: 'info',
};

/**
 * Event types with their styling
 */
const EVENT_TYPES = {
    info: { cssClass: 'info', icon: 'ℹ️' },
    success: { cssClass: 'success', icon: '✅' },
    warning: { cssClass: 'warning', icon: '⚠️' },
    danger: { cssClass: 'danger', icon: '❌' },
    event: { cssClass: 'event', icon: '📌' },
};

/**
 * EventLog - Activity logging system
 *
 * Features:
 * - Batched event processing
 * - Smart auto-scroll
 * - Scroll indicator
 * - Fullscreen toggle support
 * - XSS-safe HTML rendering
 *
 * @example
 * const eventLog = new EventLog('activity-log-content');
 * eventLog.log('Simulation started', 'success');
 * eventLog.log('Agent reproduced', 'event', 123, 'reproduction');
 */
export class EventLog extends EventTarget {
    /**
     * Create a new EventLog
     * @param {string} containerId - ID of the container element
     * @param {Object} options - Configuration options
     * @param {number} [options.maxEvents=100] - Maximum events to keep
     * @param {number} [options.batchInterval=3000] - Batch processing interval in ms
     */
    constructor(containerId, options = {}) {
        super();
        this.config = { ...CONFIG, ...options };
        this.containerId = containerId;
        this.container = null;

        // Event storage
        this.events = [];
        this.isUserScrolledUp = false;

        // Batching
        this.batch = {
            reproductions: 0,
            gamesPlayed: 0,
            groupFormations: 0,
            interactions: 0,
            otherEvents: [],
        };
        this.lastBatchTime = Date.now();

        // Initialize if DOM is ready
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', () => this.init());
        } else {
            this.init();
        }
    }

    /**
     * Initialize the event log
     * @private
     */
    init() {
        this.container = document.getElementById(this.containerId);
        if (!this.container) {
            console.warn(`[EventLog] Container #${this.containerId} not found`);
            return;
        }

        this.setupScrollListener();
        this.setupScrollIndicator();
        this.render();
    }

    /**
     * Setup scroll listener for smart auto-scroll
     * @private
     */
    setupScrollListener() {
        if (!this.container) return;

        this.container.addEventListener('scroll', () => {
            const wasScrolledUp = this.isUserScrolledUp;
            this.isUserScrolledUp = !this.isScrolledToBottom();

            if (wasScrolledUp !== this.isUserScrolledUp) {
                this.updateScrollIndicator();
            }
        });
    }

    /**
     * Setup scroll indicator click handler
     * @private
     */
    setupScrollIndicator() {
        const indicator = document.getElementById(`${this.containerId}-scroll-indicator`);
        if (indicator) {
            indicator.addEventListener('click', () => this.scrollToBottom());
        }
    }

    /**
     * Check if scrolled to bottom
     * @returns {boolean}
     * @private
     */
    isScrolledToBottom() {
        if (!this.container) return true;

        const threshold = 10; // pixels from bottom
        return (
            this.container.scrollHeight -
                this.container.scrollTop -
                this.container.clientHeight <=
            threshold
        );
    }

    /**
     * Update scroll indicator visibility
     * @private
     */
    updateScrollIndicator() {
        const indicator = document.getElementById(`${this.containerId}-scroll-indicator`);
        if (!indicator) return;

        if (this.isUserScrolledUp) {
            indicator.classList.add('visible');
        } else {
            indicator.classList.remove('visible');
        }
    }

    /**
     * Scroll to bottom
     */
    scrollToBottom() {
        if (!this.container) return;

        this.container.scrollTop = this.container.scrollHeight;
        this.isUserScrolledUp = false;
        this.updateScrollIndicator();
    }

    /**
     * Log an event
     * @param {string} message - Event message
     * @param {string} [type='info'] - Event type: 'info', 'success', 'warning', 'danger', 'event'
     * @param {number} [agentId=null] - Optional agent ID to highlight
     * @param {string} [category='other'] - Category for batching: 'reproduction', 'game', 'group', 'interaction', 'other'
     */
    log(message, type = 'info', agentId = null, category = 'other') {
        // Handle batchable categories
        if (category === 'reproduction') {
            this.batch.reproductions++;
            return;
        }
        if (category === 'game') {
            this.batch.gamesPlayed++;
            return;
        }
        if (category === 'group') {
            this.batch.groupFormations++;
            return;
        }
        if (category === 'interaction') {
            this.batch.interactions++;
            return;
        }

        // Process batch if interval has passed
        const now = Date.now();
        if (now - this.lastBatchTime >= this.config.batchInterval) {
            this.processBatch();
            this.lastBatchTime = now;
        }

        const event = {
            timestamp: this.formatTimestamp(),
            message,
            type,
            agentId,
        };

        this.events.push(event);

        // Keep only last N events
        if (this.events.length > this.config.maxEvents) {
            this.events.shift();
        }

        this.render();

        // Dispatch event for external listeners
        this.dispatchEvent(new CustomEvent('logged', { detail: event }));
    }

    /**
     * Process batched events
     * @private
     */
    processBatch() {
        const batchEvents = [];

        // Add reproduction summary
        if (this.batch.reproductions > 0) {
            const msg =
                this.batch.reproductions === 1
                    ? '1 agent reproduced'
                    : `${this.batch.reproductions} agents reproduced`;
            batchEvents.push({ timestamp: this.formatTimestamp(), message: msg, type: 'success', agentId: null });
            this.batch.reproductions = 0;
        }

        // Add games played summary
        if (this.batch.gamesPlayed > 0) {
            const msg = this.batch.gamesPlayed === 1 ? '1 game played' : `${this.batch.gamesPlayed} games played`;
            batchEvents.push({ timestamp: this.formatTimestamp(), message: msg, type: 'info', agentId: null });
            this.batch.gamesPlayed = 0;
        }

        // Add group formations summary
        if (this.batch.groupFormations > 0) {
            const msg =
                this.batch.groupFormations === 1
                    ? '1 group formed'
                    : `${this.batch.groupFormations} groups formed`;
            batchEvents.push({ timestamp: this.formatTimestamp(), message: msg, type: 'event', agentId: null });
            this.batch.groupFormations = 0;
        }

        // Add interactions summary
        if (this.batch.interactions > 0) {
            const msg =
                this.batch.interactions === 1
                    ? '1 significant interaction'
                    : `${this.batch.interactions} significant interactions`;
            batchEvents.push({ timestamp: this.formatTimestamp(), message: msg, type: 'info', agentId: null });
            this.batch.interactions = 0;
        }

        // Add all batched events
        if (batchEvents.length > 0) {
            this.events.push(...batchEvents);

            // Keep only last N events
            while (this.events.length > this.config.maxEvents) {
                this.events.shift();
            }

            this.render();
        }
    }

    /**
     * Force process any pending batch
     */
    flushBatch() {
        this.processBatch();
    }

    /**
     * Format timestamp for display
     * @returns {string} Formatted timestamp
     * @private
     */
    formatTimestamp() {
        return new Date().toLocaleTimeString('en-US', {
            hour12: false,
            hour: '2-digit',
            minute: '2-digit',
            second: '2-digit',
        });
    }

    /**
     * Escape HTML to prevent XSS
     * @param {string} text - Text to escape
     * @returns {string} Escaped text
     * @private
     */
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    /**
     * Render the event log
     * @private
     */
    render() {
        if (!this.container) return;

        // Check if we should auto-scroll
        const shouldAutoScroll = !this.isUserScrolledUp || this.isScrolledToBottom();

        // Render empty state
        if (this.events.length === 0) {
            this.container.innerHTML = `
                <div class="activity-log-welcome">Welcome to Agent Society Simulator v1.0</div>
                <div class="activity-log-welcome">Select an experiment and click RUN to begin...</div>
            `;
            this.updateScrollIndicator();
            return;
        }

        // Render events
        this.container.innerHTML = this.events
            .map((event) => {
                const agentSpan =
                    event.agentId !== null
                        ? `<span class="activity-log-agent">[#${event.agentId}]</span>`
                        : '';
                return `
                    <div class="activity-log-entry ${event.type}">
                        <span class="activity-log-timestamp">[${event.timestamp}]</span>
                        ${agentSpan}
                        <span class="activity-log-action">${this.escapeHtml(event.message)}</span>
                    </div>
                `;
            })
            .join('');

        // Auto-scroll to bottom only if user was already at bottom
        if (shouldAutoScroll) {
            this.scrollToBottom();
        }

        this.updateScrollIndicator();
    }

    /**
     * Get all events
     * @returns {Array} All logged events
     */
    getEvents() {
        return [...this.events];
    }

    /**
     * Clear all events
     */
    clear() {
        this.events = [];
        this.batch = {
            reproductions: 0,
            gamesPlayed: 0,
            groupFormations: 0,
            interactions: 0,
            otherEvents: [],
        };
        this.lastBatchTime = Date.now();
        this.render();

        this.dispatchEvent(new CustomEvent('cleared'));
    }

    /**
     * Export events to JSON
     * @returns {string} JSON string
     */
    exportToJSON() {
        return JSON.stringify(this.events, null, 2);
    }

    /**
     * Get events by type
     * @param {string} type - Event type
     * @returns {Array} Filtered events
     */
    getEventsByType(type) {
        return this.events.filter((e) => e.type === type);
    }

    /**
     * Get events for a specific agent
     * @param {number} agentId - Agent ID
     * @returns {Array} Filtered events
     */
    getEventsForAgent(agentId) {
        return this.events.filter((e) => e.agentId === agentId);
    }

    /**
     * Search events by message content
     * @param {string} query - Search query
     * @returns {Array} Matching events
     */
    search(query) {
        const lowerQuery = query.toLowerCase();
        return this.events.filter((e) => e.message.toLowerCase().includes(lowerQuery));
    }

    /**
     * Destroy the event log and cleanup
     */
    destroy() {
        this.events = [];
        this.batch = {
            reproductions: 0,
            gamesPlayed: 0,
            groupFormations: 0,
            interactions: 0,
            otherEvents: [],
        };

        if (this.container) {
            this.container.innerHTML = '';
        }

        this.dispatchEvent(new CustomEvent('destroyed'));
    }
}

export default EventLog;
