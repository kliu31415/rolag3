#[derive(Debug, Clone)]
pub enum Shape {
    Polygon(Polygon),
    Circle(Circle),
}

#[derive(Debug, Clone)]
pub struct Polygon {
    pub vertexes: Box<[Point]>,
    pub bounding_box: BoundingBox,
}

impl Polygon {
    pub fn new(vertexes: Box<[Point]>) -> Polygon {
        let mut bounding_box = BoundingBox {
            x1: f32::MAX,
            x2: f32::MIN,
            y1: f32::MAX,
            y2: f32::MIN,
        };
        for v in vertexes.iter() {
            bounding_box.x1 = f32::min(bounding_box.x1, v.x);
            bounding_box.x2 = f32::max(bounding_box.x2, v.x);
            bounding_box.y1 = f32::min(bounding_box.y1, v.y);
            bounding_box.y2 = f32::max(bounding_box.y2, v.y);
        }
        Self {vertexes, bounding_box}
    }

    pub fn rotated_and_translated(&self, dx: f32, dy: f32, dtheta: f32) -> Self {
        let mut new_v = self.vertexes.clone();

        let cos_theta = f32::cos(dtheta);
        let sin_theta = f32::sin(dtheta);
        for p in new_v.iter_mut() {
            let rot_x = cos_theta * p.x - sin_theta * p.y;
            let rot_y = sin_theta * p.x + cos_theta * p.y;
            p.x = dx + rot_x;
            p.y = dy + rot_y;
        }

        Self::new(new_v)
    }
}

impl Shape {
    pub fn of_polygon(vertexes: Box<[Point]>) -> Shape {
        Shape::Polygon(Polygon::new(vertexes))
    }

    pub fn of_rect(rect: Rect) -> Shape {
        let vertexes = [
            Point::new(rect.x, rect.y),
            Point::new(rect.x + rect.w, rect.y),
            Point::new(rect.x + rect.w, rect.y + rect.h),
            Point::new(rect.x, rect.y + rect.h),
        ];
        Shape::Polygon(Polygon::new(Box::new(vertexes)))
    }

    pub fn of_square(x: f32, y: f32, s: f32) -> Shape {
        let vertexes = [
            Point::new(x, y),
            Point::new(x + s, y),
            Point::new(x + s, y + s),
            Point::new(x, y + s),
        ];
        Shape::Polygon(Polygon::new(Box::new(vertexes)))
    }

    pub fn of_circle(x: f32, y: f32, r: f32) -> Shape {
        Shape::Circle(Circle::new(x, y, r))
    }

    pub fn dummy() -> Shape {
        Shape::Circle(Circle::new(0.0, 0.0, 0.0))
    }
}

pub fn f32pairs_to_points(vertexes: Box<[(f32, f32)]>) -> Box<[Point]> {
    vertexes.iter().map(|v| Point::new(v.0, v.1)).collect()
}

#[derive(Debug, Copy, Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self {x, y}
    }
}

impl std::ops::Sub<Point> for Point {
    type Output = Vector;

    fn sub(self, rhs: Point) -> Vector {
        Vector::new(self.x - rhs.x, self.y - rhs.y)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
}

impl Vector {
    pub fn new(x: f32, y: f32) -> Self {
        Self {x, y}
    }
    pub fn cross_product(v1: Vector, v2: Vector) -> f32 {
        v1.x * v2.y - v2.x * v1.y
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {x, y, w, h}
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BoundingBox {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BoundingBox {
    pub fn overlap(b1: &BoundingBox, b2: &BoundingBox) -> bool {
        ((b2.x1 >= b1.x1 && b2.x1 <= b1.x2) || (b1.x1 >= b2.x1 && b1.x1 <= b2.x2)) &&
        ((b2.y1 >= b1.y1 && b2.y1 <= b1.y2) || (b1.y1 >= b2.y1 && b1.y1 <= b2.y2))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Circle {
    pub r: f32,
    pub x: f32,
    pub y: f32,
}

impl Circle {
    fn new(r: f32, x: f32, y: f32) -> Self {
        Self {r, x, y}
    }
}