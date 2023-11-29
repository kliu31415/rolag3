use std::collections::HashMap;

use crate::rolag3::floor::floor_object::floor_object::FloorObjectId;

use super::{rofiz_object::{RofizObjectRef, RofizObjBasicWall, RofizObjMovable, RofizObjTypeIdx, RofizObjId, Hitbox, RofizObjectMovement}, shape::Shape, shapes_overlap::shapes_overlap};

pub struct RofizState {
    // only used before floor start
    floor_started: bool,

    basic_projectiles: HashMap<RofizObjId, RofizObjMovable>,
    spectral_units: HashMap<RofizObjId, RofizObjMovable>,
    nonspectral_units: HashMap<RofizObjId, RofizObjMovable>,

    basic_walls: HashMap<RofizObjId, RofizObjBasicWall>,
    has_wall_at_coordinate: Vec<Vec<bool>>,

    obj_creation_counter: usize,
}

impl RofizState {
    pub fn new() -> Self {
        Self {
            floor_started: false,
            basic_walls: HashMap::new(),
            basic_projectiles: HashMap::new(),
            spectral_units: HashMap::new(),
            nonspectral_units: HashMap::new(),
            has_wall_at_coordinate: Vec::new(),
            obj_creation_counter: 0,
        }
    }

    pub fn add_basic_wall(&mut self, floor_object_id: FloorObjectId, x: u32, y: u32) -> RofizObjectRef {
        let new_wall = Self::new_rofiz_obj_basic_wall(self, floor_object_id, x, y);
        self.basic_walls.insert(new_wall.0.id, new_wall.1);
        new_wall.0
    }

    pub fn finalize_start_floor(&mut self) {
        if self.floor_started {
            panic!("attempting to finalize Rofiz on floor twice");
        }
        self.floor_started = true;

        let mut max_x = 0;
        let mut max_y = 0;
        for bw in self.basic_walls.values() {
            max_x = u32::max(bw.x, max_x);
            max_y = u32::max(bw.y, max_y);
        }

        let mut has_wall_at_coordinate = vec![vec![false; (max_x+1) as usize]; (max_y+1) as usize];
        for bw in self.basic_walls.values() {
            has_wall_at_coordinate[bw.x as usize][bw.y as usize] = true;
        }

        self.has_wall_at_coordinate = has_wall_at_coordinate;
    }

    pub fn start_new_tick(&mut self) {
        for (_, obj) in self.basic_projectiles.iter_mut()
                .chain(self.spectral_units.iter_mut())
                .chain(self.nonspectral_units.iter_mut()) {
            obj.movement = RofizObjectMovement::NoMove();
        }
    }

    pub fn add_nonspectral_unit(&mut self, floor_object_id: FloorObjectId, hitbox: Hitbox) -> RofizObjectRef {
        let (obj_ref, obj) = self.new_rofiz_obj_movable(RofizObjTypeIdx::NonspectralUnit, hitbox, floor_object_id);
        self.nonspectral_units.insert(obj_ref.id, obj);
        obj_ref
    }

    pub fn move_object(&mut self, obj_ref: RofizObjectRef, movement: RofizObjectMovement) {
        self.get_rofiz_obj_movable_mut(obj_ref).movement = movement;
    }

    pub fn get_movable_object(&self, obj_ref: RofizObjectRef) -> &RofizObjMovable {
        self.get_rofiz_obj_movable(obj_ref)
    }

    pub fn move_objects_and_find_collisions(&mut self) -> Vec<RofizCollision> {
        for (_, obj) in self.basic_projectiles.iter_mut()
                .chain(self.spectral_units.iter_mut())
                .chain(self.nonspectral_units.iter_mut()) {
            obj.temp_hitbox = match obj.movement {
                RofizObjectMovement::NoMove() => obj.current.transformation.get_transformed_shape(&obj.current.shape),
                RofizObjectMovement::Move(ref t) => (obj.current.transformation.add(t)).get_transformed_shape(&obj.current.shape),
                RofizObjectMovement::NewHitbox(_) => todo!(),
            };
            obj.move_successful = true;
        }

        let mut collisions = Vec::new();

        for (_nsu_id, nsu) in self.nonspectral_units.iter_mut() {
            for (_bw_id, bw) in self.basic_walls.iter_mut() {
                if shapes_overlap(&nsu.temp_hitbox, &bw.shape) {
                    if nsu.move_successful {
                        nsu.move_successful = false;
                        nsu.temp_hitbox = nsu.current.transformation.get_transformed_shape(&nsu.current.shape);
                    }
                    collisions.push(RofizCollision::new(nsu.floor_object_id, bw.floor_object_id))
                }
            }
        }

        for (_nsu_id, nsu) in self.nonspectral_units.iter_mut() {
            if nsu.move_successful {
                match nsu.movement {
                    RofizObjectMovement::NoMove() => {},
                    RofizObjectMovement::Move(ref t) => nsu.current.transformation = nsu.current.transformation.add(t),
                    RofizObjectMovement::NewHitbox(_) => todo!(),
                }
            }
        }

        collisions
    }

    fn new_rofiz_obj_basic_wall(&mut self, floor_object_id: FloorObjectId, x: u32, y: u32) -> (RofizObjectRef, RofizObjBasicWall) {
        self.obj_creation_counter += 1;
        (RofizObjectRef{
            id: self.obj_creation_counter, 
            type_idx: RofizObjTypeIdx::BasicWall
        }, RofizObjBasicWall { 
            floor_object_id,
            x, 
            y, 
            shape: Shape::of_square(x as f32, y as f32, 1.0) 
        })
    }

    fn new_rofiz_obj_movable(&mut self, type_idx: RofizObjTypeIdx, hitbox: Hitbox, floor_object_id: FloorObjectId) -> (RofizObjectRef, RofizObjMovable) {
        self.obj_creation_counter += 1;
        (RofizObjectRef{
            id: self.obj_creation_counter, 
            type_idx: type_idx,
        }, RofizObjMovable { 
            current: hitbox, 
            movement: RofizObjectMovement::NoMove(), 
            temp_hitbox: Shape::dummy(),
            move_successful: false, // dummy
            floor_object_id,
        })
    }

    fn get_rofiz_obj_movable_mut(&mut self, obj_ref: RofizObjectRef) -> &mut RofizObjMovable {
        let error_str = format!("unable to find rofiz object with id={:?}", obj_ref);
        match obj_ref.type_idx {
            RofizObjTypeIdx::BasicWall => panic!("accessing BasicWall in get_rofiz_obj_movable_mut() is not supported"),
            RofizObjTypeIdx::Projectile => self.basic_projectiles.get_mut(&obj_ref.id).expect(&error_str),
            RofizObjTypeIdx::SpectralUnit => self.spectral_units.get_mut(&obj_ref.id).expect(&error_str),
            RofizObjTypeIdx::NonspectralUnit => self.nonspectral_units.get_mut(&obj_ref.id).expect(&error_str),
        }
    }

    fn get_rofiz_obj_movable(&self, obj_ref: RofizObjectRef) -> &RofizObjMovable {
        let error_str = format!("unable to find rofiz object with id={:?}", obj_ref);
        match obj_ref.type_idx {
            RofizObjTypeIdx::BasicWall => panic!("accessing BasicWall in get_rofiz_obj_movable() is not supported"),
            RofizObjTypeIdx::Projectile => self.basic_projectiles.get(&obj_ref.id).expect(&error_str),
            RofizObjTypeIdx::SpectralUnit => self.spectral_units.get(&obj_ref.id).expect(&error_str),
            RofizObjTypeIdx::NonspectralUnit => self.nonspectral_units.get(&obj_ref.id).expect(&error_str),
        }
    }
}

pub struct RofizCollision {
    pub floor_obj_id1: FloorObjectId,
    pub floor_obj_id2: FloorObjectId,
}

impl RofizCollision {
    fn new(floor_obj_id1: FloorObjectId, floor_obj_id2: FloorObjectId) -> Self {
        Self {
            floor_obj_id1,
            floor_obj_id2,
        }
    }
}
