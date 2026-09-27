use thiserror::Error;

/// Unified error type returned by ACE 0.1 components.
#[derive(Debug, Error)]
pub enum AceError {
    /// ACE file magic does not match the expected signature.
    #[error("invalid ACE magic")]
    InvalidMagic,
    /// The file format version is not supported.
    #[error("unsupported ACE format version {major}.{minor}")]
    UnsupportedVersion { major: u8, minor: u8 },
    /// A codec identifier is unknown.
    #[error("unsupported codec id {0}")]
    UnsupportedCodec(u8),
    /// An entropy codec identifier is unknown.
    #[error("unsupported entropy codec id {0}")]
    UnsupportedEntropyCodec(u8),
    /// A transform identifier is unknown.
    #[error("unsupported transform id {0}")]
    UnsupportedTransform(u8),
    /// A serialized stream is structurally invalid.
    #[error("malformed stream: {0}")]
    Malformed(&'static str),
    /// A block checksum does not match the reconstructed bytes.
    #[error("checksum mismatch for block {0}")]
    ChecksumMismatch(u64),
    /// Decoding would exceed a configured hard resource limit.
    #[error("resource limit exceeded: {0}")]
    ResourceLimitExceeded(&'static str),
    /// An LZ reference cannot be satisfied by already reconstructed bytes.
    #[error("invalid LZ reference")]
    InvalidLzReference,
    /// Huffman metadata or payload is invalid.
    #[error("invalid Huffman stream: {0}")]
    InvalidHuffman(&'static str),
    /// A requested configuration value is not valid.
    #[error("invalid configuration: {0}")]
    InvalidConfig(&'static str),
    /// Underlying I/O operation failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Convenience result type used throughout ACE.
pub type AceResult<T> = Result<T, AceError>;
