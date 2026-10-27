//! # rebirth-symreg
//!
//! A fast, pedagogical symbolic regression & program synthesis engine written in safe Rust.
//!
//! Features:
//! - `#![forbid(unsafe_code)]` — strictly zero unsafe code.
//! - Stack-based virtual machine (`StackVM`) with strict domain validation (no cheat singularities).
//! - Genetic programming search loop with tournament selection, subtree crossover, and constant refinement.
//! - Honest Minimum Description Length (MDL) residual telemetry: counts 5 bytes per 32-bit constant,
//!   and penalizes residuals with empirical Shannon entropy so noisy or random data cannot cheat.
//! - WebAssembly and `no_std` compatible (via `libm`).

#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod evolution;
pub mod expr;
pub mod mdl;
pub mod vm;

pub use evolution::{Individual, RegressorConfig, SymbolicRegressor};
pub use expr::{BinaryOp, Expr, UnaryOp};
pub use mdl::{compute_mdl_telemetry, MdlTelemetry};
pub use vm::{bytecode_byte_size, eval_bytecode, OpCode, VmError};
