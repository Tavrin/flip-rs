# Randomized parity sweep

Seed: `3572951` (SplitMix64); cases: **3000**; parity passes: **2974**; documented undefined cases: **26**; failures: **0**.

LDR: 1514; HDR: 1486; one-dimensional: 423; maximum width/height: 511 / 512.

Reference: `$FLIP_RS_REFERENCE`, pinned `b475eb4bf394ab877c42166c9eb0a84a02cc5b14`; clean checkout required. Inputs are shared raw f32 buffers. Gates: pixels <= 1e-5, pooled/exposure/endpoints <= 1e-6, exact histograms and exposure counts.

| Difference | Maximum | Case seed |
|---|---:|---:|
| Pixel | 0.000000000e0 | 0 |
| Pooled | 0.000000000e0 | 0 |
| Exposure map | 0.000000000e0 | 0 |
| Weighted percentile | 0.000000000e0 | 0 |
| Unweighted percentile | 0.000000000e0 | 0 |

## Percentile parity

Fractions: 0, 0.01, 0.25, 0.5, 0.75, 0.9, 0.95, 0.99, 0.999, and the largest defined f32 below 1 for each weighting and map size. Unweighted queries with out-of-bounds indices are checked for Rust errors; C++ is not called at those indices.

| Weighting | Bit-identical queries on C++ maps | Undefined queries rejected | Max difference between Rust and C++ maps |
|---|---:|---:|---:|
| Weighted | 29740 | 0 | 0.000000000e0 |
| Unweighted | 26457 | 3283 | 0.000000000e0 |

Replay a random case with `./parity/sweep.sh --count 1 --case-seed SEED`; replay dedicated degeneracies with the full seed and count >= 6.

## Undefined reference inputs

C++ exit 3 is a driver guard after the actual reference calculation found nonfinite pixels, before undefined float-to-integer histogram conversion. C++ exit 255 is the upstream exit(-1). Finite zero maps do not make a division by zero or zero-exposure comparison defined.

| Index | Case seed | Justification | Rust error | Observed C++ behavior |
|---|---:|---|---|---|
| 0 | 15649468885274262578 | zero feature-filter normalizers divide by zero; NaN intermediates may be masked | PPD produces degenerate reference filters | exit 0, max error 3.926850855e-1, resolved None |
| 1 | 3095154588644675688 | black auto reference has infinite start and reversed endpoints | exposure endpoints must be finite and ordered (an all-black reference needs explicit endpoints) | exit Some(255): Start exposure must be smaller than stop exposure! |
| 2 | 16490600500492477839 | epsilon-clamped median makes auto endpoints reversed | exposure endpoints must be finite and ordered (an all-black reference needs explicit endpoints) | exit Some(255): Start exposure must be smaller than stop exposure! |
| 3 | 8824245686364435798 | reversed endpoints: upstream exits | exposure endpoints must be finite and ordered (an all-black reference needs explicit endpoints) | exit Some(255): Start exposure must be smaller than stop exposure! |
| 4 | 7906791845423420660 | one exposure divides by zero; NaN intermediate values are hidden by max selection | exposure count must be between 2 and 128 | exit 0, max error 1.175494351e-38, resolved Some((0.0, 0.0, 1)) |
| 5 | 13877880966660990067 | zero exposures evaluate no image; not a defined HDR comparison | exposure count must be between 2 and 128 | exit 0, max error 1.175494351e-38, resolved Some((0.0, 0.0, 0)) |
| 108 | 11904993994250810263 | feature-filter normalizers underflow to zero | PPD produces degenerate reference filters | exit 0, max error 4.752037227e-1, resolved None |
| 382 | 9101930839216986354 | automatic exposure endpoints are nonfinite or reversed; upstream exits | exposure endpoints must be finite and ordered (an all-black reference needs explicit endpoints) | exit Some(255): Start exposure must be smaller than stop exposure! |
| 790 | 16134619950338725600 | automatic exposure endpoints are nonfinite or reversed; upstream exits | exposure endpoints must be finite and ordered (an all-black reference needs explicit endpoints) | exit Some(255): Start exposure must be smaller than stop exposure! |
| 814 | 6952883034662866993 | automatic exposure endpoints are nonfinite or reversed; upstream exits | exposure endpoints must be finite and ordered (an all-black reference needs explicit endpoints) | exit Some(255): Start exposure must be smaller than stop exposure! |
| 967 | 5940468195539047412 | automatic exposure endpoints are nonfinite or reversed; upstream exits | exposure endpoints must be finite and ordered (an all-black reference needs explicit endpoints) | exit Some(255): Start exposure must be smaller than stop exposure! |
| 1068 | 9829694179347947908 | feature-filter normalizers underflow to zero | PPD produces degenerate reference filters | exit 0, max error 4.294700325e-1, resolved None |
| 1074 | 6214316352306976098 | automatic exposure endpoints are nonfinite or reversed; upstream exits | exposure endpoints must be finite and ordered (an all-black reference needs explicit endpoints) | exit Some(255): Start exposure must be smaller than stop exposure! |
| 1465 | 9925699827571981145 | automatic exposure endpoints are nonfinite or reversed; upstream exits | exposure endpoints must be finite and ordered (an all-black reference needs explicit endpoints) | exit Some(255): Start exposure must be smaller than stop exposure! |
| 1477 | 2734874165146654519 | feature-filter normalizers underflow to zero | PPD produces degenerate reference filters | exit 0, max error 5.018180013e-1, resolved None |
| 2144 | 5010940923256645761 | feature-filter normalizers underflow to zero | PPD produces degenerate reference filters | exit 0, max error 7.782049775e-1, resolved Some((-5.2220497, -3.6473455, 5)) |
| 2172 | 8992704840376339644 | feature-filter normalizers underflow to zero | PPD produces degenerate reference filters | exit 0, max error 1.577955438e-3, resolved Some((-5.8721676, -2.485935, 64)) |
| 2196 | 15321742489428436198 | automatic exposure endpoints are nonfinite or reversed; upstream exits | exposure endpoints must be finite and ordered (an all-black reference needs explicit endpoints) | exit Some(255): Start exposure must be smaller than stop exposure! |
| 2232 | 10942632630381161742 | feature-filter normalizers underflow to zero | PPD produces degenerate reference filters | exit 0, max error 7.585310340e-1, resolved Some((-5.068951, -4.178854, 2)) |
| 2549 | 7838365002635659638 | feature-filter normalizers underflow to zero | PPD produces degenerate reference filters | exit 0, max error 2.741682231e-1, resolved None |
| 2562 | 13630971795698758147 | automatic exposure endpoints are nonfinite or reversed; upstream exits | exposure endpoints must be finite and ordered (an all-black reference needs explicit endpoints) | exit Some(255): Start exposure must be smaller than stop exposure! |
| 2613 | 1530906272019048687 | feature-filter normalizers underflow to zero | PPD produces degenerate reference filters | exit 0, max error 4.665400088e-1, resolved None |
| 2740 | 17007108830690619109 | feature-filter normalizers underflow to zero | PPD produces degenerate reference filters | exit 0, max error 5.523818731e-1, resolved Some((-40.0, 6.881642, 8)) |
| 2847 | 16145492563721159679 | feature-filter normalizers underflow to zero | PPD produces degenerate reference filters | exit 0, max error 1.410693675e-1, resolved None |
| 2896 | 10139891603056455469 | feature-filter normalizers underflow to zero | PPD produces degenerate reference filters | exit 0, max error 3.383930027e-1, resolved None |
| 2956 | 11650785318084195884 | feature-filter normalizers underflow to zero | PPD produces degenerate reference filters | exit 0, max error 6.917459369e-1, resolved Some((-12.91751, -2.2190409, 8)) |

## Outliers

None above the gates. The maximum-difference case seeds are recorded above.

## Qualification binding

Rust base revision: `bade279d30354848dc0446f0b613b2cb252fb863`; measurements use the working sources. The SHA-256 snapshot below binds those exact sources.

- `Cargo.lock`: `c9db872017986afc4b3040519edee341717e55decff19405f34b18019b431757`
- `Cargo.toml`: `9a78bb74085b10eaab96eea081a1b180608295dbf8595249509b4d483f7ef0c9`
- `examples/compare.rs`: `07d1c05639f15e2e8bb1bb62c163f1826a4f47dd338a34291f9e9bce0a356c11`
- `examples/parity.rs`: `2c09afbaa15112ac03d23382a1c4443bfdb041a89a0bfa52fb5e1f9947ca7e30`
- `examples/showcase.rs`: `e4b1154d5f86c6252d0d0133f1b6dc7659a04d6ecf507b46258f5ddf4d94c353`
- `examples/support/mod.rs`: `a15f6b928acf21582cb67ee516875f13b8502adc763ed7268a960552db6f751e`
- `examples/sweep/mod.rs`: `3bfc0c22e93996f25e30ed183e25435de3a1e36c566e76313a4c9846b70d0087`
- `parity/check-reference.sh`: `018f5c9de08a5509fe3e90884e8dca9f6ff3da32d5bc2c8b06e98df26f4b5c0a`
- `parity/record.py`: `2db84a40fd3137b7543621e246f1bc08a21ed691e096e142df61dc23a03f887a`
- `parity/reference.cpp`: `a6e4c8b0f5ecaea384640e8776a75cbf9eba2e8fe79926f5b87825ac57216eef`
- `parity/run.sh`: `2af5b83ce12173439766ad135ba0ae3a0f6d7a3e18d0ba23c7b8c9c3001d1bc9`
- `parity/sweep.sh`: `4937418468a0c98aebcfa7bc441e8a2cd106186edab9e1712ca814f9bcddfec5`
- `src/color.rs`: `b34d4d6266cbc166b81d58b354d5cd1c584abda530ac76477a67a429d93b69c1`
- `src/filters.rs`: `b5dc598c4e65613091c3f97e39be1b5eff7f26d8ba40433c8746a6c14609a7b7`
- `src/hdr.rs`: `828bbeceb24fa7f685bc6bde8c442e1d517f2e3cd02df1e752ae1705ebd5ea44`
- `src/io.rs`: `51c7e129202d7d7ddfcd1822d0b4d1bb2dda02952f1486be0504187638010e8d`
- `src/lib.rs`: `fd0f73dd3593abbc25d54908ca320d93eb653e6ba27283ced1f4bd1a51240933`
- `src/magma.rs`: `59340581c70404490a1b77fe932e2a0ddbe387ceaccc8c673ea5db95e6f7cf97`
- `src/pooling.rs`: `9fe4733a7004585388a12210d53b5685a0febf84eb107e53a802d62047ad6d2b`
- `src/wasm.rs`: `d17331686f40ee277ea1b557d57a5410fab687e8ed451e51c5825d8e56d1276a`
- `tests/api.rs`: `31989dea6f1b3f94473c211ef8878b8c1c91ebdcb0c65c4be684ca6f89160371`
- `tests/parity.rs`: `dc9665877fb3cb5af57faa4f80bfb111cb071660cf13b2817d9037fffe9b5fa1`
- Rust parity/benchmark executable SHA-256: `c82ce1e870d65a07276f5a9079b63e94d3837de1ddc430d547d7859c42fe89d1`
- C++ oracle executable SHA-256: `8bfe9a90f72d431fd2acf969e7c0584dc3cb29556c69e886b773b3280844cedf`
- rustc: `rustc 1.98.1 (48a229cea 2026-09-01)`
- cargo: `cargo 1.98.1 (797e8a9bc 2026-08-05)`
- g++: `13.3.0`
