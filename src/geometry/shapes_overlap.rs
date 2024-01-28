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
                if bb1.contains(bb2) {
                    return polygon_contains_polygon(p1, p2);
                } else if bb2.contains(bb1) {
                    return polygon_contains_polygon(p2, p1);
                }
                return false;
            }
            Shape::Circle(c2) => polygon_overlaps_circle(p1, c2),
        }
        Shape::Circle(c1) => match shape2 {
            Shape::Polygon(p2) => polygon_overlaps_circle(p2, c1),
            Shape::Circle(c2) => circles_overlaps_circle(c1, c2),
        }
    }
}

fn circles_overlaps_circle(c1: &Circle, c2: &Circle) -> bool {
    f32::hypot(c2.center.x - c1.center.x, c2.center.y - c1.center.y) < c1.r + c2.r
}

fn polygon_overlaps_circle(p1: &Polygon, c2: &Circle) -> bool {
    let rsq = f32::powi(c2.r, 2);
    // remember to iterate over the edge connecting vertexes with index n-1 and 0
    for i in 0..p1.vertexes.len() {
        let s = if i == 0 {p1.vertexes[p1.vertexes.len()-1]} else {p1.vertexes[i-1]};
        let e = p1.vertexes[i];
        let v = e - s;
        let f = s - Point::new(c2.center.x, c2.center.y);
        let a = Vector::dot(v, v);
        let b = 2.0 * Vector::dot(v, f);
        let c = Vector::dot(f, f) - rsq;
        let discriminant = f32::powi(b, 2) - 4.0 * a * c;
        // ignore the case where discriminant == 0, because it's ok to treat case that as non-intersecting
        if discriminant > 0.0 {
            let n1 = -b;
            let n2 = f32::sqrt(discriminant);
            let d = 2.0 * a;
            let r1 = (n1 - n2) / d;
            let r2 = (n1 + n2) / d;
            if (r1>=0.0 && r1<=1.0) || (r2>=0.0 && r2<=1.0) {
                return true;
            } 
        }
    }
    false
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
fn polygon_contains_polygon(p1: &Polygon, p2: &Polygon) -> bool {
    let b1 = p2.vertexes[0];
    let b2 = Point::new(1000.0, 1.0);
    for i in 0..p1.vertexes.len() {
        let a1 = if i == 0 {p1.vertexes[p1.vertexes.len()-1]} else {p1.vertexes[i-1]};
        let a2 = p1.vertexes[i];

        if line_segments_overlap(a1, a2, b1, b2) {
            return true;
        }
    }
    return false;
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