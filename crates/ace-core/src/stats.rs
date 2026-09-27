use std::time::Duration;

/// Aggregate telemetry produced by one compression operation.
#[derive(Debug, Clone, Default)]
pub struct CompressionStats {
    /// Number of source bytes.
    pub input_bytes: u64,
    /// Number of ACE bytes written.
    pub output_bytes: u64,
    /// Number of processed blocks.
    pub block_count: u64,
    /// Number of blocks stored as RAW.
    pub raw_blocks: u64,
    /// Number of blocks using RLE.
    pub rle_blocks: u64,
    /// Number of blocks using LZ.
    pub lz_blocks: u64,
    /// Number of blocks using byte-delta transform.
    pub delta_blocks: u64,
    /// Time spent collecting block statistics.
    pub analysis_time: Duration,
    /// Time spent planning and evaluating candidates.
    pub planning_time: Duration,
    /// Time spent encoding selected plans.
    pub encoding_time: Duration,
}

impl CompressionStats {
    /// Returns `input_bytes / output_bytes`, or `1.0` for an empty output.
    pub fn compression_ratio(&self) -> f64 {
        if self.output_bytes == 0 {
            1.0
        } else {
            self.input_bytes as f64 / self.output_bytes as f64
        }
    }
}
