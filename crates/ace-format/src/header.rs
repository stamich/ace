use ace_core::{AceError, AceResult, CodecId, DecodingPlan, EntropyCodecId, TransformId};

/// Four-byte ACE format-v1 magic.
pub const MAGIC: [u8; 4] = *b"ACE1";
/// Current major format version.
pub const FORMAT_MAJOR: u8 = 1;
/// Current minor format version.
pub const FORMAT_MINOR: u8 = 0;
/// Serialized file-header size in bytes.
pub const FILE_HEADER_SIZE: usize = 32;
/// Fixed serialized block-header size before transform descriptors and metadata.
pub const BLOCK_HEADER_SIZE: usize = 32;

/// Fixed ACE file header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileHeader {
    pub flags: u16,
    pub default_block_size: u32,
    pub original_size: u64,
    pub block_count: u64,
}

/// Fixed portion of one ACE block header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockHeader {
    pub block_id: u64,
    pub original_size: u32,
    pub encoded_size: u32,
    pub metadata_size: u32,
    pub codec: CodecId,
    pub entropy: EntropyCodecId,
    pub transforms: Vec<TransformId>,
    pub flags: u8,
    pub payload_crc32c: u32,
}

/// Serializes a file header, including a CRC32C over its first 28 bytes.
pub fn encode_file_header(h: &FileHeader) -> [u8; FILE_HEADER_SIZE] {
    let mut b = [0u8; FILE_HEADER_SIZE];
    b[0..4].copy_from_slice(&MAGIC);
    b[4] = FORMAT_MAJOR;
    b[5] = FORMAT_MINOR;
    b[6..8].copy_from_slice(&h.flags.to_le_bytes());
    b[8..12].copy_from_slice(&h.default_block_size.to_le_bytes());
    b[12..20].copy_from_slice(&h.original_size.to_le_bytes());
    b[20..28].copy_from_slice(&h.block_count.to_le_bytes());
    let crc = crate::checksum(&b[..28]);
    b[28..32].copy_from_slice(&crc.to_le_bytes());
    b
}

/// Parses and validates a fixed ACE file header.
pub fn decode_file_header(b: &[u8]) -> AceResult<FileHeader> {
    if b.len() != FILE_HEADER_SIZE {
        return Err(AceError::Malformed("truncated file header"));
    }
    if b[0..4] != MAGIC {
        return Err(AceError::InvalidMagic);
    }
    if b[4] != FORMAT_MAJOR {
        return Err(AceError::UnsupportedVersion {
            major: b[4],
            minor: b[5],
        });
    }
    let stored = u32::from_le_bytes(b[28..32].try_into().unwrap());
    if crate::checksum(&b[..28]) != stored {
        return Err(AceError::Malformed("file header checksum mismatch"));
    }
    Ok(FileHeader {
        flags: u16::from_le_bytes(b[6..8].try_into().unwrap()),
        default_block_size: u32::from_le_bytes(b[8..12].try_into().unwrap()),
        original_size: u64::from_le_bytes(b[12..20].try_into().unwrap()),
        block_count: u64::from_le_bytes(b[20..28].try_into().unwrap()),
    })
}

/// Serializes a block header and transform descriptor list.
pub fn encode_block_header(h: &BlockHeader) -> Vec<u8> {
    let mut b = vec![0u8; BLOCK_HEADER_SIZE + h.transforms.len()];
    b[0..8].copy_from_slice(&h.block_id.to_le_bytes());
    b[8..12].copy_from_slice(&h.original_size.to_le_bytes());
    b[12..16].copy_from_slice(&h.encoded_size.to_le_bytes());
    b[16..20].copy_from_slice(&h.metadata_size.to_le_bytes());
    b[20] = h.codec as u8;
    b[21] = h.entropy as u8;
    b[22] = h.transforms.len() as u8;
    b[23] = h.flags;
    b[24..28].copy_from_slice(&h.payload_crc32c.to_le_bytes());
    for (i, t) in h.transforms.iter().enumerate() {
        b[BLOCK_HEADER_SIZE + i] = *t as u8;
    }
    let crc = crate::checksum(&b[..28]);
    b[28..32].copy_from_slice(&crc.to_le_bytes());
    b
}

/// Parses a fixed block header and the caller-supplied transform descriptor bytes.
pub fn decode_block_header(fixed: &[u8], transform_bytes: &[u8]) -> AceResult<BlockHeader> {
    if fixed.len() != BLOCK_HEADER_SIZE {
        return Err(AceError::Malformed("truncated block header"));
    }
    let stored = u32::from_le_bytes(fixed[28..32].try_into().unwrap());
    if crate::checksum(&fixed[..28]) != stored {
        return Err(AceError::Malformed("block header checksum mismatch"));
    }
    let count = fixed[22] as usize;
    if transform_bytes.len() != count {
        return Err(AceError::Malformed("transform descriptor count mismatch"));
    }
    let transforms = transform_bytes
        .iter()
        .map(|&b| TransformId::try_from(b))
        .collect::<AceResult<Vec<_>>>()?;
    Ok(BlockHeader {
        block_id: u64::from_le_bytes(fixed[0..8].try_into().unwrap()),
        original_size: u32::from_le_bytes(fixed[8..12].try_into().unwrap()),
        encoded_size: u32::from_le_bytes(fixed[12..16].try_into().unwrap()),
        metadata_size: u32::from_le_bytes(fixed[16..20].try_into().unwrap()),
        codec: CodecId::try_from(fixed[20])?,
        entropy: EntropyCodecId::try_from(fixed[21])?,
        transforms,
        flags: fixed[23],
        payload_crc32c: u32::from_le_bytes(fixed[24..28].try_into().unwrap()),
    })
}

impl BlockHeader {
    /// Returns the decoding plan represented by this block header.
    pub fn decoding_plan(&self) -> DecodingPlan {
        DecodingPlan {
            transforms: self.transforms.clone(),
            codec: self.codec,
            entropy: self.entropy,
        }
    }
}
