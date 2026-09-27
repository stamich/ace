use ace_core::{
    AceConfig, BlockProfile, CodecId, CompressionProfile, DecodingPlan, EntropyCodecId, LzMode,
    PhysicalCompressionPlan, TransformId,
};

/// Generates candidate physical plans from cheap block statistics.
pub trait CompressionPlanner {
    /// Returns a small pruned list of candidate plans. RAW is always present.
    fn candidates(
        &self,
        profile: &BlockProfile,
        config: &AceConfig,
    ) -> Vec<PhysicalCompressionPlan>;
}

/// Rule-based milestone-0.1 planner.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultCompressionPlanner;

impl CompressionPlanner for DefaultCompressionPlanner {
    fn candidates(&self, p: &BlockProfile, c: &AceConfig) -> Vec<PhysicalCompressionPlan> {
        let raw = PhysicalCompressionPlan::raw();
        if c.enable_early_raw
            && p.entropy_h0 > 7.95
            && p.run_score < 0.01
            && p.delta_score < 0.01
            && p.repetition_score < 0.01
        {
            return vec![raw];
        }
        let mut v = vec![raw];
        // Huffman-only is useful for skewed alphabets even without obvious LZ matches.
        if p.entropy_h0 < 7.7 {
            v.push(plan(
                Vec::new(),
                CodecId::Raw,
                EntropyCodecId::Huffman,
                None,
                "skewed byte distribution",
            ));
        }
        if p.run_score >= 0.10 || p.zero_ratio >= 0.20 {
            v.push(plan(
                Vec::new(),
                CodecId::Rle,
                EntropyCodecId::None,
                None,
                "repeated-byte runs",
            ));
            v.push(plan(
                Vec::new(),
                CodecId::Rle,
                EntropyCodecId::Huffman,
                None,
                "runs plus skewed packet stream",
            ));
        }
        if p.delta_score >= 0.08 {
            v.push(plan(
                vec![TransformId::DeltaByte],
                CodecId::Raw,
                EntropyCodecId::Huffman,
                None,
                "lower delta entropy",
            ));
        }
        if p.repetition_score >= 0.03 || matches!(c.profile, CompressionProfile::Dense) {
            v.push(plan(
                Vec::new(),
                CodecId::Lz,
                EntropyCodecId::Huffman,
                Some(LzMode::Fast),
                "sampled repeated sequences",
            ));
            if !matches!(c.profile, CompressionProfile::Fast) {
                v.push(plan(
                    Vec::new(),
                    CodecId::Lz,
                    EntropyCodecId::Huffman,
                    Some(LzMode::Balanced),
                    "deeper LZ search",
                ));
            }
        }
        v
    }
}

/// Creates an unevaluated physical plan candidate.
fn plan(
    transforms: Vec<TransformId>,
    codec: CodecId,
    entropy: EntropyCodecId,
    lz_mode: Option<LzMode>,
    reason: &'static str,
) -> PhysicalCompressionPlan {
    PhysicalCompressionPlan {
        decoding: DecodingPlan {
            transforms,
            codec,
            entropy,
        },
        lz_mode,
        estimated_size: 0,
        estimated_encode_cost: 0.0,
        score: f32::INFINITY,
        reason,
    }
}
