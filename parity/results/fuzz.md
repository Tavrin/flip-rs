# Fuzzing record

Measured 2026-10-02 with `1.101.0-nightly (21b707e3f 2026-09-30)`,
cargo-fuzz 0.13.2 and libfuzzer-sys 0.4.13, with AddressSanitizer, on the
machine described in [qualification.md](qualification.md). The five targets
ran at the same time for 311 seconds each, with `-seed=3572951
-max_total_time=310 -max_len=4096 -print_final_stats=1`.

| Target | Runtime (s) | Executions | Crashes | Exit | Peak RSS (MiB) |
|---|---:|---:|---:|---:|---:|
| `ldr_flip` | 311 | 2,045,170 | 0 | 0 | 415 |
| `hdr_flip` | 311 | 275,883 | 0 | 0 | 390 |
| `pooling` | 311 | 39,653,438 | 0 | 0 | 457 |
| `colorize` | 311 | 101,592,493 | 0 | 0 | 472 |
| `image_loaders` | 311 | 898,354 | 0 | 0 | 490 |

Total: 144,465,338 executions, 0 crashes.

Earlier rounds on previous versions of the source also found no crashes; they
are not counted here. Raw logs were not published.

To reproduce, for each target:

```sh
cargo +nightly fuzz run TARGET -- -max_total_time=310 -seed=3572951 -max_len=4096 -print_final_stats=1
```

This run started from the corpus of an earlier round, which included valid
2×2 PNG and OpenEXR seeds. Successful calls assert the output shape and
finiteness, or histogram conservation; `FlipError` results are accepted, and
a panic or abort fails the target. The loader target exercises both public
path-based loaders and format rejection.

## 32-bit allocation audit

`dimensions_with_limit` checks width × height × channels and the `f32` byte
size against `isize::MAX` before reserving. A regression test shows that
178,956,971 scalar pixels fit the wasm32 limit but their RGB expansion does
not (2,147,483,652 > 2,147,483,647 bytes). The test computes this boundary on
the host; it does not allocate on wasm. Gray-RGB saving,
Magma expansion, workspaces and wasm RGBA conversion reuse the checked helper.
Capacity overflow in fallible reservation is tested without exhausting memory.
Filter radii are capped before integer conversion and work is checked in u64,
so the 2^34 limit remains valid when usize is 32 bits.

## Sources and binaries

Rust base revision `5f4d5c29a0dc40fed2bda8e1d6f6ccc1772d412e` plus the
uncommitted hardening changes. The library source hashes are the same as in
the corpus and sweep records. The executed fuzz binaries are hashed below.

- `src/color.rs` SHA-256: `b34d4d6266cbc166b81d58b354d5cd1c584abda530ac76477a67a429d93b69c1`
- `src/filters.rs` SHA-256: `b5dc598c4e65613091c3f97e39be1b5eff7f26d8ba40433c8746a6c14609a7b7`
- `src/hdr.rs` SHA-256: `828bbeceb24fa7f685bc6bde8c442e1d517f2e3cd02df1e752ae1705ebd5ea44`
- `src/io.rs` SHA-256: `51c7e129202d7d7ddfcd1822d0b4d1bb2dda02952f1486be0504187638010e8d`
- `src/lib.rs` SHA-256: `250f28aaed2e80159ca26b462ffd5cced2bfc56254119440b92893d78d5c67fc`
- `src/magma.rs` SHA-256: `59340581c70404490a1b77fe932e2a0ddbe387ceaccc8c673ea5db95e6f7cf97`
- `src/pooling.rs` SHA-256: `c20e188db741edbb7bc8528ded9bc788d1538748ab8d376aa86d31266a6558db`
- `src/wasm.rs` SHA-256: `d17331686f40ee277ea1b557d57a5410fab687e8ed451e51c5825d8e56d1276a`
- `Cargo.lock` SHA-256: `4e4fc4ff20f311e0daf9a9745c8d8d3e0a9dee320be259961936d22ea6e4d0b3`
- `fuzz/Cargo.lock` SHA-256: `89aac81ff7845ca57792ed14eb828288aae12204d7744760156023599bffc29a`
- `ldr_flip` executed binary SHA-256: `ff92b9c07d0dc22551cbd340317582bef026d4676af4783aa00ff80d572342d2`
- `hdr_flip` executed binary SHA-256: `84cb7d8eeaf842dc165e2419dc9e083d0d4a51378a48930d9a32527a125bd7de`
- `pooling` executed binary SHA-256: `2d2b975a5e816c4abb5dc6d712faf989a0e0f2c3adec135a2d2226645ede2413`
- `colorize` executed binary SHA-256: `46b024a65ea935c4f19ee1e595f2c8a2258e5216fbba676468976adf4cce3064`
- `image_loaders` executed binary SHA-256: `09e07add1743d79cb5ca1bbaf3e66a1fa74d0f4f04ad0e0890013392cc1057e9`
