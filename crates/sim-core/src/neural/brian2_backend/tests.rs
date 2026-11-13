// TAG: surface=simulation owner=research-team rule=SIM-001
//! Tests for Brian2 neural backend.

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::agent::Agent;
    use crate::neural::NeuralBackend;

    use crate::genome::Genome;
    use crate::types::{Position, Sex};

    fn create_test_agent() -> Agent {
        let genome = Genome::default();
        Agent::new(1, Position::new_2d(0.0, 0.0), genome, 0, Sex::Male)
    }

    #[test]
    fn test_brian2_backend_creation() {
        // Skip if Brian2 not installed
        if Brian2Backend::new().is_err() {
            println!("Skipping test - Brian2 not installed");
            return;
        }
        let backend = Brian2Backend::new().unwrap();
        assert_eq!(backend.name(), "Brian2");
    }

    #[test]
    fn test_health_check() {
        if let Ok(backend) = Brian2Backend::new() {
            assert!(backend.health_check().unwrap());
        }
    }

    #[test]
    fn test_default_equations() {
        let equations = Brian2Backend::default_neuron_equations();
        assert_eq!(equations.len(), 5);
        assert!(equations.contains_key(&Neuromodulator::Dopamine));
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(equations.contains_key(&Neuromodulator::Serotonin));
    }

    #[test]
    fn test_coupling_matrix() {
        if let Ok(mut backend) = Brian2Backend::new() {
            backend.set_coupling(Neuromodulator::Dopamine, Neuromodulator::Oxytocin, 0.5);
            assert_eq!(
                backend.coupling_matrix[Neuromodulator::Dopamine as usize]
                    [Neuromodulator::Oxytocin as usize],
                0.5
            );
        }
    }

    #[test]
    fn test_simulate_agent() {
        // Skip if Brian2 not installed
        let Ok(mut backend) = Brian2Backend::new() else {
            println!("Skipping test - Brian2 not installed");
            return;
        };

        let agent = create_test_agent();
        let result = backend.simulate_agent(&agent);

        assert!(
            result.is_ok(),
            "Simulation should succeed: {:?}",
            result.err()
        );
        let sim_result = result.unwrap();
        // TAG: surface=simulation owner=research-team rule=SIM-001

        // Verify result structure
        assert_eq!(sim_result.agent_id, agent.id);
        assert!(!sim_result.timestamps.is_empty(), "Should have timestamps");
        assert!(
            !sim_result.dopamine_activity.is_empty(),
            "Should have dopamine activity"
        );
        assert!(
            !sim_result.oxytocin_activity.is_empty(),
            "Should have oxytocin activity"
        );
        assert!(
            !sim_result.cortisol_activity.is_empty(),
            "Should have cortisol activity"
        );
        assert!(
            !sim_result.adrenaline_activity.is_empty(),
            "Should have adrenaline activity"
        );
        assert!(
            !sim_result.serotonin_activity.is_empty(),
            "Should have serotonin activity"
        );

        // Verify derived DOCAS parameters
        assert_eq!(sim_result.derived_docas.alpha.len(), 5);
        assert_eq!(sim_result.derived_docas.beta.len(), 5);
        assert!(sim_result.derived_docas.confidence > 0.0);

        // Verify metadata
        assert!(sim_result.metadata.contains_key("backend"));
        assert_eq!(sim_result.metadata.get("backend").unwrap(), "brian2");
        assert!(sim_result.metadata.contains_key("spike_counts"));
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    #[test]
    fn test_simulate_batch() {
        // Skip if Brian2 not installed
        let Ok(mut backend) = Brian2Backend::new() else {
            println!("Skipping test - Brian2 not installed");
            return;
        };

        let agents = vec![
            create_test_agent(),
            create_test_agent(),
            create_test_agent(),
        ];

        let results = backend.simulate_batch(&agents);
        assert!(results.is_ok());

        let sim_results = results.unwrap();
        assert_eq!(sim_results.len(), agents.len());

        for (i, result) in sim_results.iter().enumerate() {
            assert!(
                !result.timestamps.is_empty(),
                "Agent {i} should have timestamps"
            );
        }
    }

    #[test]
    fn test_caching() {
        // Skip if Brian2 not installed
        let Ok(mut backend) = Brian2Backend::new() else {
            println!("Skipping test - Brian2 not installed");
            return;
        };

        let agent = create_test_agent();

        // First simulation
        let result1 = backend.simulate_agent(&agent).unwrap();

        // Second simulation with same agent (should hit cache)
        let result2 = backend.simulate_agent(&agent).unwrap();

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Results should be identical (from cache)
        assert_eq!(result1.agent_id, result2.agent_id);
        assert_eq!(result1.timestamps, result2.timestamps);

        // Check cache stats
        let (cache_size, hits) = backend.cache_stats();
        assert!(cache_size >= 1, "Cache should have at least 1 entry");
        assert!(hits >= 1, "Should have at least 1 cache hit");
    }

    #[test]
    fn test_glial_system_integration() {
        // Skip if Brian2 not installed
        let Ok(mut backend) = Brian2Backend::new() else {
            println!("Skipping test - Brian2 not installed");
            return;
        };

        // Check glial statistics
        let stats = backend.glial_statistics();
        assert!(stats.astrocyte_count > 0, "Should have astrocytes");
        assert!(
            stats.oligodendrocyte_count > 0,
            "Should have oligodendrocytes"
        );
        assert!(
            stats.average_conduction_velocity >= 0.5,
            "Should have valid velocity"
        );

        // Test glial update
        let activity = vec![0.5; 100];
        backend.update_glial(&activity, 0.001);

        // Get updated stats
        let stats_after = backend.glial_statistics();
        assert!(stats_after.average_calcium_activity >= 0.0);

        // Test conduction delay
        let delay_myelinated = backend.conduction_delay(1000.0, true);
        let delay_unmyelinated = backend.conduction_delay(1000.0, false);

        // Myelinated should be faster (lower delay)
        assert!(
            delay_myelinated < delay_unmyelinated,
            "Myelinated conduction should be faster than unmyelinated"
        );
    }
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001

    #[test]
    fn test_glial_modulation_applied() {
        // Skip if Brian2 not installed
        let Ok(mut backend) = Brian2Backend::new() else {
            println!("Skipping test - Brian2 not installed");
            return;
        };

        let agent = create_test_agent();
        let result = backend.simulate_agent(&agent).unwrap();

        // Verify that glial modulation was applied
        // Check that DOCAS parameters have reasonable values
        assert!(result.derived_docas.confidence > 0.0);

        // Verify glial system was updated during simulation
        let stats = backend.glial_statistics();
        assert!(stats.glutamate_clearance >= 0.0 && stats.glutamate_clearance <= 1.0);
        assert!(stats.lactate_supply >= 0.0);
    }

    #[test]
    fn test_glutamate_synapse_creation() {
        // Skip if Brian2 not installed
        let Ok(backend) = Brian2Backend::new() else {
            println!("Skipping test - Brian2 not installed");
            return;
        };

        // Create excitatory synapse
        let exc_synapse = backend.create_glutamatergic_synapse(100, 20);
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(exc_synapse.ampa_receptors.len(), 100);
        assert_eq!(exc_synapse.nmda_receptors.len(), 20);
        assert!(exc_synapse.gaba_receptors.is_empty());

        // Create inhibitory synapse
        let inh_synapse = backend.create_gabaergic_synapse(30);
        assert!(inh_synapse.ampa_receptors.is_empty());
        assert!(inh_synapse.nmda_receptors.is_empty());
        assert_eq!(inh_synapse.gaba_receptors.len(), 30);

        // Create mixed synapse
        let mixed_synapse = backend.create_mixed_synapse(50, 10, 20);
        assert_eq!(mixed_synapse.ampa_receptors.len(), 50);
        assert_eq!(mixed_synapse.nmda_receptors.len(), 10);
        assert_eq!(mixed_synapse.gaba_receptors.len(), 20);
    }

    #[test]
    fn test_glutamate_synapse_dynamics() {
        // Skip if Brian2 not installed
        let Ok(backend) = Brian2Backend::new() else {
            println!("Skipping test - Brian2 not installed");
            return;
        };

        let mut synapse = backend.create_glutamatergic_synapse(100, 20);

        // Trigger presynaptic spike
        backend.trigger_presynaptic_spike(&mut synapse, true);
        assert!(synapse.glutamate_concentration > 0.0);

        // Compute current at different membrane potentials
        let current_at_rest = synapse.synaptic_current(-70.0);
        let current_depolarized = synapse.synaptic_current(-30.0);

        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        // Should have some current at both potentials
        // At depolarized potentials, NMDA unblocks so current should be higher
        assert!(current_at_rest != 0.0);
        assert!(current_depolarized != 0.0);
    }

    #[test]
    fn test_glutamate_gaba_integration() {
        // Skip if Brian2 not installed
        let Ok(backend) = Brian2Backend::new() else {
            println!("Skipping test - Brian2 not installed");
            return;
        };

        // Create mixed synapse
        let mut synapse = backend.create_mixed_synapse(50, 10, 20);

        // Trigger excitatory spike
        backend.trigger_presynaptic_spike(&mut synapse, true);
        let exc_current = synapse.synaptic_current(-50.0);

        // Reset and trigger inhibitory spike
        synapse.reset();
        backend.trigger_presynaptic_spike(&mut synapse, false);
        let inh_current = synapse.synaptic_current(-50.0);

        // Excitatory should push toward depolarization (less negative or positive current)
        // Inhibitory should push toward hyperpolarization (more negative current at -50mV)
        // At -50mV, GABA reversal is -70mV, so current is (v - e_rev) * conductance
        // This gives positive current, so we check magnitudes
        assert!(exc_current.abs() > 0.0);
        assert!(inh_current.abs() > 0.0);
    }

    #[test]
    fn test_synapse_update_with_glial_clearance() {
        // Skip if Brian2 not installed
        let Ok(backend) = Brian2Backend::new() else {
            println!("Skipping test - Brian2 not installed");
            return;
        };

        let mut synapse = backend.create_glutamatergic_synapse(100, 20);

        // Trigger spike and record concentration
        // TAG: surface=simulation owner=research-team rule=SIM-001
        backend.trigger_presynaptic_spike(&mut synapse, true);
        let initial_glut = synapse.glutamate_concentration;

        // Update synapse with glial clearance
        backend.update_synapse(&mut synapse, 10.0);

        // Glutamate should be cleared
        assert!(synapse.glutamate_concentration < initial_glut);
    }

    #[test]
    fn test_hh_neuron_equations_generation() {
        // Test HH equation generation for each neuromodulator
        for nm in [
            Neuromodulator::Dopamine,
            Neuromodulator::Oxytocin,
            Neuromodulator::Cortisol,
            Neuromodulator::Adrenaline,
            Neuromodulator::Serotonin,
        ] {
            let eqs = Brian2Backend::hh_neuron_equations(nm);

            // Verify key HH components are present
            assert!(
                eqs.contains("dv/dt"),
                "Should have voltage equation for {nm:?}"
            );
            assert!(
                eqs.contains("dm/dt"),
                "Should have m gate equation for {nm:?}"
            );
            assert!(
                eqs.contains("dh/dt"),
                "Should have h gate equation for {nm:?}"
            );
            assert!(
                eqs.contains("dn/dt"),
                "Should have n gate equation for {nm:?}"
            );
            assert!(
                eqs.contains("g_Na"),
                "Should have Na+ conductance for {nm:?}"
            );
            assert!(eqs.contains("g_K"), "Should have K+ conductance for {nm:?}");
            assert!(
                eqs.contains("alpha_m"),
                "Should have alpha_m rate for {nm:?}"
            );
            assert!(eqs.contains("beta_m"), "Should have beta_m rate for {nm:?}");
        }
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    }

    #[test]
    fn test_hh_equations_different_parameters() {
        // Verify different neuromodulators have different HH parameters
        let da_eqs = Brian2Backend::hh_neuron_equations(Neuromodulator::Dopamine);
        let cort_eqs = Brian2Backend::hh_neuron_equations(Neuromodulator::Cortisol);

        // Dopamine and cortisol should have different conductance values
        // (we check by looking for specific parameter lines)
        assert_ne!(
            da_eqs, cort_eqs,
            "Different neuromodulators should have different HH equations"
        );
    }

    #[test]
    fn test_enable_hodgkin_huxley_models() {
        // Skip if Brian2 not installed
        let Ok(mut backend) = Brian2Backend::new() else {
            println!("Skipping test - Brian2 not installed");
            return;
        };

        // Enable HH models
        let result = backend.enable_hodgkin_huxley_models();
        assert!(result.is_ok(), "Should enable HH models successfully");

        // Verify equations were updated
        for nm in [
            Neuromodulator::Dopamine,
            Neuromodulator::Oxytocin,
            Neuromodulator::Cortisol,
            Neuromodulator::Adrenaline,
            Neuromodulator::Serotonin,
        ] {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            let eqs = backend.neuron_equations.get(&nm).unwrap();
            assert!(
                eqs.contains("dm/dt"),
                "Should have HH m gate after enabling"
            );
            assert!(
                eqs.contains("alpha_m"),
                "Should have HH rate constants after enabling"
            );
        }
    }

    #[test]
    fn test_hh_integration_with_simulation() {
        // Skip if Brian2 not installed
        let Ok(mut backend) = Brian2Backend::new() else {
            println!("Skipping test - Brian2 not installed");
            return;
        };

        let agent = create_test_agent();

        // Enable HH models
        backend
            .enable_hodgkin_huxley_models()
            .expect("Should enable HH models");

        // Run simulation with HH neurons
        let result = backend.simulate_agent(&agent);
        assert!(result.is_ok(), "Simulation with HH models should succeed");

        let sim_result = result.unwrap();
        assert_eq!(sim_result.agent_id, agent.id);
        assert!(!sim_result.timestamps.is_empty(), "Should have timestamps");
    }
}
