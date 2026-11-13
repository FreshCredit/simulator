//! Metrics schema for per-tick aggregate data
//!
//! Per Appendix B of simulation-plan.md:
//! - `metrics.parquet` contains per-tick aggregate metrics
//! - Columnar format for efficient analysis
//!
//! Supports both JSONL (for MVP compatibility) and Parquet formats.

use parquet::basic::{Compression, ZstdLevel};
use parquet::column::writer::ColumnWriter;
use parquet::file::properties::WriterProperties;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::file::writer::SerializedFileWriter;
use parquet::record::RowAccessor;
use parquet::schema::parser::parse_message_type;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::sync::Arc;

/// Per-tick metrics record
/// Per §Appendix B: metrics.parquet schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickMetrics {
    /// Simulation tick
    pub tick: u64,

    /// Number of alive agents
    pub agent_count: u64,

    /// Cooperation rate (% cooperative interactions)
    pub cooperation_rate: f64,

    /// Gini coefficient (wealth inequality)
    pub gini: f64,

    /// Trade volume (transactions this tick)
    pub trade_volume: u64,

    /// Dispute rate (disputes / transactions)
    pub dispute_rate: f64,

    /// Verification rate (verified / total, protocol ON only)
    pub verification_rate: f64,

    /// Average energy across agents
    pub avg_energy: f64,
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Average DOCAS magnitude
    pub avg_docas_magnitude: f64,

    /// Branching ratio (criticality metric)
    pub branching_ratio: f64,

    /// Number of active norms
    pub active_norms: u64,

    /// Number of institutions
    pub institution_count: u64,

    /// Protocol-specific: total anchors
    pub total_anchors: u64,

    /// Protocol-specific: total verifications
    pub total_verifications: u64,
}

impl Default for TickMetrics {
    fn default() -> Self {
        Self {
            tick: 0,
            agent_count: 0,
            cooperation_rate: 0.0,
            gini: 0.0,
            trade_volume: 0,
            dispute_rate: 0.0,
            verification_rate: 0.0,
            avg_energy: 0.0,
            avg_docas_magnitude: 0.0,
            branching_ratio: 0.0,
            active_norms: 0,
            institution_count: 0,
            total_anchors: 0,
            total_verifications: 0,
        }
    }
}

impl TickMetrics {
    /// Parquet schema definition for `TickMetrics`
    #[must_use]
    pub const fn parquet_schema() -> &'static str {
        "message tick_metrics {\n\
            REQUIRED INT64 tick;\n\
            REQUIRED INT64 agent_count;\n\
            REQUIRED DOUBLE cooperation_rate;\n\
            REQUIRED DOUBLE gini;\n\
            REQUIRED INT64 trade_volume;\n\
            REQUIRED DOUBLE dispute_rate;\n\
            REQUIRED DOUBLE verification_rate;\n\
            REQUIRED DOUBLE avg_energy;\n\
            REQUIRED DOUBLE avg_docas_magnitude;\n\
            REQUIRED DOUBLE branching_ratio;\n\
            REQUIRED INT64 active_norms;\n\
            REQUIRED INT64 institution_count;\n\
            REQUIRED INT64 total_anchors;\n\
            REQUIRED INT64 total_verifications;\n\
        }"
    }
}

/// Parquet writer for metrics with batch buffering
pub struct ParquetWriter {
    writer: SerializedFileWriter<File>,
    buffer: Vec<TickMetrics>,
    batch_size: usize,
}

impl fmt::Debug for ParquetWriter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParquetWriter")
            .field("buffer", &self.buffer)
            .field("batch_size", &self.batch_size)
            .finish_non_exhaustive()
    }
}

impl ParquetWriter {
    /// Default batch size for writing rows
    pub const DEFAULT_BATCH_SIZE: usize = 1000;

    /// Create a new Parquet writer with default batch size
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, parquet::errors::ParquetError> {
        Self::with_batch_size(path, Self::DEFAULT_BATCH_SIZE)
    }

    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// Create a new Parquet writer with custom batch size
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn with_batch_size<P: AsRef<Path>>(
        path: P,
        batch_size: usize,
    ) -> Result<Self, parquet::errors::ParquetError> {
        let file = File::create(path)?;

        let props = WriterProperties::builder()
            .set_compression(Compression::ZSTD(ZstdLevel::try_new(3).unwrap_or_default()))
            .build();

        let schema = Arc::new(parse_message_type(TickMetrics::parquet_schema())?);
        let writer = SerializedFileWriter::new(file, schema, Arc::new(props))?;

        Ok(Self {
            writer,
            buffer: Vec::with_capacity(batch_size),
            batch_size,
        })
    }

    /// Write a single tick's metrics (buffered)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn write(&mut self, metrics: TickMetrics) -> Result<(), parquet::errors::ParquetError> {
        self.buffer.push(metrics);

        if self.buffer.len() >= self.batch_size {
            self.flush()?;
        }

        Ok(())
    }

    /// Write multiple metrics at once (buffered)
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    // TAG: surface=simulation owner=platform-team rule=GENERAL-001
    pub fn write_batch(
        &mut self,
        metrics: &[TickMetrics],
    ) -> Result<(), parquet::errors::ParquetError> {
        for m in metrics {
            self.buffer.push(m.clone());

            if self.buffer.len() >= self.batch_size {
                self.flush()?;
            }
        }

        Ok(())
    }

    /// Flush the current buffer to the Parquet file
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn flush(&mut self) -> Result<(), parquet::errors::ParquetError> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        let mut row_group_writer = self.writer.next_row_group()?;
        write_simulation_metrics_row_group(&mut row_group_writer, &self.buffer)?;
        row_group_writer.close()?;
        self.buffer.clear();

        Ok(())
    }

    /// Finish writing and close the file
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn finish(mut self) -> Result<(), parquet::errors::ParquetError> {
        self.flush()?;
        self.writer.close()?;
        Ok(())
    }
}

/// Helper function to write an i64 column
fn write_i64_column(
    row_group_writer: &mut parquet::file::writer::SerializedRowGroupWriter<'_, File>,
    values: &[i64],
) -> Result<(), parquet::errors::ParquetError> {
    if let Some(mut col_writer) = row_group_writer.next_column()? {
        if let ColumnWriter::Int64ColumnWriter(ref mut typed_writer) = col_writer.untyped() {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            typed_writer.write_batch(values, None, None)?;
        }
        col_writer.close()?;
    }
    Ok(())
}

/// Helper function to write an f64 column
/// # Errors
///
/// Returns an error if the operation fails.
fn write_f64_column(
    row_group_writer: &mut parquet::file::writer::SerializedRowGroupWriter<'_, File>,
    values: &[f64],
) -> Result<(), parquet::errors::ParquetError> {
    if let Some(mut col_writer) = row_group_writer.next_column()? {
        if let ColumnWriter::DoubleColumnWriter(ref mut typed_writer) = col_writer.untyped() {
            typed_writer.write_batch(values, None, None)?;
        }
        col_writer.close()?;
    }
    Ok(())
}

/// Parquet reader for metrics with tick range filtering
pub struct ParquetReader {
    reader: SerializedFileReader<File>,
}

impl fmt::Debug for ParquetReader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParquetReader").finish_non_exhaustive()
    }
}

impl ParquetReader {
    /// Open a Parquet file for reading
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, parquet::errors::ParquetError> {
        let file = File::open(path)?;
        let reader = SerializedFileReader::new(file)?;
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        Ok(Self { reader })
    }

    /// Read all metrics into a vector
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn read_all(&self) -> Result<Vec<TickMetrics>, parquet::errors::ParquetError> {
        self.read_range(None, None)
    }

    /// Read metrics within a tick range (inclusive)
    ///
    /// # Arguments
    /// * `start_tick` - Optional starting tick (inclusive). If None, reads from beginning.
    /// * `end_tick` - Optional ending tick (inclusive). If None, reads to end.
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn read_range(
        &self,
        start_tick: Option<u64>,
        end_tick: Option<u64>,
    ) -> Result<Vec<TickMetrics>, parquet::errors::ParquetError> {
        let iter = self.iter_range(start_tick, end_tick)?;
        iter.collect()
    }

    /// Create an iterator over all metrics
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[allow(clippy::iter_not_returning_iterator)]
    pub fn iter(&self) -> Result<ParquetMetricsIterator<'_>, parquet::errors::ParquetError> {
        self.iter_range(None, None)
    }

    /// Create an iterator over metrics within a tick range (inclusive)
    ///
    /// # Arguments
    /// * `start_tick` - Optional starting tick (inclusive). If None, reads from beginning.
    /// * `end_tick` - Optional ending tick (inclusive). If None, reads to end.
    // TAG: surface=simulation owner=research-team rule=SIM-001
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn iter_range(
        &self,
        start_tick: Option<u64>,
        end_tick: Option<u64>,
    ) -> Result<ParquetMetricsIterator<'_>, parquet::errors::ParquetError> {
        let row_iter = self.reader.get_row_iter(None)?;

        Ok(ParquetMetricsIterator {
            row_iter,
            start_tick,
            end_tick,
        })
    }
}

/// Iterator over Parquet metrics with optional tick range filtering
pub struct ParquetMetricsIterator<'a> {
    row_iter: parquet::record::reader::RowIter<'a>,
    start_tick: Option<u64>,
    end_tick: Option<u64>,
}

impl fmt::Debug for ParquetMetricsIterator<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParquetMetricsIterator")
            .field("start_tick", &self.start_tick)
            .field("end_tick", &self.end_tick)
            .finish_non_exhaustive()
    }
}

impl Iterator for ParquetMetricsIterator<'_> {
    type Item = Result<TickMetrics, parquet::errors::ParquetError>;

    fn next(&mut self) -> Option<Self::Item> {
        // SIM-LOOP-001: Iterator implementation - bounded by underlying iterator
        // POT-002: Bounded by underlying row_iter exhaustion
        loop {
            match self.row_iter.next() {
                Some(Ok(row)) => {
                    match Self::row_to_metrics(&row) {
                        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
                        Ok(metrics) => {
                            // Apply tick range filtering
                            if let Some(start) = self.start_tick {
                                if metrics.tick < start {
                                    continue;
                                }
                            }
                            if let Some(end) = self.end_tick {
                                if metrics.tick > end {
                                    return None; // Past end of range
                                }
                            }
                            return Some(Ok(metrics));
                        }
                        Err(e) => return Some(Err(e)),
                    }
                }
                Some(Err(e)) => return Some(Err(e)),
                None => return None,
            }
        }
    }
}

impl ParquetMetricsIterator<'_> {
    /// Convert a Parquet Row to `TickMetrics`
    /// Column indices are based on the schema order in `TickMetrics::parquet_schema()`
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[allow(clippy::cast_sign_loss)]
    fn row_to_metrics(
        row: &parquet::record::Row,
    ) -> Result<TickMetrics, parquet::errors::ParquetError> {
        // Column indices (0-based) corresponding to schema definition:
        // 0: tick, 1: agent_count, 2: cooperation_rate, 3: gini, 4: trade_volume,
        // 5: dispute_rate, 6: verification_rate, 7: avg_energy, 8: avg_docas_magnitude,
        // 9: branching_ratio, 10: active_norms, 11: institution_count,
        // 12: total_anchors, 13: total_verifications

        Ok(TickMetrics {
            tick: row.get_long(0)? as u64,
            // TAG: surface=simulation owner=research-team rule=SIM-001
            agent_count: row.get_long(1)? as u64,
            cooperation_rate: row.get_double(2)?,
            gini: row.get_double(3)?,
            trade_volume: row.get_long(4)? as u64,
            dispute_rate: row.get_double(5)?,
            verification_rate: row.get_double(6)?,
            avg_energy: row.get_double(7)?,
            avg_docas_magnitude: row.get_double(8)?,
            branching_ratio: row.get_double(9)?,
            active_norms: row.get_long(10)? as u64,
            institution_count: row.get_long(11)? as u64,
            total_anchors: row.get_long(12)? as u64,
            total_verifications: row.get_long(13)? as u64,
        })
    }
}

/// Metrics writer for streaming output
/// Writes JSONL format for MVP (Parquet in future)
#[derive(Debug)]
pub struct MetricsWriter {
    writer: BufWriter<File>,
}

#[allow(clippy::cast_possible_wrap)]
fn write_primary_simulation_metrics(
    writer: &mut parquet::file::writer::SerializedRowGroupWriter<'_, std::fs::File>,
    buf: &[TickMetrics],
) -> Result<(), parquet::errors::ParquetError> {
    write_i64_column(
        writer,
        &buf.iter().map(|m| m.tick as i64).collect::<Vec<_>>(),
    )?;
    write_i64_column(
        writer,
        &buf.iter().map(|m| m.agent_count as i64).collect::<Vec<_>>(),
    )?;
    write_f64_column(
        writer,
        &buf.iter().map(|m| m.cooperation_rate).collect::<Vec<_>>(),
    )?;
    write_f64_column(writer, &buf.iter().map(|m| m.gini).collect::<Vec<_>>())?;
    write_i64_column(
        writer,
        &buf.iter()
            .map(|m| m.trade_volume as i64)
            .collect::<Vec<_>>(),
    )?;
    write_f64_column(
        writer,
        &buf.iter().map(|m| m.dispute_rate).collect::<Vec<_>>(),
    )?;
    write_f64_column(
        writer,
        &buf.iter().map(|m| m.verification_rate).collect::<Vec<_>>(),
    )?;
    Ok(())
}

#[allow(clippy::cast_possible_wrap)]
fn write_secondary_simulation_metrics(
    writer: &mut parquet::file::writer::SerializedRowGroupWriter<'_, std::fs::File>,
    buf: &[TickMetrics],
) -> Result<(), parquet::errors::ParquetError> {
    write_f64_column(
        writer,
        &buf.iter().map(|m| m.avg_energy).collect::<Vec<_>>(),
    )?;
    write_f64_column(
        writer,
        &buf.iter()
            .map(|m| m.avg_docas_magnitude)
            .collect::<Vec<_>>(),
    )?;
    write_f64_column(
        writer,
        &buf.iter().map(|m| m.branching_ratio).collect::<Vec<_>>(),
    )?;
    write_i64_column(
        writer,
        &buf.iter()
            .map(|m| m.active_norms as i64)
            .collect::<Vec<_>>(),
    )?;
    write_i64_column(
        writer,
        &buf.iter()
            .map(|m| m.institution_count as i64)
            .collect::<Vec<_>>(),
    )?;
    write_i64_column(
        writer,
        &buf.iter()
            .map(|m| m.total_anchors as i64)
            .collect::<Vec<_>>(),
    )?;
    write_i64_column(
        writer,
        &buf.iter()
            .map(|m| m.total_verifications as i64)
            .collect::<Vec<_>>(),
    )?;
    Ok(())
}

fn write_simulation_metrics_row_group(
    writer: &mut parquet::file::writer::SerializedRowGroupWriter<'_, std::fs::File>,
    buf: &[TickMetrics],
) -> Result<(), parquet::errors::ParquetError> {
    write_primary_simulation_metrics(writer, buf)?;
    write_secondary_simulation_metrics(writer, buf)?;
    Ok(())
}

impl MetricsWriter {
    /// Create a new metrics writer
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            writer: BufWriter::new(file),
        })
    }

    /// Write a single tick's metrics
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn write(&mut self, metrics: &TickMetrics) -> std::io::Result<()> {
        let json = serde_json::to_string(metrics)?;
        // TAG: surface=simulation owner=research-team rule=SIM-001
        writeln!(self.writer, "{json}")?;
        Ok(())
    }

    /// Flush and close the writer
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn finish(mut self) -> std::io::Result<()> {
        self.writer.flush()
    }

    /// Write metrics to a Parquet file
    ///
    /// This is a convenience method for writing metrics to Parquet format.
    /// For more control over batch sizes and compression, use `ParquetWriter` directly.
    ///
    /// # Arguments
    /// * `path` - Path to the output Parquet file
    /// * `metrics` - Slice of `TickMetrics` to write
    ///
    /// # Example
    /// ```ignore
    /// use sim_artifacts::metrics::{TickMetrics, MetricsWriter};
    ///
    /// // Write to Parquet
    /// let metrics: Vec<TickMetrics> = vec![/* ... */];
    /// MetricsWriter::write_parquet("metrics.parquet", &metrics)?;
    /// ```
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn write_parquet<P: AsRef<Path>>(
        path: P,
        metrics: &[TickMetrics],
    ) -> Result<(), parquet::errors::ParquetError> {
        let mut writer = ParquetWriter::new(path)?;
        writer.write_batch(metrics)?;
        writer.finish()
    }
}

// TAG: surface=simulation owner=research-team rule=SIM-001
/// Metrics reader for loading saved metrics (JSONL format)
#[derive(Debug)]
pub struct MetricsReader {
    reader: BufReader<File>,
}

impl MetricsReader {
    /// Open a metrics file for reading
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::open(path)?;
        Ok(Self {
            reader: BufReader::new(file),
        })
    }

    /// Read all metrics into a vector
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn read_all(&mut self) -> std::io::Result<Vec<TickMetrics>> {
        let mut metrics = Vec::new();
        for line in self.reader.by_ref().lines() {
            let line = line?;
            if !line.is_empty() {
                let tick_metrics: TickMetrics = serde_json::from_str(&line)?;
                metrics.push(tick_metrics);
            }
        }
        Ok(metrics)
    }
}

impl Iterator for MetricsReader {
    type Item = std::io::Result<TickMetrics>;

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

    fn create_test_metrics(tick: u64) -> TickMetrics {
        TickMetrics {
            tick,
            agent_count: 100u64.saturating_sub(tick),
            cooperation_rate: 0.75,
            gini: 0.35,
            trade_volume: 120,
            dispute_rate: 0.05,
            verification_rate: 0.95,
            avg_energy: 45.5,
            avg_docas_magnitude: 1.2,
            branching_ratio: 0.98,
            active_norms: 3,
            institution_count: 2,
            total_anchors: 45,
            total_verifications: 200,
        }
    }

    #[test]
    fn test_tick_metrics_default() {
        let metrics = TickMetrics::default();
        assert_eq!(metrics.tick, 0);
        // TAG: surface=simulation owner=research-team rule=SIM-001
        assert_eq!(metrics.agent_count, 0);
    }

    #[test]
    fn test_tick_metrics_serialization() {
        let metrics = create_test_metrics(100);

        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("\"tick\":100"));
        assert!(json.contains("\"cooperation_rate\":0.75"));
    }

    #[test]
    fn test_metrics_writer_reader() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("metrics.jsonl");

        // Write metrics
        {
            let mut writer = MetricsWriter::new(&path).unwrap();
            for tick in 0..10 {
                let metrics = TickMetrics {
                    tick,
                    agent_count: 100 - tick,
                    ..Default::default()
                };
                writer.write(&metrics).unwrap();
            }
            writer.finish().unwrap();
        }

        // Read metrics
        {
            let mut reader = MetricsReader::open(&path).unwrap();
            let all_metrics = reader.read_all().unwrap();
            assert_eq!(all_metrics.len(), 10);
            assert_eq!(all_metrics[0].tick, 0);
            assert_eq!(all_metrics[9].tick, 9);
        }
    }

    #[test]
    fn test_metrics_reader_iterator() {
        // TAG: surface=simulation owner=platform-team rule=GENERAL-001
        let dir = tempdir().unwrap();
        let path = dir.path().join("metrics_iter.jsonl");

        // Write metrics
        {
            let mut writer = MetricsWriter::new(&path).unwrap();
            for tick in 0..5 {
                let metrics = TickMetrics {
                    tick,
                    ..Default::default()
                };
                writer.write(&metrics).unwrap();
            }
            writer.finish().unwrap();
        }

        // Read via iterator
        let reader = MetricsReader::open(&path).unwrap();
        let ticks: Vec<u64> = reader.map(|r| r.unwrap().tick).collect();
        assert_eq!(ticks, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_parquet_writer_reader() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("metrics.parquet");

        // Write metrics to Parquet
        {
            let mut writer = ParquetWriter::new(&path).unwrap();
            for tick in 0..100 {
                writer.write(create_test_metrics(tick)).unwrap();
            }
            writer.finish().unwrap();
        }

        // Read all metrics from Parquet
        {
            let reader = ParquetReader::open(&path).unwrap();
            let all_metrics = reader.read_all().unwrap();
            assert_eq!(all_metrics.len(), 100);
            assert_eq!(all_metrics[0].tick, 0);
            // TAG: surface=simulation owner=research-team rule=SIM-001
            assert_eq!(all_metrics[99].tick, 99);
            #[allow(clippy::float_cmp)]
            {
                assert_eq!(all_metrics[50].cooperation_rate, 0.75);
            }
        }
    }

    #[test]
    fn test_parquet_batch_write() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("metrics_batch.parquet");

        // Write metrics in batch
        {
            let mut writer = ParquetWriter::new(&path).unwrap();
            let metrics: Vec<_> = (0..500).map(create_test_metrics).collect();
            writer.write_batch(&metrics).unwrap();
            writer.finish().unwrap();
        }

        // Verify
        {
            let reader = ParquetReader::open(&path).unwrap();
            let all_metrics = reader.read_all().unwrap();
            assert_eq!(all_metrics.len(), 500);
        }
    }

    #[test]
    fn test_parquet_tick_range_filtering() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("metrics_range.parquet");

        // Write metrics
        {
            let mut writer = ParquetWriter::new(&path).unwrap();
            for tick in 0..100 {
                writer.write(create_test_metrics(tick)).unwrap();
            }
            writer.finish().unwrap();
        }

        // TAG: surface=simulation owner=research-team rule=SIM-001
        // Test range filtering
        {
            let reader = ParquetReader::open(&path).unwrap();
            let range_metrics = reader.read_range(Some(10), Some(20)).unwrap();
            assert_eq!(range_metrics.len(), 11); // inclusive: 10..=20
            assert_eq!(range_metrics[0].tick, 10);
            assert_eq!(range_metrics[10].tick, 20);
        }

        // Test start only
        {
            let reader = ParquetReader::open(&path).unwrap();
            let range_metrics = reader.read_range(Some(90), None).unwrap();
            assert_eq!(range_metrics.len(), 10); // 90..=99
            assert_eq!(range_metrics[0].tick, 90);
        }

        // Test end only
        {
            let reader = ParquetReader::open(&path).unwrap();
            let range_metrics = reader.read_range(None, Some(5)).unwrap();
            assert_eq!(range_metrics.len(), 6); // 0..=5
            assert_eq!(range_metrics[0].tick, 0);
            assert_eq!(range_metrics[5].tick, 5);
        }
    }

    #[test]
    fn test_parquet_iterator() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("metrics_iter.parquet");

        // Write metrics
        {
            let mut writer = ParquetWriter::new(&path).unwrap();
            for tick in 0..50 {
                writer.write(create_test_metrics(tick)).unwrap();
            }
            writer.finish().unwrap();
        }

        // Read via iterator
        {
            // TAG: surface=simulation owner=research-team rule=SIM-001
            let reader = ParquetReader::open(&path).unwrap();
            let ticks: Vec<u64> = reader.iter().unwrap().map(|r| r.unwrap().tick).collect();
            assert_eq!(ticks.len(), 50);
            assert_eq!(ticks[0], 0);
            assert_eq!(ticks[49], 49);
        }
    }

    #[test]
    fn test_metrics_writer_write_parquet() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("metrics_via_writer.parquet");

        // Write using MetricsWriter::write_parquet
        let metrics: Vec<_> = (0..25).map(create_test_metrics).collect();
        MetricsWriter::write_parquet(&path, &metrics).unwrap();

        // Verify
        let reader = ParquetReader::open(&path).unwrap();
        let all_metrics = reader.read_all().unwrap();
        assert_eq!(all_metrics.len(), 25);
    }

    #[test]
    fn test_parquet_batch_size_flushing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("metrics_small_batch.parquet");

        // Use small batch size to test flushing behavior
        {
            let mut writer = ParquetWriter::with_batch_size(&path, 10).unwrap();
            for tick in 0..25 {
                writer.write(create_test_metrics(tick)).unwrap();
            }
            writer.finish().unwrap();
        }

        // Verify all data was written
        let reader = ParquetReader::open(&path).unwrap();
        let all_metrics = reader.read_all().unwrap();
        assert_eq!(all_metrics.len(), 25);
    }
}
