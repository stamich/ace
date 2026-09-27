use ace_engine::AceEngine;

/// Ensures heterogeneous input survives a complete ACE encode/decode cycle.
#[test]
fn heterogeneous_roundtrip() {
    let mut d = vec![0u8; 200_000];
    d.extend((0u8..=255).cycle().take(300_000));
    for _ in 0..5000 {
        d.extend_from_slice(b"graphnet/ace/adaptive-db repeated record\n");
    }
    let e = AceEngine::default_engine();
    let encoded = e.compress(&d).unwrap();
    let restored = e.decompress(&encoded).unwrap();
    assert_eq!(restored, d);
}

/// Ensures the encoder is deterministic for identical data and configuration.
#[test]
fn deterministic_output() {
    let d = b"deterministic deterministic deterministic".repeat(5000);
    let e = AceEngine::default_engine();
    assert_eq!(e.compress(&d).unwrap(), e.compress(&d).unwrap());
}
