//! Embodied Cognition System
//!
//! Implements sensorimotor grounding and extended mind thesis:
//! - Body schema and proprioceptive awareness
//! - Sensorimotor contingencies (O'Regan & Noe 2001)
//! - Affordance detection (Gibson 1979)
//! - Extended mind (Clark & Chalmers 1998)
//! - Enactive perception (Varela et al. 1991)
//!
//! References:
//! - Varela, Thompson, Rosch (1991): The Embodied Mind
//! - Clark & Chalmers (1998): The Extended Mind
//! - O'Regan & Noe (2001): A sensorimotor account of vision
//! - Gibson (1979): The Ecological Approach to Visual Perception
//! - Wilson & Golonka (2013): Embodied cognition is not what you think

pub mod methods;
#[cfg(test)]
pub mod tests;
pub mod types;

pub use types::*;
