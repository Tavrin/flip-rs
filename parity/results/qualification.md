# Qualification record

Measured 2026-10-02 against a clean NVIDIA FLIP v1.7 checkout at
`b475eb4bf394ab877c42166c9eb0a84a02cc5b14` (`$FLIP_RS_REFERENCE`). The Rust
sources were base revision `5f4d5c29a0dc40fed2bda8e1d6f6ccc1772d412e` plus the
uncommitted hardening changes; the corpus, sweep, benchmark and fuzz records
list SHA-256 hashes of the exact sources and executables.

## Environment

AMD Ryzen 9 7945HX (16 cores, 32 logical CPUs), Linux x86-64, Rust and cargo
1.98.1, GCC 13.3.0. C++ is built with `g++ -O2 -std=c++17`, without OpenMP or
architecture flags. Rust uses the default release profile. Parallel Rust uses
8 Rayon threads; the C++ driver is single-threaded.

## Results

- [Corpus](parity.md): 201 cases, zero pixel, pooled and exposure-map
  differences, equal histograms and exposure counts.
- [Randomized sweep](sweep.md): seed `3572951`, 3,000 cases. 2,974 defined
  cases with zero differences; 26 inputs on which the reference is undefined,
  each with Rust's error and the C++ result. The two near-tie cases found by an
  earlier sweep are fixed and tested by seed.
- [Fuzzing](fuzz.md): five targets, 311 seconds each, nightly with
  AddressSanitizer, 144,465,338 executions, no crashes.
- Benchmarks: [single-threaded](benchmarks-single.md) and
  [8-thread Rust](benchmarks-parallel.md).

## Verification

| Command or check | Result |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets --all-features -- -D warnings` | exit 0 |
| `cargo test --all-features` with `FLIP_RS_PARITY_BIN` set | exit 0; 3 unit, 9 API, 3 oracle tests, 4 doctests |
| `cargo test --no-default-features` | exit 0 |
| `cargo build --examples --all-features` | exit 0 |
| `cargo build --target wasm32-unknown-unknown --no-default-features` | exit 0 |
| Same wasm build with `--features wasm` | exit 0 |
| `RUSTDOCFLAGS="--cfg docsrs -D warnings" cargo +nightly doc --all-features --no-deps` | exit 0 |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` | exit 0 |
| Same stable docs with `--no-default-features` | exit 0 |
| `cargo publish --dry-run --allow-dirty` | exit 0 |
| `parity/run.sh` | exit 0; full corpus and both benchmarks |
| `parity/sweep.sh` (3,000 cases) | exit 0 |
| `cargo +nightly fuzz run`, all five targets | exit 0 each |
| `cargo fmt --manifest-path fuzz/Cargo.toml --check` | exit 0 |
| `bash -n` on the three parity shell scripts | exit 0 |
| `check-reference.sh` with a modified reference / failing `git status` (mocked) | exit 1 / 128, before building |
| `compare` example with an EXR/PNG pair | exit 1, before decoding |

The CI workflow was parsed as YAML and checked by hand; `actionlint` was not
run.

## Benchmark method

Each timing is the median of three evaluations after one warm-up. It includes
color and tone conversion, filtering and allocation, and excludes input
construction, decoding, file I/O and pooling. Restoring the C++ input between
repeats is not timed. Every timed result is also checked against the
reference. The tables were recorded after fuzzing had finished, but other
activity on the machine was not controlled. 8-thread Rust has no
multi-threaded C++ counterpart, and NVIDIA's own build uses `-O3` and OpenMP
rather than the `-O2`, single-threaded build measured here.

The stage profiles and untiled benchmarks in this directory are older
baselines and do not describe the current code.

[hardening.md](hardening.md) lists each library change made before release,
with its reproduction and regression test.
