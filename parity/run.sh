#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "$0")/.."
source parity/check-reference.sh
mkdir -p parity/build
g++ -O2 -std=c++17 -I"$FLIP_RS_REFERENCE/src/cpp" parity/reference.cpp -o parity/build/reference
export FLIP_RS_PARITY_BIN="$PWD/parity/build/reference"
cargo run --release --features image --example parity
python3 parity/record.py "${FLIP_RS_REPORT:-parity/build/parity.md}"
FLIP_RS_REPORT=parity/build/benchmarks-single.md cargo run --release --no-default-features --features image --example parity -- --bench
python3 parity/record.py parity/build/benchmarks-single.md
RAYON_NUM_THREADS="${RAYON_NUM_THREADS:-8}" FLIP_RS_REPORT=parity/build/benchmarks-parallel.md cargo run --release --features image --example parity -- --bench
python3 parity/record.py parity/build/benchmarks-parallel.md
