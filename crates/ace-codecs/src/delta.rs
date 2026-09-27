/// Applies wrapping byte-delta transformation.
pub fn delta_encode(input: &[u8]) -> Vec<u8> {
    if input.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(input.len());
    out.push(input[0]);
    for pair in input.windows(2) {
        out.push(pair[1].wrapping_sub(pair[0]));
    }
    out
}

/// Reconstructs original bytes from wrapping byte deltas.
pub fn delta_decode(input: &[u8]) -> Vec<u8> {
    if input.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(input.len());
    let mut current = input[0];
    out.push(current);
    for &d in &input[1..] {
        current = current.wrapping_add(d);
        out.push(current);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exercises wrapping arithmetic at the byte boundary.
    #[test]
    fn roundtrip_wraps() {
        let d = [254, 255, 0, 1, 250, 2];
        assert_eq!(delta_decode(&delta_encode(&d)), d);
    }
}
