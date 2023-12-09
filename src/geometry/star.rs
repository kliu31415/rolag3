pub fn get_star_shape(num_tips: usize, inner_radius: f32, outer_radius: f32, angle: f32) -> Box<[(f32, f32)]> {
    let mut vertexes = vec![(0.0, 0.0); num_tips*2];
    for i in 0..num_tips {
        let theta = (2*i) as f32 * 2.0 * std::f32::consts::PI / (2.0 * (num_tips as f32)) + angle;
        vertexes[2*i] = (outer_radius * f32::cos(theta), outer_radius * f32::sin(theta));
        let theta = (2*i + 1) as f32 * 2.0 * std::f32::consts::PI / (2.0 * (num_tips as f32)) + angle;
        vertexes[2*i + 1] = (inner_radius * f32::cos(theta), inner_radius * f32::sin(theta));
    }
    vertexes.into_boxed_slice()
}