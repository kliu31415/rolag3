use super::shape::{Vector, Shape, Circle, Polygon, Point};

pub fn shapes_overlap(shape1: &Shape, shape2: &Shape) -> bool {
    match shape1 {
        Shape::Polygon(p1) => match shape2 {
            Shape::Polygon(p2) => polygon_overlaps_overlap(p1, p2),
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

fn polygon_overlaps_overlap(p1: &Polygon, p2: &Polygon) -> bool {
    // remember to iterate over the edge connecting vertexes with index n-1 and 0
    for i in 0..p1.vertexes.len() {
        let a1 = if i == 0 {p1.vertexes[p1.vertexes.len()-1]} else {p1.vertexes[i-1]};
        let a2 = p1.vertexes[i];
        for j in 0..p2.vertexes.len() {
            let b1 = if j == 0 {p2.vertexes[p2.vertexes.len()-1]} else {p2.vertexes[j-1]};
            let b2 = p2.vertexes[j];

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
        }
    }
    false
}
