/**
 * 3D Brain Visualization for Agent Society Simulator
 * 
 * WebGL-based brain visualization using Three.js
 * Displays DOCAS neuromodulator activation on anatomical brain regions
 * 
 * @module BrainVisualization
 * @version 1.0.0
 */

// ============================================
// Brain Region Definitions (MNI Coordinates)
// ============================================

const BRAIN_REGIONS = {
    // Dopaminergic system
    vta: {
        name: 'Ventral Tegmental Area',
        coordinates: [0, -16, -10],
        explosionVector: [0, -0.2, -0.1],
        neuromodulator: 'dopamine',
        color: 0xFFD700,  // Gold
        radius: 2.5,
        description: 'Reward prediction and motivation'
    },
    nacc: {
        name: 'Nucleus Accumbens',
        coordinates: [6, 10, -8],
        explosionVector: [0.3, 0.2, -0.1],
        neuromodulator: 'dopamine',
        color: 0xFFA500,  // Orange
        radius: 3.0,
        description: 'Reward processing and reinforcement'
    },
    
    // Oxytocin system
    hypothalamus: {
        name: 'Hypothalamus',
        coordinates: [0, -4, -8],
        explosionVector: [0, -0.1, -0.2],
        neuromodulator: 'oxytocin',
        color: 0xFF69B4,  // Hot pink
        radius: 4.0,
        description: 'Social bonding and attachment'
    },
    
    // Cortisol/stress system
    amygdala: {
        name: 'Amygdala',
        coordinates: [22, -6, -14],
        explosionVector: [0.8, -0.2, -0.3],
        neuromodulator: 'cortisol',
        color: 0xFF4500,  // Red orange
        radius: 3.5,
        description: 'Threat detection and fear'
    },
    pvn: {
        name: 'Paraventricular Nucleus',
        coordinates: [0, -12, -6],
        explosionVector: [0, -0.3, -0.1],
        neuromodulator: 'cortisol',
        color: 0xDC143C,  // Crimson
        radius: 2.0,
        description: 'HPA axis regulation'
    },
    
    // Adrenaline system
    locus_coeruleus: {
        name: 'Locus Coeruleus',
        coordinates: [-2, -36, -24],
        explosionVector: [-0.2, -0.8, -0.5],
        neuromodulator: 'adrenaline',
        color: 0xFF8C00,  // Dark orange
        radius: 1.5,
        description: 'Arousal and vigilance'
    },
    
    // Serotonin system
    raphe_nuclei: {
        name: 'Raphe Nuclei',
        coordinates: [0, -26, -20],
        explosionVector: [0, -0.6, -0.4],
        neuromodulator: 'serotonin',
        color: 0x6495ED,  // Cornflower blue
        radius: 2.5,
        description: 'Mood regulation and stability'
    },
    
    // Higher-order regions (multi-modulator)
    prefrontal_cortex: {
        name: 'dlPFC (Prefrontal Cortex · Muse EEG/fNIRS)',
        coordinates: [0, 50, 20],
        explosionVector: [0, 1.8, 0.8],
        neuromodulator: 'mixed',
        color: 0x9370DB,  // Medium purple
        radius: 8.0,
        description: 'Executive valuation, Muse Frontal Theta & fNIRS oxygenation (Sim D21)'
    },
    anterior_cingulate: {
        name: 'Anterior Cingulate Cortex',
        coordinates: [0, 24, 16],
        explosionVector: [0, 1.2, 0.5],
        neuromodulator: 'mixed',
        color: 0xBA55D3,  // Orchid
        radius: 5.0,
        description: 'Conflict monitoring and social evaluation'
    },
    insula: {
        name: 'Anterior Insula (aINS)',
        coordinates: [36, 4, -4],
        explosionVector: [1.8, 0.3, 0.0],
        neuromodulator: 'mixed',
        color: 0xFF1E1E,  // Crimson red on NPE disgust burst
        radius: 5.5,
        description: 'Norm prediction error, fairness & betrayal shock (Sim D21)'
    },
    superior_temporal_gyrus: {
        name: 'Superior Temporal Gyrus (STG · Muse TP9/TP10)',
        coordinates: [54, -18, 6],
        explosionVector: [2.0, -0.4, 0.2],
        neuromodulator: 'mixed',
        color: 0x38BDF8,  // Sky blue
        radius: 4.5,
        description: 'Acoustic envelope tracking & speech phase-locking (Sim D20/D21)'
    },
    default_mode_network: {
        name: 'Default Mode Network (DMN)',
        coordinates: [0, -52, 26],
        explosionVector: [0, -1.8, 1.0],
        neuromodulator: 'mixed',
        color: 0xA855F7,  // Purple
        radius: 6.0,
        description: 'Counterfactual deliberation & narrative reflection (Sim DMN)'
    },
    visual_cortex: {
        name: 'Visual Cortex (V1/V4)',
        coordinates: [12, -90, 0],
        explosionVector: [0.5, -2.2, 0.0],
        neuromodulator: 'mixed',
        color: 0x22C55E,  // Green
        radius: 5.0,
        description: 'Visual layout & typographic margin contrast (Sim D16)'
    },

    // Extended Full-Body Wet-Lab Visceral, Endocrine, Vascular, Cellular & Somatic Nodes
    scn_photoreceptor: {
        name: 'SCN & Retina (Hue Lux/Kelvin · Daylight · Temp Δ)',
        coordinates: [0, 8, 22],
        explosionVector: [0, 1.1, 1.5],
        neuromodulator: 'serotonin',
        color: 0x38BDF8,
        radius: 3.4,
        tier: 'cranial',
        description: 'Circadian Photic Zeitgeber (Hue Kelvin/Lux, TimeInDaylight, Watch wrist temp Δ)'
    },
    cardiac_vagus_node: {
        name: 'Cardiac Vagal Plexus (Watch RMSSD · Sleep · SpO2)',
        coordinates: [0, -18, -32],
        explosionVector: [0, -0.8, -1.5],
        neuromodulator: 'oxytocin',
        color: 0xEC4899,
        radius: 6.2,
        tier: 'body',
        description: 'Parasympathetic vagal tone, beat-to-beat RMSSD HRV & REM/N3 autonomic recovery'
    },
    pulmonary_metabolic_node: {
        name: 'Lungs & Mitochondria (Airofit · Aranet4 CO2 · PNOĒ)',
        coordinates: [24, -10, -28],
        explosionVector: [1.8, -0.3, -1.2],
        neuromodulator: 'mixed',
        color: 0x06B6D4,
        radius: 5.8,
        tier: 'body',
        description: 'Airofit MIP/MEP/VC, Aranet4 NDIR CO2/hPa, and respiratory calibration'
    },
    adrenal_glands_node: {
        name: 'Adrenal Cortex & Medulla (Serum Cortisol · 4-Pt CAR)',
        coordinates: [16, -28, -24],
        explosionVector: [1.4, -1.2, -1.0],
        neuromodulator: 'cortisol',
        color: 0xDC143C,
        radius: 4.5,
        tier: 'body',
        description: 'Morning Serum Cortisol, DHEA-S, 4-pt Salivary CAR (AUCi/AUCg) & Hair Cortisol'
    },
    hepatic_glucose_node: {
        name: 'Liver & Pancreas (CGM · HOMA-IR · Nutrition Kernel)',
        coordinates: [-22, -24, -26],
        explosionVector: [-1.7, -1.0, -1.1],
        neuromodulator: 'cortisol',
        color: 0xEAB308,
        radius: 6.0,
        tier: 'body',
        description: 'CGM interstitial dG/dt, Fasting Insulin, HOMA-IR, HbA1c & postprandial kernel'
    },
    enteric_gut_node: {
        name: 'Enteric Plexus & Gut (Throne · TBW · 5-HT Motility)',
        coordinates: [0, -36, -20],
        explosionVector: [0, -1.8, -0.8],
        neuromodulator: 'serotonin',
        color: 0x6495ED,
        radius: 5.6,
        tier: 'body',
        description: 'Throne Bristol motility, urinary hydration, BIA TBW% & peripheral 5-HT'
    },
    vascular_endothelial_node: {
        name: 'Vascular Endothelium (Hilo BP · hs-CRP · ApoB · NLR)',
        coordinates: [-24, -4, -22],
        explosionVector: [-1.8, 0.2, -0.9],
        neuromodulator: 'adrenaline',
        color: 0xFF8C00,
        radius: 5.2,
        tier: 'body',
        description: 'Hilo cuffless SBP/DBP & nocturnal dipping, hs-CRP, ApoB, Lipid Panel & CBC NLR'
    },
    skeletal_muscle_node: {
        name: 'Skeletal Muscle (MIDI Motor · Training · T/E2 · BIA)',
        coordinates: [28, -32, -14],
        explosionVector: [2.0, -1.5, -0.4],
        neuromodulator: 'dopamine',
        color: 0x22C55E,
        radius: 5.8,
        tier: 'body',
        description: 'Neuromotor cost-of-work, resistance turnover, Free T/SHBG/E2 & lean body mass'
    },
    cellular_remodeling_node: {
        name: 'Reproductive & Cellular Remodeling (90-Day Cycles)',
        coordinates: [18, -42, -8],
        explosionVector: [1.3, -2.0, -0.2],
        neuromodulator: 'oxytocin',
        color: 0xF472B6,
        radius: 4.4,
        tier: 'body',
        description: '90-day spermatogenesis remodeling (concentration, motility, morphology, HPG axis)'
    },
    genomic_follicular_node: {
        name: '30x WGS & Segmental Hair (FKBP5 · COMT · FAAH)',
        coordinates: [-26, 22, -14],
        explosionVector: [-1.9, 1.4, -0.5],
        neuromodulator: 'mixed',
        color: 0xA855F7,
        radius: 4.6,
        tier: 'body',
        description: '30x WGS priors (FKBP5/COMT/DRD2/FAAH/CNR1/CYP1A2) & segmental hair archive'
    }
};

// ============================================
// Neuromodulator Color Scales
// ============================================

const NEUROMODULATOR_COLORS = {
    dopamine: {
        low: new THREE.Color(0x332200),
        medium: new THREE.Color(0xFFD700),
        high: new THREE.Color(0xFFFF00)
    },
    oxytocin: {
        low: new THREE.Color(0x330011),
        medium: new THREE.Color(0xFF69B4),
        high: new THREE.Color(0xFFB6C1)
    },
    cortisol: {
        low: new THREE.Color(0x330000),
        medium: new THREE.Color(0xFF4500),
        high: new THREE.Color(0xFF0000)
    },
    adrenaline: {
        low: new THREE.Color(0x331100),
        medium: new THREE.Color(0xFF8C00),
        high: new THREE.Color(0xFFA500)
    },
    serotonin: {
        low: new THREE.Color(0x000033),
        medium: new THREE.Color(0x6495ED),
        high: new THREE.Color(0x87CEEB)
    },
    mixed: {
        low: new THREE.Color(0x1a1a1a),
        medium: new THREE.Color(0x9370DB),
        high: new THREE.Color(0xE6E6FA)
    }
};

// ============================================
// Brain Visualization Class
// ============================================

class BrainVisualization {
    constructor(containerId, options = {}) {
        this.container = document.getElementById(containerId);
        if (!this.container) {
            throw new Error(`Container #${containerId} not found`);
        }

        // Configuration
        this.options = {
            width: options.width || this.container.clientWidth,
            height: options.height || this.container.clientHeight,
            backgroundColor: options.backgroundColor || 0x1a1a1a,
            enableRotation: options.enableRotation !== false,
            enableZoom: options.enableZoom !== false,
            showLabels: options.showLabels !== false,
            ...options
        };

        // State
        this.regions = new Map();
        this.activationData = new Map();
        this.isAnimating = false;
        this.animationFrame = null;
        this.currentTime = 0;
        
        // CRITICAL FIX: Track destroyed state to prevent animation after cleanup
        this.isDestroyed = false;
        
        // FIX: Track current agent to detect agent switches
        this.currentAgentId = null;
        
        // Live activity indicator state
        this.liveActivityEnabled = false;
        this.activityPhase = 0;
        this.pulseSpeed = 0.05;

        // Dissection explosion state
        this.explosionFactor = 0.0;
        this.targetExplosionFactor = 0.0;
        this.connectionLines = [];

        // Three.js components
        this.scene = null;
        this.camera = null;
        this.renderer = null;
        this.controls = null;
        this.raycaster = new THREE.Raycaster();
        this.mouse = new THREE.Vector2();

        // Initialize
        this.init();
    }

    init() {
        // Scene setup
        this.scene = new THREE.Scene();
        this.scene.background = new THREE.Color(this.options.backgroundColor);

        // Camera
        const aspect = this.options.width / this.options.height;
        this.camera = new THREE.PerspectiveCamera(45, aspect, 0.1, 1000);
        this.camera.position.set(0, 0, 120);

        // Renderer with WebGL context recovery and preserved drawing buffer for PNG snapshots
        this.renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true, preserveDrawingBuffer: true });
        this.renderer.setSize(this.options.width, this.options.height);
        this.renderer.setPixelRatio(window.devicePixelRatio);
        this.renderer.shadowMap.enabled = true;
        this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;
        this.container.appendChild(this.renderer.domElement);

        this.renderer.domElement.addEventListener('webglcontextlost', (e) => {
            e.preventDefault();
            console.warn('[BrainViz] WebGL context lost.');
            if (this.animationFrame) cancelAnimationFrame(this.animationFrame);
        });
        this.renderer.domElement.addEventListener('webglcontextrestored', () => {
            console.log('[BrainViz] WebGL context restored, re-initializing.');
            this.init();
        });

        // Controls
        if (this.options.enableRotation || this.options.enableZoom) {
            this.controls = new THREE.OrbitControls(this.camera, this.renderer.domElement);
            this.controls.enableRotate = this.options.enableRotation;
            this.controls.enableZoom = this.options.enableZoom;
            this.controls.enablePan = true;
            this.controls.minDistance = 40;
            this.controls.maxDistance = 360;
        }

        // Lighting
        this.setupLighting();

        // Create brain regions
        this.createBrainRegions();

        // Create connecting lines (simplified white matter tracts)
        this.createConnections();

        // Event listeners
        this.setupEventListeners();

        // Add export PNG snapshot button
        this.addExportButton();
        // Add Explode / Dissect 3D systems button
        this.addExplodeButton();


        // Add fullscreen button (P2)
        this.addFullscreenButton();

        // Start render loop
        this.animate();
    }

    setupLighting() {
        // Ambient light
        const ambientLight = new THREE.AmbientLight(0xffffff, 0.4);
        this.scene.add(ambientLight);

        // Directional lights
        const light1 = new THREE.DirectionalLight(0xffffff, 0.8);
        light1.position.set(50, 50, 50);
        light1.castShadow = true;
        this.scene.add(light1);

        const light2 = new THREE.DirectionalLight(0xffffff, 0.4);
        light2.position.set(-50, -50, -50);
        this.scene.add(light2);

        // Point lights for depth
        const pointLight = new THREE.PointLight(0x6495ED, 0.3, 100);
        pointLight.position.set(0, 30, 0);
        this.scene.add(pointLight);
    }

    createBrainRegions() {
        Object.entries(BRAIN_REGIONS).forEach(([key, region]) => {
            // Create sphere geometry
            const geometry = new THREE.SphereGeometry(region.radius, 32, 32);
            
            // Create material with initial color
            const material = new THREE.MeshPhysicalMaterial({
                color: region.color,
                metalness: 0.3,
                roughness: 0.4,
                transparent: true,
                opacity: 0.9,
                emissive: region.color,
                emissiveIntensity: 0.1
            });

            // Create mesh
            const mesh = new THREE.Mesh(geometry, material);
            mesh.position.set(...region.coordinates);
            mesh.castShadow = true;
            mesh.receiveShadow = true;
            
            // Add glow effect (larger transparent sphere)
            const glowGeometry = new THREE.SphereGeometry(region.radius * 1.3, 16, 16);
            const glowMaterial = new THREE.MeshBasicMaterial({
                color: region.color,
                transparent: true,
                opacity: 0.1,
                side: THREE.BackSide
            });
            const glow = new THREE.Mesh(glowGeometry, glowMaterial);
            glow.position.set(...region.coordinates);
            glow.visible = false;  // Hidden by default
            
            // Store region data
            this.regions.set(key, {
                mesh: mesh,
                glow: glow,
                data: region,
                currentActivation: 0.0,
                targetActivation: 0.0
            });

            this.scene.add(mesh);
            this.scene.add(glow);

            // Add label if enabled
            if (this.options.showLabels) {
                this.createLabel(key, region, mesh.position);
            }
        });
    }

    createConnections() {
        const connections = [
            ['vta', 'nacc'],
            ['vta', 'prefrontal_cortex'],
            ['hypothalamus', 'amygdala'],
            ['amygdala', 'locus_coeruleus'],
            ['raphe_nuclei', 'amygdala'],
            ['locus_coeruleus', 'prefrontal_cortex'],
            ['prefrontal_cortex', 'anterior_cingulate'],
            ['anterior_cingulate', 'insula'],
            ['insula', 'prefrontal_cortex'],
            ['superior_temporal_gyrus', 'insula'],
            ['prefrontal_cortex', 'default_mode_network'],
            ['visual_cortex', 'prefrontal_cortex'],
            ['scn_photoreceptor', 'hypothalamus'],
            ['scn_photoreceptor', 'raphe_nuclei'],
            ['hypothalamus', 'cardiac_vagus_node'],
            ['locus_coeruleus', 'cardiac_vagus_node'],
            ['pvn', 'adrenal_glands_node'],
            ['adrenal_glands_node', 'hepatic_glucose_node'],
            ['cardiac_vagus_node', 'pulmonary_metabolic_node'],
            ['cardiac_vagus_node', 'vascular_endothelial_node'],
            ['adrenal_glands_node', 'vascular_endothelial_node'],
            ['hepatic_glucose_node', 'pulmonary_metabolic_node'],
            ['hepatic_glucose_node', 'enteric_gut_node'],
            ['raphe_nuclei', 'enteric_gut_node'],
            ['pulmonary_metabolic_node', 'skeletal_muscle_node'],
            ['hepatic_glucose_node', 'skeletal_muscle_node'],
            ['adrenal_glands_node', 'cellular_remodeling_node'],
            ['skeletal_muscle_node', 'cellular_remodeling_node'],
            ['genomic_follicular_node', 'prefrontal_cortex'],
            ['genomic_follicular_node', 'adrenal_glands_node']
        ];

        const lineMaterial = new THREE.LineBasicMaterial({
            color: 0x555555,
            transparent: true,
            opacity: 0.35
        });

        this.connectionLines = [];
        connections.forEach(([from, to]) => {
            const fromRegion = this.regions.get(from);
            const toRegion = this.regions.get(to);
            
            if (fromRegion && toRegion) {
                const points = [
                    fromRegion.mesh.position.clone(),
                    toRegion.mesh.position.clone()
                ];
                const geometry = new THREE.BufferGeometry().setFromPoints(points);
                const line = new THREE.Line(geometry, lineMaterial);
                this.scene.add(line);
                this.connectionLines.push({ line, fromKey: from, toKey: to });
            }
        });
    }

    setupEventListeners() {
        // Mouse move for hover effects
        this.renderer.domElement.addEventListener('mousemove', (event) => {
            const rect = this.renderer.domElement.getBoundingClientRect();
            this.mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
            this.mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
            this.checkIntersection();
        });

        // Window resize
        window.addEventListener('resize', () => {
            if (!this.container) return;
            this.options.width = this.container.clientWidth;
            this.options.height = this.container.clientHeight;
            
            this.camera.aspect = this.options.width / this.options.height;
            this.camera.updateProjectionMatrix();
            this.renderer.setSize(this.options.width, this.options.height);
        });
    }

    checkIntersection() {
        this.raycaster.setFromCamera(this.mouse, this.camera);
        
        const meshes = Array.from(this.regions.values()).map(r => r.mesh);
        const intersects = this.raycaster.intersectObjects(meshes);

        // Reset all labels
        this.regions.forEach(region => {
            if (region.label) region.label.visible = false;
            region.mesh.material.emissiveIntensity = 0.1;
        });

        // Show label for hovered region
        if (intersects.length > 0) {
            const hoveredMesh = intersects[0].object;
            for (const [key, region] of this.regions) {
                if (region.mesh === hoveredMesh) {
                    if (region.label) region.label.visible = true;
                    region.mesh.material.emissiveIntensity = 0.5;
                    break;
                }
            }
        }
    }

    // ============================================
    // Public API
    // ============================================

    /**
     * Update activation levels for brain regions
     * @param {Object} activations - Map of region keys to activation values (0-1)
     * @param {boolean} immediate - If true, skip interpolation and set immediately
     */
    updateActivations(activations, immediate = false) {
        console.log('[BrainViz] updateActivations called with:', activations, 'immediate:', immediate);
        
        Object.entries(activations).forEach(([key, value]) => {
            const region = this.regions.get(key);
            if (region) {
                const clampedValue = Math.max(0, Math.min(1, value));
                region.targetActivation = clampedValue;
                
                if (immediate) {
                    // CRITICAL FIX: Force both current and target to the new value immediately
                    region.currentActivation = clampedValue;
                    
                    // Also force immediate visual update to prevent caching issues
                    const nm = region.data.neuromodulator;
                    const colors = NEUROMODULATOR_COLORS[nm] || NEUROMODULATOR_COLORS.mixed;
                    
                    const baseColor = colors.medium.clone();
                    const activationColor = colors.high.clone();
                    
                    // Directly set color without interpolation
                    region.mesh.material.color.lerpColors(
                        baseColor,
                        activationColor,
                        clampedValue
                    );
                    
                    // Update emissive intensity directly
                    region.mesh.material.emissiveIntensity = 0.1 + clampedValue * 0.4;
                    
                    // Update glow visibility
                    region.glow.visible = clampedValue > 0.3;
                    region.glow.material.opacity = clampedValue * 0.3;
                    
                    console.log(`[BrainViz] Region ${key} set to activation ${clampedValue.toFixed(3)}`);
                }
            }
        });
        
        // Force a render update to ensure visual changes are applied
        if (immediate && this.renderer && this.scene && this.camera) {
            this.renderer.render(this.scene, this.camera);
        }
    }

    /**
     * Reset all activations to zero (called when switching agents)
     * CRITICAL FIX: Properly clears all visualization state to prevent cross-agent contamination
     */
    resetActivations() {
        console.log('[BrainViz] resetActivations called - clearing all state');
        
        this.regions.forEach((region, key) => {
            region.targetActivation = 0;
            region.currentActivation = 0;
            
            // CRITICAL FIX: Reset material to base state and force update
            const nm = region.data.neuromodulator;
            const colors = NEUROMODULATOR_COLORS[nm] || NEUROMODULATOR_COLORS.mixed;
            
            // Reset color to base medium color
            region.mesh.material.color.copy(colors.medium);
            
            // Reset emissive to base color with low intensity
            region.mesh.material.emissive.copy(colors.medium);
            region.mesh.material.emissiveIntensity = 0.1;
            
            // CRITICAL FIX: Force material update flags
            region.mesh.material.needsUpdate = true;
            
            // Reset glow
            region.glow.visible = false;
            region.glow.material.opacity = 0.1;
            region.glow.material.needsUpdate = true;
            
            // Clear any activity pulse
            if (region.activityLight) {
                region.activityLight.visible = false;
            }
            
            console.log(`[BrainViz] Reset region ${key} - color set to`, colors.medium.getHexString());
        });
        
        // Clear time series data
        this.activationData = [];
        this.currentTime = 0;
        this.isAnimating = false;
        
        // CRITICAL FIX: Force a render to clear the visual state
        if (this.renderer && this.scene && this.camera) {
            this.renderer.render(this.scene, this.camera);
        }
        
        console.log('[BrainViz] resetActivations complete');
    }
    
    /**
     * Set the current agent ID to track agent switches
     * @param {number} agentId - The current agent's ID
     */
    setCurrentAgent(agentId) {
        console.log(`[BrainViz] setCurrentAgent called - current: ${this.currentAgentId}, new: ${agentId}`);
        
        // If switching to a different agent, reset everything
        if (this.currentAgentId !== null && this.currentAgentId !== agentId) {
            console.log(`[BrainViz] Switching from agent ${this.currentAgentId} to ${agentId}, resetting state`);
            this.resetActivations();
        } else if (this.currentAgentId === null) {
            // First time initialization - also reset to ensure clean state
            console.log(`[BrainViz] First agent ${agentId}, ensuring clean state`);
            this.resetActivations();
        }
        
        this.currentAgentId = agentId;
        console.log(`[BrainViz] Current agent set to ${agentId}`);
    }
    
    /**
     * Enable/disable live activity indicator (Christmas lights effect)
     * @param {boolean} enabled - Whether to enable the live activity effect
     */
    setLiveActivity(enabled) {
        this.liveActivityEnabled = enabled;
        if (!enabled) {
            // Hide all activity lights
            this.regions.forEach(region => {
                if (region.activityLight) {
                    region.activityLight.visible = false;
                }
            });
        }
    }

    /**
     * Update from DOCAS state
     * @param {Object} docas - DOCAS state with levels [D, O, C, A, S]
     * @param {boolean} immediate - If true, skip interpolation and set immediately
     * @param {boolean} isAlive - Whether the agent is alive (for activity effects)
     */
    updateFromDocas(docas, immediate = false, isAlive = true) {
        const [d, o, c, a, s] = docas.levels;
        
        console.log(`[BrainViz] updateFromDocas called - DOCAS levels: D=${d.toFixed(3)}, O=${o.toFixed(3)}, C=${c.toFixed(3)}, A=${a.toFixed(3)}, S=${s.toFixed(3)}, immediate=${immediate}, isAlive=${isAlive}`);

        // Map DOCAS levels to region activations with Baylor consciousness gating and Sim D21 BOLD
        const isBaylorUnconscious = document.getElementById('toggle-baylor-mode')?.checked && c < 0.12;
        const executiveGate = isBaylorUnconscious ? 0.08 : 1.0;
        const isCloaked = (parseFloat(document.getElementById('dial-cloaking-ratio')?.value || '10') / 10) > 1.5;
        const ainsActivation = isCloaked ? 0.2324 : (c > 0.35 ? 0.5810 : 0.0037);

        const activations = {
            // Dopaminergic cranial regions
            vta: d,
            nacc: d * 0.85,
            
            // Oxytocin cranial regions
            hypothalamus: o,
            
            // Cortisol cranial regions
            amygdala: c,
            pvn: c * 0.92,
            
            // Adrenaline cranial regions
            locus_coeruleus: a,
            
            // Serotonin cranial regions
            raphe_nuclei: s,
            scn_photoreceptor: Math.max(0, Math.min(1, s * 0.85 + 0.1)),
            
            // Higher-order regions (mixed) - Sim D21 TRIBE v2 cortical mapping
            prefrontal_cortex: ((d + s) / 2) * executiveGate,
            anterior_cingulate: ((c + a) / 2) * executiveGate,
            insula: ainsActivation,
            superior_temporal_gyrus: isCloaked ? 0.08 : 0.16,
            default_mode_network: Math.max(0, Math.min(1, 0.40 + c * 0.3)),
            visual_cortex: Math.max(0, Math.min(1, 0.35 + (isCloaked ? 0.35 : 0.0))),

            // Peripheral full-body biomarker organ nodes
            cardiac_vagus_node: Math.max(0, Math.min(1, (o + s) * 0.5)),
            pulmonary_metabolic_node: Math.max(0, Math.min(1, (a + d) * 0.45 + 0.1)),
            adrenal_glands_node: Math.max(0, Math.min(1, c * 0.95 + a * 0.15)),
            hepatic_glucose_node: Math.max(0, Math.min(1, c * 0.75 + a * 0.25)),
            enteric_gut_node: Math.max(0, Math.min(1, s * 0.9)),
            vascular_endothelial_node: Math.max(0, Math.min(1, a * 0.65 + c * 0.35)),
            skeletal_muscle_node: Math.max(0, Math.min(1, (d * 0.6 + a * 0.4) * executiveGate)),
            cellular_remodeling_node: Math.max(0, Math.min(1, (o * 0.5 + s * 0.5))),
            genomic_follicular_node: Math.max(0, Math.min(1, (c + d + s) / 3))
        };
        
        console.log('[BrainViz] Computed activations:', Object.entries(activations).map(([k, v]) => `${k}=${v.toFixed(3)}`).join(', '));

        this.updateActivations(activations, immediate);
        
        // Enable live activity for living agents
        this.setLiveActivity(isAlive);
        
        console.log('[BrainViz] updateFromDocas complete');
    }

    /**
     * Update peripheral organ and neural nodes directly from normalized TFP/DOCAS biomarkers.
     */
    updateFromBiomarkers(biomarkers = {}, immediate = false) {
        const clamp01 = (v) => Math.max(0, Math.min(1, Number(v) || 0));
        const activations = {};
        if (biomarkers.fnirs_hbo2_umol != null) activations.prefrontal_cortex = clamp01(0.5 + Number(biomarkers.fnirs_hbo2_umol) / 4.0);
        if (biomarkers.hrv_rmssd_ms != null) activations.cardiac_vagus_node = clamp01(Number(biomarkers.hrv_rmssd_ms) / 100.0);
        if (biomarkers.glucose_mg_dl != null) activations.hepatic_glucose_node = clamp01((Number(biomarkers.glucose_mg_dl) - 70.0) / 80.0);
        if (biomarkers.morning_cortisol_ug_dl != null) activations.adrenal_glands_node = clamp01(Number(biomarkers.morning_cortisol_ug_dl) / 25.0);
        if (biomarkers.hs_crp_mg_l != null || biomarkers.sbp_mmhg != null) {
            const crp = Number(biomarkers.hs_crp_mg_l || 0) / 3.0;
            const sbp = biomarkers.sbp_mmhg != null ? (Number(biomarkers.sbp_mmhg) - 105.0) / 40.0 : 0;
            activations.vascular_endothelial_node = clamp01(Math.max(crp, sbp));
        }
        if (biomarkers.co2_ppm != null || biomarkers.mip_cmh2o != null) {
            activations.pulmonary_metabolic_node = clamp01(biomarkers.co2_ppm != null ? (Number(biomarkers.co2_ppm) - 400.0) / 1200.0 : 0.3);
        }
        if (biomarkers.free_testosterone_pg_ml != null) activations.skeletal_muscle_node = clamp01(Number(biomarkers.free_testosterone_pg_ml) / 180.0);
        if (biomarkers.progressive_motility_pct != null) activations.cellular_remodeling_node = clamp01(Number(biomarkers.progressive_motility_pct) / 100.0);
        if (biomarkers.hydration_index != null) activations.enteric_gut_node = clamp01(Number(biomarkers.hydration_index) / 100.0);
        if (biomarkers.photic_zeitgeber_index != null) activations.scn_photoreceptor = clamp01(Number(biomarkers.photic_zeitgeber_index) / 8.0);
        if (biomarkers.hair_cortisol_pg_mg != null) activations.genomic_follicular_node = clamp01(Number(biomarkers.hair_cortisol_pg_mg) / 25.0);
        if (Object.keys(activations).length > 0) this.updateActivations(activations, immediate);
    }

    /**
     * Update activation from server-provided region data
     * @param {Array} regions - Array of {region, activation} objects from API
     */
    updateFromRegions(regions) {
        const activations = {};
        regions.forEach(r => {
            // Map API region names to internal names
            const name = r.region.toLowerCase().replace(/ /g, '_');
            activations[name] = r.activation;
            if (name === 'raphe' || name === 'drn') activations['raphe_nuclei'] = r.activation;
            if (name === 'acc') activations['anterior_cingulate'] = r.activation;
            if (name === 'ains' || name === 'anterior_insula') activations['insula'] = r.activation;
            if (name === 'dlpfc') activations['prefrontal_cortex'] = r.activation;
            if (name === 'stg') activations['superior_temporal_gyrus'] = r.activation;
            if (name === 'dmn') activations['default_mode_network'] = r.activation;
        });
        this.updateActivations(activations);
    }

    /**
     * Set time series data for animation
     * @param {Array} timeSeries - Array of {time, activations} objects
     */
    setTimeSeries(timeSeries) {
        this.activationData = timeSeries;
        this.currentTime = 0;
    }

    /**
     * Start animation playback
     */
    play() {
        this.isAnimating = true;
    }

    /**
     * Pause animation
     */
    pause() {
        this.isAnimating = false;
    }

    /**
     * Seek to specific time
     * @param {number} time - Time in seconds
     */
    seek(time) {
        this.currentTime = time;
        this.updateCurrentFrame();
    }

    /**
     * Export current visualization to image
     * @returns {string} Data URL of PNG image
     */
    exportImage() {
        this.renderer.render(this.scene, this.camera);
        return this.renderer.domElement.toDataURL('image/png');
    }

    /**
     * Generate JSON data for Three.js scene
     * @returns {Object} Scene data
     */
    toJSON() {
        const regions = [];
        this.regions.forEach((region, key) => {
            regions.push({
                key: key,
                name: region.data.name,
                coordinates: region.data.coordinates,
                neuromodulator: region.data.neuromodulator,
                activation: region.currentActivation
            });
        });

        return {
            regions: regions,
            timestamp: Date.now()
        };
    }

    // ============================================
    // Fullscreen Mode (P2)
    // ============================================

    /**
     * Toggle fullscreen mode for the visualization
     */
    toggleFullscreen() {
        const container = this.container;
        
        if (!document.fullscreenElement) {
            // Enter fullscreen
            if (container.requestFullscreen) {
                container.requestFullscreen().then(() => {
                    this.onFullscreenChange();
                }).catch(err => {
                    console.error('[BrainViz] Error entering fullscreen:', err);
                });
            } else if (container.webkitRequestFullscreen) {
                container.webkitRequestFullscreen();
                this.onFullscreenChange();
            } else if (container.msRequestFullscreen) {
                container.msRequestFullscreen();
                this.onFullscreenChange();
            }
        } else {
            // Exit fullscreen
            if (document.exitFullscreen) {
                document.exitFullscreen().then(() => {
                    this.onFullscreenChange();
                }).catch(err => {
                    console.error('[BrainViz] Error exiting fullscreen:', err);
                });
            } else if (document.webkitExitFullscreen) {
                document.webkitExitFullscreen();
                this.onFullscreenChange();
            } else if (document.msExitFullscreen) {
                document.msExitFullscreen();
                this.onFullscreenChange();
            }
        }
    }

    /**
     * Handle fullscreen state changes
     */
    onFullscreenChange() {
        const isFullscreen = !!document.fullscreenElement;
        
        // Update container class for styling
        this.container.classList.toggle('fullscreen', isFullscreen);
        
        // Resize renderer to match new dimensions
        setTimeout(() => {
            const width = this.container.clientWidth;
            const height = this.container.clientHeight;
            
            this.camera.aspect = width / height;
            this.camera.updateProjectionMatrix();
            this.renderer.setSize(width, height);
            
            // Update label scales based on new size
            this.updateLabelScales();
            
            console.log('[BrainViz] Fullscreen:', isFullscreen, `Size: ${width}x${height}`);
        }, 100);
        
        // Dispatch event for UI updates
        window.dispatchEvent(new CustomEvent('brainvizFullscreenChange', {
            detail: { isFullscreen }
        }));
    }

    /**
     * Check if currently in fullscreen mode
     * @returns {boolean}
     */
    isFullscreen() {
        return !!document.fullscreenElement;
    }

    /**
     * Add Explode / Dissect 3D Systems button
     */
    addExplodeButton() {
        const button = document.createElement('button');
        button.className = 'brainviz-explode-btn';
        button.innerHTML = '💥 Dissect';
        button.title = 'Explode / Dissect 3D Brain & Wet-Lab Organ Systems';
        button.style.cssText = `
            position: absolute;
            top: 10px;
            right: 124px;
            height: 36px;
            padding: 0 10px;
            background: rgba(0, 0, 0, 0.6);
            border: 1px solid rgba(255, 255, 255, 0.3);
            border-radius: 4px;
            color: white;
            font-size: 11px;
            font-family: Inter, sans-serif;
            cursor: pointer;
            z-index: 1000;
            display: flex;
            align-items: center;
            justify-content: center;
            transition: background 0.2s;
        `;

        button.addEventListener('mouseenter', () => {
            button.style.background = 'rgba(0, 0, 0, 0.85)';
        });
        button.addEventListener('mouseleave', () => {
            button.style.background = 'rgba(0, 0, 0, 0.6)';
        });

        button.addEventListener('click', () => {
            this.toggleExplodedView();
        });

        this.container.appendChild(button);
        this.explodeButton = button;
    }

    setExplosionFactor(factor) {
        const clamped = Math.max(0, Math.min(1, Number(factor) || 0));
        this.targetExplosionFactor = clamped;
        if (this.explodeButton) {
            this.explodeButton.innerHTML = clamped > 0.5 ? '🧩 Assemble' : '💥 Dissect';
        }
    }

    toggleExplodedView() {
        this.setExplosionFactor(this.targetExplosionFactor > 0.5 ? 0.0 : 1.0);
    }

    updateExplosionPositions() {
        const mult = 22.0;
        const showDissectLabels = this.explosionFactor > 0.25;
        this.regions.forEach((region) => {
            const base = region.data.coordinates;
            const vec = region.data.explosionVector || [0, 0, 0];
            const curX = base[0] + vec[0] * this.explosionFactor * mult;
            const curY = base[1] + vec[1] * this.explosionFactor * mult;
            const curZ = base[2] + vec[2] * this.explosionFactor * mult;

            region.mesh.position.set(curX, curY, curZ);
            region.glow.position.set(curX, curY, curZ);
            if (region.label) {
                region.label.position.set(curX, curY + region.data.radius + 6, curZ);
                region.label.visible = showDissectLabels;
            }
        });

        if (this.camera && !this.isDestroyed) {
            const targetZ = 120 + this.explosionFactor * 95;
            this.camera.position.z += (targetZ - this.camera.position.z) * 0.15;
        }

        if (this.connectionLines) {
            this.connectionLines.forEach(({ line, fromKey, toKey }) => {
                const fromRegion = this.regions.get(fromKey);
                const toRegion = this.regions.get(toKey);
                if (fromRegion && toRegion && line.geometry) {
                    const positions = line.geometry.attributes.position.array;
                    positions[0] = fromRegion.mesh.position.x;
                    positions[1] = fromRegion.mesh.position.y;
                    positions[2] = fromRegion.mesh.position.z;
                    positions[3] = toRegion.mesh.position.x;
                    positions[4] = toRegion.mesh.position.y;
                    positions[5] = toRegion.mesh.position.z;
                    line.geometry.attributes.position.needsUpdate = true;
                }
            });
        }
    }

    /**
     * Add export PNG snapshot button to container (Sim D21 Neuroimaging Snapshot)
     */
    addExportButton() {
        const button = document.createElement('button');
        button.className = 'brainviz-export-btn';
        button.innerHTML = '📸 PNG';
        button.title = 'Download 3D Brain PNG Snapshot';
        button.style.cssText = `
            position: absolute;
            top: 10px;
            right: 52px;
            height: 36px;
            padding: 0 10px;
            background: rgba(0, 0, 0, 0.6);
            border: 1px solid rgba(255, 255, 255, 0.3);
            border-radius: 4px;
            color: white;
            font-size: 11px;
            font-family: Inter, sans-serif;
            cursor: pointer;
            z-index: 1000;
            display: flex;
            align-items: center;
            justify-content: center;
            transition: background 0.2s;
        `;

        button.addEventListener('mouseenter', () => {
            button.style.background = 'rgba(0, 0, 0, 0.85)';
        });

        button.addEventListener('mouseleave', () => {
            button.style.background = 'rgba(0, 0, 0, 0.6)';
        });

        button.addEventListener('click', () => {
            const dataUrl = this.exportImage();
            const a = document.createElement('a');
            a.href = dataUrl;
            a.download = `brain-agent-${this.currentAgentId || 'snapshot'}-tick-${Date.now()}.png`;
            a.click();
        });

        this.container.appendChild(button);
        this.exportButton = button;
    }

    /**
     * Add fullscreen button to container
     */
    addFullscreenButton() {
        const button = document.createElement('button');
        button.className = 'brainviz-fullscreen-btn';
        button.innerHTML = '⛶';
        button.title = 'Toggle Fullscreen';
        button.style.cssText = `
            position: absolute;
            top: 10px;
            right: 10px;
            width: 36px;
            height: 36px;
            background: rgba(0, 0, 0, 0.6);
            border: 1px solid rgba(255, 255, 255, 0.3);
            border-radius: 4px;
            color: white;
            font-size: 18px;
            cursor: pointer;
            z-index: 1000;
            display: flex;
            align-items: center;
            justify-content: center;
            transition: background 0.2s;
        `;
        
        button.addEventListener('mouseenter', () => {
            button.style.background = 'rgba(0, 0, 0, 0.8)';
        });
        
        button.addEventListener('mouseleave', () => {
            button.style.background = 'rgba(0, 0, 0, 0.6)';
        });
        
        button.addEventListener('click', () => this.toggleFullscreen());
        
        this.container.appendChild(button);
        this.fullscreenButton = button;
        
        // Listen for fullscreen changes
        document.addEventListener('fullscreenchange', () => this.onFullscreenChange());
        document.addEventListener('webkitfullscreenchange', () => this.onFullscreenChange());
        document.addEventListener('msfullscreenchange', () => this.onFullscreenChange());
        
        // ESC key handler
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' && this.isFullscreen()) {
                this.toggleFullscreen();
            }
        });
    }

    // ============================================
    // Text Readability Improvements (P3)
    // ============================================

    /**
     * Create improved label with larger text and better contrast
     */
    createLabel(key, region, position) {
        const canvas = document.createElement('canvas');
        const context = canvas.getContext('2d');
        
        // Higher resolution for sharper text (P3)
        const scale = 2; // Retina/high-DPI scaling
        canvas.width = 512 * scale;
        canvas.height = 128 * scale;
        context.scale(scale, scale);

        // Draw label background with border for better contrast (P3)
        context.fillStyle = 'rgba(0, 0, 0, 0.85)';
        context.strokeStyle = 'rgba(255, 255, 255, 0.3)';
        context.lineWidth = 2;
        this.roundRect(context, 0, 0, 512, 128, 16);
        context.fill();
        context.stroke();

        // Draw main text - larger font (P3)
        context.font = 'bold 28px Inter, Arial, sans-serif';
        context.fillStyle = 'white';
        context.textAlign = 'center';
        context.textBaseline = 'middle';
        context.shadowColor = 'rgba(0, 0, 0, 0.8)';
        context.shadowBlur = 4;
        context.fillText(region.name, 256, 50);
        
        // Draw neuromodulator text - larger font (P3)
        context.font = '20px Inter, Arial, sans-serif';
        context.fillStyle = this.getNeuromodulatorColor(region.neuromodulator);
        context.fillText(region.neuromodulator.toUpperCase(), 256, 90);
        
        // Reset shadow
        context.shadowColor = 'transparent';
        context.shadowBlur = 0;

        // Create texture and sprite with high resolution
        const texture = new THREE.CanvasTexture(canvas);
        texture.minFilter = THREE.LinearFilter;
        texture.magFilter = THREE.LinearFilter;
        
        const spriteMaterial = new THREE.SpriteMaterial({ 
            map: texture,
            transparent: true,
            opacity: 0.95
        });
        const sprite = new THREE.Sprite(spriteMaterial);
        
        sprite.position.copy(position);
        sprite.position.y += region.radius + 8;
        // Larger scale for better visibility (P3)
        sprite.scale.set(16, 4, 1);
        sprite.visible = false;

        this.regions.get(key).label = sprite;
        this.scene.add(sprite);
    }

    /**
     * Helper for rounded rectangle
     */
    roundRect(ctx, x, y, width, height, radius) {
        ctx.beginPath();
        ctx.moveTo(x + radius, y);
        ctx.lineTo(x + width - radius, y);
        ctx.quadraticCurveTo(x + width, y, x + width, y + radius);
        ctx.lineTo(x + width, y + height - radius);
        ctx.quadraticCurveTo(x + width, y + height, x + width - radius, y + height);
        ctx.lineTo(x + radius, y + height);
        ctx.quadraticCurveTo(x, y + height, x, y + height - radius);
        ctx.lineTo(x, y + radius);
        ctx.quadraticCurveTo(x, y, x + radius, y);
        ctx.closePath();
    }

    /**
     * Get color for neuromodulator label
     */
    getNeuromodulatorColor(nm) {
        const colors = {
            dopamine: '#FFD700',
            oxytocin: '#FF69B4',
            cortisol: '#FF4500',
            adrenaline: '#FF8C00',
            serotonin: '#6495ED',
            mixed: '#9370DB'
        };
        return colors[nm] || '#ffffff';
    }

    /**
     * Update label scales based on camera distance (P3 - dynamic text scaling)
     */
    updateLabelScales() {
        const cameraDistance = this.camera.position.distanceTo(new THREE.Vector3(0, 0, 0));
        const baseScale = Math.max(0.5, Math.min(2.0, cameraDistance / 100));
        
        this.regions.forEach(region => {
            if (region.label) {
                // Scale labels inversely with camera distance to maintain readable size
                const scaleFactor = 16 * baseScale;
                region.label.scale.set(scaleFactor, scaleFactor / 4, 1);
            }
        });
    }

    /**
     * Show all labels (useful for screenshots)
     */
    showAllLabels() {
        this.regions.forEach(region => {
            if (region.label) region.label.visible = true;
        });
    }

    /**
     * Hide all labels
     */
    hideAllLabels() {
        this.regions.forEach(region => {
            if (region.label) region.label.visible = false;
        });
    }

    // ============================================
    // Animation Loop
    // ============================================

    animate() {
        // CRITICAL FIX: Check if destroyed before scheduling next frame
        if (this.isDestroyed) {
            return;
        }
        
        this.animationFrame = requestAnimationFrame(() => this.animate());
        
        // CRITICAL FIX: Double-check destroyed state after scheduling
        if (this.isDestroyed) {
            return;
        }

        // Update controls
        if (this.controls) {
            this.controls.update();
        }

        // Smoothly interpolate dissection explosion factor
        if (Math.abs(this.targetExplosionFactor - this.explosionFactor) > 0.001) {
            this.explosionFactor += (this.targetExplosionFactor - this.explosionFactor) * 0.08;
            this.updateExplosionPositions();
        }

        // Dynamic label scaling based on camera distance (P3)
        if (this.camera && this.regions.size > 0) {
            this.updateLabelScales();
        }

        // Update activity phase for pulsing effect
        this.activityPhase += this.pulseSpeed;
        
        // Animate activations (smooth interpolation)
        this.regions.forEach((region, key) => {
            const diff = region.targetActivation - region.currentActivation;
            region.currentActivation += diff * 0.1;

            // Update material based on activation
            const nm = region.data.neuromodulator;
            const colors = NEUROMODULATOR_COLORS[nm] || NEUROMODULATOR_COLORS.mixed;
            
            const lowColor = colors.low.clone();
            const midColor = colors.medium.clone();
            const highColor = colors.high.clone();
            
            if (region.currentActivation < 0.5) {
                region.mesh.material.color.lerpColors(
                    lowColor,
                    midColor,
                    region.currentActivation * 2.0
                );
            } else {
                region.mesh.material.color.lerpColors(
                    midColor,
                    highColor,
                    (region.currentActivation - 0.5) * 2.0
                );
            }

            // Dynamically scale region sphere size with activation
            const scaleFactor = 0.75 + 0.55 * region.currentActivation;
            region.mesh.scale.set(scaleFactor, scaleFactor, scaleFactor);
            
            // Live activity indicator: Add pulsing emissive effect for living agents
            let emissiveIntensity = 0.1 + region.currentActivation * 0.4;
            
            if (this.liveActivityEnabled && region.currentActivation > 0.2) {
                // Create twinkling/christmas lights effect
                // Each region has a different phase based on its position
                const regionPhase = (region.data.coordinates[0] + region.data.coordinates[1]) * 0.1;
                const pulse = Math.sin(this.activityPhase + regionPhase) * 0.5 + 0.5;
                
                // Add pulsing glow on top of activation
                emissiveIntensity += pulse * 0.3 * region.currentActivation;
                
                // Also pulse the glow opacity
                region.glow.visible = true;
                region.glow.material.opacity = (0.1 + region.currentActivation * 0.3) * (0.7 + pulse * 0.3);
            } else {
                // Standard glow behavior
                region.glow.visible = region.currentActivation > 0.3;
                region.glow.material.opacity = region.currentActivation * 0.3;
            }
            
            region.mesh.material.emissiveIntensity = emissiveIntensity;
        });

        // Time series animation
        if (this.isAnimating && this.activationData.length > 0) {
            this.currentTime += 0.016;  // ~60fps
            this.updateCurrentFrame();
        }

        this.renderer.render(this.scene, this.camera);
    }

    updateCurrentFrame() {
        // Find closest frame in time series
        const frame = this.activationData.find(f => f.time >= this.currentTime);
        if (frame) {
            this.updateActivations(frame.activations);
        }
    }

    // ============================================
    // Cleanup
    // ============================================

    destroy() {
        // CRITICAL FIX: Set destroyed flag to stop animation loop
        this.isDestroyed = true;
        
        // Cancel animation frame
        if (this.animationFrame) {
            cancelAnimationFrame(this.animationFrame);
            this.animationFrame = null;
        }
        
        // CRITICAL FIX: Properly dispose of all Three.js objects to prevent memory leaks
        // Traverse scene and dispose of all geometries, materials, and textures
        if (this.scene) {
            this.scene.traverse((object) => {
                if (object.geometry) {
                    object.geometry.dispose();
                }
                if (object.material) {
                    if (Array.isArray(object.material)) {
                        object.material.forEach(material => {
                            if (material.map) material.map.dispose();
                            if (material.lightMap) material.lightMap.dispose();
                            if (material.bumpMap) material.bumpMap.dispose();
                            if (material.normalMap) material.normalMap.dispose();
                            if (material.specularMap) material.specularMap.dispose();
                            if (material.envMap) material.envMap.dispose();
                            material.dispose();
                        });
                    } else {
                        if (object.material.map) object.material.map.dispose();
                        if (object.material.lightMap) object.material.lightMap.dispose();
                        if (object.material.bumpMap) object.material.bumpMap.dispose();
                        if (object.material.normalMap) object.material.normalMap.dispose();
                        if (object.material.specularMap) object.material.specularMap.dispose();
                        if (object.material.envMap) object.material.envMap.dispose();
                        object.material.dispose();
                    }
                }
            });
            
            // Clear scene
            while(this.scene.children.length > 0) {
                this.scene.remove(this.scene.children[0]);
            }
        }
        
        // Dispose of controls
        if (this.controls) {
            this.controls.dispose();
            this.controls = null;
        }
        
        // CRITICAL FIX: Dispose renderer and remove canvas
        if (this.renderer) {
            this.renderer.dispose();
            // Force context loss to free GPU memory
            const gl = this.renderer.domElement.getContext('webgl') || this.renderer.domElement.getContext('experimental-webgl');
            if (gl) {
                const ext = gl.getExtension('WEBGL_lose_context');
                if (ext) ext.loseContext();
            }
            if (this.renderer.domElement && this.renderer.domElement.parentNode) {
                this.renderer.domElement.parentNode.removeChild(this.renderer.domElement);
            }
            this.renderer = null;
        }
        
        // Clear all internal maps
        if (this.regions) {
            this.regions.clear();
        }
        if (this.connections) {
            this.connections.clear();
        }
        
        // Remove all event listeners
        window.removeEventListener('resize', this.handleResize);
        window.removeEventListener('keydown', this.handleKeydown);
        this.container.removeEventListener('mousemove', this.handleMouseMove);
    }
}

// ============================================
// UI Controls Component
// ============================================

class BrainVisualizationControls {
    constructor(visualization, containerId) {
        this.viz = visualization;
        this.container = document.getElementById(containerId);
        this.createControls();
    }

    createControls() {
        this.container.innerHTML = `
            <div class="brain-controls">
                <div class="control-group">
                    <button id="btn-play" title="Play">▶</button>
                    <button id="btn-pause" title="Pause">⏸</button>
                    <input type="range" id="time-slider" min="0" max="100" value="0">
                    <span id="time-display">0.0s</span>
                </div>
                <div class="control-group">
                    <label>View:</label>
                    <select id="view-select">
                        <option value="default">All Systems (Brain + Body)</option>
                        <option value="cranial">Cranial / Neural Only</option>
                        <option value="body">Peripheral Biomarker Organs</option>
                        <option value="dopamine">Dopamine &amp; Skeletal Drive</option>
                        <option value="cortisol">Stress / HPA / Glycemic</option>
                        <option value="serotonin">Serotonin / Gut / Circadian</option>
                        <option value="social">Social / Oxytocin / Cellular</option>
                    </select>
                    <label for="dissect-slider">Dissect:</label>
                    <input type="range" id="dissect-slider" min="0" max="100" value="0">
                    <span id="dissect-display">0%</span>
                    <button id="btn-export">Export Image</button>
                </div>
                <div class="legend">
                    <div class="legend-item">
                        <span class="color-box" style="background: #FFD700"></span>
                        <span>Dopamine</span>
                    </div>
                    <div class="legend-item">
                        <span class="color-box" style="background: #FF69B4"></span>
                        <span>Oxytocin</span>
                    </div>
                    <div class="legend-item">
                        <span class="color-box" style="background: #FF4500"></span>
                        <span>Cortisol</span>
                    </div>
                    <div class="legend-item">
                        <span class="color-box" style="background: #FF8C00"></span>
                        <span>Adrenaline</span>
                    </div>
                    <div class="legend-item">
                        <span class="color-box" style="background: #6495ED"></span>
                        <span>Serotonin</span>
                    </div>
                </div>
            </div>
        `;

        // Add styles
        const style = document.createElement('style');
        style.textContent = `
            .brain-controls {
                background: rgba(0, 0, 0, 0.8);
                color: white;
                padding: 15px;
                border-radius: 8px;
                font-family: sans-serif;
            }
            .control-group {
                margin-bottom: 10px;
                display: flex;
                align-items: center;
                gap: 10px;
            }
            .control-group button {
                background: #333;
                color: white;
                border: 1px solid #555;
                padding: 5px 15px;
                cursor: pointer;
                border-radius: 4px;
            }
            .control-group button:hover {
                background: #444;
            }
            .control-group input[type="range"] {
                flex: 1;
                height: 2px;
                -webkit-appearance: none;
                appearance: none;
                background: #ffffff;
                border-radius: 0;
                outline: none;
                border: none;
            }
            .control-group input[type="range"]::-webkit-slider-thumb {
                -webkit-appearance: none;
                appearance: none;
                width: 16px;
                height: 16px;
                background: #ffffff;
                border-radius: 0;
                border: none;
                margin-top: -7px;
                cursor: pointer;
            }
            .control-group input[type="range"]::-moz-range-thumb {
                width: 16px;
                height: 16px;
                background: #ffffff;
                border-radius: 0;
                border: none;
                cursor: pointer;
            }
            .legend {
                display: flex;
                flex-wrap: wrap;
                gap: 15px;
                margin-top: 10px;
            }
            .legend-item {
                display: flex;
                align-items: center;
                gap: 5px;
                font-size: 12px;
            }
            .color-box {
                width: 16px;
                height: 16px;
                border-radius: 3px;
            }
        `;
        document.head.appendChild(style);

        // Event listeners
        document.getElementById('btn-play').addEventListener('click', () => this.viz.play());
        document.getElementById('btn-pause').addEventListener('click', () => this.viz.pause());
        
        document.getElementById('time-slider').addEventListener('input', (e) => {
            const time = parseFloat(e.target.value) / 10;
            this.viz.seek(time);
            document.getElementById('time-display').textContent = time.toFixed(1) + 's';
        });

        document.getElementById('btn-export').addEventListener('click', () => {
            const dataUrl = this.viz.exportImage();
            const link = document.createElement('a');
            link.download = 'brain-activation.png';
            link.href = dataUrl;
            link.click();
        });

        document.getElementById('view-select').addEventListener('change', (e) => {
            this.setView(e.target.value);
        });
        document.getElementById('dissect-slider')?.addEventListener('input', (e) => {
            const pct = Number(e.target.value) || 0;
            this.viz.setExplosionFactor(pct / 100.0);
            const disp = document.getElementById('dissect-display');
            if (disp) disp.textContent = `${pct}%`;
        });
    }

    setView(viewName) {
        // Reset all regions to visible
        this.viz.regions.forEach(region => {
            region.mesh.visible = true;
        });

        switch (viewName) {
            case 'cranial':
                this.viz.regions.forEach((region) => {
                    region.mesh.visible = (region.data.tier || 'cranial') === 'cranial';
                });
                break;
            case 'body':
                this.viz.regions.forEach((region) => {
                    region.mesh.visible = region.data.tier === 'body';
                });
                break;
            case 'dopamine':
                this.viz.regions.forEach((region) => {
                    region.mesh.visible = region.data.neuromodulator === 'dopamine';
                });
                break;
            case 'cortisol':
                this.viz.regions.forEach((region) => {
                    region.mesh.visible = ['cortisol', 'adrenaline'].includes(region.data.neuromodulator);
                });
                break;
            case 'serotonin':
                this.viz.regions.forEach((region) => {
                    region.mesh.visible = region.data.neuromodulator === 'serotonin';
                });
                break;
            case 'social':
                this.viz.regions.forEach((region) => {
                    region.mesh.visible = ['oxytocin', 'mixed'].includes(region.data.neuromodulator);
                });
                break;
        }
    }
}

// ============================================
// Export
// ============================================

if (typeof module !== 'undefined' && module.exports) {
    module.exports = { BrainVisualization, BrainVisualizationControls, BRAIN_REGIONS };
}
