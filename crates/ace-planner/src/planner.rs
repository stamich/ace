use ace_core::{
    AceConfig, BlockProfile, CodecId, CompressionProfile, DecodingPlan, EntropyCodecId,
    LzMode, PhysicalCompressionPlan, TransformId,
};

/// Generates a small deterministic set of candidate physical plans from block statistics.
pub trait CompressionPlanner {
    /// Returns pruned candidates in stable order. RAW is always the first candidate.
    fn candidates(&self, profile: &BlockProfile, config: &AceConfig) -> Vec<PhysicalCompressionPlan>;
}

/// Rule-based ACE 0.2 candidate generator.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultCompressionPlanner;

impl CompressionPlanner for DefaultCompressionPlanner {
    /// Generates candidates using deterministic thresholds and no wall-clock observations.
    fn candidates(&self, p: &BlockProfile, c: &AceConfig) -> Vec<PhysicalCompressionPlan> {
        let raw = PhysicalCompressionPlan::raw();
        if c.enable_early_raw
            && p.entropy_h0 > 7.97
            && p.run_score < 0.005
            && p.delta_score < 0.005
            && p.repetition_score < 0.005
        {
            return vec![raw];
        }

        let mut candidates = vec![raw];
        let prefer_dense = matches!(c.profile, CompressionProfile::Dense);
        let allow_balanced_lz = !matches!(c.profile, CompressionProfile::Fast);

        if p.entropy_h0 < 7.85 {
            candidates.push(plan(Vec::new(), CodecId::Raw, EntropyCodecId::Huffman, None, "skewed byte distribution / Huffman"));
            if p.size >= 1024 {
                candidates.push(plan(Vec::new(), CodecId::Raw, EntropyCodecId::Rans, None, "skewed byte distribution / rANS"));
            }
        }

        if p.run_score >= 0.08 || p.zero_ratio >= 0.15 {
            candidates.push(plan(Vec::new(), CodecId::Rle, EntropyCodecId::None, None, "repeated-byte runs"));
            candidates.push(plan(Vec::new(), CodecId::Rle, EntropyCodecId::Huffman, None, "runs plus Huffman"));
            if p.size >= 2048 { candidates.push(plan(Vec::new(), CodecId::Rle, EntropyCodecId::Rans, None, "runs plus rANS")); }
        }

        if p.delta_score >= 0.04 {
            candidates.push(plan(vec![TransformId::DeltaByte], CodecId::Raw, EntropyCodecId::Huffman, None, "lower entropy after byte delta"));
            if p.size >= 2048 { candidates.push(plan(vec![TransformId::DeltaByte], CodecId::Raw, EntropyCodecId::Rans, None, "lower delta entropy plus rANS")); }
        }

        if p.repetition_score >= 0.02 || p.sampled_match_length >= 6.0 || prefer_dense {
            candidates.push(plan(Vec::new(), CodecId::Lz, EntropyCodecId::Huffman, Some(LzMode::Fast), "sampled repeated sequences / LZ fast"));
            if p.size >= 2048 { candidates.push(plan(Vec::new(), CodecId::Lz, EntropyCodecId::Rans, Some(LzMode::Fast), "sampled repeated sequences / LZ fast + rANS")); }
            if allow_balanced_lz {
                candidates.push(plan(Vec::new(), CodecId::Lz, EntropyCodecId::Huffman, Some(LzMode::Balanced), "deeper LZ search"));
                if p.size >= 2048 { candidates.push(plan(Vec::new(), CodecId::Lz, EntropyCodecId::Rans, Some(LzMode::Balanced), "deeper LZ search + rANS")); }
            }
        }

        candidates
    }
}

/// Creates an unevaluated candidate with stable decoder semantics and explanation text.
fn plan(
    transforms: Vec<TransformId>,
    codec: CodecId,
    entropy: EntropyCodecId,
    lz_mode: Option<LzMode>,
    reason: &'static str,
) -> PhysicalCompressionPlan {
    PhysicalCompressionPlan {
        decoding: DecodingPlan { transforms, codec, dictionary: None, entropy },
        lz_mode,
        cost: Default::default(),
        score: u128::MAX,
        reason,
    }
}
