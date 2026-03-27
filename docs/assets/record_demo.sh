#!/bin/bash
# Scripted demo for asciinema recording
# Usage: asciinema rec --command "bash docs/assets/record_demo.sh" demo.cast

set -e
cd "$(dirname "$0")/../.."

DELAY=0.03

type_cmd() {
    echo ""
    for (( i=0; i<${#1}; i++ )); do
        printf '%s' "${1:$i:1}"
        sleep $DELAY
    done
    echo ""
    sleep 0.3
}

echo "╔══════════════════════════════════════════════════════╗"
echo "║         Awesome Bitcoin Internals — Demo            ║"
echo "║         SimpleBTC: Learn Bitcoin by Building It     ║"
echo "╚══════════════════════════════════════════════════════╝"
echo ""
sleep 1

echo "$ cargo test --quiet"
sleep 0.5
cargo test --quiet 2>&1
sleep 1.5

echo ""
echo "$ cargo run --bin btc-demo --release --quiet"
sleep 0.5
cargo run --bin btc-demo --release --quiet 2>&1
sleep 2

echo ""
echo "╔══════════════════════════════════════════════════════╗"
echo "║  101 tests passing · 0 warnings · Ready to learn!  ║"
echo "╚══════════════════════════════════════════════════════╝"
sleep 2
