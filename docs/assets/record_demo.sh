#!/bin/bash
# Scripted demo for asciinema recording — with paced output
# Usage: asciinema rec --command "bash docs/assets/record_demo.sh" demo.cast
set -e
cd "$(dirname "$0")/../.."

slow_print() {
    while IFS= read -r line; do
        echo "$line"
        sleep 0.15
    done
}

echo ""
echo "╔══════════════════════════════════════════════════════╗"
echo "║       Awesome Bitcoin Internals — Demo              ║"
echo "║       SimpleBTC: Learn Bitcoin by Building It       ║"
echo "╚══════════════════════════════════════════════════════╝"
sleep 1.5

echo ""
echo "$ cargo test --quiet"
sleep 0.8
cargo test --quiet 2>&1 | slow_print
sleep 1.5

echo ""
echo "────────────────────────────────────────────────────────"
echo ""
echo "$ cargo run --bin btc-demo --release --quiet"
sleep 0.8
cargo run --bin btc-demo --release --quiet 2>&1 | head -55 | slow_print
sleep 2

echo ""
echo "╔══════════════════════════════════════════════════════╗"
echo "║  101 tests · 0 warnings · 7,900 lines of Rust      ║"
echo "║  Ready to learn Bitcoin internals!                  ║"
echo "╚══════════════════════════════════════════════════════╝"
sleep 3
