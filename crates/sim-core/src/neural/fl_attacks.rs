//! Advanced Federated Learning Attacks for Security Testing
//!
//! Implements various privacy and security attacks against federated learning systems
//! for testing robustness and evaluating defenses.
//!
// TAG: surface=simulation owner=research-team rule=SIM-001
//! ## Attack Types
//!
//! - **Membership Inference**: Determine if specific data was used in training (Shokri et al. 2017)
//! - **Model Inversion**: Reconstruct training data from model parameters (Fredrikson et al. 2015)
//! - **Gradient Inversion (DLG)**: Recover training data from gradients (Zhu et al. 2019)
//! - **Byzantine Attacks**: Malicious clients sending corrupted updates
//! - **Backdoor Attacks**: Trigger-activated misclassification (Gu et al. 2017)
//!
//! ## References
//!
//! - Shokri et al. (2017): "Membership Inference Attacks Against Machine Learning Models"
//! - Fredrikson et al. (2015): "Model Inversion Attacks that Exploit Confidence Information"
//! - Zhu et al. (2019): "Deep Leakage from Gradients"
//! - Gu et al. (2017): "`BadNets`: Identifying Vulnerabilities in Machine Learning"

/// types module.
pub mod types;
pub use types::{AttackMetrics, AttackModel, ByzantineType, FLAttack};

/// model module.
pub mod model;
pub use model::{ModelAPI, SimpleModel};

/// privacy module.
pub mod privacy;
pub use privacy::{GradientLeakageAttack, MembershipInferenceAttack, ModelInversionAttack};

/// integrity module.
pub mod integrity;
pub use integrity::{BackdoorAttack, ByzantineAttack};

/// simulator module.
pub mod simulator;
pub use simulator::{AuditReport, FLAttackSimulator, PrivacyAuditor};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Sigmoid activation function
#[must_use]
pub fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

/// Softmax normalization
pub fn softmax(values: &mut [f32]) {
    if values.is_empty() {
        return;
    }
    let max_val = values.iter().fold(
        f32::NEG_INFINITY,
        |a, &b| if b.is_finite() { a.max(b) } else { a },
    );
    if !max_val.is_finite() {
        return;
    }
    let exp_sum: f32 = values.iter().map(|&v| (v - max_val).exp()).sum();
    if exp_sum > 0.0 && exp_sum.is_finite() {
        for v in values.iter_mut() {
            *v = (*v - max_val).exp() / exp_sum;
        }
    }
}

/// Sample from Gaussian distribution using seeded pseudo-randomness for deterministic simulation (SIM-004)
#[must_use]
pub fn sample_gaussian(mean: f32, std_dev: f32) -> f32 {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    let seed = (mean.to_bits() as u64)
        .wrapping_mul(0x9E3779B97F4A7C15)
        .wrapping_add(std_dev.to_bits() as u64);
    let mut rng = StdRng::seed_from_u64(seed);
    let u1: f32 = rng.gen_range(1e-7..1.0 - 1e-7);
    let u2: f32 = rng.gen_range(0.0..1.0);
    let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
    mean + std_dev * z
}

#[cfg(test)]
mod tests;
