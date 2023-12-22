use super::shape::{Point, Vector};

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

        let a = prev - cur;
        let b = next - cur;
        let a_norm = a.norm();
        let b_norm = b.norm();
        let angle = f32::acos(Vector::dot(a, b) / (a_norm * b_norm));
        let inner_vertex_dist = border_thickness / f32::sin(angle / 2.0);
        // TODO: handle the case when angle == PI, in which case mid_vec = 0
        let mid_vec = a + b;
        let mid_vec_norm = mid_vec.norm();
        let mut multiplier = inner_vertex_dist / mid_vec_norm;
        // flip the multiplier if the cross product is negative
        if Vector::cross_product(-a, b) < 0.0 {
            multiplier *= -1.0;
        }
        inner[i] = Point::new(cur.x + mid_vec.x * multiplier, cur.y + mid_vec.y * multiplier);
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

#[cfg(test)]
mod tests {
    use crate::geometry::shape::Point;

    use super::get_inner_polygon;

    #[test]
    fn get_inner_polygon_square() {
        let vertexes = [
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(0.0, 1.0),
        ];
        let actual_inner = get_inner_polygon(0.1, &vertexes);
        let expected_inner = [
            Point::new(0.1, 0.1),
            Point::new(0.9, 0.1),
            Point::new(0.9, 0.9),
            Point::new(0.1, 0.9),
        ];
        for i in 0..4 {
            assert!((actual_inner[i] - expected_inner[i]).norm() < 1e-4, "actual({:?}) and expected({:?}) differ by too much", actual_inner[i], expected_inner[i]);
        }
    }
}