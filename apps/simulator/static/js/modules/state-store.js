/**
 * State Store Module
 *
 * Centralized state management for the Agent Society Simulator.
 * Provides observable state with subscriptions and event-driven updates.
 *
 * @module StateStore
 * @author FreshCredit Engineering
 * @version 1.0.0
 */

/**
 * Default initial state for the simulator
 */
export const DEFAULT_STATE = {
    simulation: {
        status: 'stopped', // 'stopped' | 'running' | 'paused' | 'disconnected' | 'failed'
        currentTick: 0,
        speed: 1.0,
        isLoading: false,
    },
    frame: {
        current: null,
        previous: null,
    },
    agents: {
        all: [],
        selected: null,
        count: 0,
        aliveCount: 0,
    },
    layers: {
        agents: true,
        terrain: false,
        weather: false,
        resources: false,
        harvesting: false,
        docas: false,
        energy: false,
        groups: false,
        identity: false,
        network: false,
        relative: false,
        reputation: false,
    },
    layerOpacity: {
        agents: 1.0,
        terrain: 1.0,
        weather: 0.5,
        resources: 1.0,
        harvesting: 1.0,
        docas: 1.0,
        energy: 1.0,
        groups: 1.0,
        identity: 1.0,
        network: 1.0,
        relative: 1.0,
        reputation: 1.0,
    },
    filters: {
        trait: 'all', // 'all' | 'd' | 'o' | 'c' | 'a' | 's'
        status: 'alive', // 'alive' | 'all'
        relativeDistanceThreshold: 10,
    },
    ui: {
        connectionStatus: 'disconnected',
        isInspectorOpen: false,
        isGameOver: false,
        showExportButton: false,
    },
    stats: {
        totalAgents: 0,
        aliveAgents: 0,
        verifiedAgents: 0,
        groupCount: 0,
    },
};

/**
 * StateStore - Centralized observable state management
 *
 * Provides:
 * - Hierarchical state with dot notation paths
 * - Subscriptions for state changes
 * - Event-driven updates
 * - Immutable state updates
 *
 * @example
 * const store = new StateStore(DEFAULT_STATE);
 *
 * // Subscribe to changes
 * store.subscribe('layers.agents', (value, oldValue) => {
 *   console.log('Agents layer:', value);
 * });
 *
 * // Update state
 * store.set('layers.agents', false);
 *
 * // Get state
 * const isVisible = store.get('layers.agents');
 */
export class StateStore extends EventTarget {
    /**
     * Create a new StateStore
     * @param {Object} initialState - Initial state object
     */
    constructor(initialState = {}) {
        super();
        this.state = this.deepClone({ ...DEFAULT_STATE, ...initialState });
        this.subscribers = new Map();
        this.batchDepth = 0;
        this.pendingNotifications = new Set();
    }

    /**
     * Get state value at path
     * @param {string} [path] - Dot notation path (e.g., 'layers.agents')
     * @returns {*} Value at path, or entire state if no path provided
     *
     * @example
     * store.get(); // Returns entire state
     * store.get('layers.agents'); // Returns agents layer visibility
     * store.get('simulation.currentTick'); // Returns current tick
     */
    get(path) {
        if (!path) return this.deepClone(this.state);

        const keys = path.split('.');
        let value = this.state;

        for (const key of keys) {
            if (value === null || value === undefined) return undefined;
            value = value[key];
        }

        return this.deepClone(value);
    }

    /**
     * Set state value at path
     * @param {string} path - Dot notation path
     * @param {*} value - Value to set
     * @returns {StateStore} this (for chaining)
     *
     * @example
     * store.set('layers.agents', false);
     * store.set('simulation.currentTick', 42);
     */
    set(path, value) {
        const keys = path.split('.');
        const lastKey = keys.pop();

        // Navigate to parent object
        let target = this.state;
        for (const key of keys) {
            if (!(key in target)) target[key] = {};
            target = target[key];
        }

        const oldValue = this.deepClone(target[lastKey]);
        target[lastKey] = this.deepClone(value);

        // Queue notification if batching, otherwise notify immediately
        if (this.batchDepth > 0) {
            this.pendingNotifications.add(path);
        } else {
            this.notify(path, value, oldValue);
        }

        return this;
    }

    /**
     * Batch multiple state updates into single notification cycle
     * @param {Function} fn - Function to execute in batch context
     *
     * @example
     * store.batch(() => {
     *   store.set('layers.agents', false);
     *   store.set('layers.terrain', true);
     *   store.set('simulation.speed', 2.0);
     * });
     * // All subscribers notified once after batch completes
     */
    batch(fn) {
        this.batchDepth++;
        try {
            fn();
        } finally {
            this.batchDepth--;
            if (this.batchDepth === 0) {
                this.flushNotifications();
            }
        }
    }

    /**
     * Flush pending notifications after batch
     * @private
     */
    flushNotifications() {
        const paths = Array.from(this.pendingNotifications);
        this.pendingNotifications.clear();

        for (const path of paths) {
            const value = this.get(path);
            this.notifySubscribers(path, value);
        }
    }

    /**
     * Subscribe to state changes at path
     * @param {string} path - Dot notation path to subscribe to
     * @param {Function} callback - Called when value changes: (newValue, oldValue, path) => void
     * @returns {Function} Unsubscribe function
     *
     * @example
     * const unsubscribe = store.subscribe('layers.agents', (visible) => {
     *   render();
     * });
     *
     * // Later...
     * unsubscribe();
     */
    subscribe(path, callback) {
        if (!this.subscribers.has(path)) {
            this.subscribers.set(path, new Set());
        }

        const callbacks = this.subscribers.get(path);
        callbacks.add(callback);

        // Return unsubscribe function
        return () => {
            callbacks.delete(callback);
            if (callbacks.size === 0) {
                this.subscribers.delete(path);
            }
        };
    }

    /**
     * Notify subscribers of state change
     * @param {string} path - Path that changed
     * @param {*} newValue - New value
     * @param {*} oldValue - Old value
     * @private
     */
    notify(path, newValue, oldValue) {
        // Notify exact path subscribers
        this.notifySubscribers(path, newValue, oldValue);

        // Notify parent path subscribers (bubbling)
        const parts = path.split('.');
        while (parts.pop() && parts.length > 0) {
            const parentPath = parts.join('.');
            this.notifySubscribers(parentPath, this.get(parentPath), null);
        }

        // Dispatch global change event
        this.dispatchEvent(
            new CustomEvent('change', {
                detail: { path, newValue: this.deepClone(newValue), oldValue: this.deepClone(oldValue) },
            })
        );
    }

    /**
     * Notify subscribers at specific path
     * @param {string} path - Path to notify
     * @param {*} value - Current value
     * @param {*} oldValue - Previous value
     * @private
     */
    notifySubscribers(path, value, oldValue) {
        const callbacks = this.subscribers.get(path);
        if (callbacks) {
            for (const callback of callbacks) {
                try {
                    callback(this.deepClone(value), this.deepClone(oldValue), path);
                } catch (err) {
                    console.error(`Error in subscriber for ${path}:`, err);
                }
            }
        }
    }

    /**
     * Toggle a boolean state value
     * @param {string} path - Path to boolean value
     * @returns {boolean} New value
     *
     * @example
     * store.toggle('layers.agents'); // Toggles visibility
     */
    toggle(path) {
        const current = this.get(path);
        if (typeof current !== 'boolean') {
            throw new Error(`Cannot toggle non-boolean value at ${path}`);
        }
        this.set(path, !current);
        return !current;
    }

    /**
     * Reset state to initial values
     * @param {string} [path] - Specific path to reset, or entire state if omitted
     */
    reset(path) {
        if (path) {
            const keys = path.split('.');
            let defaultValue = DEFAULT_STATE;
            for (const key of keys) {
                defaultValue = defaultValue?.[key];
            }
            this.set(path, this.deepClone(defaultValue));
        } else {
            this.state = this.deepClone(DEFAULT_STATE);
            this.dispatchEvent(new CustomEvent('reset', { detail: { state: this.get() } }));
        }
    }

    /**
     * Deep clone an object to ensure immutability
     * @param {*} obj - Object to clone
     * @returns {*} Deep cloned object
     * @private
     */
    deepClone(obj) {
        if (obj === null || typeof obj !== 'object') return obj;
        if (obj instanceof Date) return new Date(obj.getTime());
        if (obj instanceof Array) return obj.map((item) => this.deepClone(item));
        if (obj instanceof Map) return new Map(Array.from(obj.entries()));
        if (obj instanceof Set) return new Set(Array.from(obj.values()));

        const cloned = {};
        for (const key in obj) {
            if (obj.hasOwnProperty(key)) {
                cloned[key] = this.deepClone(obj[key]);
            }
        }
        return cloned;
    }

    /**
     * Get current state as plain object (for debugging)
     * @returns {Object} Current state
     */
    toJSON() {
        return this.get();
    }
}

/**
 * Singleton instance of StateStore for application-wide state
 */
export const store = new StateStore();

export default StateStore;
