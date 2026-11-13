import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { DebugSystem } from '../modules/debug-system.js';

describe('DebugSystem', () => {
    let debug;

    beforeEach(() => {
        vi.useFakeTimers();
        debug = new DebugSystem({ enabled: true });
    });

    afterEach(() => {
        debug.destroy();
        vi.useRealTimers();
    });

    describe('Initialization', () => {
        it('should be disabled by default without debug flag', () => {
            const d = new DebugSystem();
            expect(d.isEnabled()).toBe(false);
        });

        it('should be enabled when explicitly set', () => {
            expect(debug.isEnabled()).toBe(true);
        });

        it('should toggle enabled state', () => {
            debug.setEnabled(false);
            expect(debug.isEnabled()).toBe(false);

            debug.setEnabled(true);
            expect(debug.isEnabled()).toBe(true);
        });

        it('should emit enabledChanged event', () => {
            const handler = vi.fn();
            debug.addEventListener('enabledChanged', handler);

            debug.setEnabled(false);

            expect(handler).toHaveBeenCalledWith(expect.objectContaining({ detail: { enabled: false } }));
        });
    });

    describe('Frame Tracking', () => {
        it('should record frame receipt', () => {
            const frame = { tick: 1, agents: [], stats: {} };
            debug.recordFrameReceipt(frame);

            expect(debug.frameStats.framesReceived).toBe(1);
            expect(debug.frameHistory).toHaveLength(1);
            expect(debug.frameHistory[0].tick).toBe(1);
        });

        it('should limit frame history size', () => {
            const maxHistory = debug.config.maxFrameHistory;

            for (let i = 0; i < maxHistory + 10; i++) {
                debug.recordFrameReceipt({ tick: i, agents: [], stats: {} });
            }

            expect(debug.frameHistory).toHaveLength(maxHistory);
        });

        it('should calculate average frame interval', () => {
            debug.recordFrameReceipt({ tick: 1, agents: [], stats: {} });

            vi.advanceTimersByTime(100);
            debug.recordFrameReceipt({ tick: 2, agents: [], stats: {} });

            vi.advanceTimersByTime(100);
            debug.recordFrameReceipt({ tick: 3, agents: [], stats: {} });

            expect(debug.frameStats.avgFrameInterval).toBeCloseTo(100, 0);
        });

        it('should not record when disabled', () => {
            debug.setEnabled(false);
            debug.recordFrameReceipt({ tick: 1, agents: [], stats: {} });

            expect(debug.frameStats.framesReceived).toBe(0);
        });
    });

    describe('Agent Position Tracking', () => {
        it('should record agent position', () => {
            const agent = { agent_id: 1, x: 10, y: 20, energy: 50, alive: true };
            debug.recordAgentPosition(agent, 1);

            expect(debug.agentPositionHistory[1]).toHaveLength(1);
            expect(debug.agentPositionHistory[1][0].x).toBe(10);
        });

        it('should limit agent history size', () => {
            const maxHistory = debug.config.maxAgentHistory;
            const agent = { agent_id: 1, x: 0, y: 0, energy: 50, alive: true };

            for (let i = 0; i < maxHistory + 10; i++) {
                debug.recordAgentPosition(agent, i);
            }

            expect(debug.agentPositionHistory[1]).toHaveLength(maxHistory);
        });

        it('should detect frozen agents', () => {
            const agent = { agent_id: 1, x: 10, y: 20, energy: 50, alive: true };

            // Record same position multiple times
            for (let i = 0; i < 25; i++) {
                debug.recordAgentPosition(agent, i);
            }

            expect(debug.movementStats.frozenAgentTicks[1]).toBe(24);
        });

        it('should emit FROZEN log when threshold reached', () => {
            const consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
            const agent = { agent_id: 1, x: 10, y: 20, energy: 50, alive: true, age: 10 };

            // Record same position to trigger frozen warning
            for (let i = 0; i <= debug.config.frozenAgentThreshold; i++) {
                debug.recordAgentPosition(agent, i);
            }

            expect(consoleSpy).toHaveBeenCalledWith(
                expect.stringContaining('FROZEN'),
                expect.any(String),
                expect.any(Object)
            );

            consoleSpy.mockRestore();
        });
    });

    describe('Connection Events', () => {
        it('should record connection events', () => {
            debug.recordConnectionEvent('connected', { url: 'test' });

            expect(debug.connectionEvents).toHaveLength(1);
            expect(debug.connectionEvents[0].event).toBe('connected');
        });

        it('should limit connection event history', () => {
            for (let i = 0; i < 25; i++) {
                debug.recordConnectionEvent(`event${i}`);
            }

            expect(debug.connectionEvents).toHaveLength(20);
        });
    });

    describe('Statistics', () => {
        it('should get debug stats', () => {
            debug.recordFrameReceipt({ tick: 1, agents: [{ agent_id: 1 }], stats: { alive_agents: 1 } });
            debug.recordConnectionEvent('test');

            const stats = debug.getStats();

            expect(stats.frames.received).toBe(1);
            expect(stats.agents.tracked).toBe(1);
        });

        it('should calculate average movement', () => {
            const agent = { agent_id: 1, x: 0, y: 0, energy: 50, alive: true };
            debug.recordAgentPosition(agent, 1);

            agent.x = 10;
            debug.recordAgentPosition(agent, 2);

            const stats = debug.getStats();
            expect(stats.movement.avgMovement).toBeGreaterThan(0);
        });
    });

    describe('Agent History', () => {
        it('should get agent movement history', () => {
            const agent = { agent_id: 1, x: 0, y: 0, energy: 50, alive: true };

            for (let i = 0; i < 5; i++) {
                agent.x = i * 10;
                debug.recordAgentPosition(agent, i);
            }

            const history = debug.getAgentHistory(1);

            expect(history.agentId).toBe(1);
            expect(history.totalPositions).toBe(5);
            expect(history.movementHistory).toHaveLength(4);
        });

        it('should return null for unknown agent', () => {
            const history = debug.getAgentHistory(999);
            expect(history).toBeNull();
        });
    });

    describe('Frame Flow', () => {
        it('should report healthy when frames received recently', () => {
            debug.recordFrameReceipt({ tick: 1, agents: [], stats: {} });

            const health = debug.checkFrameFlow();

            expect(health.healthy).toBe(true);
            expect(health.issue).toBeNull();
        });

        it('should report unhealthy when no frames for a while', () => {
            debug.recordFrameReceipt({ tick: 1, agents: [], stats: {} });

            vi.advanceTimersByTime(2000);

            const health = debug.checkFrameFlow();

            expect(health.healthy).toBe(false);
            expect(health.issue).toContain('No frame received');
        });

        it('should report no frames when history empty', () => {
            const health = debug.checkFrameFlow();

            expect(health.healthy).toBe(false);
            expect(health.issue).toBe('No frames received yet');
        });
    });

    describe('Diagnostics', () => {
        it('should run full diagnostics', () => {
            debug.recordFrameReceipt({ tick: 1, agents: [{ agent_id: 1 }], stats: {} });

            const diagnostics = debug.runDiagnostics();

            expect(diagnostics).toHaveProperty('timestamp');
            expect(diagnostics).toHaveProperty('frameFlow');
            expect(diagnostics).toHaveProperty('stats');
        });
    });

    describe('Cleanup', () => {
        it('should cleanup dead agents', () => {
            const agent = { agent_id: 1, x: 0, y: 0, energy: 50, alive: true };

            for (let i = 0; i < 10; i++) {
                debug.recordAgentPosition(agent, i);
            }

            // Cleanup with agent dead for 150 ticks
            debug.cleanupDeadAgents([], 160, 100);

            expect(debug.agentPositionHistory[1]).toBeUndefined();
        });

        it('should keep recent dead agents', () => {
            const agent = { agent_id: 1, x: 0, y: 0, energy: 50, alive: true };

            for (let i = 0; i < 10; i++) {
                debug.recordAgentPosition(agent, i);
            }

            // Cleanup with agent dead for only 50 ticks
            debug.cleanupDeadAgents([], 60, 100);

            expect(debug.agentPositionHistory[1]).toBeDefined();
        });
    });

    describe('Health Checks', () => {
        it('should start health checks', () => {
            debug.recordFrameReceipt({ tick: 1, agents: [], stats: {} });
            debug.startHealthChecks(100);

            vi.advanceTimersByTime(100);

            expect(debug.healthCheckInterval).not.toBeNull();
        });

        it('should emit frameFlowIssue when unhealthy', () => {
            const handler = vi.fn();
            debug.addEventListener('frameFlowIssue', handler);

            debug.recordFrameReceipt({ tick: 1, agents: [], stats: {} });
            debug.startHealthChecks(100);

            vi.advanceTimersByTime(2000);

            expect(handler).toHaveBeenCalled();
        });
    });

    describe('Clear', () => {
        it('should clear all data', () => {
            debug.recordFrameReceipt({ tick: 1, agents: [], stats: {} });
            debug.recordConnectionEvent('test');

            debug.clear();

            expect(debug.frameHistory).toHaveLength(0);
            expect(debug.connectionEvents).toHaveLength(0);
            expect(debug.frameStats.framesReceived).toBe(0);
        });
    });
});
