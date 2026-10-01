> Historical baseline before hardening. This record lacks a Rust source revision and benchmark executable hash; it does not qualify the current source. See [qualification.md](qualification.md) for current bound measurements.

| Case | C++ -O2 single thread (s) | Rust (s) | C++ / Rust |
|---|---:|---:|---:|
| LDR 1920×1080 | 0.736543 | 0.210339 | 3.50× |
| HDR ACES, 3 exposures 1920×1080 | 1.896143 | 0.304838 | 6.22× |
| LDR 3840×2160 | 3.121960 | 0.858680 | 3.64× |
| HDR ACES, 3 exposures 3840×2160 | 8.095539 | 1.143651 | 7.08× |
