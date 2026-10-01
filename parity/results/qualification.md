# Qualification record

Measured 2026-10-02 against a clean NVIDIA FLIP v1.7 checkout at
`b475eb4bf394ab877c42166c9eb0a84a02cc5b14`, referred to as
`$FLIP_RS_REFERENCE`. Rust base revision is
`5f4d5c29a0dc40fed2bda8e1d6f6ccc1772d412e`; all hardening changes remain
uncommitted. Source snapshots and executed binary SHA-256 hashes are recorded
in the linked corpus, sweep, benchmark and fuzz records, so the base revision
alone is not presented as the measured source.

## Environment and measurements

AMD Ryzen 9 7945HX (16 cores / 32 logical CPUs), Linux x86-64, Rust/cargo
1.98.1, GCC 13.3.0. C++ uses `g++ -O2 -std=c++17`, without OpenMP or architecture
flags. Rust uses ordinary release defaults. Parallel Rust uses eight Rayon
threads; the C++ driver remains single-threaded. No GPU qualification was run.

The [201-case corpus](parity.md) and [3,000-case randomized sweep](sweep.md)
pass with **zero pixel, pooled and exposure-map differences** on their defined
inputs, exact histograms/counts, and no relaxed tolerances. The sweep's seed
is `3572951`: 2,974 numerical parity cases and 26 documented undefined-reference
cases. Each rejected case records Rust's error and the actual C++ result/exit.
The two original near-tie outliers are fixed and tested with their exact seeds.

All [five fuzz targets](fuzz.md) ran on the final library for 311 seconds each,
with **144,465,338 executions and zero crashes**, using nightly and AddressSanitizer.
Toolchain/cache permissions required temporary writable locations; actual
nightly fuzzing completed, so no stable fallback was used.

## Verification

| Command or check | Exit / result |
|---|---|
| `cargo fmt --check` | 0 |
| `cargo clippy --all-targets --all-features -- -D warnings` | 0 |
| `cargo test --all-features` with the C++ oracle enabled | 0; 3 unit, 9 API, 3 oracle tests, 4 doctests |
| `cargo test --no-default-features` | 0 |
| `cargo build --examples --all-features` | 0 |
| `cargo build --target wasm32-unknown-unknown --no-default-features` | 0 |
| Same wasm build with `--features wasm` | 0 |
| `RUSTDOCFLAGS="--cfg docsrs -D warnings" cargo +nightly doc --all-features --no-deps` | 0 |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` | 0 |
| Same stable docs with `--no-default-features` | 0 |
| `cargo publish --dry-run --allow-dirty` | 0; upload aborted by dry-run as intended |
| `parity/run.sh` | 0; full corpus and both benchmark modes |
| `parity/sweep.sh` (3,000 cases) | 0 |
| `cargo +nightly fuzz run` for all five targets | 0 each; 311 seconds each |
| `cargo fmt --manifest-path fuzz/Cargo.toml --check` | 0 |
| `bash -n` on the three parity shell scripts | 0 |
| Dirty-reference / failed status-read mock probes | Expected exits 1 / 128 before build |
| Mixed EXR/PNG comparison example | Expected exit 1 before decoding |
| CI YAML parse, pinned revision and regression job checks | 0 |
| `rm -r -- "$CARGO_TARGET_DIR"` at the end | 0; directory verified absent |

`--allow-dirty` is required for a publish dry-run of the authorized uncommitted
working source. No actual publication took place. `actionlint` is unavailable
locally; CI YAML was parsed and reviewed, but no local actionlint run is claimed.
This does not substitute YAML parsing for actionlint validation.

## Benchmarks

The refreshed tables are [single-threaded](benchmarks-single.md) and
[eight-threaded](benchmarks-parallel.md). C++ input restoration is outside
its timed region. Every timed result is also compared with the oracle. Timings
are medians of three evaluations after one warmup, include color/tone
conversion, filtering and allocation, and exclude input construction, decoding,
process/file I/O and pooling. The final table refresh ran after fuzzing finished;
the first tables collected during fuzzing were discarded. Other host activity
is not controlled, so these are local observations rather than dedicated-host
performance guarantees. Eight-thread Rust has no multithreaded C++ counterpart.

Historical stage profiles/untiled benchmarks remain labeled as historical,
unbound baselines. They are not evidence for this source's current performance.

See [hardening.md](hardening.md) for every library change, its reproduction,
regressions and the added files.
