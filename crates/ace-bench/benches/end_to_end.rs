use ace_engine::AceEngine;
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

/// Generates deterministic heterogeneous data for repeatable microbenchmarks.
fn demo_data() -> Vec<u8> {
    let mut d = vec![0u8; 1024 * 1024];
    d.extend((0u8..=255).cycle().take(1024 * 1024));
    let text = b"adaptive compression engine graphnet adaptive database\n";
    for _ in 0..20_000 {
        d.extend_from_slice(text);
    }
    d
}

/// Benchmarks complete in-memory compression and decompression.
fn bench(c: &mut Criterion) {
    let d = demo_data();
    let e = AceEngine::default_engine();
    let encoded = e.compress(&d).unwrap();
    let mut g = c.benchmark_group("ace-0.1-e2e");
    g.throughput(Throughput::Bytes(d.len() as u64));
    g.bench_function("compress", |b| {
        b.iter(|| e.compress(black_box(&d)).unwrap())
    });
    g.bench_function("decompress", |b| {
        b.iter(|| e.decompress(black_box(&encoded)).unwrap())
    });
    g.finish();
}
criterion_group!(benches, bench);
criterion_main!(benches);
