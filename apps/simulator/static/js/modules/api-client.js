/**
 * API Client Module
 *
 * Centralized HTTP/SSE API communication for the Agent Society Simulator.
 * Provides type-safe methods for all simulation backend endpoints.
 *
 * @module APIClient
 * @author FreshCredit Engineering
 * @version 1.0.0
 */

/**
 * Default configuration for API client
 */
const DEFAULT_CONFIG = {
    timeout: 5000,
    retries: 3,
    retryDelay: 1000,
};

/**
 * APIError - Custom error class for API failures
 */
export class APIError extends Error {
    /**
     * Create an APIError
     * @param {string} message - Error message
     * @param {number} status - HTTP status code
     * @param {*} response - Response data
     * @param {string} endpoint - API endpoint that failed
     */
    constructor(message, status, response, endpoint) {
        super(message);
        this.name = 'APIError';
        this.status = status;
        this.response = response;
        this.endpoint = endpoint;
    }
}

/**
 * APIClient - HTTP client for simulation API
 *
 * Provides methods for all simulation control endpoints including:
 * - Simulation control (pause, resume, stop, speed)
 * - Neural data retrieval
 * - Experiment management
 * - Health checks
 *
 * @example
 * const api = new APIClient('http://localhost:8082');
 *
 * // Control simulation
 * await api.pause();
 * await api.setSpeed(2.0);
 *
 * // Get data
 * const status = await api.getStatus();
 * const neuralData = await api.getNeuralData(agentId);
 */
export class APIClient {
    /**
     * Create a new APIClient
     * @param {string} baseUrl - Base URL for the API
     * @param {Object} options - Configuration options
     * @param {number} [options.timeout=5000] - Request timeout in ms
     * @param {number} [options.retries=3] - Number of retries for failed requests
     * @param {number} [options.retryDelay=1000] - Delay between retries in ms
     */
    constructor(baseUrl, options = {}) {
        this.baseUrl = baseUrl.replace(/\/$/, ''); // Remove trailing slash
        this.config = { ...DEFAULT_CONFIG, ...options };
    }

    /**
     * Make an HTTP request
     * @param {string} endpoint - API endpoint (without base URL)
     * @param {Object} options - Fetch options
     * @returns {Promise<*>} Response data
     * @throws {APIError} On request failure
     * @private
     */
    async request(endpoint, options = {}) {
        const url = `${this.baseUrl}${endpoint}`;
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), this.config.timeout);

        let lastError;

        for (let attempt = 0; attempt < this.config.retries; attempt++) {
            try {
                const response = await fetch(url, {
                    ...options,
                    signal: controller.signal,
                    headers: {
                        Accept: 'application/json',
                        'Content-Type': 'application/json',
                        ...options.headers,
                    },
                });

                clearTimeout(timeoutId);

                if (!response.ok) {
                    const errorText = await response.text();
                    throw new APIError(
                        `HTTP ${response.status}: ${response.statusText}`,
                        response.status,
                        errorText,
                        endpoint
                    );
                }

                // Return null for 204 No Content
                if (response.status === 204) {
                    return null;
                }

                return await response.json();
            } catch (error) {
                lastError = error;

                // Don't retry on client errors (4xx)
                if (error instanceof APIError && error.status >= 400 && error.status < 500) {
                    throw error;
                }

                // Wait before retry
                if (attempt < this.config.retries - 1) {
                    await this.delay(this.config.retryDelay * (attempt + 1));
                }
            }
        }

        throw lastError;
    }

    /**
     * Delay helper
     * @param {number} ms - Milliseconds to delay
     * @returns {Promise<void>}
     * @private
     */
    delay(ms) {
        return new Promise((resolve) => setTimeout(resolve, ms));
    }

    // ============================================================================
    // Simulation Control Endpoints
    // ============================================================================

    /**
     * Set simulation speed
     * @param {number} factor - Speed factor (0.5 = half speed, 2.0 = double speed)
     * @returns {Promise<Object>} Updated speed settings
     */
    async setSpeed(factor) {
        return this.request('/api/live/speed', {
            method: 'POST',
            body: JSON.stringify({ factor }),
        });
    }

    /**
     * Get current simulation speed
     * @returns {Promise<Object>} Current speed settings
     */
    async getSpeed() {
        return this.request('/api/live/speed');
    }

    /**
     * Pause the simulation
     * @returns {Promise<Object>} Updated status
     */
    async pause() {
        return this.request('/api/live/pause', { method: 'POST' });
    }

    /**
     * Resume the simulation
     * @returns {Promise<Object>} Updated status
     */
    async resume() {
        return this.request('/api/live/resume', { method: 'POST' });
    }

    /**
     * Stop the simulation
     * @returns {Promise<Object>} Final status
     */
    async stop() {
        return this.request('/api/live/stop', { method: 'POST' });
    }

    /**
     * Get current simulation status
     * @returns {Promise<Object>} Status information
     */
    async getStatus() {
        return this.request('/api/live/status');
    }

    // ============================================================================
    // Health & Info Endpoints
    // ============================================================================

    /**
     * Check backend health
     * @returns {Promise<Object>} Health status
     */
    async getHealth() {
        return this.request('/health');
    }

    /**
     * Get lambda statistics
     * @returns {Promise<Object>} Lambda stats
     */
    async getLambdaStats() {
        return this.request('/api/stats/lambda');
    }

    // ============================================================================
    // Neural Data Endpoints
    // ============================================================================

    /**
     * Get neural brain data for an agent
     * @param {number} agentId - Agent ID
     * @returns {Promise<Object>} Neural data including region activations
     */
    async getNeuralData(agentId) {
        return this.request(`/api/neural/agent/${agentId}`);
    }

    /**
     * Get population neural statistics
     * @returns {Promise<Object>} Population neural statistics
     */
    async getNeuralPopulation() {
        return this.request('/api/neural/population');
    }

    // ============================================================================
    // Experiment Endpoints
    // ============================================================================

    /**
     * List all available experiments
     * @returns {Promise<Array>} List of experiments
     */
    async listExperiments() {
        return this.request('/api/experiments');
    }

    /**
     * Get experiment details
     * @param {string} expId - Experiment ID
     * @returns {Promise<Object>} Experiment details
     */
    async getExperiment(expId) {
        return this.request(`/api/experiments/${expId}`);
    }

    /**
     * Get experiment schema
     * @param {string} expId - Experiment ID
     * @returns {Promise<Object>} Experiment schema
     */
    async getExperimentSchema(expId) {
        return this.request(`/api/experiments/${expId}/schema`);
    }

    /**
     * Launch an experiment
     * @param {string} expId - Experiment ID
     * @param {Object} params - Experiment parameters
     * @returns {Promise<Object>} Run information
     */
    async launchExperiment(expId, params) {
        return this.request(`/api/experiments/${expId}/runs`, {
            method: 'POST',
            body: JSON.stringify(params),
        });
    }

    /**
     * Get experiment run status
     * @param {string} runId - Run ID
     * @returns {Promise<Object>} Run status
     */
    async getExperimentRunStatus(runId) {
        return this.request(`/api/experiment-runs/${runId}`);
    }

    // ============================================================================
    // Configuration Endpoints
    // ============================================================================

    /**
     * Get PDE4B configuration
     * @returns {Promise<Object>} PDE4B config
     */
    async getPDE4BConfig() {
        return this.request('/api/config/pde4b');
    }

    /**
     * Update PDE4B configuration
     * @param {Object} config - New configuration
     * @returns {Promise<Object>} Updated config
     */
    async updatePDE4BConfig(config) {
        return this.request('/api/config/pde4b', {
            method: 'POST',
            body: JSON.stringify(config),
        });
    }

    // ============================================================================
    // Validation Endpoints
    // ============================================================================

    /**
     * Run validation suite
     * @returns {Promise<Object>} Validation results
     */
    async runValidation() {
        return this.request('/api/validate', { method: 'POST' });
    }

    // ============================================================================
    // Artifact Endpoints
    // ============================================================================

    /**
     * List available simulation runs
     * @returns {Promise<Array>} List of runs
     */
    async listRuns() {
        return this.request('/api/runs');
    }

    /**
     * Get manifest for a specific run
     * @param {string} runId - Run ID
     * @returns {Promise<Object>} Run manifest
     */
    async getManifest(runId) {
        return this.request(`/api/runs/${runId}/manifest`);
    }

    /**
     * Get replay frame for a specific tick
     * @param {string} runId - Run ID
     * @param {number} tick - Tick number
     * @returns {Promise<Object>} Frame data
     */
    async getReplayFrame(runId, tick) {
        return this.request(`/api/runs/${runId}/replay/${tick}`);
    }
}

export default APIClient;
