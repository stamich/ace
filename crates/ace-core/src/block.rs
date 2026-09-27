/// Stable numeric identifier of a block inside one ACE stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u64);
