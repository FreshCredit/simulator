/**
 * CLICS-4 Colexification Graph Visualization
 * 
 * D3.js-based force-directed graph visualization showing the agent's
 * language/lexicon structure based on CLICS-4 (Database of Cross-Linguistic Colexifications v4.0).
 * 
 * Nodes represent concepts/words, edges show colexification patterns
 * (same word used for multiple concepts).
 * 
 * @module CLICSVisualization
 * @version 1.0.0
 */

// ============================================
// Concept Category Colors (matching simulator theme)
// ============================================

const CONCEPT_CATEGORY_COLORS = {
    Trust: '#FFD700',      // Gold
    Social: '#FF69B4',     // Hot pink
    Resource: '#32CD32',   // Lime green
    Emotion: '#FF4500',    // Red orange
    Action: '#6495ED',     // Cornflower blue
    Unknown: '#9370DB'     // Medium purple
};

const CONCEPT_CATEGORY_LABELS = {
    Trust: 'Trust',
    Social: 'Social',
    Resource: 'Resource',
    Emotion: 'Emotion',
    Action: 'Action'
};

// ============================================
// CLICS Visualization Class
// ============================================

class CLICSVisualization {
    constructor(containerId, options = {}) {
        this.container = document.getElementById(containerId);
        if (!this.container) {
            throw new Error(`Container #${containerId} not found`);
        }

        // Configuration
        this.options = {
            width: options.width || this.container.clientWidth,
            height: options.height || this.container.clientHeight,
            backgroundColor: options.backgroundColor || '#1a1a2e',
            nodeRadius: options.nodeRadius || 8,
            linkDistance: options.linkDistance || 60,
            chargeStrength: options.chargeStrength || -150,
            collisionRadius: options.collisionRadius || 15,
            ...options
        };

        // State
        this.nodes = [];
        this.links = [];
        this.simulation = null;
        this.svg = null;
        this.g = null;
        this.currentAgentId = null;
        
        // D3 selections
        this.linkSelection = null;
        this.nodeSelection = null;
        this.labelSelection = null;
        
        // Zoom behavior
        this.zoom = null;
        
        // CRITICAL FIX: Store bound handler for proper removal
        this.boundHandleResize = this.handleResize.bind(this);
        
        // CRITICAL FIX: Track destroyed state
        this.isDestroyed = false;

        // Initialize
        this.init();
    }

    init() {
        // Clear container
        this.container.innerHTML = '';
        
        // Create SVG
        this.svg = d3.select(this.container)
            .append('svg')
            .attr('width', this.options.width)
            .attr('height', this.options.height)
            .attr('viewBox', [0, 0, this.options.width, this.options.height]);
        
        // Add background
        this.svg.append('rect')
            .attr('width', '100%')
            .attr('height', '100%')
            .attr('fill', this.options.backgroundColor);
        
        // Add zoom group
        this.g = this.svg.append('g');
        
        // Setup zoom behavior
        this.zoom = d3.zoom()
            .scaleExtent([0.3, 3])
            .on('zoom', (event) => {
                this.g.attr('transform', event.transform);
            });
        
        this.svg.call(this.zoom);
        
        // Create legend
        this.createLegend();
        
        // Initialize empty simulation
        this.initSimulation();
        
        // CRITICAL FIX: Use bound handler so it can be properly removed
        window.addEventListener('resize', this.boundHandleResize);
    }

    createLegend() {
        const legendData = Object.entries(CONCEPT_CATEGORY_COLORS)
            .filter(([key]) => key !== 'Unknown');
        
        const legend = this.svg.append('g')
            .attr('class', 'clics-legend')
            .attr('transform', `translate(10, ${this.options.height - legendData.length * 20 - 10})`);
        
        legendData.forEach(([category, color], i) => {
            const row = legend.append('g')
                .attr('transform', `translate(0, ${i * 20})`);
            
            row.append('circle')
                .attr('r', 5)
                .attr('cx', 5)
                .attr('cy', 5)
                .attr('fill', color);
            
            row.append('text')
                .attr('x', 15)
                .attr('y', 9)
                .attr('fill', '#ccc')
                .attr('font-size', '11px')
                .attr('font-family', 'Inter, sans-serif')
                .text(CONCEPT_CATEGORY_LABELS[category] || category);
        });
    }

    initSimulation() {
        this.simulation = d3.forceSimulation()
            .force('link', d3.forceLink().id(d => d.id).distance(this.options.linkDistance))
            .force('charge', d3.forceManyBody().strength(this.options.chargeStrength))
            .force('center', d3.forceCenter(this.options.width / 2, this.options.height / 2))
            .force('collision', d3.forceCollide().radius(this.options.collisionRadius));
    }

    /**
     * Set the current agent ID to track agent switches
     * @param {number} agentId - The current agent's ID
     */
    setCurrentAgent(agentId) {
        if (this.currentAgentId !== null && this.currentAgentId !== agentId) {
            console.log(`[CLICS] Switching from agent ${this.currentAgentId} to ${agentId}`);
        }
        this.currentAgentId = agentId;
    }

    showEmptyState() {
        if (!this.g) return;
        this.nodes = [];
        this.links = [];
        this.render();
        if (this.simulation) this.simulation.stop();
        
        let emptyText = this.g.select('.clics-empty-state');
        if (emptyText.empty()) {
            this.g.append('text')
                .attr('class', 'clics-empty-state')
                .attr('x', this.options.width / 2)
                .attr('y', this.options.height / 2)
                .attr('text-anchor', 'middle')
                .attr('fill', '#6b7280')
                .attr('font-size', '13px')
                .attr('font-family', 'Inter, sans-serif')
                .text('No language concepts formed yet');
        }
    }

    /**
     * Update visualization from lexicon data
     * @param {Object} lexicon - Agent's lexicon with concepts and colexifications
     */
    updateFromLexicon(lexicon) {
        if (this.g) {
            this.g.selectAll('.clics-empty-state').remove();
        }
        if (!lexicon || !lexicon.concepts || lexicon.concepts.length === 0) {
            this.showEmptyState();
            return;
        }

        // Convert concepts to nodes
        const newNodes = [];
        const nodeMap = new Map();
        
        lexicon.concepts.forEach((concept, index) => {
            const node = {
                id: concept.id || index,
                name: concept.name || `Concept ${concept.id || index}`,
                category: concept.category || 'Unknown',
                embedding: concept.embedding || [],
                frequency: concept.frequency || 1.0,
                createdAt: concept.created_at || 0,
                radius: this.options.nodeRadius + (concept.frequency || 1.0) * 3
            };
            newNodes.push(node);
            nodeMap.set(node.id, node);
        });

        // Convert colexifications to links
        const newLinks = [];
        
        if (lexicon.colexifications) {
            lexicon.colexifications.forEach(([source, target, strength]) => {
                if (nodeMap.has(source) && nodeMap.has(target)) {
                    newLinks.push({
                        source: source,
                        target: target,
                        strength: strength || 0.5
                    });
                }
            });
        }
        
        // Also create links based on colexification_map if available
        if (lexicon.colexification_map) {
            lexicon.colexification_map.forEach((pair) => {
                const [source, target] = Array.isArray(pair) ? pair : pair.split(',').map(Number);
                if (nodeMap.has(source) && nodeMap.has(target)) {
                    // Check if link already exists
                    const exists = newLinks.some(link => 
                        (link.source === source && link.target === target) ||
                        (link.source === target && link.target === source)
                    );
                    if (!exists) {
                        newLinks.push({
                            source: source,
                            target: target,
                            strength: 0.5
                        });
                    }
                }
            });
        }
        
        // Create semantic similarity links based on embedding proximity
        // (for concepts without direct colexification)
        if (newNodes.length > 1 && newLinks.length < newNodes.length) {
            const similarityThreshold = 0.7;
            
            for (let i = 0; i < newNodes.length; i++) {
                for (let j = i + 1; j < newNodes.length; j++) {
                    // Skip if already connected
                    const alreadyConnected = newLinks.some(link => 
                        (link.source === newNodes[i].id && link.target === newNodes[j].id) ||
                        (link.source === newNodes[j].id && link.target === newNodes[i].id)
                    );
                    
                    if (!alreadyConnected && newNodes[i].embedding && newNodes[j].embedding) {
                        const similarity = this.computeCosineSimilarity(
                            newNodes[i].embedding,
                            newNodes[j].embedding
                        );
                        
                        if (similarity > similarityThreshold) {
                            newLinks.push({
                                source: newNodes[i].id,
                                target: newNodes[j].id,
                                strength: similarity * 0.3, // Weaker than colexification
                                semantic: true // Mark as semantic link
                            });
                        }
                    }
                }
            }
        }

        this.nodes = newNodes;
        this.links = newLinks;

        this.render();
    }

    computeCosineSimilarity(a, b) {
        if (!a || !b || a.length !== b.length) return 0;
        
        let dotProduct = 0;
        let normA = 0;
        let normB = 0;
        
        for (let i = 0; i < a.length; i++) {
            dotProduct += a[i] * b[i];
            normA += a[i] * a[i];
            normB += b[i] * b[i];
        }
        
        if (normA === 0 || normB === 0) return 0;
        
        return dotProduct / (Math.sqrt(normA) * Math.sqrt(normB));
    }

    render() {
        // Clear previous elements
        this.g.selectAll('*').remove();

        if (this.nodes.length === 0) {
            this.g.append('text')
                .attr('x', this.options.width / 2)
                .attr('y', this.options.height / 2)
                .attr('text-anchor', 'middle')
                .attr('fill', '#888')
                .attr('font-size', '14px')
                .text('No lexicon data available');
            return;
        }

        // Draw links
        this.linkSelection = this.g.append('g')
            .attr('class', 'links')
            .selectAll('line')
            .data(this.links)
            .join('line')
            .attr('stroke', d => d.semantic ? '#4a4a6a' : '#6a6a8a')
            .attr('stroke-width', d => Math.max(1, d.strength * 3))
            .attr('stroke-opacity', d => d.semantic ? 0.4 : 0.7)
            .attr('stroke-dasharray', d => d.semantic ? '3,3' : null);

        // Draw nodes
        this.nodeSelection = this.g.append('g')
            .attr('class', 'nodes')
            .selectAll('circle')
            .data(this.nodes)
            .join('circle')
            .attr('r', d => d.radius)
            .attr('fill', d => CONCEPT_CATEGORY_COLORS[d.category] || CONCEPT_CATEGORY_COLORS.Unknown)
            .attr('stroke', '#fff')
            .attr('stroke-width', 1.5)
            .attr('cursor', 'pointer')
            .call(d3.drag()
                .on('start', (event, d) => this.dragstarted(event, d))
                .on('drag', (event, d) => this.dragged(event, d))
                .on('end', (event, d) => this.dragended(event, d)));

        // Add tooltips
        this.nodeSelection.append('title')
            .text(d => `${d.name}\nCategory: ${d.category}\nFrequency: ${d.frequency.toFixed(2)}`);

        // Draw labels for larger nodes
        this.labelSelection = this.g.append('g')
            .attr('class', 'labels')
            .selectAll('text')
            .data(this.nodes.filter(d => d.radius > 10 || d.frequency > 1.5))
            .join('text')
            .attr('text-anchor', 'middle')
            .attr('dy', d => d.radius + 12)
            .attr('fill', '#ddd')
            .attr('font-size', '10px')
            .attr('font-family', 'Inter, sans-serif')
            .attr('pointer-events', 'none')
            .text(d => d.name.length > 12 ? d.name.substring(0, 10) + '...' : d.name);

        // Restart simulation
        this.simulation
            .nodes(this.nodes)
            .on('tick', () => this.ticked());

        this.simulation.force('link')
            .links(this.links);

        this.simulation.alpha(1).restart();
        
        // Add stats text
        this.updateStats();
    }

    ticked() {
        // Clamp nodes to SVG boundary to prevent viewport overflow
        const radius = this.options.nodeRadius || 12;
        this.nodes.forEach(d => {
            d.x = Math.max(radius + 5, Math.min(this.options.width - radius - 5, d.x));
            d.y = Math.max(radius + 5, Math.min(this.options.height - radius - 5, d.y));
        });

        if (this.linkSelection) {
            this.linkSelection
                .attr('x1', d => d.source.x)
                .attr('y1', d => d.source.y)
                .attr('x2', d => d.target.x)
                .attr('y2', d => d.target.y);
        }

        if (this.nodeSelection) {
            this.nodeSelection
                .attr('cx', d => d.x)
                .attr('cy', d => d.y);
        }

        if (this.labelSelection) {
            this.labelSelection
                .attr('x', d => d.x)
                .attr('y', d => d.y);
        }
    }

    dragstarted(event, d) {
        if (!event.active) this.simulation.alphaTarget(0.3).restart();
        d.fx = d.x;
        d.fy = d.y;
    }

    dragged(event, d) {
        d.fx = event.x;
        d.fy = event.y;
    }

    dragended(event, d) {
        if (!event.active) this.simulation.alphaTarget(0);
        d.fx = null;
        d.fy = null;
    }

    updateStats() {
        // Remove old stats
        this.svg.selectAll('.clics-stats').remove();
        
        // Add stats in top-right corner
        const stats = this.svg.append('g')
            .attr('class', 'clics-stats')
            .attr('transform', `translate(${this.options.width - 120}, 20)`);
        
        stats.append('rect')
            .attr('width', 110)
            .attr('height', 50)
            .attr('rx', 4)
            .attr('fill', 'rgba(0,0,0,0.5)')
            .attr('stroke', '#444');
        
        stats.append('text')
            .attr('x', 55)
            .attr('y', 18)
            .attr('text-anchor', 'middle')
            .attr('fill', '#fff')
            .attr('font-size', '11px')
            .attr('font-weight', 'bold')
            .text(`Concepts: ${this.nodes.length}`);
        
        stats.append('text')
            .attr('x', 55)
            .attr('y', 36)
            .attr('text-anchor', 'middle')
            .attr('fill', '#aaa')
            .attr('font-size', '10px')
            .text(`Links: ${this.links.length}`);
    }

    handleResize() {
        this.options.width = this.container.clientWidth;
        this.options.height = this.container.clientHeight;
        
        this.svg
            .attr('width', this.options.width)
            .attr('height', this.options.height)
            .attr('viewBox', [0, 0, this.options.width, this.options.height]);
        
        this.simulation.force('center', 
            d3.forceCenter(this.options.width / 2, this.options.height / 2));
        this.simulation.alpha(0.3).restart();
    }

    /**
     * Export current graph as SVG
     */
    exportSVG() {
        const svgData = new XMLSerializer().serializeToString(this.svg.node());
        return 'data:image/svg+xml;base64,' + btoa(svgData);
    }

    /**
     * Destroy the visualization and clean up
     */
    destroy() {
        // CRITICAL FIX: Set destroyed flag
        this.isDestroyed = true;
        
        // Stop simulation
        if (this.simulation) {
            this.simulation.stop();
            this.simulation = null;
        }
        
        // Remove SVG
        if (this.svg) {
            this.svg.remove();
            this.svg = null;
        }
        
        // CRITICAL FIX: Use bound handler for proper removal
        window.removeEventListener('resize', this.boundHandleResize);
        
        // Clear data
        this.nodes = [];
        this.links = [];
        this.g = null;
        this.currentAgentId = null;
    }
}

// ============================================
// Export
// ============================================

if (typeof module !== 'undefined' && module.exports) {
    module.exports = { CLICSVisualization, CONCEPT_CATEGORY_COLORS };
}
