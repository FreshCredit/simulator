/**
 * Agent Society Simulator - Viewer Module
 *
 * SIM-002: UI reads live stream from sim-service - no sim-core imports
 * SIM-003: Simulation agents, NOT FreshCredit AI agents
 *
 * This module is a RUNTIME VISUALIZATION LAYER only. Per SIM-002 compliance:
 * - Reads simulation state via HTTP API (Server-Sent Events)
 * - Visual overlays are read-only with respect to simulation state
 * - Layer toggles affect rendering only, not simulation logic
 *
 * Frame Schema (from sim-service/src/live.rs):
 * {tick, agents: [{agent_id, x, y, energy, docas: [D,O,C,A,S], genome_gains: [D,O,C,A,S],
 *   alive, has_identity, group_id, age, generation, parent1_id, parent2_id}], stats}
 */

window.SIM_SERVICE_URL = window.SIM_SERVICE_URL || document.body?.dataset?.simServiceUrl || '';
if (new URLSearchParams(window.location.search).get('embedded') === '1') {
    document.documentElement.classList.add('sim-embedded');
    document.addEventListener('DOMContentLoaded', function() {
        document.body.classList.add('sim-embedded');
    });
}

// SIM-002: Live stream viewer - viewer.js
const SimulatorViewer = (function() {
    'use strict';

    // State
    let canvas = null;
    let ctx = null;
    let currentFrame = null;
    let previousFrame = null;  // For motion interpolation
    let currentTick = 0;
    let gridWidth = 64;
    let gridHeight = 64;
    let scale = 1;
    let offsetX = 0;
    let offsetY = 0;
    let eventSource = null;
    let selectedAgentId = null;
    let frameInterpStartTime = 0;  // For motion interpolation
    let frameDurationMs = 100;     // ms, matches server tick rate dynamically
    let isSimulationPaused = false;
    let rafId = null;
    let isRenderLoopActive = false;
    const renderPerf = {
        lastRenderMs: 0,
        p99RenderMs: 0,
        renderTimes: [],
        drawCallsLastFrame: 0,
        fps: 60
    };
    
    // CRITICAL FIX: Reconnection with exponential backoff
    let reconnectAttempts = 0;
    const RECONNECT_DELAYS = [1000, 2000, 5000, 10000, 30000]; // Exponential backoff delays
    const MAX_RECONNECT_ATTEMPTS = 50; // Prevent infinite reconnection loops
    
    /**
     * Schedule reconnection with exponential backoff
     * Prevents connection spam and gives server time to recover
     */
    function scheduleReconnect(reason) {
        if (reconnectAttempts >= MAX_RECONNECT_ATTEMPTS) {
            debugLog('ERROR', `Max reconnection attempts (${MAX_RECONNECT_ATTEMPTS}) reached. Giving up.`);
            updateConnectionStatus('error');
            return;
        }
        
        const delay = RECONNECT_DELAYS[Math.min(reconnectAttempts, RECONNECT_DELAYS.length - 1)];
        reconnectAttempts++;
        
        debugLog('CONN', `Scheduling reconnect (attempt ${reconnectAttempts}/${MAX_RECONNECT_ATTEMPTS}) in ${delay}ms due to: ${reason}`);
        
        setTimeout(() => {
            debugLog('CONN', `Executing reconnect attempt ${reconnectAttempts}...`);
            connect();
        }, delay);
    }
    
    // Cached resources to prevent terrain flashing (resources sent every 10 ticks)
    let cachedResources = null;
    let cachedTerrain = null;
    let cachedWeather = null;
    
    // Activity Log tracking
    const MAX_EVENTS = 100;
    let activityLog = [];
    let lastAgentCount = 0;
    let lastTick = 0;
    let simulationStarted = false;
    let isUserScrolledUp = false;  // Track if user has scrolled up to read
    
    // Event batching system
    const BATCH_INTERVAL = 3000; // 3 seconds
    let eventBatch = {
        reproductions: 0,
        gamesPlayed: 0,
        groupFormations: 0,
        interactions: 0,
        otherEvents: []
    };
    let lastBatchTime = 0;
    
    // CRITICAL FIX: Frame flow monitoring
    let frameFlowMonitor = null;
    
    // ============================================================================
    // DEBUG LOGGING SYSTEM
    // ============================================================================
    
    /// Debug state for comprehensive diagnostics
    const debugState = {
        frameHistory: [],           // Last N frames received
        agentPositionHistory: {},   // agent_id -> array of {tick, x, y}
        connectionEvents: [],       // Connection status changes
        frameStats: {
            framesReceived: 0,
            framesRendered: 0,
            lastFrameTime: 0,
            avgFrameInterval: 0,
            frameIntervals: []
        },
        movementStats: {
            totalMovement: 0,
            samples: 0,
            frozenAgentTicks: {}      // agent_id -> consecutive ticks without movement
        },
        enabled: true
    };
    
    const MAX_FRAME_HISTORY = 50;
    const MAX_AGENT_HISTORY = 100;
    
    /// Structured logging with timestamp
    function debugLog(category, message, data = null) {
        const timestamp = new Date().toISOString();
        const prefix = `[Viewer][${category}]`;
        
        if (data) {
            console.log(`${prefix} ${message}`, data);
        } else {
            console.log(`${prefix} ${message}`);
        }
        
        // Also log to event log for critical events
        if (category === 'ERROR' || category === 'FROZEN' || category === 'CONN') {
            logEvent(`${category}: ${message}`, category === 'ERROR' ? 'danger' : 'warning');
        }
    }
    
    /// Show a temporary toast notification
    function showToast(message, duration = 3000) {
        // Create toast element
        const toast = document.createElement('div');
        toast.style.cssText = `
            position: fixed;
            bottom: 20px;
            left: 50%;
            transform: translateX(-50%);
            background: hsl(var(--card));
            color: hsl(var(--card-foreground));
            padding: 12px 24px;
            border-radius: 8px;
            border: 1px solid hsl(var(--border));
            box-shadow: 0 4px 12px rgba(0,0,0,0.3);
            z-index: 9999;
            font-size: 14px;
            max-width: 400px;
            text-align: center;
            opacity: 0;
            transition: opacity 0.3s ease;
        `;
        toast.textContent = message;
        document.body.appendChild(toast);
        
        // Fade in
        requestAnimationFrame(() => {
            toast.style.opacity = '1';
        });
        
        // Remove after duration
        setTimeout(() => {
            toast.style.opacity = '0';
            setTimeout(() => {
                document.body.removeChild(toast);
            }, 300);
        }, duration);
    }
    
    /// Record frame receipt for debugging
    function recordFrameReceipt(frame) {
        if (!frame) return;
        
        const now = performance.now();
        const receipt = {
            tick: frame.tick,
            timestamp: now,
            agentCount: frame.agents ? frame.agents.length : 0,
            aliveCount: frame.stats ? frame.stats.alive_agents : 0
        };
        
        debugState.frameHistory.push(receipt);
        if (debugState.frameHistory.length > MAX_FRAME_HISTORY) {
            debugState.frameHistory.shift();
        }
        
        // Calculate frame interval
        if (debugState.frameStats.lastFrameTime > 0) {
            const interval = now - debugState.frameStats.lastFrameTime;
            debugState.frameStats.frameIntervals.push(interval);
            if (debugState.frameStats.frameIntervals.length > 10) {
                debugState.frameStats.frameIntervals.shift();
            }
            debugState.frameStats.avgFrameInterval = 
                debugState.frameStats.frameIntervals.reduce((a, b) => a + b, 0) / 
                debugState.frameStats.frameIntervals.length;
        }
        debugState.frameStats.lastFrameTime = now;
        debugState.frameStats.framesReceived++;
        
        // Record agent positions for movement tracking
        if (frame.agents) {
            frame.agents.forEach(agent => {
                if (!debugState.agentPositionHistory[agent.agent_id]) {
                    debugState.agentPositionHistory[agent.agent_id] = [];
                }
                
                const history = debugState.agentPositionHistory[agent.agent_id];
                const prevPos = history.length > 0 ? history[history.length - 1] : null;
                
                history.push({
                    tick: frame.tick,
                    x: agent.x,
                    y: agent.y,
                    energy: agent.energy,
                    alive: agent.alive,
                    timestamp: now
                });
                
                // Keep only recent history
                if (history.length > MAX_AGENT_HISTORY) {
                    history.shift();
                }
                
                // Check for frozen agents
                if (prevPos && agent.alive) {
                    const dx = agent.x - prevPos.x;
                    const dy = agent.y - prevPos.y;
                    const moved = Math.abs(dx) > 0.001 || Math.abs(dy) > 0.001;
                    
                    if (!moved) {
                        debugState.movementStats.frozenAgentTicks[agent.agent_id] = 
                            (debugState.movementStats.frozenAgentTicks[agent.agent_id] || 0) + 1;
                        
                        // Warn if agent hasn't moved for 20+ ticks
                        if (debugState.movementStats.frozenAgentTicks[agent.agent_id] === 20) {
                            debugLog('FROZEN', `Agent ${agent.agent_id} hasn't moved for 20 ticks`, {
                                pos: `(${agent.x.toFixed(2)}, ${agent.y.toFixed(2)})`,
                                energy: agent.energy,
                                age: agent.age
                            });
                        }
                    } else {
                        debugState.movementStats.frozenAgentTicks[agent.agent_id] = 0;
                        debugState.movementStats.totalMovement += Math.sqrt(dx*dx + dy*dy);
                        debugState.movementStats.samples++;
                    }
                }
            });
        }
        
        // Log frame receipt (throttled)
        if (frame.tick <= 5 || frame.tick % 50 === 0) {
            debugLog('Frame', `Received tick=${frame.tick} agents=${receipt.agentCount} alive=${receipt.aliveCount}`, {
                avgInterval: debugState.frameStats.avgFrameInterval.toFixed(1) + 'ms'
            });
        }
    }
    
    /// Record connection event
    function recordConnectionEvent(event, details = null) {
        const entry = {
            timestamp: performance.now(),
            event,
            details
        };
        debugState.connectionEvents.push(entry);
        if (debugState.connectionEvents.length > 20) {
            debugState.connectionEvents.shift();
        }
        debugLog('CONN', event, details);
    }
    
    // ============================================================================
    // DEBUG API (exposed to window)
    // ============================================================================
    
    /// Get current debug state
    window.debugSimulation = function() {
        const state = {
            currentTick,
            frameHistory: debugState.frameHistory,
            connectionEvents: debugState.connectionEvents,
            stats: {
                framesReceived: debugState.frameStats.framesReceived,
                avgFrameInterval: debugState.frameStats.avgFrameInterval,
                avgMovement: debugState.movementStats.samples > 0 ? 
                    (debugState.movementStats.totalMovement / debugState.movementStats.samples).toFixed(4) : 0,
                frozenAgents: Object.entries(debugState.movementStats.frozenAgentTicks)
                    .filter(([_, ticks]) => ticks > 10)
                    .map(([id, ticks]) => ({ agentId: parseInt(id), frozenTicks: ticks }))
            },
            connectionStatus: eventSource ? {
                readyState: eventSource.readyState,
                url: eventSource.url
            } : null
        };
        console.log('[Viewer][debugSimulation]', state);
        return state;
    };
    
    /// Get movement history for a specific agent
    window.getAgentHistory = function(agentId) {
        const history = debugState.agentPositionHistory[agentId];
        if (!history) {
            console.log(`[Viewer][getAgentHistory] No history found for agent ${agentId}`);
            return null;
        }
        
        // Calculate movement statistics
        let totalDistance = 0;
        let movementPoints = [];
        for (let i = 1; i < history.length; i++) {
            const dx = history[i].x - history[i-1].x;
            const dy = history[i].y - history[i-1].y;
            const dist = Math.sqrt(dx*dx + dy*dy);
            totalDistance += dist;
            movementPoints.push({
                tick: history[i].tick,
                from: `(${history[i-1].x.toFixed(2)}, ${history[i-1].y.toFixed(2)})`,
                to: `(${history[i].x.toFixed(2)}, ${history[i].y.toFixed(2)})`,
                distance: dist.toFixed(4),
                energy: history[i].energy
            });
        }
        
        const result = {
            agentId,
            totalPositions: history.length,
            totalDistance: totalDistance.toFixed(4),
            currentPosition: history[history.length - 1],
            movementHistory: movementPoints.slice(-20) // Last 20 movements
        };
        console.log(`[Viewer][getAgentHistory] Agent ${agentId}`, result);
        return result;
    };
    
    /// Check frame flow health
    window.checkFrameFlow = function() {
        const now = performance.now();
        const lastFrame = debugState.frameHistory[debugState.frameHistory.length - 1];
        
        if (!lastFrame) {
            return { healthy: false, issue: 'No frames received yet' };
        }
        
        const timeSinceLastFrame = now - lastFrame.timestamp;
        const healthy = timeSinceLastFrame < 1000; // Consider stale if > 1 second
        
        const report = {
            healthy,
            lastTick: lastFrame.tick,
            timeSinceLastFrame: Math.round(timeSinceLastFrame) + 'ms',
            avgFrameInterval: Math.round(debugState.frameStats.avgFrameInterval) + 'ms',
            framesInHistory: debugState.frameHistory.length,
            connectionState: eventSource ? ['CONNECTING', 'OPEN', 'CLOSED'][eventSource.readyState] : 'NO_SOURCE',
            issue: healthy ? null : `No frame received for ${Math.round(timeSinceLastFrame)}ms`
        };
        
        console.log('[Viewer][checkFrameFlow]', report);
        return report;
    };
    
    /// Run full diagnostic check
    window.runDiagnostics = function() {
        console.log('%c[Viewer] Running Full Diagnostics...', 'color: cyan; font-weight: bold');
        
        const results = {
            timestamp: new Date().toISOString(),
            frameFlow: window.checkFrameFlow(),
            simulation: window.debugSimulation(),
            sampleAgent: null
        };
        
        // Get history for first available agent
        const agentIds = Object.keys(debugState.agentPositionHistory);
        if (agentIds.length > 0) {
            results.sampleAgent = window.getAgentHistory(parseInt(agentIds[0]));
        }
        
        console.log('%c[Viewer] Diagnostic Results:', 'color: cyan; font-weight: bold', results);
        return results;
    };
    
    // Schedule periodic diagnostics
    setInterval(() => {
        if (simulationStarted && debugState.frameStats.framesReceived > 0) {
            const health = window.checkFrameFlow();
            if (!health.healthy) {
                debugLog('ERROR', `Frame flow issue: ${health.issue}`);
            }
        }
    }, 5000);
    
    // CRITICAL FIX: Periodic health check to detect backend issues early
    setInterval(async () => {
        if (!simulationStarted) return;
        
        const apiUrl = window.SIM_SERVICE_URL || '';
        try {
            const response = await fetch(`${apiUrl}/health`, { 
                method: 'GET',
                // Short timeout for health check
                signal: AbortSignal.timeout(5000)
            });
            if (response.ok) {
                const data = await response.json();
                debugLog('Health', 'Backend health check OK', data);
            } else {
                debugLog('ERROR', `Health check failed: HTTP ${response.status}`);
            }
        } catch (err) {
            debugLog('ERROR', `Health check error: ${err.message}`);
        }
    }, 30000); // Check every 30 seconds
    
    // Setup event log scroll listener for smart auto-scroll
    setupActivityLogScrollListener();
    
    // Setup activity log fullscreen toggle
    setupActivityLogFullscreen();
    
    debugLog('Init', 'Debug logging system initialized');
    
    /**
     * Setup activity log fullscreen toggle functionality
     */
    function setupActivityLogFullscreen() {
        const fullscreenBtn = document.getElementById('activity-log-fullscreen-btn');
        const activityLogPanel = document.getElementById('activity-log-panel');
        
        if (!fullscreenBtn || !activityLogPanel) {
            console.warn('[Viewer] Fullscreen button or activity log panel not found');
            return;
        }
        
        let scrollPositionBeforeFullscreen = 0;
        
        // Toggle fullscreen on button click
        fullscreenBtn.addEventListener('click', () => {
            const isFullscreen = activityLogPanel.classList.contains('activity-log-fullscreen');
            
            if (isFullscreen) {
                // Exit fullscreen
                activityLogPanel.classList.remove('activity-log-fullscreen');
                fullscreenBtn.classList.remove('active');
                fullscreenBtn.title = 'Toggle full-screen';
                
                // Restore scroll position
                const content = document.getElementById('activity-log-content');
                if (content) {
                    content.scrollTop = scrollPositionBeforeFullscreen;
                }
                
                debugLog('UI', 'Activity log exited fullscreen mode');
            } else {
                // Enter fullscreen
                const content = document.getElementById('activity-log-content');
                if (content) {
                    scrollPositionBeforeFullscreen = content.scrollTop;
                }
                
                activityLogPanel.classList.add('activity-log-fullscreen');
                fullscreenBtn.classList.add('active');
                fullscreenBtn.title = 'Exit full-screen';
                
                // Scroll to bottom if user wasn't scrolled up
                if (!isUserScrolledUp) {
                    scrollToBottom();
                }
                
                debugLog('UI', 'Activity log entered fullscreen mode');
            }
        });
        
        // Handle Escape key to exit fullscreen
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' && activityLogPanel.classList.contains('activity-log-fullscreen')) {
                activityLogPanel.classList.remove('activity-log-fullscreen');
                fullscreenBtn.classList.remove('active');
                fullscreenBtn.title = 'Toggle full-screen';
                
                // Restore scroll position
                const content = document.getElementById('activity-log-content');
                if (content) {
                    content.scrollTop = scrollPositionBeforeFullscreen;
                }
                
                debugLog('UI', 'Activity log exited fullscreen mode (Escape key)');
            }
        });
        
        debugLog('Init', 'Activity log fullscreen functionality initialized');
    }

    /**
     * VISUALIZATION LAYERS (SIM-002 Compliant)
     * 
     * These are RUNTIME visualization controls only. They can be toggled on/off
     * WHILE the simulation is running without affecting simulation state.
     * 
     * Per SIM-002: These are purely client-side rendering options - read-only
     * with respect to simulation state. They serve dual purposes:
     * 1. Real-time monitoring during simulation execution
     * 2. Post-hoc visualization for generating analysis figures after a run completes
     * 
     * Note: Grid lines are always visible (not toggleable) per UI refactor spec.
     * 
     * Phase 2 Addition: Terrain overlay for resource density visualization
     * 
     * Updated: Only Agents is ON by default, all others OFF. Opacity controls added.
     */
    let layers = {
        agents: true,       // Agent visibility (ON by default)
        terrain: false,     // Terrain elevation heatmap (OFF by default)
        weather: false,     // Weather overlay (OFF by default)
        resources: false,   // Resource distribution (OFF by default)
        harvesting: false,  // Active harvesting behavior (OFF by default)
        docas: false,       // Emotional State overlay (OFF by default)
        energy: false,      // Energy ring overlay (OFF by default)
        groups: false,      // Group membership indicator (OFF by default)
        identity: false,    // Identity verification badge (OFF by default)
        network: false,     // Trust Network edges (OFF by default)
        relative: false,    // Relative edges (OFF by default)
        reputation: false   // Reputation ring (OFF by default)
    };
    
    /**
     * LAYER OPACITY (0-1)
     * Controls transparency for each visualization layer
     * Default: 1.0 (fully opaque)
     */
    let layerOpacity = {
        agents: 1.0,
        terrain: 1.0,
        weather: 0.5,
        resources: 1.0,
        harvesting: 1.0,
        docas: 1.0,
        energy: 1.0,
        groups: 1.0,
        identity: 1.0,
        network: 1.0,
        relative: 1.0,
        reputation: 1.0
    };

    /**
     * AGENT FILTERS (SIM-002 Compliant)
     * 
     * These are client-side visualization filters for subset selection.
     * They affect which agents are rendered but do NOT affect simulation state.
     * Per SIM-002: Visual filters are read-only with respect to simulation.
     */
    let filters = {
        // Dominant trait filter: 'all' | 'd' | 'o' | 'c' | 'a' | 's'
        trait: 'all',
        // Status filter: 'alive' | 'all'
        status: 'alive',
        // Relative distance threshold (grid units) - 0 = show all
        relativeDistanceThreshold: 10
    };

    /**
     * Initialize the viewer with canvas element
     */
    function init(canvasId) {
        console.log('[Viewer] init() called with canvasId:', canvasId);
        
        canvas = document.getElementById(canvasId);
        if (!canvas) {
            console.error('[Viewer] Canvas not found:', canvasId);
            return false;
        }
        console.log('[Viewer] Canvas element found:', canvas);
        
        ctx = canvas.getContext('2d');
        if (!ctx) {
            console.error('[Viewer] Failed to get 2D context');
            return false;
        }
        console.log('[Viewer] 2D context obtained');

        resizeCanvas();
        window.addEventListener('resize', resizeCanvas);

        if (typeof ResizeObserver !== 'undefined' && canvas.parentElement) {
            const ro = new ResizeObserver(() => {
                resizeCanvas();
            });
            ro.observe(canvas.parentElement);
        }

        // Add click handler for agent inspection
        canvas.addEventListener('click', handleCanvasClick);

        // Tab visibility management: pause RAF loop when backgrounded to prevent orphaned cycles
        document.addEventListener('visibilitychange', () => {
            if (document.hidden) {
                stopRenderLoop();
            } else if (!isSimulationPaused && currentFrame && eventSource && eventSource.readyState === EventSource.OPEN) {
                frameInterpStartTime = performance.now();
                startRenderLoop();
            }
        });

        console.log('[Viewer] Initialized successfully with canvas:', canvas.width, 'x', canvas.height);
        
        // FIXED: Listen for simulation start event from controls
        window.addEventListener('simulationStarted', (e) => {
            console.log('[Viewer] Received simulationStarted event:', e.detail);
            console.log('[Viewer] Auto-connecting to live stream...');
            
            // Mark simulation as started
            simulationStarted = true;
            
            // Dynamically set grid size from event parameters or DOM inputs
            const gWidth = e.detail?.params?.grid_width || parseInt(document.getElementById('geo-grid-width')?.value, 10) || 100;
            const gHeight = e.detail?.params?.grid_height || parseInt(document.getElementById('geo-grid-height')?.value, 10) || 100;
            setGridConfig({ width: gWidth, height: gHeight });
            console.log(`[Viewer] Grid size set to ${gWidth}x${gHeight}`);
            
            // Update overlay message
            const overlayMsg = document.getElementById('overlay-message');
            if (overlayMsg) {
                overlayMsg.textContent = 'Connecting to live simulation...';
            }
            
            // Show mechanism layers sidebar
            const mechanismLayersSection = document.getElementById('mechanism-layers-section');
            if (mechanismLayersSection) {
                mechanismLayersSection.classList.remove('hidden');
                console.log('[Viewer] Mechanism layers section shown');
            }
            
            // Show visualization layers section
            const vizLayersSection = document.getElementById('visualization-layers-section');
            if (vizLayersSection) {
                vizLayersSection.style.display = 'block';
                vizLayersSection.classList.remove('hidden');
                console.log('[Viewer] Visualization layers section shown');
            }
            
            // Show stats section
            const statsSection = document.getElementById('stats-section');
            if (statsSection) {
                statsSection.classList.remove('hidden');
                console.log('[Viewer] Stats section shown');
            }
            
            // Show event log panel
            const activityLogPanel = document.getElementById('activity-log-panel');
            if (activityLogPanel) {
                activityLogPanel.classList.remove('hidden');
                console.log('[Viewer] Event log panel shown');
            }

            // Hide export button when new simulation starts
            hideExportButton();

            // Reset event log for new simulation
            activityLog = [];
            isUserScrolledUp = false;
            eventBatch = { reproductions: 0, gamesPlayed: 0, groupFormations: 0, interactions: 0, otherEvents: [] };
            lastBatchTime = Date.now();
            logEvent('Simulation started', 'success');
            
            // Phase 5: Transition pixelated loader to spinning mode with grayscale
            if (window.PixelatedLoader) {
                window.PixelatedLoader.transitionToGrayscale(() => {
                    console.log('[PixelatedLoader] Transitioned to grayscale mode');
                });
            }
            
            // Hide game over screen if visible
            hideGameOver();
            
            // Connect to the stream
            connect();
        });
        
        return true;
    }

    /**
     * Handle canvas click to detect agent selection
     * Phase 4: Uses spatial hash for O(1) lookups instead of O(n) scan
     */
    function handleCanvasClick(event) {
        if (!currentFrame || !currentFrame.agents) return;

        const rect = canvas.getBoundingClientRect();
        const clickX = event.clientX - rect.left;
        const clickY = event.clientY - rect.top;

        // Convert to grid coordinates (accounting for zoom and pan)
        const gridX = (clickX - offsetX) / (scale * renderConfig.zoom) - renderConfig.panX;
        const gridY = (clickY - offsetY) / (scale * renderConfig.zoom) - renderConfig.panY;

        // Phase 4: Use spatial hash for O(1) lookup
        const cellX = Math.floor(gridX);
        const cellY = Math.floor(gridY);
        const key = `${cellX},${cellY}`;
        
        // Check clicked cell and neighboring cells (agents may overlap cell boundaries)
        const candidates = [];
        for (let dx = -1; dx <= 1; dx++) {
            for (let dy = -1; dy <= 1; dy++) {
                const neighborKey = `${cellX + dx},${cellY + dy}`;
                const agentIds = spatialHash.get(neighborKey);
                if (agentIds) {
                    candidates.push(...agentIds);
                }
            }
        }

        // Find closest agent among candidates
        // INCREASED: 2x larger click hitbox for better clickability (Phase 1 fix)
        const clickRadius = Math.max(16, scale * 1.2) / scale;
        let closestAgent = null;
        let closestDistance = Infinity;

        // P0 FIX: Use agentMap for O(1) lookup instead of O(n) find()
        for (const agentId of candidates) {
            const agent = currentFrame.agentMap?.get(agentId);
            if (!agent) continue;
            
            const dx = agent.x - gridX;
            const dy = agent.y - gridY;
            const distance = Math.sqrt(dx * dx + dy * dy);

            if (distance <= clickRadius && distance < closestDistance) {
                closestDistance = distance;
                closestAgent = agent;
            }
        }

        if (closestAgent) {
            openAgentModal(closestAgent.agent_id);
        } else if (selectedAgentId !== null) {
            // Clicked empty space - close modal if open
            closeAgentModal();
        }
    }

    /**
     * Open agent inspector modal with agent data
     * BUG FIX: Cache agent data to prevent modal closing on death
     */
    function openAgentModal(agentId) {
        console.log('[Viewer] openAgentModal called for agent:', agentId);
        
        // Try to find agent in current frame first
        let agent = currentFrame.agents.find(a => a.agent_id === agentId);
        
        // If not found, check cached data for deceased agents
        if (!agent) {
            if (cachedAgentData && cachedAgentData.agent_id === agentId) {
                console.log('[Viewer] Using cached data for deceased agent:', agentId);
                agent = cachedAgentData;
            } else {
                console.warn('[Viewer] Agent not found in current frame or cache:', agentId);
                showToast(`Agent #${agentId} not found`);
                return;
            }
        }

        selectedAgentId = agentId;
        cachedAgentData = { ...agent }; // Deep copy to cache
        console.log('[Viewer] Opening inspector for agent:', agentId, agent);

        try {
            // Populate identity section
            const agentIdEl = document.getElementById('inspector-agent-id');
            const ageEl = document.getElementById('inspector-age');
            
            if (!agentIdEl || !ageEl) {
                console.error('[Viewer] Missing inspector elements');
                return;
            }
            
            agentIdEl.textContent = agent.agent_id;
            ageEl.textContent = agent.age;

        const statusEl = document.getElementById('inspector-status');
        statusEl.textContent = agent.alive ? 'Alive' : 'Dead';
        statusEl.className = 'inspector-value ' + (agent.alive ? 'status-alive' : 'status-dead');

        const identityEl = document.getElementById('inspector-identity');
        identityEl.textContent = agent.has_identity ? 'Yes' : 'No';
        identityEl.className = 'inspector-value ' + (agent.has_identity ? 'status-verified' : 'status-unverified');

        // Populate genome segmented bar (heritable DOCAS gains)
        // Each segment width is proportional to that component's gain relative to total
        const genomeBar = document.getElementById('inspector-genome-bar');
        if (genomeBar && agent.genome_gains) {
            const segments = genomeBar.querySelectorAll('.genome-segment');
            const totalGain = agent.genome_gains.reduce((sum, g) => sum + g, 0);
            
            agent.genome_gains.forEach((gain, i) => {
                if (segments[i]) {
                    // Width is proportional to this gain vs total
                    const percentage = totalGain > 0 ? (gain / totalGain) * 100 : 20;
                    segments[i].style.width = `${percentage}%`;
                }
            });
        }

        // Populate DOCAS state bars (transient levels)
        const docasBars = document.querySelectorAll('#inspector-docas-bars .docas-bar-row');
        if (agent.docas) {
            agent.docas.forEach((level, i) => {
                if (docasBars[i]) {
                    const bar = docasBars[i].querySelector('.docas-bar');
                    const value = docasBars[i].querySelector('.docas-value');
                    const percentage = Math.min(100, level * 100);
                    bar.style.width = `${percentage}%`;
                    value.textContent = level.toFixed(3);
                }
            });
        }

        // Populate trust (τ) and reputation
        updateTauDisplay(agent);
        updateReputationDisplay(agent);

        // Populate state section
        document.getElementById('inspector-energy').textContent = agent.energy.toFixed(1);
        document.getElementById('inspector-position').textContent = `(${agent.x.toFixed(1)}, ${agent.y.toFixed(1)})`;
        document.getElementById('inspector-group').textContent =
            (agent.group_id !== null && agent.group_id !== undefined) ? agent.group_id : 'None';

        // Populate Relative section
        document.getElementById('inspector-generation').textContent =
            (agent.generation !== undefined) ? agent.generation : '0';

        const parent1El = document.getElementById('inspector-parent1');
        if (agent.parent1_id !== null && agent.parent1_id !== undefined) {
            parent1El.textContent = `#${agent.parent1_id}`;
            parent1El.classList.add('clickable');
            parent1El.onclick = () => openAgentModal(agent.parent1_id);
        } else {
            parent1El.textContent = '—';
            parent1El.classList.remove('clickable');
            parent1El.onclick = null;
        }

        const parent2El = document.getElementById('inspector-parent2');
        if (agent.parent2_id !== null && agent.parent2_id !== undefined) {
            parent2El.textContent = `#${agent.parent2_id}`;
            parent2El.classList.add('clickable');
            parent2El.onclick = () => openAgentModal(agent.parent2_id);
        } else {
            parent2El.textContent = '—';
            parent2El.classList.remove('clickable');
            parent2El.onclick = null;
        }

        // Populate Demographics section
        const sexEl = document.getElementById('inspector-sex');
        if (agent.sex) {
            sexEl.textContent = agent.sex;
            sexEl.className = 'inspector-value ' + (agent.sex === 'Male' ? 'status-male' : 'status-female');
        } else {
            sexEl.textContent = '—';
            sexEl.className = 'inspector-value';
        }

        // Calculate children and grandchildren from snapshot or lineage
        let childrenCount = (agent.children_count !== undefined) ? agent.children_count : 0;
        let grandchildrenCount = (agent.grandchildren_count !== undefined) ? agent.grandchildren_count : 0;
        
        if (childrenCount === 0 && currentFrame && currentFrame.agents) {
            // Fallback for active frame agents
            const children = currentFrame.agents.filter(a => 
                a.parent1_id === agent.agent_id || a.parent2_id === agent.agent_id
            );
            childrenCount = children.length;
            const childIds = new Set(children.map(c => c.agent_id));
            const grandchildren = currentFrame.agents.filter(a => 
                childIds.has(a.parent1_id) || childIds.has(a.parent2_id)
            );
            grandchildrenCount = grandchildren.length;
        }
        
        document.getElementById('inspector-children').textContent = childrenCount;
        document.getElementById('inspector-grandchildren').textContent = grandchildrenCount;

            // Origin: Initial spawn or Offspring
            const isOffspring = agent.parent1_id !== null && agent.parent1_id !== undefined;
            const originEl = document.getElementById('inspector-origin');
            if (originEl) originEl.textContent = isOffspring ? 'Offspring' : 'Initial Spawn';

            // Plaid Financial Underwriting & Sequestration Wall fields
            const mktScoreEl = document.getElementById('inspector-mkt-score');
            if (mktScoreEl) mktScoreEl.textContent = (agent.financial?.market_trust_score ?? 0.50).toFixed(2);
            const cashflowEl = document.getElementById('inspector-cashflow-buffer');
            if (cashflowEl) cashflowEl.textContent = `${(agent.financial?.cashflow_buffer_days ?? 12.5).toFixed(1)} days`;
            const ontimeEl = document.getElementById('inspector-ontime-rate');
            if (ontimeEl) ontimeEl.textContent = `${Math.round((agent.financial?.on_time_payment_rate ?? 1.0) * 100)}%`;
            const nsfEl = document.getElementById('inspector-nsf-count');
            if (nsfEl) nsfEl.textContent = agent.financial?.nsf_overdraft_count ?? 0;
            const spendVolEl = document.getElementById('inspector-spend-volatility');
            if (spendVolEl) spendVolEl.textContent = (agent.financial?.spend_volatility ?? 0.18).toFixed(2);
            const concordanceEl = document.getElementById('inspector-concordance-r');
            if (concordanceEl) concordanceEl.textContent = (agent.financial?.concordance_mantel_r ?? 0.88).toFixed(2);

            // Wet-Lab & Wearable Biomarkers fields
            const hrvEl = document.getElementById('inspector-hrv-rmssd');
            if (hrvEl) hrvEl.textContent = `${(agent.biomarkers?.hrv_rmssd_ms ?? 48.0).toFixed(1)} ms`;
            const restingHrEl = document.getElementById('inspector-resting-hr');
            if (restingHrEl) restingHrEl.textContent = `${(agent.biomarkers?.resting_hr_bpm ?? 64.0).toFixed(1)} bpm`;
            const glucoseEl = document.getElementById('inspector-glucose');
            if (glucoseEl) glucoseEl.textContent = `${(agent.biomarkers?.glucose_mg_dl ?? 92.0).toFixed(1)} mg/dL`;
            const glucoseTirEl = document.getElementById('inspector-glucose-tir');
            if (glucoseTirEl) glucoseTirEl.textContent = `${(agent.biomarkers?.glucose_tir_pct ?? 88.0).toFixed(1)}%`;
            const sleepEffEl = document.getElementById('inspector-sleep-eff');
            if (sleepEffEl) sleepEffEl.textContent = `${(agent.biomarkers?.sleep_efficiency_pct ?? 86.0).toFixed(1)}%`;
            const sleepDeepEl = document.getElementById('inspector-sleep-deep');
            if (sleepDeepEl) sleepDeepEl.textContent = `${(agent.biomarkers?.sleep_deep_pct ?? 20.0).toFixed(1)}%`;
            const eegThetaEl = document.getElementById('inspector-eeg-theta');
            if (eegThetaEl) eegThetaEl.textContent = `${(agent.biomarkers?.eeg_frontal_theta_uv2 ?? 6.2).toFixed(1)} μV²`;
            const eegFaaEl = document.getElementById('inspector-eeg-faa');
            if (eegFaaEl) {
                const faaVal = agent.biomarkers?.eeg_alpha_asymmetry ?? 0.05;
                eegFaaEl.textContent = `${faaVal >= 0 ? '+' : ''}${faaVal.toFixed(2)}`;
            }
            const wristTempEl = document.getElementById('inspector-wrist-temp');
            if (wristTempEl) wristTempEl.textContent = `${(agent.biomarkers?.sleeping_wrist_temp_c ?? 36.4).toFixed(2)} °C`;
            const fnirsEl = document.getElementById('inspector-fnirs-hbo2');
            if (fnirsEl) fnirsEl.textContent = `${(agent.biomarkers?.fnirs_prefrontal_hbo2_umol ?? 2.5).toFixed(2)} μmol/L`;

            // Show modal
            const modal = document.getElementById('agent-inspector-modal');
            if (modal) {
                modal.classList.remove('hidden');
                console.log('[Viewer] Modal shown successfully');
            } else {
                console.error('[Viewer] Modal element not found!');
            }

            // Initialize 3D brain visualization (optional - don't fail if missing)
            try {
                initBrainVisualization(agent);
            } catch (e) {
                console.warn('[Viewer] Brain visualization failed:', e);
            }

            // Re-render to show selection highlight
            render();
            
        } catch (error) {
            console.error('[Viewer] Error in openAgentModal:', error);
            console.error('[Viewer] Stack trace:', error.stack);
        }
    }

    // Brain visualization instance
    let brainViz = null;
    
    // Track last agent ID for brain viz to detect switches
    let brainVizLastAgentId = null;
    
    // CLICS visualization instance
    let clicsViz = null;

    /**
     * Initialize 3D brain visualization for an agent
     * CRITICAL FIX: Reset activations when switching agents to prevent blending
     */
    function initBrainVisualization(agent) {
        console.log('[Viewer] initBrainVisualization called for agent:', agent.agent_id, 'DOCAS:', agent.docas);
        
        const container = document.getElementById('brain-canvas');
        if (!container) {
            console.error('[Viewer] brain-canvas container not found');
            return;
        }

        // Show loading state
        const loading = document.getElementById('brain-loading');
        if (loading) loading.classList.remove('hidden');

        // CRITICAL FIX: Check if we're switching agents
        const isAgentSwitch = brainVizLastAgentId !== null && brainVizLastAgentId !== agent.agent_id;
        
        // CRITICAL FIX: Use instance reuse pattern - create once, reuse for all agents
        // This prevents WebGL context exhaustion (browser limit is ~8-16 contexts)
        if (!brainViz) {
            console.log('[Viewer] Creating new BrainVisualization instance (will be reused)');
            try {
                brainViz = new BrainVisualization('brain-canvas', {
                    width: container.clientWidth,
                    height: container.clientHeight,
                    enableRotation: true,
                    showLabels: true
                });
                console.log('[Viewer] BrainVisualization instance created successfully');
            } catch (e) {
                console.error('[BrainViz] Failed to initialize:', e);
                if (loading) loading.textContent = 'Failed to load 3D view';
                return;
            }
        } else if (isAgentSwitch) {
            // Agent switch detected - reset activations but DON'T destroy/recreate
            console.log(`[Viewer] Agent switch detected: ${brainVizLastAgentId} -> ${agent.agent_id}, reusing visualization instance`);
            // Reset activations to prevent blending between agents
            brainViz.resetActivations();
        }

        // Update the last agent ID tracker
        brainVizLastAgentId = agent.agent_id;

        // CRITICAL FIX: Always reset before setting new agent to ensure clean state
        // This prevents showing cached/stale data from previous agents
        console.log('[Viewer] Setting current agent to', agent.agent_id);
        brainViz.setCurrentAgent(agent.agent_id);

        // Update with agent's DOCAS data
        if (agent.docas && Array.isArray(agent.docas) && agent.docas.length === 5) {
            console.log('[Viewer] Updating brain viz for agent', agent.agent_id, 'with DOCAS:', agent.docas, 'alive:', agent.alive);
            
            // Pass isAlive status for live activity effect
            // true = immediate update, no interpolation
            brainViz.updateFromDocas({
                levels: agent.docas
            }, true, agent.alive);
            
            if (loading) loading.classList.add('hidden');
        } else {
            console.warn('[Viewer] Agent has no valid DOCAS data:', agent.docas);
        }

        // Fetch detailed neural data from API (non-blocking)
        fetchNeuralData(agent.agent_id);
        
        // Initialize CLICS visualization if lexicon data is available
        initCLICSVisualization(agent);
        
        console.log('[Viewer] initBrainVisualization complete for agent:', agent.agent_id);
    }

    /**
     * Fetch neural brain data from API
     */
    async function fetchNeuralData(agentId) {
        try {
            const response = await fetch(`${window.SIM_SERVICE_URL}/api/neural/agent/${agentId}`);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            
            const data = await response.json();
            if (brainViz && data.regions && data.regions.length > 0) {
                // Update with server-provided region activations
                brainViz.updateFromRegions(data.regions);
            }
        } catch (e) {
            console.warn('[BrainViz] Failed to fetch neural data:', e);
            // Fallback: visualization already shows DOCAS-based activation
        }
    }

    /**
     * Initialize CLICS³ colexification graph visualization
     * Shows the agent's lexicon structure as a force-directed graph
     */
    function initCLICSVisualization(agent) {
        const container = document.getElementById('clics-canvas');
        if (!container) return;

        // Initialize CLICS visualization if not already done
        if (!clicsViz) {
            try {
                clicsViz = new CLICSVisualization('clics-canvas', {
                    width: container.clientWidth || 300,
                    height: container.clientHeight || 250
                });
            } catch (e) {
                console.error('[CLICS] Failed to initialize:', e);
                return;
            }
        }

        // Skip if agent has no lexicon data
        if (!agent.lexicon || !agent.lexicon.concepts || agent.lexicon.concepts.length === 0) {
            clicsViz.showEmptyState();
            return;
        }

        clicsViz.setCurrentAgent(agent.agent_id);
        console.log('[CLICS] Updating for agent', agent.agent_id, 'with lexicon concepts:', agent.lexicon.concepts.length);
        clicsViz.updateFromLexicon(agent.lexicon);
    }

    /**
     * Close agent inspector modal
     * BUG FIX: Clear cached data when closing
     */
    function closeAgentModal() {
        selectedAgentId = null;
        cachedAgentData = null; // Clear cached data
        brainVizLastAgentId = null; // Reset brain viz agent tracking
        document.getElementById('agent-inspector-modal').classList.add('hidden');
        render();
    }

    // Expose closeAgentModal globally and wire CSP3 delegated click listener
    window.closeAgentModal = closeAgentModal;
    document.addEventListener('click', function(e) {
        if (e.target.closest('[data-close-agent-modal]')) {
            closeAgentModal();
        }
    });

    /**
     * Resize canvas to fill container
     */
    function resizeCanvas() {
        if (!canvas || !ctx) return;
        const container = canvas.parentElement;
        if (!container) return;

        const dpr = window.devicePixelRatio || 1;

        // Set display size
        const displayWidth = container.clientWidth;
        const displayHeight = container.clientHeight;

        // Skip if container has no size (hidden/not rendered yet)
        if (displayWidth === 0 || displayHeight === 0) return;

        canvas.style.width = displayWidth + 'px';
        canvas.style.height = displayHeight + 'px';

        // Set actual canvas size (scaled for high-DPI)
        canvas.width = displayWidth * dpr;
        canvas.height = displayHeight * dpr;

        // Update scale for grid rendering
        updateScale();

        // Render immediately with current frame
        render();
    }

    /**
     * Update scale based on grid and canvas size
     */
    function updateScale() {
        if (!canvas) return;
        const containerWidth = canvas.clientWidth;
        let containerHeight = canvas.clientHeight;
        
        // Account for real-time telemetry dashboard if visible and not collapsed
        const telemetryEl = document.getElementById('v4-telemetry-dashboard');
        if (telemetryEl && !telemetryEl.classList.contains('hidden') && !telemetryEl.classList.contains('collapsed')) {
            const telemetryHeight = telemetryEl.clientHeight || 180;
            containerHeight = Math.max(200, containerHeight - telemetryHeight);
        }

        scale = Math.min(
            containerWidth / gridWidth,
            containerHeight / gridHeight
        ) * 0.92;
        offsetX = (containerWidth - gridWidth * scale) / 2;
        offsetY = (containerHeight - gridHeight * scale) / 2;

        isGridCacheValid = false;
        isTerrainCacheValid = false;
        isWeatherTileValid = false;
        isResourceTileValid = false;
    }

    /**
     * Connect to live simulation stream
     * 
     * FIXED: Now properly integrated with simulation start flow
     * - Animation transitions from colorful to monochrome on connect
     * - Handles connection errors gracefully
     */
    function connect() {
        const apiUrl = window.SIM_SERVICE_URL || '';
        const streamUrl = `${apiUrl}/api/live`;

        debugLog('CONN', '========================================');
        debugLog('CONN', `Connecting to: ${streamUrl}`);
        debugLog('CONN', `User Agent: ${navigator.userAgent}`);
        
        recordConnectionEvent('CONNECTING', { url: streamUrl });
        updateConnectionStatus('connecting');
        
        // CRITICAL FIX: Clear any existing frame flow monitor before reconnecting
        if (frameFlowMonitor) {
            clearInterval(frameFlowMonitor);
            frameFlowMonitor = null;
        }
        
        // Start the pixelated loader animation if not already running
        if (window.PixelatedLoader && !window.PixelatedLoader.isAnimating()) {
            window.PixelatedLoader.start();
        }

        if (eventSource) {
            debugLog('CONN', 'Closing existing EventSource');
            recordConnectionEvent('CLOSING_PREVIOUS');
            eventSource.close();
            eventSource = null;
        }

        try {
            eventSource = new EventSource(streamUrl);
            debugLog('CONN', `EventSource created, readyState=${eventSource.readyState}`);
        } catch (err) {
            debugLog('ERROR', `Failed to create EventSource: ${err.message}`, { name: err.name });
            recordConnectionEvent('CREATE_FAILED', { error: err.message });
            updateConnectionStatus('disconnected');
            return;
        }

        // Set up connection timeout
        const connectionTimeout = setTimeout(() => {
            if (eventSource && eventSource.readyState === EventSource.CONNECTING) {
                debugLog('ERROR', 'Connection timeout - stuck in CONNECTING state');
                recordConnectionEvent('TIMEOUT');
                
                // Test if backend is reachable via fetch
                fetch(`${apiUrl}/health`)
                    .then(r => {
                        debugLog('CONN', `Health check status: ${r.status}`);
                        return r.json();
                    })
                    .then(data => debugLog('CONN', 'Health check response', data))
                    .catch(e => debugLog('ERROR', `Health check failed: ${e.message}`));
                
                // CRITICAL FIX: Force reconnect if stuck in CONNECTING
                debugLog('CONN', 'Force reconnecting due to connection timeout...');
                eventSource.close();
                scheduleReconnect('connection timeout');
            }
        }, 10000);
        
        // CRITICAL FIX: Set up frame flow monitoring to detect frozen streams
        let lastReceivedTick = 0;
        let lastReceivedTime = performance.now();
        frameFlowMonitor = setInterval(() => {
            if (!eventSource || eventSource.readyState !== EventSource.OPEN) {
                return; // Only monitor when connection is open
            }
            if (isSimulationPaused) {
                return; // Do not trigger stall reconnect while simulation is intentionally paused
            }
            
            const now = performance.now();
            const timeSinceLastFrame = now - debugState.frameStats.lastFrameTime;
            
            // If no frame received for 5 seconds, connection is likely stuck
            if (timeSinceLastFrame > 5000 && debugState.frameStats.framesReceived > 0) {
                debugLog('ERROR', `Frame flow stuck! No frame for ${Math.round(timeSinceLastFrame)}ms. Forcing reconnect...`);
                recordConnectionEvent('FRAME_FLOW_STUCK', { timeSinceLastFrame });
                
                // Close and reconnect
                eventSource.close();
                updateConnectionStatus('disconnected');
                
                // Clear the monitor before reconnecting
                clearInterval(frameFlowMonitor);
                
                // Attempt reconnect with exponential backoff
                scheduleReconnect('frame flow stall');
            }
        }, 2000); // Check every 2 seconds

        eventSource.onopen = () => {
            clearTimeout(connectionTimeout);
            debugLog('CONN', '========================================');
            debugLog('CONN', 'CONNECTED! EventSource opened successfully');
            debugLog('CONN', `readyState=${eventSource.readyState} (OPEN=1)`);
            debugLog('CONN', '========================================');
            recordConnectionEvent('CONNECTED');
            updateConnectionStatus('connected');
            
            // CRITICAL FIX: Reset reconnection attempts on successful connection
            reconnectAttempts = 0;
            
            // CRITICAL FIX: Reset frame tracking on successful connection
            lastReceivedTime = performance.now();
            lastReceivedTick = 0;
            
            // Transition animation from colorful to monochrome
            if (window.PixelatedLoader && window.PixelatedLoader.isAnimating()) {
                window.PixelatedLoader.transitionToGrayscale(() => {
                    if (hasUserDismissedOverlay) {
                        hideOverlay();
                    }
                });
            } else if (hasUserDismissedOverlay) {
                hideOverlay();
            }
        };

        eventSource.onmessage = (event) => {
            // DEBUG: Log SSE message receipt (throttled)
            if (debugState.frameStats.framesReceived < 5 || debugState.frameStats.framesReceived % 50 === 0) {
                debugLog('SSE', `Received message`, {
                    eventId: event.id || 'none',
                    dataLength: event.data?.length,
                    preview: event.data?.substring(0, 80) + '...'
                });
            }
            
            try {
                const frame = JSON.parse(event.data);
                handleFrame(frame);
            } catch (err) {
                debugLog('ERROR', `Failed to parse frame JSON: ${err.message}`);
                debugLog('ERROR', `Raw data: ${event.data?.substring(0, 200)}`);
            }
        };

        eventSource.onerror = (err) => {
            clearTimeout(connectionTimeout);
            // CRITICAL FIX: Clear frame flow monitor on error
            if (typeof frameFlowMonitor !== 'undefined') {
                clearInterval(frameFlowMonitor);
            }
            
            const readyState = eventSource?.readyState;
            let stateName = 'UNKNOWN';
            if (readyState === EventSource.CONNECTING) stateName = 'CONNECTING';
            else if (readyState === EventSource.OPEN) stateName = 'OPEN';
            else if (readyState === EventSource.CLOSED) stateName = 'CLOSED';
            
            debugLog('ERROR', `EventSource ERROR - state=${stateName}`);
            recordConnectionEvent('ERROR', { readyState, stateName });
            
            updateConnectionStatus('disconnected');
            
            // CRITICAL FIX: Always force reconnect on error, don't wait for CLOSED
            // Close existing connection if not already closed
            if (eventSource) {
                eventSource.close();
                eventSource = null;
            }
            // Use exponential backoff for reconnection
            scheduleReconnect('EventSource error');
        };
        
        debugLog('CONN', 'EventSource handlers registered, waiting for connection...');
    }

    /**
     * Show export button when simulation has ended
     */
    function showExportButton() {
        const exportSection = document.getElementById('export-section');
        if (exportSection) {
            exportSection.classList.remove('hidden');
            console.log('[Viewer] Export button shown');
        }
    }

    /**
     * Hide export button (when restarting simulation)
     */
    function hideExportButton() {
        const exportSection = document.getElementById('export-section');
        if (exportSection) {
            exportSection.classList.add('hidden');
            console.log('[Viewer] Export button hidden');
        }
    }

    /**
     * Handle incoming frame from live stream
     * Phase 4: Added spatial hash building and frame buffering
     * Phase 1: Added motion interpolation tracking
     * Phase 5: Added simulation status handling (running/paused/completed/stopped)
     */
    function handleFrame(frame) {
        if (!frame) {
            debugLog('ERROR', 'Received empty frame');
            return;
        }
        
        // DEBUG: Record frame receipt
        recordFrameReceipt(frame);
        
        // CRITICAL FIX: Frame gap detection - detect missed frames
        if (currentFrame && frame.tick > currentFrame.tick + 1) {
            const gap = frame.tick - currentFrame.tick - 1;
            // Only trigger alert for genuinely anomalous gaps (>25 ticks) to prevent false alerts during 5.0x fast-forward
            if (gap > 25) {
                debugLog('WARN', `Frame gap detected: missed ${gap} ticks (${currentFrame.tick} -> ${frame.tick})`);
            }
        }

        // Dynamically update tick delay duration if supplied in frame telemetry
        if (frame.tick_delay_ms && typeof frame.tick_delay_ms === 'number' && frame.tick_delay_ms > 0) {
            frameDurationMs = frame.tick_delay_ms;
        }
        
        // Dynamically synchronize grid dimensions from simulation frame
        if (frame.grid_dimensions && Array.isArray(frame.grid_dimensions)) {
            const [w, h] = frame.grid_dimensions;
            if (w > 0 && h > 0 && (w !== gridWidth || h !== gridHeight)) {
                gridWidth = w;
                gridHeight = h;
                updateScale();
            }
        }

        // DEBUG: Log frame data (throttled)
        if (frame.tick <= 5 || frame.tick % 50 === 0) {
            debugLog('Frame', `Processing tick=${frame.tick}`, {
                agents: frame.agents?.length || 0,
                alive: frame.stats?.alive_agents || 0,
                timestamp: new Date().toISOString()
            });
        }
        
        // Phase 5: Handle simulation completion
        if (frame.status === 'Completed' || frame.status === 'Stopped') {
            debugLog('INFO', `Simulation ended: ${frame.status}`);
            stopRenderLoop();
            showGameOver(frame.status);
            showExportButton();
            disconnect();
            
            // Dispatch event for controls module
            window.dispatchEvent(new CustomEvent('simulationEnded', { 
                detail: { status: frame.status }
            }));
            
            return;
        }
        
        // Phase 1: Store previous frame for motion interpolation
        previousFrame = currentFrame;
        
        // DEBUG: Log agent position changes for first 3 agents
        // P0 FIX: Use agentMap for O(1) lookup instead of O(n) find()
        if (frame.agents && frame.agents.length > 0 && previousFrame && previousFrame.agentMap) {
            for (let i = 0; i < Math.min(3, frame.agents.length); i++) {
                const agent = frame.agents[i];
                const prevAgent = previousFrame.agentMap.get(agent.agent_id);
                if (prevAgent) {
                    const dx = agent.x - prevAgent.x;
                    const dy = agent.y - prevAgent.y;
                    const moved = Math.abs(dx) > 0.001 || Math.abs(dy) > 0.001;
                    
                    if (moved && (frame.tick <= 10 || frame.tick % 100 === 0)) {
                        debugLog('Agent', `Agent ${agent.agent_id} moved`, {
                            tick: frame.tick,
                            from: `(${prevAgent.x.toFixed(2)}, ${prevAgent.y.toFixed(2)})`,
                            to: `(${agent.x.toFixed(2)}, ${agent.y.toFixed(2)})`,
                            delta: `(${dx.toFixed(3)}, ${dy.toFixed(3)})`
                        });
                    }
                }
            }
        }
        
        currentFrame = frame;
        currentTick = frame.tick;
        frameInterpStartTime = performance.now();

        // P0 FIX: Build agent ID map for O(1) lookups
        currentFrame.agentMap = new Map(frame.agents.map((a) => [a.agent_id, a]));
        if (previousFrame && !previousFrame.agentMap && previousFrame.agents) {
            previousFrame.agentMap = new Map(previousFrame.agents.map((a) => [a.agent_id, a]));
        }

        // Invalidate or update offscreen tile caches when payload arrives
        if (frame.terrain) {
            cachedTerrain = frame.terrain;
            isTerrainCacheValid = false;
        }
        if (frame.weather) {
            cachedWeather = frame.weather;
            updateWeatherTile(frame.weather);
        }
        if (frame.resources) {
            cachedResources = frame.resources;
            updateResourceTile(frame.resources);
        }

        // P0 FIX: Cleanup agent history to prevent memory leaks
        cleanupAgentHistory(frame.agents);

        // Phase 4: Build spatial hash for O(1) lookups
        buildSpatialHash();
        
        // Phase 4: Buffer frame for replay (deduplicate standby frames to preserve replay buffer)
        if (frameBuffer.length === 0 || frameBuffer[frameBuffer.length - 1].tick !== frame.tick || frame.tick > 0) {
            bufferFrame(frame);
        }
        
        // Phase 6: Detect and log significant events
        detectEvents(frame);

        // Update tick and population stats in telemetry bar
        const tickDisplay = document.getElementById('tick-display');
        if (tickDisplay) {
            tickDisplay.textContent = currentTick;
        }
        const stats = getStats();
        const statAlive = document.getElementById('stat-alive');
        if (statAlive) statAlive.textContent = stats.alive;
        const statAgents = document.getElementById('stat-agents');
        if (statAgents) statAgents.textContent = stats.agents;
        const statVerified = document.getElementById('stat-verified');
        if (statVerified) statVerified.textContent = stats.verified;
        const statGroups = document.getElementById('stat-groups');
        if (statGroups) statGroups.textContent = stats.groups;
        // Version 4.0: Render real-time telemetry analytics
        if (frame.v4_telemetry) {
            renderV4Dashboard(frame.v4_telemetry);
        }

        // Ensure 60Hz decoupled render loop is active
        if (!isRenderLoopActive) {
            startRenderLoop();
        }

        // Publish unified simulation telemetry for canonical chat assistant
        const expSelector = document.getElementById('experiment-selector');
        const activeExp = expSelector ? expSelector.value : 'default';
        window.FC_SIMULATOR_CONTEXT = {
            tick: currentTick,
            status: frame.status || 'Running',
            agents_count: frame.agents ? frame.agents.length : 0,
            alive_agents: frame.stats ? frame.stats.alive_agents : 0,
            avg_trust: frame.stats ? frame.stats.avg_trust : null,
            gini: frame.v4_telemetry ? frame.v4_telemetry.gini_coefficient : null,
            experiment: activeExp || 'default',
            selected_agent: selectedAgentId != null && currentFrame.agentMap ? currentFrame.agentMap.get(selectedAgentId) : null,
            timestamp: Date.now()
        };

        // Dispatch event for controls module
        window.dispatchEvent(new CustomEvent('simulatorFrame', { detail: frame }));
    }
    
    /**
     * Show Game Over screen
     * Phase 5: Retro pixelated game over display
     */
    function showGameOver(status) {
        const gameOverOverlay = document.getElementById('game-over-overlay');
        if (gameOverOverlay) {
            gameOverOverlay.classList.remove('hidden');
        }
        
        // Show Game Over on the loader canvas
        if (window.PixelatedLoader) {
            window.PixelatedLoader.showGameOver();
        }
        
        console.log('[Viewer] Game Over screen shown, status:', status);
    }
    
    /**
     * Hide Game Over screen and restart
     * Phase 5: Reset for new simulation
     */
    function hideGameOver() {
        const gameOverOverlay = document.getElementById('game-over-overlay');
        if (gameOverOverlay) {
            gameOverOverlay.classList.add('hidden');
        }
        
        // Reset pixelated loader to initial state
        if (window.PixelatedLoader) {
            window.PixelatedLoader.stop();
        }
        
        console.log('[Viewer] Game Over screen hidden');
    }

    // Cache for agent data when modal is opened (prevents modal closing on agent death)
    let cachedAgentData = null;
    /**
     * Render Version 4.0 real-time analytics dashboard
     * 1. Decile Vulnerability Gradient (38.9x Odds Ratio)
     * 2. Global Kuramoto Coherence Tracker
     * 3. Simplex Delta^4 Barycentric Radar
     */
    const kuramotoHistory = [];
    function renderV4Dashboard(v4) {
        const dashboard = document.getElementById('v4-telemetry-dashboard');
        if (dashboard && dashboard.classList.contains('hidden')) {
            dashboard.classList.remove('hidden');
        }

        // Update badges
        const badgeGradient = document.getElementById('v4-gradient-badge');
        if (badgeGradient) {
            badgeGradient.textContent = `${v4.odds_ratio_d1_vs_d10.toFixed(1)}x OR`;
        }
        const badgeCoherence = document.getElementById('v4-coherence-val');
        if (badgeCoherence) {
            badgeCoherence.textContent = `R: ${v4.kuramoto_coherence_r.toFixed(2)}`;
        }
        const badgeSimplex = document.getElementById('v4-simplex-state');
        if (badgeSimplex) {
            badgeSimplex.textContent = `χ: ${v4.active_cloaking_ratio.toFixed(2)}x`;
        }
        const badgeBaylor = document.getElementById('v4-baylor-val');
        if (badgeBaylor && v4.baylor_unconscious_mode !== undefined) {
            badgeBaylor.textContent = v4.baylor_unconscious_mode ? 'Baylor: M=0 (Unconscious)' : 'Baylor: M=1 (Conscious)';
        }
        const badgeFriction = document.getElementById('v4-friction-val');
        if (badgeFriction && v4.baylor_cognitive_friction !== undefined) {
            badgeFriction.textContent = `R_fric: ${v4.baylor_cognitive_friction.toFixed(2)}`;
        }

        // Update gauge readouts in sidebar
        const gaugeOR = document.getElementById('gauge-odds-ratio');
        if (gaugeOR) gaugeOR.textContent = `${v4.odds_ratio_d1_vs_d10.toFixed(1)}x`;
        const gaugeD1 = document.getElementById('gauge-d1-rate');
        if (gaugeD1 && v4.decile_uptake_rates && v4.decile_uptake_rates[0] !== undefined) {
            gaugeD1.textContent = `${(v4.decile_uptake_rates[0] * 100).toFixed(1)}%`;
        }
        const gaugeD10 = document.getElementById('gauge-d10-rate');
        if (gaugeD10 && v4.decile_uptake_rates && v4.decile_uptake_rates[9] !== undefined) {
            gaugeD10.textContent = `${(v4.decile_uptake_rates[9] * 100).toFixed(1)}%`;
        }

        // Render Chart 1: Decile Gradient
        const c1 = document.getElementById('canvas-gradient');
        if (c1 && v4.decile_uptake_rates) {
            const ctx = c1.getContext('2d');
            const w = c1.width, h = c1.height;
            ctx.clearRect(0, 0, w, h);
            ctx.fillStyle = '#0a0a10';
            ctx.fillRect(0, 0, w, h);

            const n = v4.decile_uptake_rates.length;
            const barW = (w - 20) / n;
            for (let i = 0; i < n; i++) {
                const rate = v4.decile_uptake_rates[i];
                const barH = rate * (h - 25);
                const x = 10 + i * barW;
                const y = h - 15 - barH;

                ctx.fillStyle = i === 0 ? '#ef4444' : i === 9 ? '#22c55e' : '#64748b';
                ctx.fillRect(x + 2, y, barW - 4, barH);

                ctx.fillStyle = '#94a3b8';
                ctx.font = '8px monospace';
                ctx.textAlign = 'center';
                ctx.fillText(`D${i + 1}`, x + barW / 2, h - 4);
            }
        }

        // Render Chart 2: Kuramoto Coherence Tracker
        const c2 = document.getElementById('canvas-kuramoto');
        if (c2) {
            kuramotoHistory.push(v4.kuramoto_coherence_r);
            if (kuramotoHistory.length > 50) kuramotoHistory.shift();

            const ctx = c2.getContext('2d');
            const w = c2.width, h = c2.height;
            ctx.clearRect(0, 0, w, h);
            ctx.fillStyle = '#0a0a10';
            ctx.fillRect(0, 0, w, h);

            ctx.strokeStyle = '#334155';
            ctx.lineWidth = 1;
            ctx.beginPath();
            ctx.moveTo(0, h / 2);
            ctx.lineTo(w, h / 2);
            ctx.stroke();

            ctx.strokeStyle = '#38bdf8';
            ctx.lineWidth = 2;
            ctx.beginPath();
            const step = w / Math.max(kuramotoHistory.length - 1, 1);
            for (let i = 0; i < kuramotoHistory.length; i++) {
                const r = kuramotoHistory[i];
                const x = i * step;
                const y = h - 10 - r * (h - 20);
                if (i === 0) ctx.moveTo(x, y);
                else ctx.lineTo(x, y);
            }
            ctx.stroke();
        }
        // Render Chart 3: Simplex Delta^4 Pentagon/Radar
        const c3 = document.getElementById('canvas-simplex');
        if (c3 && v4.simplex_projection) {
            const ctx = c3.getContext('2d');
            const w = c3.width, h = c3.height;
            ctx.clearRect(0, 0, w, h);
            ctx.fillStyle = '#0a0a10';
            ctx.fillRect(0, 0, w, h);

            const cx = w / 2, cy = h / 2, maxR = h / 2 - 14;
            const channels = ['D', 'O', 'C', 'A', 'S'];
            const angles = [
                -Math.PI / 2,
                -Math.PI / 2 + (2 * Math.PI / 5),
                -Math.PI / 2 + (4 * Math.PI / 5),
                -Math.PI / 2 + (6 * Math.PI / 5),
                -Math.PI / 2 + (8 * Math.PI / 5)
            ];

            ctx.strokeStyle = '#334155';
            ctx.lineWidth = 1;
            for (let i = 0; i < 5; i++) {
                ctx.beginPath();
                ctx.moveTo(cx, cy);
                ctx.lineTo(cx + Math.cos(angles[i]) * maxR, cy + Math.sin(angles[i]) * maxR);
                ctx.stroke();

                ctx.fillStyle = '#94a3b8';
                ctx.font = '8px monospace';
                ctx.textAlign = 'center';
                ctx.textBaseline = 'middle';
                const labelDist = maxR + 8;
                ctx.fillText(channels[i], cx + Math.cos(angles[i]) * labelDist, cy + Math.sin(angles[i]) * labelDist);
            }

            ctx.fillStyle = 'rgba(239, 68, 68, 0.25)';
            ctx.strokeStyle = '#ef4444';
            ctx.lineWidth = 1.5;
            ctx.beginPath();
            for (let i = 0; i < 5; i++) {
                const val = Math.min(Math.max(v4.simplex_projection[i] || 0.2, 0.05), 1.0);
                const r = val * maxR;
                const x = cx + Math.cos(angles[i]) * r;
                const y = cy + Math.sin(angles[i]) * r;
                if (i === 0) ctx.moveTo(x, y);
                else ctx.lineTo(x, y);
            }
            ctx.closePath();
            ctx.fill();
            ctx.stroke();
        }
    }



    /**
     * Update inspector modal with current agent data (called on each frame)
     * BUG FIX: Don't close modal when agent dies - show cached data instead
     */
    function updateInspectorModal() {
        if (!currentFrame || !currentFrame.agents) return;

        const agent = currentFrame.agents.find(a => a.agent_id === selectedAgentId);
        
        if (!agent) {
            // Agent no longer exists (died) - use cached data, don't close modal
            if (!cachedAgentData) {
                closeAgentModal();
                return;
            }
            // Continue with cached data - show "Deceased" indicator
            document.getElementById('inspector-status').textContent = 'Deceased';
            document.getElementById('inspector-status').className = 'inspector-value status-dead';
            return;
        }

        // Update age (increases each tick)
        document.getElementById('inspector-age').textContent = agent.age;

        // Update status
        const statusEl = document.getElementById('inspector-status');
        statusEl.textContent = agent.alive ? 'Alive' : 'Dead';
        statusEl.className = 'inspector-value ' + (agent.alive ? 'status-alive' : 'status-dead');

        // Update DOCAS state bars (transient - changes every tick)
        const docasBars = document.querySelectorAll('#inspector-docas-bars .docas-bar-row');
        if (agent.docas) {
            agent.docas.forEach((level, i) => {
                if (docasBars[i]) {
                    const bar = docasBars[i].querySelector('.docas-bar');
                    const value = docasBars[i].querySelector('.docas-value');
                    const percentage = Math.min(100, level * 100);
                    bar.style.width = `${percentage}%`;
                    value.textContent = level.toFixed(3);
                }
            });
        }

        // Update energy
        document.getElementById('inspector-energy').textContent = agent.energy.toFixed(1);

        // Update position
        document.getElementById('inspector-position').textContent = `(${agent.x.toFixed(1)}, ${agent.y.toFixed(1)})`;

        // Update group
        document.getElementById('inspector-group').textContent =
            (agent.group_id !== null && agent.group_id !== undefined) ? agent.group_id : 'None';

        // Update trust and reputation (live update)
        updateTauDisplay(agent);
        updateReputationDisplay(agent);
        
        // CRITICAL FIX: Update brain visualization with live DOCAS changes
        // This ensures the neural map reflects current agent state in real-time
        if (brainViz && agent.docas && Array.isArray(agent.docas) && agent.docas.length === 5) {
            // Use immediate=false for smooth interpolation during live updates
            brainViz.updateFromDocas({
                levels: agent.docas
            }, false, agent.alive);
        }
    }

    /**
     * Update trust (τ) display with color-coded value
     * τ = (D + O + S) / (1 + C + A)
     */
    function updateTauDisplay(agent) {
        const tauEl = document.getElementById('inspector-tau');
        if (!tauEl) return;

        const tau = agent.tau !== undefined ? agent.tau : 1.0;
        tauEl.textContent = tau.toFixed(3);

        // Color code by trust level
        let colorClass = 'tau-medium';
        if (tau > 1.2) {
            colorClass = 'tau-high';
        } else if (tau < 0.8) {
            colorClass = 'tau-low';
        }
        tauEl.className = `inspector-value mono tau-value ${colorClass}`;
    }

    /**
     * Update reputation display
     * Reputation = positive_interactions / total_interactions
     */
    function updateReputationDisplay(agent) {
        const repEl = document.getElementById('inspector-reputation');
        if (!repEl) return;

        const rep = agent.reputation !== undefined ? agent.reputation : 0.5;
        const percentage = (rep * 100).toFixed(0);
        repEl.textContent = `${percentage}%`;
    }

    /**
     * Update connection status indicator
     */
    function updateConnectionStatus(status) {
        const indicator = document.getElementById('connection-status');
        if (!indicator) return;

        indicator.className = `connection-indicator ${status}`;
        const textEl = indicator.querySelector('.status-text');
        if (textEl) {
            switch (status) {
                case 'connecting':
                    textEl.textContent = 'Connecting...';
                    break;
                case 'connected':
                    textEl.textContent = 'Live';
                    break;
                case 'disconnected':
                    textEl.textContent = 'Reconnecting...';
                    break;
            }
        }
    }

    let hasUserDismissedOverlay = false;

    /**
     * Hide canvas overlay
     */
    function hideOverlay() {
        hasUserDismissedOverlay = true;
        console.log('[Viewer] Hiding canvas overlay');
        const overlay = document.getElementById('canvas-overlay');
        if (overlay) {
            overlay.classList.add('hidden');
            overlay.style.display = 'none';
            console.log('[Viewer] Overlay hidden');
        } else {
            console.warn('[Viewer] Overlay element not found');
        }
    }

    // ============================================================================
    // EVENT LOG FUNCTIONS (Retro Terminal Style)
    // ============================================================================

    /**
     * Add an event to the event log
     * @param {string} message - Event message
     * @param {string} type - Event type: 'info', 'success', 'warning', 'danger', 'event'
     * @param {number} agentId - Optional agent ID to highlight
     * @param {string} category - Event category for batching: 'reproduction', 'game', 'group', 'interaction', 'other'
     */
    function logEvent(message, type = 'info', agentId = null, category = 'other') {
        const now = Date.now();
        
        // If this is a batchable category, add to batch instead of immediate logging
        if (category === 'reproduction') {
            eventBatch.reproductions++;
            return;
        }
        if (category === 'game') {
            eventBatch.gamesPlayed++;
            return;
        }
        if (category === 'group') {
            eventBatch.groupFormations++;
            return;
        }
        if (category === 'interaction') {
            eventBatch.interactions++;
            return;
        }
        
        // Process batch if interval has passed
        if (now - lastBatchTime >= BATCH_INTERVAL) {
            processBatch();
            lastBatchTime = now;
        }
        
        const timestamp = new Date().toLocaleTimeString('en-US', { 
            hour12: false, 
            hour: '2-digit', 
            minute: '2-digit', 
            second: '2-digit' 
        });
        
        const event = {
            timestamp,
            message,
            type,
            agentId
        };
        
        activityLog.push(event);
        
        // Keep only last MAX_EVENTS
        if (activityLog.length > MAX_EVENTS) {
            activityLog.shift();
        }
        
        renderActivityLog();
    }
    
    /**
     * Process batched events and create summary entries
     */
    function processBatch() {
        const now = new Date();
        const timestamp = now.toLocaleTimeString('en-US', { 
            hour12: false, 
            hour: '2-digit', 
            minute: '2-digit', 
            second: '2-digit' 
        });
        
        let batchEvents = [];
        
        // Add reproduction summary
        if (eventBatch.reproductions > 0) {
            const msg = eventBatch.reproductions === 1 
                ? '1 agent reproduced' 
                : `${eventBatch.reproductions} agents reproduced`;
            batchEvents.push({ timestamp, message: msg, type: 'success', agentId: null });
            eventBatch.reproductions = 0;
        }
        
        // Add games played summary
        if (eventBatch.gamesPlayed > 0) {
            const msg = eventBatch.gamesPlayed === 1 
                ? '1 game played' 
                : `${eventBatch.gamesPlayed} games played`;
            batchEvents.push({ timestamp, message: msg, type: 'info', agentId: null });
            eventBatch.gamesPlayed = 0;
        }
        
        // Add group formations summary
        if (eventBatch.groupFormations > 0) {
            const msg = eventBatch.groupFormations === 1 
                ? '1 group formed' 
                : `${eventBatch.groupFormations} groups formed`;
            batchEvents.push({ timestamp, message: msg, type: 'event', agentId: null });
            eventBatch.groupFormations = 0;
        }
        
        // Add interactions summary
        if (eventBatch.interactions > 0) {
            const msg = eventBatch.interactions === 1 
                ? '1 significant interaction' 
                : `${eventBatch.interactions} significant interactions`;
            batchEvents.push({ timestamp, message: msg, type: 'info', agentId: null });
            eventBatch.interactions = 0;
        }
        
        // Add all batched events to the log
        if (batchEvents.length > 0) {
            activityLog.push(...batchEvents);
            
            // Keep only last MAX_EVENTS
            while (activityLog.length > MAX_EVENTS) {
                activityLog.shift();
            }
            
            renderActivityLog();
        }
    }

    /**
     * Check if user is scrolled to bottom of event log
     */
    function isScrolledToBottom(container) {
        const threshold = 10; // pixels from bottom to consider "at bottom"
        return container.scrollHeight - container.scrollTop - container.clientHeight <= threshold;
    }
    
    /**
     * Update scroll indicator visibility
     */
    function updateScrollIndicator() {
        const indicator = document.getElementById('activity-log-scroll-indicator');
        if (!indicator) return;
        
        if (isUserScrolledUp) {
            indicator.classList.add('visible');
        } else {
            indicator.classList.remove('visible');
        }
    }
    
    /**
     * Scroll event log to bottom
     */
    function scrollToBottom() {
        const container = document.getElementById('activity-log-content');
        if (!container) return;
        
        container.scrollTop = container.scrollHeight;
        isUserScrolledUp = false;
        updateScrollIndicator();
    }
    
    /**
     * Setup scroll listener for event log container
     * Tracks whether user has scrolled up to read history
     */
    function setupActivityLogScrollListener() {
        const container = document.getElementById('activity-log-content');
        if (!container) return;
        
        container.addEventListener('scroll', () => {
            const wasScrolledUp = isUserScrolledUp;
            isUserScrolledUp = !isScrolledToBottom(container);
            
            // Only update indicator if state changed
            if (wasScrolledUp !== isUserScrolledUp) {
                updateScrollIndicator();
            }
        });
        
        // Setup click handler for scroll indicator
        const indicator = document.getElementById('activity-log-scroll-indicator');
        if (indicator) {
            indicator.addEventListener('click', scrollToBottom);
        }
    }
    
    /**
     * Render event log to the DOM
     * Only auto-scrolls if user is already at bottom (chat-like behavior)
     */
    function renderActivityLog() {
        const container = document.getElementById('activity-log-content');
        if (!container) return;
        
        // Check if we should auto-scroll (user is at bottom)
        const shouldAutoScroll = !isUserScrolledUp || isScrolledToBottom(container);
        
        // Clear current content (except welcome message if empty)
        if (activityLog.length === 0) {
            container.innerHTML = `
                <div class="activity-log-welcome">Welcome to Agent Society Simulator v1.0</div>
                <div class="activity-log-welcome">Select an experiment and click RUN to begin...</div>
            `;
            updateScrollIndicator();
            return;
        }
        
        container.innerHTML = activityLog.map(event => {
            const agentSpan = event.agentId !== null 
                ? `<span class="activity-log-agent">[#${event.agentId}]</span>` 
                : '';
            return `
                <div class="activity-log-entry ${event.type}">
                    <span class="activity-log-timestamp">[${event.timestamp}]</span>
                    ${agentSpan}
                    <span class="activity-log-action">${escapeHtml(event.message)}</span>
                </div>
            `;
        }).join('');
        
        // Auto-scroll to bottom only if user was already at bottom
        if (shouldAutoScroll) {
            container.scrollTop = container.scrollHeight;
            isUserScrolledUp = false;
        }
        
        updateScrollIndicator();
    }

    /**
     * Escape HTML to prevent XSS
     */
    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    /**
     * Detect and log significant events from frame data
     * Only logs meaningful activity - no spam from individual spawn/death
     */
    function detectEvents(frame) {
        if (!frame || !frame.agents) return;
        
        const agents = frame.agents;
        const aliveAgents = agents.filter(a => a.alive);
        const deadAgents = agents.filter(a => !a.alive);
        
        // Process batch periodically
        const now = Date.now();
        if (now - lastBatchTime >= BATCH_INTERVAL) {
            processBatch();
            lastBatchTime = now;
        }
        
        // Track reproduction events (from frame data if available)
        if (frame.events) {
            for (const event of frame.events) {
                if (event.type === 'reproduction') {
                    eventBatch.reproductions++;
                } else if (event.type === 'game_played') {
                    eventBatch.gamesPlayed++;
                } else if (event.type === 'group_formed') {
                    eventBatch.groupFormations++;
                } else if (event.type === 'significant_interaction') {
                    eventBatch.interactions++;
                }
            }
        }
        
        // Process game events from simulation core
        if (frame.game_events && frame.game_events.length > 0) {
            for (const gameEvent of frame.game_events) {
                eventBatch.gamesPlayed++;
                // Log detailed game event to activity log
                const gameType = gameEvent.game_type || 'Unknown';
                const outcome = gameEvent.outcome || 'Completed';
                logEvent(
                    `Game: ${gameType} between Agent #${gameEvent.agent_a} and #${gameEvent.agent_b} - ${outcome}`,
                    'info'
                );
            }
        }
        
        // Process encounter events from simulation core
        if (frame.encounter_events && frame.encounter_events.length > 0) {
            for (const encounter of frame.encounter_events) {
                eventBatch.interactions++;
                logEvent(
                    `Encounter: Agent #${encounter.agent_a} met #${encounter.agent_b} (${encounter.encounter_type})`,
                    'info'
                );
            }
        }
        
        // Detect reproduction from agent parentage (new agents with parents)
        if (frame.tick > 0 && aliveAgents.length > lastAgentCount) {
            const newAgents = aliveAgents.filter(a => 
                a.age === 0 && a.parent1_id !== null && a.parent1_id !== undefined
            );
            if (newAgents.length > 0) {
                eventBatch.reproductions += newAgents.length;
            }
        }
        
        // Low population warning (significant event)
        if (aliveAgents.length < 5 && lastAgentCount >= 5) {
            logEvent('WARNING: Population critically low!', 'danger');
        }
        
        // High population milestone (significant event)
        if (aliveAgents.length >= 100 && lastAgentCount < 100) {
            logEvent('Milestone: 100 agents alive', 'success');
        }
        
        // Track identity theft events (if data available)
        const identityThefts = agents.filter(a => a.identity_stolen).length;
        if (identityThefts > 0 && frame.tick % 50 === 0) {
            logEvent(`${identityThefts} identity theft(s) detected`, 'danger');
        }
        
        // Track data breaches (if data available)
        const breaches = agents.filter(a => a.data_breach).length;
        if (breaches > 0 && frame.tick % 50 === 0) {
            logEvent(`${breaches} data breach(es) detected`, 'danger');
        }
        
        // Tick milestones (every 100 ticks)
        if (frame.tick % 100 === 0 && frame.tick > 0) {
            logEvent(`Tick ${frame.tick}: ${aliveAgents.length} agents alive`, 'info');
        }
        
        // Update tracking
        lastAgentCount = aliveAgents.length;
        lastTick = frame.tick;
    }

    /**
     * Set layer visibility
     * Per SIM-002: Layer toggles are runtime visualization controls only.
     * They do not affect simulation state.
     */
    function setLayer(layerName, visible) {
        if (layerName in layers) {
            layers[layerName] = visible;
            render();
        }
    }

    /**
     * Set layer opacity (0-1)
     * Per SIM-002: Opacity controls are runtime visualization settings only.
     * They do not affect simulation state.
     */
    function setLayerOpacity(layerName, opacity) {
        if (layerName in layerOpacity) {
            layerOpacity[layerName] = Math.max(0, Math.min(1, opacity));
            render();
        }
    }

    /**
     * Set filter value
     * Per SIM-002: Filters are client-side visualization filters.
     * They affect rendering only, not simulation state.
     */
    function setFilter(filterPath, value) {
        const parts = filterPath.split('.');
        let target = filters;
        for (let i = 0; i < parts.length - 1; i++) {
            target = target[parts[i]];
        }
        target[parts[parts.length - 1]] = value;
        render();
    }

    /**
     * Reset all filters to defaults
     */
    function resetFilters() {
        filters.trait = 'all';
        filters.status = 'alive';
        filters.relativeDistanceThreshold = 10;
        render();
    }

    /**
     * Check if agent passes all filters
     * Per SIM-002: Filters are applied client-side for visualization only.
     */
    function agentPassesFilters(agent) {
        // Status filter
        if (filters.status === 'alive' && !agent.alive) return false;

        // Dominant trait filter
        if (filters.trait !== 'all' && agent.genome_gains && agent.genome_gains.length === 5) {
            const maxGain = Math.max(...agent.genome_gains);
            const maxIndex = agent.genome_gains.indexOf(maxGain);
            const traitKeys = ['d', 'o', 'c', 'a', 's'];
            const dominantTrait = traitKeys[maxIndex];
            if (dominantTrait !== filters.trait) return false;
        }

        return true;
    }

    /**
     * Get current tick
     */
    function getCurrentTick() {
        return currentTick;
    }

    /**
     * Get HSL color from CSS variable
     */
    function getHslColor(varName, fallback) {
        try {
            const value = getComputedStyle(document.documentElement)
                .getPropertyValue(varName).trim();
            if (!value) return fallback;
            const parts = value.split(/\s+/);
            if (parts.length >= 3) {
                return `hsl(${parts[0]}, ${parts[1]}, ${parts[2]})`;
            }
            return fallback;
        } catch (e) {
            return fallback;
        }
    }

    /**
     * Decoupled 60Hz requestAnimationFrame rendering loop (Master Directive Part 3)
     */
    function startRenderLoop() {
        if (isRenderLoopActive) return;
        isRenderLoopActive = true;
        let frameCount = 0;
        let fpsTimer = performance.now();

        function renderLoopStep(now) {
            if (!isRenderLoopActive) return;
            const start = performance.now();
            render();
            const duration = performance.now() - start;

            renderPerf.lastRenderMs = duration;
            renderPerf.renderTimes.push(duration);
            if (renderPerf.renderTimes.length > 60) renderPerf.renderTimes.shift();

            const sorted = [...renderPerf.renderTimes].sort((a, b) => a - b);
            renderPerf.p99RenderMs = sorted[Math.floor(sorted.length * 0.99)] || duration;

            frameCount++;
            if (now - fpsTimer >= 1000) {
                renderPerf.fps = Math.round((frameCount * 1000) / (now - fpsTimer));
                frameCount = 0;
                fpsTimer = now;
                const fpsEl = document.getElementById('stat-fps');
                if (fpsEl) fpsEl.textContent = renderPerf.fps;
            }

            if (selectedAgentId !== null) {
                updateInspectorModal();
            }

            rafId = requestAnimationFrame(renderLoopStep);
        }

        rafId = requestAnimationFrame(renderLoopStep);
    }

    function stopRenderLoop() {
        isRenderLoopActive = false;
        if (rafId) {
            cancelAnimationFrame(rafId);
            rafId = null;
        }
    }

    /**
     * Render current frame to canvas
     * Per SIM-002: This is a visualization-only rendering function.
     * It reads simulation state but does not modify it.
     */
    function render() {
        if (!ctx || !canvas) {
            return;
        }

        const width = canvas.clientWidth;
        const height = canvas.clientHeight;

        // Reset transform to avoid compounding scale issues
        const dpr = window.devicePixelRatio || 1;
        ctx.setTransform(1, 0, 0, 1, 0, 0);
        ctx.scale(dpr, dpr);

        // Clear entire canvas
        ctx.fillStyle = '#0a0a0c';
        ctx.fillRect(0, 0, canvas.width / dpr, canvas.height / dpr);

        // Apply CSS variable background
        const bgColor = getHslColor('--background', '#0a0a0c');
        ctx.fillStyle = bgColor;
        ctx.fillRect(0, 0, width, height);

        // Draw grid background (slightly lighter than background)
        ctx.fillStyle = 'rgba(255, 255, 255, 0.03)';
        ctx.fillRect(offsetX, offsetY, gridWidth * scale, gridHeight * scale);

        // Draw terrain elevation layer (contour lines)
        renderTerrain();

        // Draw weather layer (dynamic overlay)
        renderWeather();

        // Draw resource distribution layer
        renderResources();

        // Draw harvesting activity overlay (white rippling lines)
        renderHarvesting();

        // Draw grid lines (always visible per refactor spec)
        drawGrid();

        // Get current frame data from live stream
        if (!currentFrame || !currentFrame.agents || currentFrame.agents.length === 0) return;

        // Build agent position lookup for edge rendering (continuous sub-pixel interpolation)
        const agentPositions = {};
        for (const agent of currentFrame.agents) {
            if (agent.alive) {
                const interpPos = getInterpolatedPosition(agent);
                agentPositions[agent.agent_id] = {
                    x: offsetX + (interpPos.x + 0.5) * scale,
                    y: offsetY + (interpPos.y + 0.5) * scale,
                    tau: agent.tau || 0.5
                };
            }
        }

        // Render spatial clustering group hulls (behind network)
        if (layers.groups) {
            renderGroupHulls();
        }

        // Render trust network edges FIRST (behind agents)
        if ((layers.network || layers.relative) && currentFrame.trust_edges) {
            renderTrustNetwork(currentFrame.trust_edges, agentPositions);
        }

        // Calculate population statistics for reputation rendering
        const populationStats = calculatePopulationStats(currentFrame.agents);

        // Render agents (applying filters)
        if (layers.agents) {
            for (const agent of currentFrame.agents) {
                if (agentPassesFilters(agent)) {
                    renderAgent(agent, populationStats);
                }
            }
        }

        // Draw Civilization Challenge HUD badges (Generations Survived & Gini Index)
        drawCivilizationHUD();
    }

    /**
     * Draw Civilization Challenge HUD badges (Generations Survived & Gini Index)
     */
    function drawCivilizationHUD() {
        if (!currentFrame) return;
        const width = canvas.clientWidth;
        const generations = Math.floor(currentTick / 100) + 1;
        const aliveCount = currentFrame.agents ? currentFrame.agents.filter(a => a.alive).length : 0;
        const energies = currentFrame.agents && aliveCount > 0
            ? currentFrame.agents.filter(a => a.alive).map(a => a.energy)
            : [];
        
        let gini = 0.0;
        if (energies.length > 1) {
            energies.sort((a, b) => a - b);
            let n = energies.length;
            let sum = energies.reduce((acc, v) => acc + v, 0);
            if (sum > 0) {
                let numer = 0;
                for (let i = 0; i < n; i++) {
                    numer += (2 * (i + 1) - n - 1) * energies[i];
                }
                gini = Math.max(0, Math.min(1, numer / (n * sum)));
            }
        }

        ctx.save();
        const badgeX = width - 210;
        const badgeY = 16;
        ctx.fillStyle = 'rgba(10, 10, 18, 0.85)';
        ctx.strokeStyle = 'rgba(255, 255, 255, 0.15)';
        ctx.lineWidth = 1;
        ctx.beginPath();
        if (ctx.roundRect) {
            ctx.roundRect(badgeX, badgeY, 195, 52, 6);
        } else {
            ctx.rect(badgeX, badgeY, 195, 52);
        }
        ctx.fill();
        ctx.stroke();

        ctx.font = '10px Inter, monospace';
        ctx.fillStyle = '#94a3b8';
        ctx.fillText('CIVILIZATION CHALLENGE', badgeX + 10, badgeY + 16);

        ctx.font = 'bold 12px Inter, sans-serif';
        ctx.fillStyle = generations >= 10 ? '#22c55e' : '#f8fafc';
        ctx.fillText(`Gen: ${generations} (${aliveCount} alive)`, badgeX + 10, badgeY + 34);

        ctx.font = '11px Inter, sans-serif';
        ctx.fillStyle = gini > 0.60 ? '#ef4444' : gini > 0.45 ? '#eab308' : '#38bdf8';
        ctx.fillText(`Gini: ${gini.toFixed(3)}`, badgeX + 125, badgeY + 34);

        ctx.restore();
    }

    let cachedGridCanvas = null;
    let isGridCacheValid = false;

    function buildGridCache() {
        if (!cachedGridCanvas) {
            cachedGridCanvas = document.createElement('canvas');
        }
        const pixelWidth = Math.ceil(gridWidth * scale);
        const pixelHeight = Math.ceil(gridHeight * scale);
        if (pixelWidth <= 0 || pixelHeight <= 0) return;
        cachedGridCanvas.width = pixelWidth;
        cachedGridCanvas.height = pixelHeight;
        const gCtx = cachedGridCanvas.getContext('2d');
        if (!gCtx) return;

        gCtx.clearRect(0, 0, pixelWidth, pixelHeight);
        const gridColor = getHslColor('--border', 'hsl(240, 3.7%, 20%)');
        gCtx.strokeStyle = gridColor;
        gCtx.lineWidth = 1;

        // Minor grid lines (every cell)
        gCtx.globalAlpha = 0.15;
        gCtx.beginPath();
        for (let x = 0; x <= gridWidth; x++) {
            const px = Math.round(x * scale);
            gCtx.moveTo(px, 0);
            gCtx.lineTo(px, pixelHeight);
        }
        for (let y = 0; y <= gridHeight; y++) {
            const py = Math.round(y * scale);
            gCtx.moveTo(0, py);
            gCtx.lineTo(pixelWidth, py);
        }
        gCtx.stroke();

        // Major grid lines (every 8 cells)
        gCtx.globalAlpha = 0.4;
        gCtx.beginPath();
        const majorStep = 8;
        for (let x = 0; x <= gridWidth; x += majorStep) {
            const px = Math.round(x * scale);
            gCtx.moveTo(px, 0);
            gCtx.lineTo(px, pixelHeight);
        }
        for (let y = 0; y <= gridHeight; y += majorStep) {
            const py = Math.round(y * scale);
            gCtx.moveTo(0, py);
            gCtx.lineTo(pixelWidth, py);
        }
        gCtx.stroke();
        isGridCacheValid = true;
    }

    function drawGrid() {
        if (!isGridCacheValid || !cachedGridCanvas) {
            buildGridCache();
        }
        if (cachedGridCanvas) {
            ctx.save();
            ctx.globalAlpha = 1.0;
            ctx.drawImage(cachedGridCanvas, offsetX, offsetY);
            ctx.restore();
        }
    }

    /**
     * Render group spatial clustering convex/centroid hulls
     */
    function renderGroupHulls() {
        if (!layers.groups || !currentFrame || !currentFrame.agents) return;
        const groupPoints = new Map();
        for (const agent of currentFrame.agents) {
            if (!agent.alive || agent.group_id === null || agent.group_id === undefined) continue;
            if (!groupPoints.has(agent.group_id)) groupPoints.set(agent.group_id, []);
            const interpPos = getInterpolatedPosition(agent);
            groupPoints.get(agent.group_id).push({
                x: offsetX + (interpPos.x + 0.5) * scale,
                y: offsetY + (interpPos.y + 0.5) * scale
            });
        }

        ctx.save();
        ctx.globalAlpha = layerOpacity.groups * 0.25;
        for (const [groupId, points] of groupPoints) {
            if (points.length < 3) continue;
            const hue = (groupId * 137) % 360;
            ctx.fillStyle = `hsl(${hue}, 70%, 45%)`;
            ctx.strokeStyle = `hsl(${hue}, 80%, 60%)`;
            ctx.lineWidth = 1.5;

            const cx = points.reduce((s, p) => s + p.x, 0) / points.length;
            const cy = points.reduce((s, p) => s + p.y, 0) / points.length;
            const sorted = [...points].sort((a, b) => Math.atan2(a.y - cy, a.x - cx) - Math.atan2(b.y - cy, b.x - cx));

            ctx.beginPath();
            ctx.moveTo(sorted[0].x, sorted[0].y);
            for (let i = 1; i < sorted.length; i++) {
                ctx.lineTo(sorted[i].x, sorted[i].y);
            }
            ctx.closePath();
            ctx.fill();
            ctx.stroke();
        }
        ctx.restore();
    }

    /**
     * Render trust relationship edges with smooth Bezier curves and viewport culling
     */
    function renderTrustNetwork(edges, agentPositions) {
        if (!edges || edges.length === 0) return;

        const width = canvas.clientWidth;
        const height = canvas.clientHeight;
        ctx.save();

        for (const edge of edges) {
            const fromPos = agentPositions[edge.from_id];
            const toPos = agentPositions[edge.to_id];

            if (!fromPos || !toPos) continue;

            // Viewport spatial culling
            const minX = Math.min(fromPos.x, toPos.x);
            const maxX = Math.max(fromPos.x, toPos.x);
            const minY = Math.min(fromPos.y, toPos.y);
            const maxY = Math.max(fromPos.y, toPos.y);
            if (maxX < 0 || minX > width || maxY < 0 || minY > height) continue;

            const isGroupEdge = edge.edge_type === 'GroupMember';
            const isRelativeEdge = edge.edge_type === 'Lineage';

            if (isGroupEdge && !layers.network) continue;
            if (isRelativeEdge && !layers.relative) continue;

            if (isRelativeEdge && filters.relativeDistanceThreshold > 0) {
                const dx = toPos.x - fromPos.x;
                const dy = toPos.y - fromPos.y;
                const distance = Math.sqrt(dx * dx + dy * dy) / scale;
                if (distance > filters.relativeDistanceThreshold) continue;
            }

            let color, alpha;
            if (isRelativeEdge) {
                color = `hsla(280, 60%, 50%, ${(0.2 + edge.weight * 0.5) * layerOpacity.relative})`;
                alpha = (0.4 + edge.weight * 0.4) * layerOpacity.relative;
            } else {
                const baseAlpha = layerOpacity.network;
                const avgTau = (fromPos.tau + toPos.tau) / 2;
                if (avgTau > 1.2) {
                    color = `hsla(142, 50%, 50%, ${(0.2 + edge.weight * 0.5) * baseAlpha})`;
                } else if (avgTau < 0.8) {
                    color = `hsla(0, 60%, 50%, ${(0.2 + edge.weight * 0.4) * baseAlpha})`;
                } else {
                    color = `hsla(45, 70%, 50%, ${(0.2 + edge.weight * 0.5) * baseAlpha})`;
                }
                alpha = (0.3 + edge.weight * 0.4) * baseAlpha;
            }

            // Smooth quadratic Bezier curve
            const midX = (fromPos.x + toPos.x) / 2;
            const midY = (fromPos.y + toPos.y) / 2;
            const curveOffset = (edge.from_id % 7 - 3) * scale * 0.25;

            ctx.beginPath();
            ctx.moveTo(fromPos.x, fromPos.y);
            ctx.quadraticCurveTo(midX + curveOffset, midY - curveOffset, toPos.x, toPos.y);
            ctx.strokeStyle = color;
            ctx.lineWidth = 1 + edge.weight * 1.5;
            ctx.globalAlpha = alpha;
            ctx.stroke();
        }

        ctx.restore();
    }

    // Rendering configuration
    let renderConfig = {
        snapToGrid: true,      // Snap agents to cell centers for discrete movement
        cellSize: 1.0,         // Size of one grid cell in simulation units
        agentSize: 0.35,       // Agent radius as fraction of cell size
        showDirectionArrows: true,  // Show 8-directional movement indicators (Phase 2)
        zoom: 1.0,             // Zoom level (Phase 4)
        panX: 0,               // Pan offset X (Phase 4)
        panY: 0                // Pan offset Y (Phase 4)
    };

    /**
     * SPATIAL HASH (Phase 4: Performance Optimization)
     * 
     * Provides O(1) agent lookups for click detection and neighbor queries.
     * Maps cell coordinates to lists of agent IDs for fast spatial queries.
     */
    let spatialHash = new Map(); // Key: "x,y", Value: [agent_ids]
    let frameBuffer = [];        // Buffer of recent frames for replay (Phase 4)
    const FRAME_BUFFER_SIZE = 30; // Max frames to buffer - reduced from 100 for memory efficiency

    /**
     * Build spatial hash from current frame
     * Call this after receiving a new frame
     */
    function buildSpatialHash() {
        spatialHash.clear();
        
        if (!currentFrame || !currentFrame.agents) return;
        
        for (const agent of currentFrame.agents) {
            // Quantize to cell coordinates
            const cellX = Math.floor(agent.x);
            const cellY = Math.floor(agent.y);
            const key = `${cellX},${cellY}`;
            
            if (!spatialHash.has(key)) {
                spatialHash.set(key, []);
            }
            spatialHash.get(key).push(agent.agent_id);
        }
    }

    /**
     * Add frame to buffer for replay functionality
     * Eliminates V8 JSON stringify GC thrash via fast structured object copying
     */
    function bufferFrame(frame) {
        if (!frame || !frame.agents) return;
        const compactAgents = new Array(frame.agents.length);
        for (let i = 0; i < frame.agents.length; i++) {
            const a = frame.agents[i];
            compactAgents[i] = {
                agent_id: a.agent_id,
                x: a.x,
                y: a.y,
                z: a.z || 0,
                energy: a.energy,
                docas: a.docas ? [a.docas[0], a.docas[1], a.docas[2], a.docas[3], a.docas[4]] : [0.5, 0.5, 0.5, 0.5, 0.5],
                genome_gains: a.genome_gains ? [a.genome_gains[0], a.genome_gains[1], a.genome_gains[2], a.genome_gains[3], a.genome_gains[4]] : [0.1, 0.1, 0.1, 0.1, 0.1],
                alive: a.alive,
                has_identity: a.has_identity || a.has_verified_id,
                group_id: a.group_id,
                age: a.age,
                tau: a.tau || 0.5,
                reputation: a.reputation || 0.5,
                last_dx: a.last_dx || 0,
                last_dy: a.last_dy || 0
            };
        }
        frameBuffer.push({
            tick: frame.tick,
            agents: compactAgents,
            stats: frame.stats ? { ...frame.stats } : {},
            timestamp: performance.now()
        });
        
        // Keep buffer size limited
        if (frameBuffer.length > FRAME_BUFFER_SIZE) {
            frameBuffer.shift();
        }
    }

    /**
     * P0 FIX: Cleanup agent position history to prevent memory leaks
     * Removes dead agents from history after 100 ticks
     */
    function cleanupAgentHistory(currentAgents) {
        const aliveIds = new Set(currentAgents.map((a) => a.agent_id));
        const allIds = Object.keys(debugState.agentPositionHistory);
        
        for (const id of allIds) {
            const agentId = parseInt(id);
            if (!aliveIds.has(agentId)) {
                const history = debugState.agentPositionHistory[id];
                const lastEntry = history[history.length - 1];
                if (lastEntry && currentTick - lastEntry.tick > 100) {
                    delete debugState.agentPositionHistory[id];
                    delete debugState.movementStats.frozenAgentTicks[agentId];
                }
            }
        }
    }

    /**
     * Get frame from buffer by tick
     */
    function getBufferedFrame(tick) {
        return frameBuffer.find(f => f.tick === tick);
    }

    /**
     * Set zoom level
     */
    function setZoom(zoom) {
        renderConfig.zoom = Math.max(0.5, Math.min(3.0, zoom));
        render();
    }

    /**
     * Set pan offset
     */
    function setPan(x, y) {
        renderConfig.panX = x;
        renderConfig.panY = y;
        render();
    }

    /**
     * GRID CONFIGURATION (Phase 2: Sugarscape-style)
     * 
     * Grid dimensions are configurable to match sim-core environment.rs
     * Per Sugarscape best practices: 10-level color gradients for resources
     */
    let gridConfig = {
        width: 64,            // Match sim-core (configurable: 32-256)
        height: 64,           // Match sim-core
        cellSize: 10,         // pixels per cell
        showMinor: true,      // All cell boundaries visible
        minorOpacity: 0.15,   // Subtle cell lines
        majorStep: 8,         // Highlight every 8th line
        majorOpacity: 0.4,    // More visible major lines
        // Resource visualization (Terrain layer)
        terrainColors: [
            'hsla(30, 70%, 20%, 0.3)',   // Level 0: Very low (dark brown)
            'hsla(35, 65%, 25%, 0.35)',  // Level 1
            'hsla(40, 60%, 30%, 0.4)',   // Level 2
            'hsla(45, 70%, 35%, 0.45)',  // Level 3
            'hsla(50, 75%, 40%, 0.5)',   // Level 4: Medium (yellow)
            'hsla(55, 80%, 45%, 0.55)',  // Level 5
            'hsla(60, 85%, 50%, 0.6)',   // Level 6
            'hsla(65, 80%, 55%, 0.65)',  // Level 7
            'hsla(70, 75%, 60%, 0.7)',   // Level 8
            'hsla(75, 70%, 65%, 0.75)'   // Level 9: Very high (light yellow)
        ]
    };

    /**
     * Update grid configuration
     * Called when simulation parameters change
     */
    function setGridConfig(config) {
        Object.assign(gridConfig, config);
        gridWidth = gridConfig.width;
        gridHeight = gridConfig.height;
        updateScale();
        render();
        console.log('[Viewer] Grid config updated:', gridConfig);
    }

    /**
     * Get direction from movement delta
     * Returns one of 8 directional arrows or null
     */
    function getDirectionSymbol(dx, dy) {
        // Normalize to integer deltas
        const ndx = Math.round(dx);
        const ndy = Math.round(dy);
        
        // 8-directional mapping
        const directions = {
            '0,-1': '↑',   // North
            '0,1': '↓',    // South
            '1,0': '→',    // East
            '-1,0': '←',   // West
            '1,-1': '↗',   // North-East
            '-1,-1': '↖',  // North-West
            '1,1': '↘',    // South-East
            '-1,1': '↙'    // South-West
        };
        
        return directions[`${ndx},${ndy}`] || null;
    }

    /**
     * Draw direction arrow for agent movement
     */
    function drawDirectionArrow(x, y, radius, dx, dy) {
        if (!renderConfig.showDirectionArrows) return;
        
        const symbol = getDirectionSymbol(dx, dy);
        if (!symbol) return;
        
        ctx.save();
        ctx.fillStyle = 'rgba(255, 255, 255, 0.8)';
        ctx.font = `${Math.max(8, radius)}px JetBrains Mono, monospace`;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(symbol, x, y);
        ctx.restore();
    }

    let weatherTileCanvas = null;
    let weatherTileCtx = null;
    let weatherImgData = null;
    let weatherUint32 = null;
    let isWeatherTileValid = false;

    /**
     * Direct Uint32Array ImageData pixel blitting for dynamic weather overlay
     */
    function updateWeatherTile(weatherData) {
        if (!weatherData) return;
        if (!weatherTileCanvas || weatherTileCanvas.width !== gridWidth || weatherTileCanvas.height !== gridHeight) {
            weatherTileCanvas = document.createElement('canvas');
            weatherTileCanvas.width = gridWidth;
            weatherTileCanvas.height = gridHeight;
            weatherTileCtx = weatherTileCanvas.getContext('2d');
            weatherImgData = weatherTileCtx.createImageData(gridWidth, gridHeight);
            weatherUint32 = new Uint32Array(weatherImgData.data.buffer);
        }

        // Little-endian RGBA packing (0xAABBGGRR)
        for (let i = 0; i < gridWidth * gridHeight; i++) {
            const val = weatherData[i] || 0;
            let r, g, b, a;
            if (val < 0.2) {
                const t = val / 0.2;
                r = 255; g = Math.round(200 + t * 55); b = Math.round(100 + t * 100); a = 150;
            } else if (val < 0.4) {
                const t = (val - 0.2) / 0.2;
                r = Math.round(180 - t * 80); g = Math.round(180 - t * 80); b = Math.round(180 - t * 40); a = 175;
            } else if (val < 0.6) {
                const t = (val - 0.4) / 0.2;
                r = Math.round(50 + t * 30); g = Math.round(100 + t * 50); b = Math.round(200 + t * 55); a = 200;
            } else if (val < 0.8) {
                const t = (val - 0.6) / 0.2;
                r = Math.round(100 + t * 80); g = Math.round(50 + t * 30); b = Math.round(150 + t * 80); a = 215;
            } else {
                const t = (val - 0.8) / 0.2;
                r = Math.round(220 + t * 35); g = Math.round(150 - t * 50); b = Math.round(50 + t * 30); a = 225;
            }
            weatherUint32[i] = (a << 24) | (b << 16) | (g << 8) | r;
        }
        weatherTileCtx.putImageData(weatherImgData, 0, 0);
        isWeatherTileValid = true;
    }

    /**
     * Render weather layer via fast offscreen blitting (Strict Epistemic Alignment)
     */
    function renderWeather() {
        if (!layers.weather) return;
        const weatherData = currentFrame?.weather || cachedWeather;
        if (!weatherData) return; // Strict epistemic alignment: zero cosmetic weather hallucination

        if (!isWeatherTileValid && weatherData) {
            updateWeatherTile(weatherData);
        }
        if (weatherTileCanvas && isWeatherTileValid) {
            ctx.save();
            ctx.globalAlpha = layerOpacity.weather;
            ctx.imageSmoothingEnabled = false;
            ctx.drawImage(weatherTileCanvas, offsetX, offsetY, gridWidth * scale, gridHeight * scale);
            ctx.restore();
        }
    }

    /**
     * Render terrain elevation layer
     * Shows elevation as white contour lines on transparent background
     * Topographic map style with contour intervals
     */
    let cachedTerrainCanvas = null;
    let isTerrainCacheValid = false;

    /**
     * Draw contour line segments across a single grid cell if threshold crosses edges
     */
    function drawCellContourSegment(tCtx, x, y, scale, threshold, topLeft, topRight, bottomLeft, bottomRight) {
        const crosses = (topLeft < threshold && topRight >= threshold) ||
                       (topLeft >= threshold && topRight < threshold) ||
                       (topLeft < threshold && bottomLeft >= threshold) ||
                       (topLeft >= threshold && bottomLeft < threshold) ||
                       (topRight < threshold && bottomRight >= threshold) ||
                       (topRight >= threshold && bottomRight < threshold) ||
                       (bottomLeft < threshold && bottomRight >= threshold) ||
                       (bottomLeft >= threshold && bottomRight < threshold);
        if (!crosses) return;

        const px = x * scale;
        const py = y * scale;
        tCtx.beginPath();
        if ((topLeft < threshold) !== (topRight < threshold)) {
            const t = (threshold - topLeft) / (topRight - topLeft);
            tCtx.moveTo(px + t * scale, py);
            tCtx.lineTo(px + t * scale, py + scale);
        }
        if ((topLeft < threshold) !== (bottomLeft < threshold)) {
            const t = (threshold - topLeft) / (bottomLeft - topLeft);
            tCtx.moveTo(px, py + t * scale);
            tCtx.lineTo(px + scale, py + t * scale);
        }
        tCtx.stroke();
    }

    /**
     * Build offscreen Marching Squares terrain contour cache (reduces 100,000 evaluations to 1 drawImage)
     */
    function buildTerrainCache(terrainData) {
        if (!terrainData) return;
        if (!cachedTerrainCanvas) {
            cachedTerrainCanvas = document.createElement('canvas');
        }
        const pixelWidth = Math.ceil(gridWidth * scale);
        const pixelHeight = Math.ceil(gridHeight * scale);
        if (pixelWidth <= 0 || pixelHeight <= 0) return;
        cachedTerrainCanvas.width = pixelWidth;
        cachedTerrainCanvas.height = pixelHeight;
        const tCtx = cachedTerrainCanvas.getContext('2d');
        if (!tCtx) return;

        tCtx.clearRect(0, 0, pixelWidth, pixelHeight);
        const contourInterval = 0.1;
        const numContours = Math.floor(1.0 / contourInterval);
        tCtx.lineWidth = 1.5;
        tCtx.strokeStyle = 'rgba(255, 255, 255, 0.7)';

        for (let c = 1; c <= numContours; c++) {
            const threshold = c * contourInterval;
            for (let y = 0; y < gridHeight - 1; y++) {
                for (let x = 0; x < gridWidth - 1; x++) {
                    const cellIndex = y * gridWidth + x;
                    const topLeft = terrainData[cellIndex] || 0;
                    const topRight = terrainData[cellIndex + 1] || 0;
                    const bottomLeft = terrainData[cellIndex + gridWidth] || 0;
                    const bottomRight = terrainData[cellIndex + gridWidth + 1] || 0;
                    drawCellContourSegment(tCtx, x, y, scale, threshold, topLeft, topRight, bottomLeft, bottomRight);
                }
            }
        }
        isTerrainCacheValid = true;
    }

    /**
     * Render terrain elevation layer via cached offscreen contour canvas (Strict Epistemic Alignment)
     */
    function renderTerrain() {
        if (!layers.terrain) return;
        const terrainData = currentFrame?.terrain || cachedTerrain;
        if (!terrainData) return; // Strict epistemic alignment: zero cosmetic terrain hallucination

        if (!isTerrainCacheValid || !cachedTerrainCanvas) {
            buildTerrainCache(terrainData);
        }
        if (cachedTerrainCanvas && isTerrainCacheValid) {
            ctx.save();
            ctx.globalAlpha = layerOpacity.terrain;
            ctx.drawImage(cachedTerrainCanvas, offsetX, offsetY);
            ctx.restore();
        }
    }
    
    /**
     * Get terrain color based on elevation
     * Uses a gradient from valleys (dark green) to mountains (white/gray)
     */
    function getTerrainColor(elevation) {
        // Elevation breakpoints and colors
        if (elevation < 0.2) {
            // Deep valleys: Dark green
            return `hsla(120, 60%, ${15 + elevation * 100}%, 0.6)`;
        } else if (elevation < 0.4) {
            // Lowlands: Green to yellow-green
            const t = (elevation - 0.2) / 0.2;
            return `hsla(${120 - t * 40}, 50%, ${35 + t * 15}%, 0.5)`;
        } else if (elevation < 0.6) {
            // Mid-elevation: Yellow to tan
            const t = (elevation - 0.4) / 0.2;
            return `hsla(${80 - t * 30}, ${50 - t * 20}%, ${50 + t * 10}%, 0.5)`;
        } else if (elevation < 0.8) {
            // Highlands: Tan to brown
            const t = (elevation - 0.6) / 0.2;
            return `hsla(${35 - t * 15}, ${30 + t * 10}%, ${60 - t * 20}%, 0.5)`;
        } else {
            // Mountain peaks: Brown to gray/white
            const t = (elevation - 0.8) / 0.2;
            return `hsla(${20 - t * 20}, ${40 - t * 30}%, ${40 + t * 45}%, 0.6)`;
        }
    }

    /**
     * Render resource distribution layer
     * Shows food/energy sources available for consumption
     * Phase 2: Sugarscape-style resource visualization
     * 
     * FIXED: Uses cached resources to prevent flashing when server only sends
     * resources every 10 ticks (bandwidth optimization)
     */
    let resourceTileCanvas = null;
    let resourceTileCtx = null;
    let resourceImgData = null;
    let resourceUint32 = null;
    let isResourceTileValid = false;

    // Sugarscape 10-level ABGR colors (little-endian 0xAABBGGRR)
    const SUGARSCAPE_ABGR = [
        (76 << 24) | (15 << 16) | (35 << 8) | (87),   // Level 0 (dark brown)
        (89 << 24) | (22 << 16) | (52 << 8) | (105),  // Level 1
        (102 << 24) | (31 << 16) | (71 << 8) | (122), // Level 2
        (115 << 24) | (27 << 16) | (95 << 8) | (152), // Level 3
        (128 << 24) | (26 << 16) | (122 << 8) | (178),// Level 4 (medium yellow)
        (140 << 24) | (23 << 16) | (154 << 8) | (207),// Level 5
        (153 << 24) | (19 << 16) | (187 << 8) | (236),// Level 6
        (166 << 24) | (48 << 16) | (197 << 8) | (232),// Level 7
        (178 << 24) | (77 << 16) | (207 << 8) | (229),// Level 8
        (191 << 24) | (106 << 16) | (217 << 8) | (225) // Level 9 (light yellow)
    ];

    function updateResourceTile(resources) {
        if (!resources) return;
        if (!resourceTileCanvas || resourceTileCanvas.width !== gridWidth || resourceTileCanvas.height !== gridHeight) {
            resourceTileCanvas = document.createElement('canvas');
            resourceTileCanvas.width = gridWidth;
            resourceTileCanvas.height = gridHeight;
            resourceTileCtx = resourceTileCanvas.getContext('2d');
            resourceImgData = resourceTileCtx.createImageData(gridWidth, gridHeight);
            resourceUint32 = new Uint32Array(resourceImgData.data.buffer);
        }

        for (let i = 0; i < gridWidth * gridHeight; i++) {
            const res = resources[i];
            if (res && res.amount > 0) {
                const cap = res.capacity || 100;
                const level = Math.min(9, Math.floor((res.amount / cap) * 10));
                resourceUint32[i] = SUGARSCAPE_ABGR[level];
            } else {
                resourceUint32[i] = 0; // Transparent
            }
        }
        resourceTileCtx.putImageData(resourceImgData, 0, 0);
        isResourceTileValid = true;
    }

    /**
     * Render resource distribution layer via fast offscreen tile blit
     */
    function renderResources() {
        if (!layers.resources) return;
        const resources = currentFrame?.resources || cachedResources;
        if (!resources) return;

        if (!isResourceTileValid && resources) {
            updateResourceTile(resources);
        }
        if (resourceTileCanvas && isResourceTileValid) {
            ctx.save();
            ctx.globalAlpha = layerOpacity.resources;
            ctx.imageSmoothingEnabled = false;
            ctx.drawImage(resourceTileCanvas, offsetX, offsetY, gridWidth * scale, gridHeight * scale);
            ctx.restore();
        }
    }

    /**
     * Render harvesting activity overlay
     * Shows white rippling lines emanating from agents that are actively harvesting
     * Less distracting but still clear indication of active consumption
     */
    function renderHarvesting() {
        if (!layers.harvesting || !currentFrame?.agents) return;
        
        ctx.save();
        ctx.globalAlpha = layerOpacity.harvesting;
        
        const agents = currentFrame.agents;
        
        for (const agent of agents) {
            if (!agent.alive) continue;
            
            // Check if agent is actively harvesting
            // Visual indicator: agents with high energy or explicit harvesting flag
            const isHarvesting = agent.is_harvesting || agent.energy > 70;
            
            if (isHarvesting) {
                const interpPos = getInterpolatedPosition(agent);
                const cellX = Math.floor(interpPos.x);
                const cellY = Math.floor(interpPos.y);
                const x = offsetX + (cellX + 0.5) * scale;
                const y = offsetY + (cellY + 0.5) * scale;
                
                // Draw white rippling lines emanating from agent
                const baseRadius = Math.max(4, scale * renderConfig.agentSize);
                const maxRadius = baseRadius + scale * 1.5;
                const rippleCount = 3;
                
                ctx.save();
                ctx.strokeStyle = 'rgba(255, 255, 255, 0.7)';
                ctx.lineWidth = 1.5;
                ctx.lineCap = 'round';
                
                // Create rippling effect with multiple arcs
                for (let r = 0; r < rippleCount; r++) {
                    // Each ripple has a different phase
                    const ripplePhase = ((currentTick + r * 10) % 30) / 30; // 0-1 cycle, offset per ripple
                    const radius = baseRadius + (maxRadius - baseRadius) * ripplePhase;
                    const alpha = 0.8 * (1 - ripplePhase); // Fade out as ripple expands
                    
                    ctx.strokeStyle = `rgba(255, 255, 255, ${alpha})`;
                    ctx.lineWidth = 1.5 * (1 - ripplePhase * 0.5); // Thin as it expands
                    
                    // Draw arc segments (lines emanating outward)
                    const segmentCount = 6;
                    for (let s = 0; s < segmentCount; s++) {
                        const baseAngle = (s / segmentCount) * Math.PI * 2;
                        const angleOffset = currentTick * 0.05; // Slow rotation
                        const startAngle = baseAngle + angleOffset;
                        const endAngle = startAngle + Math.PI / segmentCount;
                        
                        ctx.beginPath();
                        ctx.arc(x, y, radius, startAngle, endAngle);
                        ctx.stroke();
                    }
                }
                
                ctx.restore();
            }
        }
        
        ctx.restore();
    }
    
    /**
     * Calculate population statistics for reputation rendering
     * Returns mean and standard deviation of reputation values
     */
    function calculatePopulationStats(agents) {
        if (!agents || agents.length === 0) {
            return { mean: 0.5, stdDev: 0.2, count: 0 };
        }
        
        const aliveAgents = agents.filter(a => a.alive);
        if (aliveAgents.length === 0) {
            return { mean: 0.5, stdDev: 0.2, count: 0 };
        }
        
        // Calculate mean reputation
        const reputations = aliveAgents.map(a => a.reputation !== undefined ? a.reputation : 0.5);
        const mean = reputations.reduce((sum, r) => sum + r, 0) / reputations.length;
        
        // Calculate standard deviation
        const variance = reputations.reduce((sum, r) => sum + Math.pow(r - mean, 2), 0) / reputations.length;
        const stdDev = Math.sqrt(variance);
        
        return { mean, stdDev, count: aliveAgents.length };
    }
    
    /**
     * Get reputation color based on graded curve relative to population
     * Uses bell curve distribution: red for below average, green for above average
     */
    function getReputationColor(reputation, stats) {
        if (stats.count === 0) return 'hsl(0, 0%, 50%)';
        
        const { mean, stdDev } = stats;
        
        // Calculate z-score (standard deviations from mean)
        const zScore = stdDev > 0 ? (reputation - mean) / stdDev : 0;
        
        // Map z-score to color
        // z < -1: Deep red (well below average)
        // z = 0: Yellow (average)
        // z > 1: Bright green (well above average)
        
        if (zScore < -1) {
            // Well below average: Deep red
            return 'hsl(0, 80%, 45%)';
        } else if (zScore < 0) {
            // Below average: Gradient from red to yellow
            const t = zScore + 1; // 0 to 1
            return `hsl(${t * 60}, 80%, ${45 + t * 5}%)`;
        } else if (zScore < 1) {
            // Above average: Gradient from yellow to green
            const t = zScore; // 0 to 1
            return `hsl(${60 + t * 60}, 80%, 50%)`;
        } else {
            // Well above average: Bright green
            return 'hsl(120, 80%, 50%)';
        }
    }

    /**
     * Get interpolated agent position for smooth motion
     * Phase 1: Motion smoothing between frames for better clickability
     */
    function getInterpolatedPosition(agent) {
        // Default to current position
        let posX = agent.x;
        let posY = agent.y;
        
        // If we have a previous frame, interpolate between positions
        // P0 FIX: Use agentMap for O(1) lookup instead of O(n) find()
        if (previousFrame && previousFrame.agentMap) {
            const prevAgent = previousFrame.agentMap.get(agent.agent_id);
            if (prevAgent) {
                const elapsed = performance.now() - frameInterpStartTime;
                const t = Math.min(elapsed / Math.max(16, frameDurationMs), 1.0);
                
                // Linear interpolation
                posX = prevAgent.x + (agent.x - prevAgent.x) * t;
                posY = prevAgent.y + (agent.y - prevAgent.y) * t;
            }
        }
        
        return { x: posX, y: posY };
    }

    /**
     * Render a single agent
     * Agents are rendered centered in their grid cells for Sugarscape-style clarity
     * Phase 1: Added motion interpolation for smooth movement
     * Phase 2: Added movement direction indicators
     * Part 3: Added left-to-right shimmer animation for "alive" effect
     * Updated: Added identity badge and reputation ring
     */
    function renderAgent(agent, populationStats) {
        // Phase 1: Get interpolated position for smooth motion
        const interpPos = getInterpolatedPosition(agent);
        
        // Continuous sub-pixel positioning for silky smooth 60fps movement
        const renderX = offsetX + (interpPos.x + 0.5) * scale;
        const renderY = offsetY + (interpPos.y + 0.5) * scale;
        
        const x = renderX;
        const y = renderY;
        const baseRadius = Math.max(3, scale * renderConfig.agentSize);

        // Part 3: Left-to-right shimmer animation for alive agents
        // Creates a shimmering wave effect that moves across the agent
        let shimmerOffset = 0;
        if (agent.alive) {
            const shimmerSpeed = 0.003; // Speed of shimmer movement
            const shimmerPhase = (Date.now() * shimmerSpeed) % (Math.PI * 2);
            shimmerOffset = Math.sin(shimmerPhase) * 0.3; // -0.3 to 0.3
        }
        
        // Apply agent layer opacity
        ctx.save();
        ctx.globalAlpha = layerOpacity.agents;

        // DOCAS color palette (shared between genome and state coloring)
        const docasHues = [120, 200, 40, 0, 280]; // D, O, C, A, S
        const docasSats = [70, 70, 80, 70, 60];   // Base saturation for each

        // Determine agent color based on alive status and coloring mode
        let agentColor;
        if (agent.alive) {
            // Per SIM-GENOME-001: DOCAS overlay mode vs default genome mode
            // When layers.docas is ON: color by TRANSIENT DOCAS state (changes every tick)
            // When layers.docas is OFF: color by HERITABLE genome gains (stable)
            if (layers.docas && agent.docas) {
                // DOCAS STATE coloring: transient, reflects current neuromodulator levels
                const docas = agent.docas;
                const maxLevel = Math.max(...docas);
                const maxIndex = docas.indexOf(maxLevel);

                // Modulate saturation/lightness by level intensity
                const levelIntensity = Math.min(1.0, maxLevel);
                const baseLightness = 40 + levelIntensity * 20; // 40-60%
                // Part 3: Apply shimmer effect to lightness
                const lightness = Math.max(30, Math.min(70, baseLightness + shimmerOffset * 20));

                agentColor = `hsl(${docasHues[maxIndex]}, ${docasSats[maxIndex]}%, ${lightness}%)`;
            } else if (agent.genome_gains) {
                // GENOME-based coloring: heritable DOCAS gain traits (stable over lifetime)
                const gains = agent.genome_gains;
                const maxGain = Math.max(...gains);
                const maxIndex = gains.indexOf(maxGain);
                // Part 3: Apply shimmer effect (45-55% lightness range)
                const lightness = 50 + shimmerOffset * 10;
                agentColor = `hsl(${docasHues[maxIndex]}, ${docasSats[maxIndex]}%, ${lightness}%)`;
            } else {
                // Fallback if genome_gains not available
                // Part 3: Apply shimmer effect
                const lightness = 45 + shimmerOffset * 10;
                agentColor = `hsl(142, 50%, ${lightness}%)`; // Default green with shimmer
            }
        } else {
            agentColor = 'hsl(0, 0%, 40%)'; // Muted gray for dead
        }

        // Calculate Pac-Man facing direction from movement vector (last_dx, last_dy)
        // Default to facing right (0 radians) if no movement data
        let facingAngle = 0;
        if (agent.last_dx !== undefined && agent.last_dy !== undefined) {
            // Math.atan2(y, x) gives angle in radians from positive X axis
            facingAngle = Math.atan2(agent.last_dy, agent.last_dx);
        }

        // Pac-Man mouth: 60-degree wedge (30 degrees each side of facing direction)
        // Convert to radians: 60 degrees = π/3 radians, so half is π/6
        const mouthHalfAngle = Math.PI / 6; // 30 degrees
        const startAngle = facingAngle + mouthHalfAngle;
        const endAngle = facingAngle - mouthHalfAngle + Math.PI * 2;

        // Draw Pac-Man shape (circle with wedge-shaped mouth)
        ctx.beginPath();
        ctx.arc(x, y, baseRadius, startAngle, endAngle);
        ctx.lineTo(x, y); // Close the wedge to center
        ctx.closePath();
        ctx.fillStyle = agentColor;
        ctx.fill();

        // Part 3: Left-to-right shimmer highlight for alive agents
        if (agent.alive) {
            const shimmerSpeed = 0.004;
            const shimmerPos = (Date.now() * shimmerSpeed) % (baseRadius * 4) - baseRadius;
            
            // Create a gradient that moves left to right
            const gradient = ctx.createLinearGradient(
                x - baseRadius + shimmerPos, y,
                x + baseRadius + shimmerPos, y
            );
            gradient.addColorStop(0, 'rgba(255, 255, 255, 0)');
            gradient.addColorStop(0.3, 'rgba(255, 255, 255, 0)');
            gradient.addColorStop(0.5, 'rgba(255, 255, 255, 0.4)');
            gradient.addColorStop(0.7, 'rgba(255, 255, 255, 0)');
            gradient.addColorStop(1, 'rgba(255, 255, 255, 0)');
            
            ctx.save();
            // Draw shimmer with same Pac-Man shape (slightly smaller radius)
            const shimmerRadius = baseRadius * 0.8;
            ctx.beginPath();
            ctx.arc(x, y, shimmerRadius, startAngle, endAngle);
            ctx.lineTo(x, y);
            ctx.closePath();
            ctx.fillStyle = gradient;
            ctx.globalCompositeOperation = 'overlay';
            ctx.fill();
            ctx.restore();
        }

        // Energy ring (if enabled)
        if (layers.energy && agent.alive) {
            const energyRatio = Math.min(1, agent.energy / 100);
            ctx.beginPath();
            ctx.arc(x, y, baseRadius + 2, 0, Math.PI * 2 * energyRatio);
            ctx.strokeStyle = `hsla(45, 80%, 50%, ${energyRatio * layerOpacity.energy})`;
            ctx.lineWidth = 2;
            ctx.stroke();
        }

        // Identity badge (if enabled and agent has verified identity)
        if (layers.identity && agent.has_identity) {
            ctx.save();
            ctx.globalAlpha = layerOpacity.identity;
            
            const badgeSize = Math.max(4, baseRadius * 0.4);
            const badgeX = x + baseRadius * 0.7;
            const badgeY = y - baseRadius * 0.7;
            
            // Draw badge background (circle)
            ctx.beginPath();
            ctx.arc(badgeX, badgeY, badgeSize, 0, Math.PI * 2);
            ctx.fillStyle = 'hsl(200, 80%, 55%)';
            ctx.fill();
            
            // Draw badge border
            ctx.beginPath();
            ctx.arc(badgeX, badgeY, badgeSize, 0, Math.PI * 2);
            ctx.strokeStyle = 'rgba(255, 255, 255, 0.9)';
            ctx.lineWidth = 1;
            ctx.stroke();
            
            // Draw checkmark symbol
            ctx.save();
            ctx.translate(badgeX, badgeY);
            ctx.strokeStyle = 'white';
            ctx.lineWidth = 1.5;
            ctx.lineCap = 'round';
            ctx.lineJoin = 'round';
            ctx.beginPath();
            ctx.moveTo(-badgeSize * 0.3, 0);
            ctx.lineTo(-badgeSize * 0.1, badgeSize * 0.3);
            ctx.lineTo(badgeSize * 0.4, -badgeSize * 0.3);
            ctx.stroke();
            ctx.restore();
            
            ctx.restore(); // Restore identity opacity
        }

        // Reputation ring (if enabled)
        // Threshold-based visualization: invisible at 50%, red below, green above
        if (layers.reputation && agent.alive) {
            const reputation = agent.reputation !== undefined ? agent.reputation : 0.5;
            
            // Only render ring if reputation is NOT exactly 0.5 (50%)
            // Use tolerance to handle floating-point precision issues
            const REPUTATION_TOLERANCE = 0.001; // 0.1% tolerance
            const isNearFifty = Math.abs(reputation - 0.5) < REPUTATION_TOLERANCE;
            
            if (!isNearFifty) {
                ctx.save();
                ctx.globalAlpha = layerOpacity.reputation;
                
                const ringRadius = baseRadius + 3;
                const ringWidth = 2;
                
                // Binary color: red for below average (< 50%), green for above (> 50%)
                const repColor = reputation < 0.5 ? 'hsl(0, 80%, 50%)' : 'hsl(120, 80%, 50%)';
                
                // Draw reputation ring
                ctx.beginPath();
                ctx.arc(x, y, ringRadius, 0, Math.PI * 2);
                ctx.strokeStyle = repColor;
                ctx.lineWidth = ringWidth;
                ctx.stroke();
                
                ctx.restore();
            }
        }

        // Group indicator (if enabled)
        if (layers.groups && agent.group_id !== undefined && agent.group_id !== null) {
            ctx.save();
            ctx.globalAlpha = layerOpacity.groups;
            
            const hue = (agent.group_id * 137) % 360;
            ctx.beginPath();
            ctx.arc(x, y, baseRadius - 1, 0, Math.PI * 2);
            ctx.strokeStyle = `hsl(${hue}, 60%, 50%)`;
            ctx.lineWidth = 2;
            ctx.stroke();
            
            ctx.restore();
        }
        
        // Restore opacity before selection highlight
        ctx.restore();

        // Selection highlight (always drawn on top when selected)
        if (selectedAgentId === agent.agent_id) {
            // Outer glow ring
            ctx.beginPath();
            ctx.arc(x, y, baseRadius + 6, 0, Math.PI * 2);
            ctx.strokeStyle = 'rgba(255, 255, 255, 0.8)';
            ctx.lineWidth = 2;
            ctx.stroke();

            // Pulsing animation ring
            const pulseRadius = baseRadius + 8 + Math.sin(Date.now() / 200) * 2;
            ctx.beginPath();
            ctx.arc(x, y, pulseRadius, 0, Math.PI * 2);
            ctx.strokeStyle = 'rgba(255, 255, 255, 0.4)';
            ctx.lineWidth = 1;
            ctx.stroke();
        }
    }

    /**
     * Get statistics for current frame
     */
    function getStats() {
        if (currentFrame?.stats) {
            return {
                agents: currentFrame.stats.total_agents || 0,
                alive: currentFrame.stats.alive_agents || 0,
                verified: currentFrame.stats.verified_agents || 0,
                groups: currentFrame.stats.groups || 0
            };
        }

        if (!currentFrame?.agents) {
            return { agents: 0, alive: 0, verified: 0, groups: 0 };
        }

        const stats = {
            total: currentFrame.agents.length,
            alive: 0,
            verified: 0,
            groups: new Set()
        };

        for (const agent of currentFrame.agents) {
            if (agent.alive) stats.alive++;
            if (agent.has_identity) stats.verified++;
            if (agent.group_id !== undefined && agent.group_id !== null) {
                stats.groups.add(agent.group_id);
            }
        }

        return {
            agents: stats.total,
            alive: stats.alive,
            verified: stats.verified,
            groups: stats.groups.size
        };
    }

    /**
     * Disconnect from live stream
     */
    function disconnect() {
        if (eventSource) {
            eventSource.close();
            eventSource = null;
        }
        updateConnectionStatus('disconnected');
        // Show export button when simulation is disconnected/stopped
        showExportButton();
    }

    /**
     * Check if connected to live stream
     */
    function isConnected() {
        return eventSource !== null && eventSource.readyState === EventSource.OPEN;
    }

    /**
     * Set rendering configuration
     */
    function setRenderConfig(config) {
        Object.assign(renderConfig, config);
        render();
    }

    /**
     * Get current rendering configuration
     */
    function getRenderConfig() {
        return { ...renderConfig };
    }

    // Public API
    return {
        init,
        connect,
        disconnect,
        setLayer,
        setLayerOpacity,   // Opacity control for visualization layers
        setFilter,
        resetFilters,
        getFilters: () => filters,
        getCurrentTick,
        render,
        getStats,
        isConnected,
        setRenderConfig,
        getRenderConfig,
        setGridConfig,     // Phase 2: Grid configuration
        invalidateCaches: () => {
            isGridCacheValid = false;
            isTerrainCacheValid = false;
            isWeatherTileValid = false;
            isResourceTileValid = false;
            render();
        },

        
        // Agent inspector modal
        openAgentModal,
        closeAgentModal,
        
        // Phase 4: Export functions
        exportToPNG: () => {
            if (!canvas) return;
            const link = document.createElement('a');
            link.download = `simulation-tick-${currentTick}.png`;
            link.href = canvas.toDataURL('image/png');
            link.click();
        },
        exportToSVG: () => {
            // Create SVG representation of current frame
            const svgNS = 'http://www.w3.org/2000/svg';
            const svg = document.createElementNS(svgNS, 'svg');
            svg.setAttribute('width', canvas.width);
            svg.setAttribute('height', canvas.height);
            svg.setAttribute('viewBox', `0 0 ${canvas.width} ${canvas.height}`);
            
            // Add background
            const bg = document.createElementNS(svgNS, 'rect');
            bg.setAttribute('width', '100%');
            bg.setAttribute('height', '100%');
            bg.setAttribute('fill', '#0a0a0c');
            svg.appendChild(bg);
            
            // Note: Full SVG export would require rendering all agents as SVG elements
            // This is a simplified version that exports the canvas as PNG for now
            console.log('[Viewer] SVG export initialized (simplified)');
            
            // Fallback to PNG
            const link = document.createElement('a');
            link.download = `simulation-tick-${currentTick}.svg`;
            const svgData = new XMLSerializer().serializeToString(svg);
            link.href = 'data:image/svg+xml;base64,' + btoa(svgData);
            link.click();
        },
        
        // Phase 4: Frame buffer access
        getFrameBuffer: () => frameBuffer,
        getBufferedFrame,
        setZoom,
        setPan,
        
        // Phase 1: Simulation speed control
        setSpeed: async (factor) => {
            const apiUrl = window.SIM_SERVICE_URL || '';
            try {
                const response = await fetch(`${apiUrl}/api/live/speed`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ factor })
                });
                if (response.ok) {
                    const data = await response.json();
                    if (data.delay_ms) {
                        frameDurationMs = data.delay_ms;
                    } else if (data.factor) {
                        frameDurationMs = Math.max(20, Math.round(100 / data.factor));
                    }
                    console.log('[Viewer] Speed set to:', data.factor + 'x, frameDurationMs:', frameDurationMs);
                    return data;
                }
            } catch (err) {
                console.warn('[Viewer] Failed to set speed:', err);
            }
            return null;
        },
        getSpeed: async () => {
            const apiUrl = window.SIM_SERVICE_URL || '';
            try {
                const response = await fetch(`${apiUrl}/api/live/speed`);
                if (response.ok) {
                    return await response.json();
                }
            } catch (err) {
                console.warn('[Viewer] Failed to get speed:', err);
            }
            return { factor: 1.0, delay_ms: 100 };
        },
        
        // Phase 5: Simulation lifecycle control
        pause: async () => {
            const apiUrl = window.SIM_SERVICE_URL || '';
            try {
                const response = await fetch(`${apiUrl}/api/live/pause`, { method: 'POST' });
                if (response.ok) {
                    isSimulationPaused = true;
                    stopRenderLoop();
                    console.log('[Viewer] Simulation paused');
                    return true;
                }
            } catch (err) {
                console.warn('[Viewer] Failed to pause:', err);
            }
            return false;
        },
        resume: async () => {
            const apiUrl = window.SIM_SERVICE_URL || '';
            try {
                const response = await fetch(`${apiUrl}/api/live/resume`, { method: 'POST' });
                if (response.ok) {
                    isSimulationPaused = false;
                    frameInterpStartTime = performance.now();
                    startRenderLoop();
                    console.log('[Viewer] Simulation resumed');
                    return true;
                }
            } catch (err) {
                console.warn('[Viewer] Failed to resume:', err);
            }
            return false;
        },
        stop: async () => {
            stopRenderLoop();
            isSimulationPaused = false;
            const apiUrl = window.SIM_SERVICE_URL || '';
            try {
                const response = await fetch(`${apiUrl}/api/live/stop`, { method: 'POST' });
                if (response.ok) {
                    console.log('[Viewer] Simulation stopped');
                    return true;
                }
            } catch (err) {
                console.warn('[Viewer] Failed to stop:', err);
            }
            return false;
        },
        getStatus: async () => {
            const apiUrl = window.SIM_SERVICE_URL || '';
            try {
                const response = await fetch(`${apiUrl}/api/live/status`);
                if (response.ok) {
                    return await response.json();
                }
            } catch (err) {
                console.warn('[Viewer] Failed to get status:', err);
            }
            return { status: 'unknown', paused: false, stopped: false };
        },
        
        // Phase 5: Game Over screen control
        hideGameOver,
        
        // Phase 6: Export button visibility control
        showExportButton,
        hideExportButton,
        
        // Phase 6: Event log functions
        logEvent,
        getActivityLog: () => activityLog,
        clearActivityLog: () => {
            activityLog = [];
            isUserScrolledUp = false;
            renderActivityLog();
        },
        
        // Phase 6: Import/Export functionality
        exportState: async () => {
            const apiUrl = window.SIM_SERVICE_URL || '';
            try {
                const response = await fetch(`${apiUrl}/api/live/state/export`);
                if (response.ok) {
                    const exportData = await response.json();
                    const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' });
                    const url = URL.createObjectURL(blob);
                    const a = document.createElement('a');
                    a.href = url;
                    a.download = `simulation-state-tick-${exportData.tick || currentTick}.json`;
                    a.click();
                    URL.revokeObjectURL(url);
                    logEvent(`Simulation state exported (tick ${exportData.tick})`, 'success');
                    return;
                }
            } catch (err) {
                console.warn('[Viewer] Remote export failed, falling back to local snapshot:', err);
            }
            const exportData = {
                exported_at: new Date().toISOString(),
                tick: currentTick,
                config: window.SimulatorControls?.collectSimulationParams ? window.SimulatorControls.collectSimulationParams() : collectSimulationParams(),
                agents: currentFrame?.agents || [],
                stats: currentFrame?.stats || {},
                history: frameBuffer.map(f => ({
                    tick: f.tick,
                    population: f.agents?.length || 0,
                    alive_agents: f.agents?.filter(a => a.alive).length || 0,
                    avg_trust: f.agents?.length ? f.agents.reduce((s, a) => s + (a.tau || 0.5), 0) / f.agents.length : 0.5
                })),
                activityLog: activityLog,
                version: '4.1'
            };
            const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `simulation-state-tick-${currentTick}.json`;
            a.click();
            URL.revokeObjectURL(url);
            logEvent('Simulation state exported (local fallback)', 'success');
        },
        
        importState: async (file) => {
            try {
                const text = await file.text();
                const data = JSON.parse(text);
                
                // Call backend import endpoint
                const apiUrl = window.SIM_SERVICE_URL || '';
                const response = await fetch(`${apiUrl}/api/live/state/import`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: text
                });
                
                if (response.ok) {
                    const result = await response.json();
                    logEvent(`Imported state from tick ${result.tick || data.tick}`, 'success');
                    connect();
                    return true;
                } else if (data.agents || data.frame) {
                    const importedFrame = data.frame || {
                        tick: data.tick || 0,
                        agents: data.agents || [],
                        stats: data.stats || { total_agents: data.agents?.length || 0, alive_agents: data.agents?.filter(a => a.alive).length || 0 }
                    };
                    handleFrame(importedFrame);
                    logEvent(`Imported state from tick ${importedFrame.tick}`, 'success');
                    return true;
                }
            } catch (err) {
                console.error('[Viewer] Failed to import state:', err);
                logEvent('Failed to import state: ' + err.message, 'danger');
            }
            return false;
        },
        
        // Phase 6: Play button - start default simulation
        startDefaultSimulation: async () => {
            console.log('[Viewer] Starting default simulation');
            
            // Collect parameters from UI (delegate to SimulatorControls if available)
            const params = window.SimulatorControls && window.SimulatorControls.collectSimulationParams
                ? window.SimulatorControls.collectSimulationParams()
                : collectSimulationParams();
            console.log('[Viewer] Collected parameters:', params);
            
            // Call restart API with parameters
            try {
                const apiUrl = window.SIM_SERVICE_URL || '';
                console.log('[Viewer] Calling restart API at', `${apiUrl}/api/live/restart`);
                
                const response = await fetch(`${apiUrl}/api/live/restart`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(params)
                });
                
                if (!response.ok) {
                    console.error('[Viewer] Restart API failed:', response.status, response.statusText);
                } else {
                    const result = await response.json();
                    console.log('[Viewer] Restart API response:', result);
                }
                
                // Wait a moment for the restart to take effect
                await new Promise(resolve => setTimeout(resolve, 500));
                
            } catch (err) {
                console.error('[Viewer] Error calling restart API:', err);
                // Continue anyway - the simulation might already be running
            }
            
            // Dispatch event for controls to handle
            window.dispatchEvent(new CustomEvent('simulationStarted', { 
                detail: { experiment: 'default', params }
            }));
            
            logEvent('Starting default simulation...', 'info');
        },

        // Public UI helpers exposed cleanly outside IIFE
        logEvent: (message, type = 'info', agentId = null, category = 'other') => {
            logEvent(message, type, agentId, category);
        },
        exportBrainSnapshot: () => {
            if (!brainViz) return false;
            const dataUrl = brainViz.exportImage();
            const a = document.createElement('a');
            a.href = dataUrl;
            a.download = `brain-agent-${brainVizLastAgentId || 'snapshot'}-tick-${currentTick}.png`;
            a.click();
            logEvent('3D Brain snapshot exported', 'success');
            return true;
        },
        
        // AI Chat integration (canonical unified agent widget)
        openAIChat: () => {
            if (window.FreshCreditAgent && window.FreshCreditAgent.open) {
                window.FreshCreditAgent.open();
            }
        },
        closeAIChat: () => {
            if (window.FreshCreditAgent && window.FreshCreditAgent.close) {
                window.FreshCreditAgent.close();
            }
        },
        toggleAIChat: () => {
            if (window.FreshCreditAgent && window.FreshCreditAgent.toggle) {
                window.FreshCreditAgent.toggle();
            }
        },
        isAIChatOpen: () => {
            return window.FreshCreditAgent && window.FreshCreditAgent.isOpen ? window.FreshCreditAgent.isOpen() : false;
        },
        
        // DEBUG: Diagnostic functions
        getDebugState: () => debugState,
        getFrameHistory: () => debugState.frameHistory,
        getAgentPositionHistory: (agentId) => debugState.agentPositionHistory[agentId],
        getConnectionEvents: () => debugState.connectionEvents,
        clearDebugState: () => {
            debugState.frameHistory = [];
            debugState.agentPositionHistory = {};
            debugState.connectionEvents = [];
            debugState.frameStats = {
                framesReceived: 0,
                framesRendered: 0,
                lastFrameTime: 0,
                avgFrameInterval: 0,
                frameIntervals: []
            };
            debugState.movementStats = {
                totalMovement: 0,
                samples: 0,
                frozenAgentTicks: {}
            };
            debugLog('DEBUG', 'Debug state cleared');
        }
    };
})();

/**
 * Collect simulation parameters from UI controls
 * Used by both controls.js and the pixelated play button
 */
function collectSimulationParams() {
    if (window.SimulatorControls && window.SimulatorControls.collectSimulationParams) {
        return window.SimulatorControls.collectSimulationParams();
    }
    const params = {
        resource_density: 0.30,
        initial_resource: 50.0,
        agent_count: 20,
        grid_width: 100,
        grid_height: 100
    };
    
    // Get resource density from slider (0-10 scale, convert to 0-1)
    const densitySlider = document.getElementById('geo-resource-density');
    if (densitySlider) {
        const sliderValue = parseFloat(densitySlider.value);
        params.resource_density = sliderValue / 10;
    }
    
    // Get initial resource from slider (0-10 scale, convert to 0-100)
    const initialResourceSlider = document.getElementById('geo-initial-resource');
    if (initialResourceSlider) {
        const sliderValue = parseFloat(initialResourceSlider.value);
        params.initial_resource = sliderValue * 10;
    }
    
    // Grid dimensions from inputs
    const gridWidthInput = document.getElementById('geo-grid-width');
    const gridHeightInput = document.getElementById('geo-grid-height');
    if (gridWidthInput) {
        params.grid_width = parseInt(gridWidthInput.value, 10) || 100;
    }
    if (gridHeightInput) {
        params.grid_height = parseInt(gridHeightInput.value, 10) || 100;
    }
    
    return params;
}

/**
 * Initialize the viewer on DOM ready
 * NOTE: No longer auto-connects. Connection happens when user clicks Play.
 */
document.addEventListener('DOMContentLoaded', () => {
    console.log('[Viewer] DOM ready, initializing viewer...');
    
    // Initialize the main simulation canvas
    const initResult = SimulatorViewer.init('simulation-canvas');
    console.log('[Viewer] Initialization result:', initResult);
    
    // Initialize the pixelated loader animation
    // Phase 5: Start in IDLE mode (rainbow shimmering, no rotation)
    if (window.PixelatedLoader) {
        console.log('[Viewer] Initializing pixelated loader...');
        const loaderResult = window.PixelatedLoader.init('loader-canvas');
        console.log('[Viewer] Pixelated loader init result:', loaderResult);
        
        // Start in IDLE mode - rainbow colors, no spinning
        window.PixelatedLoader.start(window.PixelatedLoader.MODE.IDLE);
        console.log('[Viewer] Pixelated loader started in IDLE mode');
    } else {
        console.warn('[Viewer] PixelatedLoader not found - loader script may not be loaded');
    }
    
    // Phase 5: Setup restart button handler
    const restartBtn = document.getElementById('restart-btn');
    if (restartBtn) {
        restartBtn.addEventListener('click', () => {
            console.log('[Viewer] Restart button clicked');
            SimulatorViewer.hideGameOver();
            // Reload the page to restart the simulation
            window.location.reload();
        });
    }

    const telemetryToggleBtn = document.getElementById('v4-telemetry-toggle-btn');
    if (telemetryToggleBtn) {
        telemetryToggleBtn.addEventListener('click', () => {
            const dash = document.getElementById('v4-telemetry-dashboard');
            if (dash) {
                dash.classList.toggle('collapsed');
                telemetryToggleBtn.textContent = dash.classList.contains('collapsed') ? '▲' : '_';
            }
        });
    }
    
    // Setup Landing Actions (Start in Default vs Configure Settings)
    const startDefaultBtn = document.getElementById('start-default-btn');
    if (startDefaultBtn) {
        startDefaultBtn.addEventListener('click', () => {
            console.log('[Viewer] Start in Default clicked');
            const overlay = document.getElementById('canvas-overlay');
            if (overlay) overlay.style.display = 'none';
            SimulatorViewer.startDefaultSimulation();
        });
    }

    const configureSimBtn = document.getElementById('configure-sim-btn');
    if (configureSimBtn) {
        configureSimBtn.addEventListener('click', () => {
            console.log('[Viewer] Configure Settings clicked');
            const overlay = document.getElementById('canvas-overlay');
            if (overlay) overlay.style.display = 'none';

            // Ensure sidebar is expanded
            const sidebar = document.getElementById('sim-sidebar');
            if (sidebar) sidebar.classList.remove('collapsed');
            const expandBtn = document.getElementById('sim-sidebar-expand-btn');
            if (expandBtn) expandBtn.classList.add('hidden');

            // Render live preview on canvas
            SimulatorViewer.render();
            SimulatorViewer.logEvent('Standby Mode: Configure simulation layers and parameters before running', 'info');
        });
    }

    // Phase 6: Setup PLAY button handler (legacy fallback)
    const playBtn = document.getElementById('pixelated-play-btn');
    if (playBtn) {
        playBtn.addEventListener('click', () => {
            console.log('[Viewer] PLAY button clicked');
            SimulatorViewer.startDefaultSimulation();
        });
    }
    
    // Phase 6: Setup Import/Export button handlers
    const exportBtn = document.getElementById('export-btn');
    if (exportBtn) {
        exportBtn.addEventListener('click', () => {
            console.log('[Viewer] Export button clicked');
            SimulatorViewer.exportState();
        });
    }

    const brainExportBtn = document.getElementById('brain-export-btn');
    if (brainExportBtn) {
        brainExportBtn.addEventListener('click', () => {
            console.log('[Viewer] Brain snapshot export clicked');
            SimulatorViewer.exportBrainSnapshot();
        });
    }
    
    const importBtn = document.getElementById('import-btn');
    const importFileInput = document.getElementById('import-file-input');
    if (importBtn && importFileInput) {
        importBtn.addEventListener('click', () => {
            importFileInput.click();
        });
        
        importFileInput.addEventListener('change', (e) => {
            const file = e.target.files[0];
            if (file) {
                console.log('[Viewer] Importing file:', file.name);
                SimulatorViewer.importState(file);
            }
            // Reset input so same file can be selected again
            importFileInput.value = '';
        });
    }
    
    console.log('[Viewer] Connecting to live stream in Standby Mode...');
    SimulatorViewer.connect();
});

// Close modal on Escape key
document.addEventListener('keydown', (event) => {
    if (event.key === 'Escape' && window.SimulatorViewer) {
        window.SimulatorViewer.closeAgentModal();
    }
});

// Export for other modules
window.SimulatorViewer = SimulatorViewer;
