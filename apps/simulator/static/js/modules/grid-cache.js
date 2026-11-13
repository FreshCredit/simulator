/**
 * Grid Cache Module
 *
 * Performance optimization using offscreen canvas for grid rendering.
 * Caches static grid lines to avoid redrawing every frame.
 *
 * @module GridCache
 * @author FreshCredit Engineering
 * @version 1.0.0
 */

/**
 * Configuration for grid caching
 */
const CONFIG = {
    // Cache margin for zoom/pan
    margin: 1.5,
    // Maximum cache size (prevent memory issues)
    maxCacheSize: 3,
};

/**
 * GridCache - Offscreen canvas caching for grid rendering
 *
 * Caches grid lines to an offscreen canvas, reducing draw calls
 * from O(n) per frame to O(1) when grid hasn't changed.
 *
 * @example
 * const gridCache = new GridCache();
 * gridCache.build(128, 128, 10, { minorOpacity: 0.15, majorOpacity: 0.4 });
 *
 * // In render loop
 * ctx.drawImage(gridCache.canvas, offsetX, offsetY);
 *
 * // When grid changes (zoom, resize)
 * gridCache.invalidate();
 * gridCache.build(newWidth, newHeight, newScale, config);
 */
export class GridCache {
    constructor() {
        this.canvas = null;
        this.ctx = null;
        this.isValid = false;

        // Cache parameters for invalidation detection
        this.params = {
            width: 0,
            height: 0,
            scale: 0,
            minorOpacity: 0.15,
            majorOpacity: 0.4,
            majorStep: 8,
        };

        // LRU cache for different scales
        this.cachePool = new Map();
    }

    /**
     * Check if cache is valid for given parameters
     * @param {number} width - Grid width
     * @param {number} height - Grid height
     * @param {number} scale - Pixel scale
     * @returns {boolean}
     */
    isCacheValid(width, height, scale) {
        if (!this.isValid || !this.canvas) return false;

        // Check if parameters match (with small tolerance for scale)
        const scaleTolerance = 0.01;
        const scaleMatch = Math.abs(this.params.scale - scale) < scaleTolerance;

        return (
            scaleMatch &&
            this.params.width === width &&
            this.params.height === height
        );
    }

    /**
     * Build the grid cache
     * @param {number} width - Grid width in cells
     * @param {number} height - Grid height in cells
     * @param {number} scale - Scale (pixels per cell)
     * @param {Object} options - Rendering options
     * @param {number} [options.minorOpacity=0.15] - Minor grid line opacity
     * @param {number} [options.majorOpacity=0.4] - Major grid line opacity
     * @param {number} [options.majorStep=8] - Major grid line step
     * @param {string} [options.color='hsl(240, 3.7%, 20%)'] - Grid line color
     * @returns {HTMLCanvasElement} The cached canvas
     */
    build(width, height, scale, options = {}) {
        const config = {
            minorOpacity: options.minorOpacity ?? 0.15,
            majorOpacity: options.majorOpacity ?? 0.4,
            majorStep: options.majorStep ?? 8,
            color: options.color ?? 'hsl(240, 3.7%, 20%)',
        };

        // Check LRU cache first
        const cacheKey = `${width}x${height}@${scale.toFixed(2)}`;
        if (this.cachePool.has(cacheKey)) {
            const cached = this.cachePool.get(cacheKey);
            this.canvas = cached.canvas;
            this.ctx = cached.ctx;
            this.params = cached.params;
            this.isValid = true;

            // Move to end (most recently used)
            this.cachePool.delete(cacheKey);
            this.cachePool.set(cacheKey, cached);

            return this.canvas;
        }

        // Create new offscreen canvas
        const pixelWidth = Math.ceil(width * scale * CONFIG.margin);
        const pixelHeight = Math.ceil(height * scale * CONFIG.margin);

        // Check if we need to evict from cache pool
        if (this.cachePool.size >= CONFIG.maxCacheSize) {
            const firstKey = this.cachePool.keys().next().value;
            this.cachePool.delete(firstKey);
        }

        // Create new canvas
        this.canvas = document.createElement('canvas');
        this.canvas.width = pixelWidth;
        this.canvas.height = pixelHeight;
        this.ctx = this.canvas.getContext('2d');

        // Store parameters
        this.params = {
            width,
            height,
            scale,
            ...config,
        };

        // Render grid to offscreen canvas
        this.renderGrid(width, height, scale, config);

        // Store in cache pool
        this.cachePool.set(cacheKey, {
            canvas: this.canvas,
            ctx: this.ctx,
            params: { ...this.params },
        });

        this.isValid = true;

        return this.canvas;
    }

    /**
     * Render grid lines to offscreen canvas
     * @param {number} width - Grid width
     * @param {number} height - Grid height
     * @param {number} scale - Scale
     * @param {Object} config - Rendering config
     * @private
     */
    renderGrid(width, height, scale, config) {
        const ctx = this.ctx;
        const { minorOpacity, majorOpacity, majorStep, color } = config;

        ctx.strokeStyle = color;
        ctx.lineWidth = 1;

        // Draw minor grid lines (every cell) with low opacity
        ctx.globalAlpha = minorOpacity;
        ctx.beginPath();

        for (let x = 0; x <= width; x++) {
            const px = x * scale;
            ctx.moveTo(px, 0);
            ctx.lineTo(px, height * scale);
        }

        for (let y = 0; y <= height; y++) {
            const py = y * scale;
            ctx.moveTo(0, py);
            ctx.lineTo(width * scale, py);
        }

        ctx.stroke();

        // Draw major grid lines (every majorStep cells) with higher opacity
        ctx.globalAlpha = majorOpacity;
        ctx.beginPath();

        for (let x = 0; x <= width; x += majorStep) {
            const px = x * scale;
            ctx.moveTo(px, 0);
            ctx.lineTo(px, height * scale);
        }

        for (let y = 0; y <= height; y += majorStep) {
            const py = y * scale;
            ctx.moveTo(0, py);
            ctx.lineTo(width * scale, py);
        }

        ctx.stroke();

        // Reset alpha
        ctx.globalAlpha = 1.0;
    }

    /**
     * Draw the cached grid to a context
     * @param {CanvasRenderingContext2D} ctx - Target context
     * @param {number} offsetX - X offset
     * @param {number} offsetY - Y offset
     * @param {number} [width] - Width to draw (default: canvas width)
     * @param {number} [height] - Height to draw (default: canvas height)
     */
    draw(ctx, offsetX, offsetY, width, height) {
        if (!this.canvas || !this.isValid) {
            console.warn('[GridCache] Drawing invalid cache');
            return;
        }

        const drawWidth = width ?? this.params.width * this.params.scale;
        const drawHeight = height ?? this.params.height * this.params.scale;

        ctx.drawImage(
            this.canvas,
            0,
            0,
            drawWidth,
            drawHeight,
            offsetX,
            offsetY,
            drawWidth,
            drawHeight
        );
    }

    /**
     * Invalidate the cache (force rebuild on next draw)
     */
    invalidate() {
        this.isValid = false;
    }

    /**
     * Clear the cache pool
     */
    clear() {
        this.cachePool.clear();
        this.canvas = null;
        this.ctx = null;
        this.isValid = false;
    }

    /**
     * Get cache statistics
     * @returns {Object}
     */
    getStats() {
        return {
            isValid: this.isValid,
            cacheSize: this.cachePool.size,
            maxCacheSize: CONFIG.maxCacheSize,
            currentParams: this.isValid ? { ...this.params } : null,
            cachedKeys: Array.from(this.cachePool.keys()),
        };
    }

    /**
     * Estimate memory usage
     * @returns {number} Approximate bytes used
     */
    estimateMemoryUsage() {
        let total = 0;

        for (const entry of this.cachePool.values()) {
            // Canvas memory: width * height * 4 bytes (RGBA)
            total += entry.canvas.width * entry.canvas.height * 4;
        }

        return total;
    }
}

/**
 * Singleton instance for application-wide use
 */
export const gridCache = new GridCache();

export default GridCache;
