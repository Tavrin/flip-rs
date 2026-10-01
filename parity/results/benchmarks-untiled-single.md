> Historical baseline before hardening. This record lacks a Rust source revision and benchmark executable hash; it does not qualify the current source. See [qualification.md](qualification.md) for current bound measurements.

| Case | C++ -O2 single thread (s) | Rust (s) | C++ / Rust |
|---|---:|---:|---:|
| LDR 1920×1080 | 0.831494 | 0.669013 | 1.24× |
| HDR ACES, 3 exposures 1920×1080 | 2.096644 | 1.781044 | 1.18× |
| LDR 3840×2160 | 3.091505 | 2.749022 | 1.12× |
| HDR ACES, 3 exposures 3840×2160 | 8.259546 | 7.280522 | 1.13× |
