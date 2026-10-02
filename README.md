# flip-rs

A Rust port of [NVIDIA FLIP](https://github.com/NVlabs/flip) v1.7, a
perceptual metric for the difference a viewer sees when flipping between a
reference and a test image. It computes LDR-FLIP for sRGB images and HDR-FLIP
for linear RGB images, producing a per-pixel error map that can be pooled into
statistics or rendered as a Magma heatmap.

The library has no C or C++ dependencies and no `unsafe` code. With default
features disabled it has no dependencies and builds for
`wasm32-unknown-unknown`.

## Installation

```toml
[dependencies]
flip-rs = "0.1"
```

The minimum supported Rust version is 1.88.

## Usage

LDR images are sRGB, as `u8` or as `f32` in 0..=1:

```rust
use flip_rs::{ldr_flip, RgbImage, DEFAULT_PPD};

fn main() -> Result<(), flip_rs::FlipError> {
    // 2×1 images, channels interleaved as R, G, B, R, G, B.
    let reference = RgbImage::new(2, 1, vec![128_u8, 64, 32, 255, 255, 255])?;
    let test = RgbImage::new(2, 1, vec![130_u8, 60, 32, 250, 255, 255])?;

    let error_map = ldr_flip(&reference, &test, DEFAULT_PPD)?;
    let stats = error_map.statistics();
    println!("mean {}, weighted median {}", stats.mean, stats.weighted_median);

    let heatmap = error_map.colorize()?; // sRGB floats, Magma colormap
    assert_eq!(heatmap.pixels().len(), 6);
    Ok(())
}
```

HDR images are linear RGB `f32`. The exposure range and count are derived from
the reference image unless you set them:

```rust
use flip_rs::{hdr_flip, HdrOptions, RgbImage, Tonemapper};

fn main() -> Result<(), flip_rs::FlipError> {
    let reference = RgbImage::new(1, 1, vec![1.0_f32, 2.0, 3.0])?;
    let test = RgbImage::new(1, 1, vec![1.1_f32, 1.9, 2.8])?;

    let mut options = HdrOptions::default();
    options.tonemapper = Tonemapper::Hable;
    let result = hdr_flip(&reference, &test, options)?;
    println!("{:?}", result.parameters);
    Ok(())
}
```

`ppd` is the number of pixels per degree of visual angle. `DEFAULT_PPD`
(about 67.02) is the reference tool's default: a 0.7 m wide, 3840-pixel
monitor viewed from 0.7 m. `pixels_per_degree` computes it for other setups.

The `compare` example compares two files and writes a heatmap:

```sh
cargo run --release --features image --example compare -- reference.png test.png heatmap.png
cargo run --release --features image --example compare -- reference.exr test.exr heatmap.png 67.02 hable
```

## Features

| Feature | Default | Effect |
|---|---|---|
| `parallel` | yes | Processes image rows in parallel with Rayon. Results are identical with or without it. |
| `image` | no | Adds the `io` module: PNG and OpenEXR loading and saving through the `image` crate. |
| `wasm` | no | On `wasm32`, exports `ldrFlip` through wasm-bindgen. |

## Parity with the C++ reference

The harness in [`parity/`](parity/README.md) runs NVIDIA's C++ implementation
at revision `b475eb4` (v1.7) and this crate on the same `f32` inputs. On the
201-case corpus, every output matched exactly:

| Corpus | Cases | Max pixel difference | Max pooled difference | Max exposure-map difference |
|---|---:|---:|---:|---:|
| Generated HDR | 157 | 0 | 0 | 0 |
| Generated LDR | 21 | 0 | 0 | 0 |
| Reference EXR | 18 | 0 | 0 | 0 |
| Reference PNG | 5 | 0 | 0 | 0 |

Pooled statistics, histogram counts, exposure maps and the automatically
chosen exposure ranges and counts are also equal. A seeded randomized sweep of
3,000 cases (seed `3572951`) found no differences in its 2,974 defined cases;
the other 26 are inputs on which the reference is undefined, listed in
[the sweep report](parity/results/sweep.md). These results apply to the
measured inputs and toolchains. The harness fails a case above 1e-5 per pixel
or 1e-6 for pooled values, exposure maps and exposure endpoints, or on any
histogram or exposure-count mismatch.

The corpus covers the upstream example images, flat colors, gradients, edges
and seeded noise from 1×1 to 1024×1024, all-black and mostly-black HDR images,
values from 1e-20 to 1e30, PPD 20, 67.02 and 120, all three tone mappers, and
explicit, automatic and partly automatic exposures.

## Performance

Median of three runs after one warm-up, on seeded noise at the default PPD.
HDR uses ACES with three exposures. Times cover the comparison call (color
conversion, tone mapping, filtering and allocation), not decoding or pooling.

| Case | C++, 1 thread (s) | Rust, 1 thread (s) | C++ / Rust | Rust, 8 threads (s) |
|---|---:|---:|---:|---:|
| LDR 1920×1080 | 0.697189 | 0.346759 | 2.01× | 0.203153 |
| HDR 1920×1080 | 2.069816 | 0.798613 | 2.59× | 0.189725 |
| LDR 3840×2160 | 3.145998 | 1.452090 | 2.17× | 0.603286 |
| HDR 3840×2160 | 8.251707 | 3.378650 | 2.44× | 1.198722 |

Measured on an AMD Ryzen 9 7945HX (16 cores) under Linux with Rust 1.98.1 and
GCC 13.3, without `-march=native`, fast math or FMA. The C++ column is the
reference built with `g++ -O2 -std=c++17` and OpenMP disabled. NVIDIA's CMake
build uses `-O3` and OpenMP, so the upstream tool will be faster than this
column, especially on several cores. Multi-threaded C++ was not measured; the
8-thread Rust column has no C++ counterpart. Records:
[single-threaded](parity/results/benchmarks-single.md),
[8 threads](parity/results/benchmarks-parallel.md),
[environment](parity/results/qualification.md).

Most of the single-thread difference comes from the filter loops, which run
over horizontally padded planes in 128-pixel tiles. LLVM vectorizes them
across pixels while each pixel keeps the reference's summation order.

## WebAssembly

The core library builds for `wasm32-unknown-unknown` with
`--no-default-features`. On `wasm32`, the `wasm` feature adds one JavaScript
export, `ldrFlip(referenceRgba, testRgba, width, height, ppd)`. It takes RGBA
bytes such as `ImageData.data`, ignores alpha, returns a `Float32Array` with
one error per pixel, and throws an `Error` for invalid input.

To run the browser demo in `examples/web/`:

```sh
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown --no-default-features --features wasm
wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/release/flip_rs.wasm
python3 -m http.server 8080   # then open http://localhost:8080/examples/web/
```

The `wasm-bindgen` CLI version must match the `wasm-bindgen` crate version in
`Cargo.lock`.

## Limits and differences from the reference

Where v1.7 has no defined result, this crate returns a `FlipError`:

- An all-black HDR reference has no automatic start exposure, and the C++ tool
  exits. Pass explicit endpoints to compare black images.
- A very small PPD makes the reference's filter normalizers zero, producing
  NaN. It is rejected.
- Zero or overflowing dimensions, mismatched sizes, NaN or infinite input,
  reversed exposure ranges and exposure counts outside 2..=128 are rejected.

Resource limits that the reference does not have:

- Kernels are limited to 8,193 taps (radius 4,096) and each evaluation to
  2^34 weighted channel additions, computed from image size and PPD. Larger
  work is rejected before allocating. Evaluation buffers are reserved
  fallibly and return `FlipError::Allocation` on failure.
- At most 128 exposures. This bounds repeated full-image work and still
  covers the most extreme corpus case, which needs 70 automatic exposures.
  Equal endpoints are evaluated once; the declared count and the earliest
  exposure indices are kept.
- `colorize()` returns `Result`: it checks the size of the three-channel
  output and reserves it fallibly. Saving a raw gray map as RGB does the same.

Other behaviour to be aware of:

- `load_srgb` accepts only PNG and treats its channels as sRGB, with no
  color-profile conversion. `load_linear` accepts only OpenEXR and treats its
  channels as linear. The `compare` example rejects mixed pairs.
- Two reference quirks are kept: the pooled maximum of an all-zero map is
  `f32::MIN_POSITIVE`, and errors above 1 in a map built with `ErrorMap::new`
  are left out of the histogram.
- Pooling accumulates in `f32`, as the reference does, so large externally
  supplied values can overflow it. `Statistics::finite` reports this; the
  mean and weighted quartiles are then unusable, while the extrema and
  histogram remain valid.
- Extremely large HDR values can overflow in the tone curve. The resulting NaN
  is clamped to 0, as in the reference.

## Reproducing the parity results

The harness compiles a small driver against an unmodified clone of the
reference and compares raw outputs. It is not part of the published crate.

```sh
git clone https://github.com/NVlabs/flip nvlabs-flip
git -C nvlabs-flip checkout b475eb4bf394ab877c42166c9eb0a84a02cc5b14
FLIP_RS_REFERENCE="$PWD/nvlabs-flip" ./parity/run.sh
FLIP_RS_REFERENCE="$PWD/nvlabs-flip" ./parity/sweep.sh
```

`run.sh` needs `g++` with C++17. It runs the full corpus and both benchmarks
and writes its tables to `parity/build/`. With `FLIP_RS_PARITY_BIN` pointing
to the compiled driver, `cargo test` also runs a smaller corpus against it:

```sh
FLIP_RS_PARITY_BIN="$PWD/parity/build/reference" cargo test --release --test parity
```

See [`parity/README.md`](parity/README.md) for the driver protocol and pass
criteria, and [`parity/results/`](parity/results/) for the recorded results.

## Licence and credit

BSD-3-Clause, the licence of NVIDIA FLIP. This crate is derived from NVIDIA's
implementation, so [LICENSE](LICENSE) carries NVIDIA's copyright notice as
well as that of the flip-rs contributors. flip-rs is an independent port; it
is not affiliated with or endorsed by NVIDIA.

FLIP was designed by the authors of these papers. If you use FLIP in published
work, cite them as the [upstream README](https://github.com/NVlabs/flip#citation)
describes.

- Pontus Andersson, Jim Nilsson, Tomas Akenine-Möller, Magnus Oskarsson, Kalle
  Åström and Mark D. Fairchild.
  [FLIP: A Difference Evaluator for Alternating Images](https://research.nvidia.com/publication/2020-07_FLIP).
  Proceedings of the ACM on Computer Graphics and Interactive Techniques 3(2),
  2020 (HPG 2020).
- Pontus Andersson, Jim Nilsson, Peter Shirley and Tomas Akenine-Möller.
  [Visualizing Errors in Rendered High Dynamic Range Images](https://research.nvidia.com/publication/2021-05_HDR-FLIP).
  Eurographics 2021 Short Papers.
- Pontus Andersson, Jim Nilsson and Tomas Akenine-Möller. Visualizing and
  Communicating Errors in Rendered Images. Ray Tracing Gems II, 2021.
