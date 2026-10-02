# Benchmarks, 8-thread Rust

Rust with the `parallel` feature and `RAYON_NUM_THREADS=8`; the C++ driver is single-threaded and built with `g++ -O2 -std=c++17` without OpenMP, so the ratio column is not a like-for-like comparison. Median of three runs after one warm-up. Hardware and method: [qualification.md](qualification.md).

| Case | C++ -O2, 1 thread (s) | Rust, 8 threads (s) | C++ 1 thread / Rust 8 threads |
|---|---:|---:|---:|
| LDR 1920×1080 | 0.749002 | 0.203153 | 3.69× |
| HDR ACES, 3 exposures 1920×1080 | 2.204883 | 0.189725 | 11.62× |
| LDR 3840×2160 | 3.587212 | 0.603286 | 5.95× |
| HDR ACES, 3 exposures 3840×2160 | 8.499755 | 1.198722 | 7.09× |

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
- Rust parity/benchmark executable SHA-256: `cb15b50f1848879a521679b191e5eb38ec2e225221f0713b467a1a47be4188a5`
- C++ oracle executable SHA-256: `a64f0aec7aeb2f093b2564f3beeb2df07cdd8103a16a7a6e5f029847f39ffd25`
- rustc: `rustc 1.98.1 (48a229cea 2026-09-01)`
- cargo: `cargo 1.98.1 (797e8a9bc 2026-08-05)`
- g++: `13.3.0`
