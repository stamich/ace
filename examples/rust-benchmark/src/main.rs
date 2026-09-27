use std::fs;
use std::hint::black_box;
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use ace_analysis::{BlockAnalyzer, DefaultBlockAnalyzer};
use ace_core::{
    AceConfig, CodecId, CompressionProfile, DecodingPlan, EntropyCodecId, LzMode,
    PhysicalCompressionPlan, TransformId,
};
use ace_engine::{AceEngine, AceIndexedDecoder};
use ace_entropy::{huffman_encode, rans_encode};
use ace_planner::{encode_plan_payload, evaluate_candidates, CompressionPlanner, DefaultCompressionPlanner};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::Serialize;
use serde_json::{json, Value};

const RUNS: usize = 7;
const WARMUPS: usize = 3;

/// Top-level JSON benchmark document shared by every ACE 0.2 benchmark family.
#[derive(Debug, Serialize)]
struct BenchmarkDocument {
    schema_version: &'static str,
    project: &'static str,
    milestone: &'static str,
    base: &'static str,
    scope: String,
    benchmark_contract_origin: &'static str,
    generated_at_utc_epoch_seconds: u64,
    environment: Value,
    configuration: Value,
    workloads: Vec<Value>,
}

/// Summary statistics calculated from repeated benchmark samples expressed as nanoseconds.
#[derive(Debug, Clone)]
struct SampleStats {
    samples_ns: Vec<u64>,
    median_ns: f64,
    p95_ns: f64,
    p99_ns: f64,
    mean_ns: f64,
    min_ns: u64,
    max_ns: u64,
}

/// Runs the selected benchmark family and writes its result to `examples/results/0.2-<family>.json`.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let family = std::env::args().nth(1).unwrap_or_else(|| "all".to_string());
    let families = if family == "all" {
        vec!["compression", "entropy", "planner", "parallel", "random-access"]
    } else { vec![family.as_str()] };
    for family in families {
        let workloads = match family {
            "compression" => compression_family()?,
            "entropy" => entropy_family()?,
            "planner" => planner_family()?,
            "parallel" => parallel_family()?,
            "random-access" => random_access_family()?,
            other => return Err(format!("unknown benchmark family: {other}").into()),
        };
        let document = BenchmarkDocument {
            schema_version: "1.0", project: "ace", milestone: "0.2", base: "0.1.0",
            scope: family.to_string(), benchmark_contract_origin: "ace-0.2",
            generated_at_utc_epoch_seconds: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            environment: environment_json(),
            configuration: json!({"warmup_iterations": WARMUPS, "runs": RUNS, "default_block_size_bytes": 262144}),
            workloads,
        };
        let path = result_path(family);
        if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
        fs::write(&path, serde_json::to_vec_pretty(&document)?)?;
        println!("results written to {}", path.display());
    }
    Ok(())
}

/// Returns stable host/build metadata used to interpret benchmark results across machines.
fn environment_json() -> Value {
    json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "logical_cpus": std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1),
        "build_profile": if cfg!(debug_assertions) { "debug" } else { "release" }
    })
}

/// Resolves the canonical benchmark result path under the shared `examples/results` directory.
fn result_path(family: &str) -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.parent().expect("benchmark crate is inside examples").join("results").join(format!("0.2-{family}.json"))
}

/// Measures a closure after warmup and returns sorted-distribution statistics without discarding raw samples.
fn measure<F, T>(mut operation: F) -> Result<(SampleStats, T), Box<dyn std::error::Error>>
where
    F: FnMut() -> Result<T, Box<dyn std::error::Error>>,
{
    let mut last = None;
    for _ in 0..WARMUPS { last = Some(black_box(operation()?)); }
    let mut samples = Vec::with_capacity(RUNS);
    for _ in 0..RUNS {
        let start = Instant::now();
        let value = black_box(operation()?);
        samples.push(start.elapsed().as_nanos().min(u64::MAX as u128) as u64);
        last = Some(value);
    }
    Ok((summarize(samples), last.expect("at least one benchmark run")))
}

/// Computes median, p95, p99, mean, minimum and maximum from nanosecond samples.
fn summarize(samples_ns: Vec<u64>) -> SampleStats {
    let mut sorted = samples_ns.clone(); sorted.sort_unstable();
    let percentile = |p: f64| -> f64 {
        if sorted.is_empty() { return 0.0; }
        let rank = ((sorted.len() - 1) as f64 * p).ceil() as usize;
        sorted[rank.min(sorted.len() - 1)] as f64
    };
    let mean = if sorted.is_empty() { 0.0 } else { sorted.iter().map(|&v| v as f64).sum::<f64>() / sorted.len() as f64 };
    SampleStats {
        median_ns: percentile(0.50), p95_ns: percentile(0.95), p99_ns: percentile(0.99), mean_ns: mean,
        min_ns: *sorted.first().unwrap_or(&0), max_ns: *sorted.last().unwrap_or(&0), samples_ns,
    }
}

/// Converts nanosecond samples and byte count into the common JSON timing/throughput representation.
fn timing_json(stats: &SampleStats, bytes: usize) -> Value {
    let seconds = stats.median_ns / 1_000_000_000.0;
    let mb_s = if seconds == 0.0 { 0.0 } else { bytes as f64 / 1_048_576.0 / seconds };
    json!({
        "runs": RUNS, "warmup_iterations": WARMUPS,
        "median_ns": stats.median_ns, "p95_ns": stats.p95_ns, "p99_ns": stats.p99_ns,
        "mean_ns": stats.mean_ns, "min_ns": stats.min_ns, "max_ns": stats.max_ns,
        "median_mb_s": mb_s, "samples_ns": stats.samples_ns,
    })
}

/// Generates the deterministic heterogeneous dataset used by multiple benchmark families.
fn mixed_data(mebibytes: usize) -> Vec<u8> {
    let target = mebibytes * 1024 * 1024;
    let quarter = target / 4;
    let mut data = Vec::with_capacity(target);
    data.extend(std::iter::repeat(0u8).take(quarter));
    while data.len() < quarter * 2 { let v = (data.len() as u32 / 16).to_le_bytes(); data.extend_from_slice(&v); }
    while data.len() < quarter * 3 { data.extend_from_slice(b"{\"status\":\"ACTIVE\",\"service\":\"graphnet\",\"region\":\"eu\"}\n"); }
    let mut x = 0x9e37_79b9u32;
    while data.len() < target { x ^= x << 13; x ^= x >> 17; x ^= x << 5; data.push((x & 0xff) as u8); }
    data.truncate(target); data
}

/// Benchmarks end-to-end ACE against LZ4, Zstd level 3 and gzip level 6.
fn compression_family() -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let data = mixed_data(16);
    let mut results = Vec::new();
    for (name, profile) in [("ace-balanced", CompressionProfile::Balanced), ("ace-fast", CompressionProfile::Fast), ("ace-dense", CompressionProfile::Dense)] {
        let mut cfg = AceConfig::default(); cfg.profile = profile; cfg.threads = 1;
        let engine = AceEngine::new(cfg)?;
        let (enc_stats, encoded) = measure(|| Ok(engine.compress(&data)?))?;
        let (dec_stats, decoded) = measure(|| Ok(engine.decompress(&encoded)?))?;
        assert_eq!(decoded, data);
        results.push(json!({"workload_id":"mixed_16m","path":name,"input_bytes":data.len(),"compressed_bytes":encoded.len(),"compression_ratio":data.len() as f64/encoded.len() as f64,"compression":timing_json(&enc_stats,data.len()),"decompression":timing_json(&dec_stats,data.len())}));
    }
    let (stats, lz4) = measure(|| Ok(lz4_flex::compress_prepend_size(&data)))?;
    let (dec, restored) = measure(|| Ok(lz4_flex::decompress_size_prepended(&lz4)?))?; assert_eq!(restored, data);
    results.push(json!({"workload_id":"mixed_16m","path":"lz4","input_bytes":data.len(),"compressed_bytes":lz4.len(),"compression_ratio":data.len() as f64/lz4.len() as f64,"compression":timing_json(&stats,data.len()),"decompression":timing_json(&dec,data.len())}));
    let (stats, zstd_data) = measure(|| Ok(zstd::stream::encode_all(Cursor::new(&data), 3)?))?;
    let (dec, restored) = measure(|| Ok(zstd::stream::decode_all(Cursor::new(&zstd_data))?))?; assert_eq!(restored, data);
    results.push(json!({"workload_id":"mixed_16m","path":"zstd-3","input_bytes":data.len(),"compressed_bytes":zstd_data.len(),"compression_ratio":data.len() as f64/zstd_data.len() as f64,"compression":timing_json(&stats,data.len()),"decompression":timing_json(&dec,data.len())}));
    let (stats, gzip) = measure(|| { let mut e=GzEncoder::new(Vec::new(), Compression::new(6)); e.write_all(&data)?; Ok(e.finish()?) })?;
    let (gzip_dec, restored) = measure(|| { let mut d=GzDecoder::new(Cursor::new(&gzip)); let mut out=Vec::new(); d.read_to_end(&mut out)?; Ok(out) })?; assert_eq!(restored,data);
    results.push(json!({"workload_id":"mixed_16m","path":"gzip-6","input_bytes":data.len(),"compressed_bytes":gzip.len(),"compression_ratio":data.len() as f64/gzip.len() as f64,"compression":timing_json(&stats,data.len()),"decompression":timing_json(&gzip_dec,data.len())}));
    Ok(results)
}

/// Benchmarks Huffman and scalar rANS independently on representative symbol distributions.
fn entropy_family() -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let mut workloads = Vec::new();
    let datasets = vec![("skewed", b"aaaaabbbbcccdde".repeat(200_000)), ("mixed", mixed_data(4))];
    for (id, data) in datasets {
        for path in ["huffman", "rans"] {
            let (stats, (metadata, payload)) = if path == "huffman" { measure(|| Ok(huffman_encode(&data)?))? } else { measure(|| Ok(rans_encode(&data)?))? };
            workloads.push(json!({"workload_id":id,"path":path,"input_bytes":data.len(),"metadata_bytes":metadata.len(),"payload_bytes":payload.len(),"encoded_bytes":metadata.len()+payload.len(),"compression_ratio":data.len() as f64/(metadata.len()+payload.len()).max(1) as f64,"encode":timing_json(&stats,data.len())}));
        }
    }
    Ok(workloads)
}

/// Measures planner quality against an exhaustive ACE-0.2 internal-plan oracle.
fn planner_family() -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let data = mixed_data(8);
    let cfg = AceConfig::default();
    let analyzer = DefaultBlockAnalyzer; let planner = DefaultCompressionPlanner;
    let mut regret = 0i64; let mut blocks = 0u64; let mut recall = 0u64;
    let mut planner_samples = Vec::new();
    for block in data.chunks(cfg.block_size) {
        let profile = analyzer.analyze(block);
        let candidates = planner.candidates(&profile, &cfg);
        let start = Instant::now();
        let selected = evaluate_candidates(block, &candidates, &cfg)?;
        planner_samples.push(start.elapsed().as_nanos().min(u64::MAX as u128) as u64);
        let selected_size = encoded_plan_size(block, &selected)? as i64;
        let oracle = oracle_plans().into_iter().map(|plan| {
            let size = encoded_plan_size(block, &plan).unwrap_or(usize::MAX);
            (size, plan)
        }).min_by_key(|(size, _)| *size).expect("oracle plan set is non-empty");
        regret += selected_size - oracle.0 as i64;
        if candidates.iter().any(|p| same_plan(p, &oracle.1)) { recall += 1; }
        blocks += 1;
    }
    let stats = summarize(planner_samples);
    Ok(vec![json!({"workload_id":"mixed_8m","path":"deterministic_cost_model_v2","blocks":blocks,"oracle_regret_bytes":regret,"normalized_regret_bytes_per_block":regret as f64/blocks.max(1) as f64,"candidate_recall":recall as f64/blocks.max(1) as f64,"planner_timing":timing_json(&stats,data.len())})])
}

/// Benchmarks scaling and verifies bit-for-bit determinism across worker counts.
fn parallel_family() -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let data = mixed_data(16); let mut results = Vec::new(); let mut reference = None;
    for threads in [1usize, 2, 4, 8] {
        let mut cfg=AceConfig::default(); cfg.threads=threads;
        let engine=AceEngine::new(cfg)?;
        let (stats, encoded)=measure(|| Ok(engine.compress(&data)?))?;
        if let Some(ref bytes)=reference { assert_eq!(bytes, &encoded); } else { reference=Some(encoded.clone()); }
        results.push(json!({"workload_id":"mixed_16m","path":format!("threads-{threads}"),"threads":threads,"encoded_bytes":encoded.len(),"compression":timing_json(&stats,data.len())}));
    }
    Ok(results)
}

/// Benchmarks full reconstruction against indexed single-block and logical-range access.
fn random_access_family() -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let data=mixed_data(16); let engine=AceEngine::default_engine(); let encoded=engine.compress(&data)?;
    let (full, restored)=measure(|| Ok(engine.decompress(&encoded)?))?; assert_eq!(restored,data);
    let block_id=20u64;
    let (block_stats, block)=measure(|| { let mut d=AceIndexedDecoder::open(Cursor::new(&encoded),ace_core::DecodeLimits::default())?; Ok(d.decode_block(block_id)?) })?;
    let range_start=3_000_000u64; let range_end=3_065_536u64;
    let (range_stats, range)=measure(|| { let mut d=AceIndexedDecoder::open(Cursor::new(&encoded),ace_core::DecodeLimits::default())?; Ok(d.read_range(range_start..range_end)?) })?;
    assert_eq!(range, data[range_start as usize..range_end as usize]);
    Ok(vec![
        json!({"workload_id":"mixed_16m","path":"full_decompress","bytes_returned":data.len(),"timing":timing_json(&full,data.len())}),
        json!({"workload_id":"mixed_16m","path":"decode_block","block_id":block_id,"bytes_returned":block.len(),"timing":timing_json(&block_stats,block.len())}),
        json!({"workload_id":"mixed_16m","path":"read_range_64k","offset":range_start,"bytes_returned":range.len(),"timing":timing_json(&range_stats,range.len())}),
    ])
}

/// Returns the serialized payload size of one internal plan, including entropy model metadata and primary-length prefix.
fn encoded_plan_size(input: &[u8], plan: &PhysicalCompressionPlan) -> Result<usize, Box<dyn std::error::Error>> {
    let (metadata,payload,_) = encode_plan_payload(input,plan)?;
    let prefix = if matches!(plan.decoding.entropy,EntropyCodecId::None){0}else{4};
    Ok(metadata.len()+payload.len()+prefix)
}

/// Returns true when two plans have identical decoder semantics and LZ search policy.
fn same_plan(a:&PhysicalCompressionPlan,b:&PhysicalCompressionPlan)->bool { a.decoding==b.decoding && a.lz_mode==b.lz_mode }

/// Constructs the exhaustive internal plan set used only by the offline planner-oracle benchmark.
fn oracle_plans()->Vec<PhysicalCompressionPlan>{
    let mut plans=Vec::new();
    let mut add=|transforms:Vec<TransformId>,codec:CodecId,entropy:EntropyCodecId,lz:Option<LzMode>| plans.push(PhysicalCompressionPlan{decoding:DecodingPlan{transforms,codec,dictionary:None,entropy},lz_mode:lz,cost:Default::default(),score:0,reason:"oracle"});
    add(vec![],CodecId::Raw,EntropyCodecId::None,None);
    for e in [EntropyCodecId::Huffman,EntropyCodecId::Rans]{ add(vec![],CodecId::Raw,e,None); add(vec![],CodecId::Rle,e,None); add(vec![TransformId::DeltaByte],CodecId::Raw,e,None); for lz in [LzMode::Fast,LzMode::Balanced]{add(vec![],CodecId::Lz,e,Some(lz));}}
    add(vec![],CodecId::Rle,EntropyCodecId::None,None);
    plans
}
