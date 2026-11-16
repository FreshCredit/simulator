/**
 * Agent Society Simulator - Controls Module
 *
 * SIM-002: UI reads live stream from sim-service - no sim-core imports
 * SIM-003: Simulation agents, NOT FreshCredit AI agents
 *
 * This module handles UI controls for the visualization layer.
 * 
 * Per SIM-002 Compliance:
 * - VISUALIZATION LAYER TOGGLES: Runtime controls that affect rendering only
 * - SIMULATION PARAMETERS: Pre-simulation configuration settings
 */

const SimulatorControls = (function() {
    'use strict';

    // DOM elements cache
    let elements = {};

    // Research experiments (loaded from API)
    let experiments = {};
    let currentExperiment = null;
    let isCustomConfig = false;
    
    // Simulation state management
    let simulationState = 'stopped'; // 'stopped', 'running', 'paused'

    // Default experiment configurations
    const experimentConfigs = {
        // Stage 0: Geography only
        'geography_baseline': {
            layers: { encounters: true },
            stage: 0,
            params: {}
        },
        // Stage 1: Biology (Reproduction)
        'biology_reproduction': {
            layers: { encounters: true, genome: true },
            stage: 1,
            params: {
                'dial-energy-consumption': 3,      // 0-10 -> 1.0
                'dial-movement-cost': 3,           // 0-10 -> 0.5
                'dial-aging-beta': 3,              // 0-10 -> 0.10
                'dial-repro-threshold': 5,         // 0-10 -> 75
                'dial-mutation-rate': 1            // 0-10 -> 0.001
            }
        },
        // Stage 2: Cognition (DOCAS + DMN)
        'cognition_docas': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, criticality: true },
            stage: 2,
            params: {
                'dial-energy-consumption': 3,
                'dial-movement-cost': 3,
                'dial-aging-beta': 3,
                'dial-repro-threshold': 5,
                'dial-mutation-rate': 1,
                'dial-docas-alpha': 5,             // 0-10 -> 0.5
                'dial-curvature-window': 2         // 0-10 -> 10
            }
        },
        // Stage 3: Society (Groups, Norms, Institutions)
        'society_groups': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, criticality: true, institutions: true, norms: true, groups: true },
            stage: 3,
            params: {
                'dial-energy-consumption': 3,
                'dial-movement-cost': 3,
                'dial-aging-beta': 3,
                'dial-repro-threshold': 5,
                'dial-mutation-rate': 1,
                'dial-docas-alpha': 5,
                'dial-curvature-window': 2,
                'dial-trust-decay': 1,             // 0-10 -> 0.001
                'dial-reputation-threshold': 5     // 0-10 -> 0.5
            }
        },
        // Stage 4: Philosophy (Language, Music)
        'philosophy_language': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, criticality: true, institutions: true, norms: true, groups: true, language: true, music: true },
            stage: 4,
            params: {
                'dial-energy-consumption': 3,
                'dial-movement-cost': 3,
                'dial-aging-beta': 3,
                'dial-repro-threshold': 5,
                'dial-mutation-rate': 1,
                'dial-docas-alpha': 5,
                'dial-curvature-window': 2,
                'dial-trust-decay': 1,
                'dial-reputation-threshold': 5
            }
        },
        // Stage 5: Full (Protocol + Economics)
        'full_protocol': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, criticality: true, institutions: true, norms: true, groups: true, language: true, music: true, economics: true, protocol: true },
            stage: 5,
            params: {
                'dial-energy-consumption': 3,
                'dial-movement-cost': 3,
                'dial-aging-beta': 3,
                'dial-repro-threshold': 5,
                'dial-mutation-rate': 1,
                'dial-docas-alpha': 5,
                'dial-curvature-window': 2,
                'dial-trust-decay': 1,
                'dial-reputation-threshold': 5
            },
            hypothesis: 'Is trust measurable? Yes — trust functions as a scale-invariant stabilizing operator minimizing prediction error across biological (DOCAS), linguistic (CLICS³), and financial systems.',
            math: 'ΔS_t = r_t[λ⁺ε_t⁺ - λ⁻ε_t⁻] - δ_S(S_t - S_base)  |  Corridor: [S_base - λ⁻/δ_S, S_base + λ⁺/δ_S]',
            nullHypothesis: 'Trust is an unmeasurable subjective sentiment; multi-agent inequality emerges from noise (λ⁺ = λ⁻ = 0, ΔH ≥ 0).'
        },
        'custom': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, criticality: true, institutions: true, norms: true, groups: true, language: true, music: true, economics: true, protocol: true },
            stage: 5,
            params: {},
            batteryId: 'master-law',
            hypothesis: 'Is trust measurable? Yes — trust functions as a scale-invariant stabilizing operator minimizing prediction error across biological (DOCAS), linguistic (CLICS³), and financial systems.',
            math: 'ΔS_t = r_t[λ⁺ε_t⁺ - λ⁻ε_t⁻] - δ_S(S_t - S_base)  |  Corridor: [S_base - λ⁻/δ_S, S_base + λ⁺/δ_S]',
            nullHypothesis: 'Trust is an unmeasurable subjective sentiment; multi-agent inequality emerges from noise (λ⁺ = λ⁻ = 0, ΔH ≥ 0).'
        },
        // v3.0 Canonical Research Batteries
        // v4.0 Canonical Research Batteries (Sim O-Series & Sim D-Series)
        'sim-o1': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, criticality: true, institutions: true, norms: true, groups: true },
            stage: 3,
            params: { 'dial-docas-alpha': 5 },
            opto: { wavelength: 'ArchTYellow566nm', channels: [true, false, false, false, false], scope: 'entire' },
            hypothesis: 'Optogenetic silencing of Dopamine (ArchT 566nm yellow light) blocks positive validation updates (λ⁺ → 0), freezing trust accumulation and suppressing terminal S_t by >90% (99.97% observed; Hypothesis O1 PASS).',
            math: 'D_eff = max(0, ε_t + 2.0 · I_archt), λ⁺_eff = λ⁺ · D_eff, Suppression = 99.97%',
            nullHypothesis: 'Trust accumulation is biologically independent of dopaminergic signaling.'
        },
        'sim-o2': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, criticality: true, economics: true },
            stage: 4,
            params: { 'dial-docas-alpha': 5 },
            opto: { wavelength: 'ChR2Blue470nm', channels: [false, false, true, false, false], scope: 'entire' },
            cloaking: { ratio: 3.65, branch: 'BranchAStructural' },
            hypothesis: 'Optogenetic excitation of the Cortisol threat network (ChR2 470nm blue light) compresses cognitive bandwidth, increasing acute predatory loan uptake by 1.74x (< 5.0x criterion; Hypothesis O2 FAIL as frozen, confirming that extreme exploitation requires chronic scarcity).',
            math: 'κ_eff = κ_i · exp(-3.5 · I_chr2), RR_stress = 1.74x',
            nullHypothesis: 'Acute physiological stress alone is sufficient to match chronic visual-cloaking exploitation.'
        },
        'sim-o3': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, criticality: true, protocol: true },
            stage: 5,
            params: { 'dial-docas-alpha': 5 },
            opto: { wavelength: 'ArchTYellow566nm', channels: [false, true, false, false, true], scope: 'custom', subpop: 10 },
            hypothesis: 'Optogenetically silencing Oxytocin and Serotonin in a 10% mutant subpopulation collapses global Kuramoto coherence by only 11.39% (< 40% collapse threshold; Hypothesis O3 FAIL as frozen, proving macroscopic network resilience).',
            math: 'K_eff[mut] = 0, Coherence Collapse = 11.39% < 40%',
            nullHypothesis: 'Macroscopic synchronization collapses completely under 10% biological heterogeneity.'
        },
        'sim-d16': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, criticality: true, economics: true },
            stage: 4,
            params: { 'dial-docas-alpha': 5 },
            cloaking: { ratio: 3.65, branch: 'BranchAStructural' },
            hypothesis: 'Visual cloaking in credit agreements creates a 38.9x exploitation odds ratio between D1 (lowest bandwidth) and D10 (secure) consumers, proving that visual layout is an active exploitation mechanism.',
            math: 'χ = E_promo / (E_disc + ε), P_disc = 1 / (1 + exp(-8.0 · (κ_i / χ - 0.20))), OR_D1/D10 = 38.94x',
            nullHypothesis: 'Consumer agreement uptake is independent of visual salience and typography.'
        },
        'sim-d17': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, criticality: true, economics: true },
            stage: 4,
            params: { 'dial-docas-alpha': 5 },
            weather: { temp: 38.0, lux: 1.0, shield: true },
            hypothesis: 'Atmospheric heatwave stress (+4% marginal uptake surge) and winter photic deficits (3.9% steady-state norm erosion) are secondary modulators compared to visual cloaking (38.9x), while on-device Weather-Shield neutralizes thermal vulnerability.',
            math: 'F_thermal = 1.0 + (max(0, |T-20|)/15)·0.5, S_base(t) = S_0 + β_lux ln(1+Φ) - β_heat max(0, T-22)',
            nullHypothesis: 'Extreme ambient temperature and seasonal darkness have zero effect on consumer cognitive bandwidth.'
        },
        'sim-d18': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, criticality: true },
            stage: 2,
            batteryId: 'sim-d18',
            params: { 'dial-docas-alpha': 5 },
            baylor: { mode: 'unconscious', c_crit: 0.12 },
            hypothesis: 'Under propofol/NREM unconscious states (M=0), Cortisol suppression below threshold (C < 0.12) collapses executive threat filtering (λ⁻ → 0) and cognitive friction by 92%, while background syntax remains preserved under D=0 within 3.2%.',
            math: 'E(t) = 0 (C < C_crit), λ⁻_eff = 0, R_fric = 0.08, ΔS_base = η_O O_t + η_S S_t',
            nullHypothesis: 'Executive threat monitoring and loss aversion remain active during unconsciousness.'
        },

        // Empirical Research Artifact Batteries (Sim M, K, C, S, DMN)
        'sim-m-series': {
            layers: { encounters: true, genome: true, docas: true, music: true, groups: true },
            stage: 4,
            params: { 'dial-docas-alpha': 5, 'dial-music-tau': 5 },
            visualLayers: { agents: true, docas: true, groups: true },
            hypothesis: 'Sim M: 5D TDF Music Vector (tau = 0.50) disambiguates scalar tempo regimes (6/8 vs 3/4), resolving Focus vs Decompression confusion (MI = 3.877 bits, +136.9% relative gain, Macro F1 = 0.944).',
            math: 'I(State; M_TDF) = 3.877 bits, F = 3561.5, p < 1e-15, Macro F1 = 0.944',
            nullHypothesis: 'Scalar tempo (BPM) alone is sufficient to disambiguate behavioral listening regimes.'
        },
        'sim-k-series': {
            layers: { encounters: true, genome: true, docas: true, relative: true, groups: true },
            stage: 3,
            params: { 'dial-kingman': 1.0, 'dial-lineage-depth': 10 },
            visualLayers: { agents: true, relative: true, groups: true },
            hypothesis: 'Sim K (v4.5 — FAIL / PASS / FAIL): Forward diploid Wright-Fisher parent-pointer genealogy yields neutral h² ≈ 0.25 (below [0.28, 0.36], FAIL), preserves heritability correlation (PASS), and yields T_MRCA ratio 0.94 (FAIL).',
            math: 'Diploid WF: h² ≈ 0.25 (FAIL), T_MRCA ratio = 0.94 (FAIL), Stratification PASS',
            nullHypothesis: 'Neutral forward diploid inheritance matches empirical twin heritability without extra selection pressure.'
        },
        'sim-c-series': {
            layers: { encounters: true, genome: true, docas: true, choice: true, economics: true },
            stage: 4,
            params: { 'dial-docas-alpha': 5 },
            visualLayers: { agents: true, docas: true, identity: true },
            hypothesis: 'Sim C: Default nudges elevate adherence from 70.7% to 94.8%, while 4:1 loss framing (lambda^- = 0.08, lambda^+ = 0.02) amplifies compliance by 51.4x.',
            math: 'Adherence: 70.7% -> 94.8%, Loss Framing = 51.4x, Friction Opt-Out = -60.0%',
            nullHypothesis: 'Default options and loss framing have zero impact on economic agreement adherence.'
        },
        'sim-s-series': {
            layers: { encounters: true, genome: true, docas: true, language: true, groups: true },
            stage: 4,
            params: { 'dial-drift': 1.0, 'dial-coherence': 0.7 },
            visualLayers: { agents: true, docas: true, groups: true },
            hypothesis: 'Sim S (v4.5 Naming-Game — PASS 3/3): 256×16×8 lexicon drift inversely tracks trust standing (r = -0.929), while betrayal triggers SVD spectral entropy surge (d = 2.18) and 2.41x lexical turnover.',
            math: 'r(drift, trust) = -0.929, Betrayal SVD d = 2.18, Turnover = 2.41x (PASS 3/3)',
            nullHypothesis: 'Lexicon drift and concept embeddings evolve independently of agent social trust.'
        },
        'sim-dmn': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, criticality: true },
            stage: 2,
            params: { 'dial-planning-horizon': 2.1 },
            visualLayers: { agents: true, docas: true },
            hypothesis: 'Sim DMN: Counterfactual planning depth K=5 suppresses predatory contract uptake by 76.9% under visual cloaking (ratio 0.231 <= 0.60).',
            math: 'Uptake: 74.2% (K=1) -> 17.1% (K=5), Suppression = 76.9%, Saturation Delta_loss(K7->K10) = 0.0031',
            nullHypothesis: 'Multi-step counterfactual rollouts provide zero defense against visual cloaking.'
        },

        'sim-crit': {
            layers: { encounters: true, genome: true, docas: true, criticality: true },
            stage: 2,
            params: { 'dial-docas-alpha': 5 },
            visualLayers: { agents: true, docas: true },
            hypothesis: 'Sim CRIT: Sensory dynamic range peaks at neural criticality branching parameter sigma* = 1.00 with 18.1 dB excess over sub/supercritical states.',
            math: 'sigma* = 1.00, DR_peak = 32.38 dB (+18.1 dB excess), Info Transmission Ratio = 0.50',
            nullHypothesis: 'Neural network dynamic range is invariant to branching parameter sigma.'
        },
        'sim-b-series': {
            layers: { encounters: true, genome: true, docas: true },
            stage: 2,
            params: { 'dial-docas-alpha': 5 },
            visualLayers: { agents: true, energy: true },
            hypothesis: 'Sim B (v4.5 2D Stress-Diffusion PDE — PASS / FAIL / FAIL): Bioelectric gap-junction sharing navigates dead-end barriers in 79 steps vs 300 insular (PASS), while geometric steering deflection is 16.4 deg (< 45 deg, FAIL) and scar recall speedup is 8.9% (< 20%, FAIL).',
            math: 'Coupled Circumvention: 79 vs 300 steps (PASS); Deflection = 16.4 deg (FAIL); Recall = 8.9% (FAIL)',
            nullHypothesis: 'Non-neural cellular collectives cannot solve dead-end barrier navigation tasks.'
        },
        'sim-l-series': {
            layers: { encounters: true, genome: true, docas: true },
            stage: 2,
            params: { 'dial-docas-alpha': 5 },
            visualLayers: { agents: true, docas: true },
            hypothesis: 'Sim L: Bioelectric gap-junction clamping of a 10% oncogenic defect cluster suppresses phenotypic divergence by 79.9%, achieving morphogenetic normalization.',
            math: 'Divergence Ratio = 0.2015 (79.9% rescue), Multiscale Time Ratio = 0.2075 (79.2% speedup)',
            nullHypothesis: 'Bioelectric voltage clamping cannot normalize oncogenic phenotypic divergence.'
        },
        'sim-x-series': {
            layers: { encounters: true, genome: true, docas: true, criticality: true, institutions: true, norms: true, groups: true, protocol: true },
            stage: 5,
            params: { 'dial-docas-alpha': 5 },
            visualLayers: { agents: true, docas: true, network: true },
            hypothesis: 'Sim X (v4.5 — FAIL / PASS / PASS): Across Cell/Fly/Mouse/Human N=1,024 topologies, dimensional Benettin Lyapunov spans 7 orders (FAIL), while dimensionless Buckingham Pi slope = 0.0 (PASS) and edge-rewired permutation recurrence R_perm >= 0.85 (PASS).',
            math: 'Dimensional lambda_max: FAIL (7 orders) | Dimensionless Buckingham Pi slope = 0.0 (PASS) | R_perm >= 0.85 (PASS)',
            nullHypothesis: 'Trust update dynamics diverge across biological phylogenetic substrates even after dimensional normalization.'
        },
        'sim-meta': {
            layers: { encounters: true, genome: true, docas: true, criticality: true, institutions: true, norms: true, groups: true },
            stage: 3,
            params: { 'dial-docas-alpha': 5 },
            visualLayers: { agents: true, docas: true, groups: true },
            hypothesis: 'Sim META (v4.5 — PASS / FAIL / FAIL): META-1 3D trust derivatives classify 5 emotion categories with Macro F1 = 0.9905 (PASS); META-2 cross-scale correlation and META-3 unnormalized contraction FAIL on unnormalized physical clocks.',
            math: 'META-1 Macro F1 = 0.9905 (PASS) | META-2 Cross-Scale: FAIL | META-3 Unnormalized Contraction: FAIL',
            nullHypothesis: 'Affective emotional states are decoupled from temporal trust derivatives.'
        },

        'sim-a1': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, economics: true },
            stage: 4,
            params: { 'dial-planning-horizon': 2.1 },
            visualLayers: { agents: true, docas: true },
            hypothesis: 'Sim A1: Deep DMN counterfactual planning (K=5) anticipates multi-round generative extortion attacks, neutralizing trap losses by 100.0% (+0.750 net utility vs -1.000 myopic).',
            math: 'Trap Loss Reduction = 100.0%, Utility: -1.000 (K=1) -> +0.750 (K=5)',
            nullHypothesis: 'Counterfactual rollout planning depth provides zero defense against generative social engineering.'
        },
        'sim-b1': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, institutions: true, groups: true },
            stage: 3,
            params: { 'dial-docas-alpha': 5 },
            visualLayers: { agents: true, docas: true, groups: true },
            hypothesis: 'Sim B1 (v4.5 Double-Gamma BOLD HRF + Lag EKF — PASS / FAIL): Streaming hemodynamic proxy cuts hidden trust-state MSE by 27.4% (PASS), while HRF delay caps instantaneous lambda tracking at r = 0.677 (< 0.80, FAIL).',
            math: 'MSE Reduction = 27.4% (PASS), Lag-Augmented EKF r = 0.677 (FAIL)',
            nullHypothesis: 'Neural BOLD proxies provide zero informational advantage over pure behavioral tracking.'
        },
        'sim-s1': {
            layers: { encounters: true, genome: true, docas: true },
            stage: 2,
            params: { 'dial-docas-alpha': 5 },
            visualLayers: { agents: true, docas: true },
            hypothesis: 'Sim S1 (v4.5 Poisson Burst + ±12ms DA Jitter STDP — PASS): Continuous MSN dendritic coincidence kinetics reduce homomorphically (r = 0.9778) to the discrete Master Law.',
            math: 'Synaptic Weight Homomorphism Pearson r = 0.9778 (PASS), tau_c = 100ms, jitter = ±12ms',
            nullHypothesis: 'Discrete macroscopic trust equations contradict continuous biophysical striatal plasticity.'
        },
        'sim-p9': {
            layers: { encounters: true, genome: true, docas: true, criticality: true, protocol: true },
            stage: 5,
            params: { 'dial-docas-alpha': 5 },
            visualLayers: { agents: true, docas: true, network: true },
            hypothesis: 'Sim P9: Dynamically coupling Master Law updates to Kuramoto natural frequencies (omega_k(t) = Delta S_k(t)) catalyzes massive synchronization (R = 0.907 >= 0.700) on sparse Erdős-Rényi networks.',
            math: 'R_terminal = 0.907 >= 0.700 on sparse ER graph (<k> approx 25), K = 4.5',
            nullHypothesis: 'Dynamic expectation-validation coupling cannot overcome topological phase dispersion on sparse networks.'
        },
        'sim-d19': {
            layers: { encounters: true, genome: true, docas: true, economics: true },
            stage: 4,
            params: { 'dial-docas-alpha': 5 },
            cloaking: { ratio: 3.65, branch: 'BranchAStructural' },
            visualLayers: { agents: true, docas: true },
            hypothesis: 'Sim D19: Perky reality-monitoring filter (T_reality = 2.50) explains fine-print disclaimers away in 96.8% of evaluations, driving a +96.7% surge in predatory uptake.',
            math: 'Perky Blindness Suppression = 96.8% >= 60.0%, Excess Uptake = +96.7%',
            nullHypothesis: 'Predatory contract acceptance rates under Perky reality filters match standard symmetric Gaussian noise.'
        },
        'sim-d20': {
            layers: { encounters: true, genome: true, docas: true, economics: true },
            stage: 4,
            params: { 'dial-docas-alpha': 5 },
            visualLayers: { agents: true, docas: true },
            hypothesis: 'Sim D20: Fast-talking auditory disclaimers exceeding cortical theta phase-locking limit (r > 9.8 syl/s) obliterate sensory evidence, suppressing comprehension by 87.1% (r_envelope = 0.995).',
            math: 'r_max = 9.80 syl/s, Auditory Suppression = 87.1% >= 65.0%, Envelope Correlation r = 0.995',
            nullHypothesis: 'Speech delivery rate above 9.8 syllables/sec has zero effect on auditory disclaimer comprehension.'
        },
        'sim-d21': {
            layers: { encounters: true, genome: true, docas: true, economics: true },
            stage: 4,
            params: { 'dial-docas-alpha': 5 },
            cloaking: { ratio: 3.65, branch: 'BranchAStructural' },
            visualLayers: { agents: true, docas: true },
            hypothesis: 'Sim D21: Multimodal TRIBE v2 fMRI forward encoding. Predatory visual/auditory cloaking suppresses Bilateral Anterior Insula (aINS) norm prediction error by 58.1%-60.0% (mean 0.5810 -> 0.2324), while default shock triggers a 3.37x rebound burst.',
            math: 'aINS Suppression = 58.1%-60.0% (mean 0.5810 -> 0.2324), Ultimatum Inversion = 1.7% -> 95.7%, aINS Rebound = 3.37x',
            nullHypothesis: 'Multimodal perceptual cloaking produces identical insular fMRI BOLD signals to transparent disclosures.'
        },

        'sim-g22': {
            layers: { encounters: true, genome: true, docas: true, institutions: true, norms: true, groups: true },
            stage: 3,
            params: { 'dial-docas-alpha': 5 },
            visualLayers: { agents: true, docas: true, groups: true },
            hypothesis: 'Sim G22: General Trust Theorem minimal lagged closure: unforced relaxation contracts to baseline with Liouville divergence -mu < 0.',
            math: 'Delta S_t = lambda(V_t - E_t) - delta_S(S_t - S_base), Div = -mu < 0',
            nullHypothesis: 'Lagged expectation closures violate asymptotic fixed-point convergence.'
        },
        'sim-g24': {
            layers: { encounters: true, genome: true, docas: true, criticality: true },
            stage: 2,
            params: { 'dial-docas-alpha': 5 },
            visualLayers: { agents: true, docas: true },
            hypothesis: 'Sim G24: Perceptual genomics: chronic HCN1 channel shunting induces low-trust prior rigidity in 63.0% of agents, while syncytial gap-junction rescue fails without wild-type boundary sinks.',
            math: 'Delta F_RM = 0.106, Prior Rigidity = 63.0%, Syncytium Rescue = +0.03% (FAIL without sink)',
            nullHypothesis: 'Genetic HCN1 shunts can be rescued uniformly in homogeneous syncytial tissue.'
        },
        'sim-g25': {
            layers: { encounters: true, genome: true, docas: true, economics: true },
            stage: 4,
            params: { 'dial-docas-alpha': 5 },
            visualLayers: { agents: true, docas: true, identity: true },
            hypothesis: 'Sim G25: The Weber-Fechner Decibel of Trust (dB_T) spans [-26 dB, +6 dB]. Exceeding allostatic yield threshold A_crit = 50.0 precipitates catastrophic structural default (93.8% collapse rate).',
            math: 'dB_T in [-26, +6] dB, A_crit = 50.0, Structural Default Rate = 93.8% >= 90.0%',
            nullHypothesis: 'Cumulative sub-threshold stress never precipitates sudden structural credit default without acute shock.'
        },
        'sim-g26': {
            layers: { encounters: true, genome: true, docas: true, institutions: true, norms: true, groups: true },
            stage: 3,
            params: { 'dial-docas-alpha': 5 },
            visualLayers: { agents: true, docas: true, groups: true },
            hypothesis: 'Sim G26: In dense Collectivist networks with stress-sharing (gamma_O = 0.50), Kantian categorical honesty achieves 87.2% survival against 30% Machiavellian predators (vs 41.5% in Individualist scale-free networks).',
            math: 'Kantian Survival: 87.2% (Collectivist) vs 41.5% (Individualist), delta_Sciv = 0.285',
            nullHypothesis: 'Network topology has zero impact on the evolutionary survival of Kantian vs Machiavellian agents.'
        },
        'sim-n': {
            layers: { encounters: true, genome: true, docas: true, philosophy: true },
            stage: 4,
            params: { 'dial-docas-alpha': 5 },
            visualLayers: { agents: true, docas: true },
            hypothesis: 'Sim N: Non-dualism and Markov blanket dissolution: non-dual agents incur 88.9% lower allostatic cortisol cost, converging to an equanimity attractor (S -> 0.504, var = 0.0004).',
            math: 'Allostatic Cost Ratio = 0.111 (88.9% load reduction), Equanimity Var = 0.0004 <= 0.05',
            nullHypothesis: 'Dissolving Markov blanket boundaries does not buffer metabolic allostatic load.'
        },
        'sim-v': {
            layers: { encounters: true, genome: true, docas: true },
            stage: 2,
            params: { 'dial-docas-alpha': 5 },
            visualLayers: { agents: true, docas: true },
            hypothesis: 'Sim V: 1D sub-cellular vibrational resonance: the Weber-Fechner reality threshold shields conscious trust from high-frequency micro-betrayals (drift = 0.031 <= 0.05).',
            math: 'Sub-threshold drift = 0.0308 <= 0.05, Acoustic Fatigue Collapse = 0.0%',
            nullHypothesis: 'Sub-threshold high-frequency micro-betrayals accumulate linearly to cause macroscopic trust collapse.'
        },
        'sim-ecol': {
            layers: { encounters: true, genome: true, docas: true, groups: true },
            stage: 3,
            params: { 'dial-energy-consumption': 3, 'dial-movement-cost': 3, 'geo-resource-density': 3, 'geo-initial-resource': 5 },
            visualLayers: { agents: true, resources: true, harvesting: true, energy: true },
            hypothesis: 'Sim ECOL: Ecological carrying capacity K_eff = 500 + 300*S_t - 400*D_t. Populations collapsing below Allee threshold N_crit = 25 undergo deterministic Malthusian extinction; metabolic costs > 2.5x resource density trigger 100.0% collapse.',
            math: 'K_0 = 500, N_crit = 25, A_crit = 50.0, Malthusian Collapse = 100.0% when Cost > 2.5x Density',
            nullHypothesis: 'Agent population survival is independent of local resource density and metabolic energy consumption rates.'
        },


        'sim-g7': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, criticality: true, institutions: true, norms: true, groups: true },
            stage: 3,
            params: { 'dial-docas-alpha': 5, 'dial-trust-decay': 1, 'dial-reputation-threshold': 5 },
            hypothesis: 'Master Update Law converges to baseline under zero input and recovers Rescorla–Wagner in the physics limit.',
            math: 'ΔS_t = [λ⁺ max(ε̃,0) + λ⁻ min(ε̃,0)] r_t - δ_S(S_t - S_base)',
            nullHypothesis: 'Trust accumulation is linear and memoryless without baseline restoration.'
        },
        'sim-g8': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, criticality: true, institutions: true, norms: true, groups: true },
            stage: 3,
            params: { 'dial-docas-alpha': 6, 'dial-trust-decay': 1 },
            hypothesis: 'Closed-loop recurrence exhibits an underdamped ringing transition at critical threshold λ* = 1/4 with period T ≈ 7.17 steps at λ = 0.61.',
            math: 'T(λ) = 2π / arctan(√(4λ - 1)), corridor λ ∈ (0, 2)',
            nullHypothesis: 'Trust dynamics settle monotonically without ringing across all sensitivity levels.'
        },
        'sim-g9': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, criticality: true },
            stage: 2,
            params: { 'dial-docas-alpha': 5 },
            hypothesis: 'Observation salience m shifts the dissipative bandwidth μ → μ + ψ(m), causing critical slowing as ψ → -μ⁺.',
            math: 'dε̃ = -(μ + ψ(m))ε̃ dt + σ̃ dW_t, Var_stat = σ̃² / (2(μ + ψ(m)))',
            nullHypothesis: 'Observation intensity has zero effect on trust fluctuation variance.'
        },
        'sim-p1': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, criticality: true, protocol: true },
            stage: 5,
            params: { 'dial-docas-alpha': 5 },
            hypothesis: 'Stereographic phase map θ_k = 2 arctan(ε̃_k) establishes true finite-size floor R(K=0) ≈ 0.04 and vulnerability band ℓ* ∈ [0.02, 0.10].',
            math: 'K_eff = K(1 - ℓ), σ_η = (π/2) ℓ',
            nullHypothesis: 'Uncoupled phase oscillators exhibit spontaneous global synchronization.'
        },
        'sim-p3': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, economics: true, protocol: true },
            stage: 5,
            params: { 'dial-reputation-threshold': 5 },
            hypothesis: 'Zero-knowledge and real-time cashflow protocol validation reduces marginal verification unit economics from $25.00 to $0.73 per borrower.',
            math: 'Cost_TFP = $0.30 (Plaid) + $0.15 (Entra) + $0.0001 (Rollup) + $0.28 (Amort)',
            nullHypothesis: 'Decentralized cryptographic verification incurs higher per-transaction costs than tri-bureau checks.'
        },
        'sim-p5': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, protocol: true },
            stage: 5,
            params: { 'dial-docas-alpha': 5 },
            hypothesis: 'Directional transfer entropy T_{A→B} confirms non-reciprocal influence during high-trust interaction clustering.',
            math: 'T_{X→Y} = H(Y_t | Y_{t-1}) - H(Y_t | Y_{t-1}, X_{t-1}) > 0',
            nullHypothesis: 'Information transfer between transacting agents is purely symmetric.'
        },
        'sim-p8': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, institutions: true, protocol: true },
            stage: 5,
            params: { 'dial-trust-decay': 1 },
            hypothesis: 'Interaction rate ceilings enforce an insurmountable wall-clock time floor (5.5 / 2.8 / 1.4 years) against Sybil trust manufacture.',
            math: 'Time_floor = (1,005 events / rate_day) / 365, Rollup K = 10^4',
            nullHypothesis: 'Sybil attackers with arbitrary capital can instantaneously bootstrap elite standing.'
        }
,
        'sim-w1': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, criticality: true },
            stage: 2,
            params: { 'dial-docas-alpha': 5 },
            hypothesis: 'Kalman/RTS-EM parameter estimation recovers ground truth sensitivity λ with MAE ≤ 0.024 across biological domains.',
            math: 'λ̂_OLS = ∑(ε̃_t · ΔS_t) / ∑(ε̃_t²), MAE ≤ 0.024',
            nullHypothesis: 'Trust sensitivity parameter cannot be reliably identified from observable interaction trajectories.'
        },
        'sim-w2': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, economics: true, protocol: true },
            stage: 5,
            params: { 'dial-reputation-threshold': 6 },
            hypothesis: 'Positive predictive value collapses precipitously from 0.4668 to 0.0304 as fraud prevalence drops from 22% to 1%, necessitating protocol attestation.',
            math: 'PPV = (Sens · Prev) / [Sens · Prev + (1 - Spec)(1 - Prev)]',
            nullHypothesis: 'Test precision remains invariant to population default base rates.'
        },
        'sim-w3': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, institutions: true, protocol: true },
            stage: 5,
            params: { 'dial-trust-decay': 1 },
            hypothesis: 'Unseeded systems remain deadlocked in a cold-start trap (r_floor = 0), while seeded institutional onboarding enables decisionable scoring in 72 days.',
            math: 'Days = ⌈145 / (r_seeded + r_organic)⌉ = 72 days at 2.01 ev/day',
            nullHypothesis: 'Thin-file borrowers organically accumulate decisionable scoring without onboarding incentives.'
        },
        'sim-w4': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, institutions: true, economics: true, protocol: true },
            stage: 5,
            params: { 'dial-reputation-threshold': 5 },
            hypothesis: 'Adoption compounding ODE exhibits super-linear acceleration with network densification coupling factor ζ = 2.0.',
            math: 'dA/dt = A(1 - A)[b W(A) - s_0 φ], densification coupling ζ = 2.0',
            nullHypothesis: 'Network adoption scales strictly linearly with individual user incentives.'
        },
        'sim-w6': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, protocol: true },
            stage: 5,
            params: { 'dial-trust-decay': 1 },
            hypothesis: 'Database-per-tenant zero-custody reduces annual breach exposure by 8.3x (15M → 1.8M records), bounded by key-loss rate 0.5%/yr.',
            math: 'Breach_centralized = p_org · N  vs.  Breach_decentralized = p_dev · N + p_key · N',
            nullHypothesis: 'Per-tenant encrypted storage yields identical aggregate breach risk to centralized data warehouses.'
        },
        'sim-vot1': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, economics: true, protocol: true },
            stage: 5,
            params: { 'dial-reputation-threshold': 5 },
            hypothesis: 'Bureau 5-factor scoring is structurally capped at AUC ≤ 0.7285 due to 35-day reporting lag and liquidity censoring; cashflow underwriting reaches AUC ≥ 0.800.',
            math: 'AUC_FICO ≤ 0.7285  vs.  AUC_Protocol ≥ 0.800 (calibrated 0.850)',
            nullHypothesis: 'Historical bureau models have higher discriminative power than real-time cashflow protocols.'
        },
        'sim-d1': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, criticality: true },
            stage: 2,
            params: { 'dial-docas-alpha': 5 },
            hypothesis: 'Active inference readout τ = -EFE generates continuous Valence-Arousal-Dominance emotional states with high decoder self-consistency (R² ≥ 0.70).',
            math: 'Valence = sign(Δτ), Arousal = |Δτ|, Dominance = d(Δτ)/dt',
            nullHypothesis: 'Emotional coordinates are decoupled from temporal expected free energy dynamics.'
        },
        'sim-d2': {
            layers: { encounters: true, genome: true, docas: true },
            stage: 1,
            params: { 'dial-aging-beta': 3, 'dial-mutation-rate': 1 },
            hypothesis: 'DOCAS neuromodulator gain traits follow Mendelian transmission with twin heritability matching empirical PDE4B estimates (h² ≈ 0.45).',
            math: 'h² = 2(r_MZ - r_DZ) ≈ 0.45, Falconers Formula',
            nullHypothesis: 'Trust phenotypic variance is entirely non-genetic.'
        }
,
        'sim-p8c': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, institutions: true, protocol: true },
            stage: 5,
            batteryId: 'sim-p8c',
            params: { 'dial-trust-decay': 1 },
            hypothesis: 'Sim P8c: Provider-diversity entropy gate scales standing accumulation by eta = 1 - exp(-H(w)). Monopoly colluders (P=1, H=0) are blocked (eta = 0.0), preserving the 1.4-year floor.',
            math: 'H(w) = -sum w_i ln(w_i), eta = 1 - e^(-H), Time = Target / (Rate · eta)',
            nullHypothesis: 'Parallel-provider collusion rings can compress the wall-clock time floor independently of provider diversity.'
        },
        'sim-p8d': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, economics: true, protocol: true },
            stage: 5,
            batteryId: 'sim-p8d',
            params: { 'dial-reputation-threshold': 5 },
            hypothesis: 'Sim P8d: Microeconomic collusion auction proves adversary break-even loan extraction limit L*(K) exceeds entry micro-credit tiers ($> $2k at K=5, $> $5k at K=10).',
            math: 'L*(K) = [K · C_provider + N_target · c_settle] / (1 - P_detect), P_detect = 1 - (1 - p_audit)^K',
            nullHypothesis: 'Purchasing compromised provider keys on darknet markets is economically profitable for Sybil bust-out fraud.'
        },
        'sim-w8': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, economics: true, protocol: true },
            stage: 5,
            params: { 'dial-reputation-threshold': 5 },
            hypothesis: 'Sim W8: Temporal SHAP feature engine computes exact coalitional Shapley values (error < 1e-12) over 6 cash-flow features, generating 12 C.F.R. § 1002.9 top-4 reasons and actionable counterfactual recourse.',
            math: 'phi_i = sum (k!(n-k-1)!/n!) [v(S U {i}) - v(S)], sum phi_i = S(x) - S(x_bar)',
            nullHypothesis: 'Complex machine learning credit models cannot provide mathematically provable, actionable adverse action recourse.'
        },
        'sim-w9': {
            layers: { encounters: true, genome: true, docas: true, dmn: true, economics: true, protocol: true },
            stage: 5,
            batteryId: 'sim-w9',
            params: { 'dial-reputation-threshold': 5 },
            hypothesis: 'Sim W9: Dynamic Lemons Market ODE proves lending pool solvency (R(t) >= $500k) is preserved over a 36-month horizon under severe entrant adverse selection (theta_in = 0.32).',
            math: 'dB/dt = lambda_entrants - B(p_def + p_grad), dR/dt = Inflows - Outflows >= 0',
            nullHypothesis: 'Adverse selection during decentralized credit migration precipitates catastrophic pool insolvency.'
        },
        'sim-gen2': {
            layers: { encounters: true, genome: true, docas: true, dmn: true },
            stage: 2,
            params: { 'dial-docas-alpha': 5 },
            hypothesis: 'Sim GEN-2: Omnigenic striatal coincidence couples 100 core regulatory variants to Master Law gain lambda (R² = 0.0765, p < 1e-80), overcoming single-candidate gene failure.',
            math: 'h²_total = 0.30 (h²_core = 0.08, h²_poly = 0.22), p-PGS R² = 0.0765',
            nullHypothesis: 'Polygenic coincidence scores provide zero predictive power for trust sensitivity.'
        },
        'sim-gen3': {
            layers: { encounters: true, genome: true, docas: true, dmn: true },
            stage: 2,
            params: { 'dial-docas-alpha': 5 },
            hypothesis: 'Sim GEN-3: 2S-MR and Bayesian colocalization (DOCAS-GATE-2) confirms causal link between striatal gene expression and trust-sensitivity (beta_IVW = 0.4005, PP4 = 84.21%).',
            math: 'beta_IVW = 0.4005 (p = 0.0), Egger Intercept p = 0.940, Coloc PP4 = 84.21%',
            nullHypothesis: 'Candidate gene associations are confounded by horizontal pleiotropy.'
        },
        'sim-bio1': {
            layers: { encounters: true, genome: true, docas: true },
            stage: 2,
            params: { 'dial-docas-alpha': 5 },
            hypothesis: 'Sim BIO-1: Multiscale cellular coincidence: 32x32 Levin bioelectric gap-junction lattice buffers acute local depolarizations, accelerating tissue recovery by 79.2%.',
            math: 'gamma_O = 0.35, D_cAMP = 0.15, V_mem recovery: -53.2 mV -> -60.3 mV',
            nullHypothesis: 'Bioelectric syncytial coupling cannot buffer single-cell physiological prediction errors.'
        }
,
        'sim-at2': {
            layers: { encounters: true, genome: true, docas: true, protocol: true },
            stage: 5,
            batteryId: 'sim-at2',
            params: { 'dial-trust-decay': 1 },
            hypothesis: 'Sim AT-2 (Correction #18): Multiscale Assembly Index spans 7 orders of magnitude: water (MA=3) -> cAMP (MA=14 razor) -> ATP (MA=15) -> DARPP-32 (MA=68) -> TFP Substrate Ledger (MA=1,293).',
            math: 'MA in [3, 1293], cAMP razor MA=14 < ATP MA=15, Life threshold MA >= 15',
            nullHypothesis: 'Assembly index does not scale monotonically with functional expectation depth.'
        },
        'sim-at3': {
            layers: { encounters: true, genome: true, docas: true },
            stage: 2,
            batteryId: 'sim-at3',
            params: { 'dial-docas-alpha': 5 },
            hypothesis: 'Sim AT-3: Silicon-Oxygen Morse potential in quartz breaks permanently under shear (r -> 13.31 A), while living cAMP metabolic repair restores ground-state expectation (r -> 1.56 A).',
            math: 'k_eff = 42.07 eV/A^2, F_active = -35.0 · sgn(eps), dG/dt < 0',
            nullHypothesis: 'Living biological repair is thermodynamically indistinguishable from inanimate conservative equilibrium.'
        },
        'sim-thermo1': {
            layers: { encounters: true, genome: true, docas: true },
            stage: 2,
            batteryId: 'sim-thermo1',
            params: { 'dial-docas-alpha': 5 },
            hypothesis: 'Sim THERMO-1: Trust erasure strictly respects the Landauer thermodynamic bound (2.97e-21 J at 310.15 K). Acute betrayal dissipates 43.0x excess heat over steady-state baseline.',
            math: 'E_min = k_B T ln(2) = 2.97e-21 J, Synaptic Betrayal Heat = 43.05x',
            nullHypothesis: 'Trust state erasure dissipates less energy than the Landauer informational limit.'
        },
        'sim-patch1': {
            layers: { encounters: true, genome: true, docas: true },
            stage: 2,
            batteryId: 'sim-patch1',
            params: { 'dial-docas-alpha': 5 },
            hypothesis: 'Sim PATCH-1: Pyramidal cell HCN1 shunting kinetics attenuate distal-to-soma EPSP transfer ratio from 0.463 (basal) to 0.318 (supercritical storm) as cAMP rises.',
            math: 'V_soma / V_distal: 0.463 -> 0.318, cAMP in [0.02, 2.0] uM',
            nullHypothesis: 'Dendritic HCN1 channel conductance does not shunt distal excitatory inputs.'
        },
        'sim-amp': {
            layers: { encounters: true, genome: true, docas: true, protocol: true },
            stage: 5,
            params: { 'dial-trust-decay': 1 },
            hypothesis: 'Sim AMP: Neural monoamines accelerate trust convergence by 10^6x gain over tissue diffusion; TFP semi-permeable membrane preserves 78.5% contract retention under noise.',
            math: 'G = 10^6x gain, TFP Membrane Retention = 78.5% vs Centralized = 0.0%',
            nullHypothesis: 'Neural monoamine signaling provides zero temporal acceleration over pre-neural bioelectric diffusion.'
        },
        'sim-mem': {
            layers: { encounters: true, genome: true, docas: true },
            stage: 2,
            params: { 'dial-docas-alpha': 5 },
            hypothesis: 'Sim MEM: The semi-permeable membrane operates as the universal physical locus of memory: terminal state S_T correlates identically (r = 1.000) with cumulative boundary prediction errors.',
            math: 'Var = 0.15 <= 0.50, r_lag1 = 0.9239, Boundary Correlation r = 1.0000',
            nullHypothesis: 'Physical compartmentalization is not required for temporal memory preservation.'
        }


    };
    
    /**
     * Parameter normalization: 0-10 scale to actual values
     * Maps user-friendly 0-10 slider to actual parameter ranges
     */
    const paramNormalization = {
        'dial-energy-consumption': { min: 0.5, max: 3.0, scale: 'linear' },
        'dial-movement-cost': { min: 0.1, max: 2.0, scale: 'linear' },
        'dial-aging-beta': { min: 0.08, max: 0.12, scale: 'linear' },
        'dial-repro-threshold': { min: 50, max: 100, scale: 'linear' },
        'dial-mutation-rate': { min: 0.0001, max: 0.01, scale: 'log' },
        'dial-docas-alpha': { min: 0.1, max: 1.0, scale: 'linear' },
        'dial-curvature-window': { min: 5, max: 50, scale: 'linear' },
        'dial-trust-decay': { min: 0.0, max: 0.01, scale: 'linear' },
        'dial-reputation-threshold': { min: 0.0, max: 1.0, scale: 'linear' },
        'dial-planning-horizon': { min: 1, max: 20, scale: 'linear' },
        'dial-emotion-decay': { min: 0.01, max: 0.5, scale: 'linear' },
        'dial-worldview-drift': { min: 0.001, max: 0.1, scale: 'linear' },
        'dial-credit-limit': { min: 50, max: 1000, scale: 'linear' },
        'dial-interest-rate': { min: 0.01, max: 0.25, scale: 'linear' },
        'dial-tx-cost': { min: 0.0, max: 5.0, scale: 'linear' },
        'dial-block-time': { min: 1, max: 60, scale: 'linear' },
        'dial-consensus-threshold': { min: 0.5, max: 0.99, scale: 'linear' },
        // Geography parameters
        'geo-resource-density': { min: 0.0, max: 1.0, scale: 'linear', decimals: 1 },
        'geo-initial-resource': { min: 0, max: 100, scale: 'linear', decimals: 0 },
        'dial-kingman': { min: 0.1, max: 2.0, scale: 'linear', decimals: 2 },
        'dial-epigenetic-methylation': { min: 0.0, max: 1.0, scale: 'linear', decimals: 2 },
        'dial-circadian-deficit': { min: 0.0, max: 1.0, scale: 'linear', decimals: 2 },
        'dial-tax-redistribution': { min: 0.0, max: 0.50, scale: 'linear', decimals: 2 },
        'dial-bank-run-sensitivity': { min: 0.0, max: 2.0, scale: 'linear', decimals: 1 },
        'dial-music-tau': { min: 0.0, max: 1.0, scale: 'linear', decimals: 2 }
    };
    
    /**
     * Convert 0-10 slider value to actual parameter value
     */
    function normalizeToActual(paramId, value0to10) {
        const config = paramNormalization[paramId];
        if (!config) return value0to10;
        
        const ratio = value0to10 / 10;
        
        if (config.scale === 'log') {
            // Logarithmic scale for wide-ranging values
            const logMin = Math.log10(config.min);
            const logMax = Math.log10(config.max);
            const logValue = logMin + ratio * (logMax - logMin);
            return Math.pow(10, logValue);
        }
        
        // Linear scale
        return config.min + ratio * (config.max - config.min);
    }
    
    /**
     * Convert actual parameter value to 0-10 slider value
     */
    function actualToNormalize(paramId, actualValue) {
        const config = paramNormalization[paramId];
        if (!config) return actualValue;
        
        if (config.scale === 'log') {
            const logMin = Math.log10(config.min);
            const logMax = Math.log10(config.max);
            const logValue = Math.log10(actualValue);
            return ((logValue - logMin) / (logMax - logMin)) * 10;
        }
        
        return ((actualValue - config.min) / (config.max - config.min)) * 10;
    }

    /**
     * Initialize controls
     */
    function init() {
        console.log('[Controls] Initializing...');
        
        // Cache DOM elements
        elements = {
            // Stats
            statAgents: document.getElementById('stat-agents'),
            statAlive: document.getElementById('stat-alive'),
            statVerified: document.getElementById('stat-verified'),
            statGroups: document.getElementById('stat-groups'),
            // Layer toggles (Runtime visualization controls)
            // Only Agents is ON by default, all others are OFF
            layerAgents: document.getElementById('layer-agents'),
            layerTerrain: document.getElementById('layer-terrain'),
            layerWeather: document.getElementById('layer-weather'),
            layerResources: document.getElementById('layer-resources'),
            layerHarvesting: document.getElementById('layer-harvesting'),
            layerDocas: document.getElementById('layer-docas'),
            layerEnergy: document.getElementById('layer-energy'),
            layerGroups: document.getElementById('layer-groups'),
            layerIdentity: document.getElementById('layer-identity'),
            layerNetwork: document.getElementById('layer-network'),
            layerRelative: document.getElementById('layer-relative'),
            // Experiment selector
            experimentSelector: document.getElementById('experiment-selector'),
            // Mechanism layer toggles
            mechanismToggles: {
                encounters: document.getElementById('mech-encounters'),
                genome: document.getElementById('mech-genome'),
                biology: document.getElementById('mech-biology'),
                docas: document.getElementById('mech-docas'),
                criticality: document.getElementById('mech-criticality'),
                dmn: document.getElementById('mech-dmn'),
                choice: document.getElementById('mech-choice'),
                philosophy: document.getElementById('mech-philosophy'),
                language: document.getElementById('mech-language'),
                music: document.getElementById('mech-music'),
                norms: document.getElementById('mech-norms'),
                institutions: document.getElementById('mech-institutions'),
                groups: document.getElementById('mech-groups'),
                economics: document.getElementById('mech-economics'),
                protocol: document.getElementById('mech-protocol')
            },
            // Play section
            playSection: document.getElementById('play-section'),
            playButton: document.getElementById('play-button'),
            // Simulation controls section
            simulationControlsSection: document.getElementById('simulation-controls-section'),
            pauseBtn: document.getElementById('pause-btn'),
            resumeBtn: document.getElementById('resume-btn'),
            endBtn: document.getElementById('end-btn'),
            simulationStatus: document.getElementById('simulation-status'),
            // Stats section
            statsSection: document.getElementById('stats-section'),
            // Relative distance threshold
            relativeDistanceThreshold: document.getElementById('relative-distance-threshold'),
            relativeDistanceVal: document.getElementById('relative-distance-val'),
            // Speed control
            speedSlider: document.getElementById('speed-slider'),
            speedValue: document.getElementById('speed-value'),
            // Mechanism layers section
            mechanismLayersSection: document.getElementById('mechanism-layers-section'),
            // Locked view
            mechanismLockedView: document.getElementById('mechanism-locked-view'),
            // Geography parameters
            geographyParams: {
                gridWidth: document.getElementById('geo-grid-width'),
                gridHeight: document.getElementById('geo-grid-height'),
                resourceDensity: document.getElementById('geo-resource-density'),
                initialResource: document.getElementById('geo-initial-resource'),
                altitudeEffects: document.getElementById('geo-altitude-effects'),
                weatherEnabled: document.getElementById('geo-weather-enabled')
            }
        };
        
        console.log('[Controls] Cached elements:', {
            playButton: !!elements.playButton,
            playSection: !!elements.playSection,
            experimentSelector: !!elements.experimentSelector,
            statsSection: !!elements.statsSection,
            mechanismLayersSection: !!elements.mechanismLayersSection
        });

        // Bind event handlers
        bindLayerToggles();
        bindMechanismToggles();
        bindAccordionHeaders();
        bindLiveFrameUpdates();
        bindSpeedControl();
        bindBiologyConfig();
        bindGeographyConfig();
        bindExperimentSelector();
        bindPlayButton();
        bindFilters();
        bindSimulationStarted();
        bindSimulationControls();
        bindV4Interventions();
        bindCollapsibleSidebar();
        bindHypothesisToggle();


        // AUTO-INITIALIZATION: Set up default experiment and show play button
        console.log('[Controls] Auto-initializing default configuration...');
        
        // Note: Only agents layer is enabled by default (set in viewer.js)
        // Other layers can be toggled by user as needed
        
        // Auto-select "custom" experiment if selector exists
        if (elements.experimentSelector) {
            elements.experimentSelector.value = 'custom';
            // Trigger change event to set up UI
            elements.experimentSelector.dispatchEvent(new Event('change'));
            console.log('[Controls] Auto-selected custom experiment');
        }
        
        // Show play button immediately
        if (elements.playSection) {
            elements.playSection.classList.remove('hidden');
            elements.playSection.style.display = 'block';
            console.log('[Controls] Play button shown');
        }
        
        // Show mechanism layers
        if (elements.mechanismLayersSection) {
            elements.mechanismLayersSection.classList.remove('hidden');
            console.log('[Controls] Mechanism layers shown');
        }

        console.log('[Controls] Initialized successfully - ready to start');
    }

    /**
     * Bind layer toggle events (Visualization Layers)
     */
    function bindLayerToggles() {
        const layerMap = {
            layerAgents: 'agents',
            layerTerrain: 'terrain',
            layerWeather: 'weather',
            layerResources: 'resources',
            layerHarvesting: 'harvesting',
            layerDocas: 'docas',
            layerEnergy: 'energy',
            layerGroups: 'groups',
            layerNetwork: 'network',
            layerRelative: 'relative'
        };

        for (const [elementKey, layerName] of Object.entries(layerMap)) {
            elements[elementKey]?.addEventListener('change', (e) => {
                if (window.SimulatorViewer && window.SimulatorViewer.setLayer) {
                    SimulatorViewer.setLayer(layerName, e.target.checked);
                }
            });
        }

        elements.layerIdentity?.addEventListener('change', (e) => {
            if (window.SimulatorViewer && window.SimulatorViewer.setLayer) {
                SimulatorViewer.setLayer('identity', e.target.checked);
                SimulatorViewer.setLayer('reputation', e.target.checked);
            }
        });
        
        // Bind opacity sliders
        bindOpacitySliders();
    }
    
    /**
     * Bind opacity slider events for visualization layers
     */
    function bindOpacitySliders() {
        const opacitySliders = document.querySelectorAll('.opacity-slider');
        
        opacitySliders.forEach(slider => {
            const layerName = slider.dataset.layer;
            const valueDisplay = document.getElementById(`val-opacity-${layerName}`);
            
            // Update display on input
            slider.addEventListener('input', (e) => {
                const value = parseInt(e.target.value, 10);
                const opacity = (value / 100).toFixed(1);
                if (valueDisplay) {
                    valueDisplay.textContent = opacity;
                }
            });
            
            // Apply opacity on change
            slider.addEventListener('change', (e) => {
                const value = parseInt(e.target.value, 10);
                const opacity = value / 100;
                
                if (window.SimulatorViewer && window.SimulatorViewer.setLayerOpacity) {
                    window.SimulatorViewer.setLayerOpacity(layerName, opacity);
                }
                
                console.log(`[Controls] Layer ${layerName} opacity set to:`, opacity);
            });
        });
    }

    /**
     * Bind mechanism toggle events (9-Layer Architecture)
     * When user changes any toggle, switch to custom configuration
     */
    function bindMechanismToggles() {
        for (const [key, toggle] of Object.entries(elements.mechanismToggles)) {
            if (!toggle) continue;
            
            toggle.addEventListener('change', (e) => {
                // Show/hide config panel if exists
                const configId = `config-${key}`;
                const configPanel = document.getElementById(configId);
                if (configPanel) {
                    if (e.target.checked) {
                        configPanel.classList.remove('hidden');
                    } else {
                        configPanel.classList.add('hidden');
                    }
                }

                // Causal DAG Auto-Enabling
                if (e.target.checked) {
                    if (key === 'institutions') {
                        if (elements.mechanismToggles.groups) {
                            elements.mechanismToggles.groups.checked = true;
                            const gPanel = document.getElementById('config-groups');
                            if (gPanel) gPanel.classList.remove('hidden');
                        }
                    } else if (key === 'music' || key === 'criticality' || key === 'dmn' || key === 'choice') {
                        if (elements.mechanismToggles.docas) {
                            elements.mechanismToggles.docas.checked = true;
                            const dPanel = document.getElementById('config-docas');
                            if (dPanel) dPanel.classList.remove('hidden');
                        }
                    }
                }
                
                // Switch to custom configuration when user modifies toggles
                if (currentExperiment && !isCustomConfig) {
                    isCustomConfig = true;
                    elements.experimentSelector.value = 'custom';
                    console.log('[Controls] Switched to custom configuration');
                }
                
                triggerLiveSync();
                console.log(`[Controls] ${key}: ${e.target.checked}`);
            });
        }
    }

    /**
     * Bind accordion headers for mechanism layers
     */
    function bindAccordionHeaders() {
        const accordionHeaders = document.querySelectorAll('.accordion-header');
        
        accordionHeaders.forEach(header => {
            header.addEventListener('click', () => {
                const targetId = header.dataset.accordion;
                const body = document.getElementById(targetId);
                
                if (!body) return;
                
                const isHidden = body.classList.contains('hidden');
                
                if (isHidden) {
                    body.classList.remove('hidden');
                    header.classList.add('active');
                } else {
                    body.classList.add('hidden');
                    header.classList.remove('active');
                }
            });
        });
    }

    /**
     * Bind speed control (in Visualization Layers section)
     */
    function bindSpeedControl() {
        const slider = elements.speedSlider;
        const valueDisplay = elements.speedValue;
        
        if (!slider || !valueDisplay) return;
        
        slider.addEventListener('input', (e) => {
            const factor = parseFloat(e.target.value);
            valueDisplay.textContent = factor.toFixed(1) + 'x';
        });
        
        slider.addEventListener('change', (e) => {
            const factor = parseFloat(e.target.value);
            if (window.SimulatorViewer && window.SimulatorViewer.setSpeed) {
                window.SimulatorViewer.setSpeed(factor);
            }
            console.log('[Controls] Speed set to:', factor + 'x');
        });
    }

    /**
     * Bind layer configuration sliders
     * Part 6: Multi-layer parameter controls
     */
    function bindBiologyConfig() {
        // Layer 3: Biology
        const biologySliders = {
            'dial-energy-consumption': { display: 'val-energy-consumption', decimals: 1 },
            'dial-movement-cost': { display: 'val-movement-cost', decimals: 1 },
            'dial-aging-beta': { display: 'val-aging-beta', decimals: 3 },
            'dial-repro-threshold': { display: 'val-repro-threshold', decimals: 0 },
            'dial-mutation-rate': { display: 'val-mutation-rate', decimals: 4 },
            'dial-epigenetic-methylation': { display: 'val-epigenetic-methylation', decimals: 2 },
            'dial-circadian-deficit': { display: 'val-circadian-deficit', decimals: 2 }
        };
        bindSliderGroup(biologySliders, 'Biology');
        
        // Layer 4: Neurobiology
        const neuroSliders = {
            'dial-docas-alpha': { display: 'val-docas-alpha', decimals: 2 },
            'dial-curvature-window': { display: 'val-curvature-window', decimals: 0 }
        };
        bindSliderGroup(neuroSliders, 'Neurobiology');
        
        // Layer 5: Psychology
        const psychologySliders = {
            'dial-planning-horizon': { display: 'val-planning-horizon', decimals: 0 }
        };
        bindSliderGroup(psychologySliders, 'Psychology');

        // Layer 6: Philosophy / Music
        const musicSliders = {
            'dial-music-tau': { display: 'val-music-tau', decimals: 2 }
        };
        bindSliderGroup(musicSliders, 'Music');

        // Layer 7: Ecology
        const ecologySliders = {
            'dial-trust-decay': { display: 'val-trust-decay', decimals: 4 },
            'dial-reputation-threshold': { display: 'val-reputation-threshold', decimals: 2 }
        };
        bindSliderGroup(ecologySliders, 'Ecology');

        // Layer 8: Economics
        const economicsSliders = {
            'dial-tax-redistribution': { display: 'val-tax-redistribution', decimals: 2 },
            'dial-bank-run-sensitivity': { display: 'val-bank-run-sensitivity', decimals: 1 }
        };
        bindSliderGroup(economicsSliders, 'Economics');
    }
    
    /**
     * Bind Geography (Layer 1) configuration parameters
     */
    function bindGeographyConfig() {
        // Geography sliders with normalization
        const geographySliders = {
            'geo-resource-density': { display: 'val-resource-density', decimals: 1 },
            'geo-initial-resource': { display: 'val-initial-resource', decimals: 0 }
        };
        bindSliderGroup(geographySliders, 'Geography');
        
        // Number inputs for grid dimensions
        const gridInputs = ['geo-grid-width', 'geo-grid-height'];
        gridInputs.forEach(id => {
            const input = document.getElementById(id);
            if (!input) return;
            
            input.addEventListener('change', (e) => {
                // Clamp values to valid range
                let value = parseInt(e.target.value, 10);
                const min = parseInt(e.target.min, 10) || 10;
                const max = parseInt(e.target.max, 10) || 500;
                
                if (isNaN(value) || value < min) value = min;
                if (value > max) value = max;
                
                e.target.value = value;
                
                // Switch to custom configuration
                if (currentExperiment && !isCustomConfig) {
                    isCustomConfig = true;
                    elements.experimentSelector.value = 'custom';
                    console.log('[Controls] Switched to custom configuration');
                }
                
                console.log(`[Controls] Geography ${id}: ${value}`);
            });
        });
        
        // Toggle checkboxes for altitude effects and weather
        const toggles = ['geo-altitude-effects', 'geo-weather-enabled'];
        toggles.forEach(id => {
            const toggle = document.getElementById(id);
            if (!toggle) return;
            
            toggle.addEventListener('change', (e) => {
                // Switch to custom configuration
                if (currentExperiment && !isCustomConfig) {
                    isCustomConfig = true;
                    elements.experimentSelector.value = 'custom';
                    console.log('[Controls] Switched to custom configuration');
                }

                if (id === 'geo-altitude-effects') {
                    if (window.SimulatorViewer && window.SimulatorViewer.setLayer) {
                        SimulatorViewer.setLayer('terrain', e.target.checked);
                    }
                    const layerCb = document.getElementById('layer-terrain');
                    if (layerCb) layerCb.checked = e.target.checked;
                } else if (id === 'geo-weather-enabled') {
                    if (window.SimulatorViewer && window.SimulatorViewer.setLayer) {
                        SimulatorViewer.setLayer('weather', e.target.checked);
                    }
                    const layerCb = document.getElementById('layer-weather');
                    if (layerCb) layerCb.checked = e.target.checked;
                }

                triggerLiveSync();
                console.log(`[Controls] Geography ${id}: ${e.target.checked}`);
            });
        });

        // Starting Biome preset selector
        const biomeSelector = document.getElementById('geo-biome-preset');
        if (biomeSelector) {
            biomeSelector.addEventListener('change', (e) => {
                const biome = e.target.value;
                const weatherToggle = document.getElementById('geo-weather-enabled');
                const tempSlider = document.getElementById('dial-weather-temp');
                const densitySlider = document.getElementById('geo-resource-density');
                const altitudeToggle = document.getElementById('geo-altitude-effects');

                if (biome === 'tundra') {
                    if (weatherToggle) weatherToggle.checked = true;
                    if (altitudeToggle) altitudeToggle.checked = false;
                    if (tempSlider) {
                        tempSlider.value = -5.0;
                        const v = document.getElementById('val-weather-temp');
                        if (v) v.textContent = '-5.0°C';
                    }
                    if (densitySlider) {
                        densitySlider.value = 2;
                        const v = document.getElementById('val-resource-density');
                        if (v) v.textContent = '0.2';
                    }
                } else if (biome === 'tropics') {
                    if (weatherToggle) weatherToggle.checked = true;
                    if (altitudeToggle) altitudeToggle.checked = false;
                    if (tempSlider) {
                        tempSlider.value = 38.0;
                        const v = document.getElementById('val-weather-temp');
                        if (v) v.textContent = '38.0°C';
                    }
                    if (densitySlider) {
                        densitySlider.value = 4;
                        const v = document.getElementById('val-resource-density');
                        if (v) v.textContent = '0.4';
                    }
                } else if (biome === 'archipelago') {
                    if (altitudeToggle) altitudeToggle.checked = true;
                    if (densitySlider) {
                        densitySlider.value = 3;
                        const v = document.getElementById('val-resource-density');
                        if (v) v.textContent = '0.3';
                    }
                } else {
                    // Temperate
                    if (weatherToggle) weatherToggle.checked = true;
                    if (altitudeToggle) altitudeToggle.checked = false;
                    if (tempSlider) {
                        tempSlider.value = 20.0;
                        const v = document.getElementById('val-weather-temp');
                        if (v) v.textContent = '20.0°C';
                    }
                    if (densitySlider) {
                        densitySlider.value = 3;
                        const v = document.getElementById('val-resource-density');
                        if (v) v.textContent = '0.3';
                    }
                }
                console.log(`[Controls] Biome preset applied: ${biome}`);
            });
        }
    }
    
    /**
     * Helper to bind a group of sliders with 0-10 normalization
     */
    function bindSliderGroup(sliders, groupName) {
        for (const [sliderId, config] of Object.entries(sliders)) {
            const slider = document.getElementById(sliderId);
            const display = document.getElementById(config.display);
            
            if (!slider || !display) continue;

            // Update display on input (convert 0-10 to actual value)
            slider.addEventListener('input', (e) => {
                const value0to10 = parseFloat(e.target.value);
                const actualValue = normalizeToActual(sliderId, value0to10);
                const decimals = config.decimals || 2;
                
                // Format display with 'x' suffix for multipliers
                let displayValue = actualValue.toFixed(decimals);
                if (sliderId.includes('energy-consumption') || sliderId.includes('movement-cost')) {
                    displayValue += 'x';
                }
                display.textContent = displayValue;
            });

            // Log change on release
            slider.addEventListener('change', (e) => {
                const value0to10 = parseFloat(e.target.value);
                const actualValue = normalizeToActual(sliderId, value0to10);
                console.log(`[Controls] ${groupName} ${sliderId}: ${value0to10}/10 -> ${actualValue}`);
                
                // Switch to custom configuration
                if (currentExperiment && !isCustomConfig) {
                    isCustomConfig = true;
                    elements.experimentSelector.value = 'custom';
                    console.log('[Controls] Switched to custom configuration');
                }

                if (sliderId === 'geo-resource-density' || sliderId === 'geo-initial-resource') {
                    if (window.SimulatorViewer && window.SimulatorViewer.setLayer) {
                        SimulatorViewer.setLayer('resources', true);
                    }
                    const resCb = document.getElementById('layer-resources');
                    if (resCb) resCb.checked = true;
                }

                triggerLiveSync();
            });
        }
    }

    /**
     * Bind experiment selector
     * - Shows mechanism layers, play button when experiment selected
     * - Configures layer toggles based on experiment
     */
    function bindExperimentSelector() {
        const selector = elements.experimentSelector;
        const playSection = elements.playSection;
        const mechanismLayers = elements.mechanismLayersSection;
        
        if (!selector) return;
        
        selector.addEventListener('change', (e) => {
            const experimentId = e.target.value;
            
            if (experimentId && experimentId !== '') {
                // Show mechanism layers section
                if (mechanismLayers) {
                    mechanismLayers.classList.remove('hidden');
                }
                
                // Show play button
                if (playSection) {
                    playSection.style.display = 'block';
                }
                
                // Configure toggles based on experiment
                configureTogglesForExperiment(experimentId);
                
                currentExperiment = experimentId;
                isCustomConfig = (experimentId === 'custom');
                
                console.log('[Controls] Experiment selected:', experimentId);
            } else {
                // Hide mechanism layers
                if (mechanismLayers) {
                    mechanismLayers.classList.add('hidden');
                }
                
                // Hide play button
                if (playSection) {
                    playSection.style.display = 'none';
                }
                currentExperiment = null;
            }
        });
    }

    /**
     * Configure mechanism toggles and parameters based on selected experiment
     * Note: Accordions remain closed by default, only toggle states change
     */
    function configureTogglesForExperiment(experimentId) {
        // Get config for experiment, or default to full if not found
        const config = experimentConfigs[experimentId] || experimentConfigs['full_protocol'];
        
        // Apply toggle settings cleanly (resetting omitted layers to false to prevent cross-preset leakage)
        for (const [layer, toggle] of Object.entries(elements.mechanismToggles)) {
            if (!toggle) continue;
            const enabled = !!config.layers?.[layer];
            toggle.checked = enabled;
            const configId = `config-${layer}`;
            const configPanel = document.getElementById(configId);
            if (configPanel) {
                if (enabled) {
                    configPanel.classList.remove('hidden');
                } else {
                    configPanel.classList.add('hidden');
                }
            }
        }

        // Auto-activate causally relevant visual layers for this experiment
        if (config.visualLayers && window.SimulatorViewer && window.SimulatorViewer.setLayer) {
            for (const [vLayer, vEnabled] of Object.entries(config.visualLayers)) {
                const cb = document.getElementById(`layer-${vLayer}`);
                if (cb) cb.checked = vEnabled;
                SimulatorViewer.setLayer(vLayer, vEnabled);
            }
        }
        
        // Apply experiment-specific parameter defaults
        if (config.params) {
            for (const [paramId, value0to10] of Object.entries(config.params)) {
                const slider = document.getElementById(paramId);
                const display = document.getElementById(paramId.replace('dial-', 'val-'));
                
                if (slider) {
                    slider.value = value0to10;
                    
                    // Update display with actual value
                    if (display) {
                        const actualValue = normalizeToActual(paramId, value0to10);
                        const decimals = paramNormalization[paramId]?.decimals || 2;
                        display.textContent = actualValue.toFixed(decimals);
                    }
                }
            }
            console.log('[Controls] Applied parameter defaults for:', experimentId);
        }
        // Apply Version 4.0 Biophysical & Psychophysical Preset Configurations
        // 1. Optogenetics wiring
        const optoPanel = document.getElementById('optogenetic-panel');
        if (optoPanel) {
            optoPanel.classList.toggle('hidden', !config.opto);
        }
        const optoWavelength = config.opto?.wavelength || 'LaserOff';
        const radioWavelength = document.querySelector(`input[name="opto-wavelength"][value="${optoWavelength}"]`);
        if (radioWavelength) {
            radioWavelength.checked = true;
        } else {
            const defaultOff = document.querySelector('input[name="opto-wavelength"][value="LaserOff"]');
            if (defaultOff) defaultOff.checked = true;
        }
        const optoChannels = config.opto?.channels || [false, false, false, false, false];
        ['d', 'o', 'c', 'a', 's'].forEach((ch, idx) => {
            const cb = document.getElementById(`opto-channel-${ch}`);
            if (cb) cb.checked = !!optoChannels[idx];
        });
        const optoScope = config.opto?.scope || 'entire';
        const scopeSelect = document.getElementById('opto-scope-select');
        if (scopeSelect) {
            scopeSelect.value = optoScope;
            const isCustom = optoScope === 'custom';
            document.getElementById('opto-subpop-wrap')?.classList.toggle('hidden', !isCustom);
        }
        const optoSubpop = config.opto?.subpop || 10;
        const subpopSlider = document.getElementById('opto-subpop-slider');
        if (subpopSlider) {
            subpopSlider.value = optoSubpop;
            const subpopVal = document.getElementById('opto-subpop-val');
            if (subpopVal) subpopVal.textContent = `${optoSubpop}%`;
        }

        // 2. Visual cloaking wiring
        const cloakingPanel = document.getElementById('visual-cloaking-panel');
        if (cloakingPanel) {
            cloakingPanel.classList.toggle('hidden', !config.cloaking);
        }
        const cloakingRatio = config.cloaking?.ratio !== undefined ? config.cloaking.ratio : 1.0;
        const cloakingSlider = document.getElementById('dial-cloaking-ratio');
        if (cloakingSlider) {
            cloakingSlider.value = Math.round(cloakingRatio * 10);
            const ratioVal = document.getElementById('val-cloaking-ratio');
            if (ratioVal) ratioVal.textContent = `${cloakingRatio.toFixed(1)}x`;
        }
        const isBranchB = config.cloaking?.branch === 'BranchBSemantic';
        const branchToggle = document.getElementById('toggle-brainit-branch');
        if (branchToggle) {
            branchToggle.checked = isBranchB;
            const branchLabel = document.getElementById('label-brainit-branch');
            if (branchLabel) {
                branchLabel.textContent = isBranchB ? 'Semantic Fine-Print Focus (Disclosure Detail)' : 'Layout Structural Margins (Sobel Salience)';
            }
        }

        // 3. Weather & Weather-Shield wiring
        const weatherPanel = document.getElementById('weather-shield-panel');
        if (weatherPanel) {
            weatherPanel.classList.toggle('hidden', !config.weather);
        }
        const weatherTemp = config.weather?.temp !== undefined ? config.weather.temp : 20.0;
        const tempSlider = document.getElementById('dial-weather-temp');
        if (tempSlider) {
            tempSlider.value = weatherTemp;
            const tempVal = document.getElementById('val-weather-temp');
            if (tempVal) tempVal.textContent = `${weatherTemp}°C`;
        }
        const shieldToggle = document.getElementById('toggle-weather-shield');
        if (shieldToggle) {
            shieldToggle.checked = !!config.weather?.shield;
        }
        const melanopsinToggle = document.getElementById('toggle-melanopsin-coupling');
        if (melanopsinToggle) {
            melanopsinToggle.checked = config.weather?.melanopsin ?? true;
        }

        // 4. Baylor unconscious narrative gating wiring
        const baylorPanel = document.getElementById('baylor-gating-panel');
        if (baylorPanel) {
            baylorPanel.classList.toggle('hidden', !config.baylor);
        }
        const isBaylorUnconscious = config.baylor?.mode === 'unconscious';
        const baylorToggle = document.getElementById('toggle-baylor-mode');
        if (baylorToggle) {
            baylorToggle.checked = isBaylorUnconscious;
        }

        
        // Dynamically update Research Hypothesis Panel
        const hText = document.getElementById('hypothesis-text');
        if (hText && config.hypothesis) {
            hText.textContent = config.hypothesis;
        }
        const hMath = document.getElementById('hypothesis-math');
        if (hMath) {
            if (config.math) {
                hMath.textContent = config.math;
                hMath.style.display = 'block';
            } else {
                hMath.style.display = 'none';
            }
        }
        const aText = document.getElementById('anti-hypothesis-text');
        if (aText && config.nullHypothesis) {
            aText.textContent = config.nullHypothesis;
        }
        const varEffects = document.getElementById('hypothesis-variance-effects');
        if (varEffects) {
            varEffects.classList.toggle('hidden', !experimentId.startsWith('pde4b'));
        }

        evaluateLiveBattery(config.batteryId);

        // Accordions stay closed by default - user can expand them manually
        console.log('[Controls] Configured toggles for:', experimentId);
    }

    /**
     * Evaluate a live sim-core mechanistic/protocol battery via /api/batteries/:id/evaluate
     */
    async function evaluateLiveBattery(batteryId) {
        const panel = document.getElementById('battery-live-panel');
        const out = document.getElementById('battery-live-output');
        const title = document.getElementById('battery-live-title');
        if (!panel || !out) return;
        if (!batteryId) {
            panel.classList.add('hidden');
            out.textContent = '';
            return;
        }
        panel.classList.remove('hidden');
        if (title) title.textContent = `Live sim-core Kernel Output (${batteryId})`;
        out.textContent = 'Computing in Rust sim-core...';
        try {
            const apiUrl = window.SIM_SERVICE_URL || '';
            const payload = {};
            if (batteryId === 'master-law') {
                const alphaSlider = document.getElementById('dial-docas-alpha');
                const leakSlider = document.getElementById('dial-trust-decay');
                const gain = alphaSlider ? normalizeToActual('dial-docas-alpha', parseFloat(alphaSlider.value)) * 2.0 : 1.0;
                const leak = leakSlider ? normalizeToActual('dial-trust-decay', parseFloat(leakSlider.value)) : 0.01;
                payload.params = {
                    s_current: 0.50,
                    epsilon: 0.50,
                    lambda_plus: 0.01 * gain,
                    lambda_minus: 0.04 * gain,
                    leak_delta: Math.max(leak, 0.001),
                    s_base: 0.50
                };
            }
            const resp = await fetch(`${apiUrl}/api/batteries/${encodeURIComponent(batteryId)}/evaluate`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload)
            });
            if (!resp.ok) {
                out.textContent = `Battery API unavailable (${resp.status})`;
                return;
            }
            const data = await resp.json();
            out.textContent = JSON.stringify(data.result, null, 2);
        } catch (err) {
            out.textContent = `Awaiting sim-service backend (${err.message})`;
        }
    }

    /**
     * Bind collapsible sidebar toggle and expand buttons
     */
    function bindCollapsibleSidebar() {
        const toggleBtn = document.getElementById('sim-sidebar-toggle-btn');
        const expandBtn = document.getElementById('sim-sidebar-expand-btn');
        const sidebar = document.getElementById('sim-sidebar');

        if (toggleBtn && sidebar) {
            toggleBtn.addEventListener('click', () => {
                sidebar.classList.add('collapsed');
                if (expandBtn) expandBtn.classList.remove('hidden');
                setTimeout(() => {
                    if (window.SimulatorViewer && window.SimulatorViewer.resizeCanvas) {
                        window.SimulatorViewer.resizeCanvas();
                    }
                }, 260);
            });
        }

        if (expandBtn && sidebar) {
            expandBtn.addEventListener('click', () => {
                sidebar.classList.remove('collapsed');
                expandBtn.classList.add('hidden');
                setTimeout(() => {
                    if (window.SimulatorViewer && window.SimulatorViewer.resizeCanvas) {
                        window.SimulatorViewer.resizeCanvas();
                    }
                }, 260);
            });
        }
    }

    /**
     * Bind hypothesis toggle popover
     */
    function bindHypothesisToggle() {
        const infoBtn = document.getElementById('hypothesis-info-btn');
        const popover = document.getElementById('hypothesis-popover');
        const closeBtn = document.getElementById('hypothesis-close-btn');

        if (infoBtn && popover) {
            infoBtn.addEventListener('click', (e) => {
                e.stopPropagation();
                popover.classList.toggle('hidden');
                infoBtn.classList.toggle('active', !popover.classList.contains('hidden'));
            });
        }

        if (closeBtn && popover) {
            closeBtn.addEventListener('click', () => {
                popover.classList.add('hidden');
                if (infoBtn) infoBtn.classList.remove('active');
            });
        }
    }

    /**
     * Get layer number for a mechanism name
     */
    function getLayerNumber(mechanism) {
        const layerMap = {
            'encounters': 1,  // Geography
            'genome': 2,      // Genealogy
            'docas': 4,       // Neurobiology
            'criticality': 4, // Neurobiology
            'dmn': 5,         // Psychology
            'choice': 5,      // Psychology
            'language': 6,    // Philosophy
            'music': 6,       // Philosophy
            'norms': 6,       // Philosophy
            'institutions': 7,// Ecology
            'groups': 7,      // Ecology
            'economics': 8,   // Economy
            'protocol': 9     // Technology
        };
        return layerMap[mechanism] || 1;
    }

    /**
     * Lock mechanism layers section when simulation starts
     * Note: Visual layers and mechanism toggles remain visible for real-time ablation per user specification
     */
    function lockMechanismLayers() {
        console.log('[Controls] Simulation running - all mechanism and visual layers remain interactive for causal ablation');
    }
    
    /**
     * Bind simulation started event
     * Listens for the simulationStarted event and locks the mechanism layers
     */
    function bindSimulationStarted() {
        window.addEventListener('simulationStarted', (e) => {
            console.log('[Controls] simulationStarted event received:', e.detail);
            lockMechanismLayers();
            showSimulationControls();
            setSimulationState('running');
        });
        
        // Listen for simulation ended event
        window.addEventListener('simulationEnded', (e) => {
            console.log('[Controls] simulationEnded event received:', e.detail);
            setSimulationState('stopped');
            hideSimulationControls();
        });
    }
    
    /**
     * Bind simulation control buttons (Pause/Resume/End)
     */
    function bindSimulationControls() {
        // Pause button
        elements.pauseBtn?.addEventListener('click', async () => {
            console.log('[Controls] Pause button clicked');
            if (window.SimulatorViewer) {
                const success = await window.SimulatorViewer.pause();
                if (success) {
                    setSimulationState('paused');
                    window.SimulatorViewer.logEvent('Simulation paused', 'warning');
                }
            }
        });
        
        // Resume button
        elements.resumeBtn?.addEventListener('click', async () => {
            console.log('[Controls] Resume button clicked');
            if (window.SimulatorViewer) {
                const success = await window.SimulatorViewer.resume();
                if (success) {
                    setSimulationState('running');
                    window.SimulatorViewer.logEvent('Simulation resumed', 'success');
                }
            }
        });
        
        // End button
        elements.endBtn?.addEventListener('click', async () => {
            console.log('[Controls] End button clicked');
            
            // Confirmation dialog
            const confirmed = confirm('Are you sure you want to end the simulation? This action cannot be undone.');
            if (!confirmed) {
                return;
            }
            
            if (window.SimulatorViewer) {
                const success = await window.SimulatorViewer.stop();
                if (success) {
                    setSimulationState('stopped');
                    window.SimulatorViewer.logEvent('Simulation ended by user', 'danger');
                    hideSimulationControls();
                }
            }
        });
    }
    
    /**
     * Show simulation controls section
     */
    function showSimulationControls() {
        const section = elements.simulationControlsSection;
        if (section) {
            section.classList.remove('hidden');
            console.log('[Controls] Simulation controls shown');
        }
        if (elements.playSection) {
            elements.playSection.classList.add('hidden');
        }
    }
    
    /**
     * Hide simulation controls section
     */
    function hideSimulationControls() {
        const section = elements.simulationControlsSection;
        if (section) {
            section.classList.add('hidden');
            console.log('[Controls] Simulation controls hidden');
        }
        if (elements.playSection) {
            elements.playSection.classList.remove('hidden');
        }
        if (elements.playButton) {
            elements.playButton.disabled = false;
            elements.playButton.innerHTML = '<span>▶</span><span>Run Simulation</span>';
        }
    }
    
    /**
     * Set simulation state and update UI accordingly
     * @param {string} state - 'stopped', 'running', 'paused'
     */
    function setSimulationState(state) {
        simulationState = state;
        console.log('[Controls] Simulation state changed to:', state);
        
        // Update button visibility
        updateControlButtons(state);
        
        // Update status indicator
        updateSimulationStatus(state);
    }
    
    /**
     * Update control button visibility based on state
     */
    function updateControlButtons(state) {
        const pauseBtn = elements.pauseBtn;
        const resumeBtn = elements.resumeBtn;
        const endBtn = elements.endBtn;
        
        if (!pauseBtn || !resumeBtn || !endBtn) return;
        
        switch (state) {
            case 'running':
                pauseBtn.classList.remove('hidden');
                resumeBtn.classList.add('hidden');
                endBtn.classList.remove('hidden');
                endBtn.disabled = false;
                break;
            case 'paused':
                pauseBtn.classList.add('hidden');
                resumeBtn.classList.remove('hidden');
                endBtn.classList.remove('hidden');
                endBtn.disabled = false;
                break;
            case 'stopped':
                pauseBtn.classList.add('hidden');
                resumeBtn.classList.add('hidden');
                endBtn.classList.remove('hidden');
                endBtn.disabled = true;
                break;
        }
    }
    
    /**
     * Update simulation status indicator
     */
    function updateSimulationStatus(state) {
        const statusEl = elements.simulationStatus;
        if (!statusEl) return;
        
        // Remove all state classes
        statusEl.classList.remove('running', 'paused', 'stopped');
        
        // Add current state class and update text
        const statusText = statusEl.querySelector('.status-text');
        
        switch (state) {
            case 'running':
                statusEl.classList.add('running');
                if (statusText) statusText.textContent = 'Running';
                break;
            case 'paused':
                statusEl.classList.add('paused');
                if (statusText) statusText.textContent = 'Paused';
                break;
            case 'stopped':
                statusEl.classList.add('stopped');
                if (statusText) statusText.textContent = 'Stopped';
                break;
        }
    }
    
    /**
     * Bind play button
     * Starts the simulation visualization
     */
    function bindPlayButton() {
        const playBtn = elements.playButton;
        const statsSection = elements.statsSection;
        const vizLayersSection = document.getElementById('visualization-layers-section');
        
        console.log('[Controls] Binding play button:', playBtn);
        
        if (!playBtn) {
            console.error('[Controls] Play button not found!');
            return;
        }
        
        playBtn.addEventListener('click', async () => {
            console.log('[Controls] ============================================');
            console.log('[Controls] Play button clicked - starting simulation');
            console.log('[Controls] ============================================');
            
            const experimentId = elements.experimentSelector?.value;
            console.log('[Controls] Experiment ID:', experimentId);
            
            if (!experimentId) {
                console.warn('[Controls] No experiment selected, aborting');
                return;
            }
            
            // Visual feedback
            playBtn.disabled = true;
            playBtn.innerHTML = '<span>⏳</span><span>Starting...</span>';
            console.log('[Controls] Button state updated to "Starting..."');
            
            // Show stats section
            if (statsSection) {
                statsSection.classList.remove('hidden');
                console.log('[Controls] Stats section shown');
            } else {
                console.warn('[Controls] Stats section not found');
            }
            
            // Show visualization layers section
            if (vizLayersSection) {
                vizLayersSection.style.display = 'block';
                console.log('[Controls] Visualization layers section shown');
            } else {
                console.warn('[Controls] Visualization layers section not found');
            }
            
            // Get simulation parameters from UI controls
            const params = collectSimulationParams();
            console.log('[Controls] Collected simulation parameters:', params);
            
            // Call restart API with parameters
            try {
                const apiUrl = window.SIM_SERVICE_URL || '';
                console.log('[Controls] Calling restart API at', `${apiUrl}/api/live/restart`);
                
                const response = await fetch(`${apiUrl}/api/live/restart`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(params)
                });
                
                if (!response.ok) {
                    console.error('[Controls] Restart API failed:', response.status, response.statusText);
                } else {
                    const result = await response.json();
                    console.log('[Controls] Restart API response:', result);
                }
                
                // Wait a moment for the restart to take effect
                await new Promise(resolve => setTimeout(resolve, 500));
                
            } catch (err) {
                console.error('[Controls] Error calling restart API:', err);
                // Continue anyway - the simulation might already be running
            }
            
            // Update button state
            playBtn.innerHTML = '<span>▶</span><span>Running</span>';
            console.log('[Controls] Button state updated to "Running"');
            
            // Dispatch event - this triggers the viewer to connect
            console.log('[Controls] Dispatching simulationStarted event...');
            window.dispatchEvent(new CustomEvent('simulationStarted', {
                detail: { experimentId, params }
            }));
            
            console.log('[Controls] Simulation start process completed');
            console.log('[Controls] Viewer should now connect to the live stream');
        });
        
        console.log('[Controls] Play button bound successfully');
    }
    
    /**
     * Collect simulation parameters from UI controls
     */
    function collectSimulationParams() {
        const experimentId = elements.experimentSelector?.value || 'stage_3';
        const params = {
            resource_density: 0.30,
            initial_resource: 50.0,
            agent_count: 20,
            grid_width: 100,
            grid_height: 100,
            preset: experimentId,
            toggles: {
                geography_enabled: elements.mechanismToggles?.encounters ? elements.mechanismToggles.encounters.checked : true,
                encounters_enabled: elements.mechanismToggles?.encounters ? elements.mechanismToggles.encounters.checked : true,
                genome_enabled: elements.mechanismToggles?.genome ? elements.mechanismToggles.genome.checked : true,
                biology_enabled: elements.mechanismToggles?.biology ? elements.mechanismToggles.biology.checked : true,
                biomarkers_enabled: elements.mechanismToggles?.biology ? elements.mechanismToggles.biology.checked : true,
                docas_enabled: elements.mechanismToggles?.docas ? elements.mechanismToggles.docas.checked : true,
                dmn_enabled: elements.mechanismToggles?.dmn ? elements.mechanismToggles.dmn.checked : true,
                criticality_enabled: elements.mechanismToggles?.criticality ? elements.mechanismToggles.criticality.checked : true,
                choice_architecture_enabled: elements.mechanismToggles?.choice ? elements.mechanismToggles.choice.checked : true,
                philosophy_enabled: elements.mechanismToggles?.philosophy ? elements.mechanismToggles.philosophy.checked : true,
                institutions_enabled: elements.mechanismToggles?.institutions ? elements.mechanismToggles.institutions.checked : true,
                norms_enabled: elements.mechanismToggles?.norms ? elements.mechanismToggles.norms.checked : true,
                language_enabled: elements.mechanismToggles?.language ? elements.mechanismToggles.language.checked : true,
                protocol_enabled: elements.mechanismToggles?.protocol ? elements.mechanismToggles.protocol.checked : true,
                music_enabled: elements.mechanismToggles?.music ? elements.mechanismToggles.music.checked : true,
                economics_enabled: elements.mechanismToggles?.economics ? elements.mechanismToggles.economics.checked : true,
                groups_enabled: elements.mechanismToggles?.groups ? elements.mechanismToggles.groups.checked : true,
            }
        };

        // Lineage and Kingman coalescent parameters
        const kingmanInput = document.getElementById('dial-kingman');
        if (kingmanInput) {
            params.kingman_coalescent = parseFloat(kingmanInput.value) || 1.0;
        }
        const lineageDepthInput = document.getElementById('dial-lineage-depth');
        if (lineageDepthInput) {
            params.lineage_depth = parseInt(lineageDepthInput.value, 10) || 10;
        }
        const musicTauInput = document.getElementById('dial-music-tau');
        if (musicTauInput) {
            params.music_metric_hierarchy_tau = normalizeToActual('dial-music-tau', parseFloat(musicTauInput.value));
        }
        
        // Get resource density from slider using normalized mapping
        const densitySlider = document.getElementById('geo-resource-density');
        if (densitySlider) {
            params.resource_density = normalizeToActual('geo-resource-density', parseFloat(densitySlider.value));
        }
        
        // Get initial resource from slider using normalized mapping
        const initialResourceSlider = document.getElementById('geo-initial-resource');
        if (initialResourceSlider) {
            params.initial_resource = normalizeToActual('geo-initial-resource', parseFloat(initialResourceSlider.value));
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

        // Geography and Environmental toggles
        params.altitude_effects_enabled = document.getElementById('geo-altitude-effects')?.checked ?? false;
        params.weather_enabled = document.getElementById('geo-weather-enabled')?.checked ?? true;

        // Biological, Ecological & Linguistic parameters
        const energySlider = document.getElementById('dial-energy-consumption');
        if (energySlider) params.energy_consumption = normalizeToActual('dial-energy-consumption', parseFloat(energySlider.value));
        const moveSlider = document.getElementById('dial-movement-cost');
        if (moveSlider) params.movement_cost = normalizeToActual('dial-movement-cost', parseFloat(moveSlider.value));
        const agingSlider = document.getElementById('dial-aging-beta');
        if (agingSlider) params.aging_beta = normalizeToActual('dial-aging-beta', parseFloat(agingSlider.value));
        const reproSlider = document.getElementById('dial-repro-threshold');
        if (reproSlider) params.repro_threshold = normalizeToActual('dial-repro-threshold', parseFloat(reproSlider.value));
        const mutationSlider = document.getElementById('dial-mutation-rate');
        if (mutationSlider) params.mutation_rate = normalizeToActual('dial-mutation-rate', parseFloat(mutationSlider.value));
        const driftSlider = document.getElementById('dial-drift');
        if (driftSlider) params.language_drift_rate_factor = parseFloat(driftSlider.value) || 1.0;
        const coherenceSlider = document.getElementById('dial-coherence');
        if (coherenceSlider) params.language_coherence_threshold = parseFloat(coherenceSlider.value) || 0.70;

        // Layer dials: Neurobiology, Ecology, Economics
        const docasAlphaSlider = document.getElementById('dial-docas-alpha');
        if (docasAlphaSlider) {
            params.docas_alpha_base = normalizeToActual('dial-docas-alpha', parseFloat(docasAlphaSlider.value));
        }

        const curvatureSlider = document.getElementById('dial-curvature-window');
        if (curvatureSlider) {
            params.trust_curvature_window = Math.round(normalizeToActual('dial-curvature-window', parseFloat(curvatureSlider.value)));
        }

        const trustDecaySlider = document.getElementById('dial-trust-decay');
        if (trustDecaySlider) {
            params.trust_decay_rate = normalizeToActual('dial-trust-decay', parseFloat(trustDecaySlider.value));
        }

        const reputationSlider = document.getElementById('dial-reputation-threshold');
        if (reputationSlider) {
            params.reputation_threshold = normalizeToActual('dial-reputation-threshold', parseFloat(reputationSlider.value));
        }

        const planningSlider = document.getElementById('dial-planning-horizon');
        if (planningSlider) {
            params.planning_horizon = Math.round(normalizeToActual('dial-planning-horizon', parseFloat(planningSlider.value)));
        }
        const emotionDecaySlider = document.getElementById('dial-emotion-decay');
        if (emotionDecaySlider) {
            params.emotion_decay_rate = normalizeToActual('dial-emotion-decay', parseFloat(emotionDecaySlider.value));
        }
        const worldviewSlider = document.getElementById('dial-worldview-drift');
        if (worldviewSlider) {
            params.worldview_drift_rate = normalizeToActual('dial-worldview-drift', parseFloat(worldviewSlider.value));
        }
        const creditLimitSlider = document.getElementById('dial-credit-limit');
        if (creditLimitSlider) {
            params.credit_limit = normalizeToActual('dial-credit-limit', parseFloat(creditLimitSlider.value));
        }
        const interestRateSlider = document.getElementById('dial-interest-rate');
        if (interestRateSlider) {
            params.interest_rate_base = normalizeToActual('dial-interest-rate', parseFloat(interestRateSlider.value));
        }
        const txCostSlider = document.getElementById('dial-tx-cost');
        if (txCostSlider) {
            params.transaction_cost = normalizeToActual('dial-tx-cost', parseFloat(txCostSlider.value));
        }
        const blockTimeSlider = document.getElementById('dial-block-time');
        if (blockTimeSlider) {
            params.block_time = Math.round(normalizeToActual('dial-block-time', parseFloat(blockTimeSlider.value)));
        }
        const consensusSlider = document.getElementById('dial-consensus-threshold');
        if (consensusSlider) {
            params.consensus_threshold = normalizeToActual('dial-consensus-threshold', parseFloat(consensusSlider.value));
        }
        const epigeneticSlider = document.getElementById('dial-epigenetic-methylation');
        if (epigeneticSlider) {
            params.epigenetic_methylation_rate = normalizeToActual('dial-epigenetic-methylation', parseFloat(epigeneticSlider.value));
        }
        const circadianSlider = document.getElementById('dial-circadian-deficit');
        if (circadianSlider) {
            params.circadian_sleep_deficit = normalizeToActual('dial-circadian-deficit', parseFloat(circadianSlider.value));
        }
        const taxSlider = document.getElementById('dial-tax-redistribution');
        if (taxSlider) {
            params.tax_redistribution_rate = normalizeToActual('dial-tax-redistribution', parseFloat(taxSlider.value));
        }
        const bankRunSlider = document.getElementById('dial-bank-run-sensitivity');
        if (bankRunSlider) {
            params.bank_run_sensitivity = normalizeToActual('dial-bank-run-sensitivity', parseFloat(bankRunSlider.value));
        }
        // Version 4.0: Optogenetic parameters
        const optoWavelength = document.querySelector('input[name="opto-wavelength"]:checked')?.value || 'LaserOff';
        const optoChannels = [
            document.getElementById('opto-channel-d')?.checked || false,
            document.getElementById('opto-channel-o')?.checked || false,
            document.getElementById('opto-channel-c')?.checked || false,
            document.getElementById('opto-channel-a')?.checked || false,
            document.getElementById('opto-channel-s')?.checked || false,
        ];
        const optoScope = document.getElementById('opto-scope-select')?.value || 'entire';
        const optoSubpop = parseInt(document.getElementById('opto-subpop-slider')?.value || '10', 10) / 100;
        const cloakingRatio = parseInt(document.getElementById('dial-cloaking-ratio')?.value || '36', 10) / 10;
        const brainItBranch = document.getElementById('toggle-brainit-branch')?.checked ? 'BranchBSemantic' : 'BranchAStructural';
        const weatherShield = document.getElementById('toggle-weather-shield')?.checked || false;

        params.opto_wavelength = optoWavelength;
        params.opto_target_channels = optoChannels;
        params.opto_scope = optoScope;
        params.opto_subpop_pct = optoSubpop;
        params.visual_cloaking_ratio = cloakingRatio;
        params.brain_it_branch = brainItBranch;
        params.weather_shield_enabled = weatherShield;

        const weatherTemp = parseFloat(document.getElementById('dial-weather-temp')?.value || '20');
        const melanopsinEnabled = document.getElementById('toggle-melanopsin-coupling')?.checked ?? true;
        const baylorMode = document.getElementById('toggle-baylor-mode')?.checked || false;

        params.weather_temperature = weatherTemp;
        params.melanopsin_enabled = melanopsinEnabled;
        params.baylor_unconscious_mode = baylorMode;
        
        return params;
    }
    /**
     * Bind Version 4.0 real-time interventions (Optogenetics, Cloaking, Weather-Shield)
     */
    function bindV4Interventions() {
        const sendIntervention = async () => {
            const optoWavelength = document.querySelector('input[name="opto-wavelength"]:checked')?.value || 'LaserOff';
            const optoChannels = [
                document.getElementById('opto-channel-d')?.checked || false,
                document.getElementById('opto-channel-o')?.checked || false,
                document.getElementById('opto-channel-c')?.checked || false,
                document.getElementById('opto-channel-a')?.checked || false,
                document.getElementById('opto-channel-s')?.checked || false,
            ];
            const optoScope = document.getElementById('opto-scope-select')?.value || 'entire';
            const optoSubpop = parseInt(document.getElementById('opto-subpop-slider')?.value || '10', 10) / 100;
            const cloakingRatio = parseInt(document.getElementById('dial-cloaking-ratio')?.value || '36', 10) / 10;
            const brainItBranch = document.getElementById('toggle-brainit-branch')?.checked ? 'BranchBSemantic' : 'BranchAStructural';
            const weatherShield = document.getElementById('toggle-weather-shield')?.checked || false;
            const weatherTemp = parseFloat(document.getElementById('dial-weather-temp')?.value || '20');
            const melanopsinEnabled = document.getElementById('toggle-melanopsin-coupling')?.checked ?? true;
            const baylorMode = document.getElementById('toggle-baylor-mode')?.checked || false;

            const payload = {
                opto_wavelength: optoWavelength,
                opto_target_channels: optoChannels,
                opto_scope: optoScope,
                opto_subpop_pct: optoSubpop,
                visual_cloaking_ratio: cloakingRatio,
                brain_it_branch: brainItBranch,
                weather_shield_enabled: weatherShield,
                weather_temperature: weatherTemp,
                melanopsin_enabled: melanopsinEnabled,
                baylor_unconscious_mode: baylorMode
            };

            const apiUrl = window.SIM_SERVICE_URL || '';
            try {
                await fetch(`${apiUrl}/api/live/intervention`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(payload)
                });
            } catch (err) {
                console.warn('[Controls] Intervention sync error:', err);
            }
        };
    async function triggerLiveSync() {
        const params = collectSimulationParams();
        const apiUrl = window.SIM_SERVICE_URL || '';
        try {
            await fetch(`${apiUrl}/api/live/intervention`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(params)
            });
            if (window.SimulatorViewer && window.SimulatorViewer.invalidateCaches) {
                window.SimulatorViewer.invalidateCaches();
            }
        } catch (err) {
            console.warn('[Controls] Live intervention sync error:', err);
        }
    }



        document.querySelectorAll('input[name="opto-wavelength"]').forEach(el => el.addEventListener('change', sendIntervention));
        ['d', 'o', 'c', 'a', 's'].forEach(ch => {
            document.getElementById(`opto-channel-${ch}`)?.addEventListener('change', sendIntervention);
        });
        document.getElementById('opto-scope-select')?.addEventListener('change', (e) => {
            const isCustom = e.target.value === 'custom';
            document.getElementById('opto-subpop-wrap')?.classList.toggle('hidden', !isCustom);
            sendIntervention();
        });
        document.getElementById('opto-subpop-slider')?.addEventListener('input', (e) => {
            const val = e.target.value;
            const valDisplay = document.getElementById('opto-subpop-val');
            if (valDisplay) valDisplay.textContent = `${val}%`;
            sendIntervention();
        });
        document.getElementById('dial-cloaking-ratio')?.addEventListener('input', (e) => {
            const ratio = (parseInt(e.target.value, 10) / 10).toFixed(2);
            const valDisplay = document.getElementById('val-cloaking-ratio');
            if (valDisplay) valDisplay.textContent = `${ratio}x`;
            sendIntervention();
        });
        document.getElementById('toggle-brainit-branch')?.addEventListener('change', (e) => {
            const label = document.getElementById('label-brainit-branch');
            if (label) {
                label.textContent = e.target.checked ? 'Semantic Fine-Print Focus (Disclosure Detail)' : 'Layout Structural Margins (Sobel Salience)';
            }
            sendIntervention();
        });
        document.getElementById('dial-weather-temp')?.addEventListener('input', (e) => {
            const val = e.target.value;
            const valDisplay = document.getElementById('val-weather-temp');
            if (valDisplay) valDisplay.textContent = `${val}°C`;
            sendIntervention();
        });
        document.getElementById('toggle-weather-shield')?.addEventListener('change', sendIntervention);
        document.getElementById('toggle-melanopsin-coupling')?.addEventListener('change', sendIntervention);
        document.getElementById('toggle-baylor-mode')?.addEventListener('change', sendIntervention);
    }


    /**
     * Bind agent filter controls
     */
    function bindFilters() {
        // Relative distance threshold (now in Layer 7: Ecology)
        elements.relativeDistanceThreshold?.addEventListener('input', (e) => {
            const value = parseInt(e.target.value, 10);
            if (window.SimulatorViewer && window.SimulatorViewer.setFilter) {
                SimulatorViewer.setFilter('relativeDistanceThreshold', value);
            }
            const valDisplay = document.getElementById('relative-distance-val');
            if (valDisplay) {
                valDisplay.textContent = value;
            }
        });
    }

    /**
     * Bind to live frame updates from viewer
     */
    function bindLiveFrameUpdates() {
        window.addEventListener('simulatorFrame', (e) => {
            updateStatsFromFrame(e.detail);
        });
    }

    /**
     * Update stats display from live frame
     */
    function updateStatsFromFrame(frame) {
        if (!frame) return;

        const stats = SimulatorViewer.getStats();
        if (elements.statAgents) elements.statAgents.textContent = stats.agents;
        if (elements.statAlive) elements.statAlive.textContent = stats.alive;
        if (elements.statVerified) elements.statVerified.textContent = stats.verified;
        if (elements.statGroups) elements.statGroups.textContent = stats.groups;
    }

    // Public API
    return {
        init,
        getCurrentExperiment: () => currentExperiment,
        isCustomConfig: () => isCustomConfig,
        collectSimulationParams: collectSimulationParams
    };
})();

// Initialize on DOM ready
document.addEventListener('DOMContentLoaded', () => {
    SimulatorControls.init();
});

// Keyboard shortcuts
document.addEventListener('keydown', (event) => {
    if (event.target.tagName === 'INPUT' || event.target.tagName === 'SELECT' || event.target.tagName === 'TEXTAREA') {
        return;
    }
    
    if (event.key === 'Escape') {
        if (typeof closeAgentModal === 'function') {
            closeAgentModal();
        }
    } else if (event.key === ' ') {
        event.preventDefault();
        const agentsToggle = document.getElementById('layer-agents');
        if (agentsToggle) {
            agentsToggle.checked = !agentsToggle.checked;
            agentsToggle.dispatchEvent(new Event('change'));
        }
    } else if (event.key === 'e' || event.key === 'E') {
        const docasToggle = document.getElementById('layer-docas');
        if (docasToggle) {
            docasToggle.checked = !docasToggle.checked;
            docasToggle.dispatchEvent(new Event('change'));
        }
    }
});

// Export for other modules
window.SimulatorControls = SimulatorControls;
