use crate::rolag3::floor::floor_object::floor_object::FloorObjectId;

use super::shape::Shape;

#[derive(Debug, Copy, Clone)]
pub enum RofizObjTypeIdx {
    BasicWall = 1,
    Projectile = 2,
    SpectralUnit = 3,
    NonspectralUnit = 4,
}

pub type RofizObjId = usize;

#[derive(Debug, Copy, Clone)]
pub struct RofizObjectRef {
    pub type_idx: RofizObjTypeIdx, //bijects with TYPE_IDX constants
    pub id: RofizObjId,
}

pub enum RofizObject {
    BasicWall(RofizObjBasicWall),
    BasicProjectile(RofizObjMovable),
    SpectralUnit(RofizObjMovable),
    NonspectralUnit(RofizObjMovable),
}

pub struct RofizObjBasicWall {
    // (x, y) is the top left corner of the wall. The wall is a unit square.
    pub x: u32,
    pub y: u32,

    pub floor_object_id: FloorObjectId,

    pub shape: Shape,
}

pub struct RofizObjMovable {
    pub current: Hitbox,
    pub movement: RofizObjectMovement,
    pub floor_object_id: FloorObjectId,

    pub temp_hitbox: Shape, // only used as a temporary cache in move_objects_and_find_collisions()
    pub move_successful: bool // only used in move_objects_and_find_collisions()
}

pub enum RofizObjectMovement {
    NoMove(),
    Move(Transformation),
    NewHitbox(Hitbox),
}

#[derive(Debug, Copy, Clone)]
pub struct Transformation {
    pub dx: f64,
    pub dy: f64,
    pub dtheta: f64,
}

impl Transformation {
    pub fn new(dx: f64, dy: f64, dtheta: f64) -> Self {
        Self {dx, dy, dtheta}
    }
    pub fn get_transformed_shape(&self, shape: &Shape) -> Shape {
        match shape {
            Shape::Circle(c) => Shape::of_circle(c.x + self.dx as f32, c.y + self.dy as f32, c.r),
            Shape::Polygon(ref p) => Shape::Polygon(p.rotated_and_translated(self.dtheta as f32, self.dx as f32, self.dy as f32)),
        }
    }
    pub fn add(&self, rhs: &Transformation) -> Transformation {
        Transformation { 
            dx: self.dx + rhs.dx, 
            dy: self.dy + rhs.dy, 
            dtheta: self.dtheta + rhs.dtheta, 
        }
    }
}

pub struct Hitbox {
    pub transformation: Transformation,
    pub shape: Shape,
}

impl Hitbox {
    pub fn new(transformation: Transformation, shape: Shape) -> Self {
        Self {
            transformation,
            shape,
        }
    }
    pub fn empty() -> Self {
        Self {
            transformation: Transformation{dx: 0.0, dy: 0.0, dtheta: 0.0}, 
            shape: Shape::of_circle(0.0, 0.0, 0.0)
        }
    }
}