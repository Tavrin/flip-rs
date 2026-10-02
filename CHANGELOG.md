# Changelog

All notable changes to this project are documented in this file. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the
project uses [Semantic Versioning](https://semver.org/).

## [0.1.0] - 2026-10-02

First release.

### Added

- LDR-FLIP (`ldr_flip`) for sRGB images with `u8` or `f32` channels.
- HDR-FLIP (`hdr_flip`) for linear RGB images, with the ACES, Hable and
  Reinhard tone mappers and automatic or explicit exposure ranges.
- Pooled statistics and histogram (`ErrorMap::statistics`), Magma heatmaps
  (`ErrorMap::colorize`) and per-pixel exposure maps.
- `parallel` feature (default): row-parallel evaluation with Rayon.
- `image` feature: PNG and OpenEXR loading and saving.
- `wasm` feature: a wasm-bindgen `ldrFlip` export and a browser demo.
- A parity harness against NVIDIA FLIP v1.7 (`b475eb4`) with a 201-case
  corpus, a seeded randomized sweep, and a CI job that runs 200 sweep cases
  against the pinned reference.
- Five cargo-fuzz targets.

### Changed

These changes were made before the first release, after review of the
initial port. Some change behaviour that earlier development builds had.

- **Breaking:** `ErrorMap::colorize` returns `Result`. It checks the size of
  the RGB output and reserves it fallibly. Saving a gray map as RGB does the
  same.
- Filter kernels are limited to radius 4,096 and each evaluation to 2^34
  weighted additions. Input is checked for NaN and infinity before filtering,
  and evaluation buffers are reserved fallibly (`FlipError::Allocation`).
- HDR exposure counts, explicit or automatic, are limited to 2..=128. Equal
  endpoints are evaluated once, with the same results.
- `Statistics::finite` reports when `f32` pooling overflows. Pooling
  arithmetic is unchanged.
- `io::load_srgb` accepts only PNG and `io::load_linear` only OpenEXR. The
  `compare` example rejects mixed pairs.
- The wasm module and its export are compiled only for `wasm32` with the
  `wasm` feature, so native builds with all features have no JavaScript stubs.
- The feature filter keeps the reference's `powf(x, 0.5)` rounding, which
  fixes two HDR cases where a near-tie selected a different exposure.
- Benchmarks exclude C++ input restoration from timing, and the recorded
  results include source and executable hashes.

[0.1.0]: https://github.com/Tavrin/flip-rs/releases/tag/v0.1.0
