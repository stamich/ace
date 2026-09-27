# Changelog

All notable changes to Adaptive Compression Engine are documented here.

## [0.1.0] - 2026-09-09

### Added
- Rust workspace split into core, analysis, codecs, planner, format, engine, CLI and benchmark crates.
- Fixed-size independent blocks with a default size of 256 KiB.
- `BlockProfile` with H0 entropy, zero ratio, run score, byte-delta score, repetition score, unique-byte count and incompressibility score.
- Adaptive candidate generation and sample-based plan evaluation.
- FAST, BALANCED and DENSE planner profiles.
- RAW codec and mandatory RAW fallback when encoded representation does not provide the configured minimum gain.
- Packet-oriented RLE codec.
- Byte DELTA transform using wrapping arithmetic.
- LZ77-inspired codec with FAST and BALANCED match finding modes using the same byte-level token format.
- Canonical Huffman entropy coding with serialized code lengths.
- ACE format v1 file and block headers.
- CRC32C verification of reconstructed original block bytes.
- Configurable decoder resource limits and malformed-stream validation.
- `compress`, `decompress`, `inspect`, `explain` and `verify` CLI commands.
- Compression telemetry and explain output.
- Demo corpus generator and demo shell script.
- Unit, round-trip, determinism and malformed-input tests.
- Criterion microbenchmarks and an oracle-vs-planner benchmark utility.
- Thin Java and Scala CLI usage examples with Javadoc and Scaladoc.

### Security
- Decoder validates advertised sizes before allocation.
- LZ references are checked for zero distance, out-of-window references and output overflow.
- Huffman metadata validates duplicate symbols and invalid code lengths.
- All format parsers return typed errors rather than trusting unvalidated input.

### Known limitations
- Single-threaded execution.
- No block index; random access requires scanning block headers.
- No ANS, shared dictionaries or content-defined chunking.
- Huffman metadata is intentionally simple and not space-optimal.
- LZ token stream is generic byte-oriented output rather than a specialized entropy model.
