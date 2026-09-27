/// Fixed-size block splitter used by milestone 0.1.
#[derive(Debug, Clone, Copy)]
pub struct FixedBlockChunker {
    block_size: usize,
}
impl FixedBlockChunker {
    /// Creates a chunker with a non-zero block size.
    pub fn new(block_size: usize) -> Option<Self> {
        if block_size == 0 {
            None
        } else {
            Some(Self { block_size })
        }
    }
    /// Returns an iterator over independent fixed-size source blocks.
    pub fn chunks<'a>(&self, input: &'a [u8]) -> impl Iterator<Item = &'a [u8]> {
        input.chunks(self.block_size)
    }
}
