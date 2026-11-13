/**
 * TypeScript Type Definitions for Simulator Modules
 */

// ConnectionManager
export interface ConnectionManagerConfig {
    maxReconnectAttempts?: number;
    reconnectDelays?: number[];
    frameTimeoutThreshold?: number;
    frameFlowCheckInterval?: number;
    connectionTimeout?: number;
}

export interface ConnectionState {
    connecting: 'connecting';
    connected: 'connected';
    disconnected: 'disconnected';
    failed: 'failed';
}

export declare class ConnectionManager extends EventTarget {
    constructor(apiUrl: string, options?: ConnectionManagerConfig);
    getState(): string;
    isConnected(): boolean;
    connect(): void;
    disconnect(): void;
    destroy(): void;
}

// StateStore
export interface State {
    simulation: {
        status: 'stopped' | 'running' | 'paused' | 'disconnected' | 'failed';
        currentTick: number;
        speed: number;
        isLoading: boolean;
    };
    frame: {
        current: Frame | null;
        previous: Frame | null;
    };
    agents: {
        all: Agent[];
        selected: number | null;
        count: number;
        aliveCount: number;
    };
    layers: Record<string, boolean>;
    layerOpacity: Record<string, number>;
    filters: {
        trait: string;
        status: string;
        relativeDistanceThreshold: number;
    };
}

export declare class StateStore extends EventTarget {
    constructor(initialState?: Partial<State>);
    get(path?: string): any;
    set(path: string, value: any): StateStore;
    subscribe(path: string, callback: (value: any, oldValue: any, path: string) => void): () => void;
    batch(fn: () => void): void;
    toggle(path: string): boolean;
    reset(path?: string): void;
}

export declare const store: StateStore;

// APIClient
export interface APIConfig {
    timeout?: number;
    retries?: number;
    retryDelay?: number;
}

export declare class APIError extends Error {
    status: number;
    response: any;
    endpoint: string;
}

export declare class APIClient {
    constructor(baseUrl: string, options?: APIConfig);
    setSpeed(factor: number): Promise<any>;
    getSpeed(): Promise<any>;
    pause(): Promise<any>;
    resume(): Promise<any>;
    stop(): Promise<any>;
    getStatus(): Promise<any>;
    getHealth(): Promise<any>;
    getNeuralData(agentId: number): Promise<any>;
    listExperiments(): Promise<any[]>;
    launchExperiment(expId: string, params: object): Promise<any>;
}

// DebugSystem
export interface DebugConfig {
    enabled?: boolean;
    maxFrameHistory?: number;
    maxAgentHistory?: number;
    frozenAgentThreshold?: number;
}

export interface FrameReceipt {
    tick: number;
    timestamp: number;
    agentCount: number;
    aliveCount: number;
}

export interface DebugStats {
    frames: {
        received: number;
        rendered: number;
        avgInterval: number;
        history: FrameReceipt[];
    };
    movement: {
        avgMovement: number;
        frozenAgents: Array<{ agentId: number; frozenTicks: number }>;
    };
    agents: {
        tracked: number;
    };
}

export declare class DebugSystem extends EventTarget {
    constructor(options?: DebugConfig);
    isEnabled(): boolean;
    setEnabled(enabled: boolean): void;
    log(category: string, message: string, data?: any): void;
    recordFrameReceipt(frame: Frame): void;
    recordAgentPosition(agent: Agent, tick: number): void;
    getStats(): DebugStats;
    runDiagnostics(): object;
    cleanupDeadAgents(currentAgents: Agent[], currentTick: number, threshold?: number): void;
    clear(): void;
}

// EventLog
export interface EventLogConfig {
    maxEvents?: number;
    batchInterval?: number;
}

export interface LogEvent {
    timestamp: string;
    message: string;
    type: 'info' | 'success' | 'warning' | 'danger' | 'event';
    agentId: number | null;
}

export declare class EventLog extends EventTarget {
    constructor(containerId: string, options?: EventLogConfig);
    log(message: string, type?: string, agentId?: number | null, category?: string): void;
    flushBatch(): void;
    getEvents(): LogEvent[];
    getEventsByType(type: string): LogEvent[];
    getEventsForAgent(agentId: number): LogEvent[];
    search(query: string): LogEvent[];
    clear(): void;
    exportToJSON(): string;
}

// GridCache
export interface GridCacheConfig {
    minorOpacity?: number;
    majorOpacity?: number;
    majorStep?: number;
    color?: string;
}

export interface GridCacheStats {
    isValid: boolean;
    cacheSize: number;
    maxCacheSize: number;
    currentParams: object | null;
    cachedKeys: string[];
}

export declare class GridCache {
    constructor();
    isCacheValid(width: number, height: number, scale: number): boolean;
    build(width: number, height: number, scale: number, options?: GridCacheConfig): HTMLCanvasElement;
    draw(ctx: CanvasRenderingContext2D, offsetX: number, offsetY: number, width?: number, height?: number): void;
    invalidate(): void;
    clear(): void;
    getStats(): GridCacheStats;
    estimateMemoryUsage(): number;
}

// PerformanceMonitor
export interface PerformanceMonitorConfig {
    maxSamples?: number;
    updateInterval?: number;
}

export interface FrameTimeStats {
    min: number;
    max: number;
    avg: number;
    median: number;
}

export interface PerformanceStats {
    fps: number;
    frameTime: FrameTimeStats;
    memory: object | null;
    rating: 'excellent' | 'good' | 'poor' | 'critical';
    frameCount: number;
    uptime: number;
    samples: number;
}

export declare class PerformanceMonitor extends EventTarget {
    constructor(options?: PerformanceMonitorConfig);
    start(): void;
    stop(): void;
    recordFrame(): void;
    getFPS(): number;
    getFrameTimeStats(): FrameTimeStats;
    getStats(): PerformanceStats;
    getRating(): string;
    measure<T>(name: string, fn: () => T): T;
    measureAsync<T>(name: string, fn: () => Promise<T>): Promise<T>;
    reset(): void;
}

// Common Types
export interface Agent {
    agent_id: number;
    x: number;
    y: number;
    energy: number;
    alive: boolean;
    docas?: number[];
    genome_gains?: number[];
    age?: number;
    parent1_id?: number | null;
    parent2_id?: number | null;
}

export interface Frame {
    tick: number;
    agents: Agent[];
    stats?: {
        alive_agents?: number;
        total_agents?: number;
    };
    status?: string;
    agentMap?: Map<number, Agent>;
}

// SimulatorViewer
export interface ViewerConfig {
    apiUrl?: string;
    debug?: boolean;
    store?: StateStore;
}

export declare class SimulatorViewer {
    constructor(canvasId?: string, options?: ViewerConfig);
    init(): boolean;
    connect(): void;
    disconnect(): void;
    isConnected(): boolean;
    setLayer(layerName: string, visible: boolean): void;
    setFilter(filterPath: string, value: any): void;
    resetFilters(): void;
    setSpeed(factor: number): Promise<void>;
    pause(): Promise<void>;
    resume(): Promise<void>;
    stop(): Promise<void>;
    getCurrentTick(): number;
    getStats(): object;
    selectAgent(agentId: number): void;
    deselectAgent(): void;
    render(): void;
    runDiagnostics(): object;
    logEvent(message: string, type?: string, agentId?: number): void;
    exportState(): object;
    importState(state: object): void;
    destroy(): void;
}

export declare function getViewer(canvasId?: string): SimulatorViewer;
export declare function init(canvasId?: string): boolean;
