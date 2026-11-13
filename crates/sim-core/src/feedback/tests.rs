use std::collections::HashMap;

use super::*;
use crate::types::Tick;

// === FeedbackDimension Tests ===

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_feedback_dimension_all() {
    let dims = FeedbackDimension::all();
    assert_eq!(dims.len(), 8);
    assert!(dims.contains(&FeedbackDimension::Competence));
    assert!(dims.contains(&FeedbackDimension::Integrity));
}

#[test]
fn test_feedback_dimension_description() {
    assert!(!FeedbackDimension::Competence.description().is_empty());
    assert!(!FeedbackDimension::Integrity.description().is_empty());
}

// === InteractionType Tests ===

#[test]
fn test_interaction_type_all() {
    let types = InteractionType::all();
    assert_eq!(types.len(), 7);
}

#[test]
fn test_interaction_type_relevant_dimensions() {
    let trade_dims = InteractionType::Trade.relevant_dimensions();
    assert!(trade_dims.contains(&FeedbackDimension::Integrity));

    let collab_dims = InteractionType::Collaboration.relevant_dimensions();
    assert!(collab_dims.contains(&FeedbackDimension::Collaboration));
}

// === Dot Tests ===

#[test]
fn test_dot_creation() {
    let dot = Dot::new(
        1,
        2,
        FeedbackDimension::Competence,
        0.8,
        0.9,
        100,
        "Test context",
        InteractionType::General,
    );

    // TAG: surface=simulation owner=research-team rule=SIM-001
    assert_eq!(dot.from, 1);
    assert_eq!(dot.to, 2);
    assert_eq!(dot.rating, 0.8);
    assert_eq!(dot.confidence, 0.9);
}

#[test]
fn test_dot_rating_clamping() {
    let dot = Dot::new(
        1,
        2,
        FeedbackDimension::Competence,
        1.5,
        1.2,
        100,
        "test",
        InteractionType::General,
    );
    assert_eq!(dot.rating, 1.0);
    assert_eq!(dot.confidence, 1.0);

    let dot2 = Dot::new(
        1,
        2,
        FeedbackDimension::Competence,
        -0.5,
        -0.1,
        100,
        "test",
        InteractionType::General,
    );
    assert_eq!(dot2.rating, 0.0);
    assert_eq!(dot2.confidence, 0.0);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_dot_weighted_rating() {
    let dot = Dot::new(
        1,
        2,
        FeedbackDimension::Competence,
        0.8,
        0.5,
        100,
        "test",
        InteractionType::General,
    );
    assert_eq!(dot.weighted_rating(), 0.4);
}

#[test]
fn test_dot_time_decay() {
    let dot = Dot::new(
        1,
        2,
        FeedbackDimension::Competence,
        0.8,
        1.0,
        0,
        "test",
        InteractionType::General,
    );

    // No decay at same tick
    assert_eq!(dot.time_decay(0, 0.01), 1.0);

    // Some decay after 100 ticks
    let decay = dot.time_decay(100, 0.01);
    assert!(decay < 1.0);
    assert!(decay > 0.0);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
// === DimensionScore Tests ===

#[test]
fn test_dimension_score_new() {
    let score = DimensionScore::new(FeedbackDimension::Competence, 100);
    assert_eq!(score.dimension, FeedbackDimension::Competence);
    assert_eq!(score.score, 0.5);
    assert_eq!(score.confidence, 0.0);
}

#[test]
fn test_dimension_score_update() {
    let mut score = DimensionScore::new(FeedbackDimension::Competence, 100);

    let dot = Dot::new(
        1,
        2,
        FeedbackDimension::Competence,
        0.9,
        1.0,
        100,
        "test",
        InteractionType::General,
    );

    score.update(&[&dot], 100, 0.01);

    assert!(score.score > 0.5);
    assert!(score.confidence > 0.0);
}

// === DotCollector Tests ===

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_dot_collector_new() {
    let collector = DotCollector::new(42);
    assert_eq!(collector.agent_id, 42);
    assert_eq!(collector.window_size, 100);
    assert_eq!(collector.received.len(), 0);
}

#[test]
fn test_dot_collector_with_config() {
    let collector = DotCollector::with_config(42, 50, 0.05);
    assert_eq!(collector.window_size, 50);
    assert_eq!(collector.decay_rate, 0.05);
}

#[test]
fn test_add_received_dot() {
    let mut collector = DotCollector::new(2);

    let dot = Dot::new(
        1,
        2,
        FeedbackDimension::Competence,
        0.9,
        1.0,
        100,
        "test",
        InteractionType::General,
    );

    collector.add_received_dot(dot);

    assert_eq!(collector.received.len(), 1);
    assert!(collector.believability[&FeedbackDimension::Competence] > 0.5);
}

#[test]
fn test_add_received_dot_wrong_agent() {
    let mut collector = DotCollector::new(2);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let dot = Dot::new(
        1,
        3,
        FeedbackDimension::Competence,
        0.9,
        1.0,
        100,
        "test",
        InteractionType::General,
    );

    collector.add_received_dot(dot);

    assert_eq!(collector.received.len(), 0);
}

#[test]
fn test_window_size_maintenance() {
    let mut collector = DotCollector::with_config(2, 5, 0.01);

    for i in 0..10 {
        let dot = Dot::new(
            1,
            2,
            FeedbackDimension::Competence,
            0.9,
            1.0,
            i as Tick,
            "test",
            InteractionType::General,
        );
        collector.add_received_dot(dot);
    }

    assert_eq!(collector.received.len(), 5);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_composite_believability() {
    let mut collector = DotCollector::new(2);

    // Add dots for multiple dimensions
    for dim in [FeedbackDimension::Competence, FeedbackDimension::Integrity] {
        let dot = Dot::new(1, 2, dim, 0.9, 1.0, 100, "test", InteractionType::General);
        collector.add_received_dot(dot);
    }

    let composite = collector.composite_believability();
    assert!(composite > 0.5);
    assert!(composite <= 1.0);
}

#[test]
fn test_dimension_believability() {
    let mut collector = DotCollector::new(2);

    let dot = Dot::new(
        1,
        2,
        FeedbackDimension::Competence,
        0.8,
        1.0,
        100,
        "test",
        InteractionType::General,
    );
    collector.add_received_dot(dot);

    let score = collector.dimension_believability(FeedbackDimension::Competence);
    assert!(score > 0.5);
}

#[test]
fn test_get_dots_in_period() {
    let mut collector = DotCollector::new(2);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    for i in 1..=10 {
        let dot = Dot::new(
            1,
            2,
            FeedbackDimension::Competence,
            0.9,
            1.0,
            i as Tick,
            "test",
            InteractionType::General,
        );
        collector.add_received_dot(dot);
    }

    let dots = collector.get_dots_in_period(3, 7);
    assert_eq!(dots.len(), 5);
}

#[test]
fn test_get_average_from_rater() {
    let mut collector = DotCollector::new(2);

    for _ in 0..5 {
        let dot = Dot::new(
            1,
            2,
            FeedbackDimension::Competence,
            0.8,
            1.0,
            100,
            "test",
            InteractionType::General,
        );
        collector.add_received_dot(dot);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let avg = collector.get_average_from_rater(1);
    assert_eq!(avg, Some(0.8));

    let none_avg = collector.get_average_from_rater(999);
    assert_eq!(none_avg, None);
}

#[test]
fn test_detect_rating_bias() {
    let mut collector = DotCollector::new(1);

    // Add given ratings
    for _ in 0..10 {
        let dot = Dot::new(
            1,
            2,
            FeedbackDimension::Competence,
            0.9,
            1.0,
            100,
            "test",
            InteractionType::General,
        );
        collector.add_given_dot(dot);
    }

    let mut pop_avg = HashMap::new();
    pop_avg.insert(FeedbackDimension::Competence, 0.5);

    let bias = collector.detect_rating_bias(&pop_avg);
    assert!(bias.overall_bias > 0.0); // Rates higher than average
}

#[test]
fn test_rating_distribution() {
    let mut collector = DotCollector::new(2);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Add dots with different ratings
    for rating in [0.1, 0.3, 0.5, 0.7, 0.9] {
        let dot = Dot::new(
            1,
            2,
            FeedbackDimension::Competence,
            rating,
            1.0,
            100,
            "test",
            InteractionType::General,
        );
        collector.add_received_dot(dot);
    }

    let dist = collector.get_rating_distribution();
    assert_eq!(dist.get("0.0-0.2"), Some(&1));
    assert_eq!(dist.get("0.8-1.0"), Some(&1));
}

#[test]
fn test_clear() {
    let mut collector = DotCollector::new(2);

    let dot = Dot::new(
        1,
        2,
        FeedbackDimension::Competence,
        0.9,
        1.0,
        100,
        "test",
        InteractionType::General,
    );
    collector.add_received_dot(dot.clone());
    collector.add_given_dot(dot);

    collector.clear();

    // TAG: surface=simulation owner=research-team rule=SIM-001
    assert_eq!(collector.received.len(), 0);
    assert_eq!(collector.given.len(), 0);
}

// === BaseballCard Tests ===

#[test]
fn test_baseball_card_new() {
    let card = BaseballCard::new(42, 100);
    assert_eq!(card.agent_id, 42);
    assert_eq!(card.overall_believability, 0.5);
    assert_eq!(card.dimension_scores.len(), 8);
}

#[test]
fn test_baseball_card_get_dimension_score() {
    let mut collector = DotCollector::new(42);

    let dot = Dot::new(
        1,
        42,
        FeedbackDimension::Competence,
        0.9,
        1.0,
        100,
        "test",
        InteractionType::General,
    );
    collector.add_received_dot(dot);

    let card = collector.generate_baseball_card(100);
    let score = card.get_dimension_score(FeedbackDimension::Competence);
    assert!(score.is_some());
    assert!(score.unwrap() > 0.5);
}

// === DotCollectorRegistry Tests ===

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_registry_new() {
    let registry = DotCollectorRegistry::new();
    assert!(registry.collectors.is_empty());
    assert_eq!(registry.default_window_size, 100);
}

#[test]
fn test_registry_get_collector() {
    let mut registry = DotCollectorRegistry::new();
    let collector = registry.get_collector(42);
    assert_eq!(collector.agent_id, 42);

    // Should return same collector
    let collector2 = registry.get_collector(42);
    assert_eq!(collector2.agent_id, 42);
}

#[test]
fn test_registry_record_dot() {
    let mut registry = DotCollectorRegistry::new();

    let dot = Dot::new(
        1,
        2,
        FeedbackDimension::Competence,
        0.9,
        1.0,
        100,
        "test",
        InteractionType::General,
    );

    registry.record_dot(dot);

    assert!(registry.collectors.contains_key(&1));
    assert!(registry.collectors.contains_key(&2));

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let receiver = registry.get_collector_ref(2).unwrap();
    assert_eq!(receiver.received.len(), 1);

    let giver = registry.get_collector_ref(1).unwrap();
    assert_eq!(giver.given.len(), 1);
}

#[test]
fn test_registry_get_top_by_dimension() {
    let mut registry = DotCollectorRegistry::new();

    // Add dots for two agents
    for (agent_id, rating) in [(2, 0.9), (3, 0.6)] {
        for _ in 0..5 {
            let dot = Dot::new(
                1,
                agent_id,
                FeedbackDimension::Competence,
                rating,
                1.0,
                100,
                "test",
                InteractionType::General,
            );
            registry.record_dot(dot);
        }
    }

    let top = registry.get_top_by_dimension(FeedbackDimension::Competence, 2);
    assert_eq!(top.len(), 2);
    assert_eq!(top[0].0, 2); // Higher rated agent first
}

#[test]
fn test_registry_weighted_vote() {
    let mut registry = DotCollectorRegistry::new();

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Give agent 1 high believability
    for _ in 0..10 {
        let dot = Dot::new(
            2,
            1,
            FeedbackDimension::Competence,
            0.95,
            1.0,
            100,
            "test",
            InteractionType::General,
        );
        registry.record_dot(dot);
    }

    // Give agent 2 low believability
    for _ in 0..10 {
        let dot = Dot::new(
            1,
            2,
            FeedbackDimension::Competence,
            0.3,
            1.0,
            100,
            "test",
            InteractionType::General,
        );
        registry.record_dot(dot);
    }

    let votes = vec![(1, true), (2, false)];
    let result = registry.compute_believability_weighted_vote(&votes, None);

    // Agent 1 (high believability) voted yes, should pass
    assert!(result.weighted_yes > result.weighted_no);
    assert!(result.passed);
}

#[test]
fn test_registry_get_population_stats() {
    let mut registry = DotCollectorRegistry::new();

    // TAG: surface=simulation owner=research-team rule=SIM-001
    for agent_id in [2, 3, 4] {
        for _ in 0..5 {
            let dot = Dot::new(
                1,
                agent_id,
                FeedbackDimension::Competence,
                0.7,
                1.0,
                100,
                "test",
                InteractionType::General,
            );
            registry.record_dot(dot);
        }
    }

    let stats = registry.get_population_stats();
    assert!(stats.contains_key(&FeedbackDimension::Competence));

    let comp_stats = stats.get(&FeedbackDimension::Competence).unwrap();
    assert_eq!(comp_stats.count, 3);
    assert!(comp_stats.mean > 0.0);
}

#[test]
fn test_registry_remove_agent() {
    let mut registry = DotCollectorRegistry::new();

    let dot = Dot::new(
        1,
        2,
        FeedbackDimension::Competence,
        0.9,
        1.0,
        100,
        "test",
        InteractionType::General,
    );
    registry.record_dot(dot);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    assert!(registry.collectors.contains_key(&2));

    registry.remove_agent(2);

    assert!(!registry.collectors.contains_key(&2));
}

// === BelievabilityVoteResult Tests ===

#[test]
fn test_believability_vote_result_consensus() {
    let result = BelievabilityVoteResult {
        weighted_yes: 8.0,
        weighted_no: 2.0,
        total_believability_participating: 10.0,
        passed: true,
        margin: 0.6,
        participation_rate: 1.0,
        total_eligible: 10,
        num_voted: 10,
    };

    assert_eq!(result.winner(), "yes");
    assert!(result.consensus_level() > 0.7);
}

// === RatingBias Tests ===

#[test]
fn test_rating_bias_harsh() {
    let bias = RatingBias {
        agent_id: 1,
        overall_bias: -0.2,
        dimension_biases: HashMap::new(),
        confidence: 0.8,
        ratings_count: 20,
    };

    // TAG: surface=simulation owner=research-team rule=SIM-001
    assert!(bias.is_harsh());
    assert!(!bias.is_lenient());
    assert!(!bias.is_balanced());
}

#[test]
fn test_rating_bias_lenient() {
    let bias = RatingBias {
        agent_id: 1,
        overall_bias: 0.2,
        dimension_biases: HashMap::new(),
        confidence: 0.8,
        ratings_count: 20,
    };

    assert!(!bias.is_harsh());
    assert!(bias.is_lenient());
    assert!(!bias.is_balanced());
}

#[test]
fn test_rating_bias_balanced() {
    let bias = RatingBias {
        agent_id: 1,
        overall_bias: 0.02,
        dimension_biases: HashMap::new(),
        confidence: 0.8,
        ratings_count: 20,
    };

    assert!(!bias.is_harsh());
    assert!(!bias.is_lenient());
    assert!(bias.is_balanced());
}

// TAG: surface=simulation owner=research-team rule=SIM-001
// === PopulationStats Tests ===

#[test]
fn test_population_stats_percentile() {
    let stats = PopulationStats {
        dimension: FeedbackDimension::Competence,
        mean: 0.5,
        std_dev: 0.1,
        min: 0.0,
        max: 1.0,
        count: 100,
    };

    let percentile = stats.percentile_rank(0.6);
    assert!(percentile > 50.0);

    let percentile2 = stats.percentile_rank(0.4);
    assert!(percentile2 < 50.0);
}

#[test]
fn test_population_stats_outlier() {
    let stats = PopulationStats {
        dimension: FeedbackDimension::Competence,
        mean: 0.5,
        std_dev: 0.1,
        min: 0.0,
        max: 1.0,
        count: 100,
    };

    assert!(stats.is_outlier(0.9)); // 4 std dev above mean
    assert!(!stats.is_outlier(0.55)); // Within 1 std dev
}

// === Utils Tests ===

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_utils_dot_from_trade() {
    let dot = utils::dot_from_trade(1, 2, 0.8, 100, "Fair trade");
    assert_eq!(dot.from, 1);
    assert_eq!(dot.to, 2);
    assert_eq!(dot.interaction_type, InteractionType::Trade);
    assert_eq!(dot.dimension, FeedbackDimension::Integrity);
}

#[test]
fn test_utils_dot_from_collaboration() {
    let dot = utils::dot_from_collaboration(
        1,
        2,
        FeedbackDimension::Collaboration,
        0.9,
        100,
        "Good teamwork",
    );
    assert_eq!(dot.dimension, FeedbackDimension::Collaboration);
    assert_eq!(dot.interaction_type, InteractionType::Collaboration);
}

#[test]
fn test_utils_weighted_average() {
    let ratings = vec![(0.8, 1.0), (0.6, 0.5), (0.9, 0.5)];
    let avg = utils::weighted_average(&ratings);

    // (0.8*1.0 + 0.6*0.5 + 0.9*0.5) / (1.0 + 0.5 + 0.5) = 1.55 / 2.0 = 0.775
    assert!((avg - 0.775).abs() < 0.001);
}

#[test]
fn test_utils_normalize_rating() {
    assert_eq!(utils::normalize_rating(5.0, 0.0, 10.0), 0.5);
    assert_eq!(utils::normalize_rating(0.0, 0.0, 10.0), 0.0);
    assert_eq!(utils::normalize_rating(10.0, 0.0, 10.0), 1.0);
}
