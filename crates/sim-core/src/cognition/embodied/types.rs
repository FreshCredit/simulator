// TAG: surface=simulation owner=research-team rule=SIM-001
//! Embodied cognition types, body schema, affordances, and extended mind structures.

use crate::types::{AgentId, Position};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Embodied cognition system integrating body, environment, and extended mind
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmbodiedCognition {
    /// Body representation and schema
    pub body_schema: BodySchema,
    /// Learned sensorimotor contingencies
    pub sensorimotor_contingencies: Vec<SensorimotorContingency>,
    /// Detected affordances in environment
    pub affordances: Vec<Affordance>,
    /// Body ownership level (1.0 = full ownership, <1.0 = illusion effects)
    pub body_ownership: f32,
    /// Extended mind cognitive scaffolding
    pub extended_mind: ExtendedMind,
    /// Current perceptual mode (enactive)
    pub perceptual_mode: PerceptualMode,
}

/// Body schema - neural representation of body
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BodySchema {
    /// Body parts with current state
    pub body_parts: HashMap<BodyPart, BodyPartState>,
    /// Proprioceptive awareness map
    pub proprioception: ProprioceptiveMap,
    /// Conscious body image
    pub body_image: BodyImage,
    /// Action capabilities
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub action_capabilities: ActionCapabilities,
}

/// Body part identifiers
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum BodyPart {
    /// head.
    Head,
    /// torso.
    Torso,
    /// left arm.
    LeftArm,
    /// right arm.
    RightArm,
    /// left hand.
    LeftHand,
    /// right hand.
    RightHand,
    /// left leg.
    LeftLeg,
    /// right leg.
    RightLeg,
    /// left foot.
    LeftFoot,
    /// right foot.
    RightFoot,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// State of a body part
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BodyPartState {
    /// Position in 3D space
    pub position: (f32, f32, f32),
    /// Velocity vector
    pub velocity: (f32, f32, f32),
    /// Posture (0.0 = closed/relaxed, 1.0 = open/extended)
    pub posture: f32,
    /// Temperature (Celsius)
    pub temperature: f32,
    /// Pain level (0.0 = no pain, 1.0 = severe)
    pub pain_level: f32,
}

/// Proprioceptive awareness of body position
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProprioceptiveMap {
    /// Joint angles by name
    pub joint_angles: HashMap<String, f32>,
    /// Muscle tension levels
    pub muscle_tension: HashMap<String, f32>,
    /// Derived limb positions
    pub limb_positions: Vec<(f32, f32, f32)>,
}

/// Body image - conscious representation of body
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BodyImage {
    /// Perceived body size (may differ from actual)
    pub perceived_size: f32,
    /// Perceived strength (0.0-1.0)
    pub perceived_strength: f32,
    /// Body satisfaction (0.0-1.0)
    pub body_satisfaction: f32,
    /// Attention to body signals (0.0-1.0)
    pub body_monitoring: f32,
}

/// Capabilities the body enables
// TAG: surface=simulation owner=platform-team rule=GENERAL-001
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionCapabilities {
    /// Maximum reach radius (meters)
    pub reach_radius: f32,
    /// Maximum carry weight (kg)
    pub max_carry_weight: f32,
    /// Locomotion speed (m/s)
    pub locomotion_speed: f32,
    /// Manipulation precision (0.0-1.0)
    pub manipulation_precision: f32,
    /// Balance stability (0.0-1.0)
    pub balance_stability: f32,
}

/// Sensorimotor contingency: how sensory input changes with movement
/// Per O'Regan & Noe (2001): "Perception is mastery of sensorimotor contingencies"
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SensorimotorContingency {
    /// Action that produces the contingency
    pub action: ActionType,
    /// Sensory modality affected
    pub sensory_modality: SensoryModality,
    /// Expected sensory change (simplified representation)
    pub expected_change_magnitude: f32,
    /// Direction of change
    pub change_direction: ChangeDirection,
    /// Learning progress (0.0 = novel, 1.0 = fully learned)
    pub learned_dependence: f32,
    /// Confidence in prediction
    pub prediction_confidence: f32,
}

/// Direction of sensory change
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeDirection {
    /// increase.
    Increase,
    /// decrease.
    Decrease,
    /// transform.
    Transform,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Types of actions
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ActionType {
    /// move forward.
    MoveForward,
    /// move backward.
    MoveBackward,
    /// turn left.
    TurnLeft,
    /// turn right.
    TurnRight,
    /// reach.
    Reach,
    /// grasp.
    Grasp,
    /// release.
    Release,
    /// look at.
    LookAt,
    /// focus attention.
    FocusAttention,
    /// sit.
    Sit,
    /// stand.
    Stand,
}

/// Sensory modalities
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum SensoryModality {
    /// visual.
    Visual,
    /// auditory.
    Auditory,
    /// tactile.
    Tactile,
    /// proprioceptive.
    Proprioceptive,
    /// vestibular.
    Vestibular,
}

/// Affordance: what the environment offers for action
/// Per Gibson (1979): "Affordances are opportunities for action"
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Affordance {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Object providing the affordance
    pub object_id: ObjectId,
    /// Action afforded
    pub action: ActionType,
    /// Strength of affordance (0.0-1.0)
    pub affordance_strength: f32,
    /// Body requirements to actualize
    pub body_requirement: BodyRequirement,
    /// Detection saliency
    pub detection_saliency: f32,
}

/// Physical requirements for action
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BodyRequirement {
    /// Minimum reach distance required
    pub min_reach: Option<f32>,
    /// Minimum strength required
    pub min_strength: Option<f32>,
    /// Hand requirements
    pub hand_requirement: HandRequirement,
}

/// Hand involvement in action
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HandRequirement {
    /// none.
    None,
    /// single hand.
    SingleHand,
    /// both hands.
    BothHands,
    /// fine manipulation.
    FineManipulation,
}

/// Extended mind: cognitive processes extending into environment
/// Per Clark & Chalmers (1998): "Where does the mind stop and the rest of the world begin?"
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ExtendedMind {
    /// Cognitive scaffolding (tools that extend cognition)
    pub cognitive_scaffolding: Vec<CognitiveScaffold>,
    /// External representations being used
    pub external_representations: Vec<ExternalRepresentation>,
    /// Coupled agent-environment systems
    pub coupled_systems: Vec<CoupledSystem>,
    /// Transparency gradients (how "transparent" each scaffold is)
    pub transparency_gradients: HashMap<String, f32>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Cognitive scaffold - tool/resource that extends cognition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CognitiveScaffold {
    /// Type of scaffold
    pub scaffold_type: ScaffoldType,
    /// Location in environment
    pub location: Option<Position>,
    /// Reliability of scaffold (0.0-1.0)
    pub reliability: f32,
    /// Accessibility (0.0-1.0)
    pub accessibility: f32,
    /// Integration depth (0.0 = tool, 1.0 = part of me)
    pub integration_depth: f32,
    /// Usage count (affects transparency)
    pub usage_count: u32,
}

/// Types of cognitive scaffolding
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ScaffoldType {
    /// External memory (notebook, phone)
    Notebook,
    /// External computation (calculator, computer)
    Calculator,
    /// External spatial reasoning (map, GPS)
    Map,
    /// Extended body (tool)
    Tool,
    /// Distributed cognition (other agent)
    OtherAgent,
    /// Cultural artifact
    Artifact,
    /// Language/symbol system
    Language,
}

/// External representation (diagram, list, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExternalRepresentation {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Type of representation
    pub representation_type: RepresentationType,
    /// Content as vector (encoded)
    pub content: Vec<f32>,
    /// Spatial layout
    pub spatial_layout: SpatialLayout,
    /// Coupling strength with agent
    pub coupling_strength: f32,
}

/// Types of external representations
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepresentationType {
    /// list.
    List,
    /// diagram.
    Diagram,
    /// map.
    Map,
    /// notation.
    Notation,
    /// formula.
    Formula,
}

/// Spatial layout of representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpatialLayout {
    /// Dimensions (width, height)
    pub dimensions: (f32, f32),
    /// Element positions
    pub element_positions: Vec<(f32, f32)>,
}

/// Coupled agent-environment system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoupledSystem {
    /// Components in the coupling
    pub components: Vec<SystemComponent>,
    /// Coupling strength
    pub coupling_strength: f32,
    /// Coordination pattern
    pub coordination_pattern: CoordinationPattern,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// System component identifiers
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemComponent {
    /// brain.
    Brain,
    /// body.
    Body,
    /// tool.
    Tool(ObjectId),
    /// environment.
    Environment,
    /// other agent.
    OtherAgent(AgentId),
}

/// Coordination patterns in coupled systems
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoordinationPattern {
    /// Turn-taking
    Sequential,
    /// Simultaneous action
    Parallel,
    /// Self-organized
    Emergent,
}

/// Perceptual mode (enactive)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerceptualMode {
    /// Exploratory scanning
    Exploratory,
    /// Focused inspection
    Inspect,
    /// Goal-directed
    GoalDirected,
    /// Habitual (automatic)
    Habitual,
}

/// Perceptual salience structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerceptualSalience {
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Visual channel salience
    pub visual_salience: f32,
    /// Tactile channel salience
    pub tactile_salience: f32,
    /// Proprioceptive channel salience
    pub proprioceptive_salience: f32,
    /// Auditory channel salience
    pub auditory_salience: f32,
}

/// Sensory input from environment
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SensoryInput {
    /// Hand positions (if visible/propriocepted)
    pub hand_positions: HashMap<BodyPart, (f32, f32, f32)>,
    /// Detected objects
    pub objects: Vec<DetectedObject>,
    /// Tactile stimulation
    pub tactile_stimulation: Vec<TactileEvent>,
}

/// Detected object in environment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetectedObject {
    /// id.
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub id: ObjectId,
    /// object type.
    pub object_type: ObjectType,
    /// position.
    pub position: Position,
    /// height.
    pub height: f32,
    /// distance.
    pub distance: f32,
}

/// Object types
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectType {
    /// chair.
    Chair,
    /// cup.
    Cup,
    /// tool.
    Tool,
    /// surface.
    Surface,
    /// obstacle.
    Obstacle,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Tactile event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TactileEvent {
    /// body part.
    pub body_part: BodyPart,
    /// intensity.
    pub intensity: f32,
    /// location.
    pub location: (f32, f32, f32),
}

/// Action pattern analysis result
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ActionPattern {
    /// exploratory.
    pub exploratory: bool,
    /// manipulative.
    pub manipulative: bool,
    /// locomotive.
    pub locomotive: bool,
}

/// Object identifier
pub type ObjectId = u64;
