# Fuzzing

Five cargo-fuzz targets cover LDR and HDR comparison, pooling, Magma
expansion and both image loaders (with the `image` feature). A panic or abort
is a crash; `FlipError` results are accepted. Dimensions include zeros,
`usize::MAX` and the scalar-map size whose RGB expansion exceeds the wasm32
byte limit. Successful buffers are capped at 8×8 so malformed dimensions never
make the harness itself allocate huge vectors. Float values are arbitrary bit
patterns, including NaNs, infinities, denormals and extreme finite values.

Install nightly and cargo-fuzz, and use one `CARGO_TARGET_DIR` for all
targets. For each of `ldr_flip`, `hdr_flip`, `pooling`, `colorize` and
`image_loaders`:

```sh
cargo +nightly fuzz run ldr_flip -- -max_total_time=310 -seed=3572951 -max_len=4096 -print_final_stats=1
```

Run each target for at least 300 seconds. `-seed` fixes libFuzzer's initial
random state, but mutations still depend on execution speed and libFuzzer
version. Add valid PNG and EXR files to the loader corpus to reach the
decoders. The corpus and crash artifacts are not tracked; after fixing a
crash, add the minimized input as a regression test. The last recorded run is
in [`parity/results/fuzz.md`](../parity/results/fuzz.md).

A regression test in `src/lib.rs`, run by `cargo test`, checks the exact
wasm32 `isize::MAX` byte boundary without allocating the 716 MB scalar map.
`dimensions` checks both pixel and channel multiplications before allocation;
`colorize`, gray-RGB saving, workspace layout and wasm RGBA conversion all use
that helper. Filter tap counts are capped before conversion to integers, and
the image-dependent work estimate uses checked `u64` multiplication so the
2^34 limit is valid even when `usize` is 32 bits.
