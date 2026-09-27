//! Benchmark helpers, including a planner-vs-oracle regret measurement.

use ace_analysis::{BlockAnalyzer, DefaultBlockAnalyzer};
use ace_core::{
    AceConfig, AceResult, CodecId, DecodingPlan, EntropyCodecId, LzMode, PhysicalCompressionPlan,
    TransformId,
};
use ace_planner::{evaluate_candidates, CompressionPlanner, DefaultCompressionPlanner};

/// Result of comparing the adaptive selected candidate to the smallest milestone-0.1 candidate.
#[derive(Debug, Clone)]
pub struct PlannerRegret {
    /// Planner-selected candidate.
    pub selected: PhysicalCompressionPlan,
    /// Candidate selected by exhaustive milestone-candidate evaluation.
    pub oracle: PhysicalCompressionPlan,
    /// Difference between selected and oracle estimated encoded sizes.
    pub regret_bytes: isize,
}

/// Returns every physical plan family implemented by milestone 0.1.
///
/// This list is intentionally used only for offline/benchmark work. The runtime planner prunes this
/// search space because trying every plan on every production block would defeat the purpose of ACE.
pub fn oracle_candidates() -> Vec<PhysicalCompressionPlan> {
    vec![
        candidate(Vec::new(), CodecId::Raw, EntropyCodecId::None, None, "RAW"),
        candidate(
            Vec::new(),
            CodecId::Raw,
            EntropyCodecId::Huffman,
            None,
            "Huffman",
        ),
        candidate(Vec::new(), CodecId::Rle, EntropyCodecId::None, None, "RLE"),
        candidate(
            Vec::new(),
            CodecId::Rle,
            EntropyCodecId::Huffman,
            None,
            "RLE + Huffman",
        ),
        candidate(
            vec![TransformId::DeltaByte],
            CodecId::Raw,
            EntropyCodecId::Huffman,
            None,
            "Delta + Huffman",
        ),
        candidate(
            Vec::new(),
            CodecId::Lz,
            EntropyCodecId::Huffman,
            Some(LzMode::Fast),
            "LZ FAST + Huffman",
        ),
        candidate(
            Vec::new(),
            CodecId::Lz,
            EntropyCodecId::Huffman,
            Some(LzMode::Balanced),
            "LZ BALANCED + Huffman",
        ),
    ]
}

/// Computes planner regret for one block by comparing adaptive pruning with exhaustive candidate trials.
pub fn planner_regret(block: &[u8], config: &AceConfig) -> AceResult<PlannerRegret> {
    let analyzer = DefaultBlockAnalyzer;
    let planner = DefaultCompressionPlanner;
    let profile = analyzer.analyze(block);
    let adaptive = planner.candidates(&profile, config);
    let selected = evaluate_candidates(block, &adaptive, config)?;

    let mut evaluated = Vec::new();
    for candidate in oracle_candidates() {
        evaluated.push(evaluate_candidates(
            block,
            std::slice::from_ref(&candidate),
            config,
        )?);
    }
    let oracle = evaluated
        .into_iter()
        .min_by_key(|p| p.estimated_size)
        .unwrap_or_else(PhysicalCompressionPlan::raw);
    let regret_bytes = selected.estimated_size as isize - oracle.estimated_size as isize;
    Ok(PlannerRegret {
        selected,
        oracle,
        regret_bytes,
    })
}

/// Constructs an unevaluated benchmark candidate.
fn candidate(
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
