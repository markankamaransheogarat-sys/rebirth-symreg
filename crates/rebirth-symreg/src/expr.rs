//! Abstract Syntax Tree (AST) for symbolic mathematical expressions.

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::vm::OpCode;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Sin,
    Cos,
    Exp,
    Log,
    Sqrt,
    Abs,
    Neg,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    X,
    Const(f32),
    Unary(UnaryOp, Box<Expr>),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
}

impl Expr {
    /// Compiles this expression AST into postfix StackVM bytecode.
    pub fn compile(&self) -> Vec<OpCode> {
        let mut bytecode = Vec::new();
        self.compile_into(&mut bytecode);
        bytecode
    }

    fn compile_into(&self, out: &mut Vec<OpCode>) {
        match self {
            Expr::X => out.push(OpCode::LoadX),
            Expr::Const(c) => out.push(OpCode::LoadConst(*c)),
            Expr::Unary(op, child) => {
                child.compile_into(out);
                match op {
                    UnaryOp::Sin => out.push(OpCode::Sin),
                    UnaryOp::Cos => out.push(OpCode::Cos),
                    UnaryOp::Exp => out.push(OpCode::Exp),
                    UnaryOp::Log => out.push(OpCode::Log),
                    UnaryOp::Sqrt => out.push(OpCode::Sqrt),
                    UnaryOp::Abs => out.push(OpCode::Abs),
                    UnaryOp::Neg => out.push(OpCode::Neg),
                }
            }
            Expr::Binary(op, left, right) => {
                left.compile_into(out);
                right.compile_into(out);
                match op {
                    BinaryOp::Add => out.push(OpCode::Add),
                    BinaryOp::Sub => out.push(OpCode::Sub),
                    BinaryOp::Mul => out.push(OpCode::Mul),
                    BinaryOp::Div => out.push(OpCode::Div),
                }
            }
        }
    }

    /// Number of AST nodes.
    pub fn node_count(&self) -> usize {
        match self {
            Expr::X | Expr::Const(_) => 1,
            Expr::Unary(_, child) => 1 + child.node_count(),
            Expr::Binary(_, left, right) => 1 + left.node_count() + right.node_count(),
        }
    }

    /// Tree depth.
    pub fn depth(&self) -> usize {
        match self {
            Expr::X | Expr::Const(_) => 1,
            Expr::Unary(_, child) => 1 + child.depth(),
            Expr::Binary(_, left, right) => {
                let dl = left.depth();
                let dr = right.depth();
                1 + if dl > dr { dl } else { dr }
            }
        }
    }

    /// Formats as mathematical string with appropriate parentheses.
    pub fn to_equation_string(&self) -> String {
        match self {
            Expr::X => "x".into(),
            Expr::Const(c) => format!("{:.3}", c),
            Expr::Unary(UnaryOp::Neg, child) => format!("-({})", child.to_equation_string()),
            Expr::Unary(UnaryOp::Sin, child) => format!("sin({})", child.to_equation_string()),
            Expr::Unary(UnaryOp::Cos, child) => format!("cos({})", child.to_equation_string()),
            Expr::Unary(UnaryOp::Exp, child) => format!("exp({})", child.to_equation_string()),
            Expr::Unary(UnaryOp::Log, child) => format!("ln({})", child.to_equation_string()),
            Expr::Unary(UnaryOp::Sqrt, child) => format!("sqrt({})", child.to_equation_string()),
            Expr::Unary(UnaryOp::Abs, child) => format!("abs({})", child.to_equation_string()),
            Expr::Binary(BinaryOp::Add, left, right) => {
                format!("({} + {})", left.to_equation_string(), right.to_equation_string())
            }
            Expr::Binary(BinaryOp::Sub, left, right) => {
                format!("({} - {})", left.to_equation_string(), right.to_equation_string())
            }
            Expr::Binary(BinaryOp::Mul, left, right) => {
                format!("({} * {})", left.to_equation_string(), right.to_equation_string())
            }
            Expr::Binary(BinaryOp::Div, left, right) => {
                format!("({} / {})", left.to_equation_string(), right.to_equation_string())
            }
        }
    }
}
