use crate::{analyze_runs, entropy_h0, histogram, repetition_score, unique_count};
use ace_core::BlockProfile;

/// Contract implemented by components that derive a planner profile from a byte block.
pub trait BlockAnalyzer {
    /// Analyzes one block without mutating the source bytes.
    fn analyze(&self, input: &[u8]) -> BlockProfile;
}

/// Default deterministic analyzer used by ACE 0.1.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultBlockAnalyzer;

impl BlockAnalyzer for DefaultBlockAnalyzer {
    fn analyze(&self, input: &[u8]) -> BlockProfile {
        let hist = histogram(input);
        let entropy = entropy_h0(&hist, input.len());
        let zeros = hist[0] as usize;
        let runs = analyze_runs(input);
        let repeat = repetition_score(input);

        let mut delta_hist = [0u32; 256];
        if let Some((&first, rest)) = input.split_first() {
            delta_hist[first as usize] += 1;
            let mut prev = first;
            for &b in rest {
                let d = b.wrapping_sub(prev);
                delta_hist[d as usize] += 1;
                prev = b;
            }
        }
        let delta_entropy = entropy_h0(&delta_hist, input.len());
        let delta_score = ((entropy - delta_entropy).max(0.0) / 8.0).clamp(0.0, 1.0);
        let run_score = if input.is_empty() {
            0.0
        } else {
            runs.repeated_bytes as f32 / input.len() as f32
        };
        let zero_ratio = if input.is_empty() {
            0.0
        } else {
            zeros as f32 / input.len() as f32
        };
        let structure = run_score.max(delta_score).max(repeat);
        let incompressibility = (0.7 * (entropy / 8.0) + 0.3 * (1.0 - structure)).clamp(0.0, 1.0);

        BlockProfile {
            size: input.len(),
            entropy_h0: entropy,
            zero_ratio,
            run_score,
            delta_score,
            repetition_score: repeat,
            unique_byte_count: unique_count(&hist),
            incompressibility_score: incompressibility,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that constant input has effectively zero H0 entropy.
    #[test]
    fn constant_data_has_low_entropy() {
        let p = DefaultBlockAnalyzer.analyze(&vec![7u8; 4096]);
        assert!(p.entropy_h0 < 0.001);
        assert!(p.run_score > 0.99);
    }

    /// Verifies that monotonic byte data benefits from byte-delta analysis.
    #[test]
    fn monotonic_bytes_have_delta_signal() {
        let data: Vec<u8> = (0..=255).cycle().take(4096).collect();
        let p = DefaultBlockAnalyzer.analyze(&data);
        assert!(p.delta_score > 0.5);
    }
}
