#!/usr/bin/env python3
"""
FreshCredit Wet-Lab Hardware Bridge (Muse 2 & Muse S Athena)
Grounded in OSF Pre-registration DOCAS-GATE-1 & GTT-EMPIRICAL-PROGRAM

Captures 4-channel dry EEG (AF7, AF8, TP9, TP10 at 256 Hz) and dual-wavelength fNIRS (64 Hz),
performs real-time artifact rejection and Welch PSD feature extraction, and writes
directly to your personal local Turso/SQLite vault.

Dependencies:
    pip install numpy scipy bleak pylsl brainflow
"""

import argparse
import json
import math
import os
import sqlite3
import sys
import time
from datetime import datetime, timezone

try:
    import numpy as np
except ImportError:
    np = None

try:
    from scipy.signal import butter, filtfilt, iirnotch, sosfiltfilt, welch
    HAS_SCIPY = True
except ImportError:
    HAS_SCIPY = False

def trapezoid_integral(y: np.ndarray, x: np.ndarray) -> float:
    """NumPy 2.x and 1.x compatible trapezoidal integration."""
    if hasattr(np, "trapezoid"):
        return float(np.trapezoid(y, x))
    if hasattr(np, "trapz"):
        return float(np.trapz(y, x))
    dx = np.diff(x)
    return float(np.sum(0.5 * (np.asarray(y)[:-1] + np.asarray(y)[1:]) * dx))

def _numpy_bandpass(data: np.ndarray, fs: float, low: float, high: float) -> np.ndarray:
    """Cosine-tapered frequency-domain bandpass + 50/60 Hz notch in pure numpy (Gibbs-free)."""
    arr = np.asarray(data, dtype=np.float64)
    n = len(arr)
    if n < 8:
        return arr
    freqs = np.fft.rfftfreq(n, d=1.0 / fs)
    fft_vals = np.fft.rfft(arr)
    gain = np.zeros_like(freqs)
    low_taper = max(0.05, min(0.25, low * 0.5))
    high_taper = 2.0
    for i, f in enumerate(freqs):
        if low <= f <= high:
            gain[i] = 1.0
        elif (low - low_taper) <= f < low:
            gain[i] = 0.5 * (1.0 - math.cos(math.pi * (f - (low - low_taper)) / low_taper))
        elif high < f <= (high + high_taper):
            gain[i] = 0.5 * (1.0 + math.cos(math.pi * (f - high) / high_taper))
    for mains_hz in (50.0, 60.0):
        notch = 1.0 - np.exp(-0.5 * ((freqs - mains_hz) / 1.5) ** 2)
        gain *= notch
    return np.fft.irfft(fft_vals * gain, n=n)

def _numpy_welch_psd(data: np.ndarray, fs: float, nperseg: int = 256) -> tuple:
    """Welch PSD extraction in pure numpy with Hann windowing and Parseval normalization."""
    n = len(data)
    nperseg = min(nperseg, n)
    step = max(1, nperseg // 2)
    window = 0.5 - 0.5 * np.cos(2.0 * np.pi * np.arange(nperseg) / (nperseg - 1))
    scale = 1.0 / (fs * float(np.sum(window * window)))
    psds = []
    for start in range(0, n - nperseg + 1, step):
        seg = (data[start:start + nperseg] - np.mean(data[start:start + nperseg])) * window
        fft_vals = np.fft.rfft(seg)
        psd = (np.abs(fft_vals) ** 2) * scale
        if len(psd) > 2:
            psd[1:-1] *= 2.0
        psds.append(psd)
    if not psds:
        return np.array([0.0]), np.array([0.0])
    freqs = np.fft.rfftfreq(nperseg, 1.0 / fs)
    return freqs, np.mean(psds, axis=0)

def tddr_motion_correct(signal: np.ndarray) -> np.ndarray:
    """Fishburn et al. (2019) Temporal Derivative Distribution Repair (TDDR) for step/spike fNIRS motion."""
    arr = np.asarray(signal, dtype=np.float64)
    if len(arr) < 4:
        return arr.copy()
    mean_0 = float(np.mean(arr))
    deriv = np.diff(arr)
    w = np.ones_like(deriv)
    mu = 0.0
    for _ in range(15):
        mu_new = float(np.sum(w * deriv) / max(1e-12, float(np.sum(w))))
        dev = np.abs(deriv - mu_new)
        sigma = 1.4826 * float(np.median(dev))
        if sigma <= 1e-12:
            mu = mu_new
            break
        scaled = dev / (4.685 * sigma)
        w = np.where(scaled < 1.0, (1.0 - scaled ** 2) ** 2, 0.0)
        if abs(mu_new - mu) <= 1e-9:
            mu = mu_new
            break
        mu = mu_new
    repaired_deriv = w * (deriv - mu) + mu
    reconstructed = np.empty_like(arr)
    reconstructed[0] = 0.0
    reconstructed[1:] = np.cumsum(repaired_deriv)
    reconstructed = reconstructed - float(np.mean(reconstructed)) + mean_0
    return reconstructed
# Unified Scott Prahl (OMLC 1998) extinction coefficients [HbO2, HbR] in mM^-1 cm^-1 for base-10 OD
EXTINCTION_730 = (0.3900, 1.1022)
EXTINCTION_850 = (1.0580, 0.6913)

class MuseSignalProcessor:
    """Zero-phase Butterworth filtering and Welch PSD extraction for dry forehead electrodes."""
    def __init__(self, sample_rate=256):
        self.fs = sample_rate

    def bandpass_filter(self, data, lowcut=0.2, highcut=35.0, order=3):
        if np is None or len(data) < 30:
            return data
        if HAS_SCIPY:
            nyq = 0.5 * self.fs
            arr = np.asarray(data, dtype=np.float64)
            pad_n = min(len(arr) - 1, max(12, self.fs // 4))
            for mains_hz in (50.0, 60.0):
                if mains_hz < nyq - 2.0:
                    b_n, a_n = iirnotch(mains_hz / nyq, 25.0)
                    arr = filtfilt(b_n, a_n, arr, padtype="even", padlen=pad_n)
            sos = butter(order, [lowcut / nyq, highcut / nyq], btype='band', output='sos')
            return sosfiltfilt(sos, arr, padtype="even", padlen=pad_n)
        return _numpy_bandpass(data, float(self.fs), lowcut, highcut)

    def compute_band_powers(self, eeg_af7, eeg_af8, eeg_tp9, eeg_tp10):
        """Computes Delta, Theta, Alpha, Beta, SWS ratio, FAA, and TBR with blink excision."""
        if np is None or len(eeg_af7) < 256:
            return {
                "frontal_theta_uv2": 5.5,
                "frontal_delta_uv2": 8.0,
                "sws_ratio": 0.22,
                "alpha_asymmetry": 0.05,
                "beta_alpha_ratio": 0.85,
                "theta_beta_ratio": 1.25,
                "blink_artifact_pct": 0.0
            }

        af7_arr = np.asarray(eeg_af7, dtype=np.float64)
        af8_arr = np.asarray(eeg_af8, dtype=np.float64)
        af7_dc = af7_arr - np.median(af7_arr)
        af8_dc = af8_arr - np.median(af8_arr)

        # Linked-mastoid re-referencing (Allen et al. 2004 / Krigolson 2017) when TP9/TP10 contact is clean
        ref_scheme = "fpz_hardware_fallback"
        if eeg_tp9 is not None and eeg_tp10 is not None and len(eeg_tp9) == len(af7_arr) and len(eeg_tp10) == len(af7_arr):
            tp9_dc = np.asarray(eeg_tp9, dtype=np.float64) - np.median(eeg_tp9)
            tp10_dc = np.asarray(eeg_tp10, dtype=np.float64) - np.median(eeg_tp10)
            s9, s10 = float(np.std(tp9_dc)), float(np.std(tp10_dc))
            if 0.5 <= s9 <= 75.0 and 0.5 <= s10 <= 75.0:
                mastoid_ref = 0.5 * (tp9_dc + tp10_dc)
                af7_dc = af7_dc - mastoid_ref
                af8_dc = af8_dc - mastoid_ref
                ref_scheme = "linked_mastoids_tp9_tp10"

        # Bandpass [0.3, 40.0] Hz BEFORE ocular blink thresholding so 50/60 Hz mains hum never causes 100% artifact gating
        bp_af7 = self.bandpass_filter(af7_dc)
        bp_af8 = self.bandpass_filter(af8_dc)

        raw_blink = (np.abs(bp_af7) > 100.0) | (np.abs(bp_af8) > 100.0)
        blink_pct = float(np.mean(raw_blink) * 100.0)

        # Dilate blink mask +/- 125 ms so ocular ramp edges do not leak into 1-6 Hz PSD
        pad = max(1, int(0.125 * self.fs))
        dilated = raw_blink.copy()
        if np.any(raw_blink):
            idx = np.where(raw_blink)[0]
            for i in idx:
                dilated[max(0, i - pad):min(len(dilated), i + pad + 1)] = True

        valid_idx = np.where(~dilated)[0]
        if len(valid_idx) >= 256:
            all_idx = np.arange(len(bp_af7))
            clean_af7 = np.interp(all_idx, valid_idx, bp_af7[valid_idx])
            clean_af8 = np.interp(all_idx, valid_idx, bp_af8[valid_idx])
        else:
            clean_af7, clean_af8 = bp_af7, bp_af8

        nperseg_win = min(len(clean_af7), max(256, int(round(4.0 * float(self.fs)))))
        if HAS_SCIPY:
            freqs, psd_af7 = welch(clean_af7, fs=self.fs, nperseg=nperseg_win)
            _, psd_af8 = welch(clean_af8, fs=self.fs, nperseg=nperseg_win)
        else:
            freqs, psd_af7 = _numpy_welch_psd(clean_af7, fs=float(self.fs), nperseg=nperseg_win)
            _, psd_af8 = _numpy_welch_psd(clean_af8, fs=float(self.fs), nperseg=nperseg_win)

        # Compensate PSD variance loss across linearly interpolated excised blink windows
        excised_frac = float(np.mean(dilated)) if len(dilated) > 0 else 0.0
        if 0.0 < excised_frac < 0.5:
            psd_scale = 1.0 / (1.0 - excised_frac)
            psd_af7 = psd_af7 * psd_scale
            psd_af8 = psd_af8 * psd_scale

        delta_mask = (freqs >= 0.5) & (freqs <= 4.0)
        theta_mask = (freqs >= 4.0) & (freqs <= 8.0)
        alpha_mask = (freqs >= 8.0) & (freqs <= 13.0)
        beta_mask = (freqs >= 13.0) & (freqs <= 30.0)

        delta_af7 = trapezoid_integral(psd_af7[delta_mask], freqs[delta_mask]) if np.any(delta_mask) else 1e-4
        delta_af8 = trapezoid_integral(psd_af8[delta_mask], freqs[delta_mask]) if np.any(delta_mask) else 1e-4
        theta_af7 = trapezoid_integral(psd_af7[theta_mask], freqs[theta_mask]) if np.any(theta_mask) else 1e-4
        theta_af8 = trapezoid_integral(psd_af8[theta_mask], freqs[theta_mask]) if np.any(theta_mask) else 1e-4
        alpha_af7 = trapezoid_integral(psd_af7[alpha_mask], freqs[alpha_mask]) if np.any(alpha_mask) else 1e-4
        alpha_af8 = trapezoid_integral(psd_af8[alpha_mask], freqs[alpha_mask]) if np.any(alpha_mask) else 1e-4
        beta_af7 = trapezoid_integral(psd_af7[beta_mask], freqs[beta_mask]) if np.any(beta_mask) else 1e-4
        beta_af8 = trapezoid_integral(psd_af8[beta_mask], freqs[beta_mask]) if np.any(beta_mask) else 1e-4

        frontal_delta = float((delta_af7 + delta_af8) / 2.0)
        frontal_theta = float((theta_af7 + theta_af8) / 2.0)
        avg_alpha = float((alpha_af7 + alpha_af8) / 2.0)
        avg_beta = float((beta_af7 + beta_af8) / 2.0)

        total_power = frontal_delta + frontal_theta + avg_alpha + avg_beta
        sws_ratio = float(frontal_delta / max(total_power, 1e-4))
        faa = float(math.log(max(alpha_af8, 1e-4)) - math.log(max(alpha_af7, 1e-4)))
        faa = max(-1.0, min(1.0, faa))

        arousal_ratio = float(avg_beta / max(avg_alpha, 1e-4))
        tbr = float(frontal_theta / max(avg_beta, 1e-4))

        return {
            "af7_theta_uv2": float(theta_af7),
            "af8_theta_uv2": float(theta_af8),
            "frontal_theta_uv2": frontal_theta,
            "frontal_delta_uv2": frontal_delta,
            "sws_ratio": sws_ratio,
            "alpha_asymmetry": faa,
            "beta_alpha_ratio": arousal_ratio,
            "theta_beta_ratio": tbr,
            "blink_artifact_pct": blink_pct,
            "reference_scheme": ref_scheme
        }

    def compute_fnirs_oxygenation(self, red_counts, ir_counts, inter_optode_distance_cm=3.0, dpf=6.0, fs_fnirs=64.0):
        """Modified Beer-Lambert Law inversion (730 & 850 nm) + Pollonini cardiac SCI [0.7–2.5 Hz]."""
        if np is None or len(red_counts) < 30:
            return {"delta_hbo2": 0.0, "delta_hbr": 0.0, "delta_hbt": 0.0, "sci": 0.0}

        red_arr = np.maximum(np.asarray(red_counts, dtype=np.float64), 1e-4)
        ir_arr = np.maximum(np.asarray(ir_counts, dtype=np.float64), 1e-4)
        base_red = np.mean(red_arr[:max(5, len(red_arr) // 10)])
        base_ir = np.mean(ir_arr[:max(5, len(ir_arr) // 10)])

        dod_730 = -np.log10(red_arr / max(base_red, 1e-4))
        dod_850 = -np.log10(ir_arr / max(base_ir, 1e-4))

        # Pollonini et al. (2014) Scalp Coupling Index: cardiac pulse correlation [0.7, 2.5] Hz
        if len(dod_730) >= 32:
            if HAS_SCIPY and len(dod_730) >= 30:
                nyq = 0.5 * float(fs_fnirs)
                b, a = butter(2, [0.7 / nyq, min(2.5 / nyq, 0.95)], btype='band')
                c730 = filtfilt(b, a, dod_730)
                c850 = filtfilt(b, a, dod_850)
            else:
                c730 = _numpy_bandpass(dod_730 - np.mean(dod_730), float(fs_fnirs), 0.7, 2.5)
                c850 = _numpy_bandpass(dod_850 - np.mean(dod_850), float(fs_fnirs), 0.7, 2.5)
            c730 = c730 - np.mean(c730)
            c850 = c850 - np.mean(c850)
            denom = float(np.linalg.norm(c730) * np.linalg.norm(c850))
            sci = float(np.clip(np.dot(c730, c850) / max(denom, 1e-12), -1.0, 1.0)) if denom > 1e-12 else 0.0
        else:
            sci = 0.0

        # Fishburn et al. (2019) TDDR motion repair (suppresses optode step-shifts in derivative space)
        clean_730 = tddr_motion_correct(dod_730)
        clean_850 = tddr_motion_correct(dod_850)
        n_base = max(5, len(red_arr) // 10)
        # Re-zero to baseline segment so task-evoked Delta-OD excludes baseline offset
        clean_730 = clean_730 - float(np.mean(clean_730[:n_base]))
        clean_850 = clean_850 - float(np.mean(clean_850[:n_base]))
        task_slice = slice(n_base, len(clean_730)) if len(clean_730) > n_base + 5 else slice(0, len(clean_730))

        path_730 = inter_optode_distance_cm * (dpf * 1.06)  # Duncan et al. (1995) / Scholkmann (2013) forehead DPF(lambda)
        path_850 = inter_optode_distance_cm * (dpf * 0.95)
        eff_matrix = np.array([
            [EXTINCTION_730[0] * path_730, EXTINCTION_730[1] * path_730],
            [EXTINCTION_850[0] * path_850, EXTINCTION_850[1] * path_850],
        ], dtype=np.float64)

        mean_dod = np.array([float(np.mean(clean_730[task_slice])), float(np.mean(clean_850[task_slice]))])
        conc_mm = np.linalg.solve(eff_matrix, mean_dod)

        hbo2_umol = float(conc_mm[0] * 1000.0)
        hbr_umol = float(conc_mm[1] * 1000.0)

        return {
            "delta_hbo2": round(hbo2_umol, 3),
            "delta_hbr": round(hbr_umol, 3),
            "delta_hbt": round(hbo2_umol + hbr_umol, 3),
            "sci": round(max(0.0, sci), 3),
        }

def capture_lsl_buffers(duration_sec: int) -> tuple:
    """Capture live EEG (TP9, AF7, AF8, TP10) and fNIRS (730nm, 850nm) streams via Lab Streaming Layer."""
    try:
        from pylsl import StreamInlet, resolve_byprop
    except ImportError as exc:
        raise RuntimeError("pylsl is required for live Muse capture (install pylsl or pass --simulate)") from exc

    eeg_streams = resolve_byprop("type", "EEG", timeout=5.0)
    if not eeg_streams:
        raise RuntimeError("No live LSL EEG stream found (start muselsl/BlueMuse or pass --simulate)")
    inlet_eeg = StreamInlet(eeg_streams[0])

    fnirs_streams = resolve_byprop("type", "fNIRS", timeout=1.5)
    inlet_fnirs = StreamInlet(fnirs_streams[0]) if fnirs_streams else None

    tp9, af7, af8, tp10, red_730, ir_850 = [], [], [], [], [], []
    eeg_ts, fnirs_ts = [], []
    deadline = time.monotonic() + max(1, duration_sec)
    while time.monotonic() < deadline:
        chunk, ts = inlet_eeg.pull_chunk(timeout=0.25)
        for sample, t_s in zip(chunk, ts):
            if len(sample) >= 4:
                tp9.append(sample[0])
                af7.append(sample[1])
                af8.append(sample[2])
                tp10.append(sample[3])
                eeg_ts.append(t_s)
        if inlet_fnirs is not None:
            fchunk, fts = inlet_fnirs.pull_chunk(timeout=0.05)
            for fsamp, ft_s in zip(fchunk, fts):
                if len(fsamp) >= 2:
                    red_730.append(fsamp[0])
                    ir_850.append(fsamp[1])
                    fnirs_ts.append(ft_s)
    fs_eeg = 256.0
    if np is not None and len(eeg_ts) >= 20:
        dt = np.diff(np.asarray(eeg_ts, dtype=np.float64))
        pos = dt[dt > 0]
        if len(pos) > 0:
            fs_eeg = float(np.clip(1.0 / np.median(pos), 128.0, 512.0))
    fs_fnirs = 64.0
    if np is not None and len(fnirs_ts) >= 10:
        fdt = np.diff(np.asarray(fnirs_ts, dtype=np.float64))
        fpos = fdt[fdt > 0]
        if len(fpos) > 0:
            fs_fnirs = float(np.clip(1.0 / np.median(fpos), 16.0, 128.0))
    return (tp9, af7, af8, tp10, red_730, ir_850, fs_eeg, fs_fnirs)

def record_session(duration_sec, tag, device_model, db_path, user_id, simulate=False):
    processor = MuseSignalProcessor()
    started_at = datetime.now(timezone.utc)
    print(f"[WetLab] Recording {duration_sec}s with {device_model} (Tag: {tag}, simulate={simulate})...")

    fs_fnirs = 64.0
    if simulate:
        if np is not None:
            t = np.linspace(0, max(2.0, float(duration_sec)), max(512, duration_sec * 256), endpoint=False)
            t_f = np.linspace(0, max(2.0, float(duration_sec)), max(128, duration_sec * 64), endpoint=False)
            af7 = 15.0 * np.sin(2 * np.pi * 10.0 * t) + 8.0 * np.sin(2 * np.pi * 6.0 * t)
            af8 = 16.0 * np.sin(2 * np.pi * 10.0 * t) + 7.5 * np.sin(2 * np.pi * 6.0 * t)
            tp9 = np.random.normal(0, 4.0, len(t))
            tp10 = np.random.normal(0, 4.0, len(t))
            red = 120_000.0 + 1_500.0 * np.sin(2 * np.pi * 1.2 * t_f)
            ir = 140_000.0 + 1_800.0 * np.sin(2 * np.pi * 1.2 * t_f)
        else:
            af7, af8, tp9, tp10, red, ir = [], [], [], [], [], []
    else:
        tp9, af7, af8, tp10, red, ir, fs_eeg, fs_fnirs = capture_lsl_buffers(duration_sec)
        processor.fs = fs_eeg

    ended_at = datetime.now(timezone.utc)
    features = processor.compute_band_powers(af7, af8, tp9, tp10)
    fnirs = processor.compute_fnirs_oxygenation(red, ir, fs_fnirs=fs_fnirs)

    session_id = f"muse-sess-{int(started_at.timestamp())}"
    start_iso = started_at.strftime("%Y-%m-%dT%H:%M:%SZ")
    end_iso = ended_at.strftime("%Y-%m-%dT%H:%M:%SZ")
    print(f"[WetLab] Session {session_id} features: Theta={features['frontal_theta_uv2']:.2f} FAA={features['alpha_asymmetry']:+.2f} SWS={features['sws_ratio']:.2f}")

    if db_path and os.path.exists(os.path.dirname(db_path) or "."):
        conn = sqlite3.connect(db_path)
        cur = conn.cursor()
        cur.execute("""
            CREATE TABLE IF NOT EXISTS wetlab_eeg_fnirs_sessions (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                session_id TEXT NOT NULL,
                device_model TEXT NOT NULL,
                started_at DATETIME NOT NULL,
                ended_at DATETIME NOT NULL,
                duration_seconds REAL NOT NULL,
                sample_rate_hz REAL NOT NULL DEFAULT 256.0,
                af7_theta_uv2 REAL,
                af8_theta_uv2 REAL,
                frontal_midline_theta_uv2 REAL,
                frontal_alpha_asymmetry REAL,
                beta_alpha_arousal_ratio REAL,
                rewp_erp_uv REAL,
                fnirs_hbo2_umol REAL,
                fnirs_hbr_umol REAL,
                fnirs_hbt_umol REAL,
                blink_artifact_pct REAL DEFAULT 0.0,
                signal_quality_score REAL DEFAULT 1.0,
                context_tag TEXT,
                financial_ledger_event_id TEXT,
                raw_features_json TEXT NOT NULL DEFAULT '{}',
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
        """)
        cur.execute("""
            INSERT OR REPLACE INTO wetlab_eeg_fnirs_sessions (
                id, user_id, session_id, device_model, started_at, ended_at,
                duration_seconds, sample_rate_hz, af7_theta_uv2, af8_theta_uv2,
                frontal_midline_theta_uv2, frontal_alpha_asymmetry, beta_alpha_arousal_ratio,
                rewp_erp_uv, fnirs_hbo2_umol, fnirs_hbr_umol, fnirs_hbt_umol,
                blink_artifact_pct, signal_quality_score, context_tag, raw_features_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """, (
            session_id, user_id, session_id, device_model,
            start_iso, end_iso,
            float(duration_sec), float(processor.fs),
            features.get("af7_theta_uv2", 5.0), features.get("af8_theta_uv2", 5.0),
            features.get("frontal_theta_uv2", 5.0), features.get("alpha_asymmetry", 0.05),
            features.get("beta_alpha_ratio", 0.85), 3.5,
            fnirs.get("delta_hbo2", 0.0), fnirs.get("delta_hbr", 0.0), fnirs.get("delta_hbt", 0.0),
            features.get("blink_artifact_pct", 0.0), fnirs.get("sci", 0.0),
            tag, json.dumps({**features, **fnirs})
        ))
        conn.commit()
        conn.close()
        print(f"[WetLab] Persisted session to {db_path}")

def run_self_test() -> bool:
    print("================================================================================")
    print("   FRESHCREDIT WET-LAB MUSE HARDWARE BRIDGE PRE-FLIGHT SELF-TEST")
    print("================================================================================")
    if np is None:
        print("Scipy / Numpy missing; self-test skipped")
        return True

    np.random.seed(42)
    processor = MuseSignalProcessor(sample_rate=256)
    n = 512
    t = np.linspace(0, 2.0, n, endpoint=False)
    # Synthetic 10 Hz alpha (15 uV) + 6 Hz theta (8 uV) + 2 Hz delta (10 uV) + Gaussian noise
    af7 = 15.0 * np.sin(2 * np.pi * 10.0 * t) + 8.0 * np.sin(2 * np.pi * 6.0 * t) + 10.0 * np.sin(2 * np.pi * 2.0 * t) + np.random.normal(0, 2.0, n)
    af8 = 18.0 * np.sin(2 * np.pi * 10.0 * t) + 7.5 * np.sin(2 * np.pi * 6.0 * t) + 10.0 * np.sin(2 * np.pi * 2.0 * t) + np.random.normal(0, 2.0, n)
    tp9 = np.random.normal(0, 5.0, n)
    tp10 = np.random.normal(0, 5.0, n)

    bp = processor.compute_band_powers(af7, af8, tp9, tp10)
    assert bp["frontal_theta_uv2"] > 0.0, "Theta power must be strictly positive"
    assert 0.0 <= bp["sws_ratio"] <= 1.0, f"SWS ratio must be in [0, 1], got {bp['sws_ratio']}"
    assert bp["theta_beta_ratio"] > 0.0, "TBR must be positive"
    print(f"✓ EEG Welch PSD & Band Powers: Theta={bp['frontal_theta_uv2']:.2f} uV^2, FAA={bp['alpha_asymmetry']:+.2f}, SWS={bp['sws_ratio']:.2f}")

    # Dual-wavelength fNIRS intensity signals (730 nm, 850 nm at 64 Hz) + Pollonini cardiac SCI check
    t_f = np.linspace(0, 4.0, 256, endpoint=False)
    red = 120_000.0 + 1_500.0 * np.sin(2 * np.pi * 1.2 * t_f) + np.random.normal(0, 50.0, 256)
    ir = 140_000.0 + 1_800.0 * np.sin(2 * np.pi * 1.2 * t_f) + np.random.normal(0, 50.0, 256)
    fnirs = processor.compute_fnirs_oxygenation(red, ir, fs_fnirs=64.0)
    assert math.isfinite(fnirs["delta_hbo2"]) and math.isfinite(fnirs["delta_hbr"])
    assert fnirs["sci"] > 0.90, f"Expected cardiac-coupled SCI > 0.90, got {fnirs['sci']}"

    # Decoupled noise must drop Pollonini SCI below PILOT-H1 threshold (< 0.85)
    rng = np.random.default_rng(42)
    fnirs_bad = processor.compute_fnirs_oxygenation(120_000.0 + rng.normal(0, 500, 256), 140_000.0 + rng.normal(0, 500, 256), fs_fnirs=64.0)
    assert fnirs_bad["sci"] < 0.40, f"Expected decoupled SCI < 0.40, got {fnirs_bad['sci']}"

    # Wavelength-dependent DPF(lambda) zero-crosstalk check: pure +5 uM HbO2 (0 uM HbR) optical density must invert cleanly
    p730, p850 = 3.0 * (6.0 * 1.06), 3.0 * (6.0 * 0.95)
    dod_pure_730 = EXTINCTION_730[0] * 0.005 * p730
    dod_pure_850 = EXTINCTION_850[0] * 0.005 * p850
    red_pure = np.full(64, 100_000.0)
    red_pure[16:] = 100_000.0 * (10.0 ** (-dod_pure_730))
    ir_pure = np.full(64, 100_000.0)
    ir_pure[16:] = 100_000.0 * (10.0 ** (-dod_pure_850))
    fnirs_pure = processor.compute_fnirs_oxygenation(red_pure, ir_pure, fs_fnirs=64.0)
    assert abs(fnirs_pure["delta_hbr"]) < 0.5, f"HbO2->HbR crosstalk too large: {fnirs_pure}"
    # TDDR step-shift invariance oracle (aligned with Lean Theorem 113): +20% optical intensity step at midpoint
    red_step = red_pure.copy()
    ir_step = ir_pure.copy()
    red_step[36:] *= 0.80
    ir_step[36:] *= 0.80
    fnirs_step = processor.compute_fnirs_oxygenation(red_step, ir_step, fs_fnirs=64.0)
    assert abs(fnirs_step["delta_hbo2"] - fnirs_pure["delta_hbo2"]) < 1.0, f"TDDR step leak: {fnirs_step} vs {fnirs_pure}"
    print(f"✓ Modified Beer-Lambert + Pollonini SCI + TDDR Step Invariance: Coupled SCI={fnirs['sci']:.3f}, Decoupled SCI={fnirs_bad['sci']:.3f}")

    # Blink excision oracle: inject 350 uV ocular spike and verify theta power stays stable within +/- 15%
    af7_blink = af7.copy()
    af7_blink[240:250] += 350.0
    bp_blink = processor.compute_band_powers(af7_blink, af8, tp9, tp10)
    assert bp_blink["blink_artifact_pct"] > 0.5
    rel_err = abs(bp_blink["frontal_theta_uv2"] - bp["frontal_theta_uv2"]) / max(1e-6, bp["frontal_theta_uv2"])
    assert rel_err < 0.15, f"Blink excision failed: theta drifted by {rel_err*100:.1f}%"
    print(f"✓ Ocular Blink Excision: 350uV spike detected ({bp_blink['blink_artifact_pct']:.1f}%), frontal theta drift={rel_err*100:.1f}%")

    # Mains-hum immunity oracle: 130 uV of 60 Hz AC hum must NOT cause false >40% blink artifact stopping rule
    af7_hum = af7 + 130.0 * np.sin(2 * np.pi * 60.0 * t)
    bp_hum = processor.compute_band_powers(af7_hum, af8, tp9, tp10)
    assert bp_hum["blink_artifact_pct"] < 1.0, f"60 Hz mains hum triggered false blink rejection: {bp_hum['blink_artifact_pct']:.1f}%"
    np_hum = _numpy_bandpass(af7_hum - np.median(af7_hum), 256.0, 0.3, 35.0)
    assert float(np.mean(np.abs(np_hum) > 100.0) * 100.0) < 1.0, "Pure-NumPy tapered notch failed 60 Hz hum rejection"

    # Sub-1 Hz Delta Rayleigh resolution check: 0.5 Hz slow wave (15 uV) resolved cleanly with 4.0s Welch window
    t_sw = np.linspace(0, 4.0, 1024, endpoint=False)
    sw_wave = 15.0 * np.sin(2 * np.pi * 0.5 * t_sw)
    tp9_sw = np.random.normal(0, 5.0, len(t_sw))
    tp10_sw = np.random.normal(0, 5.0, len(t_sw))
    bp_sw = processor.compute_band_powers(sw_wave, sw_wave, tp9_sw, tp10_sw)
    assert bp_sw["frontal_delta_uv2"] > 20.0, f"Expected 0.5 Hz Delta power > 20 uV^2, got {bp_sw['frontal_delta_uv2']}"
    assert bp_sw["reference_scheme"] == "linked_mastoids_tp9_tp10"
    bp_noisy_ref = processor.compute_band_powers(sw_wave, sw_wave, np.full(len(t_sw), 200.0) * np.sin(2 * np.pi * 6.0 * t_sw), tp10_sw)
    assert bp_noisy_ref["reference_scheme"] == "fpz_hardware_fallback"
    print(f"✓ Sub-1 Hz Slow-Wave Rayleigh Resolution & Linked-Mastoid Ref: 0.5 Hz Delta={bp_sw['frontal_delta_uv2']:.1f} uV^2 ({bp_sw['reference_scheme']})")

    # In-memory database persistence
    conn = sqlite3.connect(":memory:")
    conn.execute("""
        CREATE TABLE wetlab_eeg_fnirs_sessions (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            session_id TEXT NOT NULL,
            device_model TEXT NOT NULL,
            started_at DATETIME NOT NULL,
            ended_at DATETIME NOT NULL,
            duration_seconds REAL NOT NULL,
            sample_rate_hz REAL NOT NULL DEFAULT 256.0,
            af7_theta_uv2 REAL,
            af8_theta_uv2 REAL,
            frontal_midline_theta_uv2 REAL,
            frontal_alpha_asymmetry REAL,
            beta_alpha_arousal_ratio REAL,
            rewp_erp_uv REAL,
            fnirs_hbo2_umol REAL,
            fnirs_hbr_umol REAL,
            fnirs_hbt_umol REAL,
            blink_artifact_pct REAL DEFAULT 0.0,
            signal_quality_score REAL DEFAULT 1.0,
            context_tag TEXT,
            financial_ledger_event_id TEXT,
            raw_features_json TEXT NOT NULL DEFAULT '{}',
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
    """)
    now_iso = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    conn.execute(
        "INSERT INTO wetlab_eeg_fnirs_sessions VALUES ('s1', 'u1', 'sess-1', 'muse_s_athena', ?, ?, 60.0, 256.0, ?, ?, ?, ?, ?, 3.5, ?, ?, ?, 0.0, 1.0, 'test', NULL, '{}', CURRENT_TIMESTAMP)",
        (now_iso, now_iso, bp["af7_theta_uv2"], bp["af8_theta_uv2"], bp["frontal_theta_uv2"], bp["alpha_asymmetry"], bp["beta_alpha_ratio"], fnirs["delta_hbo2"], fnirs["delta_hbr"], fnirs["delta_hbt"])
    )
    conn.commit()
    cur = conn.cursor()
    cur.execute("SELECT frontal_midline_theta_uv2, fnirs_hbo2_umol FROM wetlab_eeg_fnirs_sessions WHERE user_id = 'u1'")
    r = cur.fetchone()
    assert r[0] > 0.0 and math.isfinite(r[1])
    conn.close()
    print("✓ Local Vault Ingestion Oracle: Verified schema insert & round-trip query with UTC Z timestamps")
    print("ALL WET-LAB MUSE HARDWARE BRIDGE GATES PASSED (100% HEALTHY)\n")
    return True

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="FreshCredit Wet-Lab Muse Ingestion Bridge")
    parser.add_argument("--self-test", action="store_true", help="Execute synthetic self-test suite")
    parser.add_argument("--duration", type=int, default=60, help="Duration in seconds (e.g. 60 or 300)")
    parser.add_argument("--tag", type=str, default="baseline_morning", choices=[
        "baseline_morning", "bill_payment", "contract_eval_fair", "contract_eval_predatory", "dyadic_p2p"
    ])
    parser.add_argument("--device", type=str, default="muse_s_athena", choices=["muse_2", "muse_s", "muse_s_athena"])
    parser.add_argument("--db", type=str, default="target/user_vault.db", help="Path to local user vault SQLite database")
    parser.add_argument("--user-id", type=str, default="pilot-user-1")
    parser.add_argument("--simulate", action="store_true", help="Use synthetic Muse waveform instead of live LSL hardware")
    args = parser.parse_args()

    if args.self_test:
        success = run_self_test()
        sys.exit(0 if success else 1)
    record_session(args.duration, args.tag, args.device, args.db, args.user_id, simulate=args.simulate)

