/**
 * Visualization Panel Module for Agent Society Simulator
 * 
 * Displays generated PNG charts from simulation runs.
 * Shows time series, scatter plots, and histograms in a tabbed interface.
 * 
 * Features:
 * - Auto-loads visualizations when run is selected
 * - Tabbed interface for different chart types
 * - Zoomable image viewer
 * - Download functionality
 */

const VisualizationPanel = (function() {
    'use strict';

    // DOM Elements
    let panel = null;
    let tabsContainer = null;
    let contentContainer = null;
    let currentRunId = null;

    // Chart categories
    const CHART_CATEGORIES = {
        'time-series': {
            label: 'Time Series',
            icon: '📈',
            files: [
                'agent_count.png',
                'cooperation_rate.png',
                'gini.png',
                'avg_energy.png',
                'trade_volume.png',
                'branching_ratio.png',
                'dispute_rate.png'
            ]
        },
        'correlations': {
            label: 'Correlations',
            icon: '📊',
            files: [
                'cooperation_vs_gini.png',
                'multi_metric.png'
            ]
        },
        'distributions': {
            label: 'Distributions',
            icon: '📉',
            files: [
                'lambda_distribution.png',
                'docas_distribution.png'
            ]
        }
    };

    /**
     * Initialize the visualization panel
     */
    function init() {
        console.log('[VisualizationPanel] Initializing...');

        // Create panel if it doesn't exist
        createPanel();
        
        console.log('[VisualizationPanel] Initialized successfully');
    }

    /**
     * Create the visualization panel DOM structure
     */
    function createPanel() {
        // Check if panel already exists
        if (document.getElementById('viz-panel')) {
            panel = document.getElementById('viz-panel');
            return;
        }

        // Create panel
        panel = document.createElement('div');
        panel.id = 'viz-panel';
        panel.className = 'viz-panel';
        panel.innerHTML = `
            <div class="viz-header">
                <h3 class="viz-title">📊 Visualizations</h3>
                <span class="viz-status" id="viz-status">No run selected</span>
                <button class="viz-close" id="viz-close" title="Close">×</button>
            </div>
            <div class="viz-tabs" id="viz-tabs"></div>
            <div class="viz-content" id="viz-content">
                <div class="viz-placeholder">
                    <div class="viz-placeholder-icon">📈</div>
                    <p>Select a simulation run to view visualizations</p>
                </div>
            </div>
        `;

        // Add to body
        document.body.appendChild(panel);

        // Cache elements
        tabsContainer = document.getElementById('viz-tabs');
        contentContainer = document.getElementById('viz-content');

        // Setup event listeners
        document.getElementById('viz-close').addEventListener('click', hide);

        // Create tabs
        createTabs();
    }

    /**
     * Create category tabs
     */
    function createTabs() {
        if (!tabsContainer) return;

        tabsContainer.innerHTML = '';
        
        Object.entries(CHART_CATEGORIES).forEach(([key, config], index) => {
            const tab = document.createElement('button');
            tab.className = `viz-tab ${index === 0 ? 'active' : ''}`;
            tab.dataset.category = key;
            tab.innerHTML = `${config.icon} ${config.label}`;
            tab.addEventListener('click', () => switchTab(key));
            tabsContainer.appendChild(tab);
        });
    }

    /**
     * Switch to a different tab
     */
    function switchTab(category) {
        // Update tab styles
        document.querySelectorAll('.viz-tab').forEach(tab => {
            tab.classList.toggle('active', tab.dataset.category === category);
        });

        // Load charts for this category
        loadCategoryCharts(category);
    }

    /**
     * Load charts for a category
     */
    function loadCategoryCharts(category) {
        if (!currentRunId) {
            showPlaceholder('No run selected');
            return;
        }

        const config = CHART_CATEGORIES[category];
        if (!config) return;

        contentContainer.innerHTML = '<div class="viz-loading">Loading charts...</div>';

        // Try to load each chart
        const chartsHtml = config.files.map(file => `
            <div class="viz-chart" data-file="${file}">
                <div class="viz-chart-title">${formatTitle(file)}</div>
                <img 
                    src="/api/runs/${currentRunId}/figures/${file}" 
                    alt="${formatTitle(file)}"
                    class="viz-chart-img"
                >
                <div class="viz-chart-error-msg">Chart not available</div>
            </div>
        `).join('');

        contentContainer.innerHTML = `<div class="viz-grid">${chartsHtml}</div>`;

        // Programmatic event binding compliant with W3C CSP Level 3 (script-src-attr)
        contentContainer.querySelectorAll('.viz-chart-img').forEach(img => {
            img.addEventListener('load', () => {
                if (img.parentElement) img.parentElement.classList.add('viz-chart-loaded');
            });
            img.addEventListener('error', () => {
                if (img.parentElement) img.parentElement.classList.add('viz-chart-error');
            });
        });
    }

    /**
     * Format filename into title
     */
    function formatTitle(filename) {
        return filename
            .replace('.png', '')
            .replace(/_/g, ' ')
            .replace(/\b\w/g, l => l.toUpperCase());
    }

    /**
     * Show placeholder message
     */
    function showPlaceholder(message) {
        if (!contentContainer) return;
        contentContainer.textContent = '';
        const wrap = document.createElement('div');
        wrap.className = 'viz-placeholder';
        const icon = document.createElement('div');
        icon.className = 'viz-placeholder-icon';
        icon.textContent = '📈';
        const p = document.createElement('p');
        p.textContent = message == null ? '' : String(message);
        wrap.appendChild(icon);
        wrap.appendChild(p);
        contentContainer.appendChild(wrap);
    }

    /**
     * Load visualizations for a run
     */
    function loadForRun(runId) {
        currentRunId = runId;
        
        const statusEl = document.getElementById('viz-status');
        if (statusEl) {
            statusEl.textContent = `Run: ${runId.substring(0, 8)}...`;
        }

        // Load first category
        const firstCategory = Object.keys(CHART_CATEGORIES)[0];
        switchTab(firstCategory);
        
        show();
    }

    /**
     * Show the panel
     */
    function show() {
        if (panel) {
            panel.classList.add('viz-panel-visible');
        }
    }

    /**
     * Hide the panel
     */
    function hide() {
        if (panel) {
            panel.classList.remove('viz-panel-visible');
        }
    }

    /**
     * Toggle panel visibility
     */
    function toggle() {
        if (panel) {
            panel.classList.toggle('viz-panel-visible');
        }
    }

    // Public API
    return {
        init,
        loadForRun,
        show,
        hide,
        toggle
    };
})();

// Auto-initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', VisualizationPanel.init);
} else {
    VisualizationPanel.init();
}
