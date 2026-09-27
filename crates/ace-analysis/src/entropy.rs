/// Computes zero-order Shannon entropy in bits per byte from a byte histogram.
pub fn entropy_h0(frequencies: &[u32; 256], total: usize) -> f32 {
    if total == 0 {
        return 0.0;
    }
    frequencies
        .iter()
        .copied()
        .filter(|&n| n != 0)
        .map(|n| {
            let p = n as f32 / total as f32;
            -p * p.log2()
        })
        .sum()
}

/// Builds a byte histogram for the supplied slice.
pub fn histogram(input: &[u8]) -> [u32; 256] {
    let mut out = [0u32; 256];
    for &b in input {
        out[b as usize] += 1;
    }
    out
}

/// Counts byte values with a non-zero frequency.
pub fn unique_count(histogram: &[u32; 256]) -> u16 {
    histogram.iter().filter(|&&n| n != 0).count() as u16
}
