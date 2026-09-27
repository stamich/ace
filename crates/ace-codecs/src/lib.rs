//! Reversible transforms, primary codecs and entropy coders used by ACE 0.1.

mod delta;
mod huffman;
mod lz;
mod raw;
mod rle;

pub use delta::*;
pub use huffman::*;
pub use lz::*;
pub use raw::*;
pub use rle::*;

use ace_core::{AceResult, CodecId, EntropyCodecId, LzMode, TransformId};

/// Applies one forward transform to an input byte slice.
pub fn apply_transform(id: TransformId, input: &[u8]) -> AceResult<Vec<u8>> {
    match id {
        TransformId::None => Ok(input.to_vec()),
        TransformId::DeltaByte => Ok(delta_encode(input)),
    }
}

/// Applies one inverse transform to a byte slice.
pub fn invert_transform(id: TransformId, input: &[u8]) -> AceResult<Vec<u8>> {
    match id {
        TransformId::None => Ok(input.to_vec()),
        TransformId::DeltaByte => Ok(delta_decode(input)),
    }
}

/// Encodes bytes using the requested primary codec.
pub fn encode_codec(id: CodecId, mode: Option<LzMode>, input: &[u8]) -> AceResult<Vec<u8>> {
    match id {
        CodecId::Raw => Ok(raw_encode(input)),
        CodecId::Rle => Ok(rle_encode(input)),
        CodecId::Lz => Ok(lz_encode(input, mode.unwrap_or(LzMode::Fast))),
    }
}

/// Decodes bytes using the requested primary codec and expected output size.
pub fn decode_codec(id: CodecId, input: &[u8], expected_size: usize) -> AceResult<Vec<u8>> {
    match id {
        CodecId::Raw => raw_decode(input, expected_size),
        CodecId::Rle => rle_decode(input, expected_size),
        CodecId::Lz => lz_decode(input, expected_size),
    }
}

/// Encodes bytes using the selected entropy coder and returns `(metadata, payload)`.
pub fn encode_entropy(id: EntropyCodecId, input: &[u8]) -> AceResult<(Vec<u8>, Vec<u8>)> {
    match id {
        EntropyCodecId::None => Ok((Vec::new(), input.to_vec())),
        EntropyCodecId::Huffman => huffman_encode(input),
    }
}

/// Decodes bytes using serialized entropy metadata and the expected output byte count.
pub fn decode_entropy(
    id: EntropyCodecId,
    metadata: &[u8],
    input: &[u8],
    expected_size: usize,
) -> AceResult<Vec<u8>> {
    match id {
        EntropyCodecId::None => {
            if input.len() != expected_size {
                return Err(ace_core::AceError::Malformed("entropy-none size mismatch"));
            }
            Ok(input.to_vec())
        }
        EntropyCodecId::Huffman => huffman_decode(metadata, input, expected_size),
    }
}
