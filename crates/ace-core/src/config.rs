/// Planner profile that changes the relative importance of compression ratio and encoder CPU cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionProfile {
    /// Minimizes analysis and encoding CPU cost.
    Fast,
    /// Balances encoded size and CPU cost.
    Balanced,
    /// Prefers encoded size over encoder CPU cost.
    Dense,
}

/// Runtime configuration of an ACE encoder.
#[derive(Debug, Clone)]
pub struct AceConfig {
    /// Target size of each independent input block.
    pub block_size: usize,
    /// Cost profile used by the planner.
    pub profile: CompressionProfile,
    /// Minimum number of bytes a non-RAW plan must save after block metadata overhead.
    pub min_gain_bytes: usize,
    /// Maximum bytes used for trial compression by the candidate evaluator.
    pub sample_size: usize,
    /// Enables conservative early selection of RAW for likely incompressible blocks.
    pub enable_early_raw: bool,
}

impl Default for AceConfig {
    /// Returns the recommended milestone-0.1 defaults.
    fn default() -> Self {
        Self {
            block_size: 256 * 1024,
            profile: CompressionProfile::Balanced,
            min_gain_bytes: 32,
            sample_size: 8 * 1024,
            enable_early_raw: true,
        }
    }
}
