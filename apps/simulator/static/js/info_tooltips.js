/**
 * Agent Society Simulator - Scientific Info Tooltips
 * 
 * Issue 3 Fix: Provides scientific documentation and citations for all
 * simulation attributes per the 9-layer architecture audit.
 * 
 * Sources:
 * - docs/simulation/audit/FRAMEWORK_AUDIT_REPORT.md
 * - simulator_9layer_architecture_audit.md
 */

const ScientificInfo = (function() {
    'use strict';

    // Scientific documentation database per 9-layer architecture
    const infoDatabase = {
        // Core Research Hypothesis (GTT v4.5)
        'hypothesis': {
            title: 'Core Research Hypothesis: Is Trust Measurable?',
            layer: 'General Theory of Trust (GTT v4.5)',
            description: 'Trust is not an arbitrary sentiment; it functions as a bounded, asymmetric stabilizing operator that minimizes prediction error across biological, linguistic, and financial domains.',
            science: 'v4.5.0 Master Law: ΔS_t = r_t[λ⁺ε_t⁺ - λ⁻ε_t⁻] - δ_S(S_t - S_base), proven inductively invariant on [S_base - λ⁻/δ_S, S_base + λ⁺/δ_S] in Z3 SMT and Lean 4.35 (32 theorems).',
            citation: 'The General Theory of Trust v4.5 (Shigaki, 2026); Lean 4.35 & Z3 Formal Verification Suite.',
            parameters: [
                'Master Law: ΔS_t = r_t[λ⁺ε_t⁺ - λ⁻ε_t⁻] - δ_S(S_t - S_base)',
                'BIBO Corridor: [S_base - λ⁻/δ_S, S_base + λ⁺/δ_S]',
                'Trust Emergence: τ = (D + O + S) / (1 + C + A)',
                'Anti-Hypothesis (H₀): Trust is unmeasurable subjective sentiment (λ⁺ = λ⁻ = 0, ΔH ≥ 0)'
            ]
        },

        // Layer 1: Geography
        'mech-encounters': {
            title: 'Spatial Encounters',
            layer: 'Layer 1: Geography',
            description: 'Agents move on a 2D grid and interact when they occupy adjacent cells. This spatial structure creates local neighborhoods and enables emergent social dynamics.',
            science: 'Based on Sugarscape model (Epstein & Axtell, 1996) and spatial game theory. Local interactions are the foundation of all higher-level social phenomena.',
            citation: 'Epstein, J.M. & Axtell, R. (1996). Growing Artificial Societies. Brookings Institution Press.',
            parameters: ['Grid size: 64x64', 'Movement: 8-directional', 'Interaction radius: 1 cell']
        },
        'speed-slider': {
            title: 'Simulation Speed',
            layer: 'Layer 1: Geography',
            description: 'Controls the tick rate of the simulation without affecting the underlying dynamics.',
            science: 'Purely a visualization convenience - simulation logic remains deterministic.',
            parameters: ['Range: 0.5x to 2.0x', 'Base tick: 100ms']
        },
        
        // Layer 2: Genealogy
        'mech-genome': {
            title: 'Genetic Inheritance',
            layer: 'Layer 2: Genealogy',
            description: 'Agents inherit traits from parents through sexual reproduction with crossover and mutation.',
            science: 'Implements Mendelian inheritance with Kingman coalescent for genealogy tracking. DOCAS traits (β parameters) are heritable and determine agent behavioral predispositions.',
            citation: 'Kingman, J.F.C. (1982). The Coalescent. Stochastic Processes and Applications.',
            parameters: ['Crossover rate: 0.5', 'Mutation rate: 0.01', 'Traits: 5 DOCAS β values']
        },
        'dial-kingman': {
            title: 'Kingman Intensity',
            layer: 'Layer 2: Genealogy',
            description: 'Controls the rate of lineage branching in the genealogical tree.',
            science: 'Higher intensity creates more diverse lineages; lower intensity creates more stable family structures.',
            parameters: ['Range: 0.1 - 2.0', 'Default: 1.0']
        },
        'dial-lineage-depth': {
            title: 'Lineage Depth',
            layer: 'Layer 2: Genealogy',
            description: 'Maximum number of generations tracked in the genealogy.',
            science: 'Deeper lineages enable better kin selection analysis but require more memory.',
            parameters: ['Range: 1-50 generations', 'Default: 10']
        },
        
        // Layer 3: Biology
        'mech-biology': {
            title: 'Life Dynamics',
            layer: 'Layer 3: Biology',
            description: 'Core life processes including energy metabolism, aging, and death.',
            science: 'Energy-based agent model. Agents consume energy to move and act, gain energy from resources, and die when energy reaches zero.',
            citation: 'Based on metabolic ecology (West et al., 1997) and agent-based life history theory.',
            parameters: ['Initial energy: 100', 'Metabolic cost: 1/tick', 'Max age: 1000 ticks']
        },
        
        // Layer 3: Biology - Parameter Controls
        'dial-energy-consumption': {
            title: 'Energy Consumption Rate',
            layer: 'Layer 3: Biology',
            description: 'Base energy consumed per tick for metabolic processes.',
            science: 'Higher values mean agents deplete energy faster and die sooner without resources.',
            parameters: ['Range: 0.5-3.0 energy/tick', 'Default: 1.0', 'Affects: Lifespan, resource pressure']
        },
        'dial-movement-cost': {
            title: 'Movement Cost',
            layer: 'Layer 3: Biology',
            description: 'Additional energy cost per cell movement.',
            science: 'Higher costs discourage exploration and favor staying near resources.',
            parameters: ['Range: 0.1-2.0 energy/cell', 'Default: 0.5', 'Affects: Exploration vs exploitation']
        },
        'dial-aging-beta': {
            title: 'Aging Rate (β)',
            layer: 'Layer 3: Biology',
            description: 'Gompertz aging parameter controlling mortality acceleration with age.',
            science: 'Per §6.21.4: β ∈ [0.08, 0.12]. Higher values mean faster aging.',
            citation: 'Gompertz, B. (1825). On the nature of the function expressive of the law of human mortality.',
            parameters: ['Range: 0.08-0.12', 'Default: 0.10', 'Affects: Age-related mortality']
        },
        'dial-repro-threshold': {
            title: 'Reproduction Threshold',
            layer: 'Layer 3: Biology',
            description: 'Minimum energy required to attempt reproduction.',
            science: 'Higher thresholds ensure offspring have better starting conditions but slow population growth.',
            parameters: ['Range: 50-100 energy', 'Default: 75', 'Affects: Population dynamics']
        },
        'dial-mutation-rate': {
            title: 'Mutation Rate',
            layer: 'Layer 3: Biology',
            description: 'Probability of mutation per genetic locus during reproduction.',
            science: 'Per Drake et al. 1998. Higher rates increase genetic diversity but can destabilize adaptations.',
            citation: 'Drake, J.W. et al. (1998). Rates of spontaneous mutation. Genetics.',
            parameters: ['Range: 0.0001-0.01', 'Default: 0.001', 'Affects: Genetic diversity']
        },
        'dial-epigenetic-methylation': {
            title: 'Epigenetic Methylation Rate',
            layer: 'Layer 3: Biology & Epigenetics',
            description: 'Environmental stress marks heritable chromatin methylation across generations.',
            science: 'Calibrated under Sim D2/ECOL: Higher methylation locks stress phenotypes into offspring without genome mutation.',
            parameters: ['Range: 0.0-1.0', 'Default: 0.05']
        },
        'dial-circadian-deficit': {
            title: 'Circadian Sleep Deficit',
            layer: 'Layer 3: Biology & Chronobiology',
            description: 'Daily sleep deficit elevating baseline Cortisol and suppressing Serotonin.',
            science: 'Calibrated under Sim D17: Causes chronic present bias and cognitive bandwidth degradation.',
            parameters: ['Range: 0.0-1.0', 'Default: 0.00']
        },
        'dial-tax-redistribution': {
            title: 'Wealth Redistribution Tax',
            layer: 'Layer 8: Economy',
            description: 'Progressive tax on affluent agent wealth redistributed to impoverished agents.',
            science: 'Reduces Gini inequality and dampens Malthusian starvation collapses during ecological shocks.',
            parameters: ['Range: 0-50%', 'Default: 0%']
        },
        'dial-bank-run-sensitivity': {
            title: 'Bank Run Sunspot Sensitivity',
            layer: 'Layer 8: Economy',
            description: 'Sensitivity of credit markets to default cascades and liquidity hoarding.',
            science: 'Calibrated under Diamond-Dybvig sunspot dynamics: Higher values cause abrupt credit contractions upon defaults.',
            parameters: ['Range: 0.0-2.0x', 'Default: 1.0x']
        },
        
        // Layer 4: Neurobiology
        'dial-docas-alpha': {
            title: 'DOCAS Gain Multiplier (α)',
            layer: 'Layer 4: Neurobiology',
            description: 'Scales DOCAS neuromodulatory learning sensitivities (λ⁺, λ⁻) around the calibrated baseline without distorting homeostatic equilibrium S_base.',
            science: 'Higher α increases agent responsiveness to positive and negative prediction errors ε_t = (V_t - E_t)/s_d.',
            parameters: ['Range: 0.1 - 1.0 (gain scale 0.2x - 2.0x)', 'Default: 0.5 (1.0x)', 'Affects: λ⁺ and λ⁻ sensitivity']
        },
        'dial-curvature-window': {
            title: 'Trust Curvature Window',
            layer: 'Layer 4: Neurobiology',
            description: 'Number of ticks used to detect trust instability.',
            science: 'Larger windows smooth out short-term fluctuations but delay detection of genuine trust crises.',
            parameters: ['Range: 5-50 ticks', 'Default: 10', 'Affects: Trust volatility detection']
        },

        // Layer 5: Psychology
        'dial-planning-horizon': {
            title: 'DMN Planning Horizon (K)',
            layer: 'Layer 5: Psychology',
            description: 'Counterfactual rollout steps evaluated by Default Mode Network planning before accepting or rejecting agreements.',
            science: 'Sim DMN & Sim A1 show K >= 5 anticipates delayed predatory traps and reduces multi-round extortion uptake.',
            citation: 'TDF / DOCAS §5 (Sim DMN) & Sim A1 Adversarial Deception Battery.',
            parameters: ['Range: 1 - 20 steps', 'Default: 10 (Sim DMN target K=5)', 'Affects: DMN counterfactual rollout depth']
        },

        // Layer 7: Ecology
        'dial-trust-decay': {
            title: 'Homeostatic Leak Rate (δ_S)',
            layer: 'Layer 7: Ecology',
            description: 'Baseline-restoring leak rate δ_S in ΔS_t = r_t[λ⁺ε⁺ - λ⁻ε⁻] - δ_S(S_t - S_base).',
            science: 'Controls geometric contraction toward S_base and sets the Z3-verified BIBO invariant bounds [S_base - λ⁻/δ_S, S_base + λ⁺/δ_S].',
            citation: 'The General Theory of Trust v4.5 (Theorem M2 & Z3 Invariant Corridor).',
            parameters: ['Range: 0.0 - 0.01 per tick', 'Default: 0.001', 'Affects: Master Law leak_delta δ_S']
        },
        'dial-reputation-threshold': {
            title: 'Reputation Threshold',
            layer: 'Layer 7: Ecology',
            description: 'Minimum reputation required for credit access and group membership.',
            science: 'Higher thresholds create exclusive communities with higher trust but less inclusivity.',
            parameters: ['Range: 0.0-1.0', 'Default: 0.5', 'Affects: Credit access, group formation']
        },
        
        'mech-docas': {
            title: 'DOCAS Dynamics',
            layer: 'Layer 4: Neurobiology',
            description: 'Five-neuromodulator model: Dopamine, Oxytocin, Cortisol, Adrenaline, Serotonin.',
            science: 'DOCAS model captures the major neuromodulatory systems that regulate behavior. Each neuromodulator has distinct effects: Dopamine (reward/approach), Oxytocin (bonding/trust), Cortisol (stress/avoidance), Adrenaline (arousal/action), Serotonin (contentment/status).',
            citation: 'Based on neuroeconomics research (Glimcher, 2011) and neuromodulatory systems theory.',
            parameters: ['D: [0,1] - Approach/avoidance', 'O: [0,1] - Social bonding', 'C: [0,1] - Stress response', 'A: [0,1] - Arousal/action', 'S: [0,1] - Contentment/status']
        },
        'mech-criticality': {
            title: 'Neural Criticality',
            layer: 'Layer 4: Neurobiology',
            description: 'Neural networks operate at criticality - the boundary between order and chaos.',
            science: 'Brain networks self-organize to critical points for optimal information processing. Criticality maximizes dynamic range, sensitivity, and computational power.',
            citation: 'Beggs, J.M. & Plenz, D. (2003). Neuronal avalanches in neocortical circuits. J. Neurosci.',
            parameters: ['Critical threshold: σ = 1.0', 'Avalanche size: power-law distributed']
        },
        
        // Layer 5: Psychology
        'mech-dmn': {
            title: 'Default Mode Network',
            layer: 'Layer 5: Psychology',
            description: 'Brain network active during rest and social cognition.',
            science: 'DMN enables social cognition, theory of mind, and self-referential processing. Critical for trust decisions and social learning.',
            citation: 'Raichle, M.E. et al. (2001). A default mode of brain function. PNAS.',
            parameters: ['Activation during rest', 'Social cognition substrate']
        },
        'mech-choice': {
            title: 'Choice Architecture',
            layer: 'Layer 5: Psychology',
            description: 'How agents make decisions under uncertainty.',
            science: 'Implements prospect theory-style value functions with loss aversion and probability weighting.',
            citation: 'Kahneman, D. & Tversky, A. (1979). Prospect Theory. Econometrica.',
            parameters: ['Loss aversion: λ = 2.25', 'Risk sensitivity: context-dependent']
        },
        
        // Layer 6: Philosophy
        'mech-language': {
            title: 'Language',
            layer: 'Layer 6: Philosophy',
            description: 'Communication system enabling abstract concept sharing.',
            science: 'Language allows agents to share mental models and coordinate. Semantic drift tracks how meanings evolve in social groups.',
            citation: 'Based on evolutionary linguistics (Kirby et al., 2008) and cultural evolution theory.',
            parameters: ['Semantic drift rate', 'Coherence threshold: 0.7']
        },
        'mech-music': {
            title: 'Music',
            layer: 'Layer 6: Philosophy',
            description: 'Social bonding through rhythmic entrainment.',
            science: 'Music evolved for social cohesion and group coordination. Rhythmic entrainment creates inter-brain synchrony.',
            citation: 'Cross, I. (2009). The evolutionary nature of musical meaning. Musicae Scientiae.',
            parameters: ['Beat synchronization', 'Group cohesion boost']
        },
        'mech-norms': {
            title: 'Shared Norms',
            layer: 'Layer 6: Philosophy',
            description: 'Socially shared expectations about behavior.',
            science: 'Norms emerge from repeated interactions and are enforced through reputation and punishment.',
            citation: 'Bicchieri, C. (2006). The Grammar of Society. Cambridge University Press.',
            parameters: ['Norm emergence threshold', 'Enforcement strength']
        },
        
        // Layer 7: Ecology
        'mech-institutions': {
            title: 'Institutions',
            layer: 'Layer 7: Ecology',
            description: 'Formalized social structures and governance mechanisms.',
            science: 'Institutions reduce transaction costs and enable large-scale cooperation beyond kin and small groups.',
            citation: 'North, D.C. (1990). Institutions, Institutional Change and Economic Performance.',
            parameters: ['Governance rules', 'Enforcement mechanisms']
        },
        'mech-groups': {
            title: 'Social Groups',
            layer: 'Layer 7: Ecology',
            description: 'Coalitions and group membership with in-group favoritism.',
            science: 'Group identity creates bounded reciprocity and enables collective action. Based on social identity theory.',
            citation: 'Tajfel, H. & Turner, J.C. (1979). An integrative theory of intergroup conflict.',
            parameters: ['Group formation threshold', 'In-group bias coefficient']
        },
        'relative-distance-threshold': {
            title: 'Relative Distance Limit',
            layer: 'Layer 7: Ecology',
            description: 'Maximum distance to visualize kinship (lineage) connections.',
            science: 'Limits visualization complexity for large populations while preserving local kinship structure.',
            parameters: ['Range: 0-50 grid units', '0 = show all']
        },
        
        // Layer 8: Economy
        'mech-economics': {
            title: 'Credit & Banking',
            layer: 'Layer 8: Economy',
            description: 'Financial system for resource allocation and credit relationships.',
            science: 'Credit networks enable resource flows and economic coordination. Trust-based lending creates economic interdependence.',
            citation: 'Based on credit network theory and economic sociology (Granovetter, 1985).',
            parameters: ['Credit limit', 'Interest rate', 'Default risk']
        },
        
        // Layer 9: Technology
        'mech-protocol': {
            title: 'Blockchain Protocol',
            layer: 'Layer 9: Technology',
            description: 'Distributed ledger for immutable record-keeping.',
            science: 'Blockchain provides transparency and auditability for institutional records. Enables trustless verification of historical events.',
            citation: 'Nakamoto, S. (2008). Bitcoin: A Peer-to-Peer Electronic Cash System.',
            parameters: ['Consensus mechanism', 'Block time', 'Immutability guarantee']
        },
        
        // Visualization Layers
        'layer-agents': {
            title: 'Agents',
            description: 'Render individual agents on the grid.',
            science: 'Visual representation of agent population and spatial distribution.'
        },
        'layer-terrain': {
            title: 'Terrain',
            description: 'Show topographic contour lines.',
            science: 'White contour lines on transparent background showing elevation changes like a traditional topographic map.'
        },
        'layer-weather': {
            title: 'Weather',
            description: 'Show dynamic weather conditions overlay.',
            science: 'Color-coded weather visualization: sunny (clear), cloudy (gray), rainy (blue), stormy (purple), drought (yellow). Updates dynamically based on simulation weather state.'
        },
        'layer-resources': {
            title: 'Resources',
            description: 'Display 10-level Sugarscape food and energy distribution.',
            science: 'Color-coded resource capacity based on Epstein & Axtell (1996) Sugarscape landscape dynamics.'
        },
        'layer-harvesting': {
            title: 'Harvesting',
            description: 'Display active resource harvesting ripples emanating from foraging agents.',
            science: 'Visual indicator of consumption actions and localized economic extraction.'
        },
        'layer-docas': {
            title: 'Emotions',
            description: 'Color agents by their dominant neuromodulator state.',
            science: 'Transient DOCAS coloring shows current emotional/physiological state vs. heritable genome coloring.'
        },
        'layer-energy': {
            title: 'Energy',
            description: 'Display energy rings around agents.',
            science: 'Visual indicator of metabolic health and remaining lifespan.'
        },
        'layer-groups': {
            title: 'Groups',
            description: 'Show group membership indicators.',
            science: 'Color-coded group affiliation for tracking social structure.'
        },
        'layer-identity': {
            title: 'Identity',
            description: 'Highlight agents with verified identities.',
            science: 'Visual indicator of trust network participation.'
        },
        'layer-network': {
            title: 'Reputation',
            description: 'Display trust network edges.',
            science: 'Graph visualization of attestation relationships weighted by trust (τ).' 
        },
        'layer-relative': {
            title: 'Relative',
            description: 'Show parent-child lineage links.',
            science: 'Kinship network visualization for genealogy analysis.'
        },
        'layer-directions': {
            title: 'Directions',
            description: 'Show movement direction arrows.',
            science: 'Visual indicator of agent heading and velocity.'
        },
        'geo-biome-preset': {
            title: 'Starting Biome',
            layer: 'Layer 1: Geography & Biomes',
            description: 'Selects an ecological starting biome that applies stochastic environmental stressors.',
            science: 'Calibrated under Sim D17/ECOL: Tundra (-3.9% norms, 50% regrowth penalty), Tropics (38°C heatwave, +4% predatory uptake), Archipelago (elevation barriers).'
        },
        'geo-grid-width': {
            title: 'Grid Width',
            layer: 'Layer 1: Geography',
            description: 'Horizontal cell dimension of the physical spatial grid.',
            science: 'Determines the carrying capacity space and interaction radius of agents.',
            parameters: ['Range: 32 - 512 cells', 'Default: 100']
        },
        'geo-grid-height': {
            title: 'Grid Height',
            layer: 'Layer 1: Geography',
            description: 'Vertical cell dimension of the physical spatial grid.',
            science: 'Determines spatial density and local neighborhood structure.',
            parameters: ['Range: 32 - 512 cells', 'Default: 100']
        },
        'geo-resource-density': {
            title: 'Resource Patch Density',
            layer: 'Layer 1: Geography',
            description: 'Probability that any spatial grid cell contains harvestable metabolic energy.',
            science: 'Calibrated under Sim ECOL carrying capacity dynamics. Below 0.15, populations risk Malthusian starvation collapse.',
            parameters: ['Range: 0.0 - 1.0', 'Default: 0.30']
        },
        'geo-initial-resource': {
            title: 'Initial Resource Energy',
            layer: 'Layer 1: Geography',
            description: 'Initial caloric energy capacity stored in each generated resource patch.',
            science: 'Determines baseline metabolic foraging runway for initial agent generation.',
            parameters: ['Range: 10 - 100 energy units', 'Default: 50.0']
        },
        'geo-altitude-effects': {
            title: 'Altitude Movement Cost',
            layer: 'Layer 1: Geography',
            description: 'Enables physical 3D elevation topography where uphill motion incurs caloric energy penalties.',
            science: 'Uphill locomotion costs scale as E_move = cost * dist * (1 + 2.0 * max(0, Delta z)).',
            citation: 'Sim ECOL ecological terrain dynamics (docas_revised.md §13.44).'
        },
        'dial-weather-temp': {
            title: 'Ambient Temperature',
            layer: 'Sim D17: Weather System',
            description: 'Ambient environmental temperature in Celsius.',
            science: 'Extreme temperatures (>35°C or <5°C) compress cognitive bandwidth and induce acute thermal stress.',
            citation: 'Sim D17 Technical Appendix: Weather & Climate Shocks (docas_revised.md §12.3).'
        },
        'toggle-weather-shield': {
            title: 'Weather-Shield',
            layer: 'Sim D17: Intervention',
            description: 'Enforces a 24-hour algorithmic cooling-off delay on credit agreements during extreme heatwaves.',
            science: 'Completely neutralizes predatory loan uptake surges caused by acute environmental thermal stress.',
            citation: 'Sim D17: Protective Shields against Environmental Exploitation.'
        },
        'toggle-baylor-mode': {
            title: 'Baylor Consciousness Gating',
            layer: 'Sim D18: Baylor Gating',
            description: 'Simulates propofol/NREM unconscious states where Cortisol drops below critical threshold (C < 0.12).',
            science: 'Executive threat monitoring (dlPFC, ACC) collapses by 92%, suppressing cognitive friction and loss aversion.',
            citation: 'Sim D18: Unconscious Cognitive Gating in Multi-Agent Neural Systems.'
        },
        'dial-cloaking-ratio': {
            title: 'Visual Cloaking Ratio (χ)',
            layer: 'Sim D16 & D21: Perceptual Salience',
            description: 'Ratio of promotional headline visual salience to fine-print disclaimer salience.',
            science: 'When χ > 1.5, disclaimer discovery drops sharply for low-bandwidth cohorts (Sim D16 38.9x odds ratio), while Anterior Insula norm prediction error is attenuated in Sim D21 TRIBE v2 encoding.',
            citation: 'Sim D16 & Sim D21 TRIBE v2 Forward-Encoding Models (TDF v4.5).'
        },
        'dial-music-tau': {
            title: 'Music Metric Hierarchy (τ)',
            layer: 'Sim M: TDF Music Vector',
            description: 'Ratio of hierarchical meter structure in temporal auditory drives.',
            science: 'Disambiguates 6/8 compound meter from 3/4 simple meter, yielding +136.9% relative mutual information gain.',
            citation: 'Sim M: 5D Topological Deep Feature Music Vector & Oxytocin Coupling.'
        },
        'brain-export-btn': {
            title: '3D Brain PNG Snapshot',
            layer: 'Sim D21: Neuroimaging',
            description: 'Captures a high-resolution PNG snapshot of the active 3D cortical BOLD activations.',
            science: 'Exports empirical Bilateral Anterior Insula, dlPFC, and STG forward-encoded activation maps.'
        }
    };

    // Create tooltip element
    let tooltipEl = null;

    function createTooltip() {
        if (tooltipEl) return;
        
        tooltipEl = document.createElement('div');
        tooltipEl.className = 'scientific-tooltip';
        tooltipEl.innerHTML = `
            <div class="tooltip-header">
                <span class="tooltip-title"></span>
                <button class="tooltip-close" aria-label="Close">×</button>
            </div>
            <div class="tooltip-layer"></div>
            <div class="tooltip-description"></div>
            <div class="tooltip-science"></div>
            <div class="tooltip-citation"></div>
            <div class="tooltip-parameters"></div>
        `;
        document.body.appendChild(tooltipEl);

        // Close button handler
        tooltipEl.querySelector('.tooltip-close').addEventListener('click', hideTooltip);
        
        // Click outside to close
        document.addEventListener('click', (e) => {
            if (!tooltipEl.contains(e.target) && !e.target.closest('.info-icon')) {
                hideTooltip();
            }
        });
    }

    function showTooltip(key, targetEl) {
        const info = infoDatabase[key];
        if (!info) return;

        createTooltip();
        
        // Update content
        tooltipEl.querySelector('.tooltip-title').textContent = info.title || '';
        tooltipEl.querySelector('.tooltip-layer').textContent = info.layer || '';
        tooltipEl.querySelector('.tooltip-layer').style.display = info.layer ? 'block' : 'none';
        tooltipEl.querySelector('.tooltip-description').textContent = info.description || '';
        tooltipEl.querySelector('.tooltip-science').textContent = info.science ? 'Scientific Basis: ' + info.science : '';
        tooltipEl.querySelector('.tooltip-science').style.display = info.science ? 'block' : 'none';
        tooltipEl.querySelector('.tooltip-citation').textContent = info.citation || '';
        tooltipEl.querySelector('.tooltip-citation').style.display = info.citation ? 'block' : 'none';
        
        // Parameters
        const paramsEl = tooltipEl.querySelector('.tooltip-parameters');
        if (info.parameters && info.parameters.length > 0) {
            paramsEl.innerHTML = '<strong>Parameters:</strong><ul>' + 
                info.parameters.map(p => `<li>${p}</li>`).join('') + 
                '</ul>';
            paramsEl.style.display = 'block';
        } else {
            paramsEl.style.display = 'none';
        }

        // Position tooltip
        const rect = targetEl.getBoundingClientRect();
        const tooltipRect = tooltipEl.getBoundingClientRect();
        
        let left = rect.right + 10;
        let top = rect.top;
        
        // Adjust if off-screen
        if (left + 320 > window.innerWidth) {
            left = rect.left - 330;
        }
        if (top + tooltipRect.height > window.innerHeight) {
            top = window.innerHeight - tooltipRect.height - 10;
        }
        
        tooltipEl.style.left = `${left}px`;
        tooltipEl.style.top = `${top}px`;
        tooltipEl.classList.add('visible');
    }

    function hideTooltip() {
        if (tooltipEl) {
            tooltipEl.classList.remove('visible');
        }
    }

    // Initialize info icons
    function init() {
        console.log('[ScientificInfo] Initializing info tooltips...');
        
        // Add click handlers to all info icons
        document.querySelectorAll('.info-icon').forEach(icon => {
            icon.addEventListener('click', (e) => {
                e.preventDefault();
                e.stopPropagation();
                const key = icon.dataset.infoKey;
                if (key) {
                    showTooltip(key, icon);
                }
            });
        });
        
        console.log('[ScientificInfo] Initialized', document.querySelectorAll('.info-icon').length, 'info icons');
    }

    // Auto-init on DOM ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }

    // Public API
    return {
        init,
        show: showTooltip,
        hide: hideTooltip,
        database: infoDatabase
    };
})();

// Export for global access
window.ScientificInfo = ScientificInfo;
