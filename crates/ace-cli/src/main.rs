use ace_core::{AceConfig, CompressionProfile, DecodeLimits};
use ace_engine::AceEngine;
use ace_format::AceReader;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use std::{fs, io::Cursor, path::PathBuf};

/// Command-line interface for Adaptive Compression Engine 0.1.
#[derive(Parser)]
#[command(name = "ace", version, about = "Adaptive Compression Engine 0.1")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// Supported ACE CLI operations.
#[derive(Subcommand)]
enum Command {
    /// Compresses a file into ACE format v1.
    Compress {
        input: PathBuf,
        output: PathBuf,
        #[arg(long, value_enum, default_value = "balanced")]
        profile: Profile,
        #[arg(long, default_value_t = 262_144)]
        block_size: usize,
    },
    /// Decompresses an ACE file.
    Decompress { input: PathBuf, output: PathBuf },
    /// Prints ACE container metadata without reconstructing payloads.
    Inspect {
        input: PathBuf,
        #[arg(long)]
        blocks: bool,
    },
    /// Profiles source blocks and explains planner decisions.
    Explain {
        input: PathBuf,
        #[arg(long, value_enum, default_value = "balanced")]
        profile: Profile,
    },
    /// Fully decompresses and validates checksums while discarding reconstructed bytes.
    Verify { input: PathBuf },
}

/// CLI representation of planner profiles.
#[derive(Copy, Clone, Debug, ValueEnum)]
enum Profile {
    Fast,
    Balanced,
    Dense,
}
impl From<Profile> for CompressionProfile {
    fn from(v: Profile) -> Self {
        match v {
            Profile::Fast => Self::Fast,
            Profile::Balanced => Self::Balanced,
            Profile::Dense => Self::Dense,
        }
    }
}

/// Parses command-line arguments and executes the selected operation.
fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Compress {
            input,
            output,
            profile,
            block_size,
        } => {
            let data = fs::read(&input).with_context(|| format!("read {}", input.display()))?;
            let mut c = AceConfig::default();
            c.profile = profile.into();
            c.block_size = block_size;
            let e = AceEngine::new(c)?;
            let encoded = e.compress(&data)?;
            fs::write(&output, &encoded)?;
            println!(
                "{} -> {} bytes ({:.3}x)",
                data.len(),
                encoded.len(),
                if encoded.is_empty() {
                    1.0
                } else {
                    data.len() as f64 / encoded.len() as f64
                }
            );
        }
        Command::Decompress { input, output } => {
            let encoded = fs::read(&input)?;
            let restored = AceEngine::default_engine().decompress(&encoded)?;
            fs::write(&output, &restored)?;
            println!("restored {} bytes", restored.len());
        }
        Command::Verify { input } => {
            let encoded = fs::read(&input)?;
            let restored = AceEngine::default_engine().decompress(&encoded)?;
            println!(
                "OK: {} reconstructed bytes, all block checksums valid",
                restored.len()
            );
        }
        Command::Explain { input, profile } => {
            let data = fs::read(&input)?;
            let mut c = AceConfig::default();
            c.profile = profile.into();
            let e = AceEngine::new(c)?;
            for x in e.explain(&data)? {
                println!("Block {} size={} H0={:.3} zero={:.3} run={:.3} delta={:.3} repeat={:.3} incompressible={:.3}",x.block_id,x.profile.size,x.profile.entropy_h0,x.profile.zero_ratio,x.profile.run_score,x.profile.delta_score,x.profile.repetition_score,x.profile.incompressibility_score);
                for p in &x.candidates {
                    println!(
                        "  candidate {:?}+{:?} transforms={:?}: {}",
                        p.decoding.codec, p.decoding.entropy, p.decoding.transforms, p.reason
                    );
                }
                println!(
                    "  selected {:?}+{:?} transforms={:?} score={:.4} estimated={}\n",
                    x.selected.decoding.codec,
                    x.selected.decoding.entropy,
                    x.selected.decoding.transforms,
                    x.selected.score,
                    x.selected.estimated_size
                );
            }
        }
        Command::Inspect { input, blocks } => {
            let encoded = fs::read(&input)?;
            let mut r = AceReader::new(Cursor::new(&encoded), DecodeLimits::default());
            let h = r.read_file_header()?;
            println!(
                "ACE format 1.0 original={} blocks={} default_block_size={}",
                h.original_size, h.block_count, h.default_block_size
            );
            for _ in 0..h.block_count {
                let (b, m, p) = r.read_block()?;
                if blocks {
                    println!("block {} original={} encoded={} metadata={} codec={:?} entropy={:?} transforms={:?}",b.block_id,b.original_size,p.len(),m.len(),b.codec,b.entropy,b.transforms);
                }
            }
        }
    }
    Ok(())
}
