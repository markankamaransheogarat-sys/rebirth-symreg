//! Minimum Description Length (MDL) and Information-Theoretic Residual Cost.

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::collections::BTreeMap;

use crate::vm::{bytecode_byte_size, OpCode};

/// Detailed telemetry of formula complexity, residuals, and honest compression.
#[derive(Clone, Debug, PartialEq)]
pub struct MdlTelemetry {
    /// Number of training data points N.
    pub num_points: usize,
    /// Uncompressed size of data in bytes (N * 4 bytes for float32).
    pub raw_bytes: usize,
    /// Size of the synthesized bytecode program in bytes.
    /// Every opcode = 1 byte; LoadConst(f32) = 5 bytes (1 byte opcode + 4 bytes float).
    pub bytecode_bytes: usize,
    /// Number of unique constants in formula.
    pub num_constants: usize,
    /// Error tolerance epsilon used to quantize residuals.
    pub epsilon: f32,
    /// True if all data points fall within [-epsilon, +epsilon] of formula.
    pub exact_match: bool,
    /// Information-theoretic cost of residuals in bytes (entropy coding of non-zero error bins).
    pub residual_bytes: usize,
    /// Total compressed representation: bytecode_bytes + residual_bytes.
    pub total_compressed_bytes: usize,
    /// Measured compression ratio: raw_bytes / total_compressed_bytes.
    pub compression_ratio: f32,
    /// Mean Squared Error (MSE) across all points.
    pub mse: f32,
    /// Coefficient of determination R^2 (1.0 = perfect fit).
    pub r_squared: f32,
}

#[inline]
fn math_log2(x: f32) -> f32 {
    #[cfg(feature = "std")]
    {
        x.log2()
    }
    #[cfg(all(not(feature = "std"), feature = "libm"))]
    {
        libm::log2f(x)
    }
}

/// Computes honest MDL metric and residual telemetry for predicted vs true points.
///
/// # Strict Rules:
/// 1. Bytecode size explicitly counts 5 bytes per `LoadConst(f32)` (32-bit float constant + opcode).
/// 2. Residuals are quantized at resolution `epsilon`.
/// 3. If residuals are non-zero (noisy or unfitted data), the Shannon entropy cost of storing them
///    is rigorously computed and added to `total_compressed_bytes`.
pub fn compute_mdl_telemetry(
    bytecode: &[OpCode],
    true_y: &[f32],
    predicted_y: &[f32],
    epsilon: f32,
) -> MdlTelemetry {
    let n = true_y.len();
    assert_eq!(n, predicted_y.len(), "Array lengths must match");
    assert!(n > 0, "Must have at least one data point");

    let raw_bytes = n * 4; // 4 bytes per float32
    let bytecode_bytes = bytecode_byte_size(bytecode);

    let mut num_constants = 0;
    for op in bytecode {
        if let OpCode::LoadConst(_) = op {
            num_constants += 1;
        }
    }

    let mut sum_sq_err = 0.0f32;
    let mut sum_y = 0.0f32;
    for &y in true_y {
        sum_y += y;
    }
    let mean_y = sum_y / n as f32;

    let mut total_variance = 0.0f32;
    for &y in true_y {
        let diff = y - mean_y;
        total_variance += diff * diff;
    }

    let inv_eps = if epsilon > 0.0 { 1.0 / epsilon } else { 1.0 };
    let mut residual_counts: BTreeMap<i32, usize> = BTreeMap::new();
    let mut max_abs_err = 0.0f32;

    for i in 0..n {
        let err = true_y[i] - predicted_y[i];
        let abs_err = if err < 0.0 { -err } else { err };
        if abs_err > max_abs_err {
            max_abs_err = abs_err;
        }
        sum_sq_err += err * err;

        // Quantized integer residual bin
        let bin = (err * inv_eps).round() as i32;
        *residual_counts.entry(bin).or_insert(0) += 1;
    }

    let mse = sum_sq_err / n as f32;
    let r_squared = if total_variance > 1e-12 {
        1.0 - (sum_sq_err / total_variance)
    } else if mse < 1e-6 {
        1.0
    } else {
        0.0
    };

    let exact_match = max_abs_err <= epsilon;

    let residual_bytes = if exact_match {
        // Zero residuals cost (just a single flag byte indicating zero residuals within epsilon)
        1
    } else {
        // Calculate empirical Shannon entropy of the residual distribution
        let mut entropy_bits = 0.0f32;
        for &count in residual_counts.values() {
            let p = count as f32 / n as f32;
            if p > 0.0 {
                entropy_bits -= p * math_log2(p);
            }
        }
        // Total entropy bits for N residuals
        let total_bits = (n as f32 * entropy_bits).ceil();
        // Plus 4 bytes overhead per unique non-zero symbol in residual table dictionary
        let dict_overhead = residual_counts.len() * 4;
        (total_bits / 8.0).ceil() as usize + dict_overhead
    };

    let total_compressed_bytes = bytecode_bytes + residual_bytes;
    let compression_ratio = raw_bytes as f32 / total_compressed_bytes as f32;

    MdlTelemetry {
        num_points: n,
        raw_bytes,
        bytecode_bytes,
        num_constants,
        epsilon,
        exact_match,
        residual_bytes,
        total_compressed_bytes,
        compression_ratio,
        mse,
        r_squared,
    }
}
