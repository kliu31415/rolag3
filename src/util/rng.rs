use std::ops::Range;

// Xoshiro, MIT License from https://github.com/Reputeless/Xoshiro-cpp/blob/master/LICENSE
pub struct Rng {
    s: [u64; 4],
}

impl Rng {
    pub fn new_seed_u64(seed: u64) -> Self {
        Self {
            s: [seed, seed, seed, seed],
        }
    }

    pub fn gen_u64(&mut self) -> u64 {
        let result = u64::wrapping_mul(u64::rotate_left(u64::wrapping_mul(self.s[1], 5), 7), 9);
        let t = self.s[1] << 17;
    
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
    
        self.s[2] ^= t;
        self.s[3] = u64::rotate_left(self.s[3], 45);
    
        result
    }

    pub fn gen_u64_range(&mut self, r: Range<u64>) -> u64 {
        assert!(r.start < r.end);
        let width = r.end - r.start;
        let mask = u64::wrapping_shr(!(0 as u64), u64::leading_zeros(width));
        loop {
            let candidate = self.gen_u64() & mask;
            if candidate < width {
                return r.start + candidate;
            }
        }
    }

    pub fn gen_usize_range(&mut self, r: Range<usize>) -> usize {
        self.gen_u64_range((r.start as u64)..(r.end as u64)) as usize
    }

    pub fn gen_u32_range(&mut self, r: Range<u32>) -> u32 {
        self.gen_u64_range((r.start as u64)..(r.end as u64)) as u32
    }

    pub fn gen_i64_range(&mut self, r: Range<i64>) -> i64 {
        assert!(r.start < r.end);
        r.start + (self.gen_u64_range(0..((r.end - r.start) as u64)) as i64)
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
        self.gen_u64() as f64 / u64::MAX as f64
    }
}