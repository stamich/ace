/// Decoder-side resource limits used before allocating memory from untrusted headers.
#[derive(Debug, Clone)]
pub struct DecodeLimits {
    /// Maximum total reconstructed output accepted by a decoder.
    pub max_output_size: u64,
    /// Maximum advertised original size of a single block.
    pub max_block_size: usize,
    /// Maximum encoded payload plus metadata size of a single block.
    pub max_encoded_block_size: usize,
    /// Maximum number of transform descriptors accepted for one block.
    pub max_transforms: usize,
}

impl Default for DecodeLimits {
    /// Returns conservative defaults suitable for ordinary CLI use.
    fn default() -> Self {
        Self {
            max_output_size: 64 * 1024 * 1024 * 1024,
            max_block_size: 4 * 1024 * 1024,
            max_encoded_block_size: 8 * 1024 * 1024,
            max_transforms: 8,
        }
    }
}
