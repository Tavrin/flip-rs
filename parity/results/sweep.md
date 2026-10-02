# Randomized parity sweep

Seed: `3572951` (SplitMix64); cases: **3000**; parity passes: **2974**; documented undefined cases: **26**; failures: **0**.

LDR: 1514; HDR: 1486; one-dimensional: 423; maximum width/height: 511 / 512.

Reference: `$FLIP_RS_REFERENCE`, pinned `b475eb4bf394ab877c42166c9eb0a84a02cc5b14`; clean checkout required. Inputs are shared raw f32 buffers. Gates: pixels <= 1e-5, pooled/exposure/endpoints <= 1e-6, exact histograms and exposure counts.

| Difference | Maximum | Case seed |
|---|---:|---:|
| Pixel | 0.000000000e0 | 0 |
| Pooled | 0.000000000e0 | 0 |
| Exposure map | 0.000000000e0 | 0 |

Replay a random case with `./parity/sweep.sh --count 1 --case-seed SEED`. The first six cases are fixed probes; replay them with `--seed 3572951` and a count of at least 6. A case seed of 0 in the table means every difference was zero.

## Undefined reference inputs

C++ exit 3 is the driver's guard: the reference produced nonfinite pixels, and the run stops before the histogram's undefined float-to-integer conversion. Exit 255 is the reference's own `exit(-1)`. A case where the reference exits 0 with a finite map is still undefined if it divided by zero or evaluated no exposures.

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

None.

## Sources and toolchain

Rust base revision `5f4d5c29a0dc40fed2bda8e1d6f6ccc1772d412e` plus uncommitted changes. The SHA-256 hashes below identify the measured sources and executables.

- `Cargo.lock`: `4e4fc4ff20f311e0daf9a9745c8d8d3e0a9dee320be259961936d22ea6e4d0b3`
- `Cargo.toml`: `2a7b44f5dbb2086373b00a6d3745097e38573b5ce416baa7539ae204e66c185e`
- `examples/compare.rs`: `07d1c05639f15e2e8bb1bb62c163f1826a4f47dd338a34291f9e9bce0a356c11`
- `examples/parity.rs`: `6825a0c86ca186482ee7130dbf4a6cf5b8686ae236abe8ab7bdeb993669dd434`
- `examples/support/mod.rs`: `c2d15ee9220af7331f1db2b628b13244a13704cd5bac6d0beb99e36ea23d78fa`
- `examples/sweep/mod.rs`: `000b31b50fb2119d4775d4e16dd7a8c3c3c59a5f332369b9a69945e91ec84642`
- `parity/check-reference.sh`: `018f5c9de08a5509fe3e90884e8dca9f6ff3da32d5bc2c8b06e98df26f4b5c0a`
- `parity/reference.cpp`: `d668adc55432eff523460a84cb2a0735c40ce28b9107f76dc70599d7c7a5eeeb`
- `parity/run.sh`: `2af5b83ce12173439766ad135ba0ae3a0f6d7a3e18d0ba23c7b8c9c3001d1bc9`
- `parity/sweep.sh`: `4937418468a0c98aebcfa7bc441e8a2cd106186edab9e1712ca814f9bcddfec5`
- `src/color.rs`: `b34d4d6266cbc166b81d58b354d5cd1c584abda530ac76477a67a429d93b69c1`
- `src/filters.rs`: `b5dc598c4e65613091c3f97e39be1b5eff7f26d8ba40433c8746a6c14609a7b7`
- `src/hdr.rs`: `828bbeceb24fa7f685bc6bde8c442e1d517f2e3cd02df1e752ae1705ebd5ea44`
- `src/io.rs`: `51c7e129202d7d7ddfcd1822d0b4d1bb2dda02952f1486be0504187638010e8d`
- `src/lib.rs`: `250f28aaed2e80159ca26b462ffd5cced2bfc56254119440b92893d78d5c67fc`
- `src/magma.rs`: `59340581c70404490a1b77fe932e2a0ddbe387ceaccc8c673ea5db95e6f7cf97`
- `src/pooling.rs`: `c20e188db741edbb7bc8528ded9bc788d1538748ab8d376aa86d31266a6558db`
- `src/wasm.rs`: `d17331686f40ee277ea1b557d57a5410fab687e8ed451e51c5825d8e56d1276a`
- Rust parity/benchmark executable SHA-256: `3090a009359280b5a40e572edc2284b6cadf29698ebaf2d75fe3f21ad2c6e30e`
- C++ oracle executable SHA-256: `a64f0aec7aeb2f093b2564f3beeb2df07cdd8103a16a7a6e5f029847f39ffd25`
- rustc: `rustc 1.98.1 (48a229cea 2026-09-01)`
- cargo: `cargo 1.98.1 (797e8a9bc 2026-08-05)`
- g++: `13.3.0`

## Fixed near-tie outliers

In an earlier sweep, case seeds `15158193341402106541` and
`12092506619092908091` selected different HDR exposure indices (differences
of 0.5 and about 1/3) while pixel differences were only 2.9802322e-8. Keeping
the reference's `powf(x, 0.5)` instead of `sqrt` fixed both. They pass in this
sweep, are oracle regression tests, and no tolerance was changed.
