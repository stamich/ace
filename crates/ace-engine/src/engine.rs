use crate::{BlockExplanation, FixedBlockChunker};
use ace_analysis::{BlockAnalyzer, DefaultBlockAnalyzer};
use ace_codecs::{decode_codec, decode_entropy, encode_codec, encode_entropy, invert_transform};
use ace_core::{
    AceConfig, AceError, AceResult, CodecId, CompressionStats, DecodeLimits,
    PhysicalCompressionPlan,
};
use ace_format::{
    checksum, AceReader, AceWriter, BlockHeader, FileHeader, BLOCK_HEADER_SIZE, FILE_HEADER_SIZE,
};
use ace_planner::{evaluate_candidates, CompressionPlanner, DefaultCompressionPlanner};
use std::io::{Cursor, Read, Write};
use std::time::Instant;

/// Public façade of the ACE 0.1 compression engine.
#[derive(Debug, Clone)]
pub struct AceEngine {
    config: AceConfig,
    limits: DecodeLimits,
}

impl AceEngine {
    /// Creates an engine with explicit encoder configuration and default decoder limits.
    pub fn new(config: AceConfig) -> AceResult<Self> {
        if config.block_size == 0 || config.block_size > u32::MAX as usize {
            return Err(AceError::InvalidConfig(
                "block size must fit u32 and be non-zero",
            ));
        }
        Ok(Self {
            config,
            limits: DecodeLimits::default(),
        })
    }
    /// Creates an engine using milestone-0.1 defaults.
    pub fn default_engine() -> Self {
        Self::new(AceConfig::default()).expect("default ACE configuration is valid")
    }
    /// Replaces decoder resource limits.
    pub fn with_decode_limits(mut self, limits: DecodeLimits) -> Self {
        self.limits = limits;
        self
    }
    /// Returns a read-only view of encoder configuration.
    pub fn config(&self) -> &AceConfig {
        &self.config
    }

    /// Compresses an in-memory byte slice into a complete ACE file.
    pub fn compress(&self, input: &[u8]) -> AceResult<Vec<u8>> {
        let mut out = Vec::new();
        self.compress_to(input, &mut out)?;
        Ok(out)
    }
    /// Decompresses an in-memory ACE file into reconstructed bytes.
    pub fn decompress(&self, input: &[u8]) -> AceResult<Vec<u8>> {
        let mut out = Vec::new();
        self.decompress_from(Cursor::new(input), &mut out)?;
        Ok(out)
    }

    /// Compresses source bytes to an arbitrary writer and returns telemetry.
    pub fn compress_to<W: Write>(&self, input: &[u8], writer: W) -> AceResult<CompressionStats> {
        let chunker = FixedBlockChunker::new(self.config.block_size)
            .ok_or(AceError::InvalidConfig("zero block size"))?;
        let block_count = if input.is_empty() {
            0
        } else {
            (input.len() + self.config.block_size - 1) / self.config.block_size
        };
        let file = FileHeader {
            flags: 0,
            default_block_size: self.config.block_size as u32,
            original_size: input.len() as u64,
            block_count: block_count as u64,
        };
        let mut ace = AceWriter::new(writer);
        ace.write_file_header(&file)?;
        let analyzer = DefaultBlockAnalyzer;
        let planner = DefaultCompressionPlanner;
        let mut stats = CompressionStats::default();
        stats.input_bytes = input.len() as u64;
        for (id, block) in chunker.chunks(input).enumerate() {
            let t = Instant::now();
            let profile = analyzer.analyze(block);
            stats.analysis_time += t.elapsed();
            let t = Instant::now();
            let candidates = planner.candidates(&profile, &self.config);
            let selected = evaluate_candidates(block, &candidates, &self.config)?;
            stats.planning_time += t.elapsed();
            let t = Instant::now();
            let (mut metadata, mut payload) = _encode_plan_for_block(block, &selected)?;
            let mut final_plan = selected.clone();
            let nonraw_overhead =
                BLOCK_HEADER_SIZE + selected.decoding.transforms.len() + metadata.len();
            if selected.decoding.codec != CodecId::Raw
                || !selected.decoding.transforms.is_empty()
                || !metadata.is_empty()
            {
                if payload.len() + nonraw_overhead + self.config.min_gain_bytes
                    >= block.len() + BLOCK_HEADER_SIZE
                {
                    final_plan = PhysicalCompressionPlan::raw();
                    metadata.clear();
                    payload = block.to_vec();
                }
            }
            stats.encoding_time += t.elapsed();
            let h = BlockHeader {
                block_id: id as u64,
                original_size: block.len() as u32,
                encoded_size: payload.len() as u32,
                metadata_size: metadata.len() as u32,
                codec: final_plan.decoding.codec,
                entropy: final_plan.decoding.entropy,
                transforms: final_plan.decoding.transforms.clone(),
                flags: 0,
                payload_crc32c: checksum(block),
            };
            ace.write_block(&h, &metadata, &payload)?;
            stats.block_count += 1;
            match h.codec {
                CodecId::Raw => stats.raw_blocks += 1,
                CodecId::Rle => stats.rle_blocks += 1,
                CodecId::Lz => stats.lz_blocks += 1,
            }
            if h.transforms
                .iter()
                .any(|t| matches!(t, ace_core::TransformId::DeltaByte))
            {
                stats.delta_blocks += 1;
            }
            stats.output_bytes +=
                (BLOCK_HEADER_SIZE + h.transforms.len() + metadata.len() + payload.len()) as u64;
        }
        stats.output_bytes += FILE_HEADER_SIZE as u64;
        Ok(stats)
    }

    /// Reads, validates and decompresses an ACE stream to an arbitrary writer.
    pub fn decompress_from<R: Read, W: Write>(&self, reader: R, mut writer: W) -> AceResult<()> {
        let mut ace = AceReader::new(reader, self.limits.clone());
        let file = ace.read_file_header()?;
        let mut total = 0u64;
        for expected_id in 0..file.block_count {
            let (h, metadata, payload) = ace.read_block()?;
            if h.block_id != expected_id {
                return Err(AceError::Malformed("unexpected block id"));
            }
            let transformed_size = h.original_size as usize; // all milestone transforms preserve byte length
            let (entropy_meta, primary_size) =
                if matches!(h.entropy, ace_core::EntropyCodecId::Huffman) {
                    if metadata.len() < 260 {
                        return Err(AceError::InvalidHuffman("missing primary-stream length"));
                    }
                    (
                        &metadata[4..],
                        u32::from_le_bytes(metadata[0..4].try_into().unwrap()) as usize,
                    )
                } else {
                    (&metadata[..], payload.len())
                };
            let codec_bytes = decode_entropy(h.entropy, entropy_meta, &payload, primary_size)?;
            let mut stage = decode_codec(h.codec, &codec_bytes, transformed_size)?;
            for &t in h.transforms.iter().rev() {
                stage = invert_transform(t, &stage)?;
            }
            if stage.len() != h.original_size as usize {
                return Err(AceError::Malformed("decoded block size mismatch"));
            }
            if checksum(&stage) != h.payload_crc32c {
                return Err(AceError::ChecksumMismatch(h.block_id));
            }
            writer.write_all(&stage)?;
            total = total
                .checked_add(stage.len() as u64)
                .ok_or(AceError::Malformed("output size overflow"))?;
            if total > file.original_size || total > self.limits.max_output_size {
                return Err(AceError::ResourceLimitExceeded("decoded output size"));
            }
        }
        if total != file.original_size {
            return Err(AceError::Malformed(
                "file original size does not equal reconstructed block sizes",
            ));
        }
        Ok(())
    }

    /// Produces detailed planner explanations without writing an ACE stream.
    pub fn explain(&self, input: &[u8]) -> AceResult<Vec<BlockExplanation>> {
        let chunker = FixedBlockChunker::new(self.config.block_size)
            .ok_or(AceError::InvalidConfig("zero block size"))?;
        let analyzer = DefaultBlockAnalyzer;
        let planner = DefaultCompressionPlanner;
        let mut out = Vec::new();
        for (id, block) in chunker.chunks(input).enumerate() {
            let p = analyzer.analyze(block);
            let c = planner.candidates(&p, &self.config);
            let s = evaluate_candidates(block, &c, &self.config)?;
            out.push(BlockExplanation {
                block_id: id as u64,
                profile: p,
                candidates: c,
                selected: s,
            });
        }
        Ok(out)
    }
}

/// Encodes a selected plan while prefixing Huffman metadata with the primary token-stream byte count.
fn _encode_plan_for_block(
    input: &[u8],
    plan: &PhysicalCompressionPlan,
) -> AceResult<(Vec<u8>, Vec<u8>)> {
    let mut stage = input.to_vec();
    for &t in &plan.decoding.transforms {
        stage = ace_codecs::apply_transform(t, &stage)?;
    }
    stage = encode_codec(plan.decoding.codec, plan.lz_mode, &stage)?;
    let primary_len = stage.len();
    let (mut metadata, payload) = encode_entropy(plan.decoding.entropy, &stage)?;
    if matches!(plan.decoding.entropy, ace_core::EntropyCodecId::Huffman) {
        let mut pref = Vec::with_capacity(metadata.len() + 4);
        pref.extend_from_slice(&(primary_len as u32).to_le_bytes());
        pref.append(&mut metadata);
        metadata = pref;
    }
    Ok((metadata, payload))
}
