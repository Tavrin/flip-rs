# Hardening changes

Changes made before the first release in response to review of the initial
port, each with a reproduction and the regression test that covers it.

| Library change | Reproduction / regression |
|---|---|
| Validate finite inputs before filters; cap kernels at radius 4,096 and image-dependent work at 2^34 weighted additions; reserve filter/workspace buffers fallibly. | A 1×1 image with PPD `1e10` returns `InvalidParameter`; the same call with NaN channels returns `NonFiniteInput` first. `hostile_ppd_is_bounded_and_input_validation_precedes_filters`; `filter_work_counts_all_vertical_feature_accumulators` also rejects the 11M-pixel, PPD-200 budget boundary without allocating an image. |
| Fallible checked RGB expansion in `colorize()` and raw gray-RGB saving; `colorize()` now returns `Result`. RGB saving and wasm RGBA conversion reserve fallibly too. | 178,956,971 scalar f32 pixels fit the wasm32 byte limit, but three channels require 2,147,483,652 bytes. `rgb_expansion_checks_wasm32_byte_limit_before_reserving` checks that exact boundary without allocating it, plus multiplication and reservation overflow. |
| Cap explicit and automatic exposure counts at 128; evaluate equal endpoints once while preserving the declared count and earliest exposure indices. Automatic luminance and HDR output buffers use fallible reservations. | 1×1, equal zero endpoints, count `i32::MAX` returns an error. Counts 2 and 128 yield identical pixels/indices and preserve their counts. An automatic 160-stop range is rejected. `hdr_workload_is_bounded_and_equal_endpoints_preserve_semantics`. |
| Retain f32 pooling arithmetic and expose `Statistics::finite`. | `[f32::MAX; 2]` flags overflow while retaining reference mean/quartile behavior, extrema and histogram. `pooling_flags_nonfinite_total_without_changing_reference_arithmetic`. |
| `load_linear` accepts only detected OpenEXR; `load_srgb` accepts only detected PNG and documents the sRGB assumption/8-bit quantization. | A PNG is rejected by the linear loader and an EXR by the sRGB loader. `png_and_exr_helpers_round_trip`; the compare example rejects mixed extension pairs before decoding. |
| Gate the wasm module/export by both feature and `target_arch = "wasm32"`. | Native all-features builds expose no JavaScript stubs; native tests and wasm builds pass. |
| Make the feature exponent opaque to LLVM so reference `powf(x, 0.5)` rounding is preserved. | Random seeds `15158193341402106541` and `12092506619092908091` previously differed in exposure index by 0.5 and about 1/3 despite pixel differences of only 2.98e-8. `hdr_near_ties_preserve_reference_exposure_indices` compares both seeds with the oracle. |
| Remove the `io` intra-doc link that did not resolve without the `image` feature; limit numerical claims to the measured inputs. | Docs build under `-D warnings` with default, no default and all features (nightly). |

## Exposure cap

A cap of 64 would reject a valid corpus case:
`hdr-extreme-17x19-20-Aces-autotrue` resolves a 69.74684-stop range and 70
automatic exposures. 128 keeps that case, including input values up to 1e30,
while bounding a count that previously could request billions of full-image
evaluations. The sweep does not classify these Rust limits as reference
undefined behaviour.

## Harness and CI changes

- Added `parity/sweep.sh` with the seeded Rust sweep module
  (`examples/sweep/mod.rs`) and C++ rejection probes.
- Added `parity/check-reference.sh`, which rejects a reference at the wrong
  revision or with local changes (mocked probes exit 1 and 128 before
  building), and `parity/record.py`, which adds source and executable hashes
  to each record.
- C++ benchmark input restoration moved outside the timed region.
- CI runs a 200-case sweep against the pinned reference and builds the docs
  with default and no default features.
- Added the `fuzz/` crate with five targets (`ldr_flip`, `hdr_flip`,
  `pooling`, `colorize`, `image_loaders`) and the records `sweep.md`,
  `fuzz.md` and this file.

Verification results and source hashes are in
[qualification.md](qualification.md), [sweep.md](sweep.md) and
[fuzz.md](fuzz.md).
