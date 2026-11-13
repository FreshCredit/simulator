//! Theory of Mind Module (P1-AGENT-2)
//!
//! Recursive belief modeling for agents.

pub mod belief;
pub mod inference;
pub mod recursive;

pub use belief::{Belief, BeliefState, MentalModel};
pub use inference::{BeliefInference, InferenceEngine};
pub use recursive::{RecursionDepth, RecursiveModel, TheoryOfMind};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Theory of Mind capability level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum TomLevel {
    /// No theory of mind
    Level0 = 0,
    /// First-order (I believe)
    Level1 = 1,
    /// Second-order (I believe you believe)
    #[default]
    Level2 = 2,
    /// Third-order (I believe you believe they believe)
    Level3 = 3,
    /// Higher-order reasoning
    Level4 = 4,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl TomLevel {
    #[must_use]
    /// max depth.
    pub const fn max_depth(&self) -> usize {
        *self as usize
    }

    #[must_use]
    /// can represent.
    pub const fn can_represent(&self, depth: usize) -> bool {
        depth <= self.max_depth()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tom_level_max_depth() {
        assert_eq!(TomLevel::Level0.max_depth(), 0);
        assert_eq!(TomLevel::Level1.max_depth(), 1);
        assert_eq!(TomLevel::Level2.max_depth(), 2);
        assert_eq!(TomLevel::Level3.max_depth(), 3);
        assert_eq!(TomLevel::Level4.max_depth(), 4);
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    #[test]
    fn test_tom_level_can_represent() {
        assert!(TomLevel::Level2.can_represent(0));
        assert!(TomLevel::Level2.can_represent(1));
        assert!(TomLevel::Level2.can_represent(2));
        assert!(!TomLevel::Level2.can_represent(3));
        assert!(!TomLevel::Level0.can_represent(1));
        assert!(TomLevel::Level4.can_represent(4));
    }

    #[test]
    fn test_tom_level_default() {
        let level: TomLevel = TomLevel::default();
        assert_eq!(level, TomLevel::Level2);
    }

    #[test]
    fn test_tom_level_ordering() {
        assert!(TomLevel::Level0 < TomLevel::Level1);
        assert!(TomLevel::Level1 < TomLevel::Level2);
        assert!(TomLevel::Level2 < TomLevel::Level3);
        assert!(TomLevel::Level3 < TomLevel::Level4);
    }
}
