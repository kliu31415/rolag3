use std::sync::Arc;

use crate::{rolag3::floor::room_object::room_object_def::RoomObjectRef, geometry::{shape::{Shape, Polygon, Point, BoundingBox}, shapes_overlap::shapes_overlap}};

pub type RofizObjId = usize;

// external_ref_count is 1 greater than the number of external references. Rofiz itself will never create additional
// Rcs to external_ref_count. When external_ref_count reaches 1 (i.e. no external references are left), the rofiz object 
// is deleted.
pub struct RofizObjBasicWall {
    pub id: RofizObjId,
    pub external_ref_count: Arc<()>,

    // (x, y) is the top left corner of the wall. The wall is a unit square.
    pub x: u32,
    pub y: u32,

    pub room_object_ref: RoomObjectRef,

    // only used in move_objects_and_find_collisions()
    pub bounding_box: BoundingBox,
    pub shape: Shape,
}

pub struct RofizObjMovable {
    pub id: RofizObjId,
    pub external_ref_count: Arc<()>,

    pub current: Hitbox,
    pub movement: RofizObjectMovement,
    pub move_with_fallbacks_idx: usize,
    pub room_object_ref: RoomObjectRef,
    pub is_spectral: bool,

    pub cached_mem_hitbox: Hitbox,

    // only used in move_objects_and_find_collisions()
    pub bounding_box: BoundingBox,
    pub initial_hitbox: Shape,
    pub temp_hitbox: Shape,
    pub move_successful: bool,
    pub fallback_idx: usize,
    pub shape_scratchpad: Shape,
}

impl RofizObjMovable {
    pub fn start_moafc(&mut self) {
        let current = &mut self.current;
        current.transformation.replace_shape_with_transformed(&mut self.initial_hitbox, &current.shape);
        match self.movement {
            RofizObjectMovement::NoMove() => self.current.transformation.replace_shape_with_transformed(&mut self.temp_hitbox, &self.current.shape),
            RofizObjectMovement::Move(ref t) => (self.current.transformation.add(t)).replace_shape_with_transformed(&mut self.temp_hitbox, &self.current.shape),
            RofizObjectMovement::MoveWithFallbacks(ref v) => (self.current.transformation.add(&v[0])).replace_shape_with_transformed(&mut self.temp_hitbox, &self.current.shape),
            RofizObjectMovement::NewHitbox(ref h) => h.transformation.replace_shape_with_transformed(&mut self.temp_hitbox, &h.shape), 
            RofizObjectMovement::_Delete() => panic!("there should be no rofiz objects with Delete movement. Loc 1a."),
        };

        self.bounding_box = match self.movement {
            RofizObjectMovement::NoMove() => BoundingBox::of_shape(&self.temp_hitbox),
            RofizObjectMovement::Move(_) => {
                let mut b = BoundingBox::of_shape(&self.initial_hitbox);
                b.combine(&BoundingBox::of_shape(&self.temp_hitbox));
                b
            }
            RofizObjectMovement::MoveWithFallbacks(ref v) => {
                let mut all_bb = BoundingBox::of_shape(&self.initial_hitbox);
                for t in v {
                    self.current.transformation.add(t).replace_shape_with_transformed(&mut self.shape_scratchpad, &self.current.shape);
                    let bb = BoundingBox::of_shape(&self.shape_scratchpad);
                    all_bb.combine(&bb);
                }
                all_bb
            }
            RofizObjectMovement::NewHitbox(_) => {
                let mut b = BoundingBox::of_shape(&self.initial_hitbox);
                b.combine(&BoundingBox::of_shape(&self.temp_hitbox));
                b
            }
            RofizObjectMovement::_Delete() => panic!("there should be no rofiz objects with Delete movement. Loc 1b."),
        };
        self.fallback_idx = 0;
        self.move_successful = true;
    }

    pub fn overlaps_ro_wall(&self, other: &RofizObjBasicWall) -> bool {
        if !BoundingBox::overlap(&self.bounding_box, &other.bounding_box) {
            return false;
        }
        shapes_overlap(&self.temp_hitbox, &other.shape)
    }

    pub fn initial_overlaps_ro_movable(&self, other: &RofizObjMovable) -> bool {
        if !BoundingBox::overlap(&self.bounding_box, &other.bounding_box) {
            return false;
        }
        shapes_overlap(&self.initial_hitbox, &other.temp_hitbox)
    }
    
    pub fn overlaps_ro_movable(&self, other: &RofizObjMovable) -> bool {
        if !BoundingBox::overlap(&self.bounding_box, &other.bounding_box) {
            return false;
        }
        shapes_overlap(&self.temp_hitbox, &other.temp_hitbox)
    }

    pub fn officially_move(&mut self) {
        if !self.move_successful {
            return;
        }
        match self.movement {
            RofizObjectMovement::NoMove() => {},
            RofizObjectMovement::Move(ref t) => self.current.transformation = self.current.transformation.add(t),
            RofizObjectMovement::MoveWithFallbacks(ref v) => {
                if self.fallback_idx < v.len() {
                    self.current.transformation = self.current.transformation.add(&v[self.fallback_idx]);
                }
            }
            RofizObjectMovement::NewHitbox(ref mut new) => {
                std::mem::swap(new, &mut self.current);
                std::mem::swap(new, &mut self.cached_mem_hitbox);
            }
            RofizObjectMovement::_Delete() => panic!("there should be no rofiz objects with Delete movement. Loc 2."),
        }
    }
}

pub enum RofizObjectMovement {
    NoMove(),
    Move(Transformation),
    MoveWithFallbacks(Vec<Transformation>),
    NewHitbox(Hitbox),
    _Delete(),
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Transformation {
    pub dx: f64,
    pub dy: f64,
    pub dtheta: f64,
}

impl Transformation {
    pub fn new(dx: f64, dy: f64, dtheta: f64) -> Self {
        Self {dx, dy, dtheta}
    }

    pub fn replace_shape_with_transformed(&self, dst: &mut Shape, src: &Shape) {
        match src {
            Shape::Circle(src_c) => match dst {
                Shape::Circle(dst_c) => {
                    dst_c.r = src_c.r;
                    dst_c.center.x = src_c.center.x + self.dx as f32;
                    dst_c.center.y = src_c.center.y + self.dy as f32;
                }
                _ => {
                    *dst = Shape::of_circle(Point::new(src_c.center.x + self.dx as f32, src_c.center.y + self.dy as f32), src_c.r);
                }
            },
            Shape::Polygon(src_p) => match dst {
                Shape::Polygon(dst_p) => {
                    dst_p.replace_with_rotated_and_translated(src_p, self.dx as f32, self.dy as f32, self.dtheta as f32);
                },
                _ => {
                    let mut dst_p = Polygon::new(vec![Point::default(); src_p.vertexes.len()].into());
                    dst_p.replace_with_rotated_and_translated(src_p, self.dx as f32, self.dy as f32, self.dtheta as f32);
                    *dst = Shape::Polygon(dst_p);
                }
            }
        }
    }

    pub fn get_transformed_polygon(&self, polygon: &Polygon) -> Polygon {
        polygon.rotated_and_translated(self.dx as f32, self.dy as f32, self.dtheta as f32)
    }

    pub fn get_transformed_point(&self, p: Point) -> Point {
        let cos_theta = f32::cos(self.dtheta as f32);
        let sin_theta = f32::sin(self.dtheta as f32);
        let rot_x = cos_theta * p.x - sin_theta * p.y;
        let rot_y = sin_theta * p.x + cos_theta * p.y;
        Point::new(self.dx as f32 + rot_x, self.dy as f32 + rot_y)
    }

    pub fn add(&self, rhs: &Transformation) -> Transformation {
        Transformation { 
            dx: self.dx + rhs.dx, 
            dy: self.dy + rhs.dy, 
            dtheta: self.dtheta + rhs.dtheta, 
        }
    }

    pub fn sub(&self, rhs: &Transformation) -> Transformation {
        Transformation { 
            dx: self.dx - rhs.dx, 
            dy: self.dy - rhs.dy, 
            dtheta: self.dtheta - rhs.dtheta, 
        }
    }
}

#[derive(Debug, Clone, Default)]
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