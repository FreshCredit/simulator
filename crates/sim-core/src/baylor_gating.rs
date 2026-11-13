//! # Sim D18: Baylor Unconscious Narrative Gating & Attractor Drift
//!
//! Models the Baylor College of Medicine studies on narrative and syntactic
//! processing during propofol anesthesia and unconscious states within DOCAS:
//! - Conscious mode (M=1) vs. Unconscious mode (M=0)
//! - Cortisol threshold bypassing (C < C_crit = 0.12) & loss-aversion collapse (lambda^- -> 0)
//! - Dopamine-decoupled syntactic schema preservation (D=0)
//! - Relational (O/S) narrative tagging & waking trust attractor drift vs. waking executive armor

use serde::{Deserialize, Serialize};

/// Critical cortisol threshold for executive threat gate deactivation
pub const CORTISOL_CRITICAL_THRESHOLD: f32 = 0.12;

/// Baseline conscious cognitive friction
pub const CONSCIOUS_FRICTION: f32 = 1.0;

/// Unconscious cognitive friction under propofol/NREM Cortisol suppression (92% reduction)
pub const UNCONSCIOUS_FRICTION: f32 = 0.08;

/// Consciousness Mode for narrative processing (DOCAS §4 / Baylor paradigm)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ConsciousnessMode {
    /// Conscious mode (M=1): Active executive threat monitoring, full loss aversion
    #[default]
    Conscious,
    /// Unconscious mode (M=0): Anesthesia / deep NREM sleep, executive gate collapsed
    Unconscious,
}

/// Baylor state configuration for agent narrative processing
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BaylorStateConfig {
    /// Consciousness state mode M in {0, 1}
    pub mode: ConsciousnessMode,
    /// Cortisol threat threshold C_crit (default: 0.12)
    pub cortisol_crit: f32,
    /// Relational Oxytocin tagging weight eta_O
    pub eta_oxytocin: f32,
    /// Relational Serotonin tagging weight eta_S
    pub eta_serotonin: f32,
    /// Waking executive reactivation flag
    pub waking_reactivated: bool,
}

impl Default for BaylorStateConfig {
    fn default() -> Self {
        Self {
            mode: ConsciousnessMode::Conscious,
            cortisol_crit: CORTISOL_CRITICAL_THRESHOLD,
            eta_oxytocin: 0.15,
            eta_serotonin: 0.10,
            waking_reactivated: true,
        }
    }
}

impl BaylorStateConfig {
    /// Construct unconscious propofol/deep-sleep mode
    #[must_use]
    pub fn unconscious() -> Self {
        Self {
            mode: ConsciousnessMode::Unconscious,
            cortisol_crit: CORTISOL_CRITICAL_THRESHOLD,
            eta_oxytocin: 0.15,
            eta_serotonin: 0.10,
            waking_reactivated: false,
        }
    }

    /// Evaluate cognitive friction R_fric given current Cortisol level
    #[must_use]
    pub fn cognitive_friction(&self, cortisol: f32) -> f32 {
        if self.mode == ConsciousnessMode::Unconscious && cortisol < self.cortisol_crit {
            UNCONSCIOUS_FRICTION
        } else {
            CONSCIOUS_FRICTION
        }
    }

    /// Evaluate executive threat gate E(t) in [0, 1]
    #[must_use]
    pub fn executive_threat_gate(&self, cortisol: f32) -> f32 {
        if self.mode == ConsciousnessMode::Unconscious && cortisol < self.cortisol_crit {
            0.0
        } else {
            1.0
        }
    }

    /// Effective loss aversion factor lambda^- (collapses to 0 when executive gate deactivated)
    #[must_use]
    pub fn effective_loss_aversion(&self, cortisol: f32, lambda_base: f32) -> f32 {
        let gate = self.executive_threat_gate(cortisol);
        gate * 2.4 * lambda_base
    }

    /// Baseline trust attractor shift induced by relational O/S narrative tagging:
    /// Delta S_base = eta_O * O + eta_S * S
    #[must_use]
    pub fn attractor_shift(&self, oxytocin: f32, serotonin: f32) -> f32 {
        if self.mode == ConsciousnessMode::Unconscious {
            self.eta_oxytocin * oxytocin + self.eta_serotonin * serotonin
        } else {
            0.0
        }
    }
}

/// Evaluation result for Sim D18 Baylor Narrative Gating Battery
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaylorEvaluationResult {
    /// Criterion C1: Cortisol Gate Bypassing & Friction Reduction (observed: ~0.92, threshold: >=0.75)
    pub friction_reduction: f32,
    /// Criterion C1 verdict
    pub c1_pass: bool,
    /// Criterion C2: Syntactic sequence prediction perplexity difference ratio under D=0 vs D>0
    pub syntax_difference_ratio: f32,
    /// Baseline active perplexity under D > 0
    pub ppl_active: f32,
    /// Silenced perplexity under D = 0
    pub ppl_silenced: f32,
    /// Criterion C2 verdict
    pub c2_pass: bool,
    /// Criterion C3: Waking predatory uptake shift (observed: ~+0.040, threshold: >=0.25)
    pub waking_delta_uptake: f32,
    /// Control group uptake
    pub uptake_control: f32,
    /// Exposed waking group uptake
    pub uptake_exposed_waking: f32,
    /// Criterion C3 verdict (FAIL as frozen in paper, confirming waking executive resilience)
    pub c3_pass: bool,
    /// Criterion C4: Permutation significance p-value
    pub permutation_p_value: f64,
    /// Criterion C4 verdict
    pub c4_pass: bool,
    /// Compound verdict ("PASS" if all 4 met, "FAIL" as frozen if any fails)
    pub compound_verdict: String,
}

/// Evaluates syntactic sequence prediction perplexity under D>0 vs D=0
pub fn evaluate_syntax_perplexity(seed: u64) -> (f32, f32, f32) {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    let mut rng = StdRng::seed_from_u64(seed);

    let n_tokens = 500usize;
    let base_entropy = 1.85f32;

    let mut sum_d_active = 0.0f32;
    let mut sum_d_silenced = 0.0f32;

    for _ in 0..n_tokens {
        let u1: f32 = rng.gen_range(0.0001f32..1.0f32);
        let u2: f32 = rng.gen_range(0.0f32..1.0f32);
        let radius = (-2.0 * u1.ln()).sqrt();
        let angle = 2.0 * std::f32::consts::PI * u2;
        let z1 = radius * angle.cos();
        let z2 = radius * angle.sin();

        sum_d_active += z1 * 0.02f32;
        sum_d_silenced += z2 * 0.04f32;
    }

    let ppl_active = (base_entropy + sum_d_active / n_tokens as f32).exp();
    let ppl_silenced = (base_entropy + 0.03f32 + sum_d_silenced / n_tokens as f32).exp();

    let diff_ratio = (ppl_silenced - ppl_active).abs() / ppl_active;
    (ppl_active, ppl_silenced, diff_ratio)
}

/// Evaluates the complete Sim D18 Baylor unconscious narrative gating experiment on 1,000 agents
pub fn run_baylor_simulation(kappas: &[f32], seed: u64) -> BaylorEvaluationResult {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    let mut rng = StdRng::seed_from_u64(seed);
    let n_agents = kappas.len();

    // 1. Criterion C1: Cortisol Gate Bypassing & Friction Reduction
    let friction_reduction = (CONSCIOUS_FRICTION - UNCONSCIOUS_FRICTION) / CONSCIOUS_FRICTION;
    let c1_pass = friction_reduction >= 0.75;

    // 2. Criterion C2: Syntax Preservation Without Dopamine
    let (ppl_active, ppl_silenced, syntax_diff) = evaluate_syntax_perplexity(seed);
    let c2_pass = syntax_diff <= 0.05;

    // 3. Criterion C3 & C4: Waking Attractor Drift & Predatory Uptake
    let chi_pred = 3.65f32;
    let gamma = 8.0f32;
    let tau = 0.20f32;
    let apr_promo = 0.0f32;
    let apr_true = 390.0f32;
    let urgency = 0.25f32;
    let amt_norm = 0.40f32;
    let u_cash = 0.40f32 * amt_norm;

    let mut u_res = Vec::with_capacity(n_agents);
    let mut r_draws = Vec::with_capacity(n_agents);
    for _ in 0..n_agents {
        let u1: f32 = rng.gen_range(0.0001f32..1.0f32);
        let u2: f32 = rng.gen_range(0.0f32..1.0f32);
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
        u_res.push(0.10f32 + z * 0.04f32);
        r_draws.push(rng.gen_range(0.0f32..1.0f32));
    }

    // Control group
    let mut dec_ctrl = Vec::with_capacity(n_agents);
    for (i, &k) in kappas.iter().enumerate() {
        let p_disc = 1.0f32 / (1.0f32 + (-gamma * (k / chi_pred - tau)).exp());
        let u_seen = r_draws[i] < p_disc;
        let u_eff = if u_seen {
            -(apr_true / 100.0) * amt_norm + u_cash
        } else {
            -(apr_promo / 100.0) * amt_norm + u_cash + urgency
        };
        dec_ctrl.push(u_eff > u_res[i]);
    }
    let uptake_ctrl = dec_ctrl.iter().filter(|&&b| b).count() as f32 / n_agents.max(1) as f32;

    // Exposed waking group (M: 0 -> 1 after unconscious O/S narrative exposure)
    let mut dec_exp = Vec::with_capacity(n_agents);
    for (i, &k) in kappas.iter().enumerate() {
        let p_disc = 1.0f32 / (1.0f32 + (-gamma * (k / (chi_pred * 1.35) - tau)).exp());
        let u_seen = r_draws[i] < p_disc;
        let u_eff = if u_seen {
            -(apr_true / 100.0) * amt_norm + u_cash + 0.15
        } else {
            -(apr_promo / 100.0) * amt_norm + u_cash + urgency + 0.20
        };
        let u_res_shifted = u_res[i] - 0.12;
        dec_exp.push(u_eff > u_res_shifted);
    }
    let uptake_exp = dec_exp.iter().filter(|&&b| b).count() as f32 / n_agents.max(1) as f32;

    let delta_uptake = uptake_exp - uptake_ctrl;
    let c3_pass = delta_uptake >= 0.25;

    // Criterion C4: Exact Paired Permutation Significance Test under H0: Y_exp = Y_ctrl
    let n_perms = 1000usize;
    let mut perm_count = 0usize;
    for _ in 0..n_perms {
        let mut sum_treat = 0.0f32;
        let mut sum_ctrl = 0.0f32;
        for i in 0..n_agents {
            let swap: bool = rng.gen_bool(0.5);
            let y_treat = if swap { dec_ctrl[i] } else { dec_exp[i] };
            let y_ctrl = if swap { dec_exp[i] } else { dec_ctrl[i] };
            if y_treat {
                sum_treat += 1.0;
            }
            if y_ctrl {
                sum_ctrl += 1.0;
            }
        }
        let null_delta = (sum_treat - sum_ctrl) / n_agents.max(1) as f32;
        if null_delta >= delta_uptake {
            perm_count += 1;
        }
    }

    // Phipson-Smyth (2010) exact permutation p-value: (b + 1) / (M + 1)
    let p_value = (perm_count as f64 + 1.0) / (n_perms as f64 + 1.0);
    let c4_pass = p_value <= 0.001;

    let compound_verdict = if c1_pass && c2_pass && c3_pass && c4_pass {
        "PASS".to_string()
    } else {
        "FAIL".to_string()
    };

    BaylorEvaluationResult {
        friction_reduction,
        c1_pass,
        syntax_difference_ratio: syntax_diff,
        ppl_active,
        ppl_silenced,
        c2_pass,
        waking_delta_uptake: delta_uptake,
        uptake_control: uptake_ctrl,
        uptake_exposed_waking: uptake_exp,
        c3_pass,
        permutation_p_value: p_value,
        c4_pass,
        compound_verdict,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_baylor_cortisol_friction_reduction_c1() {
        let config_conscious = BaylorStateConfig::default();
        let config_unconscious = BaylorStateConfig::unconscious();

        let f_conscious = config_conscious.cognitive_friction(0.40);
        let f_unconscious = config_unconscious.cognitive_friction(0.04);

        assert_eq!(f_conscious, 1.0);
        assert_eq!(f_unconscious, 0.08);

        let reduction = (f_conscious - f_unconscious) / f_conscious;
        assert!(
            (reduction - 0.92).abs() < 0.01,
            "Friction reduction must equal 92%"
        );
        assert!(reduction >= 0.75, "Criterion C1 must pass >= 75%");
    }

    #[test]
    fn test_baylor_dopamine_decoupled_syntax_c2() {
        let (ppl_active, ppl_silenced, diff) = evaluate_syntax_perplexity(42);
        assert!(ppl_active > 0.0);
        assert!(ppl_silenced > 0.0);
        assert!(
            diff <= 0.05,
            "Syntactic sequence perplexity difference ratio must be <= 5% (got {diff:.4})"
        );
    }

    #[test]
    fn test_baylor_simulation_replicates_paper_metrics() {
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};
        let mut rng = StdRng::seed_from_u64(42);
        let mut kappas = Vec::with_capacity(1000);
        for _ in 0..1000 {
            let u1: f32 = rng.gen_range(0.001..1.0);
            let u2: f32 = rng.gen_range(0.001..1.0);
            let k = (u1.powf(1.0 / 2.0) * u2.powf(1.0 / 5.0)).clamp(0.05, 0.95);
            kappas.push(k);
        }

        let res = run_baylor_simulation(&kappas, 42);
        assert!(res.c1_pass, "C1 Cortisol friction reduction must pass");
        assert!(res.c2_pass, "C2 syntax preservation must pass");
        assert!(
            !res.c3_pass,
            "C3 fails as frozen, confirming waking executive resilience"
        );
        assert_eq!(
            res.compound_verdict, "FAIL",
            "Compound verdict must be FAIL as frozen"
        );
    }
}
