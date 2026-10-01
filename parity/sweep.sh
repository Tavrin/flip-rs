#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "$0")/.."
source parity/check-reference.sh
mkdir -p parity/build parity/results
g++ -O2 -std=c++17 -I"$FLIP_RS_REFERENCE/src/cpp" parity/reference.cpp -o parity/build/reference
export FLIP_RS_PARITY_BIN="$PWD/parity/build/reference"
cargo run --release --no-default-features --features image --example parity -- --sweep "$@"
python3 parity/record.py "${FLIP_RS_REPORT:-parity/results/sweep.md}"
