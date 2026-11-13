use super::types::{Dot, FeedbackDimension, InteractionType};
use crate::types::{AgentId, Tick};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Create a dot from a trade interaction
pub fn dot_from_trade(
    from: AgentId,
    to: AgentId,
    rating: f32,
    tick: Tick,
    context: impl Into<String>,
) -> Dot {
    Dot::new(
        from,
        to,
        FeedbackDimension::Integrity,
        rating,
        0.8,
        tick,
        context,
        InteractionType::Trade,
    )
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Create a dot from a collaboration interaction
pub fn dot_from_collaboration(
    from: AgentId,
    to: AgentId,
    dimension: FeedbackDimension,
    rating: f32,
    tick: Tick,
    context: impl Into<String>,
) -> Dot {
    Dot::new(
        from,
        to,
        dimension,
        rating,
        0.85,
        tick,
        context,
        InteractionType::Collaboration,
    )
}

/// Calculate believability-weighted average of ratings
#[must_use]
pub fn weighted_average(ratings: &[(f32, f32)]) -> f32 {
    if ratings.is_empty() {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        return 0.5;
    }

    let mut weighted_sum = 0.0;
    let mut weight_sum = 0.0;

    for (value, weight) in ratings {
        weighted_sum += value * weight;
        weight_sum += weight;
    }

    if weight_sum > 0.0 {
        weighted_sum / weight_sum
    } else {
        0.5
    }
}

/// Normalize ratings to 0-1 scale
#[must_use]
pub fn normalize_rating(rating: f32, min: f32, max: f32) -> f32 {
    if max == min {
        return 0.5;
    }
    ((rating - min) / (max - min)).clamp(0.0, 1.0)
}
