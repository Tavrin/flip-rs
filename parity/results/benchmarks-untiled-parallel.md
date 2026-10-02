# Benchmarks, untiled filters, 8-thread Rust

> Historical: measured with the earlier untiled filters, before the 0.1.0 hardening changes, with no recorded Rust revision or executable hash. It does not describe the current code; see [qualification.md](qualification.md) for current measurements.

| Case | C++ -O2, 1 thread (s) | Rust, 8 threads (s) | C++ 1 thread / Rust 8 threads |
|---|---:|---:|---:|
| LDR 1920×1080 | 0.736543 | 0.210339 | 3.50× |
| HDR ACES, 3 exposures 1920×1080 | 1.896143 | 0.304838 | 6.22× |
| LDR 3840×2160 | 3.121960 | 0.858680 | 3.64× |
| HDR ACES, 3 exposures 3840×2160 | 8.095539 | 1.143651 | 7.08× |
