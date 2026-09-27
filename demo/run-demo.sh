#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
python3 demo/generate_demo.py
cargo run -p ace-cli -- explain demo/data/mixed-demo.bin
cargo run -p ace-cli -- compress demo/data/mixed-demo.bin demo/data/mixed-demo.ace
cargo run -p ace-cli -- inspect demo/data/mixed-demo.ace --blocks
cargo run -p ace-cli -- verify demo/data/mixed-demo.ace
cargo run -p ace-cli -- decompress demo/data/mixed-demo.ace demo/data/restored.bin
cmp demo/data/mixed-demo.bin demo/data/restored.bin
echo "ACE 0.1 demo round-trip: OK"
