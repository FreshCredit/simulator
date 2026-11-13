//! Agent cognition: working memory, time preference, philosophy, game theory

use super::Agent;

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Agent {
    /// Follow pheromone gradient (returns direction to move)
    #[must_use]
    pub fn follow_gradient(
        &self,
        grid: &crate::stigmergy::PheromoneGrid,
        pheromone: crate::stigmergy::PheromoneType,
    ) -> crate::types::Direction {
        grid.gradient(&self.position, pheromone)
    }

    /// Deposit pheromone at current position
    pub fn deposit_pheromone(
        &self,
        grid: &mut crate::stigmergy::PheromoneGrid,
        pheromone: crate::stigmergy::PheromoneType,
        amount: f32,
        _tick: u32,
    ) {
        grid.deposit(&self.position, pheromone, amount);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Deposit pheromones based on current state and activity
    ///
    /// This method determines which pheromones to deposit based on the agent's
    /// current physiological and social state. Agents continuously emit chemical
    /// signals that other agents can detect and follow.
    ///
    /// # Arguments
    /// * `grid` - The pheromone grid to deposit into
    /// * `tick` - Current simulation tick (for timing-based deposits)
    pub fn deposit_pheromones(&self, grid: &mut crate::stigmergy::PheromoneGrid, tick: u32) {
        use crate::stigmergy::PheromoneType;
        use crate::types::C;

        // Deposit food trail when carrying significant energy (analogous to carrying food)
        // High energy agents mark paths to help others find resources
        if self.energy > 60.0 {
            // Deposit amount proportional to energy surplus
            let amount = ((self.energy - 60.0) / 40.0 * 0.5).min(0.5);
            if amount > 0.05 {
                grid.deposit(&self.position, PheromoneType::FoodTrail, amount);
            }
        }

        // Deposit danger signal when cortisol is high (stress/threat detected)
        // Warns other agents of potential danger
        if self.docas.levels[C] > 0.7 {
            let amount = ((self.docas.levels[C] - 0.7) / 0.3 * 0.8).min(0.8);
            grid.deposit(&self.position, PheromoneType::DangerSignal, amount);
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001

        // Deposit reproduction marker when ready to mate
        // Signals availability to potential mates
        if self.can_reproduce(tick) {
            grid.deposit(&self.position, PheromoneType::Reproduction, 0.6);
        }

        // Deposit territory marker periodically for grouped agents
        // Reinforces group boundaries
        if tick % 100 == 0 && self.group_id.is_some() {
            grid.deposit(&self.position, PheromoneType::Territory, 0.3);
        }

        // Update last deposit tick
        // Note: This is stored in agent, but we need &mut self to update it
        // The caller should update this after calling deposit_pheromones
    }

    /// Check if agent is ready to reproduce (for pheromone deposition)
    ///
    /// An agent can reproduce if:
    /// - It has sufficient energy
    /// - It has reached neural maturity
    /// - It is of appropriate age
    fn can_reproduce(&self, tick: u32) -> bool {
        // Basic checks
        if !self.alive {
            return false;
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Energy threshold for reproduction
        if self.energy < 50.0 {
            return false;
        }

        // Age requirement (at least "adolescent" ~100 ticks)
        let age = tick.saturating_sub(self.birth_tick as u32);
        if age < 100 {
            return false;
        }

        // Neural maturity check (if PFC is sufficiently developed)
        self.is_neurally_mature()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Apply social force to agent (affects movement)
    pub fn apply_social_force(&mut self, force: crate::types::Vector) {
        // Store force to be applied during movement phase
        self.social_force = force;
    }

    /// Clear accumulated social force
    pub fn clear_social_force(&mut self) {
        self.social_force = crate::types::Vector::zero();
    }

    // === Temporal Decision Methods (Hyperbolic Discounting) ===

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Decide between immediate and delayed reward using hyperbolic discounting
    ///
    /// **Theoretical Basis:**
    /// - Mazur (1987): Hyperbolic discounting model V = A / (1 + k·t)
    /// - People exhibit present bias: prefer immediate rewards over larger future ones
    ///
    /// **Parameters:**
    /// - `immediate_reward`: Value available now
    /// - `delayed_reward`: Value available after delay
    /// - `delay_ticks`: Time delay in simulation ticks
    ///
    /// **Returns:** true if immediate reward is preferred
    ///
    /// **Example (Debt Decision):**
    /// ```ignore
    /// // Keep money now vs pay debt for reputation later
    /// let prefer_now = agent.choose_intertemporal_hyperbolic(
    ///     debt_amount,      // Keep money now
    ///     reputation_gain,  // Reputation from paying
    ///     repayment_delay,  // Ticks until reputation benefit
    /// );
    /// if prefer_now {
    ///     // Agent defaults (prefers immediate gratification)
    /// } else {
    ///     // Agent repays (values future reputation)
    /// }
    /// ```
    #[must_use]
    pub fn choose_intertemporal_hyperbolic(
        &self,
        immediate_reward: f32,
        delayed_reward: f32,
        delay_ticks: u32,
    ) -> bool {
        use crate::economics::hyperbolic_discount;

        let discounted_delayed = hyperbolic_discount(
            delayed_reward,
            delay_ticks,
            self.time_preference.impatience_k,
        );

        immediate_reward > discounted_delayed
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Decide using quasi-hyperbolic (β-δ) model
    ///
    /// **Theoretical Basis:**
    /// - Laibson (1997): "Golden Eggs and Hyperbolic Discounting"
    /// - Separates present bias (β) from long-term impatience (δ)
    ///
    /// **Formula:**
    /// - V = A if t = 0 (immediate)
    /// - V = β × δ^t × A if t > 0 (delayed)
    ///
    /// **Returns:** true if immediate reward is preferred
    #[must_use]
    pub fn choose_intertemporal_quasi_hyperbolic(
        &self,
        immediate_reward: f32,
        delayed_reward: f32,
        delay_ticks: u32,
    ) -> bool {
        use crate::economics::quasi_hyperbolic_discount;

        let discounted = quasi_hyperbolic_discount(
            delayed_reward,
            delay_ticks,
            self.time_preference.present_bias,
            self.time_preference.long_term_discount,
        );

        immediate_reward > discounted
    }

    /// Compute present value of a future reward using agent's time preference
    ///
    /// **Returns:** Discounted value of the reward
    #[must_use]
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn present_value(&self, reward: f32, delay_ticks: u32) -> f32 {
        self.time_preference.present_value(reward, delay_ticks)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Sophisticated choice (accounts for future self-control problems)
    ///
    /// **Agent Types:**
    /// - Sophisticated: Aware of their present bias, anticipate preference reversals
    /// - Naive: Unaware, constantly planning to start tomorrow
    ///
    /// **Returns:** Index of chosen option from rewards/delays arrays
    ///
    /// **Note:** This is a simplified implementation. Full sophisticated choice
    /// requires solving a complex dynamic programming problem.
    #[must_use]
    /// # Panics
    ///
    /// Panics if preconditions are not met.
    pub fn choose_sophisticated(&self, rewards: &[f32], delays: &[u32]) -> usize {
        debug_assert!(!rewards.is_empty(), "Must provide at least one reward");
        debug_assert_eq!(rewards.len(), delays.len(), "Rewards and delays must match");

        if !self.time_preference.is_sophisticated {
            // Naive: choose based on current preferences (may reverse later)
            return self.choose_naive(rewards, delays);
        }

        // Sophisticated: anticipate future preference reversals
        // Choose commitment strategy if available
        self.choose_with_commitment(rewards, delays)
    }

    /// Naive choice: select option with highest present value
    fn choose_naive(&self, rewards: &[f32], delays: &[u32]) -> usize {
        let mut best_idx = 0;
        let mut best_value = self.present_value(rewards[0], delays[0]);

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        for (i, (&reward, &delay)) in rewards.iter().zip(delays.iter()).enumerate().skip(1) {
            let value = self.present_value(reward, delay);
            if value > best_value {
                best_value = value;
                best_idx = i;
            }
        }

        best_idx
    }

    /// Sophisticated choice with commitment consideration
    ///
    /// Agents who recognize their present bias may:
    /// 1. Choose commitment devices (options that restrict future choices)
    /// 2. Avoid options that will trigger future preference reversals
    fn choose_with_commitment(&self, rewards: &[f32], delays: &[u32]) -> usize {
        // Simplified sophisticated choice:
        // Prefer options with shorter delays when similar rewards
        // This avoids the temptation of near-term preference reversals

        let mut best_idx = 0;
        let mut best_score = self.compute_sophisticated_score(rewards[0], delays[0]);

        for (i, (&reward, &delay)) in rewards.iter().zip(delays.iter()).enumerate().skip(1) {
            let score = self.compute_sophisticated_score(reward, delay);
            if score > best_score {
                best_score = score;
                best_idx = i;
            }
        }

        best_idx
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute choice score for sophisticated agents
    /// Penalizes long delays to avoid future temptation
    fn compute_sophisticated_score(&self, reward: f32, delay: u32) -> f32 {
        let present_val = self.present_value(reward, delay);
        // Sophisticated agents discount future rewards more heavily
        // to avoid putting themselves in tempting situations
        let commitment_factor = if delay > 0 {
            // Extra discount for long delays (avoiding temptation)
            1.0 - (self.time_preference.present_bias * 0.2)
        } else {
            1.0 // No extra discount for immediate rewards
        };
        present_val * commitment_factor
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Decide whether to default on a debt obligation
    ///
    /// **Parameters:**
    /// - `debt_amount`: Amount owed (immediate benefit of keeping)
    /// - `reputation_value`: Long-term reputation gain from paying
    /// - `delay_ticks`: Time until reputation benefit realized
    ///
    /// **Returns:** true if agent chooses to default (prefers immediate money)
    ///
    /// **Example:**
    /// ```ignore
    /// let should_default = agent.decide_debt_default(
    ///     debt.amount,
    ///     reputation_gain,
    ///     debt.due_tick - current_tick,
    /// );
    /// ```
    #[must_use]
    pub fn decide_debt_default(
        &self,
        debt_amount: f32,
        reputation_value: f32,
        delay_ticks: u32,
    ) -> bool {
        // Use quasi-hyperbolic model for debt decisions
        // Immediate benefit = keeping the debt money
        // Delayed benefit = reputation gain from repayment
        self.choose_intertemporal_quasi_hyperbolic(debt_amount, reputation_value, delay_ticks)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update time preference profile based on life experiences
    ///
    /// Time preferences can shift based on:
    /// - Age (older agents become more patient)
    /// - Stress (high cortisol increases present bias)
    /// - Social context (trust affects future orientation)
    pub fn update_time_preference(&mut self) {
        // Age effect: agents become more patient as they mature
        // vmPFC development correlates with reduced present bias
        let age_factor = (self.prefrontal_cortex.vmpfc.development * 0.1).min(0.1);
        self.time_preference.present_bias =
            (self.time_preference.present_bias + age_factor * 0.05).min(1.0);

        // Stress effect: high cortisol increases impulsivity
        let cortisol = self.docas.cortisol();
        if cortisol > 0.5 {
            // Stress makes agents more present-biased
            self.time_preference.impatience_k =
                (self.time_preference.impatience_k + cortisol * 0.01).min(0.5);
        }

        // Normalize values
        self.time_preference.present_bias = self.time_preference.present_bias.clamp(0.5, 1.0);
        self.time_preference.long_term_discount =
            self.time_preference.long_term_discount.clamp(0.9, 0.99);
        self.time_preference.impatience_k = self.time_preference.impatience_k.clamp(0.01, 0.5);
    }

    // === Cross-Domain Coherence (Ψ_trust) Methods ===

    /// Make decision using cross-domain coherence
    ///
    /// Returns the index of the chosen option based on:
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// - High coherence + high trust → choose cooperative/max option
    /// - High coherence + low trust → choose conservative/min option
    /// - Low coherence → choose default/safest option (index 0)
    #[must_use]
    pub fn decide_with_coherence(&self, options: &[f32]) -> usize {
        if options.is_empty() {
            return 0;
        }

        // Compute cross-domain coherence from psi_trust fields
        let coherence = self.compute_psi_coherence();
        let confidence = 0.3 + 0.7 * coherence; // Same mapping as CrossDomainTrust

        if confidence < 0.4 || options.len() == 1 {
            // Low coherence → choose default/safest option
            return 0;
        }

        // Get unified trust estimate
        let unified_tau = self.unified_trust();

        if unified_tau > 0.5 {
            // Cooperative option: choose max
            options
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map_or(0, |(i, _)| i)
        } else {
            // Conservative option: choose min (safest)
            options
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map_or(0, |(i, _)| i)
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if agent should seek social information
    ///
    /// Returns true when cross-domain coherence is low, indicating
    /// conflicting signals that require more data to resolve
    #[must_use]
    pub fn should_seek_information(&self) -> bool {
        let coherence = self.compute_psi_coherence();
        // Below threshold, need more data
        coherence < 0.5
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get unified trust estimate across all domains
    ///
    /// Combines DOCAS-based trust with stored `psi_trust` values
    /// and philosophical state for a comprehensive trust estimate.
    ///
    /// This is the unified interface for external use.
    #[must_use]
    pub fn unified_trust(&self) -> f32 {
        // Combine all trust sources with weights
        let docas_tau = self.compute_trust();

        // Philosophy-based trust component (τ_φ axis mapped to [0,1])
        let philosophy_tau = self
            .philosophical_state
            .as_ref()
            .map_or(docas_tau / 2.0, |p| (p.worldview.trust_axis + 1.0) / 2.0);

        // Cross-domain (psi) trust component
        let psi_tau = (self.psi_bio + self.psi_fin + self.psi_ling) / 3.0;

        // Weighted combination: DOCAS 50%, Philosophy 25%, Psi 25%
        0.5 * docas_tau + 0.25 * philosophy_tau + 0.25 * psi_tau
    }

    /// Compute cross-domain coherence from psi fields
    ///
    /// Returns coherence in [0, 1] where higher = more agreement
    /// between biological, financial, and linguistic domains
    #[must_use]
    pub fn compute_psi_coherence(&self) -> f32 {
        let scores = [self.psi_bio, self.psi_fin, self.psi_ling];
        let mean = scores.iter().sum::<f32>() / scores.len() as f32;

        if mean <= 0.0 {
            return 0.0;
        }

        // Compute variance
        let variance =
            scores.iter().map(|&s| (s - mean).powi(2)).sum::<f32>() / scores.len() as f32;

        // Coefficient of variation inverted = coherence
        let cv = (variance.sqrt() / mean).min(1.0);
        1.0 - cv
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Log domain conflicts for debugging
    ///
    /// Prints warnings when psi domains show significant disagreement
    pub fn log_domain_conflicts(&self) {
        let mean_psi = (self.psi_bio + self.psi_fin + self.psi_ling) / 3.0;
        const CONFLICT_THRESHOLD: f32 = 0.3;

        let bio_dev = (self.psi_bio - mean_psi).abs();
        let fin_dev = (self.psi_fin - mean_psi).abs();
        let ling_dev = (self.psi_ling - mean_psi).abs();

        if bio_dev > CONFLICT_THRESHOLD
            || fin_dev > CONFLICT_THRESHOLD
            || ling_dev > CONFLICT_THRESHOLD
        {
            eprintln!(
                "Agent {} domain conflicts: bio={:.2}(dev={:.2}), fin={:.2}(dev={:.2}), ling={:.2}(dev={:.2})",
                self.id, self.psi_bio, bio_dev, self.psi_fin, fin_dev, self.psi_ling, ling_dev
            );
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update `psi_trust` fields from cross-domain alignment
    ///
    /// This should be called periodically to sync psi fields with
    /// the current cross-domain trust state
    pub fn update_psi_from_alignment(&mut self, psi_trust: &crate::cross_domain::CrossDomainTrust) {
        // Map domain scores to psi fields
        self.psi_bio = psi_trust.domain_scores[crate::cross_domain::DOMAIN_DOCAS];
        self.psi_fin = psi_trust.domain_scores[crate::cross_domain::DOMAIN_DOCAS];
        self.psi_ling = psi_trust.domain_scores[crate::cross_domain::DOMAIN_LANGUAGE];
        self.psi_trust = psi_trust.psi_trust;
    }

    // === Philosophy-Based Decision Methods (τ_φ Integration) ===

    /// Make decision using philosophical framework
    ///
    /// # Arguments
    /// * `cooperate_utility` - Expected utility from cooperating
    /// * `defect_utility` - Expected utility from defecting
    /// * `partner` - The other agent in the interaction
    ///
    /// Returns true if agent chooses to cooperate
    #[must_use]
    pub fn decide_with_philosophy(
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        &self,
        cooperate_utility: f32,
        defect_utility: f32,
        partner: &Self,
    ) -> bool {
        if let Some(ref phil) = self.philosophical_state {
            // Get partner's τ_φ
            let partner_tau_phi = partner
                .philosophical_state
                .as_ref()
                .map_or(0.0, |p| p.worldview.trust_axis);

            // Cooperation willingness based on philosophical alignment
            let coop_willingness = phil.cooperation_willingness(partner_tau_phi);

            // Ethical weighting of outcomes
            let coop_weight = phil.ethical_weight(&[cooperate_utility, partner_tau_phi], true);
            let defect_weight = phil.ethical_weight(&[defect_utility], false);

            // Risk tolerance affects decision threshold
            let threshold = phil.decision_threshold();

            // Final decision: cooperate if weighted willingness exceeds threshold
            coop_willingness * coop_weight > threshold * defect_weight
        } else {
            // Fall back to trust-based decision
            self.compute_trust() > 0.5
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Check if agent will honor commitment based on philosophy
    ///
    /// # Arguments
    /// * `cost` - Cost of honoring the commitment
    /// * `benefit` - Benefit from honoring the commitment
    ///
    /// Returns true if agent will honor the commitment
    #[must_use]
    pub fn will_honor_commitment(&self, cost: f32, benefit: f32) -> bool {
        if let Some(ref phil) = self.philosophical_state {
            let _commitment = phil.commitment_strength();
            let net_benefit = benefit - cost;

            // Kantian: Honor regardless of cost (duty-based)
            // Utilitarian: Honor if net benefit positive
            // Virtue: Honor if virtuous (high integrity)
            match phil.ethical_framework {
                crate::philosophy::EthicalFramework::Kantian => true,
                crate::philosophy::EthicalFramework::Utilitarian => net_benefit > 0.0,
                crate::philosophy::EthicalFramework::VirtueEthics => {
                    // Check if breaking commitment harms integrity
                    let integrity = phil.virtues.get(crate::philosophy::Virtue::Integrity);
                    integrity > 0.5 || net_benefit > cost * 2.0
                }
            }
        } else {
            // Default: cost-benefit analysis
            benefit > cost
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Update philosophy based on life experiences
    ///
    /// # Arguments
    /// * `outcome` - Outcome of interaction: positive = good, negative = bad
    /// * `was_cooperative` - Whether agent acted cooperatively
    pub fn update_philosophy_from_experience(&mut self, outcome: f32, was_cooperative: bool) {
        if let Some(ref mut phil) = self.philosophical_state {
            phil.update_from_experience(outcome, was_cooperative);
        }
    }

    /// Get philosophy-based risk tolerance
    ///
    /// Returns value in [0.0, 1.0] where 1.0 = high risk tolerance
    #[must_use]
    pub fn philosophy_risk_tolerance(&self) -> f32 {
        self.philosophical_state
            .as_ref()
            .map_or(0.5, crate::philosophy::PhilosophicalState::risk_tolerance) // Neutral default
    }

    /// Get philosophy-based cooperation willingness with a specific partner
    ///
    /// # Arguments
    /// * `partner` - The other agent
    ///
    /// Returns value in [0.0, 1.0] where 1.0 = high cooperation willingness
    #[must_use]
    pub fn philosophy_cooperation_willingness(&self, partner: &Self) -> f32 {
        if let Some(ref phil) = self.philosophical_state {
            let partner_tau_phi = partner
                .philosophical_state
                .as_ref()
                .map_or(0.0, |p| p.worldview.trust_axis);
            phil.cooperation_willingness(partner_tau_phi)
        } else {
            // Fall back to trust computation
            let trust = self.compute_trust();
            (trust / 2.0).clamp(0.0, 1.0)
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get philosophy alignment with another agent
    ///
    /// Returns value in [0.0, 1.0] where 1.0 = complete alignment
    #[must_use]
    pub fn philosophy_alignment(&self, other: &Self) -> f32 {
        match (&self.philosophical_state, &other.philosophical_state) {
            (Some(p1), Some(p2)) => p1.philosophy_alignment(p2),
            _ => 0.5, // Neutral if either has no philosophy
        }
    }

    /// Make game-theoretic decision using philosophy
    ///
    /// This is used during encounters to decide cooperate/defect
    ///
    /// # Arguments
    /// * `game_type` - Type of game being played
    /// * `partner` - The other player
    /// * `partner_last_action` - Partner's last action (if known)
    /// * `rng` - Random number generator for stochastic decisions
    ///
    /// Returns true if agent chooses to cooperate
    pub fn decide_game_action_with_philosophy(
        &self,
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        _game_type: crate::games::GameType,
        partner: &Self,
        partner_last_action: Option<bool>,
        rng: &mut impl rand::Rng,
    ) -> bool {
        // Base cooperation probability on philosophy
        let base_willingness = self.philosophy_cooperation_willingness(partner);

        // Adjust based on partner's reputation/history
        let reputation_adjustment = if let Some(last_action) = partner_last_action {
            if last_action {
                // Partner cooperated last time → more willing
                0.15
            } else {
                // Partner defected last time → less willing
                -0.20
            }
        } else {
            // No history → neutral
            0.0
        };

        // Apply philosophy-based risk tolerance
        let risk_adj = (self.philosophy_risk_tolerance() - 0.5) * 0.1;

        // Final cooperation probability
        let coop_prob = (base_willingness + reputation_adjustment + risk_adj).clamp(0.0, 1.0);

        // Stochastic decision
        rng.gen::<f32>() < coop_prob
    }
}
