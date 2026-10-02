# Parity harness

This directory checks flip-rs against NVIDIA's C++ implementation of FLIP v1.7
(revision `b475eb4bf394ab877c42166c9eb0a84a02cc5b14`). None of it is part of
the published crate.

| File | Purpose |
|---|---|
| `sweep.sh` | Seeded randomized oracle gate (default 3,000 cases; `--count`, `--seed`, `--case-seed`). |
| `check-reference.sh` | Rejects a reference checkout at the wrong revision or with local changes. |
| `record.py` | Appends the Rust revision, source hashes and executable hashes to each report. |
| `run.sh` | Builds the C++ driver, runs the full corpus and both benchmarks. |
| `reference.cpp` | Driver that runs the unmodified reference `FLIP.h` on raw float input. |
| `profile.py` | Optional single-thread stage profile of the Rust code. |
| `results/` | Recorded parity, benchmark and profile results. |
| `build/` | Output of `run.sh`; ignored by Git. |

The Rust side lives in `examples/parity.rs`, `examples/support/mod.rs` and
`tests/parity.rs`.

## Running

Set `FLIP_RS_REFERENCE` to a clean clone of <https://github.com/NVlabs/flip>
at the revision above. Both scripts check the revision and refuse a modified
checkout; neither downloads the reference.

```sh
git clone https://github.com/NVlabs/flip nvlabs-flip
git -C nvlabs-flip checkout b475eb4bf394ab877c42166c9eb0a84a02cc5b14
export FLIP_RS_REFERENCE="$PWD/nvlabs-flip"

./parity/run.sh
./parity/sweep.sh
FLIP_RS_REPORT=parity/build/sweep-short.md ./parity/sweep.sh --count 200
```

`run.sh` compiles `reference.cpp` with `g++ -O2 -std=c++17` (no OpenMP or
architecture flags) and then:

1. runs all 201 cases and writes `build/parity.md`;
2. benchmarks Rust built with `--no-default-features` (single-threaded) and
   writes `build/benchmarks-single.md`;
3. benchmarks Rust with the `parallel` feature and `RAYON_NUM_THREADS=8`
   (override with the environment variable) and writes
   `build/benchmarks-parallel.md`.

The C++ driver is always single-threaded. With `FLIP_RS_PARITY_BIN` set to the
compiled driver, `cargo test --test parity` runs the corpus without the
1024×1024 and file-based cases, plus 24 cases at tile boundaries. Without it,
those tests are skipped.

## Corpus and pass criteria

Both implementations read the same `f32` inputs, so image decoding is not part
of the comparison. The corpus contains:

- generated LDR and HDR images: zeros, flat colors, gradients, edges and
  seeded noise from 1×1 to 1024×1024, plus extreme values (1e-20 to 1e30) and
  mostly-black images for HDR;
- the upstream `images/` files: the reference/test PNG and EXR pairs, and the
  teaser PNG compared with itself and with a perturbed copy;
- PPD 20, 67.02 (the default) and 120; the ACES, Hable and Reinhard tone
  mappers; explicit, automatic and partly automatic exposure parameters.

The harness first checks that the driver's pooled statistics and histogram
agree with pooling its own error map in Rust. A case then passes if the
maximum absolute difference is at most 1e-5 per pixel and 1e-6 for pooled
statistics, exposure maps and exposure endpoints, and if histogram counts and
exposure counts are equal.

## Driver protocol

The driver takes tightly packed little-endian `f32` RGB files:

```text
reference ldr|hdr WIDTH HEIGHT PPD TONEMAPPER START|auto STOP|auto COUNT|auto REF.f32 TEST.f32 OUT.bin REPEATS
```

and writes:

| Field | Type |
|---|---|
| Magic `0x464c1737` | u32 |
| Resolved start and stop exposure (zero for LDR) | 2 × f32 |
| Exposure count (zero for LDR) | u32 |
| Median evaluation time in seconds | f64 |
| Mean, weighted median, weighted first and third quartiles, min, max | 6 × f32 |
| Histogram | 100 × u64 |
| Error map | WIDTH × HEIGHT × f32 |
| Exposure map (HDR only) | WIDTH × HEIGHT × f32 |

With `REPEATS` above 1 the driver runs one warm-up first. The timed region is
the `FLIP::evaluate` call, plus the clamp and sRGB conversion for LDR, which
the Rust LDR timing also includes. Restoring the input between repeats is not
timed on either side. Each report records the benchmark executable's hash
before the next feature configuration rebuilds it.

## Stage profile

`profile.py` copies the crate to a temporary directory, inserts timers at fixed
points in `src/lib.rs`, `src/filters.rs` and `src/hdr.rs`, and runs the
benchmark inputs single-threaded:

```sh
python3 parity/profile.py --report profile.md
```

It uses `CARGO_TARGET_DIR` if set. Instrumentation affects timing, so use the
benchmark tables, not the profile, for performance comparisons.

## Randomized sweep

`sweep.sh` generates cases from 1×N, N×1, prime and random dimensions up to
512×512; noise, gradients, edges, constant, near-black, saturated and
logarithmic HDR content; PPD in [1, 200]; all three tone mappers; explicit,
partly automatic and fully automatic exposures, including equal endpoints and
random counts in 2..=128. Each case has its own SplitMix64 seed and can be
replayed:

```sh
./parity/sweep.sh --count 1 --case-seed SEED
```

The first six cases probe inputs on which the reference is undefined. Every
case runs the C++ driver, including those Rust rejects, and the report records
both sides. The driver exits with code 3 if the reference produced nonfinite
error pixels, before its histogram would perform an undefined float-to-integer
conversion; other reference failures keep their exit code and message. Inputs
rejected only by Rust's resource limits (more than 128 exposures, oversized
kernels) are covered by API tests and fuzzing, not by the sweep.

Pass criteria are the same as for the corpus. `results/sweep.md` records the
maxima with their case seeds (seed 0 when every difference is zero), each
undefined-input case and any outliers. CI runs 200 cases against the pinned
reference.
