use super::shape::{Vector, Shape, Circle, Polygon, BoundingBox};

pub fn shapes_overlap(shape1: &Shape, shape2: &Shape) -> bool {
    match shape1 {
        Shape::Polygon(p1) => match shape2 {
            Shape::Polygon(p2) => polygon_overlaps_overlap(p1, p2),
            Shape::Circle(c2) => circle_overlaps_polygon(c2, p1),
        }
        Shape::Circle(c1) => match shape2 {
            Shape::Polygon(p2) => circle_overlaps_polygon(c1, p2),
            Shape::Circle(c2) => circles_overlaps_circle(c1, c2),
        }
    }
}

fn circles_overlaps_circle(c1: &Circle, c2: &Circle) -> bool {
    f32::hypot(c2.x - c1.x, c2.y - c1.y) < c1.r + c2.r
}

fn circle_overlaps_polygon(_c1: &Circle, _p2: &Polygon) -> bool {
    todo!();
}

fn polygon_overlaps_overlap(p1: &Polygon, p2: &Polygon) -> bool {
    if !BoundingBox::overlap(&p1.bounding_box, &p2.bounding_box) {
        return false;
    }
    
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
