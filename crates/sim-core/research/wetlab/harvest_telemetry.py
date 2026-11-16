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
    
    -- Quality Gates
    fnirs_motion_artifact_pct REAL DEFAULT 0.0,
    stopping_rule_triggered INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_telemetry_master_time ON daily_telemetry_master(subject_id, epoch_timestamp);
CREATE INDEX IF NOT EXISTS idx_telemetry_master_day ON daily_telemetry_master(day_index);
"""

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

class TelemetryHarvester:
    def __init__(self, db_path: str = "trust_wearables_edge.db", subject_id: str = "pilot-user-1", cohort_stage: int = 1):
        self.db_path = db_path
        self.subject_id = subject_id
        self.cohort_stage = cohort_stage
        dirname = os.path.dirname(os.path.abspath(db_path))
        if dirname:
            os.makedirs(dirname, exist_ok=True)
        self.conn = sqlite3.connect(db_path)
        self.conn.executescript(SCHEMA_HARVEST_SQL)
        self.conn.commit()

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
            else:
                cur.execute("""
                    SELECT fnirs_hbo2_umol, fnirs_hbr_umol, blink_artifact_pct
                    FROM wetlab_eeg_fnirs_sessions
                    WHERE user_id = ? AND started_at >= ? AND started_at < ?
                    ORDER BY started_at DESC LIMIT 1
                """, (self.subject_id, epoch_start_iso, epoch_end_iso))
                w_row = cur.fetchone()
                if w_row and w_row[0] is not None:
                    res["hbo2"] = round(float(w_row[0]), 3)
                    res["hbr"] = round(float(w_row[1]), 3)
                    res["motion_artifact_pct"] = float(w_row[2] or 0.0)
        except Exception:
            pass

        # 2. Ambient Environment (Aranet4 Home/PRO + Outdoor PM2.5)
        try:
            cur.execute("""
                SELECT co2_ppm, voc_ppb, pm25_ug_m3
                FROM ambient_environment_stream
                WHERE user_id = ? AND ts_utc >= ? AND ts_utc < ?
                ORDER BY ts_utc DESC LIMIT 1
            """, (self.subject_id, epoch_start_iso, epoch_end_iso))
            row = cur.fetchone()
            if row:
                res["co2"] = row[0]
                res["voc"] = row[1]
                res["pm25"] = row[2]
        except Exception:
            pass

        # 3. Respiratory Strength (Airofit Pro MIP/MEP/VC, with legacy SMI fallback)
        try:
            cur.execute("""
                SELECT best_mip_cmh2o, mep_cmh2o, vc_liters FROM respiratory_strength_stream
                WHERE user_id = ? AND ts_utc >= ? AND ts_utc < ?
                ORDER BY ts_utc DESC LIMIT 1
            """, (self.subject_id, epoch_start_iso, epoch_end_iso))
            row = cur.fetchone()
            if row:
                res["airofit_mip"] = row[0]
                res["airofit_mep"] = row[1]
                res["airofit_vc"] = row[2]
        except Exception:
            pass
        if "airofit_mip" not in res:
            try:
                cur.execute("""
                    SELECT best_hold_sec FROM sustained_inhalation_stream
                    WHERE user_id = ? AND ts_utc >= ? AND ts_utc < ?
                    ORDER BY ts_utc DESC LIMIT 1
                """, (self.subject_id, epoch_start_iso, epoch_end_iso))
                row = cur.fetchone()
                if row:
                    res["smi_hold"] = row[0]
            except Exception:
                pass

        # 4. Reading Logs
        try:
            cur.execute("""
                SELECT pages_read, flesch_kincaid_grade, latency_sec
                FROM manual_reading_log_stream
                WHERE user_id = ? AND digitized_utc >= ? AND digitized_utc < ?
                ORDER BY digitized_utc DESC LIMIT 1
            """, (self.subject_id, epoch_start_iso, epoch_end_iso))
            row = cur.fetchone()
            if row:
                res["pages"] = row[0]
                res["fk_grade"] = row[1]
                res["read_latency"] = row[2]
        except Exception:
            pass

        # 5. Financial Shocks
        try:
            cur.execute("""
                SELECT settled_amount, prediction_error_amount, trust_outcome
                FROM financial_trust_ledger
                WHERE user_id = ? AND settled_datetime >= ? AND settled_datetime < ?
                ORDER BY settled_datetime DESC LIMIT 1
            """, (self.subject_id, epoch_start_iso, epoch_end_iso))
            row = cur.fetchone()
            if row:
                res["plaid_shock"] = row[0]
                res["plaid_pred_err"] = row[1]
                res["plaid_outcome"] = row[2]
        except Exception:
            pass

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
                if len(row) > 3 and row[3] is not None:
                    res["sws_ratio"] = row[3]
            else:
                cur.execute("""
                    SELECT frontal_midline_theta_uv2, frontal_alpha_asymmetry, beta_alpha_arousal_ratio
                    FROM wetlab_eeg_fnirs_sessions
                    WHERE user_id = ? AND started_at >= ? AND started_at < ?
                    ORDER BY started_at DESC LIMIT 1
                """, (self.subject_id, epoch_start_iso, epoch_end_iso))
                w_row = cur.fetchone()
                if w_row and w_row[0] is not None:
                    res["fm_theta"], res["faa"] = w_row[0], w_row[1]
                    res["tbr"] = 1.0 / max(1e-4, float(w_row[2] or 1.0))
        except Exception:
            pass

        # 7. Cognitive Battery Striatal Tapping
        try:
            cur.execute("""
                SELECT cv_iti_pct FROM cognitive_battery_tapping
                WHERE user_id = ? AND started_at >= ? AND started_at < ?
                ORDER BY started_at DESC LIMIT 1
            """, (self.subject_id, epoch_start_iso, epoch_end_iso))
            row = cur.fetchone()
            if row:
                res["cv_iti_pct"] = row[0]
        except Exception:
            pass

        # 8. Apple HealthKit Records / Dexcom CGM / Sleeping Wrist Temp / Sleep Analysis / Ambient Daylight Fallback
        try:
            cur.execute("""
                SELECT record_type, value, metadata, source_name FROM healthkit_records
                WHERE user_id = ? AND (
                    (start_date >= ? AND start_date < ?)
                    OR (record_type LIKE '%SleepAnalysis%' AND start_date < ? AND end_date > ?)
                )
            """, (self.subject_id, epoch_start_iso, epoch_end_iso, epoch_end_iso, epoch_start_iso))
            for rec in cur.fetchall():
                t, v = rec[0], rec[1]
                m = rec[2] if len(rec) > 2 else None
                s_name = str(rec[3] if len(rec) > 3 else "unknown").lower()
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
                    res["raw_glucose"] = v
                elif "AppleSleepingWristTemperature" in t:
                    res["wrist_temp"] = v
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
                elif "TimeInDaylight" in t or "AmbientLight" in t or "AmbientLux" in t:
                    if res.get("lux") is None:
                        res["lux"] = float(v)
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
                WHERE user_id = ? AND ts_utc >= ? AND ts_utc < ?
                ORDER BY ts_utc DESC LIMIT 1
            """, (self.subject_id, epoch_start_iso, epoch_end_iso))
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
                WHERE user_id = ? AND ts_utc >= ? AND ts_utc < ?
                ORDER BY ts_utc DESC LIMIT 1
            """, (self.subject_id, epoch_start_iso, epoch_end_iso))
            row = cur.fetchone()
            if row:
                res["bristol"] = row[0]
        except Exception:
            pass

        return res

    def harvest_day(self, target_date: date, day_index: int, body_fat_pct: float = 18.5, simulated: bool = False) -> Dict[str, Any]:
        """Compile all 25 streams across 1,440 one-minute epochs for target_date."""
        calibrated_dpf = cranial_dpf_calibration(6.0, body_fat_pct)
        epochs: List[Tuple] = []
        motion_artifact_total_pct = 0.0
        cur = self.conn.cursor()

        # Pre-load meals and workouts across [target_date - 4h, target_date + 24h]
        day_start_dt = datetime(target_date.year, target_date.month, target_date.day, 0, 0, 0, tzinfo=timezone.utc)
        scan_start_iso = (day_start_dt - timedelta(hours=4)).strftime("%Y-%m-%dT%H:%M:%SZ")
        scan_end_iso = (day_start_dt + timedelta(hours=24)).strftime("%Y-%m-%dT%H:%M:%SZ")

        meal_events: List[Tuple[float, float]] = []
        try:
            cur.execute("""
                SELECT start_date, value FROM healthkit_records
                WHERE user_id = ? AND record_type LIKE '%DietaryCarbohydrates%'
                  AND start_date >= ? AND start_date <= ?
            """, (self.subject_id, scan_start_iso, scan_end_iso))
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
                WHERE user_id = ? AND start_date >= ? AND start_date <= ?
            """, (self.subject_id, scan_start_iso, scan_end_iso))
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

        dipping_pct = None
        if not simulated:
            try:
                cur.execute("""
                    SELECT
                        AVG(CASE WHEN strftime('%H:%M', start_date) BETWEEN '08:00' AND '22:00' THEN value END),
                        AVG(CASE WHEN strftime('%H:%M', start_date) BETWEEN '00:00' AND '06:00' THEN value END)
                    FROM healthkit_records
                    WHERE user_id = ? AND record_type LIKE '%BloodPressureSystolic%'
                      AND start_date >= ? AND start_date <= ?
                """, (self.subject_id, scan_start_iso, scan_end_iso))
                bp_row = cur.fetchone()
                if bp_row and bp_row[0] is not None and bp_row[1] is not None and float(bp_row[0]) > 0:
                    dipping_pct = round(((float(bp_row[0]) - float(bp_row[1])) / float(bp_row[0])) * 100.0, 2)
            except Exception:
                pass

        daily_rem_pct = None
        if not simulated:
            try:
                cur.execute("""
                    SELECT value, start_date, end_date
                    FROM healthkit_records
                    WHERE user_id = ? AND record_type LIKE '%SleepAnalysis%'
                      AND start_date < ? AND end_date > ?
                """, (self.subject_id, scan_end_iso, scan_start_iso))
                sleep_recs = cur.fetchall()
                if sleep_recs:
                    total_sleep_sec = 0.0
                    rem_sec = 0.0
                    scan_t0 = datetime.fromisoformat(scan_start_iso.replace("Z", "+00:00")).timestamp()
                    scan_t1 = datetime.fromisoformat(scan_end_iso.replace("Z", "+00:00")).timestamp()
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

        last_glucose_val = None
        last_glucose_min = None

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
                dipping = dipping_pct

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
                zeitgeber = photic_zeitgeber_filter(lux, 4500.0) if lux is not None else None
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

            motion_artifact_total_pct += motion_pct
            epochs.append((
                epoch_id, self.subject_id, self.cohort_stage, epoch_ts, day_index, minute,
                hbo2, hbr, round(calibrated_dpf, 3),
                fm_theta if simulated else real.get("fm_theta"),
                faa if simulated else real.get("faa"),
                tbr if simulated else real.get("tbr"),
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
                wrist_temp, delta_wrist_temp if simulated else (round(wrist_temp - 36.40, 3) if wrist_temp else None),
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
                pages, fk_grade, read_latency, motion_pct, 0
            ))

        avg_motion = motion_artifact_total_pct / 1440.0
        missing_temp_nights = 0
        if not simulated and day_index >= 3:
            try:
                cur.execute("""
                    SELECT COUNT(DISTINCT strftime('%Y-%m-%d', start_date))
                    FROM healthkit_records
                    WHERE user_id = ? AND record_type LIKE '%AppleSleepingWristTemperature%'
                      AND start_date >= date(?, '-3 days') AND start_date < ?
                """, (self.subject_id, target_date.isoformat(), target_date.isoformat()))
                row = cur.fetchone()
                if row and row[0] == 0:
                    missing_temp_nights = 3
            except Exception:
                pass

        stop_rule = 1 if (avg_motion > 40.0 or missing_temp_nights >= 3) else 0
        epochs_with_stop = [
            tup[:-1] + (stop_rule,) for tup in epochs
        ]
        self.conn.executemany(
            "INSERT OR REPLACE INTO daily_telemetry_master VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)",
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
            table_dict = {col: [row[i] for row in rows] for i, col in enumerate(col_names)}
            arrow_table = pa.Table.from_pydict(table_dict)
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
    harvester_live.conn.execute("INSERT INTO ambient_environment_stream VALUES ('amb-1', 'pilot_live_01', '2026-11-12T14:30:15Z', 485.0, 62.0, 4.8, 21.5, 45.0, 92.0, 'aranet4_home', CURRENT_TIMESTAMP)")
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



    # Seed reading log at 20:30:15Z (minute 1230)
    harvester_live.conn.execute("CREATE TABLE IF NOT EXISTS manual_reading_log_stream (id TEXT, user_id TEXT, session_id TEXT, isbn TEXT, book_title TEXT, read_start_utc DATETIME, read_end_utc DATETIME, digitized_utc DATETIME, latency_sec REAL, pages_read INTEGER, ocr_text TEXT, word_count INTEGER, syllable_count INTEGER, sentence_count INTEGER, flesch_kincaid_grade REAL, semantic_valence REAL, is_nce_control INTEGER, created_at DATETIME)")
    harvester_live.conn.execute("INSERT INTO manual_reading_log_stream VALUES ('rd-1', 'pilot_live_01', 's1', '123', 'Book', '2026-11-12T20:00:00Z', '2026-11-12T20:30:00Z', '2026-11-12T20:30:15Z', 45.0, 20, 'text', 100, 150, 10, 11.5, 0.25, 0, CURRENT_TIMESTAMP)")
    # Seed rogue subject reading log
    harvester_live.conn.execute("INSERT INTO manual_reading_log_stream VALUES ('rd-rogue', 'rogue_sub_99', 's99', '999', 'RogueBook', '2026-11-12T20:00:00Z', '2026-11-12T20:30:00Z', '2026-11-12T20:30:15Z', 99.0, 999, 'text', 999, 999, 10, 99.0, 0.0, 0, CURRENT_TIMESTAMP)")

    harvester_live.conn.commit()
    res_live = harvester_live.harvest_day(target_date=date(2026, 11, 12), day_index=1, simulated=False)
    assert res_live["epochs_compiled"] == 1440
    cur_live = harvester_live.conn.cursor()
    cur_live.execute("SELECT ambient_co2_ppm, glucose_raw_mgdl FROM daily_telemetry_master WHERE subject_id = 'pilot_live_01' AND minute_of_day = 870")
    m870 = cur_live.fetchone()
    assert m870[0] == 485.0, f"Expected subject's Aranet4 CO2 485.0 (isolated from rogue 9999.0), got {m870[0]}"

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

    cur_live.execute("SELECT ambient_co2_ppm, glucose_raw_mgdl, reading_flesch_kincaid FROM daily_telemetry_master WHERE subject_id = 'pilot_live_01' AND minute_of_day = 0")
    m0 = cur_live.fetchone()
    assert m0[0] is None and m0[1] is None and m0[2] is None, f"Expected unseeded epoch to be NULL, got {m0}"
    print("✓ Live-Mode SQL Oracle: Real SQL rows populated without synthetic leakage (Unobserved epochs = NULL)")

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

    # 7. Stopping Rule Gate: 3 consecutive missing wrist temp nights triggers stopping rule on day_index >= 3
    harvester_stop = TelemetryHarvester(db_path=test_db, subject_id="pilot_stop_01", cohort_stage=1)
    res_stop = harvester_stop.harvest_day(target_date=date(2026, 11, 15), day_index=4, simulated=False)
    assert res_stop["stopping_rule_triggered"] is True, "Stopping rule must trigger when 3 trailing nights lack wrist temp"
    print("✓ Stopping Rule Gate: 3 consecutive missing nocturnal temp nights triggered study stopping rule")
    harvester_stop.close()

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
    args = parser.parse_args()

    if args.self_test:
        success = run_self_test()
        sys.exit(0 if success else 1)

    harvester = TelemetryHarvester(db_path=args.db_path, subject_id=args.subject_id, cohort_stage=args.cohort)
    today = date.today()
    print(f"Harvesting daily telemetry for Subject {args.subject_id} on {today} (Cohort {args.cohort}, Day {args.day})...")
    res = harvester.harvest_day(target_date=today, day_index=args.day, body_fat_pct=18.5)
    out_file = harvester.export_parquet(output_dir=args.export_dir)
    harvester.close()
    print(f"Harvest complete. Compiled {res['epochs_compiled']} epochs to {args.db_path} and exported to {out_file}.")
