use crate::{CodecId, EntropyCodecId, LzMode, TransformId};

/// Decoder-relevant physical plan for one ACE block.
#[derive(Debug, Clone, PartialEq)]
pub struct DecodingPlan {
    /// Ordered forward transforms that were applied before the primary codec.
    pub transforms: Vec<TransformId>,
    /// Primary byte-oriented codec.
    pub codec: CodecId,
    /// Optional final entropy coder.
    pub entropy: EntropyCodecId,
}

/// Encoder-side candidate plan including ephemeral cost estimates.
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicalCompressionPlan {
    /// Decoder-relevant portion of the plan.
    pub decoding: DecodingPlan,
    /// Optional LZ search strategy; this is not serialized because it does not change the wire format.
    pub lz_mode: Option<LzMode>,
    /// Estimated number of encoded bytes for a full block.
    pub estimated_size: usize,
    /// Relative encoding work estimate used by the cost model.
    pub estimated_encode_cost: f32,
    /// Final normalized candidate score; lower is better.
    pub score: f32,
    /// Short human-readable reason used by `ace explain`.
    pub reason: &'static str,
}

impl PhysicalCompressionPlan {
    /// Creates the universal RAW fallback plan.
    pub fn raw() -> Self {
        Self {
            decoding: DecodingPlan {
                transforms: Vec::new(),
                codec: CodecId::Raw,
                entropy: EntropyCodecId::None,
            },
            lz_mode: None,
            estimated_size: 0,
            estimated_encode_cost: 0.0,
            score: 0.0,
            reason: "RAW fallback",
        }
    }
}
