use ace_bench::planner_regret;
use ace_core::AceConfig;
use std::{env, fs};

/// Runs an offline planner-regret report over every ACE-sized block in a file.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args()
        .nth(1)
        .ok_or("usage: cargo run -p ace-bench --bin oracle -- <file>")?;
    let data = fs::read(&path)?;
    let config = AceConfig::default();
    let mut total_regret = 0isize;
    for (id, block) in data.chunks(config.block_size).enumerate() {
        let r = planner_regret(block, &config)?;
        total_regret += r.regret_bytes.max(0);
        println!(
            "block={id} selected={} oracle={} regret_bytes={}",
            r.selected.reason, r.oracle.reason, r.regret_bytes
        );
    }
    println!("total_positive_regret_bytes={total_regret}");
    Ok(())
}
