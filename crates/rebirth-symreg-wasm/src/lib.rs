//! WebAssembly bindings for rebirth-symreg.

use rebirth_symreg::{
    eval_bytecode, RegressorConfig, SymbolicRegressor,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmRegressor {
    regressor: SymbolicRegressor,
    xs: Vec<f32>,
    ys: Vec<f32>,
    preset_id: u32,
}

#[wasm_bindgen]
impl WasmRegressor {
    #[wasm_bindgen(constructor)]
    pub fn new(
        pop_size: usize,
        max_depth: usize,
        epsilon: f32,
        penalty: f32,
        seed: u32,
    ) -> Self {
        let config = RegressorConfig {
            population_size: pop_size,
            max_depth,
            epsilon,
            complexity_penalty: penalty,
            ..Default::default()
        };
        let regressor = SymbolicRegressor::new(config, seed as u64);
        let mut inst = Self {
            regressor,
            xs: Vec::new(),
            ys: Vec::new(),
            preset_id: 0,
        };
        inst.load_preset(0);
        inst
    }

    pub fn load_preset(&mut self, preset_id: u32) {
        self.preset_id = preset_id;
        self.xs.clear();
        self.ys.clear();

        let n = 150;
        let mut rng = rebirth_symreg::evolution::Rng::new(42);

        match preset_id {
            0 => {
                // Harmonic: y = 2.0 * sin(x) + x
                for i in 0..n {
                    let x = -3.0 + 6.0 * (i as f32) / (n as f32);
                    self.xs.push(x);
                    self.ys.push(2.0 * x.sin() + x);
                }
            }
            1 => {
                // Parabola: y = x^2 - 3.0
                for i in 0..n {
                    let x = -2.0 + 4.0 * (i as f32) / (n as f32);
                    self.xs.push(x);
                    self.ys.push(x * x - 3.0);
                }
            }
            2 => {
                // Parabola + 15% Noise (Real measurement noise)
                for i in 0..n {
                    let x = -2.0 + 4.0 * (i as f32) / (n as f32);
                    let clean = x * x - 3.0;
                    let noise = rng.next_range_f32(-0.4, 0.4);
                    self.xs.push(x);
                    self.ys.push(clean + noise);
                }
            }
            3 => {
                // Pure Random Data (No Law / Uniform noise)
                for i in 0..n {
                    let x = -3.0 + 6.0 * (i as f32) / (n as f32);
                    self.xs.push(x);
                    self.ys.push(rng.next_range_f32(-4.0, 4.0));
                }
            }
            _ => self.load_preset(0),
        }

        self.reset();
    }

    pub fn reset(&mut self) {
        let seed = self.regressor.rng.next_u64();
        let config = self.regressor.config.clone();
        self.regressor = SymbolicRegressor::new(config, seed);
    }

    pub fn step(&mut self, generations: u32) {
        for _ in 0..generations {
            self.regressor.step_generation(&self.xs, &self.ys);
        }
    }

    pub fn get_generation(&self) -> usize {
        self.regressor.generation
    }

    pub fn get_best_equation(&self) -> String {
        if let Some(ref best) = self.regressor.best_individual {
            best.expr.to_equation_string()
        } else {
            "Searching...".into()
        }
    }

    pub fn get_best_mse(&self) -> f32 {
        if let Some(ref best) = self.regressor.best_individual {
            best.mse
        } else {
            1e9
        }
    }

    pub fn get_data_x(&self) -> Vec<f32> {
        self.xs.clone()
    }

    pub fn get_data_y(&self) -> Vec<f32> {
        self.ys.clone()
    }

    pub fn get_predicted_y(&self) -> Vec<f32> {
        if let Some(ref best) = self.regressor.best_individual {
            let bytecode = best.expr.compile();
            self.xs
                .iter()
                .map(|&x| eval_bytecode(&bytecode, x).unwrap_or(0.0))
                .collect()
        } else {
            vec![0.0; self.xs.len()]
        }
    }

    /// Returns packed telemetry array:
    /// [raw_bytes, bytecode_bytes, residual_bytes, total_bytes, ratio, exact_match, mse, r_squared]
    pub fn get_telemetry_array(&self) -> Vec<f32> {
        if let Some(t) = self.regressor.get_telemetry(&self.xs, &self.ys) {
            vec![
                t.raw_bytes as f32,
                t.bytecode_bytes as f32,
                t.residual_bytes as f32,
                t.total_compressed_bytes as f32,
                t.compression_ratio,
                if t.exact_match { 1.0 } else { 0.0 },
                t.mse,
                t.r_squared,
            ]
        } else {
            vec![0.0; 8]
        }
    }
}
