/**
 * Pixelated Circle Loading Animation + Game Over Screen
 * 
 * Features:
 * - Initial state: Rainbow shimmering circle (not spinning)
 * - Running state: Spinning circle that transitions to grayscale
 * - Game Over: Retro pixelated "GAME OVER" screen
 */
(function() {
    'use strict';

    // Animation modes
    const MODE = {
        IDLE: 'idle',           // Initial rainbow shimmering, no rotation
        SPINNING: 'spinning',   // Rainbow with rotation
        GRAYSCALE: 'grayscale', // Spinning grayscale
        TRANSITION: 'transition', // Transitioning to grayscale
        GAMEOVER: 'gameover'    // Game over screen
    };

    // Animation configuration
    const CONFIG = {
        width: 200,
        height: 200,
        radius: 70,
        pixelSize: 5,
        speed: 1.5,
        // Rainbow colors for initial state
        rainbowColors: [0, 30, 60, 90, 120, 150, 180, 210, 240, 270, 300, 330],
        // DOCAS-inspired bright colors: Dopamine(green), Oxytocin(blue), Cortisol(orange), Adrenaline(red), Serotonin(purple)
        docasColors: [120, 200, 35, 0, 280],
        segments: 16,
        // Shimmer effect config
        shimmerSpeed: 3.0,
        shimmerIntensity: 0.4,
        // Brightness settings for "agent" look
        saturation: 85,
        lightness: 65,
        glowLightness: 80,
        // Game Over text config
        gameOverText: [
            "####   ###   ##  #####  ####",
            "#     #   # #  # #     #   #",
            "# ### ##### #  # ####  #   #",
            "#   # #   # #  # #     #   #",
            "####  #   #  ##  ##### ####"
        ]
    };

    // Animation state
    let animationId = null;
    let canvas = null;
    let ctx = null;
    let startTime = 0;
    let isRunning = false;
    let currentMode = MODE.IDLE;
    let isTransitioning = false;
    let transitionStartTime = 0;
    let transitionCallback = null;
    const TRANSITION_DURATION = 800; // ms for grayscale transition

    /**
     * Initialize the animation canvas
     */
    function init(canvasId) {
        canvas = document.getElementById(canvasId);
        if (!canvas) {
            console.error('[PixelatedLoader] Canvas not found:', canvasId);
            return false;
        }

        ctx = canvas.getContext('2d');
        if (!ctx) {
            console.error('[PixelatedLoader] Failed to get 2D context');
            return false;
        }

        // Set canvas size
        canvas.width = CONFIG.width;
        canvas.height = CONFIG.height;

        console.log('[PixelatedLoader] Initialized');
        return true;
    }

    /**
     * Draw a single pixel block
     */
    function drawPixel(x, y, size, color) {
        const px = Math.floor(x / size) * size;
        const py = Math.floor(y / size) * size;
        ctx.fillStyle = color;
        ctx.fillRect(px, py, size, size);
    }

    /**
     * Check if a point is within the circle
     */
    function isInCircle(x, y, centerX, centerY, radius) {
        const dx = x - centerX;
        const dy = y - centerY;
        return (dx * dx + dy * dy) <= (radius * radius);
    }

    /**
     * Get color for a segment with shimmering effect
     */
    function getSegmentColor(segmentIndex, rotationOffset, shimmerPhase, useRainbow = false) {
        const colors = useRainbow ? CONFIG.rainbowColors : CONFIG.docasColors;
        const colorIndex = Math.floor((segmentIndex + rotationOffset) % colors.length);
        const baseHue = colors[colorIndex];
        
        // Add shimmer variation to hue
        const shimmerOffset = Math.sin(shimmerPhase + segmentIndex * 0.5) * 15;
        const hue = (baseHue + shimmerOffset + 360) % 360;
        
        return `hsl(${hue}, ${CONFIG.saturation}%, ${CONFIG.lightness}%)`;
    }

    /**
     * Draw the Game Over screen
     */
    function drawGameOver() {
        ctx.fillStyle = '#000';
        ctx.fillRect(0, 0, CONFIG.width, CONFIG.height);
        
        const pixelSize = 4;
        const startX = 10;
        const startY = 70;
        
        // Draw GAME OVER text pixel by pixel
        CONFIG.gameOverText.forEach((row, rowIndex) => {
            for (let i = 0; i < row.length; i++) {
                if (row[i] === '#') {
                    const x = startX + i * pixelSize;
                    const y = startY + rowIndex * pixelSize * 1.5;
                    
                    // Rainbow gradient based on column
                    const hue = (i * 15) % 360;
                    ctx.fillStyle = `hsl(${hue}, 100%, 50%)`;
                    ctx.fillRect(x, y, pixelSize, pixelSize);
                }
            }
        });
        
        // Draw blinking "Press to restart" text
        const blink = Math.sin(Date.now() / 200) > 0;
        if (blink) {
            ctx.fillStyle = '#fff';
            ctx.font = '10px monospace';
            ctx.textAlign = 'center';
            ctx.fillText('CLICK TO RESTART', CONFIG.width / 2, CONFIG.height - 20);
        }
    }

    /**
     * Render a single frame of the pixelated circle
     * 
     * MODES:
     * - IDLE: Rainbow shimmering, no rotation (initial state)
     * - SPINNING: Rainbow with rotation
     * - GRAYSCALE: Spinning grayscale
     * - TRANSITION: Transitioning to grayscale
     * - GAMEOVER: Retro Game Over screen
     */
    function renderFrame(timestamp) {
        if (!ctx || !canvas) return;

        // Handle Game Over mode separately
        if (currentMode === MODE.GAMEOVER) {
            drawGameOver();
            if (isRunning) {
                animationId = requestAnimationFrame(renderFrame);
            }
            return;
        }

        // Calculate elapsed time in seconds
        if (!startTime) startTime = timestamp;
        const elapsed = (timestamp - startTime) / 1000;

        // Calculate rotation angle (only if spinning or transitioning)
        let rotation = 0;
        let rotationOffset = 0;
        if (currentMode === MODE.SPINNING || currentMode === MODE.GRAYSCALE || currentMode === MODE.TRANSITION) {
            rotation = (elapsed * CONFIG.speed * Math.PI * 2) % (Math.PI * 2);
            rotationOffset = (rotation / (Math.PI * 2)) * CONFIG.segments;
        }

        // Shimmer phase for color pulsing
        const shimmerPhase = elapsed * CONFIG.shimmerSpeed;

        // Clear canvas
        ctx.clearRect(0, 0, CONFIG.width, CONFIG.height);

        const centerX = CONFIG.width / 2;
        const centerY = CONFIG.height / 2;
        const pixelSize = CONFIG.pixelSize;

        // Calculate transition progress for grayscale mode
        let transitionProgress = 0;
        if (currentMode === MODE.TRANSITION) {
            transitionProgress = Math.min(1, (timestamp - transitionStartTime) / TRANSITION_DURATION);
            if (transitionProgress >= 1) {
                currentMode = MODE.GRAYSCALE;
                isTransitioning = false;
                if (transitionCallback) {
                    transitionCallback();
                    transitionCallback = null;
                }
            }
        }

        // Use rainbow colors for IDLE mode, DOCAS colors for others
        const useRainbow = currentMode === MODE.IDLE;

        // Draw pixelated circle with shimmer effect
        for (let y = centerY - CONFIG.radius; y <= centerY + CONFIG.radius; y += pixelSize) {
            for (let x = centerX - CONFIG.radius; x <= centerX + CONFIG.radius; x += pixelSize) {
                if (isInCircle(x + pixelSize/2, y + pixelSize/2, centerX, centerY, CONFIG.radius)) {
                    // Calculate angle for segment coloring
                    const angle = Math.atan2(y - centerY, x - centerX);
                    const normalizedAngle = (angle + Math.PI) / (Math.PI * 2);
                    const segmentIndex = Math.floor(normalizedAngle * CONFIG.segments);
                    
                    // Rotation wave effect - segments light up as they rotate
                    const rotationWave = Math.sin((segmentIndex / CONFIG.segments + elapsed * CONFIG.speed) * Math.PI * 2);
                    
                    // Shimmer brightness variation
                    const shimmerBrightness = Math.sin(shimmerPhase + segmentIndex * 0.3) * CONFIG.shimmerIntensity;
                    
                    // Combined intensity from rotation and shimmer
                    const baseIntensity = (rotationWave + 1) / 2; // 0 to 1
                    const finalIntensity = Math.max(0.3, Math.min(1, baseIntensity + shimmerBrightness));
                    
                    // Get color based on mode
                    let color;
                    if (currentMode === MODE.GRAYSCALE) {
                        // Full grayscale
                        const lightness = CONFIG.lightness * finalIntensity;
                        color = `hsl(0, 0%, ${lightness}%)`;
                    } else if (currentMode === MODE.TRANSITION) {
                        // Transitioning: desaturate progressively
                        const saturation = CONFIG.saturation * (1 - transitionProgress);
                        const baseColor = getSegmentColor(segmentIndex, Math.floor(rotationOffset), shimmerPhase, useRainbow);
                        const hslMatch = baseColor.match(/hsl\((\d+),\s*([\d.]+)%,\s*([\d.]+)%\)/);
                        if (hslMatch) {
                            const [, h, , l] = hslMatch;
                            color = `hsla(${h}, ${saturation}%, ${l}%, ${finalIntensity})`;
                        } else {
                            color = `hsla(0, 0%, ${CONFIG.lightness}%, ${finalIntensity})`;
                        }
                    } else {
                        // Full color (IDLE or SPINNING)
                        color = getSegmentColor(segmentIndex, Math.floor(rotationOffset), shimmerPhase, useRainbow);
                        const baseColor = color;
                        const hslMatch = baseColor.match(/hsl\((\d+),\s*([\d.]+)%,\s*([\d.]+)%\)/);
                        if (hslMatch) {
                            const [, h, s, l] = hslMatch;
                            color = `hsla(${h}, ${s}%, ${l}%, ${finalIntensity})`;
                        }
                    }
                    
                    drawPixel(x, y, pixelSize, color);
                }
            }
        }

        // Draw center glow with shimmer
        const glowSize = CONFIG.pixelSize * 3;
        const glowX = Math.floor((centerX - glowSize/2) / pixelSize) * pixelSize;
        const glowY = Math.floor((centerY - glowSize/2) / pixelSize) * pixelSize;
        
        // Pulsing center glow
        const glowPulse = 0.7 + Math.sin(shimmerPhase * 2) * 0.3;
        let glowSat = CONFIG.saturation;
        
        if (currentMode === MODE.GRAYSCALE) {
            glowSat = 0;
        } else if (currentMode === MODE.TRANSITION) {
            glowSat = CONFIG.saturation * (1 - transitionProgress);
        }
        
        const glowHue = useRainbow ? (elapsed * 60) % 360 : CONFIG.docasColors[0];
        ctx.fillStyle = `hsla(${glowHue}, ${glowSat}%, ${CONFIG.glowLightness}%, ${glowPulse})`;
        ctx.fillRect(glowX, glowY, glowSize, glowSize);

        // Schedule next frame if still running
        if (isRunning) {
            animationId = requestAnimationFrame(renderFrame);
        }
    }

    /**
     * Start the animation in a specific mode
     * @param {string} mode - MODE.IDLE, MODE.SPINNING, MODE.GRAYSCALE, or MODE.GAMEOVER
     */
    function start(mode = MODE.IDLE) {
        if (isRunning && currentMode === mode) return;
        
        // If already running, stop first
        if (isRunning) {
            stop();
        }
        
        currentMode = mode;
        isRunning = true;
        isTransitioning = false;
        startTime = 0;
        console.log(`[PixelatedLoader] Animation started in ${mode} mode`);
        animationId = requestAnimationFrame(renderFrame);
    }

    /**
     * Stop the animation immediately
     */
    function stop() {
        isRunning = false;
        isTransitioning = false;
        if (animationId) {
            cancelAnimationFrame(animationId);
            animationId = null;
        }
        console.log('[PixelatedLoader] Animation stopped');
    }

    /**
     * Transition from color to grayscale
     * @param {Function} callback - Called when transition completes
     */
    function transitionToGrayscale(callback) {
        if (!isRunning) {
            start(MODE.GRAYSCALE);
            if (callback) setTimeout(callback, 100);
            return;
        }
        
        if (currentMode === MODE.GRAYSCALE) {
            if (callback) callback();
            return;
        }
        
        isTransitioning = true;
        transitionStartTime = performance.now();
        transitionCallback = callback;
        currentMode = MODE.TRANSITION;
        console.log('[PixelatedLoader] Starting grayscale transition');
    }

    /**
     * Show Game Over screen
     */
    function showGameOver() {
        stop();
        currentMode = MODE.GAMEOVER;
        isRunning = true;
        startTime = 0;
        console.log('[PixelatedLoader] Game Over screen displayed');
        animationId = requestAnimationFrame(renderFrame);
    }

    /**
     * Check if animation is running
     */
    function isAnimating() {
        return isRunning;
    }

    /**
     * Get current mode
     */
    function getMode() {
        return currentMode;
    }

    // Export public API
    window.PixelatedLoader = {
        MODE,
        init,
        start,
        stop,
        transitionToGrayscale,
        showGameOver,
        isAnimating,
        getMode
    };
})();
