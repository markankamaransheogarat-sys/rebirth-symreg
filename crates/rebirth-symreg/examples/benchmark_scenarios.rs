use rebirth_symreg::{RegressorConfig, SymbolicRegressor};

fn main() {
    println!("================================================================================");
    println!("     REBIRTH-SYMREG: EMPIRICAL BENCHMARK OF 4 SCENARIOS (HONEST MDL)            ");
    println!("================================================================================");
    println!("Tolerance epsilon: 0.05");
    println!("Constant storage cost: 5 bytes per f32 constant (1 opcode + 4 bytes float)");
    println!("Opcode storage cost:   1 byte per instruction\n");

    let n = 200; // 200 data points -> Raw size: 800 bytes
    let epsilon = 0.05;

    // -------------------------------------------------------------------------
    // Scenario 1: Clean Mathematical Law: y = 2.0 * sin(x) + x
    // -------------------------------------------------------------------------
    println!(">>> SCENARIO 1: Pure Deterministic Law (y = 2.0 * sin(x) + x)");
    let mut xs1 = Vec::with_capacity(n);
    let mut ys1 = Vec::with_capacity(n);
    for i in 0..n {
        let x = -3.0 + 6.0 * (i as f32) / (n as f32);
        xs1.push(x);
        ys1.push(2.0 * x.sin() + x);
    }

    let config1 = RegressorConfig {
        population_size: 200,
        max_depth: 4,
        tournament_size: 4,
        complexity_penalty: 0.0001,
        epsilon,
        ..Default::default()
    };
    let mut reg1 = SymbolicRegressor::new(config1.clone(), 42);

    for _ in 0..60 {
        reg1.step_generation(&xs1, &ys1);
        if let Some(ref best) = reg1.best_individual {
            if best.mse < 0.001 { break; }
        }
    }

    let t1 = reg1.get_telemetry(&xs1, &ys1).unwrap();
    let best1 = reg1.best_individual.as_ref().unwrap();
    println!("Discovered Equation:  {}", best1.expr.to_equation_string());
    println!("MSE:                  {:.6}", t1.mse);
    println!("Raw Array Size:       {} bytes (200 x 4 bytes)", t1.raw_bytes);
    println!("Bytecode Size:        {} bytes (incl. constants)", t1.bytecode_bytes);
    println!("Residuals Size:       {} bytes (exact match: {})", t1.residual_bytes, t1.exact_match);
    println!("Total Compressed:     {} bytes", t1.total_compressed_bytes);
    println!("MEASURED RATIO:       {:.2}:1 ({:.2}% reduction)\n", t1.compression_ratio, (1.0 - 1.0 / t1.compression_ratio) * 100.0);

    // -------------------------------------------------------------------------
    // Scenario 2: Polynomial Law (y = x * x - 3.0)
    // -------------------------------------------------------------------------
    println!(">>> SCENARIO 2: Quadratic Parabola (y = x * x - 3.0)");
    let mut xs2 = Vec::with_capacity(n);
    let mut ys2 = Vec::with_capacity(n);
    for i in 0..n {
        let x = -2.0 + 4.0 * (i as f32) / (n as f32);
        xs2.push(x);
        ys2.push(x * x - 3.0);
    }

    let mut reg2 = SymbolicRegressor::new(config1.clone(), 123);
    for _ in 0..60 {
        reg2.step_generation(&xs2, &ys2);
        if let Some(ref best) = reg2.best_individual {
            if best.mse < 0.001 { break; }
        }
    }

    let t2 = reg2.get_telemetry(&xs2, &ys2).unwrap();
    let best2 = reg2.best_individual.as_ref().unwrap();
    println!("Discovered Equation:  {}", best2.expr.to_equation_string());
    println!("MSE:                  {:.6}", t2.mse);
    println!("Raw Array Size:       {} bytes", t2.raw_bytes);
    println!("Bytecode Size:        {} bytes", t2.bytecode_bytes);
    println!("Residuals Size:       {} bytes (exact match: {})", t2.residual_bytes, t2.exact_match);
    println!("Total Compressed:     {} bytes", t2.total_compressed_bytes);
    println!("MEASURED RATIO:       {:.2}:1 ({:.2}% reduction)\n", t2.compression_ratio, (1.0 - 1.0 / t2.compression_ratio) * 100.0);

    // -------------------------------------------------------------------------
    // Scenario 3: Signal + 15% Noise (y = x * x - 3.0 + noise)
    // -------------------------------------------------------------------------
    println!(">>> SCENARIO 3: Same Quadratic Law + 15% Real Measurement Noise");
    let mut ys3 = Vec::with_capacity(n);
    let mut rng = rebirth_symreg::evolution::Rng::new(999);
    for &y in &ys2 {
        let noise = rng.next_range_f32(-0.4, 0.4);
        ys3.push(y + noise);
    }

    let mut reg3 = SymbolicRegressor::new(config1.clone(), 123);
    for _ in 0..60 {
        reg3.step_generation(&xs2, &ys3);
    }

    let t3 = reg3.get_telemetry(&xs2, &ys3).unwrap();
    let best3 = reg3.best_individual.as_ref().unwrap();
    println!("Discovered Equation:  {}", best3.expr.to_equation_string());
    println!("MSE:                  {:.6}", t3.mse);
    println!("Raw Array Size:       {} bytes", t3.raw_bytes);
    println!("Bytecode Size:        {} bytes", t3.bytecode_bytes);
    println!("Residuals Size:       {} bytes (exact match: {})", t3.residual_bytes, t3.exact_match);
    println!("Total Compressed:     {} bytes (Formula + Noise Residuals)", t3.total_compressed_bytes);
    println!("MEASURED RATIO:       {:.2}:1 ({:.2}% reduction) -> NOISE CONSUMED THE GAINS!\n", t3.compression_ratio, (1.0 - 1.0 / t3.compression_ratio) * 100.0);

    // -------------------------------------------------------------------------
    // Scenario 4: Pure Uniform Random Noise (No Law)
    // -------------------------------------------------------------------------
    println!(">>> SCENARIO 4: Pure Random Data (No Law / Uniform Noise)");
    let mut ys4 = Vec::with_capacity(n);
    let mut noise_rng = rebirth_symreg::evolution::Rng::new(777);
    for _ in 0..n {
        ys4.push(noise_rng.next_range_f32(-5.0, 5.0));
    }

    let mut reg4 = SymbolicRegressor::new(config1, 555);
    for _ in 0..60 {
        reg4.step_generation(&xs1, &ys4);
    }

    let t4 = reg4.get_telemetry(&xs1, &ys4).unwrap();
    let best4 = reg4.best_individual.as_ref().unwrap();
    println!("Discovered Equation:  {}", best4.expr.to_equation_string());
    println!("MSE:                  {:.6} (High residual error)", t4.mse);
    println!("Raw Array Size:       {} bytes", t4.raw_bytes);
    println!("Bytecode Size:        {} bytes", t4.bytecode_bytes);
    println!("Residuals Size:       {} bytes (residuals dominate)", t4.residual_bytes);
    println!("Total Compressed:     {} bytes", t4.total_compressed_bytes);
    println!("MEASURED RATIO:       {:.2}:1 -> HONEST FAILURE (No compression on pure random data!)\n", t4.compression_ratio);
}
