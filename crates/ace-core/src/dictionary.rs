/// Stable dictionary identifier stored in ACE format metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct DictionaryId(pub u64);

/// Lifetime and reuse scope of one compression dictionary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DictionaryScope {
    /// Dictionary is useful only for one logical block.
    Block,
    /// Dictionary is shared by a logical segment of blocks.
    Segment,
    /// Dictionary belongs to the current ACE file.
    File,
    /// Dictionary is managed outside the ACE file and resolved by identifier.
    External,
}

/// Decoder-facing dictionary reference embedded into one block plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DictionaryRef {
    /// Stable identifier of the dictionary.
    pub id: DictionaryId,
    /// Scope in which the dictionary is expected to be resolved.
    pub scope: DictionaryScope,
}
