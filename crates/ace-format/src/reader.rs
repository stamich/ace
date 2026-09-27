use crate::{
    decode_block_header, decode_file_header, BlockHeader, FileHeader, BLOCK_HEADER_SIZE,
    FILE_HEADER_SIZE,
};
use ace_core::{AceError, AceResult, DecodeLimits};
use std::io::Read;

/// Streaming ACE reader that validates structural limits before allocating block buffers.
pub struct AceReader<R: Read> {
    inner: R,
    limits: DecodeLimits,
}
impl<R: Read> AceReader<R> {
    /// Creates a reader with explicit decoder resource limits.
    pub fn new(inner: R, limits: DecodeLimits) -> Self {
        Self { inner, limits }
    }
    /// Reads and validates the fixed file header.
    pub fn read_file_header(&mut self) -> AceResult<FileHeader> {
        let mut b = [0u8; FILE_HEADER_SIZE];
        self.inner.read_exact(&mut b)?;
        let h = decode_file_header(&b)?;
        if h.original_size > self.limits.max_output_size {
            return Err(AceError::ResourceLimitExceeded("file output size"));
        }
        Ok(h)
    }
    /// Reads one block and returns `(header, entropy metadata, encoded payload)`.
    pub fn read_block(&mut self) -> AceResult<(BlockHeader, Vec<u8>, Vec<u8>)> {
        let mut fixed = [0u8; BLOCK_HEADER_SIZE];
        self.inner.read_exact(&mut fixed)?;
        let transform_count = fixed[22] as usize;
        if transform_count > self.limits.max_transforms {
            return Err(AceError::ResourceLimitExceeded("transform count"));
        }
        let mut tb = vec![0u8; transform_count];
        self.inner.read_exact(&mut tb)?;
        let h = decode_block_header(&fixed, &tb)?;
        if h.original_size as usize > self.limits.max_block_size {
            return Err(AceError::ResourceLimitExceeded("block output size"));
        }
        let total = (h.metadata_size as usize)
            .checked_add(h.encoded_size as usize)
            .ok_or(AceError::Malformed("encoded block size overflow"))?;
        if total > self.limits.max_encoded_block_size {
            return Err(AceError::ResourceLimitExceeded("encoded block size"));
        }
        let mut metadata = vec![0u8; h.metadata_size as usize];
        self.inner.read_exact(&mut metadata)?;
        let mut payload = vec![0u8; h.encoded_size as usize];
        self.inner.read_exact(&mut payload)?;
        Ok((h, metadata, payload))
    }
}
