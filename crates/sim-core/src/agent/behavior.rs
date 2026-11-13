//! Agent behavior: personality, epigenetics, reproduction, pheromones

use crate::cognition::{
    DecisionOption, MemoryChunk, MemoryContent, PerceivedInfo, WorkingMemory, WorkingMemoryResult,
};
use crate::docas::TrustViscosity;
use crate::epigenome::Epigenome;
use crate::neural::pde4b_pathway::{compute_pathway, PDE4BPathwayParams, PathwayOutput};
use crate::psychology::PersonalityTraits;
use crate::types::{AgentId, Tick};

use super::Agent;

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Agent {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Initialize personality at birth
    ///
    /// Per Costa & `McCrae` (1992): Big Five traits emerge from:
    /// - 50% genetic component (genome loci 20-29)
    /// - 50% environmental component (maternal care + early stress)
    ///
    /// # Arguments
    /// * `maternal_care` - Quality of maternal care [0.0, 1.0]
    /// * `early_stress` - Early life stress exposure [0.0, 1.0]
    /// * `rng` - Random number generator
    pub fn initialize_personality(
        &mut self,
        maternal_care: f32,
        early_stress: f32,
        rng: &mut impl rand::Rng,
    ) {
        use rand::rngs::StdRng;
        use rand::SeedableRng;

        // Create a deterministic RNG from the provided RNG for compatibility
        let seed = rng.gen::<u64>();
        let mut std_rng = StdRng::seed_from_u64(seed);

        self.personality = PersonalityTraits::from_genome_and_environment(
            &self.genome,
            maternal_care,
            early_stress,
            &mut std_rng,
        );

        // Apply initial personality effects to DOCAS
        self.update_docas_from_personality();
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    /// Update DOCAS baseline levels based on personality
    ///
    /// Personality creates stable individual differences in neuromodulator baselines:
    /// - Neuroticism → higher baseline cortisol (stress reactivity)
    /// - Extraversion → higher baseline dopamine (reward sensitivity)
    /// - Agreeableness → higher baseline oxytocin (social bonding)
    pub fn update_docas_from_personality(&mut self) {
        use crate::types::{A, C, D, O, S};

        // Neuroticism → higher baseline cortisol (stress reactivity)
        let cortisol_mod = self.personality.neuroticism * 0.2;
        self.docas.levels[C] = (self.docas.levels[C] + cortisol_mod).min(1.0);

        // Extraversion → higher baseline dopamine (reward approach)
        let dopamine_mod = self.personality.extraversion * 0.15;
        self.docas.levels[D] = (self.docas.levels[D] + dopamine_mod).min(1.0);

        // Agreeableness → higher baseline oxytocin (social bonding)
        let oxytocin_mod = self.personality.agreeableness * 0.15;
        self.docas.levels[O] = (self.docas.levels[O] + oxytocin_mod).min(1.0);

        // Conscientiousness → higher baseline serotonin (stability/self-control)
        let serotonin_mod = self.personality.conscientiousness * 0.1;
        self.docas.levels[S] = (self.docas.levels[S] + serotonin_mod).min(1.0);

        // Openness → slight adrenaline modulation (novelty seeking)
        let adrenaline_mod = self.personality.openness * 0.05;
        self.docas.levels[A] = (self.docas.levels[A] + adrenaline_mod).min(1.0);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Personality-driven economic decision bias
    ///
    /// High Conscientiousness + Low Neuroticism → conservative/risk-averse
    /// High Openness + High Extraversion → risk-taking/exploratory
    ///
    /// Returns value in [-1.0, 1.0] where:
    /// - Negative = conservative/risk-averse bias
    /// - Positive = risk-taking/exploratory bias
    #[must_use]
    pub fn personality_economic_bias(&self) -> f32 {
        let conservatism =
            self.personality.conscientiousness * 0.6 + (1.0 - self.personality.neuroticism) * 0.4;
        let risk_taking = self.personality.openness * 0.5 + self.personality.extraversion * 0.5;

        (risk_taking - conservatism) * 2.0 // Scale to [-1, 1]
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get trust propensity from personality
    ///
    /// High Agreeableness + Low Neuroticism → trusting
    /// Low Agreeableness + High Neuroticism → distrustful
    #[must_use]
    pub fn personality_trust_propensity(&self) -> f32 {
        self.personality.trust_orientation()
    }

    /// Get exploration tendency from personality
    ///
    /// High Openness + High Extraversion → high exploration
    #[must_use]
    pub fn personality_exploration_tendency(&self) -> f32 {
        self.personality.exploration_tendency()
    }

    /// Get stress resilience from personality
    ///
    /// Low Neuroticism + High Conscientiousness → high resilience
    #[must_use]
    pub fn personality_stress_resilience(&self) -> f32 {
        self.personality.stress_resilience()
    }

    /// Compute interaction reputation score
    #[must_use]
    pub fn interaction_reputation(&self) -> f32 {
        let total = self.positive_interactions + self.negative_interactions;
        if total == 0 {
            0.5 // Neutral if no interactions
        } else {
            self.positive_interactions as f32 / total as f32
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Decide whether to trust based on reputation and threshold
    #[must_use]
    pub fn should_trust(&self, reputation: f32, threshold: f32) -> bool {
        reputation >= threshold
    }

    /// Check if this agent can reproduce with another
    #[must_use]
    pub fn can_reproduce_with(&self, other: &Self) -> bool {
        // Different sexes required
        if self.sex == other.sex {
            return false;
        }

        // Both must be alive
        if !self.alive || !other.alive {
            return false;
        }

        // Both must have sufficient energy
        if self.energy < 50.0 || other.energy < 50.0 {
            return false;
        }

        true
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    /// Compute reproduction probability modifier with Allee effect
    ///
    /// # Arguments
    /// * `allee_multiplier` - Current Allee fitness multiplier
    /// * `mate_encounter_prob` - Probability of encountering a mate (0-1)
    ///
    /// # Returns
    /// Modified reproduction probability in [0, 1]
    #[must_use]
    pub fn reproduction_probability_allee(
        &self,
        allee_multiplier: f32,
        mate_encounter_prob: f32,
    ) -> f32 {
        // Base reproduction probability based on energy level
        let base_prob = if self.energy >= 75.0 {
            0.8
        } else if self.energy >= 50.0 {
            0.5
        } else {
            0.0
        };

        // Apply Allee effects: fitness * mate encounter probability
        base_prob * allee_multiplier * mate_encounter_prob
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute survival probability modifier with Allee effect
    ///
    /// # Arguments
    /// * `allee_multiplier` - Current Allee fitness multiplier
    /// * `defense_bonus` - Cooperative defense bonus from Allee effect
    ///
    /// # Returns
    /// Modified survival probability multiplier
    #[must_use]
    pub fn survival_probability_allee(&self, allee_multiplier: f32, defense_bonus: f32) -> f32 {
        // Base survival modifier
        let base = 1.0;

        // Defense bonus reduces mortality (max 10% bonus)
        let defense_factor = 1.0 + defense_bonus * 0.1;

        // Combined: fitness multiplier improves survival, defense adds bonus
        base * allee_multiplier * defense_factor
    }

    // === Phase 3.1: Geometric Trust Properties ===

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update geometric properties from DOCAS history (P3-1)
    ///
    /// Updates trust curvature and viscosity based on recent DOCAS states.
    /// This should be called before DOCAS update each tick.
    pub fn update_geometric_properties(&mut self) {
        // Push current DOCAS to curvature history
        self.trust_curvature.push_state(self.docas);
        self.trust_curvature.update();

        // Update tau history for viscosity computation
        let current_tau = self.docas.compute_tau();
        self.update_tau_history(current_tau);

        // Compute current viscosity from recent volatility
        let volatility = TrustViscosity::compute_volatility_from_changes(&self.tau_history);
        let psi = self
            .trust_viscosity
            .compute_viscosity(volatility, current_tau);

        // Update adaptation rate (inverse of viscosity)
        self.adaptation_rate = self.trust_viscosity.adaptation_rate(psi);
    }

    /// Get current curvature (κ) value (P3-1)
    #[must_use]
    pub const fn curvature_kappa(&self) -> f32 {
        self.trust_curvature.curvature()
    }

    /// Get current viscosity (ψ) value (P3-1)
    #[must_use]
    pub fn viscosity_psi(&self) -> f32 {
        let volatility = TrustViscosity::compute_volatility_from_changes(&self.tau_history);
        self.trust_viscosity
            .compute_viscosity(volatility, self.docas.compute_tau())
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get coupling strength to another agent using cosine similarity (P3-1)
    ///
    /// Returns value in [-1.0, 1.0] where:
    /// - 1.0 = identical DOCAS states (maximum coupling)
    /// - 0.0 = orthogonal states (no coupling)
    /// - -1.0 = opposite states (repulsion)
    #[must_use]
    pub fn coupling_strength(&self, other: &Self) -> f32 {
        crate::docas::cosine_similarity(&self.docas.to_array(), &other.docas.to_array())
    }

    /// Compute the complete PDE4B → cAMP → PKA → CREB pathway
    ///
    /// Returns the full pathway output for debugging and analysis.
    #[must_use]
    pub fn compute_pde4b_pathway(&self) -> PathwayOutput {
        let pde4b_level = self.genome.pde4b_level();
        let params = PDE4BPathwayParams::default();
        compute_pathway(pde4b_level, &params)
    }

    /// Compute maturation rate using the PDE4B signaling pathway
    ///
    /// Uses the full 4-step biochemical cascade:
    /// PDE4B → cAMP → PKA → CREB → maturation modulation
    ///
    /// # Returns
    /// Effective maturation rate multiplier
    #[must_use]
    pub fn compute_maturation_rate_pathway(&self) -> f32 {
        let pde4b = self.genome.pde4b_level();
        let stress_level = self.prefrontal_cortex.chronic_cortisol;
        let resource_level = self.energy / 100.0;
        let stress_sensitivity = self.genome.stress_sensitivity();

        crate::neural::pde4b_pathway::compute_maturation_rate_pathway(
            pde4b,
            stress_level,
            resource_level,
            stress_sensitivity,
        )
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute weighted trust update with coupling field influence (P3-1)
    ///
    /// When agents have high coupling (similar DOCAS), trust updates
    /// are reinforced. Low coupling dampens external trust signals.
    #[must_use]
    pub fn coupled_trust_update(&self, external_trust_signal: f32, coupling: f32) -> f32 {
        // Weighted combination: coupling determines how much external signal affects trust
        // High coupling = more influence from external signals
        let coupling_weight = (coupling + 1.0) / 2.0; // Normalize to [0, 1]
        coupling_weight * external_trust_signal + (1.0 - coupling_weight) * self.expectation
    }

    // === Epigenetics Integration (DNA Methylation) ===

    /// Apply early life programming (Meaney & Szyf model)
    ///
    /// Programs DNA methylation patterns based on early life experiences.
    /// High maternal care → protective methylation (stress resilience)
    /// Low care + high stress → stress-sensitizing methylation
    ///
    /// # Arguments
    /// * `maternal_care` - Level of maternal care [0, 1], higher = better care
    /// * `tick` - Current simulation tick
    pub fn program_early_life(&mut self, maternal_care: f32, tick: Tick) {
        self.epigenome
            .apply_early_life_experience(maternal_care, self.docas.cortisol(), tick);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update stress-related epigenetics
    ///
    /// Applies chronic stress effects and ages the epigenome.
    /// Should be called each tick.
    ///
    /// # Arguments
    /// * `tick` - Current simulation tick
    pub fn update_epigenetics(&mut self, tick: Tick) {
        // Apply chronic stress methylation
        self.epigenome
            .apply_chronic_stress(self.docas.cortisol(), tick);

        // Age the epigenome (with stress acceleration)
        self.epigenome.age_tick(self.docas.cortisol());
    }

    /// Get effective stress sensitivity considering epigenetics
    ///
    /// Methylation at stress sensitivity loci (38-39) affects how
    /// strongly the agent responds to stress.
    ///
    /// # Returns
    /// Epigenetically-modified stress sensitivity value
    #[must_use]
    pub fn effective_stress_sensitivity(&self) -> f32 {
        let base = self.genome.stress_sensitivity();

        // Apply methylation effects at both stress loci
        let expr_38 = self.epigenome.get_expression(38, 1.0);
        let expr_39 = self.epigenome.get_expression(39, 1.0);

        // Average expression across both loci
        let avg_expression = (expr_38 + expr_39) / 2.0;

        // Higher methylation (lower expression) → higher stress sensitivity
        // (silenced GR gene → poor stress regulation)
        base * (1.5 - avg_expression * 0.5)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Inherit epigenome from parents (transgenerational epigenetics)
    ///
    /// Called during reproduction to pass methylation patterns to offspring.
    /// Some marks persist across generations, others are reset.
    ///
    /// # Arguments
    /// * `parent1` - First parent's epigenome
    /// * `parent2` - Second parent's epigenome
    pub fn inherit_epigenome(&mut self, parent1: &Epigenome, parent2: &Epigenome) {
        self.epigenome.inherit_from_parents(parent1, parent2);
    }

    /// Get epigenetic age acceleration ratio
    ///
    /// Returns `epigenetic_age` / `chronological_age`
    /// > 1.0 indicates accelerated aging from stress
    #[must_use]
    pub fn epigenetic_clock(&self) -> f32 {
        self.epigenome.epigenetic_clock(self.age)
    }

    /// Check if agent has been programmed by early life experiences
    #[must_use]
    pub const fn is_early_life_programmed(&self) -> bool {
        self.epigenome.early_life_programmed
    }

    /// Update epigenetics from environmental factors
    ///
    /// Applies environmental effects like nutrition, toxins, etc.
    ///
    /// # Arguments
    /// * `tick` - Current simulation tick
    /// * `rng` - Random number generator
    pub fn update_environmental_epigenetics(&mut self, tick: Tick, rng: &mut impl rand::Rng) {
        let energy_ratio = (self.energy / 100.0).clamp(0.0, 1.0);

        self.epigenome
            .update_environmental_methylation(energy_ratio, tick, rng);
        self.epigenome.add_spontaneous_methylation(tick, rng);
    }

    // === Working Memory Integration (Miller's Law) ===

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Process information with working memory limits
    /// Encodes perceived information into working memory if capacity allows
    pub fn perceive_and_remember(&mut self, info: PerceivedInfo, tick: Tick) {
        // Calculate priority based on agent's current state
        let adjusted_priority = self.calculate_priority(&info);

        let chunk = MemoryChunk::new(info.content.clone(), adjusted_priority, tick);

        // Try to add to working memory
        let result = self.working_memory.add(chunk);

        // Handle rejection - if high priority, force addition
        if let WorkingMemoryResult::Rejected { .. } = result {
            if adjusted_priority > 0.8 {
                let chunk = MemoryChunk::new(info.content, adjusted_priority, tick);
                self.working_memory.force_add(chunk);
            }
            // Otherwise, info is lost (not encoded)
        }
    }

    /// Calculate priority of information based on agent state
    fn calculate_priority(&self, info: &PerceivedInfo) -> f32 {
        let base_priority = info.priority;

        // Stress increases priority of threat-related info
        let stress_factor = self.docas.cortisol();

        // Emotional arousal can distort priorities
        let arousal = self.docas.adrenaline();
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // Modulate based on content type
        let content_multiplier = match &info.content {
            MemoryContent::AgentInfo { trust_level, .. } => {
                // Novel agents or high-threat agents get priority
                if *trust_level < 0.3 {
                    1.5 * (1.0 + stress_factor) // Threat prioritization
                } else {
                    1.0
                }
            }
            MemoryContent::SpatialInfo { threat_level, .. } => {
                // Threats get priority when stressed
                1.0 + (*threat_level * stress_factor)
            }
            MemoryContent::EconomicInfo { value, risk, .. } => {
                // Economic opportunities modulated by risk tolerance
                let risk_tolerance = 1.0 - self.personality.neuroticism;
                let risk_adjustment = *risk * risk_tolerance;
                (*value * 0.5 + risk_adjustment * 0.5).clamp(0.0, 1.0)
            }
            _ => 1.0,
        };

        // Arousal can distort priorities (system 1 override)
        let arousal_distortion = if arousal > 0.7 {
            1.3 // Boost priority under high arousal
        } else {
            1.0
        };

        (base_priority * content_multiplier * arousal_distortion).clamp(0.0, 1.0)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Make decision with limited working memory
    /// Can only consider options that fit in available capacity
    #[must_use]
    pub fn make_decision(&self, options: &[DecisionOption]) -> Option<DecisionOption> {
        // Filter options by complexity
        let available_capacity = self.working_memory.available_slots();

        let feasible_options: Vec<&DecisionOption> = options
            .iter()
            .filter(|opt| opt.complexity <= available_capacity)
            .collect();

        if feasible_options.is_empty() {
            // Cannot process any option - fallback to simplest
            return options.iter().min_by_key(|opt| opt.complexity).cloned();
        }

        // Evaluate feasible options using only focused items
        let available_info: Vec<&MemoryChunk> = self
            .working_memory
            .focused_items()
            .iter()
            .chain(self.working_memory.items.iter())
            .collect();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Choose best option based on expected value and available information
        feasible_options
            .iter()
            .max_by(|a, b| {
                let score_a = self.evaluate_option(a, &available_info);
                let score_b = self.evaluate_option(b, &available_info);
                score_a
                    .partial_cmp(&score_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
            .cloned()
    }

    /// Evaluate a decision option based on available information
    fn evaluate_option(&self, option: &DecisionOption, available_info: &[&MemoryChunk]) -> f32 {
        // Base score from expected value
        let mut score = option.expected_value;

        // Modulate by risk and personality
        let risk_weight = self.personality.neuroticism;
        let risk_penalty = option.risk * risk_weight;
        score -= risk_penalty;

        // Bonus for options matching goals in working memory
        for chunk in available_info {
            if let MemoryContent::Goal { priority, .. } = &chunk.content {
                // If option aligns with goal, boost score
                score += priority * 0.2;
            }
        }

        // Penalty for complexity if working memory is strained
        let utilization = self.working_memory.utilization();
        let complexity_penalty = option.complexity as f32 * utilization * 0.1;
        score -= complexity_penalty;

        score.clamp(0.0, 1.0)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update working memory each tick
    /// Decay old memories, rehearse important items, update focus
    pub fn update_working_memory(&mut self, tick: Tick) {
        // Decay old memories
        self.working_memory.decay(tick);

        // Rehearse items in rehearsal queue
        self.working_memory.rehearse(tick);

        // Update focus based on current situation
        self.working_memory
            .focus_attention(WorkingMemory::FOCUS_LIMIT);

        // Update executive control based on stress/arousal
        self.working_memory
            .update_executive_control(self.docas.cortisol(), self.docas.adrenaline());
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update working memory capacity based on neural maturation
    /// Called by `update_neural_maturation` to sync PFC development with WM capacity
    pub fn update_working_memory_capacity(&mut self) {
        let dlpfc_development = self.prefrontal_cortex.dlpfc.development;

        // Calculate new capacity based on development
        // Base 5 + up to 4 from development = 9 max (Miller's 7+2)
        let new_capacity = (5.0 + dlpfc_development * 4.0) as usize;
        let clamped_capacity =
            new_capacity.clamp(WorkingMemory::MIN_CAPACITY, WorkingMemory::MAX_CAPACITY);

        // Update capacity if it changed
        if clamped_capacity != self.working_memory.capacity {
            // If new capacity is smaller, we may need to drop items
            while self.working_memory.used_slots > clamped_capacity {
                if self.working_memory.drop_lowest_priority().is_none() {
                    break;
                }
            }
            self.working_memory.capacity = clamped_capacity;
        }
    }

    /// Check if agent can hold an item in working memory
    #[must_use]
    pub const fn can_remember(&self, chunk: &MemoryChunk) -> bool {
        self.working_memory.can_add(chunk)
    }

    /// Get working memory statistics
    #[must_use]
    pub fn working_memory_stats(&self) -> crate::cognition::WorkingMemoryStats {
        self.working_memory.stats()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Force attention on a specific category
    pub fn focus_on(&mut self, category: crate::cognition::MemoryCategory) {
        self.working_memory.focus_on_category(category);
    }

    // === Inbreeding Depression Methods ===

    /// Initialize inbreeding tracker from parentage
    ///
    /// Call this when creating an offspring to compute F coefficient
    /// and apply fitness effects.
    ///
    /// # Arguments
    /// * `parent1_id` - First parent's agent ID
    /// * `parent2_id` - Second parent's agent ID (None for asexual/cloning)
    /// * `lineage` - Lineage tree for ancestry lookup
    pub fn initialize_inbreeding(
        &mut self,
        parent1_id: AgentId,
        parent2_id: Option<AgentId>,
        lineage: &crate::lineage::LineageTree,
    ) {
        use crate::genetics::InbreedingTracker;

        let p2_id = parent2_id.unwrap_or(parent1_id);
        let f = InbreedingTracker::from_pedigree(parent1_id, p2_id, lineage);

        self.inbreeding.f_coefficient = f;
        self.inbreeding.compute_fitness(&self.genome);

        // Apply inbreeding effects to initial energy
        self.energy = self.inbreeding.apply_energy_penalty(self.energy);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if this agent can reproduce with another considering inbreeding avoidance
    ///
    /// # Arguments
    /// * `other` - Potential mate
    /// * `lineage` - Lineage tree for ancestry lookup
    ///
    /// # Returns
    /// true if reproduction is permitted based on inbreeding coefficient
    #[must_use]
    pub fn can_reproduce_inbreeding(
        &self,
        other: &Self,
        lineage: &crate::lineage::LineageTree,
    ) -> bool {
        crate::genetics::can_reproduce_inbreeding(self.id, other.id, lineage)
    }

    /// Get the inbreeding coefficient F
    #[must_use]
    pub const fn inbreeding_coefficient(&self) -> f32 {
        self.inbreeding.f_coefficient
    }

    /// Get the fitness multiplier from inbreeding depression
    #[must_use]
    pub const fn inbreeding_fitness(&self) -> f32 {
        self.inbreeding.fitness_multiplier
    }

    /// Check if this agent shows inbreeding depression
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[must_use]
    pub fn has_inbreeding_depression(&self) -> bool {
        self.inbreeding.has_depression()
    }

    /// Get mortality risk increase from inbreeding
    #[must_use]
    pub fn inbreeding_mortality_risk(&self) -> f32 {
        self.inbreeding.mortality_risk_increase()
    }

    // === Stigmergy Methods (SIM-STIG-001) ===

    /// Threshold for following pheromone gradients
    pub const PHEROMONE_THRESHOLD: f32 = 0.1;

    /// Weight for blending base movement with chemotaxis (0.0 = all base, 1.0 = all chemotaxis)
    pub const CHEMOTAXIS_WEIGHT: f32 = 0.4;

    /// Sense pheromones at current position
    #[must_use]
    pub fn sense_pheromones(&self, grid: &crate::stigmergy::PheromoneGrid) -> [f32; 5] {
        use crate::stigmergy::PheromoneType;
        [
            grid.concentration(&self.position, PheromoneType::FoodTrail)
                * self.pheromone_sensitivity[0],
            grid.concentration(&self.position, PheromoneType::DangerSignal)
                * self.pheromone_sensitivity[1],
            grid.concentration(&self.position, PheromoneType::Reproduction)
                * self.pheromone_sensitivity[2],
            grid.concentration(&self.position, PheromoneType::Territory)
                * self.pheromone_sensitivity[3],
            grid.concentration(&self.position, PheromoneType::Recruitment)
                * self.pheromone_sensitivity[4],
        ]
    }
}
