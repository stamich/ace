use ace_core::{CodecId, CompressionProfile, CostWeights, EntropyCodecId, LzMode, PhysicalCompressionPlan, PlanCost};

/// Assigns deterministic relative work and memory costs to an encoded candidate.
#[derive(Debug, Default, Clone, Copy)]
pub struct DeterministicCostModel;

impl DeterministicCostModel {
    /// Computes a deterministic multidimensional cost using actual encoded size and static work coefficients.
    pub fn cost(&self, plan: &PhysicalCompressionPlan, input_bytes: usize, encoded_bytes: usize) -> PlanCost {
        let n = input_bytes as u64;
        let codec_encode = match (plan.decoding.codec, plan.lz_mode) {
            (CodecId::Raw, _) => 1,
            (CodecId::Rle, _) => 2,
            (CodecId::Lz, Some(LzMode::Balanced)) => 12,
            (CodecId::Lz, _) => 5,
        };
        let codec_decode = match plan.decoding.codec { CodecId::Raw => 1, CodecId::Rle => 2, CodecId::Lz => 3 };
        let entropy_encode = match plan.decoding.entropy { EntropyCodecId::None => 0, EntropyCodecId::Huffman => 6, EntropyCodecId::Rans => 5 };
        let entropy_decode = match plan.decoding.entropy { EntropyCodecId::None => 0, EntropyCodecId::Huffman => 7, EntropyCodecId::Rans => 4 };
        let transform_units = plan.decoding.transforms.len() as u64;
        PlanCost {
            predicted_size_bytes: encoded_bytes as u64,
            encode_units: n.saturating_mul(codec_encode + entropy_encode + transform_units),
            decode_units: n.saturating_mul(codec_decode + entropy_decode + transform_units),
            memory_bytes: (input_bytes.saturating_mul(3).saturating_add(encoded_bytes)) as u64,
        }
    }

    /// Converts multidimensional cost into a stable sortable scalar using the selected profile.
    pub fn score(&self, profile: CompressionProfile, cost: PlanCost, input_bytes: usize) -> u128 {
        let w = CostWeights::for_profile(profile);
        let divisor = input_bytes.max(1) as u128;
        let size = cost.predicted_size_bytes as u128 * w.size as u128 * 1_000;
        let encode = cost.encode_units as u128 * w.encode_cpu as u128 / divisor;
        let decode = cost.decode_units as u128 * w.decode_cpu as u128 / divisor;
        let memory = cost.memory_bytes as u128 * w.memory as u128 / divisor;
        size + encode + decode + memory
    }
}
