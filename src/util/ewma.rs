pub struct Ewma {
    val: f64,
    decay_mult: f64,
    last_time_used: f64,
}

impl Ewma {
    pub fn new(decay_mult: f64) -> Self {
        Self {
            val: 0.0,
            decay_mult,
            last_time_used: 0.0,
        }
    }

    pub fn add(&mut self, t: f64, v: f64) {
        let time_diff = t - self.last_time_used;
        assert!(time_diff >= 0.0);
        self.val *= f64::powf(self.decay_mult, time_diff);
        self.val += v;
        self.last_time_used = t;
    }

    pub fn get(&mut self, t: f64) -> f64 {
        let time_diff = t - self.last_time_used;
        assert!(time_diff >= 0.0);
        self.val *= f64::powf(self.decay_mult, time_diff);
        self.last_time_used = t;
        self.val
    }

    pub fn get_decay_mult(&self) -> f64 {
        self.decay_mult
    }
}