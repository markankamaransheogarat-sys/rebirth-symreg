//! Stack-based Virtual Machine and Bytecode representation.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Opcodes for the symbolic regression virtual machine.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OpCode {
    /// Push input variable x onto the stack. Cost: 1 byte.
    LoadX,
    /// Push a 32-bit float constant onto the stack. Cost: 1 byte opcode + 4 bytes float = 5 bytes.
    LoadConst(f32),
    /// Pop b, pop a, push a + b. Cost: 1 byte.
    Add,
    /// Pop b, pop a, push a - b. Cost: 1 byte.
    Sub,
    /// Pop b, pop a, push a * b. Cost: 1 byte.
    Mul,
    /// Pop b, pop a, push a / b. Strict: if |b| < 1e-6, fails as MathDomainError. Cost: 1 byte.
    Div,
    /// Pop a, push sin(a). Cost: 1 byte.
    Sin,
    /// Pop a, push cos(a). Cost: 1 byte.
    Cos,
    /// Pop a, push exp(a). Strict: if a > 20.0 (overflow), fails as MathDomainError. Cost: 1 byte.
    Exp,
    /// Pop a, push ln(a). Strict: if a <= 0.0, fails as MathDomainError. Cost: 1 byte.
    Log,
    /// Pop a, push sqrt(a). Strict: if a < 0.0, fails as MathDomainError. Cost: 1 byte.
    Sqrt,
    /// Pop a, push |a|. Cost: 1 byte.
    Abs,
    /// Pop a, push -a. Cost: 1 byte.
    Neg,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VmError {
    StackUnderflow,
    StackOverflow,
    MathDomainError,
    InvalidResult,
}

/// Floating point helpers supporting both std and no_std.
#[inline]
fn math_sin(x: f32) -> f32 {
    #[cfg(feature = "std")]
    { x.sin() }
    #[cfg(all(not(feature = "std"), feature = "libm"))]
    { libm::sinf(x) }
}

#[inline]
fn math_cos(x: f32) -> f32 {
    #[cfg(feature = "std")]
    { x.cos() }
    #[cfg(all(not(feature = "std"), feature = "libm"))]
    { libm::cosf(x) }
}

#[inline]
fn math_exp(x: f32) -> f32 {
    #[cfg(feature = "std")]
    { x.exp() }
    #[cfg(all(not(feature = "std"), feature = "libm"))]
    { libm::expf(x) }
}

#[inline]
fn math_ln(x: f32) -> f32 {
    #[cfg(feature = "std")]
    { x.ln() }
    #[cfg(all(not(feature = "std"), feature = "libm"))]
    { libm::logf(x) }
}

#[inline]
fn math_sqrt(x: f32) -> f32 {
    #[cfg(feature = "std")]
    { x.sqrt() }
    #[cfg(all(not(feature = "std"), feature = "libm"))]
    { libm::sqrtf(x) }
}

#[inline]
fn math_abs(x: f32) -> f32 {
    #[cfg(feature = "std")]
    { x.abs() }
    #[cfg(not(feature = "std"))]
    { if x < 0.0 { -x } else { x } }
}

/// Evaluates a bytecode program on input `x` using a fixed-capacity stack (zero allocations).
pub fn eval_bytecode(bytecode: &[OpCode], x: f32) -> Result<f32, VmError> {
    const STACK_CAPACITY: usize = 32;
    let mut stack = [0.0f32; STACK_CAPACITY];
    let mut sp = 0; // Stack pointer

    for &op in bytecode {
        match op {
            OpCode::LoadX => {
                if sp >= STACK_CAPACITY {
                    return Err(VmError::StackOverflow);
                }
                stack[sp] = x;
                sp += 1;
            }
            OpCode::LoadConst(c) => {
                if sp >= STACK_CAPACITY {
                    return Err(VmError::StackOverflow);
                }
                stack[sp] = c;
                sp += 1;
            }
            OpCode::Add => {
                if sp < 2 { return Err(VmError::StackUnderflow); }
                let b = stack[sp - 1];
                let a = stack[sp - 2];
                sp -= 1;
                let res = a + b;
                if !res.is_finite() { return Err(VmError::InvalidResult); }
                stack[sp - 1] = res;
            }
            OpCode::Sub => {
                if sp < 2 { return Err(VmError::StackUnderflow); }
                let b = stack[sp - 1];
                let a = stack[sp - 2];
                sp -= 1;
                let res = a - b;
                if !res.is_finite() { return Err(VmError::InvalidResult); }
                stack[sp - 1] = res;
            }
            OpCode::Mul => {
                if sp < 2 { return Err(VmError::StackUnderflow); }
                let b = stack[sp - 1];
                let a = stack[sp - 2];
                sp -= 1;
                let res = a * b;
                if !res.is_finite() { return Err(VmError::InvalidResult); }
                stack[sp - 1] = res;
            }
            OpCode::Div => {
                if sp < 2 { return Err(VmError::StackUnderflow); }
                let b = stack[sp - 1];
                let a = stack[sp - 2];
                sp -= 1;
                // Strict: prevent artificial singularity exploitation
                if math_abs(b) < 1e-6 {
                    return Err(VmError::MathDomainError);
                }
                let res = a / b;
                if !res.is_finite() { return Err(VmError::InvalidResult); }
                stack[sp - 1] = res;
            }
            OpCode::Sin => {
                if sp < 1 { return Err(VmError::StackUnderflow); }
                let a = stack[sp - 1];
                stack[sp - 1] = math_sin(a);
            }
            OpCode::Cos => {
                if sp < 1 { return Err(VmError::StackUnderflow); }
                let a = stack[sp - 1];
                stack[sp - 1] = math_cos(a);
            }
            OpCode::Exp => {
                if sp < 1 { return Err(VmError::StackUnderflow); }
                let a = stack[sp - 1];
                if a > 20.0 {
                    return Err(VmError::MathDomainError);
                }
                let res = math_exp(a);
                if !res.is_finite() { return Err(VmError::InvalidResult); }
                stack[sp - 1] = res;
            }
            OpCode::Log => {
                if sp < 1 { return Err(VmError::StackUnderflow); }
                let a = stack[sp - 1];
                if a <= 1e-6 {
                    return Err(VmError::MathDomainError);
                }
                let res = math_ln(a);
                if !res.is_finite() { return Err(VmError::InvalidResult); }
                stack[sp - 1] = res;
            }
            OpCode::Sqrt => {
                if sp < 1 { return Err(VmError::StackUnderflow); }
                let a = stack[sp - 1];
                if a < 0.0 {
                    return Err(VmError::MathDomainError);
                }
                stack[sp - 1] = math_sqrt(a);
            }
            OpCode::Abs => {
                if sp < 1 { return Err(VmError::StackUnderflow); }
                stack[sp - 1] = math_abs(stack[sp - 1]);
            }
            OpCode::Neg => {
                if sp < 1 { return Err(VmError::StackUnderflow); }
                stack[sp - 1] = -stack[sp - 1];
            }
        }
    }

    if sp == 1 {
        let val = stack[0];
        if val.is_finite() {
            Ok(val)
        } else {
            Err(VmError::InvalidResult)
        }
    } else {
        Err(VmError::StackUnderflow)
    }
}

/// Calculate the exact binary byte size of this bytecode.
/// Every standard opcode is 1 byte.
/// Every LoadConst(f32) is 1 byte opcode + 4 bytes IEEE-754 float = 5 bytes.
pub fn bytecode_byte_size(bytecode: &[OpCode]) -> usize {
    let mut total = 0;
    for op in bytecode {
        match op {
            OpCode::LoadConst(_) => total += 5,
            _ => total += 1,
        }
    }
    total
}
