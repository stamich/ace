# Adaptive Compression Engine (ACE) 0.1.0

ACE 0.1 is the first executable milestone of an adaptive lossless compression engine.
It deliberately focuses on architecture rather than competing with Zstandard on raw codec quality.

## What 0.1 proves

For every independent fixed-size block ACE:

1. profiles the data,
2. generates candidate physical plans,
3. samples candidate plans,
4. chooses a plan using a simple cost model,
5. encodes the block,
6. falls back to RAW when compression does not provide the configured gain,
7. stores enough metadata for a planner-free deterministic decoder.

Implemented building blocks:

- RAW
- packet RLE
- byte DELTA transform
- LZ77-style byte-token encoder (FAST and BALANCED search modes share a wire format)
- canonical Huffman entropy coding
- CRC32C over original block bytes
- independent 256 KiB blocks by default
- CLI: `compress`, `decompress`, `inspect`, `explain`, `verify`
- demo corpus generator
- Criterion microbenchmarks and planner/oracle benchmark command

## Build

```bash
cargo build --workspace --release
cargo test --workspace
```

## Demo

```bash
./demo/run-demo.sh
```

## CLI

```bash
cargo run -p ace-cli -- compress demo/data/mixed-demo.bin demo/data/mixed-demo.ace
cargo run -p ace-cli -- decompress demo/data/mixed-demo.ace demo/data/restored.bin
cargo run -p ace-cli -- inspect demo/data/mixed-demo.ace --blocks
cargo run -p ace-cli -- explain demo/data/mixed-demo.bin
cargo run -p ace-cli -- verify demo/data/mixed-demo.ace
cargo run -p ace-bench --bin oracle -- demo/data/mixed-demo.bin
cargo bench -p ace-bench
```

## Design limits of milestone 0.1

ACE 0.1 is intentionally single-threaded and uses independent blocks. It has no ANS, shared dictionaries,
content-defined chunking, cross-block LZ history, SIMD dispatch, GraphNet-specific transforms, or AdaptiveDB hints.
Those belong to later milestones.

## JVM examples

`integrations/java-demo` and `integrations/scala-demo` are intentionally thin process-based examples showing
how a JVM application can call the 0.1 CLI without adding JNI/Panama to the milestone. They are not part of the
compression core. Their public APIs are fully documented with Javadoc/Scaladoc.
