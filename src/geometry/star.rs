use super::shape::{Point, Vector};

pub fn get_star_shape(num_tips: usize, inner_radius: f32, outer_radius: f32, angle: f32) -> Box<[Point]> {
    let mut vertexes = vec![Point::new(0.0, 0.0); num_tips*2];
    get_star_shape_in_dst(vertexes.as_mut_slice(), num_tips, inner_radius, outer_radius, angle);
    vertexes.into_boxed_slice()
}

pub fn get_star_shape_in_dst(dst: &mut [Point], num_tips: usize, inner_radius: f32, outer_radius: f32, angle: f32) {
    assert!(inner_radius > 0.0);
    assert!(outer_radius > inner_radius);
    assert_eq!(dst.len(), 2 * num_tips);
    
    for i in 0..num_tips {
        let theta = (2*i) as f32 * 2.0 * std::f32::consts::PI / (2.0 * (num_tips as f32)) + angle;
        dst[2*i] = Point::new(outer_radius * f32::cos(theta), outer_radius * f32::sin(theta));
        let theta = (2*i + 1) as f32 * 2.0 * std::f32::consts::PI / (2.0 * (num_tips as f32)) + angle;
        dst[2*i + 1] = Point::new(inner_radius * f32::cos(theta), inner_radius * f32::sin(theta));
    }
}

pub fn get_blunt_star(num_tips: usize, inner_radius: f32, outer_radius: f32, angle: f32) -> Box<[Point]> {
    assert!(inner_radius > 0.0);
    assert!(outer_radius > inner_radius);

    let v1 = Point::new(inner_radius, 0.0);
    let theta = std::f32::consts::PI - (0.5 * (num_tips - 2) as f32 * std::f32::consts::PI / (num_tips as f32) + std::f32::consts::FRAC_PI_2);
    let vector = Vector::new(f32::cos(theta), f32::sin(theta)).normalized();
    let t = -inner_radius * vector.x + f32::sqrt(f32::powi(outer_radius, 2) - f32::powi(inner_radius * vector.y, 2));
    assert!(f32::is_normal(t));
    assert!(t > 0.0);
    let side_angle = 2.0 * std::f32::consts::PI / (num_tips as f32);
    let v2 = &v1 + Vector::new(t * vector.x, t * vector.y);
    let v3 = &v2 + Vector::new(inner_radius as f32 * f32::cos(side_angle) - inner_radius, inner_radius as f32 * f32::sin(side_angle));

    let mut vertexes = vec![Point::new(0.0, 0.0); num_tips*3];
    for i in 0..num_tips {
        let theta = angle + (i as f32 / num_tips as f32) * (2.0 * std::f32::consts::PI);
        vertexes[3*i] = v1.rotated(theta);
        vertexes[3*i + 1] = v2.rotated(theta);
        vertexes[3*i + 2] = v3.rotated(theta);
    }
    vertexes.into()
}