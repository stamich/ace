use ace_engine::AceEngine;

/// Rejects arbitrary invalid magic without panicking.
#[test]
fn rejects_bad_magic() {
    let e = AceEngine::default_engine();
    let err = e.decompress(b"NOPE-not-an-ace-file").unwrap_err();
    assert!(format!("{err}").contains("magic") || format!("{err}").contains("failed to fill"));
}

/// Detects payload corruption through reconstructed-block CRC32C.
#[test]
fn detects_corruption() {
    let e = AceEngine::default_engine();
    let mut encoded = e.compress(&vec![7u8; 100_000]).unwrap();
    let n = encoded.len();
    encoded[n - 1] ^= 0x55;
    assert!(e.decompress(&encoded).is_err());
}
