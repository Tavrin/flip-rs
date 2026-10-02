# Contributing

Bug reports and pull requests are welcome. For security issues, see
[SECURITY.md](SECURITY.md).

## Build and test

```sh
cargo build
cargo test --all-features
cargo test --no-default-features
cargo build --target wasm32-unknown-unknown --no-default-features --features wasm
```

The minimum supported Rust version is 1.88.

## Style

CI runs these, and they must pass:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
```

New code should not use `unsafe`.

## Parity

The crate must produce the same results as NVIDIA FLIP v1.7. Any change to
numerical code (color conversion, tone mapping, filters, pooling, exposure
selection) must keep the corpus and the sweep passing against the reference,
without loosening the tolerances. Run them before opening a pull request:

```sh
git clone https://github.com/NVlabs/flip nvlabs-flip
git -C nvlabs-flip checkout b475eb4bf394ab877c42166c9eb0a84a02cc5b14
export FLIP_RS_REFERENCE="$PWD/nvlabs-flip"

./parity/run.sh     # 201-case corpus and benchmarks
./parity/sweep.sh   # 3,000-case randomized sweep
FLIP_RS_PARITY_BIN="$PWD/parity/build/reference" cargo test --release --test parity
```

`run.sh` needs `g++` with C++17. [`parity/README.md`](parity/README.md)
describes the harness. If a change affects performance, include the benchmark
tables from `parity/build/` in the pull request.

## Sign-off

Commits must be signed off under the Developer Certificate of Origin
(<https://developercertificate.org/>): use `git commit -s`, which adds a
`Signed-off-by:` line with your name and email.

## Licence

Contributions are licensed under the BSD-3-Clause licence in
[LICENSE](LICENSE).
