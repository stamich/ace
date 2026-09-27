use std::io::{Read, Seek};
use std::ops::Range;
use ace_core::{AceError, AceResult, DecodeLimits};
use ace_index::AceIndexReader;
use crate::decode_encoded_block;

/// High-level random-access decoder backed by the serialized ACE 1.1 block index.
pub struct AceIndexedDecoder<R: Read + Seek> {
    indexed: AceIndexReader<R>,
    limits: DecodeLimits,
}

impl<R: Read + Seek> AceIndexedDecoder<R> {
    /// Opens and validates an indexed ACE stream.
    pub fn open(reader: R, limits: DecodeLimits) -> AceResult<Self> {
        let indexed = AceIndexReader::open(reader, limits.clone())?;
        Ok(Self { indexed, limits })
    }

    /// Decodes exactly one independent block by stable block identifier.
    pub fn decode_block(&mut self, block_id: u64) -> AceResult<Vec<u8>> {
        let (header, metadata, payload) = self.indexed.read_encoded_block(block_id)?;
        decode_encoded_block(&header, &metadata, &payload, &self.limits)
    }

    /// Decodes only blocks intersecting the requested logical byte range and trims boundary blocks.
    pub fn read_range(&mut self, range: Range<u64>) -> AceResult<Vec<u8>> {
        if range.start > range.end || range.end > self.indexed.file_header.original_size { return Err(AceError::Malformed("requested range lies outside reconstructed file")); }
        if range.start == range.end { return Ok(Vec::new()); }
        let entries = self.indexed.index.intersecting(range.start, range.end).into_iter().cloned().collect::<Vec<_>>();
        let requested = range.end - range.start;
        if requested > usize::MAX as u64 { return Err(AceError::ResourceLimitExceeded("range output size")); }
        let mut out = Vec::with_capacity(requested as usize);
        for entry in entries {
            let block = self.decode_block(entry.block_id)?;
            let block_start = entry.original_offset;
            let block_end = block_start + entry.original_size as u64;
            let copy_start = range.start.max(block_start) - block_start;
            let copy_end = range.end.min(block_end) - block_start;
            out.extend_from_slice(&block[copy_start as usize..copy_end as usize]);
        }
        Ok(out)
    }

    /// Returns the number of indexed independent blocks.
    pub fn block_count(&self) -> usize { self.indexed.index.entries.len() }
}
