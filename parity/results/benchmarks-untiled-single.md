# Benchmarks, untiled filters, single-threaded Rust

> Historical: measured with the earlier untiled filters, before the 0.1.0 hardening changes, with no recorded Rust revision or executable hash. It does not describe the current code; see [qualification.md](qualification.md) for current measurements.

| Case | C++ -O2, 1 thread (s) | Rust, 1 thread (s) | C++ / Rust |
|---|---:|---:|---:|
| LDR 1920×1080 | 0.831494 | 0.669013 | 1.24× |
| HDR ACES, 3 exposures 1920×1080 | 2.096644 | 1.781044 | 1.18× |
| LDR 3840×2160 | 3.091505 | 2.749022 | 1.12× |
| HDR ACES, 3 exposures 3840×2160 | 8.259546 | 7.280522 | 1.13× |
