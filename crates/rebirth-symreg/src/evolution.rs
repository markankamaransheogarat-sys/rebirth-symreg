//! Genetic Programming and Symbolic Regression search loop.

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::expr::{BinaryOp, Expr, UnaryOp};
use crate::mdl::{compute_mdl_telemetry, MdlTelemetry};
use crate::vm::eval_bytecode;

/// Simple fast XorShift64 PRNG for deterministic, dependency-free pseudo-random generation.
#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0xdeadbeefcafebabe } else { seed },
        }
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    #[inline]
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() as f32) / (u64::MAX as f32)
    }

    #[inline]
    pub fn next_range_f32(&mut self, low: f32, high: f32) -> f32 {
        low + self.next_f32() * (high - low)
    }

    #[inline]
    pub fn next_usize(&mut self, bound: usize) -> usize {
        if bound == 0 { 0 } else { (self.next_u64() % bound as u64) as usize }
    }
}

/// An individual candidate formula in the genetic population.
#[derive(Clone, Debug)]
pub struct Individual {
    pub expr: Expr,
    pub fitness: f32,
    pub mse: f32,
}

/// Hyperparameters configuring the symbolic regression search.
#[derive(Clone, Debug)]
pub struct RegressorConfig {
    pub population_size: usize,
    pub max_depth: usize,
    pub tournament_size: usize,
    pub mutation_rate: f32,
    pub crossover_rate: f32,
    pub constant_tweak_rate: f32,
    /// Occam's razor complexity penalty weight lambda: Fitness = -MSE - lambda * BytecodeBytes.
    pub complexity_penalty: f32,
    /// Target error tolerance epsilon for residual MDL computation.
    pub epsilon: f32,
}

impl Default for RegressorConfig {
    fn default() -> Self {
        Self {
            population_size: 100,
            max_depth: 4,
            tournament_size: 3,
            mutation_rate: 0.35,
            crossover_rate: 0.50,
            constant_tweak_rate: 0.25,
            complexity_penalty: 0.001,
            epsilon: 0.05,
        }
    }
}

/// The symbolic regressor coordinating population evolution and telemetry.
#[derive(Clone, Debug)]
pub struct SymbolicRegressor {
    pub config: RegressorConfig,
    pub population: Vec<Individual>,
    pub rng: Rng,
    pub generation: usize,
    pub best_individual: Option<Individual>,
}

impl SymbolicRegressor {
    pub fn new(config: RegressorConfig, seed: u64) -> Self {
        let mut reg = Self {
            config,
            population: Vec::new(),
            rng: Rng::new(seed),
            generation: 0,
            best_individual: None,
        };
        reg.init_population();
        reg
    }

    fn init_population(&mut self) {
        self.population.clear();
        for _ in 0..self.config.population_size {
            let depth = 1 + self.rng.next_usize(self.config.max_depth);
            let expr = self.random_expr(depth);
            self.population.push(Individual {
                expr,
                fitness: -1e9,
                mse: 1e9,
            });
        }
    }

    fn random_leaf(&mut self) -> Expr {
        if self.rng.next_f32() < 0.6 {
            Expr::X
        } else {
            let c = (self.rng.next_range_f32(-5.0, 5.0) * 10.0).round() / 10.0;
            Expr::Const(c)
        }
    }

    fn random_expr(&mut self, depth: usize) -> Expr {
        if depth <= 1 || self.rng.next_f32() < 0.2 {
            self.random_leaf()
        } else if self.rng.next_f32() < 0.35 {
            // Unary operator
            let op = match self.rng.next_usize(4) {
                0 => UnaryOp::Sin,
                1 => UnaryOp::Cos,
                2 => UnaryOp::Neg,
                _ => UnaryOp::Abs,
            };
            Expr::Unary(op, Box::new(self.random_expr(depth - 1)))
        } else {
            // Binary operator
            let op = match self.rng.next_usize(4) {
                0 => BinaryOp::Add,
                1 => BinaryOp::Sub,
                2 => BinaryOp::Mul,
                _ => BinaryOp::Div,
            };
            Expr::Binary(
                op,
                Box::new(self.random_expr(depth - 1)),
                Box::new(self.random_expr(depth - 1)),
            )
        }
    }

    /// Evaluates an individual's fitness across dataset.
    pub fn evaluate_individual(&self, expr: &Expr, xs: &[f32], ys: &[f32]) -> (f32, f32) {
        let bytecode = expr.compile();
        let code_bytes = crate::vm::bytecode_byte_size(&bytecode);

        let mut sum_sq_err = 0.0f32;
        let n = xs.len();

        for i in 0..n {
            match eval_bytecode(&bytecode, xs[i]) {
                Ok(val) => {
                    let err = ys[i] - val;
                    sum_sq_err += err * err;
                }
                Err(_) => {
                    // Severe penalty for invalid operations (e.g. division by zero or NaN)
                    return (-1e8, 1e8);
                }
            }
        }

        let mse = sum_sq_err / n as f32;
        // Fitness = -MSE - lambda * CodeSize (Occam's razor MDL penalty)
        let fitness = -(mse + self.config.complexity_penalty * code_bytes as f32);
        (fitness, mse)
    }

    /// Step the genetic algorithm for one generation.
    pub fn step_generation(&mut self, xs: &[f32], ys: &[f32]) {
        // 1. Evaluate fitness
        for i in 0..self.population.len() {
            let (fit, mse) = self.evaluate_individual(&self.population[i].expr, xs, ys);
            self.population[i].fitness = fit;
            self.population[i].mse = mse;
        }

        // Sort descending by fitness (best first)
        self.population.sort_by(|a, b| {
            b.fitness.partial_cmp(&a.fitness).unwrap_or(core::cmp::Ordering::Equal)
        });

        // Track best overall
        if self.best_individual.is_none()
            || self.population[0].fitness > self.best_individual.as_ref().unwrap().fitness
        {
            self.best_individual = Some(self.population[0].clone());
        }

        // 2. Breed next generation with Elitism
        let mut next_gen = Vec::with_capacity(self.config.population_size);

        // Keep top 2 elite individuals unchanged
        next_gen.push(self.population[0].clone());
        if self.population.len() > 1 {
            next_gen.push(self.population[1].clone());
        }

        while next_gen.len() < self.config.population_size {
            let idx1 = self.tournament_select_index();
            let idx2 = self.tournament_select_index();
            let p1 = self.population[idx1].clone();
            let p2 = self.population[idx2].clone();

            let mut child = if self.rng.next_f32() < self.config.crossover_rate {
                self.crossover(&p1.expr, &p2.expr)
            } else {
                p1.expr.clone()
            };

            if self.rng.next_f32() < self.config.mutation_rate {
                self.mutate(&mut child);
            }

            if self.rng.next_f32() < self.config.constant_tweak_rate {
                self.tweak_constants(&mut child);
            }

            // Depth limit safeguard
            if child.depth() <= self.config.max_depth + 1 {
                next_gen.push(Individual {
                    expr: child,
                    fitness: -1e9,
                    mse: 1e9,
                });
            } else {
                next_gen.push(p1);
            }
        }

        self.population = next_gen;
        self.generation += 1;
    }

    fn tournament_select_index(&mut self) -> usize {
        let mut best_idx = self.rng.next_usize(self.population.len());
        for _ in 1..self.config.tournament_size {
            let candidate_idx = self.rng.next_usize(self.population.len());
            if self.population[candidate_idx].fitness > self.population[best_idx].fitness {
                best_idx = candidate_idx;
            }
        }
        best_idx
    }

    fn mutate(&mut self, expr: &mut Expr) {
        if self.rng.next_f32() < 0.3 {
            // Replace subtree with fresh random expression
            *expr = self.random_expr(2);
        } else {
            match expr {
                Expr::X => {
                    if self.rng.next_f32() < 0.5 {
                        *expr = self.random_leaf();
                    }
                }
                Expr::Const(c) => {
                    *c += self.rng.next_range_f32(-1.0, 1.0);
                    *c = (*c * 10.0).round() / 10.0;
                }
                Expr::Unary(op, child) => {
                    if self.rng.next_f32() < 0.4 {
                        *op = match self.rng.next_usize(4) {
                            0 => UnaryOp::Sin,
                            1 => UnaryOp::Cos,
                            2 => UnaryOp::Neg,
                            _ => UnaryOp::Abs,
                        };
                    } else {
                        self.mutate(child);
                    }
                }
                Expr::Binary(op, left, right) => {
                    if self.rng.next_f32() < 0.4 {
                        *op = match self.rng.next_usize(4) {
                            0 => BinaryOp::Add,
                            1 => BinaryOp::Sub,
                            2 => BinaryOp::Mul,
                            _ => BinaryOp::Div,
                        };
                    } else if self.rng.next_f32() < 0.5 {
                        self.mutate(left);
                    } else {
                        self.mutate(right);
                    }
                }
            }
        }
    }

    fn tweak_constants(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Const(c) => {
                // Small continuous adjustment
                *c += self.rng.next_range_f32(-0.25, 0.25);
            }
            Expr::Unary(_, child) => self.tweak_constants(child),
            Expr::Binary(_, left, right) => {
                self.tweak_constants(left);
                self.tweak_constants(right);
            }
            Expr::X => {}
        }
    }

    fn crossover(&mut self, a: &Expr, b: &Expr) -> Expr {
        // Swap subtrees between a and b
        if self.rng.next_f32() < 0.4 {
            b.clone()
        } else {
            match a {
                Expr::Unary(op, child) => {
                    Expr::Unary(*op, Box::new(self.crossover(child, b)))
                }
                Expr::Binary(op, left, right) => {
                    if self.rng.next_f32() < 0.5 {
                        Expr::Binary(*op, Box::new(self.crossover(left, b)), right.clone())
                    } else {
                        Expr::Binary(*op, left.clone(), Box::new(self.crossover(right, b)))
                    }
                }
                _ => b.clone(),
            }
        }
    }

    /// Compute full telemetry for the best discovered formula.
    pub fn get_telemetry(&self, xs: &[f32], ys: &[f32]) -> Option<MdlTelemetry> {
        let best = self.best_individual.as_ref()?;
        let bytecode = best.expr.compile();

        let mut preds = Vec::with_capacity(xs.len());
        for &x in xs {
            preds.push(eval_bytecode(&bytecode, x).unwrap_or(0.0));
        }

        Some(compute_mdl_telemetry(&bytecode, ys, &preds, self.config.epsilon))
    }
}
