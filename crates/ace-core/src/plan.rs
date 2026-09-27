use crate::{CodecId, CompressionProfile, DictionaryRef, EntropyCodecId, LzMode, TransformId};

/// Decoder-relevant physical plan for one ACE block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodingPlan {
    /// Ordered forward transforms applied before the primary codec.
    pub transforms: Vec<TransformId>,
    /// Primary byte-oriented codec.
    pub codec: CodecId,
    /// Optional shared dictionary reference.
    pub dictionary: Option<DictionaryRef>,
    /// Optional final entropy coder.
    pub entropy: EntropyCodecId,
}

/// Deterministic multidimensional estimate used by the runtime planner.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlanCost {
    /// Predicted total encoded bytes including codec and entropy metadata.
    pub predicted_size_bytes: u64,
    /// Deterministic relative encoder work units.
    pub encode_units: u64,
    /// Deterministic relative decoder work units.
    pub decode_units: u64,
    /// Predicted temporary memory requirement.
    pub memory_bytes: u64,
}

/// Weights that collapse a multidimensional cost into a sortable scalar score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CostWeights {
    /// Weight assigned to predicted size.
    pub size: u64,
    /// Weight assigned to encoder work.
    pub encode_cpu: u64,
    /// Weight assigned to decoder work.
    pub decode_cpu: u64,
    /// Weight assigned to temporary memory.
    pub memory: u64,
}

impl CostWeights {
    /// Returns deterministic weights for a high-throughput encoder.
    pub fn fast() -> Self {
        Self {
            size: 25,
            encode_cpu: 45,
            decode_cpu: 25,
            memory: 5,
        }
    }
    /// Returns deterministic balanced weights.
    pub fn balanced() -> Self {
        Self {
            size: 55,
            encode_cpu: 20,
            decode_cpu: 20,
            memory: 5,
        }
    }
    /// Returns deterministic weights favoring compressed size.
    pub fn dense() -> Self {
        Self {
            size: 80,
            encode_cpu: 8,
            decode_cpu: 8,
            memory: 4,
        }
    }
    /// Maps a public compression profile to its deterministic cost weights.
    pub fn for_profile(profile: CompressionProfile) -> Self {
        match profile {
            CompressionProfile::Fast => Self::fast(),
            CompressionProfile::Balanced => Self::balanced(),
            CompressionProfile::Dense => Self::dense(),
        }
    }
}

/// Encoder-side candidate plan including ephemeral cost estimates and explanation metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalCompressionPlan {
    /// Decoder-relevant portion of the plan.
    pub decoding: DecodingPlan,
    /// Optional LZ search strategy; it is not serialized because it does not change the wire format.
    pub lz_mode: Option<LzMode>,
    /// Deterministic predicted resource cost.
    pub cost: PlanCost,
    /// Final deterministic scalar score; lower is better.
    pub score: u128,
    /// Short human-readable reason used by `ace explain` and benchmark diagnostics.
    pub reason: &'static str,
}

impl PhysicalCompressionPlan {
    /// Creates the universal RAW fallback plan.
    pub fn raw() -> Self {
        Self {
            decoding: DecodingPlan {
                transforms: Vec::new(),
                codec: CodecId::Raw,
                dictionary: None,
                entropy: EntropyCodecId::None,
            },
            lz_mode: None,
            cost: PlanCost::default(),
            score: 0,
            reason: "RAW fallback",
        }
    }
}
