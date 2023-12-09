/*  Takes in a polygon A with vertices in CCW order. 
    Returns an inner polygon B whose edges are exactly border_thickness away from A's edges.
    This function fails on some edge cases. 
    Currently, it only handles the case where the inner polygon has the same number of vertices.
*/
pub fn _get_inner_polygon(border_thickness: f32, vertexes: &[(f32, f32)]) -> Box<[(f32, f32)]> {
    let mut inner = vec![(0.0, 0.0); vertexes.len()];
    for i in 0..vertexes.len() {
        let prev: (f32, f32);
        if i > 0 {
            prev = vertexes[i-1];
        } else {
            prev = vertexes[vertexes.len()-1];
        }

        let next: (f32, f32);
        if i + 1 != vertexes.len() {
            next = vertexes[i+1];
        } else {
            next = vertexes[0];
        }

        let cur = vertexes[i];

        let a = (prev.0 - cur.0, prev.1 - cur.1);
        let b = (next.0 - cur.0, next.1 - cur.1);
        let a_norm = f32::hypot(a.0, a.1);
        let b_norm = f32::hypot(b.0, b.1);
        let angle = (a.0*b.0 + a.1*b.1) / (a_norm * b_norm);
        let inner_vertex_dist = border_thickness / f32::sin(angle / 2.0);
        // TODO: handle the case when angle == PI, in which case mid_vec = 0
        let mid_vec = (a.0 + b.0, a.1 + b.1);
        let mid_vec_norm = f32::hypot(mid_vec.0, mid_vec.1);
        let mut multiplier = inner_vertex_dist / mid_vec_norm;
        if angle > std::f32::consts::PI {
            multiplier *= -1.0;
        }
        inner.push((cur.0 + mid_vec.0 * multiplier, cur.1 + mid_vec.1 * multiplier));
    }
    inner.into_boxed_slice()
}