// TAG: surface=simulation owner=research-team rule=SIM-001
//! Live Mechanistic & Protocol Battery Evaluator API
//!
//! Exposes `sim_core::v42_batteries`, `sim_core::master_law`, and `sim_core::baylor_gating`
//! directly to the simulator UI so every battery preset runs real mathematical models.

use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Metadata describing an interactively evaluable research battery
#[derive(Debug, Clone, Serialize)]
pub struct BatterySummary {
    pub id: &'static str,
    pub name: &'static str,
    pub paper: &'static str,
    pub verdict_v45: &'static str,
    pub description: &'static str,
    pub default_params: Value,
}

/// Canonical list of interactive mechanistic batteries
#[must_use]
pub fn battery_catalog() -> Vec<BatterySummary> {
    vec![
        BatterySummary {
            id: "sim-p8c",
            name: "Sim P8c: Provider-Diversity Entropy Gate",
            paper: "TFP §6 / ZK-STP v8",
            verdict_v45: "PASS",
            description: "Evaluates Shannon diversity entropy H(w) and accumulation efficiency eta = 1 - exp(-H(w)).",
            default_params: json!({ "weights": [10.0, 10.0, 10.0, 10.0, 10.0], "raw_daily_rate": 2.0 }),
        },
        BatterySummary {
            id: "sim-p8d",
            name: "Sim P8d: Microeconomic Collusion Auction",
            paper: "TFP §6",
            verdict_v45: "PASS",
            description: "Computes adversary Nash break-even credit extraction limit L*(K) across K compromised provider keys.",
            default_params: json!({ "k_keys": 5, "key_cost_usd": 350.0, "v_attack_daily": 10.0, "p_audit_per_key": 0.06 }),
        },
        BatterySummary {
            id: "sim-w9",
            name: "Sim W9: Dynamic Lemons Market Solvency ODE",
            paper: "Whitepaper §5",
            verdict_v45: "PASS",
            description: "Integrates lending pool reserve R(t) and entrant adverse selection over a multi-month horizon.",
            default_params: json!({ "initial_reserve_usd": 500000.0, "months": 36 }),
        },
        BatterySummary {
            id: "sim-at2",
            name: "Sim AT-2: Multiscale Assembly Index Spectrum",
            paper: "TDF / Assembly",
            verdict_v45: "PASS",
            description: "Evaluates Assembly Index ordering (cAMP MA=14 razor < ATP MA=15 < TFP Ledger MA=1,293).",
            default_params: json!({}),
        },
        BatterySummary {
            id: "sim-at3",
            name: "Sim AT-3: Quantum Morse vs Living Metabolic Repair",
            paper: "TDF / Assembly",
            verdict_v45: "PASS",
            description: "Compares abiotic Si-O bond fracture against active cAMP metabolic repair.",
            default_params: json!({ "displacement_angstroms": 4.0, "has_active_living_repair": true }),
        },
        BatterySummary {
            id: "sim-thermo1",
            name: "Sim THERMO-1: Landauer Erasure Bound",
            paper: "GTT / TDF",
            verdict_v45: "PASS",
            description: "Computes Landauer bound k_B T ln(2) per erased bit at physiological or ambient temperature.",
            default_params: json!({ "temp_kelvin": 310.15 }),
        },
        BatterySummary {
            id: "sim-patch1",
            name: "Sim PATCH-1: HCN1 Dendritic Shunting Kinetics",
            paper: "TDF §3",
            verdict_v45: "PASS",
            description: "Evaluates cAMP-gated HCN1 dendritic shunting ratio across basal and storm cAMP concentrations.",
            default_params: json!({ "camp_concentration_um": 0.50 }),
        },
        BatterySummary {
            id: "sim-d18",
            name: "Sim D18: Baylor Unconscious Narrative Gating",
            paper: "TDF §4",
            verdict_v45: "MIXED (C1-C2 PASS, C3-C4 FAIL)",
            description: "Evaluates unconscious cortical friction reduction (92%) and waking executive reactivation resilience.",
            default_params: json!({ "n_agents": 1000, "seed": 42 }),
        },
        BatterySummary {
            id: "master-law",
            name: "Master Stability Law & Z3 Asymmetric BIBO Corridor",
            paper: "GTT §2 (v4.5.0)",
            verdict_v45: "VERIFIED (Lean 4 + Z3)",
            description: "Evaluates Master Law transition, Z3-verified asymmetric BIBO invariant bounds, and closed-loop damping.",
            default_params: json!({ "s_current": 0.50, "epsilon": 0.50, "lambda_plus": 0.02, "lambda_minus": 0.08, "leak_delta": 0.10, "s_base": 0.50 }),
        },
    ]
}

/// GET /api/batteries - List all live-evaluable mechanistic batteries
pub async fn list_batteries() -> impl IntoResponse {
    let catalog = battery_catalog();
    Json(json!({ "count": catalog.len(), "batteries": catalog }))
}

#[derive(Debug, Deserialize, Default)]
pub struct BatteryEvalRequest {
    #[serde(default)]
    pub params: Value,
}

/// POST /api/batteries/:id/evaluate - Evaluate a research battery live in sim-core
pub async fn evaluate_battery(
    Path(id): Path<String>,
    Json(req): Json<BatteryEvalRequest>,
) -> impl IntoResponse {
    let p = &req.params;
    match id.as_str() {
        "sim-p8c" => {
            let weights: Vec<f64> = p
                .get("weights")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(serde_json::Value::as_f64)
                        .filter(|x| x.is_finite() && *x >= 0.0)
                        .collect()
                })
                .filter(|v: &Vec<f64>| !v.is_empty())
                .unwrap_or_else(|| vec![10.0; 5]);
            let rate = p
                .get("raw_daily_rate")
                .and_then(serde_json::Value::as_f64)
                .filter(|v| v.is_finite() && *v > 0.0)
                .unwrap_or(2.0);
            let res = sim_core::v42_batteries::run_sim_p8c_provider_diversity(&weights, rate);
            (
                StatusCode::OK,
                Json(json!({ "battery_id": id, "result": res })),
            )
                .into_response()
        }
        "sim-p8d" => {
            let k_keys = p
                .get("k_keys")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(5)
                .clamp(1, 100) as usize;
            let key_cost = p
                .get("key_cost_usd")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(350.0)
                .clamp(1.0, 100_000.0);
            let v_attack = p
                .get("v_attack_daily")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(10.0)
                .clamp(0.1, 1000.0);
            let p_audit = p
                .get("p_audit_per_key")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.06)
                .clamp(0.001, 1.0);
            let res = sim_core::v42_batteries::run_sim_p8d_collusion_auction(
                k_keys, key_cost, v_attack, p_audit,
            );
            (
                StatusCode::OK,
                Json(json!({ "battery_id": id, "result": res })),
            )
                .into_response()
        }
        "sim-w9" => {
            let reserve = p
                .get("initial_reserve_usd")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(500_000.0)
                .clamp(1_000.0, 50_000_000.0);
            let months = p
                .get("months")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(36)
                .clamp(1, 120) as usize;
            let res = sim_core::v42_batteries::run_sim_w9_lemons_ode(reserve, months);
            (
                StatusCode::OK,
                Json(json!({ "battery_id": id, "result": res })),
            )
                .into_response()
        }
        "sim-at2" => {
            use sim_core::v42_batteries::AssemblyMagnitude as AM;
            let spectrum = vec![
                json!({"label": "Water / Sub-Molecular", "ma": AM::SubMolecularPhysics as u32}),
                json!({"label": "Dopamine", "ma": AM::MolecularDopamine as u32}),
                json!({"label": "Serotonin", "ma": AM::MolecularSerotonin as u32}),
                json!({"label": "cAMP (Abiogenesis Razor)", "ma": AM::MolecularCampRazor as u32}),
                json!({"label": "ATP (Correction #18: > cAMP)", "ma": AM::MolecularAtp as u32}),
                json!({"label": "Cortisol", "ma": AM::MolecularCortisol as u32}),
                json!({"label": "Oxytocin", "ma": AM::MolecularOxytocin as u32}),
                json!({"label": "DARPP-32", "ma": AM::MacromolecularMemoryDarpp as u32}),
                json!({"label": "ADCY5 Coincidence Machine", "ma": AM::CoincidenceMachineAdcy5 as u32}),
                json!({"label": "TFP Substrate Ledger", "ma": AM::SocietalLedgerTfp as u32}),
                json!({"label": "Bioelectric Syncytium", "ma": AM::BioelectricSyncytium as u32}),
            ];
            (StatusCode::OK, Json(json!({ "battery_id": id, "result": { "spectrum": spectrum, "atp_gt_camp": AM::MolecularAtp > AM::MolecularCampRazor } }))).into_response()
        }
        _ => evaluate_battery_part2(&id, p),
    }
}

fn evaluate_battery_part2(id: &str, p: &Value) -> axum::response::Response {
    match id {
        "sim-at3" => {
            let disp = p
                .get("displacement_angstroms")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(4.0)
                .clamp(-20.0, 20.0);
            let repair = p
                .get("has_active_living_repair")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true);
            let (final_r_angstroms, is_fractured) =
                sim_core::v42_batteries::evaluate_sim_at3_quantum_morse_vs_living_repair(
                    disp, repair,
                );
            (StatusCode::OK, Json(json!({
                "battery_id": id,
                "result": { "displacement_angstroms": disp, "has_active_living_repair": repair, "final_bond_distance_angstroms": final_r_angstroms, "is_fractured": is_fractured }
            }))).into_response()
        }
        "sim-thermo1" => {
            let temp_k = p
                .get("temp_kelvin")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(310.15)
                .clamp(1.0, 2000.0);
            let joules_per_bit = sim_core::v42_batteries::run_sim_thermo1_landauer_limit(temp_k);
            (StatusCode::OK, Json(json!({ "battery_id": id, "result": { "temp_kelvin": temp_k, "landauer_limit_joules_per_bit": joules_per_bit } }))).into_response()
        }
        "sim-patch1" => {
            let camp = p
                .get("camp_concentration_um")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.50)
                .clamp(0.01, 2.0);
            let ratio = sim_core::v42_batteries::run_sim_patch1_hcn1_transfer_ratio(camp);
            (StatusCode::OK, Json(json!({ "battery_id": id, "result": { "camp_concentration_um": camp, "shunting_gain_ratio": ratio } }))).into_response()
        }
        "sim-d18" => {
            use rand::{rngs::StdRng, Rng, SeedableRng};
            let n_agents = p
                .get("n_agents")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(1000)
                .clamp(50, 5000) as usize;
            let seed = p
                .get("seed")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(42);
            let mut rng = StdRng::seed_from_u64(seed);
            let kappas: Vec<f32> = (0..n_agents)
                .map(|_| {
                    let u1: f32 = rng.gen_range(0.001..1.0);
                    let u2: f32 = rng.gen_range(0.001..1.0);
                    (u1.powf(0.5) * u2.powf(0.2)).clamp(0.05, 0.95)
                })
                .collect();
            let res = sim_core::baylor_gating::run_baylor_simulation(&kappas, seed);
            (
                StatusCode::OK,
                Json(json!({ "battery_id": id, "result": res })),
            )
                .into_response()
        }
        "master-law" => {
            let s_curr = p
                .get("s_current")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.50);
            let eps = p
                .get("epsilon")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.50);
            let lp = p
                .get("lambda_plus")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.02)
                .clamp(0.0, 2.0);
            let lm = p
                .get("lambda_minus")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.08)
                .clamp(0.0, 2.0);
            let delta = p
                .get("leak_delta")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.10)
                .clamp(0.0, 1.0);
            let s_base = p
                .get("s_base")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.50);
            let cfg = sim_core::master_law::MasterLawConfig {
                preset: sim_core::master_law::MasterLawPreset::Custom,
                lambda_plus: lp,
                lambda_minus: lm,
                recognition_gate: 1.0,
                leak_delta: delta,
                s_base,
                scale_s_d: 1.0,
                corrective_bandwidth_mu: 1.0,
            };
            let s_next = cfg.step(s_curr, eps);
            let bounds = cfg.asymmetric_invariant_bounds();
            let damping = cfg.analyze_closed_loop_damping(lp);
            (StatusCode::OK, Json(json!({
                "battery_id": id,
                "result": { "s_next": s_next, "asymmetric_invariant_bounds": bounds, "closed_loop_damping": damping }
            }))).into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Battery not found", "id": id })),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_battery_list_and_p8c_evaluate() {
        assert!(battery_catalog().len() >= 9);
        let resp = evaluate_battery(
            Path("sim-p8c".to_string()),
            Json(BatteryEvalRequest {
                params: json!({ "weights": [100.0], "raw_daily_rate": 2.0 }),
            }),
        )
        .await
        .into_response();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), 65536).await.unwrap();
        let val: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(val["result"]["is_monopoly_blocked"], true);
    }

    #[tokio::test]
    async fn test_master_law_battery_returns_z3_bounds() {
        let resp = evaluate_battery(
            Path("master-law".to_string()),
            Json(BatteryEvalRequest { params: json!({ "s_current": 0.5, "epsilon": 1.0, "lambda_plus": 0.02, "lambda_minus": 0.08, "leak_delta": 0.10, "s_base": 0.5 }) }),
        ).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), 65536).await.unwrap();
        let val: Value = serde_json::from_slice(&body).unwrap();
        assert!(
            (val["result"]["asymmetric_invariant_bounds"][0]
                .as_f64()
                .unwrap()
                - (-0.30))
                .abs()
                < 1e-12
        );
        assert!(
            (val["result"]["asymmetric_invariant_bounds"][1]
                .as_f64()
                .unwrap()
                - 0.70)
                .abs()
                < 1e-12
        );
    }
}
