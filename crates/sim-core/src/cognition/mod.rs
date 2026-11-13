// TAG: surface=simulation owner=research-team rule=SIM-001 test-coverage=unit
//! Cognition Module: Working Memory, Development, Cognitive Constraints, and Embodied Cognition
//!
//! Implements cognitive limitations and capacities for agents:
//! - Working memory with Miller's Law capacity limits (7 ± 2 chunks)
//! - Attention focus (Cowan's ~4 items)
//! - Executive control modulation
//! - Developmental robotics: maturational constraints and skill hierarchies
//! - Embodied cognition (sensorimotor grounding, extended mind)
//!
//! References:
//! - Miller (1956): The magical number seven, plus or minus two
//! - Cowan (2001): The magical number 4 in short-term memory
//! - Baddeley (2000): The episodic buffer
//! - Elman (1993): Starting small
//! - Cangelosi & Schlesinger (2015): Developmental robotics
//! - Smith & Gasser (2005): Development of embodied cognition
//! - Varela et al. (1991): The Embodied Mind (Enaction)
//! - Clark & Chalmers (1998): The Extended Mind
//! - O'Regan & Noe (2001): A sensorimotor account of vision
//! - Gibson (1979): The Ecological Approach to Visual Perception

pub mod embodied;
pub mod working_memory;

// Re-export main types
pub use embodied::{
    ActionCapabilities, ActionType, Affordance, BodyImage, BodyPart, BodyPartState,
    BodyRequirement, BodySchema, ChangeDirection, CognitiveScaffold, CoordinationPattern,
    CoupledSystem, DetectedObject, EmbodiedCognition, ExtendedMind, ExternalRepresentation,
    HandRequirement, ObjectId, ObjectType, PerceptualMode, PerceptualSalience, ProprioceptiveMap,
    RepresentationType, ScaffoldType, SensorimotorContingency, SensoryInput, SensoryModality,
    SpatialLayout, SystemComponent, TactileEvent,
};
pub use working_memory::{
    DecisionOption, MemoryCategory, MemoryChunk, MemoryContent, PerceivedInfo, RejectionReason,
    WorkingMemory, WorkingMemoryResult, WorkingMemoryStats,
};
