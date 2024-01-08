use crate::util::lerp::lerp_f32;

#[derive(Debug, Clone)]
pub enum Shape {
    Polygon(Polygon),
    Circle(Circle),
}

impl Default for Shape {
    fn default() -> Self {
        Shape::Circle(Circle { center: Point::default(), r: f32::default() })
    }
}

#[derive(Debug, Clone)]
pub struct Polygon {
    pub vertexes: Box<[Point]>,
}

impl Polygon {
    pub fn new(vertexes: Box<[Point]>) -> Polygon {
        Self {vertexes}
    }

    pub fn new_from_slice(vertexes: &[Point]) -> Polygon {
        let mut v = vec![Point::default(); vertexes.len()].into_boxed_slice();
        v.copy_from_slice(vertexes);
        Self { vertexes: v }
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

    pub fn replace_with_rotated_and_translated(&mut self, src: &Polygon, dx: f32, dy: f32, dtheta: f32) {
        if src.vertexes.len() != self.vertexes.len() {
            self.vertexes = vec![Point::default(); src.vertexes.len()].into();
        }
        self.vertexes.copy_from_slice(&src.vertexes);
        // most of the time, dtheta is 0, so I decided to handle this as a special case as an optimization
        if dtheta == 0.0 {
            for p in self.vertexes.iter_mut() {
                p.x += dx;
                p.y += dy;
            }
        } else {
            let cos_theta = f32::cos(dtheta);
            let sin_theta = f32::sin(dtheta);
            for p in self.vertexes.iter_mut() {
                let rot_x = cos_theta * p.x - sin_theta * p.y;
                let rot_y = sin_theta * p.x + cos_theta * p.y;
                p.x = dx + rot_x;
                p.y = dy + rot_y;
            }
        }
    }

    pub fn replace_with_vertexes(&mut self, vertexes: &[Point]) {
        if self.vertexes.len() != vertexes.len() {
            self.vertexes = vec![Point::default(); vertexes.len()].into();
        }
        self.vertexes.copy_from_slice(vertexes);
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

    pub fn of_circle(center: Point, r: f32) -> Shape {
        Shape::Circle(Circle::new(center, r))
    }

    pub fn dummy() -> Shape {
        Shape::Circle(Circle::new(Point::new(0.0, 0.0), 0.0))
    }

    pub fn replace_with_polygon(&mut self, vertexes: &[Point]) {
        match self {
            Shape::Polygon(ref mut p) => p.replace_with_vertexes(vertexes),
            Shape::Circle(_) => *self = Shape::Polygon(Polygon::new_from_slice(vertexes)),
        };
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
    pub const fn new(x: f32, y: f32) -> Self {
        Self {x, y}
    }
    pub fn lerp(u: Point, v: Point, a: f32) -> Point {
        Self {
            x: lerp_f32(u.x, v.x, a),
            y: lerp_f32(u.y, v.y, a),
        }
    }

    pub fn rotated(&self, theta: f32) -> Self {
        let cos_theta = f32::cos(theta);
        let sin_theta = f32::sin(theta);
        Point::new(cos_theta * self.x - sin_theta * self.y, sin_theta * self.x + cos_theta * self.y)
    }

    pub fn translated(&self, d: Vector) -> Self {
        Point::new(self.x + d.x, self.y + d.y)
    }
}

impl Default for Point {
    fn default() -> Self {
        Self { x: Default::default(), y: Default::default() }
    }
}

impl std::ops::Sub<Point> for Point {
    type Output = Vector;

    fn sub(self, rhs: Point) -> Vector {
        Vector::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl std::ops::Add<Vector> for &Point {
    type Output = Point;

    fn add(self, rhs: Vector) -> Point {
        Point::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Add<Vector> for Point {
    type Output = Point;

    fn add(self, rhs: Vector) -> Point {
        Point::new(self.x + rhs.x, self.y + rhs.y)
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

    pub fn dot(v1: Vector, v2: Vector) -> f32 {
        v1.x * v2.x + v1.y * v2.y
    }

    pub fn norm(&self) -> f32 {
        f32::hypot(self.x, self.y)
    }

    pub fn normalized(&self) -> Self {
        let n = self.norm();
        Self {
            x: self.x / n,
            y: self.y / n,
        }
    }

    pub fn rotated(&self, theta: f32) -> Self {
        let cos_theta = f32::cos(theta);
        let sin_theta = f32::sin(theta);
        Vector::new(cos_theta * self.x - sin_theta * self.y, sin_theta * self.x + cos_theta * self.y)
    }
}

impl std::ops::Add<Vector> for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Vector {
        Vector::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Neg for Vector {
    type Output = Vector;

    fn neg(self) -> Self {
        Vector::new(-self.x, -self.y)
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
    pub x1: f32,
    pub x2: f32,
    pub y1: f32,
    pub y2: f32,
}

impl BoundingBox {
    pub fn zero_state() -> Self {
        Self {
            x1: f32::MAX,
            x2: f32::MIN,
            y1: f32::MAX,
            y2: f32::MIN,
        }
    }
    pub fn of_polygon(p: &Polygon) -> Self {
        let mut bounding_box = Self {
            x1: f32::MAX,
            x2: f32::MIN,
            y1: f32::MAX,
            y2: f32::MIN,
        };
        for v in p.vertexes.iter() {
            bounding_box.x1 = f32::min(bounding_box.x1, v.x);
            bounding_box.x2 = f32::max(bounding_box.x2, v.x);
            bounding_box.y1 = f32::min(bounding_box.y1, v.y);
            bounding_box.y2 = f32::max(bounding_box.y2, v.y);
        }
        bounding_box
    }

    pub fn of_circle(c: &Circle) -> Self {
        Self {
            x1: c.center.x - c.r,
            x2: c.center.x + c.r,
            y1: c.center.y - c.r,
            y2: c.center.y + c.r,
        }
    }

    pub fn of_shape(s: &Shape) -> Self {
        match s {
            Shape::Polygon(p) => Self::of_polygon(p),
            Shape::Circle(c) => Self::of_circle(c),
        }
    }

    pub fn combine(&mut self, other: &BoundingBox) {
        self.x1 = f32::min(self.x1, other.x1); 
        self.x2 = f32::max(self.x2, other.x2);
        self.y1 = f32::min(self.y1, other.y1); 
        self.y2 = f32::max(self.y2, other.y2);
    }

    pub fn overlap(b1: &BoundingBox, b2: &BoundingBox) -> bool {
        ((b2.x1 >= b1.x1 && b2.x1 <= b1.x2) || (b1.x1 >= b2.x1 && b1.x1 <= b2.x2)) &&
        ((b2.y1 >= b1.y1 && b2.y1 <= b1.y2) || (b1.y1 >= b2.y1 && b1.y1 <= b2.y2))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Circle {
    pub center: Point,
    pub r: f32,
}

impl Circle {
    pub const fn new(center: Point, r: f32) -> Self {
        Self {center, r}
    }
}