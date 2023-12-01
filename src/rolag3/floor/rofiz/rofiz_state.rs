use std::{rc::{Rc, Weak}, cell::RefCell};

use crate::rolag3::floor::room_object::room_object_def::RoomObjectId;

use super::{rofiz_object::{RofizObjBasicWall, RofizObjMovable, Hitbox, RofizObjectMovement, Transformation}, shape::Shape, shapes_overlap::shapes_overlap};


#[derive(Debug, Clone)]
enum RofizObjectRefVal {
    BasicWall(Weak<RefCell<RofizObjBasicWall>>),
    BasicProjectile(Weak<RefCell<RofizObjMovable>>),
    _SpectralUnit(Weak<RefCell<RofizObjMovable>>),
    NonspectralUnit(Weak<RefCell<RofizObjMovable>>),
}

#[derive(Debug, Clone)]
pub struct RofizObjectRef {
    _ref_count: Rc<()>,
    val: RofizObjectRefVal // purely used to hide the actual ref from external callers
}

impl RofizObjectRef {
    fn new(val : RofizObjectRefVal) -> Self {
        let ref_count = match val {
            RofizObjectRefVal::BasicWall(ref x) => x.upgrade().unwrap().borrow().external_ref_count.clone(),
            RofizObjectRefVal::BasicProjectile(ref x) => x.upgrade().unwrap().borrow().external_ref_count.clone(),
            RofizObjectRefVal::_SpectralUnit(ref x) => x.upgrade().unwrap().borrow().external_ref_count.clone(),
            RofizObjectRefVal::NonspectralUnit(ref x) => x.upgrade().unwrap().borrow().external_ref_count.clone(),
        };
        Self {
            _ref_count: ref_count,
            val
        }
    }
}

pub struct RofizState {
    // only used before floor start
    floor_started: bool,

    basic_walls: Vec<Rc<RefCell<RofizObjBasicWall>>>,
    basic_projectiles: Vec<Rc<RefCell<RofizObjMovable>>>,
    spectral_units: Vec<Rc<RefCell<RofizObjMovable>>>,
    nonspectral_units: Vec<Rc<RefCell<RofizObjMovable>>>,

    has_wall_at_coordinate: Vec<Vec<bool>>,

    obj_creation_counter: usize,
}

impl RofizState {
    const MAX_XY_WARN: u32 = 1000; 

    pub fn new() -> Self {
        Self {
            floor_started: false,
            basic_walls: Vec::new(),
            basic_projectiles: Vec::new(),
            spectral_units: Vec::new(),
            nonspectral_units: Vec::new(),
            has_wall_at_coordinate: Vec::new(),
            obj_creation_counter: 0,
        }
    }

    pub fn finalize_start_floor(&mut self) {
        if self.floor_started {
            panic!("attempting to finalize Rofiz on floor twice");
        }
        self.floor_started = true;

        let mut max_x = 0;
        let mut max_y = 0;
        for bw in self.basic_walls.iter().map(|x| x.borrow()) {
            max_x = u32::max(bw.x, max_x);
            max_y = u32::max(bw.y, max_y);
        }

        if max_x > Self::MAX_XY_WARN || max_y > Self::MAX_XY_WARN {
            println!("basic wall max_x({}) and max_y({}) are large. This has negative performance implications", max_x, max_y);
        }

        let mut has_wall_at_coordinate = vec![vec![false; (max_x+1) as usize]; (max_y+1) as usize];
        for bw in self.basic_walls.iter().map(|x| x.borrow()) {
            has_wall_at_coordinate[bw.x as usize][bw.y as usize] = true;
        }

        self.has_wall_at_coordinate = has_wall_at_coordinate;
    }

    pub fn start_new_tick(&mut self) {
        for obj in self.basic_projectiles.iter_mut()
                .chain(self.spectral_units.iter_mut())
                .chain(self.nonspectral_units.iter_mut()) {
            obj.as_ref().borrow_mut().movement = RofizObjectMovement::NoMove();
        }
        self.basic_projectiles.retain(|x| Rc::strong_count(&x.borrow().external_ref_count) > 1);
        self.nonspectral_units.retain(|x| Rc::strong_count(&x.borrow().external_ref_count) > 1);
        self.spectral_units.retain(|x| Rc::strong_count(&x.borrow().external_ref_count) > 1);
        self.basic_walls.iter().for_each(|x| {
            let count = Rc::strong_count(&x.borrow().external_ref_count);
            if count <= 1 {
                panic!("Basic wall with id {} has Rc={}. Basic walls can't be deleted, so expected Rc>1.", x.borrow().id, count);
            }
        });
    }

    pub fn add_basic_wall(&mut self, floor_object_id: RoomObjectId, x: u32, y: u32) -> RofizObjectRef {
        let new_wall = Self::new_rofiz_obj_basic_wall(self, floor_object_id, x, y);
        let rc = Rc::new(RefCell::new(new_wall));
        self.basic_walls.push(rc.clone());
        RofizObjectRef::new(RofizObjectRefVal::BasicWall(Rc::downgrade(&rc)))
    }

    pub fn add_nonspectral_unit(&mut self, floor_object_id: RoomObjectId, hitbox: Hitbox) -> RofizObjectRef {
        let obj = self.new_rofiz_obj_movable(hitbox, floor_object_id);
        let rc = Rc::new(RefCell::new(obj));
        self.nonspectral_units.push(rc.clone());
        RofizObjectRef::new(RofizObjectRefVal::NonspectralUnit(Rc::downgrade(&rc)))
    }

    pub fn add_basic_projectile(&mut self, floor_object_id: RoomObjectId, hitbox: Hitbox) -> RofizObjectRef {
        let obj = self.new_rofiz_obj_movable(hitbox, floor_object_id);
        let rc = Rc::new(RefCell::new(obj));
        self.basic_projectiles.push(rc.clone());
        RofizObjectRef::new(RofizObjectRefVal::BasicProjectile(Rc::downgrade(&rc)))
    }

    pub fn move_object(&mut self, obj_ref: &RofizObjectRef, movement: RofizObjectMovement) {
        match obj_ref.val {
            RofizObjectRefVal::BasicWall(_) => panic!("accessing BasicWall in get_rofiz_obj_movable_mut() is not supported"),
            RofizObjectRefVal::BasicProjectile(ref p) => p.upgrade().unwrap().as_ref().borrow_mut().movement = movement,
            RofizObjectRefVal::_SpectralUnit(ref p) => p.upgrade().unwrap().as_ref().borrow_mut().movement = movement,
            RofizObjectRefVal::NonspectralUnit(ref p) => p.upgrade().unwrap().as_ref().borrow_mut().movement = movement,
        };
    }

    pub fn get_movable_object_xform(&self, obj_ref: &RofizObjectRef) -> Transformation {
        match obj_ref.val {
            RofizObjectRefVal::BasicWall(_) => panic!("accessing BasicWall in get_movable_object_xform() is not supported"),
            RofizObjectRefVal::BasicProjectile(ref p) => p.upgrade().unwrap().borrow().current.transformation,
            RofizObjectRefVal::_SpectralUnit(ref p) => p.upgrade().unwrap().borrow().current.transformation,
            RofizObjectRefVal::NonspectralUnit(ref p) => p.upgrade().unwrap().borrow().current.transformation,
        }
    }
    
    pub fn move_objects_and_find_collisions(&mut self) -> Vec<RofizCollision> {
        self.basic_projectiles.retain(|x| !matches!(x.borrow().movement, RofizObjectMovement::Delete()));
        self.nonspectral_units.retain(|x| !matches!(x.borrow().movement, RofizObjectMovement::Delete()));
        self.spectral_units.retain(|x| !matches!(x.borrow().movement, RofizObjectMovement::Delete()));
        for obj_rc in self.movable_objs_iter() {
            let mut obj = obj_rc.as_ref().borrow_mut();
            obj.temp_hitbox = match obj.movement {
                RofizObjectMovement::NoMove() => obj.current.transformation.get_transformed_shape(&obj.current.shape),
                RofizObjectMovement::Move(ref t) => (obj.current.transformation.add(t)).get_transformed_shape(&obj.current.shape),
                RofizObjectMovement::_NewHitbox(_) => todo!(),
                RofizObjectMovement::Delete() => panic!("there should be no rofiz objects with Delete movement. Loc 1."),
            };
            obj.move_successful = true;
        }

        let mut collisions = Vec::new();

        // Phase 1a: Check for collisions between nonspectral units and basic walls
        for nsu_rc in self.nonspectral_units.iter() {
            let mut nsu = nsu_rc.as_ref().borrow_mut();
            for bw_rc in self.basic_walls.iter() {
                let bw = bw_rc.as_ref().borrow_mut();
                if shapes_overlap(&nsu.temp_hitbox, &bw.shape) {
                    collisions.push(RofizCollision::new(nsu.floor_object_id, bw.floor_object_id));
                    if nsu.move_successful {
                        nsu.move_successful = false;
                        nsu.temp_hitbox = nsu.current.transformation.get_transformed_shape(&nsu.current.shape);
                    }
                }
            }
        }

        // Phase 1b: Check for collisions between nonspectral units
        for i in 0..self.nonspectral_units.len() {
            let mut nsu_i = self.nonspectral_units[i].as_ref().borrow_mut();
            let mut j = i + 1;
            while j < self.nonspectral_units.len() {
                let mut nsu_j = self.nonspectral_units[j].as_ref().borrow_mut();
                if shapes_overlap(&nsu_i.temp_hitbox, &nsu_j.temp_hitbox) {
                    collisions.push(RofizCollision::new(nsu_i.floor_object_id, nsu_j.floor_object_id));
                    if nsu_j.move_successful {
                        nsu_j.move_successful = false;
                        nsu_j.temp_hitbox = nsu_j.current.transformation.get_transformed_shape(&nsu_j.current.shape);
                    }
                    if nsu_i.move_successful {
                        nsu_i.move_successful = false;
                        nsu_i.temp_hitbox = nsu_i.current.transformation.get_transformed_shape(&nsu_i.current.shape);
                        // if nsu_i was previously considered as moving successfully, then it moves back to its
                        // original place. Additionally, j is reset, because collisions between nsu_i's original hitbox
                        // and other nsus need to be rechecked.
                        j = i + 1;
                        continue;
                    }
                }
                j += 1;
            }
        }

        // Phase 2: Spectral units (TODO)

        // Phase 3: Projectiles
        for bp_rc in self.basic_projectiles.iter() {
            let bp = bp_rc.as_ref().borrow_mut();
            for bw_rc in self.basic_walls.iter() {
                let bw = bw_rc.as_ref().borrow_mut();
                if shapes_overlap(&bp.temp_hitbox, &bw.shape) {
                    collisions.push(RofizCollision::new(bp.floor_object_id, bw.floor_object_id));
                }
            }

            for nsu_rc in self.nonspectral_units.iter() {
                let nsu = nsu_rc.as_ref().borrow_mut();
                if shapes_overlap(&bp.temp_hitbox, &nsu.temp_hitbox) {
                    collisions.push(RofizCollision::new(bp.floor_object_id, nsu.floor_object_id));
                }
            }
        }

        // wrap up by officially moving objects that have move_successful=true
        for mo_rc in self.movable_objs_iter() {
            let mut mo = mo_rc.as_ref().borrow_mut();
            if mo.move_successful {
                match mo.movement {
                    RofizObjectMovement::NoMove() => {},
                    RofizObjectMovement::Move(ref t) => mo.current.transformation = mo.current.transformation.add(t),
                    RofizObjectMovement::_NewHitbox(_) => todo!(),
                    RofizObjectMovement::Delete() => panic!("there should be no rofiz objects with Delete movement. Loc 2."),
                }
            }
        }

        collisions
    }

    fn movable_objs_iter(&mut self) -> impl Iterator<Item = &Rc<RefCell<RofizObjMovable>>> {
        self.basic_projectiles.iter()
                .chain(self.spectral_units.iter())
                .chain(self.nonspectral_units.iter())
    }

    fn new_rofiz_obj_basic_wall(&mut self, floor_object_id: RoomObjectId, x: u32, y: u32) -> RofizObjBasicWall {
        self.obj_creation_counter += 1;
        RofizObjBasicWall { 
            id: self.obj_creation_counter, 
            external_ref_count: Rc::new(()),
            floor_object_id,
            x, 
            y, 
            shape: Shape::of_square(x as f32, y as f32, 1.0) 
        }
    }

    fn new_rofiz_obj_movable(&mut self, hitbox: Hitbox, floor_object_id: RoomObjectId) -> RofizObjMovable {
        self.obj_creation_counter += 1;
        RofizObjMovable { 
            id: self.obj_creation_counter, 
            external_ref_count: Rc::new(()),
            current: hitbox, 
            movement: RofizObjectMovement::NoMove(), 
            temp_hitbox: Shape::dummy(),
            move_successful: false, // dummy
            floor_object_id,
        }
    }
}

pub struct RofizCollision {
    pub room_obj_id1: RoomObjectId,
    pub room_obj_id2: RoomObjectId,
}

impl RofizCollision {
    fn new(floor_obj_id1: RoomObjectId, floor_obj_id2: RoomObjectId) -> Self {
        Self {
            room_obj_id1: floor_obj_id1,
            room_obj_id2: floor_obj_id2,
        }
    }
}
