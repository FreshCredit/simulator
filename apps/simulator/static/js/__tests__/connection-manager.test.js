import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { ConnectionManager } from '../modules/connection-manager.js';

describe('ConnectionManager', () => {
    let conn;
    let mockEventSource;

    beforeEach(() => {
        // Mock EventSource
        mockEventSource = {
            close: vi.fn(),
            readyState: EventSource.CONNECTING,
            _handlers: {},
            set onopen(fn) {
                this._handlers.open = fn;
            },
            set onmessage(fn) {
                this._handlers.message = fn;
            },
            set onerror(fn) {
                this._handlers.error = fn;
            },
            trigger(event, data) {
                if (this._handlers[event]) {
                    this._handlers[event](data);
                }
            },
        };

        global.EventSource = vi.fn(() => mockEventSource);
        vi.useFakeTimers();

        conn = new ConnectionManager('http://localhost:8082');
    });

    afterEach(() => {
        conn.destroy();
        vi.useRealTimers();
        vi.restoreAllMocks();
    });

    describe('Connection Lifecycle', () => {
        it('should connect to SSE endpoint', () => {
            conn.connect();

            expect(global.EventSource).toHaveBeenCalledWith('http://localhost:8082/api/live');
            expect(conn.getState()).toBe('connecting');
        });

        it('should emit connected event on open', () => {
            const handler = vi.fn();
            conn.addEventListener('connected', handler);

            conn.connect();
            mockEventSource.readyState = EventSource.OPEN;
            mockEventSource.trigger('open');

            expect(handler).toHaveBeenCalled();
            expect(conn.isConnected()).toBe(true);
        });

        it('should emit frame events on message', () => {
            const handler = vi.fn();
            conn.addEventListener('frame', handler);

            conn.connect();
            mockEventSource.readyState = EventSource.OPEN;
            mockEventSource.trigger('open');

            const frame = { tick: 1, agents: [] };
            mockEventSource.trigger('message', { data: JSON.stringify(frame) });

            expect(handler).toHaveBeenCalledWith(expect.objectContaining({ detail: frame }));
        });

        it('should handle disconnect', () => {
            conn.connect();
            conn.disconnect();

            expect(mockEventSource.close).toHaveBeenCalled();
            expect(conn.isConnected()).toBe(false);
        });

        it('should emit disconnected event', () => {
            const handler = vi.fn();
            conn.addEventListener('disconnected', handler);

            conn.connect();
            mockEventSource.trigger('error');

            expect(handler).toHaveBeenCalled();
        });
    });

    describe('Reconnection', () => {
        it('should reconnect with exponential backoff', () => {
            conn.connect();
            mockEventSource.trigger('error');

            // First reconnect attempt after 1s
            vi.advanceTimersByTime(1000);
            expect(global.EventSource).toHaveBeenCalledTimes(2);

            // Second reconnect attempt after 2s more (total 3s)
            mockEventSource.trigger('error');
            vi.advanceTimersByTime(2000);
            expect(global.EventSource).toHaveBeenCalledTimes(3);
        });

        it('should emit reconnecting event with attempt count', () => {
            const handler = vi.fn();
            conn.addEventListener('reconnecting', handler);

            conn.connect();
            mockEventSource.trigger('error');

            expect(handler).toHaveBeenCalledWith(
                expect.objectContaining({
                    detail: expect.objectContaining({
                        attempt: 1,
                        delay: 1000,
                    }),
                })
            );
        });

        it('should give up after max attempts', () => {
            const handler = vi.fn();
            conn.addEventListener('failed', handler);

            conn.connect();

            // Exhaust all retry attempts
            for (let i = 0; i < 50; i++) {
                mockEventSource.trigger('error');
                vi.advanceTimersByTime(30000);
            }

            expect(handler).toHaveBeenCalled();
        });
    });

    describe('Frame Flow Monitoring', () => {
        it('should detect stalled connections', () => {
            const handler = vi.fn();
            conn.addEventListener('stalled', handler);

            conn.connect();
            mockEventSource.readyState = EventSource.OPEN;
            mockEventSource.trigger('open');

            // Simulate no frames for 6 seconds
            vi.advanceTimersByTime(6000);

            expect(handler).toHaveBeenCalledWith(
                expect.objectContaining({
                    detail: expect.objectContaining({
                        timeSinceLastFrame: expect.any(Number),
                    }),
                })
            );
        });

        it('should reset stall timer on frame receipt', () => {
            const handler = vi.fn();
            conn.addEventListener('stalled', handler);

            conn.connect();
            mockEventSource.readyState = EventSource.OPEN;
            mockEventSource.trigger('open');

            // Send frames every second
            for (let i = 0; i < 10; i++) {
                vi.advanceTimersByTime(1000);
                mockEventSource.trigger('message', { data: JSON.stringify({ tick: i }) });
            }

            // Should not be stalled
            expect(handler).not.toHaveBeenCalled();
        });
    });

    describe('Error Handling', () => {
        it('should emit error on connection failure', () => {
            global.EventSource = vi.fn(() => {
                throw new Error('Connection refused');
            });

            const handler = vi.fn();
            conn.addEventListener('error', handler);

            conn.connect();

            expect(handler).toHaveBeenCalled();
        });

        it('should emit error on invalid JSON', () => {
            const handler = vi.fn();
            conn.addEventListener('error', handler);

            conn.connect();
            mockEventSource.readyState = EventSource.OPEN;
            mockEventSource.trigger('open');

            mockEventSource.trigger('message', { data: 'invalid json' });

            expect(handler).toHaveBeenCalled();
        });
    });

    describe('Cleanup', () => {
        it('should cleanup resources on destroy', () => {
            conn.connect();
            conn.destroy();

            expect(mockEventSource.close).toHaveBeenCalled();
            expect(conn.isConnected()).toBe(false);
        });
    });
});
