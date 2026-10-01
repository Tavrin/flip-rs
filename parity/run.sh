#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "$0")/.."
: "${FLIP_RS_REFERENCE:?set FLIP_RS_REFERENCE to a clone of https://github.com/NVlabs/flip at b475eb4 (FLIP v1.7)}"
export FLIP_RS_REFERENCE
if [[ "$(git -C "$FLIP_RS_REFERENCE" rev-parse HEAD)" != b475eb4bf394ab877c42166c9eb0a84a02cc5b14 ]]; then
  echo 'The parity oracle requires NVIDIA FLIP v1.7 at b475eb4.' >&2
  exit 1
fi
mkdir -p parity/build
g++ -O2 -std=c++17 -I"$FLIP_RS_REFERENCE/src/cpp" parity/reference.cpp -o parity/build/reference
export FLIP_RS_PARITY_BIN="$PWD/parity/build/reference"
cargo run --release --features image --example parity
FLIP_RS_REPORT=parity/build/benchmarks-single.md cargo run --release --no-default-features --features image --example parity -- --bench
RAYON_NUM_THREADS="${RAYON_NUM_THREADS:-8}" FLIP_RS_REPORT=parity/build/benchmarks-parallel.md cargo run --release --features image --example parity -- --bench
