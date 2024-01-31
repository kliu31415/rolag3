use super::shape::{Vector, Shape, Circle, Polygon, Point, BoundingBox};

pub fn shapes_overlap(shape1: &Shape, bb1: &BoundingBox, shape2: &Shape, bb2: &BoundingBox) -> bool {
    if !BoundingBox::overlap(bb1, bb2) {
        return false;
    }

    match shape1 {
        Shape::Polygon(p1) => match shape2 {
            Shape::Polygon(p2) => {
                if polygon_edges_overlap(p1, p2) {
                    return true;
                }
                if bb1.contains(bb2) && polygon_contains_point(p1, bb1, p2.vertexes[0]) {
                    return true;
                } else if bb2.contains(bb1) && polygon_contains_point(p2, bb2, p1.vertexes[0]) {
                    return true;
                }
                return false;
            }
            Shape::Circle(c2) => polygon_overlaps_circle(p1, bb1, c2),
        }
        Shape::Circle(c1) => match shape2 {
            Shape::Polygon(p2) => polygon_overlaps_circle(p2, bb2, c1),
            Shape::Circle(c2) => circles_overlaps_circle(c1, c2),
        }
    }
}

fn circles_overlaps_circle(c1: &Circle, c2: &Circle) -> bool {
    f32::hypot(c2.center.x - c1.center.x, c2.center.y - c1.center.y) < c1.r + c2.r
}

fn polygon_overlaps_circle(p1: &Polygon, bb1: &BoundingBox, c2: &Circle) -> bool {
    let rsq = f32::powi(c2.r, 2);
    // this loop checks if the circle intersects a polygon edge or the circle contains the polygon
    for i in 0..p1.vertexes.len() {
        let v1 = if i == 0 {p1.vertexes[p1.vertexes.len()-1]} else {p1.vertexes[i-1]};
        let v2 = p1.vertexes[i];
        if line_segment_to_point_dist_sq(v1, v2, c2.center) <= rsq {
            return true;
        }
    }

    // remember to also check if the polygon contains the circle
    polygon_contains_point(p1, bb1, c2.center)
}

fn line_segment_to_point_dist_sq(v1: Point, v2: Point, p: Point) -> f32 {
    let segment_len_sq = (v1 - v2).norm_sq();
    if segment_len_sq == 0.0 {
        return (v1 - p).norm_sq();
    }

    let t = f32::clamp(Vector::dot(v2 - v1, p - v1) / segment_len_sq, 0.0, 1.0);
    let projection = v1 + (v2 - p) * t;
    (projection - p).norm_sq()
}

fn polygon_edges_overlap(p1: &Polygon, p2: &Polygon) -> bool {
    for i in 0..p1.vertexes.len() {
        let a1 = if i == 0 {p1.vertexes[p1.vertexes.len()-1]} else {p1.vertexes[i-1]};
        let a2 = p1.vertexes[i];
        for j in 0..p2.vertexes.len() {
            let b1 = if j == 0 {p2.vertexes[p2.vertexes.len()-1]} else {p2.vertexes[j-1]};
            let b2 = p2.vertexes[j];

            if line_segments_overlap(a1, a2, b1, b2) {
                return true;
            }
        }
    }
    false
}

// TODO: verify this function works
pub fn polygon_contains_point(polygon: &Polygon, bb: &BoundingBox, point: Point) -> bool {
    let b1 = point;
    // 0.8 and 0.7 are arbitrary. We just need the other point to be outside the p1's bounding box
    let b2 = Point::new(bb.x2 + 0.8, bb.y2 + 0.7);
    let mut num_intersections = 0;
    for i in 0..polygon.vertexes.len() {
        let a1 = if i == 0 {polygon.vertexes[polygon.vertexes.len()-1]} else {polygon.vertexes[i-1]};
        let a2 = polygon.vertexes[i];

        if line_segments_overlap(a1, a2, b1, b2) {
            num_intersections += 1;
        }
    }
    return num_intersections % 2 == 1;
}

fn line_segments_overlap(a1: Point, a2: Point, b1: Point, b2: Point) -> bool {
    let v1 = b1 - a1;
    let v2 = a2 - b1;
    let v3 = b2 - a2;
    let v4 = a1 - b2;

    let cp1 = Vector::cross_product(v1, v2);
    let cp2 = Vector::cross_product(v2, v3);
    let cp3 = Vector::cross_product(v3, v4);
    let cp4 = Vector::cross_product(v4, v1);

    if cp1 < 0.0 && cp2 < 0.0 && cp3 < 0.0 && cp4 < 0.0 {
        return true;
    }
    if cp1 > 0.0 && cp2 > 0.0 && cp3 > 0.0 && cp4 > 0.0 {
        return true;
    }
    return false;
}