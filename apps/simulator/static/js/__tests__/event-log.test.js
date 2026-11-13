import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { EventLog } from '../modules/event-log.js';

describe('EventLog', () => {
    let eventLog;
    let mockContainer;

    beforeEach(() => {
        // Mock DOM
        mockContainer = {
            scrollHeight: 100,
            scrollTop: 0,
            clientHeight: 50,
            innerHTML: '',
            addEventListener: vi.fn(),
        };

        document.getElementById = vi.fn((id) => {
            if (id === 'test-log') return mockContainer;
            if (id === 'test-log-scroll-indicator') return { classList: { add: vi.fn(), remove: vi.fn() } };
            return null;
        });

        document.addEventListener = vi.fn();

        eventLog = new EventLog('test-log');
        eventLog.container = mockContainer; // Direct assignment for testing
    });

    afterEach(() => {
        eventLog.destroy();
    });

    describe('Initialization', () => {
        it('should initialize with container', () => {
            expect(eventLog.containerId).toBe('test-log');
            expect(eventLog.events).toHaveLength(0);
        });

        it('should warn if container not found', () => {
            const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
            document.getElementById = vi.fn(() => null);

            const el = new EventLog('missing');

            expect(consoleSpy).toHaveBeenCalledWith('[EventLog] Container #missing not found');
            consoleSpy.mockRestore();
        });
    });

    describe('Logging', () => {
        it('should log an event', () => {
            eventLog.log('Test message', 'info');

            expect(eventLog.events).toHaveLength(1);
            expect(eventLog.events[0].message).toBe('Test message');
            expect(eventLog.events[0].type).toBe('info');
        });

        it('should log with agent ID', () => {
            eventLog.log('Agent action', 'success', 123);

            expect(eventLog.events[0].agentId).toBe(123);
        });

        it('should limit event count', () => {
            const maxEvents = eventLog.config.maxEvents;

            for (let i = 0; i < maxEvents + 10; i++) {
                eventLog.log(`Message ${i}`);
            }

            expect(eventLog.events).toHaveLength(maxEvents);
        });

        it('should emit logged event', () => {
            const handler = vi.fn();
            eventLog.addEventListener('logged', handler);

            eventLog.log('Test');

            expect(handler).toHaveBeenCalledWith(
                expect.objectContaining({
                    detail: expect.objectContaining({ message: 'Test' }),
                })
            );
        });
    });

    describe('Batching', () => {
        it('should batch reproduction events', () => {
            eventLog.log('Reproduction 1', 'success', null, 'reproduction');
            eventLog.log('Reproduction 2', 'success', null, 'reproduction');

            // Should not appear in events yet
            expect(eventLog.events).toHaveLength(0);
            expect(eventLog.batch.reproductions).toBe(2);
        });

        it('should batch game events', () => {
            eventLog.log('Game 1', 'info', null, 'game');
            expect(eventLog.batch.gamesPlayed).toBe(1);
        });

        it('should batch group events', () => {
            eventLog.log('Group 1', 'event', null, 'group');
            expect(eventLog.batch.groupFormations).toBe(1);
        });

        it('should batch interaction events', () => {
            eventLog.log('Interaction 1', 'info', null, 'interaction');
            expect(eventLog.batch.interactions).toBe(1);
        });

        it('should process batch after interval', () => {
            eventLog.log('Reproduction 1', 'success', null, 'reproduction');
            eventLog.log('Reproduction 2', 'success', null, 'reproduction');

            vi.useFakeTimers();
            vi.advanceTimersByTime(eventLog.config.batchInterval + 100);

            eventLog.log('Trigger batch processing');

            expect(eventLog.events.some((e) => e.message.includes('2 agents reproduced'))).toBe(true);
            expect(eventLog.batch.reproductions).toBe(0);

            vi.useRealTimers();
        });

        it('should flush batch manually', () => {
            eventLog.log('Reproduction 1', 'success', null, 'reproduction');
            eventLog.flushBatch();

            expect(eventLog.events.some((e) => e.message.includes('1 agent reproduced'))).toBe(true);
        });
    });

    describe('Scrolling', () => {
        it('should detect scrolled to bottom', () => {
            mockContainer.scrollTop = 50; // scrollHeight - clientHeight - threshold

            expect(eventLog.isScrolledToBottom()).toBe(true);
        });

        it('should detect scrolled up', () => {
            mockContainer.scrollTop = 0;

            expect(eventLog.isScrolledToBottom()).toBe(false);
        });

        it('should scroll to bottom', () => {
            mockContainer.scrollTop = 0;

            eventLog.scrollToBottom();

            expect(mockContainer.scrollTop).toBe(mockContainer.scrollHeight);
            expect(eventLog.isUserScrolledUp).toBe(false);
        });
    });

    describe('Rendering', () => {
        it('should render welcome message when empty', () => {
            eventLog.render();

            expect(mockContainer.innerHTML).toContain('Welcome to Agent Society Simulator');
        });

        it('should render events', () => {
            eventLog.log('Test message', 'success', 123);

            eventLog.render();

            expect(mockContainer.innerHTML).toContain('Test message');
            expect(mockContainer.innerHTML).toContain('[#123]');
        });

        it('should escape HTML in messages', () => {
            eventLog.log('<script>alert("xss")</script>');

            eventLog.render();

            expect(mockContainer.innerHTML).not.toContain('<script>');
            expect(mockContainer.innerHTML).toContain('&lt;script&gt;');
        });

        it('should auto-scroll when at bottom', () => {
            mockContainer.scrollTop = 50; // At bottom

            eventLog.log('Test');

            expect(mockContainer.scrollTop).toBe(mockContainer.scrollHeight);
        });

        it('should not auto-scroll when user scrolled up', () => {
            eventLog.isUserScrolledUp = true;
            mockContainer.scrollTop = 0;

            eventLog.log('Test');

            expect(mockContainer.scrollTop).toBe(0);
        });
    });

    describe('Queries', () => {
        beforeEach(() => {
            eventLog.log('Info message', 'info');
            eventLog.log('Success message', 'success');
            eventLog.log('Another info', 'info');
            eventLog.log('Agent action', 'success', 123);
        });

        it('should get all events', () => {
            const events = eventLog.getEvents();

            expect(events).toHaveLength(4);
        });

        it('should get events by type', () => {
            const infoEvents = eventLog.getEventsByType('info');

            expect(infoEvents).toHaveLength(2);
        });

        it('should get events for agent', () => {
            const agentEvents = eventLog.getEventsForAgent(123);

            expect(agentEvents).toHaveLength(1);
            expect(agentEvents[0].message).toBe('Agent action');
        });

        it('should search events', () => {
            const results = eventLog.search('message');

            expect(results.length).toBeGreaterThan(0);
            expect(results.every((e) => e.message.includes('message'))).toBe(true);
        });

        it('should search case-insensitive', () => {
            const results = eventLog.search('INFO');

            expect(results.length).toBeGreaterThan(0);
        });
    });

    describe('Clear', () => {
        it('should clear all events', () => {
            eventLog.log('Test 1');
            eventLog.log('Test 2');

            eventLog.clear();

            expect(eventLog.events).toHaveLength(0);
            expect(eventLog.batch.reproductions).toBe(0);
            expect(eventLog.batch.gamesPlayed).toBe(0);
        });

        it('should emit cleared event', () => {
            const handler = vi.fn();
            eventLog.addEventListener('cleared', handler);

            eventLog.clear();

            expect(handler).toHaveBeenCalled();
        });

        it('should render empty state after clear', () => {
            eventLog.log('Test');
            eventLog.clear();

            expect(mockContainer.innerHTML).toContain('Welcome to Agent Society Simulator');
        });
    });

    describe('Export', () => {
        it('should export to JSON', () => {
            eventLog.log('Test message', 'info', 123);

            const json = eventLog.exportToJSON();
            const parsed = JSON.parse(json);

            expect(parsed).toHaveLength(1);
            expect(parsed[0].message).toBe('Test message');
        });
    });

    describe('Timestamp', () => {
        it('should format timestamp', () => {
            const timestamp = eventLog.formatTimestamp();

            expect(timestamp).toMatch(/^\d{2}:\d{2}:\d{2}$/);
        });
    });

    describe('Destroy', () => {
        it('should cleanup on destroy', () => {
            eventLog.log('Test');

            eventLog.destroy();

            expect(eventLog.events).toHaveLength(0);
        });

        it('should emit destroyed event', () => {
            const handler = vi.fn();
            eventLog.addEventListener('destroyed', handler);

            eventLog.destroy();

            expect(handler).toHaveBeenCalled();
        });
    });
});
