import { describe, it, expect, vi, beforeEach } from 'vitest';
import { StateStore, DEFAULT_STATE } from '../modules/state-store.js';

describe('StateStore', () => {
    let store;

    beforeEach(() => {
        store = new StateStore();
    });

    describe('Basic Operations', () => {
        it('should get entire state', () => {
            const state = store.get();
            expect(state).toEqual(DEFAULT_STATE);
        });

        it('should get nested state by path', () => {
            expect(store.get('simulation.status')).toBe('stopped');
            expect(store.get('layers.agents')).toBe(true);
        });

        it('should set state at path', () => {
            store.set('simulation.status', 'running');
            expect(store.get('simulation.status')).toBe('running');
        });

        it('should set nested state', () => {
            store.set('layers.terrain', true);
            expect(store.get('layers.terrain')).toBe(true);
        });

        it('should return undefined for non-existent paths', () => {
            expect(store.get('nonexistent.path')).toBeUndefined();
        });

        it('should handle chained sets', () => {
            store
                .set('layers.agents', false)
                .set('layers.terrain', true)
                .set('simulation.speed', 2.0);

            expect(store.get('layers.agents')).toBe(false);
            expect(store.get('layers.terrain')).toBe(true);
            expect(store.get('simulation.speed')).toBe(2.0);
        });
    });

    describe('Subscriptions', () => {
        it('should notify subscribers on change', () => {
            const handler = vi.fn();
            store.subscribe('layers.agents', handler);

            store.set('layers.agents', false);

            expect(handler).toHaveBeenCalledWith(false, true, 'layers.agents');
        });

        it('should provide unsubscribe function', () => {
            const handler = vi.fn();
            const unsubscribe = store.subscribe('layers.agents', handler);

            unsubscribe();
            store.set('layers.agents', false);

            expect(handler).not.toHaveBeenCalled();
        });

        it('should notify parent path subscribers', () => {
            const handler = vi.fn();
            store.subscribe('layers', handler);

            store.set('layers.agents', false);

            expect(handler).toHaveBeenCalled();
        });

        it('should emit global change event', () => {
            const handler = vi.fn();
            store.addEventListener('change', handler);

            store.set('simulation.status', 'running');

            expect(handler).toHaveBeenCalledWith(
                expect.objectContaining({
                    detail: expect.objectContaining({
                        path: 'simulation.status',
                        newValue: 'running',
                        oldValue: 'stopped',
                    }),
                })
            );
        });

        it('should handle multiple subscribers', () => {
            const handler1 = vi.fn();
            const handler2 = vi.fn();

            store.subscribe('layers.agents', handler1);
            store.subscribe('layers.agents', handler2);

            store.set('layers.agents', false);

            expect(handler1).toHaveBeenCalled();
            expect(handler2).toHaveBeenCalled();
        });
    });

    describe('Immutability', () => {
        it('should return deep clones', () => {
            store.set('agents.all', [{ id: 1 }]);
            const agents1 = store.get('agents.all');
            const agents2 = store.get('agents.all');

            expect(agents1).toEqual(agents2);
            expect(agents1).not.toBe(agents2);
        });

        it('should not mutate external objects', () => {
            const obj = { nested: { value: 1 } };
            store.set('test', obj);

            obj.nested.value = 999;

            expect(store.get('test.nested.value')).toBe(1);
        });
    });

    describe('Batching', () => {
        it('should batch multiple updates', () => {
            const handler = vi.fn();
            store.subscribe('layers.agents', handler);
            store.subscribe('layers.terrain', handler);

            store.batch(() => {
                store.set('layers.agents', false);
                store.set('layers.terrain', true);
            });

            expect(handler).toHaveBeenCalledTimes(2);
        });

        it('should handle errors in batch', () => {
            const handler = vi.fn();
            store.subscribe('layers.agents', handler);

            expect(() => {
                store.batch(() => {
                    store.set('layers.agents', false);
                    throw new Error('Test error');
                });
            }).toThrow('Test error');

            // Batch should still flush
            expect(handler).toHaveBeenCalled();
        });
    });

    describe('Toggle', () => {
        it('should toggle boolean values', () => {
            expect(store.get('layers.agents')).toBe(true);

            store.toggle('layers.agents');
            expect(store.get('layers.agents')).toBe(false);

            store.toggle('layers.agents');
            expect(store.get('layers.agents')).toBe(true);
        });

        it('should throw on non-boolean toggle', () => {
            expect(() => store.toggle('simulation.speed')).toThrow();
        });
    });

    describe('Reset', () => {
        it('should reset specific path to default', () => {
            store.set('simulation.status', 'running');
            expect(store.get('simulation.status')).toBe('running');

            store.reset('simulation.status');
            expect(store.get('simulation.status')).toBe('stopped');
        });

        it('should reset entire state', () => {
            store.set('simulation.status', 'running');
            store.set('layers.agents', false);

            store.reset();

            expect(store.get()).toEqual(DEFAULT_STATE);
        });

        it('should emit reset event', () => {
            const handler = vi.fn();
            store.addEventListener('reset', handler);

            store.reset();

            expect(handler).toHaveBeenCalled();
        });
    });

    describe('Error Handling', () => {
        it('should catch subscriber errors', () => {
            const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});

            store.subscribe('layers.agents', () => {
                throw new Error('Subscriber error');
            });

            store.set('layers.agents', false);

            expect(consoleError).toHaveBeenCalledWith(
                expect.stringContaining('Error in subscriber'),
                expect.any(Error)
            );

            consoleError.mockRestore();
        });
    });
});
