use std::rc::Rc;

use crate::rolag3::floor::room_object::room_object_def::RoomObjectId;

use super::shape::Shape;

pub type RofizObjId = usize;

// external_ref_count is 1 greater than the number of external references. Rofiz itself will never create additional
// Rcs to external_ref_count. When external_ref_count reaches 1 (i.e. no external references are left), the rofiz object 
// is deleted.
pub struct RofizObjBasicWall {
    pub id: RofizObjId,
    pub external_ref_count: Rc<()>,

    // (x, y) is the top left corner of the wall. The wall is a unit square.
    pub x: u32,
    pub y: u32,

    pub floor_object_id: RoomObjectId,

    pub shape: Shape,
}

pub struct RofizObjMovable {
    pub id: RofizObjId,
    pub external_ref_count: Rc<()>,

    pub current: Hitbox,
    pub movement: RofizObjectMovement,
    pub move_with_fallbacks_idx: usize,
    pub room_object_id: RoomObjectId,

    // only used in move_objects_and_find_collisions()
    pub initial_hitbox: Shape,
    pub temp_hitbox: Shape,
    pub move_successful: bool,
    pub fallback_idx: usize,
}

pub enum RofizObjectMovement {
    NoMove(),
    Move(Transformation),
    _MoveWithFallbacks(Vec<Transformation>),
    _NewHitbox(Hitbox),
    Delete(),
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
}