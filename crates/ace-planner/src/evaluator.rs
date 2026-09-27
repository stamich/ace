use ace_codecs::{apply_transform, encode_codec, encode_entropy};
use ace_core::{AceConfig, AceResult, CompressionProfile, PhysicalCompressionPlan};
use std::time::Instant;

/// Trial-compresses candidates on a bounded sample and chooses the lowest normalized cost.
pub fn evaluate_candidates(
    input: &[u8],
    candidates: &[PhysicalCompressionPlan],
    config: &AceConfig,
) -> AceResult<PhysicalCompressionPlan> {
    let sample = &input[..input.len().min(config.sample_size.max(1))];
    let mut evaluated = Vec::with_capacity(candidates.len());
    let baseline_ns = 1_000f32;
    for candidate in candidates {
        let start = Instant::now();
        let (meta, payload) = encode_plan_payload(sample, candidate)?;
        let nanos = start.elapsed().as_nanos().max(1) as f32;
        let size_ratio = if sample.is_empty() {
            1.0
        } else {
            (meta.len() + payload.len()) as f32 / sample.len() as f32
        };
        let cpu_ratio = (nanos / baseline_ns).ln_1p();
        let (ws, wc) = match config.profile {
            CompressionProfile::Fast => (0.35, 0.65),
            CompressionProfile::Balanced => (0.65, 0.35),
            CompressionProfile::Dense => (0.90, 0.10),
        };
        let mut p = candidate.clone();
        p.estimated_size = ((meta.len() + payload.len()) as f64
            * (input.len().max(1) as f64 / sample.len().max(1) as f64))
            .ceil() as usize;
        p.estimated_encode_cost = cpu_ratio;
        p.score = ws * size_ratio + wc * cpu_ratio;
        evaluated.push(p);
    }
    evaluated
        .into_iter()
        .min_by(|a, b| a.score.total_cmp(&b.score))
        .ok_or(ace_core::AceError::Malformed(
            "planner generated no candidates",
        ))
}

/// Executes only the reversible payload pipeline of one plan, without ACE block framing.
pub fn encode_plan_payload(
    input: &[u8],
    plan: &PhysicalCompressionPlan,
) -> AceResult<(Vec<u8>, Vec<u8>)> {
    let mut stage = input.to_vec();
    for &t in &plan.decoding.transforms {
        stage = apply_transform(t, &stage)?;
    }
    stage = encode_codec(plan.decoding.codec, plan.lz_mode, &stage)?;
    encode_entropy(plan.decoding.entropy, &stage)
}
