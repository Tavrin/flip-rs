# Arbitrary percentile qualification (0.1.2)

Measured 2026-10-02 against a clean NVIDIA FLIP v1.7 checkout at
`b475eb4bf394ab877c42166c9eb0a84a02cc5b14`, using
`FLIP_RS_REFERENCE=<path to the reference checkout>`. The refreshed
[corpus](parity.md) and [sweep](sweep.md) records bind the working sources,
test files, driver and report scripts, and measured executables by SHA-256.
Rust and Cargo: 1.98.1; GCC: 13.3.0. The C++ driver uses
`g++ -O2 -std=c++17` without OpenMP or architecture flags. Corpus Rust uses
the default release features with eight Rayon threads; sweep Rust uses
`--no-default-features --features image`.

## API and decisions

- `ErrorMap::percentile(p, Weighting)` is the fallible one-shot call.
  `Weighting::{Weighted, Unweighted}` replaces a boolean parameter.
- `ErrorMap::percentiles()` returns an owned `Percentiles` value. It copies
  and sorts once, computes the row-major total once, and allocates nothing
  during queries. Weighted queries scan the sorted data; unweighted queries
  index it.
- Invalid fractions and out-of-bounds unweighted indices reuse
  `FlipError::InvalidParameter`. Empty maps return
  `FlipError::InvalidDimensions`; allocating the copy is fallible.
- The reference's sequential `f32` arithmetic, strict weighted threshold
  and fallback to zero are retained. Counts are converted to `f32` before
  computing the unweighted index. Existing `Statistics` calculations are
  unchanged. Evaluation timing excludes the added percentile checks.

## Parity

Both weightings are checked at 0, 0.01, 0.25, 0.5, 0.75, 0.9, 0.95, 0.99,
0.999 and the largest defined `f32` below 1 for each weighting and map size.
The C++ driver marks undefined indices without calling `getPercentile` at
them. The Rust API must reject those indices. Defined queries must match
bit for bit on the C++ error map; separately evaluated Rust maps retain the
existing 1e-6 pooling gate.

| Run | Defined cases | Weighted exact queries | Unweighted exact queries | Undefined unweighted queries rejected | Max percentile difference |
|---|---:|---:|---:|---:|---:|
| Corpus | 201 | 2,010 | 1,579 | 431 | 0 |
| Sweep (3,000 cases, seed 3572951) | 2,974 | 29,740 | 26,457 | 3,283 | 0 |

Pixel, existing pooled and exposure-map differences are also zero. Histogram
and exposure counts match. The sweep retains its 26 documented inputs on
which image evaluation in the reference is undefined, with no failures.

## Undefined percentile queries and retained edge behaviour

Fractions outside `0..=1`, NaN and infinity are rejected. The reference's
zero-based unweighted `ceil((count as f32) * p)` index can exceed the map
even below 1: three pixels reject 0.99, and one pixel defines only zero.
At 1 the index is usually out of bounds; very large counts that round down
in `f32` can still yield a defined index. The index is never clamped.
Public construction rejects empty maps; the pooling constructor's empty
guard is also tested directly.

Weighted all-zero maps return zero at every valid fraction. Weighted 1 is
defined and normally returns zero; sorted accumulation can exceed the
row-major total because of rounding (tested with `[1e8, 4, 4]`). Overflow
of an externally supplied map's total retains the reference's zero fallback.

## Verification

All Cargo commands use
`CARGO_TARGET_DIR=/mnt/linux-extra/moss-cargo-targets/codex-flip-rs-percentile`.
Tests use `FLIP_RS_PARITY_BIN=parity/build/reference` and eight Rayon threads.

| Command or check | Result |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets --all-features -- -D warnings` | exit 0 |
| `cargo test` | exit 0; 4 unit, 11 API, 3 oracle tests, 6 doctests |
| `cargo test --all-features` | exit 0; 4 unit, 12 API, 3 oracle tests, 6 doctests |
| `cargo test --no-default-features` | exit 0; 4 unit, 11 API, 3 oracle tests, 6 doctests |
| `cargo build --target wasm32-unknown-unknown --no-default-features` | exit 0 |
| Same wasm build with `--features wasm` | exit 0 |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features` | exit 0 |
| `cargo publish --dry-run --allow-dirty` | exit 0; package verified, nothing published |
| `cargo run --release --features image --example parity` with the oracle and report variables set | exit 0; full 201-case corpus |
| `parity/sweep.sh` | exit 0; full 3,000-case sweep |

The dry-run uses `--allow-dirty` because the coordinator owns the commit.
README and the wgpu outreach draft now document the percentile API; the
crate version and changelog are 0.1.2. No benchmark or fuzzing campaign was
rerun for this feature. The isolated Cargo target directory is removed
after qualification; Git metadata is unchanged.
