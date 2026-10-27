# rebirth-symreg 🔬

[![CI](https://github.com/markankamaransheogarat-sys/rebirth-symreg/actions/workflows/ci.yml/badge.svg)](https://github.com/markankamaransheogarat-sys/rebirth-symreg/actions)
[![Crates.io](https://img.shields.io/crates/v/rebirth-symreg.svg)](https://crates.io/crates/rebirth-symreg)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Safety: 100% Safe](https://img.shields.io/badge/unsafe-forbidden-success.svg)](src/lib.rs)
[![Interactive Demo](https://img.shields.io/badge/demo-Live%20WASM-cyan.svg)](https://markankamaransheogarat-sys.github.io/rebirth-symreg/)

A pedagogical, mathematically rigorous **Symbolic Regression & Genetic Program Synthesis engine** written in 100% safe Rust.

Instead of hand-waving "1000:1 compression on arbitrary data" (which is mathematically impossible by Shannon's source coding theorem and the Pigeonhole principle), `rebirth-symreg` implements **honest Minimum Description Length (MDL)** telemetry:
- Formula ASTs are compiled into bytecode for a compact zero-allocation StackVM.
- Floating-point constants are explicitly billed **5 bytes** (1-byte opcode + 4-byte IEEE-754 float) so the genetic algorithm cannot cheat by hiding complex data inside uncompressed constants.
- Residuals (differences between true signal and formula) are quantized at tolerance $\varepsilon = 0.05$ and encoded using empirical **Shannon entropy** $H(R)$ plus dictionary overhead.
- When an exact law exists, residual entropy collapses to 1 byte and yields **100:1** compression.
- When data is pure noise, residual entropy equals the raw entropy, proving Shannon's theorem: **compression collapses to 1.14:1**.

---

## 🎮 [Interactive WebAssembly Live Demo](https://markankamaransheogarat-sys.github.io/rebirth-symreg/)

Run real-time genetic synthesis directly in your browser with 60 FPS Canvas visualization and live MDL telemetry cards.

---

## 📊 Empirical Measured Benchmarks

All metrics below are measured directly on 200 `f32` data points (800 bytes raw data, $\varepsilon = 0.05$ tolerance) using `cargo run --release --example benchmark_scenarios`:

| Scenario | True Signal / Generator | Best Discovered AST | MSE | Raw Bytes | Code Bytes | Residual Bytes | Total Bytes | **Measured Ratio** | Reduction |
|:---|:---|:---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **1. Harmonic** | $y = 2\sin(x) + x$ | `(sin(x) + (sin(x) + x))` | 0.000000 | 800 B | 7 B | 1 B | 8 B | **100.00:1** | 99.00% |
| **2. Parabola** | $y = x^2 - 3.0$ | `(-3.000 + (x * x))` | 0.000000 | 800 B | 9 B | 1 B | 10 B | **80.00:1** | 98.75% |
| **3. Parabola + 15% Noise** | $y = x^2 - 3.0 + \mathcal{N}$ | `(-3.048 + (x * x))` | 0.051262 | 800 B | 9 B | 169 B | 178 B | **4.49:1** | 77.75% |
| **4. Pure Random Noise** | Uniform $\mathcal{U}(-4, 4)$ | `(-0.165 + ...)` | 8.731953 | 800 B | 6 B | 697 B | 703 B | **1.14:1** | 12.12% |

### Key Empirical Observations
1. **Harmonic & Parabolic signals:** The evolutionary search converges on the exact analytical generator. Because $|y_i - \hat{y}_i| \le \varepsilon$ for all $i$, residual cost is a single 1-byte flag, achieving an 80:1 to 100:1 compression ratio.
2. **Signal with 15% noise:** The formula is still discovered accurately (`x*x - 3.048`), but the residual noise cannot be compressed, requiring 169 bytes of entropy. The ratio drops to 4.49:1.
3. **Pure random data:** No underlying formula exists. The genetic search finds a near-constant baseline, and residual cost consumes 697 bytes. Total compressed size is 703 bytes (1.14:1 ratio).

---

## 🏛️ Architecture & Mathematical Guarantees

### 1. Zero-Allocation StackVM
Expressions are compiled into linear bytecode executed on a fixed-capacity evaluation stack:
```rust
pub enum OpCode {
    LoadX,
    LoadConst(f32),
    Add, Sub, Mul, Div,
    Sin, Cos, Exp, Log, Sqrt, Abs, Neg,
}
```

### 2. Strict Math Domain Protection
Singularities (division by zero, logarithm of non-positive numbers, overflow in exponentiation) are not masked or replaced with false constants. The VM returns a typed `VmError::MathDomainError` which assigns $-\infty$ fitness to invalid chromosomes, preventing singularity cheats during evolution.

### 3. Minimum Description Length (MDL) Formulation
Total description length is given by:
$$L(D) = L(H) + L(D \mid H)$$

Where:
- $L(H)$: Size of bytecode hypothesis in bytes (1 byte per opcode + 5 bytes per float constant).
- $L(D \mid H)$: Size of quantized residuals under empirical Shannon entropy:
  $$H(R) = -\sum_{r \in \mathcal{R}} p(r) \log_2 p(r)$$
  plus unique dictionary overhead (4 bytes per unique quantized symbol).

---

## 🚀 Quick Start (Rust Library)

Add to `Cargo.toml`:
```toml
[dependencies]
rebirth-symreg = "0.1"
```

### Example Usage:
```rust
use rebirth_symreg::{SymbolicRegressor, RegressorConfig};

fn main() {
    let xs: Vec<f32> = (-50..=50).map(|i| i as f32 * 0.1).collect();
    let ys: Vec<f32> = xs.iter().map(|&x| x * x - 2.0).collect();

    let config = RegressorConfig {
        population_size: 200,
        max_depth: 5,
        complexity_penalty: 0.001,
        epsilon: 0.05,
        ..Default::default()
    };

    let mut regressor = SymbolicRegressor::new(config, 42);

    for gen in 0..100 {
        regressor.step_generation(&xs, &ys);
    }

    if let Some(best) = regressor.best_individual {
        println!("Discovered Equation: {}", best.expr.to_equation_string());
        println!("MSE: {:.6}", best.mse);
    }

    if let Some(telemetry) = regressor.get_telemetry(&xs, &ys) {
        println!("Measured Ratio: {:.2}:1", telemetry.compression_ratio);
    }
}
```

---

## 📦 Workspace Layout

```
rebirth-symreg/
├── crates/
│   ├── rebirth-symreg/         # Core engine: VM, AST, Genetic Search, MDL
│   │   ├── src/vm.rs           # StackVM & opcodes
│   │   ├── src/expr.rs         # AST expression tree & compiler
│   │   ├── src/mdl.rs          # Shannon entropy & MDL telemetry
│   │   └── src/evolution.rs    # Genetic algorithm, tournament, crossover
│   └── rebirth-symreg-wasm/    # WebAssembly bindings (wasm-bindgen)
│       └── src/lib.rs          # WasmRegressor wrapper
├── web/                        # Interactive GitHub Pages web demo
│   ├── index.html              # Dark cyberpunk UI + Canvas plot
│   └── pkg/                    # Generated WASM artifacts
└── examples/
    └── benchmark_scenarios.rs  # Real empirical reproducibility benchmark
```

---

## 🛡️ Verification & Safety

- `#![forbid(unsafe_code)]` enforced across all crates.
- Comprehensive integration tests covering arithmetic execution, singularity errors, constant byte accounting, and genetic convergence.
- Deterministic pseudo-random number generator (`XorShift64`) allows reproducible runs across architectures.

---

## 📜 License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.
