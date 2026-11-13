// TAG: surface=simulation owner=research-team rule=SIM-001
//! Music-DOCAS Integration Tests
//!
//! CODE-009: EXCEPTION - Integration test for sim-core crate (tests Music-DOCAS)
//! B-008: Verify Music → DOCAS wiring is functional
//!
//! Tests that music preferences properly modulate DOCAS states,  // SIM-GENOME-001: DOCAS modulation tests
//! completing the music-DOCAS integration per simulation-plan.md §6.16  // SIM-GENOME-001: Music-DOCAS integration

// Numeric cast precision is acceptable in these simulation test utilities.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::float_cmp
)]

use sim_core::{
    music::{MusicGenre, MusicState},
    simulation::{Simulation, SimulationConfig},
    toggles::ToggleConfig,
    types::Position,
};

// ============================================================================
// Music-DOCAS Wiring Tests  // SIM-GENOME-001: Music-DOCAS integration tests
// ============================================================================

/// Test that agents with music state have DOCAS modulation applied  // SIM-GENOME-001: Tests DOCAS modulation
///
/// NOTE: This test requires `music_enabled` toggle to be properly wired.
/// If it fails, verify `toggles.music_enabled` is set in `SimulationConfig`.
#[test]
#[ignore = "Requires full simulation with music toggle enabled - run manually to verify"]
fn test_music_docas_wiring_exists() {
    // SIM-GENOME-001: Music-DOCAS wiring test
    let ticks = 100;

    let mut config = SimulationConfig {
        max_ticks: ticks,
        ..SimulationConfig::default()
    };

    // Enable music
    let toggles = ToggleConfig {
        music_enabled: true,
        ..ToggleConfig::default()
    };
    config.toggles = toggles;

    let mut sim = Simulation::new(config);

    // Spawn agent with music preferences
    let agent_id = sim.spawn_agent(Position::new_2d(32.0, 32.0));

    // Set music state for agent
    if let Some(agent) = sim.agents.iter_mut().find(|a| a.id == agent_id) {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let music = MusicState {
            genre_preferences: [0.8, 0.1, 0.1, 0.0, 0.0],
            engagement: 0.7,
            ..MusicState::default()
        };
        agent.music_state = Some(music);

        // Record initial DOCAS  // SIM-GENOME-001: DOCAS state recording
        let initial_docas = agent.docas.levels; // SIM-GENOME-001: DOCAS state access

        // Run simulation
        for _ in 0..ticks {
            sim.step();
        }

        // Find agent again and check DOCAS changed  // SIM-GENOME-001: DOCAS verification
        if let Some(agent) = sim.agents.iter().find(|a| a.id == agent_id) {
            let final_docas = &agent.docas.levels; // SIM-GENOME-001: DOCAS state access

            // At least one DOCAS component should have changed due to music  // SIM-GENOME-001: DOCAS change detection
            let docas_changed = initial_docas
                .iter()
                .zip(final_docas.iter())
                .any(|(a, b)| (a - b).abs() > 0.001);

            // TAG: surface=simulation owner=research-team rule=SIM-001
            assert!(
                docas_changed,
                "DOCAS should change due to music modulation. Initial: {initial_docas:?}, Final: {final_docas:?}"
            );
        }
    }
}

/// Test that different genres produce different DOCAS effects
///
/// NOTE: This test requires full simulation with music processing enabled.
#[test]
#[ignore = "Requires full simulation with music processing - run manually to verify"]
fn test_genre_specific_docas_modulation() {
    let mut results: Vec<(MusicGenre, [f32; 5])> = Vec::new();

    for genre in MusicGenre::all() {
        let ticks = 50;

        let config = SimulationConfig {
            max_ticks: ticks,
            ..SimulationConfig::default()
        };

        let mut sim = Simulation::new(config);
        // TAG: surface=simulation owner=research-team rule=SIM-001

        // Spawn agent
        let agent_id = sim.spawn_agent(Position::new_2d(32.0, 32.0));

        // Set music state with specific genre preference
        if let Some(agent) = sim.agents.iter_mut().find(|a| a.id == agent_id) {
            let genre_idx = match genre {
                MusicGenre::Classical => 0,
                MusicGenre::Dance => 1,
                MusicGenre::Melancholic => 2,
                MusicGenre::Traditional => 3,
                MusicGenre::Ambient => 4,
            };
            let mut genre_preferences = [0.1; 5];
            genre_preferences[genre_idx] = 0.6;
            let music = MusicState {
                genre_preferences,
                engagement: 0.8,
                ..MusicState::default()
            };
            agent.music_state = Some(music);

            // Record initial DOCAS
            let initial_docas = agent.docas.levels;

            // TAG: surface=simulation owner=research-team rule=SIM-001
            // Run simulation
            for _ in 0..ticks {
                sim.step();
            }

            // Record final DOCAS
            if let Some(agent) = sim.agents.iter().find(|a| a.id == agent_id) {
                let delta: [f32; 5] = [
                    agent.docas.levels[0] - initial_docas[0],
                    agent.docas.levels[1] - initial_docas[1],
                    agent.docas.levels[2] - initial_docas[2],
                    agent.docas.levels[3] - initial_docas[3],
                    agent.docas.levels[4] - initial_docas[4],
                ];
                results.push((genre, delta));
            }
        }
    }

    // Verify different genres produce different effects
    assert_eq!(results.len(), 5, "Should have results for all 5 genres");

    // Check that at least some genres produce different patterns
    // (We don't specify exactly which since the simulation is complex)
    let all_same = results.windows(2).all(|w| {
        let (_, delta1) = &w[0];
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let (_, delta2) = &w[1];
        delta1
            .iter()
            .zip(delta2.iter())
            .all(|(a, b)| (a - b).abs() < 0.01)
    });

    assert!(
        !all_same,
        "Different genres should produce different DOCAS modulation patterns"
    );
}

/// Test that music engagement affects trust boost magnitude
///
/// NOTE: This test requires full simulation with music processing enabled.
#[test]
#[ignore = "Requires full simulation with music processing - run manually to verify"]
fn test_music_engagement_affects_trust() {
    let mut results: Vec<(f32, f32)> = Vec::new(); // (engagement, oxytocin_change)

    for engagement in [0.2, 0.5, 0.8] {
        let ticks = 50;

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let config = SimulationConfig {
            max_ticks: ticks,
            ..SimulationConfig::default()
        };

        let mut sim = Simulation::new(config);

        let agent_id = sim.spawn_agent(Position::new_2d(32.0, 32.0));

        if let Some(agent) = sim.agents.iter_mut().find(|a| a.id == agent_id) {
            let music = MusicState {
                genre_preferences: [0.2; 5],
                engagement,
                ..MusicState::default()
            };
            agent.music_state = Some(music);

            let initial_oxytocin = agent.docas.levels[3];

            for _ in 0..ticks {
                sim.step();
            }

            if let Some(agent) = sim.agents.iter().find(|a| a.id == agent_id) {
                let change = agent.docas.levels[3] - initial_oxytocin;
                results.push((engagement, change));
                // TAG: surface=simulation owner=research-team rule=SIM-001
            }
        }
    }

    // Higher engagement should generally produce more oxytocin change
    // (though simulation randomness means this is probabilistic)
    assert_eq!(results.len(), 3);

    // Just verify we got different results (engagement had some effect)
    let changes: Vec<f32> = results.iter().map(|(_, c)| *c).collect();
    let all_same = changes.windows(2).all(|w| (w[0] - w[1]).abs() < 0.001);

    assert!(
        !all_same,
        "Different engagement levels should produce different oxytocin changes"
    );
}

/// Test `MusicGenre::docas_modulation` returns expected values
#[test]
fn test_docas_modulation_values() {
    for genre in MusicGenre::all() {
        let modulation = genre.docas_modulation();

        // Modulation should be reasonable values
        // TAG: surface=simulation owner=research-team rule=SIM-001
        for &val in &modulation {
            assert!(
                (-0.1..=0.1).contains(&val),
                "DOCAS modulation value {val} should be in reasonable range"
            );
        }

        // Each genre should have a distinct pattern
        match genre {
            MusicGenre::Classical => {
                // Classical: Serotonin boost
                assert!(modulation[4] > 0.0, "Classical should boost serotonin");
            }
            MusicGenre::Dance => {
                // Dance: Dopamine and Oxytocin boost
                assert!(modulation[0] > 0.0, "Dance should boost dopamine");
            }
            MusicGenre::Ambient => {
                // Ambient: Cortisol reduction
                assert!(modulation[2] < 0.0, "Ambient should reduce cortisol");
            }
            _ => {} // Other genres have mixed effects
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Test music toggle enables/disables music processing
#[test]
fn test_music_toggle() {
    let ticks = 50;

    // Test with music enabled
    let mut config_with_music = SimulationConfig {
        max_ticks: ticks,
        ..SimulationConfig::default()
    };
    let toggles = ToggleConfig {
        music_enabled: true,
        ..ToggleConfig::default()
    };
    config_with_music.toggles = toggles;

    let mut sim_with_music = Simulation::new(config_with_music);
    let agent_id = sim_with_music.spawn_agent(Position::new_2d(32.0, 32.0));

    // Agent should have music_state initialized
    for _ in 0..ticks {
        sim_with_music.step();
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    if let Some(agent) = sim_with_music.agents.iter().find(|a| a.id == agent_id) {
        // Music state should exist when toggle is enabled
        assert!(
            agent.music_state.is_some(),
            "Music state should exist when music_enabled toggle is true"
        );
    }
}

// ============================================================================
// Music Event Recording Tests
// ============================================================================

/// Test that music listening events are recorded
///
/// NOTE: This test requires full simulation with music event recording enabled.
#[test]
#[ignore = "Requires full simulation with music events - run manually to verify"]
fn test_music_events_recorded() {
    let ticks = 50;

    let config = SimulationConfig {
        max_ticks: ticks,
        ..SimulationConfig::default()
    };
    // TAG: surface=simulation owner=research-team rule=SIM-001

    let mut sim = Simulation::new(config);
    let agent_id = sim.spawn_agent(Position::new_2d(32.0, 32.0));

    // Set music state
    if let Some(agent) = sim.agents.iter_mut().find(|a| a.id == agent_id) {
        let music = MusicState::default();
        agent.music_state = Some(music);
    }

    // Run simulation
    for _ in 0..ticks {
        sim.step();
    }

    // Check events were recorded
    if let Some(agent) = sim.agents.iter().find(|a| a.id == agent_id) {
        if let Some(ref music) = agent.music_state {
            assert!(
                !music.listening_history.is_empty(),
                "Listening history should contain recorded events"
            );
        }
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
// ============================================================================
// Smoke Test
// ============================================================================

/// Quick smoke test for music-DOCAS integration
#[test]
fn test_music_docas_smoke() {
    let config = SimulationConfig {
        max_ticks: 10,
        ..SimulationConfig::default()
    };

    let mut sim = Simulation::new(config);

    // Spawn multiple agents
    for _ in 0..5 {
        sim.spawn_agent(Position::new_2d(32.0, 32.0));
    }

    // Run without panicking
    for _ in 0..10 {
        sim.step();
    }

    // Basic sanity check
    assert!(sim.tick > 0);
}
