import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { GridCache } from '../modules/grid-cache.js';

describe('GridCache', () => {
    let cache;

    beforeEach(() => {
        // Mock canvas context
        const mockCtx = {
            stroke: vi.fn(),
            beginPath: vi.fn(),
            moveTo: vi.fn(),
            lineTo: vi.fn(),
        };

        // Mock canvas
        const mockCanvas = {
            getContext: vi.fn(() => mockCtx),
            width: 0,
            height: 0,
        };

        // Mock document.createElement
        document.createElement = vi.fn((tag) => {
            if (tag === 'canvas') {
                return { ...mockCanvas };
            }
            return {};
        });

        cache = new GridCache();
    });

    afterEach(() => {
        cache.clear();
    });

    describe('Initialization', () => {
        it('should start with invalid cache', () => {
            expect(cache.isValid).toBe(false);
            expect(cache.canvas).toBeNull();
        });

        it('should have empty cache pool', () => {
            expect(cache.cachePool.size).toBe(0);
        });
    });

    describe('Cache Building', () => {
        it('should build cache with correct dimensions', () => {
            const canvas = cache.build(128, 128, 10, {});

            expect(canvas).toBeDefined();
            expect(cache.isValid).toBe(true);
            expect(cache.isCacheValid(128, 128, 10)).toBe(true);
        });

        it('should store parameters', () => {
            cache.build(128, 128, 10, { minorOpacity: 0.2, majorOpacity: 0.5 });

            expect(cache.params.width).toBe(128);
            expect(cache.params.height).toBe(128);
            expect(cache.params.scale).toBe(10);
            expect(cache.params.minorOpacity).toBe(0.2);
            expect(cache.params.majorOpacity).toBe(0.5);
        });

        it('should invalidate cache with different parameters', () => {
            cache.build(128, 128, 10, {});

            expect(cache.isCacheValid(64, 64, 10)).toBe(false);
            expect(cache.isCacheValid(128, 128, 20)).toBe(false);
        });

        it('should accept similar scale within tolerance', () => {
            cache.build(128, 128, 10.0, {});

            expect(cache.isCacheValid(128, 128, 10.005)).toBe(true);
            expect(cache.isCacheValid(128, 128, 10.1)).toBe(false);
        });

        it('should render grid lines', () => {
            const canvas = cache.build(10, 10, 10, {});
            const ctx = canvas.getContext('2d');

            expect(ctx.beginPath).toHaveBeenCalled();
            expect(ctx.moveTo).toHaveBeenCalled();
            expect(ctx.lineTo).toHaveBeenCalled();
            expect(ctx.stroke).toHaveBeenCalled();
        });
    });

    describe('LRU Caching', () => {
        it('should cache multiple scales', () => {
            cache.build(128, 128, 10, {});
            cache.build(128, 128, 20, {});
            cache.build(64, 64, 10, {});

            expect(cache.cachePool.size).toBe(3);
        });

        it('should evict oldest when cache full', () => {
            // Build more than maxCacheSize
            cache.build(128, 128, 10, {});
            cache.build(128, 128, 20, {});
            cache.build(128, 128, 30, {});
            cache.build(128, 128, 40, {});

            expect(cache.cachePool.size).toBeLessThanOrEqual(3);
        });

        it('should reuse cached canvas', () => {
            const canvas1 = cache.build(128, 128, 10, {});
            const canvas2 = cache.build(128, 128, 10, {});

            expect(canvas1).toBe(canvas2);
        });
    });

    describe('Drawing', () => {
        it('should draw cached grid to context', () => {
            cache.build(128, 128, 10, {});

            const targetCtx = {
                drawImage: vi.fn(),
            };

            cache.draw(targetCtx, 100, 200);

            expect(targetCtx.drawImage).toHaveBeenCalledWith(
                expect.any(Object),
                0,
                0,
                1280,
                1280,
                100,
                200,
                1280,
                1280
            );
        });

        it('should warn when drawing invalid cache', () => {
            const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

            const targetCtx = { drawImage: vi.fn() };
            cache.draw(targetCtx, 0, 0);

            expect(consoleSpy).toHaveBeenCalledWith('[GridCache] Drawing invalid cache');

            consoleSpy.mockRestore();
        });
    });

    describe('Invalidation', () => {
        it('should invalidate cache', () => {
            cache.build(128, 128, 10, {});
            expect(cache.isValid).toBe(true);

            cache.invalidate();
            expect(cache.isValid).toBe(false);
        });
    });

    describe('Cleanup', () => {
        it('should clear all caches', () => {
            cache.build(128, 128, 10, {});
            cache.build(64, 64, 20, {});

            cache.clear();

            expect(cache.cachePool.size).toBe(0);
            expect(cache.canvas).toBeNull();
            expect(cache.isValid).toBe(false);
        });
    });

    describe('Statistics', () => {
        it('should report cache stats', () => {
            cache.build(128, 128, 10, {});

            const stats = cache.getStats();

            expect(stats.isValid).toBe(true);
            expect(stats.cacheSize).toBe(1);
            expect(stats.maxCacheSize).toBe(3);
            expect(stats.currentParams).toBeDefined();
        });

        it('should estimate memory usage', () => {
            cache.build(100, 100, 10, {}); // 1000x1000 pixel canvas

            const usage = cache.estimateMemoryUsage();

            // 1000 * 1000 * 4 bytes (RGBA) = 4,000,000 bytes = ~4MB
            expect(usage).toBeGreaterThan(0);
        });
    });

    describe('Margin', () => {
        it('should create canvas with margin', () => {
            const canvas = cache.build(100, 100, 10, {});

            // With 1.5x margin: 100 * 10 * 1.5 = 1500 pixels
            expect(canvas.width).toBeGreaterThan(1000);
            expect(canvas.height).toBeGreaterThan(1000);
        });
    });
});
