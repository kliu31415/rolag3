use crate::{geometry::shape::{Vector, Point}, util::{lerp::lerp_f64, rng::Prng}};

// x ranges from [0, 1]
#[derive(Debug, Clone)]
pub struct ChunkedBrownianBridge {
    x: Vec<f64>,
    y: Vec<f64>,
}

impl ChunkedBrownianBridge {
    pub fn new(rng: &mut Prng, num_chunks: usize, len_max_ratio: f64, y_sd: f64) -> Self {
        assert!(len_max_ratio >= 1.0);
        assert!(num_chunks > 0);
    
        let mut x: Vec<f64> = Vec::new();
        x.resize(num_chunks, 0.0);
        let mut total_len = 0.0;
        for i in 0..num_chunks {
            x[i] = (len_max_ratio - 1.0) * rng.gen_f64() + 1.0;
            total_len += x[i];
        }
        x.iter_mut().for_each(|x| *x /= total_len);
    
        let mut y = Vec::new();
        y.resize(num_chunks, 0.0);
        for i in 0..num_chunks {
            let prev = if i > 0 {y[i-1]} else {0.0};
            // TODO: sample from a real gaussian
            let diff = rng.gen_normal(0.0, y_sd * f64::sqrt(x[i]));
            y[i] = prev + diff;
        }

        // convert distances between consecutive xs into prefix sums
        for i in 1..num_chunks {
            x[i] += x[i-1];
            x[i] = f64::min(1.0, x[i]); // the sum can go slightly over 1.0 due to floating point error
        }
    
        let ysum = y[num_chunks-1];
        for i in 0..num_chunks {
            y[i] -= x[i] * ysum;
        }

        assert!(x.len() > 0);
        assert!(x.len() == y.len());
        
        // x and y are always the same length.
        // It always holds that y[num_chunks-1] = 0, which represents the end of the brownian bridge.
        // However, the beginning of the brownian bridge (0, 0) isn't represented in the x or y arrays
        Self {
            x,
            y,
        }
    }

    pub fn lerp(a: &ChunkedBrownianBridge, b: &ChunkedBrownianBridge, lerp_t: f64) -> Self {
        let mut a_idx = 0;
        let mut b_idx = 0;
        let mut lerped_x = Vec::new();
        let mut lerped_y = Vec::new();
        lerped_x.reserve(a.x.len() + b.x.len());
        lerped_y.reserve(a.x.len() + b.x.len());
        while a_idx < a.x.len() || b_idx < b.x.len() {
            if b_idx == b.x.len() || (a_idx < a.x.len() && a.x[a_idx] < b.x[b_idx]) {
                lerped_x.push(a.x[a_idx]);
                let b1_x = if b_idx > 0 {b.x[b_idx - 1]} else {0.0};
                let b1_y = if b_idx > 0 {b.y[b_idx - 1]} else {0.0};
                let b2_x = if b_idx < b.x.len() {b.x[b_idx]} else {1.0};
                let b2_y = if b_idx < b.x.len() {b.y[b_idx]} else {0.0};
                assert!(b1_x <= b2_x, "expected {} <= {}", b1_x, b2_x);
                assert!(b1_x <= a.x[a_idx], "expected {} <= {}", b1_x, a.x[a_idx]);
                assert!(a.x[a_idx] <= b2_x, "expected {} <= {}", a.x[a_idx], b2_x);
                if b1_x == b2_x {
                    lerped_y.push(a.y[a_idx]);
                } else {
                    let b_y = lerp_f64(b1_y, b2_y, (a.x[a_idx] - b1_x) / (b2_x - b1_x));
                    lerped_y.push(lerp_f64(a.y[a_idx], b_y, lerp_t));
                }
                a_idx += 1;
            } else {
                lerped_x.push(b.x[b_idx]);
                let a1_x = if a_idx > 0 {a.x[a_idx - 1]} else {0.0};
                let a1_y = if a_idx > 0 {a.y[a_idx - 1]} else {0.0};
                let a2_x = if a_idx < a.x.len() {a.x[a_idx]} else {1.0};
                let a2_y = if a_idx < a.x.len() {a.y[a_idx]} else {0.0};
                assert!(a1_x <= a2_x, "expected {} <= {}", a1_x, a2_x);
                assert!(a1_x <= b.x[b_idx], "expected {} <= {}", a1_x, b.x[b_idx]);
                assert!(b.x[b_idx] <= a2_x, "expected {} <= {}", b.x[b_idx], a2_x);
                if a1_x == a2_x {
                    lerped_y.push(b.y[b_idx]);
                } else {
                    let a_y = lerp_f64(a1_y, a2_y, (b.x[b_idx] - a1_x) / (a2_x - a1_x));
                    lerped_y.push(lerp_f64(b.y[b_idx], a_y, 1.0 - lerp_t));
                }
                b_idx += 1;
            }
        }

        Self {
            x: lerped_x,
            y: lerped_y,
        }
    }

    pub fn to_quads(&self, dst: &mut Vec<[Point; 4]>, thickness: f32, start: Point, end: Point) {
        let mut prev = Point::new(0.0, 0.0);
        let dir = end - start;
        let dir_r = dir.norm();
        let dir_theta = f32::atan2(dir.y, dir.x);
        let start_xlate = Vector::new(start.x, start.y);
        for (x, y) in self.x.iter().zip(self.y.iter()) {
            let x = *x as f32;
            let y = *y as f32;
            let mut quad = [
                Point::new(prev.x * dir_r, prev.y - thickness / 2.0),
                Point::new(prev.x * dir_r, prev.y + thickness / 2.0),
                Point::new(x * dir_r, y + thickness / 2.0),
                Point::new(x * dir_r, y - thickness / 2.0),
            ];
            quad.iter_mut().for_each(|p| *p = p.rotated(dir_theta).translated(start_xlate));
            dst.push(quad);
            prev = Point::new(x, y);
        }
    }
}