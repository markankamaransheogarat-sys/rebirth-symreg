# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-10-28

### Added
- Core `rebirth-symreg` engine in 100% safe Rust (`#![forbid(unsafe_code)]`).
- Fixed-capacity zero-allocation `StackVM` with support for unary and binary operators (`Add`, `Sub`, `Mul`, `Div`, `Sin`, `Cos`, `Exp`, `Log`, `Sqrt`, `Abs`, `Neg`).
- Strict math domain handling preventing numerical overflow and division by zero cheats.
- Postfix AST bytecode compiler and mathematical equation formatter.
- Rigorous Minimum Description Length (MDL) telemetry calculator:
  - 5-byte accounting for floating-point constants (1B opcode + 4B IEEE-754).
  - Quantized error entropy calculation via empirical Shannon entropy $H(R)$.
  - Exact match detection with 1-byte flag fallback.
- Genetic programming engine with tournament selection, subtree crossover, point mutation, constant jitter, and elitism.
- `rebirth-symreg-wasm`: WebAssembly bindings exposing `WasmRegressor` for high-frequency browser execution.
- Interactive Web demo with Canvas 60 FPS plotting, harmonic, quadratic, noisy, and random data presets, and live MDL telemetry cards.
- Comprehensive integration tests and empirical reproducible benchmarks.
