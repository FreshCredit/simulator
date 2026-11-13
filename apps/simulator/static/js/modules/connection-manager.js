/**
 * Connection Manager Module
 *
 * Handles SSE connection lifecycle, reconnection with exponential backoff,
 * and frame flow monitoring for the Agent Society Simulator.
 *
 * @module ConnectionManager
 * @author FreshCredit Engineering
 * @version 1.0.0
 */

/**
 * Configuration constants for connection management
 */
const DEFAULT_CONFIG = {
    maxReconnectAttempts: 50,
    reconnectDelays: [1000, 2000, 5000, 10000, 30000],
    frameTimeoutThreshold: 5000,
    frameFlowCheckInterval: 2000,
    connectionTimeout: 10000,
};

/**
 * ConnectionManager - Manages SSE connection to the simulation backend
 *
 * Emits events:
 * - 'connecting': When connection attempt starts
 * - 'connected': When connection is established
 * - 'disconnected': When connection is lost
 * - 'frame': When a frame is received (detail: frame data)
 * - 'error': When an error occurs (detail: error)
 * - 'stalled': When frame flow is detected as stalled
 * - 'failed': When max reconnection attempts reached
 *
 * @example
 * const conn = new ConnectionManager('http://localhost:8082');
 * conn.addEventListener('frame', (e) => console.log('Frame:', e.detail));
 * conn.addEventListener('connected', () => console.log('Connected!'));
 * conn.connect();
 */
export class ConnectionManager extends EventTarget {
    /**
     * Create a new ConnectionManager
     * @param {string} apiUrl - Base URL for the simulation API
     * @param {Object} options - Configuration options
     * @param {number} [options.maxReconnectAttempts=50] - Max reconnection attempts
     * @param {number[]} [options.reconnectDelays=[1000,2000,5000,10000,30000]] - Delay between reconnects
     * @param {number} [options.frameTimeoutThreshold=5000] - Time before considering stalled
     * @param {number} [options.frameFlowCheckInterval=2000] - How often to check frame flow
     * @param {number} [options.connectionTimeout=10000] - Connection timeout in ms
     */
    constructor(apiUrl, options = {}) {
        super();
        this.apiUrl = apiUrl;
        this.config = { ...DEFAULT_CONFIG, ...options };

        this.eventSource = null;
        this.reconnectAttempts = 0;
        this.frameFlowMonitor = null;
        this.lastFrameTime = 0;
        this.connectionTimeoutId = null;
    }

    /**
     * Get current connection state
     * @returns {string} 'connecting' | 'connected' | 'disconnected' | 'failed'
     */
    getState() {
        if (!this.eventSource) return 'disconnected';
        if (this.reconnectAttempts >= this.config.maxReconnectAttempts) return 'failed';

        switch (this.eventSource.readyState) {
            case EventSource.CONNECTING:
                return 'connecting';
            case EventSource.OPEN:
                return 'connected';
            case EventSource.CLOSED:
                return 'disconnected';
            default:
                return 'disconnected';
        }
    }

    /**
     * Check if currently connected
     * @returns {boolean}
     */
    isConnected() {
        return this.eventSource?.readyState === EventSource.OPEN;
    }

    /**
     * Connect to the simulation SSE stream
     */
    connect() {
        if (this.eventSource) {
            this.disconnect();
        }

        const streamUrl = `${this.apiUrl}/api/live`;

        this.dispatchEvent(new CustomEvent('connecting', { detail: { url: streamUrl } }));

        try {
            this.eventSource = new EventSource(streamUrl);
        } catch (err) {
            this.dispatchEvent(new CustomEvent('error', { detail: err }));
            this.scheduleReconnect();
            return;
        }

        this.setupEventHandlers();
        this.startConnectionTimeout();
    }

    /**
     * Setup SSE event handlers
     * @private
     */
    setupEventHandlers() {
        this.eventSource.onopen = () => {
            this.clearConnectionTimeout();
            this.reconnectAttempts = 0;
            this.lastFrameTime = performance.now();
            this.startFrameFlowMonitor();
            this.dispatchEvent(new CustomEvent('connected'));
        };

        this.eventSource.onmessage = (event) => {
            this.lastFrameTime = performance.now();

            try {
                const frame = JSON.parse(event.data);
                this.dispatchEvent(new CustomEvent('frame', { detail: frame }));
            } catch (err) {
                this.dispatchEvent(
                    new CustomEvent('error', {
                        detail: new Error(`Failed to parse frame: ${err.message}`),
                    })
                );
            }
        };

        this.eventSource.onerror = () => {
            this.clearConnectionTimeout();
            this.stopFrameFlowMonitor();
            this.dispatchEvent(new CustomEvent('disconnected'));

            // Close and schedule reconnect
            if (this.eventSource) {
                this.eventSource.close();
                this.eventSource = null;
            }
            this.scheduleReconnect();
        };
    }

    /**
     * Start connection timeout timer
     * @private
     */
    startConnectionTimeout() {
        this.connectionTimeoutId = setTimeout(() => {
            if (this.eventSource?.readyState === EventSource.CONNECTING) {
                this.dispatchEvent(new CustomEvent('error', { detail: new Error('Connection timeout') }));
                this.eventSource.close();
                this.scheduleReconnect();
            }
        }, this.config.connectionTimeout);
    }

    /**
     * Clear connection timeout
     * @private
     */
    clearConnectionTimeout() {
        if (this.connectionTimeoutId) {
            clearTimeout(this.connectionTimeoutId);
            this.connectionTimeoutId = null;
        }
    }

    /**
     * Schedule reconnection with exponential backoff
     * @private
     */
    scheduleReconnect() {
        if (this.reconnectAttempts >= this.config.maxReconnectAttempts) {
            this.dispatchEvent(
                new CustomEvent('failed', {
                    detail: { attempts: this.reconnectAttempts },
                })
            );
            return;
        }

        const delay = this.config.reconnectDelays[
            Math.min(this.reconnectAttempts, this.config.reconnectDelays.length - 1)
        ];

        this.reconnectAttempts++;

        this.dispatchEvent(
            new CustomEvent('reconnecting', {
                detail: { attempt: this.reconnectAttempts, delay },
            })
        );

        setTimeout(() => this.connect(), delay);
    }

    /**
     * Start monitoring frame flow to detect stalled connections
     * @private
     */
    startFrameFlowMonitor() {
        this.stopFrameFlowMonitor();

        this.frameFlowMonitor = setInterval(() => {
            if (!this.isConnected()) return;

            const timeSinceLastFrame = performance.now() - this.lastFrameTime;

            if (timeSinceLastFrame > this.config.frameTimeoutThreshold) {
                this.dispatchEvent(
                    new CustomEvent('stalled', {
                        detail: { timeSinceLastFrame },
                    })
                );

                // Trigger reconnect
                this.stopFrameFlowMonitor();
                this.eventSource?.close();
                this.scheduleReconnect();
            }
        }, this.config.frameFlowCheckInterval);
    }

    /**
     * Stop frame flow monitoring
     * @private
     */
    stopFrameFlowMonitor() {
        if (this.frameFlowMonitor) {
            clearInterval(this.frameFlowMonitor);
            this.frameFlowMonitor = null;
        }
    }

    /**
     * Disconnect from the SSE stream
     */
    disconnect() {
        this.clearConnectionTimeout();
        this.stopFrameFlowMonitor();

        if (this.eventSource) {
            this.eventSource.close();
            this.eventSource = null;
        }

        this.reconnectAttempts = 0;
        this.dispatchEvent(new CustomEvent('disconnected'));
    }

    /**
     * Destroy the connection manager and cleanup resources
     */
    destroy() {
        this.disconnect();
        // Remove all event listeners
        this.removeEventListener('frame', null);
        this.removeEventListener('connected', null);
        this.removeEventListener('disconnected', null);
        this.removeEventListener('error', null);
    }
}

export default ConnectionManager;
