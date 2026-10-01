# Qualification record

Measured on 2026-10-01 against NVIDIA FLIP v1.7 at
`b475eb4bf394ab877c42166c9eb0a84a02cc5b14`, built from an unmodified clone
referred to below as `$FLIP_RS_REFERENCE`.

## Environment

| Item | Value |
|---|---|
| CPU | AMD Ryzen 9 7945HX, 16 cores / 32 logical CPUs |
| OS | Linux x86-64 |
| Rust | rustc and cargo 1.98.1, release profile defaults |
| C++ | GCC 13.3.0, `g++ -O2 -std=c++17`, no OpenMP |
| Flags | No `-march=native`/`target-cpu=native`, fast math or FMA flags on either side |
| Parallel Rust | `RAYON_NUM_THREADS=8`; the C++ driver is single-threaded in every row |

At the time of measurement:

- `Cargo.lock` SHA-256: `4e4fc4ff20f311e0daf9a9745c8d8d3e0a9dee320be259961936d22ea6e4d0b3`
- C++ driver (`parity/build/reference`) SHA-256: `e2b0f3e8a4baeeaa600b54a9e86371fe76d4fb2ec58f6906a96d09d7d338d9f7`

## Checks

| Command | Result |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets --all-features -- -D warnings` | exit 0 |
| `cargo test --all-features` | exit 0, including PNG/EXR round trips |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features` | exit 0 |
| `cargo build --target wasm32-unknown-unknown --no-default-features` | exit 0 |
| `cargo build --release --target wasm32-unknown-unknown --no-default-features --features wasm` | exit 0 |
| `wasm-bindgen --target web` (0.2.114) | exit 0 |
| `FLIP_RS_PARITY_BIN=parity/build/reference cargo test --release --no-default-features --features image` | exit 0, oracle tests run |
| `FLIP_RS_PARITY_BIN=parity/build/reference cargo test --release` | exit 0, oracle tests run |
| `FLIP_RS_REFERENCE=$FLIP_RS_REFERENCE ./parity/run.sh` | exit 0, 201 cases and both benchmark modes |
| Browser demo in Chromium | `ldrFlip` returns a `Float32Array` for the upstream PNG pair and rejects mismatched sizes |

## Numerical parity

From [parity.md](parity.md):

| Corpus | Cases | Max pixel difference | Max pooled difference | Max exposure-map difference |
|---|---:|---:|---:|---:|
| Generated HDR | 157 | 2.980232239e-8 | 0.000000000e0 | 0.000000000e0 |
| Generated LDR | 21 | 2.980232239e-8 | 0.000000000e0 | 0.000000000e0 |
| Reference EXR | 18 | 5.960464478e-8 | 0.000000000e0 | 0.000000000e0 |
| Reference PNG | 5 | 5.960464478e-8 | 0.000000000e0 | 0.000000000e0 |

All pooled statistics, histogram counts and exposure maps match exactly.
Resolved exposure endpoints agree within 1e-6 and exposure counts are equal.

### Source of the residual

LLVM compiles the feature exponent `powf(x, 0.5)` in
`Filters::feature_difference_and_final_error` to `sqrtss`/`sqrtps`. GCC calls
glibc `powf` for the same expression in reference
`image::computeFeatureDifferenceAndFinalError` (`FLIP.h` line 2036). A
diagnostic copy of `FLIP.h` with only that expression replaced by `std::sqrt`
removes the difference. The diagnostic copy was not used for any other result
in this record.

| Input, default PPD | Unmodified reference | Reference with `std::sqrt` |
|---|---:|---:|
| Seeded noise 127×127 | 1.4901161e-8 | 0 |
| Seeded noise 1024×1024 | 1.4901161e-8 | 0 |
| Upstream PNG pair | 2.9802322e-8 | 0 |

## Benchmarks

Seeded noise pairs at the default PPD; HDR uses ACES with exposures −4, 0 and
+4. Each value is the median of three evaluations after one warm-up. The timed
region covers color conversion, tone mapping, filtering and allocation, and
excludes decoding, raw file I/O, process startup, image construction and
pooling. Every timed output was also checked against the reference.

Single-threaded Rust (`--no-default-features`), from
[benchmarks-single.md](benchmarks-single.md):

| Case | C++ -O2 single thread (s) | Rust (s) | C++ / Rust |
|---|---:|---:|---:|
| LDR 1920×1080 | 0.663150 | 0.312559 | 2.12× |
| HDR ACES, 3 exposures 1920×1080 | 1.823382 | 0.664654 | 2.74× |
| LDR 3840×2160 | 2.824842 | 1.402038 | 2.01× |
| HDR ACES, 3 exposures 3840×2160 | 7.700175 | 2.927472 | 2.63× |

Rust with 8 Rayon threads, from
[benchmarks-parallel.md](benchmarks-parallel.md). The C++ times are from the
same run and are single-threaded, so the ratio is not a like-for-like
comparison:

| Case | C++ -O2 single thread (s) | Rust (s) | C++ / Rust |
|---|---:|---:|---:|
| LDR 1920×1080 | 0.680124 | 0.053970 | 12.60× |
| HDR ACES, 3 exposures 1920×1080 | 1.831335 | 0.118964 | 15.39× |
| LDR 3840×2160 | 2.882731 | 0.222860 | 12.94× |
| HDR ACES, 3 exposures 3840×2160 | 7.696405 | 0.537772 | 14.31× |

### Untiled filters

An earlier version of the filters, with per-tap border checks and no tiling,
was measured on the same host and inputs in a separate run with different
background load. Its results are in
[benchmarks-untiled-single.md](benchmarks-untiled-single.md) and
[benchmarks-untiled-parallel.md](benchmarks-untiled-parallel.md):

| Case | Rust threads | Untiled (s) | Tiled (s) | Untiled / tiled |
|---|---:|---:|---:|---:|
| LDR 1920×1080 | 1 | 0.669013 | 0.312559 | 2.14× |
| HDR ACES, 3 exposures 1920×1080 | 1 | 1.781044 | 0.664654 | 2.68× |
| LDR 3840×2160 | 1 | 2.749022 | 1.402038 | 1.96× |
| HDR ACES, 3 exposures 3840×2160 | 1 | 7.280522 | 2.927472 | 2.49× |
| LDR 1920×1080 | 8 | 0.210339 | 0.053970 | 3.90× |
| HDR ACES, 3 exposures 1920×1080 | 8 | 0.304838 | 0.118964 | 2.56× |
| LDR 3840×2160 | 8 | 0.858680 | 0.222860 | 3.85× |
| HDR ACES, 3 exposures 3840×2160 | 8 | 1.143651 | 0.537772 | 2.13× |

The tiled filters store Y, Cx and Cz as horizontally padded planes, so the
horizontal pass needs no border checks, and both passes run over contiguous
128-pixel tiles. Each pixel's taps are still summed in reference order;
vectorization is across pixels. Calls to `powf` stay scalar. HDR evaluation
allocates its planes, intermediate buffer and error buffer once and reuses
them for every exposure.

## Code generation

Packed SSE instruction counts in the release parity executable. These are
counts of emitted instructions, not runtime counters:

```text
<flip_rs::filters::Filters>::color_difference::{closure#0}>:
  mulps=2, addps=2, sqrtps=0, FMA=0
<flip_rs::filters::Filters>::color_difference::{closure#1}>:
  mulps=83, addps=54, sqrtps=1, FMA=0
<flip_rs::filters::Filters>::feature_difference_and_final_error::{closure#0}>:
  mulps=2, addps=2, sqrtps=0, FMA=0
<flip_rs::filters::Filters>::feature_difference_and_final_error::{closure#1}>:
  mulps=29, addps=22, sqrtps=7, FMA=0
<flip_rs::filters::Filters>::new>:
  mulps=8, addps=2, sqrtps=0, FMA=0
```

## Stage profile

Collected with `parity/profile.py`, single-threaded, untiled and tiled filters.
Hardware performance counters were unavailable (`perf_event_paranoid=4`), so
these are wall-clock stage times. Full breakdowns, raw samples and source
hashes are in [profile-untiled.md](profile-untiled.md) and
[profile.md](profile.md).

| Stage (ms, untiled → tiled) | 1080p LDR | 1080p HDR | 4K LDR | 4K HDR |
|---|---:|---:|---:|---:|
| Conversion/setup | 108.8 → 89.2 | 126.5 → 48.7 | 500.9 → 393.5 | 653.7 → 213.0 |
| Color H + V/Lab metric | 304.5 → 181.2 | 969.1 → 501.2 | 1220.9 → 780.9 | 4607.1 → 2022.6 |
| Features H + V/final metric | 172.6 → 73.3 | 538.3 → 241.8 | 731.8 → 319.7 | 3070.2 → 950.1 |
| HDR setup/merge | — | 8.6 → 8.4 | — | 54.3 → 50.4 |
| API total | 595.0 → 354.4 | 1685.3 → 803.4 | 2499.6 → 1547.1 | 8677.5 → 3399.9 |
| Pooling, outside API | 53.2 → 58.9 | 59.2 → 61.9 | 226.6 → 245.6 | 270.0 → 240.0 |

Combined rows add the medians of their component stages. HDR profiles use
explicit exposures, so setup does not include the automatic median luminance.
Instrumentation changes code generation as well as timing; the benchmark
tables above are the performance figures.

## Not established

- The macOS and Windows CI jobs, Firefox and Safari were not run for this
  record.
- Rust 1.88, the declared minimum, was not tested for this record.
- Rust and C++ read identical decoded floats, so this does not test decoder
  agreement, and the corpus does not prove equivalence for every input.
- Timings are for this host and these inputs.
