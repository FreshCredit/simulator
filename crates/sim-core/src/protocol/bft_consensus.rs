use crate::types::Tick;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// BYZANTINE FAULT TOLERANCE (BFT) SIMULATION
// ============================================================================

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Byzantine node behavior type
/// Represents different types of faulty/malicious nodes in a distributed system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ByzantineBehavior {
    /// Honest node - follows protocol correctly
    Honest,
    /// Crash fault - node stops responding
    Crash,
    /// Omission fault - node occasionally drops messages
    Omission,
    /// Timing fault - node sends messages with delays
    Timing,
    /// Byzantine fault - node sends arbitrary/malicious data
    Malicious,
    /// Equivocation - node sends conflicting messages to different peers
    Equivocation,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl ByzantineBehavior {
    /// Get probability of message delivery for this behavior
    #[must_use]
    pub const fn delivery_probability(&self) -> f32 {
        match self {
            Self::Honest => 1.0,
            Self::Crash => 0.0,
            Self::Omission => 0.5,     // 50% delivery
            Self::Timing => 1.0,       // Delivers but late
            Self::Malicious => 0.8,    // May deliver corrupt data
            Self::Equivocation => 1.0, // Delivers conflicting data
        }
    }

    /// Get probability of sending correct data
    #[must_use]
    pub const fn correctness_probability(&self) -> f32 {
        match self {
            Self::Honest => 1.0,
            Self::Crash => 0.0,
            Self::Omission => 1.0, // Correct but may not arrive
            Self::Timing => 1.0,
            Self::Malicious => 0.0,    // Arbitrary data
            Self::Equivocation => 0.5, // Conflicting
        }
    }
}

/// Consensus message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusMessage {
    /// Sender node ID
    pub sender: u32,
    /// Message content (hash)
    pub content: u64,
    /// Timestamp
    pub timestamp: Tick,
    /// Message type
    pub msg_type: MessageType,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Message type in consensus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Propose a value
    Propose,
    /// Pre-vote on a proposal
    Prevote,
    /// Pre-commit to a value
    Precommit,
    /// Commit a value
    Commit,
}

/// Simulated validator node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorNode {
    /// Node ID
    pub id: u32,
    /// Node behavior type
    pub behavior: ByzantineBehavior,
    /// Current view/round number
    pub current_view: u32,
    /// Messages received
    pub received_messages: Vec<ConsensusMessage>,
    /// Messages sent
    pub sent_messages: Vec<ConsensusMessage>,
    /// Whether node is currently online
    pub online: bool,
}
// TAG: surface=simulation owner=platform-team rule=GENERAL-001

impl ValidatorNode {
    /// Create new honest validator
    #[must_use]
    pub const fn new_honest(id: u32) -> Self {
        Self {
            id,
            behavior: ByzantineBehavior::Honest,
            current_view: 0,
            received_messages: Vec::new(),
            sent_messages: Vec::new(),
            online: true,
        }
    }

    /// Create node with specific behavior
    #[must_use]
    pub const fn with_behavior(id: u32, behavior: ByzantineBehavior) -> Self {
        let online = !matches!(behavior, ByzantineBehavior::Crash);
        Self {
            id,
            behavior,
            current_view: 0,
            received_messages: Vec::new(),
            sent_messages: Vec::new(),
            online,
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Send a message (may be altered based on behavior)
    pub fn send_message<R: Rng>(
        &mut self,
        content: u64,
        msg_type: MessageType,
        timestamp: Tick,
        rng: &mut R,
    ) -> Option<ConsensusMessage> {
        // Check if node delivers at all
        if rng.gen::<f32>() > self.behavior.delivery_probability() {
            return None;
        }

        // Check if data is correct
        let final_content = if rng.gen::<f32>() > self.behavior.correctness_probability() {
            // Send arbitrary/corrupt data
            rng.gen::<u64>()
        } else {
            content
        };

        let msg = ConsensusMessage {
            sender: self.id,
            content: final_content,
            timestamp,
            msg_type,
        };

        self.sent_messages.push(msg.clone());
        Some(msg)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Receive a message
    pub fn receive_message(&mut self, msg: ConsensusMessage) {
        self.received_messages.push(msg);
    }
}

/// BFT Consensus Simulation (Tendermint-style)
///
/// **Theoretical Basis:**
/// - Tendermint: Practical Byzantine Fault Tolerance (Kwon 2014)
/// - Requires 2f+1 validators for f faults
/// - Three-phase commit: Propose → Prevote → Precommit → Commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BFTConsensus {
    /// Total number of validators
    pub n_validators: u32,
    /// Maximum faulty validators (f)
    pub max_faulty: u32,
    /// Minimum validators needed for consensus (2f+1)
    pub quorum: u32,
    /// All validator nodes
    pub validators: Vec<ValidatorNode>,
    /// Current round/view
    pub current_round: u32,
    /// Proposer rotation index
    pub proposer_index: u32,
    /// Committed blocks
    pub committed_blocks: Vec<u64>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl BFTConsensus {
    /// Create new BFT consensus network
    ///
    /// # Arguments
    /// * `n` - Total number of validators (must be ≥ 4 for BFT guarantees)
    /// * `f` - Maximum number of Byzantine faults to tolerate
    pub fn new(n: u32, f: u32) -> Self {
        debug_assert!(n > 3 * f, "Need n ≥ 3f+1 for BFT");
        debug_assert!(n >= 4, "Need at least 4 validators for BFT");

        let quorum = 2 * f + 1;
        let validators = (0..n).map(ValidatorNode::new_honest).collect();

        Self {
            n_validators: n,
            max_faulty: f,
            quorum,
            validators,
            current_round: 0,
            proposer_index: 0,
            committed_blocks: Vec::new(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create network with specific faulty nodes
    #[must_use]
    pub fn with_faulty_nodes(n: u32, f: u32, faulty_behavior: ByzantineBehavior) -> Self {
        let mut consensus = Self::new(n, f);

        // Make first f nodes faulty
        for i in 0..f {
            consensus.validators[i as usize].behavior = faulty_behavior;
            consensus.validators[i as usize].online =
                !matches!(faulty_behavior, ByzantineBehavior::Crash);
        }

        consensus
    }

    /// Simulate one consensus round
    ///
    /// **Algorithm:**
    /// 1. Proposer broadcasts proposal
    /// 2. Validators send prevotes
    /// 3. If 2f+1 prevotes received, send precommits
    /// 4. If 2f+1 precommits received, commit block
    ///
    /// Deliver a slice of messages to all validators
    fn deliver_messages(&mut self, messages: &[ConsensusMessage]) {
        for i in 0..self.n_validators as usize {
            for msg in messages {
                self.validators[i].receive_message(msg.clone());
            }
        }
    }

    /// Collect messages sent by all validators for a given phase
    fn collect_messages<R: Rng>(
        &mut self,
        proposed_value: u64,
        msg_type: MessageType,
        tick: Tick,
        rng: &mut R,
    ) -> Vec<ConsensusMessage> {
        let mut messages = Vec::new();
        for i in 0..self.n_validators as usize {
            if let Some(msg) = self.validators[i].send_message(proposed_value, msg_type, tick, rng)
            {
                messages.push(msg);
            }
        }
        messages
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// run round.
    pub fn run_round<R: Rng>(
        &mut self,
        proposed_value: u64,
        tick: Tick,
        rng: &mut R,
    ) -> RoundResult {
        let proposer = self.proposer_index as usize;
        let round = self.current_round;

        // Phase 1: Propose (proposer sends; if offline, view change to next online validator)
        let mut proposals = Vec::new();
        let mut active_proposer = proposer;
        for attempt in 0..self.n_validators as usize {
            let cand = (proposer + attempt) % self.n_validators as usize;
            if self.validators[cand].online {
                if let Some(proposal) = self.validators[cand].send_message(
                    proposed_value,
                    MessageType::Propose,
                    tick,
                    rng,
                ) {
                    proposals.push(proposal);
                    active_proposer = cand;
                    break;
                }
            }
        }
        self.deliver_messages(&proposals);

        // If no proposal was delivered, round fails to commit (Tendermint Proposal Validity)
        let Some(proposal) = proposals.first() else {
            self.proposer_index = (self.proposer_index + 1) % self.n_validators;
            self.current_round += 1;
            return RoundResult {
                round,
                committed: false,
                committed_value: None,
                proposal_count: 0,
                prevote_count: 0,
                precommit_count: 0,
                quorum_reached: false,
            };
        };

        let round_val = proposal.content;
        // Phase 2: Prevote
        let prevotes = self.collect_messages(round_val, MessageType::Prevote, tick, rng);
        self.deliver_messages(&prevotes);

        let prevote_counts = self.count_votes(&prevotes, MessageType::Prevote);
        let quorum_val = prevote_counts
            .iter()
            .find(|(_, &c)| c >= self.quorum)
            .map(|(&v, _)| v);

        // Phase 3: Precommit
        let precommits = if let Some(v) = quorum_val {
            self.collect_messages(v, MessageType::Precommit, tick, rng)
        } else {
            Vec::new()
        };
        self.deliver_messages(&precommits);

        // Phase 4: Commit
        let precommit_counts = self.count_votes(&precommits, MessageType::Precommit);
        let committed_value = if let Some(v) = quorum_val {
            if precommit_counts.get(&v).copied().unwrap_or(0) >= self.quorum {
                Some(v)
            } else {
                None
            }
        } else {
            None
        };

        let committed = committed_value.is_some();
        if let Some(value) = committed_value {
            self.committed_blocks.push(value);
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Rotate proposer
        self.proposer_index = (active_proposer as u32 + 1) % self.n_validators;
        self.current_round += 1;

        RoundResult {
            round,
            committed,
            committed_value,
            proposal_count: proposals.len() as u32,
            prevote_count: prevotes.len() as u32,
            precommit_count: precommits.len() as u32,
            quorum_reached: quorum_val.is_some(),
        }
    }

    /// Count votes by value with validator deduplication and type check
    fn count_votes(
        &self,
        messages: &[ConsensusMessage],
        expected_type: MessageType,
    ) -> HashMap<u64, u32> {
        let mut counts = HashMap::new();
        let mut seen_senders = std::collections::HashSet::new();
        for msg in messages {
            if msg.msg_type == expected_type
                && msg.sender < self.n_validators
                && seen_senders.insert(msg.sender)
            {
                *counts.entry(msg.content).or_insert(0) += 1;
            }
        }
        counts
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Get number of honest nodes
    #[must_use]
    pub fn honest_count(&self) -> u32 {
        self.validators
            .iter()
            .filter(|v| v.behavior == ByzantineBehavior::Honest)
            .count() as u32
    }

    /// Get number of faulty nodes
    #[must_use]
    pub fn faulty_count(&self) -> u32 {
        self.validators
            .iter()
            .filter(|v| v.behavior != ByzantineBehavior::Honest)
            .count() as u32
    }

    /// Check if BFT guarantees hold
    /// Safety: No two honest nodes commit different values
    /// Liveness: Eventually some value is committed
    #[must_use]
    pub fn check_bft_guarantees(&self) -> BFTAnalysis {
        let committed_values: std::collections::HashSet<_> =
            self.committed_blocks.iter().copied().collect();

        // Safety check: All committed values should be the same
        // (In real BFT, we check for safety violations per block height)
        let safety_violations = committed_values.len().saturating_sub(1);

        // Liveness: Committed at least one block
        let live = !self.committed_blocks.is_empty();

        BFTAnalysis {
            n: self.n_validators,
            f: self.faulty_count(),
            quorum: self.quorum,
            committed_blocks: self.committed_blocks.len() as u32,
            safety_violations: safety_violations as u32,
            live,
            bft_guarantees_hold: self.faulty_count() <= self.max_faulty && live,
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Result of one consensus round
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundResult {
    /// Round number
    pub round: u32,
    /// Whether a value was committed
    pub committed: bool,
    /// The committed value (if any)
    pub committed_value: Option<u64>,
    /// Number of proposals
    pub proposal_count: u32,
    /// Number of prevotes
    pub prevote_count: u32,
    /// Number of precommits
    pub precommit_count: u32,
    /// Whether quorum prevotes were reached
    pub quorum_reached: bool,
}

/// BFT analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BFTAnalysis {
    /// Total nodes
    pub n: u32,
    /// Faulty nodes observed
    pub f: u32,
    /// Quorum size
    pub quorum: u32,
    /// Total committed blocks
    pub committed_blocks: u32,
    /// Number of safety violations
    pub safety_violations: u32,
    /// Whether liveness was achieved
    pub live: bool,
    /// Whether BFT guarantees hold
    pub bft_guarantees_hold: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_bft_honest_consensus_round_commits() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut bft = BFTConsensus::new(4, 1);
        let result = bft.run_round(12345, 1, &mut rng);
        assert!(result.committed);
        assert_eq!(result.committed_value, Some(12345));
        assert_eq!(bft.committed_blocks, vec![12345]);
    }

    #[test]
    fn test_bft_crashed_proposer_does_not_commit_phantom() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut bft = BFTConsensus::new(4, 1);
        for v in &mut bft.validators {
            v.online = false;
        }
        let result = bft.run_round(99999, 1, &mut rng);
        assert!(
            !result.committed,
            "All-crashed network cannot commit a block"
        );
        assert_eq!(result.committed_value, None);
        assert!(bft.committed_blocks.is_empty());
    }

    #[test]
    fn test_bft_deduplicates_equivocating_validator_votes() {
        let bft = BFTConsensus::new(4, 1);
        let messages = vec![
            ConsensusMessage {
                sender: 1,
                content: 42,
                timestamp: 1,
                msg_type: MessageType::Prevote,
            };
            5
        ];
        let counts = bft.count_votes(&messages, MessageType::Prevote);
        assert_eq!(counts.get(&42).copied(), Some(1));
    }
}
