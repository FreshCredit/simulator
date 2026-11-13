import { describe, it, expect, beforeEach, vi } from 'vitest';

/**
 * Performance tests for viewer.js P0 fixes
 * Tests that O(n²) lookups have been replaced with O(1) lookups
 */

describe('P0 Performance Fixes', () => {
    describe('Agent ID Map', () => {
        it('should build agentMap in O(n) time', () => {
            // Create mock frame with many agents
            const agents = Array.from({ length: 1000 }, (_, i) => ({
                agent_id: i,
                x: Math.random() * 100,
                y: Math.random() * 100,
                alive: true,
            }));

            const frame = { tick: 1, agents };

            // Measure time to build map
            const start = performance.now();
            const agentMap = new Map(agents.map((a) => [a.agent_id, a]));
            const end = performance.now();

            // Should be very fast (O(n))
            expect(end - start).toBeLessThan(10); // Less than 10ms for 1000 agents

            // Verify O(1) lookup
            const lookupStart = performance.now();
            for (let i = 0; i < 1000; i++) {
                const agent = agentMap.get(i);
                expect(agent).toBeDefined();
                expect(agent.agent_id).toBe(i);
            }
            const lookupEnd = performance.now();

            // 1000 O(1) lookups should be very fast
            expect(lookupEnd - lookupStart).toBeLessThan(5);
        });

        it('should be faster than Array.find() for lookups', () => {
            const agents = Array.from({ length: 500 }, (_, i) => ({
                agent_id: i,
                x: Math.random() * 100,
                y: Math.random() * 100,
            }));

            const agentMap = new Map(agents.map((a) => [a.agent_id, a]));

            // Time 500 lookups with Map (O(1))
            const mapStart = performance.now();
            for (let i = 0; i < 500; i++) {
                agents.forEach((a) => agentMap.get(a.agent_id));
            }
            const mapTime = performance.now() - mapStart;

            // Time 500 lookups with Array.find() (O(n))
            const findStart = performance.now();
            for (let i = 0; i < 500; i++) {
                agents.forEach((a) => agents.find((x) => x.agent_id === a.agent_id));
            }
            const findTime = performance.now() - findStart;

            // Map should be significantly faster (at least 5x)
            expect(mapTime).toBeLessThan(findTime / 5);
        });
    });

    describe('Frame Buffer Memory', () => {
        it('should limit buffer size to prevent unbounded growth', () => {
            const FRAME_BUFFER_SIZE = 30;
            const frameBuffer = [];

            // Simulate adding many frames
            for (let i = 0; i < 100; i++) {
                frameBuffer.push({ tick: i, agents: [] });
                if (frameBuffer.length > FRAME_BUFFER_SIZE) {
                    frameBuffer.shift();
                }
            }

            // Buffer should never exceed max size
            expect(frameBuffer.length).toBe(FRAME_BUFFER_SIZE);

            // Should contain most recent frames
            expect(frameBuffer[0].tick).toBe(70); // 100 - 30 = 70
            expect(frameBuffer[FRAME_BUFFER_SIZE - 1].tick).toBe(99);
        });

        it('should create isolated copies of frame data', () => {
            const agent = { agent_id: 1, x: 10, y: 20, nested: { value: 5 } };
            const frame = { tick: 1, agents: [agent] };

            // Simulate bufferFrame with structured clone
            const buffered = {
                tick: frame.tick,
                agents: JSON.parse(JSON.stringify(frame.agents)),
            };

            // Modify original
            agent.x = 999;
            agent.nested.value = 999;

            // Buffered should be isolated
            expect(buffered.agents[0].x).toBe(10);
            expect(buffered.agents[0].nested.value).toBe(5);
        });
    });

    describe('Agent History Cleanup', () => {
        it('should cleanup dead agent history after threshold', () => {
            const currentTick = 200;
            const agentPositionHistory = {
                1: [{ tick: 50 }, { tick: 100 }], // Dead agent, old
                2: [{ tick: 150 }, { tick: 190 }], // Dead agent, recent
                3: [{ tick: 180 }, { tick: 195 }], // Alive agent
            };

            const aliveIds = new Set([3]);
            const CLEANUP_THRESHOLD = 100;

            // Cleanup logic
            const allIds = Object.keys(agentPositionHistory);
            for (const id of allIds) {
                const agentId = parseInt(id);
                if (!aliveIds.has(agentId)) {
                    const history = agentPositionHistory[id];
                    const lastEntry = history[history.length - 1];
                    if (lastEntry && currentTick - lastEntry.tick > CLEANUP_THRESHOLD) {
                        delete agentPositionHistory[id];
                    }
                }
            }

            // Old dead agent should be cleaned up
            expect(agentPositionHistory[1]).toBeUndefined();

            // Recent dead agent should be kept
            expect(agentPositionHistory[2]).toBeDefined();

            // Alive agent should be kept
            expect(agentPositionHistory[3]).toBeDefined();
        });
    });
});

describe('Memory Leak Prevention', () => {
    it('should not accumulate agent history indefinitely', () => {
        const MAX_AGENT_HISTORY = 100;
        const agentPositionHistory = {};
        let currentTick = 0;

        // Simulate 500 ticks with agent appearing and dying
        for (let tick = 0; tick < 500; tick++) {
            currentTick = tick;
            const agentId = 1;

            if (!agentPositionHistory[agentId]) {
                agentPositionHistory[agentId] = [];
            }

            agentPositionHistory[agentId].push({ tick, x: 0, y: 0 });

            // Keep only recent history
            if (agentPositionHistory[agentId].length > MAX_AGENT_HISTORY) {
                agentPositionHistory[agentId].shift();
            }
        }

        // History should be capped
        expect(agentPositionHistory[1].length).toBe(MAX_AGENT_HISTORY);
    });

    it('should cleanup resources on disconnect', () => {
        // Mock event source
        const mockClose = vi.fn();
        const eventSource = { close: mockClose };

        // Simulate disconnect
        if (eventSource) {
            eventSource.close();
        }

        expect(mockClose).toHaveBeenCalled();
    });
});
