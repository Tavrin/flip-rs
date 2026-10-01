# Changelog

All notable changes to this project are documented in this file. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the
project uses [Semantic Versioning](https://semver.org/).

## [0.1.0] - Unreleased

### Added

- LDR-FLIP (`ldr_flip`) for sRGB images with `u8` or `f32` channels.
- HDR-FLIP (`hdr_flip`) for linear RGB images, with the ACES, Hable and
  Reinhard tone mappers and automatic or explicit exposure ranges.
- Pooled statistics and histogram (`ErrorMap::statistics`), Magma heatmaps
  (`ErrorMap::colorize`) and per-pixel exposure maps.
- `parallel` feature (default): row-parallel evaluation with Rayon.
- `image` feature: PNG and OpenEXR loading and saving.
- `wasm` feature: a wasm-bindgen `ldrFlip` export and a browser demo.
- A parity harness against NVIDIA FLIP v1.7 (`b475eb4`) covering 201 cases,
  a reproducible randomized sweep and an oracle-backed CI job.
- Five cargo-fuzz targets and arithmetic regressions for resource limits.

### Changed

- Bound filter kernels/work and HDR exposures; reserve evaluation buffers
  fallibly, and collapse equal-endpoint exposures while preserving results.
- `ErrorMap::colorize` returns `Result`; gray-RGB saving checks expansion too.
- Flag overflowing reference-style pooling with `Statistics::finite`.
- Require PNG for sRGB loading and OpenEXR for linear loading; reject mixed pairs.
- Export wasm bindings only on wasm32 and check docs in all feature modes.
- Preserve reference `powf` rounding so HDR near-ties retain exposure parity.
- Exclude C++ input restoration from benchmark timing and bind measurements
  to source and executable identities.

[0.1.0]: https://github.com/Tavrin/flip-rs/releases/tag/v0.1.0
