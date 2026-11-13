import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { PerformanceMonitor } from '../modules/perf-monitor.js';

describe('PerformanceMonitor', () => {
    let monitor;

    beforeEach(() => {
        vi.useFakeTimers();
        monitor = new PerformanceMonitor();
    });

    afterEach(() => {
        monitor.destroy();
        vi.useRealTimers();
    });

    describe('Lifecycle', () => {
        it('should start monitoring', () => {
            monitor.start();
            expect(monitor.isRunning).toBe(true);
        });

        it('should emit started event', () => {
            const handler = vi.fn();
            monitor.addEventListener('started', handler);

            monitor.start();

            expect(handler).toHaveBeenCalled();
        });

        it('should stop monitoring', () => {
            monitor.start();
            monitor.stop();

            expect(monitor.isRunning).toBe(false);
        });

        it('should emit stopped event', () => {
            const handler = vi.fn();
            monitor.addEventListener('stopped', handler);

            monitor.start();
            monitor.stop();

            expect(handler).toHaveBeenCalled();
        });
    });

    describe('Frame Tracking', () => {
        it('should record frames', () => {
            monitor.start();

            monitor.recordFrame();
            monitor.recordFrame();
            monitor.recordFrame();

            expect(monitor.frames).toHaveLength(3);
            expect(monitor.frameCount).toBe(3);
        });

        it('should not record when stopped', () => {
            monitor.recordFrame();

            expect(monitor.frames).toHaveLength(0);
        });

        it('should limit frame samples', () => {
            monitor.start();
            monitor.config.maxSamples = 10;

            for (let i = 0; i < 20; i++) {
                monitor.recordFrame();
            }

            expect(monitor.frames).toHaveLength(10);
        });

        it('should calculate frame duration', () => {
            monitor.start();

            vi.advanceTimersByTime(16);
            monitor.recordFrame();

            expect(monitor.frames[0].duration).toBeCloseTo(16, 0);
        });
    });

    describe('FPS Calculation', () => {
        it('should calculate FPS', () => {
            monitor.start();

            // Simulate 60fps (16.67ms per frame)
            for (let i = 0; i < 10; i++) {
                vi.advanceTimersByTime(16.67);
                monitor.recordFrame();
            }

            const fps = monitor.getFPS();
            expect(fps).toBeCloseTo(60, 0);
        });

        it('should return 0 with no frames', () => {
            expect(monitor.getFPS()).toBe(0);
        });
    });

    describe('Frame Time Stats', () => {
        it('should calculate frame time statistics', () => {
            monitor.start();

            // Record frames with varying durations
            vi.advanceTimersByTime(10);
            monitor.recordFrame();

            vi.advanceTimersByTime(20);
            monitor.recordFrame();

            vi.advanceTimersByTime(30);
            monitor.recordFrame();

            const stats = monitor.getFrameTimeStats();

            expect(stats.min).toBe(10);
            expect(stats.max).toBe(30);
            expect(stats.avg).toBe(20);
            expect(stats.median).toBe(20);
        });

        it('should return zeros with no frames', () => {
            const stats = monitor.getFrameTimeStats();

            expect(stats).toEqual({ min: 0, max: 0, avg: 0, median: 0 });
        });
    });

    describe('Performance Warnings', () => {
        it('should emit slowFrame for slow frames', () => {
            const handler = vi.fn();
            monitor.addEventListener('slowFrame', handler);
            monitor.start();

            vi.advanceTimersByTime(60); // > 50ms threshold
            monitor.recordFrame();

            expect(handler).toHaveBeenCalledWith(
                expect.objectContaining({
                    detail: expect.objectContaining({
                        frameTime: expect.any(Number),
                        threshold: 50,
                    }),
                })
            );
        });

        it('should emit poorPerformance for consistent slowdown', () => {
            const handler = vi.fn();
            monitor.addEventListener('poorPerformance', handler);
            monitor.start();

            // Record 10 slow frames (< 30fps = > 33.33ms)
            for (let i = 0; i < 10; i++) {
                vi.advanceTimersByTime(40);
                monitor.recordFrame();
            }

            expect(handler).toHaveBeenCalled();
        });
    });

    describe('Performance Rating', () => {
        it('should rate excellent performance', () => {
            monitor.start();

            // Simulate 60fps
            for (let i = 0; i < 10; i++) {
                vi.advanceTimersByTime(16);
                monitor.recordFrame();
            }

            expect(monitor.getRating()).toBe('excellent');
        });

        it('should rate good performance', () => {
            monitor.start();

            // Simulate ~50fps
            for (let i = 0; i < 10; i++) {
                vi.advanceTimersByTime(20);
                monitor.recordFrame();
            }

            expect(monitor.getRating()).toBe('good');
        });

        it('should rate poor performance', () => {
            monitor.start();

            // Simulate ~25fps
            for (let i = 0; i < 10; i++) {
                vi.advanceTimersByTime(40);
                monitor.recordFrame();
            }

            expect(monitor.getRating()).toBe('poor');
        });

        it('should rate critical performance', () => {
            monitor.start();

            // Simulate very slow
            for (let i = 0; i < 10; i++) {
                vi.advanceTimersByTime(100);
                monitor.recordFrame();
            }

            expect(monitor.getRating()).toBe('critical');
        });
    });

    describe('Measurement', () => {
        it('should measure function execution', () => {
            monitor.start();

            const fn = vi.fn(() => 'result');
            const result = monitor.measure('testOp', fn);

            expect(result).toBe('result');
            expect(fn).toHaveBeenCalled();
        });

        it('should measure async function', async () => {
            monitor.start();

            const fn = vi.fn(async () => 'asyncResult');
            const result = await monitor.measureAsync('testAsync', fn);

            expect(result).toBe('asyncResult');
        });

        it('should warn on slow operations', () => {
            const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
            monitor.start();

            monitor.measure('slowOp', () => {
                vi.advanceTimersByTime(20);
            });

            expect(consoleSpy).toHaveBeenCalledWith(expect.stringContaining('Slow operation'));

            consoleSpy.mockRestore();
        });
    });

    describe('Reporting', () => {
        it('should generate report', () => {
            monitor.start();

            vi.advanceTimersByTime(16);
            monitor.recordFrame();

            const report = monitor.getReport();

            expect(report).toContain('Performance Report');
            expect(report).toContain('FPS:');
            expect(report).toContain('Frame Time:');
        });

        it('should include full stats', () => {
            monitor.start();

            vi.advanceTimersByTime(16);
            monitor.recordFrame();

            const stats = monitor.getStats();

            expect(stats).toHaveProperty('fps');
            expect(stats).toHaveProperty('frameTime');
            expect(stats).toHaveProperty('rating');
            expect(stats).toHaveProperty('frameCount');
            expect(stats).toHaveProperty('uptime');
        });
    });

    describe('Reset', () => {
        it('should reset all data', () => {
            monitor.start();

            vi.advanceTimersByTime(16);
            monitor.recordFrame();

            monitor.reset();

            expect(monitor.frames).toHaveLength(0);
            expect(monitor.frameCount).toBe(0);
        });
    });

    describe('Update Interval', () => {
        it('should emit update event', () => {
            const handler = vi.fn();
            monitor.addEventListener('update', handler);
            monitor.start();

            vi.advanceTimersByTime(1000);

            expect(handler).toHaveBeenCalled();
        });

        it('should include stats in update', () => {
            const handler = vi.fn();
            monitor.addEventListener('update', handler);
            monitor.start();

            vi.advanceTimersByTime(16);
            monitor.recordFrame();
            vi.advanceTimersByTime(1000);

            expect(handler).toHaveBeenCalledWith(
                expect.objectContaining({
                    detail: expect.objectContaining({
                        fps: expect.any(Number),
                        frameTime: expect.any(Object),
                    }),
                })
            );
        });
    });
});
