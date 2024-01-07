use std::ops::Range;

use rand::{SeedableRng, Rng};

use rand_distr::Distribution;

pub struct Prng {
    state: rand_chacha::ChaCha8Rng,
}

impl Prng {
    pub fn new_seed_u64(seed: u64) -> Self {
        Self {
            state: rand_chacha::ChaCha8Rng::seed_from_u64(seed),
        }
    }

    pub fn gen_u64_range(&mut self, r: Range<u64>) -> u64 {
        self.state.gen_range(r)
    }

    pub fn gen_usize_range(&mut self, r: Range<usize>) -> usize {
        self.state.gen_range(r)
    }

    pub fn gen_u32_range(&mut self, r: Range<u32>) -> u32 {
        self.state.gen_range(r)
    }

    pub fn gen_i64_range(&mut self, r: Range<i64>) -> i64 {
        self.state.gen_range(r)
    }

    pub fn sample_weighted_iter_f64(&mut self, weights: &[f64]) -> usize {
        let sum: f64 = weights.iter().sum();
        let mut cur = self.gen_f64() * sum;
        for i in 0..weights.len() {
            assert!(weights[i] >= 0.0);
            cur -= weights[i];
            if cur < 0.0 {
                return i;
            }
        }
        return weights.len() - 1;
    }

    // [0..1)
    pub fn gen_f64(&mut self) -> f64 {
        self.state.gen_range(0.0..1.0)
    }

    pub fn gen_normal(&mut self, mean: f64, sd: f64) -> f64 {
        let normal = rand_distr::Normal::new(mean, sd).unwrap();
        normal.sample(&mut self.state)
    }
}