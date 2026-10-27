use rebirth_symreg::{
    eval_bytecode, BinaryOp, Expr, OpCode, RegressorConfig, SymbolicRegressor, UnaryOp, VmError,
};

#[test]
fn test_vm_evaluation() {
    // 2.0 * 3.0 + 4.0 = 10.0
    let code = vec![
        OpCode::LoadConst(2.0),
        OpCode::LoadConst(3.0),
        OpCode::Mul,
        OpCode::LoadConst(4.0),
        OpCode::Add,
    ];
    let res = eval_bytecode(&code, 0.0).unwrap();
    assert!((res - 10.0).abs() < 1e-6);

    // Strict division by near-zero must fail with MathDomainError
    let div_zero = vec![
        OpCode::LoadConst(5.0),
        OpCode::LoadConst(0.0),
        OpCode::Div,
    ];
    assert_eq!(eval_bytecode(&div_zero, 0.0), Err(VmError::MathDomainError));

    // Strict log of negative must fail
    let log_neg = vec![
        OpCode::LoadConst(-1.0),
        OpCode::Log,
    ];
    assert_eq!(eval_bytecode(&log_neg, 0.0), Err(VmError::MathDomainError));
}

#[test]
fn test_expr_compilation() {
    // (x * 2.0) + sin(x)
    let expr = Expr::Binary(
        BinaryOp::Add,
        Box::new(Expr::Binary(
            BinaryOp::Mul,
            Box::new(Expr::X),
            Box::new(Expr::Const(2.0)),
        )),
        Box::new(Expr::Unary(UnaryOp::Sin, Box::new(Expr::X))),
    );

    let bytecode = expr.compile();
    let val = eval_bytecode(&bytecode, 1.0).unwrap();
    let expected = (1.0 * 2.0) + 1.0f32.sin();
    assert!((val - expected).abs() < 1e-5);
}

#[test]
fn test_mdl_telemetry_accounts_constants() {
    // Formula with two constants: const(1.0) and const(2.0)
    let code = vec![
        OpCode::LoadX,
        OpCode::LoadConst(1.0), // 5 bytes
        OpCode::Add,            // 1 byte
        OpCode::LoadConst(2.0), // 5 bytes
        OpCode::Mul,            // 1 byte
    ];
    // Total bytecode bytes: 1 (LoadX) + 5 (Const) + 1 (Add) + 5 (Const) + 1 (Mul) = 13 bytes
    assert_eq!(rebirth_symreg::bytecode_byte_size(&code), 13);
}

#[test]
fn test_genetic_regression_finds_linear() {
    let n = 20;
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for i in 0..n {
        let x = i as f32;
        xs.push(x);
        ys.push(x + 2.0);
    }

    let config = RegressorConfig {
        population_size: 100,
        max_depth: 3,
        epsilon: 0.01,
        ..Default::default()
    };
    let mut reg = SymbolicRegressor::new(config, 42);

    for _ in 0..30 {
        reg.step_generation(&xs, &ys);
        if let Some(ref best) = reg.best_individual {
            if best.mse < 0.01 {
                break;
            }
        }
    }

    let best = reg.best_individual.unwrap();
    assert!(best.mse < 0.1, "MSE was {}, expected < 0.1", best.mse);
}
