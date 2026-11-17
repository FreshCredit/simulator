#!/usr/bin/env python3
"""
DOCAS-PILOT-1: Daily Telemetry Harvester & Confounder Isolation Engine
Protocol Reference: DOCAS_PILOT_1_HUMAN_PROTOCOL.md §4.A, §5 & §6

Compiles continuous 1-minute epoch telemetry (1,440 epochs/day) across all 25 streams
from real edge database tables, applies the 4 pre-registered confounder isolation filters,
verifies stopping rules, and exports Snappy-compressed columnar Parquet files.
"""

import argparse
import hashlib
import json
import math
import os
import sqlite3
import sys
import time
from datetime import datetime, date, timedelta, timezone
from typing import Any, Dict, List, Optional, Tuple

SCHEMA_HARVEST_SQL = """
PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

CREATE TABLE IF NOT EXISTS daily_telemetry_master (
    epoch_id TEXT PRIMARY KEY,
    subject_id TEXT NOT NULL,
    cohort_stage INTEGER NOT NULL,
    epoch_timestamp DATETIME NOT NULL,
    day_index INTEGER NOT NULL,
    minute_of_day INTEGER NOT NULL CHECK (minute_of_day BETWEEN 0 AND 1439),
    
    -- Channel D: Expectation (Dopamine & Prefrontal Hemodynamics)
    fnirs_hbo2_umol REAL,
    fnirs_hbr_umol REAL,
    fnirs_dpf_calibrated REAL,
    eeg_midline_theta_uv2 REAL,
    eeg_alpha_asymmetry REAL,
    eeg_beta_alpha_arousal REAL,
    motor_cv_iti_pct REAL,
    
    -- Channel O: Validation (Vagal Parasympathetic Tone)
    hrv_rmssd_raw_ms REAL,
    hrv_rmssd_stress_ms REAL,
    heart_rate_bpm REAL,
    resting_hr_bpm REAL,
    walking_hr_bpm REAL,
    respiratory_rate_brpm REAL,
    smi_inspiratory_hold_sec REAL,
    airofit_mip_cmh2o REAL,
    airofit_mep_cmh2o REAL,
    airofit_vc_liters REAL,
    
    -- Channel C: Recognition (Cortisol / Stress & Metabolic Fuel)
    glucose_raw_mgdl REAL,
    glucose_stress_mgdl REAL,
    glucose_rate_dg_dt REAL,
    sleeping_wrist_temp_c REAL,
    delta_wrist_temp_c REAL,
    blood_pressure_systolic REAL,
    blood_pressure_diastolic REAL,
    vascular_dipping_pct REAL,
    
    -- Channel A: Activation (Noradrenaline / Somatic Mobilization)
    active_energy_kcal REAL,
    workout_met_intensity REAL,
    step_count INTEGER,
    ambient_co2_ppm REAL,
    ambient_voc_ppb REAL,
    ambient_pm25 REAL,
    
    -- Channel S: Stabilization (Circadian / Serotonin & Sleep Recovery)
    sleep_stage TEXT,
    sws_deep_sleep_pct REAL,
    rem_sleep_pct REAL,
    photic_zeitgeber_index REAL,
    bedroom_pir_motion INTEGER,
    enteric_bristol_type INTEGER,
    
    -- Financial Real-World Shocks (Plaid OAuth Ledger)
    plaid_financial_shock_usd REAL,
    plaid_prediction_error_amount REAL,
    trust_event_outcome TEXT,
    
    -- Behavioral Reading NLP
    reading_pages_count INTEGER,
    reading_flesch_kincaid REAL,
    reading_time_to_digitize_sec REAL,
    
    -- Quality Gates & Pharmacokinetic Confounders
    caffeine_active_mg REAL DEFAULT 0.0,
    thc_active_mg REAL DEFAULT 0.0,
    cog_ez_ddm_drift_v REAL,
    cog_ez_ddm_boundary_a REAL,
    cog_ez_ddm_ter_s REAL,
    cog_hick_slope_ms_per_bit REAL,
    cog_d_prime REAL,
    cog_plv_r_sync REAL,
    cog_nma_ms REAL,
    cog_max_span INTEGER,
    cog_practice_residual_ms REAL,
    fnirs_motion_artifact_pct REAL DEFAULT 0.0,
    stopping_rule_triggered INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_telemetry_master_time ON daily_telemetry_master(subject_id, epoch_timestamp);
CREATE INDEX IF NOT EXISTS idx_telemetry_master_day ON daily_telemetry_master(day_index);
"""

def _parse_local_hour(ts_str: str, fallback_offset_hours: float = 0.0) -> Optional[float]:
    """Parse ISO-8601 timestamp into local wall-clock fractional hour [0, 24)."""
    if not ts_str:
        return None
    try:
        clean = ts_str.strip()
        if clean.endswith("Z"):
            dt = datetime.fromisoformat(clean.replace("Z", "+00:00"))
            dt_local = dt + timedelta(hours=fallback_offset_hours)
        else:
            dt = datetime.fromisoformat(clean)
            if dt.tzinfo is not None:
                dt_local = dt
            else:
                dt_local = dt + timedelta(hours=fallback_offset_hours)
        return dt_local.hour + dt_local.minute / 60.0 + dt_local.second / 3600.0
    except Exception:
        return None

def _pk_active_load(events: List[Tuple[float, float, float]], t_sec: float) -> float:
    """First-order exponential PK accumulation across prior doses (dose, t_dose_sec, half_life_hours)."""
    total = 0.0
    for t_dose, dose_val, hl_h in events:
        dt_h = (t_sec - t_dose) / 3600.0
        if 0.0 <= dt_h <= hl_h * 10.0 and hl_h > 0.0:
            total += dose_val * math.pow(0.5, dt_h / hl_h)
    return round(total, 4)

# ─────────────────────────────────────────────────────────────────────────────
# 4 Confounder Isolation Mathematical Filters
# ─────────────────────────────────────────────────────────────────────────────

def dietary_glycemic_filter(glucose_mgdl: float, carbs_g: float, minutes_post_meal: float) -> float:
    """Filter 1: G_stress(t) = G_measured(t) - G_postprandial(t; Carbs, t_meal)."""
    if carbs_g <= 0.0 or minutes_post_meal < 0.0 or minutes_post_meal > 180.0:
        return glucose_mgdl
    tau_peak = 45.0
    clearance_kernel = (minutes_post_meal / tau_peak) * math.exp(1.0 - minutes_post_meal / tau_peak)
    expected_delta_glucose = (carbs_g * 0.45) * clearance_kernel
    residual_stress_glucose = glucose_mgdl - expected_delta_glucose
    return max(50.0, residual_stress_glucose)

def exertion_tachycardia_filter(hrv_rmssd_ms: float, mets: float, minutes_post_workout: float) -> float:
    """Filter 2: HRV_stress(t) = HRV_measured(t) - HRV_exertion(t; METs, t_workout)."""
    if mets <= 1.2 or minutes_post_workout < 0.0 or minutes_post_workout > 120.0:
        return hrv_rmssd_ms
    tau_rec = 30.0
    recovery_suppression = (mets * 2.8) * math.exp(-minutes_post_workout / tau_rec)
    unconfounded_hrv = hrv_rmssd_ms + recovery_suppression
    return max(5.0, unconfounded_hrv)

def cranial_dpf_calibration(base_dpf: float, body_fat_pct: float, age_years: float = 30.0) -> float:
    """Filter 3: Individualized cranial Differential Pathlength Factor DPF_u(lambda)."""
    gamma_bia = 0.012
    dpf_age_adjusted = base_dpf * (1.0 + 0.002 * (age_years - 25.0))
    dpf_calibrated = dpf_age_adjusted * (1.0 + gamma_bia * (body_fat_pct - 20.0))
    return max(4.0, min(8.5, dpf_calibrated))

def photic_zeitgeber_filter(lux: float, cct_kelvin: float) -> float:
    """Filter 4: Circadian Photic Zeitgeber Index."""
    safe_lux = max(0.0, lux)
    safe_cct = max(1000.0, cct_kelvin)
    return math.log(1.0 + safe_lux) * (safe_cct / 5500.0)

def _ar1_lag1(series: List[float]) -> float:
    if len(series) < 3:
        return 0.0
    m = sum(series) / len(series)
    dev = [v - m for v in series]
    denom = sum(d * d for d in dev)
    if denom <= 1e-12:
        return 0.0
    num = sum(dev[i] * dev[i + 1] for i in range(len(dev) - 1))
    return max(-0.95, min(0.95, num / denom))

def _jcs_format_number(val: float) -> str:
    """Serialize finite Python float using exact ECMA-262 §6.1.6.1.20 / RFC 8785 JCS rules."""
    from decimal import Decimal
    if not math.isfinite(val):
        return "null"
    if val == 0.0:
        return "0"
    d = Decimal(repr(val))
    sign_str = "-" if d.is_signed() else ""
    d_abs = abs(d)
    tup = d_abs.as_tuple()
    digits = "".join(str(x) for x in tup.digits).rstrip("0") or "0"
    n = d_abs.adjusted() + 1  # number of digits left of decimal point (n = k + exp)
    k = len(digits)
    if k <= n <= 21:
        return f"{sign_str}{digits}{'0' * (n - k)}"
    if 0 < n <= 21:
        return f"{sign_str}{digits[:n]}.{digits[n:]}"
    if -6 < n <= 0:
        return f"{sign_str}0.{'0' * (-n)}{digits}"
    exp = n - 1
    exp_sign = "+" if exp >= 0 else "-"
    if k == 1:
        return f"{sign_str}{digits}e{exp_sign}{abs(exp)}"
    return f"{sign_str}{digits[0]}.{digits[1:]}e{exp_sign}{abs(exp)}"

def canonicalize_jcs(obj: Any) -> str:
    """IETF RFC 8785 JSON Canonicalization Scheme (JCS) across Python and ECMAScript."""
    if obj is None:
        return "null"
    if isinstance(obj, bool):
        return "true" if obj else "false"
    if isinstance(obj, int):
        return str(obj)
    if isinstance(obj, float):
        return _jcs_format_number(obj)
    if isinstance(obj, str):
        return json.dumps(obj, ensure_ascii=False)
    if isinstance(obj, (list, tuple)):
        return "[" + ",".join(canonicalize_jcs(v) for v in obj) + "]"
    if isinstance(obj, dict):
        items = sorted((str(k), v) for k, v in obj.items())
        return "{" + ",".join(f"{json.dumps(k, ensure_ascii=False)}:{canonicalize_jcs(v)}" for k, v in items) + "}"
    return json.dumps(obj, ensure_ascii=False)

def evaluate_ar1_concordance(x: List[Optional[float]], y: List[Optional[float]]) -> Dict[str, float]:
    """Pearson correlation with Quenouille/Zwiers–von Storch AR(1) effective sample size N_eff."""
    pairs = [
        (float(a), float(b))
        for a, b in zip(x, y)
        if a is not None and b is not None and math.isfinite(float(a)) and math.isfinite(float(b))
    ]
    n = len(pairs)
    if n < 5:
        return {"r": 0.0, "n_eff": float(n), "p_value": 1.0}
    xa = [p[0] for p in pairs]
    ya = [p[1] for p in pairs]
    mx, my = sum(xa) / n, sum(ya) / n
    dx = [v - mx for v in xa]
    dy = [v - my for v in ya]
    denom = math.sqrt(sum(a * a for a in dx) * sum(b * b for b in dy))
    if denom <= 1e-12:
        return {"r": 0.0, "n_eff": float(n), "p_value": 1.0}
    r = max(-0.9999, min(0.9999, sum(a * b for a, b in zip(dx, dy)) / denom))
    rx, ry = max(0.0, _ar1_lag1(xa)), max(0.0, _ar1_lag1(ya))
    # Quenouille effective degrees of freedom under joint AR(1) autocorrelation
    n_eff = max(4.0, float(n) * (1.0 - rx * ry) / (1.0 + rx * ry))
    se = math.sqrt(max(1e-12, (1.0 - r * r) / max(2.0, n_eff - 2.0)))
    z = abs(r) / se
    p_val = max(0.0, min(1.0, math.erfc(z / math.sqrt(2.0))))
    return {"r": round(r, 4), "n_eff": round(n_eff, 2), "p_value": round(p_val, 6)}

BUNDLE_ALLOWED_TABLES = (
    "cognitive_battery_sessions",
    "cognitive_battery_trials",
    "environmental_context_stream",
    "respiratory_strength_stream",
    "pharmacological_exposure_stream",
    "cuffless_bp_stream",
    "biomarker_panel_manifest",
    "salivary_cortisol_car",
    "cellular_remodeling_manifest",
    "screen_app_telemetry_stream",
    "media_listening_archive_stream",
    "commerce_order_archive_stream",
    "deidentified_text_nlp_stream",
    "sustained_inhalation_stream",
    "manual_reading_log_stream",
    "sleep_boundary_pir_stream",
    "enteric_motility_stream",
    "wetlab_eeg_fnirs_sessions",
    "hair_metabolomics_stream",
    "genomic_wgs_manifest",
    "acoustic_debrief_stream",
    "video_debrief_stream",
    "healthkit_records",
    "healthkit_workouts",
)

def normalize_censored_hscrp(raw_value: Any, loq_mg_l: float = 0.2) -> Tuple[Optional[float], int]:
    """Handle left-censored '< 0.2 mg/L' hs-CRP strings via LOQ / sqrt(2)."""
    if raw_value is None:
        return None, 0
    if isinstance(raw_value, str):
        s = raw_value.strip()
        if s.startswith("<"):
            try:
                bound = float(s[1:].strip())
            except ValueError:
                bound = loq_mg_l
            return round(bound / math.sqrt(2.0), 4), 1
        try:
            return float(s), 0
        except ValueError:
            return None, 0
    return float(raw_value), 0

def compute_zrt_car_aucs(c0: float, c30: float, c45: float) -> Tuple[float, float]:
    """Compute Pruessner (2003) CAR AUC_G and AUC_I across 0, +30, +45 min samples."""
    auc_g = 0.5 * (c0 + c30) * 30.0 + 0.5 * (c30 + c45) * 15.0
    auc_i = auc_g - (c0 * 45.0)
    return round(auc_g, 4), round(auc_i, 4)

def evaluate_clinical_panic_check(panel: Dict[str, Any]) -> str:
    """Silent clinical panic-value gate: returns ONLY SAFE or CLINICAL_REVIEW_REQUIRED."""
    k = panel.get("cmp_potassium_meq_l")
    glu = panel.get("cmp_glucose_mg_dl")
    hba1c = panel.get("hba1c_pct")
    tsh = panel.get("tsh_miu_l")
    wbc = panel.get("cbc_wbc_k_ul")
    hb = panel.get("cbc_hemoglobin_g_dl")
    plt = panel.get("cbc_platelets_k_ul")
    crp = panel.get("hs_crp_mg_l")
    if (
        (k is not None and (k < 2.8 or k > 6.0))
        or (glu is not None and (glu < 50.0 or glu > 250.0))
        or (hba1c is not None and hba1c >= 6.5)
        or (tsh is not None and (tsh < 0.1 or tsh > 10.0))
        or (wbc is not None and (wbc < 2.0 or wbc > 20.0))
        or (hb is not None and hb < 10.0)
        or (plt is not None and (plt < 75.0 or plt > 600.0))
        or (crp is not None and crp > 10.0)
    ):
        return "CLINICAL_REVIEW_REQUIRED"
    return "SAFE"

def compute_bp_deconfounded_nvc(hbo2_umol: float, sbp_mmhg: float, sbp_baseline: float = 120.0, beta_bp: float = 0.035) -> float:
    """Triangulation #1: subtract systemic arterial BP surge from fNIRS HbO2 to isolate cortical NVC."""
    return round(float(hbo2_umol) - beta_bp * (float(sbp_mmhg) - float(sbp_baseline)), 4)

def compute_masking_divergence(somatic_arousal_z: float, facial_vocal_z: float, db_exposure: float = 45.0) -> float:
    """Triangulation #7: quantify internal sympathetic arousal masked beneath calm expression after removing dB load."""
    db_excess = max(0.0, (float(db_exposure) - 65.0) / 20.0)
    return round(max(0.0, float(somatic_arousal_z) - db_excess) - float(facial_vocal_z), 4)

def _warn_non_missing_sql_error(section: str, exc: Exception) -> None:
    msg = str(exc).lower()
    if isinstance(exc, sqlite3.OperationalError) and ("no such table" in msg or "no such column" in msg):
        return
    sys.stderr.write(f"[harvest_telemetry] {section} query warning: {exc}\n")

def decrypt_study_bundle(bundle: Dict[str, Any], passphrase: str) -> Dict[str, Any]:
    """Decrypt a `docas-pilot1-enc-v1` PBKDF2-SHA256 + AES-256-GCM study bundle."""
    import base64
    if not passphrase:
        raise ValueError("Passphrase is required to decrypt docas-pilot1-enc-v1 study bundle")
    from cryptography.hazmat.primitives.ciphers.aead import AESGCM
    salt = base64.b64decode(bundle["salt_b64"])
    iv = base64.b64decode(bundle["iv_b64"])
    ct = base64.b64decode(bundle["ciphertext_b64"])
    iters = int(bundle.get("kdf_iterations", 600_000))
    key = hashlib.pbkdf2_hmac("sha256", passphrase.encode("utf-8"), salt, iters, dklen=32)
    pt = AESGCM(key).decrypt(iv, ct, None)
    return json.loads(pt.decode("utf-8"))

class TelemetryHarvester:
    def __init__(self, db_path: str = "trust_wearables_edge.db", subject_id: str = "pilot-user-1", cohort_stage: int = 1, tz_offset_hours: float = 0.0):
        self.db_path = db_path
        self.subject_id = subject_id
        self.cohort_stage = cohort_stage
        self.tz_offset_hours = tz_offset_hours
        dirname = os.path.dirname(os.path.abspath(db_path))
        if dirname:
            os.makedirs(dirname, exist_ok=True)
        self.conn = sqlite3.connect(db_path)
        self.conn.executescript(SCHEMA_HARVEST_SQL)
        for col_def in (
            "caffeine_active_mg REAL DEFAULT 0.0",
            "thc_active_mg REAL DEFAULT 0.0",
            "cog_ez_ddm_drift_v REAL",
            "cog_ez_ddm_boundary_a REAL",
            "cog_ez_ddm_ter_s REAL",
            "cog_hick_slope_ms_per_bit REAL",
            "cog_d_prime REAL",
            "cog_plv_r_sync REAL",
            "cog_nma_ms REAL",
            "cog_max_span INTEGER",
            "cog_practice_residual_ms REAL",
        ):
            try:
                self.conn.execute(f"ALTER TABLE daily_telemetry_master ADD COLUMN {col_def}")
            except sqlite3.OperationalError:
                pass
        self.conn.commit()

    def import_study_bundle(self, bundle_path: str, verify_sha256: bool = True, passphrase: Optional[str] = None) -> Dict[str, int]:
        """Verify SHA-256 manifest (decrypting AES-256-GCM if needed) and ingest a `docas-pilot1-v1` study bundle."""
        import re
        with open(bundle_path, "r", encoding="utf-8") as f:
            bundle = json.load(f)
        if bundle.get("schema_version") == "docas-pilot1-enc-v1":
            bundle = decrypt_study_bundle(bundle, passphrase or "")
        if bundle.get("schema_version") != "docas-pilot1-v1":
            raise ValueError(f"Unsupported study bundle schema_version: {bundle.get('schema_version')}")
        resolved_subject = bundle.get("user_id") or bundle.get("pseudonymized_subject_id")
        if self.subject_id == "pilot-user-1" and resolved_subject:
            self.subject_id = str(resolved_subject)
        if verify_sha256 and bundle.get("sha256_manifest"):
            payload = {
                "schema_version": bundle["schema_version"],
                "exported_at": bundle["exported_at"],
            }
            if "user_id" in bundle:
                payload["user_id"] = bundle["user_id"]
            if "pseudonymized_subject_id" in bundle:
                payload["pseudonymized_subject_id"] = bundle["pseudonymized_subject_id"]
            payload["counts"] = bundle["counts"]
            payload["tables"] = bundle["tables"]
            canonical_jcs = canonicalize_jcs(payload).encode("utf-8")
            canonical_legacy = json.dumps(payload, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
            computed_jcs = hashlib.sha256(canonical_jcs).hexdigest()
            computed_legacy = hashlib.sha256(canonical_legacy).hexdigest()
            if bundle["sha256_manifest"] not in (computed_jcs, computed_legacy):
                raise ValueError(f"Study bundle SHA-256 mismatch: expected {bundle['sha256_manifest']}, got {computed_jcs}")

        imported: Dict[str, int] = {}
        tables = bundle.get("tables") or {}
        cur = self.conn.cursor()
        for tbl in BUNDLE_ALLOWED_TABLES:
            rows = tables.get(tbl) or []
            if not rows:
                imported[tbl] = 0
                continue
            existing_cols = {r[1] for r in cur.execute(f"PRAGMA table_info({tbl})").fetchall()}
            if not existing_cols:
                # Bootstrap minimal table from row keys when importing into an empty edge DB
                cols_init = [c for c in rows[0].keys() if re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", str(c))]
                col_defs = ", ".join(f"{c} PRIMARY KEY" if c == "id" else c for c in cols_init)
                cur.execute(f"CREATE TABLE IF NOT EXISTS {tbl} ({col_defs})")
                existing_cols = set(cols_init)
            count = 0
            for row in rows:
                cols = [c for c in row.keys() if c in existing_cols and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", str(c))]
                if not cols:
                    continue
                placeholders = ", ".join("?" for _ in cols)
                col_list = ", ".join(cols)
                cur.execute(f"INSERT OR REPLACE INTO {tbl} ({col_list}) VALUES ({placeholders})", [row[c] for c in cols])
                count += 1
            imported[tbl] = count
        self.conn.commit()
        return imported

    def _query_real_epoch_data(self, cur: sqlite3.Cursor, t_start_epoch: float, t_end_epoch: float, epoch_start_iso: str, epoch_end_iso: str) -> Dict[str, Any]:
        """Query real observations across existing tables for a 1-minute window."""
        res: Dict[str, Any] = {}
        
        # 1. Prefrontal fNIRS from muse_fnirs_mbll or wetlab_eeg_fnirs_sessions
        try:
            cur.execute("""
                SELECT AVG(delta_hbo2_umol), AVG(delta_hbr_umol), AVG(is_motion_artifact)
                FROM muse_fnirs_mbll
                WHERE (session_id IN (SELECT session_id FROM pilot_sessions WHERE subject_id = ?) OR session_id LIKE ? || '%')
                  AND timestamp_epoch >= ? AND timestamp_epoch < ?
            """, (self.subject_id, self.subject_id, t_start_epoch, t_end_epoch))
            row = cur.fetchone()
            if row and row[0] is not None:
                res["hbo2"] = round(float(row[0]), 3)
                res["hbr"] = round(float(row[1]), 3)
                res["motion_artifact_pct"] = float(row[2]) * 100.0
        except Exception as exc:
            _warn_non_missing_sql_error("fnirs_muse_mbll", exc)

        if res.get("hbo2") is None:
            try:
                cur.execute("""
                    SELECT fnirs_hbo2_umol, fnirs_hbr_umol, blink_artifact_pct
                    FROM wetlab_eeg_fnirs_sessions
                    WHERE user_id = ? AND unixepoch(started_at, 'subsec') >= ? AND unixepoch(started_at, 'subsec') < ?
                    ORDER BY unixepoch(started_at, 'subsec') DESC LIMIT 1
                """, (self.subject_id, t_start_epoch, t_end_epoch))
                w_row = cur.fetchone()
                if w_row and w_row[0] is not None:
                    res["hbo2"] = round(float(w_row[0]), 3)
                    res["hbr"] = round(float(w_row[1]), 3)
                    res["motion_artifact_pct"] = float(w_row[2] or 0.0)
            except Exception as exc:
                _warn_non_missing_sql_error("fnirs_wetlab", exc)

        # 2. Ambient Environment (Awair)
        try:
            cur.execute("""
                SELECT co2_ppm, voc_ppb, pm25_ug_m3
                FROM ambient_environment_stream
                WHERE user_id = ? AND unixepoch(ts_utc, 'subsec') >= ? AND unixepoch(ts_utc, 'subsec') < ?
                ORDER BY unixepoch(ts_utc, 'subsec') DESC LIMIT 1
            """, (self.subject_id, t_start_epoch, t_end_epoch))
            row = cur.fetchone()
            if row:
                res["co2"] = row[0]
                res["voc"] = row[1]
                res["pm25"] = row[2]
        except Exception as exc:
            _warn_non_missing_sql_error("ambient_env", exc)

        # 3. Respiratory Strength (Airofit Pro MIP/MEP/VC, with legacy SMI fallback)
        try:
            cur.execute("""
                SELECT best_mip_cmh2o, mep_cmh2o, vc_liters FROM respiratory_strength_stream
                WHERE user_id = ? AND unixepoch(ts_utc, 'subsec') >= ? AND unixepoch(ts_utc, 'subsec') < ?
                ORDER BY unixepoch(ts_utc, 'subsec') DESC LIMIT 1
            """, (self.subject_id, t_start_epoch, t_end_epoch))
            row = cur.fetchone()
            if row:
                res["airofit_mip"] = row[0]
                res["airofit_mep"] = row[1]
                res["airofit_vc"] = row[2]
        except Exception as exc:
            _warn_non_missing_sql_error("respiratory_strength", exc)
        if "airofit_mip" not in res:
            try:
                cur.execute("""
                    SELECT best_hold_sec FROM sustained_inhalation_stream
                    WHERE user_id = ? AND unixepoch(ts_utc, 'subsec') >= ? AND unixepoch(ts_utc, 'subsec') < ?
                    ORDER BY unixepoch(ts_utc, 'subsec') DESC LIMIT 1
                """, (self.subject_id, t_start_epoch, t_end_epoch))
                row = cur.fetchone()
                if row:
                    res["smi_hold"] = row[0]
            except Exception as exc:
                _warn_non_missing_sql_error("sustained_inhalation", exc)

        # 4. Reading Logs
        try:
            cur.execute("""
                SELECT pages_read, flesch_kincaid_grade, latency_sec
                FROM manual_reading_log_stream
                WHERE user_id = ? AND unixepoch(digitized_utc, 'subsec') >= ? AND unixepoch(digitized_utc, 'subsec') < ?
                ORDER BY unixepoch(digitized_utc, 'subsec') DESC LIMIT 1
            """, (self.subject_id, t_start_epoch, t_end_epoch))
            row = cur.fetchone()
            if row:
                res["pages"] = row[0]
                res["fk_grade"] = row[1]
                res["read_latency"] = row[2]
        except Exception as exc:
            _warn_non_missing_sql_error("reading_logs", exc)

        # 5. Financial Shocks
        try:
            cur.execute("""
                SELECT settled_amount, prediction_error_amount, trust_outcome
                FROM financial_trust_ledger
                WHERE user_id = ? AND unixepoch(settled_datetime, 'subsec') >= ? AND unixepoch(settled_datetime, 'subsec') < ?
                ORDER BY unixepoch(settled_datetime, 'subsec') DESC LIMIT 1
            """, (self.subject_id, t_start_epoch, t_end_epoch))
            row = cur.fetchone()
            if row:
                res["plaid_shock"] = row[0]
                res["plaid_pred_err"] = row[1]
                res["plaid_outcome"] = row[2]
        except Exception as exc:
            _warn_non_missing_sql_error("financial_trust_ledger", exc)

        # 6. Muse EEG Features
        try:
            cur.execute("""
                SELECT fm_theta_uv2, frontal_alpha_asymmetry, theta_beta_ratio, sws_ratio
                FROM muse_eeg_features
                WHERE (session_id IN (SELECT session_id FROM pilot_sessions WHERE subject_id = ?) OR session_id LIKE ? || '%')
                  AND timestamp_epoch >= ? AND timestamp_epoch < ?
                ORDER BY timestamp_epoch DESC LIMIT 1
            """, (self.subject_id, self.subject_id, t_start_epoch, t_end_epoch))
            row = cur.fetchone()
            if row and row[0] is not None:
                res["fm_theta"], res["faa"], res["tbr"] = row[0], row[1], row[2]
                if row[2] is not None and float(row[2]) > 0.0:
                    res["beta_alpha_arousal"] = 1.0 / float(row[2])
                if len(row) > 3 and row[3] is not None:
                    res["sws_ratio"] = row[3]
        except Exception as exc:
            _warn_non_missing_sql_error("eeg_muse_features", exc)

        if res.get("fm_theta") is None:
            try:
                cur.execute("""
                    SELECT frontal_midline_theta_uv2, frontal_alpha_asymmetry, beta_alpha_arousal_ratio, raw_features_json
                    FROM wetlab_eeg_fnirs_sessions
                    WHERE user_id = ? AND unixepoch(started_at, 'subsec') >= ? AND unixepoch(started_at, 'subsec') < ?
                    ORDER BY unixepoch(started_at, 'subsec') DESC LIMIT 1
                """, (self.subject_id, t_start_epoch, t_end_epoch))
                w_row = cur.fetchone()
                if w_row and w_row[0] is not None:
                    res["fm_theta"], res["faa"] = w_row[0], w_row[1]
                    arousal_val = float(w_row[2]) if w_row[2] is not None else None
                    if arousal_val is None and len(w_row) > 3 and w_row[3]:
                        try:
                            raw_feat = json.loads(w_row[3])
                            if raw_feat.get("beta_alpha_ratio") is not None:
                                arousal_val = float(raw_feat["beta_alpha_ratio"])
                        except Exception:
                            pass
                    res["beta_alpha_arousal"] = arousal_val
                    res["tbr"] = (1.0 / max(1e-4, arousal_val)) if arousal_val is not None else None
            except Exception as exc:
                _warn_non_missing_sql_error("eeg_wetlab", exc)

        # 7. Cognitive Battery Striatal Tapping
        try:
            cur.execute("""
                SELECT cv_iti_pct FROM cognitive_battery_tapping
                WHERE user_id = ? AND unixepoch(started_at, 'subsec') >= ? AND unixepoch(started_at, 'subsec') < ?
                ORDER BY unixepoch(started_at, 'subsec') DESC LIMIT 1
            """, (self.subject_id, t_start_epoch, t_end_epoch))
            row = cur.fetchone()
            if row:
                res["cv_iti_pct"] = row[0]
        except Exception:
            pass

        # 8. Apple HealthKit Records / Dexcom CGM / Sleeping Wrist Temp / Sleep Analysis / Ambient Daylight Fallback
        try:
            cur.execute("""
                SELECT record_type, value, metadata, source_name, unit FROM healthkit_records
                WHERE user_id = ? AND (
                    (unixepoch(start_date, 'subsec') >= ? AND unixepoch(start_date, 'subsec') < ?)
                    OR ((record_type LIKE '%SleepAnalysis%' OR record_type LIKE '%AppleSleepingWristTemperature%')
                        AND unixepoch(start_date, 'subsec') < ? AND unixepoch(end_date, 'subsec') > ?)
                )
            """, (self.subject_id, t_start_epoch, t_end_epoch, t_end_epoch, t_start_epoch))
            for rec in cur.fetchall():
                t, v = rec[0], rec[1]
                m = rec[2] if len(rec) > 2 else None
                s_name = str(rec[3] if len(rec) > 3 else "unknown").lower()
                rec_unit = str(rec[4] if len(rec) > 4 else "").strip().lower()
                is_wearable = any(w in s_name for w in ["watch", "oura", "whoop", "garmin"])
                is_rmssd = "HeartRateVariabilityRMSSD" in t or (m and "rmssd_heartbeat_series" in str(m))
                is_sdnn = "HeartRateVariabilitySDNN" in t or ("HeartRateVariability" in t and not is_rmssd)
                if is_rmssd:
                    res["raw_rmssd"] = v
                    res["_rmssd_is_beat_to_beat"] = True
                elif is_sdnn:
                    if not res.get("_rmssd_is_beat_to_beat", False):
                        res["raw_rmssd"] = v
                elif "WalkingHeartRateAverage" in t:
                    res["walking_hr"] = v
                elif "RestingHeartRate" in t:
                    res["resting_hr"] = v
                elif "HeartRate" in t:
                    res["raw_hr"] = v
                elif "BloodGlucose" in t:
                    g_val = float(v)
                    if "mmol" in rec_unit:
                        g_val *= 18.0182
                    res["raw_glucose"] = round(g_val, 3)
                elif "AppleSleepingWristTemperature" in t:
                    temp_c = float(v)
                    if "degf" in rec_unit or "°f" in rec_unit or "fahrenheit" in rec_unit or temp_c > 60.0:
                        temp_c = (temp_c - 32.0) * (5.0 / 9.0)
                    res["wrist_temp"] = round(temp_c, 3)
                elif "BloodPressureSystolic" in t:
                    res["sbp"] = v
                elif "BloodPressureDiastolic" in t:
                    res["dbp"] = v
                elif "DietaryCarbohydrates" in t:
                    if "_carbs_set" not in res:
                        res["carbs_g"] = v
                        res["_carbs_set"] = True
                elif "RespiratoryRate" in t:
                    res["resp_rate"] = v
                elif "ActiveEnergy" in t:
                    if is_wearable:
                        res["active_energy"] = v
                        res["_energy_is_wearable"] = True
                    elif not res.get("_energy_is_wearable", False):
                        res["active_energy"] = v
                elif "StepCount" in t:
                    if is_wearable:
                        res["steps"] = int(v)
                        res["_steps_is_wearable"] = True
                    elif not res.get("_steps_is_wearable", False):
                        res["steps"] = int(v)
                elif "AmbientLight" in t or "AmbientLux" in t or ("TimeInDaylight" in t and rec_unit == "lux"):
                    if res.get("lux") is None:
                        res["lux"] = float(v)
                elif "TimeInDaylight" in t:
                    res["time_in_daylight_min"] = float(v)
                elif "SleepAnalysis" in t:
                    code = int(float(v)) if v is not None else 1
                    if code == 5:
                        res["sleep_stage"] = "REM"
                    elif code == 4:
                        res["sleep_stage"] = "Deep"
                    elif code == 3:
                        res["sleep_stage"] = "Core"
                    elif code == 2:
                        res["sleep_stage"] = "Awake"
                    else:
                        res["sleep_stage"] = "Asleep"
            res.pop("_rmssd_is_beat_to_beat", None)
            res.pop("_steps_is_wearable", None)
            res.pop("_energy_is_wearable", None)
            res.pop("_carbs_set", None)
        except Exception:
            pass

        # 9. Sleep Boundary PIR & Light
        try:
            cur.execute("""
                SELECT motion_detected, ambient_lux FROM sleep_boundary_pir_stream
                WHERE user_id = ? AND unixepoch(ts_utc, 'subsec') >= ? AND unixepoch(ts_utc, 'subsec') < ?
                ORDER BY unixepoch(ts_utc, 'subsec') DESC LIMIT 1
            """, (self.subject_id, t_start_epoch, t_end_epoch))
            row = cur.fetchone()
            if row:
                if row[0] is not None:
                    res["pir_motion"] = row[0]
                if row[1] is not None:
                    res["lux"] = row[1]
        except Exception:
            pass

        # 10. Enteric Motility (Throne)
        try:
            cur.execute("""
                SELECT bristol_type FROM enteric_motility_stream
                WHERE user_id = ? AND unixepoch(ts_utc, 'subsec') >= ? AND unixepoch(ts_utc, 'subsec') < ?
                ORDER BY unixepoch(ts_utc, 'subsec') DESC LIMIT 1
            """, (self.subject_id, t_start_epoch, t_end_epoch))
            row = cur.fetchone()
            if row:
                res["bristol"] = row[0]
        except Exception:
            pass

        # 11. Migration 029 Unified Environmental Context (AQI, PM2.5, Indoor Lux, Kelvin CCT)
        try:
            cur.execute("""
                SELECT pm2_5_ug_m3, co2_ppm, voc_ppb, indoor_lux, indoor_kelvin_cct
                FROM environmental_context_stream
                WHERE user_id = ? AND unixepoch(ts_utc, 'subsec') >= ? AND unixepoch(ts_utc, 'subsec') < ?
                ORDER BY unixepoch(ts_utc, 'subsec') DESC LIMIT 1
            """, (self.subject_id, t_start_epoch, t_end_epoch))
            row = cur.fetchone()
            if row:
                if row[0] is not None and res.get("pm25") is None:
                    res["pm25"] = float(row[0])
                if row[1] is not None and res.get("co2") is None:
                    res["co2"] = float(row[1])
                if row[2] is not None and res.get("voc") is None:
                    res["voc"] = float(row[2])
                if row[3] is not None:
                    res["lux"] = float(row[3])
                if row[4] is not None:
                    res["cct_kelvin"] = float(row[4])
        except Exception:
            pass

        # 12. Migration 029 Respiratory Strength (Airofit MIP), Cuffless BP (Aktiia), Adaptive Cognitive
        try:
            cur.execute("""
                SELECT mip_cmh2o FROM respiratory_strength_stream
                WHERE user_id = ? AND unixepoch(ts_utc, 'subsec') >= ? AND unixepoch(ts_utc, 'subsec') < ?
                ORDER BY unixepoch(ts_utc, 'subsec') DESC LIMIT 1
            """, (self.subject_id, t_start_epoch, t_end_epoch))
            row = cur.fetchone()
            if row and row[0] is not None and res.get("smi_hold") is None:
                res["smi_hold"] = float(row[0])
        except Exception as exc:
            _warn_non_missing_sql_error("respiratory_strength", exc)

        try:
            cur.execute("""
                SELECT sbp_mmhg, dbp_mmhg, nocturnal_dip_pct FROM cuffless_bp_stream
                WHERE user_id = ? AND unixepoch(ts_utc, 'subsec') >= ? AND unixepoch(ts_utc, 'subsec') < ?
                ORDER BY unixepoch(ts_utc, 'subsec') DESC LIMIT 1
            """, (self.subject_id, t_start_epoch, t_end_epoch))
            row = cur.fetchone()
            if row:
                if row[0] is not None and res.get("sbp") is None:
                    res["sbp"] = float(row[0])
                if row[1] is not None and res.get("dbp") is None:
                    res["dbp"] = float(row[1])
                if row[2] is not None:
                    res["dipping_pct"] = float(row[2])
        except Exception as exc:
            _warn_non_missing_sql_error("cuffless_bp", exc)

        try:
            cur.execute("""
                SELECT task_type, mean_rt_ms, sigma_clock_sq, sigma_motor_sq, metrics_json,
                       ez_ddm_drift_v, ez_ddm_boundary_a, ez_ddm_ter_s, hick_slope_ms_per_bit,
                       d_prime, plv_r_sync, nma_ms, max_span, practice_residual_ms
                FROM cognitive_battery_sessions
                WHERE user_id = ? AND unixepoch(started_at, 'subsec') >= ? AND unixepoch(started_at, 'subsec') < ?
                ORDER BY unixepoch(started_at, 'subsec') DESC LIMIT 10
            """, (self.subject_id, t_start_epoch, t_end_epoch))
            for row in cur.fetchall():
                if row[0] == "beat_sync" and res.get("cv_iti_pct") is None:
                    mean_iti = float(row[1]) if row[1] is not None else 0.0
                    cv_pct = None
                    if row[4]:
                        try:
                            meta = json.loads(row[4])
                            if meta.get("cv_iti_pct") is not None:
                                cv_pct = float(meta["cv_iti_pct"])
                            elif meta.get("sd_iti_ms") is not None and mean_iti > 0:
                                cv_pct = (float(meta["sd_iti_ms"]) / mean_iti) * 100.0
                        except Exception:
                            pass
                    if cv_pct is None and mean_iti > 0 and row[2] is not None and row[3] is not None:
                        sd_est = math.sqrt(max(0.0, float(row[2]) + 2.0 * float(row[3])))
                        cv_pct = (sd_est / mean_iti) * 100.0
                    if cv_pct is not None:
                        res["cv_iti_pct"] = round(cv_pct, 3)
                for idx, key in (
                    (5, "cog_ez_ddm_drift_v"), (6, "cog_ez_ddm_boundary_a"), (7, "cog_ez_ddm_ter_s"),
                    (8, "cog_hick_slope_ms_per_bit"), (9, "cog_d_prime"), (10, "cog_plv_r_sync"),
                    (11, "cog_nma_ms"), (12, "cog_max_span"), (13, "cog_practice_residual_ms"),
                ):
                    if len(row) > idx and row[idx] is not None and res.get(key) is None:
                        res[key] = int(row[idx]) if key == "cog_max_span" else float(row[idx])
        except Exception as exc:
            _warn_non_missing_sql_error("cognitive_battery", exc)

        if res.get("cog_d_prime") is None:
            try:
                cur.execute("""
                    SELECT d_prime FROM cognitive_battery_gonogo
                    WHERE user_id = ? AND unixepoch(started_at, 'subsec') >= ? AND unixepoch(started_at, 'subsec') < ?
                    ORDER BY unixepoch(started_at, 'subsec') DESC LIMIT 1
                """, (self.subject_id, t_start_epoch, t_end_epoch))
                g_row = cur.fetchone()
                if g_row and g_row[0] is not None:
                    res["cog_d_prime"] = float(g_row[0])
            except Exception as exc:
                _warn_non_missing_sql_error("gonogo_028", exc)

        return res

    def harvest_day(self, target_date: date, day_index: int, body_fat_pct: float = 18.5, simulated: bool = False) -> Dict[str, Any]:
        """Compile all 25 streams across 1,440 one-minute epochs for target_date."""
        calibrated_dpf = cranial_dpf_calibration(6.0, body_fat_pct)
        epochs: List[Tuple] = []
        motion_artifact_total_pct = 0.0
        recorded_fnirs_epochs = 0
        cur = self.conn.cursor()

        # Pre-load meals and workouts across [target_date - 4h, target_date + 24h]
        day_start_dt = datetime(target_date.year, target_date.month, target_date.day, 0, 0, 0, tzinfo=timezone.utc)
        scan_start_sec = (day_start_dt - timedelta(hours=4)).timestamp()
        scan_end_sec = (day_start_dt + timedelta(hours=24)).timestamp()

        meal_events: List[Tuple[float, float]] = []
        try:
            cur.execute("""
                SELECT start_date, value FROM healthkit_records
                WHERE user_id = ? AND record_type LIKE '%DietaryCarbohydrates%'
                  AND unixepoch(start_date, 'subsec') >= ? AND unixepoch(start_date, 'subsec') <= ?
            """, (self.subject_id, scan_start_sec, scan_end_sec))
            for r in cur.fetchall():
                ts_str, c_val = r[0], float(r[1])
                t_sec = datetime.fromisoformat(ts_str.replace("Z", "+00:00")).timestamp()
                meal_events.append((t_sec, c_val))
        except Exception:
            pass

        workout_events: List[Tuple[float, float, float]] = []
        try:
            cur.execute("""
                SELECT start_date, end_date, total_energy_burned, duration, duration_unit FROM healthkit_workouts
                WHERE user_id = ? AND unixepoch(start_date, 'subsec') >= ? AND unixepoch(start_date, 'subsec') <= ?
            """, (self.subject_id, scan_start_sec, scan_end_sec))
            for r in cur.fetchall():
                start_str, end_str, kcal, raw_dur = r[0], r[1], float(r[2] or 0.0), float(r[3] or 30.0)
                dur_unit = str(r[4] or "").strip().lower() if len(r) > 4 else ""
                # Unit-dimensional normalization: convert seconds to minutes if duration_unit is 's'/'sec'
                if dur_unit in ("s", "sec", "second", "seconds") or (not dur_unit and raw_dur > 300.0):
                    dur_min = raw_dur / 60.0
                else:
                    dur_min = raw_dur
                mets = (kcal / max(1.0, dur_min)) * 0.95
                t_start = datetime.fromisoformat(start_str.replace("Z", "+00:00")).timestamp()
                t_end = datetime.fromisoformat(end_str.replace("Z", "+00:00")).timestamp() if end_str else (t_start + dur_min * 60.0)
                workout_events.append((t_start, t_end, mets))
        except Exception:
            pass

        # Pre-load pharmacological exposures (7-day lookback for slow lipophilic THC clearance)
        caffeine_events: List[Tuple[float, float, float]] = []
        thc_events: List[Tuple[float, float, float]] = []
        pk_lookback_sec = (day_start_dt - timedelta(days=7)).timestamp()
        try:
            cur.execute("""
                SELECT ts_utc, substance, dose_value, half_life_hours FROM pharmacological_exposure_stream
                WHERE user_id = ? AND unixepoch(ts_utc, 'subsec') >= ? AND unixepoch(ts_utc, 'subsec') <= ?
            """, (self.subject_id, pk_lookback_sec, scan_end_sec))
            for r in cur.fetchall():
                ts_str, sub, dose_v, hl_h = r[0], str(r[1]), float(r[2] or 0.0), r[3]
                t_sec = datetime.fromisoformat(ts_str.replace("Z", "+00:00")).timestamp()
                if sub == "caffeine" and dose_v > 0.0:
                    caffeine_events.append((t_sec, dose_v, float(hl_h or 5.0)))
                elif sub == "cannabis_thc" and dose_v > 0.0:
                    thc_events.append((t_sec, dose_v, float(hl_h or 48.0)))
                elif sub == "carbohydrates" and dose_v > 0.0 and t_sec >= scan_start_sec:
                    meal_events.append((t_sec, dose_v))
        except Exception:
            pass

        dipping_pct = None
        if not simulated:
            try:
                bp_start_sec = (day_start_dt - timedelta(hours=12)).timestamp()
                bp_end_sec = (day_start_dt + timedelta(hours=36)).timestamp()
                sleep_windows: List[Tuple[float, float]] = []
                cur.execute("""
                    SELECT start_date, end_date, value FROM healthkit_records
                    WHERE user_id = ? AND record_type LIKE '%SleepAnalysis%'
                      AND unixepoch(end_date, 'subsec') > ? AND unixepoch(start_date, 'subsec') < ?
                """, (self.subject_id, bp_start_sec, bp_end_sec))
                for sr in cur.fetchall():
                    code = int(float(sr[2])) if sr[2] is not None else 1
                    if code in (1, 3, 4, 5) and sr[0] and sr[1]:
                        s0 = datetime.fromisoformat(sr[0].replace("Z", "+00:00")).timestamp()
                        s1 = datetime.fromisoformat(sr[1].replace("Z", "+00:00")).timestamp()
                        sleep_windows.append((s0, s1))

                cur.execute("""
                    SELECT start_date, value FROM healthkit_records
                    WHERE user_id = ? AND record_type LIKE '%BloodPressureSystolic%'
                      AND unixepoch(start_date, 'subsec') >= ? AND unixepoch(start_date, 'subsec') <= ?
                """, (self.subject_id, bp_start_sec, bp_end_sec))
                day_bps: List[float] = []
                night_bps: List[float] = []
                for br in cur.fetchall():
                    ts_s, val = br[0], float(br[1])
                    dt_utc = datetime.fromisoformat(ts_s.replace("Z", "+00:00"))
                    t_sec = dt_utc.timestamp()
                    if self.tz_offset_hours != 0.0:
                        local_dt = dt_utc.astimezone(timezone(timedelta(hours=self.tz_offset_hours))) if dt_utc.tzinfo is not None and not ts_s.endswith("Z") else (dt_utc + timedelta(hours=self.tz_offset_hours))
                        if local_dt.date() != target_date:
                            continue
                    elif not (scan_start_sec <= t_sec <= scan_end_sec):
                        continue
                    in_sleep = any(w0 <= t_sec <= w1 for w0, w1 in sleep_windows)
                    if in_sleep:
                        night_bps.append(val)
                    else:
                        lh = _parse_local_hour(ts_s, self.tz_offset_hours)
                        if lh is not None:
                            if 0.0 <= lh <= 6.0:
                                night_bps.append(val)
                            elif 8.0 <= lh <= 22.0:
                                day_bps.append(val)
                if day_bps and night_bps:
                    mean_day = sum(day_bps) / len(day_bps)
                    mean_night = sum(night_bps) / len(night_bps)
                    if mean_day > 0:
                        dipping_pct = round(((mean_day - mean_night) / mean_day) * 100.0, 2)
            except Exception:
                pass

        daily_rem_pct = None
        if not simulated:
            try:
                cur.execute("""
                    SELECT value, start_date, end_date
                    FROM healthkit_records
                    WHERE user_id = ? AND record_type LIKE '%SleepAnalysis%'
                      AND unixepoch(start_date, 'subsec') < ? AND unixepoch(end_date, 'subsec') > ?
                """, (self.subject_id, scan_end_sec, scan_start_sec))
                sleep_recs = cur.fetchall()
                if sleep_recs:
                    total_sleep_sec = 0.0
                    rem_sec = 0.0
                    scan_t0 = scan_start_sec
                    scan_t1 = scan_end_sec
                    for s_val, s_start, s_end in sleep_recs:
                        try:
                            t0 = datetime.fromisoformat(s_start.replace("Z", "+00:00")).timestamp()
                            t1 = datetime.fromisoformat(s_end.replace("Z", "+00:00")).timestamp()
                            dur = max(0.0, min(t1, scan_t1) - max(t0, scan_t0))
                            code = int(float(s_val or 1))
                            if code in (1, 3, 4, 5):
                                total_sleep_sec += dur
                            if code == 5:
                                rem_sec += dur
                        except Exception:
                            pass
                    if total_sleep_sec > 0:
                        daily_rem_pct = round((rem_sec / total_sleep_sec) * 100.0, 2)
            except Exception:
                pass

        # Trailing causal 14-night personal nocturnal wrist temperature baseline (< day_start_dt)
        baseline_wrist_temp_c = 36.40
        if not simulated:
            try:
                bl_start_sec = (day_start_dt - timedelta(days=14)).timestamp()
                bl_end_sec = day_start_dt.timestamp()
                cur.execute("""
                    SELECT value, unit FROM healthkit_records
                    WHERE user_id = ? AND record_type LIKE '%AppleSleepingWristTemperature%'
                      AND unixepoch(end_date, 'subsec') > ? AND unixepoch(start_date, 'subsec') < ?
                    ORDER BY unixepoch(start_date, 'subsec') ASC
                """, (self.subject_id, bl_start_sec, bl_end_sec))
                prior_temps: List[float] = []
                for tr in cur.fetchall():
                    if tr[0] is None:
                        continue
                    tc = float(tr[0])
                    tu = str(tr[1] or "").strip().lower()
                    if "degf" in tu or "°f" in tu or "fahrenheit" in tu or tc > 60.0:
                        tc = (tc - 32.0) * (5.0 / 9.0)
                    prior_temps.append(tc)
                if len(prior_temps) >= 3:
                    prior_temps.sort()
                    mid = len(prior_temps) // 2
                    baseline_wrist_temp_c = prior_temps[mid] if len(prior_temps) % 2 == 1 else 0.5 * (prior_temps[mid - 1] + prior_temps[mid])
            except Exception as exc:
                _warn_non_missing_sql_error("wrist_temp_baseline", exc)

        last_glucose_val = None
        last_glucose_min = None
        if not simulated:
            try:
                g_pre_start = (day_start_dt - timedelta(minutes=30)).timestamp()
                g_pre_end = day_start_dt.timestamp()
                cur.execute("""
                    SELECT value, unit, start_date FROM healthkit_records
                    WHERE user_id = ? AND record_type LIKE '%BloodGlucose%'
                      AND unixepoch(start_date, 'subsec') >= ? AND unixepoch(start_date, 'subsec') < ?
                    ORDER BY unixepoch(start_date, 'subsec') DESC LIMIT 1
                """, (self.subject_id, g_pre_start, g_pre_end))
                gr = cur.fetchone()
                if gr and gr[0] is not None:
                    gv = float(gr[0])
                    if "mmol" in str(gr[1] or "").lower():
                        gv *= 18.0182
                    t_prev = datetime.fromisoformat(gr[2].replace("Z", "+00:00")).timestamp()
                    last_glucose_val = round(gv, 3)
                    last_glucose_min = int((t_prev - g_pre_end) // 60)
            except Exception as exc:
                _warn_non_missing_sql_error("glucose_pre_midnight", exc)

        for minute in range(1440):
            epoch_dt = datetime(target_date.year, target_date.month, target_date.day, 0, 0, 0, tzinfo=timezone.utc) + timedelta(minutes=minute)
            epoch_end_dt = epoch_dt + timedelta(minutes=1)
            epoch_ts = epoch_dt.strftime("%Y-%m-%dT%H:%M:%SZ")
            epoch_end_ts = epoch_end_dt.strftime("%Y-%m-%dT%H:%M:%SZ")
            epoch_id = f"epoch-{self.subject_id}-{target_date.isoformat()}-{minute:04d}"
            hour = minute / 60.0

            if simulated:
                is_sleep = hour >= 22.0 or hour < 8.0
                sleep_stage = "deep" if (23.0 <= hour < 1.5 or 3.0 <= hour < 4.5) else ("rem" if (1.5 <= hour < 3.0 or 4.5 <= hour < 6.5) else "light") if is_sleep else None
                raw_hr = 54.0 + math.sin(minute / 60.0) * 3.0 if is_sleep else 72.0 + math.sin(minute / 90.0) * 8.0
                raw_rmssd = 58.0 + math.cos(minute / 60.0) * 6.0 if is_sleep else 44.0 + math.cos(minute / 90.0) * 5.0
                raw_glucose = 88.0 + math.sin(minute / 120.0) * 4.0 if is_sleep else 96.0 + math.sin(minute / 80.0) * 8.0
                wrist_temp = 36.42 + math.sin((hour - 3.0) * math.pi / 12.0) * 0.18 if is_sleep else 36.35
                delta_wrist_temp = round(wrist_temp - 36.40, 3)
                lux, cct, co2 = (0.0, 2200.0, 620.0) if is_sleep else (420.0, 4800.0, 540.0)
                carbs_g = 45.0 if (12.0 <= hour < 12.5 or 18.5 <= hour < 19.0) else 0.0
                mins_post_meal = (minute - 720) if (12.0 <= hour < 15.0) else ((minute - 1110) if (18.5 <= hour < 21.5) else -1.0)
                stress_glucose = dietary_glycemic_filter(raw_glucose, carbs_g, mins_post_meal)
                mets = 5.5 if (8.5 <= hour < 9.5) else 1.0
                mins_post_workout = (minute - 570) if (9.5 <= hour < 11.5) else -1.0
                stress_rmssd = exertion_tachycardia_filter(raw_rmssd, mets, mins_post_workout)
                zeitgeber = photic_zeitgeber_filter(lux, cct)
                hbo2 = 2.50 + (0.8 if (10.0 <= hour < 20.0) else 0.0)
                hbr = -0.50 - (0.2 if (10.0 <= hour < 20.0) else 0.0)
                motion_pct = 2.5 if not is_sleep else 0.5
                fm_theta, faa, tbr = 5.2, 0.05, 0.85
                motor_cv_iti = 3.8
                resting_hr, walking_hr = 52.0, 78.0
                sbp, dbp, dipping = 118.0, 76.0, 12.5
                pir_motion, bristol = 0, 4
                smi_hold = 15.5 if (20.0 <= hour < 20.2) else None
                pages, fk_grade, read_latency = (20, 11.5, 45.0) if (20.2 <= hour < 20.4) else (0, 0.0, 0.0)
                plaid_shock, plaid_pred_err, plaid_outcome = (0.0, 0.0, "expected_fulfillment")
            else:
                t_start_epoch = epoch_dt.timestamp()
                t_end_epoch = epoch_end_dt.timestamp()
                real = self._query_real_epoch_data(cur, t_start_epoch, t_end_epoch, epoch_ts, epoch_end_ts)
                hbo2, hbr = real.get("hbo2"), real.get("hbr")
                has_fnirs_obs = real.get("motion_artifact_pct") is not None or hbo2 is not None
                if has_fnirs_obs:
                    recorded_fnirs_epochs += 1
                motion_pct = real.get("motion_artifact_pct", 0.0)
                fm_theta, faa, tbr = real.get("fm_theta"), real.get("faa"), real.get("tbr")
                motor_cv_iti = real.get("cv_iti_pct")
                raw_hr = real.get("raw_hr")
                resting_hr = real.get("resting_hr")
                walking_hr = real.get("walking_hr")
                raw_rmssd = real.get("raw_rmssd")
                raw_glucose = real.get("raw_glucose")

                delta_g_postprandial = 0.0
                for m_sec, carbs in meal_events:
                    dt_min = (t_start_epoch - m_sec) / 60.0
                    if 0.0 <= dt_min <= 180.0:
                        tau_peak = 45.0
                        k = (dt_min / tau_peak) * math.exp(1.0 - dt_min / tau_peak)
                        delta_g_postprandial += (carbs * 0.45) * k
                stress_glucose = max(50.0, raw_glucose - delta_g_postprandial) if raw_glucose is not None else None

                hrv_suppression = 0.0
                active_mets = 1.0
                for w_start_sec, w_end_sec, w_mets in workout_events:
                    if w_start_sec <= t_start_epoch <= w_end_sec:
                        # Intra-workout: peak sympathetic suppression
                        hrv_suppression += (w_mets * 2.8)
                        active_mets = max(active_mets, w_mets)
                    elif w_end_sec < t_start_epoch <= (w_end_sec + 7200.0):
                        # Post-workout recovery decay
                        dt_min = (t_start_epoch - w_end_sec) / 60.0
                        hrv_suppression += (w_mets * 2.8) * math.exp(-dt_min / 30.0)
                        active_mets = max(active_mets, w_mets)
                stress_rmssd = max(5.0, raw_rmssd + hrv_suppression) if raw_rmssd is not None else None
                mets = active_mets

                wrist_temp = real.get("wrist_temp")
                delta_wrist_temp = round(wrist_temp - 36.40, 3) if wrist_temp is not None else None
                sbp = real.get("sbp")
                dbp = real.get("dbp")
                dipping = real.get("dipping_pct") if real.get("dipping_pct") is not None else dipping_pct

                if raw_glucose is not None:
                    if last_glucose_val is not None and last_glucose_min is not None:
                        dt_g = minute - last_glucose_min
                        dg_dt = round((raw_glucose - last_glucose_val) / float(dt_g), 3) if 0 < dt_g <= 30 else None
                    else:
                        dg_dt = None
                    last_glucose_val = raw_glucose
                    last_glucose_min = minute
                else:
                    dg_dt = None

                co2, voc, pm25 = real.get("co2"), real.get("voc"), real.get("pm25")
                pir_motion = real.get("pir_motion")
                lux = real.get("lux")
                cct_k = real.get("cct_kelvin") or 4500.0
                zeitgeber = photic_zeitgeber_filter(lux, cct_k) if lux is not None else None
                bristol = real.get("bristol")
                smi_hold = real.get("smi_hold")
                airofit_mip = real.get("airofit_mip")
                airofit_mep = real.get("airofit_mep")
                airofit_vc = real.get("airofit_vc")
                pages = real.get("pages")
                fk_grade = real.get("fk_grade")
                read_latency = real.get("read_latency")
                plaid_shock = real.get("plaid_shock")
                plaid_pred_err = real.get("plaid_pred_err")
                plaid_outcome = real.get("plaid_outcome")
                sleep_stage = real.get("sleep_stage")

            if simulated:
                recorded_fnirs_epochs += 1
            t_epoch_sec = epoch_dt.timestamp()
            caf_mg = 0.0 if simulated else _pk_active_load(caffeine_events, t_epoch_sec)
            thc_mg = 0.0 if simulated else _pk_active_load(thc_events, t_epoch_sec)
            motion_artifact_total_pct += motion_pct if (simulated or real.get("motion_artifact_pct") is not None or hbo2 is not None) else 0.0
            epochs.append((
                epoch_id, self.subject_id, self.cohort_stage, epoch_ts, day_index, minute,
                hbo2, hbr, round(calibrated_dpf, 3),
                fm_theta if simulated else real.get("fm_theta"),
                faa if simulated else real.get("faa"),
                tbr if simulated else (real.get("beta_alpha_arousal") if real.get("beta_alpha_arousal") is not None else real.get("tbr")),
                3.8 if simulated else real.get("cv_iti_pct"),
                raw_rmssd, stress_rmssd, raw_hr,
                52.0 if simulated else resting_hr,
                78.0 if simulated else walking_hr,
                14.5 if simulated else real.get("resp_rate"),
                smi_hold,
                (108.5 if (20.0 <= hour < 20.2) else None) if simulated else airofit_mip,
                (132.0 if (20.0 <= hour < 20.2) else None) if simulated else airofit_mep,
                (4.85 if (20.0 <= hour < 20.2) else None) if simulated else airofit_vc,
                raw_glucose, stress_glucose,
                0.12 if simulated else dg_dt,
                wrist_temp, delta_wrist_temp if simulated else (round(wrist_temp - baseline_wrist_temp_c, 3) if wrist_temp is not None else None),
                118.0 if simulated else sbp, 76.0 if simulated else dbp,
                12.5 if simulated else dipping,
                (1.2 if mets > 1.0 else 0.05) if simulated else real.get("active_energy"),
                mets,
                (85 if mets > 1.0 else 4) if simulated else real.get("steps"),
                co2, 85.0 if simulated else voc, 6.2 if simulated else pm25,
                sleep_stage,
                22.0 if simulated else (round(float(real.get("sws_ratio")) * 100.0, 2) if real.get("sws_ratio") is not None else None),
                24.5 if simulated else daily_rem_pct, zeitgeber, pir_motion if not simulated else 0, bristol if not simulated else 4,
                plaid_shock, plaid_pred_err, plaid_outcome,
                pages, fk_grade, read_latency, caf_mg, thc_mg,
                None if simulated else real.get("cog_ez_ddm_drift_v"),
                None if simulated else real.get("cog_ez_ddm_boundary_a"),
                None if simulated else real.get("cog_ez_ddm_ter_s"),
                None if simulated else real.get("cog_hick_slope_ms_per_bit"),
                None if simulated else real.get("cog_d_prime"),
                None if simulated else real.get("cog_plv_r_sync"),
                None if simulated else real.get("cog_nma_ms"),
                None if simulated else real.get("cog_max_span"),
                None if simulated else real.get("cog_practice_residual_ms"),
                motion_pct, 0
            ))

        avg_motion = (motion_artifact_total_pct / float(recorded_fnirs_epochs)) if recorded_fnirs_epochs > 0 else 0.0
        missing_temp_nights = 0
        if not simulated and day_index >= 3:
            try:
                w3_start = (day_start_dt - timedelta(days=3)).timestamp()
                w3_end = day_start_dt.timestamp()
                cur.execute("""
                    SELECT COUNT(DISTINCT date(unixepoch(start_date, 'subsec'), 'unixepoch'))
                    FROM healthkit_records
                    WHERE user_id = ? AND record_type LIKE '%AppleSleepingWristTemperature%'
                      AND unixepoch(end_date, 'subsec') > ? AND unixepoch(start_date, 'subsec') < ?
                """, (self.subject_id, w3_start, w3_end))
                row = cur.fetchone()
                if row and row[0] == 0:
                    missing_temp_nights = 3
            except Exception as exc:
                _warn_non_missing_sql_error("stopping_rule_temp", exc)

        stop_rule = 1 if (avg_motion > 40.0 or missing_temp_nights >= 3) else 0
        epochs_with_stop = [
            tup[:-1] + (stop_rule,) for tup in epochs
        ]
        self.conn.executemany(
            "INSERT OR REPLACE INTO daily_telemetry_master VALUES (" + ", ".join(["?"] * 62) + ", CURRENT_TIMESTAMP)",
            epochs_with_stop
        )
        self.conn.commit()
        chash = hashlib.sha256(f"telemetry-{target_date.isoformat()}-{len(epochs)}".encode()).hexdigest()
        return {
            "target_date": target_date.isoformat(), "epochs_compiled": len(epochs),
            "calibrated_dpf": calibrated_dpf, "avg_motion_artifact_pct": round(avg_motion, 2),
            "stopping_rule_triggered": bool(stop_rule), "content_hash": chash
        }

    def export_parquet(self, output_dir: str = "data/pilot1/parquet") -> str:
        os.makedirs(output_dir, exist_ok=True)
        cur = self.conn.cursor()
        cur.execute("SELECT * FROM daily_telemetry_master WHERE subject_id = ? ORDER BY epoch_timestamp ASC", (self.subject_id,))
        rows = cur.fetchall()
        col_names = [d[0] for d in cur.description]

        try:
            import pyarrow as pa
            import pyarrow.parquet as pq
            str_cols = {
                "epoch_id", "subject_id", "epoch_timestamp", "sleep_stage",
                "trust_event_outcome", "created_at",
            }
            int_cols = {
                "cohort_stage", "day_index", "minute_of_day", "step_count",
                "bedroom_pir_motion", "enteric_bristol_type", "reading_pages_count",
                "cog_max_span", "stopping_rule_triggered",
            }
            schema = pa.schema([
                pa.field(
                    col,
                    pa.string() if col in str_cols else (pa.int64() if col in int_cols else pa.float64())
                )
                for col in col_names
            ])
            table_dict = {col: [row[i] for row in rows] for i, col in enumerate(col_names)}
            arrow_table = pa.Table.from_pydict(table_dict, schema=schema)
            out_file = os.path.join(output_dir, f"docas_pilot1_{self.subject_id}_master.parquet")
            pq.write_table(arrow_table, out_file, compression="snappy")
            return out_file
        except ImportError:
            out_file = os.path.join(output_dir, f"docas_pilot1_{self.subject_id}_master.jsonl")
            with open(out_file, "w") as f:
                for row in rows:
                    f.write(json.dumps(dict(zip(col_names, row))) + "\n")
            return out_file

    def close(self):
        self.conn.close()

def run_self_test() -> bool:
    print("================================================================================")
    print("   DOCAS-PILOT-1: DAILY TELEMETRY HARVESTER PRE-FLIGHT SELF-TEST")
    print("================================================================================")
    test_db = "test_harvester_dryrun.db"
    if os.path.exists(test_db):
        os.remove(test_db)

    # 1. Filter 1: Dietary Glycemic Confounder Filter
    g_raw = 145.0
    g_stress = dietary_glycemic_filter(g_raw, carbs_g=50.0, minutes_post_meal=45.0)
    assert g_stress < g_raw, f"Expected postprandial glucose subtraction, got {g_stress} vs {g_raw}"
    print(f"✓ Filter 1 (Dietary Glycemic): 50g carbs at 45m subtracted {g_raw - g_stress:.1f} mg/dL postprandial excursion")

    # 2. Filter 2: Exertion Tachycardia Confounder Filter
    hrv_raw = 28.0
    hrv_stress = exertion_tachycardia_filter(hrv_raw, mets=6.0, minutes_post_workout=15.0)
    assert hrv_stress > hrv_raw, f"Expected post-workout vagal recovery rebound, got {hrv_stress} vs {hrv_raw}"
    print(f"✓ Filter 2 (Exertion Tachycardia): 6.0 METs at 15m post-workout residualized to {hrv_stress:.1f} ms stress HRV")

    # 3. Filter 3: Individualized Cranial DPF Calibration
    dpf_base = 6.0
    dpf_adipose = cranial_dpf_calibration(dpf_base, body_fat_pct=25.0)
    assert dpf_adipose > dpf_base, f"Expected DPF elevation with higher body fat, got {dpf_adipose} vs {dpf_base}"
    print(f"✓ Filter 3 (Cranial DPF): Fitindex BIA 25.0% body fat scaled optical pathlength DPF to {dpf_adipose:.2f}")

    # 4. Filter 4: Circadian Photic Zeitgeber Filter
    zeit = photic_zeitgeber_filter(lux=450.0, cct_kelvin=6000.0)
    assert zeit > 0.0, f"Zeitgeber index must be positive, got {zeit}"
    print(f"✓ Filter 4 (Circadian Photic): 450 Lux at 6000K evaluated to Zeitgeber Index = {zeit:.2f}")

    # 5. Full 1,440-Epoch Harvest Run
    harvester = TelemetryHarvester(db_path=test_db, subject_id="pilot_sub_01", cohort_stage=1)
    res = harvester.harvest_day(target_date=date(2026, 11, 12), day_index=1, body_fat_pct=18.5, simulated=True)
    assert res["epochs_compiled"] == 1440, f"Expected 1440 epochs, got {res['epochs_compiled']}"
    assert not res["stopping_rule_triggered"], "Stopping rule should not trigger on clean data"
    print(f"✓ Master Epoch Grid: 1,440 continuous 1-minute epochs compiled (Content SHA-256: {res['content_hash'][:16]}...)")

    export_path = harvester.export_parquet(output_dir="target/test_harvest_parquet")
    assert os.path.exists(export_path), f"Export file {export_path} must exist"
    print(f"✓ Columnar Export: Persisted to {export_path}")
    harvester.close()

    # 6. Live Mode (simulated=False) SQL Round-Trip Oracle with Multi-Subject Isolation
    harvester_live = TelemetryHarvester(db_path=test_db, subject_id="pilot_live_01", cohort_stage=1)
    harvester_live.conn.execute("CREATE TABLE IF NOT EXISTS ambient_environment_stream (id TEXT, user_id TEXT, ts_utc DATETIME, co2_ppm REAL, voc_ppb REAL, pm25_ug_m3 REAL, temp_c REAL, relative_humidity_pct REAL, air_quality_score REAL, source_device TEXT, created_at DATETIME)")
    harvester_live.conn.execute("INSERT INTO ambient_environment_stream VALUES ('amb-1', 'pilot_live_01', '2026-11-12T14:30:15Z', 485.0, 62.0, 4.8, 21.5, 45.0, 92.0, 'awair_element', CURRENT_TIMESTAMP)")
    # Seed rogue subject into the same table at the same minute
    harvester_live.conn.execute("INSERT INTO ambient_environment_stream VALUES ('amb-rogue', 'rogue_sub_99', '2026-11-12T14:30:15Z', 9999.0, 999.0, 99.0, 50.0, 90.0, 10.0, 'rogue_device', CURRENT_TIMESTAMP)")

    harvester_live.conn.execute("CREATE TABLE IF NOT EXISTS healthkit_records (id TEXT, user_id TEXT, healthkit_profile_id TEXT, record_type TEXT, source_name TEXT, source_version TEXT, device TEXT, unit TEXT, value REAL, start_date DATETIME, end_date DATETIME, creation_date DATETIME, metadata TEXT, raw_record_data TEXT, created_at DATETIME)")
    harvester_live.conn.execute("INSERT INTO healthkit_records VALUES ('c-1', 'pilot_live_01', 'p1', 'HKQuantityTypeIdentifierDietaryCarbohydrates', 'Apple Health', '1.0', 'iPhone', 'g', 50.0, '2026-11-12T12:00:00Z', '2026-11-12T12:00:00Z', CURRENT_TIMESTAMP, '{}', '{}', CURRENT_TIMESTAMP)")
    harvester_live.conn.execute("INSERT INTO healthkit_records VALUES ('g-1', 'pilot_live_01', 'p1', 'HKQuantityTypeIdentifierBloodGlucose', 'Dexcom Stelo', '1.0', 'Transmitter', 'mg/dL', 145.0, '2026-11-12T12:45:00Z', '2026-11-12T12:45:00Z', CURRENT_TIMESTAMP, '{}', '{}', CURRENT_TIMESTAMP)")

    # Seed 30-minute REM sleep interval from 02:00:00Z to 02:30:00Z (code 5 = REM)
    harvester_live.conn.execute("INSERT INTO healthkit_records VALUES ('sl-1', 'pilot_live_01', 'p1', 'HKCategoryTypeIdentifierSleepAnalysis', 'Apple Watch', '1.0', 'Watch', 'count', 5.0, '2026-11-12T02:00:00Z', '2026-11-12T02:30:00Z', CURRENT_TIMESTAMP, '{}', '{}', CURRENT_TIMESTAMP)")
    # Seed rogue sleep record that must not leak
    harvester_live.conn.execute("INSERT INTO healthkit_records VALUES ('sl-rogue', 'rogue_sub_99', 'p99', 'HKCategoryTypeIdentifierSleepAnalysis', 'Apple Watch', '1.0', 'Watch', 'count', 4.0, '2026-11-12T02:00:00Z', '2026-11-12T02:30:00Z', CURRENT_TIMESTAMP, '{}', '{}', CURRENT_TIMESTAMP)")

    # Seed SDNN first, then true beat-to-beat RMSSD at 10:00:00Z (minute 600)
    harvester_live.conn.execute("INSERT INTO healthkit_records VALUES ('hrv-sdnn', 'pilot_live_01', 'p1', 'HKQuantityTypeIdentifierHeartRateVariabilitySDNN', 'Apple Watch', '1.0', 'Watch', 'ms', 35.0, '2026-11-12T10:00:00Z', '2026-11-12T10:00:00Z', CURRENT_TIMESTAMP, '{}', '{}', CURRENT_TIMESTAMP)")
    harvester_live.conn.execute("INSERT INTO healthkit_records VALUES ('hrv-rmssd', 'pilot_live_01', 'p1', 'HKQuantityTypeIdentifierHeartRateVariabilityRMSSD', 'Apple Watch', '1.0', 'Watch', 'ms', 62.0, '2026-11-12T10:00:00Z', '2026-11-12T10:00:00Z', CURRENT_TIMESTAMP, '{\"rmssd_heartbeat_series\": true}', '{}', CURRENT_TIMESTAMP)")
    # Permutation-Invariance Test: Seed phone step record BEFORE watch step record at 12:00 (minute 720)
    harvester_live.conn.execute("INSERT INTO healthkit_records VALUES ('step-phone', 'pilot_live_01', 'p1', 'HKQuantityTypeIdentifierStepCount', 'iPhone', '1.0', 'Phone', 'count', 65.0, '2026-11-12T12:00:00Z', '2026-11-12T12:00:00Z', CURRENT_TIMESTAMP, '{}', '{}', CURRENT_TIMESTAMP)")
    harvester_live.conn.execute("INSERT INTO healthkit_records VALUES ('step-watch', 'pilot_live_01', 'p1', 'HKQuantityTypeIdentifierStepCount', 'Apple Watch', '1.0', 'Watch', 'count', 75.0, '2026-11-12T12:00:00Z', '2026-11-12T12:00:00Z', CURRENT_TIMESTAMP, '{}', '{}', CURRENT_TIMESTAMP)")
    # Permutation-Invariance Test: Seed watch step record BEFORE phone step record at 12:01 (minute 721)
    harvester_live.conn.execute("INSERT INTO healthkit_records VALUES ('step-watch-2', 'pilot_live_01', 'p1', 'HKQuantityTypeIdentifierStepCount', 'Apple Watch', '1.0', 'Watch', 'count', 82.0, '2026-11-12T12:01:00Z', '2026-11-12T12:01:00Z', CURRENT_TIMESTAMP, '{}', '{}', CURRENT_TIMESTAMP)")
    harvester_live.conn.execute("INSERT INTO healthkit_records VALUES ('step-phone-2', 'pilot_live_01', 'p1', 'HKQuantityTypeIdentifierStepCount', 'iPhone', '1.0', 'Phone', 'count', 50.0, '2026-11-12T12:01:00Z', '2026-11-12T12:01:00Z', CURRENT_TIMESTAMP, '{}', '{}', CURRENT_TIMESTAMP)")

    # Seed ambient daylight / lux from Apple Watch / HomeKit
    harvester_live.conn.execute("INSERT INTO healthkit_records VALUES ('lux-watch', 'pilot_live_01', 'p1', 'HKQuantityTypeIdentifierTimeInDaylight', 'Apple Watch', '1.0', 'Watch', 'lux', 550.0, '2026-11-12T12:00:00Z', '2026-11-12T12:00:00Z', CURRENT_TIMESTAMP, '{}', '{}', CURRENT_TIMESTAMP)")
    # Seed 1800-second (30-min) workout with 300 kcal at 08:30:00Z (minute 510) to test second-to-minute normalization
    harvester_live.conn.execute("CREATE TABLE IF NOT EXISTS healthkit_workouts (id TEXT, user_id TEXT, healthkit_profile_id TEXT, workout_activity_type TEXT, duration REAL, duration_unit TEXT, total_distance REAL, distance_unit TEXT, total_energy_burned REAL, energy_unit TEXT, source_name TEXT, source_version TEXT, device TEXT, start_date DATETIME, end_date DATETIME, creation_date DATETIME, metadata TEXT, raw_workout_data TEXT, created_at DATETIME)")
    harvester_live.conn.execute("INSERT INTO healthkit_workouts VALUES ('w-1', 'pilot_live_01', 'p1', 'HKWorkoutActivityTypeRunning', 1800.0, 's', 3.1, 'mi', 300.0, 'kcal', 'Apple Watch', '1.0', 'Watch', '2026-11-12T08:00:00Z', '2026-11-12T08:30:00Z', CURRENT_TIMESTAMP, '{}', '{}', CURRENT_TIMESTAMP)")
    # Also seed raw HRV at minute 525 (08:45:00Z, 15 min post-workout)
    harvester_live.conn.execute("INSERT INTO healthkit_records VALUES ('hrv-post-w', 'pilot_live_01', 'p1', 'HKQuantityTypeIdentifierHeartRateVariabilityRMSSD', 'Apple Watch', '1.0', 'Watch', 'ms', 45.0, '2026-11-12T08:45:00Z', '2026-11-12T08:45:00Z', CURRENT_TIMESTAMP, '{\"rmssd_heartbeat_series\": true}', '{}', CURRENT_TIMESTAMP)")

    # Seed motion-only record (ambient_lux is NULL) at minute 720: must NOT clobber Watch daylight lux 550.0!
    harvester_live.conn.execute("CREATE TABLE IF NOT EXISTS sleep_boundary_pir_stream (id TEXT, user_id TEXT, ts_utc DATETIME, motion_detected INTEGER, ambient_lux REAL, created_at DATETIME)")
    harvester_live.conn.execute("INSERT INTO sleep_boundary_pir_stream VALUES ('pir-1', 'pilot_live_01', '2026-11-12T12:00:30Z', 1, NULL, CURRENT_TIMESTAMP)")

    # Seed real TimeInDaylight (in minutes!) at minute 722 + Migration 029 environmental_context_stream (indoor_lux=300, cct=6500K)
    harvester_live.conn.execute("INSERT INTO healthkit_records VALUES ('tid-min', 'pilot_live_01', 'p1', 'HKQuantityTypeIdentifierTimeInDaylight', 'Apple Watch', '1.0', 'Watch', 'min', 25.0, '2026-11-12T12:02:00Z', '2026-11-12T12:02:00Z', CURRENT_TIMESTAMP, '{}', '{}', CURRENT_TIMESTAMP)")
    harvester_live.conn.execute("CREATE TABLE IF NOT EXISTS environmental_context_stream (id TEXT, user_id TEXT, ts_utc DATETIME, pm2_5_ug_m3 REAL, co2_ppm REAL, voc_ppb REAL, indoor_lux REAL, indoor_kelvin_cct REAL, created_at DATETIME)")
    harvester_live.conn.execute("INSERT INTO environmental_context_stream VALUES ('env-1', 'pilot_live_01', '2026-11-12T12:02:10Z', 4.2, 510.0, 40.0, 300.0, 6500.0, CURRENT_TIMESTAMP)")

    # Seed TZ-aware BP records (-08:00 PST), 029 cognitive sessions (pvt_hick must NOT leak into cv_iti_pct!), and PK caffeine dose
    harvester_live.conn.execute("INSERT INTO healthkit_records VALUES ('bp-night', 'pilot_live_01', 'p1', 'HKQuantityTypeIdentifierBloodPressureSystolic', 'Aktiia', '1.0', 'Hilo', 'mmHg', 104.0, '2026-11-12T03:00:00-08:00', '2026-11-12T03:00:00-08:00', CURRENT_TIMESTAMP, '{}', '{}', CURRENT_TIMESTAMP)")
    harvester_live.conn.execute("INSERT INTO healthkit_records VALUES ('bp-day', 'pilot_live_01', 'p1', 'HKQuantityTypeIdentifierBloodPressureSystolic', 'Aktiia', '1.0', 'Hilo', 'mmHg', 124.0, '2026-11-12T09:00:00-08:00', '2026-11-12T09:00:00-08:00', CURRENT_TIMESTAMP, '{}', '{}', CURRENT_TIMESTAMP)")
    harvester_live.conn.execute("CREATE TABLE IF NOT EXISTS cognitive_battery_sessions (id TEXT, user_id TEXT, task_type TEXT, started_at DATETIME, mean_rt_ms REAL, sigma_clock_sq REAL, sigma_motor_sq REAL, metrics_json TEXT, ez_ddm_drift_v REAL, ez_ddm_boundary_a REAL, ez_ddm_ter_s REAL, hick_slope_ms_per_bit REAL, d_prime REAL, plv_r_sync REAL, nma_ms REAL, max_span INTEGER, practice_residual_ms REAL)")
    harvester_live.conn.execute("INSERT INTO cognitive_battery_sessions VALUES ('cs-pvt', 'pilot_live_01', 'pvt_hick', '2026-11-12T12:02:15Z', 260.0, NULL, NULL, '{}', 0.34, 0.11, 0.21, 68.5, NULL, NULL, NULL, NULL, -12.4)")
    harvester_live.conn.execute("INSERT INTO cognitive_battery_sessions VALUES ('cs-beat', 'pilot_live_01', 'beat_sync', '2026-11-12T12:03:15Z', 500.0, 225.0, 87.5, '{\"sd_iti_ms\": 20.0, \"cv_iti_pct\": 4.0}', NULL, NULL, NULL, NULL, NULL, 0.96, -18.2, NULL, NULL)")
    harvester_live.conn.execute("CREATE TABLE IF NOT EXISTS pharmacological_exposure_stream (id TEXT, user_id TEXT, ts_utc DATETIME, substance TEXT, dose_value REAL, half_life_hours REAL)")
    harvester_live.conn.execute("INSERT INTO pharmacological_exposure_stream VALUES ('pk-caf', 'pilot_live_01', '2026-11-12T07:00:00Z', 'caffeine', 200.0, 5.0)")



    # Seed reading log at 20:30:15Z (minute 1230)
    harvester_live.conn.execute("CREATE TABLE IF NOT EXISTS manual_reading_log_stream (id TEXT, user_id TEXT, session_id TEXT, isbn TEXT, book_title TEXT, read_start_utc DATETIME, read_end_utc DATETIME, digitized_utc DATETIME, latency_sec REAL, pages_read INTEGER, ocr_text TEXT, word_count INTEGER, syllable_count INTEGER, sentence_count INTEGER, flesch_kincaid_grade REAL, semantic_valence REAL, is_nce_control INTEGER, created_at DATETIME)")
    harvester_live.conn.execute("INSERT INTO manual_reading_log_stream VALUES ('rd-1', 'pilot_live_01', 's1', '123', 'Book', '2026-11-12T20:00:00Z', '2026-11-12T20:30:00Z', '2026-11-12T20:30:15Z', 45.0, 20, 'text', 100, 150, 10, 11.5, 0.25, 0, CURRENT_TIMESTAMP)")
    # Seed rogue subject reading log
    harvester_live.conn.execute("INSERT INTO manual_reading_log_stream VALUES ('rd-rogue', 'rogue_sub_99', 's99', '999', 'RogueBook', '2026-11-12T20:00:00Z', '2026-11-12T20:30:00Z', '2026-11-12T20:30:15Z', 99.0, 999, 'text', 999, 999, 10, 99.0, 0.0, 0, CURRENT_TIMESTAMP)")

    # Seed midnight millisecond .250Z (minute 0!) and -08:00 offset HealthKit HR (04:05:10-08:00 == 12:05:10Z == minute 725!)
    harvester_live.conn.execute("INSERT INTO cognitive_battery_sessions VALUES ('cs-mid', 'pilot_live_01', 'gonogo_conflict', '2026-11-12T00:00:00.250Z', 290.0, NULL, NULL, '{}', NULL, NULL, NULL, NULL, 2.85, NULL, NULL, NULL, NULL)")
    harvester_live.conn.execute("INSERT INTO healthkit_records VALUES ('hr-tz', 'pilot_live_01', 'p1', 'HKQuantityTypeIdentifierHeartRate', 'Apple Watch', '1.0', 'Watch', 'bpm', 68.0, '2026-11-12T04:05:10-08:00', '2026-11-12T04:05:10-08:00', CURRENT_TIMESTAMP, '{}', '{}', CURRENT_TIMESTAMP)")
    # Cross-midnight AppleSleepingWristTemperature (degF) + international mmol/L BloodGlucose
    # Plus 3 prior nights at 35.60C for causal personal baseline + future Day 6 decoy 37.20C (must NOT leak!)
    for d_off in (9, 10, 11):
        harvester_live.conn.execute(
            "INSERT INTO healthkit_records VALUES (?, 'pilot_live_01', 'p1', 'HKQuantityTypeIdentifierAppleSleepingWristTemperature', 'Apple Watch', '1.0', 'Watch', 'degC', 35.60, ?, ?, CURRENT_TIMESTAMP, '{}', '{}', CURRENT_TIMESTAMP)",
            (f"wt-p{d_off}", f"2026-11-{d_off:02d}T01:00:00Z", f"2026-11-{d_off:02d}T06:00:00Z")
        )
    harvester_live.conn.execute("INSERT INTO healthkit_records VALUES ('wt-fut', 'pilot_live_01', 'p1', 'HKQuantityTypeIdentifierAppleSleepingWristTemperature', 'Apple Watch', '1.0', 'Watch', 'degC', 37.20, '2026-11-15T01:00:00Z', '2026-11-15T06:00:00Z', CURRENT_TIMESTAMP, '{}', '{}', CURRENT_TIMESTAMP)")
    harvester_live.conn.execute("INSERT INTO healthkit_records VALUES ('wt-ov', 'pilot_live_01', 'p1', 'HKQuantityTypeIdentifierAppleSleepingWristTemperature', 'Apple Watch', '1.0', 'Watch', 'degF', 97.7, '2026-11-11T23:30:00Z', '2026-11-12T07:00:00Z', CURRENT_TIMESTAMP, '{}', '{}', CURRENT_TIMESTAMP)")
    harvester_live.conn.execute("INSERT INTO healthkit_records VALUES ('bg-pre', 'pilot_live_01', 'p1', 'HKQuantityTypeIdentifierBloodGlucose', 'CGM', '1.0', 'Sensor', 'mg/dL', 110.0, '2026-11-11T23:50:00Z', '2026-11-11T23:50:00Z', CURRENT_TIMESTAMP, '{}', '{}', CURRENT_TIMESTAMP)")
    harvester_live.conn.execute("INSERT INTO healthkit_records VALUES ('bg-mid', 'pilot_live_01', 'p1', 'HKQuantityTypeIdentifierBloodGlucose', 'CGM', '1.0', 'Sensor', 'mg/dL', 125.0, '2026-11-12T00:05:00Z', '2026-11-12T00:05:00Z', CURRENT_TIMESTAMP, '{}', '{}', CURRENT_TIMESTAMP)")
    harvester_live.conn.execute("INSERT INTO healthkit_records VALUES ('bg-mmol', 'pilot_live_01', 'p1', 'HKQuantityTypeIdentifierBloodGlucose', 'CGM', '1.0', 'Sensor', 'mmol/L', 5.5, '2026-11-12T08:00:00Z', '2026-11-12T08:00:00Z', CURRENT_TIMESTAMP, '{}', '{}', CURRENT_TIMESTAMP)")

    harvester_live.conn.commit()
    res_live = harvester_live.harvest_day(target_date=date(2026, 11, 12), day_index=1, simulated=False)
    assert res_live["epochs_compiled"] == 1440
    cur_live = harvester_live.conn.cursor()
    cur_live.execute("SELECT ambient_co2_ppm, glucose_raw_mgdl FROM daily_telemetry_master WHERE subject_id = 'pilot_live_01' AND minute_of_day = 870")
    m870 = cur_live.fetchone()
    assert m870[0] == 485.0, f"Expected subject's Awair CO2 485.0 (isolated from rogue 9999.0), got {m870[0]}"

    cur_live.execute("SELECT glucose_raw_mgdl, glucose_stress_mgdl FROM daily_telemetry_master WHERE subject_id = 'pilot_live_01' AND minute_of_day = 765")
    m765 = cur_live.fetchone()
    assert m765[0] == 145.0, f"Expected raw glucose 145.0, got {m765[0]}"
    assert m765[1] < 145.0, f"Expected de-biased stress glucose < 145.0, got {m765[1]}"
    print(f"✓ Cross-Epoch Confounder Propagation: Seeded 50g meal at 12:00 de-biased 12:45 glucose from {m765[0]} to {m765[1]:.1f} mg/dL")

    # Verify Event-Stream Non-Dilution: 1 reading log evaluated to exact AVG(FK)=11.5 across daily master
    cur_live.execute("SELECT AVG(reading_flesch_kincaid), COUNT(reading_flesch_kincaid) FROM daily_telemetry_master WHERE subject_id = 'pilot_live_01'")
    avg_fk, count_fk = cur_live.fetchone()
    assert count_fk == 1, f"Expected exactly 1 reading event, got {count_fk}"
    assert abs(avg_fk - 11.5) < 1e-6, f"Expected non-diluted AVG(FK) == 11.5, got {avg_fk}"
    print(f"✓ Event-Stream Honest Missingness: Exactly 1 reading log evaluated to non-diluted AVG(FK)={avg_fk} (zero dilution across 1,439 NULL epochs)")
    print("✓ Multi-Subject Tenant Isolation: Rogue subject readings (9999 CO2, 99.0 FK) strictly isolated")

    cur_live.execute("SELECT ambient_co2_ppm, glucose_raw_mgdl, reading_flesch_kincaid, cog_d_prime FROM daily_telemetry_master WHERE subject_id = 'pilot_live_01' AND minute_of_day = 0")
    m0 = cur_live.fetchone()
    assert m0[0] is None and m0[1] is None and m0[2] is None, f"Expected unseeded epoch to be NULL, got {m0}"
    assert m0[3] is not None and abs(m0[3] - 2.85) < 1e-6, f"Expected midnight .250Z session cog_d_prime=2.85 at minute 0, got {m0[3]}"
    cur_live.execute("SELECT heart_rate_bpm FROM daily_telemetry_master WHERE subject_id = 'pilot_live_01' AND minute_of_day = 725")
    m725_hr = cur_live.fetchone()[0]
    assert m725_hr is not None and abs(m725_hr - 68.0) < 1e-6, f"Expected 04:05:10-08:00 HR=68.0 at UTC minute 725, got {m725_hr}"
    cur_live.execute("SELECT sleeping_wrist_temp_c, delta_wrist_temp_c FROM daily_telemetry_master WHERE subject_id = 'pilot_live_01' AND minute_of_day = 180")
    m180_wt, m180_dwt = cur_live.fetchone()
    assert m180_wt is not None and abs(m180_wt - 36.5) < 0.02, f"Expected cross-midnight 97.7 degF -> 36.5C at min 180, got {m180_wt}"
    assert m180_dwt is not None and abs(m180_dwt - 0.90) < 0.02, f"Expected causal personal baseline 35.6C -> delta +0.90C, got {m180_dwt}"
    cur_live.execute("SELECT glucose_raw_mgdl FROM daily_telemetry_master WHERE subject_id = 'pilot_live_01' AND minute_of_day = 480")
    m480_bg = cur_live.fetchone()[0]
    assert m480_bg is not None and abs(m480_bg - 99.1) < 0.1, f"Expected 5.5 mmol/L -> 99.1 mg/dL at min 480, got {m480_bg}"
    cur_live.execute("SELECT glucose_rate_dg_dt FROM daily_telemetry_master WHERE subject_id = 'pilot_live_01' AND minute_of_day = 5")
    m5_dg = cur_live.fetchone()[0]
    assert m5_dg is not None and abs(m5_dg - 1.0) < 0.01, f"Expected cross-midnight dg/dt=1.0 mg/dL/min at min 5, got {m5_dg}"
    print("✓ Live-Mode SQL Oracle + Subsec/TZ + Interval WristTemp + mmol/L CGM Alignment verified")

    # Verify Sleep Interval Allen Overlap: 30 epochs (minutes 120..149) all populated with "REM", minute 150 is None
    cur_live.execute("SELECT minute_of_day, sleep_stage, rem_sleep_pct FROM daily_telemetry_master WHERE subject_id = 'pilot_live_01' AND minute_of_day BETWEEN 120 AND 150 ORDER BY minute_of_day ASC")
    sleep_rows = cur_live.fetchall()
    assert len(sleep_rows) == 31, f"Expected 31 rows, got {len(sleep_rows)}"
    for minute_idx, stage, rem_pct in sleep_rows[:30]:
        assert stage == "REM", f"Expected minute {minute_idx} to have sleep_stage 'REM', got {stage}"
        assert rem_pct == 100.0, f"Expected rem_sleep_pct 100.0, got {rem_pct}"
    assert sleep_rows[30][1] is None, f"Expected minute 150 (outside interval) sleep_stage to be None, got {sleep_rows[30][1]}"
    print("✓ Allen's Interval Overlap Soundness: 30-minute REM sleep record populated all 30 1-minute epochs (120..149)")

    # Verify RMSSD Beat-to-Beat Priority Over SDNN
    cur_live.execute("SELECT hrv_rmssd_raw_ms FROM daily_telemetry_master WHERE subject_id = 'pilot_live_01' AND minute_of_day = 600")
    m600 = cur_live.fetchone()
    assert m600[0] == 62.0, f"Expected RMSSD 62.0 to take precedence over SDNN 35.0, got {m600[0]}"
    print("✓ Beat-to-Beat RMSSD Precedence: Verified RMSSD (62.0 ms) prioritized over SDNN (35.0 ms)")

    # Verify BYOD Multi-Source Set-Cover Deduplication Permutation Invariance & Ambient Fallback
    cur_live.execute("SELECT step_count, photic_zeitgeber_index FROM daily_telemetry_master WHERE subject_id = 'pilot_live_01' AND minute_of_day = 720")
    m720 = cur_live.fetchone()
    assert m720[0] == 75, f"Expected deduplicated steps 75 (phone first, watch second), got {m720[0]}"
    assert m720[1] is not None and m720[1] > 5.0, f"Expected ambient daylight fallback Zeitgeber > 5.0, got {m720[1]}"

    cur_live.execute("SELECT step_count FROM daily_telemetry_master WHERE subject_id = 'pilot_live_01' AND minute_of_day = 721")
    m721 = cur_live.fetchone()
    assert m721[0] == 82, f"Expected deduplicated steps 82 (watch first, phone second), got {m721[0]}"
    print("✓ Permutation-Invariant Wearable Source Priority: Verified 75 steps (reverse order) & 82 steps (forward order) prioritized over phone")

    # Verify Workout Duration Second-to-Minute Normalization: 1800s normalized to 30 min (METs = 9.5), de-biasing minute 525 (08:45) HRV
    cur_live.execute("SELECT hrv_rmssd_raw_ms, hrv_rmssd_stress_ms, workout_met_intensity FROM daily_telemetry_master WHERE subject_id = 'pilot_live_01' AND minute_of_day = 525")
    m525 = cur_live.fetchone()
    assert m525[0] == 45.0, f"Expected raw HRV 45.0, got {m525[0]}"
    assert m525[1] > 45.0, f"Expected de-biased stress HRV > 45.0 (METs=9.5 active), got {m525[1]}"
    assert m525[2] >= 9.0, f"Expected workout METs >= 9.0 (normalized from 1800s to 30m), got {m525[2]}"
    print(f"✓ Workout Duration Normalization: 1800s workout normalized to 30m ({m525[2]:.2f} METs) correctly de-biased HRV from {m525[0]} to {m525[1]:.1f} ms")

    # Verify Sensor Fusion Non-Null Coalescing: Motion-only PIR event (lux=NULL) preserved Watch daylight lux 550.0
    cur_live.execute("SELECT bedroom_pir_motion, photic_zeitgeber_index FROM daily_telemetry_master WHERE subject_id = 'pilot_live_01' AND minute_of_day = 720")
    m720_fusion = cur_live.fetchone()
    assert m720_fusion[0] == 1, f"Expected bedroom PIR motion 1, got {m720_fusion[0]}"
    assert m720_fusion[1] is not None and m720_fusion[1] > 5.0, f"Expected photic zeitgeber preserved despite NULL lux PIR row, got {m720_fusion[1]}"
    print("✓ Non-Null Sensor Fusion: Verified PIR motion=1 and Watch daylight lux 550.0 coalesced without NULL clobbering")

    # Verify TimeInDaylight ('min') is NOT misread as lux, and 029 environmental_context_stream (300 lux @ 6500K) drives exact Zeitgeber
    cur_live.execute("SELECT photic_zeitgeber_index, ambient_pm25 FROM daily_telemetry_master WHERE subject_id = 'pilot_live_01' AND minute_of_day = 722")
    m722 = cur_live.fetchone()
    expected_z = math.log(1.0 + 300.0) * (6500.0 / 5500.0)
    assert m722[0] is not None and abs(m722[0] - expected_z) < 0.05, f"Expected 029 Zeitgeber {expected_z:.2f}, got {m722[0]}"
    assert abs(m722[1] - 4.2) < 1e-6, f"Expected 029 PM2.5 4.2, got {m722[1]}"
    print(f"✓ 029 Environmental Fusion & Unit Separation: TimeInDaylight(min) isolated; 300 Lux @ 6500K -> Zeitgeber={m722[0]:.2f}")

    # Verify TZ-aware BP dipping, cv_iti_pct unit separation (ms != %), and PK caffeine decay
    cur_live.execute("SELECT motor_cv_iti_pct, vascular_dipping_pct, caffeine_active_mg FROM daily_telemetry_master WHERE subject_id = 'pilot_live_01' AND minute_of_day = 720")
    m720_pk = cur_live.fetchone()
    assert abs(m720_pk[1] - 16.13) < 0.05, f"Expected TZ-aware BP dipping 16.13%, got {m720_pk[1]}"
    assert abs(m720_pk[2] - 100.0) < 0.05, f"Expected 200mg caffeine after 1 half-life (5h) == 100.0 mg, got {m720_pk[2]}"
    cur_live.execute("SELECT motor_cv_iti_pct, cog_ez_ddm_drift_v, cog_hick_slope_ms_per_bit, cog_practice_residual_ms FROM daily_telemetry_master WHERE subject_id = 'pilot_live_01' AND minute_of_day = 722")
    m722_cog = cur_live.fetchone()
    assert m722_cog[0] is None, "pvt_hick mean_rt_ms must NOT be written into motor_cv_iti_pct"
    assert m722_cog[1] is not None and abs(m722_cog[1] - 0.34) < 1e-6, f"Expected cog_ez_ddm_drift_v 0.34, got {m722_cog[1]}"
    assert m722_cog[2] is not None and abs(m722_cog[2] - 68.5) < 1e-6, f"Expected cog_hick_slope 68.5, got {m722_cog[2]}"
    assert m722_cog[3] is not None and abs(m722_cog[3] - (-12.4)) < 1e-6, f"Expected cog_practice_residual -12.4, got {m722_cog[3]}"
    cur_live.execute("SELECT motor_cv_iti_pct, cog_plv_r_sync, cog_nma_ms FROM daily_telemetry_master WHERE subject_id = 'pilot_live_01' AND minute_of_day = 723")
    m723_cv, m723_plv, m723_nma = cur_live.fetchone()
    assert m723_cv is not None and abs(m723_cv - 4.0) < 0.05, f"Expected beat_sync cv_iti_pct 4.0%, got {m723_cv}"
    assert m723_plv is not None and abs(m723_plv - 0.96) < 1e-6
    assert m723_nma is not None and abs(m723_nma - (-18.2)) < 1e-6
    print(f"✓ Metrology & PK Gates: BP dipping={m720_pk[1]}%, caffeine_active={m720_pk[2]}mg, beat_sync CV={m723_cv}%, EZ-DDM v={m722_cog[1]}, Hick={m722_cog[2]}")

    # 7. Stopping Rule Gate: 3 consecutive missing wrist temp nights + 10-min 55% motion artifact active-denominator trigger
    harvester_stop = TelemetryHarvester(db_path=test_db, subject_id="pilot_stop_01", cohort_stage=1)
    res_stop = harvester_stop.harvest_day(target_date=date(2026, 11, 15), day_index=4, simulated=False)
    assert res_stop["stopping_rule_triggered"] is True, "Stopping rule must trigger when 3 trailing nights lack wrist temp"
    harvester_motion = TelemetryHarvester(db_path=test_db, subject_id="pilot_motion_01", cohort_stage=1)
    harvester_motion.conn.execute("CREATE TABLE IF NOT EXISTS wetlab_eeg_fnirs_sessions (id TEXT, user_id TEXT, started_at DATETIME, fnirs_hbo2_umol REAL, fnirs_hbr_umol REAL, blink_artifact_pct REAL, frontal_midline_theta_uv2 REAL, frontal_alpha_asymmetry REAL, beta_alpha_arousal_ratio REAL, raw_features_json TEXT)")
    harvester_motion.conn.execute("INSERT INTO wetlab_eeg_fnirs_sessions VALUES ('w-m1', 'pilot_motion_01', '2026-11-15T20:05:10Z', 1.2, -0.3, 55.0, 5.0, 0.1, 0.9, '{}')")
    harvester_motion.conn.execute("INSERT INTO healthkit_records VALUES ('wt-m1', 'pilot_motion_01', 'p1', 'HKQuantityTypeIdentifierAppleSleepingWristTemperature', 'Apple Watch', '1.0', 'Watch', 'degC', 36.4, '2026-11-14T23:00:00Z', '2026-11-15T06:00:00Z', CURRENT_TIMESTAMP, '{}', '{}', CURRENT_TIMESTAMP)")
    harvester_motion.conn.commit()
    harvester_motion.execute_res = None
    res_motion = harvester_motion.harvest_day(target_date=date(2026, 11, 15), day_index=1, simulated=False)
    assert res_motion["avg_motion_artifact_pct"] == 55.0 and res_motion["stopping_rule_triggered"] is True, f"Expected active-window motion=55% stopping trigger, got {res_motion}"
    # Verify eeg_beta_alpha_arousal preserves direct beta/alpha polarity (2.40 != 1/2.40)
    harvester_motion.conn.execute("INSERT INTO wetlab_eeg_fnirs_sessions VALUES ('w-ar', 'pilot_motion_01', '2026-11-15T20:06:10Z', 1.2, -0.3, 10.0, 5.0, 0.1, 2.40, '{}')")
    harvester_motion.conn.commit()
    harvester_motion.harvest_day(target_date=date(2026, 11, 15), day_index=1, simulated=False)
    arousal_val = harvester_motion.conn.execute("SELECT eeg_beta_alpha_arousal FROM daily_telemetry_master WHERE subject_id='pilot_motion_01' AND minute_of_day=1206").fetchone()[0]
    assert arousal_val is not None and abs(arousal_val - 2.40) < 1e-6, f"Expected direct beta/alpha arousal 2.40, got {arousal_val}"
    harvester_motion.close()
    print("✓ Stopping Rule Gate: 3 missing nocturnal temp nights + active-wear 55% motion artifact triggered stopping rule")
    harvester_stop.close()

    # 8. Browser Study Bundle SHA-256 Tamper + Round-Trip Import Oracle
    import tempfile
    bundle_payload = {
        "schema_version": "docas-pilot1-v1",
        "exported_at": "2026-11-12T20:00:00Z",
        "user_id": "pilot_bundle_01",
        "counts": {"respiratory_strength_stream": 1},
        "tables": {
            "respiratory_strength_stream": [{
                "id": "resp-b1", "user_id": "pilot_bundle_01", "ts_utc": "2026-11-12T12:00:00Z",
                "mip_cmh2o": 112.5, "mep_cmh2o": 128.0, "vc_liters": 4.8, "posture_deg": 90.0,
                "source_device": "airofit_pro"
            }]
        }
    }
    manifest_hash = hashlib.sha256(json.dumps(bundle_payload, separators=(",", ":"), ensure_ascii=False).encode("utf-8")).hexdigest()
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as tf:
        json.dump({**bundle_payload, "sha256_manifest": manifest_hash}, tf)
        bundle_file = tf.name
    try:
        hb = TelemetryHarvester(db_path=test_db, cohort_stage=1) # default subject_id auto-adopts bundle["user_id"]
        imp = hb.import_study_bundle(bundle_file, verify_sha256=True)
        assert hb.subject_id == "pilot_bundle_01"
        assert imp["respiratory_strength_stream"] == 1
        hb.harvest_day(target_date=date(2026, 11, 12), day_index=1, simulated=False)
        mip_val = hb.conn.execute("SELECT smi_inspiratory_hold_sec FROM daily_telemetry_master WHERE subject_id='pilot_bundle_01' AND minute_of_day=720").fetchone()[0]
        assert abs(mip_val - 112.5) < 1e-6
        # Tamper test must reject
        with open(bundle_file, "w", encoding="utf-8") as tf2:
            json.dump({**bundle_payload, "sha256_manifest": "0" * 64}, tf2)
        tamper_rejected = False
        try:
            hb.import_study_bundle(bundle_file, verify_sha256=True)
        except ValueError:
            tamper_rejected = True
        assert tamper_rejected, "Tampered study bundle SHA-256 must raise ValueError"

        # AES-256-GCM encrypted bundle round-trip + wrong-passphrase rejection
        import base64
        from cryptography.hazmat.primitives.ciphers.aead import AESGCM
        salt = os.urandom(16)
        iv = os.urandom(12)
        key = hashlib.pbkdf2_hmac("sha256", b"pilot-pass-42", salt, 600_000, dklen=32)
        ct = AESGCM(key).encrypt(iv, json.dumps({**bundle_payload, "sha256_manifest": manifest_hash}).encode("utf-8"), None)
        enc_bundle = {
            "schema_version": "docas-pilot1-enc-v1",
            "exported_at": bundle_payload["exported_at"],
            "pseudonymized_subject_id": "sub_deadbeef",
            "kdf": "PBKDF2-SHA256",
            "kdf_iterations": 600_000,
            "cipher": "AES-256-GCM",
            "salt_b64": base64.b64encode(salt).decode("ascii"),
            "iv_b64": base64.b64encode(iv).decode("ascii"),
            "ciphertext_b64": base64.b64encode(ct).decode("ascii"),
        }
        with open(bundle_file, "w", encoding="utf-8") as tf3:
            json.dump(enc_bundle, tf3)
        imp_enc = hb.import_study_bundle(bundle_file, verify_sha256=True, passphrase="pilot-pass-42")
        assert imp_enc["respiratory_strength_stream"] == 1

        # Pseudonymized encrypted bundle (no root "user_id", only "pseudonymized_subject_id") must import cleanly
        pseudo_payload = {
            "schema_version": "docas-pilot1-v1",
            "exported_at": "2026-11-12T20:05:00Z",
            "pseudonymized_subject_id": "sub_deadbeef00112233",
            "counts": {"respiratory_strength_stream": 1},
            "tables": {
                "respiratory_strength_stream": [{
                    "id": "resp-p1", "user_id": "sub_deadbeef00112233", "ts_utc": "2026-11-12T12:05:00Z",
                    "mip_cmh2o": 108.0, "mep_cmh2o": 120.0, "vc_liters": 4.5, "posture_deg": 90.0,
                    "source_device": "airofit_pro"
                }]
            }
        }
        p_hash = hashlib.sha256(json.dumps(pseudo_payload, separators=(",", ":"), ensure_ascii=False).encode("utf-8")).hexdigest()
        p_ct = AESGCM(key).encrypt(iv, json.dumps({**pseudo_payload, "sha256_manifest": p_hash}).encode("utf-8"), None)
        with open(bundle_file, "w", encoding="utf-8") as tf4:
            json.dump({**enc_bundle, "pseudonymized_subject_id": "sub_deadbeef00112233", "ciphertext_b64": base64.b64encode(p_ct).decode("ascii")}, tf4)
        hp = TelemetryHarvester(db_path=test_db, cohort_stage=1)
        imp_p = hp.import_study_bundle(bundle_file, verify_sha256=True, passphrase="pilot-pass-42")
        assert hp.subject_id == "sub_deadbeef00112233" and imp_p["respiratory_strength_stream"] == 1
        hp.close()
        wrong_pw_rejected = False
        try:
            hb.import_study_bundle(bundle_file, verify_sha256=True, passphrase="wrong-password")
        except Exception:
            wrong_pw_rejected = True
        assert wrong_pw_rejected, "Wrong AES-GCM passphrase must fail authentication tag"
        hb.close()
        print("✓ Study Bundle Round-Trip, AES-256-GCM Decryption & Tamper Oracle: Verified 028+029 import & auth tags")
    finally:
        if os.path.exists(bundle_file):
            os.remove(bundle_file)

    # 9. Monte Carlo AR(1) (rho=0.7) Null Calibration Oracle (Type-I Error <= 0.08 across 200 trials)
    import random
    rng = random.Random(20261115)
    false_positives = 0
    for _ in range(200):
        xs, ys = [0.0], [0.0]
        for _t in range(59):
            xs.append(0.7 * xs[-1] + rng.gauss(0.0, 1.0))
            ys.append(0.7 * ys[-1] + rng.gauss(0.0, 1.0))
        if evaluate_ar1_concordance(xs, ys)["p_value"] < 0.05:
            false_positives += 1
    fpr = false_positives / 200.0
    assert fpr <= 0.08, f"AR(1) null Type-I error inflated to {fpr:.3f}"
    ar_nan = evaluate_ar1_concordance([1.0, None, 2.0, 3.0, 4.0, 5.0, float("nan"), 6.0], [2.0, 9.0, 4.0, 6.0, 8.0, 10.0, 1.0, 12.0])
    assert ar_nan["r"] > 0.99, f"Expected gap-tolerant AR(1) concordance > 0.99, got {ar_nan}"
    assert canonicalize_jcs({"b": 90.0, "a": -0.0, "c": 1e-7}) == '{"a":0,"b":90,"c":1e-7}'
    for num_val, exp_str in ((1e-5, "0.00001"), (1e-6, "0.000001"), (8.33e-5, "0.0000833"), (1e16, "10000000000000000"), (1e21, "1e+21")):
        assert canonicalize_jcs(num_val) == exp_str, f"JCS mismatch for {num_val}: {canonicalize_jcs(num_val)} != {exp_str}"
    print(f"✓ Monte Carlo AR(1) Null Calibration: Type-I error = {fpr:.3f} (<= 0.08 target under rho=0.7 red noise)")

    # 10. Multi-Day Sparse + Dense Parquet Schema Unification Oracle (zero Arrow null-type merge errors)
    try:
        import pyarrow as pa
        import pyarrow.dataset as ds
        with tempfile.TemporaryDirectory() as pdir:
            hs = TelemetryHarvester(db_path=test_db, subject_id="pilot_sparse_01", cohort_stage=1)
            hs.harvest_day(target_date=date(2026, 11, 10), day_index=1, simulated=False)
            p_sparse = hs.export_parquet(output_dir=os.path.join(pdir, "d1"))
            hs.close()
            hd = TelemetryHarvester(db_path=test_db, subject_id="pilot_dense_01", cohort_stage=1)
            hd.conn.execute("CREATE TABLE IF NOT EXISTS cognitive_battery_sessions (id TEXT, user_id TEXT, task_type TEXT, started_at DATETIME, mean_rt_ms REAL, sigma_clock_sq REAL, sigma_motor_sq REAL, metrics_json TEXT, ez_ddm_drift_v REAL, ez_ddm_boundary_a REAL, ez_ddm_ter_s REAL, hick_slope_ms_per_bit REAL, d_prime REAL, plv_r_sync REAL, nma_ms REAL, max_span INTEGER, practice_residual_ms REAL)")
            hd.conn.execute("INSERT INTO cognitive_battery_sessions VALUES ('cs-sp', 'pilot_dense_01', 'spatial_span', '2026-11-11T12:00:00Z', NULL, NULL, NULL, '{}', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 7, NULL)")
            hd.conn.commit()
            hd.harvest_day(target_date=date(2026, 11, 11), day_index=2, simulated=False)
            p_dense = hd.export_parquet(output_dir=os.path.join(pdir, "d2"))
            hd.close()
            unified = ds.dataset([p_sparse, p_dense], format="parquet").to_table()
            assert unified.schema.field("cog_max_span").type == pa.int64()
            assert unified.num_rows == 2880
            print("✓ Multi-Day Arrow Schema Unification Oracle: Merged sparse (all-NULL) + dense Parquet (2,880 epochs)")
    except ImportError:
        pass

    harvester_live.close()

    if os.path.exists(test_db):
        os.remove(test_db)
    print("ALL HARVESTER GATES PASSED (100% HEALTHY)\n")
    return True

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="DOCAS-PILOT-1: Daily Telemetry Harvester")
    parser.add_argument("--self-test", action="store_true", help="Run automated self-test on filters and 1440-epoch grid")
    parser.add_argument("--subject-id", type=str, default="pilot-user-1", help="Participant User ID")
    parser.add_argument("--cohort", type=int, default=1, choices=[1, 2, 3, 4], help="Cohort Stage (1: 180d, 2: 10d, 3: 3d, 4: 1d)")
    parser.add_argument("--day", type=int, default=1, help="Day index in cohort")
    parser.add_argument("--db-path", type=str, default="trust_wearables_edge.db", help="Local edge SQLite database path")
    parser.add_argument("--export-dir", type=str, default="data/pilot1/parquet", help="Parquet export directory")
    parser.add_argument("--tz-offset-hours", type=float, default=0.0, help="Fallback UTC offset hours for naive/Z timestamps")
    parser.add_argument("--import-bundle", type=str, default=None, help="Path to browser-exported docas-pilot1-*.json study bundle")
    parser.add_argument("--bundle-passphrase", type=str, default=None, help="Optional AES-256-GCM passphrase for encrypted study bundles")
    args = parser.parse_args()

    if args.self_test:
        success = run_self_test()
        sys.exit(0 if success else 1)

    harvester = TelemetryHarvester(db_path=args.db_path, subject_id=args.subject_id, cohort_stage=args.cohort, tz_offset_hours=args.tz_offset_hours)
    if args.import_bundle:
        imp = harvester.import_study_bundle(args.import_bundle, verify_sha256=True, passphrase=args.bundle_passphrase)
        print(f"Imported study bundle {args.import_bundle}: {imp}")
    today = date.today()
    print(f"Harvesting daily telemetry for Subject {args.subject_id} on {today} (Cohort {args.cohort}, Day {args.day})...")
    res = harvester.harvest_day(target_date=today, day_index=args.day, body_fat_pct=18.5)
    out_file = harvester.export_parquet(output_dir=args.export_dir)
    harvester.close()
    print(f"Harvest complete. Compiled {res['epochs_compiled']} epochs to {args.db_path} and exported to {out_file}.")
