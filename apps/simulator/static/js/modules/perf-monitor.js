/**
 * Performance Monitor Module
 *
 * Real-time performance monitoring and metrics collection for the simulator.
 * Tracks FPS, frame times, memory usage, and provides performance insights.
 *
 * @module PerformanceMonitor
 * @author FreshCredit Engineering
 * @version 1.0.0
 */

/**
 * Configuration for performance monitoring
 */
const CONFIG = {
    // Sample history size
    maxSamples: 60,
    // Update interval in ms
    updateInterval: 1000,
    // FPS thresholds
    fps: {
        excellent: 55,
        good: 45,
        poor: 30,
    },
    // Frame time thresholds (ms)
    frameTime: {
        excellent: 16.67, // 60fps
        good: 22.22, // 45fps
        poor: 33.33, // 30fps
    },
};

/**
 * PerformanceMonitor - Real-time performance tracking
 *
 * Tracks:
 * - FPS (frames per second)
 * - Frame times (min, max, avg)
 * - Memory usage (if available)
 * - Render loop performance
 *
 * @example
 * const perf = new PerformanceMonitor();
 * perf.start();
 *
 * // In render loop
 * perf.recordFrame();
 *
 * // Get stats
 * console.log(perf.getFPS()); // Current FPS
 * console.log(perf.getStats()); // Full stats
 */
export class PerformanceMonitor extends EventTarget {
    /**
     * Create a new PerformanceMonitor
     * @param {Object} options - Configuration options
     */
    constructor(options = {}) {
        super();
        this.config = { ...CONFIG, ...options };

        // Frame tracking
        this.frames = [];
        this.lastFrameTime = 0;
        this.frameCount = 0;

        // Timing
        this.startTime = 0;
        this.lastUpdateTime = 0;

        // Running state
        this.isRunning = false;
        this.updateIntervalId = null;

        // Memory tracking
        this.memorySamples = [];
    }

    /**
     * Start performance monitoring
     */
    start() {
        if (this.isRunning) return;

        this.isRunning = true;
        this.startTime = performance.now();
        this.lastFrameTime = this.startTime;
        this.lastUpdateTime = this.startTime;

        // Start update loop
        this.updateIntervalId = setInterval(() => {
            this.update();
        }, this.config.updateInterval);

        this.dispatchEvent(new CustomEvent('started'));
    }

    /**
     * Stop performance monitoring
     */
    stop() {
        if (!this.isRunning) return;

        this.isRunning = false;

        if (this.updateIntervalId) {
            clearInterval(this.updateIntervalId);
            this.updateIntervalId = null;
        }

        this.dispatchEvent(new CustomEvent('stopped'));
    }

    /**
     * Record a frame render
     * Call this at the end of your render loop
     */
    recordFrame() {
        if (!this.isRunning) return;

        const now = performance.now();
        const frameTime = now - this.lastFrameTime;

        this.frames.push({
            time: now,
            duration: frameTime,
        });

        // Keep only recent samples
        if (this.frames.length > this.config.maxSamples) {
            this.frames.shift();
        }

        this.lastFrameTime = now;
        this.frameCount++;

        // Check for performance issues
        this.checkPerformance(frameTime);
    }

    /**
     * Check for performance issues
     * @param {number} frameTime - Frame duration in ms
     * @private
     */
    checkPerformance(frameTime) {
        // Warn on very slow frames
        if (frameTime > 50) {
            // > 50ms = < 20fps
            this.dispatchEvent(
                new CustomEvent('slowFrame', {
                    detail: { frameTime, threshold: 50 },
                })
            );
        }

        // Warn on consistent slowdown
        if (this.frames.length >= 10) {
            const recent = this.frames.slice(-10);
            const avgTime = recent.reduce((sum, f) => sum + f.duration, 0) / recent.length;

            if (avgTime > 33.33) {
                // < 30fps average
                this.dispatchEvent(
                    new CustomEvent('poorPerformance', {
                        detail: { avgTime, fps: 1000 / avgTime },
                    })
                );
            }
        }
    }

    /**
     * Update statistics (called on interval)
     * @private
     */
    update() {
        const now = performance.now();
        const elapsed = now - this.lastUpdateTime;

        // Record memory if available
        if (performance.memory) {
            this.memorySamples.push({
                time: now,
                used: performance.memory.usedJSHeapSize,
                total: performance.memory.totalJSHeapSize,
                limit: performance.memory.jsHeapSizeLimit,
            });

            if (this.memorySamples.length > this.config.maxSamples) {
                this.memorySamples.shift();
            }
        }

        // Emit stats update
        this.dispatchEvent(
            new CustomEvent('update', {
                detail: this.getStats(),
            })
        );

        this.lastUpdateTime = now;
    }

    /**
     * Get current FPS
     * @returns {number} Current FPS
     */
    getFPS() {
        if (this.frames.length < 2) return 0;

        const recent = this.frames.slice(-10);
        const totalTime = recent[recent.length - 1].time - recent[0].time;
        const frameCount = recent.length - 1;

        return totalTime > 0 ? (frameCount / totalTime) * 1000 : 0;
    }

    /**
     * Get frame time statistics
     * @returns {Object} Frame time stats
     */
    getFrameTimeStats() {
        if (this.frames.length === 0) {
            return { min: 0, max: 0, avg: 0, median: 0 };
        }

        const durations = this.frames.map((f) => f.duration);
        durations.sort((a, b) => a - b);

        const min = durations[0];
        const max = durations[durations.length - 1];
        const avg = durations.reduce((sum, d) => sum + d, 0) / durations.length;
        const median = durations[Math.floor(durations.length / 2)];

        return { min, max, avg, median };
    }

    /**
     * Get memory statistics
     * @returns {Object|null} Memory stats or null if not available
     */
    getMemoryStats() {
        if (this.memorySamples.length === 0) return null;

        const current = this.memorySamples[this.memorySamples.length - 1];
        const prev = this.memorySamples[0];

        const usedMB = (current.used / 1024 / 1024).toFixed(2);
        const totalMB = (current.total / 1024 / 1024).toFixed(2);
        const limitMB = (current.limit / 1024 / 1024).toFixed(2);
        const percent = ((current.used / current.limit) * 100).toFixed(1);

        // Calculate growth rate
        let growthRate = 0;
        if (this.memorySamples.length >= 2) {
            const timeDiff = (current.time - prev.time) / 1000; // seconds
            const memDiff = (current.used - prev.used) / 1024 / 1024; // MB
            growthRate = timeDiff > 0 ? memDiff / timeDiff : 0; // MB/s
        }

        return {
            used: current.used,
            total: current.total,
            limit: current.limit,
            usedMB,
            totalMB,
            limitMB,
            percent,
            growthRate: growthRate.toFixed(2),
            samples: this.memorySamples.length,
        };
    }

    /**
     * Get performance rating
     * @returns {string} 'excellent' | 'good' | 'poor' | 'critical'
     */
    getRating() {
        const fps = this.getFPS();
        const { avg } = this.getFrameTimeStats();

        if (fps >= this.config.fps.excellent && avg <= this.config.frameTime.excellent) {
            return 'excellent';
        }
        if (fps >= this.config.fps.good && avg <= this.config.frameTime.good) {
            return 'good';
        }
        if (fps >= this.config.fps.poor && avg <= this.config.frameTime.poor) {
            return 'poor';
        }
        return 'critical';
    }

    /**
     * Get full statistics
     * @returns {Object} Complete performance stats
     */
    getStats() {
        return {
            fps: this.getFPS(),
            frameTime: this.getFrameTimeStats(),
            memory: this.getMemoryStats(),
            rating: this.getRating(),
            frameCount: this.frameCount,
            uptime: this.isRunning ? (performance.now() - this.startTime) / 1000 : 0,
            samples: this.frames.length,
        };
    }

    /**
     * Get performance report as formatted string
     * @returns {string}
     */
    getReport() {
        const stats = this.getStats();
        const lines = [
            'Performance Report',
            '==================',
            `FPS: ${stats.fps.toFixed(1)} (${stats.rating})`,
            `Frame Time: avg=${stats.frameTime.avg.toFixed(2)}ms, min=${stats.frameTime.min.toFixed(2)}ms, max=${stats.frameTime.max.toFixed(2)}ms`,
            `Frames: ${stats.frameCount}`,
            `Uptime: ${stats.uptime.toFixed(1)}s`,
        ];

        if (stats.memory) {
            lines.push(`Memory: ${stats.memory.usedMB}MB / ${stats.memory.limitMB}MB (${stats.memory.percent}%)`);
            if (parseFloat(stats.memory.growthRate) !== 0) {
                lines.push(`Growth: ${stats.memory.growthRate}MB/s`);
            }
        }

        return lines.join('\n');
    }

    /**
     * Mark a specific operation start
     * @param {string} name - Operation name
     * @returns {number} Mark ID
     */
    markStart(name) {
        const mark = performance.mark(`${name}-start`);
        return mark?.startTime ?? performance.now();
    }

    /**
     * Mark a specific operation end
     * @param {string} name - Operation name
     * @returns {number} Duration in ms
     */
    markEnd(name) {
        const endTime = performance.now();
        performance.mark(`${name}-end`);

        try {
            performance.measure(name, `${name}-start`, `${name}-end`);
        } catch (e) {
            // Measure might fail if marks don't exist
        }

        // Get duration
        const entries = performance.getEntriesByName(name, 'measure');
        if (entries.length > 0) {
            return entries[entries.length - 1].duration;
        }

        return 0;
    }

    /**
     * Measure a function execution time
     * @param {string} name - Measurement name
     * @param {Function} fn - Function to measure
     * @param {*} context - Context for function
     * @returns {*} Function result
     */
    measure(name, fn, context = null) {
        const start = this.markStart(name);
        let result;

        try {
            result = fn.call(context);
        } finally {
            const duration = this.markEnd(name);

            // Log slow operations
            if (duration > 16.67) {
                console.warn(`[PerformanceMonitor] Slow operation: ${name} took ${duration.toFixed(2)}ms`);
            }
        }

        return result;
    }

    /**
     * Measure an async function execution time
     * @param {string} name - Measurement name
     * @param {Function} fn - Async function to measure
     * @param {*} context - Context for function
     * @returns {Promise<*>} Function result
     */
    async measureAsync(name, fn, context = null) {
        const start = this.markStart(name);
        let result;

        try {
            result = await fn.call(context);
        } finally {
            this.markEnd(name);
        }

        return result;
    }

    /**
     * Clear all samples and reset
     */
    reset() {
        this.frames = [];
        this.memorySamples = [];
        this.frameCount = 0;
        this.startTime = performance.now();
        this.lastFrameTime = this.startTime;

        // Clear performance entries
        if (performance.clearMarks) {
            performance.clearMarks();
        }
        if (performance.clearMeasures) {
            performance.clearMeasures();
        }
    }

    /**
     * Destroy the monitor and cleanup
     */
    destroy() {
        this.stop();
        this.reset();
    }
}

/**
 * Singleton instance for application-wide use
 */
export const perfMonitor = new PerformanceMonitor();

export default PerformanceMonitor;
