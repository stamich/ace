/// Estimates repeated-sequence density by sampling 4-byte fingerprints.
///
/// The detector is intentionally cheap and bounded. It is not a correctness mechanism and does not
/// replace the real match finder used by the LZ codec.
pub fn repetition_score(input: &[u8]) -> f32 {
    const TABLE: usize = 4096;
    if input.len() < 4 {
        return 0.0;
    }
    let mut seen = [u32::MAX; TABLE];
    let mut samples = 0usize;
    let mut hits = 0usize;
    let step = 16usize;
    let mut i = 0usize;
    while i + 4 <= input.len() {
        let v = u32::from_le_bytes([input[i], input[i + 1], input[i + 2], input[i + 3]]);
        let h = ((v.wrapping_mul(0x9E37_79B1) >> 20) as usize) & (TABLE - 1);
        if seen[h] == v {
            hits += 1;
        } else {
            seen[h] = v;
        }
        samples += 1;
        i += step;
    }
    if samples == 0 {
        0.0
    } else {
        hits as f32 / samples as f32
    }
}
