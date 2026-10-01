# Hostile-input fuzzing

The five cargo-fuzz targets exercise LDR/HDR comparisons, pooling, Magma
expansion and both public image loaders (`image` enabled). A panic or abort is
a crash; ordinary `FlipError` results are accepted. Dimensions include zeros,
`usize::MAX` and the scalar-map size whose RGB expansion exceeds the wasm32
byte limit. Successful buffers are capped at 8×8 so malformed dimensions never
make the harness itself allocate huge vectors. Float values are arbitrary bit
patterns, including NaNs, infinities, denormals and extreme finite values.

Install nightly and cargo-fuzz, then use a single externally selected target
directory for all builds. For each of `ldr_flip`, `hdr_flip`, `pooling`,
`colorize`, and `image_loaders`:

```sh
cargo +nightly fuzz run ldr_flip -- -max_total_time=310 -seed=3572951 -max_len=4096 -print_final_stats=1
```

Run for at least 300 seconds per target. `-seed` fixes libFuzzer's starting
random state; mutations can still vary with execution speed and libFuzzer
version. Use valid PNG/EXR seeds for loader decode coverage. Generated corpus,
coverage and crash artifacts are ignored; add any minimized crashing input to
regression tests after fixing it. See [the measured runs](../parity/results/fuzz.md).

The stable arithmetic regression in `src/lib.rs` checks the exact wasm32
`isize::MAX` byte boundary without allocating the reviewed 716 MB scalar map.
`dimensions` checks both pixel and channel multiplications before allocation;
`colorize`, gray-RGB saving, workspace layout and wasm RGBA conversion all use
that helper. Filter tap counts are capped before conversion to integers, and
the image-dependent work estimate uses checked `u64` multiplication so the
2^34 limit is valid even when `usize` is 32 bits.
