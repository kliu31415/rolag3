use super::shape::Point;

/*  Takes in a polygon A with vertices in CCW order. 
    Returns an inner polygon B whose edges are exactly border_thickness away from A's edges.
    This function fails on some edge cases. 
    Currently, it only handles the case where the inner polygon has the same number of vertices.
*/
pub fn get_inner_polygon(border_thickness: f32, vertexes: &[Point]) -> Box<[Point]> {
    let mut inner = vec![Point::new(0.0, 0.0); vertexes.len()];
    for i in 0..vertexes.len() {
        let prev: Point;
        if i > 0 {
            prev = vertexes[i-1];
        } else {
            prev = vertexes[vertexes.len()-1];
        }

        let next: Point;
        if i + 1 != vertexes.len() {
            next = vertexes[i+1];
        } else {
            next = vertexes[0];
        }

        let cur = vertexes[i];

        let a = (prev.x - cur.x, prev.y - cur.y);
        let b = (next.x - cur.x, next.y - cur.y);
        let a_norm = f32::hypot(a.0, a.1);
        let b_norm = f32::hypot(b.0, b.1);
        let angle = f32::acos((a.0*b.0 + a.1*b.1) / (a_norm * b_norm));
        let inner_vertex_dist = border_thickness / f32::sin(angle / 2.0);
        // TODO: handle the case when angle == PI, in which case mid_vec = 0
        let mid_vec = (a.0 + b.0, a.1 + b.1);
        let mid_vec_norm = f32::hypot(mid_vec.0, mid_vec.1);
        let mut multiplier = inner_vertex_dist / mid_vec_norm;
        if angle > std::f32::consts::PI {
            multiplier *= -1.0;
        }
        inner[i] = Point::new(cur.x + mid_vec.0 * multiplier, cur.y + mid_vec.1 * multiplier);
    }
    inner.into_boxed_slice()
}

pub fn regular_polygon(num_sides: usize, radius: f32) -> Box<[Point]> {
    let mut vertexes = vec![Point::new(0.0, 0.0); num_sides];
    for i in 0..num_sides {
        let angle = (i as f32) / (num_sides as f32) * 2.0 * std::f32::consts::PI;
        vertexes[i] = Point::new(radius * f32::cos(angle), radius * f32::sin(angle));
    }
    vertexes.into_boxed_slice()
}