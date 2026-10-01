# flip-rs

A Rust port of [NVIDIA FLIP](https://github.com/NVlabs/flip) v1.7, a
perceptual metric for the difference a viewer sees when flipping between a
reference image and a test image. FLIP produces a per-pixel error map, normally
in 0..1, which can be pooled into statistics or shown as a Magma heatmap. It
handles both LDR (sRGB) and HDR (linear RGB) images.

This crate is an independent port and is not affiliated with or endorsed by
NVIDIA. FLIP was designed by the authors of the papers listed under
[References](#references); the reference implementation is NVIDIA's.

The library has no C or C++ dependencies and no `unsafe` code. With default
features disabled it has no dependencies at all and builds for
`wasm32-unknown-unknown`.

## Parity with the C++ reference

Every case in a measured 201-case corpus is compared against NVIDIA's C++
implementation at revision `b475eb4` (v1.7):

| Corpus | Cases | Max pixel difference | Max pooled difference | Max exposure-map difference |
|---|---:|---:|---:|---:|
| Generated HDR | 157 | 0 | 0 | 0 |
| Generated LDR | 21 | 0 | 0 | 0 |
| Reference EXR | 18 | 0 | 0 | 0 |
| Reference PNG | 5 | 0 | 0 | 0 |

Pooled statistics (mean, weighted quartiles, minimum, maximum), histogram
counts and exposure maps match exactly, and the automatically chosen exposure
counts are equal. The randomized sweep additionally checks 3,000 seeded cases, including
narrow and prime dimensions, random PPD and HDR exposure degeneracies. Its
statistics and replay seeds are in [the sweep report](parity/results/sweep.md).
Measured agreement applies to these inputs and toolchains. The harness fails
a case above 1e-5 per pixel, 1e-6 for pooled values, exposure maps and exposure
endpoints, or on any histogram or exposure-count mismatch. The feature
exponent preserves the reference `powf(x, 0.5)` operation because substituting
`sqrt` can change the winning exposure on HDR near-ties.

The corpus covers all of the upstream example images, flat colors, gradients,
edges, seeded noise, sizes from 1×1 to 1930×1080, all-black and mostly-black
HDR images, values from 1e-20 to 1e30, PPD values of 20, 67.02 and 120, all
three tone mappers, and explicit, automatic and partly automatic exposures.
See [Reproducing the parity results](#reproducing-the-parity-results).

## Performance

Median of three runs after one warm-up, on seeded noise at the default PPD.
HDR uses ACES with three exposures. Timings cover the comparison call
(color conversion, tone mapping, filtering and allocation); decoding and
pooling are excluded. The tables were refreshed after the hardening fixes,
with C++ input restoration outside timing and fuzzing finished.

| Case | C++, 1 thread (s) | Rust, 1 thread (s) | C++ / Rust | Rust, 8 threads (s) |
|---|---:|---:|---:|---:|
| LDR 1920×1080 | 0.697189 | 0.346759 | 2.01× | 0.203153 |
| HDR 1920×1080 | 2.069816 | 0.798613 | 2.59× | 0.189725 |
| LDR 3840×2160 | 3.145998 | 1.452090 | 2.17× | 0.603286 |
| HDR 3840×2160 | 8.251707 | 3.378650 | 2.44× | 1.198722 |

Measured on an AMD Ryzen 9 7945HX (16 cores) under Linux, with Rust 1.98.1 and
GCC 13.3, using portable code generation for both (no `-march=native`, fast
math or FMA). The C++ column is the reference built with `g++ -O2 -std=c++17`
and OpenMP disabled. The upstream CMake build uses `-O3` and OpenMP, so the
upstream tool on several cores is faster than this column; multi-threaded C++
was not measured, and the 8-thread Rust column has no C++ counterpart. Your
results will depend on hardware, image size, PPD and exposure count.

The single-thread difference comes from the filter loops. They run over
horizontally padded planes in 128-pixel tiles, which LLVM vectorizes across
pixels while keeping each pixel's summation order the same as the reference.

## Usage

```toml
[dependencies]
flip-rs = "0.1"
```

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

HDR images are linear RGB `f32`. Exposure endpoints and count are chosen from
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

`ppd` is the number of pixels per degree of visual angle. The default,
`DEFAULT_PPD` (about 67.02), is the reference tool's: a 0.7 m wide 3840-pixel
monitor viewed from 0.7 m. `pixels_per_degree` computes it for other setups.

The `compare` example evaluates two files and writes a heatmap:

```sh
cargo run --release --features image --example compare -- reference.png test.png heatmap.png
cargo run --release --features image --example compare -- reference.exr test.exr heatmap.png 67.02 hable
```

## Features

| Feature | Default | Effect |
|---|---|---|
| `parallel` | yes | Processes image rows in parallel with Rayon. Results are identical with or without it. |
| `image` | no | Adds the `io` module: PNG and OpenEXR loading and saving through the `image` crate. |
| `wasm` | no | Exports `ldrFlip` through wasm-bindgen. |

The minimum supported Rust version is 1.88.

## WebAssembly

The core library builds for `wasm32-unknown-unknown` with
`--no-default-features`. On `wasm32`, the `wasm` feature adds one JavaScript export,
`ldrFlip(referenceRgba, testRgba, width, height, ppd)`. It takes RGBA bytes
such as `ImageData.data`, ignores alpha, returns a `Float32Array` with one
error per pixel, and throws an `Error` for invalid input.

To run the browser demo in `examples/web/`:

```sh
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown --no-default-features --features wasm
wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/release/flip_rs.wasm
python3 -m http.server 8080   # then open http://localhost:8080/examples/web/
```

The `wasm-bindgen` CLI version must match the `wasm-bindgen` crate version in
`Cargo.lock`.

## Reproducing the parity results

The harness in `parity/` compiles a small driver against an unmodified clone
of the reference and compares raw outputs. It is not part of the published
crate.

```sh
git clone https://github.com/NVlabs/flip nvlabs-flip
git -C nvlabs-flip checkout b475eb4bf394ab877c42166c9eb0a84a02cc5b14
FLIP_RS_REFERENCE="$PWD/nvlabs-flip" ./parity/run.sh
```

`run.sh` needs `g++` with C++17. It runs the full corpus, then the single-thread
and 8-thread benchmarks, and writes the tables to `parity/build/`. The recorded
results and test environment are in [`parity/results/`](parity/results/), and
the driver's protocol is described in [`parity/README.md`](parity/README.md).
`cargo test` runs a smaller corpus against the same driver when
`FLIP_RS_PARITY_BIN` points to it:

```sh
FLIP_RS_PARITY_BIN="$PWD/parity/build/reference" cargo test --release --test parity
```

## Behaviour on undefined input

Where v1.7 has no defined result, this crate returns a `FlipError` instead:

- An all-black HDR reference has no automatic start exposure; the C++ tool
  exits. Pass explicit endpoints to compare black images.
- A very small PPD gives zero filter normalizers in the reference, producing
  NaN. It is rejected.
- Zero or overflowing dimensions, mismatched sizes, NaN or infinite input,
  reversed exposure ranges and exposure counts outside 2..=128 are rejected.
- Kernels are limited to 8,193 taps (radius 4,096), and each evaluation to
  2^34 weighted channel additions, calculated from both image size and PPD.
  Full kernels are retained for reference summation order. Excessive work is
  rejected before allocating; internal evaluation buffers use fallible
  reservations and return `FlipError::Allocation` on failure.
- The 128-exposure cap bounds repeated full-image work and accommodates up to
  128 samples across HDR ranges, including the original extreme corpus
  requiring 70 automatic exposures; it is a Rust resource policy, not an
  upstream limit. Equal endpoints evaluate once while keeping the declared
  count and earliest exposure indices.
- `colorize()` returns `Result`, checking the three-channel byte layout and
  reserving fallibly. Raw gray-RGB saving uses the same checks.
- `load_srgb` accepts PNG and assumes its channels are sRGB (no profile
  transformation). `load_linear` accepts only OpenEXR and assumes linear
  channels. Mixed pairs are rejected by the comparison example.

Two reference quirks are kept: the pooled maximum of an all-zero map is
`f32::MIN_POSITIVE`, and errors above 1 in a map built with `ErrorMap::new`
are left out of the histogram. Pooling intentionally accumulates in `f32`;
large externally supplied values can overflow it. `Statistics::finite` flags
that condition; the mean and weighted quartiles then cannot be used, while
extrema and histogram remain valid. Extremely large HDR values can overflow in the
tone curve; the resulting NaN is clamped to 0, as in the reference.

## References

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

If you use FLIP in published work, cite these papers as the
[upstream README](https://github.com/NVlabs/flip#citation) describes.

## License

BSD-3-Clause, the same licence as NVIDIA FLIP. This crate is a derivative of
NVIDIA's implementation, so [LICENSE](LICENSE) carries NVIDIA's copyright notice
as well as the flip-rs contributors'.
