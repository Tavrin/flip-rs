# Images

The images in this directory are used by the top-level README and the crate
documentation. They are not part of the published crate.

| File | Content | Source |
|---|---|---|
| `hero.webp` | Reference, test and LDR-FLIP heatmap | `examples/showcase.rs` |
| `hdr-exposures.webp` | HDR reference at four exposures, the error at each, and the HDR-FLIP result | `examples/showcase.rs` |
| `benchmarks.svg` | Timings from `parity/results/benchmarks-single.md` and `benchmarks-parallel.md` | `generate.py` |
| `demo.webp` | Screenshot of the browser demo in `examples/web/` with its sample pair | Headless Chromium |

All scenes are procedural and were made for this repository. They contain no
third-party images, and they are covered by the repository's BSD-3-Clause
licence. The demo's sample pair, `examples/web/sample-reference.png` and
`examples/web/sample-test.png`, comes from the same example.

NVIDIA's example images in the upstream repository (`images/reference.png`,
`test.png` and the EXR pair) are used by the parity harness but not
reproduced here. The upstream README places "this work" under BSD-3-Clause
but does not state where the rendered scenes come from or whether their
content is covered, so they were not redistributed.

The two WebP figures are lossy (quality 90) to keep them small; the heatmaps
in them are therefore close to, but not exactly, the crate's output.

## Regenerating

From the repository root, with cargo and Pillow (built with WebP support):

```sh
python3 docs/img/generate.py
```

This rewrites `hero.webp`, `hdr-exposures.webp`, `benchmarks.svg` and the
demo's sample pair. The output is deterministic for a given toolchain.

`demo.webp` was captured with Playwright's Chromium at 1100×640 after building
the demo as described in the README's WebAssembly section and waiting for the
statistics line to appear; it was then converted to WebP at quality 88.
