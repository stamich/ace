/// Byte-oriented primary codec used by a physical compression plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CodecId {
    /// The input is stored verbatim.
    Raw = 0,
    /// Packet run-length encoding.
    Rle = 1,
    /// LZ77-style literal/match token encoding.
    Lz = 2,
}

impl TryFrom<u8> for CodecId {
    type Error = crate::AceError;

    /// Converts a serialized codec identifier into a supported codec.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Raw),
            1 => Ok(Self::Rle),
            2 => Ok(Self::Lz),
            other => Err(crate::AceError::UnsupportedCodec(other)),
        }
    }
}

/// Optional entropy coder applied after the primary codec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum EntropyCodecId {
    /// No entropy coding is applied.
    None = 0,
    /// Canonical Huffman coding.
    Huffman = 1,
}

impl TryFrom<u8> for EntropyCodecId {
    type Error = crate::AceError;

    /// Converts a serialized entropy identifier into a supported entropy codec.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Huffman),
            other => Err(crate::AceError::UnsupportedEntropyCodec(other)),
        }
    }
}

/// Reversible byte transformation applied before the primary codec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TransformId {
    /// No transform.
    None = 0,
    /// Byte-wise delta transform using wrapping subtraction.
    DeltaByte = 1,
}

impl TryFrom<u8> for TransformId {
    type Error = crate::AceError;

    /// Converts a serialized transform identifier into a supported transform.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::DeltaByte),
            other => Err(crate::AceError::UnsupportedTransform(other)),
        }
    }
}

/// Encoder-side LZ search strategy. Both variants produce the same wire format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LzMode {
    /// Checks only the most recent hashed candidate.
    Fast,
    /// Walks a bounded previous-position chain to find a longer match.
    Balanced,
}
