pub fn lerp_f64(x: f64, y: f64, a: f64) -> f64 {
    assert!(a>=0.0 && a<=1.0);
    x * (1.0 - a) + y * a
}

pub fn lerp_f32(x: f32, y: f32, a: f32) -> f32 {
    assert!(a>=0.0 && a<=1.0);
    x * (1.0 - a) + y * a
}