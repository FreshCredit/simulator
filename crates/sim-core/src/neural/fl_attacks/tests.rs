use super::*;
use crate::neural::federated_learning::TrainingSample;
use crate::DocasState;

// === AttackModel Tests ===

#[test]
fn test_attack_model_creation() {
    let model = AttackModel::new(5);
    assert_eq!(model.weights.len(), 5);
    assert_eq!(model.bias, 0.0);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_attack_model_predict() {
    let model = AttackModel {
        weights: vec![1.0, 0.0, 0.0],
        bias: 0.0,
    };
    let features = vec![1.0, 0.0, 0.0];
    let pred = model.predict(&features);
    assert!(pred > 0.5); // sigmoid(1.0) > 0.5
}

#[test]
fn test_attack_model_train() {
    let mut model = AttackModel::new(2);
    let data = vec![
        (vec![1.0, 0.0], true),
        (vec![0.0, 1.0], false),
        (vec![1.0, 1.0], true),
        (vec![0.0, 0.0], false),
    ];

    model.train(&data, 10, 0.1);

    // After training, predictions should be somewhat accurate
    let pred1 = model.predict(&[1.0, 0.0]);
    let pred0 = model.predict(&[0.0, 1.0]);
    assert!(pred1 > pred0);
}

// === SimpleModel Tests ===

#[test]
fn test_simple_model_creation() {
    let model = SimpleModel::new(10, 5);
    assert_eq!(model.input_dim, 10);
    assert_eq!(model.output_dim, 5);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_simple_model_predict() {
    let model = SimpleModel::new(5, 3);
    let input = vec![0.5; 5];
    let output = model.predict(&input);

    assert_eq!(output.len(), 3);
    // Check softmax sums to ~1
    let sum: f32 = output.iter().sum();
    assert!((sum - 1.0).abs() < 0.001);
}

#[test]
fn test_simple_model_train_step() {
    let mut model = SimpleModel::new(5, 3);
    let input = vec![0.5; 5];
    let initial_weights = model.get_weights();

    model.train_step(&input, 0, 0.1);

    let new_weights = model.get_weights();
    assert_ne!(initial_weights, new_weights);
}

#[test]
fn test_model_api_compute_gradients() {
    let model = SimpleModel::new(5, 3);
    let input = vec![0.5; 5];
    let grads = model.compute_gradients(&input, 0);

    assert!(!grads.is_empty());
}

// TAG: surface=simulation owner=research-team rule=SIM-001
// === MembershipInferenceAttack Tests ===

#[test]
fn test_membership_inference_creation() {
    let attack = MembershipInferenceAttack::new(3);
    assert_eq!(attack.feature_dim, 3);
    assert!(attack.shadow_dataset.is_empty());
}

#[test]
fn test_membership_inference_entropy() {
    let attack = MembershipInferenceAttack::new(3);
    let probs = vec![0.7, 0.2, 0.1];
    let entropy = attack.compute_entropy(&probs);
    assert!(entropy > 0.0);

    // Uniform distribution has higher entropy
    let uniform = vec![0.333, 0.333, 0.334];
    let uniform_entropy = attack.compute_entropy(&uniform);
    assert!(uniform_entropy > entropy);
}

#[test]
fn test_membership_inference_infer() {
    let attack = MembershipInferenceAttack::new(3);
    let sample = vec![0.1, 0.2, 0.3];
    let output = vec![0.9, 0.05, 0.05];

    let confidence = attack.infer_membership(&sample, &output);
    assert!((0.0..=1.0).contains(&confidence));
}

#[test]
fn test_membership_inference_shadow_training() {
    let mut attack = MembershipInferenceAttack::new(3);
    let data: Vec<Vec<f32>> = (0..100).map(|i| vec![i as f32 / 100.0; 10]).collect();

    // TAG: surface=simulation owner=research-team rule=SIM-001
    attack.train_shadow_models(2, &data);

    // Should have created shadow dataset
    assert!(!attack.shadow_dataset.is_empty());
}

// === ModelInversionAttack Tests ===

#[test]
fn test_model_inversion_creation() {
    let attack = ModelInversionAttack::new(2);
    assert_eq!(attack.target_class, 2);
}

#[test]
fn test_model_inversion_with_params() {
    let attack = ModelInversionAttack::new(1).with_params(0.05, 500);

    assert_eq!(attack.learning_rate, 0.05);
    assert_eq!(attack.iterations, 500);
}

#[test]
fn test_model_inversion_invert() {
    let mut attack = ModelInversionAttack::new(1).with_params(0.1, 50);
    let model = SimpleModel::new(5, 3);

    let reconstructed = attack.invert(&model, 5);

    assert_eq!(reconstructed.len(), 5);
    // All values should be in [0, 1]
    assert!(reconstructed.iter().all(|&x| (0.0..=1.0).contains(&x)));
}

#[test]
fn test_model_inversion_reconstruction_confidence() {
    let mut attack = ModelInversionAttack::new(1).with_params(0.1, 50);
    let model = SimpleModel::new(5, 3);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    attack.invert(&model, 5);
    let conf = attack.reconstruction_confidence(&model);

    assert!((0.0..=1.0).contains(&conf));
}

// === GradientLeakageAttack Tests ===

#[test]
fn test_gradient_leakage_creation() {
    let attack = GradientLeakageAttack::new();
    assert!(attack.dummy_data.is_empty());
}

#[test]
fn test_gradient_leakage_with_params() {
    let attack = GradientLeakageAttack::new().with_params(0.05, 200);

    assert_eq!(attack.learning_rate, 0.05);
    assert_eq!(attack.iterations, 200);
}

#[test]
fn test_gradient_leakage_reconstruct() {
    let mut attack = GradientLeakageAttack::new().with_params(0.1, 50);
    let model = SimpleModel::new(5, 3);

    let input = vec![0.5; 5];
    let grads = model.compute_gradients(&input, 0);

    let (reconstructed, label) = attack.reconstruct_from_gradients(&grads, &model, 5);

    assert_eq!(reconstructed.len(), 5);
    assert!(label < 3);
}

#[test]
fn test_gradient_leakage_get_reconstruction() {
    let mut attack = GradientLeakageAttack::new().with_params(0.1, 50);
    let model = SimpleModel::new(5, 3);

    // TAG: surface=simulation owner=research-team rule=SIM-001
    let input = vec![0.5; 5];
    let grads = model.compute_gradients(&input, 0);

    attack.reconstruct_from_gradients(&grads, &model, 5);

    let (data, label) = attack.get_reconstruction();
    assert_eq!(data.len(), 5);
    assert!(label < 3);
}

// === ByzantineAttack Tests ===

#[test]
fn test_byzantine_creation() {
    let attack = ByzantineAttack::new(ByzantineType::BitFlip, vec![0, 1, 2]);
    assert_eq!(attack.compromised_clients.len(), 3);
    assert!(attack.is_compromised(0));
    assert!(!attack.is_compromised(5));
}

#[test]
fn test_byzantine_with_noise_scale() {
    let attack = ByzantineAttack::new(ByzantineType::RandomNoise, vec![0]).with_noise_scale(1.5);
    assert_eq!(attack.noise_scale, 1.5);
}

#[test]
fn test_byzantine_bitflip() {
    let attack = ByzantineAttack::new(ByzantineType::BitFlip, vec![1]);
    let mut grads = vec![1.0, -2.0, 3.0];

    attack.poison_gradients(&mut grads, 1);

    assert_eq!(grads, vec![-1.0, 2.0, -3.0]);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_byzantine_no_poison_uncompromised() {
    let attack = ByzantineAttack::new(ByzantineType::BitFlip, vec![1]);
    let mut grads = vec![1.0, -2.0, 3.0];

    attack.poison_gradients(&mut grads, 0); // Not compromised

    assert_eq!(grads, vec![1.0, -2.0, 3.0]);
}

#[test]
fn test_byzantine_targeted_poison() {
    let attack = ByzantineAttack::new(ByzantineType::TargetedPoison, vec![0]);
    let mut grads = vec![1.0, 2.0, 3.0];

    attack.poison_gradients(&mut grads, 0);

    // Should be flipped and amplified
    assert!(grads[0] < 0.0);
    assert!(grads[1] < 0.0);
}

#[test]
fn test_byzantine_label_flip() {
    let attack = ByzantineAttack::new(ByzantineType::LabelFlip, vec![0]);
    let mut labels = vec![0, 1, 2, 0, 1];

    attack.poison_labels(&mut labels, 0, 3);

    // Labels should be flipped: 0->2, 1->1, 2->0
    assert_eq!(labels, vec![2, 1, 0, 2, 1]);
}

// === BackdoorAttack Tests ===

#[test]
fn test_backdoor_creation() {
    let trigger = vec![1.0, 1.0, 1.0];
    let attack = BackdoorAttack::new(trigger.clone(), 0);
    assert_eq!(attack.trigger, trigger);
    assert_eq!(attack.target_label, 0);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_backdoor_with_poison_rate() {
    let attack = BackdoorAttack::new(vec![1.0], 0).with_poison_rate(0.25);
    assert_eq!(attack.poison_rate, 0.25);
}

#[test]
fn test_backdoor_apply_trigger() {
    let attack = BackdoorAttack::new(vec![0.5, 0.5], 0);
    let mut input = vec![0.0; 5];

    attack.apply_trigger(&mut input, 0);

    assert_eq!(input[0], 0.5);
    assert_eq!(input[1], 0.5);
}

#[test]
fn test_backdoor_poison_training_data() {
    let attack = BackdoorAttack::new(vec![1.0], 2).with_poison_rate(0.5);

    let mut data: Vec<(Vec<f32>, usize)> = (0..10).map(|i| (vec![i as f32], i % 3)).collect();

    let poisoned = attack.poison_training_data(&mut data);

    // Should have poisoned ~50% (5 samples)
    assert_eq!(poisoned.len(), 5);
    // First 5 should have trigger and target label
    assert_eq!(data[0].1, 2);
    assert_eq!(data[4].1, 2);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_backdoor_detect_trigger() {
    let attack = BackdoorAttack::new(vec![0.5, 0.5], 0);
    let mut input = vec![0.0; 5];
    input[0] = 0.5;
    input[1] = 0.5;

    assert!(attack.detect_trigger(&input, 0.01));

    input[0] = 0.6; // Different value
    assert!(!attack.detect_trigger(&input, 0.01));
}

// === AttackMetrics Tests ===

#[test]
fn test_attack_metrics_default() {
    let metrics = AttackMetrics::default();
    assert_eq!(metrics.membership_accuracy, 0.0);
}

#[test]
fn test_attack_metrics_overall_risk() {
    let metrics = AttackMetrics {
        membership_accuracy: 0.8,
        gradient_leakage: 0.6,
        byzantine_impact: 0.4,
        backdoor_success_rate: 0.3,
        ..Default::default()
    };

    let risk = metrics.overall_risk();
    assert!(risk > 0.0);
}

#[test]
fn test_attack_metrics_privacy_at_risk() {
    let mut metrics = AttackMetrics::default();
    assert!(!metrics.privacy_at_risk(0.5));

    // TAG: surface=simulation owner=research-team rule=SIM-001
    metrics.membership_accuracy = 0.6;
    assert!(metrics.privacy_at_risk(0.5));
}

#[test]
fn test_attack_metrics_integrity_at_risk() {
    let mut metrics = AttackMetrics::default();
    assert!(!metrics.integrity_at_risk(0.5));

    metrics.byzantine_impact = 0.6;
    assert!(metrics.integrity_at_risk(0.5));
}

// === FLAttackSimulator Tests ===

#[test]
fn test_simulator_creation() {
    let sim = FLAttackSimulator::new();
    assert!(sim.membership_attack.is_none());
}

#[test]
fn test_simulator_with_attacks() {
    let sim = FLAttackSimulator::new()
        .with_membership_attack(MembershipInferenceAttack::new(3))
        .with_inversion_attack(ModelInversionAttack::new(0))
        .with_gradient_attack(GradientLeakageAttack::new());

    assert!(sim.membership_attack.is_some());
    assert!(sim.inversion_attack.is_some());
    assert!(sim.gradient_attack.is_some());
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_simulator_average_metrics() {
    let mut sim = FLAttackSimulator::new();
    sim.metrics_history.push(AttackMetrics {
        membership_accuracy: 0.5,
        ..Default::default()
    });
    sim.metrics_history.push(AttackMetrics {
        membership_accuracy: 0.7,
        ..Default::default()
    });

    let avg = sim.average_metrics();
    assert!((avg.membership_accuracy - 0.6).abs() < 0.01);
}

// === PrivacyAuditor Tests ===

#[test]
fn test_privacy_auditor_creation() {
    let auditor = PrivacyAuditor::new(0.5);
    assert_eq!(auditor.risk_threshold, 0.5);
}

#[test]
fn test_privacy_auditor_default() {
    let auditor = PrivacyAuditor::default();
    assert_eq!(auditor.risk_threshold, 0.5);
}

#[test]
fn test_audit_report_structure() {
    let report = AuditReport {
        timestamp: 12345,
        metrics: AttackMetrics::default(),
        privacy_at_risk: false,
        integrity_at_risk: false,
        recommendations: vec!["Test recommendation".to_string()],
    };

    // TAG: surface=simulation owner=research-team rule=SIM-001
    assert_eq!(report.timestamp, 12345);
    assert!(!report.privacy_at_risk);
}

// === Utility Function Tests ===

#[test]
fn test_sigmoid() {
    assert!((sigmoid(0.0) - 0.5).abs() < 0.001);
    assert!(sigmoid(5.0) > 0.9);
    assert!(sigmoid(-5.0) < 0.1);
}

#[test]
fn test_softmax() {
    let mut input = vec![1.0, 2.0, 3.0];
    softmax(&mut input);
    let output = input;

    let sum: f32 = output.iter().sum();
    assert!((sum - 1.0).abs() < 0.001);

    // Higher input should give higher probability
    assert!(output[2] > output[1]);
    assert!(output[1] > output[0]);
}

// === Integration Tests ===

#[test]
fn test_full_attack_pipeline() {
    // Create model
    let mut model = SimpleModel::new(10, 5);

    // Train model on some data
    for i in 0..20 {
        let input = vec![i as f32 / 20.0; 10];
        model.train_step(&input, i % 5, 0.01);
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    // Test membership inference
    let mut membership_attack = MembershipInferenceAttack::new(3);
    let test_data: Vec<Vec<f32>> = (0..10).map(|i| vec![i as f32 / 10.0; 10]).collect();
    membership_attack.train_shadow_models(2, &test_data);

    let sample = vec![0.5; 10];
    let output = model.predict(&sample);
    let confidence = membership_attack.infer_membership(&sample, &output);
    assert!((0.0..=1.0).contains(&confidence));

    // Test model inversion
    let mut inversion_attack = ModelInversionAttack::new(2).with_params(0.1, 100);
    let _reconstructed = inversion_attack.invert(&model, 10);

    // Test gradient leakage
    let mut grad_attack = GradientLeakageAttack::new().with_params(0.1, 100);
    let grads = model.compute_gradients(&sample, 0);
    let (_recovered, label) = grad_attack.reconstruct_from_gradients(&grads, &model, 10);
    assert!(label < 5);
}

#[test]
fn test_byzantine_defense_simulation() {
    // Simulate FL with Byzantine attack
    let byzantine = ByzantineAttack::new(ByzantineType::BitFlip, vec![0, 1]);

    let mut grads = vec![1.0, 2.0, 3.0, 4.0, 5.0];

    // Apply attack to compromised client
    byzantine.poison_gradients(&mut grads, 0);

    // Should be flipped
    assert!(grads[0] < 0.0);
    assert!(grads[1] < 0.0);

    // Apply to uncompromised client - no change
    let mut grads2 = vec![1.0, 2.0, 3.0];
    byzantine.poison_gradients(&mut grads2, 5);
    assert_eq!(grads2, vec![1.0, 2.0, 3.0]);
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_backdoor_attack_simulation() {
    let backdoor = BackdoorAttack::new(vec![1.0, 1.0, 1.0], 0).with_poison_rate(0.3);

    // Create sample data
    let mut samples: Vec<TrainingSample> = (0..10)
        .map(|i| TrainingSample::new(vec![i as f32 / 10.0; 5], DocasState::default()))
        .collect();

    // Poison samples
    let n_poisoned = backdoor.poison_samples(&mut samples);
    assert_eq!(n_poisoned, 3); // 30% of 10

    // First samples should have trigger
    assert!(samples[0].input[0] > 0.0);
}

#[test]
fn test_multiple_attacks_simulation() {
    // Setup simulator with all attack types
    let simulator = FLAttackSimulator::new()
        .with_membership_attack(MembershipInferenceAttack::new(3))
        .with_inversion_attack(ModelInversionAttack::new(0))
        .with_gradient_attack(GradientLeakageAttack::new())
        .with_byzantine_attack(ByzantineAttack::new(ByzantineType::RandomNoise, vec![0]))
        .with_backdoor_attack(BackdoorAttack::new(vec![1.0], 0));

    // Verify all attacks are present
    assert!(simulator.membership_attack.is_some());
    assert!(simulator.inversion_attack.is_some());
    assert!(simulator.gradient_attack.is_some());
    assert!(simulator.byzantine_attack.is_some());
    assert!(simulator.backdoor_attack.is_some());
}

// TAG: surface=simulation owner=research-team rule=SIM-001
#[test]
fn test_fl_attack_enum_variants() {
    let attacks = [
        FLAttack::MembershipInference {
            shadow_models: 3,
            attack_model: AttackModel::default(),
        },
        FLAttack::ModelInversion {
            iterations: 100,
            learning_rate: 0.1,
            target_class: 0,
        },
        FLAttack::GradientInversion { use_lbl: true },
        FLAttack::Byzantine {
            attack_type: ByzantineType::BitFlip,
            compromised_clients: vec![0, 1],
        },
        FLAttack::Backdoor {
            trigger_pattern: vec![1.0, 1.0],
            target_label: 0,
            poison_rate: 0.1,
        },
    ];

    assert_eq!(attacks.len(), 5);
}

#[test]
fn test_byzantine_type_variants() {
    let types = [
        ByzantineType::BitFlip,
        ByzantineType::LabelFlip,
        ByzantineType::RandomNoise,
        ByzantineType::TargetedPoison,
        ByzantineType::Omniscient,
    ];

    for (i, t) in types.iter().enumerate() {
        let attack = ByzantineAttack::new(*t, vec![0]);
        assert_eq!(attack.attack_type, types[i]);
    }
}
