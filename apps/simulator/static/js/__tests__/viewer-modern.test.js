import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { SimulatorViewer, getViewer, init } from '../viewer-modern.js';

describe('SimulatorViewer (Modern)', () => {
    let viewer;
    let mockCanvas;
    let mockCtx;

    beforeEach(() => {
        // Mock canvas and context
        mockCtx = {
            setTransform: vi.fn(),
            scale: vi.fn(),
            fillRect: vi.fn(),
            beginPath: vi.fn(),
            arc: vi.fn(),
            fill: vi.fn(),
        };

        mockCanvas = {
            getContext: vi.fn(() => mockCtx),
            getBoundingClientRect: vi.fn(() => ({ left: 0, top: 0, width: 800, height: 600 })),
            addEventListener: vi.fn(),
            removeEventListener: vi.fn(),
            clientWidth: 800,
            clientHeight: 600,
            style: {},
            width: 800,
            height: 600,
            parentElement: { clientWidth: 800, clientHeight: 600 },
        };

        // Mock document
        document.getElementById = vi.fn((id) => {
            if (id === 'simulation-canvas') return mockCanvas;
            if (id === 'activity-log-content') return { scrollHeight: 100, scrollTop: 0, clientHeight: 50 };
            if (id === 'tick-display') return { textContent: '' };
            if (id === 'canvas-overlay') return { classList: { add: vi.fn(), remove: vi.fn() } };
            if (id === 'game-over-overlay') return { classList: { add: vi.fn(), remove: vi.fn() } };
            return null;
        });

        document.addEventListener = vi.fn();
        document.createElement = vi.fn(() => ({ textContent: '' }));

        // Mock window
        global.window = {
            SIM_SERVICE_URL: 'http://localhost:8082',
            addEventListener: vi.fn(),
            dispatchEvent: vi.fn(),
            devicePixelRatio: 1,
            PixelatedLoader: { showGameOver: vi.fn(), stop: vi.fn() },
            location: { search: '' },
        };

        // Mock performance
        global.performance = { now: vi.fn(() => Date.now()) };

        vi.useFakeTimers();
    });

    afterEach(() => {
        if (viewer) {
            viewer.destroy();
        }
        vi.useRealTimers();
        vi.restoreAllMocks();
    });

    describe('Initialization', () => {
        it('should create viewer with canvas', () => {
            viewer = new SimulatorViewer('simulation-canvas');

            expect(viewer.canvas).toBe(mockCanvas);
            expect(viewer.ctx).toBe(mockCtx);
        });

        it('should init successfully', () => {
            viewer = new SimulatorViewer('simulation-canvas');
            const result = viewer.init();

            expect(result).toBe(true);
        });

        it('should fail init without canvas', () => {
            document.getElementById = vi.fn(() => null);
            viewer = new SimulatorViewer('missing');
            const result = viewer.init();

            expect(result).toBe(false);
        });

        it('should setup event listeners on init', () => {
            viewer = new SimulatorViewer('simulation-canvas');
            viewer.init();

            expect(mockCanvas.addEventListener).toHaveBeenCalledWith('click', expect.any(Function));
            expect(window.addEventListener).toHaveBeenCalledWith('resize', expect.any(Function));
        });
    });

    describe('Connection Management', () => {
        beforeEach(() => {
            viewer = new SimulatorViewer('simulation-canvas');
            viewer.init();
        });

        it('should connect to stream', () => {
            const connectSpy = vi.spyOn(viewer.connection, 'connect');
            viewer.connect();
            expect(connectSpy).toHaveBeenCalled();
        });

        it('should disconnect from stream', () => {
            const disconnectSpy = vi.spyOn(viewer.connection, 'disconnect');
            viewer.disconnect();
            expect(disconnectSpy).toHaveBeenCalled();
        });

        it('should check connection status', () => {
            vi.spyOn(viewer.connection, 'isConnected').mockReturnValue(true);
            expect(viewer.isConnected()).toBe(true);
        });
    });

    describe('Frame Handling', () => {
        beforeEach(() => {
            viewer = new SimulatorViewer('simulation-canvas');
            viewer.init();
        });

        it('should handle frame data', () => {
            const frame = {
                tick: 1,
                agents: [{ agent_id: 1, x: 10, y: 20, alive: true }],
                stats: { alive_agents: 1 },
            };

            viewer.handleFrame(frame);

            expect(viewer.store.get('simulation.currentTick')).toBe(1);
            expect(viewer.store.get('agents.all')).toHaveLength(1);
        });

        it('should handle simulation completion', () => {
            const frame = { tick: 100, status: 'Completed', agents: [] };

            viewer.handleFrame(frame);

            expect(viewer.store.get('simulation.status')).toBe('completed');
        });

        it('should build agent map on frame receipt', () => {
            const frame = {
                tick: 1,
                agents: [
                    { agent_id: 1, x: 10, y: 20 },
                    { agent_id: 2, x: 30, y: 40 },
                ],
            };

            viewer.handleFrame(frame);

            const storedFrame = viewer.store.get('frame.current');
            expect(storedFrame.agentMap).toBeInstanceOf(Map);
            expect(storedFrame.agentMap.get(1)).toBeDefined();
            expect(storedFrame.agentMap.get(2)).toBeDefined();
        });

        it('should dispatch simulatorFrame event', () => {
            const frame = { tick: 1, agents: [] };

            viewer.handleFrame(frame);

            expect(window.dispatchEvent).toHaveBeenCalledWith(
                expect.objectContaining({
                    type: 'simulatorFrame',
                    detail: frame,
                })
            );
        });
    });

    describe('State Management', () => {
        beforeEach(() => {
            viewer = new SimulatorViewer('simulation-canvas');
            viewer.init();
        });

        it('should set layer visibility', () => {
            viewer.setLayer('agents', false);
            expect(viewer.store.get('layers.agents')).toBe(false);
        });

        it('should set filter value', () => {
            viewer.setFilter('trait', 'd');
            expect(viewer.store.get('filters.trait')).toBe('d');
        });

        it('should reset filters', () => {
            viewer.setFilter('trait', 'd');
            viewer.setFilter('status', 'all');

            viewer.resetFilters();

            expect(viewer.store.get('filters.trait')).toBe('all');
            expect(viewer.store.get('filters.status')).toBe('alive');
        });

        it('should get current tick', () => {
            viewer.store.set('simulation.currentTick', 42);
            expect(viewer.getCurrentTick()).toBe(42);
        });

        it('should get stats', () => {
            const stats = { aliveAgents: 10 };
            viewer.store.set('stats', stats);
            expect(viewer.getStats()).toEqual(stats);
        });
    });

    describe('Simulation Control', () => {
        beforeEach(() => {
            viewer = new SimulatorViewer('simulation-canvas');
            viewer.init();
        });

        it('should pause simulation', async () => {
            vi.spyOn(viewer.api, 'pause').mockResolvedValue({});

            await viewer.pause();

            expect(viewer.store.get('simulation.status')).toBe('paused');
        });

        it('should resume simulation', async () => {
            vi.spyOn(viewer.api, 'resume').mockResolvedValue({});

            await viewer.resume();

            expect(viewer.store.get('simulation.status')).toBe('running');
        });

        it('should stop simulation', async () => {
            vi.spyOn(viewer.api, 'stop').mockResolvedValue({});

            await viewer.stop();

            expect(viewer.store.get('simulation.status')).toBe('stopped');
        });

        it('should set speed', async () => {
            vi.spyOn(viewer.api, 'setSpeed').mockResolvedValue({});

            await viewer.setSpeed(2.0);

            expect(viewer.store.get('simulation.speed')).toBe(2.0);
        });

        it('should handle API errors gracefully', async () => {
            vi.spyOn(viewer.api, 'pause').mockRejectedValue(new Error('API Error'));

            await viewer.pause();

            // Should not throw, error logged to event log
            expect(viewer.store.get('simulation.status')).not.toBe('paused');
        });
    });

    describe('Agent Selection', () => {
        beforeEach(() => {
            viewer = new SimulatorViewer('simulation-canvas');
            viewer.init();
        });

        it('should select agent', () => {
            viewer.selectAgent(123);

            expect(viewer.store.get('agents.selected')).toBe(123);
        });

        it('should deselect agent', () => {
            viewer.selectAgent(123);
            viewer.deselectAgent();

            expect(viewer.store.get('agents.selected')).toBeNull();
        });
    });

    describe('Rendering', () => {
        beforeEach(() => {
            viewer = new SimulatorViewer('simulation-canvas');
            viewer.init();
        });

        it('should schedule render on RAF', () => {
            viewer.scheduleRender();

            expect(viewer.renderPending).toBe(true);
        });

        it('should render frame', () => {
            viewer.store.set('frame.current', {
                agents: [{ agent_id: 1, x: 64, y: 64, alive: true }],
            });
            viewer.store.set('layers.agents', true);

            viewer.render();

            expect(mockCtx.setTransform).toHaveBeenCalled();
            expect(mockCtx.fillRect).toHaveBeenCalled(); // Clear
        });
    });

    describe('Event Detection', () => {
        beforeEach(() => {
            viewer = new SimulatorViewer('simulation-canvas');
            viewer.init();
        });

        it('should detect game events', () => {
            const frame = {
                tick: 1,
                agents: [],
                game_events: [
                    { game_type: 'PrisonersDilemma', agent_a: 1, agent_b: 2, outcome: 'Cooperate' },
                ],
            };

            viewer.detectEvents(frame);

            const events = viewer.eventLog.getEvents();
            expect(events.some((e) => e.message.includes('PrisonersDilemma'))).toBe(true);
        });

        it('should detect low population', () => {
            viewer.store.set('agents.aliveCount', 10);

            const frame = {
                tick: 1,
                agents: Array(3)
                    .fill(null)
                    .map((_, i) => ({ agent_id: i, alive: true })),
            };

            viewer.detectEvents(frame);

            const events = viewer.eventLog.getEvents();
            expect(events.some((e) => e.message.includes('critically low'))).toBe(true);
        });

        it('should detect population milestone', () => {
            viewer.store.set('agents.aliveCount', 50);

            const frame = {
                tick: 1,
                agents: Array(100)
                    .fill(null)
                    .map((_, i) => ({ agent_id: i, alive: true })),
            };

            viewer.detectEvents(frame);

            const events = viewer.eventLog.getEvents();
            expect(events.some((e) => e.message.includes('Milestone: 100'))).toBe(true);
        });

        it('should log tick milestones', () => {
            const frame = {
                tick: 100,
                agents: [{ agent_id: 1, alive: true }],
            };

            viewer.detectEvents(frame);

            const events = viewer.eventLog.getEvents();
            expect(events.some((e) => e.message.includes('Tick 100'))).toBe(true);
        });
    });

    describe('Export/Import', () => {
        beforeEach(() => {
            viewer = new SimulatorViewer('simulation-canvas');
            viewer.init();
        });

        it('should export state', () => {
            viewer.store.set('simulation.currentTick', 42);
            viewer.eventLog.log('Test event');

            const state = viewer.exportState();

            expect(state).toHaveProperty('store');
            expect(state).toHaveProperty('events');
            expect(state).toHaveProperty('timestamp');
            expect(state.store.simulation.currentTick).toBe(42);
        });
    });

    describe('Singleton', () => {
        it('should return same instance', () => {
            const v1 = getViewer('simulation-canvas');
            const v2 = getViewer('simulation-canvas');

            expect(v1).toBe(v2);
        });

        it('should init via init function', () => {
            const result = init('simulation-canvas');
            expect(result).toBe(true);
        });
    });

    describe('Cleanup', () => {
        it('should destroy and cleanup', () => {
            viewer = new SimulatorViewer('simulation-canvas');
            viewer.init();

            const destroySpy = vi.spyOn(viewer.connection, 'destroy');

            viewer.destroy();

            expect(destroySpy).toHaveBeenCalled();
        });
    });
});
