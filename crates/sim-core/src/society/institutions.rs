//! Institutional evolution subsystem
//!
//! Per §4311-4338: Beinhocker selection-amplification dynamics
//! Per §6.18: Full Selection → Amplification → Variation cycle

use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::society::norms::CoordinationTracker;
use crate::types::{GroupId, InstitutionId, Tick};

// ============================================================================
// INSTITUTIONAL EVOLUTION
// Per §4311-4338: Beinhocker selection-amplification dynamics
// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Rule types for institutions
/// Per §4315-4318: Types of rules that can emerge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleType {
    /// Resource sharing rules
    ResourceSharing,
    /// Conflict resolution procedures
    ConflictResolution,
    /// Membership criteria
    MembershipCriteria,
    /// Decision-making procedures
    DecisionMaking,
    /// Trade/exchange rules
    TradeRules,
    /// Punishment procedures
    PunishmentProcedures,
}

/// Institution structure
/// Per §4311-4338: Institutions are rules that persist and spread
/// Per §8.2.1 lines 4654-4684: Institutions toggle contract
/// Per §6.18: Selection-amplification dynamics with full evolution tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Institution {
    /// Unique identifier
    pub id: InstitutionId,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Human-readable name for the institution
    pub name: String,

    /// Type of rule this institution encodes
    pub rule_type: RuleType,

    /// Group that owns this institution (None = cross-group)
    pub group_id: Option<GroupId>,

    /// Tick when this institution emerged
    pub created_tick: Tick,

    /// Fitness: coordination success rate [0.0, 1.0]
    /// Per §4320: Fitness = coordination success rate
    pub fitness: f32,

    /// Salience: how prominent this rule is [0.0, 1.0]
    /// Per §4330-4333: Salience decays when not invoked
    pub salience: f32,

    /// Number of times this institution was invoked
    pub invocation_count: u64,

    /// Last tick this institution was invoked
    pub last_invoked_tick: Tick,

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // === Selection-Amplification Fields (§6.18) ===
    /// Current number of members in this institution
    pub member_count: usize,

    /// Total resources accumulated through institutional activities
    pub total_resources_accumulated: f32,

    /// Number of successful cooperations facilitated
    pub successful_cooperations: u32,

    /// Number of failed cooperations
    pub failed_cooperations: u32,

    /// Computed efficiency score: `success_rate` × `resource_efficiency`
    pub efficiency_score: f32,

    /// Mutation rate for this institution's variants
    pub mutation_rate: f32,

    /// Parent institution ID for lineage tracking (None = original)
    pub parent_institution: Option<InstitutionId>,

    /// Governance structure type
    pub governance_type: GovernanceType,

    /// Membership criteria threshold [0.0, 1.0]
    pub membership_criteria: f32,

    /// Resource distribution parameter [0.0, 1.0]
    pub resource_distribution: f32,
}

/// Governance structure types for institutions
/// Per §6.18: Different organizational forms emerge through selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GovernanceType {
    /// Hierarchical: centralized decision making
    Hierarchical,
    /// Egalitarian: distributed decision making
    Egalitarian,
    /// Meritocratic: reputation-weighted decisions
    Meritocratic,
    /// Consensus-based: unanimous agreement required
    Consensus,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Institution mutation types for variation phase
/// Per §6.18: Variation introduces institutional diversity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InstitutionMutation {
    /// Change institution name
    /// new name.
    NameChange {
        /// new name.
        new_name: String,
    },
    /// Modify membership criteria threshold
    /// new criteria.
    MembershipCriteriaChange {
        /// new criteria.
        new_criteria: f32,
    },
    /// Modify resource distribution rules
    /// new distribution.
    ResourceDistributionChange {
        /// new distribution.
        new_distribution: f32,
    },
    /// Change governance structure
    /// new structure.
    GovernanceStructureChange {
        /// new structure.
        new_structure: GovernanceType,
    },
    /// Adjust mutation rate
    /// new rate.
    MutationRateChange {
        /// new rate.
        new_rate: f32,
    },
}

/// Selection criteria for institution evaluation
/// Per §6.18: Multiple selection mechanisms identify high performers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SelectionCriterion {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Efficiency-based: success/failure ratio
    EfficiencyBased,
    /// Resource-based: total resources accumulated
    ResourceBased,
    /// Growth-based: membership growth rate
    GrowthBased,
    /// Combined: weighted combination of all criteria
    Combined,
}

impl Institution {
    /// Create a new institution
    #[must_use]
    pub fn new(
        id: InstitutionId,
        rule_type: RuleType,
        group_id: Option<GroupId>,
        created_tick: Tick,
    ) -> Self {
        Self {
            id,
            name: format!("Institution-{id}"),
            rule_type,
            group_id,
            created_tick,
            fitness: 0.5, // Initial moderate fitness
            salience: 0.5,
            invocation_count: 0,
            last_invoked_tick: created_tick,
            // Selection-amplification fields
            member_count: 0,
            total_resources_accumulated: 0.0,
            successful_cooperations: 0,
            failed_cooperations: 0,
            efficiency_score: 0.5,
            mutation_rate: 0.05, // 5% base mutation rate
            parent_institution: None,
            governance_type: GovernanceType::Egalitarian,
            membership_criteria: 0.5,
            resource_distribution: 0.5,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create a new institution with a custom name
    #[must_use]
    pub const fn new_with_name(
        id: InstitutionId,
        name: String,
        rule_type: RuleType,
        group_id: Option<GroupId>,
        created_tick: Tick,
    ) -> Self {
        Self {
            id,
            name,
            rule_type,
            group_id,
            created_tick,
            fitness: 0.5,
            salience: 0.5,
            invocation_count: 0,
            last_invoked_tick: created_tick,
            member_count: 0,
            total_resources_accumulated: 0.0,
            successful_cooperations: 0,
            failed_cooperations: 0,
            efficiency_score: 0.5,
            mutation_rate: 0.05,
            parent_institution: None,
            governance_type: GovernanceType::Egalitarian,
            membership_criteria: 0.5,
            resource_distribution: 0.5,
        }
    }

    /// Create a variant of an existing institution (for amplification phase)
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Per §6.18: High-performing institutions spawn variants with mutation
    #[must_use]
    pub fn create_variant(
        &self,
        new_id: InstitutionId,
        current_tick: Tick,
        mutation: Option<InstitutionMutation>,
    ) -> Self {
        let mut variant = Self {
            id: new_id,
            name: format!("{}-Variant-{new_id}", self.name),
            rule_type: self.rule_type,
            group_id: self.group_id,
            created_tick: current_tick,
            fitness: self.fitness * 0.7, // Inherited but reduced
            salience: self.salience * 0.8,
            invocation_count: 0,
            last_invoked_tick: current_tick,
            member_count: 0,
            total_resources_accumulated: 0.0,
            successful_cooperations: 0,
            failed_cooperations: 0,
            efficiency_score: self.efficiency_score,
            mutation_rate: self.mutation_rate,
            parent_institution: Some(self.id),
            governance_type: self.governance_type,
            membership_criteria: self.membership_criteria,
            resource_distribution: self.resource_distribution,
        };

        // Apply mutation if specified
        if let Some(m) = mutation {
            variant.apply_mutation(m);
        }

        variant
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Apply a mutation to this institution
    /// Per §6.18: Variation introduces diversity
    pub fn apply_mutation(&mut self, mutation: InstitutionMutation) {
        match mutation {
            InstitutionMutation::NameChange { new_name } => {
                self.name = new_name;
            }
            InstitutionMutation::MembershipCriteriaChange { new_criteria } => {
                self.membership_criteria = new_criteria.clamp(0.0, 1.0);
            }
            InstitutionMutation::ResourceDistributionChange { new_distribution } => {
                self.resource_distribution = new_distribution.clamp(0.0, 1.0);
            }
            InstitutionMutation::GovernanceStructureChange { new_structure } => {
                self.governance_type = new_structure;
            }
            InstitutionMutation::MutationRateChange { new_rate } => {
                self.mutation_rate = new_rate.clamp(0.0, 0.5);
            }
        }
    }

    /// Compute efficiency score from cooperation outcomes
    /// Per §6.18: Efficiency-based selection
    pub fn compute_efficiency(&mut self) {
        let total = self.successful_cooperations + self.failed_cooperations;
        if total > 0 {
            let success_rate = self.successful_cooperations as f32 / total as f32;
            let resource_efficiency = if self.total_resources_accumulated > 0.0 {
                (self.total_resources_accumulated / (total as f32 * 10.0)).min(1.0)
            } else {
                0.5
            };
            self.efficiency_score = success_rate * resource_efficiency;
        } else {
            self.efficiency_score = 0.5; // Neutral default
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Record a cooperation outcome
    pub fn record_cooperation(&mut self, success: bool, resources_exchanged: f32) {
        if success {
            self.successful_cooperations += 1;
            self.total_resources_accumulated += resources_exchanged;
        } else {
            self.failed_cooperations += 1;
        }
        self.compute_efficiency();
    }

    /// Update member count
    pub fn update_member_count(&mut self, count: usize) {
        self.member_count = count;
    }

    /// Decay institution fitness and salience over time
    /// Per §4330-4333: Salience decays when not invoked
    pub fn decay(&mut self, current_tick: Tick, decay_rate: f32) {
        let ticks_since_invoked = current_tick.saturating_sub(self.last_invoked_tick);
        let time_decay = 1.0 - (ticks_since_invoked as f32 * decay_rate * 0.001);
        self.salience = (self.salience * time_decay).max(0.0);

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Per §4325-4328: Low-fitness institutions decay faster
        if self.fitness < 0.30 {
            self.salience = (self.salience - decay_rate * 0.5).max(0.0);
        }
    }

    /// Invoke this institution (record usage)
    pub fn invoke(&mut self, current_tick: Tick, success: bool) {
        self.invocation_count += 1;
        self.last_invoked_tick = current_tick;

        // Update fitness based on success
        // Exponential moving average
        let alpha = 0.1; // Learning rate
        let success_val = if success { 1.0 } else { 0.0 };
        self.fitness = self.fitness * (1.0 - alpha) + success_val * alpha;

        // Successful invocation increases salience
        if success {
            self.salience = (self.salience + 0.05).min(1.0);
        }
    }

    /// Check if institution should be removed
    /// Per §4333: Removal when salience < 0.10
    #[must_use]
    pub fn should_remove(&self) -> bool {
        self.salience < 0.10
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get selection score based on criterion
    /// Per §6.18: Selection mechanisms identify high performers
    #[must_use]
    pub fn selection_score(&self, criterion: SelectionCriterion) -> f32 {
        match criterion {
            SelectionCriterion::EfficiencyBased => self.efficiency_score,
            SelectionCriterion::ResourceBased => {
                // Normalize resources (assume max 10,000 for scaling)
                (self.total_resources_accumulated / 10000.0).min(1.0)
            }
            SelectionCriterion::GrowthBased => {
                // Normalize member count (assume max 100 for scaling)
                (self.member_count as f32 / 100.0).min(1.0)
            }
            SelectionCriterion::Combined => {
                // Weighted combination of all criteria
                let efficiency_weight = 0.4;
                let resource_weight = 0.3;
                let growth_weight = 0.3;

                efficiency_weight * self.efficiency_score
                    + resource_weight * (self.total_resources_accumulated / 10000.0).min(1.0)
                    + growth_weight * (self.member_count as f32 / 100.0).min(1.0)
            }
        }
    }
}

/// Selection-Amplification Engine for institutional evolution
/// Per §6.18 Beinhocker: Selection → Amplification → Variation cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionAmplificationEngine {
    /// Selection phase: Pressure to identify high performers [0.0, 1.0]
    pub selection_pressure: f32,
    /// Amplification phase: Rate of replication for successful institutions
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub amplification_rate: f32,
    /// Variation phase: Base mutation rate for new variants
    pub mutation_rate: f32,
    /// Current selection criterion
    pub selection_criterion: SelectionCriterion,
    /// Fitness threshold for selection (institutions above this get amplified)
    pub selection_threshold: f32,
    /// Maximum number of institutions per niche (prevents overpopulation)
    pub max_institutions_per_niche: usize,
}

impl Default for SelectionAmplificationEngine {
    fn default() -> Self {
        Self {
            selection_pressure: 0.3,
            amplification_rate: 0.2,
            mutation_rate: 0.05,
            selection_criterion: SelectionCriterion::Combined,
            selection_threshold: 0.6,
            max_institutions_per_niche: 10,
        }
    }
}

impl SelectionAmplificationEngine {
    /// Create new engine with custom parameters
    #[must_use]
    pub fn new(
        selection_pressure: f32,
        amplification_rate: f32,
        mutation_rate: f32,
        selection_criterion: SelectionCriterion,
    ) -> Self {
        Self {
            selection_pressure: selection_pressure.clamp(0.0, 1.0),
            amplification_rate: amplification_rate.clamp(0.0, 1.0),
            mutation_rate: mutation_rate.clamp(0.0, 1.0),
            selection_criterion,
            selection_threshold: 0.6,
            max_institutions_per_niche: 10,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Run full selection-amplification-variation cycle
    /// Per §6.18: Three-phase evolutionary dynamics
    pub fn evolve(
        &self,
        memory: &mut InstitutionalMemory,
        current_tick: Tick,
        rng: &mut StdRng,
    ) -> EvolutionCycleResult {
        let mut result = EvolutionCycleResult::default();

        // Phase 1: Selection - identify high-performing institutions
        let selected = self.select_institutions(&memory.institutions);
        result.selected_count = selected.len();

        // Phase 2: Amplification - replicate successful institutions
        // Collect parent institutions first to avoid borrow issues
        let parent_institutions: Vec<Institution> = selected
            .iter()
            .filter_map(|&id| memory.get(id).cloned())
            .collect();

        let new_institutions =
            self.create_variants(&parent_institutions, memory.next_id, current_tick, rng);
        let new_ids: Vec<InstitutionId> = new_institutions.iter().map(|i| i.id).collect();
        result.new_institutions = new_institutions.len();

        // Add new institutions to memory
        memory.institutions.extend(new_institutions);
        memory.next_id += result.new_institutions as InstitutionId;

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Phase 3: Variation - apply mutations to new variants
        let mutated = self.apply_variation_to_memory(memory, &new_ids, rng);
        result.mutations_applied = mutated;

        result
    }

    /// Selection phase: identify high-performing institutions
    /// Per §6.18: Institutions with higher fitness are selected
    pub(crate) fn select_institutions(&self, institutions: &[Institution]) -> Vec<InstitutionId> {
        institutions
            .iter()
            .filter(|inst| {
                let score = inst.selection_score(self.selection_criterion);
                score >= self.selection_threshold
            })
            .map(|inst| inst.id)
            .collect()
    }

    /// Amplification phase: create variants of selected institutions
    /// Per §6.18: High-performing institutions spawn variants
    pub(crate) fn create_variants(
        &self,
        parent_institutions: &[Institution],
        starting_id: InstitutionId,
        current_tick: Tick,
        rng: &mut StdRng,
    ) -> Vec<Institution> {
        let mut new_institutions = Vec::new();
        let mut next_id = starting_id;

        for parent in parent_institutions {
            // Check amplification probability
            if rng.gen::<f32>() > self.amplification_rate {
                continue;
            }

            // Note: niche capacity check is skipped here as it requires access to the full memory
            // This is handled at a higher level if needed

            // TAG: surface=simulation owner=research-team rule=SIM-001
            // Create variant with mutation
            let mutation = self.generate_mutation(rng);
            let variant = parent.create_variant(next_id, current_tick, mutation);
            new_institutions.push(variant);
            next_id += 1;
        }

        new_institutions
    }

    /// Variation phase: generate random mutation
    /// Per §6.18: Variation introduces institutional diversity
    fn generate_mutation(&self, rng: &mut StdRng) -> Option<InstitutionMutation> {
        if rng.gen::<f32>() > self.mutation_rate {
            return None;
        }

        let mutation_type = rng.gen_range(0..5);
        match mutation_type {
            0 => Some(InstitutionMutation::NameChange {
                new_name: format!("Mutated-{val}", val = rng.gen::<u32>()),
            }),
            1 => Some(InstitutionMutation::MembershipCriteriaChange {
                new_criteria: rng.gen::<f32>(),
            }),
            2 => Some(InstitutionMutation::ResourceDistributionChange {
                // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                new_distribution: rng.gen::<f32>(),
            }),
            3 => Some(InstitutionMutation::GovernanceStructureChange {
                new_structure: match rng.gen_range(0..4) {
                    0 => GovernanceType::Hierarchical,
                    1 => GovernanceType::Egalitarian,
                    2 => GovernanceType::Meritocratic,
                    _ => GovernanceType::Consensus,
                },
            }),
            _ => Some(InstitutionMutation::MutationRateChange {
                new_rate: (self.mutation_rate * (0.8 + rng.gen::<f32>() * 0.4)).clamp(0.0, 0.5),
            }),
        }
    }

    /// Apply variation to new institutions in memory
    fn apply_variation_to_memory(
        &self,
        memory: &mut InstitutionalMemory,
        new_ids: &[InstitutionId],
        rng: &mut StdRng,
    ) -> usize {
        let mut count = 0;

        for &id in new_ids {
            if let Some(mutation) = self.generate_mutation(rng) {
                if let Some(inst) = memory.get_mut(id) {
                    inst.apply_mutation(mutation);
                    count += 1;
                }
            }
        }

        count
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Result of one evolution cycle
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvolutionCycleResult {
    /// Number of institutions selected
    pub selected_count: usize,
    /// Number of new institutions created
    pub new_institutions: usize,
    /// Number of mutations applied
    pub mutations_applied: usize,
    /// Number of institutions that went extinct
    pub extinct_count: usize,
}

/// Institutional memory for a group
/// Per §4335-4338: Rules persist and spread via selection-amplification
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstitutionalMemory {
    /// Active institutions
    pub institutions: Vec<Institution>,
    /// Next institution ID
    pub next_id: InstitutionId,
    /// Selection-amplification engine
    pub evolution_engine: SelectionAmplificationEngine,
    /// History of evolution cycles
    pub evolution_history: Vec<EvolutionCycleResult>,
    /// Lineage tracking: parent -> children
    pub lineage: std::collections::HashMap<InstitutionId, Vec<InstitutionId>>,
}

impl InstitutionalMemory {
    /// Add a new institution
    pub fn add(
        &mut self,
        rule_type: RuleType,
        group_id: Option<GroupId>,
        current_tick: Tick,
    ) -> InstitutionId {
        let id = self.next_id;
        self.next_id += 1;
        self.institutions
            .push(Institution::new(id, rule_type, group_id, current_tick));
        id
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Add a new institution with name
    #[must_use]
    pub fn add_with_name(
        &mut self,
        name: String,
        rule_type: RuleType,
        group_id: Option<GroupId>,
        current_tick: Tick,
    ) -> InstitutionId {
        let id = self.next_id;
        self.next_id += 1;
        self.institutions.push(Institution::new_with_name(
            id,
            name,
            rule_type,
            group_id,
            current_tick,
        ));
        id
    }

    /// Get institution by ID
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    #[must_use]
    pub fn get(&self, id: InstitutionId) -> Option<&Institution> {
        self.institutions.iter().find(|i| i.id == id)
    }

    /// Get mutable institution by ID
    #[must_use]
    pub fn get_mut(&mut self, id: InstitutionId) -> Option<&mut Institution> {
        self.institutions.iter_mut().find(|i| i.id == id)
    }

    /// Remove dead institutions
    /// Per §4333: Removal when salience < 0.10
    pub fn prune(&mut self) {
        self.institutions.retain(|i| !i.should_remove());
    }

    /// Get institutions for a specific group
    #[must_use]
    pub fn for_group(&self, group_id: Option<GroupId>) -> Vec<&Institution> {
        self.institutions
            .iter()
            .filter(|i| i.group_id == group_id || i.group_id.is_none())
            .collect()
    }

    /// Count institutions by rule type (for diversity metric)
    #[must_use]
    pub fn count_by_rule_type(&self) -> std::collections::HashMap<RuleType, usize> {
        let mut counts = std::collections::HashMap::new();
        for inst in &self.institutions {
            *counts.entry(inst.rule_type).or_insert(0) += 1;
        }
        counts
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Count institutions by governance type
    #[must_use]
    pub fn count_by_governance(&self) -> std::collections::HashMap<GovernanceType, usize> {
        let mut counts = std::collections::HashMap::new();
        for inst in &self.institutions {
            *counts.entry(inst.governance_type).or_insert(0) += 1;
        }
        counts
    }

    /// Update lineage tracking
    pub fn record_lineage(&mut self, parent_id: InstitutionId, child_id: InstitutionId) {
        if let Some(child) = self.institutions.iter_mut().find(|i| i.id == child_id) {
            child.parent_institution = Some(parent_id);
        }
        self.lineage.entry(parent_id).or_default().push(child_id);
    }

    /// Get all descendants of an institution
    #[must_use]
    pub fn get_descendants(&self, parent_id: InstitutionId) -> Vec<InstitutionId> {
        self.lineage.get(&parent_id).cloned().unwrap_or_default()
    }
}

/// Check if new institution should emerge from coordination failures
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
/// Per §4315-4317: Trigger when coordination failure rate > 0.40 for 100 ticks
pub fn check_institution_emergence(
    tracker: &CoordinationTracker,
    group_id: Option<GroupId>,
    current_tick: Tick,
    memory: &mut InstitutionalMemory,
    rng: &mut StdRng,
) -> Option<InstitutionId> {
    const FAILURE_THRESHOLD: f32 = 0.40;
    const MIN_WINDOW_TICKS: Tick = 100;
    const MIN_ATTEMPTS: u32 = 20;

    // Need enough observations
    if tracker.attempts < MIN_ATTEMPTS {
        return None;
    }

    // Check if window is long enough
    if current_tick.saturating_sub(tracker.window_start) < MIN_WINDOW_TICKS {
        return None;
    }

    // High failure rate triggers institution proposal
    if tracker.failure_rate() > FAILURE_THRESHOLD {
        // Per §4317: Random rule mutation OR copy from successful neighbor
        let rule_types = [
            RuleType::ResourceSharing,
            RuleType::ConflictResolution,
            RuleType::TradeRules,
            RuleType::DecisionMaking,
        ];
        let rule_type = rule_types[rng.gen_range(0..rule_types.len())];

        // Create new institution
        let id = memory.add(rule_type, group_id, current_tick);
        return Some(id);
    }
    None
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Evolve institutions using Beinhocker selection-amplification
/// Per §4320-4338: Selection and amplification dynamics
/// Per §6.18: Full Selection → Amplification → Variation cycle
pub fn evolve_institutions(
    memory: &mut InstitutionalMemory,
    current_tick: Tick,
    decay_rate: f32,
    rng: &mut StdRng,
) -> EvolutionCycleResult {
    // Phase 0: Apply time-based decay to all institutions
    for inst in &mut memory.institutions {
        inst.decay(current_tick, decay_rate);
    }

    // Remove dead institutions (extinction)
    let pre_count = memory.institutions.len();
    memory.prune();
    let extinct_count = pre_count - memory.institutions.len();

    // Phase 1-3: Run selection-amplification-variation cycle
    // Extract the engine temporarily to avoid borrow issues
    let engine = std::mem::take(&mut memory.evolution_engine);
    let mut result = engine.evolve(memory, current_tick, rng);
    result.extinct_count = extinct_count;
    memory.evolution_engine = engine;
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    // Collect lineage info first to avoid borrow issues
    let lineage_entries: Vec<(InstitutionId, InstitutionId)> = memory
        .institutions
        .iter()
        .filter_map(|inst| {
            inst.parent_institution
                .map(|parent_id| (parent_id, inst.id))
        })
        .collect();

    // Record lineage
    for (parent_id, child_id) in lineage_entries {
        memory.record_lineage(parent_id, child_id);
    }

    // Store in history
    memory.evolution_history.push(result.clone());

    result
}

/// Copy successful institution from another group (imitation)
/// Per §4337-4338: Successful rules copied by neighboring groups (prestige bias)
pub fn copy_institution(
    source: &Institution,
    target_group_id: Option<GroupId>,
    current_tick: Tick,
    memory: &mut InstitutionalMemory,
) -> InstitutionId {
    let id = memory.next_id;
    memory.next_id += 1;

    let mut copied = Institution::new(id, source.rule_type, target_group_id, current_tick);

    // Inherit some fitness from source (but not all - adaptation needed)
    // Per §4338: Rules in high-τ groups spread faster (prestige bias)
    copied.fitness = source.fitness * 0.7;
    copied.salience = source.salience * 0.8;

    memory.institutions.push(copied);
    id
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Compute institutional evolution metrics
/// Per §8.3.12: Institutional Evolution Metrics
/// Per §6.18: Extended metrics for selection-amplification dynamics
#[must_use]
pub fn compute_institutional_metrics(
    memory: &InstitutionalMemory,
    current_tick: Tick,
) -> InstitutionalMetrics {
    let active_count = memory.institutions.len();

    if active_count == 0 {
        return InstitutionalMetrics::default();
    }

    let ages: Vec<u64> = memory
        .institutions
        .iter()
        .map(|i| current_tick.saturating_sub(i.created_tick))
        .collect();

    let mean_age = ages.iter().sum::<u64>() as f32 / active_count as f32;
    let mean_fitness =
        memory.institutions.iter().map(|i| i.fitness).sum::<f32>() / active_count as f32;
    let mean_salience =
        memory.institutions.iter().map(|i| i.salience).sum::<f32>() / active_count as f32;

    // Compute diversity metrics
    let rule_type_counts = memory.count_by_rule_type();
    let governance_counts = memory.count_by_governance();

    // Institution diversity: count of unique rule types
    let institution_diversity = rule_type_counts.len() as u32;
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    // Institution concentration: Herfindahl index (market share concentration)
    let concentration = if active_count > 0 {
        rule_type_counts
            .values()
            .map(|&count| {
                let share = count as f32 / active_count as f32;
                share * share
            })
            .sum()
    } else {
        0.0
    };

    // Compute mean efficiency score
    let mean_efficiency = memory
        .institutions
        .iter()
        .map(|i| i.efficiency_score)
        .sum::<f32>()
        / active_count as f32;

    // Compute total members across all institutions
    let total_members: usize = memory.institutions.iter().map(|i| i.member_count).sum();
    let mean_member_count = total_members as f32 / active_count as f32;

    // Count root (original) vs variant institutions
    let root_count = memory
        .institutions
        .iter()
        .filter(|i| i.parent_institution.is_none())
        .count();
    let variant_count = active_count - root_count;

    InstitutionalMetrics {
        active_count,
        mean_age,
        mean_fitness,
        mean_salience,
        // Extended metrics per §6.18
        institution_diversity,
        institution_concentration: concentration,
        mean_efficiency_score: mean_efficiency,
        mean_member_count,
        total_members,
        root_institution_count: root_count,
        variant_count,
        rule_type_distribution: rule_type_counts,
        governance_distribution: governance_counts,
        // TAG: surface=simulation owner=research-team rule=SIM-001
    }
}

/// Compute evolution rate metrics from history
/// Per §6.18: Innovation rate and extinction rate tracking
#[must_use]
pub fn compute_evolution_rates(
    memory: &InstitutionalMemory,
    window_size: usize,
) -> EvolutionRateMetrics {
    let recent_history: Vec<_> = memory
        .evolution_history
        .iter()
        .rev()
        .take(window_size)
        .collect();

    if recent_history.is_empty() {
        return EvolutionRateMetrics::default();
    }

    let total_new: usize = recent_history.iter().map(|r| r.new_institutions).sum();
    let total_extinct: usize = recent_history.iter().map(|r| r.extinct_count).sum();
    let total_mutations: usize = recent_history.iter().map(|r| r.mutations_applied).sum();

    let n = recent_history.len() as f32;

    EvolutionRateMetrics {
        innovation_rate: total_new as f32 / n, // New institutions per tick
        extinction_rate: total_extinct as f32 / n, // Dying institutions per tick
        mutation_rate: total_mutations as f32 / n, // Mutations per tick
        window_size: recent_history.len(),
    }
}

/// Metrics for institutional evolution
/// Per §8.3.12: Institutional Evolution Metrics
#[derive(Debug, Clone, Default)]
pub struct InstitutionalMetrics {
    /// Number of active institutions
    pub active_count: usize,
    /// Mean age of institutions in ticks
    pub mean_age: f32,
    /// Mean fitness of institutions
    pub mean_fitness: f32,
    /// Mean salience of institutions
    pub mean_salience: f32,
    // === Extended metrics per §6.18 ===
    /// Institution diversity: count of unique rule types
    pub institution_diversity: u32,
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Institution concentration: Herfindahl index [0.0, 1.0]
    pub institution_concentration: f32,
    /// Mean efficiency score across institutions
    pub mean_efficiency_score: f32,
    /// Mean member count per institution
    pub mean_member_count: f32,
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Total members across all institutions
    pub total_members: usize,
    /// Number of original (root) institutions
    pub root_institution_count: usize,
    /// Number of variant institutions
    pub variant_count: usize,
    /// Distribution of institutions by rule type
    pub rule_type_distribution: std::collections::HashMap<RuleType, usize>,
    /// Distribution of institutions by governance type
    pub governance_distribution: std::collections::HashMap<GovernanceType, usize>,
}

/// Evolution rate metrics per §6.18
#[derive(Debug, Clone, Default)]
pub struct EvolutionRateMetrics {
    /// Innovation rate: new institutions per tick
    pub innovation_rate: f32,
    /// Extinction rate: dying institutions per tick
    pub extinction_rate: f32,
    /// Mutation rate: mutations per tick
    pub mutation_rate: f32,
    /// Size of the analysis window
    pub window_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_institution_new() {
        let inst = Institution::new(1, RuleType::ResourceSharing, Some(10), 0);
        assert_eq!(inst.id, 1);
        assert_eq!(inst.rule_type, RuleType::ResourceSharing);
        assert_eq!(inst.group_id, Some(10));
        assert_eq!(inst.fitness, 0.5);
        assert_eq!(inst.salience, 0.5);
        assert_eq!(inst.invocation_count, 0);
        assert_eq!(inst.member_count, 0);
    }

    #[test]
    fn test_institution_new_with_name() {
        let inst =
            Institution::new_with_name(1, "TestInst".to_string(), RuleType::TradeRules, None, 0);
        assert_eq!(inst.name, "TestInst");
        assert_eq!(inst.rule_type, RuleType::TradeRules);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_institution_create_variant() {
        let parent = Institution::new(1, RuleType::ResourceSharing, Some(10), 0);
        let variant = parent.create_variant(2, 10, None);
        assert_eq!(variant.id, 2);
        assert_eq!(variant.parent_institution, Some(1));
        assert_eq!(variant.rule_type, RuleType::ResourceSharing);
        assert!(variant.fitness < parent.fitness);
    }

    #[test]
    fn test_institution_apply_mutation_name_change() {
        let mut inst = Institution::new(1, RuleType::ResourceSharing, None, 0);
        inst.apply_mutation(InstitutionMutation::NameChange {
            new_name: "Renamed".to_string(),
        });
        assert_eq!(inst.name, "Renamed");
    }

    #[test]
    fn test_institution_apply_mutation_membership_criteria_clamped() {
        let mut inst = Institution::new(1, RuleType::ResourceSharing, None, 0);
        inst.apply_mutation(InstitutionMutation::MembershipCriteriaChange { new_criteria: 1.5 });
        assert_eq!(inst.membership_criteria, 1.0);
        inst.apply_mutation(InstitutionMutation::MembershipCriteriaChange { new_criteria: -0.5 });
        assert_eq!(inst.membership_criteria, 0.0);
    }

    #[test]
    fn test_institution_apply_mutation_governance_change() {
        let mut inst = Institution::new(1, RuleType::ResourceSharing, None, 0);
        inst.apply_mutation(InstitutionMutation::GovernanceStructureChange {
            new_structure: GovernanceType::Hierarchical,
        });
        assert_eq!(inst.governance_type, GovernanceType::Hierarchical);
    }

    #[test]
    fn test_institution_record_cooperation_success() {
        let mut inst = Institution::new(1, RuleType::TradeRules, None, 0);
        inst.record_cooperation(true, 100.0);
        assert_eq!(inst.successful_cooperations, 1);
        assert_eq!(inst.total_resources_accumulated, 100.0);
    }

    #[test]
    fn test_institution_record_cooperation_failure() {
        let mut inst = Institution::new(1, RuleType::TradeRules, None, 0);
        inst.record_cooperation(false, 0.0);
        assert_eq!(inst.failed_cooperations, 1);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        assert_eq!(inst.successful_cooperations, 0);
    }

    #[test]
    fn test_institution_compute_efficiency() {
        let mut inst = Institution::new(1, RuleType::TradeRules, None, 0);
        inst.record_cooperation(true, 100.0);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        inst.record_cooperation(true, 100.0);
        inst.record_cooperation(false, 0.0);
        // success_rate = 2/3, resource_efficiency = 200 / (3*10) = 6.67 -> min(1.0, 6.67) = 1.0
        // efficiency = 0.667 * 1.0 = 0.667
        assert!(inst.efficiency_score > 0.0);
    }

    #[test]
    fn test_institution_update_member_count() {
        let mut inst = Institution::new(1, RuleType::ResourceSharing, None, 0);
        inst.update_member_count(42);
        assert_eq!(inst.member_count, 42);
    }

    #[test]
    fn test_institution_should_remove() {
        let mut inst = Institution::new(1, RuleType::ResourceSharing, None, 0);
        assert!(!inst.should_remove());
        inst.salience = 0.05;
        assert!(inst.should_remove());
    }

    #[test]
    fn test_institution_invoke_success() {
        let mut inst = Institution::new(1, RuleType::ResourceSharing, None, 0);
        inst.invoke(10, true);
        assert_eq!(inst.invocation_count, 1);
        assert_eq!(inst.last_invoked_tick, 10);
        assert!(inst.fitness > 0.5);
        assert!(inst.salience > 0.5);
    }

    #[test]
    fn test_institution_invoke_failure() {
        let mut inst = Institution::new(1, RuleType::ResourceSharing, None, 0);
        let prev_fitness = inst.fitness;
        inst.invoke(10, false);
        assert!(inst.fitness < prev_fitness);
    }

    #[test]
    fn test_institution_decay() {
        let mut inst = Institution::new(1, RuleType::ResourceSharing, None, 0);
        inst.last_invoked_tick = 0;
        inst.salience = 0.5;
        inst.decay(1000, 0.1);
        assert!(inst.salience < 0.5);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_institution_selection_score_efficiency() {
        let mut inst = Institution::new(1, RuleType::ResourceSharing, None, 0);
        inst.efficiency_score = 0.8;
        let score = inst.selection_score(SelectionCriterion::EfficiencyBased);
        assert!((score - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_institution_selection_score_combined() {
        let mut inst = Institution::new(1, RuleType::ResourceSharing, None, 0);
        inst.efficiency_score = 1.0;
        inst.total_resources_accumulated = 10000.0;
        inst.member_count = 100;
        let score = inst.selection_score(SelectionCriterion::Combined);
        assert!(score > 0.0);
    }

    #[test]
    fn test_institutional_memory_add_and_get() {
        let mut memory = InstitutionalMemory::default();
        let id = memory.add(RuleType::ConflictResolution, Some(5), 0);
        assert_eq!(id, 0);
        let inst = memory.get(id).unwrap();
        assert_eq!(inst.rule_type, RuleType::ConflictResolution);
        assert_eq!(inst.group_id, Some(5));
    }

    #[test]
    fn test_institutional_memory_get_mut() {
        let mut memory = InstitutionalMemory::default();
        let id = memory.add(RuleType::ResourceSharing, None, 0);
        {
            let inst = memory.get_mut(id).unwrap();
            inst.salience = 0.05;
        }
        assert!(memory.get(id).unwrap().should_remove());
    }

    #[test]
    fn test_institutional_memory_prune() {
        let mut memory = InstitutionalMemory::default();
        let id = memory.add(RuleType::ResourceSharing, None, 0);
        {
            let inst = memory.get_mut(id).unwrap();
            inst.salience = 0.05;
        }
        memory.prune();
        assert!(memory.get(id).is_none());
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    #[test]
    fn test_institutional_memory_for_group() {
        let mut memory = InstitutionalMemory::default();
        memory.add(RuleType::ResourceSharing, Some(5), 0);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        memory.add(RuleType::TradeRules, Some(5), 0);
        memory.add(RuleType::ConflictResolution, Some(10), 0);
        let group5 = memory.for_group(Some(5));
        assert_eq!(group5.len(), 2);
    }

    #[test]
    fn test_institutional_memory_count_by_rule_type() {
        let mut memory = InstitutionalMemory::default();
        memory.add(RuleType::ResourceSharing, None, 0);
        memory.add(RuleType::ResourceSharing, None, 0);
        memory.add(RuleType::TradeRules, None, 0);
        let counts = memory.count_by_rule_type();
        assert_eq!(
            counts.get(&RuleType::ResourceSharing).copied().unwrap_or(0),
            2
        );
        assert_eq!(counts.get(&RuleType::TradeRules).copied().unwrap_or(0), 1);
    }

    #[test]
    fn test_institutional_memory_lineage() {
        let mut memory = InstitutionalMemory::default();
        memory.record_lineage(1, 2);
        memory.record_lineage(1, 3);
        let descendants = memory.get_descendants(1);
        assert_eq!(descendants.len(), 2);
        assert!(descendants.contains(&2));
        assert!(descendants.contains(&3));
    }

    #[test]
    fn test_selection_amplification_engine_new_clamped() {
        let engine = SelectionAmplificationEngine::new(1.5, 1.5, 1.5, SelectionCriterion::Combined);
        assert_eq!(engine.selection_pressure, 1.0);
        assert_eq!(engine.amplification_rate, 1.0);
        assert_eq!(engine.mutation_rate, 1.0);
    }

    #[test]
    fn test_selection_amplification_engine_select_institutions() {
        let engine =
            SelectionAmplificationEngine::new(0.3, 0.2, 0.05, SelectionCriterion::EfficiencyBased);
        let institutions = vec![
            Institution::new(1, RuleType::ResourceSharing, None, 0),
            Institution::new(2, RuleType::TradeRules, None, 0),
        ];
        // Default efficiency_score is 0.5, threshold is 0.6, so none selected
        let selected = engine.select_institutions(&institutions);
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(selected.is_empty());
    }

    #[test]
    fn test_copy_institution() {
        let mut memory = InstitutionalMemory::default();
        let source = Institution::new(1, RuleType::ResourceSharing, None, 0);
        let id = copy_institution(&source, Some(10), 5, &mut memory);
        let copied = memory.get(id).unwrap();
        assert_eq!(copied.rule_type, RuleType::ResourceSharing);
        assert_eq!(copied.group_id, Some(10));
        assert_eq!(copied.created_tick, 5);
        assert!(copied.fitness < source.fitness);
    }

    #[test]
    fn test_compute_institutional_metrics_empty() {
        let memory = InstitutionalMemory::default();
        let metrics = compute_institutional_metrics(&memory, 100);
        assert_eq!(metrics.active_count, 0);
        assert_eq!(metrics.mean_age, 0.0);
    }

    #[test]
    fn test_compute_institutional_metrics() {
        let mut memory = InstitutionalMemory::default();
        memory.add(RuleType::ResourceSharing, None, 0);
        memory.add(RuleType::TradeRules, None, 0);
        let metrics = compute_institutional_metrics(&memory, 100);
        assert_eq!(metrics.active_count, 2);
        assert!(metrics.mean_age > 0.0);
        assert_eq!(metrics.institution_diversity, 2);
    }

    #[test]
    fn test_compute_evolution_rates_empty() {
        let memory = InstitutionalMemory::default();
        let rates = compute_evolution_rates(&memory, 10);
        assert_eq!(rates.innovation_rate, 0.0);
        assert_eq!(rates.extinction_rate, 0.0);
        assert_eq!(rates.mutation_rate, 0.0);
    }

    #[test]
    fn test_compute_evolution_rates_with_history() {
        let mut memory = InstitutionalMemory::default();
        memory.evolution_history.push(EvolutionCycleResult {
            selected_count: 2,
            new_institutions: 3,
            mutations_applied: 1,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            extinct_count: 1,
        });
        let rates = compute_evolution_rates(&memory, 10);
        assert_eq!(rates.innovation_rate, 3.0);
        assert_eq!(rates.extinction_rate, 1.0);
        assert_eq!(rates.mutation_rate, 1.0);
    }
}
