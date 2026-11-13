//! Replay frame schema for visualization
//!
//! Per Appendix B of simulation-plan.md:
//! - `replay/*.jsonl` contains per-agent state for visualization replay
//! - One file per sampled tick (e.g., every 100 ticks)
//! - JSONL format for streaming reads

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

/// Per-agent replay frame
/// Per §Appendix B: replay/NNNNNN.jsonl schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayFrame {
    /// Simulation tick
    pub tick: u64,

    /// Agent ID
    pub agent_id: u64,

    /// X position
    pub x: f32,

    /// Y position
    pub y: f32,

    /// Energy level
    pub energy: f32,

    /// DOCAS state [D, O, C, A, S]
    pub docas: [f32; 5],

    /// Whether agent is alive
    pub alive: bool,

    /// Whether agent has verified identity (protocol)
    pub has_verified_id: bool,

    /// Group ID (if in a group)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<u64>,

    /// Age in ticks
    pub age: u64,
}

impl ReplayFrame {
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create a new replay frame
    #[must_use]
    pub const fn new(tick: u64, agent_id: u64, x: f32, y: f32, energy: f32) -> Self {
        Self {
            tick,
            agent_id,
            x,
            y,
            energy,
            docas: [0.0; 5],
            alive: true,
            has_verified_id: false,
            group_id: None,
            age: 0,
        }
    }
}

/// Replay writer for a single tick file
#[derive(Debug)]
pub struct ReplayWriter {
    writer: BufWriter<File>,
    tick: u64,
}

impl ReplayWriter {
    /// Create a new replay writer for a specific tick
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn new<P: AsRef<Path>>(dir: P, tick: u64) -> std::io::Result<Self> {
        let path = dir.as_ref().join(format!("{tick:06}.jsonl"));
        let file = File::create(path)?;
        Ok(Self {
            writer: BufWriter::new(file),
            tick,
        })
    }

    /// Write a single agent's frame
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn write(&mut self, frame: &ReplayFrame) -> std::io::Result<()> {
        debug_assert_eq!(frame.tick, self.tick, "Frame tick must match writer tick");
        let json = serde_json::to_string(frame)?;
        writeln!(self.writer, "{json}")?;
        Ok(())
    }

    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    /// Flush and close the writer
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn finish(mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

/// Replay reader for loading saved frames
#[derive(Debug)]
pub struct ReplayReader {
    reader: BufReader<File>,
}

impl ReplayReader {
    /// Open a replay file for reading
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::open(path)?;
        Ok(Self {
            reader: BufReader::new(file),
        })
    }

    /// Open a replay file for a specific tick
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn open_tick<P: AsRef<Path>>(dir: P, tick: u64) -> std::io::Result<Self> {
        let path = dir.as_ref().join(format!("{tick:06}.jsonl"));
        Self::open(path)
    }

    /// Read all frames into a vector
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn read_all(&mut self) -> std::io::Result<Vec<ReplayFrame>> {
        // TAG: surface=simulation owner=research-team rule=SIM-001
        let mut frames = Vec::new();
        for line in self.reader.by_ref().lines() {
            let line = line?;
            if !line.is_empty() {
                let frame: ReplayFrame = serde_json::from_str(&line)?;
                frames.push(frame);
            }
        }
        Ok(frames)
    }
}

impl Iterator for ReplayReader {
    type Item = std::io::Result<ReplayFrame>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();
        // POT-002: Bounded by reader stream EOF
        loop {
            line.clear();
            match self.reader.read_line(&mut line) {
                Ok(0) => break None,
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    break Some(
                        serde_json::from_str(trimmed)
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
                    );
                }
                Err(e) => break Some(Err(e)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_replay_frame_creation() {
        let frame = ReplayFrame::new(100, 42, 50.5, 30.2, 80.0);
        assert_eq!(frame.tick, 100);
        assert_eq!(frame.agent_id, 42);
        #[allow(clippy::float_cmp)]
        {
            assert_eq!(frame.x, 50.5);
            assert_eq!(frame.y, 30.2);
            assert_eq!(frame.energy, 80.0);
        }
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert!(frame.alive);
    }

    #[test]
    fn test_replay_frame_serialization() {
        let frame = ReplayFrame {
            tick: 1000,
            agent_id: 123,
            x: 50.5,
            y: 30.2,
            energy: 80.0,
            docas: [0.5, 0.3, 0.2, 0.1, 0.4],
            alive: true,
            has_verified_id: true,
            group_id: Some(5),
            age: 500,
        };

        let json = serde_json::to_string(&frame).unwrap();
        assert!(json.contains("\"tick\":1000"));
        assert!(json.contains("\"agent_id\":123"));
        assert!(json.contains("\"docas\":[0.5,0.3,0.2,0.1,0.4]"));
    }

    #[test]
    fn test_replay_writer_reader() {
        let dir = tempdir().unwrap();
        let replay_dir = dir.path().join("replay");
        std::fs::create_dir(&replay_dir).unwrap();

        // Write frames for tick 100
        {
            let mut writer = ReplayWriter::new(&replay_dir, 100).unwrap();
            for agent_id in 0..10 {
                #[allow(clippy::cast_precision_loss)]
                let frame = ReplayFrame::new(100, agent_id, agent_id as f32, 0.0, 50.0);
                writer.write(&frame).unwrap();
            }
            writer.finish().unwrap();
        }

        // Read frames
        {
            let mut reader = ReplayReader::open_tick(&replay_dir, 100).unwrap();
            let frames = reader.read_all().unwrap();
            assert_eq!(frames.len(), 10);
            // TAG: surface=simulation owner=research-team rule=SIM-001
            assert_eq!(frames[0].agent_id, 0);
            assert_eq!(frames[9].agent_id, 9);
        }
    }

    #[test]
    fn test_replay_file_naming() {
        let dir = tempdir().unwrap();
        let replay_dir = dir.path().join("replay");
        std::fs::create_dir(&replay_dir).unwrap();

        // Write frames for tick 100
        {
            let mut writer = ReplayWriter::new(&replay_dir, 100).unwrap();
            let frame = ReplayFrame::new(100, 0, 0.0, 0.0, 50.0);
            writer.write(&frame).unwrap();
            writer.finish().unwrap();
        }

        // Check file exists with correct name
        let expected_path = replay_dir.join("000100.jsonl");
        assert!(expected_path.exists());
    }

    #[test]
    fn test_replay_reader_iterator() {
        let dir = tempdir().unwrap();
        let replay_dir = dir.path().join("replay");
        std::fs::create_dir(&replay_dir).unwrap();

        // Write frames
        {
            let mut writer = ReplayWriter::new(&replay_dir, 200).unwrap();
            for agent_id in 0..5 {
                let frame = ReplayFrame::new(200, agent_id, 0.0, 0.0, 50.0);
                writer.write(&frame).unwrap();
            }
            writer.finish().unwrap();
        }

        // Read via iterator
        let reader = ReplayReader::open_tick(&replay_dir, 200).unwrap();
        let agent_ids: Vec<u64> = reader.map(|r| r.unwrap().agent_id).collect();
        assert_eq!(agent_ids, vec![0, 1, 2, 3, 4]);
    }
}
