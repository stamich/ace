use ace_codecs::encode_codec;
use ace_core::{AceConfig, AceError, AceResult, EntropyCodecId, PhysicalCompressionPlan};
use ace_entropy::encode_entropy;
use ace_transforms::apply_transform;
use crate::DeterministicCostModel;

/// Deterministically executes candidate pipelines and chooses the lowest cost without wall-clock timing.
pub fn evaluate_candidates(
    input: &[u8],
    candidates: &[PhysicalCompressionPlan],
    config: &AceConfig,
) -> AceResult<PhysicalCompressionPlan> {
    evaluate_all_candidates(input, candidates, config)?.into_iter().min_by(|a, b| {
        a.score.cmp(&b.score).then_with(|| stable_plan_key(a).cmp(&stable_plan_key(b)))
    }).ok_or(AceError::Malformed("planner generated no candidates"))
}

/// Deterministically evaluates every candidate and returns populated cost/score fields in stable input order.
pub fn evaluate_all_candidates(
    input: &[u8],
    candidates: &[PhysicalCompressionPlan],
    config: &AceConfig,
) -> AceResult<Vec<PhysicalCompressionPlan>> {
    let model = DeterministicCostModel;
    let mut evaluated = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let (metadata, payload, _) = encode_plan_payload(input, candidate)?;
        let metadata_prefix = if matches!(candidate.decoding.entropy, EntropyCodecId::None) { 0 } else { 4 };
        let encoded_bytes = metadata.len().saturating_add(payload.len()).saturating_add(metadata_prefix);
        let mut plan = candidate.clone();
        plan.cost = model.cost(&plan, input.len(), encoded_bytes);
        plan.score = model.score(config.profile, plan.cost, input.len());
        evaluated.push(plan);
    }
    Ok(evaluated)
}

/// Executes only the reversible payload pipeline of one plan and returns entropy metadata, payload and primary-stream size.
pub fn encode_plan_payload(
    input: &[u8],
    plan: &PhysicalCompressionPlan,
) -> AceResult<(Vec<u8>, Vec<u8>, usize)> {
    let mut stage = input.to_vec();
    for &transform in &plan.decoding.transforms { stage = apply_transform(transform, &stage)?; }
    stage = encode_codec(plan.decoding.codec, plan.lz_mode, &stage)?;
    let primary_len = stage.len();
    let (metadata, payload) = encode_entropy(plan.decoding.entropy, &stage)?;
    Ok((metadata, payload, primary_len))
}

/// Produces a stable tie-break key that does not depend on address, thread scheduling or hash iteration order.
fn stable_plan_key(plan: &PhysicalCompressionPlan) -> (Vec<u8>, u8, u8, u8) {
    let transforms = plan.decoding.transforms.iter().map(|t| *t as u8).collect::<Vec<_>>();
    let lz = match plan.lz_mode { None => 0, Some(ace_core::LzMode::Fast) => 1, Some(ace_core::LzMode::Balanced) => 2 };
    (transforms, plan.decoding.codec as u8, plan.decoding.entropy as u8, lz)
}
