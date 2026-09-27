//! ACE file-format v1 reader and writer.

mod header;
mod reader;
mod writer;

pub use header::*;
pub use reader::*;
pub use writer::*;

/// Returns CRC32C of a byte slice.
pub fn checksum(data: &[u8]) -> u32 {
    crc32c::crc32c(data)
}
