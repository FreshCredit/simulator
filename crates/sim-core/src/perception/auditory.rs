//! Auditory Perception Module (P1-AGENT-1)
//!
//! Sound processing and speech recognition.

use serde::{Deserialize, Serialize};

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Auditory perception from the environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditoryPerception {
    /// sounds.
    pub sounds: Vec<SoundEvent>,
    /// salience.
    pub salience: f32,
    /// ambient noise.
    pub ambient_noise: f32,
    /// speech detected.
    pub speech_detected: bool,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Individual sound event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundEvent {
    /// id.
    pub id: u64,
    /// source type.
    pub source_type: SoundSourceType,
    /// direction.
    pub direction: f32, // Angle in degrees (0 = front)
    /// distance.
    pub distance: f32,
    /// amplitude.
    pub amplitude: f32,
    /// frequency.
    pub frequency: f32,
    /// duration ms.
    pub duration_ms: u32,
    /// content.
    pub content: Option<SpeechContent>,
}

/// Type of sound source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoundSourceType {
    /// speech.
    Speech,
    /// alert.
    Alert,
    /// music.
    Music,
    /// ambient.
    Ambient,
    /// threat.
    Threat,
    /// unknown.
    Unknown,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Speech content extracted from sound
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechContent {
    /// text.
    pub text: String,
    /// language.
    pub language: String,
    /// confidence.
    pub confidence: f32,
    /// speaker id.
    pub speaker_id: Option<u64>,
    /// emotional tone.
    pub emotional_tone: EmotionalTone,
}

/// Emotional tone of speech
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmotionalTone {
    /// neutral.
    Neutral,
    /// happy.
    Happy,
    /// sad.
    Sad,
    /// angry.
    Angry,
    /// fearful.
    Fearful,
    /// surprised.
    Surprised,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Audio signal representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSignal {
    /// sample rate.
    pub sample_rate: u32,
    /// samples.
    pub samples: Vec<f32>,
    /// channels.
    pub channels: u8,
    /// duration ms.
    pub duration_ms: u32,
}

impl AudioSignal {
    /// Create silent signal
    #[must_use]
    pub fn silence(sample_rate: u32, duration_ms: u32) -> Self {
        let num_samples = (sample_rate as f32 * duration_ms as f32 / 1000.0) as usize;
        Self {
            sample_rate,
            samples: vec![0.0; num_samples],
            channels: 1,
            duration_ms,
        }
    }

    /// Calculate RMS amplitude
    #[must_use]
    pub fn rms_amplitude(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum_sq: f32 = self.samples.iter().map(|&s| s * s).sum();
        (sum_sq / self.samples.len() as f32).sqrt()
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Detect peaks in signal
    ///
    /// Returns indices of local maxima that exceed the threshold.
    /// Handles both sharp peaks and plateaus (flat maxima).
    /// Requires at least 3 samples to detect peaks (returns empty vec otherwise).
    #[must_use]
    pub fn detect_peaks(&self, threshold: f32) -> Vec<usize> {
        let mut peaks = Vec::new();
        // Need at least 3 samples to have a peak with neighbors
        if self.samples.len() < 3 {
            return peaks;
        }

        let samples = &self.samples;
        let mut i = 1;
        while i < samples.len() - 1 {
            // Check if current point exceeds threshold and is at least as high as neighbors
            if samples[i] > threshold && samples[i] >= samples[i - 1] {
                // Find extent of plateau if present
                let mut j = i;
                while j < samples.len() - 1 && samples[j] == samples[j + 1] {
                    j += 1;
                }
                // If plateau ends with a decline (and we haven't hit the array end), it's a peak
                if j < samples.len() - 1 && samples[j] > samples[j + 1] {
                    // Use center of plateau for symmetric plateaus
                    peaks.push((i + j) / 2);
                }
                i = j + 1;
            } else {
                i += 1;
            }
        }
        peaks
    }

    /// Apply simple low-pass filter
    pub fn low_pass_filter(&mut self, cutoff_ratio: f32) {
        let alpha = cutoff_ratio;
        let mut prev = 0.0;
        for sample in &mut self.samples {
            *sample = alpha * *sample + (1.0 - alpha) * prev;
            prev = *sample;
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Compute spectrogram (simplified)
    #[must_use]
    pub fn spectrogram(&self, window_size: usize) -> Vec<Vec<f32>> {
        let mut spec = Vec::new();
        for chunk in self.samples.chunks(window_size) {
            if chunk.len() == window_size {
                let fft = self.simple_fft(chunk);
                spec.push(fft);
            }
        }
        spec
    }

    /// Simple FFT placeholder
    fn simple_fft(&self, _samples: &[f32]) -> Vec<f32> {
        // Placeholder - would use real FFT library
        vec![0.0; _samples.len() / 2]
    }
}

/// Auditory processor for agents
#[derive(Debug)]
pub struct AuditoryProcessor {
    /// config.
    pub config: AuditoryConfig,
    /// recent sounds.
    pub recent_sounds: Vec<SoundEvent>,
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Auditory processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditoryConfig {
    /// hearing range.
    pub hearing_range: f32,
    /// frequency range.
    pub frequency_range: (f32, f32), // Hz
    /// direction resolution.
    pub direction_resolution: f32, // degrees
    /// sound memory duration.
    pub sound_memory_duration: u32, // ticks
}

impl Default for AuditoryConfig {
    fn default() -> Self {
        Self {
            hearing_range: 100.0,
            frequency_range: (20.0, 20000.0),
            direction_resolution: 15.0,
            sound_memory_duration: 100,
        }
    }
}

impl AuditoryProcessor {
    /// Create new auditory processor
    #[must_use]
    pub const fn new(config: AuditoryConfig) -> Self {
        Self {
            config,
            recent_sounds: Vec::new(),
        }
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Process audio signal into perception
    pub fn process(&mut self, signal: &AudioSignal) -> AuditoryPerception {
        let amplitude = signal.rms_amplitude();
        let sounds = self.extract_sounds(signal);
        let speech_detected = sounds
            .iter()
            .any(|s| matches!(s.source_type, SoundSourceType::Speech));

        // Calculate salience based on sound characteristics
        let salience = if sounds.is_empty() {
            0.0
        } else {
            sounds.iter().map(|s| s.amplitude).sum::<f32>() / sounds.len() as f32
        };

        // Update recent sounds
        self.recent_sounds.extend(sounds.clone());
        self.recent_sounds
            .retain(|s| s.duration_ms < self.config.sound_memory_duration);

        AuditoryPerception {
            sounds,
            salience,
            ambient_noise: amplitude * 0.1,
            speech_detected,
        }
    }

    /// Extract individual sounds from signal
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    fn extract_sounds(&self, signal: &AudioSignal) -> Vec<SoundEvent> {
        let mut sounds = Vec::new();
        let threshold = 0.1;

        // Simple peak detection for sound events
        let peaks = signal.detect_peaks(threshold);

        for (i, &peak_idx) in peaks.iter().enumerate() {
            let _time_ms = (peak_idx as f32 / signal.sample_rate as f32 * 1000.0) as u32;

            sounds.push(SoundEvent {
                id: i as u64,
                source_type: self.classify_sound(signal, peak_idx),
                direction: 0.0, // Would use stereo/multichannel for direction
                distance: self.estimate_distance(signal, peak_idx),
                amplitude: signal.samples[peak_idx],
                frequency: self.estimate_frequency(signal, peak_idx),
                duration_ms: 100, // Simplified
                content: None,
            });
        }

        sounds
    }

    /// Classify sound type
    fn classify_sound(&self, signal: &AudioSignal, peak_idx: usize) -> SoundSourceType {
        let freq = self.estimate_frequency(signal, peak_idx);

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Simple frequency-based classification
        if freq > 1000.0 && freq < 4000.0 {
            SoundSourceType::Speech
        } else if freq > 5000.0 {
            SoundSourceType::Alert
        } else if freq < 500.0 {
            SoundSourceType::Ambient
        } else {
            SoundSourceType::Unknown
        }
    }

    /// Estimate distance from amplitude
    fn estimate_distance(&self, signal: &AudioSignal, peak_idx: usize) -> f32 {
        let amplitude = signal.samples[peak_idx].abs();
        if amplitude < 0.001 {
            return self.config.hearing_range;
        }
        // Inverse square law approximation
        let distance = (1.0 / amplitude).sqrt();
        distance.min(self.config.hearing_range)
    }

    /// Estimate dominant frequency
    fn estimate_frequency(&self, signal: &AudioSignal, peak_idx: usize) -> f32 {
        // Simplified - count zero crossings in neighborhood
        let window = 100;
        let start = peak_idx.saturating_sub(window);
        let end = (peak_idx + window).min(signal.samples.len());

        // Guard against empty or invalid range
        if end <= start {
            return 0.0;
        }

        let mut crossings = 0;
        for i in start..end - 1 {
            // Use signum comparison to avoid overflow from multiplication
            let sign_curr = signal.samples[i].signum();
            let sign_next = signal.samples[i + 1].signum();
            if sign_curr != sign_next && sign_curr != 0.0 && sign_next != 0.0 {
                crossings += 1;
            }
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        let duration = (end - start) as f32 / signal.sample_rate as f32;
        if duration <= 0.0 {
            return 0.0;
        }
        crossings as f32 / (2.0 * duration)
    }

    /// Recognize speech in sound event
    #[must_use]
    pub const fn recognize_speech(&self, _sound: &SoundEvent) -> Option<SpeechContent> {
        // Placeholder - would use speech recognition model
        None
    }

    /// Localize sound source direction
    #[must_use]
    pub const fn localize_sound(&self, _left_ear: &AudioSignal, _right_ear: &AudioSignal) -> f32 {
        // Placeholder - would use interaural time/level difference
        0.0
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
impl Default for AuditoryProcessor {
    fn default() -> Self {
        Self::new(AuditoryConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_signal_silence() {
        let signal = AudioSignal::silence(44100, 1000);
        assert_eq!(signal.rms_amplitude(), 0.0);
    }

    #[test]
    fn test_detect_peaks_empty_samples() {
        let signal = AudioSignal {
            sample_rate: 44100,
            samples: vec![],
            channels: 1,
            duration_ms: 0,
        };
        let peaks = signal.detect_peaks(0.5);
        assert!(peaks.is_empty());
    }

    #[test]
    fn test_detect_peaks_small_samples() {
        // Less than 3 samples should return empty
        let signal = AudioSignal {
            sample_rate: 44100,
            samples: vec![0.5, 0.8],
            channels: 1,
            duration_ms: 10,
        };
        let peaks = signal.detect_peaks(0.5);
        assert!(peaks.is_empty());
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    #[test]
    fn test_detect_peaks_finds_peak() {
        // A clear peak in the middle
        let signal = AudioSignal {
            sample_rate: 44100,
            samples: vec![0.1, 0.3, 0.9, 0.3, 0.1],
            channels: 1,
            duration_ms: 100,
        };
        let peaks = signal.detect_peaks(0.5);
        assert_eq!(peaks, vec![2]); // Peak at index 2
    }

    #[test]
    fn test_detect_peaks_below_threshold() {
        // Peak exists but below threshold
        let signal = AudioSignal {
            sample_rate: 44100,
            samples: vec![0.1, 0.3, 0.4, 0.3, 0.1],
            channels: 1,
            duration_ms: 100,
        };
        let peaks = signal.detect_peaks(0.5);
        assert!(peaks.is_empty()); // No peaks above threshold 0.5
    }

    #[test]
    fn test_auditory_processor() {
        let mut processor = AuditoryProcessor::default();
        let signal = AudioSignal::silence(44100, 100);
        let perception = processor.process(&signal);
        assert!(!perception.speech_detected);
        assert_eq!(perception.sounds.len(), 0);
    }

    #[test]
    fn test_sound_source_classification() {
        assert_eq!(SoundSourceType::Speech, SoundSourceType::Speech);
        assert_ne!(SoundSourceType::Speech, SoundSourceType::Ambient);
    }
}
