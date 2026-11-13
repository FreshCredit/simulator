import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { APIClient, APIError } from '../modules/api-client.js';

describe('APIClient', () => {
    let api;
    let fetchMock;

    beforeEach(() => {
        api = new APIClient('http://localhost:8082');
        fetchMock = vi.fn();
        global.fetch = fetchMock;
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
        vi.restoreAllMocks();
    });

    describe('Request Handling', () => {
        it('should make GET request', async () => {
            fetchMock.mockResolvedValueOnce({
                ok: true,
                status: 200,
                json: async () => ({ status: 'ok' }),
            });

            const result = await api.getHealth();

            expect(fetchMock).toHaveBeenCalledWith(
                'http://localhost:8082/health',
                expect.objectContaining({
                    method: 'GET',
                    headers: expect.objectContaining({
                        Accept: 'application/json',
                    }),
                })
            );
            expect(result).toEqual({ status: 'ok' });
        });

        it('should make POST request with body', async () => {
            fetchMock.mockResolvedValueOnce({
                ok: true,
                status: 200,
                json: async () => ({ success: true }),
            });

            await api.setSpeed(2.0);

            expect(fetchMock).toHaveBeenCalledWith(
                'http://localhost:8082/api/live/speed',
                expect.objectContaining({
                    method: 'POST',
                    body: JSON.stringify({ factor: 2.0 }),
                })
            );
        });

        it('should throw APIError on HTTP error', async () => {
            fetchMock.mockResolvedValueOnce({
                ok: false,
                status: 500,
                statusText: 'Internal Server Error',
                text: async () => 'Server error',
            });

            await expect(api.getHealth()).rejects.toThrow(APIError);
        });

        it('should include error details', async () => {
            fetchMock.mockResolvedValueOnce({
                ok: false,
                status: 404,
                statusText: 'Not Found',
                text: async () => 'Resource not found',
            });

            try {
                await api.getExperiment('nonexistent');
            } catch (err) {
                expect(err.status).toBe(404);
                expect(err.endpoint).toBe('/api/experiments/nonexistent');
            }
        });

        it('should handle 204 No Content', async () => {
            fetchMock.mockResolvedValueOnce({
                ok: true,
                status: 204,
            });

            const result = await api.stop();
            expect(result).toBeNull();
        });
    });

    describe('Retries', () => {
        it('should retry on network error', async () => {
            fetchMock
                .mockRejectedValueOnce(new Error('Network error'))
                .mockResolvedValueOnce({
                    ok: true,
                    status: 200,
                    json: async () => ({ status: 'ok' }),
                });

            const result = await api.getHealth();

            expect(fetchMock).toHaveBeenCalledTimes(2);
            expect(result).toEqual({ status: 'ok' });
        });

        it('should not retry on 4xx errors', async () => {
            fetchMock.mockResolvedValue({
                ok: false,
                status: 400,
                statusText: 'Bad Request',
                text: async () => 'Bad request',
            });

            await expect(api.getHealth()).rejects.toThrow(APIError);
            expect(fetchMock).toHaveBeenCalledTimes(1);
        });

        it('should respect max retries', async () => {
            fetchMock.mockRejectedValue(new Error('Network error'));

            api = new APIClient('http://localhost:8082', { retries: 2 });

            await expect(api.getHealth()).rejects.toThrow('Network error');
            expect(fetchMock).toHaveBeenCalledTimes(2);
        });
    });

    describe('Timeout', () => {
        it('should abort request after timeout', async () => {
            fetchMock.mockImplementation(() => new Promise(() => {})); // Never resolves

            api = new APIClient('http://localhost:8082', { timeout: 1000 });

            const promise = api.getHealth();
            vi.advanceTimersByTime(1000);

            await expect(promise).rejects.toThrow();
        });
    });

    describe('Simulation Control', () => {
        it('should pause simulation', async () => {
            fetchMock.mockResolvedValue({
                ok: true,
                status: 200,
                json: async () => ({ paused: true }),
            });

            const result = await api.pause();
            expect(result.paused).toBe(true);
        });

        it('should resume simulation', async () => {
            fetchMock.mockResolvedValue({
                ok: true,
                status: 200,
                json: async () => ({ paused: false }),
            });

            const result = await api.resume();
            expect(result.paused).toBe(false);
        });

        it('should set speed', async () => {
            fetchMock.mockResolvedValue({
                ok: true,
                status: 200,
                json: async () => ({ speed: 2.0 }),
            });

            const result = await api.setSpeed(2.0);
            expect(result.speed).toBe(2.0);
        });

        it('should get status', async () => {
            fetchMock.mockResolvedValue({
                ok: true,
                status: 200,
                json: async () => ({ status: 'running', tick: 42 }),
            });

            const result = await api.getStatus();
            expect(result.status).toBe('running');
            expect(result.tick).toBe(42);
        });
    });

    describe('Neural Data', () => {
        it('should get neural data for agent', async () => {
            fetchMock.mockResolvedValue({
                ok: true,
                status: 200,
                json: async () => ({ agent_id: 123, regions: [] }),
            });

            const result = await api.getNeuralData(123);
            expect(result.agent_id).toBe(123);
        });

        it('should get population neural stats', async () => {
            fetchMock.mockResolvedValue({
                ok: true,
                status: 200,
                json: async () => ({ population_size: 100 }),
            });

            const result = await api.getNeuralPopulation();
            expect(result.population_size).toBe(100);
        });
    });

    describe('Experiments', () => {
        it('should list experiments', async () => {
            fetchMock.mockResolvedValue({
                ok: true,
                status: 200,
                json: async () => [{ id: 'exp1' }, { id: 'exp2' }],
            });

            const result = await api.listExperiments();
            expect(result).toHaveLength(2);
        });

        it('should launch experiment', async () => {
            fetchMock.mockResolvedValue({
                ok: true,
                status: 200,
                json: async () => ({ run_id: 'run123' }),
            });

            const result = await api.launchExperiment('exp1', { param: 'value' });
            expect(result.run_id).toBe('run123');
        });
    });

    describe('URL Handling', () => {
        it('should handle trailing slash in base URL', () => {
            api = new APIClient('http://localhost:8082/');
            expect(api.baseUrl).toBe('http://localhost:8082');
        });
    });
});
