# Contributing to rebirth-symreg

Thank you for your interest in contributing to `rebirth-symreg`!

## Architectural & Safety Principles

1. **Zero Unsafe Code:** `#![forbid(unsafe_code)]` must remain active and strictly enforced on all crates.
2. **Honest Information Theory:** No unmeasured compression claims. Any changes to the VM or MDL accounting must preserve empirical mathematical rigor (e.g. proper constant byte pricing and residual entropy accounting).
3. **Deterministic Testing:** Random number generation in tests must use seeded deterministic PRNGs (`XorShift64`).

## Development Workflow

1. Format and lint:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets -- -D warnings
   ```
2. Run test suite:
   ```bash
   cargo test --all
   ```
3. Run empirical benchmarks:
   ```bash
   cargo run --release --example benchmark_scenarios
   ```
4. Build WASM package:
   ```bash
   cargo build -p rebirth-symreg-wasm --target wasm32-unknown-unknown --release
   wasm-bindgen target/wasm32-unknown-unknown/release/rebirth_symreg_wasm.wasm --out-dir web/pkg --target web
   ```
